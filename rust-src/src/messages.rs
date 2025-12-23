use axum::extract::ws::Message;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
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

/// Information contained in a server response to a client message
#[derive(Debug, Serialize, Default)]
pub struct ServerResponseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_subscribers: Option<usize>,
}

/// Response from the server to a client message
#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ServerResponseInfo>,
}

impl<T: std::fmt::Display> From<T> for ServerResponse {
    /// Utility to create response from error object / string
    fn from(value: T) -> Self {
        Self {
            status: "error".to_string(),
            info: Some(ServerResponseInfo {
                detail: Some(format!("Error encountered: {}", value)),
                total_subscribers: None,
                channel_name: None,
                client_name: None,
            }),
        }
    }
}

impl Default for ServerResponse {
    /// Default simple success response
    fn default() -> Self {
        Self {
            status: "ok".to_string(),
            info: None,
        }
    }
}

impl TryFrom<&ServerResponse> for Message {
    type Error = serde_json::Error;

    fn try_from(value: &ServerResponse) -> serde_json::Result<Self> {
        let json_str = serde_json::to_string(value)?;
        Ok(Self::text(json_str))
    }
}

#[derive(Debug, Serialize)]
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

impl TryFrom<&PublishedMessage> for Message {
    type Error = serde_json::Error;

    fn try_from(value: &PublishedMessage) -> serde_json::Result<Self> {
        let json_str = serde_json::to_string(value)?;
        Ok(Self::text(json_str))
    }
}

/// Query parameters for opening new connection
#[derive(Deserialize, Debug, Default)]
pub struct ConnectParams {
    #[serde(default)]
    pub client_name: Option<String>,
}

/// Generate a random alphanumeric string to use as client name
pub fn random_client_name(length: usize) -> String {
    let mut out = Vec::with_capacity(length);
    let mut rng = rand::rng();
    for _ in 0..length {
        out.push(rng.sample(Alphanumeric));
    }
    String::from_utf8(out).unwrap()
}
