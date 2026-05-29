# Reference: Distributed Tracing and Telemetry

**File**: `src/telemetry.rs`

The `clap-noun-verb` telemetry module provides distributed tracing, metrics aggregation, W3C traceparent propagation, and autonomic telemetry envelope formatting for machine-to-machine integration.

---

## TraceContext

`TraceContext` represents a distributed trace context following the W3C Trace Context specification. It enables tracing requests across distributed services, agents, and CLI invocations.

### Signature
```rust
#[derive(Debug, Clone)]
pub struct TraceContext {
    trace_id: String,
    span_id: String,
    sampled: bool,
    baggage: std::collections::HashMap<String, String>,
}
```

### Methods
* `pub fn new() -> Result<Self, NounVerbError>`: Generates a new `TraceContext` with a randomized 128-bit `trace_id` (32-character hex string) and a 64-bit `span_id` (16-character hex string).
* `pub fn trace_id(&self) -> &str`: Accesses the current trace ID.
* `pub fn span_id(&self) -> &str`: Accesses the current span ID.
* `pub fn is_sampled(&self) -> bool`: Checks if the trace context is flagged as sampled.
* `pub fn set_sampled(&mut self, sampled: bool)`: Sets the sampled flag.
* `pub fn set_baggage(&mut self, key: &str, value: &str)`: Stores a key-value pair in the baggage map, enabling context propagation of metadata across boundaries.
* `pub fn get_baggage(&self, key: &str) -> Option<String>`: Retrieves baggage metadata by key.

---

## W3C traceparent Propagation

The W3C `traceparent` header coordinates distributed tracing by defining a standard header format:
`version-trace_id-parent_id-trace_flags`

* **Version**: Currently supported version is `00`.
* **Trace ID**: A 16-byte (32-hex-character) unique identifier.
* **Parent ID (Span ID)**: An 8-byte (16-hex-character) unique identifier.
* **Trace Flags**: An 8-bit field (2-hex-characters), where `01` means sampled and `00` means not sampled.

### Serialization & Deserialization

* `pub fn to_traceparent(&self) -> Result<String, NounVerbError>`: Serializes the `TraceContext` into a valid W3C traceparent header value.
  ```rust
  let context = TraceContext::new()?;
  let header_value = context.to_traceparent()?;
  // Example format: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
  ```
* `pub fn from_traceparent(traceparent: &str) -> Result<Self, NounVerbError>`: Parses a W3C traceparent header string, validating format constraints, hex digits, and lengths.

### CLI and Environment Extraction

To facilitate zero-configuration distributed tracing, `TraceContext` can automatically extract its values from the environment or invocation arguments:

* `pub fn extract_from_env_or_flags() -> Option<Self>`: Resolves the trace context by checking, in priority order:
  1. The `TRACEPARENT` environment variable.
  2. The `traceparent` environment variable.
  3. The `--traceparent <value>` command line flag.
  4. The `--traceparent=<value>` command line flag.

---

## TelemetryManager

Coordinates the trace lifecycle, managing spans, events, and injection/extraction of trace contexts across boundaries.

### Methods

* `pub fn new(app_name: &str) -> Result<Self, NounVerbError>`: Creates a new manager instance.
* `pub fn start_span(&self, name: &str) -> Result<Span, NounVerbError>`: Starts a new root span.
* `pub fn start_child_span(&self, parent: &Span, name: &str) -> Result<Span, NounVerbError>`: Starts a child span referencing a parent's trace and span IDs.
* `pub fn end_span(&self, mut span: Span) -> Result<Span, NounVerbError>`: Closes the span and records its duration.
* `pub fn create_trace_context(&self) -> Result<TraceContext, NounVerbError>`: Instantiates a new context representation.
* `pub fn inject_context(&self, context: &TraceContext) -> Result<HashMap<String, String>, NounVerbError>`: Injects the trace context into a header map. Baggage values are added with the prefix `otbaggage-`.
* `pub fn extract_context(&self, headers: &HashMap<String, String>) -> Result<TraceContext, NounVerbError>`: Extracts the context from a header map, restoring baggage keys.
* `pub fn record_event(&self, span: &Span, name: &str, details: &str) -> Result<(), NounVerbError>`: Logs an event under the trace context using the structured telemetry envelope.

---

## AutonomicTelemetryEnvelope

A standard machine-readable wrapper format for tracing messages:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutonomicTelemetryEnvelope<T> {
    pub schema_version: String,
    pub cli_version: String,
    pub timestamp: String,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub payload: T,
}
```

---

## Integration Example

```rust
use clap_noun_verb::telemetry::{TelemetryManager, TraceContext};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TelemetryManager::new("my-service")?;

    // Extract context from CLI inputs or incoming headers
    let context = TraceContext::extract_from_env_or_flags()
        .unwrap_or_else(|| manager.create_trace_context().unwrap());

    // Inject to forward tracking to downstream HTTP/RPC calls
    let headers = manager.inject_context(&context)?;
    assert!(headers.contains_key("traceparent"));

    Ok(())
}
```
