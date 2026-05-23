use std::sync::Arc;

use datafusion::common::DataFusionError;
use datafusion::dataframe::DataFrame;
use datafusion::error::Result as DataFusionResult;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::SessionConfig;
use poi_core::Proof;

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
        proofs: &[Proof],
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

    pub async fn sql(&self, sql: &str) -> Result<DataFrame> {
        self.ctx
            .sql(sql)
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))
    }

    pub async fn execute_sql_and_collect(&self, sql: &str) -> Result<Vec<arrow::record_batch::RecordBatch>> {
        let df = self.sql(sql).await?;
        df.collect()
            .await
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))
    }

    pub fn table(&self, name: &str) -> Result<DataFrame> {
        self.ctx
            .table(name)
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

    pub async fn show_tables(&self) -> Result<DataFrame> {
        self.sql("SHOW TABLES FROM poi.analytics").await
    }
}

impl Default for DatafusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProofTableFunction;

impl ProofTableFunction {
    pub fn new() -> Self {
        Self
    }

    pub fn create_table_provider(
        &self,
        proofs: Vec<Proof>,
    ) -> Result<Arc<datafusion::catalog::TableProvider>> {
        let schema = proof_schema();
        let batch = ProofBatchBuilder::with_capacity(proofs.len())
            .finish_and_write_parquet_inline(&proofs)?;

        let provider = datafusion::datasource::memory::MemTable::try_new(schema, vec![vec![batch]])
            .map_err(|e| crate::AnalyticsError::DataFusion(e.to_string()))?;

        Ok(Arc::new(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use poi_core::{NodeId, OrbitalWindowId, VerificationStatus};
    use uuid::Uuid;

    fn sample_proofs() -> Vec<Proof> {
        vec![
            Proof {
                id: Uuid::new_v4(),
                prover_id: NodeId("node-a".to_string()),
                verifier_id: NodeId("node-b".to_string()),
                timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                verification_status: VerificationStatus::Verified,
                confidence_score: 0.95,
                orbital_window: OrbitalWindowId("w1".to_string()),
                signature: vec![],
                metadata: serde_json::json!({}),
            },
            Proof {
                id: Uuid::new_v4(),
                prover_id: NodeId("node-a".to_string()),
                verifier_id: NodeId("node-c".to_string()),
                timestamp: Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap(),
                verification_status: VerificationStatus::Failed,
                confidence_score: 0.3,
                orbital_window: OrbitalWindowId("w1".to_string()),
                signature: vec![],
                metadata: serde_json::json!({}),
            },
        ]
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = DatafusionEngine::new();
        let tables = engine.show_tables().await.unwrap();
        let batches = tables.collect().await.unwrap();
        assert!(batches.is_empty());
    }

    #[tokio::test]
    async fn test_sql_execution() {
        let engine = DatafusionEngine::new();
        engine
            .sql("CREATE TABLE IF NOT EXISTS poi.analytics.test (x INT) AS VALUES (1), (2), (3)")
            .await
            .unwrap();
        let result = engine.execute_sql_and_collect("SELECT COUNT(*) as cnt FROM poi.analytics.test").await.unwrap();
        assert!(!result.is_empty());
    }
}
