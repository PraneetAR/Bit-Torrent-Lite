use clap::{Parser, Subcommand};

/// A lightweight P2P file sharing app
#[derive(Parser)]
#[command(name = "bittorrent-lite")]
#[command(version = "1.0")]
#[command(about = "BitTorrent Lite - Educational P2P File Sharing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the P2P node and listen for peers on the local network
    Start,
    /// List available files from connected peers
    List,
    /// Request a specific file from the network
    Request {
        /// The name of the file you want to download
        file_name: String,
    },
}

#[tokio::main]
async fn main() {
    // Parse the command line arguments
    let cli = Cli::parse();

    // Route the command to the right logic
    match &cli.command {
        Commands::Start => {
            println!("🚀 Starting BitTorrent Lite node...");
            println!("(Network discovery and listening will be implemented here in Phase 2)");
        }
        Commands::List => {
            println!("🔍 Searching for available files on the network...");
            println!("(Peer querying will be implemented here later)");
        }
        Commands::Request { file_name } => {
            println!("📥 Requesting file: '{}'", file_name);
            println!("(File transfer logic will go here later)");
        }
    }
}