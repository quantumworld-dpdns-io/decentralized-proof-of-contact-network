use serde_json::{json, Value};

use crate::error::{McpError, Result};

fn verify_proof_prompt() -> Value {
    json!({
        "name": "verify-proof",
        "description": "Verify a proof of contact and get a detailed analysis of its validity",
        "arguments": [
            {
                "name": "proof_id",
                "description": "UUID of the proof to verify",
                "required": true
            }
        ]
    })
}

fn analyze_network_prompt() -> Value {
    json!({
        "name": "analyze-network",
        "description": "Analyze the current state of the proof-of-contact network and identify patterns",
        "arguments": [
            {
                "name": "timeframe",
                "description": "Time period to analyze (hour, day, week, month, all)",
                "required": false,
                "default": "day"
            },
            {
                "name": "focus",
                "description": "Area of analysis: general, peers, proofs, anomalies",
                "required": false,
                "default": "general"
            }
        ]
    })
}

fn investigate_anomaly_prompt() -> Value {
    json!({
        "name": "investigate-anomaly",
        "description": "Investigate a potential anomaly in the network, such as unusual proof patterns or suspicious peer behavior",
        "arguments": [
            {
                "name": "anomaly_type",
                "description": "Type of anomaly (unusual_proof_rate, unexpected_peer, signature_irregularity, window_violation)",
                "required": true
            },
            {
                "name": "target_id",
                "description": "Node ID or proof ID associated with the anomaly",
                "required": true
            },
            {
                "name": "severity",
                "description": "Perceived severity of the anomaly (low, medium, high, critical)",
                "required": false,
                "default": "medium"
            }
        ]
    })
}

pub fn list_prompts() -> Vec<Value> {
    vec![verify_proof_prompt(), analyze_network_prompt(), investigate_anomaly_prompt()]
}

pub fn get_prompt(name: &str, args: &Value) -> Result<Value> {
    match name {
        "verify-proof" => render_verify_proof(args),
        "analyze-network" => render_analyze_network(args),
        "investigate-anomaly" => render_investigate_anomaly(args),
        _ => Err(McpError::InvalidRequest(format!("Prompt not found: {}", name))),
    }
}

fn render_verify_proof(args: &Value) -> Result<Value> {
    let proof_id = args
        .get("proof_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: proof_id".into()))?;

    let messages = vec![
        json!({
            "role": "user",
            "content": {
                "type": "text",
                "text": format!(
                    "Please verify proof of contact {} and provide a detailed analysis.\n\n\
                     Use the verify_proof tool to check the proof's cryptographic signature, \
                     timestamp freshness, and orbital window compliance. Then summarize the \
                     findings in a clear, actionable format.\n\n\
                     Include:\n\
                     1. Overall verification status\n\
                     2. Signature validity\n\
                     3. Timestamp analysis\n\
                     4. Orbital window compliance\n\
                     5. Any warnings or recommendations",
                    proof_id
                )
            }
        }),
    ];

    Ok(json!({
        "description": format!("Verify proof of contact {}", proof_id),
        "messages": messages
    }))
}

fn render_analyze_network(args: &Value) -> Result<Value> {
    let timeframe = args
        .get("timeframe")
        .and_then(|v| v.as_str())
        .unwrap_or("day");
    let focus = args
        .get("focus")
        .and_then(|v| v.as_str())
        .unwrap_or("general");

    let focus_text = match focus {
        "peers" => "Analyze peer connectivity, distribution, and behavior patterns. Identify \
                     any peers with unusual activity or connection patterns.",
        "proofs" => "Analyze proof creation and verification patterns. Look for trends in \
                     proof rates, confidence scores, and verification success rates.",
        "anomalies" => "Search for anomalous patterns in the network including unusual proof \
                        rates, unexpected peers, signature irregularities, or window violations.",
        _ => "Provide a comprehensive overview of the network's current state including peer \
              counts, proof activity, verification rates, and any notable patterns or events.",
    };

    let messages = vec![
        json!({
            "role": "user",
            "content": {
                "type": "text",
                "text": format!(
                    "Please analyze the proof-of-contact network over the last {} timeframe.\n\n\
                     {}\n\n\
                     Use the available tools (get_network_status, list_peers, get_analytics, \
                     search_proofs) to gather data and provide insights.\n\n\
                     Include:\n\
                     1. Key metrics and trends\n\
                     2. Notable observations\n\
                     3. Health assessment\n\
                     4. Recommendations if applicable",
                    timeframe, focus_text
                )
            }
        }),
    ];

    Ok(json!({
        "description": format!("Analyze network state over the last {}", timeframe),
        "messages": messages
    }))
}

