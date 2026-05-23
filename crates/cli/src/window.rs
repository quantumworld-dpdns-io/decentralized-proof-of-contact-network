use anyhow::Result;
use clap::Subcommand;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum WindowCommands {
    /// List orbital windows
    List,
    /// Create a new orbital window
    Create {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        window_type: Option<String>,
    },
    /// Get active orbital window
    Active,
}

pub async fn run(cmd: WindowCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        WindowCommands::List => {
            let val = client.list_windows().await?;
            if json {
                output::print_json(&val);
            } else {
                let windows = val
                    .as_array()
                    .or_else(|| val.get("windows").and_then(|v| v.as_array()))
                    .cloned()
                    .unwrap_or_default();
                if windows.is_empty() {
                    output::print_colored("info", "No orbital windows found.");
                } else {
                    let headers = &["ID", "Start", "End", "Type"];
                    let rows: Vec<Vec<String>> = windows
                        .iter()
                        .map(|w| {
                            vec![
                                w.get("id")
                                    .or_else(|| w.get("window_id"))
                                    .map(|v| format!("{}", v))
                                    .unwrap_or_default(),
                                w.get("start_time")
                                    .or_else(|| w.get("start"))
                                    .map(|v| format!("{}", v))
                                    .unwrap_or_default(),
                                w.get("end_time")
                                    .or_else(|| w.get("end"))
                                    .map(|v| format!("{}", v))
                                    .unwrap_or_default(),
                                w.get("window_type")
                                    .or_else(|| w.get("type"))
                                    .map(|v| format!("{}", v))
                                    .unwrap_or_else(|| "Standard".into()),
                            ]
                        })
                        .collect();
                    output::print_table(headers, &rows);
                }
            }
        }
        WindowCommands::Create {
            start,
            end,
            window_type,
        } => {
            let val = client.create_window(&start, &end, window_type.as_deref()).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_success("Orbital window created");
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                }
            }
        }
        WindowCommands::Active => {
            let val = client.active_window().await?;
            if json {
                output::print_json(&val);
            } else if val.is_null() {
                output::print_colored("info", "No active orbital window.");
            } else {
                output::print_colored("success", "Active Orbital Window");
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
