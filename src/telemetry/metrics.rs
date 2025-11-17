//! Metrics collection and recording.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Counter metric for tracking occurrences.
#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    value: Arc<RwLock<u64>>,
    labels: HashMap<String, String>,
}

impl Counter {
    /// Create a new counter.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Arc::new(RwLock::new(0)),
            labels: HashMap::new(),
        }
    }

    /// Increment the counter.
    pub fn inc(&self) {
        let mut val = self.value.write();
        *val = val.saturating_add(1);
    }

    /// Increment by a specific amount.
    pub fn add(&self, amount: u64) {
        let mut val = self.value.write();
        *val = val.saturating_add(amount);
    }

    /// Get the current value.
    pub fn value(&self) -> u64 {
        *self.value.read()
    }

    /// Get the counter name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Get the labels.
    pub fn labels(&self) -> &HashMap<String, String> {
        &self.labels
    }
}

/// Histogram metric for recording distributions.
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    values: Arc<RwLock<Vec<u64>>>,
    labels: HashMap<String, String>,
}

impl Histogram {
    /// Create a new histogram.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: Arc::new(RwLock::new(Vec::new())),
            labels: HashMap::new(),
        }
    }

    /// Record a value.
    pub fn record(&self, value: u64) {
        let mut vals = self.values.write();
        vals.push(value);
    }

    /// Get all recorded values.
    pub fn values(&self) -> Vec<u64> {
        self.values.read().clone()
    }

    /// Get the mean value.
    pub fn mean(&self) -> Option<f64> {
        let vals = self.values.read();
        if vals.is_empty() {
            return None;
        }
        Some(vals.iter().sum::<u64>() as f64 / vals.len() as f64)
    }

    /// Get the min value.
    pub fn min(&self) -> Option<u64> {
        self.values.read().iter().copied().min()
    }

    /// Get the max value.
    pub fn max(&self) -> Option<u64> {
        self.values.read().iter().copied().max()
    }

    /// Get the 50th percentile (median).
    pub fn p50(&self) -> Option<u64> {
        self.percentile(50)
    }

    /// Get the 95th percentile.
    pub fn p95(&self) -> Option<u64> {
        self.percentile(95)
    }

    /// Get the 99th percentile.
    pub fn p99(&self) -> Option<u64> {
        self.percentile(99)
    }

    fn percentile(&self, p: usize) -> Option<u64> {
        let mut vals = self.values.read().clone();
        if vals.is_empty() {
            return None;
        }
        vals.sort_unstable();
        let idx = (vals.len() * p) / 100;
        Some(vals[idx.min(vals.len() - 1)])
    }

    /// Get the histogram name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Get the labels.
    pub fn labels(&self) -> &HashMap<String, String> {
        &self.labels
    }
}

/// Metrics collector for aggregating all metrics.
#[derive(Debug)]
pub struct MetricsCollector {
    counters: HashMap<String, Counter>,
    histograms: HashMap<String, Histogram>,
    command_executions: Counter,
    command_errors: Counter,
    execution_times: Histogram,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            histograms: HashMap::new(),
            command_executions: Counter::new("command_executions_total"),
            command_errors: Counter::new("command_errors_total"),
            execution_times: Histogram::new("command_execution_duration_ms"),
        }
    }

    /// Record a command execution.
    ///
    /// # Errors
    ///
    /// Returns an error if recording fails.
    pub fn record_command_execution(&self, _command: &str, duration_ms: u64) -> crate::Result<()> {
        self.command_executions.inc();
        self.execution_times.record(duration_ms);
        Ok(())
    }

    /// Record a command error.
    ///
    /// # Errors
    ///
    /// Returns an error if recording fails.
    pub fn record_command_error(&self, _command: &str, _error: &str) -> crate::Result<()> {
        self.command_errors.inc();
        Ok(())
    }

    /// Register a counter.
    pub fn register_counter(&mut self, name: impl Into<String>, counter: Counter) {
        self.counters.insert(name.into(), counter);
    }

    /// Register a histogram.
    pub fn register_histogram(&mut self, name: impl Into<String>, histogram: Histogram) {
        self.histograms.insert(name.into(), histogram);
    }

    /// Get a counter.
    pub fn get_counter(&self, name: &str) -> Option<&Counter> {
        self.counters.get(name)
    }

    /// Get a histogram.
    pub fn get_histogram(&self, name: &str) -> Option<&Histogram> {
        self.histograms.get(name)
    }

    /// Get all counters.
    pub fn counters(&self) -> &HashMap<String, Counter> {
        &self.counters
    }

    /// Get all histograms.
    pub fn histograms(&self) -> &HashMap<String, Histogram> {
        &self.histograms
    }

    /// Get the total command executions.
    pub fn command_count(&self) -> u64 {
        self.command_executions.value()
    }

    /// Get the total command errors.
    pub fn error_count(&self) -> u64 {
        self.command_errors.value()
    }

    /// Get the execution time histogram.
    pub fn execution_times(&self) -> &Histogram {
        &self.execution_times
    }

    /// Clear all metrics.
    pub fn clear(&mut self) {
        self.counters.clear();
        self.histograms.clear();
        *self.command_executions.value.write() = 0;
        *self.command_errors.value.write() = 0;
        *self.execution_times.values.write() = Vec::new();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_creation() {
        let counter = Counter::new("test");
        assert_eq!(counter.value(), 0);
    }

    #[test]
    fn test_counter_increment() {
        let counter = Counter::new("test");
        counter.inc();
        assert_eq!(counter.value(), 1);
        counter.add(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_histogram_creation() {
        let hist = Histogram::new("latency");
        assert!(hist.values().is_empty());
    }

    #[test]
    fn test_histogram_record() {
        let hist = Histogram::new("latency");
        hist.record(100);
        hist.record(200);
        hist.record(150);
        assert_eq!(hist.values().len(), 3);
    }

    #[test]
    fn test_histogram_percentiles() {
        let hist = Histogram::new("latency");
        for i in 1..=100 {
            hist.record(i);
        }
        assert!(hist.p50().is_some());
        assert!(hist.p95().is_some());
        assert!(hist.p99().is_some());
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.command_count(), 0);
        collector.record_command_execution("test", 100).unwrap();
        assert_eq!(collector.command_count(), 1);
    }
}
