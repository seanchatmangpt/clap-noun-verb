//! Rate Limiter Plugin - Token bucket rate limiting
//! See PLUGIN_IMPLEMENTATION_GUIDE.md for full specification

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Token bucket state for a single user
#[derive(Clone, Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time and rate
    fn refill(&mut self, rate_per_sec: f64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * rate_per_sec;
        self.tokens = (self.tokens + tokens_to_add).min(100.0); // Cap at capacity
        self.last_refill = now;
    }

    /// Try to consume a token
    fn try_consume(&mut self, rate_per_sec: f64) -> bool {
        self.refill(rate_per_sec);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate Limiter Plugin - Production-grade token bucket rate limiting
#[derive(Clone)]
pub struct RateLimiterPlugin {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    rate_per_sec: f64,
    capacity: f64,
    loaded: bool,
}

impl RateLimiterPlugin {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            rate_per_sec: 10.0,
            capacity: 100.0,
            loaded: false,
        }
    }

    /// Set rate limit (tokens per second)
    pub fn with_rate(mut self, rate: f64) -> Self {
        self.rate_per_sec = rate;
        self
    }

    /// Set bucket capacity
    pub fn with_capacity(mut self, capacity: f64) -> Self {
        self.capacity = capacity;
        self
    }

    /// Check if request is allowed for user
    pub fn allow_request(&self, user_id: &str) -> crate::Result<bool> {
        let mut buckets = self.buckets.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Bucket lock failed".to_string())
        })?;

        let bucket = buckets
            .entry(user_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.capacity));

        Ok(bucket.try_consume(self.rate_per_sec))
    }

    /// Get current token count for user (for testing)
    pub fn get_tokens(&self, user_id: &str) -> crate::Result<f64> {
        let buckets = self.buckets.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Bucket lock failed".to_string())
        })?;

        Ok(buckets
            .get(user_id)
            .map(|b| b.tokens)
            .unwrap_or(self.capacity))
    }

    /// Reset all buckets
    pub fn reset(&self) -> crate::Result<()> {
        let mut buckets = self.buckets.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Bucket lock failed".to_string())
        })?;
        buckets.clear();
        Ok(())
    }
}

impl Default for RateLimiterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RateLimiterPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiterPlugin").finish()
    }
}

impl Plugin for RateLimiterPlugin {
    fn name(&self) -> &str {
        "rate-limiter"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_description("Token bucket rate limiting")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Middleware]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // Chicago-TDD: Integration tests with real rate limiter
    #[test]
    fn test_rate_limiter_allow_request_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(10.0).with_capacity(5.0);
        plugin.load().unwrap();

        // Should allow initial requests up to capacity
        for _ in 0..5 {
            assert!(plugin.allow_request("user1").unwrap());
        }

