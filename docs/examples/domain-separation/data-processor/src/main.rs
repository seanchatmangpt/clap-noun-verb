mod cli;
mod domain;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "data-processor")]
#[command(about = "Transform CSV data with domain logic separation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Process and transform input data
    Process {
        /// Input CSV file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output CSV file path
        #[arg(short, long)]
        output: PathBuf,

        /// Scale factor for values
        #[arg(long)]
        scale: Option<f64>,

        /// Score multiplier
        #[arg(long)]
        multiplier: Option<f64>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { input, output, scale, multiplier } => {
            cli::commands::process(input, output, scale, multiplier)?;
        }
    }

    Ok(())
}
