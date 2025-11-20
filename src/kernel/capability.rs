//! Capability-based security module

/// Security capability token
#[derive(Debug, Clone)]
pub struct Capability {
    pub id: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<std::time::SystemTime>,
}

impl Default for Capability {
    fn default() -> Self {
        Self { id: String::from("default-capability"), permissions: vec![], expires_at: None }
    }
}

/// Manages capabilities and permissions
#[derive(Debug, Clone, Default)]
pub struct CapabilityManager {
    capabilities: std::collections::HashMap<String, Capability>,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant_capability(&mut self, user_id: String, permissions: Vec<String>) -> Capability {
        let capability = Capability {
            id: format!("cap-{}", self.capabilities.len()),
            permissions,
            expires_at: None,
        };
        self.capabilities.insert(user_id, capability.clone());
        capability
    }

    pub fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(cap) = self.capabilities.get(user_id) {
            cap.permissions.iter().any(|p| p == permission)
        } else {
            false
        }
    }

    pub fn revoke_capability(&mut self, user_id: &str) {
        self.capabilities.remove(user_id);
    }
}
