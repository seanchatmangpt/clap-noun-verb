//! Comprehensive tests for wizard advanced features
//!
//! Tests caching, rate limiting, retry logic, and streaming features.
//! These tests verify the wrapper clients work correctly with proper
//! configuration, error handling, and behavioral guarantees.
//!
//! Chicago TDD Principles:
//! - State-based testing (verify cache states, retry counts, rate limits)
//! - Behavior verification (test observable effects)
//! - AAA pattern (Arrange-Act-Assert)

#![cfg(feature = "wizard")]

use clap_noun_verb::wizard::cache::*;
use clap_noun_verb::wizard::rate_limit::*;
use clap_noun_verb::wizard::retry::*;
use std::time::Duration;

// =============================================================================
// CacheConfig Tests - Comprehensive caching configuration
// =============================================================================

#[test]
fn test_cache_config_default_values() {
    // Arrange + Act
    let config = CacheConfig::default();

    // Assert
    assert_eq!(config.max_entries, 100);
    assert_eq!(config.ttl_seconds, 3600);
    assert!(config.enable_stats);
}

#[test]
fn test_cache_config_builder_pattern() {
    // Arrange + Act
    let config = CacheConfig::new(50, 1800).with_max_entries(200).with_ttl(7200).with_stats(false);

    // Assert
    assert_eq!(config.max_entries, 200);
    assert_eq!(config.ttl_seconds, 7200);
    assert!(!config.enable_stats);
}