fn render_investigate_anomaly(args: &Value) -> Result<Value> {
    let anomaly_type = args
        .get("anomaly_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: anomaly_type".into()))?;
    let target_id = args
        .get("target_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing required argument: target_id".into()))?;
    let _severity = args
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");

    let investigation_steps = match anomaly_type {
        "unusual_proof_rate" => format!(
            "Investigate unusual proof rate for target {}.\n\n\
             Steps:\n\
             1. Search for recent proofs involving this node\n\
             2. Check proof creation frequency and patterns\n\
             3. Verify proof signatures and timestamps\n\
             4. Compare activity against historical baselines",
            target_id
        ),
        "unexpected_peer" => format!(
            "Investigate unexpected peer {}.\n\n\
             Steps:\n\
             1. Verify peer identity and credentials\n\
             2. Check peer's proof history and reputation\n\
             3. Analyze connection patterns\n\
             4. Cross-reference with known peer database",
            target_id
        ),
        "signature_irregularity" => format!(
            "Investigate signature irregularity for {}.\n\n\
             Steps:\n\
             1. Retrieve and verify all proofs associated with this entity\n\
             2. Check for signature reuse or patterns\n\
             3. Verify public key integrity\n\
             4. Compare against expected cryptographic standards",
            target_id
        ),
        "window_violation" => format!(
            "Investigate orbital window violation for {}.\n\n\
             Steps:\n\
             1. Check orbital window parameters\n\
             2. Verify timestamps against window bounds\n\
             3. Analyze window type and duration compliance\n\
             4. Check for related proofs that may indicate pattern",
            target_id
        ),
        _ => format!(
            "Investigate anomaly of type '{}' for target {}.\n\n\
             Steps:\n\
             1. Gather all available data about the target\n\
             2. Check proofs, peers, and network status\n\
             3. Identify any irregularities or patterns\n\
             4. Provide assessment and recommendations",
            anomaly_type, target_id
        ),
    };

    let messages = vec![
        json!({
            "role": "user",
            "content": {
                "type": "text",
                "text": format!(
                    "Anomaly investigation required.\n\n\
                     Type: {}\n\
                     Target: {}\n\n\
                     {}",
                    anomaly_type, target_id, investigation_steps
                )
            }
        }),
    ];

    Ok(json!({
        "description": format!("Investigate {} anomaly involving {}", anomaly_type, target_id),
        "messages": messages
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_prompts_count() {
        let prompts = list_prompts();
        assert_eq!(prompts.len(), 3);
    }

    #[test]
    fn test_list_prompts_contains_names() {
        let prompts = list_prompts();
        let names: Vec<&str> = prompts
            .iter()
            .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"verify-proof"));
        assert!(names.contains(&"analyze-network"));
        assert!(names.contains(&"investigate-anomaly"));
    }

    #[test]
    fn test_get_prompt_verify_proof() {
        let result = get_prompt("verify-proof", &json!({"proof_id": "abc-123"}));
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert_eq!(prompt["description"], "Verify proof of contact abc-123");
        assert!(prompt["messages"].is_array());
        assert!(!prompt["messages"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_get_prompt_verify_proof_missing_arg() {
        let result = get_prompt("verify-proof", &json!({}));
        assert!(matches!(result, Err(McpError::InvalidRequest(_))));
    }

    #[test]
    fn test_get_prompt_unknown() {
        let result = get_prompt("nonexistent", &json!({}));
        assert!(matches!(result, Err(McpError::InvalidRequest(_))));
    }

    #[test]
    fn test_get_prompt_analyze_network_defaults() {
        let result = get_prompt("analyze-network", &json!({}));
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(prompt["description"].as_str().unwrap().contains("day"));
    }

    #[test]
    fn test_get_prompt_investigate_anomaly() {
        let result = get_prompt(
            "investigate-anomaly",
            &json!({
                "anomaly_type": "unusual_proof_rate",
                "target_id": "node-456"
            }),
        );
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(prompt["description"]
            .as_str()
            .unwrap()
            .contains("unusual_proof_rate"));
    }

    #[test]
    fn test_get_prompt_investigate_anomaly_missing_args() {
        let result = get_prompt("investigate-anomaly", &json!({}));
        assert!(matches!(result, Err(McpError::InvalidRequest(_))));

        let result = get_prompt("investigate-anomaly", &json!({"anomaly_type": "test"}));
        assert!(matches!(result, Err(McpError::InvalidRequest(_))));
    }

    #[test]
    fn test_all_prompts_have_required_structure() {
        let prompts = list_prompts();
        for prompt in &prompts {
            assert!(prompt.get("name").is_some(), "Prompt missing name");
            assert!(prompt.get("description").is_some(), "Prompt missing description");
            assert!(prompt.get("arguments").is_some(), "Prompt missing arguments");
        }
    }
}
