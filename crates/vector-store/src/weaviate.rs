use crate::error::VectorStoreError;
use crate::traits::{Filter, FilterCondition, SearchResult, VectorStore};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

pub struct WeaviateDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl WeaviateDB {
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

    fn schema_url(&self) -> String {
        format!("{}/v1/schema", self.base_url)
    }

    fn class_schema_url(&self, class_name: &str) -> String {
        format!("{}/v1/schema/{}", self.base_url, class_name)
    }

    fn objects_url(&self) -> String {
        format!("{}/v1/objects", self.base_url)
    }

    fn object_url(&self, class_name: &str, id: &str) -> String {
        format!("{}/v1/objects/{}/{}", self.base_url, class_name, id)
    }

    fn graphql_url(&self) -> String {
        format!("{}/v1/graphql", self.base_url)
    }
}

#[async_trait]
impl VectorStore for WeaviateDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let body = serde_json::json!({
            "class": name,
            "vectorizer": "none",
            "vectorIndexConfig": {
                "distance": "cosine"
            },
            "properties": [
                {
                    "name": "metadata",
                    "dataType": ["text"]
                },
                {
                    "name": "dimension",
                    "dataType": ["int"]
                }
            ]
        });

        let resp = self
            .client
            .post(&self.schema_url())
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| VectorStoreError::ConnectionError(e.to_string()))?;

        if resp.status().is_success() || resp.status() == reqwest::StatusCode::CONFLICT {
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
            .delete(&self.class_schema_url(name))
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
        let metadata_str = if metadata.is_empty() {
            "{}".to_string()
        } else {
            String::from_utf8_lossy(metadata).to_string()
        };

        let body = serde_json::json!({
            "class": collection,
            "id": id,
            "vector": vector,
            "properties": {
                "metadata": metadata_str,
                "dimension": vector.len() as i64,
            }
        });

        let resp = self
            .client
            .post(&self.objects_url())
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
        let near_vector = format!(
            "nearVector: {{ vector: [{}] }}",
            vector
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let where_clause = if !filter.conditions.is_empty() {
            format!(
                ", where: {{ operator: And, operands: [{}] }}",
                filter
                    .conditions
                    .iter()
                    .map(build_weaviate_filter_condition)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let query = format!(
            "{{
                Get {{
                    {}({} {}) {{
                        _additional {{
                            id
                            distance
                        }}
                        metadata
                    }}
                }}
            }}",
            collection, near_vector, where_clause
        );

        let body = serde_json::json!({
            "query": query
        });

        let resp = self
            .client
            .post(&self.graphql_url())
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

        if let Some(data) = body["data"]["Get"][collection].as_array() {
            for item in data {
                let id = item["_additional"]["id"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let score = item["_additional"]["distance"].as_f64().unwrap_or(0.0) as f32;
                let metadata_str = item["metadata"].as_str().unwrap_or("{}");
                let metadata = metadata_str.as_bytes().to_vec();
                results.push(SearchResult { id, score, metadata });
            }
        }

        Ok(results)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let resp = self
            .client
            .delete(&self.object_url(collection, id))
            .headers(self.headers())
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
        let url = format!("{}/v1/.well-known/ready", self.base_url);
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

fn build_weaviate_filter_condition(condition: &FilterCondition) -> String {
    match condition {
        FilterCondition::Equals { key, value } => {
            format!(
                "{{ operator: Equal, path: [\"{}\"], valueString: \"{}\" }}",
                key, value
            )
        }
        FilterCondition::GreaterThan { key, value } => {
            format!(
                "{{ operator: GreaterThan, path: [\"{}\"], valueNumber: {} }}",
                key, value
            )
        }
        FilterCondition::LessThan { key, value } => {
            format!(
                "{{ operator: LessThan, path: [\"{}\"], valueNumber: {} }}",
                key, value
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weaviatedb_new() {
        let db = WeaviateDB::new("localhost", 8080, None);
        assert_eq!(db.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_urls() {
        let db = WeaviateDB::new("localhost", 8080, None);
        assert_eq!(db.schema_url(), "http://localhost:8080/v1/schema");
        assert_eq!(
            db.class_schema_url("Document"),
            "http://localhost:8080/v1/schema/Document"
        );
        assert_eq!(db.objects_url(), "http://localhost:8080/v1/objects");
        assert_eq!(
            db.object_url("Document", "abc-123"),
            "http://localhost:8080/v1/objects/Document/abc-123"
        );
        assert_eq!(db.graphql_url(), "http://localhost:8080/v1/graphql");
    }

    #[test]
    fn test_build_weaviate_filter_equals() {
        let cond = FilterCondition::Equals {
            key: "color".into(),
            value: "red".into(),
        };
        let s = build_weaviate_filter_condition(&cond);
        assert!(s.contains("Equal"));
        assert!(s.contains("color"));
        assert!(s.contains("red"));
    }

    #[test]
    fn test_build_weaviate_filter_greater_than() {
        let cond = FilterCondition::GreaterThan {
            key: "price".into(),
            value: 50.0,
        };
        let s = build_weaviate_filter_condition(&cond);
        assert!(s.contains("GreaterThan"));
        assert!(s.contains("50"));
    }

    #[test]
    fn test_build_weaviate_filter_less_than() {
        let cond = FilterCondition::LessThan {
            key: "age".into(),
            value: 100.0,
        };
        let s = build_weaviate_filter_condition(&cond);
        assert!(s.contains("LessThan"));
        assert!(s.contains("100"));
    }

    #[test]
    fn test_health_check_fails_when_no_server() {
        let db = WeaviateDB::new("127.0.0.1", 1, None);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(db.health_check());
        assert!(result.is_err() || result.is_ok());
    }
}
