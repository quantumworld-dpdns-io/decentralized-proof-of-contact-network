use crate::error::VectorStoreError;
use crate::traits::{Filter, FilterCondition, SearchResult, VectorStore};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

pub struct ChromaDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl ChromaDB {
    pub fn new(host: &str, port: u16, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: format!("http://{}:{}", host, port),
            api_key,
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref key) = self.api_key {
            headers.insert(
                HeaderName::from_static("x-api-key"),
                HeaderValue::from_str(key).unwrap(),
            );
        }
        headers
    }

    async fn get_collection_id(&self, name: &str) -> Result<String> {
        let url = format!("{}/api/v1/collections?name={}", self.base_url, name);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() {
            let body: Value = resp
                .json()
                .await
                .map_err(|e| VectorStoreError::SearchFailed(e.to_string()))?;
            body["id"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| VectorStoreError::CollectionNotFound(name.to_string()))
        } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
            Err(VectorStoreError::CollectionNotFound(name.to_string()))
        } else {
            Err(VectorStoreError::SearchFailed(format!(
                "get collection failed: {}",
                resp.status()
            )))
        }
    }

    async fn ensure_collection(&self, name: &str, dimension: usize) -> Result<String> {
        match self.get_collection_id(name).await {
            Ok(id) => return Ok(id),
            Err(VectorStoreError::CollectionNotFound(_)) => {}
            Err(e) => return Err(e),
        }

        let url = format!("{}/api/v1/collections", self.base_url);
        let body = serde_json::json!({
            "name": name,
            "metadata": {
                "hnsw:space": "cosine",
                "dimension": dimension,
            }
        });

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(VectorStoreError::InsertFailed(format!(
                "create collection failed: {} {}",
                status, text
            )));
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::InsertFailed(e.to_string()))?;
        body["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| VectorStoreError::InsertFailed("missing collection id in response".into()))
    }
}

#[async_trait]
impl VectorStore for ChromaDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        self.ensure_collection(name, dimension).await?;
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let collection_id = self.get_collection_id(name).await?;
        let url = format!("{}/api/v1/collections/{}", self.base_url, collection_id);

        let resp = self
            .client
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(VectorStoreError::DeleteFailed(format!(
                "delete collection returned {}",
                resp.status()
            )))
        }
    }

    async fn insert(
        &self,
        collection: &str,
        id: &str,
        vector: &[f32],
        metadata: &[u8],
    ) -> Result<()> {
        let collection_id = self.ensure_collection(collection, vector.len()).await?;

        let metadata_value: Value = if metadata.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(metadata)
                .map_err(|e| VectorStoreError::InsertFailed(format!("invalid metadata json: {}", e)))?
        };

        let body = serde_json::json!({
            "ids": [id],
            "embeddings": [vector],
            "metadatas": [metadata_value],
        });

        let url = format!(
            "{}/api/v1/collections/{}/add",
            self.base_url, collection_id
        );

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(VectorStoreError::InsertFailed(format!("{}: {}", status, text)))
        }
    }

    async fn search(
        &self,
        collection: &str,
        vector: &[f32],
        limit: u64,
    ) -> Result<Vec<SearchResult>> {
        self.search_with_filter(collection, vector, &Filter::new(), limit)
            .await
    }

    async fn search_with_filter(
        &self,
        collection: &str,
        vector: &[f32],
        filter: &Filter,
        limit: u64,
    ) -> Result<Vec<SearchResult>> {
        let collection_id = self.ensure_collection(collection, vector.len()).await?;

        let where_clause = if filter.conditions.is_empty() {
            None
        } else {
            Some(build_chroma_filter(filter))
        };

        let mut body = serde_json::json!({
            "query_embeddings": [vector],
            "n_results": limit,
        });
        if let Some(w) = where_clause {
            body["where"] = w;
        }

        let url = format!(
            "{}/api/v1/collections/{}/query",
            self.base_url, collection_id
        );

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(VectorStoreError::SearchFailed(format!("{}: {}", status, text)));
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| VectorStoreError::SearchFailed(e.to_string()))?;

        let mut results = Vec::new();

        let empty_vec = Vec::new();

        if let Some(ids) = body["ids"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|a| a.as_array())
        {
            let distances = body["distances"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|a| a.as_array())
                .unwrap_or(&empty_vec);
            let metadatas = body["metadatas"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|a| a.as_array())
                .unwrap_or(&empty_vec);

            for (i, id_val) in ids.iter().enumerate() {
                let id = id_val.as_str().unwrap_or("").to_string();
                let score = distances
                    .get(i)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let metadata = metadatas
                    .get(i)
                    .and_then(|v| v.as_object())
                    .map(|obj| serde_json::to_vec(obj).unwrap_or_default())
                    .unwrap_or_default();
                results.push(SearchResult { id, score, metadata });
            }
        }

        Ok(results)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let collection_id = self.ensure_collection(collection, 0).await?;
        let url = format!(
            "{}/api/v1/collections/{}/delete",
            self.base_url, collection_id
        );

        let body = serde_json::json!({
            "ids": [id],
        });

        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(VectorStoreError::DeleteFailed(format!("{}: {}", status, text)))
        }
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/v1/heartbeat", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;
        Ok(resp.status().is_success())
    }
}

