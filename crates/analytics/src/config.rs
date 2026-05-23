use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub iceberg_warehouse: String,
    pub duckdb_path: String,
    pub enable_arrow: bool,
    pub enable_datafusion: bool,
    pub enable_iceberg: bool,
    pub trino_host: Option<String>,
    pub trino_port: Option<u16>,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            iceberg_warehouse: "/tmp/poi/warehouse".to_string(),
            duckdb_path: "/tmp/poi/analytics.duckdb".to_string(),
            enable_arrow: true,
            enable_datafusion: false,
            enable_iceberg: false,
            trino_host: None,
            trino_port: None,
        }
    }
}

impl AnalyticsConfig {
    pub fn new(
        iceberg_warehouse: impl Into<String>,
        duckdb_path: impl Into<String>,
    ) -> Self {
        Self {
            iceberg_warehouse: iceberg_warehouse.into(),
            duckdb_path: duckdb_path.into(),
            ..Default::default()
        }
    }

    pub fn with_trino(mut self, host: impl Into<String>, port: u16) -> Self {
        self.trino_host = Some(host.into());
        self.trino_port = Some(port);
        self
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.iceberg_warehouse.is_empty() {
            return Err(crate::AnalyticsError::Config(
                "iceberg_warehouse must not be empty".to_string(),
            ));
        }
        if self.duckdb_path.is_empty() {
            return Err(crate::AnalyticsError::Config(
                "duckdb_path must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    pub fn trino_address(&self) -> Option<String> {
        match (self.trino_host.as_ref(), self.trino_port) {
            (Some(host), Some(port)) => Some(format!("{}:{}", host, port)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnalyticsConfig::default();
        assert!(!config.iceberg_warehouse.is_empty());
        assert!(!config.duckdb_path.is_empty());
        assert!(config.enable_arrow);
        assert!(!config.enable_iceberg);
        assert!(config.trino_host.is_none());
    }

    #[test]
    fn test_config_validation() {
        let config = AnalyticsConfig::new("/warehouse", "/db.duckdb");
        assert!(config.validate().is_ok());

        let invalid = AnalyticsConfig {
            iceberg_warehouse: String::new(),
            duckdb_path: "/db.duckdb".to_string(),
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_trino_address() {
        let config = AnalyticsConfig::default().with_trino("localhost", 8080);
        assert_eq!(config.trino_address(), Some("localhost:8080".to_string()));

        let config = AnalyticsConfig::default();
        assert_eq!(config.trino_address(), None);
    }
}
