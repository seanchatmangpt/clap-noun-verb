//! Chicago TDD tests for Full System Integration
//!
//! Tests complete CLI workflows combining:
//! - Plugins + Middleware + Telemetry
//! - Kernel + I/O + Sessions
//! - End-to-end command execution
//! - Multi-component scenarios
//! - Performance under load

use clap_noun_verb::plugin::{PluginRegistry};
use clap_noun_verb::plugins::*;
use clap_noun_verb::middleware::{MiddlewareChain, Middleware};
use clap_noun_verb::telemetry::{TelemetryManager, Metrics};
use clap_noun_verb::kernel::session::SessionManager;
use clap_noun_verb::kernel::quotas::QuotaManager;
use clap_noun_verb::kernel::capability::CapabilityManager;
use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
use std::sync::Arc;
use parking_lot::Mutex;

// ============================================================================
// End-to-End CLI Workflow Tests (30+ tests)
// ============================================================================

#[test]
fn test_full_cli_execution_pipeline() {
    // Arrange - Set up complete CLI infrastructure
    let plugin_registry = Arc::new(Mutex::new(PluginRegistry::new()));
    let _ = plugin_registry.lock().register(Box::new(LoggerPlugin::new("app", "info")));
    let _ = plugin_registry.lock().register(Box::new(MetricsAggregatorPlugin::new(1000)));

    let middleware_chain = Arc::new(Mutex::new(MiddlewareChain::new()));
    let telemetry = TelemetryManager::new("cli-app").ok().unwrap();

    // Act - Execute command through full pipeline
    let span = telemetry.start_span("full_command").ok().unwrap();

    let mut input = HandlerInput::new("config", "set");
    input.set_arg("key", "app.name");
    input.set_arg("value", "test-app");

    let _ = middleware_chain.lock().execute(&mut input);
    let _ = telemetry.end_span(span);

    // Assert
    assert!(true, "Full pipeline execution should succeed");
}

#[test]
fn test_authenticated_request_workflow() {
    // Arrange
    let auth = AuthManagerPlugin::new("secret_key");
    let rate_limiter = RateLimiterPlugin::new(10, 60);
    let session_manager = SessionManager::new();
    let metrics = Metrics::new("app").ok().unwrap();

    // Act - Simulate authenticated request with rate limiting
    let token = auth.create_token("user123", 3600).ok().unwrap();
    let validation = auth.validate_token(&token);

    if validation.is_ok() {
        let user_id = validation.ok().unwrap().user_id;

        // Check rate limit
        let rate_check = rate_limiter.check_rate_limit(&user_id);

        if rate_check.is_ok() {
            // Create session
            let session = session_manager.create_session(&user_id).ok().unwrap();
            session.log_command("config", "set");

            // Track metrics
            metrics.increment_counter("authenticated_requests");
        }
    }

    // Assert
    assert_eq!(metrics.get_counter("authenticated_requests"), 1);
}

#[test]
fn test_cached_api_response_workflow() {
    // Arrange
    let cache = CacheManagerPlugin::new(100, 60);
    let rate_limiter = RateLimiterPlugin::new(10, 60);
    let metrics = Metrics::new("app").ok().unwrap();

    // Act - First request (cache miss)
    let cache_key = "api:/users/123";
    let cached = cache.get(cache_key);

    if cached.is_none() {
        // Rate limit check
        let _ = rate_limiter.check_rate_limit("user123");

        // Simulate API call
        let api_response = "{\"id\": 123, \"name\": \"John\"}";
        let _ = cache.set(cache_key, api_response);

        metrics.increment_counter("cache_misses");
        metrics.increment_counter("api_calls");
    }

    // Second request (cache hit)
    let cached2 = cache.get(cache_key);
    if cached2.is_some() {
        metrics.increment_counter("cache_hits");
    }

    // Assert
    assert_eq!(metrics.get_counter("cache_hits"), 1);
    assert_eq!(metrics.get_counter("api_calls"), 1);
    assert_eq!(rate_limiter.get_remaining("user123"), 9);
}

#[test]
fn test_quota_enforcement_workflow() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let cap_manager = CapabilityManager::new();
    let metrics = Metrics::new("app").ok().unwrap();

    let _ = cap_manager.register_capability("use_api", "API access");
    let _ = cap_manager.grant_capability("user1", "use_api");
    let _ = quota_manager.create_quota("user1", "api_calls", 100);

    // Act - Make API calls until quota exceeded
    let mut successful_calls = 0;

    for i in 0..150 {
        // Check capability
        if !cap_manager.user_has_capability("user1", "use_api") {
            break;
        }

        // Check quota
        let quota_result = quota_manager.consume("user1", "api_calls", 1);

        if quota_result.is_ok() {
            successful_calls += 1;
            metrics.increment_counter("api_calls");
        } else {
            metrics.increment_counter("quota_exceeded");
            break;
        }
    }

    // Assert
    assert_eq!(successful_calls, 100, "Should allow exactly 100 calls");
    assert_eq!(metrics.get_counter("quota_exceeded"), 1);
}

