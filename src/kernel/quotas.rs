//! Resource quota management module

/// Resource quota limits
#[derive(Debug, Clone)]
pub struct ResourceQuota {
    pub cpu_limit: u64,
    pub memory_limit: u64,
    pub storage_limit: u64,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            cpu_limit: 100,
            memory_limit: 1024 * 1024 * 1024, // 1GB
            storage_limit: 10 * 1024 * 1024 * 1024, // 10GB
        }
    }
}

/// Manages resource quotas and enforcement
#[derive(Debug, Clone, Default)]
pub struct QuotaManager {
    quotas: std::collections::HashMap<String, ResourceQuota>,
}

impl QuotaManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_quota(&mut self, user_id: String, quota: ResourceQuota) {
        self.quotas.insert(user_id, quota);
    }

    pub fn get_quota(&self, user_id: &str) -> Option<&ResourceQuota> {
        self.quotas.get(user_id)
    }

    pub fn check_quota(&self, user_id: &str, cpu: u64, memory: u64, storage: u64) -> bool {
        if let Some(quota) = self.get_quota(user_id) {
            cpu <= quota.cpu_limit && memory <= quota.memory_limit && storage <= quota.storage_limit
        } else {
            false
        }
    }
}
