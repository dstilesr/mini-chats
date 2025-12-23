mod application;
mod messages;
mod settings;

use axum::{
    Router,
    body::Bytes,
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
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
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

    let router = Router::new()
        .fallback_service(
            ServeDir::new(&settings.static_path).append_index_html_on_directories(true),
        )
        .route("/api/connect", any(handle_socket));

    println!("Application Settings: {settings:?}");

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", settings.port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}

/// Handle socket upgrade
async fn handle_socket(ws: WebSocketUpgrade, qry: Query<ConnectParams>) -> impl IntoResponse {
    let mut client_name = qry.0.client_name.unwrap_or_else(|| random_client_name(28));
    if client_name.trim().is_empty() {
        log::debug!("Client name empty - generating random name");
        client_name = random_client_name(28);
    }
    ws.on_upgrade(move |socket| client_connection(socket, client_name))
}

/// Handle the connection to a client via socket
async fn client_connection(mut sock: WebSocket, client_name: String) {
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
                log::error!("Socket disconnected for client '{}': {}", client_name, e);
                break;
            }
            Ok(msg) => match msg {
                Message::Text(t) => {
                    let user_msg: ClientMessage = match serde_json::from_str(&t) {
                        Ok(elem) => elem,
                        Err(e) => {
                            log::error!("Failed to parse client message: {}", e);
                            break;
                        }
                    };
                    println!("Client message: {user_msg:?}")
                }
                _ => continue,
            },
        }
    }
}
