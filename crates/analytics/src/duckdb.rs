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
                timestamp TIMESTAMP,
                window_id VARCHAR,
                verification_status VARCHAR,
                confidence_score DOUBLE,
                protocol_version VARCHAR,
                proof_purpose VARCHAR,
                chain_position BIGINT,
                signature VARCHAR
            )",
            table_name
        ))?;

        let mut builder = ProofBatchBuilder::with_capacity(proofs.len());
        for p in proofs {
            builder.add_proof(p);
        }
        let batch = builder.finish()?;

        let mut appender = self
            .conn
            .appender(table_name)
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;

        for row_idx in 0..batch.num_rows() {
            let id = batch
                .column(0)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| {
                    crate::AnalyticsError::DuckDb("Failed to cast id column".to_string())
                })?
                .value(row_idx);
            let proving = batch
                .column(1)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| {
                    crate::AnalyticsError::DuckDb("Failed to cast proving column".to_string())
                })?
                .value(row_idx);
            let target = batch
                .column(2)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| {
                    crate::AnalyticsError::DuckDb("Failed to cast target column".to_string())
                })?
                .value(row_idx);
            let status = batch
                .column(5)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .map(|arr| arr.value(row_idx))
                .unwrap_or("pending");
            let score = batch
                .column(6)
                .as_any()
                .downcast_ref::<arrow::array::Float64Array>()
                .ok_or_else(|| {
                    crate::AnalyticsError::DuckDb("Failed to cast score column".to_string())
                })?
                .value(row_idx);
            let window = batch
                .column(4)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .ok_or_else(|| {
                    crate::AnalyticsError::DuckDb("Failed to cast window column".to_string())
                })?
                .value(row_idx);

            appender
                .append_row(duckdb::params![id, proving, target, status, score, window])
                .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;
        }

        appender
            .flush()
            .map_err(|e| crate::AnalyticsError::DuckDb(e.to_string()))?;

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
    use chrono::TimeZone;
    use poi_core::{
        NodeId, OrbitalWindow, ProofId, ProofMetadata, Signature, WindowType,
    };
    use uuid::Uuid;

    #[test]
    fn test_in_memory() {
        let engine = DuckDbEngine::in_memory().unwrap();
        let result = engine.execute("CREATE TABLE test AS SELECT 1 AS x").unwrap();
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
        let proofs = vec![ContactProof {
            id: ProofId(Uuid::new_v4()),
            proving_node: NodeId("node-a".to_string()),
            target_node: NodeId("node-b".to_string()),
            orbital_window: OrbitalWindow {
                id: Uuid::new_v4(),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                end_time: Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap(),
                window_type: WindowType::Standard,
            },
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            signature: Signature("sig".to_string()),
            pqc_signature: None,
            metadata: ProofMetadata {
                protocol_version: "1.0".to_string(),
                chain_position: None,
                confidence_score: 0.95,
                proof_purpose: "test".to_string(),
            },
        }];
        engine.create_table_from_proofs("proofs", &proofs).unwrap();
    }
}
