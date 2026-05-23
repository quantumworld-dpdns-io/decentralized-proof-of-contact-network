use clap::Parser;
use poi_node::NodeBuilder;

#[derive(Parser)]
#[command(
    name = "poi-node",
    about = "Decentralized Proof-of-Contact Network Node",
    version = "0.1.0"
)]
struct Cli {
    #[arg(short, long, default_value = "config.toml", help = "Path to configuration file")]
    config: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let node = match NodeBuilder::with_config(&cli.config) {
        Ok(builder) => builder.build()?,
        Err(e) => {
            eprintln!("Failed to load configuration from '{}': {e}", cli.config);
            std::process::exit(1);
        }
    };

    if let Err(e) = node.start() {
        eprintln!("Failed to start node: {e}");
        std::process::exit(1);
    }

    println!("poi-node started. Press Ctrl+C to stop.");
    node.wait_for_shutdown();

    println!("\nShutting down...");
    if let Err(e) = node.shutdown() {
        eprintln!("Shutdown error: {e}");
    }

    println!("Node stopped.");
    Ok(())
}
