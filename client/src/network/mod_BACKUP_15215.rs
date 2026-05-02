pub mod connection;
pub mod protocol;

pub use connection::Connection;
pub use protocol::{Message, MessageType};

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Network error types
#[derive(Debug, Clone)]
pub enum NetworkError {
    ConnectionFailed(String),
    SendError(String),
    ReceiveError(String),
    SerializationError(String),
    DeserializationError(String),
    Timeout,
    DisconnectedError,
    InvalidMessage(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            NetworkError::SendError(msg) => write!(f, "Send error: {}", msg),
            NetworkError::ReceiveError(msg) => write!(f, "Receive error: {}", msg),
            NetworkError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            NetworkError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            NetworkError::Timeout => write!(f, "Network timeout"),
            NetworkError::DisconnectedError => write!(f, "Client disconnected"),
            NetworkError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
        }
    }
}

impl std::error::Error for NetworkError {}

pub use connection::Connection;
