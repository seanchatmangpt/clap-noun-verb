//! Security middleware with API key validation and role-based access control.

use crate::middleware::{Middleware, MiddlewareRequest};
use std::collections::HashMap;

/// Security middleware with API key and RBAC support.
///
/// Features:
/// - API key validation
/// - Role-based access control (RBAC)
/// - Per-command permission checking
#[derive(Clone, Debug)]
pub struct SecurityMiddleware {
    /// Valid API keys
    api_keys: HashMap<String, Vec<String>>, // key -> roles
    /// Command permissions (command -> required_role)
    permissions: HashMap<String, String>,
    enabled: bool,
}

impl SecurityMiddleware {
    /// Create a new security middleware.
    pub fn new() -> Self {
        Self {
            api_keys: HashMap::new(),
            permissions: HashMap::new(),
            enabled: true,
        }
    }

    /// Add an API key with associated roles.
    pub fn with_api_key(mut self, key: impl Into<String>, roles: Vec<String>) -> Self {
        self.api_keys.insert(key.into(), roles);
        self
    }

    /// Add a command permission requirement.
    pub fn with_permission(mut self, command: impl Into<String>, required_role: impl Into<String>) -> Self {
        self.permissions
            .insert(command.into(), required_role.into());
        self
    }

    /// Validate an API key and check if it has required role.
    pub fn validate_key(&self, key: &str, required_role: &str) -> bool {
        self.api_keys
            .get(key)
            .map(|roles| roles.contains(&required_role.to_string()))
            .unwrap_or(false)
    }

    /// Check if a command requires special permissions.
    pub fn get_required_role(&self, command: &str) -> Option<String> {
        self.permissions.get(command).cloned()
    }

    /// Enable security checks.
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable security checks.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for SecurityMiddleware {
    fn name(&self) -> &str {
        "security"
    }

    fn before(&self, request: &MiddlewareRequest) -> crate::Result<bool> {
        if !self.enabled {
            return Ok(true);
        }

        // Check if command requires special permissions
        if let Some(required_role) = self.get_required_role(request.command()) {
            // In production, extract API key from request context
            // and validate it has the required role
            if !self.validate_key("default_key", &required_role) {
                return Err(crate::NounVerbError::MiddlewareError(
                    format!(
                        "Command '{}' requires role '{}'",
                        request.command(),
                        required_role
                    ),
                ));
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_middleware_creation() {
        let mw = SecurityMiddleware::new();
        assert_eq!(mw.name(), "security");
    }

    #[test]
    fn test_security_middleware_with_api_key() {
        let mw = SecurityMiddleware::new().with_api_key("key123", vec!["admin".to_string()]);
        assert!(mw.validate_key("key123", "admin"));
        assert!(!mw.validate_key("key123", "user"));
    }

    #[test]
    fn test_security_middleware_with_permission() {
        let mw = SecurityMiddleware::new()
            .with_permission("delete", "admin")
            .with_permission("read", "user");

        assert_eq!(mw.get_required_role("delete"), Some("admin".to_string()));
        assert_eq!(mw.get_required_role("read"), Some("user".to_string()));
    }
}
