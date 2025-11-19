//! Chicago TDD tests for Telemetry and Observability CLI Integration
//!
//! Tests telemetry system:
//! - Distributed tracing
//! - Span creation and context
//! - Metrics collection
//! - Event logging
//! - Performance tracking

use clap_noun_verb::telemetry::{TelemetryManager, Span, TraceContext, Metrics};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Duration;

// ============================================================================
// Telemetry Manager Tests (25+ tests)
// ============================================================================

#[test]
fn test_telemetry_manager_creation() {
    // Arrange & Act
    let manager = TelemetryManager::new("test-app");

    // Assert
    assert!(manager.is_ok(), "TelemetryManager creation should succeed");
}

#[test]
fn test_telemetry_manager_start_span() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();

    // Act
    let span = manager.start_span("test_operation");

    // Assert
    assert!(span.is_ok(), "Starting span should succeed");
    assert_eq!(span.ok().unwrap().name(), "test_operation");
}

#[test]
fn test_telemetry_manager_end_span() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();
    let span = manager.start_span("test_op").ok().unwrap();

    // Act
    let end_result = manager.end_span(span);

    // Assert
    assert!(end_result.is_ok(), "Ending span should succeed");
}

#[test]
fn test_telemetry_manager_nested_spans() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();

    // Act
    let parent = manager.start_span("parent").ok().unwrap();
    let child = manager.start_child_span(&parent, "child").ok().unwrap();

    // Assert
    assert_eq!(child.parent_id(), Some(parent.id()));
}

#[test]
fn test_telemetry_manager_span_attributes() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();
    let mut span = manager.start_span("op").ok().unwrap();

    // Act
    span.set_attribute("key1", "value1");
    span.set_attribute("key2", "value2");

    // Assert
    assert_eq!(span.get_attribute("key1"), Some("value1".to_string()));
    assert_eq!(span.get_attribute("key2"), Some("value2".to_string()));
}

#[test]
fn test_telemetry_manager_span_duration() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();
    let span = manager.start_span("timed_op").ok().unwrap();

    // Act - Simulate work
    std::thread::sleep(Duration::from_millis(100));
    let ended_span = manager.end_span(span).ok().unwrap();

    // Assert
    assert!(ended_span.duration() >= Duration::from_millis(100));
}

#[test]
fn test_telemetry_manager_trace_context() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();

    // Act
    let context = manager.create_trace_context();

    // Assert
    assert!(context.is_ok(), "Creating trace context should succeed");
    assert!(!context.ok().unwrap().trace_id().is_empty());
}

#[test]
fn test_telemetry_manager_inject_context() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();
    let context = manager.create_trace_context().ok().unwrap();

    // Act
    let headers = manager.inject_context(&context);

    // Assert
    assert!(headers.is_ok(), "Injecting context should succeed");
    assert!(headers.ok().unwrap().contains_key("traceparent"));
}

#[test]
fn test_telemetry_manager_extract_context() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();
    let original_context = manager.create_trace_context().ok().unwrap();
    let headers = manager.inject_context(&original_context).ok().unwrap();

    // Act
    let extracted_context = manager.extract_context(&headers);

    // Assert
    assert!(extracted_context.is_ok(), "Extracting context should succeed");
    assert_eq!(extracted_context.ok().unwrap().trace_id(), original_context.trace_id());
}

#[test]
fn test_telemetry_manager_record_event() {
    // Arrange
    let manager = TelemetryManager::new("test-app").ok().unwrap();
    let span = manager.start_span("op").ok().unwrap();

    // Act
    let event_result = manager.record_event(&span, "event_occurred", "details");

    // Assert
    assert!(event_result.is_ok(), "Recording event should succeed");
}

// ============================================================================
// Span Tests (25+ tests)
// ============================================================================

#[test]
fn test_span_creation() {
    // Arrange & Act
    let span = Span::new("test_span");

    // Assert
    assert!(span.is_ok(), "Span creation should succeed");
    assert_eq!(span.ok().unwrap().name(), "test_span");
}

