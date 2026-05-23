use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    #[serde(default)]
    pub allowed_origins: Vec<String>,

    #[serde(default)]
    pub tls_cert: Option<String>,

    #[serde(default)]
    pub tls_key: Option<String>,

    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,

    #[serde(default = "default_rate_limit_rpm")]
    pub rate_limit_rpm: u32,

    #[serde(default = "default_enable_swagger")]
    pub enable_swagger: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: default_bind_addr(),
            allowed_origins: Vec::new(),
            tls_cert: None,
            tls_key: None,
            max_body_size: default_max_body_size(),
            rate_limit_rpm: default_rate_limit_rpm(),
            enable_swagger: default_enable_swagger(),
        }
    }
}

fn default_bind_addr() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024
}

fn default_rate_limit_rpm() -> u32 {
    60
}

fn default_enable_swagger() -> bool {
    true
}
