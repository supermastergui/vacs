use thiserror::Error;
use tokio_tungstenite::tungstenite;
use vacs_protocol::ws::{ErrorReason, LoginFailureReason};

#[derive(Debug, Error)]
pub enum SignalingError {
    #[error("connection error: {0}")]
    ConnectionError(#[from] Box<tungstenite::error::Error>),
    #[error("disconnected")]
    Disconnected,
    #[error("serialization error: {0}")]
    SerializationError(#[from] Box<serde_json::Error>),
    #[error("signaling protocol error: {0}")]
    ProtocolError(String),
    #[error("login failed: {0:?}")]
    LoginError(LoginFailureReason),
    #[error("server error: {0:?}")]
    ServerError(ErrorReason),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("transport error: {0}")]
    Transport(#[from] Box<anyhow::Error>),
}
