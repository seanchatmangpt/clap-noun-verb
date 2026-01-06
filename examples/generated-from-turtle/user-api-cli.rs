//! User API CLI Example - Generated from user-api.ttl
//!
//! This example demonstrates a CRUD API CLI for managing users.
//! Shows argument handling and JSON output formatting.
//!
//! ## Usage
//!
//! ```bash
//! # Create a user
//! cargo run --example user_api_cli -- user create john john@example.com
//!
//! # Get user by username
//! cargo run --example user_api_cli -- user get alice
//!
//! # List all users
//! cargo run --example user_api_cli -- user list
//! ```

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserList {
    pub users: Vec<User>,
    pub total: usize,
}

// ============================================================================
// Business Logic Layer
// ============================================================================

fn create_user(username: String, email: String) -> Result<User> {
    if !email.contains('@') {
        return Err(clap_noun_verb::NounVerbError::argument_error(
            "Invalid email format",
        ));
    }

    Ok(User {
        id: 1,
        username,
        email,
        created_at: "2026-01-06T00:00:00Z".to_string(),
    })
}

fn get_user(username: String) -> User {
    User {
        id: 1,
        username,
        email: "user@example.com".to_string(),
        created_at: "2026-01-06T00:00:00Z".to_string(),
    }
}

fn list_users() -> UserList {
    UserList {
        users: vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                created_at: "2026-01-02T00:00:00Z".to_string(),
            },
        ],
        total: 2,
    }
}

// ============================================================================
// CLI Layer - Generated from Turtle specification
// ============================================================================

/// Create a new user
#[verb("create", "user")]
fn create_user_cmd(username: String, email: String) -> Result<User> {
    create_user(username, email)
}

/// Get user by username
#[verb("get", "user")]
fn get_user_cmd(username: String) -> Result<User> {
    Ok(get_user(username))
}

/// List all users
#[verb("list", "user")]
fn list_users_cmd() -> Result<UserList> {
    Ok(list_users())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<()> {
    clap_noun_verb::run()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_validates_email() {
        // Arrange
        let username = "test".to_string();
        let invalid_email = "invalid-email".to_string();

        // Act
        let result = create_user(username, invalid_email);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_valid() {
        // Arrange
        let username = "test".to_string();
        let email = "test@example.com".to_string();

        // Act
        let result = create_user(username.clone(), email.clone());

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
    }

    #[test]
    fn test_list_users() {
        // Arrange & Act
        let result = list_users();

        // Assert
        assert_eq!(result.total, 2);
        assert_eq!(result.users.len(), 2);
    }

    #[test]
    fn test_get_user() {
        // Arrange
        let username = "alice".to_string();

        // Act
        let user = get_user(username.clone());

        // Assert
        assert_eq!(user.username, username);
    }
}
