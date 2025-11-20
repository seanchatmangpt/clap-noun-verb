//! CLI commands - thin wrappers around domain logic
//!
//! Responsibilities:
//! - Parse command-line arguments
//! - Open/close files
//! - Format output for users
//! - Convert domain errors to CLI errors

use crate::domain::{self, Input};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Process file command
///
/// This is a THIN wrapper - only handles I/O, delegates to domain
pub fn process_file(input_path: PathBuf) -> Result<()> {
    // CLI: Load file
    let data = std::fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file: {:?}", input_path))?;

    // Build domain input
    let input = Input { data };

    // Domain: Process
    let output = domain::process(input)
        .context("Processing failed")?;

    // CLI: Display result
    println!("✓ Processing complete!");
    println!("  Result: {}", output.result);
    println!("  Length: {}", output.metadata.processed_length);
    println!("  Transformations: {}", output.metadata.transformations);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_success() {
        // Arrange - create temp file
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "test data").unwrap();

        // Act
        let result = process_file(input_file.path().to_path_buf());

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_file_missing_file_fails() {
        // Arrange
        let input_path = PathBuf::from("/nonexistent/file.txt");

        // Act
        let result = process_file(input_path);

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(err_msg.contains("Failed to read input file"));
    }
}
