//! Distributed Tracing Integration for CNV 4.1.0
//!
//! OpenTelemetry-compatible tracing for command execution across distributed systems.
//! Uses advanced patterns: context propagation, baggage, and sampling strategies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trace context propagated across service boundaries
/// Compatible with W3C Trace Context and Jaeger formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// W3C Trace ID (128-bit)
    pub trace_id: String,
    /// Span ID (64-bit)
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Trace flags (sampled, etc.)
    pub trace_flags: TraceFlags,
    /// Baggage items (key-value context)
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            trace_flags: TraceFlags::default(),
            baggage: HashMap::new(),
        }
    }

    /// Create child span from this context
    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            trace_flags: self.trace_flags,
            baggage: self.baggage.clone(),
        }
    }

    /// Inject into W3C Trace Context header format
    pub fn to_trace_context_header(&self) -> String {
        format!(
            "00-{}-{}-{}",
            self.trace_id,
            self.span_id,
            format!("{:02x}", self.trace_flags.to_byte())
        )
    }

    /// Parse from W3C Trace Context header format
    pub fn from_trace_context_header(header: &str) -> Result<Self, String> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 || parts[0] != "00" {
            return Err("Invalid W3C Trace Context format".to_string());
        }

        Ok(Self {
            trace_id: parts[1].to_string(),
            span_id: parts[2].to_string(),
            parent_span_id: None,
            trace_flags: TraceFlags::from_byte(
                u8::from_str_radix(parts[3], 16).map_err(|_| "Invalid flags".to_string())?,
            ),
            baggage: HashMap::new(),
        })
    }

    pub fn add_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace flags (bit flags for sampling, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TraceFlags {
    pub sampled: bool,
    pub debug: bool,
    pub delayed: bool,
}

impl TraceFlags {
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.sampled {
            byte |= 0x01;
        }
        if self.debug {
            byte |= 0x02;
        }
        if self.delayed {
            byte |= 0x04;
        }
        byte
    }

    pub fn from_byte(byte: u8) -> Self {
        Self {
            sampled: (byte & 0x01) != 0,
            debug: (byte & 0x02) != 0,
            delayed: (byte & 0x04) != 0,
        }
    }
}

impl Default for TraceFlags {
    fn default() -> Self {
        Self {
            sampled: true,
            debug: false,
            delayed: false,
        }
    }
}

/// Span representing a unit of work within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub start_time_ns: u64,
    pub end_time_ns: Option<u64>,
    pub attributes: HashMap<String, SpanAttribute>,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

impl Span {
    pub fn new(name: String, context: &TraceContext) -> Self {
        Self {
            name,
            trace_id: context.trace_id.clone(),
            span_id: context.span_id.clone(),
            parent_span_id: context.parent_span_id.clone(),
            start_time_ns: current_time_ns(),
            end_time_ns: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Unset,
        }
    }

    pub fn add_attribute(&mut self, key: String, value: SpanAttribute) {
        self.attributes.insert(key, value);
    }

    pub fn add_event(&mut self, event: SpanEvent) {
        self.events.push(event);
    }

    pub fn set_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    pub fn set_error(&mut self, message: String) {
        self.status = SpanStatus::Error(message);
    }

    pub fn end(&mut self) {
        self.end_time_ns = Some(current_time_ns());
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.end_time_ns
            .map(|end| (end - self.start_time_ns) / 1_000_000)
    }
}

/// Attribute value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SpanAttribute {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "number")]
    Number(f64),
    #[serde(rename = "boolean")]
    Boolean(bool),
    #[serde(rename = "bytes")]
    Bytes(Vec<u8>),
}

/// Span event (timing event within a span)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp_ns: u64,
    pub attributes: HashMap<String, SpanAttribute>,
}

/// Span status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SpanStatus {
    #[serde(rename = "unset")]
    Unset,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error(String),
}

/// Sampling strategy for deciding which traces to collect
pub trait SamplingStrategy: Send + Sync {
    fn should_sample(&self, context: &TraceContext) -> bool;
}

/// Always-on sampler (for testing/debugging)
#[derive(Debug)]
pub struct AlwaysSampler;

impl SamplingStrategy for AlwaysSampler {
    fn should_sample(&self, _context: &TraceContext) -> bool {
        true
    }
}

/// Probabilistic sampler
#[derive(Debug)]
pub struct ProbabilisticSampler {
    pub probability: f64,
}

impl SamplingStrategy for ProbabilisticSampler {
    fn should_sample(&self, context: &TraceContext) -> bool {
        // Use trace ID to make decision deterministic
        let hash = context.trace_id.bytes().map(|b| b as u64).sum::<u64>();
        let normalized = (hash as f64 % 1.0) / 1.0;
        normalized < self.probability
    }
}

