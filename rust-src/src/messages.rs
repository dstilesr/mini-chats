use chrono;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "info")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { channel_name: String },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { channel_name: String },

    #[serde(rename = "publish")]
    Publish {
        channel_name: String,
        content: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishedMessage {
    pub sender: String,
    pub channel_name: String,
    pub content: String,
    pub sent_at: String,
}

impl PublishedMessage {
    /// Instantiate a new published message from the content and metadata
    pub fn new(sender: &str, content: &str, channel_name: &str) -> Self {
        let sent_at = chrono::Utc::now().to_rfc3339();
        Self {
            sender: sender.to_string(),
            channel_name: channel_name.to_string(),
            content: content.to_string(),
            sent_at,
        }
    }
}
