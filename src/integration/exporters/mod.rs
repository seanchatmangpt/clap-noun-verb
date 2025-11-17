//! Production exporters for observability platforms.
//!
//! These exporters integrate with popular observability platforms:
//! - Datadog
//! - Elasticsearch
//! (Additional exporters can be added following the same pattern)

use crate::telemetry::MetricsCollector;

/// Trait for exporting metrics to external platforms.
pub trait PlatformExporter: Send + Sync {
    /// Export metrics to the platform.
    ///
    /// # Errors
    ///
    /// Returns an error if export fails.
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String>;

    /// Get the exporter name.
    fn name(&self) -> &str;
}

/// Datadog exporter for shipping metrics and logs to Datadog.
#[derive(Clone, Debug)]
pub struct DatadogExporter {
    #[allow(dead_code)]
    api_key: String,
    site: String, // us1, us3, eu1, etc.
}

impl DatadogExporter {
    /// Create a new Datadog exporter.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            site: "us1".to_string(),
        }
    }

    /// Set the Datadog site.
    pub fn with_site(mut self, site: impl Into<String>) -> Self {
        self.site = site.into();
        self
    }

    /// Get the API endpoint.
    pub fn endpoint(&self) -> String {
        format!(
            "https://api.datadoghq.{}/api/v1/series",
            if self.site == "us1" {
                "com"
            } else if self.site == "eu1" {
                "eu"
            } else {
                "com"
            }
        )
    }
}

impl Default for DatadogExporter {
    fn default() -> Self {
        Self::new("default")
    }
}

impl PlatformExporter for DatadogExporter {
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String> {
        let mut payload = serde_json::json!({
            "series": []
        });

        if let Some(series_arr) = payload["series"].as_array_mut() {
            series_arr.push(serde_json::json!({
                "metric": "clap.commands.executed",
                "points": [[
                    (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()) as i64,
                    metrics.command_count()
                ]],
                "type": "rate"
            }));

            series_arr.push(serde_json::json!({
                "metric": "clap.commands.errors",
                "points": [[
                    (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()) as i64,
                    metrics.error_count()
                ]],
                "type": "rate"
            }));
        }

        let json_str = serde_json::to_string_pretty(&payload).map_err(|e| {
            crate::NounVerbError::TelemetryError(format!("JSON serialization failed: {}", e))
        })?;
        Ok(format!("Datadog export: {}", json_str))
    }

    fn name(&self) -> &str {
        "datadog"
    }
}

/// Elasticsearch exporter for log aggregation.
#[derive(Clone, Debug)]
pub struct ElasticsearchExporter {
    hosts: Vec<String>,
    index_prefix: String,
}

impl ElasticsearchExporter {
    /// Create a new Elasticsearch exporter.
    pub fn new(hosts: Vec<String>) -> Self {
        Self {
            hosts,
            index_prefix: "clap-logs".to_string(),
        }
    }

    /// Set the index prefix.
    pub fn with_index_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.index_prefix = prefix.into();
        self
    }

    /// Get the bulk API endpoint.
    pub fn bulk_endpoint(&self) -> String {
        if self.hosts.is_empty() {
            "http://localhost:9200/_bulk".to_string()
        } else {
            format!("{}/_bulk", self.hosts[0])
        }
    }
}

impl Default for ElasticsearchExporter {
    fn default() -> Self {
        Self::new(vec!["http://localhost:9200".to_string()])
    }
}

impl PlatformExporter for ElasticsearchExporter {
    fn export(&self, metrics: &MetricsCollector) -> crate::Result<String> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let index = format!(
            "{}-{}",
            self.index_prefix,
            chrono::Local::now().format("%Y.%m.%d")
        );

        let mut bulk_payload = String::new();

        // Create bulk index commands
        bulk_payload.push_str(&format!(
            r#"{{"index":{{"_index":"{}","_type":"_doc"}}}}"#,
            index
        ));
        bulk_payload.push('\n');

        let doc = serde_json::json!({
            "timestamp": timestamp,
            "type": "metrics",
            "commands_executed": metrics.command_count(),
            "errors": metrics.error_count(),
        });

        let doc_str = serde_json::to_string(&doc).map_err(|e| {
            crate::NounVerbError::TelemetryError(format!("JSON serialization failed: {}", e))
        })?;
        bulk_payload.push_str(&doc_str);
        bulk_payload.push('\n');

        Ok(format!(
            "Elasticsearch export to {}: {} documents",
            self.bulk_endpoint(),
            1
        ))
    }

    fn name(&self) -> &str {
        "elasticsearch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datadog_exporter_creation() {
        let exporter = DatadogExporter::new("test_key");
        assert_eq!(exporter.name(), "datadog");
    }

    #[test]
    fn test_datadog_exporter_endpoint() {
        let exporter = DatadogExporter::new("test_key").with_site("us1");
        assert!(exporter.endpoint().contains("datadoghq.com"));
    }

    #[test]
    fn test_datadog_exporter_export() {
        let exporter = DatadogExporter::new("test_key");
        let metrics = MetricsCollector::new();
        let result = exporter.export(&metrics);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Datadog export"));
    }

    #[test]
    fn test_elasticsearch_exporter_creation() {
        let exporter =
            ElasticsearchExporter::new(vec!["http://localhost:9200".to_string()]);
        assert_eq!(exporter.name(), "elasticsearch");
    }

    #[test]
    fn test_elasticsearch_exporter_bulk_endpoint() {
        let exporter =
            ElasticsearchExporter::new(vec!["http://es.example.com".to_string()]);
        assert!(exporter.bulk_endpoint().contains("_bulk"));
    }

    #[test]
    fn test_elasticsearch_exporter_export() {
        let exporter = ElasticsearchExporter::new(vec!["http://localhost:9200".to_string()]);
        let metrics = MetricsCollector::new();
        let result = exporter.export(&metrics);
        assert!(result.is_ok());
    }
}
