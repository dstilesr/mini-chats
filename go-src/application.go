package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"strconv"
	"strings"
	"time"
)

const SendTimeout = 20 * time.Second

// GetSettings instantiates the application settings from environment variables
func GetSettings() (Settings, error) {
	port_str := os.Getenv("PORT")
	var port int
	if port_str == "" {
		port = 3501
	} else {
		port_num, err := strconv.Atoi(port_str)
		if err != nil {
			return Settings{}, errors.New("Unable to parse port number")
		}
		port = port_num
	}

	static_path := os.Getenv("APP_STATIC_PATH")
	env := os.Getenv("APP_ENVIRONMENT")
	if env == "" {
		env = "dev"
	}

	log_level := strings.ToLower(os.Getenv("APP_LOG_LEVEL"))
	if log_level == "" {
		log_level = "info"
	}

	return Settings{
		Port:        port,
		StaticPath:  static_path,
		Version:     Version,
		Environment: env,
		LogLevel:    log_level,
	}, nil
}

// NewApplication Creates a new application and returns a pointer to it
func NewApplication() (*Application, error) {
	settings, err := GetSettings()
	if err != nil {
		return nil, err
	}

	app := Application{
		AppSettings:     settings,
		ClientChannels:  make(map[string]chan *PublishedMessage),
		ClientToChannel: make(map[string]map[string]bool),
		ChannelToClient: make(map[string]map[string]bool),
	}
	err = app.SetUpRoutes()
	if err != nil {
		return nil, err
	}
	return &app, nil
}

func NewMessage(sender, channel, content string) PublishedMessage {
	now := time.Now().Format(time.RFC3339)
	return PublishedMessage{
		Sender:      sender,
		SentAt:      now,
		ChannelName: channel,
		Content:     content,
	}
}

// Serve starts the server and listens for incoming requests
func (a *Application) Serve() error {
	// Set up logs
	var level slog.Level
	switch a.AppSettings.LogLevel {
	case "debug":
		level = slog.LevelDebug
	case "info":
		level = slog.LevelInfo
	case "warn":
		level = slog.LevelWarn
	case "warning":
		level = slog.LevelWarn
	case "error":
		level = slog.LevelError
	default:
		return fmt.Errorf("Invalid log level given: %s", a.AppSettings.LogLevel)
	}
	slog.SetDefault(slog.New(slog.NewTextHandler(
		os.Stderr,
		&slog.HandlerOptions{Level: level},
	)))

	slog.Info("Starting server", "port", a.AppSettings.Port)
	return a.Server.ListenAndServe()
}

// ValidateParams verifies that the correct parameters for the given action are present.
func (m *ClientMessage) ValidateParams() error {
	switch m.Action {
	case "subscribe":
		if m.Params.ChannelName == nil || *m.Params.ChannelName == "" {
			return errors.New("Subscribe requires a channel name!")
		}
	case "unsubscribe":
		if m.Params.ChannelName == nil || *m.Params.ChannelName == "" {
			return errors.New("Unsubscribe requires a channel name!")
		}
	case "publish":
		if m.Params.ChannelName == nil || *m.Params.ChannelName == "" {
			return errors.New("Publish requires a channel name!")
		} else if m.Params.Content == nil || *m.Params.Content == "" {
			return errors.New("Publish requires message content!")
		}
	default:
		return fmt.Errorf("Unknown action specified: '%s'", m.Action)
	}
	return nil
}

// SubscribeClient subscribes a client to a channel. Returns the total subscribers to the
// channel and an error value.
func (a *Application) SubscribeClient(client, channel string) (int, error) {
	a.lock.Lock()
	defer a.lock.Unlock()

	_, ok := a.ClientToChannel[client]
	if !ok {
		return 0, fmt.Errorf("Found no channel set for %s", client)
	}

	_, ok = a.ChannelToClient[channel]
	if !ok {
		slog.Debug("Creating new channel", "channel", channel)
		a.ChannelToClient[channel] = make(map[string]bool)
	}

	a.ClientToChannel[client][channel] = true
	a.ChannelToClient[channel][client] = true
	slog.Info("Client subscribed", "client", client, "channel", channel)
	return len(a.ChannelToClient[channel]), nil
}

