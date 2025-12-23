package main

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"math/rand"
	"net/http"

	"github.com/gorilla/websocket"
)

const NameCharset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_"

var upgrader = websocket.Upgrader{}

// Set up the application server routes
func (a *Application) SetUpRoutes() error {

	mux := http.NewServeMux()
	mux.HandleFunc("/api/connect", a.HandleClientSocket)

	// Setup static routes
	fs := http.FileServer(http.Dir(a.AppSettings.StaticPath))
	mux.Handle("/", fs)

	a.Server = http.Server{
		Addr:    fmt.Sprintf("0.0.0.0:%d", a.AppSettings.Port),
		Handler: mux,
	}
	return nil
}

func (a *Application) HandleClientSocket(res http.ResponseWriter, req *http.Request) {
	clientName := req.URL.Query().Get("client_name")
	if clientName == "" {
		clientName = RandomName(24)
	}

	sock, err := upgrader.Upgrade(res, req, nil)
	if err != nil {
		slog.Error("Failed to open socket connection", "error", err)
		return
	}
	defer sock.Close()

	// Register client
	recv, err := a.AddClient(clientName)
	if err != nil {
		slog.Error("Failed to register client", "error", err)
		sock.WriteMessage(
			websocket.TextMessage,
			[]byte(MakeErrorResponse(fmt.Sprintf(
				"Failed to register client %s: %s",
				clientName,
				err,
			))),
		)
		return
	}
	defer a.RemoveClient(clientName)

	// Start listener
	go ListenAndPublish(recv, sock)

	// Listen for messages
	for {
		var parsedMsg ClientMessage
		_, msg, err := sock.ReadMessage()
		if err != nil {
			slog.Error(
				"Socket error - stopping connection",
				"error", err,
				"client", clientName,
			)
			break
		}

		// Parse JSON - validate
		err = json.Unmarshal(msg, &parsedMsg)
		if err != nil {
			slog.Error(
				"Failed to parse JSON message",
				"error", err,
				"client", clientName,
			)
			sock.WriteMessage(
				websocket.TextMessage,
				[]byte(MakeErrorResponse("Failed to parse JSON message")),
			)
		}
		err = parsedMsg.ValidateParams()
		if err != nil {
			slog.Error(
				"Invalid parameters",
				"error", err,
				"client", clientName,
			)
			sock.WriteMessage(
				websocket.TextMessage,
				[]byte(MakeErrorResponse(fmt.Sprintf("Invalid parameters: %s", err))),
			)
		}

		rsp := a.ProcessMessage(parsedMsg, clientName)
		err = sock.WriteMessage(websocket.TextMessage, rsp)
		if err != nil {
			slog.Error(
				"Failed to write response",
				"error", err,
				"client", clientName,
			)
			break
		}
	}
	slog.Info("Stopped Client Listener", "client", clientName)
}

// Generate a random client name
func RandomName(length int) string {
	name := make([]byte, 0, length)
	totalChars := len(NameCharset)
	for range length {
		idx := rand.Intn(totalChars)
		name = append(name, NameCharset[idx])
	}
	return string(name)
}

// ListenAndPublish listens for published messages and send them to the socket
func ListenAndPublish(c <-chan *PublishedMessage, socket *websocket.Conn) {
	for msg := range c {
		slog.Debug("Received message - forwarding to socket")
		content, _ := json.Marshal(msg)
		err := socket.WriteMessage(websocket.TextMessage, content)
		if err != nil {
			slog.Error("Error in web socket! Stopping publishing", "error", err)
			break
		}
	}
	slog.Debug("Stopped Client Message Listener")
}
