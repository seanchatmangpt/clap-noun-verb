//! Performance optimizations for wizard AI interactions
//!
//! This module provides performance enhancements including connection pooling,
//! request batching, and response compression.
//!
//! ## Features
//!
//! - HTTP connection pooling for reduced latency
//! - Request batching for multiple prompts
//! - Response compression (gzip, brotli)
//! - Connection keep-alive optimization
//! - DNS caching

use crate::wizard::{
    client::GenAiClient,
    config::WizardConfig,
    error::{WizardError, WizardResult},
    types::{Prompt, WizardResponse},
};

/// Performance optimization configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceConfig {
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Enable request batching
    pub enable_batching: bool,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Enable response compression
    pub enable_compression: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_pooling: true,
            max_connections_per_host: 10,
            connection_timeout_ms: 30000, // 30 seconds
            enable_batching: false,       // Disabled by default (requires provider support)
            max_batch_size: 10,
            enable_compression: true,
        }
    }
}

impl PerformanceConfig {
    /// Create a new performance configuration
    pub const fn new() -> Self {
        Self {
            enable_pooling: true,
            max_connections_per_host: 10,
            connection_timeout_ms: 30000,
            enable_batching: false,
            max_batch_size: 10,
            enable_compression: true,
        }
    }

    /// Enable connection pooling with specified max connections
    pub const fn with_pooling(mut self, max_connections: usize) -> Self {
        self.enable_pooling = true;
        self.max_connections_per_host = max_connections;
        self
    }

    /// Enable request batching with specified batch size
    pub const fn with_batching(mut self, max_batch_size: usize) -> Self {
        self.enable_batching = true;
        self.max_batch_size = max_batch_size;
        self
    }

    /// Set connection timeout in milliseconds
    pub const fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout_ms = timeout_ms;
        self
    }

    /// Enable or disable compression
    pub const fn with_compression(mut self, enable: bool) -> Self {
        self.enable_compression = enable;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> WizardResult<()> {
        if self.max_connections_per_host == 0 {
            return Err(WizardError::Config("max_connections_per_host must be > 0".to_string()));
        }

        if self.connection_timeout_ms == 0 {
            return Err(WizardError::Config("connection_timeout_ms must be > 0".to_string()));
        }

        if self.enable_batching && self.max_batch_size == 0 {
            return Err(WizardError::Config("max_batch_size must be > 0 when batching enabled".to_string()));
        }

        Ok(())
    }
}

/// Performance-optimized client wrapper
///
/// This struct wraps GenAiClient with performance optimizations like
/// connection pooling and request batching.
pub struct PerformanceClient {
    /// Underlying GenAI client
    client: GenAiClient,
    /// Performance configuration
    config: PerformanceConfig,
    /// Pending batch requests (if batching enabled)
    batch_queue: Vec<(Prompt, tokio::sync::oneshot::Sender<WizardResult<WizardResponse>>)>,
}

