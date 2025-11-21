//! Domain Logic: Telemetry and Observability
//!
//! Pure data structures for telemetry, metrics, and tracing.
//! NO actual telemetry collection - just data models.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Execution span for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation: String,
    pub start_time: u64,
    pub duration_ms: Option<u64>,
    pub status: SpanStatus,
    pub attributes: Vec<SpanAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAttribute {
    pub key: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl ExecutionSpan {
    pub fn new(operation: &str) -> Self {
        Self {
            trace_id: generate_id(),
            span_id: generate_id(),
            parent_span_id: None,
            operation: operation.to_string(),
            start_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis() as u64,
            duration_ms: None,
            status: SpanStatus::Unset,
            attributes: Vec::new(),
        }
    }

    pub fn with_attribute(mut self, key: &str, value: impl Into<AttributeValue>) -> Self {
        self.attributes.push(SpanAttribute {
            key: key.to_string(),
            value: value.into(),
        });
        self
    }

    pub fn complete(mut self, status: SpanStatus) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;
        self.duration_ms = Some(now.saturating_sub(self.start_time));
        self.status = status;
        self
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self { Self::String(s) }
}
impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self { Self::String(s.to_string()) }
}
impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self { Self::Int(i) }
}
impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self { Self::Float(f) }
}
impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self { Self::Bool(b) }
}

/// CLI metrics
#[allow(dead_code)] // FUTURE: Used for CLI health monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliMetrics {
    pub total_invocations: u64,
    pub successful_invocations: u64,
    pub failed_invocations: u64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub commands_by_noun: std::collections::HashMap<String, u64>,
}

impl CliMetrics {
    #[allow(dead_code)] // FUTURE: Used for CLI health monitoring
    pub fn success_rate(&self) -> f64 {
        if self.total_invocations == 0 {
            100.0
        } else {
            self.successful_invocations as f64 / self.total_invocations as f64 * 100.0
        }
    }
}

/// Execution receipt for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub receipt_id: String,
    pub trace_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub status: String,
    pub result_hash: Option<String>,
    pub agent_id: Option<String>,
}

impl ExecutionReceipt {
    pub fn new(command: &str, args: &[String], duration_ms: u64, success: bool) -> Self {
        Self {
            receipt_id: generate_id(),
            trace_id: generate_id(),
            command: command.to_string(),
            args: args.to_vec(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            duration_ms,
            status: if success { "success" } else { "error" }.to_string(),
            result_hash: None,
            agent_id: None,
        }
    }

    pub fn with_agent(mut self, agent_id: &str) -> Self {
        self.agent_id = Some(agent_id.to_string());
        self
    }
}

fn generate_id() -> String {
    use std::time::UNIX_EPOCH;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    format!("{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_span() {
        let span = ExecutionSpan::new("papers.generate")
            .with_attribute("family", "IMRaD")
            .complete(SpanStatus::Ok);
        assert_eq!(span.operation, "papers.generate");
        assert_eq!(span.status, SpanStatus::Ok);
        assert!(span.duration_ms.is_some());
    }

    #[test]
    fn test_cli_metrics_success_rate() {
        let mut metrics = CliMetrics::default();
        metrics.total_invocations = 100;
        metrics.successful_invocations = 95;
        assert!((metrics.success_rate() - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_execution_receipt() {
        let receipt = ExecutionReceipt::new("papers generate", &["IMRaD".to_string()], 42, true)
            .with_agent("agent-001");
        assert_eq!(receipt.command, "papers generate");
        assert_eq!(receipt.agent_id, Some("agent-001".to_string()));
    }
}
