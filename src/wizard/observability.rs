//! Observability layer for wizard AI interactions
//!
//! This module provides metrics, tracing, and health checking for AI model interactions.
//!
//! ## Features
//!
//! - Token usage metrics
//! - Latency tracking (p50, p90, p99)
//! - Error rate monitoring
//! - Provider health checking
//! - OpenTelemetry integration (optional)

use crate::wizard::{
    config::{Model, Provider},
    error::{WizardError, WizardResult},
    types::{TokenUsage, WizardResponse},
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Health status for a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Provider is healthy and responsive
    Healthy,
    /// Provider is experiencing degraded performance
    Degraded,
    /// Provider is unavailable
    Unhealthy,
    /// Health status unknown (not yet checked)
    Unknown,
}

/// Health check result for a provider
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheck {
    /// Provider being checked
    pub provider: Provider,
    /// Health status
    pub status: HealthStatus,
    /// Response time for health check (milliseconds)
    pub response_time_ms: Option<u64>,
    /// Last check time
    pub last_check: Option<Instant>,
    /// Error message if unhealthy
    pub error: Option<String>,
}

impl HealthCheck {
    /// Create a new health check result
    pub fn new(provider: Provider) -> Self {
        Self { provider, status: HealthStatus::Unknown, response_time_ms: None, last_check: None, error: None }
    }

    /// Update health status
    pub fn update_status(&mut self, status: HealthStatus, response_time_ms: Option<u64>, error: Option<String>) {
        self.status = status;
        self.response_time_ms = response_time_ms;
        self.last_check = Some(Instant::now());
        self.error = error;
    }
}

/// Metrics collector for wizard operations
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Total requests made
    total_requests: usize,
    /// Total errors encountered
    total_errors: usize,
    /// Total tokens used
    total_tokens: usize,
    /// Latency samples (milliseconds)
    latency_samples: Vec<u64>,
    /// Error counts by type
    error_counts: HashMap<String, usize>,
    /// Token usage by model
    token_usage_by_model: HashMap<String, usize>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            total_tokens: 0,
            latency_samples: Vec::new(),
            error_counts: HashMap::new(),
            token_usage_by_model: HashMap::new(),
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self, response: &WizardResponse, latency_ms: u64) {
        self.total_requests += 1;
        self.latency_samples.push(latency_ms);

        // Record token usage
        if let Some(usage) = &response.usage {
            self.total_tokens += usage.total_tokens;

            // Track by model
            *self.token_usage_by_model.entry(response.model.clone()).or_insert(0) += usage.total_tokens;
        }

        // Keep only last 1000 samples for percentile calculation
        if self.latency_samples.len() > 1000 {
            self.latency_samples.remove(0);
        }
    }

    /// Record an error
    pub fn record_error(&mut self, error: &WizardError) {
        self.total_requests += 1;
        self.total_errors += 1;

        // Count by error type
        let error_type = format!("{:?}", error).split('(').next().unwrap_or("Unknown").to_string();
        *self.error_counts.entry(error_type).or_insert(0) += 1;
    }

    /// Get error rate (0.0 to 1.0)
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_errors as f64 / self.total_requests as f64
    }

    /// Get average latency in milliseconds
    pub fn avg_latency_ms(&self) -> u64 {
        if self.latency_samples.is_empty() {
            return 0;
        }
        self.latency_samples.iter().sum::<u64>() / self.latency_samples.len() as u64
    }

    /// Get latency percentile (e.g., 0.50 for p50, 0.99 for p99)
    pub fn latency_percentile(&self, percentile: f64) -> u64 {
        if self.latency_samples.is_empty() {
            return 0;
        }

        let mut sorted = self.latency_samples.clone();
        sorted.sort_unstable();

        let index = ((sorted.len() as f64) * percentile) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0)
    }

    /// Get p50 latency (median)
    pub fn p50_latency_ms(&self) -> u64 {
        self.latency_percentile(0.50)
    }

    /// Get p90 latency
    pub fn p90_latency_ms(&self) -> u64 {
        self.latency_percentile(0.90)
    }

    /// Get p99 latency
    pub fn p99_latency_ms(&self) -> u64 {
        self.latency_percentile(0.99)
    }

    /// Get total tokens used
    pub const fn total_tokens(&self) -> usize {
        self.total_tokens
    }

    /// Get token usage by model
    pub fn tokens_by_model(&self, model: &str) -> usize {
        self.token_usage_by_model.get(model).copied().unwrap_or(0)
    }

    /// Get error count by type
    pub fn errors_by_type(&self, error_type: &str) -> usize {
        self.error_counts.get(error_type).copied().unwrap_or(0)
    }

    /// Get all metrics as a summary
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_requests: self.total_requests,
            total_errors: self.total_errors,
            error_rate: self.error_rate(),
            total_tokens: self.total_tokens,
            avg_latency_ms: self.avg_latency_ms(),
            p50_latency_ms: self.p50_latency_ms(),
            p90_latency_ms: self.p90_latency_ms(),
            p99_latency_ms: self.p99_latency_ms(),
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.total_requests = 0;
        self.total_errors = 0;
        self.total_tokens = 0;
        self.latency_samples.clear();
        self.error_counts.clear();
        self.token_usage_by_model.clear();
    }
}

