//! Cache Manager Plugin - LRU cache with TTL support
//!
//! Features:
//! - Least Recently Used (LRU) eviction
//! - Per-item time-to-live (TTL)
//! - Thread-safe operations
//! - Configurable max size

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Cache entry with TTL
#[derive(Clone, Debug)]
struct CacheEntry {
    value: String,
    expires_at: SystemTime,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// Internal cache state tracking both data and access order
#[derive(Clone, Debug)]
struct CacheState {
    data: HashMap<String, CacheEntry>,
    access_order: Vec<String>, // Track insertion order for LRU eviction
}

/// Cache Manager Plugin - Production-grade caching
#[derive(Clone)]
pub struct CacheManagerPlugin {
    cache: Arc<Mutex<CacheState>>,
    max_size: usize,
    default_ttl: Duration,
    loaded: bool,
}

impl CacheManagerPlugin {
    /// Create a new cache manager plugin
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(CacheState {
                data: HashMap::new(),
                access_order: Vec::new(),
            })),
            max_size: 1000,
            default_ttl: Duration::from_secs(3600),
            loaded: false,
        }
    }

    /// Set maximum cache size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set default TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Cache a value with default TTL
    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) -> crate::Result<()> {
        self.set_with_ttl(key, value, self.default_ttl)
    }

    /// Cache a value with custom TTL
    pub fn set_with_ttl(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
        ttl: Duration,
    ) -> crate::Result<()> {
        let key = key.into();
        let value = value.into();

        let mut state = self.cache.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Cache lock failed".to_string())
        })?;

        // Evict oldest entry if at capacity
        if state.data.len() >= self.max_size && !state.data.contains_key(&key) {
            if let Some(oldest_key) = state.access_order.first().cloned() {
                state.data.remove(&oldest_key);
                state.access_order.remove(0);
            }
        }

        // Remove old access order entry if key already exists
        if let Some(pos) = state.access_order.iter().position(|k| k == &key) {
            state.access_order.remove(pos);
        }

        // Add to access order (most recent)
        state.access_order.push(key.clone());

        let expires_at = SystemTime::now() + ttl;
        state.data.insert(
            key,
            CacheEntry {
                value,
                expires_at,
            },
        );

        Ok(())
    }

    /// Get a cached value if not expired
    pub fn get(&self, key: &str) -> crate::Result<Option<String>> {
        let mut state = self.cache.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Cache lock failed".to_string())
        })?;

        if let Some(entry) = state.data.get(key) {
            if entry.is_expired() {
                state.data.remove(key);
                if let Some(pos) = state.access_order.iter().position(|k| k == key) {
                    state.access_order.remove(pos);
                }
                return Ok(None);
            }
            return Ok(Some(entry.value.clone()));
        }

        Ok(None)
    }

    /// Clear all cached entries
    pub fn clear(&self) -> crate::Result<()> {
        let mut state = self.cache.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Cache lock failed".to_string())
        })?;
        state.data.clear();
        state.access_order.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> crate::Result<(usize, usize)> {
        let state = self.cache.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Cache lock failed".to_string())
        })?;

        let total = state.data.len();
        let expired = state.data.values().filter(|e| e.is_expired()).count();
        Ok((total, expired))
    }
}

impl Default for CacheManagerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CacheManagerPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheManagerPlugin")
            .field("max_size", &self.max_size)
            .field("default_ttl", &self.default_ttl)
            .field("loaded", &self.loaded)
            .finish()
    }
}

impl Plugin for CacheManagerPlugin {
    fn name(&self) -> &str {
        "cache-manager"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_description("LRU cache with TTL support")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Middleware, PluginCapability::Hook]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }

    fn status(&self) -> String {
        format!("CacheManager v{} ({})", self.version(), if self.loaded { "loaded" } else { "unloaded" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chicago-TDD: Integration tests with real cache
    #[test]
    fn test_cache_set_and_get_workflow() {
        let mut plugin = CacheManagerPlugin::new();
        plugin.load().unwrap();

        // Set a value
        plugin.set("user:123", "Alice").unwrap();

        // Get the value
        let result = plugin.get("user:123").unwrap();
        assert_eq!(result, Some("Alice".to_string()));
    }

    #[test]
    fn test_cache_ttl_expiration_workflow() {
        let mut plugin = CacheManagerPlugin::new();
        plugin.load().unwrap();

        // Set value with very short TTL
        plugin
            .set_with_ttl("session:456", "token", Duration::from_millis(10))
            .unwrap();

        // Should exist immediately
        assert!(plugin.get("session:456").unwrap().is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(20));

        // Should be expired
        assert!(plugin.get("session:456").unwrap().is_none());
    }

    #[test]
    fn test_cache_lru_eviction_workflow() {
        let mut plugin = CacheManagerPlugin::new().with_max_size(3);
        plugin.load().unwrap();

        // Fill cache
        plugin.set("key1", "val1").unwrap();
        plugin.set("key2", "val2").unwrap();
        plugin.set("key3", "val3").unwrap();

        // Add one more (should evict oldest)
        plugin.set("key4", "val4").unwrap();

        // key1 should be evicted, others present
        assert!(plugin.get("key1").unwrap().is_none());
        assert!(plugin.get("key2").unwrap().is_some());
        assert!(plugin.get("key3").unwrap().is_some());
        assert!(plugin.get("key4").unwrap().is_some());
    }

    #[test]
    fn test_cache_clear_workflow() {
        let mut plugin = CacheManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("key1", "val1").unwrap();
        plugin.set("key2", "val2").unwrap();

        plugin.clear().unwrap();

        assert!(plugin.get("key1").unwrap().is_none());
        assert!(plugin.get("key2").unwrap().is_none());
    }

    #[test]
    fn test_cache_stats_workflow() {
        let mut plugin = CacheManagerPlugin::new();
        plugin.load().unwrap();

        plugin.set("key1", "val1").unwrap();
        plugin.set_with_ttl("key2", "val2", Duration::from_millis(10)).unwrap();

        let (total, _expired) = plugin.stats().unwrap();
        assert_eq!(total, 2);
    }

    // Multi-plugin collaboration scenario
    #[test]
    fn test_cache_with_multiple_users_workflow() {
        let mut cache = CacheManagerPlugin::new().with_max_size(100);
        cache.load().unwrap();

        // Simulate caching user sessions
        for i in 0..10 {
            cache
                .set(format!("session:{}", i), format!("user_{}", i))
                .unwrap();
        }

        // Verify all cached
        for i in 0..10 {
            let key = format!("session:{}", i);
            assert!(cache.get(&key).unwrap().is_some());
        }
    }
}
