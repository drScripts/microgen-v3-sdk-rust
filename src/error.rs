use thiserror::Error;

/// Unified error type for the Microgen SDK.
#[derive(Debug, Error)]
pub enum MicrogenError {
    /// The API returned a non-success HTTP status.
    #[error("API error ({status}): {message} — {body}")]
    Api {
        status: u16,
        message: String,
        body: String,
    },
    /// An error from the reqwest HTTP client.
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),
    /// An error during JSON serialization/deserialization.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    /// WebSocket connection or protocol error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    /// Invalid configuration or arguments.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, MicrogenError>;
