use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_provider")]
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
            provider: default_provider(),
            model: String::new(),
            embedding_model: String::new(),
            endpoint: String::new(),
        }
    }
}

fn default_provider() -> String {
    "ollama".to_string()
}
