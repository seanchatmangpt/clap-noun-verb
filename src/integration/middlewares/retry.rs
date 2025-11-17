//! Smart retry middleware with exponential backoff and jitter.

use crate::middleware::{Middleware, MiddlewareRequest};

/// Smart retry middleware with configurable backoff strategy.
///
/// Features:
/// - Exponential backoff: base_ms * (multiplier ^ attempt)
/// - Jitter to prevent thundering herd
/// - Maximum retry limit
/// - Configurable backoff parameters
#[derive(Clone, Debug)]
pub struct SmartRetryMiddleware {
    /// Maximum number of retries
    max_retries: usize,
    /// Base backoff in milliseconds
    base_backoff_ms: u64,
    /// Multiplier for exponential backoff
    multiplier: f64,
    /// Maximum backoff in milliseconds
    max_backoff_ms: u64,
    /// Enable jitter (random variation)
    use_jitter: bool,
}

impl SmartRetryMiddleware {
    /// Create a new retry middleware with defaults.
    ///
    /// Defaults:
    /// - Max retries: 3
    /// - Base backoff: 100ms
    /// - Multiplier: 2.0 (exponential)
    /// - Max backoff: 30000ms (30s)
    /// - Jitter: enabled
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            base_backoff_ms: 100,
            multiplier: 2.0,
            max_backoff_ms: 30000,
            use_jitter: true,
        }
    }

    /// Set the maximum number of retries.
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set the base backoff time in milliseconds.
    pub fn with_base_backoff(mut self, ms: u64) -> Self {
        self.base_backoff_ms = ms;
        self
    }

    /// Set the backoff multiplier for exponential growth.
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Set the maximum backoff time in milliseconds.
    pub fn with_max_backoff(mut self, ms: u64) -> Self {
        self.max_backoff_ms = ms;
        self
    }

    /// Enable or disable jitter (random variation in backoff).
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.use_jitter = jitter;
        self
    }

    /// Calculate backoff time for a given attempt number.
    ///
    /// Formula: min(base_ms * (multiplier ^ attempt), max_backoff_ms) + jitter
    pub fn calculate_backoff(&self, attempt: usize) -> u64 {
        let exponential = self.base_backoff_ms as f64 * self.multiplier.powi(attempt as i32);
        let backoff = (exponential as u64).min(self.max_backoff_ms);

        if self.use_jitter {
            // Add jitter: ±20% of backoff
            let jitter_range = (backoff as f64 * 0.2) as u64;
            let jitter = (std::process::id() as u64 % (jitter_range * 2)) - jitter_range;
            ((backoff as i64 + jitter as i64).max(0)) as u64
        } else {
            backoff
        }
    }

    /// Get the maximum number of retries.
    pub fn max_retries(&self) -> usize {
        self.max_retries
    }

    /// Get the base backoff time.
    pub fn base_backoff_ms(&self) -> u64 {
        self.base_backoff_ms
    }
}

impl Default for SmartRetryMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for SmartRetryMiddleware {
    fn name(&self) -> &str {
        "smart_retry"
    }

    fn before(&self, _request: &MiddlewareRequest) -> crate::Result<bool> {
        // Retry logic is handled at executor level
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_middleware_creation() {
        let mw = SmartRetryMiddleware::new();
        assert_eq!(mw.max_retries(), 3);
    }

    #[test]
    fn test_retry_middleware_with_options() {
        let mw = SmartRetryMiddleware::new()
            .with_max_retries(5)
            .with_base_backoff(50);
        assert_eq!(mw.max_retries(), 5);
        assert_eq!(mw.base_backoff_ms(), 50);
    }

    #[test]
    fn test_backoff_calculation() {
        let mw = SmartRetryMiddleware::new().with_jitter(false);
        let backoff_0 = mw.calculate_backoff(0);
        let backoff_1 = mw.calculate_backoff(1);
        let backoff_2 = mw.calculate_backoff(2);

        assert_eq!(backoff_0, 100); // 100 * 2^0
        assert_eq!(backoff_1, 200); // 100 * 2^1
        assert_eq!(backoff_2, 400); // 100 * 2^2
    }

    #[test]
    fn test_backoff_max_limit() {
        let mw = SmartRetryMiddleware::new()
            .with_max_backoff(1000)
            .with_jitter(false);

        let backoff_10 = mw.calculate_backoff(10);
        assert!(backoff_10 <= 1000);
    }
}