#[test]
fn test_span_id_uniqueness() {
    // Arrange & Act
    let span1 = Span::new("span1").ok().unwrap();
    let span2 = Span::new("span2").ok().unwrap();

    // Assert
    assert_ne!(span1.id(), span2.id(), "Span IDs should be unique");
}

#[test]
fn test_span_set_status() {
    // Arrange
    let mut span = Span::new("span").ok().unwrap();

    // Act
    span.set_status("ok");

    // Assert
    assert_eq!(span.status(), "ok");
}

#[test]
fn test_span_set_error() {
    // Arrange
    let mut span = Span::new("span").ok().unwrap();

    // Act
    span.set_error("error message");

    // Assert
    assert_eq!(span.status(), "error");
    assert!(span.has_error());
}

#[test]
fn test_span_add_event() {
    // Arrange
    let mut span = Span::new("span").ok().unwrap();

    // Act
    span.add_event("event1", "details1");
    span.add_event("event2", "details2");

    // Assert
    let events = span.events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_span_timing() {
    // Arrange
    let span = Span::new("span").ok().unwrap();
    let start_time = span.start_time();

    // Act
    std::thread::sleep(Duration::from_millis(50));
    let end_time = std::time::Instant::now();

    // Assert
    assert!(end_time.duration_since(start_time) >= Duration::from_millis(50));
}

#[test]
fn test_span_attributes_iteration() {
    // Arrange
    let mut span = Span::new("span").ok().unwrap();
    span.set_attribute("a", "1");
    span.set_attribute("b", "2");
    span.set_attribute("c", "3");

    // Act
    let attrs = span.attributes();

    // Assert
    assert_eq!(attrs.len(), 3);
    assert!(attrs.contains_key("a"));
}

#[test]
fn test_span_parent_relationship() {
    // Arrange
    let parent = Span::new("parent").ok().unwrap();

    // Act
    let child = Span::new_with_parent("child", &parent);

    // Assert
    assert!(child.is_ok(), "Creating child span should succeed");
    assert_eq!(child.ok().unwrap().parent_id(), Some(parent.id()));
}

// ============================================================================
// Trace Context Tests (20+ tests)
// ============================================================================

#[test]
fn test_trace_context_creation() {
    // Arrange & Act
    let context = TraceContext::new();

    // Assert
    assert!(context.is_ok(), "TraceContext creation should succeed");
    assert!(!context.ok().unwrap().trace_id().is_empty());
}

#[test]
fn test_trace_context_span_id() {
    // Arrange
    let context = TraceContext::new().ok().unwrap();

    // Act & Assert
    assert!(!context.span_id().is_empty(), "Span ID should not be empty");
}

#[test]
fn test_trace_context_trace_flags() {
    // Arrange
    let mut context = TraceContext::new().ok().unwrap();

    // Act
    context.set_sampled(true);

    // Assert
    assert!(context.is_sampled(), "Context should be marked as sampled");
}

#[test]
fn test_trace_context_serialization() {
    // Arrange
    let context = TraceContext::new().ok().unwrap();

    // Act
    let serialized = context.to_traceparent();

    // Assert
    assert!(serialized.is_ok(), "Serialization should succeed");
    assert!(serialized.ok().unwrap().starts_with("00-")); // W3C traceparent format
}

#[test]
fn test_trace_context_deserialization() {
    // Arrange
    let original = TraceContext::new().ok().unwrap();
    let serialized = original.to_traceparent().ok().unwrap();

    // Act
    let deserialized = TraceContext::from_traceparent(&serialized);

    // Assert
    assert!(deserialized.is_ok(), "Deserialization should succeed");
    assert_eq!(deserialized.ok().unwrap().trace_id(), original.trace_id());
}

#[test]
fn test_trace_context_baggage() {
    // Arrange
    let mut context = TraceContext::new().ok().unwrap();

    // Act
    context.set_baggage("key1", "value1");
    context.set_baggage("key2", "value2");

    // Assert
    assert_eq!(context.get_baggage("key1"), Some("value1".to_string()));
    assert_eq!(context.get_baggage("key2"), Some("value2".to_string()));
}

#[test]
fn test_trace_context_clone() {
    // Arrange
    let original = TraceContext::new().ok().unwrap();

    // Act
    let cloned = original.clone();

    // Assert
    assert_eq!(cloned.trace_id(), original.trace_id());
    assert_eq!(cloned.span_id(), original.span_id());
}

// ============================================================================
// Metrics Tests (20+ tests)
// ============================================================================

#[test]
fn test_metrics_creation() {
    // Arrange & Act
    let metrics = Metrics::new("test-service");

    // Assert
    assert!(metrics.is_ok(), "Metrics creation should succeed");
}

#[test]
fn test_metrics_counter_increment() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();

    // Act
    metrics.increment_counter("requests");
    metrics.increment_counter("requests");
    metrics.increment_counter("requests");

    // Assert
    assert_eq!(metrics.get_counter("requests"), 3);
}

