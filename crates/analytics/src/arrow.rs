use std::sync::Arc;

use arrow::array::{
    ArrayRef, Float64Array, StringBuilder, StringArray, TimestampNanosecondArray,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatch;
use poi_core::{ContactProof, NodeId, VerificationStatus};

use crate::Result;

pub fn proof_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("proving_node", DataType::Utf8, false),
        Field::new("target_node", DataType::Utf8, false),
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("window_id", DataType::Utf8, false),
        Field::new("verification_status", DataType::Utf8, false),
        Field::new("confidence_score", DataType::Float64, false),
        Field::new("protocol_version", DataType::Utf8, false),
        Field::new("proof_purpose", DataType::Utf8, false),
        Field::new("chain_position", DataType::Int64, true),
        Field::new("signature", DataType::Utf8, false),
    ]))
}

pub struct ProofBatchBuilder {
    ids: Vec<String>,
    proving_nodes: Vec<String>,
    target_nodes: Vec<String>,
    timestamps: Vec<i64>,
    window_ids: Vec<String>,
    statuses: Vec<String>,
    scores: Vec<f64>,
    protocol_versions: Vec<String>,
    proof_purposes: Vec<String>,
    chain_positions: Vec<Option<i64>>,
    signatures: Vec<String>,
}

impl ProofBatchBuilder {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            proving_nodes: Vec::new(),
            target_nodes: Vec::new(),
            timestamps: Vec::new(),
            window_ids: Vec::new(),
            statuses: Vec::new(),
            scores: Vec::new(),
            protocol_versions: Vec::new(),
            proof_purposes: Vec::new(),
            chain_positions: Vec::new(),
            signatures: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            proving_nodes: Vec::with_capacity(capacity),
            target_nodes: Vec::with_capacity(capacity),
            timestamps: Vec::with_capacity(capacity),
            window_ids: Vec::with_capacity(capacity),
            statuses: Vec::with_capacity(capacity),
            scores: Vec::with_capacity(capacity),
            protocol_versions: Vec::with_capacity(capacity),
            proof_purposes: Vec::with_capacity(capacity),
            chain_positions: Vec::with_capacity(capacity),
            signatures: Vec::with_capacity(capacity),
        }
    }

    fn derive_status(proof: &ContactProof) -> VerificationStatus {
        if proof.signature.0.is_empty() {
            VerificationStatus::Pending
        } else if proof.metadata.confidence_score >= 0.5 {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Failed
        }
    }

    pub fn add_proof(&mut self, proof: &ContactProof) {
        self.ids.push(proof.id.0.to_string());
        self.proving_nodes.push(proof.proving_node.0.clone());
        self.target_nodes.push(proof.target_node.0.clone());
        self.timestamps
            .push(proof.timestamp.timestamp_nanos_opt().unwrap_or(0));
        self.window_ids.push(proof.orbital_window.id.to_string());
        let status = Self::derive_status(proof);
        self.statuses.push(status.as_str().to_string());
        self.scores.push(proof.metadata.confidence_score);
        self.protocol_versions
            .push(proof.metadata.protocol_version.clone());
        self.proof_purposes.push(proof.metadata.proof_purpose.clone());
        self.chain_positions.push(proof.metadata.chain_position);
        self.signatures.push(proof.signature.0.clone());
    }

    pub fn finish(&self) -> Result<RecordBatch> {
        let schema = proof_schema();

        let id_array = Arc::new(StringArray::from(self.ids.clone())) as ArrayRef;
        let proving_array = Arc::new(StringArray::from(self.proving_nodes.clone())) as ArrayRef;
        let target_array = Arc::new(StringArray::from(self.target_nodes.clone())) as ArrayRef;
        let timestamp_array =
            Arc::new(TimestampNanosecondArray::from(self.timestamps.clone())) as ArrayRef;
        let window_array = Arc::new(StringArray::from(self.window_ids.clone())) as ArrayRef;
        let status_array = Arc::new(StringArray::from(self.statuses.clone())) as ArrayRef;
        let score_array = Arc::new(Float64Array::from(self.scores.clone())) as ArrayRef;
        let protocol_array =
            Arc::new(StringArray::from(self.protocol_versions.clone())) as ArrayRef;
        let purpose_array = Arc::new(StringArray::from(self.proof_purposes.clone())) as ArrayRef;

        let cp_array = {
            let mut builder = arrow::array::Int64Builder::new();
            for pos in &self.chain_positions {
                match pos {
                    Some(p) => builder.append_value(*p),
                    None => builder.append_null(),
                }
            }
            Arc::new(builder.finish()) as ArrayRef
        };

        let sig_array = Arc::new(StringArray::from(self.signatures.clone())) as ArrayRef;

        let batch = RecordBatch::try_new(
            schema,
            vec![
                id_array, proving_array, target_array, timestamp_array, window_array,
                status_array, score_array, protocol_array, purpose_array, cp_array,
                sig_array,
            ],
        )
        .map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;

        Ok(batch)
    }

    pub fn write_parquet(&self, path: &str) -> Result<()> {
        let batch = self.finish()?;
        let file = std::fs::File::create(path)?;
        let writer = parquet::arrow::ArrowWriter::try_new(file, batch.schema(), None)
            .map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;
        let mut writer = writer;
        writer
            .write(&batch)
            .map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;
        writer
            .close()
            .map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

impl Default for ProofBatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn proofs_to_batch(proofs: &[ContactProof]) -> Result<RecordBatch> {
    let mut builder = ProofBatchBuilder::with_capacity(proofs.len());
    for proof in proofs {
        builder.add_proof(proof);
    }
    builder.finish()
}

#[derive(Debug)]
pub struct FlightSqlClient {
    host: String,
    port: u16,
}

impl FlightSqlClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub async fn connect(&self) -> Result<()> {
        tracing::info!("Connecting to Arrow Flight SQL at {}", self.address());
        Ok(())
    }

    pub async fn execute_query(&self, _query: &str) -> Result<Vec<RecordBatch>> {
        Err(crate::AnalyticsError::Arrow(
            "Flight SQL client stub: not implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use poi_core::{NodeId, OrbitalWindow, ProofId, ProofMetadata, Signature, WindowType};
    use uuid::Uuid;

    fn sample_proof() -> ContactProof {
        ContactProof {
            id: ProofId(Uuid::new_v4()),
            proving_node: NodeId("node-1".to_string()),
            target_node: NodeId("node-2".to_string()),
            orbital_window: OrbitalWindow {
                id: Uuid::new_v4(),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                end_time: Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap(),
                window_type: WindowType::Standard,
            },
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            signature: Signature("test_sig".to_string()),
            pqc_signature: None,
            metadata: ProofMetadata {
                protocol_version: "1.0".to_string(),
                chain_position: None,
                confidence_score: 0.95,
                proof_purpose: "test".to_string(),
            },
        }
    }

    #[test]
    fn test_proof_schema() {
        let schema = proof_schema();
        assert_eq!(schema.fields().len(), 11);
        assert_eq!(schema.fields()[0].name(), "id");
    }

    #[test]
    fn test_proof_batch_builder() {
        let proof = sample_proof();
        let mut builder = ProofBatchBuilder::new();
        builder.add_proof(&proof);
        assert_eq!(builder.len(), 1);

        let batch = builder.finish().unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 11);
    }

    #[test]
    fn test_proofs_to_batch() {
        let proofs = vec![sample_proof(), sample_proof()];
        let batch = proofs_to_batch(&proofs).unwrap();
        assert_eq!(batch.num_rows(), 2);
    }

    #[test]
    fn test_flight_sql_client() {
        let client = FlightSqlClient::new("localhost", 8080);
        assert_eq!(client.address(), "localhost:8080");
    }
}
