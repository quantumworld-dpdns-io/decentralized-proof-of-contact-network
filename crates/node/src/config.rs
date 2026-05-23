use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "node")]
pub struct NodeSection {
    #[serde(default = "default_node_id")]
    pub id: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for NodeSection {
    fn default() -> Self {
        Self {
            id: default_node_id(),
            data_dir: default_data_dir(),
            log_level: default_log_level(),
        }
    }
}

fn default_node_id() -> String {
    "poi-node".to_string()
}

fn default_data_dir() -> String {
    "./data".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default)]
    pub external_addr: Option<String>,
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,
    #[serde(default = "default_handshake_timeout")]
    pub handshake_timeout_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            external_addr: None,
            bootstrap_peers: Vec::new(),
            max_peers: default_max_peers(),
            handshake_timeout_secs: default_handshake_timeout(),
        }
    }
}

fn default_listen_addr() -> String {
    "0.0.0.0:9090".to_string()
}

fn default_max_peers() -> usize {
    50
}

fn default_handshake_timeout() -> u64 {
    10
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: default_bind_addr(),
            allowed_origins: Vec::new(),
        }
    }
}

fn default_bind_addr() -> String {
    "0.0.0.0:3000".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_provider")]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub embedding_model: String,
    #[serde(default)]
    pub endpoint: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_ai_provider(),
            model: String::new(),
            embedding_model: String::new(),
            endpoint: String::new(),
        }
    }
}

fn default_ai_provider() -> String {
    "ollama".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    #[serde(default = "default_vs_provider")]
    pub provider: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_vs_port")]
    pub port: u16,
    #[serde(default)]
    pub collection: String,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: default_vs_provider(),
            host: String::new(),
            port: default_vs_port(),
            collection: String::new(),
        }
    }
}

fn default_vs_provider() -> String {
    "chroma".to_string()
}

fn default_vs_port() -> u16 {
    8000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    #[serde(default)]
    pub iceberg_warehouse: Option<String>,
    #[serde(default)]
    pub duckdb_path: Option<String>,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            iceberg_warehouse: None,
            duckdb_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    #[serde(rename = "node")]
    pub node: NodeSection,
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
            node: NodeSection::default(),
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
    pub fn from_file(path: &str) -> Result<Self, crate::ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| crate::ConfigError::IoError {
            path: path.to_string(),
            cause: e.to_string(),
        })?;
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("toml");
        let result = match ext {
            "toml" => toml::from_str(&content)
                .map_err(|e| crate::ConfigError::ParseError(e.to_string())),
            "yaml" | "yml" => serde_yaml::from_str(&content)
                .map_err(|e| crate::ConfigError::ParseError(e.to_string())),
            "json" => serde_json::from_str(&content)
                .map_err(|e| crate::ConfigError::ParseError(e.to_string())),
            other => Err(crate::ConfigError::UnsupportedFormat(other.to_string())),
        };
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NodeConfig::default();
        assert_eq!(config.node.log_level, "info");
        assert_eq!(config.network.listen_addr, "0.0.0.0:9090");
        assert_eq!(config.storage.provider, "duckdb");
        assert_eq!(config.observability.log_level, "info");
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
[node]
id = "poi-test-node"
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
        assert_eq!(config.node.id, "poi-test-node");
        assert_eq!(config.node.data_dir, "/tmp/data");
        assert_eq!(config.node.log_level, "debug");
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
    fn test_node_section_default() {
        let ns = NodeSection::default();
        assert_eq!(ns.id, "poi-node");
        assert_eq!(ns.data_dir, "./data");
    }

    #[test]
    fn test_network_config_default() {
        let nc = NetworkConfig::default();
        assert_eq!(nc.listen_addr, "0.0.0.0:9090");
        assert!(nc.external_addr.is_none());
    }

    #[test]
    fn test_observability_config_default() {
        let oc = ObservabilityConfig::default();
        assert!(oc.otlp_endpoint.is_none());
        assert!(oc.prometheus_port.is_none());
        assert_eq!(oc.log_level, "info");
    }
}
