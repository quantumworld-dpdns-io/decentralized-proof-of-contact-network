use anyhow::Result;
use clap::Subcommand;

use crate::client::ApiClient;
use crate::output;

#[derive(Subcommand)]
pub enum AiCommands {
    /// Ask a question to the AI
    Query {
        #[arg(allow_hyphen_values = true)]
        question: String,
    },
    /// Analyze a proof using AI
    Analyze {
        proof_id: String,
    },
    /// Detect anomalies using AI
    Anomalies,
    /// Summarize using AI
    Summarize,
}

pub async fn run(cmd: AiCommands, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        AiCommands::Query { question } => {
            let val = client.ai_query(&question).await?;
            if json {
                output::print_json(&val);
            } else {
                if let Some(answer) = val
                    .get("answer")
                    .or_else(|| val.get("response"))
                    .or_else(|| val.get("result"))
                    .and_then(|v| v.as_str())
                {
                    println!("{}", answer);
                } else {
                    output::print_key_value("Response", &format!("{}", val));
                }
            }
        }
        AiCommands::Analyze { proof_id } => {
            let val = client.ai_analyze(&proof_id).await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_colored("info", &format!("AI Analysis for proof: {}", proof_id));
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
        AiCommands::Anomalies => {
            let val = client.ai_anomalies().await?;
            if json {
                output::print_json(&val);
            } else {
                output::print_colored("info", "AI Anomaly Detection Results");
                if let Some(obj) = val.as_object() {
                    for (key, value) in obj {
                        output::print_key_value(key, &format!("{}", value));
                    }
                } else {
                    output::print_json(&val);
                }
            }
        }
        AiCommands::Summarize => {
            let val = client.ai_summarize().await?;
            if json {
                output::print_json(&val);
            } else {
                if let Some(summary) = val
                    .get("summary")
                    .or_else(|| val.get("response"))
                    .or_else(|| val.get("result"))
                    .and_then(|v| v.as_str())
                {
                    println!("{}", summary);
                } else {
                    output::print_key_value("Summary", &format!("{}", val));
                }
            }
        }
    }
    Ok(())
}
