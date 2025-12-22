package main

import (
	"fmt"
	"net/http"
)

// Set up the application server routes
func (a *Application) SetUpRoutes() error {

	mux := http.NewServeMux()

	// Setup static routes
	fs := http.FileServer(http.Dir(a.AppSettings.StaticPath))
	mux.Handle("/", fs)

	a.Server = http.Server{
		Addr:    fmt.Sprintf(":%d", a.AppSettings.Port),
		Handler: mux,
	}
	return nil
}
