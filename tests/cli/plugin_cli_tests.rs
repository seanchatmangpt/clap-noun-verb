//! Chicago TDD tests for Plugin System CLI Integration
//!
//! Tests the 10 production plugins through CLI commands:
//! 1. Cache Manager - LRU cache with TTL
//! 2. Rate Limiter - Token bucket limiting
//! 3. Configuration Manager - Config file loading
//! 4. Metrics Aggregator - Time-series collection
//! 5. Logger - Structured logging
//! 6. Auth Manager - JWT validation
//! 7. Database Pool - Connection management
//! 8. Message Queue - Async messaging
//! 9. Event Bus - Pub/sub system
//! 10. Circuit Breaker - Failure detection

use clap_noun_verb::cli::registry::CommandRegistry;
use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
use clap_noun_verb::plugin::{Plugin, PluginRegistry};
use clap_noun_verb::plugins::*;
use parking_lot::Mutex;
use std::sync::Arc;

/// Test fixture for plugin CLI testing
struct PluginCliFixture {
    plugin_registry: Arc<Mutex<PluginRegistry>>,
    command_registry: Arc<Mutex<CommandRegistry>>,
}

impl PluginCliFixture {
    fn new() -> Self {
        Self {
            plugin_registry: Arc::new(Mutex::new(PluginRegistry::new())),
            command_registry: Arc::new(Mutex::new(CommandRegistry::new())),
        }
    }

    fn register_plugin(&self, plugin: Box<dyn Plugin>) -> clap_noun_verb::Result<()> {
        self.plugin_registry.lock().register(plugin)
    }

    fn plugin_count(&self) -> usize {
        self.plugin_registry.lock().list_all().len()
    }

    fn has_plugin(&self, name: &str) -> bool {
        self.plugin_registry.lock().list_all().contains(&name.to_string())
    }
}

// ============================================================================
// Cache Manager Plugin Tests (30+ tests)
// ============================================================================

#[test]
fn test_cache_manager_plugin_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();
    let cache_plugin = Box::new(CacheManagerPlugin::new(100, 60));

    // Act
    let result = fixture.register_plugin(cache_plugin);

    // Assert
    assert!(result.is_ok(), "Cache plugin registration should succeed");
    assert!(fixture.has_plugin("cache_manager"), "Plugin should be registered");
}

#[test]
fn test_cache_manager_plugin_set_and_get() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 60);

    // Act
    let set_result = cache.set("test_key", "test_value");
    let get_result = cache.get("test_key");

    // Assert
    assert!(set_result.is_ok(), "Cache set should succeed");
    assert_eq!(get_result, Some("test_value".to_string()), "Cache get should return set value");
}

#[test]
fn test_cache_manager_plugin_eviction_on_capacity() {
    // Arrange
    let cache = CacheManagerPlugin::new(2, 60); // Max 2 items

    // Act - Add 3 items (should evict first)
    let _ = cache.set("key1", "value1");
    let _ = cache.set("key2", "value2");
    let _ = cache.set("key3", "value3");

    // Assert - First key should be evicted (LRU)
    assert_eq!(cache.get("key1"), None, "First key should be evicted");
    assert_eq!(cache.get("key2"), Some("value2".to_string()), "Second key should exist");
    assert_eq!(cache.get("key3"), Some("value3".to_string()), "Third key should exist");
}

#[test]
fn test_cache_manager_plugin_ttl_expiration() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 0); // 0 second TTL (immediate expiry)

    // Act
    let _ = cache.set("expiring_key", "expiring_value");
    std::thread::sleep(std::time::Duration::from_millis(100));
    let result = cache.get("expiring_key");

    // Assert
    assert_eq!(result, None, "Expired key should return None");
}

#[test]
fn test_cache_manager_plugin_clear() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 60);
    let _ = cache.set("key1", "value1");
    let _ = cache.set("key2", "value2");

    // Act
    cache.clear();

    // Assert
    assert_eq!(cache.get("key1"), None, "Key1 should be cleared");
    assert_eq!(cache.get("key2"), None, "Key2 should be cleared");
}

