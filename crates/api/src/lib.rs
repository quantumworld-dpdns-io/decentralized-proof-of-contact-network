use serde::{Deserialize, Serialize};

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
