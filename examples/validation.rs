//! Example: Automatic Validation
//!
//! This example demonstrates automatic validation from types and explicit
//! validation attributes, similar to Typer's validation system.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct User {
    name: String,
    age: u8,
    email: String,
    port: Option<u16>,
}

// Business Logic Layer (Pure Functions - Reusable)
fn create_user(name: String, age: u8, email: String, port: Option<u16>) -> User {
    User { name, age, email, port }
}

// CLI Layer with Automatic Validation
// Type-based validation:
// - `age: u8` automatically validates to 0-255
// - `port: Option<u16>` automatically validates to 0-65535 when provided
//
// Explicit validation attributes:
// - `#[validate(min_length = 1, max_length = 50)]` on name
// - `#[validate(min = 18, max = 120)]` on age (overrides type constraints)

/// Create a new user
///
/// # Arguments
/// * `name` - User name
/// * `age` - User age (u8 gives 0-255 range automatically)
/// * `email` - User email address
/// * `port` - Optional port number (u16 gives 0-65535 range automatically)
#[verb("create", "users")] // Explicit noun since filename is "validation.rs"
fn create_user_command(
    name: String,
    age: u8, // Automatically validates to 0-255 range
    email: String,
    port: Option<u16>, // Automatically validates to 0-65535 when provided
) -> Result<User> {
    // All validation happens automatically before this function is called
    // - age validated (0-255 range from u8 type)
    // - port validated (0-65535 range from u16 type) if provided
    Ok(create_user(name, age, email, port))
}

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
