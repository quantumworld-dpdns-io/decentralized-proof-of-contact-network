pub mod config;
pub mod embedding;
pub mod error;
pub mod factory;
pub mod traits;

#[cfg(feature = "chroma")]
pub mod chroma;

#[cfg(feature = "qdrant")]
pub mod qdrant;

#[cfg(feature = "weaviate")]
pub mod weaviate;

#[cfg(feature = "lancedb")]
pub mod lancedb;

#[cfg(feature = "milvus")]
pub mod milvus;

pub use config::{VectorStoreConfig, VectorStoreProvider};
pub use error::VectorStoreError;
pub use factory::create_vector_store;
pub use traits::{Filter, FilterCondition, SearchResult, VectorStore};
