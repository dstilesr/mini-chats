use super::messages::*;
use std::collections::{HashMap, HashSet};

/// Dispatcher to handle running the chat application.
#[derive(Debug)]
pub struct Dispatcher {
    client_to_channels: HashMap<String, HashSet<String>>,
    channel_to_clients: HashMap<String, HashSet<String>>,
}

impl Dispatcher {
    /// Instantiate new dispatcher
    pub fn new() -> Self {
        Self {
            channel_to_clients: HashMap::new(),
            client_to_channels: HashMap::new(),
        }
    }

    /// Add a new client to the service
    pub fn add_client(&mut self, client: &str) -> Result<(), String> {
        if self.client_to_channels.contains_key(client) {
            return Err(format!("Client {} already exists!", client));
        }
        self.client_to_channels
            .insert(client.to_string(), HashSet::new());
        Ok(())
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
                    .or_insert_with(HashSet::new)
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
            .map_or(true, |s| s.is_empty())
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

    pub fn publish_message(
        &self,
        sender: &str,
        channel: String,
        content: String,
    ) -> ServerResponse {
        // TODO!
        let to_publish = PublishedMessage::new(sender, &content, &channel);
        log::debug!("TEMP - Publish: {}--{}--{:?}", sender, channel, to_publish);
        ServerResponse::default()
    }

    /// Process a message from the client
    pub fn process_message(&mut self, msg: ClientMessage, client: &str) -> ServerResponse {
        match msg {
            ClientMessage::Subscribe { channel_name } => self.subscribe(client, channel_name),
            ClientMessage::Unsubscribe { channel_name } => self.unsubscribe(client, channel_name),
            ClientMessage::Publish {
                channel_name,
                content,
            } => self.publish_message(client, channel_name, content),
        }
    }
}
