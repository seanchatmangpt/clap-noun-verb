mod cli;
mod domain;

use clap::Parser;

#[derive(Parser)]
#[command(name = "api-client")]
#[command(about = "API client with circuit breaker and retry logic")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Query the API
    Query {
        /// Base URL of the API
        #[arg(short, long, default_value = "http://localhost:8080")]
        url: String,

        /// API endpoint
        #[arg(short, long, default_value = "/search")]
        endpoint: String,

        /// Search query
        #[arg(short, long)]
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "10")]
        max: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query { url, endpoint, query, max } => {
            cli::commands::query(url, endpoint, query, max).await?;
        }
    }

    Ok(())
}