// UnSubscribeClient unsubscribes a client from a channel.
func (a *Application) UnSubscribeClient(client, channel string) error {
	a.lock.Lock()
	defer a.lock.Unlock()

	_, ok := a.ClientToChannel[client]
	if !ok {
		return fmt.Errorf("Found no channel set for %s", client)
	}
	_, ok = a.ChannelToClient[channel]
	if !ok {
		return fmt.Errorf("Did not find channel %s", channel)
	}

	delete(a.ChannelToClient[channel], client)
	delete(a.ClientToChannel[client], channel)
	if len(a.ChannelToClient[channel]) == 0 {
		slog.Debug("Deleting empty channel", "channel", channel)
		delete(a.ChannelToClient, channel)
	}
	slog.Info("Client unsubscribed", "client", client, "channel", channel)
	return nil
}

// AddClient adds a new client to the application.
func (a *Application) AddClient(clientName string) (chan *PublishedMessage, error) {
	a.lock.Lock()
	defer a.lock.Unlock()

	_, ok := a.ClientChannels[clientName]
	if ok {
		return nil, fmt.Errorf("Client %s already exists", clientName)
	}

	newChan := make(chan *PublishedMessage)
	a.ClientToChannel[clientName] = make(map[string]bool)
	a.ClientChannels[clientName] = newChan
	slog.Info("Client added", "client", clientName)
	return newChan, nil
}

// RemoveClient removes a client from the application.
func (a *Application) RemoveClient(clientName string) error {
	a.lock.Lock()
	defer a.lock.Unlock()

	c, ok := a.ClientChannels[clientName]
	if !ok {
		return fmt.Errorf("Client %s does not exist", clientName)
	}
	close(c)

	delete(a.ClientChannels, clientName)
	delete(a.ClientToChannel, clientName)

	for _, subscribers := range a.ChannelToClient {
		_, ok := subscribers[clientName]
		if ok {
			delete(subscribers, clientName)
		}
	}

	slog.Info("Client removed", "client", clientName)
	return nil
}

// Send message to all subscribers of the channel
func (a *Application) Publish(sender, channel, content string) error {
	a.lock.RLock()
	defer a.lock.RUnlock()

	sendTo, ok := a.ChannelToClient[channel]
	if !ok {
		return fmt.Errorf("Channel %s does not exist", channel)
	}

	msg := NewMessage(sender, channel, content)
	for subscriber, _ := range sendTo {
		if subscriber == sender {
			continue
		}
		c, ok := a.ClientChannels[subscriber]
		if !ok {
			slog.Warn(
				"Client has no associated channel",
				"client", subscriber,
			)
			continue
		}
		go SendWithTimeout(c, &msg)
	}

	return nil
}

// SendWithTimeout sends a message to a channel with a timeout
func SendWithTimeout[T any](c chan<- T, value T) {
	to := time.After(SendTimeout)
	select {
	case c <- value:
		slog.Info("Message dispatched")
	case <-to:
		slog.Error("Message dispatch timed out!")
	}
}

// MakeErrorResponse creates an error response string
func MakeErrorResponse(err string) string {
	info := map[string]string{"detail": err}
	infoJson, _ := json.Marshal(info)

	return fmt.Sprintf(`{"status": "error", "info": %s}`, string(infoJson))
}

// ProcessMessage processes a user request and returns the text that should
// be returned to the client
func (a *Application) ProcessMessage(m ClientMessage, clientName string) []byte {
	var err error = nil
	rsp := make([]byte, 0)

	switch m.Action {
	case "publish":
		err = a.Publish(clientName, *m.Params.ChannelName, *m.Params.Content)
		rsp = []byte(`{"status": "ok"}`)

	case "subscribe":
		var num int
		num, err = a.SubscribeClient(clientName, *m.Params.ChannelName)
		rspRaw := SubscribeResponse{
			Status: "ok",
			Info: subscribeInfo{
				ChannelName: *m.Params.ChannelName,
				TotalSubs:   num,
			},
		}
		rsp, _ = json.Marshal(rspRaw)

	case "unsubscribe":
		err = a.UnSubscribeClient(clientName, *m.Params.ChannelName)
		rsp = []byte(`{"status": "ok"}`)

	default:
		err = fmt.Errorf("Invalid action: %s", m.Action)
	}

	if err != nil {
		slog.Error("Error processing user message", "error", err, "client", clientName)
		rsp = []byte(MakeErrorResponse(err.Error()))
	}
	return rsp
}
