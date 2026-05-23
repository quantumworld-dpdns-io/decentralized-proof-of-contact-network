use crate::error::ConfigError;
use poi_ai::AiConfig;
use poi_analytics::AnalyticsConfig;
use poi_api::ApiConfig;
use poi_core::CoreConfig;
use poi_networking::NetworkConfig;
use poi_vector_store::VectorStoreConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_provider")]
    pub provider: String,
    #[serde(default)]
    pub duckdb_path: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            provider: default_storage_provider(),
            duckdb_path: None,
        }
    }
}

fn default_storage_provider() -> String {
    "duckdb".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default)]
    pub prometheus_port: Option<u16>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: None,
            prometheus_port: None,
            log_level: default_log_level(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    #[serde(rename = "node")]
    pub core: CoreConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    #[serde(rename = "vector_store")]
    pub vector_store: VectorStoreConfig,
    pub api: ApiConfig,
    pub ai: AiConfig,
    pub analytics: AnalyticsConfig,
    pub observability: ObservabilityConfig,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            vector_store: VectorStoreConfig::default(),
            api: ApiConfig::default(),
            ai: AiConfig::default(),
            analytics: AnalyticsConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

impl NodeConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::IoError {
            path: path.to_string(),
            cause: e.to_string(),
        })?;
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("toml");
        match ext {
            "toml" => toml::from_str(&content),
            "yaml" | "yml" => {
                serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
            }
            "json" => {
                serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
            }
            other => Err(ConfigError::UnsupportedFormat(other.to_string())),
        }
        .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NodeConfig::default();
        assert_eq!(config.core.node_id, "poi-node");
        assert_eq!(config.network.listen_addr, "0.0.0.0:9090");
        assert_eq!(config.storage.provider, "duckdb");
        assert_eq!(config.observability.log_level, "info");
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
[node]
id = "test-node"
data_dir = "/tmp/data"
log_level = "debug"

[network]
listen_addr = "0.0.0.0:9091"
bootstrap_peers = ["/ip4/1.2.3.4/tcp/9090"]

[storage]
provider = "duckdb"
duckdb_path = "/tmp/data/proofs.duckdb"

[vector_store]
provider = "chroma"
host = "localhost"
port = 8000
collection = "test-proofs"

[api]
bind_addr = "0.0.0.0:3001"
allowed_origins = ["http://localhost:3001"]

[ai]
provider = "ollama"
model = "gemma3:latest"
embedding_model = "nomic-embed-text:latest"
endpoint = "http://localhost:11434"

[analytics]
iceberg_warehouse = "/tmp/data/iceberg"
duckdb_path = "/tmp/data/analytics.duckdb"

[observability]
otlp_endpoint = "http://localhost:4317"
prometheus_port = 9091
log_level = "debug"
"#;
        let config: NodeConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.core.node_id, "test-node");
        assert_eq!(config.core.data_dir, "/tmp/data");
        assert_eq!(config.core.log_level, "debug");
        assert_eq!(config.network.listen_addr, "0.0.0.0:9091");
        assert_eq!(config.network.bootstrap_peers.len(), 1);
        assert_eq!(config.api.bind_addr, "0.0.0.0:3001");
        assert_eq!(config.observability.prometheus_port, Some(9091));
    }

    #[test]
    fn test_config_from_file_not_found() {
        let err = NodeConfig::from_file("/nonexistent/config.toml");
        assert!(err.is_err());
    }

    #[test]
    fn test_storage_config_default() {
        let sc = StorageConfig::default();
        assert_eq!(sc.provider, "duckdb");
        assert!(sc.duckdb_path.is_none());
    }

    #[test]
    fn test_observability_config_default() {
        let oc = ObservabilityConfig::default();
        assert!(oc.otlp_endpoint.is_none());
        assert!(oc.prometheus_port.is_none());
        assert_eq!(oc.log_level, "info");
    }
}
