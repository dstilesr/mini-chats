package main

import "net/http"

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
	AppSettings Settings
	Server      http.Server
}
