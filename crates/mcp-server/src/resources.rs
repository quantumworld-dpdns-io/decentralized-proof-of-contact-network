use serde_json::{json, Value};
use tracing::warn;

use crate::error::{McpError, Result};
use crate::server::ApiClient;

pub fn list_resources() -> Vec<Value> {
    vec![
        json!({
            "uri": "poi://proofs/list",
            "name": "Recent Proofs",
            "description": "List of the most recent proofs of contact in the network",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "poi://peers",
            "name": "Peer List",
            "description": "Current list of peers in the proof-of-contact network",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "poi://status",
            "name": "Node Status",
            "description": "Current status of the proof-of-contact node and network",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "poi://proofs/{id}",
            "name": "Proof by ID",
            "description": "Retrieve a specific proof of contact by its UUID. Replace {id} with the proof UUID.",
            "mimeType": "application/json"
        }),
    ]
}

pub async fn read_resource(client: &ApiClient, uri: &str) -> Result<Value> {
    if uri == "poi://proofs/list" {
        let proofs = client.list_recent_proofs().await?;
        return Ok(json!({
            "uri": uri,
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&proofs)
                .unwrap_or_else(|_| "[]".to_string())
        }));
    }

    if uri == "poi://peers" {
        let peers = client.list_peers().await?;
        return Ok(json!({
            "uri": uri,
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&peers)
                .unwrap_or_else(|_| "[]".to_string())
        }));
    }

    if uri == "poi://status" {
        let status = client.get_network_status().await?;
        return Ok(json!({
            "uri": uri,
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&status)
                .unwrap_or_else(|_| "{}".to_string())
        }));
    }

    if let Some(proof_id) = uri.strip_prefix("poi://proofs/") {
        if !proof_id.is_empty() && proof_id != "list" {
            let proof = client.get_proof(proof_id).await?;
            return Ok(json!({
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&proof)
                    .unwrap_or_else(|_| "{}".to_string())
            }));
        }
    }

    warn!("Resource not found: {}", uri);
    Err(McpError::ResourceNotFound(uri.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::McpConfig;

    #[test]
    fn test_list_resources_count() {
        let resources = list_resources();
        assert_eq!(resources.len(), 4);
    }

    #[test]
    fn test_list_resources_contains_uris() {
        let resources = list_resources();
        let uris: Vec<&str> = resources
            .iter()
            .filter_map(|r| r.get("uri").and_then(|u| u.as_str()))
            .collect();
        assert!(uris.contains(&"poi://proofs/list"));
        assert!(uris.contains(&"poi://peers"));
        assert!(uris.contains(&"poi://status"));
        assert!(uris.contains(&"poi://proofs/{id}"));
    }

    #[test]
    fn test_read_resource_unknown_uri() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = McpConfig::default();
        let client = ApiClient::new(&config);
        let result = rt.block_on(read_resource(&client, "poi://unknown"));
        assert!(matches!(result, Err(McpError::ResourceNotFound(_))));
    }

    #[test]
    fn test_read_resource_invalid_uri() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = McpConfig::default();
        let client = ApiClient::new(&config);
        let result = rt.block_on(read_resource(&client, "invalid-uri"));
        assert!(matches!(result, Err(McpError::ResourceNotFound(_))));
    }

    #[test]
    fn test_resource_uri_format() {
        let resources = list_resources();
        for res in &resources {
            let uri = res.get("uri").and_then(|u| u.as_str()).unwrap();
            assert!(uri.starts_with("poi://"), "Resource URI should start with poi://");
            assert!(res.get("name").is_some(), "Resource should have a name");
            assert!(res.get("description").is_some(), "Resource should have a description");
            assert!(res.get("mimeType").is_some(), "Resource should have a mimeType");
        }
    }
}
