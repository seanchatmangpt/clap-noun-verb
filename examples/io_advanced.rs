//! Advanced I/O Example
//!
//! Demonstrates advanced I/O patterns using clio and clap-noun-verb:
//! - Stream processing with the IoPipeline
//! - Error handling with IoError
//! - Multiple inputs handling
//! - Type detection and introspection
//!
//! # Running
//!
//! ```bash
//! # Process with transformation
//! echo "hello world" | cargo run --example io_advanced -- transform
//!
//! # Merge multiple files
//! cargo run --example io_advanced -- merge file1.txt file2.txt -o merged.txt
//!
//! # Validate I/O types
//! cargo run --example io_advanced -- inspect
//! ```

use clap::Parser;
use clap_noun_verb::io::{pipeline, Input, Output, IoType, IoTypeRegistry};
use std::io::{Read, Write};

#[derive(Parser)]
#[command(name = "io-advanced")]
#[command(about = "Advanced I/O patterns with clap-noun-verb")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Transform input through a filter
    Transform {
        /// Transformation type
        #[arg(short, long, default_value = "uppercase")]
        kind: TransformKind,

        /// Input file or path (use '-' for stdin)
        #[arg(default_value = "-")]
        input: Input,

        /// Output file or path (use '-' for stdout)
        #[arg(short, long)]
        output: Option<Output>,
    },

    /// Merge multiple input files
    Merge {
        /// Input files to merge
        inputs: Vec<Input>,

        /// Output file or path (use '-' for stdout)
        #[arg(short, long, default_value = "-")]
        output: Output,
    },

    /// Inspect available I/O types
    Inspect,

    /// Benchmark I/O throughput
    Benchmark {
        /// Input file for benchmarking
        #[arg(default_value = "-")]
        input: Input,

        /// Number of iterations
        #[arg(short, long, default_value = "1")]
        iterations: u32,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum TransformKind {
    /// Convert to uppercase
    Uppercase,
    /// Convert to lowercase
    Lowercase,
    /// Reverse lines
    Reverse,
    /// Sort lines
    Sort,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transform {
            kind,
            mut input,
            output,
        } => {
            let mut content = String::new();
            input.read_to_string(&mut content)?;

            let transformed = match kind {
                TransformKind::Uppercase => content.to_uppercase(),
                TransformKind::Lowercase => content.to_lowercase(),
                TransformKind::Reverse => content.lines().rev().collect::<Vec<_>>().join("\n"),
                TransformKind::Sort => {
                    let mut lines: Vec<_> = content.lines().collect();
                    lines.sort();
                    lines.join("\n")
                }
            };

            if let Some(mut out) = output {
                out.write_all(transformed.as_bytes())?;
            } else {
                println!("{}", transformed);
            }
            Ok(())
        }

        Commands::Merge { inputs, mut output } => {
            for (idx, mut input) in inputs.into_iter().enumerate() {
                let mut content = String::new();
                input.read_to_string(&mut content)?;
                output.write_all(content.as_bytes())?;
                if idx > 0 {
                    output.write_all(b"\n")?;
                }
            }
            eprintln!("Successfully merged {} files", inputs.len());
            Ok(())
        }

        Commands::Inspect => {
            let registry = IoTypeRegistry::new();
            println!("Registered I/O Types:");
            println!("====================");
            for (name, io_type) in registry.list_types() {
                println!("\n{}: {:?}", name, io_type);
                println!("  ValueParser: {}", io_type.value_parser_expr());
                println!("  Help: {}", io_type.help_text());
            }
            Ok(())
        }

        Commands::Benchmark { mut input, iterations } => {
            let start = std::time::Instant::now();
            let mut total_bytes = 0u64;

            for _ in 0..iterations {
                let mut buffer = Vec::new();
                input.read_to_end(&mut buffer)?;
                total_bytes += buffer.len() as u64;
            }

            let elapsed = start.elapsed();
            let throughput = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;

            println!("Benchmark Results:");
            println!("  Total bytes: {}", total_bytes);
            println!("  Elapsed: {:?}", elapsed);
            println!("  Throughput: {:.2} MB/s", throughput);
            Ok(())
        }
    }
}

// Advanced pattern: IoPipeline for stream processing
#[allow(dead_code)]
fn example_pipeline() -> anyhow::Result<()> {
    // This demonstrates the IoPipeline API for advanced scenarios
    let mut pipe = pipeline()
        .buffer_size(65536)
        .build();

    // Example processor function
    let processor = |data: &[u8]| -> std::io::Result<Vec<u8>> {
        // Transform data here
        Ok(data.to_vec())
    };

    let _total = pipe.process(processor)?;
    Ok(())
}

// Advanced pattern: Custom type registration
#[allow(dead_code)]
fn example_type_registry() {
    let registry = IoTypeRegistry::new();

    // Register custom I/O type
    let _result = registry.register(
        "CustomInput".to_string(),
        IoType::Custom {
            name: "CustomInput".to_string(),
            properties: std::collections::HashMap::new(),
        },
    );

    // Detect type
    if let Some(io_type) = registry.detect("Input") {
        println!("Detected type: {:?}", io_type);
    }
}
