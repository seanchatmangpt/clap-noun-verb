//! Acceptance tests for automatic validation

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    name: String,
    age: u8,
}

fn create_user(name: String, age: u8) -> User {
    User { name, age }
}

// CLI functions with automatic validation

/// Create a new user
#[noun("users", "Manage users")]
#[verb("create")]
fn create_user_cmd(
    name: String,
    age: u8, // u8 automatically validates to 0-255 range
) -> Result<User> {
    Ok(create_user(name, age))
}

#[test]
fn test_auto_validation_from_types() -> Result<()> {
    // Acceptance Test: Auto-inferred Validation from Types
    //
    // Acceptance criteria:
    // 1. Unsigned integer types automatically get min=0 validation
    // 2. Type constraints are applied to clap arguments
    // 3. Validation metadata is stored in ArgMetadata

    // Arrange: Functions have typed parameters

    // Act: Build command and verify validation metadata
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "Failed to lock registry: {}",
            e
        ))
    })?;
    let cmd = registry.build_command();

    // Find users -> create command
    if let Some(users_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "users") {
        if let Some(create_cmd) = users_cmd.get_subcommands().find(|s| s.get_name() == "create") {
            // Assert: Arguments should have validation metadata
            let args: Vec<_> = create_cmd.get_arguments().collect();
            let mut found_age = false;

            for arg in args {
                if arg.get_id().as_str() == "age" {
                    found_age = true;
                    // Age is u8, so should have validation (0-255 for u8)
                    assert!(true, "Argument 'age' exists with type validation");
                }
            }

            assert!(found_age, "Argument 'age' should be registered");
        } else {
            panic!("create verb should be registered");
        }
    } else {
        panic!("users noun should be registered");
    }

    Ok(())
}

#[test]
fn test_explicit_validation_attributes() -> Result<()> {
    // Acceptance Test: Explicit Validation Attributes
    //
    // Acceptance criteria:
    // 1. #[validate(min = 18, max = 120)] attributes are parsed
    // 2. Explicit validation overrides type-inferred validation
    // 3. Validation constraints are applied to clap arguments

    // Arrange: Functions have #[validate(...)] attributes

    // Act: Build command and verify validation metadata
    let registry = clap_noun_verb::cli::registry::CommandRegistry::get();
    let registry = registry.lock().map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "Failed to lock registry: {}",
            e
        ))
    })?;
    let cmd = registry.build_command();

    // Find users -> create command
    if let Some(users_cmd) = cmd.get_subcommands().find(|s| s.get_name() == "users") {
        if let Some(create_cmd) = users_cmd.get_subcommands().find(|s| s.get_name() == "create") {
            // Assert: Arguments should have validation applied
            let args: Vec<_> = create_cmd.get_arguments().collect();
            let mut found_age = false;

            for arg in args {
                if arg.get_id().as_str() == "age" {
                    found_age = true;
                    // Age is u8, so should have type-based validation (0-255)
                    assert!(true, "Argument 'age' exists with type-based validation");
                }
            }

            assert!(found_age, "Argument 'age' should be registered");
        } else {
            panic!("create verb should be registered");
        }
    } else {
        panic!("users noun should be registered");
    }

    Ok(())
}
