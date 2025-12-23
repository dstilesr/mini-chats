use super::messages::*;
use axum::extract::ws::Message;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::Sender;

/// Dispatcher to handle running the chat application.
#[derive(Debug)]
pub struct Dispatcher {
    client_to_channels: HashMap<String, HashSet<String>>,
    channel_to_clients: HashMap<String, HashSet<String>>,
    client_sinks: HashMap<String, Sender<Message>>,
}

impl Dispatcher {
    /// Instantiate new dispatcher
    pub fn new() -> Self {
        Self {
            channel_to_clients: HashMap::new(),
            client_to_channels: HashMap::new(),
            client_sinks: HashMap::new(),
        }
    }

    /// Add a new client to the service
    pub fn add_client(&mut self, client: &str, channel: Sender<Message>) -> Result<(), String> {
        if self.client_to_channels.contains_key(client) {
            return Err(format!("Client {} already exists!", client));
        }
        self.client_to_channels
            .insert(client.to_string(), HashSet::new());
        self.client_sinks.insert(client.to_string(), channel);
        Ok(())
    }

    /// Remove a client from the application
    pub fn remove_client(&mut self, client: &str) {
        self.client_to_channels.remove(client);
        self.client_sinks.remove(client);
        for (_, v) in self.channel_to_clients.iter_mut() {
            v.remove(client);
        }
    }

    /// Subscribe a client to a channel
    pub fn subscribe(&mut self, client: &str, channel: String) -> ServerResponse {
        log::info!("Client {} subscribing to channel {}", client, channel);
        let total_subscribers = match self.client_to_channels.get_mut(client) {
            None => {
                return ServerResponse::from(format!(
                    "Did not find channels for client {}",
                    client
                ));
            }
            Some(c) => {
                c.insert(channel.clone());
                let total_prev = self.channel_to_clients.get(&channel).map_or(0, |s| s.len());
                self.channel_to_clients
                    .entry(channel)
                    .or_default()
                    .insert(client.to_string());
                total_prev + 1
            }
        };
        ServerResponse {
            status: "ok".to_string(),
            info: Some(ServerResponseInfo {
                total_subscribers: Some(total_subscribers),
                ..Default::default()
            }),
        }
    }

    /// Unsubscribe a client from a channel
    pub fn unsubscribe(&mut self, client: &str, channel: String) -> ServerResponse {
        match self.client_to_channels.get_mut(client) {
            None => {
                return ServerResponse::from(format!("Did not find channel set for {}", client));
            }
            Some(e) => {
                e.remove(&channel);
            }
        }
        self.channel_to_clients
            .get_mut(&channel)
            .map(|s| s.remove(client));

        if self
            .channel_to_clients
            .get(&channel)
            .is_none_or(|s| s.is_empty())
        {
            // Cleanup empty channel
            log::debug!("Removing empty channel {}", channel);
            self.channel_to_clients.remove(&channel);
        }
        log::info!(
            "Unsubscribed client '{}' from channel '{}'",
            client,
            channel
        );
        ServerResponse::default()
    }

    pub async fn publish_message(
        &self,
        sender: &str,
        channel: String,
        content: String,
    ) -> ServerResponse {
        // TODO!
        let to_publish = PublishedMessage::new(sender, &content, &channel);
        let msg = match Message::try_from(&to_publish) {
            Err(e) => return ServerResponse::from(e),
            Ok(m) => m,
        };
        match self.channel_to_clients.get(&channel) {
            None => return ServerResponse::from(format!("Channel '{}' not found!", channel)),
            Some(clients) => {
                for c in clients {
                    if let Some(snd) = self.client_sinks.get(c) {
                        if let Err(e) = snd.send(msg.clone()).await {
                            log::error!("Failed to send message to client '{}': {}", c, e);
                        };
                    } else {
                        log::warn!("Sink for Client '{}' not found!", c);
                    }
                }
            }
        }
        log::debug!("TEMP - Publish: {}--{}--{:?}", sender, channel, to_publish);
        ServerResponse::default()
    }

    /// Process a message from the client
    pub async fn process_message(&mut self, msg: ClientMessage, client: &str) -> ServerResponse {
        match msg {
            ClientMessage::Subscribe { channel_name } => self.subscribe(client, channel_name),
            ClientMessage::Unsubscribe { channel_name } => self.unsubscribe(client, channel_name),
            ClientMessage::Publish {
                channel_name,
                content,
            } => self.publish_message(client, channel_name, content).await,
        }
    }
}
