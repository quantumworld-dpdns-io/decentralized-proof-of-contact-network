use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage], config: &ChatConfig) -> Result<ChatResponse>;

    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    fn model(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    #[serde(default)]
    pub stream: bool,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            stream: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

fn default_temperature() -> f64 {
    0.7
}

fn default_max_tokens() -> u32 {
    4096
}