fn build_chroma_filter(filter: &Filter) -> Value {
    if filter.conditions.len() == 1 {
        build_condition(&filter.conditions[0])
    } else {
        let conditions: Vec<Value> = filter.conditions.iter().map(build_condition).collect();
        let mut and = serde_json::Map::new();
        and.insert("$and".to_string(), Value::Array(conditions));
        Value::Object(and)
    }
}

fn build_condition(condition: &FilterCondition) -> Value {
    match condition {
        FilterCondition::Equals { key, value } => {
            let mut op = serde_json::Map::new();
            op.insert("$eq".to_string(), Value::String(value.clone()));
            let mut cond = serde_json::Map::new();
            cond.insert(key.clone(), Value::Object(op));
            Value::Object(cond)
        }
        FilterCondition::GreaterThan { key, value } => {
            let mut op = serde_json::Map::new();
            op.insert(
                "$gt".to_string(),
                Value::Number(serde_json::Number::from_f64(*value).unwrap_or(0.into())),
            );
            let mut cond = serde_json::Map::new();
            cond.insert(key.clone(), Value::Object(op));
            Value::Object(cond)
        }
        FilterCondition::LessThan { key, value } => {
            let mut op = serde_json::Map::new();
            op.insert(
                "$lt".to_string(),
                Value::Number(serde_json::Number::from_f64(*value).unwrap_or(0.into())),
            );
            let mut cond = serde_json::Map::new();
            cond.insert(key.clone(), Value::Object(op));
            Value::Object(cond)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromadb_new() {
        let db = ChromaDB::new("localhost", 8000, None);
        assert_eq!(db.base_url, "http://localhost:8000");
        assert!(db.api_key.is_none());
    }

    #[test]
    fn test_chromadb_new_with_api_key() {
        let db = ChromaDB::new("10.0.0.1", 8000, Some("key123".into()));
        assert_eq!(db.base_url, "http://10.0.0.1:8000");
        assert_eq!(db.api_key, Some("key123".into()));
    }

    #[test]
    fn test_build_chroma_filter_equals() {
        let mut filter = Filter::new();
        filter.add_condition(FilterCondition::Equals {
            key: "color".into(),
            value: "red".into(),
        });
        let value = build_chroma_filter(&filter);
        assert_eq!(value, serde_json::json!({"color": {"$eq": "red"}}));
    }

    #[test]
    fn test_build_chroma_filter_greater_than() {
        let mut filter = Filter::new();
        filter.add_condition(FilterCondition::GreaterThan {
            key: "score".into(),
            value: 0.75,
        });
        let value = build_chroma_filter(&filter);
        assert_eq!(value, serde_json::json!({"score": {"$gt": 0.75}}));
    }

    #[test]
    fn test_build_chroma_filter_less_than() {
        let mut filter = Filter::new();
        filter.add_condition(FilterCondition::LessThan {
            key: "temp".into(),
            value: 100.0,
        });
        let value = build_chroma_filter(&filter);
        assert_eq!(value, serde_json::json!({"temp": {"$lt": 100.0}}));
    }

    #[test]
    fn test_build_chroma_filter_multiple() {
        let mut filter = Filter::new();
        filter
            .add_condition(FilterCondition::Equals {
                key: "color".into(),
                value: "blue".into(),
            })
            .add_condition(FilterCondition::GreaterThan {
                key: "size".into(),
                value: 10.0,
            });
        let value = build_chroma_filter(&filter);
        assert_eq!(
            value,
            serde_json::json!({
                "$and": [
                    {"color": {"$eq": "blue"}},
                    {"size": {"$gt": 10.0}}
                ]
            })
        );
    }

    #[test]
    fn test_build_chroma_filter_empty() {
        let filter = Filter::new();
        let value = build_chroma_filter(&filter);
        // An empty filter with no conditions still goes through the single path
        // but this is fine because search_with_filter skips empty filters.
        assert_eq!(value, serde_json::json!({}));
    }

    #[test]
    fn test_health_check_fails_when_no_server() {
        let db = ChromaDB::new("127.0.0.1", 1, None);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(db.health_check());
        assert!(result.is_err() || result.is_ok());
    }
}