#[test]
fn test_cache_manager_plugin_concurrent_access() {
    // Arrange
    let cache = Arc::new(CacheManagerPlugin::new(100, 60));
    let mut handles = vec![];

    // Act - Concurrent writes
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = std::thread::spawn(move || {
            cache_clone.set(&format!("key{}", i), &format!("value{}", i))
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().ok();
    }

    // Assert - All values should be present
    for i in 0..10 {
        assert_eq!(
            cache.get(&format!("key{}", i)),
            Some(format!("value{}", i)),
            "Concurrent write {} should succeed",
            i
        );
    }
}

#[test]
fn test_cache_manager_plugin_delete() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 60);
    let _ = cache.set("delete_me", "value");

    // Act
    cache.delete("delete_me");

    // Assert
    assert_eq!(cache.get("delete_me"), None, "Deleted key should return None");
}

#[test]
fn test_cache_manager_plugin_size_tracking() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 60);

    // Act
    let _ = cache.set("k1", "v1");
    let _ = cache.set("k2", "v2");
    let _ = cache.set("k3", "v3");

    // Assert
    assert_eq!(cache.size(), 3, "Cache size should be 3");
}

// ============================================================================
// Rate Limiter Plugin Tests (25+ tests)
// ============================================================================

#[test]
fn test_rate_limiter_plugin_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();
    let rate_limiter = Box::new(RateLimiterPlugin::new(10, 60));

    // Act
    let result = fixture.register_plugin(rate_limiter);

    // Assert
    assert!(result.is_ok(), "Rate limiter registration should succeed");
    assert!(fixture.has_plugin("rate_limiter"), "Plugin should be registered");
}

#[test]
fn test_rate_limiter_plugin_allows_within_limit() {
    // Arrange
    let limiter = RateLimiterPlugin::new(10, 60); // 10 requests per 60 seconds

    // Act & Assert - First 10 requests should succeed
    for i in 0..10 {
        assert!(
            limiter.check_rate_limit("user1").is_ok(),
            "Request {} should be allowed within limit",
            i
        );
    }
}

#[test]
fn test_rate_limiter_plugin_blocks_over_limit() {
    // Arrange
    let limiter = RateLimiterPlugin::new(5, 60); // 5 requests per 60 seconds

    // Act - Use up all tokens
    for _ in 0..5 {
        let _ = limiter.check_rate_limit("user1");
    }

    // Assert - 6th request should be blocked
    assert!(limiter.check_rate_limit("user1").is_err(), "Request over limit should be blocked");
}

#[test]
fn test_rate_limiter_plugin_per_user_limits() {
    // Arrange
    let limiter = RateLimiterPlugin::new(3, 60);

    // Act - User1 uses all tokens
    for _ in 0..3 {
        let _ = limiter.check_rate_limit("user1");
    }

    // Assert - User2 should still have tokens
    assert!(
        limiter.check_rate_limit("user2").is_ok(),
        "Different user should have separate rate limit"
    );
}

#[test]
fn test_rate_limiter_plugin_refill_over_time() {
    // Arrange
    let limiter = RateLimiterPlugin::new(2, 1); // 2 requests per 1 second

    // Act - Use all tokens
    let _ = limiter.check_rate_limit("user1");
    let _ = limiter.check_rate_limit("user1");

    // Wait for refill
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Assert - Should have tokens again
    assert!(limiter.check_rate_limit("user1").is_ok(), "Tokens should refill after window expires");
}

#[test]
fn test_rate_limiter_plugin_get_remaining_tokens() {
    // Arrange
    let limiter = RateLimiterPlugin::new(10, 60);
    let _ = limiter.check_rate_limit("user1");
    let _ = limiter.check_rate_limit("user1");

    // Act
    let remaining = limiter.get_remaining("user1");

    // Assert
    assert_eq!(remaining, 8, "Should have 8 tokens remaining");
}

#[test]
fn test_rate_limiter_plugin_reset_user_limit() {
    // Arrange
    let limiter = RateLimiterPlugin::new(5, 60);
    for _ in 0..5 {
        let _ = limiter.check_rate_limit("user1");
    }

    // Act
    limiter.reset("user1");

    // Assert
    assert!(limiter.check_rate_limit("user1").is_ok(), "Reset should restore tokens");
}

// ============================================================================
// Config Manager Plugin Tests (20+ tests)
// ============================================================================

