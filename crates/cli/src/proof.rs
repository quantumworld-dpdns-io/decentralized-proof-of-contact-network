use anyhow::Result;
use clap::Subcommand;
use poi_core::ContactProof;
use serde_json::Value;

use crate::client::ApiClient;
use crate::output;

fn extract_proof(val: Value) -> Result<ContactProof> {
    Ok(serde_json::from_value(val)?)
}

fn extract_proofs(val: Value) -> Result<Vec<ContactProof>> {
    if let Some(proofs) = val.get("proofs").and_then(|p| p.as_array()) {
        return Ok(serde_json::from_value(serde_json::Value::Array(proofs.clone()))?);
    }
    if val.is_array() {
        return Ok(serde_json::from_value(val)?);
    }
    Ok(serde_json::from_value(val)?)
}

fn display_proof(proof: &ContactProof, json: bool) {
    if json {
        output::print_json(proof);
    } else {
        output::print_key_value("ID", &proof.id.0.to_string());
        output::print_key_value("Proving Node", &proof.proving_node.to_string());
        output::print_key_value("Target Node", &proof.target_node.to_string());
        output::print_key_value("Timestamp", &proof.timestamp.to_rfc3339());
        output::print_key_value(
            "Window",
            &format!(
                "{} - {} ({:?})",
                proof.orbital_window.start_time.to_rfc3339(),
                proof.orbital_window.end_time.to_rfc3339(),
                proof.orbital_window.window_type,
            ),
        );
        output::print_key_value("Signature", &proof.signature.0.chars().take(32).collect::<String>());
        output::print_key_value(
            "Purpose",
            &proof.metadata.proof_purpose,
        );
        output::print_key_value(
            "Confidence",
            &format!("{:.2}", proof.metadata.confidence_score),
        );
        output::print_key_value(
            "Protocol",
            &proof.metadata.protocol_version,
        );
        if let Some(pos) = proof.metadata.chain_position {
            output::print_key_value("Chain Position", &pos.to_string());
        }
    }
}

#[derive(Subcommand)]
pub enum ProofCommands {
    /// Create a new contact proof
    Create {
        #[arg(long)]
        target: String,
        #[arg(long)]
        window: String,
        #[arg(long)]
        purpose: Option<String>,
    },
    /// Get a proof by ID
    Get {
        id: String,
    },
    /// List proofs
    List {
        #[arg(long, default_value = "10")]
        limit: u64,
        #[arg(long, default_value = "0")]
        offset: u64,
    },
    /// Verify a proof
    Verify {
        id: String,
    },
    /// Search proofs
    Search {
        query: String,
    },
}

pub async fn run(cmd: ProofCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        ProofCommands::Create {
            target,
            window,
            purpose,
        } => {
            let val = client.create_proof(&target, &window, purpose.as_deref()).await?;
            let proof = extract_proof(val)?;
            if json {
                output::print_json(&proof);
            } else {
                output::print_success("Proof created");
                display_proof(&proof, false);
            }
        }
        ProofCommands::Get { id } => {
            let val = client.get_proof(&id).await?;
            let proof = extract_proof(val)?;
            display_proof(&proof, json);
        }
        ProofCommands::List { limit, offset } => {
            let val = client.list_proofs(Some(limit), Some(offset)).await?;
            let proofs = extract_proofs(val)?;
            if json {
                output::print_json(&proofs);
            } else {
                let headers = &["ID", "Prover", "Target", "Timestamp", "Purpose"];
                let rows: Vec<Vec<String>> = proofs
                    .iter()
                    .map(|p| {
                        vec![
                            p.id.0.to_string(),
                            p.proving_node.to_string(),
                            p.target_node.to_string(),
                            p.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                            p.metadata.proof_purpose.clone(),
                        ]
                    })
                    .collect();
                output::print_table(headers, &rows);
            }
        }
        ProofCommands::Verify { id } => {
            let val = client.verify_proof(&id).await?;
            if json {
                output::print_json(&val);
            } else {
                let all_passed = val
                    .get("all_passed")
                    .or_else(|| val.get("success"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if all_passed {
                    output::print_success("Proof verification passed");
                } else {
                    output::print_error("Proof verification failed");
                }
                for (key, value) in val.as_object().unwrap_or(&serde_json::Map::new()) {
                    output::print_key_value(key, &format!("{}", value));
                }
            }
        }
        ProofCommands::Search { query } => {
            let val = client.search_proofs(&query).await?;
            let proofs = extract_proofs(val)?;
            if json {
                output::print_json(&proofs);
            } else {
                let headers = &["ID", "Prover", "Target", "Timestamp", "Purpose"];
                let rows: Vec<Vec<String>> = proofs
                    .iter()
                    .map(|p| {
                        vec![
                            p.id.0.to_string(),
                            p.proving_node.to_string(),
                            p.target_node.to_string(),
                            p.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                            p.metadata.proof_purpose.clone(),
                        ]
                    })
                    .collect();
                output::print_table(headers, &rows);
            }
        }
    }
    Ok(())
}
