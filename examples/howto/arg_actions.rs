//! Example: Enhanced ArgAction Support
//!
//! This example demonstrates enhanced ArgAction support in v3.2.0:
//! - Count action: Count occurrences (e.g., `-vvv` → 3)
//! - SetFalse action: Inverse flags (e.g., `--no-cache`)
//! - Auto-inference: `usize` type automatically uses Count action
//! - Auto-inference: `bool` type automatically uses SetTrue action

use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
#[derive(Serialize, Debug)]
struct BuildConfig {
    verbosity: usize,
    cache: bool,
    debug: bool,
    quiet: bool,
}

fn create_build_config(verbosity: usize, cache: bool, debug: bool, quiet: bool) -> BuildConfig {
    BuildConfig { verbosity, cache, debug, quiet }
}

// CLI Layer with Enhanced Actions

/// Build project with enhanced actions
///
/// This command demonstrates enhanced ArgAction support:
/// - `verbosity` uses Count action (auto-inferred from `usize` type)
///   - `-v` = 1, `-vv` = 2, `-vvv` = 3
/// - `cache` uses SetFalse action (inverse flag: `--no-cache`)
/// - `debug` uses SetTrue action (auto-inferred from `bool` type)
/// - `quiet` uses SetTrue action (auto-inferred from `bool` type)
///
/// # Arguments
/// * `verbosity` - Verbosity level (Count action: -v = 1, -vv = 2, -vvv = 3)
/// * `cache` - Disable cache (SetFalse action: --no-cache)
/// * `debug` - Enable debug mode (SetTrue action: --debug)
/// * `quiet` - Suppress output (SetTrue action: --quiet)
///
/// **Note**: In v3.2.0, you can use `#[arg(action = "...")]` attributes:
/// - `usize` type auto-infers Count action (e.g., `-vvv` → 3)
/// - `bool` type auto-infers SetTrue action
/// - Explicit action: `#[arg(action = "set_false")]` for inverse flags
#[noun("build", "Build commands")]
#[verb("project")]
fn build_project(
    // In v3.2.0: #[arg(short = 'v')] - Verbosity level (Count action: -v = 1, -vv = 2, -vvv = 3)
    // Auto-inferred as Count action from usize type
    verbosity: usize,
    // In v3.2.0: #[arg(action = "set_false")] - Disable cache (SetFalse action: --no-cache)
    cache: bool,
    // Enable debug mode (SetTrue action: --debug) - Auto-inferred from bool type
    debug: bool,
    // Suppress output (SetTrue action: --quiet) - Auto-inferred from bool type
    quiet: bool,
) -> Result<BuildConfig> {
    Ok(create_build_config(verbosity, cache, debug, quiet))
}

fn main() -> Result<()> {
    // Usage examples:
    //
    // 1. Count action (verbosity):
    //    $ cargo run --example arg_actions -- build project -v
    //    Output: {"verbosity":1,"cache":true,"debug":false,"quiet":false}
    //
    //    $ cargo run --example arg_actions -- build project -vvv
    //    Output: {"verbosity":3,"cache":true,"debug":false,"quiet":false}
    //
    // 2. SetFalse action (cache):
    //    $ cargo run --example arg_actions -- build project --no-cache
    //    Output: {"verbosity":0,"cache":false,"debug":false,"quiet":false}
    //
    // 3. SetTrue action (debug, quiet):
    //    $ cargo run --example arg_actions -- build project --debug
    //    Output: {"verbosity":0,"cache":true,"debug":true,"quiet":false}
    //
    //    $ cargo run --example arg_actions -- build project --quiet
    //    Output: {"verbosity":0,"cache":true,"debug":false,"quiet":true}
    //
    // 4. Combined:
    //    $ cargo run --example arg_actions -- build project -vvv --no-cache --debug
    //    Output: {"verbosity":3,"cache":false,"debug":true,"quiet":false}

    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
