use poi_core::{ContactProof, OrbitalWindow};
use reqwest::header;

use crate::error::*;
use crate::models::*;

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            api_key: None,
        }
    }

    pub fn with_api_key(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            api_key: Some(api_key.to_string()),
        }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let req = self.client.request(method, &url);
        if let Some(ref key) = self.api_key {
            req.header(header::AUTHORIZATION, format!("Bearer {}", key))
        } else {
            req
        }
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            Ok(resp)
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            Err(ClientError::AuthError)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(ClientError::ApiError {
                status: status.as_u16(),
                message: body,
            })
        }
    }

    // ── Proof operations ──────────────────────────────────────────

    pub async fn create_proof(&self, req: &CreateProofRequest) -> Result<CreateProofResponse> {
        let resp = self
            .send(self.request(reqwest::Method::POST, "/api/v1/proofs").json(req))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_proof(&self, id: &str) -> Result<ContactProof> {
        let resp = self
            .send(self.request(
                reqwest::Method::GET,
                &format!("/api/v1/proofs/{}", id),
            ))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn list_proofs(&self, limit: u64, offset: u64) -> Result<Vec<ContactProof>> {
        let resp = self
            .send(
                self.request(reqwest::Method::GET, "/api/v1/proofs")
                    .query(&[("limit", &limit.to_string()), ("offset", &offset.to_string())]),
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn verify_proof(&self, id: &str) -> Result<VerifyResponse> {
        let resp = self
            .send(self.request(
                reqwest::Method::POST,
                &format!("/api/v1/proofs/{}/verify", id),
            ))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn search_proofs(&self, query: &str) -> Result<Vec<ContactProof>> {
        let resp = self
            .send(
                self.request(reqwest::Method::GET, "/api/v1/proofs/search")
                    .query(&[("q", query)]),
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn delete_proof(&self, id: &str) -> Result<()> {
        self.send(self.request(
            reqwest::Method::DELETE,
            &format!("/api/v1/proofs/{}", id),
        ))
        .await?;
        Ok(())
    }

    // ── Node operations ───────────────────────────────────────────

    pub async fn get_node_info(&self) -> Result<NodeInfo> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/api/v1/node"))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn list_peers(&self) -> Result<Vec<PeerInfo>> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/api/v1/node/peers"))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn connect_peer(&self, addr: &str) -> Result<PeerInfo> {
        let body = serde_json::json!({ "address": addr });
        let resp = self
            .send(
                self.request(reqwest::Method::POST, "/api/v1/node/peers")
                    .json(&body),
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn disconnect_peer(&self, id: &str) -> Result<()> {
        self.send(self.request(
            reqwest::Method::DELETE,
            &format!("/api/v1/node/peers/{}", id),
        ))
        .await?;
        Ok(())
    }

    // ── Window operations ─────────────────────────────────────────

    pub async fn list_windows(&self) -> Result<Vec<OrbitalWindow>> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/api/v1/windows"))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn create_window(&self, req: &CreateWindowRequest) -> Result<OrbitalWindow> {
        let resp = self
            .send(
                self.request(reqwest::Method::POST, "/api/v1/windows")
                    .json(req),
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_active_windows(&self) -> Result<Vec<OrbitalWindow>> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/api/v1/windows/active"))
            .await?;
        Ok(resp.json().await?)
    }

    // ── Stats operations ──────────────────────────────────────────

    pub async fn get_node_stats(&self) -> Result<NodeStats> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/api/v1/stats/node"))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        let resp = self
            .send(self.request(
                reqwest::Method::GET,
                "/api/v1/stats/network",
            ))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_proof_stats(&self) -> Result<ProofStats> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/api/v1/stats/proofs"))
            .await?;
        Ok(resp.json().await?)
    }

    // ── Analytics operations ──────────────────────────────────────

    pub async fn run_analytics_query(&self, sql: &str) -> Result<QueryResult> {
        let body = serde_json::json!({ "sql": sql });
        let resp = self
            .send(
                self.request(reqwest::Method::POST, "/api/v1/analytics/query")
                    .json(&body),
            )
            .await?;
        Ok(resp.json().await?)
    }

    // ── AI operations ─────────────────────────────────────────────

    pub async fn ai_query(&self, question: &str) -> Result<AiQueryResponse> {
        let body = serde_json::json!({ "question": question });
        let resp = self
            .send(
                self.request(reqwest::Method::POST, "/api/v1/ai/query")
                    .json(&body),
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn analyze_proof(&self, id: &str) -> Result<ProofAnalysis> {
        let resp = self
            .send(self.request(
                reqwest::Method::POST,
                &format!("/api/v1/ai/analyze/{}", id),
            ))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn detect_anomalies(&self) -> Result<Vec<AnomalyReport>> {
        let resp = self
            .send(self.request(
                reqwest::Method::GET,
                "/api/v1/ai/anomalies",
            ))
            .await?;
        Ok(resp.json().await?)
    }

    // ── Health ────────────────────────────────────────────────────

    pub async fn health_check(&self) -> Result<HealthStatus> {
        let resp = self
            .send(self.request(reqwest::Method::GET, "/health"))
            .await?;
        Ok(resp.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mockito::Server;
    use poi_core::{NodeId, OrbitalWindow, ProofMetadata, WindowType};

    #[tokio::test]
    async fn test_health_check() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/health")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"status":"ok","version":"0.1.0","uptime_seconds":123,"node_id":"poi-node","checks":{}}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let health = client.health_check().await.unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.version, "0.1.0");
        assert_eq!(health.uptime_seconds, 123);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_auth_error() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/health")
            .with_status(401)
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let err = client.health_check().await.unwrap_err();
        assert!(matches!(err, ClientError::AuthError));
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_api_error() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/health")
            .with_status(400)
            .with_body("bad request")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let err = client.health_check().await.unwrap_err();
        match err {
            ClientError::ApiError { status, message } => {
                assert_eq!(status, 400);
                assert_eq!(message, "bad request");
            }
            _ => panic!("expected ApiError"),
        }
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_with_api_key() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/health")
            .match_header("authorization", "Bearer my-secret-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"status":"ok","version":"0.1.0","uptime_seconds":0,"node_id":"poi-node","checks":{}}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::with_api_key(&server.url(), "my-secret-key");
        client.health_check().await.unwrap();
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_proofs() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/proofs")
            .match_query("limit=10&offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let proofs = client.list_proofs(10, 0).await.unwrap();
        assert!(proofs.is_empty());
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_proof() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/proofs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"proof":{"id":"550e8400-e29b-41d4-a716-446655440000","proving_node":"poi-a","target_node":"poi-b","orbital_window":{"id":"00000000-0000-0000-0000-000000000000","start_time":"2025-01-01T00:00:00Z","end_time":"2025-01-01T01:00:00Z","window_type":"Standard"},"timestamp":"2025-01-01T00:00:00Z","signature":"","pqc_signature":null,"metadata":{"protocol_version":"1.0","chain_position":null,"confidence_score":1.0,"proof_purpose":"test"}}}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let req = CreateProofRequest {
            proving_node: NodeId("poi-a".into()),
            target_node: NodeId("poi-b".into()),
            orbital_window: OrbitalWindow::new(
                Utc::now(),
                Utc::now() + chrono::Duration::hours(1),
                WindowType::Standard,
            ),
            metadata: ProofMetadata {
                protocol_version: "1.0".into(),
                chain_position: None,
                confidence_score: 1.0,
                proof_purpose: "test".into(),
            },
        };
        let resp = client.create_proof(&req).await.unwrap();
        assert_eq!(resp.proof.proving_node.0, "poi-a");
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_proof() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/proofs/some-id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"550e8400-e29b-41d4-a716-446655440000","proving_node":"poi-a","target_node":"poi-b","orbital_window":{"id":"00000000-0000-0000-0000-000000000000","start_time":"2025-01-01T00:00:00Z","end_time":"2025-01-01T01:00:00Z","window_type":"Standard"},"timestamp":"2025-01-01T00:00:00Z","signature":"","pqc_signature":null,"metadata":{"protocol_version":"1.0","chain_position":null,"confidence_score":1.0,"proof_purpose":"test"}}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let proof = client.get_proof("some-id").await.unwrap();
        assert_eq!(proof.proving_node.0, "poi-a");
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_verify_proof() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/proofs/test-id/verify")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"valid":true,"status":"verified","details":null}"#)
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let resp = client.verify_proof("test-id").await.unwrap();
        assert!(resp.valid);
        assert_eq!(resp.status, "verified");
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_search_proofs() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/proofs/search")
            .match_query("q=test-query")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let proofs = client.search_proofs("test-query").await.unwrap();
        assert!(proofs.is_empty());
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_delete_proof() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("DELETE", "/api/v1/proofs/test-id")
            .with_status(200)
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        client.delete_proof("test-id").await.unwrap();
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_node_info() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/node")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"poi-node","version":"0.1.0","uptime_seconds":3600,"proof_count":42,"peer_count":5,"state":"Active"}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let info = client.get_node_info().await.unwrap();
        assert_eq!(info.id.0, "poi-node");
        assert_eq!(info.proof_count, 42);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_peers() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/node/peers")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[{"id":"peer-1","address":"127.0.0.1:9090","connected_since":null,"protocol_version":"0.1.0","latency_ms":null}]"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let peers = client.list_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, "peer-1");
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_connect_peer() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/node/peers")
            .match_body(r#"{"address":"127.0.0.1:9091"}"#)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"peer-2","address":"127.0.0.1:9091","connected_since":null,"protocol_version":"0.1.0","latency_ms":null}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let peer = client.connect_peer("127.0.0.1:9091").await.unwrap();
        assert_eq!(peer.address, "127.0.0.1:9091");
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_disconnect_peer() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("DELETE", "/api/v1/node/peers/peer-1")
            .with_status(200)
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        client.disconnect_peer("peer-1").await.unwrap();
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_windows() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/windows")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let windows = client.list_windows().await.unwrap();
        assert!(windows.is_empty());
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_create_window() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/windows")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"00000000-0000-0000-0000-000000000000","start_time":"2025-01-01T00:00:00Z","end_time":"2025-01-01T01:00:00Z","window_type":"Standard"}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let req = CreateWindowRequest {
            window_type: WindowType::Standard,
            duration_minutes: Some(60),
            start_time: None,
            end_time: None,
        };
        let window = client.create_window(&req).await.unwrap();
        assert_eq!(window.window_type, WindowType::Standard);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_active_windows() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/windows/active")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let windows = client.get_active_windows().await.unwrap();
        assert!(windows.is_empty());
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_node_stats() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/stats/node")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"total_proofs":100,"active_peers":5,"uptime_seconds":3600,"memory_usage_bytes":null,"cpu_usage_percent":null}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let stats = client.get_node_stats().await.unwrap();
        assert_eq!(stats.total_proofs, 100);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_network_stats() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/stats/network")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"total_peers":10,"active_connections":5,"messages_sent":1000,"messages_received":800,"bytes_sent":50000,"bytes_received":40000,"avg_latency_ms":null}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let stats = client.get_network_stats().await.unwrap();
        assert_eq!(stats.messages_sent, 1000);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_proof_stats() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/stats/proofs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"total_proofs":100,"verified_proofs":80,"pending_proofs":10,"expired_proofs":5,"failed_proofs":5,"avg_confidence_score":0.95}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let stats = client.get_proof_stats().await.unwrap();
        assert_eq!(stats.avg_confidence_score, 0.95);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_run_analytics_query() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/analytics/query")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"columns":["count"],"rows":[["42"]],"row_count":1,"execution_time_ms":10}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let result = client.run_analytics_query("SELECT count(*) FROM proofs").await.unwrap();
        assert_eq!(result.row_count, 1);
        assert_eq!(result.columns, vec!["count"]);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_ai_query() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/ai/query")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"answer":"The network has 100 proofs.","confidence":0.95,"sources":null,"processing_time_ms":150}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let result = client.ai_query("How many proofs?").await.unwrap();
        assert_eq!(result.answer, "The network has 100 proofs.");
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_analyze_proof() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/ai/analyze/test-id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"proof_id":"test-id","risk_score":0.1,"anomalies":[],"recommendations":["No action needed"]}"#,
            )
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let analysis = client.analyze_proof("test-id").await.unwrap();
        assert_eq!(analysis.risk_score, 0.1);
        _m.assert_async().await;
    }

    #[tokio::test]
    async fn test_detect_anomalies() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v1/ai/anomalies")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url());
        let anomalies = client.detect_anomalies().await.unwrap();
        assert!(anomalies.is_empty());
        _m.assert_async().await;
    }
}
