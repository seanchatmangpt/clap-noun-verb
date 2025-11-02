//! Example: Positional Arguments
//!
//! This example demonstrates how to use positional arguments in CLI commands.
//! Positional arguments can be specified using `#[arg(index = N)]` attributes.
//!
//! Positional arguments are accessed by position (e.g., first argument, second argument)
//! rather than by name (e.g., --flag value).

use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
#[derive(Serialize, Debug)]
struct Repo {
    url: String,
    destination: Option<String>,
}

fn clone_repository(url: String, destination: Option<String>) -> Repo {
    Repo { url, destination }
}

// CLI Layer with Positional Arguments

/// Clone a repository
///
/// This command demonstrates positional arguments:
/// - `url` is the first positional argument (index = 0)
/// - `destination` is the second positional argument (index = 1, optional)
///
/// # Usage
///
/// Basic usage with required positional argument:
/// ```bash
/// cargo run --example positional -- git clone https://example.com/repo.git
/// ```
///
/// With optional destination:
/// ```bash
/// cargo run --example positional -- git clone https://example.com/repo.git /path/to/dest
/// ```
///
/// **Note**: In v3.2.0, you can use `#[arg(index = N)]` attributes on parameters:
/// ```rust,ignore
/// #[arg(index = 0)]
/// url: String,
/// #[arg(index = 1)]
/// destination: Option<String>,
/// ```
/// This makes arguments positional (accessed by position rather than name).
#[noun("git", "Git commands")]
#[verb("clone")]
fn clone_repo(
    // In v3.2.0: #[arg(index = 0)] - Repository URL (first positional argument)
    url: String,
    // In v3.2.0: #[arg(index = 1)] - Destination directory (second positional argument, optional)
    destination: Option<String>,
) -> Result<Repo> {
    Ok(clone_repository(url, destination))
}

fn main() -> Result<()> {
    // Usage examples:
    //
    // 1. Basic usage with required positional argument:
    //    $ cargo run --example positional -- git clone https://example.com/repo.git
    //    Output: {"url":"https://example.com/repo.git","destination":null}
    //
    // 2. With optional destination:
    //    $ cargo run --example positional -- git clone https://example.com/repo.git /path/to/dest
    //    Output: {"url":"https://example.com/repo.git","destination":"/path/to/dest"}
    //
    // 3. Positional arguments must come after the verb:
    //    $ cargo run --example positional -- git clone https://example.com/repo.git
    //
    // Note: Positional arguments cannot have short flags or aliases.
    // They are accessed by position, not by name.

    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}