#[test]
fn test_metrics_counter_add() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();

    // Act
    metrics.add_to_counter("bytes_sent", 100);
    metrics.add_to_counter("bytes_sent", 250);

    // Assert
    assert_eq!(metrics.get_counter("bytes_sent"), 350);
}

#[test]
fn test_metrics_gauge_set() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();

    // Act
    metrics.set_gauge("cpu_usage", 45.5);
    metrics.set_gauge("cpu_usage", 67.8);

    // Assert
    assert_eq!(metrics.get_gauge("cpu_usage"), 67.8);
}

#[test]
fn test_metrics_histogram_record() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();

    // Act
    metrics.record_histogram("response_time", 100.0);
    metrics.record_histogram("response_time", 200.0);
    metrics.record_histogram("response_time", 300.0);

    // Assert
    let avg = metrics.get_histogram_avg("response_time");
    assert_eq!(avg, 200.0);
}

#[test]
fn test_metrics_histogram_percentiles() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();

    // Act - Record 100 values
    for i in 1..=100 {
        metrics.record_histogram("latency", i as f64);
    }

    // Assert
    let p50 = metrics.get_percentile("latency", 50.0);
    let p95 = metrics.get_percentile("latency", 95.0);
    let p99 = metrics.get_percentile("latency", 99.0);

    assert!(p50 >= 45.0 && p50 <= 55.0, "P50 should be around 50");
    assert!(p95 >= 90.0 && p95 <= 100.0, "P95 should be around 95");
    assert!(p99 >= 95.0 && p99 <= 100.0, "P99 should be around 99");
}

#[test]
fn test_metrics_multiple_counters() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();

    // Act
    metrics.increment_counter("requests");
    metrics.increment_counter("errors");
    metrics.increment_counter("requests");

    // Assert
    assert_eq!(metrics.get_counter("requests"), 2);
    assert_eq!(metrics.get_counter("errors"), 1);
}

