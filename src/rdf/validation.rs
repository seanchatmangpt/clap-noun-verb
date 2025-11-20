//! SHACL shape validation for guarding invocations
//!
//! FUTURE v5: This module is a placeholder for later implementation

use crate::rdf::invocation::ParsedInvocation;
use thiserror::Error;

/// SHACL shape validator
pub struct ShapeValidator {
    shapes: Vec<ShaclShape>,
}

/// A SHACL shape definition
#[derive(Debug, Clone)]
pub struct ShaclShape {
    pub name: String,
    pub constraints: Vec<Constraint>,
}

/// SHACL constraint types
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Data type constraint (e.g., xsd:string, xsd:integer)
    DataType(String),
    /// Minimum value (inclusive)
    MinInclusive(i64),
    /// Maximum value (inclusive)
    MaxInclusive(i64),
    /// Regular expression pattern
    Pattern(String),
    /// Required property
    Required(bool),
    /// Minimum count
    MinCount(usize),
    /// Maximum count
    MaxCount(usize),
    /// Maximum length
    MaxLength(usize),
    /// Minimum length
    MinLength(usize),
}

/// Shape validation errors
#[derive(Debug, Error)]
pub enum ShapeError {
    #[error("Constraint violation in shape '{shape}' for property '{property}': {message}")]
    ConstraintViolation { shape: String, property: String, message: String },
    #[error("Missing required property: {property}")]
    MissingRequired { property: String },
    #[error("Invalid type for property '{property}': expected {expected}, got {got}")]
    InvalidType { property: String, expected: String, got: String },
}

impl ShapeValidator {
    /// Create a new shape validator
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    /// Add a shape to the validator
    pub fn add_shape(&mut self, shape: ShaclShape) -> std::result::Result<(), ShapeError> {
        self.shapes.push(shape);
        Ok(())
    }

    /// Add multiple shapes
    pub fn add_shapes(&mut self, shapes: Vec<ShaclShape>) -> std::result::Result<(), ShapeError> {
        for shape in shapes {
            self.add_shape(shape)?;
        }
        Ok(())
    }

    /// Validate an invocation against all shapes
    pub fn validate(&self, invocation: &ParsedInvocation) -> std::result::Result<(), ShapeError> {
        for shape in &self.shapes {
            self.validate_against_shape(invocation, shape)?;
        }
        Ok(())
    }

    /// Validate invocation against a single shape
    fn validate_against_shape(
        &self,
        invocation: &ParsedInvocation,
        shape: &ShaclShape,
    ) -> std::result::Result<(), ShapeError> {
        for constraint in &shape.constraints {
            self.validate_constraint(invocation, shape, constraint)?;
        }
        Ok(())
    }