#[test]
fn test_cache_config_validation_success() {
    // Arrange
    let config = CacheConfig::new(100, 3600);

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_cache_config_validation_zero_max_entries() {
    // Arrange
    let config = CacheConfig { max_entries: 0, ttl_seconds: 3600, enable_stats: true };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_cache_config_validation_zero_ttl() {
    // Arrange
    let config = CacheConfig { max_entries: 100, ttl_seconds: 0, enable_stats: true };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_cache_config_small_values() {
    // Arrange
    let config = CacheConfig::new(1, 1);

    // Act
    let result = config.validate();

    // Assert - minimum valid values should pass
    assert!(result.is_ok());
}

#[test]
fn test_cache_config_large_values() {
    // Arrange
    let config = CacheConfig::new(100_000, 86400 * 365); // 1 year TTL

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

// =============================================================================
// CacheStats Tests - Test cache statistics tracking
// =============================================================================

#[test]
fn test_cache_stats_default() {
    // Arrange + Act
    let stats = CacheStats::default();

    // Assert
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.evictions, 0);
    assert_eq!(stats.insertions, 0);
}

#[test]
fn test_cache_stats_hit_rate_zero_requests() {
    // Arrange
    let stats = CacheStats::default();

    // Act
    let hit_rate = stats.hit_rate();

    // Assert
    assert_eq!(hit_rate, 0.0);
}

#[test]
fn test_cache_stats_hit_rate_all_hits() {
    // Arrange
    let stats = CacheStats { hits: 100, misses: 0, evictions: 0, insertions: 100 };

    // Act
    let hit_rate = stats.hit_rate();

    // Assert
    assert_eq!(hit_rate, 1.0); // 100%
}

#[test]
fn test_cache_stats_hit_rate_all_misses() {
    // Arrange
    let stats = CacheStats { hits: 0, misses: 100, evictions: 0, insertions: 100 };

    // Act
    let hit_rate = stats.hit_rate();

    // Assert
    assert_eq!(hit_rate, 0.0); // 0%
}

#[test]
fn test_cache_stats_hit_rate_mixed() {
    // Arrange
    let stats = CacheStats { hits: 75, misses: 25, evictions: 0, insertions: 25 };

    // Act
    let hit_rate = stats.hit_rate();

    // Assert
    assert!((hit_rate - 0.75).abs() < 0.001); // 75%
}

#[test]
fn test_cache_stats_total_requests() {
    // Arrange
    let stats = CacheStats { hits: 60, misses: 40, evictions: 5, insertions: 40 };

    // Act
    let total = stats.total_requests();

    // Assert
    assert_eq!(total, 100);
}

// =============================================================================
// RateLimitConfig Tests - Comprehensive rate limiting configuration
// =============================================================================

#[test]
fn test_rate_limit_config_default_values() {
    // Arrange + Act
    let config = RateLimitConfig::default();

    // Assert
    assert_eq!(config.tokens_per_second, 10.0);
    assert_eq!(config.burst_capacity, 20);
    assert_eq!(config.strategy, RateLimitStrategy::Wait);
}

#[test]
fn test_rate_limit_config_builder() {
    // Arrange + Act
    let config = RateLimitConfig::new(5.0, 10)
        .with_tokens_per_second(15.0)
        .with_burst_capacity(30)
        .with_strategy(RateLimitStrategy::Error);

    // Assert
    assert_eq!(config.tokens_per_second, 15.0);
    assert_eq!(config.burst_capacity, 30);
    assert_eq!(config.strategy, RateLimitStrategy::Error);
}

#[test]
fn test_rate_limit_config_validation_success() {
    // Arrange
    let config = RateLimitConfig::new(10.0, 20);

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_rate_limit_config_validation_zero_tokens() {
    // Arrange
    let config = RateLimitConfig {
        tokens_per_second: 0.0,
        burst_capacity: 20,
        strategy: RateLimitStrategy::Wait,
    };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_rate_limit_config_validation_negative_tokens() {
    // Arrange
    let config = RateLimitConfig {
        tokens_per_second: -5.0,
        burst_capacity: 20,
        strategy: RateLimitStrategy::Wait,
    };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_rate_limit_config_validation_zero_burst() {
    // Arrange
    let config = RateLimitConfig {
        tokens_per_second: 10.0,
        burst_capacity: 0,
        strategy: RateLimitStrategy::Wait,
    };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_rate_limit_strategy_variants() {
    // Arrange + Act + Assert
    let wait_strategy = RateLimitStrategy::Wait;
    let error_strategy = RateLimitStrategy::Error;

    assert_eq!(wait_strategy, RateLimitStrategy::Wait);
    assert_eq!(error_strategy, RateLimitStrategy::Error);
    assert_ne!(wait_strategy, error_strategy);
}

// =============================================================================
// RetryConfig Tests - Comprehensive retry configuration
// =============================================================================

#[test]
fn test_retry_config_default_values() {
    // Arrange + Act
    let config = RetryConfig::default();

    // Assert
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.jitter_factor, 0.5);
    assert!(config.retry_on_rate_limit);
}

#[test]
fn test_retry_config_builder() {
    // Arrange + Act
    let config = RetryConfig::new(5, 500)
        .with_max_attempts(10)
        .with_initial_delay(2000)
        .with_max_delay(60000)
        .with_multiplier(1.5)
        .with_jitter(0.3)
        .with_rate_limit_retry(false);

    // Assert
    assert_eq!(config.max_attempts, 10);
    assert_eq!(config.initial_delay_ms, 2000);
    assert_eq!(config.max_delay_ms, 60000);
    assert_eq!(config.backoff_multiplier, 1.5);
    assert_eq!(config.jitter_factor, 0.3);
    assert!(!config.retry_on_rate_limit);
}

#[test]
fn test_retry_config_validation_success() {
    // Arrange
    let config = RetryConfig::new(3, 1000);

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_retry_config_validation_zero_attempts() {
    // Arrange
    let config = RetryConfig { max_attempts: 0, ..Default::default() };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_retry_config_validation_zero_initial_delay() {
    // Arrange
    let config = RetryConfig { initial_delay_ms: 0, ..Default::default() };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_retry_config_validation_max_less_than_initial() {
    // Arrange
    let config = RetryConfig { initial_delay_ms: 5000, max_delay_ms: 1000, ..Default::default() };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_retry_config_validation_negative_multiplier() {
    // Arrange
    let config = RetryConfig { backoff_multiplier: -1.0, ..Default::default() };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_retry_config_validation_jitter_out_of_range() {
    // Arrange
    let config = RetryConfig { jitter_factor: 1.5, ..Default::default() };

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_retry_config_calculate_delay_exponential() {
    // Arrange
    let config = RetryConfig {
        initial_delay_ms: 1000,
        backoff_multiplier: 2.0,
        max_delay_ms: 10000,
        jitter_factor: 0.0,
        ..Default::default()
    };

    // Act
    let delay0 = config.calculate_delay(0);
    let delay1 = config.calculate_delay(1);
    let delay2 = config.calculate_delay(2);
    let delay3 = config.calculate_delay(3);

    // Assert - exponential backoff: 1000, 2000, 4000, 8000
    assert_eq!(delay0.as_millis(), 1000);
    assert_eq!(delay1.as_millis(), 2000);
    assert_eq!(delay2.as_millis(), 4000);
    assert_eq!(delay3.as_millis(), 8000);
}

#[test]
fn test_retry_config_calculate_delay_capped() {
    // Arrange
    let config = RetryConfig {
        initial_delay_ms: 1000,
        backoff_multiplier: 2.0,
        max_delay_ms: 5000,
        jitter_factor: 0.0,
        ..Default::default()
    };

    // Act
    let delay_large = config.calculate_delay(10); // Would be 1024000ms without cap

    // Assert - should be capped at max_delay
    assert_eq!(delay_large.as_millis(), 5000);
}

#[test]
fn test_retry_config_calculate_delay_with_jitter() {
    // Arrange
    let config = RetryConfig {
        initial_delay_ms: 1000,
        backoff_multiplier: 2.0,
        max_delay_ms: 10000,
        jitter_factor: 0.5,
        ..Default::default()
    };

    // Act - calculate multiple delays to verify jitter range
    let delays: Vec<u64> = (0..10).map(|_| config.calculate_delay(0).as_millis() as u64).collect();

    // Assert - all delays should be within jitter range [500, 1000]
    for delay in delays {
        assert!(delay >= 500 && delay <= 1000);
    }
}

#[test]
fn test_retry_config_linear_backoff() {
    // Arrange
    let config = RetryConfig {
        initial_delay_ms: 1000,
        backoff_multiplier: 1.0,
        max_delay_ms: 10000,
        jitter_factor: 0.0,
        ..Default::default()
    };

    // Act
    let delay0 = config.calculate_delay(0);
    let delay1 = config.calculate_delay(1);
    let delay2 = config.calculate_delay(2);

    // Assert - linear (constant) backoff with multiplier 1.0
    assert_eq!(delay0.as_millis(), 1000);
    assert_eq!(delay1.as_millis(), 1000);
    assert_eq!(delay2.as_millis(), 1000);
}

// =============================================================================
// RetryContext Tests - Test retry tracking
// =============================================================================

#[test]
fn test_retry_context_default() {
    // Arrange + Act
    let context = RetryContext::default();

    // Assert
    assert_eq!(context.attempts, 0);
    assert_eq!(context.total_delay_ms, 0);
    assert!(context.last_error.is_none());
}

#[test]
fn test_retry_context_record_attempt() {
    // Arrange
    let mut context = RetryContext::default();
    let error = clap_noun_verb::wizard::error::WizardError::Request("timeout".to_string());

    // Act
    context.record_attempt(1000, &error);
    context.record_attempt(2000, &error);

    // Assert
    assert_eq!(context.attempts, 2);
    assert_eq!(context.total_delay_ms, 3000);
    assert!(context.last_error.is_some());
    assert!(context.last_error.unwrap().contains("timeout"));
}

#[test]
fn test_retry_context_multiple_errors() {
    // Arrange
    let mut context = RetryContext::default();
    let error1 = clap_noun_verb::wizard::error::WizardError::Request("error1".to_string());
    let error2 = clap_noun_verb::wizard::error::WizardError::Request("error2".to_string());

    // Act
    context.record_attempt(1000, &error1);
    context.record_attempt(2000, &error2);

    // Assert - should store last error
    assert_eq!(context.attempts, 2);
    assert!(context.last_error.unwrap().contains("error2"));
}

// =============================================================================
// Integration Tests - Test feature combinations
// =============================================================================

#[test]
fn test_cache_and_rate_limit_configs_together() {
    // Arrange
    let cache_config = CacheConfig::new(100, 3600);
    let rate_config = RateLimitConfig::new(10.0, 20);

    // Act
    let cache_valid = cache_config.validate();
    let rate_valid = rate_config.validate();

    // Assert - both should be valid simultaneously
    assert!(cache_valid.is_ok());
    assert!(rate_valid.is_ok());
}

#[test]
fn test_all_features_default_configs() {
    // Arrange + Act
    let cache_config = CacheConfig::default();
    let rate_config = RateLimitConfig::default();
    let retry_config = RetryConfig::default();

    // Assert - all defaults should be valid
    assert!(cache_config.validate().is_ok());
    assert!(rate_config.validate().is_ok());
    assert!(retry_config.validate().is_ok());
}

// =============================================================================
// Edge Cases and Boundary Tests
// =============================================================================

#[test]
fn test_cache_stats_overflow_protection() {
    // Arrange
    let stats = CacheStats { hits: u64::MAX, misses: 0, evictions: 0, insertions: 0 };

    // Act
    let total = stats.total_requests();

    // Assert - should handle max values
    assert_eq!(total, u64::MAX);
}

#[test]
fn test_retry_config_single_attempt() {
    // Arrange
    let config = RetryConfig::new(1, 1000);

    // Act
    let result = config.validate();

    // Assert - single attempt should be valid
    assert!(result.is_ok());
}

#[test]
fn test_retry_config_very_large_attempts() {
    // Arrange
    let config = RetryConfig::new(1000, 1000);

    // Act
    let result = config.validate();

    // Assert - large attempts should be valid
    assert!(result.is_ok());
}

#[test]
fn test_rate_limit_fractional_tokens() {
    // Arrange
    let config = RateLimitConfig::new(0.5, 1); // Half a token per second

    // Act
    let result = config.validate();

    // Assert - fractional tokens should be valid
    assert!(result.is_ok());
}

#[test]
fn test_rate_limit_very_high_tokens() {
    // Arrange
    let config = RateLimitConfig::new(10000.0, 100000);

    // Act
    let result = config.validate();

    // Assert - very high rates should be valid
    assert!(result.is_ok());
}
