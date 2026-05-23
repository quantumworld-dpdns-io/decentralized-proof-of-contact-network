use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Handshake failed with peer {peer}: {reason}")]
    HandshakeFailed { peer: String, reason: String },

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Failed to decode message: {0}")]
    MessageDecode(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Peer discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("State synchronization error: {0}")]
    SyncError(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),

    #[cfg(feature = "tls")]
    #[error("TLS error: {0}")]
    TlsError(String),

    #[cfg(feature = "quic")]
    #[error("QUIC error: {0}")]
    QuicError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for NetworkError {
    fn from(e: std::io::Error) -> Self {
        NetworkError::IoError(e.to_string())
    }
}

impl From<bincode::Error> for NetworkError {
    fn from(e: bincode::Error) -> Self {
        NetworkError::SerializationError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for NetworkError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        NetworkError::TransportError(e.to_string())
    }
}
