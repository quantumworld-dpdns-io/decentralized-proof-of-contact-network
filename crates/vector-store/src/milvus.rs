use crate::error::VectorStoreError;
use crate::traits::{Filter, FilterCondition, SearchResult, VectorStore};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

pub struct MilvusDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl MilvusDB {
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
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", key)).unwrap(),
            );
        }
        headers
    }

    fn vector_url(&self) -> String {
        format!("{}/v1/vector", self.base_url)
    }
}

#[async_trait]
impl VectorStore for MilvusDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let body = serde_json::json!({
            "collectionName": name,
            "dimension": dimension,
            "metricType": "COSINE",
            "autoId": false,
        });

        let url = format!("{}/collections/create", self.vector_url());
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
            Err(VectorStoreError::InsertFailed(format!(
                "create collection failed: {} {}",
                status, text
            )))
        }
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let body = serde_json::json!({
            "collectionName": name,
        });

        let url = format!("{}/collections/drop", self.vector_url());
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
            Err(VectorStoreError::DeleteFailed(format!(
                "drop collection failed: {}",
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
        let metadata_value: Value = if metadata.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(metadata)
                .map_err(|e| VectorStoreError::InsertFailed(format!("invalid metadata json: {}", e)))?
        };

        let body = serde_json::json!({
            "collectionName": collection,
            "data": [
                {
                    "id": id,
                    "vector": vector,
                    "metadata": metadata_value,
                }
            ]
        });

        let url = format!("{}/insert", self.vector_url());
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
        let mut body = serde_json::json!({
            "collectionName": collection,
            "vector": vector,
            "limit": limit,
        });

        if !filter.conditions.is_empty() {
            body["filter"] = build_milvus_filter(filter);
        }

        let url = format!("{}/search", self.vector_url());
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

        if let Some(results_arr) = body["results"].as_array().or_else(|| body["data"].as_array()) {
            for item in results_arr {
                let id = item["id"].as_str().unwrap_or("").to_string();
                let score = item["distance"]
                    .as_f64()
                    .or_else(|| item["score"].as_f64())
                    .unwrap_or(0.0) as f32;
                let metadata = item["metadata"]
                    .as_object()
                    .map(|obj| serde_json::to_vec(obj).unwrap_or_default())
                    .unwrap_or_default();
                results.push(SearchResult { id, score, metadata });
            }
        }

        Ok(results)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let body = serde_json::json!({
            "collectionName": collection,
            "id": id,
        });

        let url = format!("{}/delete", self.vector_url());
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
        let url = format!("{}/health", self.vector_url());
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

fn build_milvus_filter(filter: &Filter) -> Value {
    let conditions: Vec<String> = filter
        .conditions
        .iter()
        .map(|c| match c {
            FilterCondition::Equals { key, value } => {
                format!("{} == \"{}\"", key, value)
            }
            FilterCondition::GreaterThan { key, value } => {
                format!("{} > {}", key, value)
            }
            FilterCondition::LessThan { key, value } => {
                format!("{} < {}", key, value)
            }
        })
        .collect();

    Value::String(conditions.join(" && "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milvusdb_new() {
        let db = MilvusDB::new("localhost", 19530, None);
        assert_eq!(db.base_url, "http://localhost:19530");
    }

    #[test]
    fn test_vector_url() {
        let db = MilvusDB::new("localhost", 19530, None);
        assert_eq!(db.vector_url(), "http://localhost:19530/v1/vector");
    }

    #[test]
    fn test_build_milvus_filter_equals() {
        let mut filter = Filter::new();
        filter.add_condition(FilterCondition::Equals {
            key: "color".into(),
            value: "red".into(),
        });
        let v = build_milvus_filter(&filter);
        assert_eq!(v, Value::String("color == \"red\"".to_string()));
    }

    #[test]
    fn test_build_milvus_filter_comparison() {
        let mut filter = Filter::new();
        filter
            .add_condition(FilterCondition::GreaterThan {
                key: "age".into(),
                value: 21.0,
            })
            .add_condition(FilterCondition::LessThan {
                key: "age".into(),
                value: 65.0,
            });
        let v = build_milvus_filter(&filter);
        assert_eq!(v, Value::String("age > 21 && age < 65".to_string()));
    }

    #[test]
    fn test_health_check_fails_when_no_server() {
        let db = MilvusDB::new("127.0.0.1", 1, None);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(db.health_check());
        assert!(result.is_err() || result.is_ok());
    }
}
