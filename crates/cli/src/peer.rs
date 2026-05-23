use anyhow::Result;
use clap::Subcommand;
use serde_json::Value;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum PeerCommands {
    /// List connected peers
    List,
    /// Connect to a peer
    Connect {
        addr: String,
    },
    /// Disconnect a peer
    Disconnect {
        id: String,
    },
}

pub async fn run(cmd: PeerCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        PeerCommands::List => {
            let val = client.list_peers().await?;
            if json {
                output::print_json(&val);
            } else {
                if let Some(peers) = val.as_array() {
                    if peers.is_empty() {
                        output::print_colored("info", "No peers connected.");
                    } else {
                        let headers = &["ID", "Address", "Status"];
                        let rows: Vec<Vec<String>> = peers
                            .iter()
                            .map(|p| {
                                vec![
                                    p.get("id")
                                        .or_else(|| p.get("peer_id"))
                                        .map(|v| format!("{}", v))
                                        .unwrap_or_default(),
                                    p.get("address")
                                        .or_else(|| p.get("addr"))
                                        .map(|v| format!("{}", v))
                                        .unwrap_or_default(),
                                    p.get("status")
                                        .map(|v| format!("{}", v))
                                        .unwrap_or_else(|| "connected".into()),
                                ]
                            })
                            .collect();
                        output::print_table(headers, &rows);
                    }
                } else if let Some(arr) = val
                    .get("peers")
                    .or_else(|| val.get("data"))
                    .and_then(|v| v.as_array())
                {
                    if arr.is_empty() {
                        output::print_colored("info", "No peers connected.");
                    } else {
                        let headers = &["ID", "Address", "Status"];
                        let rows: Vec<Vec<String>> = arr
                            .iter()
                            .map(|p| {
                                vec![
                                    p.get("id").map(|v| format!("{}", v)).unwrap_or_default(),
                                    p.get("address")
                                        .or_else(|| p.get("addr"))
                                        .map(|v| format!("{}", v))
                                        .unwrap_or_default(),
                                    p.get("status")
                                        .map(|v| format!("{}", v))
                                        .unwrap_or_else(|| "connected".into()),
                                ]
                            })
                            .collect();
                        output::print_table(headers, &rows);
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
        PeerCommands::Connect { addr } => {
            let val = client.connect_peer(&addr).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_success(&format!("Connecting to peer: {}", addr));
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                }
            }
        }
        PeerCommands::Disconnect { id } => {
            let val = client.disconnect_peer(&id).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_success(&format!("Disconnected peer: {}", id));
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
