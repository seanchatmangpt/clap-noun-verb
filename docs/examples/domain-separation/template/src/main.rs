mod cli;
mod domain;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "my-cli")]
#[command(about = "Template CLI with domain separation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Process input file
    Process {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { input } => {
            cli::commands::process_file(input)?;
        }
    }

    Ok(())
}
