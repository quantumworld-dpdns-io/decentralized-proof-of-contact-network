use chrono::{DateTime, Utc};
use poi_core::ContactProof;
use tokio::sync::Mutex;

use crate::arrow::ProofBatchBuilder;
use crate::config::AnalyticsConfig;
use crate::queries::QueryDefinition;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStage {
    Export,
    Sync,
    Compact,
    Analyze,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStatus {
    Idle,
    Running(PipelineStage),
    Completed,
    Failed(String),
}

#[derive(Debug)]
pub struct PipelineMetrics {
    pub proofs_exported: u64,
    pub proofs_synced: u64,
    pub files_compacted: u64,
    pub queries_executed: u64,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self {
            proofs_exported: 0,
            proofs_synced: 0,
            files_compacted: 0,
            queries_executed: 0,
            start_time: None,
            end_time: None,
        }
    }
}

pub struct AnalyticsPipeline {
    config: AnalyticsConfig,
    status: std::sync::Arc<Mutex<PipelineStatus>>,
    metrics: std::sync::Arc<Mutex<PipelineMetrics>>,
}

impl AnalyticsPipeline {
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            config,
            status: std::sync::Arc::new(Mutex::new(PipelineStatus::Idle)),
            metrics: std::sync::Arc::new(Mutex::new(PipelineMetrics::default())),
        }
    }

    pub fn config(&self) -> &AnalyticsConfig {
        &self.config
    }

    pub async fn status(&self) -> PipelineStatus {
        let s = self.status.lock().await;
        s.clone()
    }

    pub async fn metrics(&self) -> PipelineMetrics {
        let m = self.metrics.lock().await;
        PipelineMetrics {
            proofs_exported: m.proofs_exported,
            proofs_synced: m.proofs_synced,
            files_compacted: m.files_compacted,
            queries_executed: m.queries_executed,
            start_time: m.start_time,
            end_time: m.end_time,
        }
    }

    pub async fn run_export(&self, proofs: &[ContactProof], output_path: &str) -> Result<()> {
        *self.status.lock().await = PipelineStatus::Running(PipelineStage::Export);

        let mut builder = ProofBatchBuilder::with_capacity(proofs.len());
        for proof in proofs {
            builder.add_proof(proof);
        }
        let batch = builder.finish()?;

        let file = std::fs::File::create(output_path)?;
        let writer = parquet::arrow::ArrowWriter::try_new(file, batch.schema(), None)
            .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
        let mut writer = writer;
        writer
            .write(&batch)
            .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
        writer
            .close()
            .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;

        {
            let mut metrics = self.metrics.lock().await;
            metrics.proofs_exported += proofs.len() as u64;
        }

        tracing::info!("Exported {} proofs to {}", proofs.len(), output_path);
        *self.status.lock().await = PipelineStatus::Completed;
        Ok(())
    }

    pub async fn run_sync(
        &self,
        proofs: &[ContactProof],
        table_name: &str,
        location: &str,
    ) -> Result<()> {
        *self.status.lock().await = PipelineStatus::Running(PipelineStage::Sync);

        let data_path = format!("{}/{}.parquet", location, uuid::Uuid::new_v4());
        std::fs::create_dir_all(location)?;

        self.run_export(proofs, &data_path).await?;

        {
            let mut metrics = self.metrics.lock().await;
            metrics.proofs_synced += proofs.len() as u64;
        }

        tracing::info!(
            "Synced {} proofs to table '{}' at {}",
            proofs.len(),
            table_name,
            location
        );

        *self.status.lock().await = PipelineStatus::Completed;
        Ok(())
    }

    pub async fn run_compact(&self, location: &str) -> Result<()> {
        *self.status.lock().await = PipelineStatus::Running(PipelineStage::Compact);

        let data_dir = std::path::Path::new(location);
        if !data_dir.exists() {
            return Err(crate::AnalyticsError::Pipeline(format!(
                "Data location does not exist: {}",
                location
            )));
        }

        let mut parquet_files: Vec<_> = std::fs::read_dir(data_dir)
            .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?
            .filter_map(|entry| entry.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "parquet")
                    .unwrap_or(false)
            })
            .collect();

        parquet_files.sort_by_key(|e| e.path());

        if parquet_files.len() <= 1 {
            tracing::info!("Nothing to compact: {} parquet files", parquet_files.len());
            *self.status.lock().await = PipelineStatus::Completed;
            return Ok(());
        }

        let mut all_batches = Vec::new();
        for file in &parquet_files {
            let file_path = file.path();
            let file_reader = std::fs::File::open(&file_path)
                .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
            let reader =
                parquet::arrow::ParquetRecordBatchReaderBuilder::try_new(file_reader)
                    .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?
                    .build()
                    .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
            for batch in reader {
                let batch = batch.map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
                all_batches.push(batch);
            }
        }

        if all_batches.is_empty() {
            *self.status.lock().await = PipelineStatus::Completed;
            return Ok(());
        }

        let compacted_path =
            format!("{}/compacted-{}.parquet", location, uuid::Uuid::new_v4());
        let schema = all_batches[0].schema();
        let file = std::fs::File::create(&compacted_path)?;
        let writer = parquet::arrow::ArrowWriter::try_new(file, schema, None)
            .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
        let mut writer = writer;
        for batch in &all_batches {
            writer
                .write(batch)
                .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
        }
        writer
            .close()
            .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;

        for file in &parquet_files {
            std::fs::remove_file(file.path())
                .map_err(|e| crate::AnalyticsError::Pipeline(e.to_string()))?;
        }

        let files_removed = parquet_files.len() as u64;
        {
            let mut metrics = self.metrics.lock().await;
            metrics.files_compacted += files_removed;
        }

        tracing::info!(
            "Compacted {} parquet files into {}",
            parquet_files.len(),
            compacted_path
        );

        *self.status.lock().await = PipelineStatus::Completed;
        Ok(())
    }

    pub async fn run_analytics(
        &self,
        queries: &[QueryDefinition],
    ) -> Result<Vec<(String, String, String)>> {
        *self.status.lock().await = PipelineStatus::Running(PipelineStage::Analyze);

        let mut results = Vec::new();
        for query in queries {
            tracing::info!("Executing analytics query: {}", query.name);
            results.push((query.name.to_string(), query.description.to_string(), query.sql.clone()));
        }

        {
            let mut metrics = self.metrics.lock().await;
            metrics.queries_executed += queries.len() as u64;
        }

        *self.status.lock().await = PipelineStatus::Completed;
        Ok(results)
    }

    pub async fn run_all(
        &self,
        proofs: &[ContactProof],
        export_path: &str,
        sync_location: &str,
        queries: &[QueryDefinition],
    ) -> Result<PipelineMetrics> {
        {
            let mut metrics = self.metrics.lock().await;
            metrics.start_time = Some(Utc::now());
        }

        self.run_export(proofs, export_path).await?;
        self.run_sync(proofs, "proofs", sync_location).await?;
        self.run_compact(sync_location).await?;
        self.run_analytics(queries).await?;

        {
            let mut metrics = self.metrics.lock().await;
            metrics.end_time = Some(Utc::now());
        }

        Ok(self.metrics().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnalyticsConfig;
    use chrono::TimeZone;
    use poi_core::{
        NodeId, OrbitalWindow, ProofId, ProofMetadata, Signature, WindowType,
    };
    use uuid::Uuid;

    fn sample_proofs() -> Vec<ContactProof> {
        vec![ContactProof {
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
        }]
    }

    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = AnalyticsConfig::default();
        let pipeline = AnalyticsPipeline::new(config);
        assert_eq!(pipeline.status().await, PipelineStatus::Idle);
    }

    #[tokio::test]
    async fn test_run_export() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("export.parquet");
        let config = AnalyticsConfig::default();
        let pipeline = AnalyticsPipeline::new(config);
        pipeline
            .run_export(&sample_proofs(), path.to_str().unwrap())
            .await
            .unwrap();
        assert!(path.exists());
        let metrics = pipeline.metrics().await;
        assert_eq!(metrics.proofs_exported, 1);
    }

    #[tokio::test]
    async fn test_run_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let location = tmp.path().join("sync_loc");
        let config = AnalyticsConfig::default();
        let pipeline = AnalyticsPipeline::new(config);
        pipeline
            .run_sync(&sample_proofs(), "test_table", location.to_str().unwrap())
            .await
            .unwrap();
        let metrics = pipeline.metrics().await;
        assert_eq!(metrics.proofs_synced, 1);
    }

    #[tokio::test]
    async fn test_run_analytics() {
        let config = AnalyticsConfig::default();
        let pipeline = AnalyticsPipeline::new(config);
        let queries = crate::queries::predefined_queries();
        let results = pipeline.run_analytics(&queries).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_compact_no_files() {
        let tmp = tempfile::tempdir().unwrap();
        let config = AnalyticsConfig::default();
        let pipeline = AnalyticsPipeline::new(config);
        pipeline
            .run_compact(tmp.path().to_str().unwrap())
            .await
            .unwrap();
    }
}
