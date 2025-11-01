//! Manage OpenTelemetry collector

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
fn start_collector() -> CollectorStatus {
    CollectorStatus {
        state: "Running".to_string(),
        http_port: 4318,
        grpc_port: 4317,
        message: "Collector started successfully".to_string(),
    }
}

fn stop_collector() -> CollectorStatus {
    CollectorStatus {
        state: "Stopped".to_string(),
        http_port: 4318,
        grpc_port: 4317,
        message: "Collector stopped".to_string(),
    }
}

fn get_collector_status() -> CollectorStatus {
    CollectorStatus {
        state: "Running".to_string(),
        http_port: 4318,
        grpc_port: 4317,
        message: "Collector is running".to_string(),
    }
}

fn get_collector_logs() -> Logs {
    Logs {
        entries: vec![
            "[2024-01-01 10:00:00] INFO: Collector started".to_string(),
            "[2024-01-01 10:00:01] INFO: HTTP server listening on :4318".to_string(),
            "[2024-01-01 10:00:01] INFO: gRPC server listening on :4317".to_string(),
            "[2024-01-01 10:05:23] INFO: Received 150 spans".to_string(),
        ],
    }
}

#[derive(Serialize, Debug)]
struct CollectorStatus {
    state: String,
    http_port: u16,
    grpc_port: u16,
    message: String,
}

#[derive(Serialize, Debug)]
struct Logs {
    entries: Vec<String>,
}

// CLI Layer (Input Validation + Output Shaping Only)

/// Start the collector
#[verb("up")] // Custom verb name, noun "collector" auto-inferred from filename
fn start_collector_cmd() -> Result<CollectorStatus> {
    Ok(start_collector())
}

/// Stop the collector
#[verb("down")] // Custom verb name, noun "collector" auto-inferred from filename
fn stop_collector_cmd() -> Result<CollectorStatus> {
    Ok(stop_collector())
}

/// Show collector status
#[verb] // Verb name "status" auto-inferred, noun "collector" auto-inferred from filename
fn show_collector_status() -> Result<CollectorStatus> {
    Ok(get_collector_status())
}

/// Show collector logs
#[verb] // Verb name "logs" auto-inferred, noun "collector" auto-inferred from filename
fn show_collector_logs() -> Result<Logs> {
    Ok(get_collector_logs())
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
