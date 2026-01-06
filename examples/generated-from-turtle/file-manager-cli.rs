//! File Manager CLI Example - Generated from file-manager.ttl
//!
//! This example demonstrates a file management CLI with create, delete, and list operations.
//! Shows simple verb patterns without complex arguments (demonstration purposes only).
//!
//! ## Usage
//!
//! ```bash
//! # Check file operations status
//! cargo run --example file_manager_cli -- file status
//!
//! # Check directory operations status
//! cargo run --example file_manager_cli -- dir status
//! ```

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct OperationStatus {
    pub operation_type: String,
    pub status: String,
    pub message: String,
}

// ============================================================================
// Business Logic Layer
// ============================================================================

fn get_file_status() -> OperationStatus {
    OperationStatus {
        operation_type: "file".to_string(),
        status: "ready".to_string(),
        message: "File operations are available".to_string(),
    }
}

fn get_dir_status() -> OperationStatus {
    OperationStatus {
        operation_type: "directory".to_string(),
        status: "ready".to_string(),
        message: "Directory operations are available".to_string(),
    }
}

// ============================================================================
// CLI Layer - Generated from Turtle specification
// ============================================================================

/// Show file operations status
#[verb("status", "file")]
fn file_status_cmd() -> Result<OperationStatus> {
    Ok(get_file_status())
}

/// Show directory operations status
#[verb("status", "dir")]
fn dir_status_cmd() -> Result<OperationStatus> {
    Ok(get_dir_status())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<()> {
    clap_noun_verb::run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_status() {
        // Arrange & Act
        let status = get_file_status();

        // Assert
        assert_eq!(status.operation_type, "file");
        assert_eq!(status.status, "ready");
    }

    #[test]
    fn test_dir_status() {
        // Arrange & Act
        let status = get_dir_status();

        // Assert
        assert_eq!(status.operation_type, "directory");
        assert_eq!(status.status, "ready");
    }
}
