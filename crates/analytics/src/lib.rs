use serde::{Deserialize, Serialize};

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
