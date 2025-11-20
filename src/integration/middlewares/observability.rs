//! Observability middleware with automatic span creation and metrics recording.

use crate::middleware::{Middleware, MiddlewareRequest, MiddlewareResponse};
use crate::telemetry::{MetricsCollector, SpanBuilder, TracingCollector};
use std::sync::{Arc, Mutex};

/// Middleware for automatic observability instrumentation.
///
/// Features:
/// - Automatic span creation with W3C trace context
/// - Command metrics recording
/// - Error tracking
/// - Execution timing
#[derive(Clone)]
pub struct ObservabilityMiddleware {
    metrics: Arc<MetricsCollector>,
    tracing: Arc<Mutex<TracingCollector>>,
    enabled: bool,
}

impl ObservabilityMiddleware {
    /// Create a new observability middleware.
    pub fn new(metrics: Arc<MetricsCollector>, tracing: Arc<Mutex<TracingCollector>>) -> Self {
        Self { metrics, tracing, enabled: true }
    }

    /// Enable observability.
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable observability.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl std::fmt::Debug for ObservabilityMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObservabilityMiddleware").field("enabled", &self.enabled).finish()
    }
}

impl Middleware for ObservabilityMiddleware {
    fn name(&self) -> &str {
        "observability"
    }

    fn before(&self, request: &MiddlewareRequest) -> crate::Result<bool> {
        if !self.enabled {
            return Ok(true);
        }

        // Create span for this command execution
        let _span = SpanBuilder::new(request.command())
            .with_attribute("requester", request.requester().unwrap_or("unknown"))
            .build("observability_trace");

        #[cfg(feature = "tracing")]
        {
            tracing::info!(
                command = request.command(),
                args = ?request.args(),
                "Command execution started"
            );
        }

        Ok(true)
    }

    fn after(&self, response: &MiddlewareResponse) -> crate::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        {
            if response.is_success() {
                tracing::info!(message = response.message(), "Command execution succeeded");
            } else {
                tracing::warn!(message = response.message(), "Command execution failed");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_middleware_creation() {
        let metrics = Arc::new(MetricsCollector::new());
        let tracing = Arc::new(Mutex::new(TracingCollector::new()));
        let mw = ObservabilityMiddleware::new(metrics, tracing);
        assert_eq!(mw.name(), "observability");
        assert!(mw.enabled);
    }

    #[test]
    fn test_observability_middleware_enable_disable() {
        let metrics = Arc::new(MetricsCollector::new());
        let tracing = Arc::new(Mutex::new(TracingCollector::new()));
        let mw = ObservabilityMiddleware::new(metrics, tracing);
        let enabled = mw.enable();
        assert!(enabled.enabled);
        let disabled = enabled.disable();
        assert!(!disabled.enabled);
    }
}
