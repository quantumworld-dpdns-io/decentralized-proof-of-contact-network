use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::AiConfig;
use crate::error::{AiError, Result};
use crate::provider::{AiProvider, ChatConfig, ChatMessage, ChatResponse, TokenUsage};

#[derive(Debug)]
pub struct OllamaProvider {
    client: Client,
    endpoint: String,
    model: String,
    embedding_model: String,
    timeout_secs: u64,
    #[allow(dead_code)]
    max_retries: u32,
}

impl OllamaProvider {
    pub fn new(config: &AiConfig) -> Self {
        let endpoint = if config.endpoint.is_empty() {
            config.provider.default_endpoint().to_string()
        } else {
            config.endpoint.clone()
        };
        Self {
            client: Client::new(),
            endpoint,
            model: config.model.clone(),
            embedding_model: config.embedding_model.clone(),
            timeout_secs: config.timeout_secs,
            max_retries: config.max_retries,
        }
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(&self, messages: &[ChatMessage], config: &ChatConfig) -> Result<ChatResponse> {
        #[derive(Serialize)]
        struct OllamaChatRequest {
            model: String,
            messages: Vec<OllamaMessage>,
            stream: bool,
            options: OllamaOptions,
        }

        #[derive(Serialize)]
        struct OllamaMessage {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct OllamaOptions {
            temperature: f64,
            num_predict: u32,
        }

        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: config.stream,
            options: OllamaOptions {
                temperature: config.temperature,
                num_predict: config.max_tokens,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&request)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiError::Timeout
                } else {
                    AiError::ProviderError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(match status.as_u16() {
                429 => AiError::RateLimited,
                401 | 403 => AiError::AuthError(status.to_string()),
                404 => AiError::ModelNotFound(self.model.clone()),
                _ => AiError::ProviderError(format!("HTTP {}", status)),
            });
        }

        #[derive(Deserialize)]
        struct OllamaChatResponse {
            message: OllamaResponseMessage,
            #[serde(default)]
            eval_count: Option<u32>,
            #[serde(default)]
            prompt_eval_count: Option<u32>,
        }

        #[derive(Deserialize)]
        struct OllamaResponseMessage {
            content: String,
        }

        let body: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::ProviderError(format!("Failed to parse response: {}", e)))?;

        Ok(ChatResponse {
            content: body.message.content,
            usage: TokenUsage {
                prompt_tokens: body.prompt_eval_count.unwrap_or(0),
                completion_tokens: body.eval_count.unwrap_or(0),
            },
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_batch(&[text]).await.map(|mut v| v.remove(0))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        #[derive(Serialize)]
        struct OllamaEmbedRequest<'a> {
            model: String,
            input: Vec<&'a str>,
        }

        #[derive(Deserialize)]
        struct OllamaEmbedResponse {
            embeddings: Vec<Vec<f32>>,
        }

        let request = OllamaEmbedRequest {
            model: self.embedding_model.clone(),
            input: texts.to_vec(),
        };

        let response = self
            .client
            .post(format!("{}/api/embed", self.endpoint))
            .json(&request)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiError::Timeout
                } else {
                    AiError::EmbeddingError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                429 => AiError::RateLimited,
                _ => AiError::EmbeddingError(format!("HTTP {}", response.status())),
            });
        }

        let body: OllamaEmbedResponse = response
            .json()
            .await
            .map_err(|e| AiError::EmbeddingError(format!("Failed to parse: {}", e)))?;

        Ok(body.embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_construction() {
        let config = AiConfig::default();
        let provider = OllamaProvider::new(&config);
        assert_eq!(provider.model(), "llama3.2");
        assert_eq!(provider.endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_provider_custom_endpoint() {
        let mut config = AiConfig::default();
        config.endpoint = "http://custom:11434".to_string();
        let provider = OllamaProvider::new(&config);
        assert_eq!(provider.endpoint, "http://custom:11434");
    }
}
