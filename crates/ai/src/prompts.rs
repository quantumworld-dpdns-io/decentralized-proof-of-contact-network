use poi_core::ContactProof;

use crate::query::NetworkStats;

pub const PROOF_ANALYSIS_SYSTEM_PROMPT: &str =
    "You are a proof analysis AI. Analyze contact proofs and provide detailed assessment.";

pub const ANOMALY_DETECTION_SYSTEM_PROMPT: &str =
    "You are an anomaly detection AI. Identify suspicious patterns in contact proofs.";

pub const NL_TO_QUERY_SYSTEM_PROMPT: &str =
    "You are a natural language to query translator. Convert user questions into structured queries.";

pub const NETWORK_SUMMARY_SYSTEM_PROMPT: &str =
    "You are a network analysis AI. Summarize network activity and provide insights.";

pub fn render_proof_analysis(proof: &ContactProof) -> String {
    format!(
        r#"Analyze this contact proof:
Proving Node: {}
Target Node: {}
Timestamp: {}
Confidence Score: {}
Proof Purpose: {}
Protocol Version: {}

Provide a JSON response with fields: summary, confidence_assessment, temporal_analysis, verification_status, recommendations"#,
        proof.proving_node,
        proof.target_node,
        proof.timestamp,
        proof.metadata.confidence_score,
        proof.metadata.proof_purpose,
        proof.metadata.protocol_version,
    )
}

pub fn render_anomaly_detection(proofs: &[ContactProof]) -> String {
    let mut result = String::from(
        "Analyze these contact proofs for anomalies:\n\n",
    );
    for (i, proof) in proofs.iter().enumerate() {
        result.push_str(&format!(
            "Proof {}: node={} -> target={}, time={}, conf={}, purpose={}\n",
            i + 1,
            proof.proving_node,
            proof.target_node,
            proof.timestamp,
            proof.metadata.confidence_score,
            proof.metadata.proof_purpose,
        ));
    }
    result.push_str(
        "\nProvide a JSON array of anomaly reports with fields: \
         severity (Low/Medium/High/Critical), description, proof_id, node_id, timestamp, suggested_action",
    );
    result
}

pub fn render_nl_to_query(question: &str) -> String {
    format!(
        r#"Convert this natural language question into a structured query:

{}

Provide a JSON response with fields: answer, confidence, related_proofs (array of proof IDs), sql_query"#,
        question
    )
}

pub fn render_network_summary(stats: &NetworkStats) -> String {
    format!(
        r#"Summarize this network activity:
Total Proofs: {}
Total Nodes: {}
Active Nodes: {}
Proofs/Hour: {:.2}
Average Confidence: {:.2}
Chain Depth: {}
Time Window: {:.1} hours

Provide a concise natural language summary."#,
        stats.total_proofs,
        stats.total_nodes,
        stats.active_nodes,
        stats.proofs_per_hour,
        stats.avg_confidence,
        stats.chain_depth,
        stats.time_window_hours,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use poi_core::{ContactProof, ProofMetadata, NodeId, ProofId, OrbitalWindow, WindowType, Signature};
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn sample_proof() -> ContactProof {
        let window = OrbitalWindow {
            id: Uuid::new_v4(),
            start_time: DateTime::from_timestamp_millis(1700000000000).unwrap(),
            end_time: DateTime::from_timestamp_millis(1700003600000).unwrap(),
            window_type: WindowType::Standard,
        };
        ContactProof {
            id: ProofId(Uuid::new_v4()),
            proving_node: NodeId("node-1".to_string()),
            target_node: NodeId("node-2".to_string()),
            orbital_window: window,
            timestamp: DateTime::from_timestamp_millis(1700000000000).unwrap(),
            signature: Signature("sig".to_string()),
            pqc_signature: None,
            metadata: ProofMetadata {
                protocol_version: "1.0".to_string(),
                chain_position: Some(1),
                confidence_score: 0.95,
                proof_purpose: "contact-verification".to_string(),
            },
        }
    }

    #[test]
    fn test_render_proof_analysis() {
        let proof = sample_proof();
        let prompt = render_proof_analysis(&proof);
        assert!(prompt.contains("node-1"));
        assert!(prompt.contains("node-2"));
        assert!(prompt.contains("0.95"));
        assert!(prompt.contains("1.0"));
    }

    #[test]
    fn test_render_anomaly_detection() {
        let proofs = vec![sample_proof()];
        let prompt = render_anomaly_detection(&proofs);
        assert!(prompt.contains("Proof 1"));
        assert!(prompt.contains("node-1"));
    }

    #[test]
    fn test_render_nl_to_query() {
        let prompt = render_nl_to_query("How many proofs today?");
        assert!(prompt.contains("How many proofs today?"));
        assert!(prompt.contains("sql_query"));
    }

    #[test]
    fn test_render_network_summary() {
        let stats = NetworkStats {
            total_proofs: 100,
            total_nodes: 10,
            active_nodes: 8,
            proofs_per_hour: 12.5,
            avg_confidence: 0.87,
            chain_depth: 5,
            time_window_hours: 24.0,
        };
        let prompt = render_network_summary(&stats);
        assert!(prompt.contains("100"));
        assert!(prompt.contains("10"));
        assert!(prompt.contains("12.50"));
        assert!(prompt.contains("0.87"));
    }
}