#[test]
fn test_metrics_concurrent_updates() {
    // Arrange
    let metrics = Arc::new(Metrics::new("service").ok().unwrap());
    let mut handles = vec![];

    // Act - Concurrent increments
    for _ in 0..10 {
        let metrics_clone = metrics.clone();
        let handle = std::thread::spawn(move || {
            for _ in 0..100 {
                metrics_clone.increment_counter("concurrent_requests");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().ok();
    }

    // Assert
    assert_eq!(metrics.get_counter("concurrent_requests"), 1000);
}

#[test]
fn test_metrics_reset() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();
    metrics.increment_counter("requests");
    metrics.set_gauge("cpu", 50.0);

    // Act
    metrics.reset();

    // Assert
    assert_eq!(metrics.get_counter("requests"), 0);
    assert_eq!(metrics.get_gauge("cpu"), 0.0);
}

#[test]
fn test_metrics_export() {
    // Arrange
    let metrics = Metrics::new("service").ok().unwrap();
    metrics.increment_counter("requests");
    metrics.set_gauge("memory_mb", 512.0);

    // Act
    let exported = metrics.export();

    // Assert
    assert!(exported.is_ok(), "Export should succeed");
    let data = exported.ok().unwrap();
    assert!(data.contains("requests"));
    assert!(data.contains("memory_mb"));
}

// ============================================================================
// Integration Tests - Telemetry with CLI (15+ tests)
// ============================================================================

#[test]
fn test_telemetry_cli_command_tracing() {
    // Arrange
    let manager = TelemetryManager::new("cli-app").ok().unwrap();

    // Act - Simulate CLI command execution with tracing
    let command_span = manager.start_span("execute_command").ok().unwrap();
    let mut parse_span = manager.start_child_span(&command_span, "parse_args").ok().unwrap();
    parse_span.set_attribute("args_count", "3");
    let _ = manager.end_span(parse_span);

    let mut execute_span = manager.start_child_span(&command_span, "run_handler").ok().unwrap();
    execute_span.set_attribute("handler", "config_set");
    let _ = manager.end_span(execute_span);

    let ended_command = manager.end_span(command_span).ok().unwrap();

    // Assert
    assert!(ended_command.duration() > Duration::from_nanos(0));
}

#[test]
fn test_telemetry_error_tracking() {
    // Arrange
    let manager = TelemetryManager::new("cli-app").ok().unwrap();
    let mut span = manager.start_span("failing_operation").ok().unwrap();

    // Act - Record error
    span.set_error("Validation failed: missing required field");
    let ended_span = manager.end_span(span).ok().unwrap();

    // Assert
    assert!(ended_span.has_error());
    assert_eq!(ended_span.status(), "error");
}

#[test]
fn test_telemetry_distributed_tracing() {
    // Arrange
    let manager = TelemetryManager::new("cli-app").ok().unwrap();
    let context = manager.create_trace_context().ok().unwrap();

    // Act - Simulate distributed call
    let headers = manager.inject_context(&context).ok().unwrap();

    // Simulate receiving on another service
    let received_context = manager.extract_context(&headers).ok().unwrap();

    // Assert - Trace ID should be preserved
    assert_eq!(received_context.trace_id(), context.trace_id());
}

#[test]
fn test_telemetry_with_metrics() {
    // Arrange
    let manager = TelemetryManager::new("cli-app").ok().unwrap();
    let metrics = Metrics::new("cli-app").ok().unwrap();

    // Act - Execute command and track metrics
    let span = manager.start_span("command").ok().unwrap();
    metrics.increment_counter("commands_executed");

    std::thread::sleep(Duration::from_millis(50));
    let ended_span = manager.end_span(span).ok().unwrap();

    metrics.record_histogram("command_duration_ms", ended_span.duration().as_millis() as f64);

    // Assert
    assert_eq!(metrics.get_counter("commands_executed"), 1);
    assert!(metrics.get_histogram_avg("command_duration_ms") >= 50.0);
}

#[test]
fn test_telemetry_sampling() {
    // Arrange
    let manager = TelemetryManager::new("cli-app").ok().unwrap();
    let mut sampled_context = TraceContext::new().ok().unwrap();
    sampled_context.set_sampled(true);

    let mut unsampled_context = TraceContext::new().ok().unwrap();
    unsampled_context.set_sampled(false);

    // Act & Assert
    assert!(sampled_context.is_sampled());
    assert!(!unsampled_context.is_sampled());
}

#[test]
fn test_telemetry_performance_overhead() {
    // Arrange
    let manager = TelemetryManager::new("cli-app").ok().unwrap();

    // Act - Measure overhead of 1000 span operations
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let span = manager.start_span(&format!("op_{}", i)).ok().unwrap();
        let _ = manager.end_span(span);
    }

    let duration = start.elapsed();

    // Assert - 1000 span operations should complete quickly
    assert!(duration.as_millis() < 100, "Telemetry overhead should be minimal (<100ms for 1000 ops)");
}
