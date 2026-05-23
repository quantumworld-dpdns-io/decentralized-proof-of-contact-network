use std::sync::Arc;

use arrow::array::{
    ArrayRef, Float64Array, StringBuilder, StringArray, TimestampNanosecondArray,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, Utc};
use poi_core::{NodeId, OrbitalWindowId, Proof, VerificationStatus};

use crate::Result;

pub fn proof_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("prover_id", DataType::Utf8, false),
        Field::new("verifier_id", DataType::Utf8, false),
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("verification_status", DataType::Utf8, false),
        Field::new("confidence_score", DataType::Float64, false),
        Field::new("orbital_window", DataType::Utf8, false),
        Field::new("signature", DataType::Binary, false),
        Field::new("metadata", DataType::Utf8, false),
    ]))
}

pub fn node_activity_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("node_id", DataType::Utf8, false),
        Field::new("window", DataType::Utf8, false),
        Field::new("proof_count", DataType::Int64, false),
        Field::new("verified_count", DataType::Int64, false),
        Field::new("avg_confidence", DataType::Float64, false),
    ]))
}

pub struct ProofBatchBuilder {
    ids: Vec<String>,
    prover_ids: Vec<String>,
    verifier_ids: Vec<String>,
    timestamps: Vec<i64>,
    statuses: Vec<String>,
    scores: Vec<f64>,
    windows: Vec<String>,
    signatures: Vec<Vec<u8>>,
    metadatas: Vec<String>,
}

impl ProofBatchBuilder {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            prover_ids: Vec::new(),
            verifier_ids: Vec::new(),
            timestamps: Vec::new(),
            statuses: Vec::new(),
            scores: Vec::new(),
            windows: Vec::new(),
            signatures: Vec::new(),
            metadatas: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            prover_ids: Vec::with_capacity(capacity),
            verifier_ids: Vec::with_capacity(capacity),
            timestamps: Vec::with_capacity(capacity),
            statuses: Vec::with_capacity(capacity),
            scores: Vec::with_capacity(capacity),
            windows: Vec::with_capacity(capacity),
            signatures: Vec::with_capacity(capacity),
            metadatas: Vec::with_capacity(capacity),
        }
    }

    pub fn add_proof(&mut self, proof: &Proof) {
        self.ids.push(proof.id.to_string());
        self.prover_ids.push(proof.prover_id.0.clone());
        self.verifier_ids.push(proof.verifier_id.0.clone());
        self.timestamps.push(proof.timestamp.timestamp_nanos_opt().unwrap_or(0));
        self.statuses.push(proof.verification_status.as_str().to_string());
        self.scores.push(proof.confidence_score);
        self.windows.push(proof.orbital_window.0.clone());
        self.signatures.push(proof.signature.clone());
        self.metadatas.push(proof.metadata.to_string());
    }

    pub fn extend_from_iter(&mut self, proofs: impl IntoIterator<Item = Proof>) {
        for proof in proofs {
            self.add_proof(&proof);
        }
    }

    pub fn finish(&self) -> Result<RecordBatch> {
        let schema = proof_schema();

        let id_array = Arc::new(StringArray::from(self.ids.clone())) as ArrayRef;
        let prover_array = Arc::new(StringArray::from(self.prover_ids.clone())) as ArrayRef;
        let verifier_array = Arc::new(StringArray::from(self.verifier_ids.clone())) as ArrayRef;
        let timestamp_array =
            Arc::new(TimestampNanosecondArray::from(self.timestamps.clone())) as ArrayRef;
        let status_array = Arc::new(StringArray::from(self.statuses.clone())) as ArrayRef;
        let score_array = Arc::new(Float64Array::from(self.scores.clone())) as ArrayRef;
        let window_array = Arc::new(StringArray::from(self.windows.clone())) as ArrayRef;

        let mut sig_builder = StringBuilder::new();
        for sig in &self.signatures {
            sig_builder.append_value(hex::encode(sig));
        }
        let signature_array = Arc::new(sig_builder.finish()) as ArrayRef;

        let metadata_array = Arc::new(StringArray::from(self.metadatas.clone())) as ArrayRef;

        let batch = RecordBatch::try_new(
            schema,
            vec![
                id_array, prover_array, verifier_array, timestamp_array, status_array,
                score_array, window_array, signature_array, metadata_array,
            ],
        )
        .map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;

        Ok(batch)
    }

    pub fn finish_and_write_parquet(&self, path: &str) -> Result<()> {
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

pub fn proofs_to_batch(proofs: &[Proof]) -> Result<RecordBatch> {
    let mut builder = ProofBatchBuilder::with_capacity(proofs.len());
    for proof in proofs {
        builder.add_proof(proof);
    }
    builder.finish()
}

pub fn batch_from_json(json_data: &str) -> Result<RecordBatch> {
    let schema = proof_schema();
    let reader =
        arrow::json::ReaderBuilder::new(schema).build(std::io::Cursor::new(json_data.as_bytes()));
    let mut reader = reader.map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;
    let batch = reader
        .next()
        .ok_or_else(|| crate::AnalyticsError::Arrow("Empty JSON data".to_string()))?
        .map_err(|e| crate::AnalyticsError::Arrow(e.to_string()))?;
    Ok(batch)
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
        tracing::info!("Executing query via Flight SQL: {}", _query);
        Err(crate::AnalyticsError::Arrow(
            "Flight SQL client stub: not implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use poi_core::{NodeId, OrbitalWindowId, Proof, VerificationStatus};
    use uuid::Uuid;

    fn sample_proof() -> Proof {
        Proof {
            id: Uuid::new_v4(),
            prover_id: NodeId("node-1".to_string()),
            verifier_id: NodeId("node-2".to_string()),
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            verification_status: VerificationStatus::Verified,
            confidence_score: 0.95,
            orbital_window: OrbitalWindowId("window-1".to_string()),
            signature: vec![1, 2, 3, 4],
            metadata: serde_json::json!({"location": "NYC"}),
        }
    }

    #[test]
    fn test_proof_schema() {
        let schema = proof_schema();
        assert_eq!(schema.fields().len(), 9);
        assert_eq!(schema.fields()[0].name(), "id");
        assert_eq!(schema.fields()[4].name(), "verification_status");
    }

    #[test]
    fn test_proof_batch_builder() {
        let proof = sample_proof();
        let mut builder = ProofBatchBuilder::new();
        builder.add_proof(&proof);
        assert_eq!(builder.len(), 1);

        let batch = builder.finish().unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 9);
    }

    #[test]
    fn test_proofs_to_batch() {
        let proofs = vec![sample_proof(), sample_proof()];
        let batch = proofs_to_batch(&proofs).unwrap();
        assert_eq!(batch.num_rows(), 2);
    }

    #[test]
    fn test_batch_builder_empty() {
        let builder = ProofBatchBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_flight_sql_client() {
        let client = FlightSqlClient::new("localhost", 8080);
        assert_eq!(client.address(), "localhost:8080");
    }
}
