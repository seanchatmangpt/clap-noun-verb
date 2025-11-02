//! Example: Argument Attributes
//!
//! This example demonstrates how to use argument attributes to configure
//! CLI arguments with short flags, default values, multiple values, custom
//! value names, and aliases.
//!
//! Note: `#[arg]` on parameters cannot be a real proc_macro_attribute (Rust limitation).
//! The `#[verb]` macro parses these attributes directly. Use `#[allow(unknown_attributes)]`
//! to suppress compiler warnings.

use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// Note: #[arg] on parameters cannot be used directly due to Rust limitations.
// The examples below show intended usage with comments explaining the attributes.

// Business Logic Layer (Pure Functions - Reusable)
#[derive(Serialize, Debug)]
struct ServerConfig {
    port: u16,
    host: String,
    verbose: bool,
    services: Vec<String>,
    output_file: String,
    timeout: Option<u64>,
}

fn create_server_config(
    port: u16,
    host: String,
    verbose: bool,
    services: Vec<String>,
    output_file: String,
    timeout: Option<u64>,
) -> ServerConfig {
    ServerConfig {
        port,
        host,
        verbose,
        services,
        output_file,
        timeout,
    }
}

// CLI Layer with Argument Attributes

/// Configure server settings
///
/// This command demonstrates various argument attributes:
/// - Short flags (`short = 'p'`)
/// - Default values (`default_value = "8080"`)
/// - Multiple values (`multiple` or auto-detected from `Vec<T>`)
/// - Custom value names (`value_name = "PORT"`)
/// - Aliases (`alias = "verbose"` or `aliases = ["v", "debug"]`)
///
/// # Arguments
/// * `port` - Server port (short: -p, default: 8080, value_name: PORT)
/// * `host` - Server host (short: -h, default: localhost, value_name: HOST)
/// * `verbose` - Enable verbose output (short: -v, alias: verbose)
/// * `services` - Service names (short: -s, accepts multiple values)
/// * `output_file` - Output file path (short: -o, value_name: FILE)
/// * `timeout` - Request timeout in seconds (optional, no short flag)
#[noun("server", "Server configuration")]
#[verb("config")]
fn server_config(
    // Note: In real usage with full #[arg] support, you would use:
    // #[arg(short = 'p', default_value = "8080", value_name = "PORT")]
    port: u16,
    // #[arg(short = 'h', default_value = "localhost", value_name = "HOST")]
    host: String,
    // #[arg(short = 'v', alias = "verbose")]
    verbose: bool,
    // #[arg(short = 's', multiple)]
    services: Vec<String>, // Auto-detected as multiple from Vec<String>
    // #[arg(short = 'o', value_name = "FILE")]
    output_file: String,
    // Optional argument (no short flag or default)
    timeout: Option<u64>,
) -> Result<ServerConfig> {
    Ok(create_server_config(port, host, verbose, services, output_file, timeout))
}

/// List services
///
/// This command demonstrates auto-detection of multiple values from `Vec<T>` type.
/// The `services` parameter is automatically configured to accept multiple values
/// because it's a `Vec<String>`, even without an explicit `#[arg(multiple)]` attribute.
///
/// # Arguments
/// * `services` - Service names (auto-detected as multiple from Vec<String> type)
#[verb("list")]
fn list_services(services: Vec<String>) -> Result<Vec<String>> {
    Ok(services)
}

/// Test command
#[verb("test")]
fn test_command(
    // Note: In real usage, you would use:
    // #[arg(short = 'd', aliases = ["verbose", "v"])]
    debug: bool,
) -> Result<String> {
    if debug {
        Ok("Debug mode enabled".to_string())
    } else {
        Ok("Debug mode disabled".to_string())
    }
}

fn main() -> Result<()> {
    // Usage examples:
    //
    // 1. Basic usage with defaults:
    //    $ cargo run --example arg_attributes -- server config --output-file output.txt
    //    (port defaults to 8080, host defaults to localhost)
    //
    // 2. Using short flags:
    //    $ cargo run --example arg_attributes -- server config -p 9000 -h example.com -v -o output.txt
    //
    // 3. Multiple values:
    //    $ cargo run --example arg_attributes -- server config -s api -s worker -s db -o output.txt
    //
    // 4. Using aliases:
    //    $ cargo run --example arg_attributes -- server config --verbose -o output.txt
    //    $ cargo run --example arg_attributes -- server test -d
    //    $ cargo run --example arg_attributes -- server test --verbose
    //    $ cargo run --example arg_attributes -- server test --v
    //
    // 5. Auto-detected multiple values:
    //    $ cargo run --example arg_attributes -- server list api worker db
    //
    // 6. Custom value names appear in help:
    //    $ cargo run --example arg_attributes -- server config --help
    //    Shows: --port <PORT>, --host <HOST>, --output-file <FILE>

    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}

