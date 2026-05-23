use std::collections::HashMap;

use chrono::{DateTime, Utc};
use poi_core::ContactProof;

use crate::arrow::ProofBatchBuilder;
use crate::Result;

#[derive(Debug, Clone)]
pub struct IcebergTableConfig {
    pub name: String,
    pub location: String,
    pub partition_by: Vec<String>,
    pub properties: HashMap<String, String>,
}

impl Default for IcebergTableConfig {
    fn default() -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "write.format.default".to_string(),
            "parquet".to_string(),
        );
        properties.insert(
            "write.target-file-size-bytes".to_string(),
            "134217728".to_string(),
        );

        Self {
            name: "proofs".to_string(),
            location: "/tmp/poi/warehouse/proofs".to_string(),
            partition_by: vec!["verification_status".to_string()],
            properties,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub parent_id: Option<i64>,
    pub operation: String,
}

#[derive(Debug)]
pub struct IcebergEngine {
    warehouse_path: String,
    tables: HashMap<String, IcebergTableConfig>,
}

impl IcebergEngine {
    pub fn new(warehouse_path: impl Into<String>) -> Self {
        Self {
            warehouse_path: warehouse_path.into(),
            tables: HashMap::new(),
        }
    }

    pub fn warehouse_path(&self) -> &str {
        &self.warehouse_path
    }

    pub fn register_table(&mut self, config: IcebergTableConfig) {
        self.tables.insert(config.name.clone(), config);
    }

    pub fn get_table(&self, name: &str) -> Option<&IcebergTableConfig> {
        self.tables.get(name)
    }

    pub fn create_table(&self, config: &IcebergTableConfig) -> Result<()> {
        std::fs::create_dir_all(&config.location)
            .map_err(|e| crate::AnalyticsError::Iceberg(e.to_string()))?;

        let metadata_path = format!("{}/metadata", config.location);
        std::fs::create_dir_all(&metadata_path)
            .map_err(|e| crate::AnalyticsError::Iceberg(e.to_string()))?;

        let metadata = serde_json::json!({
            "format-version": 2,
            "table-uuid": uuid::Uuid::new_v4().to_string(),
            "location": config.location,
            "last-sequence-number": 0,
            "last-updated-ms": Utc::now().timestamp_millis(),
            "partition-spec": [],
            "properties": config.properties,
            "snapshots": [],
            "snapshot-log": [],
            "metadata-log": [],
        });

        let metadata_file = format!("{}/v0.metadata.json", metadata_path);
        std::fs::write(
            &metadata_file,
            serde_json::to_string_pretty(&metadata)
                .map_err(|e| crate::AnalyticsError::Serialization(e.to_string()))?,
        )
        .map_err(|e| crate::AnalyticsError::Iceberg(e.to_string()))?;

        tracing::info!("Created Iceberg table at {}", config.location);
        Ok(())
    }

    pub fn append_proofs(
        &self,
        table: &str,
        proofs: &[ContactProof],
    ) -> Result<Snapshot> {
        let config = self.tables.get(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;

        let data_path = format!(
            "{}/data/{}.parquet",
            config.location,
            uuid::Uuid::new_v4()
        );

        let data_dir = format!("{}/data", config.location);
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| crate::AnalyticsError::Iceberg(e.to_string()))?;

        let mut builder = ProofBatchBuilder::with_capacity(proofs.len());
        for p in proofs {
            builder.add_proof(p);
        }
        builder.write_parquet(&data_path)?;

        let snapshot = Snapshot {
            id: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            timestamp: Utc::now(),
            parent_id: None,
            operation: "append".to_string(),
        };

        tracing::info!(
            "Appended {} proofs to Iceberg table '{}' at {}",
            proofs.len(),
            table,
            data_path
        );

        Ok(snapshot)
    }

    pub fn list_snapshots(&self, table: &str) -> Result<Vec<Snapshot>> {
        let _config = self.tables.get(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;
        Ok(Vec::new())
    }

    pub fn time_travel(&self, table: &str) -> Result<Vec<String>> {
        let config = self.tables.get(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;

        let data_dir_path = format!("{}/data", config.location);
        let data_dir = std::fs::read_dir(&data_dir_path)
            .map_err(|e| crate::AnalyticsError::Iceberg(e.to_string()))?;

        let files: Vec<String> = data_dir
            .filter_map(|entry| entry.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "parquet")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();

        Ok(files)
    }

    pub fn compact_data(&self, table: &str) -> Result<u64> {
        let _config = self.tables.get(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;

        tracing::info!("Compacting data for Iceberg table '{}'", table);
        Ok(0)
    }

    pub fn expire_snapshots(&self, table: &str, _older_than: DateTime<Utc>) -> Result<u64> {
        let _config = self.tables.get(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;

        tracing::info!("Expiring snapshots for Iceberg table '{}'", table);
        Ok(0)
    }

    pub fn table_location(&self, table: &str) -> Result<String> {
        let config = self.tables.get(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;
        Ok(config.location.clone())
    }

    pub fn drop_table(&mut self, table: &str) -> Result<()> {
        self.tables.remove(table).ok_or_else(|| {
            crate::AnalyticsError::Iceberg(format!("Table '{}' not found", table))
        })?;
        Ok(())
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

    #[test]
    fn test_engine_creation() {
        let engine = IcebergEngine::new("/tmp/poi/test_warehouse");
        assert_eq!(engine.warehouse_path(), "/tmp/poi/test_warehouse");
    }

    #[test]
    fn test_table_registration() {
        let mut engine = IcebergEngine::new("/tmp/warehouse");
        let config = IcebergTableConfig {
            name: "proofs".to_string(),
            ..Default::default()
        };
        engine.register_table(config);
        assert!(engine.get_table("proofs").is_some());
    }

    #[test]
    fn test_create_and_append() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let warehouse = tmp_dir.path().join("warehouse");
        let mut engine = IcebergEngine::new(warehouse.to_str().unwrap());

        let config = IcebergTableConfig {
            name: "proofs".to_string(),
            location: warehouse.join("proofs").to_str().unwrap().to_string(),
            ..Default::default()
        };
        engine.register_table(config.clone());
        engine.create_table(&config).unwrap();

        let snapshot = engine.append_proofs("proofs", &sample_proofs()).unwrap();
        assert_eq!(snapshot.operation, "append");
    }

    #[test]
    fn test_drop_table() {
        let mut engine = IcebergEngine::new("/tmp/warehouse");
        let config = IcebergTableConfig {
            name: "proofs".to_string(),
            ..Default::default()
        };
        engine.register_table(config);
        assert!(engine.get_table("proofs").is_some());
        engine.drop_table("proofs").unwrap();
        assert!(engine.get_table("proofs").is_none());
    }
}
