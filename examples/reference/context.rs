//! Example: Global Application Context
//!
//! Demonstrates how to use AppContext<T> to share state across all commands.
//! This is useful for sharing database connections, configuration, loggers, etc.

use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;

// Shared application state
#[derive(Clone)]
struct AppConfig {
    database_url: String,
    api_key: String,
    debug: bool,
}

// Simple in-memory cache
#[derive(Clone)]
struct Cache {
    data: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl Cache {
    fn new() -> Self {
        Cache { data: Arc::new(Mutex::new(std::collections::HashMap::new())) }
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.lock().ok().and_then(|m| m.get(key).cloned())
    }

    fn set(&self, key: String, value: String) {
        if let Ok(mut data) = self.data.lock() {
            data.insert(key, value);
        }
    }
}

#[derive(Serialize)]
struct CacheResult {
    key: String,
    value: String,
    source: String,
}

#[derive(Serialize)]
struct ConfigInfo {
    database_url: String,
    debug: bool,
    has_api_key: bool,
}

/// Get configuration info
#[noun("app", "Application configuration")]
#[verb("config")]
fn show_config() -> Result<ConfigInfo> {
    // In a real app, you'd pass context through args or a global
    // For this example, we'll create fresh config
    let config = AppConfig {
        database_url: "postgresql://localhost/mydb".to_string(),
        api_key: "secret-key-12345".to_string(),
        debug: true,
    };

    Ok(ConfigInfo {
        database_url: config.database_url,
        debug: config.debug,
        has_api_key: !config.api_key.is_empty(),
    })
}

/// Get a value from cache
#[noun("cache", "In-memory cache management")]
#[verb("get")]
fn cache_get(key: String) -> Result<CacheResult> {
    let cache = Cache::new();

    // Try to get from cache
    let value = cache.get(&key).unwrap_or_else(|| {
        // Simulate fetching from source
        format!("value-for-{}", key)
    });

    let source =
        if cache.get(&key).is_some() { "cache".to_string() } else { "computed".to_string() };

    Ok(CacheResult { key, value, source })
}

/// Set a value in cache
#[noun("cache", "In-memory cache management")]
#[verb("set")]
fn cache_set(key: String, value: String) -> Result<CacheResult> {
    let cache = Cache::new();
    cache.set(key.clone(), value.clone());

    Ok(CacheResult { key, value, source: "cache".to_string() })
}

/// Show cache statistics
#[noun("cache", "In-memory cache management")]
#[verb("stats")]
fn cache_stats() -> Result<String> {
    Ok("Cache is empty (in-memory demo)".to_string())
}

fn main() -> Result<()> {
    // In a real application, you would:
    // 1. Create AppContext once at startup
    // 2. Insert shared state (database connections, config, etc.)
    // 3. Pass context to all handlers

    clap_noun_verb::run()
}