/// Metrics summary
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricsSummary {
    /// Total requests
    pub total_requests: usize,
    /// Total errors
    pub total_errors: usize,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Total tokens used
    pub total_tokens: usize,
    /// Average latency (milliseconds)
    pub avg_latency_ms: u64,
    /// P50 latency (milliseconds)
    pub p50_latency_ms: u64,
    /// P90 latency (milliseconds)
    pub p90_latency_ms: u64,
    /// P99 latency (milliseconds)
    pub p99_latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_new() {
        // Arrange + Act
        let health = HealthCheck::new(Provider::OpenAI);

        // Assert
        assert_eq!(health.provider, Provider::OpenAI);
        assert_eq!(health.status, HealthStatus::Unknown);
        assert!(health.response_time_ms.is_none());
    }

    #[test]
    fn test_health_check_update() {
        // Arrange
        let mut health = HealthCheck::new(Provider::Anthropic);

        // Act
        health.update_status(HealthStatus::Healthy, Some(150), None);

        // Assert
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.response_time_ms, Some(150));
        assert!(health.last_check.is_some());
    }

    #[test]
    fn test_metrics_collector_new() {
        // Arrange + Act
        let collector = MetricsCollector::new();

        // Assert
        assert_eq!(collector.total_requests, 0);
        assert_eq!(collector.total_errors, 0);
        assert_eq!(collector.total_tokens, 0);
    }

    #[test]
    fn test_metrics_record_success() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let usage = TokenUsage::new(100, 50);
        let response = WizardResponse::new("test", "gpt-4").with_usage(usage);

        // Act
        collector.record_success(&response, 200);

        // Assert
        assert_eq!(collector.total_requests, 1);
        assert_eq!(collector.total_tokens, 150);
        assert_eq!(collector.avg_latency_ms(), 200);
    }

    #[test]
    fn test_metrics_record_error() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let error = WizardError::Request("test error".to_string());

        // Act
        collector.record_error(&error);

        // Assert
        assert_eq!(collector.total_requests, 1);
        assert_eq!(collector.total_errors, 1);
        assert_eq!(collector.error_rate(), 1.0);
    }

    #[test]
    fn test_metrics_error_rate() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let usage = TokenUsage::new(100, 50);
        let response = WizardResponse::new("test", "gpt-4").with_usage(usage);

        // Act - 2 successes, 1 error
        collector.record_success(&response, 200);
        collector.record_success(&response, 150);
        collector.record_error(&WizardError::Request("error".to_string()));

        // Assert - error rate should be 1/3 = 0.333...
        assert!((collector.error_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_metrics_latency_percentiles() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let usage = TokenUsage::new(100, 50);
        let response = WizardResponse::new("test", "gpt-4").with_usage(usage);

        // Act - Add latencies: 100, 200, 300, 400, 500
        for latency in [100, 200, 300, 400, 500] {
            collector.record_success(&response, latency);
        }

        // Assert
        assert_eq!(collector.p50_latency_ms(), 300); // Median
        assert_eq!(collector.p90_latency_ms(), 450); // 90th percentile
    }

    #[test]
    fn test_metrics_tokens_by_model() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let usage = TokenUsage::new(100, 50);
        let response1 = WizardResponse::new("test", "gpt-4").with_usage(usage);
        let response2 = WizardResponse::new("test", "gpt-3.5-turbo").with_usage(usage);

        // Act
        collector.record_success(&response1, 200);
        collector.record_success(&response1, 200);
        collector.record_success(&response2, 150);

        // Assert
        assert_eq!(collector.tokens_by_model("gpt-4"), 300); // 2 * 150
        assert_eq!(collector.tokens_by_model("gpt-3.5-turbo"), 150); // 1 * 150
    }

    #[test]
    fn test_metrics_summary() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let usage = TokenUsage::new(100, 50);
        let response = WizardResponse::new("test", "gpt-4").with_usage(usage);

        // Act
        collector.record_success(&response, 200);
        collector.record_error(&WizardError::Request("error".to_string()));
        let summary = collector.summary();

        // Assert
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_errors, 1);
        assert_eq!(summary.total_tokens, 150);
    }

    #[test]
    fn test_metrics_reset() {
        // Arrange
        let mut collector = MetricsCollector::new();
        let usage = TokenUsage::new(100, 50);
        let response = WizardResponse::new("test", "gpt-4").with_usage(usage);
        collector.record_success(&response, 200);

        // Act
        collector.reset();

        // Assert
        assert_eq!(collector.total_requests, 0);
        assert_eq!(collector.total_tokens, 0);
    }
}
