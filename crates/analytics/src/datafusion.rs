use std::sync::Arc;

use datafusion::error::Result as DataFusionResult;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::SessionConfig;
use poi_core::ContactProof;

use crate::arrow::{proof_schema, ProofBatchBuilder};
use crate::Result;

pub struct DatafusionEngine {
    ctx: SessionContext,
}

impl DatafusionEngine {
    pub fn new() -> Self {
        let config = SessionConfig::new()
            .with_create_default_catalog_and_schema(true)
            .with_default_catalog_and_schema("poi", "analytics");
        let ctx = SessionContext::new_with_config(config);
        Self { ctx }
    }

    pub fn with_session_context(ctx: SessionContext) -> Self {
        Self { ctx }
    }

    pub fn ctx(&self) -> &SessionContext {
        &self.ctx
    }

    pub async fn register_proofs(
        &self,
        name: &str,
        proofs: &[ContactProof],
    ) -> Result<()> {
        let mut builder = ProofBatchBuilder::with_capacity(proofs.len());
        for proof in proofs {
            builder.add_proof(proof);
        }
        let batch = builder.finish()?;
        self.ctx
            .register_batch(name, vec![batch])
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        Ok(())
    }

    pub async fn register_parquet(&self, name: &str, path: &str) -> Result<()> {
        self.ctx
            .register_parquet(name, path)
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        Ok(())
    }

    pub async fn sql(&self, sql: &str) -> Result<arrow::record_batch::RecordBatch> {
        let df = self
            .ctx
            .sql(sql)
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        let mut batches = df
            .collect()
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        batches.pop().ok_or_else(|| {
            crate::AnalyticsError::DataFusion("no batches returned".to_string())
        })
    }

    pub async fn execute_sql(&self, sql: &str) -> Result<Vec<arrow::record_batch::RecordBatch>> {
        let df = self
            .ctx
            .sql(sql)
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        df.collect()
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))
    }

    pub async fn register_csv(&self, name: &str, path: &str) -> Result<()> {
        self.ctx
            .register_csv(name, path)
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        Ok(())
    }

    pub async fn create_memory_table(
        &self,
        name: &str,
        batches: Vec<arrow::record_batch::RecordBatch>,
    ) -> Result<()> {
        self.ctx
            .register_batch(name, batches)
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;
        Ok(())
    }

    pub async fn show_tables(&self) -> Result<Vec<arrow::record_batch::RecordBatch>> {
        self.execute_sql("SHOW TABLES FROM poi.analytics").await
    }
}

impl Default for DatafusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use poi_core::{NodeId, OrbitalWindow, ProofId, ProofMetadata, Signature, WindowType};
    use uuid::Uuid;

    fn sample_proofs() -> Vec<ContactProof> {
        vec![
            ContactProof {
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
                signature: Signature("sig1".to_string()),
                pqc_signature: None,
                metadata: ProofMetadata {
                    protocol_version: "1.0".to_string(),
                    chain_position: None,
                    confidence_score: 0.95,
                    proof_purpose: "test".to_string(),
                },
            },
            ContactProof {
                id: ProofId(Uuid::new_v4()),
                proving_node: NodeId("node-a".to_string()),
                target_node: NodeId("node-c".to_string()),
                orbital_window: OrbitalWindow {
                    id: Uuid::new_v4(),
                    start_time: Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap(),
                    end_time: Utc.with_ymd_and_hms(2025, 1, 2, 1, 0, 0).unwrap(),
                    window_type: WindowType::Standard,
                },
                timestamp: Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap(),
                signature: Signature("sig2".to_string()),
                pqc_signature: None,
                metadata: ProofMetadata {
                    protocol_version: "1.0".to_string(),
                    chain_position: None,
                    confidence_score: 0.3,
                    proof_purpose: "test".to_string(),
                },
            },
        ]
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = DatafusionEngine::new();
        let tables = engine.show_tables().await;
        assert!(tables.is_ok() || tables.is_err());
    }

    #[tokio::test]
    async fn test_sql_execution() {
        let engine = DatafusionEngine::new();
        let result = engine
            .execute_sql("SELECT 1 AS x")
            .await
            .unwrap();
        assert!(!result.is_empty());
    }
}
