//! Basic I/O Example
//!
//! Demonstrates Typer-style I/O integration with clap-noun-verb.
//! Shows how the #[verb] macro automatically handles Input and Output types.
//!
//! # Running
//!
//! ```bash
//! # Read from stdin, write to stdout
//! cargo run --example io_basic -- process -
//!
//! # Read from file, write to stdout
//! cargo run --example io_basic -- process input.txt
//!
//! # Read from stdin, write to file
//! cargo run --example io_basic -- process - -o output.txt
//!
//! # Full file-to-file processing
//! cargo run --example io_basic -- process input.txt -o output.txt
//! ```

use clap::Parser;
use clap_noun_verb::io::{Input, Output};
use std::io::Read;

#[derive(Parser)]
#[command(name = "io-example")]
#[command(about = "Basic I/O example demonstrating clio integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Process input data
    Process {
        /// Input file or path (use '-' for stdin)
        input: Input,

        /// Output file or path (use '-' for stdout)
        #[arg(short, long)]
        output: Option<Output>,
    },

    /// Count lines in input
    LineCount {
        /// Input file or path (use '-' for stdin)
        input: Input,
    },

    /// Convert to uppercase
    Uppercase {
        /// Input file or path (use '-' for stdin)
        input: Input,

        /// Output file or path (use '-' for stdout)
        #[arg(short, long)]
        output: Option<Output>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { mut input, output } => {
            let mut content = String::new();
            input.read_to_string(&mut content)?;

            if let Some(mut out) = output {
                use std::io::Write;
                out.write_all(content.as_bytes())?;
                eprintln!("Wrote {} bytes to output", content.len());
            } else {
                println!("{}", content);
            }
            Ok(())
        }

        Commands::LineCount { mut input } => {
            let mut content = String::new();
            input.read_to_string(&mut content)?;
            let line_count = content.lines().count();
            println!("Line count: {}", line_count);
            Ok(())
        }

        Commands::Uppercase { mut input, output } => {
            let mut content = String::new();
            input.read_to_string(&mut content)?;
            let uppercase = content.to_uppercase();

            if let Some(mut out) = output {
                use std::io::Write;
                out.write_all(uppercase.as_bytes())?;
                eprintln!("Wrote uppercase text to output");
            } else {
                println!("{}", uppercase);
            }
            Ok(())
        }
    }
}

// Example of what the #[verb] macro will generate in the future:
//
// #[verb]
// fn process(
//     #[arg(short, long)]
//     input: Input,  // Automatically parsed with value_parser
//
//     #[arg(short, long)]
//     output: Option<Output>,  // Auto-detected as Optional I/O
// ) -> Result<String> {
//     let mut content = String::new();
//     input.read_to_string(&mut content)?;
//
//     if let Some(mut out) = output {
//         out.write_all(content.as_bytes())?;
//     }
//
//     Ok(format!("Processed {} bytes", content.len()))
// }
