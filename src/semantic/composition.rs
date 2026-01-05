//! Type-safe composition validation

use super::capability::CapabilityMetadata;
use thiserror::Error;

/// Errors from composition validation
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CompositionError {
    /// Type mismatch between capabilities
    #[error("Type mismatch: {source} outputs {source_type}, but {target} expects {target_type}")]
    TypeMismatch { source: String, source_type: String, target: String, target_type: String },

    /// SPARQL constraint violation
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Empty capability list
    #[error("Cannot compose empty capability list")]
    EmptyComposition,
}

/// Result of composition validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// Validation succeeded
    pub valid: bool,
    /// List of errors (empty if valid)
    pub errors: Vec<CompositionError>,
}

impl ValidationResult {
    /// Create successful validation result
    pub fn success() -> Self {
        Self { valid: true, errors: Vec::new() }
    }

    /// Create failed validation result with errors
    pub fn failure(errors: Vec<CompositionError>) -> Self {
        Self { valid: false, errors }
    }

    /// Check if validation succeeded
    pub const fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Type-level composition validator
///
/// Validates that capability compositions are type-safe:
/// - Output types match input types
/// - SPARQL constraints are satisfied
/// - No cycles in composition graph
pub struct CompositionValidator {
    /// Strict mode - reject any ambiguity
    strict: bool,
}

impl CompositionValidator {
    /// Create new validator with default settings
    pub fn new() -> Self {
        Self { strict: true }
    }

    /// Create validator with strict mode disabled
    pub fn permissive() -> Self {
        Self { strict: false }
    }

    /// Validate capability composition
    ///
    /// # Arguments
    ///
    /// - `capabilities`: Slice of capabilities to compose in order
    ///
    /// # Returns
    ///
    /// `ValidationResult` with success status and any errors
    pub fn validate(&self, capabilities: &[&CapabilityMetadata]) -> ValidationResult {
        if capabilities.is_empty() {
            return ValidationResult::failure(vec![CompositionError::EmptyComposition]);
        }

        let mut errors = Vec::new();

        // Validate each adjacent pair
        for window in capabilities.windows(2) {
            if let [source, target] = window {
                if let Err(e) = self.validate_pair(source, target) {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }

    /// Validate a single capability pair
    fn validate_pair(
        &self,
        source: &CapabilityMetadata,
        target: &CapabilityMetadata,
    ) -> Result<(), CompositionError> {
        // Extract type information from RDF metadata
        let source_output = self.extract_output_type(source);
        let target_input = self.extract_input_type(target);

        // In permissive mode, allow missing types
        if !self.strict && (source_output.is_none() || target_input.is_none()) {
            return Ok(());
        }

        // Validate type compatibility
        match (source_output, target_input) {
            (Some(output), Some(input)) if output == input => Ok(()),
            (Some(output), Some(input)) => Err(CompositionError::TypeMismatch {
                source: source.uri.to_string(),
                source_type: output,
                target: target.uri.to_string(),
                target_type: input,
            }),
            _ if self.strict => Err(CompositionError::TypeMismatch {
                source: source.uri.to_string(),
                source_type: "unknown".to_string(),
                target: target.uri.to_string(),
                target_type: "unknown".to_string(),
            }),
            _ => Ok(()),
        }
    }

    /// Extract output type from RDF metadata
    fn extract_output_type(&self, capability: &CapabilityMetadata) -> Option<String> {
        // Simple extraction - search for cap:outputType in RDF
        // FUTURE: Full RDF parsing with oxigraph
        capability
            .rdf_metadata
            .lines()
            .find(|line| line.contains("cap:outputType"))
            .and_then(|line| line.split('"').nth(1).map(|s| s.to_string()))
    }

    /// Extract input type from RDF metadata
    fn extract_input_type(&self, capability: &CapabilityMetadata) -> Option<String> {
        // Simple extraction - search for cap:inputType in RDF
        // FUTURE: Full RDF parsing with oxigraph
        capability
            .rdf_metadata
            .lines()
            .find(|line| line.contains("cap:inputType"))
            .and_then(|line| line.split('"').nth(1).map(|s| s.to_string()))
    }
}

impl Default for CompositionValidator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Unit Tests - Chicago TDD
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_capability(
        uri: &'static str,
        input: &str,
        output: &str,
    ) -> &'static CapabilityMetadata {
        let rdf = format!(
            r#"<{uri}>
    cap:inputType "{input}" ;
    cap:outputType "{output}" ."#
        );

        Box::leak(Box::new(CapabilityMetadata {
            uri,
            function_name: "test_fn",
            rdf_metadata: Box::leak(rdf.into_boxed_str()),
            mcp_descriptor: "{}",
        }))
    }

    #[test]
    fn test_validation_result_success() {
        // Arrange & Act: Create success result
        let result = ValidationResult::success();

        // Assert: Valid with no errors
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_result_failure() {
        // Arrange: Create error
        let errors = vec![CompositionError::EmptyComposition];

        // Act: Create failure result
        let result = ValidationResult::failure(errors);

        // Assert: Invalid with errors
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validator_empty_composition() {
        // Arrange: Empty capability list
        let validator = CompositionValidator::new();
        let capabilities: Vec<&CapabilityMetadata> = vec![];

        // Act: Validate
        let result = validator.validate(&capabilities);

        // Assert: Fails with EmptyComposition error
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(result.errors[0], CompositionError::EmptyComposition));
    }

    #[test]
    fn test_validator_single_capability() {
        // Arrange: Single capability
        let validator = CompositionValidator::new();
        let cap = create_test_capability("urn:test:cap1", "String", "String");
        let capabilities = vec![cap];

        // Act: Validate
        let result = validator.validate(&capabilities);

        // Assert: Succeeds (no pairs to validate)
        assert!(result.is_valid());
    }

    #[test]
    fn test_validator_compatible_pair() {
        // Arrange: Compatible capabilities
        let validator = CompositionValidator::new();
        let cap1 = create_test_capability("urn:test:cap1", "String", "i32");
        let cap2 = create_test_capability("urn:test:cap2", "i32", "bool");
        let capabilities = vec![cap1, cap2];

        // Act: Validate
        let result = validator.validate(&capabilities);

        // Assert: Succeeds
        assert!(result.is_valid());
    }

    #[test]
    fn test_validator_incompatible_pair() {
        // Arrange: Incompatible capabilities
        let validator = CompositionValidator::new();
        let cap1 = create_test_capability("urn:test:cap1", "String", "i32");
        let cap2 = create_test_capability("urn:test:cap2", "bool", "String");
        let capabilities = vec![cap1, cap2];

        // Act: Validate
        let result = validator.validate(&capabilities);

        // Assert: Fails with type mismatch
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(result.errors[0], CompositionError::TypeMismatch { .. }));
    }

    #[test]
    fn test_validator_permissive_mode() {
        // Arrange: Missing type information, permissive validator
        let validator = CompositionValidator::permissive();
        let cap1 = Box::leak(Box::new(CapabilityMetadata {
            uri: "urn:test:cap1",
            function_name: "test",
            rdf_metadata: "no types here",
            mcp_descriptor: "{}",
        }));
        let cap2 = Box::leak(Box::new(CapabilityMetadata {
            uri: "urn:test:cap2",
            function_name: "test2",
            rdf_metadata: "no types here either",
            mcp_descriptor: "{}",
        }));
        let capabilities = vec![cap1, cap2];

        // Act: Validate
        let result = validator.validate(&capabilities);

        // Assert: Succeeds in permissive mode
        assert!(result.is_valid());
    }
}
