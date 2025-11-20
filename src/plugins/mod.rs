//! 10 Production Plugins with Chicago-TDD Testing
//!
//! This module contains production-grade plugins demonstrating:
//! - Chicago-TDD style integration testing (no mocks, real components)
//! - Plugin lifecycle management
//! - Inter-plugin collaboration
//! - Error recovery and resilience
//!
//! Plugins Implemented:
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
//!
//! Each plugin includes:
//! - Full implementation (~300-500 lines)
//! - Chicago-TDD integration tests
//! - Multi-plugin workflow tests
//! - Error recovery scenarios

pub mod auth_manager;
pub mod cache;
pub mod circuit_breaker;
pub mod config_manager;
pub mod database_pool;
pub mod event_bus;
pub mod logger;
pub mod message_queue;
pub mod metrics_aggregator;
pub mod rate_limiter;

// Re-exports
pub use auth_manager::AuthManagerPlugin;
pub use cache::CacheManagerPlugin;
pub use circuit_breaker::CircuitBreakerPlugin;
pub use config_manager::ConfigManagerPlugin;
pub use database_pool::DatabasePoolPlugin;
pub use event_bus::EventBusPlugin;
pub use logger::LoggerPlugin;
pub use message_queue::MessageQueuePlugin;
pub use metrics_aggregator::MetricsAggregatorPlugin;
pub use rate_limiter::RateLimiterPlugin;

/// Test utilities for Chicago-TDD style testing
#[cfg(test)]
pub mod test_utils {
    use crate::plugin::{Plugin, PluginRegistry};
    use parking_lot::Mutex;
    use std::sync::Arc;

    /// Test fixture for plugin integration testing
    pub struct PluginTestFixture {
        registry: Arc<Mutex<PluginRegistry>>,
    }

    impl PluginTestFixture {
        /// Create a new test fixture
        pub fn new() -> Self {
            Self { registry: Arc::new(Mutex::new(PluginRegistry::new())) }
        }

        /// Register a plugin
        pub fn register_plugin(&self, plugin: Box<dyn Plugin>) -> crate::Result<()> {
            self.registry.lock().register(plugin)
        }

        /// Get the registry
        pub fn registry(&self) -> Arc<Mutex<PluginRegistry>> {
            self.registry.clone()
        }

        /// Get plugin list
        pub fn plugin_list(&self) -> Vec<String> {
            self.registry.lock().list_all().iter().map(|s| s.to_string()).collect()
        }
    }

    impl Default for PluginTestFixture {
        fn default() -> Self {
            Self::new()
        }
    }
}
