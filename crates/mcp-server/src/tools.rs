use serde_json::{json, Value};

use crate::error::{McpError, Result};
use crate::server::ApiClient;

fn create_proof_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "proving_node": {
                "type": "string",
                "description": "Node ID creating the proof of contact"
            },
            "target_node": {
                "type": "string",
                "description": "Node ID that is the target of the proof"
            },
            "confidence_score": {
                "type": "number",
                "description": "Confidence score for the proof (0.0 to 1.0)",
                "minimum": 0.0,
                "maximum": 1.0,
                "default": 1.0
            },
            "proof_purpose": {
                "type": "string",
                "description": "Purpose or context of the proof",
                "default": "contact_verification"
            },
            "window_type": {
                "type": "string",
                "enum": ["Standard", "Extended", "Emergency"],
                "description": "Orbital window type for the proof",
                "default": "Standard"
            },
            "window_duration_hours": {
                "type": "number",
                "description": "Duration of the orbital window in hours",
                "default": 1.0
            }
        },
        "required": ["proving_node", "target_node"]
    })
}

fn verify_proof_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "proof_id": {
                "type": "string",
                "description": "UUID of the proof to verify"
            }
        },
        "required": ["proof_id"]
    })
}

fn search_proofs_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query string"
            },
            "proving_node": {
                "type": "string",
                "description": "Filter by proving node ID"
            },
            "target_node": {
                "type": "string",
                "description": "Filter by target node ID"
            },
            "status": {
                "type": "string",
                "enum": ["verified", "failed", "pending", "expired"],
                "description": "Filter by verification status"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum number of results",
                "default": 50
            },
            "offset": {
                "type": "integer",
                "description": "Number of results to skip",
                "default": 0
            }
        }
    })
}

fn get_network_status_schema() -> Value {
    json!({
        "type": "object",
        "properties": {}
    })
}

fn list_peers_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "connected_only": {
                "type": "boolean",
                "description": "Only return connected peers",
                "default": true
            }
        }
    })
}

fn get_analytics_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "timeframe": {
                "type": "string",
                "enum": ["hour", "day", "week", "month", "all"],
                "description": "Analytics timeframe",
                "default": "day"
            }
        }
    })
}

fn ai_query_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Natural language query about the network"
            },
            "context": {
                "type": "string",
                "description": "Additional context or constraints for the query"
            }
        },
        "required": ["query"]
    })
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "create_proof",
            "description": "Create a new proof of contact between two nodes in the network. Used to establish and record cryptographic proof that two nodes had contact within a specific orbital time window.",
            "inputSchema": create_proof_schema()
        }),
        json!({
            "name": "verify_proof",
            "description": "Verify a proof of contact by its UUID. Checks cryptographic signature validity, timestamp freshness, and orbital window compliance. Returns a detailed verification report.",
            "inputSchema": verify_proof_schema()
        }),
        json!({
            "name": "search_proofs",
            "description": "Search and filter proofs of contact across the network. Supports filtering by node IDs, verification status, and full-text search. Returns paginated results.",
            "inputSchema": search_proofs_schema()
        }),
        json!({
            "name": "get_network_status",
            "description": "Get the current status of the proof-of-contact network. Returns node health, chain height, active window count, and overall network metrics.",
            "inputSchema": get_network_status_schema()
        }),
        json!({
            "name": "list_peers",
            "description": "List peers in the proof-of-contact network. Returns peer node IDs, connection status, protocol versions, and recent activity timestamps.",
            "inputSchema": list_peers_schema()
        }),
        json!({
            "name": "get_analytics",
            "description": "Get network analytics and statistics. Returns metrics like proof creation rates, verification success rates, peer distribution, and historical trends.",
            "inputSchema": get_analytics_schema()
        }),
        json!({
            "name": "ai_query",
            "description": "Submit a natural language query to the AI analysis engine. The AI can analyze network patterns, detect anomalies, answer questions about proofs and peers, and provide insights about network health.",
            "inputSchema": ai_query_schema()
        }),
    ]
}

pub async fn list_tools(_client: &ApiClient) -> Result<Vec<Value>> {
    Ok(tool_definitions())
}

pub async fn call_tool(client: &ApiClient, name: &str, args: &Value) -> Result<Value> {
    match name {
        "create_proof" => tool_create_proof(client, args).await,
        "verify_proof" => tool_verify_proof(client, args).await,
        "search_proofs" => tool_search_proofs(client, args).await,
        "get_network_status" => tool_get_network_status(client, args).await,
        "list_peers" => tool_list_peers(client, args).await,
        "get_analytics" => tool_get_analytics(client, args).await,
        "ai_query" => tool_ai_query(client, args).await,
        _ => Err(McpError::ToolNotFound(name.to_string())),
    }
}

