//! Example: Deprecation & Migration System
//!
//! Demonstrates how to mark commands as deprecated and provide migration guidance.

use clap_noun_verb_macros::verb;
use clap_noun_verb::{Result, Deprecation, DeprecationType};
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct ServiceStatus {
    service: String,
    is_running: bool,
    uptime_seconds: u64,
}

/// Modern endpoint for service health check
#[verb("health")]
fn check_health() -> Result<ServiceStatus> {
    Ok(ServiceStatus {
        service: "api-server".to_string(),
        is_running: true,
        uptime_seconds: 86400,
    })
}

/// Old endpoint (deprecated in 3.5.0, will be removed in 4.0.0)
///
/// ⚠️  This command is deprecated. Use 'health' instead.
#[verb("status")]
fn check_status() -> Result<ApiResponse> {
    // Show deprecation warning
    let deprecation = Deprecation::new(DeprecationType::Verb)
        .since("3.5.0")
        .removed_in("4.0.0")
        .note("The 'status' endpoint has been renamed for clarity")
        .suggestion("Use 'health' instead - it provides more detailed information");

    let warning = deprecation.warning_message("status");
    eprintln!("{}", warning);

    Ok(ApiResponse {
        status: "ok".to_string(),
        message: "Service is running (deprecated endpoint - use 'health')".to_string(),
    })
}

/// Old restart command (deprecated)
#[verb("restart")]
fn restart_service() -> Result<ApiResponse> {
    let deprecation = Deprecation::new(DeprecationType::Verb)
        .since("3.3.0")
        .removed_in("4.0.0")
        .note("Service restart functionality has been moved to admin console")
        .suggestion("Use the admin web interface at https://admin.example.com/services");

    let warning = deprecation.warning_message("restart");
    eprintln!("{}", warning);

    Ok(ApiResponse {
        status: "restarted".to_string(),
        message: "Service restarted (deprecated - use admin console)".to_string(),
    })
}

/// View deprecation information for a command
#[verb("deprecation-info")]
fn show_deprecation_info() -> Result<String> {
    let mut info = String::new();

    // Status command deprecation
    let status_dep = Deprecation::new(DeprecationType::Verb)
        .since("3.5.0")
        .removed_in("4.0.0")
        .suggestion("Use 'health' instead");

    info.push_str(&format!(
        "Command 'status':\n{}\n\n",
        status_dep.warning_message("status")
    ));

    // Restart command deprecation
    let restart_dep = Deprecation::new(DeprecationType::Verb)
        .since("3.3.0")
        .removed_in("4.0.0")
        .suggestion("Use admin console");

    info.push_str(&format!(
        "Command 'restart':\n{}\n\n",
        restart_dep.warning_message("restart")
    ));

    info.push_str("Deprecation timeline:\n");
    info.push_str("- v3.3.0+: 'restart' deprecated\n");
    info.push_str("- v3.5.0+: 'status' deprecated\n");
    info.push_str("- v4.0.0: All deprecated commands removed\n");

    Ok(info)
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}

// Usage examples:
// $ cargo run --example deprecation_example -- server health
// $ cargo run --example deprecation_example -- server status    # Shows deprecation warning
// $ cargo run --example deprecation_example -- server restart   # Shows deprecation warning
// $ cargo run --example deprecation_example -- server deprecation-info
