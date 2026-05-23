use crate::config::{VectorStoreConfig, VectorStoreProvider};
use crate::error::VectorStoreError;
use crate::traits::VectorStore;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

pub fn create_vector_store(config: &VectorStoreConfig) -> Result<Box<dyn VectorStore>> {
    match config.provider {
        #[cfg(feature = "chroma")]
        VectorStoreProvider::Chroma => Ok(Box::new(crate::chroma::ChromaDB::new(
            &config.host,
            config.port,
            config.api_key.clone(),
        ))),
        #[cfg(not(feature = "chroma"))]
        VectorStoreProvider::Chroma => Err(VectorStoreError::ProviderNotAvailable(
            "ChromaDB support not enabled (compile with feature 'chroma')".into(),
        )),

        #[cfg(feature = "qdrant")]
        VectorStoreProvider::Qdrant => Ok(Box::new(crate::qdrant::QdrantDB::new(
            &config.host,
            config.port,
            config.api_key.clone(),
        ))),
        #[cfg(not(feature = "qdrant"))]
        VectorStoreProvider::Qdrant => Err(VectorStoreError::ProviderNotAvailable(
            "Qdrant support not enabled (compile with feature 'qdrant')".into(),
        )),

        #[cfg(feature = "weaviate")]
        VectorStoreProvider::Weaviate => Ok(Box::new(crate::weaviate::WeaviateDB::new(
            &config.host,
            config.port,
            config.api_key.clone(),
        ))),
        #[cfg(not(feature = "weaviate"))]
        VectorStoreProvider::Weaviate => Err(VectorStoreError::ProviderNotAvailable(
            "Weaviate support not enabled (compile with feature 'weaviate')".into(),
        )),

        #[cfg(feature = "lancedb")]
        VectorStoreProvider::LanceDb => Ok(Box::new(crate::lancedb::LanceDB::new(
            &config.host,
            config.port,
            config.api_key.clone(),
        ))),
        #[cfg(not(feature = "lancedb"))]
        VectorStoreProvider::LanceDb => Err(VectorStoreError::ProviderNotAvailable(
            "LanceDB support not enabled (compile with feature 'lancedb')".into(),
        )),

        #[cfg(feature = "milvus")]
        VectorStoreProvider::Milvus => Ok(Box::new(crate::milvus::MilvusDB::new(
            &config.host,
            config.port,
            config.api_key.clone(),
        ))),
        #[cfg(not(feature = "milvus"))]
        VectorStoreProvider::Milvus => Err(VectorStoreError::ProviderNotAvailable(
            "Milvus support not enabled (compile with feature 'milvus')".into(),
        )),
    }
}

pub async fn validate_connection(config: &VectorStoreConfig) -> Result<()> {
    let store = create_vector_store(config)?;
    let healthy = store
        .health_check()
        .await
        .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;
    if healthy {
        Ok(())
    } else {
        Err(VectorStoreError::ConnectionError(
            "health check returned unhealthy".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VectorStoreProvider;

    #[test]
    fn test_create_chroma_store() {
        let config = VectorStoreConfig {
            provider: VectorStoreProvider::Chroma,
            host: "localhost".into(),
            port: 8000,
            collection: "test".into(),
            api_key: None,
            embedding_dimension: 768,
        };
        let store = create_vector_store(&config);
        #[cfg(feature = "chroma")]
        assert!(store.is_ok());
        #[cfg(not(feature = "chroma"))]
        assert!(store.is_err());
    }

    #[test]
    fn test_create_qdrant_store() {
        let config = VectorStoreConfig {
            provider: VectorStoreProvider::Qdrant,
            host: "localhost".into(),
            port: 6333,
            collection: "test".into(),
            api_key: None,
            embedding_dimension: 384,
        };
        let store = create_vector_store(&config);
        #[cfg(feature = "qdrant")]
        assert!(store.is_ok());
        #[cfg(not(feature = "qdrant"))]
        assert!(store.is_err());
    }

    #[test]
    fn test_create_store_with_api_key() {
        let config = VectorStoreConfig {
            provider: VectorStoreProvider::Chroma,
            host: "10.0.0.1".into(),
            port: 8000,
            collection: "sec".into(),
            api_key: Some("secret-key".into()),
            embedding_dimension: 768,
        };
        let store = create_vector_store(&config);
        #[cfg(feature = "chroma")]
        {
            assert!(store.is_ok());
            let _s = store.unwrap();
        }
    }

    #[test]
    fn test_validate_connection_fails_no_server() {
        let config = VectorStoreConfig {
            provider: VectorStoreProvider::Chroma,
            host: "127.0.0.1".into(),
            port: 1,
            collection: "test".into(),
            api_key: None,
            embedding_dimension: 768,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(validate_connection(&config));
        #[cfg(feature = "chroma")]
        assert!(result.is_err());
        #[cfg(not(feature = "chroma"))]
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_provider_fails() {
        let config = VectorStoreConfig {
            provider: VectorStoreProvider::Weaviate,
            host: "localhost".into(),
            port: 8080,
            collection: "test".into(),
            api_key: None,
            embedding_dimension: 768,
        };
        let store = create_vector_store(&config);
        #[cfg(not(any(feature = "weaviate")))]
        assert!(store.is_err());
    }
}
