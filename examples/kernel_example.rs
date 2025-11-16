//! CNV Kernel Capabilities Example
//!
//! Demonstrates the new kernel features in clap-noun-verb v3.8.0:
//! - Telemetry Profile (verbosity, color, format)
//! - Output Pipeline (structured results)
//! - Grammar Introspection
//! - Manpage Generation
//! - File IO
//! - Test Harness
//!
//! Run examples:
//!   cargo run --example kernel_example data process -v --format json
//!   cargo run --example kernel_example data process --input test.txt --output out.txt
//!   cargo run --example kernel_example data process -vv --color always
//!   cargo run --example kernel_example --dump-grammar
//!   cargo run --example kernel_example --generate-manpages ./man

use clap_noun_verb::kernel::{
    FileIO, Grammar, ManpageGenerator, OutputPipeline, TelemetryProfile, VerbosityLevel,
    ColorPolicy,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Data structure for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessResult {
    lines_processed: usize,
    bytes_processed: usize,
    status: String,
}

/// Main entry point with kernel capabilities
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for special flags first
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--dump-grammar".to_string()) {
        // Dump grammar as JSON
        let grammar_json = Grammar::dump_json()?;
        println!("{}", grammar_json);
        return Ok(());
    }

    if args.contains(&"--generate-manpages".to_string()) {
        // Generate manpages
        let output_dir = args
            .get(args.iter().position(|a| a == "--generate-manpages").unwrap() + 1)
            .map(|s| s.as_str())
            .unwrap_or("./man");

        let generator = ManpageGenerator::new("kernel_example", "3.8.0")
            .with_author("CNV Team");

        let paths = generator.generate_all(Path::new(output_dir))?;
        println!("Generated {} manpages in {}", paths.len(), output_dir);
        for path in paths {
            println!("  - {}", path.display());
        }
        return Ok(());
    }

    // Regular CLI execution
    run_cli()
}

/// Run the CLI with kernel capabilities
fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    // In a real implementation, this would be integrated with the clap builder
    // For now, we'll demonstrate the kernel APIs directly

    // Parse global flags (in real implementation, these come from clap)
    let verbose_count = std::env::args()
        .filter(|a| a == "-v" || a == "--verbose")
        .count() as u8;

    let quiet = std::env::args().any(|a| a == "-q" || a == "--quiet");

    let color = std::env::args()
        .position(|a| a == "--color")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|s| s.parse::<ColorPolicy>().ok())
        .unwrap_or(ColorPolicy::Auto);

    let format = std::env::args()
        .position(|a| a == "--format")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|s| s.parse::<clap_noun_verb::OutputFormat>().ok())
        .unwrap_or(clap_noun_verb::OutputFormat::Json);

    // Create telemetry profile
    let profile = TelemetryProfile::from_args(verbose_count, quiet, color, format);

    // Log telemetry state
    if profile.is_verbose() {
        eprintln!("Verbosity: {}", profile.verbosity());
        eprintln!("Color: {}", profile.color_policy());
        eprintln!("Format: {}", profile.format());
    }

    // Simulate verb execution
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 3 && args[1] == "data" && args[2] == "process" {
        process_verb(&profile, &args[3..])?;
    } else {
        println!("Usage: kernel_example [--dump-grammar | --generate-manpages <dir>] [data process] [options]");
        println!("\nGlobal Options:");
        println!("  -v, --verbose       Increase verbosity (-v, -vv, -vvv)");
        println!("  -q, --quiet         Quiet mode");
        println!("  --color <WHEN>      Color output (auto, always, never)");
        println!("  --format <FORMAT>   Output format (json, yaml, toml, table, tsv)");
        println!("\nCommands:");
        println!("  data process        Process data with kernel features");
        println!("\nSpecial Flags:");
        println!("  --dump-grammar      Dump command grammar as JSON");
        println!("  --generate-manpages Generate manpages");
    }

    Ok(())
}

/// Process verb demonstrating all kernel features
fn process_verb(profile: &TelemetryProfile, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Parse file IO arguments
    let input = args
        .iter()
        .position(|a| a == "--input" || a == "-i")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let output = args
        .iter()
        .position(|a| a == "--output" || a == "-o")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    // Create FileIO
    let io = FileIO::from_args(input, output);

    if profile.is_verbose() {
        eprintln!(
            "Input: {}",
            io.input()
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "stdin".to_string())
        );
        eprintln!(
            "Output: {}",
            io.output()
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "stdout".to_string())
        );
    }

    // Read input
    let content = io.read_to_string()?;

    if profile.is_debug() {
        eprintln!("Read {} bytes", content.len());
    }

    // Process data
    let lines_processed = content.lines().count();
    let bytes_processed = content.len();

    // Create result
    let result = ProcessResult {
        lines_processed,
        bytes_processed,
        status: "success".to_string(),
    };

    // Use output pipeline
    OutputPipeline::render(Ok(result), profile)?;

    if profile.is_verbose() {
        eprintln!("Processing complete!");
    }

    Ok(())
}

// In a real implementation, we would use the #[noun] and #[verb] macros:
// This is commented out to keep the example self-contained

/*
use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs};

#[derive(Debug)]
struct Data;

impl NounCommand for Data {
    fn name(&self) -> &str {
        "data"
    }

    fn about(&self) -> &str {
        "Data processing commands"
    }
}

#[derive(Debug)]
struct ProcessVerb;

impl VerbCommand for ProcessVerb {
    type Noun = Data;

    fn name(&self) -> &str {
        "process"
    }

    fn about(&self) -> &str {
        "Process data with kernel features"
    }

    fn run(&self, args: &VerbArgs) -> Result<(), Box<dyn std::error::Error>> {
        // Get telemetry profile from context
        let profile = args.get_telemetry_profile();

        // Get file IO
        let input = args.get_one::<String>("input");
        let output = args.get_one::<String>("output");
        let io = FileIO::from_args(input.map(|s| s.as_str()), output.map(|s| s.as_str()));

        // ... process ...

        Ok(())
    }

    fn configure(&self, cmd: clap::Command) -> clap::Command {
        cmd
            .arg(clap::Arg::new("input")
                .short('i')
                .long("input")
                .help("Input file (or stdin if not specified)"))
            .arg(clap::Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (or stdout if not specified)"))
    }
}
*/
