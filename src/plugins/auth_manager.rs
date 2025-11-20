//! Auth Manager Plugin - User authentication and authorization
//! See PLUGIN_IMPLEMENTATION_GUIDE.md for full specification

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// User credentials
#[derive(Clone, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
    pub roles: Vec<String>,
}

#[derive(Clone)]
pub struct AuthManagerPlugin {
    users: Arc<Mutex<HashMap<String, User>>>,
    tokens: Arc<Mutex<HashMap<String, String>>>, // token -> username
    loaded: bool,
}

impl AuthManagerPlugin {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            tokens: Arc::new(Mutex::new(HashMap::new())),
            loaded: false,
        }
    }

    /// Register a new user
    pub fn register(&self, username: &str, password: &str) -> crate::Result<()> {
        let mut users = self
            .users
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Users lock failed".to_string()))?;

        if users.contains_key(username) {
            return Err(crate::NounVerbError::MiddlewareError("User already exists".to_string()));
        }

        users.insert(
            username.to_string(),
            User {
                username: username.to_string(),
                password: password.to_string(),
                roles: vec!["user".to_string()],
            },
        );

        Ok(())
    }

    /// Authenticate a user and return a token
    pub fn authenticate(&self, username: &str, password: &str) -> crate::Result<String> {
        let users = self
            .users
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Users lock failed".to_string()))?;

        let user = users
            .get(username)
            .ok_or_else(|| crate::NounVerbError::MiddlewareError("User not found".to_string()))?;

        if user.password != password {
            return Err(crate::NounVerbError::MiddlewareError("Invalid credentials".to_string()));
        }

        // Generate token using current time and username
        let timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let token = format!("token_{}__{}", username, timestamp);
        drop(users);

        // Store token
        let mut tokens = self
            .tokens
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Tokens lock failed".to_string()))?;
        tokens.insert(token.clone(), username.to_string());

        Ok(token)
    }

    /// Verify a token and return the username
    pub fn verify_token(&self, token: &str) -> crate::Result<Option<String>> {
        let tokens = self
            .tokens
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Tokens lock failed".to_string()))?;
        Ok(tokens.get(token).cloned())
    }

    /// Revoke a token (logout)
    pub fn revoke_token(&self, token: &str) -> crate::Result<()> {
        let mut tokens = self
            .tokens
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Tokens lock failed".to_string()))?;
        tokens.remove(token);
        Ok(())
    }

    /// Add role to user
    pub fn add_role(&self, username: &str, role: &str) -> crate::Result<()> {
        let mut users = self
            .users
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Users lock failed".to_string()))?;

        let user = users
            .get_mut(username)
            .ok_or_else(|| crate::NounVerbError::MiddlewareError("User not found".to_string()))?;

        if !user.roles.contains(&role.to_string()) {
            user.roles.push(role.to_string());
        }

        Ok(())
    }

    /// Check if user has role
    pub fn has_role(&self, username: &str, role: &str) -> crate::Result<bool> {
        let users = self
            .users
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Users lock failed".to_string()))?;

        let user = users
            .get(username)
            .ok_or_else(|| crate::NounVerbError::MiddlewareError("User not found".to_string()))?;

        Ok(user.roles.contains(&role.to_string()))
    }

    /// Get all users
    pub fn list_users(&self) -> crate::Result<Vec<String>> {
        let users = self
            .users
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Users lock failed".to_string()))?;
        Ok(users.keys().cloned().collect())
    }

    /// Delete a user
    pub fn delete_user(&self, username: &str) -> crate::Result<()> {
        let mut users = self
            .users
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Users lock failed".to_string()))?;
        users.remove(username);
        Ok(())
    }
}

impl Default for AuthManagerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AuthManagerPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthManagerPlugin").finish()
    }
}

impl Plugin for AuthManagerPlugin {
    fn name(&self) -> &str {
        "auth-manager"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_description("Authentication and authorization")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Validator]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chicago-TDD: Integration tests with real auth manager
    #[test]
    fn test_auth_register_and_authenticate_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("alice", "secret123").unwrap();

        let token = plugin.authenticate("alice", "secret123").unwrap();
        assert!(!token.is_empty());
        assert!(token.contains("alice"));
    }

    #[test]
    fn test_auth_verify_token_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("bob", "pass456").unwrap();
        let token = plugin.authenticate("bob", "pass456").unwrap();

        let username = plugin.verify_token(&token).unwrap();
        assert_eq!(username, Some("bob".to_string()));
    }

    #[test]
    fn test_auth_invalid_password_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("charlie", "correct").unwrap();

        let result = plugin.authenticate("charlie", "wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_nonexistent_user_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        let result = plugin.authenticate("nonexistent", "password");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_revoke_token_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("david", "pass").unwrap();
        let token = plugin.authenticate("david", "pass").unwrap();

        // Token should work
        assert!(plugin.verify_token(&token).unwrap().is_some());

        // Revoke it
        plugin.revoke_token(&token).unwrap();

        // Token should be invalid
        assert!(plugin.verify_token(&token).unwrap().is_none());
    }

    #[test]
    fn test_auth_roles_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("eve", "pass").unwrap();
        plugin.add_role("eve", "admin").unwrap();
        plugin.add_role("eve", "moderator").unwrap();

        assert!(plugin.has_role("eve", "admin").unwrap());
        assert!(plugin.has_role("eve", "moderator").unwrap());
        assert!(!plugin.has_role("eve", "superadmin").unwrap());
    }

    #[test]
    fn test_auth_list_users_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("user1", "pass").unwrap();
        plugin.register("user2", "pass").unwrap();
        plugin.register("user3", "pass").unwrap();

        let users = plugin.list_users().unwrap();
        assert_eq!(users.len(), 3);
        assert!(users.contains(&"user1".to_string()));
        assert!(users.contains(&"user2".to_string()));
        assert!(users.contains(&"user3".to_string()));
    }

    #[test]
    fn test_auth_delete_user_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("frank", "pass").unwrap();
        assert_eq!(plugin.list_users().unwrap().len(), 1);

        plugin.delete_user("frank").unwrap();
        assert_eq!(plugin.list_users().unwrap().len(), 0);
    }

    #[test]
    fn test_auth_duplicate_user_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("grace", "pass1").unwrap();
        let result = plugin.register("grace", "pass2");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_multiple_tokens_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        plugin.register("henry", "pass").unwrap();

        let token1 = plugin.authenticate("henry", "pass").unwrap();
        let token2 = plugin.authenticate("henry", "pass").unwrap();

        // Both tokens should be valid
        assert!(plugin.verify_token(&token1).unwrap().is_some());
        assert!(plugin.verify_token(&token2).unwrap().is_some());

        // Revoking one doesn't affect the other
        plugin.revoke_token(&token1).unwrap();
        assert!(plugin.verify_token(&token1).unwrap().is_none());
        assert!(plugin.verify_token(&token2).unwrap().is_some());
    }

    #[test]
    fn test_auth_concurrent_auth_workflow() {
        let mut plugin = AuthManagerPlugin::new();
        plugin.load().unwrap();

        // Setup users
        for i in 0..5 {
            plugin.register(&format!("user_{}", i), "pass").unwrap();
        }

        let plugin = Arc::new(plugin);
        let mut handles = vec![];

        for i in 0..5 {
            let p = Arc::clone(&plugin);
            let handle = std::thread::spawn(move || {
                let username = format!("user_{}", i);
                let token = p.authenticate(&username, "pass").unwrap();
                let verified = p.verify_token(&token).unwrap();
                assert_eq!(verified, Some(username));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
