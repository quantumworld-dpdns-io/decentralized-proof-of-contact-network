use duckdb::{AccessMode, Config, Connection};
use poi_core::ContactProof;

use crate::arrow::ProofBatchBuilder;
use crate::Result;

pub struct DuckDbEngine {
    conn: Connection,
    path: String,
}

impl DuckDbEngine {
    pub fn open(path: impl Into<String>) -> Result<Self> {
        let path: String = path.into();
        let config = Config::default()
            .access_mode(AccessMode::ReadWrite)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;
        let conn = Connection::open_with_flags(&path, config)
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

    pub fn execute(&self, sql: &str) -> Result<usize> {
        tracing::debug!("DuckDB executing: {}", sql);
        self.conn
            .execute(sql, [])
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))
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

    pub fn create_table_from_proofs(&self, table_name: &str, proofs: &[ContactProof]) -> Result<()> {
        self.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR,
                proving_node VARCHAR,
                target_node VARCHAR,
                timestamp VARCHAR,
                window_id VARCHAR,
                verification_status VARCHAR,
                confidence_score DOUBLE,
                protocol_version VARCHAR,
                proof_purpose VARCHAR,
                signature VARCHAR
            )",
            table_name
        ))?;

        for p in proofs {
            let timestamp_str = p.timestamp.to_rfc3339();
            let status = if p.signature.0.is_empty() { "pending" } else if p.metadata.confidence_score >= 0.5 { "verified" } else { "failed" };
            self.execute(&format!(
                "INSERT INTO {} VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}')",
                table_name,
                p.id.0,
                p.proving_node.0,
                p.target_node.0,
                timestamp_str,
                p.orbital_window.id,
                status,
                p.metadata.confidence_score,
                p.metadata.protocol_version,
                p.metadata.proof_purpose,
                p.signature.0,
            ))?;
        }

        Ok(())
    }

    pub fn create_table_from_parquet(&self, table_name: &str, parquet_path: &str) -> Result<()> {
        let sql = format!(
            "CREATE OR REPLACE TABLE {} AS SELECT * FROM read_parquet('{}')",
            table_name, parquet_path
        );
        self.execute(&sql)?;
        Ok(())
    }

    pub fn export_to_parquet(&self, query: &str, output_path: &str) -> Result<()> {
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
    use chrono::{TimeZone, Utc};
    use poi_core::{
        NodeId, OrbitalWindow, ProofId, ProofMetadata, Signature, WindowType,
    };
    use uuid::Uuid;

    #[test]
    fn test_in_memory() {
        let engine = DuckDbEngine::in_memory().unwrap();
        let result = engine.execute("CREATE TABLE test (x INTEGER)");
        assert!(result.is_ok());
        let result = engine.execute("INSERT INTO test VALUES (1)").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_extension_loading() {
        let engine = DuckDbEngine::in_memory().unwrap();
        engine.install_and_load("parquet").unwrap();
    }

    #[test]
    fn test_create_proof_table() {
        let engine = DuckDbEngine::in_memory().unwrap();
        // First create a simple table
        engine.execute("CREATE TABLE test_proofs (id VARCHAR, node VARCHAR, score DOUBLE)").unwrap();
        engine.execute("INSERT INTO test_proofs VALUES ('p1', 'node-a', 0.95)").unwrap();
        let proofs = engine.execute("SELECT count(*) FROM test_proofs").unwrap();
        assert_eq!(proofs, 1);
    }
}
