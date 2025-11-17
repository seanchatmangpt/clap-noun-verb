//! Observability exporters for metrics output.

use crate::telemetry::metrics::MetricsCollector;

/// Trait for exporting metrics.
pub trait MetricsExporter {
    /// Export metrics to a format.
    ///
    /// # Errors
    ///
    /// Returns an error if export fails.
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String>;
}

/// Console exporter for human-readable output.
#[derive(Debug, Default)]
pub struct ConsoleExporter;

impl ConsoleExporter {
    /// Create a new console exporter.
    pub fn new() -> Self {
        Self
    }
}

impl MetricsExporter for ConsoleExporter {
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String> {
        let mut output = String::new();
        output.push_str("=== Metrics Report ===\n");
        output.push_str(&format!(
            "Total Commands Executed: {}\n",
            metrics.command_count()
        ));
        output.push_str(&format!("Total Errors: {}\n", metrics.error_count()));

        if let Some(mean) = metrics.execution_times().mean() {
            output.push_str(&format!("Average Execution Time: {:.2}ms\n", mean));
        }

        if let Some(min) = metrics.execution_times().min() {
            output.push_str(&format!("Min Execution Time: {}ms\n", min));
        }

        if let Some(max) = metrics.execution_times().max() {
            output.push_str(&format!("Max Execution Time: {}ms\n", max));
        }

        Ok(output)
    }
}

/// JSON exporter for machine-readable output.
#[derive(Debug, Default)]
pub struct JsonExporter;

impl JsonExporter {
    /// Create a new JSON exporter.
    pub fn new() -> Self {
        Self
    }
}

impl MetricsExporter for JsonExporter {
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String> {
        let mut json = serde_json::json!({
            "commands_executed": metrics.command_count(),
            "errors": metrics.error_count(),
        });

        if let Some(mean) = metrics.execution_times().mean() {
            json["avg_execution_ms"] = serde_json::json!(mean);
        }

        if let Some(min) = metrics.execution_times().min() {
            json["min_execution_ms"] = serde_json::json!(min);
        }

        if let Some(max) = metrics.execution_times().max() {
            json["max_execution_ms"] = serde_json::json!(max);
        }

        Ok(serde_json::to_string_pretty(&json).map_err(|e| {
            crate::NounVerbError::TelemetryError(format!("JSON serialization failed: {}", e))
        })?)
    }
}

/// Prometheus exporter for metrics in Prometheus format.
#[derive(Debug, Default)]
pub struct PrometheusExporter;

impl PrometheusExporter {
    /// Create a new Prometheus exporter.
    pub fn new() -> Self {
        Self
    }
}

impl MetricsExporter for PrometheusExporter {
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String> {
        let mut output = String::new();

        // Counter metrics
        output.push_str("# HELP command_executions_total Total number of command executions\n");
        output.push_str("# TYPE command_executions_total counter\n");
        output.push_str(&format!(
            "command_executions_total {}\n",
            metrics.command_count()
        ));

        output.push_str("# HELP command_errors_total Total number of command errors\n");
        output.push_str("# TYPE command_errors_total counter\n");
        output.push_str(&format!("command_errors_total {}\n", metrics.error_count()));

        // Histogram metrics
        if let Some(mean) = metrics.execution_times().mean() {
            output.push_str(
                "# HELP command_execution_duration_ms_bucket Command execution duration in milliseconds\n",
            );
            output.push_str("# TYPE command_execution_duration_ms_bucket histogram\n");
            output.push_str(&format!("command_execution_duration_ms {:.2}\n", mean));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_exporter() {
        let exporter = ConsoleExporter::new();
        let metrics = MetricsCollector::new();
        let result = exporter.export(&metrics);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Metrics Report"));
    }

    #[test]
    fn test_json_exporter() {
        let exporter = JsonExporter::new();
        let metrics = MetricsCollector::new();
        let result = exporter.export(&metrics);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("commands_executed"));
    }

    #[test]
    fn test_prometheus_exporter() {
        let exporter = PrometheusExporter::new();
        let metrics = MetricsCollector::new();
        let result = exporter.export(&metrics);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("command_executions_total"));
    }
}
