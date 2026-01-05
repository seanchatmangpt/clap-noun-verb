//! Example demonstrating root-level verbs (verbs without a noun)
//!
//! Root verbs appear directly under the CLI binary, e.g.:
//! - `mycli sync` instead of `mycli noun sync`
//!
//! Declared with: `#[verb("sync", "root")]`

use clap_noun_verb::cli::registry::CommandRegistry;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Output for the sync command
#[derive(Debug, Clone, Serialize)]
pub struct SyncOutput {
    pub status: String,
    pub files_synced: usize,
}

/// Synchronize files (root-level command)
///
/// This command appears directly under the CLI binary without a noun prefix.
#[verb("sync", "root")]
fn sync() -> clap_noun_verb::Result<SyncOutput> {
    Ok(SyncOutput { status: "success".to_string(), files_synced: 42 })
}

fn main() {
    // Initialize registry and run
    let registry = CommandRegistry::get();
    let reg = registry.lock().unwrap();

    // Get args - use --help to show structure
    let args: Vec<String> = std::env::args().collect();

    // Release lock before run
    drop(reg);

    // Run with the collected args
    let registry = CommandRegistry::get();
    let reg = registry.lock().unwrap();
    if let Err(e) = reg.run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
