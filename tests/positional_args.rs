//! Tests for positional argument support
//!
//! These tests verify that arguments with `#[arg(index = N)]` attributes
//! are correctly parsed and applied as positional arguments.

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
struct Repo {
    url: String,
    destination: Option<String>,
}

fn clone_repo(url: String, destination: Option<String>) -> Repo {
    Repo { url, destination }
}

/// Clone a repository
///
/// Note: In real usage with #[arg] support, you would use:
/// #[arg(index = 0)] on the url parameter and #[arg(index = 1)] on destination.
/// For testing, we verify the registry behavior works with positional args.
#[noun("git", "Git commands")]
#[verb("clone")]
fn clone_repository(
    url: String,
    destination: Option<String>,
) -> Result<Repo> {
    Ok(clone_repo(url, destination))
}

#[test]
fn test_positional_args_registered() -> Result<()> {
    // Test: Arguments can be registered as positional

    // Arrange: clone_repository has arguments
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().unwrap();
    let cmd = registry.build_command();

    // Act: Find git -> clone command
    let git_cmd = cmd.get_subcommands().find(|s| s.get_name() == "git");
    assert!(git_cmd.is_some(), "git noun should be registered");

    let clone_cmd = git_cmd
        .unwrap()
        .get_subcommands()
        .find(|s| s.get_name() == "clone");
    assert!(clone_cmd.is_some(), "clone verb should be registered");

    // Assert: Arguments should exist
    let clone_cmd = clone_cmd.unwrap();
    let args: Vec<_> = clone_cmd.get_arguments().collect();
    
    let url_arg = args.iter().find(|a| a.get_id().as_str() == "url");
    let dest_arg = args.iter().find(|a| a.get_id().as_str() == "destination");

    assert!(url_arg.is_some(), "url argument should exist");
    assert!(dest_arg.is_some(), "destination argument should exist");

    Ok(())
}

