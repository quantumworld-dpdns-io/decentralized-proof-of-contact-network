use chrono::{DateTime, Utc};
use poi_core::{NodeId, PeerId};

pub fn query_proof_count_by_window(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    format!(
        "SELECT orbital_window, COUNT(*) AS proof_count, COUNT(DISTINCT prover_id) AS unique_provers, COUNT(DISTINCT verifier_id) AS unique_verifiers
FROM proofs
WHERE timestamp >= '{}' AND timestamp < '{}'
GROUP BY orbital_window
ORDER BY proof_count DESC",
        start.format("%Y-%m-%d %H:%M:%S"),
        end.format("%Y-%m-%d %H:%M:%S")
    )
}

pub fn query_node_activity(node_id: &NodeId, start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    format!(
        "SELECT 
    '{}' AS node_id,
    orbital_window,
    COUNT(*) AS total_proofs,
    SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END) AS verified_count,
    SUM(CASE WHEN verification_status = 'failed' THEN 1 ELSE 0 END) AS failed_count,
    AVG(confidence_score) AS avg_confidence
FROM proofs
WHERE (prover_id = '{}' OR verifier_id = '{}')
  AND timestamp >= '{}' AND timestamp < '{}'
GROUP BY orbital_window
ORDER BY orbital_window",
        node_id,
        node_id,
        node_id,
        start.format("%Y-%m-%d %H:%M:%S"),
        end.format("%Y-%m-%d %H:%M:%S")
    )
}

pub fn query_network_topology() -> String {
    "SELECT 
    prover_id AS source,
    verifier_id AS target,
    COUNT(*) AS interaction_count,
    AVG(confidence_score) AS avg_confidence,
    MIN(timestamp) AS first_seen,
    MAX(timestamp) AS last_seen
FROM proofs
GROUP BY prover_id, verifier_id
ORDER BY interaction_count DESC"
        .to_string()
}

pub fn query_verification_rate(window: &str) -> String {
    format!(
        "SELECT 
    orbital_window,
    COUNT(*) AS total,
    SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END) AS verified,
    ROUND(SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END) * 100.0 / COUNT(*), 2) AS verification_rate_pct
FROM proofs
WHERE orbital_window = '{}'
GROUP BY orbital_window",
        window
    )
}

pub fn query_peer_reputation_trends(peer_id: &PeerId) -> String {
    format!(
        "SELECT 
    '{}' AS peer_id,
    orbital_window,
    COUNT(*) AS total_interactions,
    AVG(confidence_score) AS avg_confidence,
    SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END) AS successful_verifications,
    SUM(CASE WHEN verification_status = 'failed' THEN 1 ELSE 0 END) AS failed_verifications
FROM proofs
WHERE prover_id = '{}' OR verifier_id = '{}'
GROUP BY orbital_window
ORDER BY orbital_window",
        peer_id, peer_id, peer_id
    )
}

pub fn query_orbital_window_utilization() -> String {
    "SELECT 
    orbital_window,
    COUNT(*) AS proof_count,
    COUNT(DISTINCT prover_id) AS active_provers,
    COUNT(DISTINCT verifier_id) AS active_verifiers,
    MIN(timestamp) AS window_start,
    MAX(timestamp) AS window_end,
    AVG(confidence_score) AS avg_confidence,
    SUM(CASE WHEN verification_status = 'verified' THEN 1 ELSE 0 END) * 100.0 / COUNT(*) AS verification_rate
FROM proofs
GROUP BY orbital_window
ORDER BY orbital_window"
        .to_string()
}

pub fn query_proof_chain_depth_distribution() -> String {
    "SELECT 
    depth,
    COUNT(*) AS chain_count,
    COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () AS percentage
FROM proof_chains
GROUP BY depth
ORDER BY depth"
        .to_string()
}

pub fn query_anomalous_patterns(threshold: f64) -> String {
    format!(
        "SELECT 
    prover_id,
    COUNT(*) AS total_proofs,
    SUM(CASE WHEN verification_status = 'failed' THEN 1 ELSE 0 END) AS failed_count,
    ROUND(SUM(CASE WHEN verification_status = 'failed' THEN 1 ELSE 0 END) * 100.0 / COUNT(*), 2) AS failure_rate_pct,
    AVG(confidence_score) AS avg_confidence,
    COUNT(DISTINCT verifier_id) AS unique_verifiers
FROM proofs
GROUP BY prover_id
HAVING (SUM(CASE WHEN verification_status = 'failed' THEN 1 ELSE 0 END) * 100.0 / COUNT(*)) > {}
ORDER BY failure_rate_pct DESC",
        threshold
    )
}

