use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::node_id;
use crate::types::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreConfig {
    pub node_id: NodeId,
    pub data_dir: String,
    pub log_level: String,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub max_time_drift_secs: u64,
    pub max_window_duration_secs: u64,
    pub protocol_version: String,
    pub enable_pqc: bool,
    pub metrics_enabled: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        CoreConfig {
            node_id: node_id::generate_node_id(),
            data_dir: String::from("./data"),
            log_level: String::from("info"),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            max_time_drift_secs: 300,
            max_window_duration_secs: 86400,
            protocol_version: String::from("1.0"),
            enable_pqc: false,
            metrics_enabled: true,
        }
    }
}

impl CoreConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            Ok(serde_yaml::from_str(&content)?)
        } else {
            Ok(serde_json::from_str(&content)?)
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !node_id::validate_node_id(&self.node_id) {
            return Err(crate::error::Error::InvalidKey(format!(
                "invalid node id: {}",
                self.node_id.0
            )));
        }
        if self.data_dir.is_empty() {
            return Err(crate::error::Error::InvalidKey(
                "data_dir must not be empty".into(),
            ));
        }
        if self.max_time_drift_secs == 0 {
            return Err(crate::error::Error::InvalidKey(
                "max_time_drift_secs must be > 0".into(),
            ));
        }
        if self.max_window_duration_secs == 0 {
            return Err(crate::error::Error::InvalidKey(
                "max_window_duration_secs must be > 0".into(),
            ));
        }
        if self.protocol_version.is_empty() {
            return Err(crate::error::Error::InvalidKey(
                "protocol_version must not be empty".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub listen_address: String,
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: u32,
    pub handshake_timeout_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_address: String::from("0.0.0.0"),
            listen_port: 9734,
            bootstrap_peers: Vec::new(),
            max_peers: 50,
            handshake_timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    pub backend: String,
    pub db_path: String,
    pub max_entries: u64,
    pub auto_prune: bool,
    pub prune_interval_secs: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            backend: String::from("sled"),
            db_path: String::from("./data/db"),
            max_entries: 1_000_000,
            auto_prune: true,
            prune_interval_secs: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = CoreConfig::default();
        assert!(config.node_id.0.starts_with("poi-"));
        assert_eq!(config.data_dir, "./data");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.max_time_drift_secs, 300);
    }

    #[test]
    fn test_config_validate_valid() {
        let config = CoreConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_invalid_node_id() {
        let mut config = CoreConfig::default();
        config.node_id = NodeId("bad".into());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_empty_data_dir() {
        let mut config = CoreConfig::default();
        config.data_dir = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_zero_drift() {
        let mut config = CoreConfig::default();
        config.max_time_drift_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_config_default() {
        let net = NetworkConfig::default();
        assert_eq!(net.listen_address, "0.0.0.0");
        assert_eq!(net.listen_port, 9734);
        assert_eq!(net.max_peers, 50);
    }

    #[test]
    fn test_storage_config_default() {
        let storage = StorageConfig::default();
        assert_eq!(storage.backend, "sled");
        assert_eq!(storage.max_entries, 1_000_000);
        assert!(storage.auto_prune);
    }

    #[test]
    fn test_config_from_json() {
        let json = r#"{
            "node_id": "poi-550e8400-e29b-41d4-a716-446655440000",
            "data_dir": "/tmp/data",
            "log_level": "debug",
            "network": {
                "listen_port": 9999
            },
            "storage": {
                "backend": "rocksdb"
            },
            "max_time_drift_secs": 600,
            "max_window_duration_secs": 43200,
            "protocol_version": "2.0",
            "enable_pqc": true,
            "metrics_enabled": false
        }"#;
        let config: CoreConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.data_dir, "/tmp/data");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.network.listen_port, 9999);
        assert_eq!(config.storage.backend, "rocksdb");
        assert_eq!(config.max_time_drift_secs, 600);
        assert!(config.enable_pqc);
        assert!(!config.metrics_enabled);
    }
}