/// Tracing collector - collects and exports spans
pub trait SpanExporter: Send + Sync {
    fn export(&self, spans: Vec<Span>) -> Result<(), String>;
    fn shutdown(&self) -> Result<(), String>;
}

/// In-memory span exporter (for testing)
#[derive(Debug)]
pub struct InMemoryExporter {
    spans: Arc<std::sync::Mutex<Vec<Span>>>,
}

impl InMemoryExporter {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_spans(&self) -> Vec<Span> {
        self.spans.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.spans.lock().unwrap().clear();
    }
}

impl Default for InMemoryExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanExporter for InMemoryExporter {
    fn export(&self, spans: Vec<Span>) -> Result<(), String> {
        let mut all_spans = self.spans.lock().unwrap();
        all_spans.extend(spans);
        Ok(())
    }

    fn shutdown(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Tracing provider - manages context and span collection
pub struct TracingProvider {
    sampler: Arc<dyn SamplingStrategy>,
    exporter: Arc<dyn SpanExporter>,
    active_spans: Arc<std::sync::Mutex<Vec<Span>>>,
}

impl TracingProvider {
    pub fn new(
        sampler: Arc<dyn SamplingStrategy>,
        exporter: Arc<dyn SpanExporter>,
    ) -> Self {
        Self {
            sampler,
            exporter,
            active_spans: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn should_sample(&self, context: &TraceContext) -> bool {
        self.sampler.should_sample(context)
    }

    pub fn start_span(&self, span: Span) {
        if self.should_sample(&TraceContext {
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            parent_span_id: span.parent_span_id.clone(),
            trace_flags: TraceFlags::default(),
            baggage: HashMap::new(),
        }) {
            let mut spans = self.active_spans.lock().unwrap();
            spans.push(span);
        }
    }

    pub fn end_span(&self, span_id: &str) {
        let mut spans = self.active_spans.lock().unwrap();
        if let Some(pos) = spans.iter().position(|s| s.span_id == span_id) {
            let mut span = spans.remove(pos);
            span.end();
            // In real implementation, would export here
            let _ = self.exporter.export(vec![span]);
        }
    }

    pub fn flush(&self) -> Result<(), String> {
        let spans = self.active_spans.lock().unwrap().clone();
        if !spans.is_empty() {
            self.exporter.export(spans)?;
        }
        self.exporter.shutdown()
    }
}

// Helper functions
fn generate_trace_id() -> String {
    format!("{:032x}", rand_u128())
}

fn generate_span_id() -> String {
    format!("{:016x}", rand_u64())
}

fn rand_u64() -> u64 {
    // Simple PRNG (production code would use proper RNG)
    let nanos = current_time_ns();
    nanos.wrapping_mul(2654435761).wrapping_add(2246822519)
}

fn rand_u128() -> u128 {
    ((rand_u64() as u128) << 64) | (rand_u64() as u128)
}

fn current_time_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_creation() {
        let ctx = TraceContext::new();
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
    }

    #[test]
    fn test_child_span() {
        let parent = TraceContext::new();
        let child = parent.child_span();

        assert_eq!(parent.trace_id, child.trace_id);
        assert_ne!(parent.span_id, child.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_w3c_trace_context_header() {
        let ctx = TraceContext::new();
        let header = ctx.to_trace_context_header();

        assert!(header.starts_with("00-"));
        let parts: Vec<&str> = header.split('-').collect();
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn test_trace_flags() {
        let flags = TraceFlags {
            sampled: true,
            debug: false,
            delayed: false,
        };

        let byte = flags.to_byte();
        let restored = TraceFlags::from_byte(byte);

        assert!(restored.sampled);
        assert!(!restored.debug);
    }

    #[test]
    fn test_span_creation() {
        let ctx = TraceContext::new();
        let span = Span::new("test-op".to_string(), &ctx);

        assert_eq!(span.name, "test-op");
        assert_eq!(span.trace_id, ctx.trace_id);
        assert!(span.end_time_ns.is_none());
    }

    #[test]
    fn test_span_attributes() {
        let ctx = TraceContext::new();
        let mut span = Span::new("test".to_string(), &ctx);

        span.add_attribute("user".to_string(), SpanAttribute::String("alice".to_string()));
        span.add_attribute("count".to_string(), SpanAttribute::Number(42.0));

        assert_eq!(span.attributes.len(), 2);
    }

    #[test]
    fn test_always_sampler() {
        let sampler = AlwaysSampler;
        let ctx = TraceContext::new();

        assert!(sampler.should_sample(&ctx));
    }

    #[test]
    fn test_in_memory_exporter() {
        let exporter = InMemoryExporter::new();
        let ctx = TraceContext::new();
        let span = Span::new("test".to_string(), &ctx);

        exporter.export(vec![span]).unwrap();
        assert_eq!(exporter.get_spans().len(), 1);
    }
}
