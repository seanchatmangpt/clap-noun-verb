//! Web Server CLI Example - Generated from web-server.ttl
//!
//! This example demonstrates a web server lifecycle management CLI with start, stop,
//! restart, and status commands. Shows config validation patterns.
//!
//! ## Usage
//!
//! ```bash
//! # Start server
//! cargo run --example web_server_cli -- server start
//!
//! # Stop server
//! cargo run --example web_server_cli -- server stop
//!
//! # Restart server
//! cargo run --example web_server_cli -- server restart
//!
//! # Check server status
//! cargo run --example web_server_cli -- server status
//! ```

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            workers: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub running: bool,
    pub uptime_seconds: u64,
    pub active_connections: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationResult {
    pub operation: String,
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Business Logic Layer
// ============================================================================

fn start_server() -> OperationResult {
    OperationResult {
        operation: "start".to_string(),
        success: true,
        message: "Server started at 127.0.0.1:3000 with 4 workers".to_string(),
    }
}

fn stop_server() -> OperationResult {
    OperationResult {
        operation: "stop".to_string(),
        success: true,
        message: "Server stopped successfully".to_string(),
    }
}

fn restart_server() -> OperationResult {
    OperationResult {
        operation: "restart".to_string(),
        success: true,
        message: "Server restarted successfully".to_string(),
    }
}

fn get_server_status() -> ServerStatus {
    ServerStatus {
        running: true,
        uptime_seconds: 3600,
        active_connections: 42,
    }
}

// ============================================================================
// CLI Layer - Generated from Turtle specification
// ============================================================================

/// Start the web server
#[verb("start", "server")]
fn start_server_cmd() -> Result<OperationResult> {
    Ok(start_server())
}

/// Stop the web server
#[verb("stop", "server")]
fn stop_server_cmd() -> Result<OperationResult> {
    Ok(stop_server())
}

/// Restart the web server
#[verb("restart", "server")]
fn restart_server_cmd() -> Result<OperationResult> {
    Ok(restart_server())
}

/// Show server status
#[verb("status", "server")]
fn server_status_cmd() -> Result<ServerStatus> {
    Ok(get_server_status())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<()> {
    clap_noun_verb::run()
}

// ============================================================================
// Tests - Chicago TDD with AAA Pattern
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_start() {
        // Arrange & Act
        let result = start_server();

        // Assert
        assert!(result.success);
        assert_eq!(result.operation, "start");
    }

    #[test]
    fn test_server_stop() {
        // Arrange & Act
        let result = stop_server();

        // Assert
        assert!(result.success);
        assert_eq!(result.operation, "stop");
    }

    #[test]
    fn test_server_restart() {
        // Arrange & Act
        let result = restart_server();

        // Assert
        assert!(result.success);
        assert_eq!(result.operation, "restart");
    }

    #[test]
    fn test_server_status() {
        // Arrange & Act
        let status = get_server_status();

        // Assert
        assert!(status.running);
        assert_eq!(status.uptime_seconds, 3600);
        assert_eq!(status.active_connections, 42);
    }

    #[test]
    fn test_server_config_default() {
        // Arrange & Act
        let config = ServerConfig::default();

        // Assert
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.workers, 4);
    }
}