    /// Validate a single constraint
    fn validate_constraint(
        &self,
        invocation: &ParsedInvocation,
        shape: &ShaclShape,
        constraint: &Constraint,
    ) -> std::result::Result<(), ShapeError> {
        match constraint {
            Constraint::Required(true) => {
                // Check that command is present (always true for ParsedInvocation)
                if invocation.command.is_empty() {
                    return Err(ShapeError::MissingRequired { property: "command".to_string() });
                }
            }
            Constraint::DataType(expected_type) => {
                self.validate_datatype(invocation, shape, expected_type)?;
            }
            Constraint::Pattern(pattern) => {
                self.validate_pattern(invocation, shape, pattern)?;
            }
            Constraint::MinInclusive(_min) => {
                // Placeholder for numeric validation
            }
            Constraint::MaxInclusive(_max) => {
                // Placeholder for numeric validation
            }
            Constraint::MinLength(min) => {
                self.validate_min_length(invocation, shape, *min)?;
            }
            Constraint::MaxLength(max) => {
                self.validate_max_length(invocation, shape, *max)?;
            }
            Constraint::MinCount(min) => {
                if invocation.args.len() < *min {
                    return Err(ShapeError::ConstraintViolation {
                        shape: shape.name.clone(),
                        property: "args".to_string(),
                        message: format!(
                            "Expected at least {} arguments, got {}",
                            min,
                            invocation.args.len()
                        ),
                    });
                }
            }
            Constraint::MaxCount(max) => {
                if invocation.args.len() > *max {
                    return Err(ShapeError::ConstraintViolation {
                        shape: shape.name.clone(),
                        property: "args".to_string(),
                        message: format!(
                            "Expected at most {} arguments, got {}",
                            max,
                            invocation.args.len()
                        ),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn validate_datatype(
        &self,
        invocation: &ParsedInvocation,
        _shape: &ShaclShape,
        expected_type: &str,
    ) -> std::result::Result<(), ShapeError> {
        // For simplicity, validate that string values are present
        if expected_type.contains("string") && invocation.command.is_empty() {
            return Err(ShapeError::InvalidType {
                property: "command".to_string(),
                expected: expected_type.to_string(),
                got: "empty".to_string(),
            });
        }
        Ok(())
    }

    fn validate_pattern(
        &self,
        invocation: &ParsedInvocation,
        shape: &ShaclShape,
        pattern: &str,
    ) -> std::result::Result<(), ShapeError> {
        // Simple pattern matching without regex dependency
        if pattern.contains("noun-verb") && !invocation.command.contains('-') {
            return Err(ShapeError::ConstraintViolation {
                shape: shape.name.clone(),
                property: "command".to_string(),
                message: format!("Command does not match pattern: {}", pattern),
            });
        }
        Ok(())
    }

    fn validate_min_length(
        &self,
        invocation: &ParsedInvocation,
        shape: &ShaclShape,
        min: usize,
    ) -> std::result::Result<(), ShapeError> {
        if invocation.command.len() < min {
            return Err(ShapeError::ConstraintViolation {
                shape: shape.name.clone(),
                property: "command".to_string(),
                message: format!(
                    "Command length {} is less than minimum {}",
                    invocation.command.len(),
                    min
                ),
            });
        }
        Ok(())
    }

    fn validate_max_length(
        &self,
        invocation: &ParsedInvocation,
        shape: &ShaclShape,
        max: usize,
    ) -> std::result::Result<(), ShapeError> {
        if invocation.command.len() > max {
            return Err(ShapeError::ConstraintViolation {
                shape: shape.name.clone(),
                property: "command".to_string(),
                message: format!(
                    "Command length {} exceeds maximum {}",
                    invocation.command.len(),
                    max
                ),
            });
        }
        Ok(())
    }

    /// Get number of shapes
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }
}

impl Default for ShapeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaclShape {
    /// Create a new SHACL shape
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), constraints: Vec::new() }
    }

    /// Add a constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Add multiple constraints
    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints.extend(constraints);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_invocation() -> ParsedInvocation {
        ParsedInvocation {
            command: "services-status".to_string(),
            args: BTreeMap::from([
                ("noun".to_string(), "services".to_string()),
                ("verb".to_string(), "status".to_string()),
            ]),
            output_format: Some("json".to_string()),
        }
    }

    #[test]
    fn test_shape_validator_creation() {
        let validator = ShapeValidator::new();
        assert_eq!(validator.shape_count(), 0);
    }

    #[test]
    fn test_add_shape() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::Required(true));

        validator.add_shape(shape).expect("Failed to add shape");
        assert_eq!(validator.shape_count(), 1);
    }

    #[test]
    fn test_validate_required() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::Required(true));
        validator.add_shape(shape).expect("add shape");

        let invocation = create_test_invocation();
        let result = validator.validate(&invocation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_min_length() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::MinLength(5));
        validator.add_shape(shape).expect("add shape");

        let invocation = create_test_invocation();
        let result = validator.validate(&invocation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_max_length_violation() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::MaxLength(5));
        validator.add_shape(shape).expect("add shape");

        let invocation = create_test_invocation();
        let result = validator.validate(&invocation);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_min_count() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::MinCount(2));
        validator.add_shape(shape).expect("add shape");

        let invocation = create_test_invocation();
        let result = validator.validate(&invocation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_max_count_violation() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::MaxCount(1));
        validator.add_shape(shape).expect("add shape");

        let invocation = create_test_invocation();
        let result = validator.validate(&invocation);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_constraints() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("ComplexShape").with_constraints(vec![
            Constraint::Required(true),
            Constraint::MinLength(3),
            Constraint::MaxLength(100),
        ]);
        validator.add_shape(shape).expect("add shape");

        let invocation = create_test_invocation();
        let result = validator.validate(&invocation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_command_violation() {
        let mut validator = ShapeValidator::new();
        let shape = ShaclShape::new("TestShape").with_constraint(Constraint::Required(true));
        validator.add_shape(shape).expect("add shape");

        let mut invocation = create_test_invocation();
        invocation.command = String::new();

        let result = validator.validate(&invocation);
        assert!(result.is_err());
    }
}
