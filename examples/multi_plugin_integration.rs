//! Multi-Plugin Integration Example
//!
//! Demonstrates how to use multiple plugins together in a real-world scenario:
//! - Auth Manager for user authentication
//! - Logger for recording events
//! - Metrics Aggregator for tracking statistics
//! - Rate Limiter for API rate limiting
//! - Cache Manager for caching user data
//!
//! This example simulates an API server handling user requests

use clap_noun_verb::plugin::Plugin;
use std::sync::Arc;

fn main() {
    println!("=== Multi-Plugin Integration Example ===\n");

    // Initialize plugins
    let mut auth = clap_noun_verb::plugins::auth_manager::AuthManagerPlugin::new();
    auth.load().expect("Failed to load auth");

    let mut logger = clap_noun_verb::plugins::logger::LoggerPlugin::new();
    logger.load().expect("Failed to load logger");

    let mut metrics = clap_noun_verb::plugins::metrics_aggregator::MetricsAggregatorPlugin::new();
    metrics.load().expect("Failed to load metrics");

    let mut rate_limiter = clap_noun_verb::plugins::rate_limiter::RateLimiterPlugin::new()
        .with_rate(10.0)
        .with_capacity(20.0);
    rate_limiter.load().expect("Failed to load rate limiter");

    let mut cache = clap_noun_verb::plugins::cache::CacheManagerPlugin::new().with_max_size(100);
    cache.load().expect("Failed to load cache");

    println!("✓ All plugins loaded\n");

    // Scenario 1: User Registration and Authentication
    println!("--- Scenario 1: User Registration ---");
    auth.register("alice", "secret123").expect("Failed to register");
    logger.info("User alice registered").expect("Log failed");
    println!("✓ User alice registered");

    auth.register("bob", "password456").expect("Failed to register");
    logger.info("User bob registered").expect("Log failed");
    println!("✓ User bob registered\n");

    // Scenario 2: Authentication with Rate Limiting
    println!("--- Scenario 2: Authentication with Rate Limiting ---");
    for i in 0..5 {
        if rate_limiter.allow_request("alice").expect("Rate limit check failed") {
            match auth.authenticate("alice", "secret123") {
                Ok(token) => {
                    cache.set("user:alice:token", format!("{}", token)).ok();
                    metrics.record("auth_success", 1.0).ok();
                    logger.info(&format!("Alice authenticated (attempt {})", i + 1)).ok();
                    println!("  ✓ Auth attempt {} succeeded", i + 1);
                }
                Err(_) => {
                    metrics.record("auth_failure", 1.0).ok();
                    logger.error("Authentication failed").ok();
                }
            }
        } else {
            metrics.record("auth_rate_limited", 1.0).ok();
            logger.warn("Auth request rate limited").ok();
            println!("  ⚠ Auth attempt {} rate limited", i + 1);
        }
    }
    println!();

    // Scenario 3: Cache and Metrics
    println!("--- Scenario 3: Data Caching ---");
    cache.set("user:alice:name", "Alice Smith").expect("Cache set failed");
    cache.set("user:alice:email", "alice@example.com").expect("Cache set failed");
    cache.set("user:bob:name", "Bob Jones").expect("Cache set failed");

    metrics.record("cache_writes", 3.0).ok();
    logger.info("User data cached").ok();

    if let Ok(Some(name)) = cache.get("user:alice:name") {
        metrics.record("cache_hits", 1.0).ok();
        logger.info("Cache hit: fetched user name").ok();
        println!("✓ Found cached user: {}", name);
    }

    if cache.get("user:bob:name").ok().flatten().is_some() {
        metrics.record("cache_hits", 1.0).ok();
        println!("✓ Found cached user: bob");
    }
    println!();

    // Scenario 4: Request Metrics
    println!("--- Scenario 4: Metrics Summary ---");
    if let Ok(Some(stats)) = metrics.get_stats("auth_success") {
        println!("✓ Successful authentications: {}", stats.count);
    }

    if let Ok(Some(stats)) = metrics.get_stats("cache_hits") {
        println!("✓ Cache hits: {}", stats.count);
    }

    if let Ok(Some(stats)) = metrics.get_stats("cache_writes") {
        println!("✓ Cache writes: {}", stats.sum as u64);
    }
    println!();

    // Scenario 5: User Roles and Authorization
    println!("--- Scenario 5: Role Management ---");
    auth.add_role("alice", "admin").ok();
    logger.info("Role 'admin' added to alice").ok();

    if let Ok(is_admin) = auth.has_role("alice", "admin") {
        println!("✓ Alice is admin: {}", is_admin);
    }

    if let Ok(is_mod) = auth.has_role("alice", "moderator") {
        println!("✓ Alice is moderator: {}", is_mod);
    }
    println!();

    // Scenario 6: Activity Logging
    println!("--- Scenario 6: Activity Log Summary ---");
    if let Ok(logs) = logger.get_logs() {
        println!("✓ Total log entries: {}", logs.len());
        println!("  Log levels:");
        if let Ok(infos) = logger.get_logs_by_level("INFO") {
            println!("    - INFO: {}", infos.len());
        }
        if let Ok(warns) = logger.get_logs_by_level("WARN") {
            println!("    - WARN: {}", warns.len());
        }
        if let Ok(errors) = logger.get_logs_by_level("ERROR") {
            println!("    - ERROR: {}", errors.len());
        }
    }
    println!();

    // Scenario 7: Concurrent Multi-User Scenario
    println!("--- Scenario 7: Concurrent Operations (Simulated) ---");
    let rate_limiter_arc = Arc::new(rate_limiter);
    let logger_arc = Arc::new(logger);
    let metrics_arc = Arc::new(metrics);

    let mut handles = vec![];
    for user_id in 0..3 {
        let rl = Arc::clone(&rate_limiter_arc);
        let lg = Arc::clone(&logger_arc);
        let mt = Arc::clone(&metrics_arc);

        let handle = std::thread::spawn(move || {
            for req_id in 0..5 {
                if rl.allow_request(&format!("user_{}", user_id)).ok().unwrap_or(false) {
                    mt.record("request_success", 1.0).ok();
                    lg.info(&format!("User {} request {} processed", user_id, req_id)).ok();
                } else {
                    mt.record("request_limited", 1.0).ok();
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().ok();
    }

    println!("✓ Concurrent operations completed");
    println!();

    // Scenario 8: Final Summary
    println!("--- Final Summary ---");
    if let Ok(users) = auth.list_users() {
        println!("✓ Registered users: {}", users.len());
    }
    println!("✓ All multi-plugin integration scenarios completed successfully!");
}
