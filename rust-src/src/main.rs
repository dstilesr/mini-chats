mod application;
mod messages;
mod settings;

use application::Dispatcher;
use axum::{
    Router,
    extract::Query,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
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
    let (chan_send, chan_recv) = mpsc::channel(32);
    if let Err(s) = dispatch
        .lock()
        .await
        .add_client(&client_name, chan_send.clone())
    {
        log::error!("Failed to add client: {}", s);
        return;
    }

    log::info!("Client {} connected", client_name);

    // Initial acknowledge message
    let ack_msg = ServerResponse {
        status: "ok".to_string(),
        info: Some(ServerResponseInfo {
            client_name: Some(client_name.clone()),
            ..Default::default()
        }),
    };
    let msg = match Message::try_from(&ack_msg) {
        Err(e) => {
            log::error!("Failed to serialize message {}", e);
            return;
        }
        Ok(m) => m,
    };
    if let Err(e) = sock.send(msg).await {
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
                let response = dsp.process_message(user_msg, &client_name).await;
                let out_msg = match Message::try_from(&response) {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("Failed to convert response to message: {}", e);
                        continue;
                    }
                };
                drop(dsp);
                if let Err(e) = channel.send(out_msg).await {
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
