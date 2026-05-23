pub mod arrow;
pub mod config;
pub mod datafusion;
pub mod duckdb;
pub mod error;
pub mod iceberg;
pub mod pipeline;
pub mod queries;

pub use config::AnalyticsConfig;
pub use error::AnalyticsError;

pub type Result<T> = std::result::Result<T, AnalyticsError>;
