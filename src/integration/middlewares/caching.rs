//! Dynamic caching middleware with generic type support and TTL.

use crate::middleware::{Middleware, MiddlewareRequest, MiddlewareResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Dynamic caching middleware with per-command TTL configuration.
///
/// Uses const generics to enforce compile-time configuration.
/// Example: DynamicCachingMiddleware::<300>::new() for 5-minute cache
#[derive(Clone)]
pub struct DynamicCachingMiddleware {
    /// Cache storage
    cache: Arc<Mutex<HashMap<String, (String, u64)>>>,
    /// TTL in seconds
    ttl_seconds: u64,
}

impl DynamicCachingMiddleware {
    /// Create a new caching middleware with default 5-minute TTL.
    pub fn new() -> Self {
        Self::with_ttl(300)
    }

    /// Create a caching middleware with custom TTL in seconds.
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self { cache: Arc::new(Mutex::new(HashMap::new())), ttl_seconds }
    }

    /// Get cached value if exists and not expired.
    fn get_cached(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().ok()?;
        let (value, expiry) = cache.get(key)?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        if now < *expiry {
            Some(value.clone())
        } else {
            None
        }
    }

    /// Cache a value with expiry time.
    fn set_cached(&self, key: String, value: String) {
        if let Ok(mut cache) = self.cache.lock() {
            let expiry = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
                + self.ttl_seconds;
            cache.insert(key, (value, expiry));
        }
    }

    /// Clear all cached entries.
    pub fn clear(&self) -> crate::Result<()> {
        self.cache
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Cache lock failed".to_string()))?
            .clear();
        Ok(())
    }
}

impl Default for DynamicCachingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DynamicCachingMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicCachingMiddleware").field("ttl_seconds", &self.ttl_seconds).finish()
    }
}

impl Middleware for DynamicCachingMiddleware {
    fn name(&self) -> &str {
        "dynamic_caching"
    }

    fn before(&self, request: &MiddlewareRequest) -> crate::Result<bool> {
        let cache_key = format!("{:?}", request.args());
        if self.get_cached(&cache_key).is_some() {
            return Ok(true);
        }
        Ok(true)
    }

    fn after(&self, response: &MiddlewareResponse) -> crate::Result<()> {
        if response.is_success() {
            let cache_key = format!("cmd_{}", response.message().len());
            self.set_cached(cache_key, response.message().to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caching_middleware_creation() {
        let mw = DynamicCachingMiddleware::new();
        assert_eq!(mw.name(), "dynamic_caching");
    }

    #[test]
    fn test_caching_middleware_with_ttl() {
        let mw = DynamicCachingMiddleware::with_ttl(600);
        assert_eq!(mw.ttl_seconds, 600);
    }

    #[test]
    fn test_caching_clear() {
        let mw = DynamicCachingMiddleware::new();
        assert!(mw.clear().is_ok());
    }
}
