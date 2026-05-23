use anyhow::Result;
use clap::Subcommand;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum NodeCommands {
    /// Get node information
    Info,
    /// Start the node
    Start {
        #[arg(long)]
        config: Option<String>,
    },
    /// Stop the node
    Stop,
    /// Get node status
    Status,
}

pub async fn run(cmd: NodeCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        NodeCommands::Info => {
            let val = client.node_info().await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_colored("info", "Node Information");
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
        NodeCommands::Start { config } => {
            let val = client.node_start(config.as_deref()).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_success("Node start command sent");
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                }
            }
        }
        NodeCommands::Stop => {
            let val = client.node_stop().await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_success("Node stop command sent");
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                }
            }
        }
        NodeCommands::Status => {
            let val = client.node_status().await?;
            if json {
                output::print_json(&val);
            } else {
                let running = val
                    .get("running")
                    .or_else(|| val.get("status"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if running {
                    output::print_success("Node is running");
                } else {
                    output::print_error("Node is stopped");
                }
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                }
            }
        }
    }
    Ok(())
}
