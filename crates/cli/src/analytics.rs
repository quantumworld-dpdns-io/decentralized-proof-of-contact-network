use anyhow::Result;
use clap::Subcommand;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum AnalyticsCommands {
    /// Run an analytics query
    Run {
        query: String,
    },
    /// Get analytics summary
    Summary,
}

pub async fn run(cmd: AnalyticsCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        AnalyticsCommands::Run { query } => {
            let val = client.analytics_run(&query).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_colored("info", &format!("Analytics: {}", query));
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
        AnalyticsCommands::Summary => {
            let val = client.analytics_summary().await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_colored("info", "Analytics Summary");
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
    }
    Ok(())
}
