use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{AiError, Result};
use crate::provider::{AiProvider, ChatConfig, ChatMessage, ChatResponse, TokenUsage};

#[derive(Debug)]
pub struct OpenAiCompatibleProvider {
    client: Client,
    endpoint: String,
    model: String,
    embedding_model: String,
    timeout_secs: u64,
    api_key: Option<String>,
    #[allow(dead_code)]
    max_retries: u32,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        endpoint: &str,
        model: &str,
        embedding_model: &str,
        api_key: Option<&str>,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.to_string(),
            model: model.to_string(),
            embedding_model: embedding_model.to_string(),
            timeout_secs,
            api_key: api_key.map(String::from),
            max_retries,
        }
    }

    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref key) = self.api_key {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", key).parse().unwrap(),
            );
        }
        headers
    }
}

#[async_trait]
impl AiProvider for OpenAiCompatibleProvider {
    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(&self, messages: &[ChatMessage], config: &ChatConfig) -> Result<ChatResponse> {
        #[derive(Serialize)]
        struct ChatRequest {
            model: String,
            messages: Vec<ChatMessageInner>,
            temperature: f64,
            max_tokens: u32,
            stream: bool,
        }

        #[derive(Serialize)]
        struct ChatMessageInner {
            role: String,
            content: String,
        }

        let msgs: Vec<ChatMessageInner> = messages
            .iter()
            .map(|m| ChatMessageInner {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: msgs,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            stream: config.stream,
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .headers(self.build_headers())
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
        struct ChatResponseBody {
            choices: Vec<Choice>,
            usage: Option<UsageData>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: ResponseMessage,
        }

        #[derive(Deserialize)]
        struct ResponseMessage {
            content: Option<String>,
        }

        #[derive(Deserialize)]
        struct UsageData {
            prompt_tokens: u32,
            completion_tokens: u32,
        }

        let body: ChatResponseBody = response
            .json()
            .await
            .map_err(|e| AiError::ProviderError(format!("Failed to parse response: {}", e)))?;

        let content = body
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        let usage = body
            .usage
            .unwrap_or(UsageData {
                prompt_tokens: 0,
                completion_tokens: 0,
            });

        Ok(ChatResponse {
            content,
            usage: TokenUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
            },
        })
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_batch(&[text]).await.map(|mut v| v.remove(0))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        #[derive(Serialize)]
        struct EmbedRequest<'a> {
            model: String,
            input: Vec<&'a str>,
        }

        #[derive(Deserialize)]
        struct EmbedResponse {
            data: Vec<EmbedData>,
        }

        #[derive(Deserialize)]
        struct EmbedData {
            embedding: Vec<f32>,
        }

        let request = EmbedRequest {
            model: self.embedding_model.clone(),
            input: texts.to_vec(),
        };

        let response = self
            .client
            .post(format!("{}/v1/embeddings", self.endpoint))
            .headers(self.build_headers())
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

        let body: EmbedResponse = response
            .json()
            .await
            .map_err(|e| AiError::EmbeddingError(format!("Failed to parse: {}", e)))?;

        let embeddings: Vec<Vec<f32>> = body.data.into_iter().map(|d| d.embedding).collect();

        Ok(embeddings)
    }
}

#[macro_export]
macro_rules! make_openai_compat_provider {
    ($name:ident, $default_endpoint:expr) => {
        #[derive(Debug)]
        pub struct $name(OpenAiCompatibleProvider);

        impl $name {
            pub fn new(config: &$crate::config::AiConfig) -> Self {
                let endpoint = if config.endpoint.is_empty() {
                    $default_endpoint.to_string()
                } else {
                    config.endpoint.clone()
                };
                Self(OpenAiCompatibleProvider::new(
                    &endpoint,
                    &config.model,
                    &config.embedding_model,
                    config.api_key.as_deref(),
                    config.timeout_secs,
                    config.max_retries,
                ))
            }
        }

        #[async_trait]
        impl AiProvider for $name {
            fn model(&self) -> &str {
                self.0.model()
            }

            async fn chat(
                &self,
                messages: &[ChatMessage],
                config: &ChatConfig,
            ) -> Result<ChatResponse> {
                self.0.chat(messages, config).await
            }

            async fn embed(&self, text: &str) -> Result<Vec<f32>> {
                self.0.embed(text).await
            }

            async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
                self.0.embed_batch(texts).await
            }
        }
    };
}
