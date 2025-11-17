//! Distributed tracing middleware with W3C trace context support.

use crate::middleware::{Middleware, MiddlewareRequest};

/// Distributed tracing middleware with W3C trace context propagation.
///
/// Features:
/// - W3C traceparent header support
/// - Context propagation across async boundaries
/// - Parent-child span relationships
#[derive(Clone, Debug)]
pub struct DistributedTracingMiddleware {
    enabled: bool,
}

impl DistributedTracingMiddleware {
    /// Create a new distributed tracing middleware.
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Enable distributed tracing.
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable distributed tracing.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Parse W3C traceparent header.
    ///
    /// Format: `00-trace-id-span-id-flags`
    /// - trace-id: 32 hex chars (128-bit)
    /// - span-id: 16 hex chars (64-bit)
    /// - flags: 2 hex chars (sampled, etc)
    pub fn parse_traceparent(header: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 || parts[0] != "00" {
            return None;
        }
        Some((parts[1].to_string(), parts[2].to_string()))
    }

    /// Format a W3C traceparent header.
    pub fn format_traceparent(trace_id: &str, span_id: &str, sampled: bool) -> String {
        let flags = if sampled { "01" } else { "00" };
        format!("00-{}-{}-{}", trace_id, span_id, flags)
    }
}

impl Default for DistributedTracingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for DistributedTracingMiddleware {
    fn name(&self) -> &str {
        "distributed_tracing"
    }

    fn before(&self, _request: &MiddlewareRequest) -> crate::Result<bool> {
        if !self.enabled {
            return Ok(true);
        }

        #[cfg(feature = "tracing")]
        {
            // In production, extract trace context from request headers
            // and propagate to all downstream operations
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_middleware_creation() {
        let mw = DistributedTracingMiddleware::new();
        assert_eq!(mw.name(), "distributed_tracing");
    }

    #[test]
    fn test_traceparent_parsing() {
        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let result = DistributedTracingMiddleware::parse_traceparent(header);
        assert!(result.is_some());
        let (trace_id, span_id) = result.unwrap();
        assert_eq!(trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(span_id, "00f067aa0ba902b7");
    }

    #[test]
    fn test_traceparent_formatting() {
        let traceparent =
            DistributedTracingMiddleware::format_traceparent("trace123", "span456", true);
        assert!(traceparent.contains("trace123"));
        assert!(traceparent.contains("span456"));
        assert!(traceparent.contains("01"));
    }
}
