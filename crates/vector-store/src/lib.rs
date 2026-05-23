use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub collection: String,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            host: String::new(),
            port: default_port(),
            collection: String::new(),
        }
    }
}

fn default_provider() -> String {
    "chroma".to_string()
}

fn default_port() -> u16 {
    8000
}
