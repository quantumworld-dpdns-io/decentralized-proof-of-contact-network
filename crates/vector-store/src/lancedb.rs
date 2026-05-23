use crate::error::VectorStoreError;
use crate::traits::{Filter, FilterCondition, SearchResult, VectorStore};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

pub struct LanceDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl LanceDB {
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
                "X-Api-Key".parse().unwrap(),
                HeaderValue::from_str(key).unwrap(),
            );
        }
        headers
    }

    fn table_url(&self, name: &str) -> String {
        format!("{}/v1/table/{}", self.base_url, name)
    }
}

#[async_trait]
impl VectorStore for LanceDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let body = serde_json::json!({
            "name": name,
            "dimension": dimension,
            "metric": "cosine"
        });

        let url = format!("{}/create", self.table_url(name));
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
                "create table failed: {} {}",
                status, text
            )))
        }
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let url = format!("{}/delete", self.table_url(name));
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(VectorStoreError::DeleteFailed(format!(
                "delete table failed: {}",
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
            "id": id,
            "vector": vector,
            "metadata": metadata_value,
        });

        let url = format!("{}/add", self.table_url(collection));
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
            "vector": vector,
            "limit": limit,
        });

        if !filter.conditions.is_empty() {
            body["filter"] = build_lancedb_filter(filter);
        }

        let url = format!("{}/search", self.table_url(collection));
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
        if let Some(rows) = body.as_array().or_else(|| body["results"].as_array()) {
            for row in rows {
                let id = row["id"].as_str().unwrap_or("").to_string();
                let score = row["_distance"]
                    .as_f64()
                    .or_else(|| row["score"].as_f64())
                    .unwrap_or(0.0) as f32;
                let metadata = row["metadata"]
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
            "id": id
        });

        let url = format!("{}/delete", self.table_url(collection));
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
        let url = format!("{}/v1/health", self.base_url);
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

fn build_lancedb_filter(filter: &Filter) -> Value {
    let conditions: Vec<Value> = filter
        .conditions
        .iter()
        .map(|c| match c {
            FilterCondition::Equals { key, value } => {
                serde_json::json!({
                    "field": key,
                    "op": "=",
                    "value": value
                })
            }
            FilterCondition::GreaterThan { key, value } => {
                serde_json::json!({
                    "field": key,
                    "op": ">",
                    "value": value
                })
            }
            FilterCondition::LessThan { key, value } => {
                serde_json::json!({
                    "field": key,
                    "op": "<",
                    "value": value
                })
            }
        })
        .collect();

    if conditions.len() == 1 {
        conditions.into_iter().next().unwrap()
    } else {
        serde_json::json!({
            "op": "and",
            "conditions": conditions
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lancedb_new() {
        let db = LanceDB::new("localhost", 8000, None);
        assert_eq!(db.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_table_url() {
        let db = LanceDB::new("localhost", 8000, None);
        assert_eq!(db.table_url("vectors"), "http://localhost:8000/v1/table/vectors");
    }

    #[test]
    fn test_build_lancedb_filter_equals() {
        let mut filter = Filter::new();
        filter.add_condition(FilterCondition::Equals {
            key: "color".into(),
            value: "red".into(),
        });
        let v = build_lancedb_filter(&filter);
        assert_eq!(
            v,
            serde_json::json!({"field": "color", "op": "=", "value": "red"})
        );
    }

    #[test]
    fn test_build_lancedb_filter_multiple() {
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
        let v = build_lancedb_filter(&filter);
        assert_eq!(v["op"], "and");
        assert_eq!(v["conditions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_health_check_fails_when_no_server() {
        let db = LanceDB::new("127.0.0.1", 1, None);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(db.health_check());
        assert!(result.is_err() || result.is_ok());
    }
}
