//! CLI layer - thin wrapper around domain logic
//!
//! Responsibilities:
//! - Parse command-line arguments
//! - Open/close files
//! - Convert domain errors to CLI errors
//! - Format output for users
//!
//! Does NOT contain business logic!

use crate::domain::transform::{self, TransformConfig};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Process command - CLI entry point
///
/// This function is THIN - it only handles I/O and delegates to domain layer
pub fn process(
    input_path: PathBuf,
    output_path: PathBuf,
    scale: Option<f64>,
    multiplier: Option<f64>,
) -> Result<()> {
    // Open files - CLI responsibility
    let input_file = File::open(&input_path)
        .with_context(|| format!("Failed to open input file: {:?}", input_path))?;
    let output_file = File::create(&output_path)
        .with_context(|| format!("Failed to create output file: {:?}", output_path))?;

    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    // Build domain config from CLI args
    let config = TransformConfig {
        scale_factor: scale.unwrap_or(1.0),
        score_multiplier: multiplier.unwrap_or(10.0),
        normalize_names: true,
    };

    // Delegate to domain layer
    let stats = transform::process_stream(reader, &mut writer, &config)
        .context("Processing failed")?;

    // Format output for user - CLI responsibility
    println!("✓ Processing complete!");
    println!("  Processed: {}", stats.processed);
    println!("  Failed: {}", stats.failed);
    println!("  Success rate: {:.1}%", stats.success_rate());

    if !stats.errors.is_empty() {
        println!("\nErrors:");
        for (line, error) in stats.errors.iter().take(5) {
            println!("  Line {}: {}", line, error);
        }
        if stats.errors.len() > 5 {
            println!("  ... and {} more", stats.errors.len() - 5);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_command_success() {
        // Arrange - create temporary input file
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,name,value,category").unwrap();
        writeln!(input_file, "1,Test,10.0,A").unwrap();
        writeln!(input_file, "2,Example,20.0,B").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_path_buf();

        // Act
        let result = process(
            input_file.path().to_path_buf(),
            output_path.clone(),
            Some(2.0),
            Some(5.0),
        );

        // Assert
        assert!(result.is_ok());

        // Verify output file was created and contains data
        let output_content = std::fs::read_to_string(output_path).unwrap();
        assert!(output_content.contains("test"));
        assert!(output_content.contains("example"));
    }

    #[test]
    fn test_process_command_missing_input_fails() {
        // Arrange
        let input_path = PathBuf::from("/nonexistent/input.csv");
        let output_file = NamedTempFile::new().unwrap();

        // Act
        let result = process(
            input_path,
            output_file.path().to_path_buf(),
            None,
            None,
        );

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(err_msg.contains("Failed to open input file"));
    }
}
