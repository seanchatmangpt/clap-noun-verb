//! Telemetry & Observability System (Feature 5 - v4.3)
//!
//! Comprehensive observability infrastructure with tracing, metrics, and structured logging.
//!
//! # Features
//!
//! - Structured distributed tracing with span creation
//! - Comprehensive metrics (counters, histograms, gauges)
//! - Multiple export formats (console, JSON lines, Prometheus)
//! - Automatic command instrumentation
//! - Context propagation for async operations
//!
//! # Example
//!
//! ```ignore
//! use clap_noun_verb::telemetry::{TelemetryCollector, MetricsCollector};
//!
//! let mut telemetry = TelemetryCollector::new();
//! let metrics = MetricsCollector::new();
//! metrics.record_command_execution("start", 150)?;
//! ```

pub mod exporters;
pub mod metrics;
pub mod tracing;

use std::fmt;

pub use exporters::{ConsoleExporter, JsonExporter, MetricsExporter, PrometheusExporter};
pub use metrics::{Counter, Histogram, MetricsCollector};
pub use tracing::{Span, SpanBuilder, TracingCollector};

/// Telemetry collector for aggregating metrics and traces.
#[derive(Debug)]
pub struct TelemetryCollector {
    metrics: MetricsCollector,
    tracing: TracingCollector,
    enabled: bool,
}

impl TelemetryCollector {
    /// Create a new telemetry collector.
    pub fn new() -> Self {
        Self { metrics: MetricsCollector::new(), tracing: TracingCollector::new(), enabled: true }
    }

    /// Enable the telemetry collector.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the telemetry collector.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if telemetry is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the metrics collector.
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics
    }

    /// Get the metrics collector mutably.
    pub fn metrics_mut(&mut self) -> &mut MetricsCollector {
        &mut self.metrics
    }

    /// Get the tracing collector.
    pub fn tracing(&self) -> &TracingCollector {
        &self.tracing
    }

    /// Get the tracing collector mutably.
    pub fn tracing_mut(&mut self) -> &mut TracingCollector {
        &mut self.tracing
    }

    /// Record a command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if recording fails.
    pub fn record_command(&self, command: &str, duration_ms: u64) -> crate::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.metrics.record_command_execution(command, duration_ms)?;
        Ok(())
    }

    /// Record a command error.
    ///
    /// # Errors
    ///
    /// Returns an error if recording fails.
    pub fn record_error(&self, command: &str, error: &str) -> crate::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        self.metrics.record_command_error(command, error)?;
        Ok(())
    }

    /// Create a span for an operation.
    pub fn span(&self, name: impl Into<String>) -> SpanBuilder {
        SpanBuilder::new(name)
    }

    /// Export metrics using the specified exporter.
    ///
    /// # Errors
    ///
    /// Returns an error if export fails.
    pub fn export_metrics<E: MetricsExporter>(&self, exporter: &E) -> crate::Result<String> {
        exporter.export(&self.metrics)
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TelemetryCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TelemetryCollector(metrics: {}, traces: {}, enabled: {})",
            self.metrics.command_count(),
            self.tracing.span_count(),
            self.enabled
        )
    }
}

/// Configuration for telemetry collection.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Enable metrics collection
    metrics_enabled: bool,
    /// Enable tracing
    tracing_enabled: bool,
    /// Sample rate for tracing (0.0 - 1.0)
    sample_rate: f64,
    /// Maximum number of spans to keep in memory
    max_spans: usize,
    /// Maximum number of metrics to keep
    max_metrics: usize,
}

impl TelemetryConfig {
    /// Create a new telemetry configuration.
    pub fn new() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            sample_rate: 1.0,
            max_spans: 10000,
            max_metrics: 10000,
        }
    }

    /// Enable metrics collection.
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.metrics_enabled = enabled;
        self
    }

    /// Enable tracing.
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }

    /// Set the tracing sample rate.
    pub fn with_sample_rate(mut self, rate: f64) -> Self {
        self.sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set the maximum number of spans.
    pub fn with_max_spans(mut self, max: usize) -> Self {
        self.max_spans = max;
        self
    }

    /// Set the maximum number of metrics.
    pub fn with_max_metrics(mut self, max: usize) -> Self {
        self.max_metrics = max;
        self
    }

    /// Check if metrics are enabled.
    pub fn is_metrics_enabled(&self) -> bool {
        self.metrics_enabled
    }

    /// Check if tracing is enabled.
    pub fn is_tracing_enabled(&self) -> bool {
        self.tracing_enabled
    }

    /// Get the sample rate.
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Get the max spans.
    pub fn max_spans(&self) -> usize {
        self.max_spans
    }

    /// Get the max metrics.
    pub fn max_metrics(&self) -> usize {
        self.max_metrics
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_collector_creation() {
        let collector = TelemetryCollector::new();
        assert!(collector.is_enabled());
    }

    #[test]
    fn test_telemetry_collector_enable_disable() {
        let mut collector = TelemetryCollector::new();
        collector.disable();
        assert!(!collector.is_enabled());
        collector.enable();
        assert!(collector.is_enabled());
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.is_metrics_enabled());
        assert!(config.is_tracing_enabled());
        assert_eq!(config.sample_rate(), 1.0);
    }

    #[test]
    fn test_telemetry_config_with_sample_rate() {
        let config = TelemetryConfig::new().with_sample_rate(0.5);
        assert_eq!(config.sample_rate(), 0.5);
    }

    #[test]
    fn test_telemetry_config_sample_rate_clamping() {
        let config1 = TelemetryConfig::new().with_sample_rate(2.0);
        assert_eq!(config1.sample_rate(), 1.0);

        let config2 = TelemetryConfig::new().with_sample_rate(-0.5);
        assert_eq!(config2.sample_rate(), 0.0);
    }
}
