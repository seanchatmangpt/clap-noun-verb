//! Distributed tracing support for observability.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A span represents a unit of work in a distributed trace.
#[derive(Debug, Clone)]
pub struct Span {
    /// Span name/operation name
    name: String,
    /// Unique span ID
    id: String,
    /// Parent span ID (if any)
    parent_id: Option<String>,
    /// Trace ID for the entire trace
    trace_id: String,
    /// When the span started (ms since epoch)
    start_time: u64,
    /// When the span ended (ms since epoch)
    end_time: Option<u64>,
    /// Span status
    status: SpanStatus,
    /// Key-value attributes
    attributes: HashMap<String, String>,
    /// Events recorded during the span
    events: Vec<SpanEvent>,
}

/// Status of a span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    /// Span is running
    Unset,
    /// Span completed successfully
    Ok,
    /// Span encountered an error
    Error,
}

/// Event recorded during a span.
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Event name
    name: String,
    /// Event timestamp (ms since epoch)
    timestamp: u64,
    /// Event attributes
    attributes: HashMap<String, String>,
}

impl Span {
    /// Create a new span.
    pub fn new(name: impl Into<String>, trace_id: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            name: name.into(),
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            trace_id: trace_id.into(),
            start_time: now,
            end_time: None,
            status: SpanStatus::Unset,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Set the parent span ID.
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Add an attribute to the span.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Mark the span as successfully completed.
    pub fn end_ok(&mut self) {
        self.status = SpanStatus::Ok;
        self.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
    }

    /// Mark the span as having encountered an error.
    pub fn end_error(&mut self, error: impl Into<String>) {
        self.status = SpanStatus::Error;
        self.attributes
            .insert("error.message".to_string(), error.into());
        self.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
    }

    /// Record an event in this span.
    pub fn add_event(&mut self, name: impl Into<String>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: now,
            attributes: HashMap::new(),
        });
    }

    /// Get the span name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the span ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the trace ID.
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get the parent span ID.
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    /// Get the span status.
    pub fn status(&self) -> SpanStatus {
        self.status
    }

    /// Get the duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Get the attributes.
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// Get the events.
    pub fn events(&self) -> &[SpanEvent] {
        &self.events
    }
}

/// Builder for creating spans.
#[derive(Debug)]
pub struct SpanBuilder {
    name: String,
    parent_id: Option<String>,
    attributes: HashMap<String, String>,
}

impl SpanBuilder {
    /// Create a new span builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_id: None,
            attributes: HashMap::new(),
        }
    }

    /// Set the parent span ID.
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Build the span.
    pub fn build(self, trace_id: impl Into<String>) -> Span {
        let mut span = Span::new(self.name, trace_id);
        if let Some(parent_id) = self.parent_id {
            span.parent_id = Some(parent_id);
        }
        span.attributes = self.attributes;
        span
    }
}

/// Tracing collector for managing spans and traces.
#[derive(Debug)]
pub struct TracingCollector {
    /// All collected spans
    spans: Vec<Span>,
    /// Current trace ID
    current_trace_id: String,
}

impl TracingCollector {
    /// Create a new tracing collector.
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            current_trace_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create a new trace.
    pub fn new_trace(&mut self) {
        self.current_trace_id = uuid::Uuid::new_v4().to_string();
        self.spans.clear();
    }

    /// Get the current trace ID.
    pub fn current_trace_id(&self) -> &str {
        &self.current_trace_id
    }

    /// Add a span to the collector.
    pub fn add_span(&mut self, span: Span) {
        self.spans.push(span);
    }

    /// Get all spans.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Get the number of spans collected.
    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// Clear all spans.
    pub fn clear(&mut self) {
        self.spans.clear();
    }

    /// Get spans by status.
    pub fn spans_by_status(&self, status: SpanStatus) -> Vec<&Span> {
        self.spans.iter().filter(|s| s.status() == status).collect()
    }

    /// Get the total duration of all spans.
    pub fn total_duration_ms(&self) -> u64 {
        self.spans
            .iter()
            .filter_map(|s| s.duration_ms())
            .sum()
    }
}

impl Default for TracingCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new("test_op", "trace123");
        assert_eq!(span.name(), "test_op");
        assert_eq!(span.trace_id(), "trace123");
        assert_eq!(span.status(), SpanStatus::Unset);
    }

    #[test]
    fn test_span_with_parent() {
        let span = Span::new("test_op", "trace123").with_parent("parent_id");
        assert_eq!(span.parent_id(), Some("parent_id"));
    }

    #[test]
    fn test_span_end_ok() {
        let mut span = Span::new("test_op", "trace123");
        span.end_ok();
        assert_eq!(span.status(), SpanStatus::Ok);
        assert!(span.duration_ms().is_some());
    }

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("operation")
            .with_parent("parent")
            .with_attribute("key", "value")
            .build("trace123");

        assert_eq!(span.name(), "operation");
        assert_eq!(span.parent_id(), Some("parent"));
        assert_eq!(span.attributes().get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_tracing_collector() {
        let mut collector = TracingCollector::new();
        let span = Span::new("test", collector.current_trace_id());
        collector.add_span(span);
        assert_eq!(collector.span_count(), 1);
    }
}
