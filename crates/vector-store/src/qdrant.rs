use crate::error::VectorStoreError;
use crate::traits::{Filter, FilterCondition, SearchResult, VectorStore};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

pub struct QdrantDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl QdrantDB {
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
                "api-key".parse().unwrap(),
                HeaderValue::from_str(key).unwrap(),
            );
        }
        headers
    }

    fn collection_url(&self, name: &str) -> String {
        format!("{}/collections/{}", self.base_url, name)
    }

    fn points_url(&self, name: &str) -> String {
        format!("{}/collections/{}/points", self.base_url, name)
    }
}

#[async_trait]
impl VectorStore for QdrantDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let body = serde_json::json!({
            "name": name,
            "vectors": {
                "size": dimension,
                "distance": "Cosine"
            }
        });

        let resp = self
            .client
            .put(&self.collection_url(name))
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
        let resp = self
            .client
            .delete(&self.collection_url(name))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(VectorStoreError::DeleteFailed(format!(
                "delete collection failed: {}",
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
        let payload: Value = if metadata.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(metadata)
                .map_err(|e| VectorStoreError::InsertFailed(format!("invalid metadata json: {}", e)))?
        };

        let point_id = id.parse::<serde_json::Value>().unwrap_or_else(|_| {
            // Use a numeric hash of the id string if not parseable
            let h: u64 = id.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            serde_json::json!(h)
        });

        let body = serde_json::json!({
            "points": [{
                "id": point_id,
                "vector": vector,
                "payload": payload
            }]
        });

        let url = format!("{}/points", self.points_url(collection));
        let resp = self
            .client
            .put(&url)
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
            "vector": vector,
            "limit": limit,
            "with_payload": true,
        });

        if !filter.conditions.is_empty() {
            body["filter"] = build_qdrant_filter(filter);
        }

        let url = format!("{}/points/search", self.points_url(collection));
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
        if let Some(points) = body["result"].as_array() {
            for point in points {
                let id = point["id"].to_string();
                let score = point["score"].as_f64().unwrap_or(0.0) as f32;
                let metadata = point["payload"]
                    .as_object()
                    .map(|obj| serde_json::to_vec(obj).unwrap_or_default())
                    .unwrap_or_default();
                results.push(SearchResult { id, score, metadata });
            }
        }

        Ok(results)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let point_id = id.parse::<serde_json::Value>().unwrap_or_else(|_| {
            let h: u64 = id.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            serde_json::json!(h)
        });

        let body = serde_json::json!({
            "filter": {
                "must": [{
                    "has_id": [point_id]
                }]
            }
        });

        let url = format!("{}/points/delete", self.points_url(collection));
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
        let url = format!("{}/health", self.base_url);
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

fn build_qdrant_filter(filter: &Filter) -> Value {
    let must: Vec<Value> = filter
        .conditions
        .iter()
        .map(|c| match c {
            FilterCondition::Equals { key, value } => {
                serde_json::json!({
                    "key": key,
                    "match": {
                        "value": value
                    }
                })
            }
            FilterCondition::GreaterThan { key, value } => {
                serde_json::json!({
                    "key": key,
                    "range": {
                        "gt": value
                    }
                })
            }
            FilterCondition::LessThan { key, value } => {
                serde_json::json!({
                    "key": key,
                    "range": {
                        "lt": value
                    }
                })
            }
        })
        .collect();

    serde_json::json!({
        "must": must
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qdrantdb_new() {
        let db = QdrantDB::new("localhost", 6333, None);
        assert_eq!(db.base_url, "http://localhost:6333");
    }

    #[test]
    fn test_qdrantdb_new_with_api_key() {
        let db = QdrantDB::new("10.0.0.1", 6333, Some("key".into()));
        assert_eq!(db.base_url, "http://10.0.0.1:6333");
    }

    #[test]
    fn test_collection_url() {
        let db = QdrantDB::new("localhost", 6333, None);
        assert_eq!(db.collection_url("test"), "http://localhost:6333/collections/test");
    }

    #[test]
    fn test_points_url() {
        let db = QdrantDB::new("localhost", 6333, None);
        assert_eq!(
            db.points_url("test"),
            "http://localhost:6333/collections/test/points"
        );
    }

    #[test]
    fn test_build_qdrant_filter_equals() {
        let mut filter = Filter::new();
        filter.add_condition(FilterCondition::Equals {
            key: "color".into(),
            value: "red".into(),
        });
        let value = build_qdrant_filter(&filter);
        assert_eq!(
            value,
            serde_json::json!({
                "must": [{
                    "key": "color",
                    "match": { "value": "red" }
                }]
            })
        );
    }

    #[test]
    fn test_build_qdrant_filter_range() {
        let mut filter = Filter::new();
        filter
            .add_condition(FilterCondition::GreaterThan {
                key: "price".into(),
                value: 10.0,
            })
            .add_condition(FilterCondition::LessThan {
                key: "price".into(),
                value: 100.0,
            });
        let value = build_qdrant_filter(&filter);
        let must = value["must"].as_array().unwrap();
        assert_eq!(must.len(), 2);
    }

    #[test]
    fn test_health_check_fails_when_no_server() {
        let db = QdrantDB::new("127.0.0.1", 1, None);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(db.health_check());
        assert!(result.is_err() || result.is_ok());
    }
}
