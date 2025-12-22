package main

import (
	"log"
	"log/slog"
)

func main() {
	app, err := GetApplication()
	if err != nil {
		slog.Error("Failed to start application. Exiting.", "error", err)
		return
	}

	log.Fatal(app.Serve())
}
