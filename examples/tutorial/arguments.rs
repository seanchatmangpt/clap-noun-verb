//! Example demonstrating argument extraction with automatic type inference

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
fn get_service_logs(service: String, lines: usize) -> Logs {
    Logs {
        service: service.clone(),
        lines,
        entries: (1..=lines.min(10)).map(|i| format!("[{}] Log entry {}", i, i)).collect(),
    }
}

fn restart_service(service: String, force: bool) -> RestartResult {
    RestartResult {
        service,
        force,
        success: true,
        message: if force {
            "Service force restarted".to_string()
        } else {
            "Service gracefully restarted".to_string()
        },
    }
}

fn deploy_service(service: String, image: Option<String>, config: Option<String>) -> DeployResult {
    DeployResult {
        service,
        image: image.unwrap_or_else(|| "latest".to_string()),
        config,
        success: true,
    }
}

#[derive(Serialize, Debug)]
struct Logs {
    service: String,
    lines: usize,
    entries: Vec<String>,
}

#[derive(Serialize, Debug)]
struct RestartResult {
    service: String,
    force: bool,
    success: bool,
    message: String,
}

#[derive(Serialize, Debug)]
struct DeployResult {
    service: String,
    image: String,
    config: Option<String>,
    success: bool,
}

// CLI Layer (Input Validation + Output Shaping Only)
// Arguments are automatically inferred from function signatures:
// - `service: String` = required argument
// - `lines: Option<usize>` = optional argument (usize auto-validates to >= 0)
// - `force: bool` = flag (auto-detected)
// - `image: Option<String>` = optional argument

/// Show logs for a service
///
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50, auto-validated: usize >= 0)
#[verb("logs", "services")] // Explicit noun since filename is "arguments.rs"
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> {
    let lines = lines.unwrap_or(50);
    Ok(get_service_logs(service, lines))
}

/// Restart a service
///
/// # Arguments
/// * `service` - Service name (required)
/// * `force` - Force restart (flag, auto-detected from bool type)
#[verb("restart", "services")] // Explicit noun since filename is "arguments.rs"
fn restart_service_cmd(service: String, force: bool) -> Result<RestartResult> {
    Ok(restart_service(service, force))
}

/// Deploy a service
///
/// # Arguments
/// * `service` - Service name (required)
/// * `image` - Container image (optional)
/// * `config` - Configuration file path (optional)
#[verb("deploy", "services")] // Explicit noun since filename is "arguments.rs"
fn deploy_service_cmd(
    service: String,
    image: Option<String>,
    config: Option<String>,
) -> Result<DeployResult> {
    Ok(deploy_service(service, image, config))
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