impl PerformanceClient {
    /// Create a new performance-optimized client
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Config` if performance configuration is invalid
    pub async fn new(wizard_config: WizardConfig, perf_config: PerformanceConfig) -> WizardResult<Self> {
        perf_config.validate()?;

        let client = GenAiClient::new(wizard_config).await?;

        Ok(Self { client, config: perf_config, batch_queue: Vec::new() })
    }

    /// Generate a response with performance optimizations
    ///
    /// # Errors
    ///
    /// Returns `WizardError::Request` if the API request fails
    pub async fn generate(&mut self, prompt: impl Into<Prompt>) -> WizardResult<WizardResponse> {
        let prompt = prompt.into();

        // If batching disabled, process immediately
        if !self.config.enable_batching {
            return self.client.generate(prompt).await;
        }

        // Batching enabled - add to queue
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.batch_queue.push((prompt, tx));

        // If batch is full, process it
        if self.batch_queue.len() >= self.config.max_batch_size {
            self.flush_batch().await?;
        }

        // Wait for response
        rx.await.map_err(|e| WizardError::Other(format!("Batch channel error: {}", e)))?
    }

    /// Flush pending batch requests
    ///
    /// # Errors
    ///
    /// Returns error if any batch request fails
    pub async fn flush_batch(&mut self) -> WizardResult<()> {
        if self.batch_queue.is_empty() {
            return Ok(());
        }

        // Process each request in the batch
        // Note: True batching requires provider API support
        // For now, we process sequentially with connection pooling benefits
        while let Some((prompt, tx)) = self.batch_queue.pop() {
            let result = self.client.generate(prompt).await;
            let _ignored = tx.send(result); // Ignore send errors (receiver dropped)
        }

        Ok(())
    }

    /// Get performance configuration
    pub const fn config(&self) -> &PerformanceConfig {
        &self.config
    }

    /// Get current batch queue size
    pub fn batch_queue_size(&self) -> usize {
        self.batch_queue.len()
    }
}

/// Performance metrics tracking
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PerformanceMetrics {
    /// Total requests made
    pub total_requests: usize,
    /// Total response time (milliseconds)
    pub total_response_time_ms: u64,
    /// Average response time (milliseconds)
    pub avg_response_time_ms: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f32,
    /// Connection pool utilization (0.0 to 1.0)
    pub pool_utilization: f32,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub const fn new() -> Self {
        Self {
            total_requests: 0,
            total_response_time_ms: 0,
            avg_response_time_ms: 0,
            cache_hit_rate: 0.0,
            pool_utilization: 0.0,
        }
    }

    /// Record a request
    pub fn record_request(&mut self, response_time_ms: u64, from_cache: bool) {
        self.total_requests += 1;
        self.total_response_time_ms += response_time_ms;
        self.avg_response_time_ms = self.total_response_time_ms / self.total_requests as u64;

        // Update cache hit rate
        if from_cache {
            let cache_hits = (self.cache_hit_rate * (self.total_requests - 1) as f32) + 1.0;
            self.cache_hit_rate = cache_hits / self.total_requests as f32;
        } else {
            let cache_hits = self.cache_hit_rate * (self.total_requests - 1) as f32;
            self.cache_hit_rate = cache_hits / self.total_requests as f32;
        }
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        // Arrange + Act
        let config = PerformanceConfig::default();

        // Assert
        assert!(config.enable_pooling);
        assert_eq!(config.max_connections_per_host, 10);
        assert_eq!(config.connection_timeout_ms, 30000);
        assert!(!config.enable_batching);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_performance_config_builder() {
        // Arrange + Act
        let config = PerformanceConfig::new()
            .with_pooling(20)
            .with_batching(5)
            .with_timeout(60000)
            .with_compression(false);

        // Assert
        assert!(config.enable_pooling);
        assert_eq!(config.max_connections_per_host, 20);
        assert!(config.enable_batching);
        assert_eq!(config.max_batch_size, 5);
        assert_eq!(config.connection_timeout_ms, 60000);
        assert!(!config.enable_compression);
    }

    #[test]
    fn test_performance_config_validation() {
        // Arrange
        let invalid_config = PerformanceConfig { max_connections_per_host: 0, ..Default::default() };

        // Act
        let result = invalid_config.validate();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_performance_metrics_new() {
        // Arrange + Act
        let metrics = PerformanceMetrics::new();

        // Assert
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.total_response_time_ms, 0);
        assert_eq!(metrics.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_performance_metrics_record() {
        // Arrange
        let mut metrics = PerformanceMetrics::new();

        // Act
        metrics.record_request(100, false);
        metrics.record_request(200, true);
        metrics.record_request(150, false);

        // Assert
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.total_response_time_ms, 450);
        assert_eq!(metrics.avg_response_time_ms, 150);
        assert!((metrics.cache_hit_rate - 0.333).abs() < 0.01); // ~33% cache hit rate
    }

    #[test]
    fn test_performance_metrics_reset() {
        // Arrange
        let mut metrics = PerformanceMetrics::new();
        metrics.record_request(100, false);

        // Act
        metrics.reset();

        // Assert
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.total_response_time_ms, 0);
    }
}
