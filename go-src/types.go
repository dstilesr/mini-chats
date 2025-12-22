package main

import (
	"net/http"
	"sync"
)

// Version is the version of the application
const Version = "0.1.0"

// Settings represents the overall settings for running the application
type Settings struct {
	Port        int
	StaticPath  string
	Version     string
	Environment string
	LogLevel    string
}

// Application represents the overall application
type Application struct {
	AppSettings     Settings
	Server          http.Server
	lock            sync.RWMutex
	ClientChannels  map[string]chan *PublishedMessage
	ClientToChannel map[string]map[string]bool
	ChannelToClient map[string]map[string]bool
}

// PublishedMessage is a published message sent to a client
type PublishedMessage struct {
	Sender      string `json:"sender"`
	ChannelName string `json:"channel_name"`
	SentAt      string `json:"sent_at"`
	Content     string `json:"content"`
}

// Params contains the parameters send in a client message
type Params struct {
	ChannelName      *string `json:"channel_name"`
	Content          *string `json:"content"`
	TotalSubscribers *int    `json:"total_subscribers"`
}

// ClientMessage is a message sent by the client over the Socket
type ClientMessage struct {
	Action string `json:"action"`
	Params Params `json:"params"`
}