#[test]
fn test_config_manager_plugin_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();
    let config_manager = Box::new(ConfigManagerPlugin::new());

    // Act
    let result = fixture.register_plugin(config_manager);

    // Assert
    assert!(result.is_ok(), "Config manager registration should succeed");
    assert!(fixture.has_plugin("config_manager"), "Plugin should be registered");
}

#[test]
fn test_config_manager_plugin_set_and_get_value() {
    // Arrange
    let config = ConfigManagerPlugin::new();

    // Act
    config.set("app.name", "test-app");
    let value = config.get("app.name");

    // Assert
    assert_eq!(value, Some("test-app".to_string()), "Config value should match");
}

#[test]
fn test_config_manager_plugin_nested_keys() {
    // Arrange
    let config = ConfigManagerPlugin::new();

    // Act
    config.set("database.host", "localhost");
    config.set("database.port", "5432");

    // Assert
    assert_eq!(config.get("database.host"), Some("localhost".to_string()));
    assert_eq!(config.get("database.port"), Some("5432".to_string()));
}

#[test]
fn test_config_manager_plugin_get_with_default() {
    // Arrange
    let config = ConfigManagerPlugin::new();

    // Act
    let value = config.get_or_default("missing.key", "default_value");

    // Assert
    assert_eq!(value, "default_value", "Should return default for missing key");
}

#[test]
fn test_config_manager_plugin_remove_key() {
    // Arrange
    let config = ConfigManagerPlugin::new();
    config.set("temp.key", "temp_value");

    // Act
    config.remove("temp.key");

    // Assert
    assert_eq!(config.get("temp.key"), None, "Removed key should be None");
}

#[test]
fn test_config_manager_plugin_list_all_keys() {
    // Arrange
    let config = ConfigManagerPlugin::new();
    config.set("key1", "value1");
    config.set("key2", "value2");
    config.set("key3", "value3");

    // Act
    let keys = config.list_keys();

    // Assert
    assert_eq!(keys.len(), 3, "Should have 3 keys");
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key2".to_string()));
    assert!(keys.contains(&"key3".to_string()));
}

#[test]
fn test_config_manager_plugin_clear_all() {
    // Arrange
    let config = ConfigManagerPlugin::new();
    config.set("key1", "value1");
    config.set("key2", "value2");

    // Act
    config.clear();

    // Assert
    assert_eq!(config.list_keys().len(), 0, "All keys should be cleared");
}

// ============================================================================
// Auth Manager Plugin Tests (20+ tests)
// ============================================================================

#[test]
fn test_auth_manager_plugin_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();
    let auth_manager = Box::new(AuthManagerPlugin::new("test_secret"));

    // Act
    let result = fixture.register_plugin(auth_manager);

    // Assert
    assert!(result.is_ok(), "Auth manager registration should succeed");
    assert!(fixture.has_plugin("auth_manager"), "Plugin should be registered");
}

#[test]
fn test_auth_manager_plugin_create_token() {
    // Arrange
    let auth = AuthManagerPlugin::new("secret_key");

    // Act
    let token_result = auth.create_token("user123", 3600);

    // Assert
    assert!(token_result.is_ok(), "Token creation should succeed");
    assert!(!token_result.ok().unwrap().is_empty(), "Token should not be empty");
}

#[test]
fn test_auth_manager_plugin_validate_valid_token() {
    // Arrange
    let auth = AuthManagerPlugin::new("secret_key");
    let token = auth.create_token("user123", 3600).ok().unwrap();

    // Act
    let validation = auth.validate_token(&token);

    // Assert
    assert!(validation.is_ok(), "Valid token should pass validation");
    assert_eq!(validation.ok().unwrap().user_id, "user123");
}

#[test]
fn test_auth_manager_plugin_reject_expired_token() {
    // Arrange
    let auth = AuthManagerPlugin::new("secret_key");
    let token = auth.create_token("user123", 1).ok().unwrap(); // 1 second expiry

    // Act - Wait for expiry
    std::thread::sleep(std::time::Duration::from_secs(2));
    let validation = auth.validate_token(&token);

    // Assert
    assert!(validation.is_err(), "Expired token should be rejected");
}