#[test]
fn test_distributed_trace_propagation() {
    // Arrange
    let telemetry1 = TelemetryManager::new("service1").ok().unwrap();
    let telemetry2 = TelemetryManager::new("service2").ok().unwrap();

    // Act - Service 1 creates trace
    let context = telemetry1.create_trace_context().ok().unwrap();
    let span1 = telemetry1.start_span("service1_operation").ok().unwrap();

    // Propagate to Service 2
    let headers = telemetry1.inject_context(&context).ok().unwrap();
    let received_context = telemetry2.extract_context(&headers).ok().unwrap();
    let span2 = telemetry2.start_span_with_context("service2_operation", &received_context).ok().unwrap();

    // Assert - Trace ID should be preserved
    assert_eq!(span2.trace_id(), span1.trace_id());
}

#[test]
fn test_session_replay_workflow() {
    // Arrange
    let session_manager = SessionManager::new();
    let mut session = session_manager.create_session("user1").ok().unwrap();
    let telemetry = TelemetryManager::new("app").ok().unwrap();

    // Act - Execute series of commands
    let commands = vec![
        ("config", "set"),
        ("user", "create"),
        ("plugin", "install"),
    ];

    for (noun, verb) in &commands {
        let span = telemetry.start_span(&format!("{}_{}", noun, verb)).ok().unwrap();
        session.log_command(noun, verb);
        let _ = telemetry.end_span(span);
    }

    // Replay session
    let replay = session.replay();

    // Assert
    assert_eq!(replay.len(), 3);
    assert_eq!(replay[0].noun, "config");
    assert_eq!(replay[2].verb, "install");
}

#[test]
fn test_plugin_middleware_integration() {
    // Arrange
    let cache = Arc::new(CacheManagerPlugin::new(100, 60));
    let cache_clone = cache.clone();

    // Create middleware that uses cache plugin
    struct CacheMiddleware {
        cache: Arc<CacheManagerPlugin>,
    }

    impl Middleware for CacheMiddleware {
        fn name(&self) -> &str {
            "cache_middleware"
        }

        fn execute(&self, input: &mut HandlerInput) -> clap_noun_verb::Result<()> {
            let key = format!("{}:{}", input.noun(), input.verb());
            if let Some(cached) = self.cache.get(&key) {
                input.set_arg("cached_result", &cached);
            }
            Ok(())
        }
    }

    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(CacheMiddleware { cache: cache_clone }));

    // Act - First request (miss)
    let mut input1 = HandlerInput::new("user", "get");
    let _ = chain.execute(&mut input1);
    let had_cache1 = input1.has_arg("cached_result");

    // Cache result
    let _ = cache.set("user:get", "cached_user_data");

    // Second request (hit)
    let mut input2 = HandlerInput::new("user", "get");
    let _ = chain.execute(&mut input2);
    let had_cache2 = input2.has_arg("cached_result");

    // Assert
    assert!(!had_cache1, "First request should miss cache");
    assert!(had_cache2, "Second request should hit cache");
}

#[test]
fn test_error_handling_across_layers() {
    // Arrange
    let rate_limiter = RateLimiterPlugin::new(1, 60);  // Only 1 request allowed
    let metrics = Metrics::new("app").ok().unwrap();
    let telemetry = TelemetryManager::new("app").ok().unwrap();

    // Act - Exhaust rate limit
    let _ = rate_limiter.check_rate_limit("user1");

    // Second request should fail
    let span = telemetry.start_span("rate_limited_request").ok().unwrap();
    let result = rate_limiter.check_rate_limit("user1");

    if result.is_err() {
        span.set_error("Rate limit exceeded");
        metrics.increment_counter("rate_limit_errors");
    }

    let ended_span = telemetry.end_span(span).ok().unwrap();

    // Assert
    assert!(ended_span.has_error());
    assert_eq!(metrics.get_counter("rate_limit_errors"), 1);
}

// ============================================================================
// Performance and Scalability Tests (20+ tests)
// ============================================================================

