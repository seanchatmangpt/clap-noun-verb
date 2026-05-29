// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! How-to: Use clap-nv.toml for default configuration
//!
//! This example demonstrates how the framework automatically discovers
//! and applies configuration from a `clap-nv.toml` file.

use clap_noun_verb::cli::CliBuilder;
use clap_noun_verb::config::ConfigLoader;
use clap_noun_verb_macros::verb;

/// A test command to verify configuration
#[verb("test", "")]
fn test_cmd(host: String, port: u16, verbose: bool) -> clap_noun_verb::Result<()> {
    println!("Host: {}", host);
    println!("Port: {}", port);
    println!("Verbose: {}", verbose);
    Ok(())
}

fn main() -> clap_noun_verb::Result<()> {
    // 1. Create a dummy clap-nv.toml file for this example
    let config_content = r#"
host = "localhost"
port = 9000
verbose = true
"#;
    std::fs::write("clap-nv.toml", config_content).expect("Failed to write config file");

    println!("--- Using clap-nv.toml configuration ---");

    // 2. Load the configuration (it will find clap-nv.toml automatically)
    let config = ConfigLoader::new().load()?;
    let config_args = config.to_cli_args();

    let mut args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        // Just the binary name, use default test args
        args.extend(vec!["test".to_string()]);
    }

    // Only inject config args if we are running the 'test' command
    // to avoid 'unexpected argument' errors on commands like 'doctor'
    if args.get(1).map(|s| s.as_str()) == Some("test") {
        args.extend(config_args);
    }

    CliBuilder::new("config-demo").version("1.0.0").run_with_args(args)?;

    // Clean up
    let _ = std::fs::remove_file("clap-nv.toml");

    Ok(())
}
