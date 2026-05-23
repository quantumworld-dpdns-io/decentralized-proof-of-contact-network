use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum McpError {
    ToolNotFound(String),
    ToolExecutionError(String),
    ResourceNotFound(String),
    InvalidRequest(String),
    Internal(String),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            McpError::ToolExecutionError(msg) => write!(f, "Tool execution error: {}", msg),
            McpError::ResourceNotFound(uri) => write!(f, "Resource not found: {}", uri),
            McpError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            McpError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for McpError {}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            McpError::InvalidRequest(_) => -32600,
            McpError::ToolNotFound(_) => -32601,
            McpError::ResourceNotFound(_) => -32602,
            McpError::ToolExecutionError(_) => -32603,
            McpError::Internal(_) => -32603,
        }
    }
}

pub type Result<T> = std::result::Result<T, McpError>;

impl From<McpError> for JsonRpcErrorBody {
    fn from(err: McpError) -> Self {
        JsonRpcErrorBody {
            code: err.code(),
            message: err.to_string(),
            data: None,
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        McpError::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for McpError {
    fn from(e: reqwest::Error) -> Self {
        McpError::Internal(format!("HTTP request failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            McpError::ToolNotFound("foo".into()).to_string(),
            "Tool not found: foo"
        );
        assert_eq!(
            McpError::ToolExecutionError("fail".into()).to_string(),
            "Tool execution error: fail"
        );
        assert_eq!(
            McpError::ResourceNotFound("poi://x".into()).to_string(),
            "Resource not found: poi://x"
        );
        assert_eq!(
            McpError::InvalidRequest("bad".into()).to_string(),
            "Invalid request: bad"
        );
        assert_eq!(
            McpError::Internal("oops".into()).to_string(),
            "Internal error: oops"
        );
    }

    #[test]
    fn test_error_code() {
        assert_eq!(McpError::InvalidRequest("".into()).code(), -32600);
        assert_eq!(McpError::ToolNotFound("".into()).code(), -32601);
        assert_eq!(McpError::ResourceNotFound("".into()).code(), -32602);
        assert_eq!(McpError::ToolExecutionError("".into()).code(), -32603);
        assert_eq!(McpError::Internal("".into()).code(), -32603);
    }

    #[test]
    fn test_jsonrpc_error_body_from_error() {
        let err = McpError::ToolNotFound("test_tool".into());
        let body: JsonRpcErrorBody = err.into();
        assert_eq!(body.code, -32601);
        assert_eq!(body.message, "Tool not found: test_tool");
        assert!(body.data.is_none());
    }
}
