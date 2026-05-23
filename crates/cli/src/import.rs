use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use poi_core::ContactProof;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum ImportCommands {
    /// Import proofs from a file
    Proofs {
        #[arg(long)]
        file: String,
    },
}

pub async fn run(cmd: ImportCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        ImportCommands::Proofs { file } => {
            let file_path = PathBuf::from(&file);
            if !file_path.exists() {
                anyhow::bail!("File not found: {}", file_path.display());
            }
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("failed to read file {}", file_path.display()))?;
            let proofs: Vec<ContactProof> = serde_json::from_str(&content)
                .context("failed to parse proofs from JSON")?;
            if proofs.is_empty() {
                anyhow::bail!("No proofs found in file");
            }
            let pb = output::create_progress_bar(proofs.len() as u64, "Importing proofs...");
            let mut imported = 0usize;
            let mut errors = 0usize;
            for proof in &proofs {
                let proof_val = serde_json::to_value(proof)?;
                match client.import_proof(&proof_val).await {
                    Ok(_) => {
                        imported += 1;
                    }
                    Err(e) => {
                        errors += 1;
                        if !json {
                            output::print_warning(&format!(
                                "Failed to import proof {}: {}",
                                proof.id.0, e
                            ));
                        }
                    }
                }
                pb.inc(1);
            }
            pb.finish_with_message("Import complete");
            if json {
                output::print_json(&serde_json::json!({
                    "file": file,
                    "imported": imported,
                    "errors": errors,
                    "total": proofs.len(),
                }));
            } else {
                output::print_success(&format!(
                    "Imported {}/{} proofs from {}",
                    imported,
                    proofs.len(),
                    file_path.display()
                ));
                if errors > 0 {
                    output::print_warning(&format!("{} proofs failed to import", errors));
                }
            }
        }
    }
    Ok(())
}
