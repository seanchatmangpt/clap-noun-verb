//! Calculator CLI Example - Generated from calculator.ttl
//!
//! This example demonstrates CLI code generation from turtle specifications.
//! It shows a calculator with four operations: add, subtract, multiply, divide.
//!
//! ## Example Turtle Specification (calculator.ttl)
//!
//! ```turtle
//! @prefix cli: <http://clap-noun-verb.org/ontology#> .
//! @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
//!
//! :Calculator a cli:Noun ;
//!     cli:name "calc" ;
//!     cli:description "Calculator operations" ;
//!     cli:hasVerb :Add, :Subtract, :Multiply, :Divide .
//!
//! :Add a cli:Verb ;
//!     cli:name "add" ;
//!     cli:description "Add two numbers" ;
//!     cli:hasArgument :LeftOperand, :RightOperand .
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Add two numbers
//! cargo run --example calculator_cli -- calc add 5.5 3.2
//!
//! # Subtract numbers
//! cargo run --example calculator_cli -- calc subtract 10 3
//!
//! # Multiply numbers
//! cargo run --example calculator_cli -- calc multiply 4 2.5
//!
//! # Divide numbers
//! cargo run --example calculator_cli -- calc divide 10 2
//! ```

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CalculationResult {
    pub operation: String,
    pub left: f64,
    pub right: f64,
    pub result: f64,
}

// ============================================================================
// Business Logic Layer (Pure Functions)
// ============================================================================

fn add_numbers(left: f64, right: f64) -> CalculationResult {
    CalculationResult {
        operation: "addition".to_string(),
        left,
        right,
        result: left + right,
    }
}

fn subtract_numbers(left: f64, right: f64) -> CalculationResult {
    CalculationResult {
        operation: "subtraction".to_string(),
        left,
        right,
        result: left - right,
    }
}

fn multiply_numbers(left: f64, right: f64) -> CalculationResult {
    CalculationResult {
        operation: "multiplication".to_string(),
        left,
        right,
        result: left * right,
    }
}

fn divide_numbers(left: f64, right: f64) -> Result<CalculationResult> {
    if right == 0.0 {
        return Err(clap_noun_verb::NounVerbError::argument_error(
            "Division by zero is not allowed",
        ));
    }

    Ok(CalculationResult {
        operation: "division".to_string(),
        left,
        right,
        result: left / right,
    })
}

// ============================================================================
// CLI Layer - Generated from Turtle specification
// ============================================================================

/// Add two numbers
#[verb("add", "calc")]
fn add_cmd(left: f64, right: f64) -> Result<CalculationResult> {
    Ok(add_numbers(left, right))
}

/// Subtract two numbers
#[verb("subtract", "calc")]
fn subtract_cmd(left: f64, right: f64) -> Result<CalculationResult> {
    Ok(subtract_numbers(left, right))
}

/// Multiply two numbers
#[verb("multiply", "calc")]
fn multiply_cmd(left: f64, right: f64) -> Result<CalculationResult> {
    Ok(multiply_numbers(left, right))
}

/// Divide two numbers
#[verb("divide", "calc")]
fn divide_cmd(left: f64, right: f64) -> Result<CalculationResult> {
    divide_numbers(left, right)
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
    fn test_add_positive_numbers() {
        // Arrange
        let left = 5.5;
        let right = 3.2;

        // Act
        let result = add_numbers(left, right);

        // Assert
        assert_eq!(result.left, 5.5);
        assert_eq!(result.right, 3.2);
        assert!((result.result - 8.7).abs() < 0.001);
        assert_eq!(result.operation, "addition");
    }

    #[test]
    fn test_subtract_positive_numbers() {
        // Arrange
        let left = 10.0;
        let right = 3.0;

        // Act
        let result = subtract_numbers(left, right);

        // Assert
        assert_eq!(result.result, 7.0);
    }

    #[test]
    fn test_multiply_positive_numbers() {
        // Arrange
        let left = 4.0;
        let right = 2.5;

        // Act
        let result = multiply_numbers(left, right);

        // Assert
        assert_eq!(result.result, 10.0);
    }

    #[test]
    fn test_divide_positive_numbers() {
        // Arrange
        let left = 10.0;
        let right = 2.0;

        // Act
        let result = divide_numbers(left, right);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().result, 5.0);
    }

    #[test]
    fn test_divide_by_zero_returns_error() {
        // Arrange
        let left = 10.0;
        let right = 0.0;

        // Act
        let result = divide_numbers(left, right);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_add_negative_numbers() {
        // Arrange
        let left = -5.0;
        let right = -3.0;

        // Act
        let result = add_numbers(left, right);

        // Assert
        assert_eq!(result.result, -8.0);
    }
}
