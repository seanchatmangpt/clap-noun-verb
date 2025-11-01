//! Framework-level composition example using v3 attribute macro API
//!
//! This example demonstrates how to use clap-noun-verb v3's attribute macro API
//! for building composable CLI applications with automatic type inference,
//! JSON output, and separation of concerns.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)

fn get_service_status() -> ServiceStatus {
    ServiceStatus {
        services: vec![
            ServiceInfo {
                name: "web-server".to_string(),
                state: "Running".to_string(),
                port: 8080,
            },
            ServiceInfo { name: "database".to_string(), state: "Running".to_string(), port: 5432 },
            ServiceInfo { name: "redis".to_string(), state: "Running".to_string(), port: 6379 },
        ],
    }
}

fn get_service_logs(service: String) -> Logs {
    Logs {
        service: service.clone(),
        entries: vec![
            format!("[2024-01-01 10:00:00] INFO: {} started", service),
            format!("[2024-01-01 10:00:01] INFO: {} listening", service),
        ],
    }
}

fn restart_service(service: String) -> RestartResult {
    RestartResult { service, success: true }
}

fn get_collector_status() -> CollectorStatus {
    CollectorStatus { state: "Running".to_string(), http_port: 4318, grpc_port: 4317 }
}

fn start_collector() -> CollectorStatus {
    CollectorStatus { state: "Running".to_string(), http_port: 4318, grpc_port: 4317 }
}

fn stop_collector() -> CollectorStatus {
    CollectorStatus { state: "Stopped".to_string(), http_port: 4318, grpc_port: 4317 }
}

#[derive(Serialize, Debug)]
struct ServiceInfo {
    name: String,
    state: String,
    port: u16,
}

#[derive(Serialize, Debug)]
struct ServiceStatus {
    services: Vec<ServiceInfo>,
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
}

#[derive(Serialize, Debug)]
struct CollectorStatus {
    state: String,
    http_port: u16,
    grpc_port: u16,
}

// CLI Layer (Input Validation + Output Shaping Only)
// All business logic is separated into pure functions above

// Note: This file has multiple nouns ("services" and "collector"), so we need to keep #[noun]
// For single-noun files, we can remove #[noun] and auto-infer from filename

/// Show status of all services
#[verb("status", "services")] // Explicit noun since filename is "framework.rs"
fn show_status() -> Result<ServiceStatus> {
    Ok(get_service_status())
}

/// Show logs for a service
///
/// # Arguments
/// * `service` - Service name
#[verb("logs", "services")] // Explicit noun since filename is "framework.rs"
fn show_logs(service: String) -> Result<Logs> {
    Ok(get_service_logs(service))
}

/// Restart a service
///
/// # Arguments
/// * `service` - Service name to restart
#[verb("restart", "services")] // Explicit noun since filename is "framework.rs"
fn restart_service_cmd(service: String) -> Result<RestartResult> {
    Ok(restart_service(service))
}

/// Start the collector
#[verb("up", "collector")] // Custom verb name, explicit noun since filename is "framework.rs"
fn start_collector_cmd() -> Result<CollectorStatus> {
    Ok(start_collector())
}

/// Stop the collector
#[verb("down", "collector")] // Custom verb name, explicit noun since filename is "framework.rs"
fn stop_collector_cmd() -> Result<CollectorStatus> {
    Ok(stop_collector())
}

/// Show collector status
#[verb("status", "collector")] // Explicit noun since filename is "framework.rs"
fn show_collector_status() -> Result<CollectorStatus> {
    Ok(get_collector_status())
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    // The framework automatically:
    // - Discovers all #[noun] and #[verb] functions
    // - Builds the CLI structure
    // - Handles argument parsing and validation
    // - Executes the appropriate handler
    // - Serializes output to JSON
    clap_noun_verb::run()
}
