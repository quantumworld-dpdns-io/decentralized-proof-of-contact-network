use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub provider: AiProvider,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,

    #[serde(default)]
    pub endpoint: String,

    pub api_key: Option<String>,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    #[serde(default = "default_temperature")]
    pub temperature: f64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::default(),
            model: default_model(),
            embedding_model: default_embedding_model(),
            endpoint: String::new(),
            api_key: None,
            max_retries: default_max_retries(),
            timeout_secs: default_timeout_secs(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiProvider {
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "lm-studio")]
    LMStudio,
    #[serde(rename = "vllm")]
    Vllm,
    #[serde(rename = "sglang")]
    Sglang,
    #[serde(rename = "llamacpp")]
    Llamacpp,
}

impl Default for AiProvider {
    fn default() -> Self {
        Self::Ollama
    }
}

impl AiProvider {
    pub fn default_endpoint(&self) -> &str {
        match self {
            Self::Ollama => "http://localhost:11434",
            Self::LMStudio => "http://localhost:1234",
            Self::Vllm => "http://localhost:8000",
            Self::Sglang => "http://localhost:30000",
            Self::Llamacpp => "http://localhost:8080",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Ollama => "ollama",
            Self::LMStudio => "lm-studio",
            Self::Vllm => "vllm",
            Self::Sglang => "sglang",
            Self::Llamacpp => "llamacpp",
        }
    }
}

fn default_model() -> String {
    "llama3.2".to_string()
}

fn default_embedding_model() -> String {
    "nomic-embed-text".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_timeout_secs() -> u64 {
    60
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f64 {
    0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AiConfig::default();
        assert_eq!(config.provider, AiProvider::Ollama);
        assert_eq!(config.model, "llama3.2");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_tokens, 4096);
        assert!(config.temperature - 0.7 < f64::EPSILON);
    }

    #[test]
    fn test_provider_defaults() {
        assert_eq!(AiProvider::Ollama.default_endpoint(), "http://localhost:11434");
        assert_eq!(AiProvider::LMStudio.default_endpoint(), "http://localhost:1234");
        assert_eq!(AiProvider::Vllm.default_endpoint(), "http://localhost:8000");
        assert_eq!(AiProvider::Sglang.default_endpoint(), "http://localhost:30000");
        assert_eq!(AiProvider::Llamacpp.default_endpoint(), "http://localhost:8080");
    }

    #[test]
    fn test_serialize_roundtrip() {
        let config = AiConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.provider, deserialized.provider);
        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.max_retries, deserialized.max_retries);
    }

    #[test]
    fn test_provider_as_str() {
        assert_eq!(AiProvider::Ollama.as_str(), "ollama");
        assert_eq!(AiProvider::LMStudio.as_str(), "lm-studio");
        assert_eq!(AiProvider::Vllm.as_str(), "vllm");
        assert_eq!(AiProvider::Sglang.as_str(), "sglang");
        assert_eq!(AiProvider::Llamacpp.as_str(), "llamacpp");
    }
}