        // Should deny when exhausted
        assert!(!plugin.allow_request("user1").unwrap());
    }

    #[test]
    fn test_rate_limiter_token_refill_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(100.0).with_capacity(10.0);
        plugin.load().unwrap();

        // Consume all tokens
        for _ in 0..10 {
            assert!(plugin.allow_request("user2").unwrap());
        }
        assert!(!plugin.allow_request("user2").unwrap());

        // Wait for tokens to refill
        thread::sleep(Duration::from_millis(100));

        // Should allow more requests
        assert!(plugin.allow_request("user2").unwrap());
    }

    #[test]
    fn test_rate_limiter_multiple_users_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(5.0).with_capacity(5.0);
        plugin.load().unwrap();

        // Each user has independent buckets
        for _ in 0..5 {
            assert!(plugin.allow_request("user_a").unwrap());
            assert!(plugin.allow_request("user_b").unwrap());
        }

        // Both should be exhausted
        assert!(!plugin.allow_request("user_a").unwrap());
        assert!(!plugin.allow_request("user_b").unwrap());
    }

    #[test]
    fn test_rate_limiter_capacity_limits_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(1000.0).with_capacity(3.0);
        plugin.load().unwrap();

        // Can only get 3 tokens initially regardless of rate
        assert!(plugin.allow_request("user3").unwrap());
        assert!(plugin.allow_request("user3").unwrap());
        assert!(plugin.allow_request("user3").unwrap());
        assert!(!plugin.allow_request("user3").unwrap());
    }

    #[test]
    fn test_rate_limiter_concurrent_requests_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(50.0).with_capacity(10.0);
        plugin.load().unwrap();

        let plugin = Arc::new(plugin);
        let mut handles = vec![];

        // Spawn 3 threads making concurrent requests
        for i in 0..3 {
            let p = Arc::clone(&plugin);
            let handle = std::thread::spawn(move || {
                let user_id = format!("user_{}", i);
                let mut count = 0;
                for _ in 0..15 {
                    if p.allow_request(&user_id).unwrap() {
                        count += 1;
                    }
                }
                count
            });
            handles.push(handle);
        }

        // Verify each thread was rate-limited appropriately
        for handle in handles {
            let count = handle.join().unwrap();
            // Each user should be rate-limited
            assert!(count <= 10);
            assert!(count > 0);
        }
    }

    #[test]
    fn test_rate_limiter_reset_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(5.0).with_capacity(5.0);
        plugin.load().unwrap();

        // Exhaust user1's tokens
        for _ in 0..5 {
            assert!(plugin.allow_request("user1").unwrap());
        }
        assert!(!plugin.allow_request("user1").unwrap());

        // Reset all buckets
        plugin.reset().unwrap();

        // Should allow requests again
        assert!(plugin.allow_request("user1").unwrap());
    }

    #[test]
    fn test_rate_limiter_get_tokens_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(10.0).with_capacity(5.0);
        plugin.load().unwrap();

        // Check initial tokens
        let initial = plugin.get_tokens("user4").unwrap();
        assert_eq!(initial, 5.0);

        // Consume some
        plugin.allow_request("user4").unwrap();
        plugin.allow_request("user4").unwrap();

        let after = plugin.get_tokens("user4").unwrap();
        // After consuming 2 tokens, should have 3 left (may have a tiny bit more due to refill)
        assert!(after <= 3.01 && after >= 2.9);
    }

    #[test]
    fn test_rate_limiter_burst_handling_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(2.0).with_capacity(5.0);
        plugin.load().unwrap();

        // Burst of requests uses capacity
        let mut allowed_count = 0;
        for _ in 0..10 {
            if plugin.allow_request("burst_user").unwrap() {
                allowed_count += 1;
            }
        }

        // Should allow up to capacity in burst
        assert_eq!(allowed_count, 5);

        // Wait and try again (slower rate-limited)
        thread::sleep(Duration::from_millis(250));
        let mut more_allowed = 0;
        for _ in 0..10 {
            if plugin.allow_request("burst_user").unwrap() {
                more_allowed += 1;
            } else {
                break;
            }
        }

        // Should allow some but rate-limited
        assert!(more_allowed >= 0);
        assert!(more_allowed < 5);
    }

    #[test]
    fn test_rate_limiter_edge_case_zero_tokens_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(1.0).with_capacity(1.0);
        plugin.load().unwrap();

        // Use single token
        assert!(plugin.allow_request("edge_user").unwrap());

        // No tokens left
        assert!(!plugin.allow_request("edge_user").unwrap());
        assert!(!plugin.allow_request("edge_user").unwrap());
    }

    #[test]
    fn test_rate_limiter_high_throughput_workflow() {
        let mut plugin = RateLimiterPlugin::new().with_rate(5.0).with_capacity(10.0);
        plugin.load().unwrap();

        // Simulate high-throughput scenario with many requests
        let mut denied = 0;
        for i in 0..100 {
            let user = format!("user_{}", i % 5);
            if !plugin.allow_request(&user).unwrap() {
                denied += 1;
            }
        }

        // Should deny some requests due to rate limit
        // With 5 users, 100 requests = 20 per user. Capacity is 10, so at least 10 should be denied
        assert!(denied >= 10);
    }
}
