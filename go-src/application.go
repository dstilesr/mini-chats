package main

import (
	"errors"
	"fmt"
	"log/slog"
	"os"
	"strconv"
	"strings"
)

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

// GetApplication Creates a new application and returns a pointer to it
func GetApplication() (*Application, error) {
	settings, err := GetSettings()
	if err != nil {
		return nil, err
	}

	app := Application{AppSettings: settings}
	err = app.SetUpRoutes()
	if err != nil {
		return nil, err
	}
	return &app, nil
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
