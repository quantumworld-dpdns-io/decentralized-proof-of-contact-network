use crate::error::VectorStoreError;
use async_trait::async_trait;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

#[async_trait]
pub trait EmbeddingFunction: Send + Sync {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

// ---------------------------------------------------------------------------
// Ollama embedding provider
// ---------------------------------------------------------------------------

pub struct OllamaEmbedding {
    client: reqwest::Client,
    base_url: String,
    model: String,
    dim: usize,
}

impl OllamaEmbedding {
    pub fn new(base_url: &str, model: &str, dim: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            dim,
        }
    }
}

#[async_trait]
impl EmbeddingFunction for OllamaEmbedding {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": text,
            }))
            .send()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let embedding: Vec<f32> = body["embedding"]
            .as_array()
            .ok_or_else(|| {
                VectorStoreError::EmbeddingError("Ollama response missing 'embedding' field".into())
            })?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed_text(text).await?);
        }
        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

// ---------------------------------------------------------------------------
// LM Studio embedding provider (OpenAI-compatible /v1/embeddings)
// ---------------------------------------------------------------------------

pub struct LMStudioEmbedding {
    client: reqwest::Client,
    base_url: String,
    model: String,
    dim: usize,
}

impl LMStudioEmbedding {
    pub fn new(base_url: &str, model: &str, dim: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            dim,
        }
    }
}

#[async_trait]
impl EmbeddingFunction for LMStudioEmbedding {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "input": text,
            }))
            .send()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let embedding: Vec<f32> = body["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| {
                VectorStoreError::EmbeddingError(
                    "LM Studio response missing 'data[0].embedding' field".into(),
                )
            })?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let resp = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "input": texts,
            }))
            .send()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let data = body["data"].as_array().ok_or_else(|| {
            VectorStoreError::EmbeddingError("LM Studio response missing 'data' field".into())
        })?;

        let mut embeddings = Vec::with_capacity(data.len());
        for entry in data {
            let emb: Vec<f32> = entry["embedding"]
                .as_array()
                .ok_or_else(|| {
                    VectorStoreError::EmbeddingError("missing 'embedding' in data entry".into())
                })?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            embeddings.push(emb);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible embedding provider
// ---------------------------------------------------------------------------

pub struct OpenAIEmbedding {
    client: reqwest::Client,
    base_url: String,
    model: String,
    api_key: String,
    dim: usize,
}

impl OpenAIEmbedding {
    pub fn new(base_url: &str, model: &str, api_key: &str, dim: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
            dim,
        }
    }
}

#[async_trait]
impl EmbeddingFunction for OpenAIEmbedding {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "input": text,
            }))
            .send()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let embedding: Vec<f32> = body["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| {
                VectorStoreError::EmbeddingError(
                    "OpenAI response missing 'data[0].embedding' field".into(),
                )
            })?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let resp = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "input": texts,
            }))
            .send()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;

        let data = body["data"].as_array().ok_or_else(|| {
            VectorStoreError::EmbeddingError("OpenAI response missing 'data' field".into())
        })?;

        let mut embeddings = Vec::with_capacity(data.len());
        for entry in data {
            let emb: Vec<f32> = entry["embedding"]
                .as_array()
                .ok_or_else(|| {
                    VectorStoreError::EmbeddingError("missing 'embedding' in data entry".into())
                })?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            embeddings.push(emb);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_embedding_new() {
        let emb = OllamaEmbedding::new("http://localhost:11434", "nomic-embed-text", 768);
        assert_eq!(emb.dim, 768);
        assert_eq!(emb.model, "nomic-embed-text");
        assert_eq!(emb.base_url, "http://localhost:11434");
        assert_eq!(emb.dimension(), 768);
    }

    #[test]
    fn test_lm_studio_embedding_new() {
        let emb = LMStudioEmbedding::new("http://localhost:1234", "model-identifier", 384);
        assert_eq!(emb.dim, 384);
        assert_eq!(emb.dimension(), 384);
    }

    #[test]
    fn test_openai_embedding_new() {
        let emb = OpenAIEmbedding::new(
            "https://api.openai.com",
            "text-embedding-3-small",
            "sk-test",
            1536,
        );
        assert_eq!(emb.dim, 1536);
        assert_eq!(emb.dimension(), 1536);
        assert_eq!(emb.api_key, "sk-test");
    }

    #[test]
    fn test_ollama_url_trailing_slash() {
        let emb = OllamaEmbedding::new("http://localhost:11434/", "test", 64);
        assert_eq!(emb.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_lm_studio_embedding_trait_object() {
        let emb = LMStudioEmbedding::new("http://localhost:1234", "test", 128);
        let trait_obj: &dyn EmbeddingFunction = &emb;
        assert_eq!(trait_obj.dimension(), 128);
    }

    #[test]
    fn test_openai_embedding_trait_object() {
        let emb = OpenAIEmbedding::new("http://localhost:9999", "test", "key", 256);
        let trait_obj: &dyn EmbeddingFunction = &emb;
        assert_eq!(trait_obj.dimension(), 256);
    }
}
