use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid proof: {0}")]
    InvalidProof(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Window expired")]
    WindowExpired,

    #[error("Chain broken at position {0}")]
    ChainBroken(u64),

    #[error("PQC error: {0}")]
    PqcError(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::StorageError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
