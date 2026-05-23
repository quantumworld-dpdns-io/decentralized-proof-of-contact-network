pub mod error;
pub mod config;

#[cfg(feature = "arrow")]
pub mod arrow;

#[cfg(feature = "datafusion")]
pub mod datafusion;

#[cfg(feature = "duckdb")]
pub mod duckdb;

#[cfg(feature = "iceberg")]
pub mod iceberg;

pub mod queries;
pub mod pipeline;

pub use error::AnalyticsError;
pub use config::AnalyticsConfig;

pub type Result<T> = std::result::Result<T, AnalyticsError>;
