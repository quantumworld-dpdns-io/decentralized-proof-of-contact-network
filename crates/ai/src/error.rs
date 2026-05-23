use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Rate limited by provider")]
    RateLimited,

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Request timed out")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, AiError>;