pub fn query_confidence_score_distribution() -> String {
    "SELECT 
    CASE 
        WHEN confidence_score >= 0.9 THEN '0.9-1.0'
        WHEN confidence_score >= 0.8 THEN '0.8-0.9'
        WHEN confidence_score >= 0.7 THEN '0.7-0.8'
        WHEN confidence_score >= 0.6 THEN '0.6-0.7'
        WHEN confidence_score >= 0.5 THEN '0.5-0.6'
        WHEN confidence_score >= 0.4 THEN '0.4-0.5'
        WHEN confidence_score >= 0.3 THEN '0.3-0.4'
        WHEN confidence_score >= 0.2 THEN '0.2-0.3'
        WHEN confidence_score >= 0.1 THEN '0.1-0.2'
        ELSE '0.0-0.1'
    END AS score_range,
    COUNT(*) AS count,
    ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER (), 2) AS percentage
FROM proofs
GROUP BY score_range
ORDER BY score_range DESC"
        .to_string()
}

#[derive(Debug, Clone)]
pub struct QueryDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub sql: String,
}

impl QueryDefinition {
    pub fn new(name: &'static str, description: &'static str, sql: String) -> Self {
        Self {
            name,
            description,
            sql,
        }
    }
}

pub fn predefined_queries() -> Vec<QueryDefinition> {
    let now = Utc::now();
    let week_ago = now - chrono::Duration::days(7);

    vec![
        QueryDefinition::new(
            "proof_count_by_window",
            "Count proofs grouped by orbital window for the last 7 days",
            query_proof_count_by_window(week_ago, now),
        ),
        QueryDefinition::new(
            "network_topology",
            "Network topology from proof interactions",
            query_network_topology(),
        ),
        QueryDefinition::new(
            "orbital_window_utilization",
            "Utilization metrics for each orbital window",
            query_orbital_window_utilization(),
        ),
        QueryDefinition::new(
            "confidence_score_distribution",
            "Distribution of confidence scores",
            query_confidence_score_distribution(),
        ),
        QueryDefinition::new(
            "anomalous_patterns",
            "Detect nodes with high failure rates (threshold: 50%)",
            query_anomalous_patterns(50.0),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_query_proof_count_by_window() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).unwrap();
        let sql = query_proof_count_by_window(start, end);
        assert!(sql.contains("orbital_window"));
        assert!(sql.contains("2025-01-01"));
        assert!(sql.contains("2025-01-08"));
    }

    #[test]
    fn test_query_node_activity() {
        let node = NodeId("test-node".to_string());
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).unwrap();
        let sql = query_node_activity(&node, start, end);
        assert!(sql.contains("test-node"));
        assert!(sql.contains("verified_count"));
    }

    #[test]
    fn test_query_network_topology() {
        let sql = query_network_topology();
        assert!(sql.contains("prover_id"));
        assert!(sql.contains("verifier_id"));
        assert!(sql.contains("source"));
        assert!(sql.contains("target"));
    }

    #[test]
    fn test_query_verification_rate() {
        let sql = query_verification_rate("window-1");
        assert!(sql.contains("window-1"));
        assert!(sql.contains("verification_rate_pct"));
    }

    #[test]
    fn test_query_peer_reputation_trends() {
        let peer = PeerId("peer-1".to_string());
        let sql = query_peer_reputation_trends(&peer);
        assert!(sql.contains("peer-1"));
        assert!(sql.contains("successful_verifications"));
    }

    #[test]
    fn test_query_orbital_window_utilization() {
        let sql = query_orbital_window_utilization();
        assert!(sql.contains("orbital_window"));
        assert!(sql.contains("active_provers"));
    }

    #[test]
    fn test_query_proof_chain_depth_distribution() {
        let sql = query_proof_chain_depth_distribution();
        assert!(sql.contains("depth"));
        assert!(sql.contains("chain_count"));
    }

    #[test]
    fn test_query_anomalous_patterns() {
        let sql = query_anomalous_patterns(50.0);
        assert!(sql.contains("50"));
        assert!(sql.contains("failure_rate_pct"));
    }

    #[test]
    fn test_query_confidence_score_distribution() {
        let sql = query_confidence_score_distribution();
        assert!(sql.contains("confidence_score"));
        assert!(sql.contains("0.9-1.0"));
    }

    #[test]
    fn test_predefined_queries() {
        let queries = predefined_queries();
        assert!(!queries.is_empty());
        assert!(queries.iter().any(|q| q.name == "network_topology"));
    }
}