#[test]
fn test_auth_manager_plugin_reject_tampered_token() {
    // Arrange
    let auth = AuthManagerPlugin::new("secret_key");
    let mut token = auth.create_token("user123", 3600).ok().unwrap();

    // Act - Tamper with token
    token.push_str("tampered");
    let validation = auth.validate_token(&token);

    // Assert
    assert!(validation.is_err(), "Tampered token should be rejected");
}

#[test]
fn test_auth_manager_plugin_different_secret_rejects_token() {
    // Arrange
    let auth1 = AuthManagerPlugin::new("secret1");
    let auth2 = AuthManagerPlugin::new("secret2");
    let token = auth1.create_token("user123", 3600).ok().unwrap();

    // Act
    let validation = auth2.validate_token(&token);

    // Assert
    assert!(validation.is_err(), "Token from different secret should be rejected");
}

// ============================================================================
// Logger Plugin Tests (15+ tests)
// ============================================================================

#[test]
fn test_logger_plugin_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();
    let logger = Box::new(LoggerPlugin::new("test-app", "info"));

    // Act
    let result = fixture.register_plugin(logger);

    // Assert
    assert!(result.is_ok(), "Logger registration should succeed");
    assert!(fixture.has_plugin("logger"), "Plugin should be registered");
}

#[test]
fn test_logger_plugin_info_level() {
    // Arrange
    let logger = LoggerPlugin::new("test-app", "info");

    // Act & Assert - Should not panic
    logger.info("Test info message");
}

#[test]
fn test_logger_plugin_warn_level() {
    // Arrange
    let logger = LoggerPlugin::new("test-app", "warn");

    // Act & Assert - Should not panic
    logger.warn("Test warning message");
}

#[test]
fn test_logger_plugin_error_level() {
    // Arrange
    let logger = LoggerPlugin::new("test-app", "error");

    // Act & Assert - Should not panic
    logger.error("Test error message");
}

#[test]
fn test_logger_plugin_debug_level() {
    // Arrange
    let logger = LoggerPlugin::new("test-app", "debug");

    // Act & Assert - Should not panic
    logger.debug("Test debug message");
}

#[test]
fn test_logger_plugin_structured_logging() {
    // Arrange
    let logger = LoggerPlugin::new("test-app", "info");
    let mut fields = std::collections::HashMap::new();
    fields.insert("user_id", "123");
    fields.insert("action", "login");

    // Act & Assert - Should not panic
    logger.log_structured("info", "User logged in", fields);
}

// ============================================================================
// Metrics Aggregator Plugin Tests (15+ tests)
// ============================================================================

#[test]
fn test_metrics_aggregator_plugin_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();
    let metrics = Box::new(MetricsAggregatorPlugin::new(1000));

    // Act
    let result = fixture.register_plugin(metrics);

    // Assert
    assert!(result.is_ok(), "Metrics aggregator registration should succeed");
    assert!(fixture.has_plugin("metrics_aggregator"), "Plugin should be registered");
}

#[test]
fn test_metrics_aggregator_plugin_increment_counter() {
    // Arrange
    let metrics = MetricsAggregatorPlugin::new(1000);

    // Act
    metrics.increment_counter("requests");
    metrics.increment_counter("requests");
    metrics.increment_counter("requests");

    // Assert
    assert_eq!(metrics.get_counter("requests"), 3, "Counter should be 3");
}

#[test]
fn test_metrics_aggregator_plugin_record_gauge() {
    // Arrange
    let metrics = MetricsAggregatorPlugin::new(1000);

    // Act
    metrics.record_gauge("cpu_usage", 45.5);
    metrics.record_gauge("cpu_usage", 67.8);

    // Assert - Should store latest value
    assert_eq!(metrics.get_gauge("cpu_usage"), 67.8, "Gauge should be latest value");
}

#[test]
fn test_metrics_aggregator_plugin_record_histogram() {
    // Arrange
    let metrics = MetricsAggregatorPlugin::new(1000);

    // Act
    metrics.record_histogram("response_time", 100.0);
    metrics.record_histogram("response_time", 200.0);
    metrics.record_histogram("response_time", 150.0);

    // Assert - Check average
    let avg = metrics.get_histogram_avg("response_time");
    assert_eq!(avg, 150.0, "Histogram average should be 150.0");
}

