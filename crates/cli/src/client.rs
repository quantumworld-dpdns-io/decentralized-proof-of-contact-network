use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn get_raw(&self, path: &str) -> Result<reqwest::Response> {
        self.client
            .get(&self.url(path))
            .send()
            .await
            .context("failed to send GET request")
    }

    async fn post_raw(&self, path: &str, body: &Value) -> Result<reqwest::Response> {
        self.client
            .post(&self.url(path))
            .json(body)
            .send()
            .await
            .context("failed to send POST request")
    }

    async fn delete_raw(&self, path: &str) -> Result<reqwest::Response> {
        self.client
            .delete(&self.url(path))
            .send()
            .await
            .context("failed to send DELETE request")
    }

    async fn parse_value(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        let body = resp.text().await.context("failed to read response body")?;
        if let Ok(api) = serde_json::from_str::<ApiResponse<Value>>(&body) {
            if !api.success {
                bail!("{}", api.error.unwrap_or_else(|| "unknown API error".into()));
            }
            return Ok(api.data.unwrap_or(Value::Null));
        }
        if status.is_success() {
            return serde_json::from_str(&body).context("failed to parse response JSON");
        }
        bail!("HTTP {}: {}", status.as_u16(), body);
    }

    async fn get(&self, path: &str) -> Result<Value> {
        self.parse_value(self.get_raw(path).await?).await
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.parse_value(self.post_raw(path, body).await?).await
    }

    async fn delete(&self, path: &str) -> Result<Value> {
        self.parse_value(self.delete_raw(path).await?).await
    }

    fn extract<T: DeserializeOwned>(&self, val: Value) -> Result<T> {
        serde_json::from_value(val).context("failed to deserialize API response")
    }

    // ---- Proof endpoints ----

    pub async fn create_proof(
        &self,
        target: &str,
        window: &str,
        purpose: Option<&str>,
    ) -> Result<Value> {
        let mut body = serde_json::json!({
            "target_node": target,
            "window_id": window,
        });
        if let Some(p) = purpose {
            body["proof_purpose"] = serde_json::json!(p);
        }
        self.post("/api/proofs", &body).await
    }

    pub async fn get_proof(&self, id: &str) -> Result<Value> {
        self.get(&format!("/api/proofs/{}", id)).await
    }

    pub async fn list_proofs(&self, limit: Option<u64>, offset: Option<u64>) -> Result<Value> {
        let mut path = String::from("/api/proofs");
        let mut sep = '?';
        if let Some(l) = limit {
            path.push_str(&format!("{}limit={}", sep, l));
            sep = '&';
        }
        if let Some(o) = offset {
            path.push_str(&format!("{}offset={}", sep, o));
        }
        self.get(&path).await
    }

    pub async fn verify_proof(&self, id: &str) -> Result<Value> {
        let body = serde_json::json!({ "proof_id": id });
        self.post("/api/proofs/verify", &body).await
    }

    pub async fn search_proofs(&self, query: &str) -> Result<Value> {
        let path = format!("/api/proofs/search?q={}", query);
        self.get(&path).await
    }

    // ---- Node endpoints ----

    pub async fn node_info(&self) -> Result<Value> {
        self.get("/api/node/info").await
    }

    pub async fn node_start(&self, config_path: Option<&str>) -> Result<Value> {
        let body = match config_path {
            Some(p) => serde_json::json!({ "config": p }),
            None => serde_json::json!({}),
        };
        self.post("/api/node/start", &body).await
    }

    pub async fn node_stop(&self) -> Result<Value> {
        self.post("/api/node/stop", &serde_json::json!({})).await
    }

    pub async fn node_status(&self) -> Result<Value> {
        self.get("/api/node/status").await
    }

    // ---- Peer endpoints ----

    pub async fn list_peers(&self) -> Result<Value> {
        self.get("/api/peers").await
    }

    pub async fn connect_peer(&self, addr: &str) -> Result<Value> {
        let body = serde_json::json!({ "address": addr });
        self.post("/api/peers/connect", &body).await
    }

    pub async fn disconnect_peer(&self, id: &str) -> Result<Value> {
        self.post(&format!("/api/peers/{}/disconnect", id), &serde_json::json!({}))
            .await
    }

    // ---- Window endpoints ----

    pub async fn list_windows(&self) -> Result<Value> {
        self.get("/api/windows").await
    }

    pub async fn create_window(&self, start: &str, end: &str, window_type: Option<&str>) -> Result<Value> {
        let mut body = serde_json::json!({
            "start_time": start,
            "end_time": end,
        });
        if let Some(wt) = window_type {
            body["window_type"] = serde_json::json!(wt);
        }
        self.post("/api/windows", &body).await
    }

    pub async fn active_window(&self) -> Result<Value> {
        self.get("/api/windows/active").await
    }

    // ---- Config endpoints ----

    pub async fn config_show(&self) -> Result<Value> {
        self.get("/api/config").await
    }

    pub async fn config_set(&self, key: &str, value: &str) -> Result<Value> {
        let body = serde_json::json!({ "key": key, "value": value });
        self.post("/api/config", &body).await
    }

    // ---- Analytics endpoints ----

    pub async fn analytics_run(&self, query: &str) -> Result<Value> {
        let body = serde_json::json!({ "query": query });
        self.post("/api/analytics/run", &body).await
    }

    pub async fn analytics_summary(&self) -> Result<Value> {
        self.get("/api/analytics/summary").await
    }

    // ---- AI endpoints ----

    pub async fn ai_query(&self, question: &str) -> Result<Value> {
        let body = serde_json::json!({ "question": question });
        self.post("/api/ai/query", &body).await
    }

    pub async fn ai_analyze(&self, proof_id: &str) -> Result<Value> {
        let body = serde_json::json!({ "proof_id": proof_id });
        self.post("/api/ai/analyze", &body).await
    }

    pub async fn ai_anomalies(&self) -> Result<Value> {
        self.get("/api/ai/anomalies").await
    }

    pub async fn ai_summarize(&self) -> Result<Value> {
        self.post("/api/ai/summarize", &serde_json::json!({})).await
    }

    // ---- Export / Import helpers ----

    pub async fn fetch_all_proofs(&self) -> Result<Value> {
        self.get("/api/proofs").await
    }

    pub async fn import_proof(&self, proof: &Value) -> Result<Value> {
        self.post("/api/proofs", proof).await
    }
}
