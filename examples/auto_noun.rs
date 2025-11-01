//! Manage services automatically
//!
//! This module demonstrates auto-inference of noun name from filename
//! and noun about from module doc comments.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

fn get_service_status() -> Status {
    Status {
        services: vec!["api".to_string(), "worker".to_string()],
        healthy: true,
    }
}

/// Show service status
#[verb] // Noun name "auto_noun" auto-inferred from filename, about from module doc
fn show_status() -> Result<Status> {
    Ok(get_service_status())
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}

