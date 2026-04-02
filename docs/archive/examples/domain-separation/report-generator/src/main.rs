mod cli;
mod domain;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "report-generator")]
#[command(about = "Generate sales reports with multiple output formats")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Generate a sales report
    Generate {
        /// Input CSV file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Report format: json, csv, or markdown
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { input, output, format } => {
            cli::commands::generate(input, output, format)?;
        }
    }

    Ok(())
}
