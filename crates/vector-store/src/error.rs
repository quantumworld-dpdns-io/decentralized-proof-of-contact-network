use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum VectorStoreError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Insert failed: {0}")]
    InsertFailed(String),

    #[error("Search failed: {0}")]
    SearchFailed(String),

    #[error("Delete failed: {0}")]
    DeleteFailed(String),

    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

pub type Result<T> = std::result::Result<T, VectorStoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            VectorStoreError::ConnectionError("refused".into()).to_string(),
            "Connection error: refused"
        );
        assert_eq!(
            VectorStoreError::CollectionNotFound("my_collection".into()).to_string(),
            "Collection not found: my_collection"
        );
        assert_eq!(
            VectorStoreError::InsertFailed("timeout".into()).to_string(),
            "Insert failed: timeout"
        );
        assert_eq!(
            VectorStoreError::SearchFailed("no results".into()).to_string(),
            "Search failed: no results"
        );
        assert_eq!(
            VectorStoreError::DeleteFailed("not found".into()).to_string(),
            "Delete failed: not found"
        );
        assert_eq!(
            VectorStoreError::ProviderNotAvailable("chroma".into()).to_string(),
            "Provider not available: chroma"
        );
        assert_eq!(
            VectorStoreError::EmbeddingError("invalid dimension".into()).to_string(),
            "Embedding error: invalid dimension"
        );
        assert_eq!(
            VectorStoreError::AuthenticationError("invalid key".into()).to_string(),
            "Authentication error: invalid key"
        );
    }

    #[test]
    fn test_error_clone() {
        let err = VectorStoreError::ConnectionError("test".into());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }
}
