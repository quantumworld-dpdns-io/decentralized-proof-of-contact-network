use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use poi_core::CoreConfig;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize a configuration file
    Init {
        #[arg(long)]
        path: Option<String>,
    },
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },
}

pub async fn run(cmd: ConfigCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        ConfigCommands::Init { path } => {
            let config_path = path
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("./poi-config.yaml"));
            if config_path.exists() {
                anyhow::bail!(
                    "Config file already exists at {}",
                    config_path.display()
                );
            }
            let config = CoreConfig::default();
            let content = serde_yaml::to_string(&config)
                .context("failed to serialize default config")?;
            std::fs::write(&config_path, &content)
                .with_context(|| format!("failed to write config to {}", config_path.display()))?;
            if json {
                output::print_json(&serde_json::json!({
                    "path": config_path.to_string_lossy(),
                    "status": "created"
                }));
            } else {
                output::print_success(&format!(
                    "Config initialized at {}",
                    config_path.display()
                ));
            }
        }
        ConfigCommands::Show => {
            let val = client.config_show().await?;
            if json {
                output::print_json(&val);
            } else {
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
        ConfigCommands::Set { key, value } => {
            let val = client.config_set(&key, &value).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_success(&format!("Config '{}' set to '{}'", key, value));
                if let Some(obj) = val.as_object() {
                    for (k, v) in obj {
                        output::print_key_value(k, &format!("{}", v));
                    }
                }
            }
        }
    }
    Ok(())
}
