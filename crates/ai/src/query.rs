use poi_core::ContactProof;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::prompts;
use crate::provider::{AiProvider, ChatConfig, ChatMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlQueryResult {
    pub answer: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub related_proofs: Vec<String>,
    pub sql_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofAnalysis {
    pub summary: String,
    #[serde(default)]
    pub confidence_assessment: String,
    #[serde(default)]
    pub temporal_analysis: String,
    #[serde(default)]
    pub verification_status: String,
    #[serde(default)]
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub severity: AnomalySeverity,
    pub description: String,
    pub proof_id: Option<String>,
    pub node_id: Option<String>,
    pub timestamp: String,
    #[serde(default)]
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_proofs: u64,
    pub total_nodes: u64,
    pub active_nodes: u64,
    pub proofs_per_hour: f64,
    pub avg_confidence: f64,
    pub chain_depth: usize,
    pub time_window_hours: f64,
}

pub struct NlQueryEngine {
    provider: Box<dyn AiProvider>,
}

impl NlQueryEngine {
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self { provider }
    }

    pub async fn query(&self, question: &str) -> Result<NlQueryResult> {
        let prompt = prompts::render_nl_to_query(question);
        let messages = vec![
            ChatMessage::new("system", prompts::NL_TO_QUERY_SYSTEM_PROMPT),
            ChatMessage::new("user", prompt),
        ];
        let config = ChatConfig {
            temperature: 0.1,
            max_tokens: 1024,
            stream: false,
        };
        let response = self.provider.chat(&messages, &config).await?;
        parse_query_result(&response.content)
    }

    pub async fn analyze_proof(&self, proof: &ContactProof) -> Result<ProofAnalysis> {
        let prompt = prompts::render_proof_analysis(proof);
        let messages = vec![
            ChatMessage::new("system", prompts::PROOF_ANALYSIS_SYSTEM_PROMPT),
            ChatMessage::new("user", prompt),
        ];
        let config = ChatConfig {
            temperature: 0.3,
            max_tokens: 2048,
            stream: false,
        };
        let response = self.provider.chat(&messages, &config).await?;
        parse_proof_analysis(&response.content)
    }

    pub async fn detect_anomalies(&self, proofs: &[ContactProof]) -> Result<Vec<AnomalyReport>> {
        let prompt = prompts::render_anomaly_detection(proofs);
        let messages = vec![
            ChatMessage::new("system", prompts::ANOMALY_DETECTION_SYSTEM_PROMPT),
            ChatMessage::new("user", prompt),
        ];
        let config = ChatConfig {
            temperature: 0.2,
            max_tokens: 2048,
            stream: false,
        };
        let response = self.provider.chat(&messages, &config).await?;
        parse_anomaly_reports(&response.content)
    }

    pub async fn summarize_network(&self, stats: &NetworkStats) -> Result<String> {
        let prompt = prompts::render_network_summary(stats);
        let messages = vec![
            ChatMessage::new("system", prompts::NETWORK_SUMMARY_SYSTEM_PROMPT),
            ChatMessage::new("user", prompt),
        ];
        let config = ChatConfig {
            temperature: 0.5,
            max_tokens: 1024,
            stream: false,
        };
        let response = self.provider.chat(&messages, &config).await?;
        Ok(response.content)
    }
}

fn parse_query_result(content: &str) -> Result<NlQueryResult> {
    match serde_json::from_str::<NlQueryResult>(content) {
        Ok(result) => Ok(result),
        Err(_) => Ok(NlQueryResult {
            answer: content.to_string(),
            confidence: 0.5,
            related_proofs: Vec::new(),
            sql_query: None,
        }),
    }
}

fn parse_proof_analysis(content: &str) -> Result<ProofAnalysis> {
    match serde_json::from_str::<ProofAnalysis>(content) {
        Ok(result) => Ok(result),
        Err(_) => Ok(ProofAnalysis {
            summary: content.to_string(),
            confidence_assessment: String::new(),
            temporal_analysis: String::new(),
            verification_status: String::new(),
            recommendations: Vec::new(),
        }),
    }
}

fn parse_anomaly_reports(content: &str) -> Result<Vec<AnomalyReport>> {
    match serde_json::from_str::<Vec<AnomalyReport>>(content) {
        Ok(reports) => Ok(reports),
        Err(_) => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_stats_default_serde() {
        let stats = NetworkStats {
            total_proofs: 100,
            total_nodes: 10,
            active_nodes: 8,
            proofs_per_hour: 12.5,
            avg_confidence: 0.87,
            chain_depth: 5,
            time_window_hours: 24.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: NetworkStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_proofs, 100);
        assert_eq!(deserialized.total_nodes, 10);
    }

    #[test]
    fn test_parse_query_result_fallback() {
        let content = "I found 5 proofs today";
        let result = parse_query_result(content).unwrap();
        assert_eq!(result.answer, "I found 5 proofs today");
        assert!((result.confidence - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_proof_analysis_fallback() {
        let content = "The proof looks valid";
        let result = parse_proof_analysis(content).unwrap();
        assert_eq!(result.summary, "The proof looks valid");
    }

    #[test]
    fn test_parse_anomaly_reports_fallback() {
        let content = "No anomalies found";
        let result = parse_anomaly_reports(content).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_anomaly_severity_partial_eq() {
        assert_eq!(AnomalySeverity::Low, AnomalySeverity::Low);
        assert_ne!(AnomalySeverity::Low, AnomalySeverity::High);
    }

    #[test]
    fn test_nl_query_engine_construction() {
        // Test with a mock provider that returns basic responses
        // For now, just test that the engine can be constructed
        struct MockProvider;
        #[async_trait::async_trait]
        impl AiProvider for MockProvider {
            fn model(&self) -> &str { "mock" }
            async fn chat(&self, _: &[ChatMessage], _: &ChatConfig) -> Result<crate::provider::ChatResponse> {
                Ok(crate::provider::ChatResponse {
                    content: r#"{"answer": "test", "confidence": 0.9, "related_proofs": [], "sql_query": null}"#.to_string(),
                    usage: crate::provider::TokenUsage { prompt_tokens: 0, completion_tokens: 0 },
                })
            }
            async fn embed(&self, _: &str) -> Result<Vec<f32>> { Ok(vec![0.0; 384]) }
            async fn embed_batch(&self, _: &[&str]) -> Result<Vec<Vec<f32>>> { Ok(vec![vec![0.0; 384]]) }
        }

        let engine = NlQueryEngine::new(Box::new(MockProvider));
        assert!(engine.provider.model() == "mock");
    }

    #[tokio::test]
    async fn test_nl_query_engine_query() {
        struct MockProvider;
        #[async_trait::async_trait]
        impl AiProvider for MockProvider {
            fn model(&self) -> &str { "mock" }
            async fn chat(&self, _: &[ChatMessage], _: &ChatConfig) -> Result<crate::provider::ChatResponse> {
                Ok(crate::provider::ChatResponse {
                    content: r#"{"answer": "42 proofs", "confidence": 0.95, "related_proofs": ["p1", "p2"], "sql_query": "SELECT * FROM proofs"}"#.to_string(),
                    usage: crate::provider::TokenUsage { prompt_tokens: 10, completion_tokens: 20 },
                })
            }
            async fn embed(&self, _: &str) -> Result<Vec<f32>> { Ok(vec![0.0; 384]) }
            async fn embed_batch(&self, _: &[&str]) -> Result<Vec<Vec<f32>>> { Ok(vec![vec![0.0; 384]]) }
        }

        let engine = NlQueryEngine::new(Box::new(MockProvider));
        let result = engine.query("How many proofs?").await.unwrap();
        assert_eq!(result.answer, "42 proofs");
        assert_eq!(result.sql_query, Some("SELECT * FROM proofs".to_string()));
    }
}
