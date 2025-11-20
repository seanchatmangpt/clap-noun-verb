//! Metrics Aggregator Plugin - Record and aggregate metrics
//! See PLUGIN_IMPLEMENTATION_GUIDE.md for full specification

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Metric statistics
#[derive(Clone, Debug)]
pub struct MetricStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

#[derive(Clone)]
pub struct MetricsAggregatorPlugin {
    metrics: Arc<Mutex<HashMap<String, Vec<f64>>>>,
    loaded: bool,
}

impl MetricsAggregatorPlugin {
    pub fn new() -> Self {
        Self { metrics: Arc::new(Mutex::new(HashMap::new())), loaded: false }
    }

    pub fn record(&self, name: &str, value: f64) -> crate::Result<()> {
        let mut metrics = self.metrics.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Metrics lock failed".to_string())
        })?;
        metrics.entry(name.to_string()).or_insert_with(Vec::new).push(value);
        Ok(())
    }

    pub fn get_stats(&self, name: &str) -> crate::Result<Option<MetricStats>> {
        let metrics = self.metrics.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Metrics lock failed".to_string())
        })?;

        Ok(metrics.get(name).map(|values| {
            let count = values.len() as u64;
            let sum: f64 = values.iter().sum();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = sum / count as f64;
            MetricStats { count, sum, min, max, avg }
        }))
    }

    pub fn clear_metric(&self, name: &str) -> crate::Result<()> {
        let mut metrics = self.metrics.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Metrics lock failed".to_string())
        })?;
        metrics.remove(name);
        Ok(())
    }

    pub fn all_metrics(&self) -> crate::Result<Vec<String>> {
        let metrics = self.metrics.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Metrics lock failed".to_string())
        })?;
        Ok(metrics.keys().cloned().collect())
    }

    pub fn clear_all(&self) -> crate::Result<()> {
        let mut metrics = self.metrics.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Metrics lock failed".to_string())
        })?;
        metrics.clear();
        Ok(())
    }
}

impl Default for MetricsAggregatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MetricsAggregatorPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsAggregatorPlugin").finish()
    }
}

impl Plugin for MetricsAggregatorPlugin {
    fn name(&self) -> &str {
        "metrics-aggregator"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_description("Metrics aggregation with statistics")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Middleware]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chicago-TDD: Integration tests with real metrics aggregator
    #[test]
    fn test_metrics_record_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("requests", 100.0).unwrap();
        plugin.record("requests", 200.0).unwrap();
        plugin.record("requests", 150.0).unwrap();

        let stats = plugin.get_stats("requests").unwrap();
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 450.0);
        assert_eq!(stats.min, 100.0);
        assert_eq!(stats.max, 200.0);
    }

    #[test]
    fn test_metrics_average_calculation_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        for i in 1..=10 {
            plugin.record("latency", i as f64 * 10.0).unwrap();
        }

        let stats = plugin.get_stats("latency").unwrap().unwrap();
        assert_eq!(stats.count, 10);
        assert_eq!(stats.avg, 55.0); // (10+20+...+100)/10 = 55
    }

    #[test]
    fn test_metrics_multiple_metrics_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("cpu", 50.0).unwrap();
        plugin.record("cpu", 60.0).unwrap();
        plugin.record("memory", 800.0).unwrap();
        plugin.record("memory", 900.0).unwrap();

        let cpu_stats = plugin.get_stats("cpu").unwrap().unwrap();
        assert_eq!(cpu_stats.count, 2);

        let mem_stats = plugin.get_stats("memory").unwrap().unwrap();
        assert_eq!(mem_stats.count, 2);
    }

    #[test]
    fn test_metrics_clear_metric_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("temp", 25.0).unwrap();
        plugin.record("humidity", 60.0).unwrap();

        plugin.clear_metric("temp").unwrap();

        assert!(plugin.get_stats("temp").unwrap().is_none());
        assert!(plugin.get_stats("humidity").unwrap().is_some());
    }

    #[test]
    fn test_metrics_all_metrics_listing_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("m1", 1.0).unwrap();
        plugin.record("m2", 2.0).unwrap();
        plugin.record("m3", 3.0).unwrap();

        let metrics = plugin.all_metrics().unwrap();
        assert_eq!(metrics.len(), 3);
        assert!(metrics.contains(&"m1".to_string()));
        assert!(metrics.contains(&"m2".to_string()));
        assert!(metrics.contains(&"m3".to_string()));
    }

    #[test]
    fn test_metrics_min_max_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("values", 5.0).unwrap();
        plugin.record("values", 15.0).unwrap();
        plugin.record("values", 10.0).unwrap();
        plugin.record("values", 3.0).unwrap();
        plugin.record("values", 20.0).unwrap();

        let stats = plugin.get_stats("values").unwrap().unwrap();
        assert_eq!(stats.min, 3.0);
        assert_eq!(stats.max, 20.0);
    }

    #[test]
    fn test_metrics_clear_all_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("a", 1.0).unwrap();
        plugin.record("b", 2.0).unwrap();
        plugin.record("c", 3.0).unwrap();

        plugin.clear_all().unwrap();

        let metrics = plugin.all_metrics().unwrap();
        assert_eq!(metrics.len(), 0);
    }

    #[test]
    fn test_metrics_single_value_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        plugin.record("single", 42.0).unwrap();

        let stats = plugin.get_stats("single").unwrap().unwrap();
        assert_eq!(stats.count, 1);
        assert_eq!(stats.sum, 42.0);
        assert_eq!(stats.min, 42.0);
        assert_eq!(stats.max, 42.0);
        assert_eq!(stats.avg, 42.0);
    }

    #[test]
    fn test_metrics_concurrent_recording_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        let plugin = Arc::new(plugin);
        let mut handles = vec![];

        for i in 0..10 {
            let p = Arc::clone(&plugin);
            let handle = std::thread::spawn(move || {
                for j in 0..10 {
                    p.record("concurrent", (i * 10 + j) as f64).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = plugin.get_stats("concurrent").unwrap().unwrap();
        assert_eq!(stats.count, 100);
    }

    #[test]
    fn test_metrics_nonexistent_metric_workflow() {
        let mut plugin = MetricsAggregatorPlugin::new();
        plugin.load().unwrap();

        let stats = plugin.get_stats("nonexistent").unwrap();
        assert!(stats.is_none());
    }
}
