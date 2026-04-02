//! Domain logic - pure business rules
//!
//! This module contains ZERO CLI dependencies:
//! - No PathBuf
//! - No File I/O
//! - No println!
//! - No user interaction

use thiserror::Error;

/// Domain error types
#[derive(Debug, Error, PartialEq)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}

/// Input domain model
#[derive(Debug, Clone)]
pub struct Input {
    pub data: String,
}

/// Output domain model
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub result: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub processed_length: usize,
    pub transformations: u32,
}

/// Core domain logic - pure function
///
/// Takes data, returns data. No I/O, no side effects.
pub fn process(input: Input) -> Result<Output, DomainError> {
    // Validate
    validate_input(&input)?;

    // Transform
    let result = transform(&input.data);

    // Compute metadata
    let metadata = Metadata {
        processed_length: result.len(),
        transformations: 1,
    };

    Ok(Output { result, metadata })
}

fn validate_input(input: &Input) -> Result<(), DomainError> {
    if input.data.is_empty() {
        return Err(DomainError::ValidationFailed(
            "Input data cannot be empty".to_string()
        ));
    }

    if input.data.len() > 1_000_000 {
        return Err(DomainError::ValidationFailed(
            "Input data too large".to_string()
        ));
    }

    Ok(())
}

fn transform(data: &str) -> String {
    // Example transformation: uppercase and add prefix
    format!("PROCESSED: {}", data.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_success() {
        // Arrange
        let input = Input {
            data: "hello world".to_string(),
        };

        // Act
        let output = process(input).unwrap();

        // Assert
        assert_eq!(output.result, "PROCESSED: HELLO WORLD");
        assert_eq!(output.metadata.processed_length, 22);
        assert_eq!(output.metadata.transformations, 1);
    }

    #[test]
    fn test_process_empty_input_fails() {
        // Arrange
        let input = Input {
            data: "".to_string(),
        };

        // Act
        let result = process(input);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            DomainError::ValidationFailed("Input data cannot be empty".to_string())
        );
    }

    #[test]
    fn test_process_large_input_fails() {
        // Arrange
        let input = Input {
            data: "x".repeat(1_000_001),
        };

        // Act
        let result = process(input);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationFailed(_)));
    }
}
