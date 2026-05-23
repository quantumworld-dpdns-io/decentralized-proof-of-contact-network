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

pub use error::{AnalyticsError, Result};
pub use config::AnalyticsConfig;

#[cfg(feature = "arrow")]
pub use arrow::*;

#[cfg(feature = "datafusion")]
pub use datafusion::*;

#[cfg(feature = "duckdb")]
pub use duckdb::*;

#[cfg(feature = "iceberg")]
pub use iceberg::*;

pub use queries::*;
pub use pipeline::*;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
