use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VectorStoreProvider {
    Chroma,
    Qdrant,
    Weaviate,
    LanceDb,
    Milvus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub provider: VectorStoreProvider,
    pub host: String,
    pub port: u16,
    pub collection: String,
    pub api_key: Option<String>,
    pub embedding_dimension: usize,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: VectorStoreProvider::Chroma,
            host: "127.0.0.1".to_string(),
            port: 8000,
            collection: "default".to_string(),
            api_key: None,
            embedding_dimension: 768,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VectorStoreConfig::default();
        assert_eq!(config.provider, VectorStoreProvider::Chroma);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8000);
        assert_eq!(config.collection, "default");
        assert!(config.api_key.is_none());
        assert_eq!(config.embedding_dimension, 768);
    }

    #[test]
    fn test_config_serialization() {
        let config = VectorStoreConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: VectorStoreConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.provider, deserialized.provider);
        assert_eq!(config.host, deserialized.host);
        assert_eq!(config.port, deserialized.port);
    }

    #[test]
    fn test_config_with_api_key() {
        let config = VectorStoreConfig {
            provider: VectorStoreProvider::Qdrant,
            host: "10.0.0.1".to_string(),
            port: 6333,
            collection: "vectors".to_string(),
            api_key: Some("secret".to_string()),
            embedding_dimension: 384,
        };
        assert_eq!(config.provider, VectorStoreProvider::Qdrant);
        assert_eq!(config.api_key, Some("secret".to_string()));
        assert_eq!(config.embedding_dimension, 384);
    }

    #[test]
    fn test_provider_equality() {
        assert_eq!(VectorStoreProvider::Chroma, VectorStoreProvider::Chroma);
        assert_ne!(VectorStoreProvider::Chroma, VectorStoreProvider::Qdrant);
    }
}
