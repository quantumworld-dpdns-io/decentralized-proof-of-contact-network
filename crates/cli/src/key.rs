use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use poi_core::keypair::KeyPairExt;
use poi_core::KeyPair;

use crate::output;

#[derive(Subcommand)]
pub enum KeyCommands {
    /// Generate a new key pair
    Generate {
        #[arg(long)]
        output: Option<String>,
    },
    /// Show public key information
    Show,
    /// Export key to a file
    Export {
        path: String,
    },
    /// Import key from a file
    Import {
        path: String,
    },
}

fn default_key_path() -> PathBuf {
    PathBuf::from("./poi-key.pem")
}

pub async fn run(cmd: KeyCommands, json: bool) -> Result<()> {
    match cmd {
        KeyCommands::Generate { output } => {
            let key_path = output
                .map(PathBuf::from)
                .unwrap_or_else(default_key_path);
            if key_path.exists() {
                anyhow::bail!("Key file already exists at {}", key_path.display());
            }
            let kp = KeyPair::generate();
            let pem = kp.to_pem();
            std::fs::write(&key_path, &pem)
                .with_context(|| format!("failed to write key to {}", key_path.display()))?;
            if json {
                output::print_json(&serde_json::json!({
                    "path": key_path.to_string_lossy(),
                    "public_key": kp.public.0,
                }));
            } else {
                output::print_success(&format!(
                    "Key pair generated and saved to {}",
                    key_path.display()
                ));
                output::print_key_value("Public Key", &kp.public.0);
                output::print_key_value("Path", &key_path.to_string_lossy());
            }
        }
        KeyCommands::Show => {
            let key_path = default_key_path();
            if !key_path.exists() {
                anyhow::bail!(
                    "No key file found at {}. Generate one with 'poi key generate'.",
                    key_path.display()
                );
            }
            let pem = std::fs::read_to_string(&key_path)
                .with_context(|| format!("failed to read key from {}", key_path.display()))?;
            let kp = KeyPair::from_pem(&pem)?;
            if json {
                output::print_json(&serde_json::json!({
                    "public_key": kp.public.0,
                    "path": key_path.to_string_lossy(),
                }));
            } else {
                output::print_key_value("Public Key", &kp.public.0);
                output::print_key_value("Path", &key_path.to_string_lossy());
            }
        }
        KeyCommands::Export { path } => {
            let src = default_key_path();
            if !src.exists() {
                anyhow::bail!(
                    "No key file found at {}. Generate one with 'poi key generate'.",
                    src.display()
                );
            }
            let pem = std::fs::read_to_string(&src)
                .with_context(|| format!("failed to read key from {}", src.display()))?;
            let dst = PathBuf::from(&path);
            std::fs::write(&dst, &pem)
                .with_context(|| format!("failed to export key to {}", dst.display()))?;
            if json {
                output::print_json(&serde_json::json!({
                    "source": src.to_string_lossy(),
                    "destination": dst.to_string_lossy(),
                    "status": "exported"
                }));
            } else {
                output::print_success(&format!(
                    "Key exported from {} to {}",
                    src.display(),
                    dst.display()
                ));
            }
        }
        KeyCommands::Import { path } => {
            let src = PathBuf::from(&path);
            if !src.exists() {
                anyhow::bail!("Key file not found at {}", src.display());
            }
            let pem = std::fs::read_to_string(&src)
                .with_context(|| format!("failed to read key from {}", src.display()))?;
            let kp = KeyPair::from_pem(&pem)
                .context("invalid key file format")?;
            let dst = default_key_path();
            if dst.exists() {
                anyhow::bail!(
                    "Key file already exists at {}. Delete it first.",
                    dst.display()
                );
            }
            std::fs::write(&dst, &pem)
                .with_context(|| format!("failed to write key to {}", dst.display()))?;
            if json {
                output::print_json(&serde_json::json!({
                    "source": src.to_string_lossy(),
                    "destination": dst.to_string_lossy(),
                    "public_key": kp.public.0,
                    "status": "imported"
                }));
            } else {
                output::print_success(&format!(
                    "Key imported from {} to {}",
                    src.display(),
                    dst.display()
                ));
                output::print_key_value("Public Key", &kp.public.0);
            }
        }
    }
    Ok(())
}
