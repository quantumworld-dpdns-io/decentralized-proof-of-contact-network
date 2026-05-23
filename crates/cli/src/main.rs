use anyhow::Result;
use clap::{Parser, Subcommand};
use poi_cli::*;

#[derive(Parser)]
#[command(name = "poi", about = "Decentralized Proof-of-Contact Network CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(global = true, long, default_value = "http://localhost:3000")]
    api_url: String,

    #[arg(global = true, long)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Proof(proof::ProofCommands),
    #[command(subcommand)]
    Node(node::NodeCommands),
    #[command(subcommand)]
    Peer(peer::PeerCommands),
    #[command(subcommand)]
    Window(window::WindowCommands),
    #[command(subcommand)]
    Config(config::ConfigCommands),
    #[command(subcommand)]
    Key(key::KeyCommands),
    #[command(subcommand)]
    Analytics(analytics::AnalyticsCommands),
    #[command(subcommand)]
    Ai(ai::AiCommands),
    #[command(subcommand)]
    Export(export::ExportCommands),
    #[command(subcommand)]
    Import(import::ImportCommands),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = ApiClient::new(cli.api_url);

    let result = match cli.command {
        Command::Proof(cmd) => proof::run(cmd, &client, cli.json).await,
        Command::Node(cmd) => node::run(cmd, &client, cli.json).await,
        Command::Peer(cmd) => peer::run(cmd, &client, cli.json).await,
        Command::Window(cmd) => window::run(cmd, &client, cli.json).await,
        Command::Config(cmd) => config::run(cmd, &client, cli.json).await,
        Command::Key(cmd) => key::run(cmd, cli.json).await,
        Command::Analytics(cmd) => analytics::run(cmd, &client, cli.json).await,
        Command::Ai(cmd) => ai::run(cmd, &client, cli.json).await,
        Command::Export(cmd) => export::run(cmd, &client, cli.json).await,
        Command::Import(cmd) => import::run(cmd, &client, cli.json).await,
    };

    if let Err(e) = &result {
        output::print_error(&format!("{}", e));
    }

    result
}
