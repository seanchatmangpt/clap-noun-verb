//! Basic example of noun-verb CLI usage with v3 attribute macro API

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
fn get_service_status() -> ServiceStatus {
    ServiceStatus {
        services: vec!["web-server".to_string(), "database".to_string(), "redis".to_string()],
        all_running: true,
    }
}

fn get_service_logs(service: String) -> Logs {
    Logs {
        service: service.clone(),
        entries: vec![
            format!("[2024-01-01 10:00:00] INFO: {} started", service),
            format!("[2024-01-01 10:00:01] INFO: {} listening on port", service),
        ],
    }
}

fn restart_service(service: String) -> RestartResult {
    RestartResult { service, success: true, message: "Service restarted successfully".to_string() }
}

#[derive(Serialize, Debug)]
struct ServiceStatus {
    services: Vec<String>,
    all_running: bool,
}

#[derive(Serialize, Debug)]
struct Logs {
    service: String,
    entries: Vec<String>,
}

#[derive(Serialize, Debug)]
struct RestartResult {
    service: String,
    success: bool,
    message: String,
}

#[derive(Serialize, Debug)]
struct CollectorStatus {
    state: String,
    http_port: u16,
    grpc_port: u16,
}

fn get_collector_status() -> CollectorStatus {
    CollectorStatus { state: "Running".to_string(), http_port: 4318, grpc_port: 4317 }
}

// CLI Layer (Input Validation + Output Shaping Only)

// Note: This file has multiple nouns ("services" and "collector"), so we need explicit nouns
// For single-noun files like services.rs, we can remove #[noun] and auto-infer from filename

/// Show status of all services
#[verb("status", "services")] // Explicit noun since filename is "basic.rs"
fn show_status() -> Result<ServiceStatus> {
    Ok(get_service_status())
}

/// Show logs for a service
#[verb("logs", "services")] // Explicit noun since filename is "basic.rs"
fn show_logs(service: String) -> Result<Logs> {
    Ok(get_service_logs(service))
}

/// Restart a service
#[verb("restart", "services")] // Explicit noun since filename is "basic.rs"
fn restart_service_cmd(service: String) -> Result<RestartResult> {
    Ok(restart_service(service))
}

/// Start the collector
#[verb("up", "collector")] // Custom verb name, explicit noun since filename is "basic.rs"
fn start_collector() -> Result<CollectorStatus> {
    Ok(get_collector_status())
}

/// Stop the collector
#[verb("down", "collector")] // Custom verb name, explicit noun since filename is "basic.rs"
fn stop_collector() -> Result<CollectorStatus> {
    Ok(CollectorStatus { state: "Stopped".to_string(), http_port: 4318, grpc_port: 4317 })
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