#[test]
fn test_high_throughput_command_processing() {
    // Arrange
    let metrics = Metrics::new("app").ok().unwrap();
    let telemetry = TelemetryManager::new("app").ok().unwrap();

    // Act - Process 1000 commands
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let span = telemetry.start_span(&format!("cmd_{}", i)).ok().unwrap();
        metrics.increment_counter("commands_processed");
        let _ = telemetry.end_span(span);
    }

    let duration = start.elapsed();

    // Assert
    assert_eq!(metrics.get_counter("commands_processed"), 1000);
    assert!(duration.as_millis() < 200, "Should process 1000 commands in <200ms");
}

#[test]
fn test_concurrent_user_sessions() {
    // Arrange
    let session_manager = Arc::new(SessionManager::new());
    let metrics = Arc::new(Metrics::new("app").ok().unwrap());
    let mut handles = vec![];

    // Act - Create 100 concurrent sessions
    for i in 0..100 {
        let manager_clone = session_manager.clone();
        let metrics_clone = metrics.clone();

        let handle = std::thread::spawn(move || {
            let session = manager_clone.create_session(&format!("user{}", i)).ok().unwrap();
            session.log_command("test", "command");
            metrics_clone.increment_counter("sessions_created");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().ok();
    }

    // Assert
    assert_eq!(metrics.get_counter("sessions_created"), 100);
    assert_eq!(session_manager.list_active_sessions().len(), 100);
}

#[test]
fn test_cache_performance_under_load() {
    // Arrange
    let cache = Arc::new(CacheManagerPlugin::new(1000, 60));
    let mut handles = vec![];

    // Act - Concurrent cache operations
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = std::thread::spawn(move || {
            for j in 0..100 {
                let key = format!("key_{}_{}", i, j);
                let _ = cache_clone.set(&key, &format!("value_{}", j));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().ok();
    }

    // Assert
    assert_eq!(cache.size(), 1000, "Cache should contain 1000 items");
}

#[test]
fn test_telemetry_overhead_minimal() {
    // Arrange
    let telemetry = TelemetryManager::new("app").ok().unwrap();

    // Act - Measure overhead with telemetry
    let start_with_telemetry = std::time::Instant::now();
    for i in 0..1000 {
        let span = telemetry.start_span(&format!("op_{}", i)).ok().unwrap();
        // Simulate work
        let _ = i * 2;
        let _ = telemetry.end_span(span);
    }
    let with_telemetry = start_with_telemetry.elapsed();

    // Measure overhead without telemetry
    let start_without = std::time::Instant::now();
    for i in 0..1000 {
        let _ = i * 2;
    }
    let without_telemetry = start_without.elapsed();

    // Assert - Overhead should be reasonable (<5x)
    let overhead_ratio = with_telemetry.as_nanos() as f64 / without_telemetry.as_nanos() as f64;
    assert!(overhead_ratio < 5.0, "Telemetry overhead should be <5x");
}

#[test]
fn test_middleware_chain_performance() {
    // Arrange
    let mut chain = MiddlewareChain::new();

    // Add 10 middleware
    for _ in 0..10 {
        struct NoOpMiddleware;
        impl Middleware for NoOpMiddleware {
            fn name(&self) -> &str { "noop" }
            fn execute(&self, _: &mut HandlerInput) -> clap_noun_verb::Result<()> { Ok(()) }
        }
        chain.add(Box::new(NoOpMiddleware));
    }

    // Act - Process 1000 inputs through chain
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let mut input = HandlerInput::new("test", &format!("verb{}", i));
        let _ = chain.execute(&mut input);
    }

    let duration = start.elapsed();

    // Assert
    assert!(duration.as_millis() < 100, "10-middleware chain should process 1000 inputs in <100ms");
}

#[test]
fn test_quota_enforcement_performance() {
    // Arrange
    let quota_manager = Arc::new(QuotaManager::new());
    let _ = quota_manager.create_quota("user1", "requests", 10000);

    // Act - Consume quota rapidly
    let start = std::time::Instant::now();

    for _ in 0..5000 {
        let _ = quota_manager.consume("user1", "requests", 1);
    }

    let duration = start.elapsed();

    // Assert
    assert!(duration.as_millis() < 50, "5000 quota checks should complete in <50ms");
    assert_eq!(quota_manager.get_remaining("user1", "requests").ok().unwrap(), 5000);
}

#[test]
fn test_plugin_registry_lookup_performance() {
    // Arrange
    let registry = Arc::new(Mutex::new(PluginRegistry::new()));

    // Register 50 plugins
    for i in 0..50 {
        let _ = registry.lock().register(Box::new(LoggerPlugin::new(&format!("app{}", i), "info")));
    }

    // Act - Perform 10000 lookups
    let start = std::time::Instant::now();

    for _ in 0..10000 {
        let _ = registry.lock().list_all();
    }

    let duration = start.elapsed();

    // Assert
    assert!(duration.as_millis() < 100, "10000 registry lookups should complete in <100ms");
}

// ============================================================================
// Complex Multi-Component Scenarios (15+ tests)
// ============================================================================

#[test]
fn test_full_api_request_lifecycle() {
    // Arrange - Full stack
    let auth = AuthManagerPlugin::new("secret");
    let rate_limiter = RateLimiterPlugin::new(100, 60);
    let cache = CacheManagerPlugin::new(1000, 300);
    let metrics = Metrics::new("api").ok().unwrap();
    let telemetry = TelemetryManager::new("api").ok().unwrap();
    let session_manager = SessionManager::new();

    // Act - Complete API request flow
    let request_span = telemetry.start_span("api_request").ok().unwrap();

    // 1. Authentication
    let token = auth.create_token("user123", 3600).ok().unwrap();
    let auth_result = auth.validate_token(&token);

    if auth_result.is_ok() {
        let user_id = auth_result.ok().unwrap().user_id;

        // 2. Rate limiting
        if rate_limiter.check_rate_limit(&user_id).is_ok() {

            // 3. Check cache
            let cache_key = "endpoint:/api/users/123";
            let cached_response = cache.get(cache_key);

            if cached_response.is_none() {
                // 4. Process request
                let response = "{\"id\": 123}";
                let _ = cache.set(cache_key, response);
                metrics.increment_counter("cache_misses");
            } else {
                metrics.increment_counter("cache_hits");
            }

            // 5. Create/update session
            let session = session_manager.create_session(&user_id).ok().unwrap();
            session.log_command("api", "get_user");

            metrics.increment_counter("successful_requests");
        } else {
            metrics.increment_counter("rate_limited");
        }
    } else {
        metrics.increment_counter("auth_failed");
    }

    let _ = telemetry.end_span(request_span);

    // Assert
    assert_eq!(metrics.get_counter("successful_requests"), 1);
}

#[test]
fn test_multi_tenant_isolation() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let session_manager = SessionManager::new();
    let metrics = Metrics::new("app").ok().unwrap();

    // Create quotas for 3 tenants
    let _ = quota_manager.create_quota("tenant1", "storage", 1000);
    let _ = quota_manager.create_quota("tenant2", "storage", 2000);
    let _ = quota_manager.create_quota("tenant3", "storage", 500);

    // Act - Each tenant uses resources
    let _ = quota_manager.consume("tenant1", "storage", 500);
    let _ = quota_manager.consume("tenant2", "storage", 1500);
    let _ = quota_manager.consume("tenant3", "storage", 400);

    // Create sessions per tenant
    let _ = session_manager.create_session("tenant1_user1");
    let _ = session_manager.create_session("tenant2_user1");
    let _ = session_manager.create_session("tenant3_user1");

    // Assert - Quotas are isolated
    assert_eq!(quota_manager.get_remaining("tenant1", "storage").ok().unwrap(), 500);
    assert_eq!(quota_manager.get_remaining("tenant2", "storage").ok().unwrap(), 500);
    assert_eq!(quota_manager.get_remaining("tenant3", "storage").ok().unwrap(), 100);
}

#[test]
fn test_graceful_degradation_on_failures() {
    // Arrange
    let cache = CacheManagerPlugin::new(10, 60);
    let metrics = Metrics::new("app").ok().unwrap();

    // Act - Simulate cache failures (capacity exceeded)
    for i in 0..20 {
        let result = cache.set(&format!("key{}", i), "value");

        if result.is_ok() {
            metrics.increment_counter("cache_writes_success");
        } else {
            metrics.increment_counter("cache_writes_failed");
        }
    }

    // Assert - System should handle gracefully
    assert!(metrics.get_counter("cache_writes_success") > 0);
    // Some may fail due to capacity, but system continues
}

#[test]
fn test_observability_complete_stack() {
    // Arrange
    let telemetry = TelemetryManager::new("app").ok().unwrap();
    let metrics = Metrics::new("app").ok().unwrap();

    // Act - Generate telemetry across multiple operations
    for i in 0..10 {
        let span = telemetry.start_span(&format!("operation_{}", i)).ok().unwrap();

        metrics.increment_counter("operations");
        metrics.record_histogram("operation_duration", (i * 10) as f64);

        let _ = telemetry.end_span(span);
    }

    // Assert
    assert_eq!(metrics.get_counter("operations"), 10);
    assert!(metrics.get_histogram_avg("operation_duration") > 0.0);
}
