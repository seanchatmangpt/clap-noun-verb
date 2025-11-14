//! Example: Environment Variable Support
//!
//! This example demonstrates how to use environment variables with CLI arguments.
//! Arguments can read values from environment variables using `#[arg(env = "...")]`.
//!
//! Environment variables take precedence over default values but can be overridden
//! by explicit command-line arguments.

use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
#[derive(Serialize, Debug)]
struct ServerConfig {
    port: u16,
    host: String,
    timeout: Option<u64>,
    debug: bool,
}

fn create_server_config(
    port: u16,
    host: String,
    timeout: Option<u64>,
    debug: bool,
) -> ServerConfig {
    ServerConfig { port, host, timeout, debug }
}

// CLI Layer with Environment Variables

/// Configure server settings
///
/// This command demonstrates environment variable support:
/// - `port` reads from `SERVER_PORT` environment variable (default: 8080)
/// - `host` reads from `SERVER_HOST` environment variable (default: localhost)
/// - `timeout` reads from `SERVER_TIMEOUT` environment variable (optional)
/// - `debug` is a flag (no env var support)
///
/// # Usage
///
/// Set environment variables and run:
/// ```bash
/// export SERVER_PORT=9000
/// export SERVER_HOST=example.com
/// cargo run --example env_vars -- server config
/// ```
///
/// Or override with command-line arguments:
/// ```bash
/// cargo run --example env_vars -- server config --port 9090 --host test.com
/// ```
///
/// # Arguments
/// * `port` - Server port (env: SERVER_PORT, default: 8080)
/// * `host` - Server host (env: SERVER_HOST, default: localhost)
/// * `timeout` - Request timeout in seconds (env: SERVER_TIMEOUT, optional)
/// * `debug` - Enable debug mode (no env var)
///
/// **Note**: In v3.2.0, you can use `#[arg(env = "...")]` attributes on parameters:
/// ```rust,ignore
/// #[arg(env = "SERVER_PORT", default_value = "8080")]
/// port: u16,
/// ```
/// The environment variable will be read if set, with CLI arguments taking precedence.
#[noun("server", "Server configuration")]
#[verb("config")]
fn server_config(
    // In v3.2.0: #[arg(env = "SERVER_PORT", default_value = "8080")] - Server port (env: SERVER_PORT, default: 8080)
    port: u16,
    // In v3.2.0: #[arg(env = "SERVER_HOST", default_value = "localhost")] - Server host (env: SERVER_HOST, default: localhost)
    host: String,
    // In v3.2.0: #[arg(env = "SERVER_TIMEOUT")] - Request timeout in seconds (env: SERVER_TIMEOUT)
    timeout: Option<u64>,
    // Enable debug mode
    debug: bool,
) -> Result<ServerConfig> {
    Ok(create_server_config(port, host, timeout, debug))
}

fn main() -> Result<()> {
    // Usage examples:
    //
    // 1. Using environment variables:
    //    $ export SERVER_PORT=9000
    //    $ export SERVER_HOST=example.com
    //    $ cargo run --example env_vars -- server config
    //    Output: {"port":9000,"host":"example.com","timeout":null,"debug":false}
    //
    // 2. Overriding environment variables with CLI args:
    //    $ export SERVER_PORT=9000
    //    $ cargo run --example env_vars -- server config --port 9090
    //    Output: {"port":9090,"host":"localhost","timeout":null,"debug":false}
    //
    // 3. Using defaults when env vars not set:
    //    $ cargo run --example env_vars -- server config
    //    Output: {"port":8080,"host":"localhost","timeout":null,"debug":false}
    //
    // 4. Environment variables take precedence over defaults:
    //    $ export SERVER_PORT=9000
    //    $ cargo run --example env_vars -- server config
    //    Output: {"port":9000,"host":"localhost","timeout":null,"debug":false}
    //
    // 5. CLI arguments take precedence over everything:
    //    $ export SERVER_PORT=9000
    //    $ cargo run --example env_vars -- server config --port 9090
    //    Output: {"port":9090,"host":"localhost","timeout":null,"debug":false}

    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
