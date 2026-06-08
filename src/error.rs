use std::fmt;

/// Unified error type for the Microgen SDK.
#[derive(Debug)]
pub enum MicrogenError {
    /// The API returned a non-success HTTP status.
    Api {
        status: u16,
        message: String,
        body: String,
    },
    /// An error from the reqwest HTTP client.
    Request(reqwest::Error),
    /// An error during JSON serialization/deserialization.
    Serde(serde_json::Error),
    /// WebSocket connection or protocol error.
    WebSocket(String),
    /// Invalid configuration or arguments.
    InvalidArgument(String),
}

impl fmt::Display for MicrogenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api { status, message, body } => {
                write!(f, "API error ({}): {} — {}", status, message, body)
            }
            Self::Request(e) => write!(f, "HTTP request error: {}", e),
            Self::Serde(e) => write!(f, "Serialization error: {}", e),
            Self::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
        }
    }
}

impl std::error::Error for MicrogenError {}

impl From<reqwest::Error> for MicrogenError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl From<serde_json::Error> for MicrogenError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

pub type Result<T> = std::result::Result<T, MicrogenError>;
