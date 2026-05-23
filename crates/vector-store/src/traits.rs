use crate::error::VectorStoreError;
use async_trait::async_trait;

pub type Result<T> = std::result::Result<T, VectorStoreError>;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()>;

    async fn delete_collection(&self, name: &str) -> Result<()>;

    async fn insert(
        &self,
        collection: &str,
        id: &str,
        vector: &[f32],
        metadata: &[u8],
    ) -> Result<()>;

    async fn search(
        &self,
        collection: &str,
        vector: &[f32],
        limit: u64,
    ) -> Result<Vec<SearchResult>>;

    async fn search_with_filter(
        &self,
        collection: &str,
        vector: &[f32],
        filter: &Filter,
        limit: u64,
    ) -> Result<Vec<SearchResult>>;

    async fn delete(&self, collection: &str, id: &str) -> Result<()>;

    async fn health_check(&self) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub conditions: Vec<FilterCondition>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn add_condition(&mut self, condition: FilterCondition) -> &mut Self {
        self.conditions.push(condition);
        self
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum FilterCondition {
    Equals {
        key: String,
        value: String,
    },
    GreaterThan {
        key: String,
        value: f64,
    },
    LessThan {
        key: String,
        value: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            id: "vec1".to_string(),
            score: 0.95,
            metadata: br#"{"key":"value"}"#.to_vec(),
        };
        assert_eq!(result.id, "vec1");
        assert_eq!(result.score, 0.95);
        assert_eq!(result.metadata, br#"{"key":"value"}"#);
    }

    #[test]
    fn test_filter_new_is_empty() {
        let filter = Filter::new();
        assert!(filter.conditions.is_empty());
    }

    #[test]
    fn test_filter_add_conditions() {
        let mut filter = Filter::new();
        filter
            .add_condition(FilterCondition::Equals {
                key: "color".into(),
                value: "red".into(),
            })
            .add_condition(FilterCondition::GreaterThan {
                key: "score".into(),
                value: 0.5,
            });

        assert_eq!(filter.conditions.len(), 2);
        match &filter.conditions[0] {
            FilterCondition::Equals { key, value } => {
                assert_eq!(key, "color");
                assert_eq!(value, "red");
            }
            _ => panic!("expected Equals"),
        }
    }

    #[test]
    fn test_filter_default() {
        let filter: Filter = Default::default();
        assert!(filter.conditions.is_empty());
    }

    #[test]
    fn test_filter_condition_debug() {
        let cond = FilterCondition::LessThan {
            key: "temp".into(),
            value: 100.0,
        };
        let debug = format!("{:?}", cond);
        assert!(debug.contains("LessThan"));
        assert!(debug.contains("temp"));
        assert!(debug.contains("100.0"));
    }
}
