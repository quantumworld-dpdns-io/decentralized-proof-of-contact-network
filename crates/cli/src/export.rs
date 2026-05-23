use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use poi_core::ContactProof;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum ExportCommands {
    /// Export proofs to a file
    Proofs {
        #[arg(long)]
        format: String,
        #[arg(long)]
        output: String,
    },
}

pub async fn run(cmd: ExportCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        ExportCommands::Proofs { format, output } => {
            let val = client.fetch_all_proofs().await?;
            let proofs: Vec<ContactProof> = if let Some(proofs) = val.get("proofs").and_then(|p| p.as_array()) {
                serde_json::from_value(serde_json::Value::Array(proofs.clone()))?
            } else if val.is_array() {
                serde_json::from_value(val)?
            } else {
                serde_json::from_value(val)?
            };

            let output_path = PathBuf::from(&output);

            match format.to_lowercase().as_str() {
                "json" => {
                    let content = serde_json::to_string_pretty(&proofs)
                        .context("failed to serialize proofs to JSON")?;
                    std::fs::write(&output_path, &content)
                        .with_context(|| format!("failed to write to {}", output_path.display()))?;
                    if json {
                        output::print_json(&serde_json::json!({
                            "format": "json",
                            "path": output,
                            "count": proofs.len(),
                            "status": "exported"
                        }));
                    } else {
                        output::print_success(&format!(
                            "Exported {} proofs to {} (JSON)",
                            proofs.len(),
                            output_path.display()
                        ));
                    }
                }
                "csv" => {
                    let mut wtr = csv::Writer::from_path(&output_path)
                        .with_context(|| format!("failed to create CSV writer at {}", output_path.display()))?;
                    wtr.write_record(&[
                        "id",
                        "proving_node",
                        "target_node",
                        "timestamp",
                        "proof_purpose",
                        "confidence_score",
                        "protocol_version",
                        "signature",
                        "window_start",
                        "window_end",
                        "window_type",
                    ])
                    .context("failed to write CSV header")?;
                    for proof in &proofs {
                        wtr.write_record(&[
                            proof.id.0.to_string(),
                            proof.proving_node.to_string(),
                            proof.target_node.to_string(),
                            proof.timestamp.to_rfc3339(),
                            proof.metadata.proof_purpose.clone(),
                            proof.metadata.confidence_score.to_string(),
                            proof.metadata.protocol_version.clone(),
                            proof.signature.0.clone(),
                            proof.orbital_window.start_time.to_rfc3339(),
                            proof.orbital_window.end_time.to_rfc3339(),
                            format!("{:?}", proof.orbital_window.window_type),
                        ])
                        .context("failed to write CSV record")?;
                    }
                    wtr.flush().context("failed to flush CSV writer")?;
                    if json {
                        output::print_json(&serde_json::json!({
                            "format": "csv",
                            "path": output,
                            "count": proofs.len(),
                            "status": "exported"
                        }));
                    } else {
                        output::print_success(&format!(
                            "Exported {} proofs to {} (CSV)",
                            proofs.len(),
                            output_path.display()
                        ));
                    }
                }
                "parquet" => {
                    anyhow::bail!("Parquet export is not yet implemented. Use 'json' or 'csv' format.");
                }
                other => {
                    anyhow::bail!(
                        "Unsupported format: '{}'. Supported formats: json, csv, parquet",
                        other
                    );
                }
            }
        }
    }
    Ok(())
}
