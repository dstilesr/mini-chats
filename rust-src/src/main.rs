mod application;
mod messages;
mod settings;

use application::Dispatcher;
use axum::{
    Router,
    extract::Query,
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use log;
use messages::*;
use serde_json;
use settings::AppSettings;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Logging setup
    let settings = AppSettings::new();
    let log_level = match settings.log_level.to_uppercase().trim() {
        "DEBUG" => log::LevelFilter::Debug,
        "INFO" => log::LevelFilter::Info,
        "WARN" => log::LevelFilter::Warn,
        "WARNING" => log::LevelFilter::Warn,
        "ERROR" => log::LevelFilter::Error,
        _ => panic!("Unknown log level!"),
    };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let dispatcher = Arc::new(Mutex::new(Dispatcher::new()));

    let router = Router::new()
        .fallback_service(
            ServeDir::new(&settings.static_path).append_index_html_on_directories(true),
        )
        .route(
            "/api/connect",
            any(move |ws, qry| handle_socket(ws, qry, Arc::clone(&dispatcher))),
        );

    println!("Application Settings: {settings:?}");

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", settings.port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}

/// Handle socket upgrade
async fn handle_socket(
    ws: WebSocketUpgrade,
    qry: Query<ConnectParams>,
    dispatch: Arc<Mutex<Dispatcher>>,
) -> impl IntoResponse {
    let mut client_name = qry.0.client_name.unwrap_or_else(|| random_client_name(28));
    if client_name.trim().is_empty() {
        log::debug!("Client name empty - generating random name");
        client_name = random_client_name(28);
    }
    ws.on_upgrade(move |socket| client_connection(socket, client_name, dispatch))
}

/// Handle the connection to a client via socket
async fn client_connection(
    mut sock: WebSocket,
    client_name: String,
    dispatch: Arc<Mutex<Dispatcher>>,
) {
    if let Err(s) = dispatch.lock().await.add_client(&client_name) {
        log::error!("Failed to add client: {}", s);
        return;
    }

    log::info!("Client {} connected", client_name);

    // Initial acknowledge message
    let ack_str = serde_json::to_string(&ServerResponse {
        status: "ok".to_string(),
        info: Some(ServerResponseInfo {
            client_name: Some(client_name.clone()),
            ..Default::default()
        }),
    });
    let json_str = match ack_str {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to serialize ServerResponse: {}", e);
            return;
        }
    };
    if let Err(e) = sock.send(Message::Text(Utf8Bytes::from(json_str))).await {
        log::error!(
            "Failed to send initial message to client '{}': {}",
            client_name,
            e
        );
        return;
    };

    // Listen for client messages
    while let Some(msg) = sock.recv().await {
        match msg {
            Err(e) => {
                log::error!("Socket disconnected for client: {}", e);
                break;
            }
            Ok(msg) => match msg {
                Message::Text(t) => {
                    let user_msg: ClientMessage = match serde_json::from_str(&t) {
                        Err(e) => {
                            log::error!("Failed to parse client message: {}", e);
                            break;
                        }
                        Ok(elem) => elem,
                    };
                    let mut dsp = dispatch.lock().await;
                    let response = dsp.process_message(user_msg, &client_name);
                    let rsp_json = Utf8Bytes::from(serde_json::to_string(&response).unwrap());
                    drop(dsp);
                    if let Err(e) = sock.send(Message::Text(rsp_json)).await {
                        log::error!("Could not send response to client '{}': {}", client_name, e);
                        break;
                    };
                }
                // Other message types
                _ => continue,
            },
        }
    }
}
