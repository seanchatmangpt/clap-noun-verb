//! Domain Logic: Middleware Configuration
//!
//! Pure data structures for middleware configuration.
//! Actual middleware execution is in integration layer.

use serde::{Deserialize, Serialize};

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub logging: LoggingConfig,
    pub auth: AuthConfig,
    pub rate_limiting: RateLimitConfig,
    pub caching: CacheConfig,
    pub profiling: ProfilingConfig,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            auth: AuthConfig::default(),
            rate_limiting: RateLimitConfig::default(),
            caching: CacheConfig::default(),
            profiling: ProfilingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub verbose: bool,
    pub log_args: bool,
    pub log_result: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            verbose: false,
            log_args: true,
            log_result: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub require_auth: bool,
    pub allowed_users: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_auth: false,
            allowed_users: vec!["admin".to_string(), "user".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_requests: usize,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ttl_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    pub enabled: bool,
    pub track_timing: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_timing: true,
        }
    }
}

/// Middleware statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiddlewareStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub auth_failures: u64,
    pub rate_limit_hits: u64,
    pub avg_latency_ms: f64,
}

impl MiddlewareStats {
    #[allow(dead_code)] // FUTURE: Used for middleware statistics reporting
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 { 0.0 } else { self.cache_hits as f64 / total as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_config_default() {
        let config = MiddlewareConfig::default();
        assert!(config.logging.enabled);
        assert!(!config.auth.require_auth);
        assert_eq!(config.rate_limiting.max_requests, 100);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut stats = MiddlewareStats::default();
        stats.cache_hits = 80;
        stats.cache_misses = 20;
        assert!((stats.cache_hit_rate() - 80.0).abs() < 0.01);
    }
}
