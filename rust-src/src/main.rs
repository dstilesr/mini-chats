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
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use messages::*;
use settings::AppSettings;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
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

    log::info!(
        "Starting mini chat server on port {}. Version: {}",
        settings.port,
        settings.version
    );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", settings.port))
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
    let (mut chan_send, mut chan_recv) = mpsc::channel(32);

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
    let (sender, receiver) = sock.split();

    // Forwarder to send messages to client
    tokio::spawn(async move { forward_to_socket(chan_recv, sender).await });

    // Listen to messages from client
    tokio::spawn(async move { socket_listener(dispatch, client_name, chan_send, receiver).await });
}

/// Listener task to receive messages from an MPSC channel and forward them to the socket
async fn forward_to_socket(
    mut listener: mpsc::Receiver<Message>,
    mut sock: SplitSink<WebSocket, Message>,
) {
    while let Some(msg) = listener.recv().await {
        if let Err(e) = sock.send(msg).await {
            log::error!("Unable to send message to socket: {}", e);
            break;
        }
    }
    log::debug!("Forwarder task exited");
}

async fn socket_listener(
    dispatch: Arc<Mutex<Dispatcher>>,
    client_name: String,
    channel: mpsc::Sender<Message>,
    mut sock: SplitStream<WebSocket>,
) {
    while let Some(Ok(msg)) = sock.next().await {
        match msg {
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
                if let Err(e) = channel.send(Message::Text(rsp_json)).await {
                    log::error!("Could not send response to client '{}': {}", client_name, e);
                    break;
                };
            }
            // Other message types
            _ => continue,
        }
    }

    // Listener loop exited - remove client
    let mut dsp = dispatch.lock().await;
    dsp.remove_client(&client_name);
    log::info!("Client {} disconnected", client_name);
}
