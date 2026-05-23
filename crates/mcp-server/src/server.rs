use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    routing::post,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::config::McpConfig;
use crate::error::{JsonRpcErrorBody, McpError, Result};
use crate::tools;
use crate::resources;
use crate::prompts;

#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcErrorBody>,
}

impl McpResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, error: JsonRpcErrorBody) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

pub struct ApiClient {
    http_client: reqwest::Client,
    api_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(config: &McpConfig) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_url: config.api_url.clone(),
            api_key: config.api_key.clone(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if let Some(ref key) = self.api_key {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", key).parse().unwrap(),
            );
        }
        headers
    }

    pub async fn create_proof(&self, args: &Value) -> Result<Value> {
        let url = format!("{}/api/v1/proofs", self.api_url);
        let resp = self
            .http_client
            .post(&url)
            .headers(self.headers())
            .json(args)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn verify_proof(&self, proof_id: &str) -> Result<Value> {
        let url = format!("{}/api/v1/proofs/{}/verify", self.api_url, proof_id);
        let resp = self.http_client.post(&url).headers(self.headers()).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn search_proofs(&self, query: &Value) -> Result<Value> {
        let url = format!("{}/api/v1/proofs/search", self.api_url);
        let resp = self
            .http_client
            .post(&url)
            .headers(self.headers())
            .json(query)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn get_proof(&self, proof_id: &str) -> Result<Value> {
        let url = format!("{}/api/v1/proofs/{}", self.api_url, proof_id);
        let resp = self.http_client.get(&url).headers(self.headers()).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ResourceNotFound(format!(
                "Proof {} not found ({}): {}",
                proof_id, status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn list_recent_proofs(&self) -> Result<Value> {
        let url = format!("{}/api/v1/proofs", self.api_url);
        let resp = self.http_client.get(&url).headers(self.headers()).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn get_network_status(&self) -> Result<Value> {
        let url = format!("{}/api/v1/status", self.api_url);
        let resp = self.http_client.get(&url).headers(self.headers()).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn list_peers(&self) -> Result<Value> {
        let url = format!("{}/api/v1/peers", self.api_url);
        let resp = self.http_client.get(&url).headers(self.headers()).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn get_analytics(&self) -> Result<Value> {
        let url = format!("{}/api/v1/analytics", self.api_url);
        let resp = self.http_client.get(&url).headers(self.headers()).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "API returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }

    pub(crate) async fn ai_query(&self, query: &Value) -> Result<Value> {
        let url = format!("{}/ai/query", self.api_url);
        let resp = self
            .http_client
            .post(&url)
            .headers(self.headers())
            .json(query)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::ToolExecutionError(format!(
                "AI query returned {}: {}",
                status, text
            )));
        }
        resp.json().await.map_err(McpError::from)
    }
}

pub struct McpServer {
    config: McpConfig,
    api_client: ApiClient,
}

impl McpServer {
    pub fn new(config: McpConfig) -> Self {
        let api_client = ApiClient::new(&config);
        McpServer { config, api_client }
    }

    pub async fn start(&self) -> Result<()> {
        let app = self.build_router();

        let addr = self
            .config
            .bind_addr
            .parse::<std::net::SocketAddr>()
            .map_err(|e| McpError::Internal(format!("Invalid bind address: {}", e)))?;

        info!(
            "Starting MCP server '{}' v{} on {}",
            self.config.server_name, self.config.server_version, addr
        );

        axum::serve(
            tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| McpError::Internal(format!("Failed to bind: {}", e)))?,
            app,
        )
        .await
        .map_err(|e| McpError::Internal(format!("Server error: {}", e)))
    }

    fn build_router(&self) -> Router {
        let state = Arc::new(ApiClient::new(&self.config));

        Router::new()
            .route("/mcp", post(handle_mcp_request))
            .layer(Extension(state))
            .layer(CorsLayer::permissive())
    }

    pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
        match request.method.as_str() {
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tools_call(&request).await,
            "resources/list" => self.handle_resources_list(&request).await,
            "resources/read" => self.handle_resources_read(&request).await,
            "prompts/list" => self.handle_prompts_list(&request).await,
            "prompts/get" => self.handle_prompts_get(&request).await,
            _ => {
                let body = JsonRpcErrorBody {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                };
                Ok(McpResponse::error(request.id, body))
            }
        }
    }

    async fn handle_tools_list(&self, request: &McpRequest) -> Result<McpResponse> {
        let tools = tools::list_tools(&self.api_client).await?;
        Ok(McpResponse::success(request.id.clone(), serde_json::json!({ "tools": tools })))
    }

    async fn handle_tools_call(&self, request: &McpRequest) -> Result<McpResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest("Missing tool name".into()))?;
        let args = params.get("arguments").unwrap_or(&serde_json::Value::Null);

        let result = tools::call_tool(&self.api_client, tool_name, args).await?;
        Ok(McpResponse::success(request.id.clone(), result))
    }

    async fn handle_resources_list(&self, request: &McpRequest) -> Result<McpResponse> {
        let resources = resources::list_resources();
        Ok(McpResponse::success(
            request.id.clone(),
            serde_json::json!({ "resources": resources }),
        ))
    }

    async fn handle_resources_read(&self, request: &McpRequest) -> Result<McpResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest("Missing resource uri".into()))?;

        let content = resources::read_resource(&self.api_client, uri).await?;
        Ok(McpResponse::success(
            request.id.clone(),
            serde_json::json!({ "contents": [content] }),
        ))
    }

    async fn handle_prompts_list(&self, request: &McpRequest) -> Result<McpResponse> {
        let prompts = prompts::list_prompts();
        Ok(McpResponse::success(request.id.clone(), serde_json::json!({ "prompts": prompts })))
    }

    async fn handle_prompts_get(&self, request: &McpRequest) -> Result<McpResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest("Missing prompt name".into()))?;
        let args = params.get("arguments").cloned().unwrap_or(serde_json::Value::Null);

        let prompt = prompts::get_prompt(name, &args)?;
        Ok(McpResponse::success(request.id.clone(), prompt))
    }
}

async fn handle_mcp_request(
    Extension(api_client): Extension<Arc<ApiClient>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let jsonrpc = body
        .get("jsonrpc")
        .and_then(|v| v.as_str())
        .unwrap_or("2.0")
        .to_string();
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let params = body.get("params").cloned();

    let request = McpRequest {
        jsonrpc,
        id: id.clone(),
        method,
        params,
    };

    let config = McpConfig::default();
    let server = McpServer::new(config);

    match server.handle_request(request).await {
        Ok(response) => Json(serde_json::to_value(&response).unwrap_or_default()),
        Err(err) => {
            let body: JsonRpcErrorBody = err.into();
            let response = McpResponse::error(id, body);
            Json(serde_json::to_value(&response).unwrap_or_default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_response_success() {
        let resp = McpResponse::success(Value::Number(1.into()), serde_json::json!({"ok": true}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, 1);
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_mcp_response_error() {
        let body = JsonRpcErrorBody {
            code: -32601,
            message: "not found".into(),
            data: None,
        };
        let resp = McpResponse::error(Value::Number(1.into()), body);
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn test_mcp_server_new() {
        let config = McpConfig::default();
        let server = McpServer::new(config);
        assert_eq!(server.config.server_name, "poi-mcp-server");
    }
}