pub async fn tool_create_proof(client: &ApiClient, args: &Value) -> Result<Value> {
    let proving_node = args
        .get("proving_node")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: proving_node".into()))?;
    let target_node = args
        .get("target_node")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: target_node".into()))?;

    let body = json!({
        "proving_node": proving_node,
        "target_node": target_node,
        "confidence_score": args.get("confidence_score").and_then(|v| v.as_f64()).unwrap_or(1.0),
        "proof_purpose": args.get("proof_purpose").and_then(|v| v.as_str()).unwrap_or("contact_verification"),
        "window_type": args.get("window_type").and_then(|v| v.as_str()).unwrap_or("Standard"),
        "window_duration_hours": args.get("window_duration_hours").and_then(|v| v.as_f64()).unwrap_or(1.0)
    });

    let result = client.create_proof(&body).await?;
    Ok(json!({
        "status": "success",
        "proof": result,
        "message": "Proof of contact created successfully"
    }))
}

pub async fn tool_verify_proof(client: &ApiClient, args: &Value) -> Result<Value> {
    let proof_id = args
        .get("proof_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: proof_id".into()))?;

    let result = client.verify_proof(proof_id).await?;
    Ok(json!({
        "status": "success",
        "proof_id": proof_id,
        "verification": result
    }))
}

pub async fn tool_search_proofs(client: &ApiClient, args: &Value) -> Result<Value> {
    let query = json!({
        "query": args.get("query").and_then(|v| v.as_str()).unwrap_or(""),
        "proving_node": args.get("proving_node").and_then(|v| v.as_str()),
        "target_node": args.get("target_node").and_then(|v| v.as_str()),
        "status": args.get("status").and_then(|v| v.as_str()),
        "limit": args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50),
        "offset": args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0)
    });

    let result = client.search_proofs(&query).await?;
    Ok(json!({
        "status": "success",
        "results": result
    }))
}

pub async fn tool_get_network_status(client: &ApiClient, _args: &Value) -> Result<Value> {
    let result = client.get_network_status().await?;
    Ok(json!({
        "status": "success",
        "network": result
    }))
}

pub async fn tool_list_peers(client: &ApiClient, args: &Value) -> Result<Value> {
    let _connected_only = args
        .get("connected_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let result = client.list_peers().await?;
    Ok(json!({
        "status": "success",
        "peers": result
    }))
}

pub async fn tool_get_analytics(client: &ApiClient, args: &Value) -> Result<Value> {
    let _timeframe = args
        .get("timeframe")
        .and_then(|v| v.as_str())
        .unwrap_or("day");

    let result = client.get_analytics().await?;
    Ok(json!({
        "status": "success",
        "analytics": result
    }))
}

pub async fn tool_ai_query(client: &ApiClient, args: &Value) -> Result<Value> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: query".into()))?;

    let body = json!({
        "query": query,
        "context": args.get("context").and_then(|v| v.as_str()).unwrap_or("")
    });

    let result = client.ai_query(&body).await?;
    Ok(json!({
        "status": "success",
        "response": result
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions_count() {
        let defs = tool_definitions();
        assert_eq!(defs.len(), 7);
    }

    #[test]
    fn test_tool_definitions_have_names() {
        let defs = tool_definitions();
        let names: Vec<&str> = defs
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"create_proof"));
        assert!(names.contains(&"verify_proof"));
        assert!(names.contains(&"search_proofs"));
        assert!(names.contains(&"get_network_status"));
        assert!(names.contains(&"list_peers"));
        assert!(names.contains(&"get_analytics"));
        assert!(names.contains(&"ai_query"));
    }

    #[test]
    fn test_tool_definitions_have_schemas() {
        let defs = tool_definitions();
        for tool in &defs {
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool {:?} missing inputSchema",
                tool.get("name")
            );
        }
    }

    #[test]
    fn test_create_proof_schema_required_fields() {
        let schema = create_proof_schema();
        let required = schema.get("required").and_then(|r| r.as_array()).unwrap();
        let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(req_strs.contains(&"proving_node"));
        assert!(req_strs.contains(&"target_node"));
    }

    #[test]
    fn test_verify_proof_schema_requires_proof_id() {
        let schema = verify_proof_schema();
        let required = schema.get("required").and_then(|r| r.as_array()).unwrap();
        let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(req_strs.contains(&"proof_id"));
    }

    #[test]
    fn test_ai_query_schema_requires_query() {
        let schema = ai_query_schema();
        let required = schema.get("required").and_then(|r| r.as_array()).unwrap();
        let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(req_strs.contains(&"query"));
    }

    #[test]
    fn test_call_tool_unknown_tool() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = crate::McpConfig::default();
        let client = ApiClient::new(&config);
        let result = rt.block_on(call_tool(&client, "nonexistent", &json!({})));
        assert!(matches!(result, Err(McpError::ToolNotFound(_))));
    }

    #[test]
    fn test_call_tool_create_proof_missing_args() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = crate::McpConfig::default();
        let client = ApiClient::new(&config);
        let result = rt.block_on(call_tool(&client, "create_proof", &json!({})));
        assert!(matches!(result, Err(McpError::InvalidRequest(_))));
    }

    #[test]
    fn test_call_tool_verify_proof_missing_args() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = crate::McpConfig::default();
        let client = ApiClient::new(&config);
        let result = rt.block_on(call_tool(&client, "verify_proof", &json!({})));
        assert!(matches!(result, Err(McpError::InvalidRequest(_))));
    }
}
