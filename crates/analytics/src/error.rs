use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query execution error: {0}")]
    Query(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Arrow error: {0}")]
    Arrow(String),

    #[error("DuckDB error: {0}")]
    DuckDb(String),

    #[error("DataFusion error: {0}")]
    DataFusion(String),

    #[error("Iceberg error: {0}")]
    Iceberg(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;
