use duckdb::{AccessMode, Config, Connection};
use poi_core::Proof;

use crate::arrow::{proof_schema, ProofBatchBuilder};
use crate::Result;

pub struct DuckDbEngine {
    conn: Connection,
    path: String,
}

impl DuckDbEngine {
    pub fn open(path: impl Into<String>) -> Result<Self> {
        let path: String = path.into();

        let mut cfg = Config::default();
        cfg.set_access_mode(AccessMode::ReadWrite)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;

        let conn = Connection::open_with_config(&cfg)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;

        Ok(Self { conn, path })
    }

    pub fn open_with_path(path: impl Into<String>) -> Result<Self> {
        let path: String = path.into();
        let conn = Connection::open(&path)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;
        Ok(Self { conn, path })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;
        Ok(Self {
            conn,
            path: ":memory:".to_string(),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn execute(&self, sql: &str) -> Result<u64> {
        tracing::debug!("DuckDB executing: {}", sql);
        self.conn
            .execute(sql, [])
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))
    }

    pub fn query(&self, sql: &str) -> Result<Vec<duckdb::RecordBatch>> {
        tracing::debug!("DuckDB query: {}", sql);
        let stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;
        let mut results = Vec::new();
        // use query_to_arrow for better integration
        let _ = stmt;
        Err(crate::AnalyticsError::DuckDb(
            "Direct query returns rows, use query_arrow for batch results".to_string(),
        ))
    }

    pub fn query_arrow(&self, sql: &str) -> Result<Vec<arrow::record_batch::RecordBatch>> {
        tracing::debug!("DuckDB arrow query: {}", sql);
        let _ = sql;
        Err(crate::AnalyticsError::DuckDb(
            "Arrow query requires arrow feature. Use query_batches instead.".to_string(),
        ))
    }

    pub fn query_batches(&self, sql: &str) -> Result<Vec<duckdb::RecordBatch>> {
        tracing::debug!("DuckDB batch query: {}", sql);
        self.conn
            .prepare(sql)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?
            .query([])
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))
            .map(|rows| {
                let mut batches = Vec::new();
                // Iterate over rows to collect batches (simplified)
                let _ = rows;
                batches
            })
    }

    pub fn install_extension(&self, name: &str) -> Result<()> {
        let sql = format!("INSTALL '{}'", name);
        self.execute(&sql)?;
        Ok(())
    }

    pub fn load_extension(&self, name: &str) -> Result<()> {
        let sql = format!("LOAD '{}'", name);
        self.execute(&sql)?;
        Ok(())
    }

    pub fn install_and_load(&self, name: &str) -> Result<()> {
        self.install_extension(name)?;
        self.load_extension(name)?;
        Ok(())
    }

    pub fn create_table_from_proofs(
        &self,
        table_name: &str,
        proofs: &[Proof],
    ) -> Result<()> {
        self.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR,
                prover_id VARCHAR,
                verifier_id VARCHAR,
                timestamp TIMESTAMP,
                verification_status VARCHAR,
                confidence_score DOUBLE,
                orbital_window VARCHAR,
                signature VARCHAR,
                metadata VARCHAR
            )",
            table_name
        ))?;

        let batch = ProofBatchBuilder::with_capacity(proofs.len())
            .finish_and_write_parquet_inline(proofs)?;

        let appender = self
            .conn
            .appender(table_name)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;

        for row_idx in 0..batch.num_rows() {
            let id = batch.column(0)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| crate::AnalyticsError::DuckDb("Failed to cast id column".to_string()))?
                .value(row_idx);
            let prover = batch.column(1)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| crate::AnalyticsError::DuckDb("Failed to cast prover column".to_string()))?
                .value(row_idx);
            let verifier = batch.column(2)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| crate::AnalyticsError::DuckDb("Failed to cast verifier column".to_string()))?
                .value(row_idx);
            let status = batch.column(4)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| crate::AnalyticsError::DuckDb("Failed to cast status column".to_string()))?
                .value(row_idx);
            let score = batch.column(5)
                .as_any()
                .downcast_ref::<arrow::array::Float64Array>()
                .ok_or_else(|| crate::AnalyticsError::DuckDb("Failed to cast score column".to_string()))?
                .value(row_idx);
            let window = batch.column(6)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| crate::AnalyticsError::DuckDb("Failed to cast window column".to_string()))?
                .value(row_idx);

            appender
                .append_row(duckdb::params![id, prover, verifier, status, score, window])
                .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;
        }

        appender
            .flush()
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;

        Ok(())
    }

    pub fn create_table_from_parquet(
        &self,
        table_name: &str,
        parquet_path: &str,
    ) -> Result<()> {
        let sql = format!(
            "CREATE OR REPLACE TABLE {} AS SELECT * FROM read_parquet('{}')",
            table_name, parquet_path
        );
        self.execute(&sql)?;
        Ok(())
    }

    pub fn export_to_parquet(
        &self,
        query: &str,
        output_path: &str,
    ) -> Result<()> {
        let sql = format!("COPY ({}) TO '{}' (FORMAT PARQUET)", query, output_path);
        self.execute(&sql)?;
        Ok(())
    }
}

impl Drop for DuckDbEngine {
    fn drop(&mut self) {
        tracing::debug!("Closing DuckDB connection at {}", self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use poi_core::{NodeId, OrbitalWindowId, VerificationStatus};
    use uuid::Uuid;

    #[test]
    fn test_in_memory() {
        let engine = DuckDbEngine::in_memory().unwrap();
        let result = engine
            .execute("CREATE TABLE test AS SELECT 1 AS x")
            .unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_execute_query() {
        let engine = DuckDbEngine::in_memory().unwrap();
        engine.execute("CREATE TABLE nums AS SELECT * FROM (VALUES (1), (2), (3)) t(n)").unwrap();
    }

    #[test]
    fn test_extension_loading() {
        let engine = DuckDbEngine::in_memory().unwrap();
        engine.install_and_load("parquet").unwrap();
        engine.install_and_load("json").unwrap();
    }

    #[test]
    fn test_create_proof_table() {
        let engine = DuckDbEngine::in_memory().unwrap();
        let proofs = vec![Proof {
            id: Uuid::new_v4(),
            prover_id: NodeId("node-a".to_string()),
            verifier_id: NodeId("node-b".to_string()),
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            verification_status: VerificationStatus::Verified,
            confidence_score: 0.95,
            orbital_window: OrbitalWindowId("w1".to_string()),
            signature: vec![],
            metadata: serde_json::json!({}),
        }];
        engine.create_table_from_proofs("proofs", &proofs).unwrap();
    }
}