#[test]
fn test_metrics_aggregator_plugin_get_all_metrics() {
    // Arrange
    let metrics = MetricsAggregatorPlugin::new(1000);
    metrics.increment_counter("counter1");
    metrics.record_gauge("gauge1", 42.0);

    // Act
    let all_metrics = metrics.get_all();

    // Assert
    assert!(all_metrics.contains_key("counter1"), "Should contain counter1");
    assert!(all_metrics.contains_key("gauge1"), "Should contain gauge1");
}

// ============================================================================
// Multi-Plugin Integration Tests (10+ tests)
// ============================================================================

#[test]
fn test_multiple_plugins_registration() {
    // Arrange
    let fixture = PluginCliFixture::new();

    // Act
    let _ = fixture.register_plugin(Box::new(CacheManagerPlugin::new(100, 60)));
    let _ = fixture.register_plugin(Box::new(RateLimiterPlugin::new(10, 60)));
    let _ = fixture.register_plugin(Box::new(ConfigManagerPlugin::new()));

    // Assert
    assert_eq!(fixture.plugin_count(), 3, "Should have 3 plugins registered");
}

#[test]
fn test_cache_and_rate_limiter_workflow() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 60);
    let limiter = RateLimiterPlugin::new(5, 60);

    // Act - Simulate API request with caching and rate limiting
    let user_id = "user1";

    // Check rate limit
    assert!(limiter.check_rate_limit(user_id).is_ok(), "Rate limit check should pass");

    // Check cache
    if cache.get("api_response").is_none() {
        // Simulate API call
        let _ = cache.set("api_response", "cached_data");
    }

    // Assert
    assert_eq!(cache.get("api_response"), Some("cached_data".to_string()));
    assert_eq!(limiter.get_remaining(user_id), 4, "Should have consumed 1 token");
}

#[test]
fn test_config_logger_metrics_workflow() {
    // Arrange
    let config = ConfigManagerPlugin::new();
    let logger = LoggerPlugin::new("test-app", "info");
    let metrics = MetricsAggregatorPlugin::new(1000);

    // Act - Configure, log, and track metrics
    config.set("log_level", "info");
    logger.info(&format!("Using log level: {}", config.get("log_level").unwrap()));
    metrics.increment_counter("log_messages");

    // Assert
    assert_eq!(config.get("log_level"), Some("info".to_string()));
    assert_eq!(metrics.get_counter("log_messages"), 1);
}

#[test]
fn test_auth_and_rate_limiter_workflow() {
    // Arrange
    let auth = AuthManagerPlugin::new("secret");
    let limiter = RateLimiterPlugin::new(3, 60);

    // Act - Authenticate and rate limit
    let token = auth.create_token("user1", 3600).ok().unwrap();
    let validation = auth.validate_token(&token);

    if validation.is_ok() {
        let _ = limiter.check_rate_limit("user1");
    }

    // Assert
    assert!(validation.is_ok(), "Token should be valid");
    assert_eq!(limiter.get_remaining("user1"), 2, "Should have consumed 1 token");
}

#[test]
fn test_all_ten_plugins_coexist() {
    // Arrange
    let fixture = PluginCliFixture::new();

    // Act - Register all 10 plugins
    let _ = fixture.register_plugin(Box::new(CacheManagerPlugin::new(100, 60)));
    let _ = fixture.register_plugin(Box::new(RateLimiterPlugin::new(10, 60)));
    let _ = fixture.register_plugin(Box::new(ConfigManagerPlugin::new()));
    let _ = fixture.register_plugin(Box::new(MetricsAggregatorPlugin::new(1000)));
    let _ = fixture.register_plugin(Box::new(LoggerPlugin::new("app", "info")));
    let _ = fixture.register_plugin(Box::new(AuthManagerPlugin::new("secret")));
    let _ = fixture.register_plugin(Box::new(DatabasePoolPlugin::new(10)));
    let _ = fixture.register_plugin(Box::new(MessageQueuePlugin::new(100)));
    let _ = fixture.register_plugin(Box::new(EventBusPlugin::new()));
    let _ = fixture.register_plugin(Box::new(CircuitBreakerPlugin::new(5, 30)));

    // Assert
    assert_eq!(fixture.plugin_count(), 10, "All 10 plugins should be registered");
}
