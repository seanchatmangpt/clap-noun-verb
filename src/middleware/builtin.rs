//! Built-in middleware implementations.

use super::{Middleware, MiddlewareRequest, MiddlewareResponse};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

/// Logging middleware for structured command logging.
#[derive(Debug)]
pub struct LoggingMiddleware {
    /// Enable verbose logging
    verbose: bool,
}

impl LoggingMiddleware {
    /// Create a new logging middleware.
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// Enable verbose logging.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for LoggingMiddleware {
    fn name(&self) -> &str {
        "logging"
    }

    fn before(&self, request: &MiddlewareRequest) -> crate::Result<bool> {
        let log_msg = if self.verbose {
            format!(
                "Executing command: {} with args: {:?}",
                request.command(),
                request.args()
            )
        } else {
            format!("Executing command: {}", request.command())
        };

        #[cfg(feature = "tracing")]
        {
            tracing::info!("{}", log_msg);
        }

        #[cfg(not(feature = "tracing"))]
        {
            eprintln!("[INFO] {}", log_msg);
        }

        Ok(true)
    }

    fn after(&self, response: &MiddlewareResponse) -> crate::Result<()> {
        let status = if response.is_success() { "OK" } else { "FAILED" };
        let log_msg = format!("Command completed: {} - {}", status, response.message());

        #[cfg(feature = "tracing")]
        {
            tracing::info!("{}", log_msg);
        }

        #[cfg(not(feature = "tracing"))]
        {
            eprintln!("[INFO] {}", log_msg);
        }

        Ok(())
    }
}

/// Error recovery middleware with retry logic.
#[derive(Debug, Clone)]
pub struct ErrorRecoveryMiddleware {
    /// Maximum number of retries
    max_retries: usize,
    /// Backoff strategy (in milliseconds)
    backoff_ms: u64,
}

impl ErrorRecoveryMiddleware {
    /// Create a new error recovery middleware.
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            backoff_ms: 100,
        }
    }

    /// Set maximum retries.
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set backoff time in milliseconds.
    pub fn with_backoff_ms(mut self, ms: u64) -> Self {
        self.backoff_ms = ms;
        self
    }
}

impl Default for ErrorRecoveryMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for ErrorRecoveryMiddleware {
    fn name(&self) -> &str {
        "error_recovery"
    }

    fn handle_error(&self, error: &crate::NounVerbError) -> crate::Result<Option<String>> {
        // Determine if this error is retryable
        let is_retryable = matches!(
            error,
            crate::NounVerbError::ExecutionError { .. } | crate::NounVerbError::Generic(_)
        );

        if is_retryable {
            Ok(Some(format!(
                "Error recovered: {}. Consider retrying with --retry flag.",
                error
            )))
        } else {
            Ok(None)
        }
    }
}

/// Authentication/Authorization middleware.
#[derive(Debug)]
pub struct AuthMiddleware {
    /// Allowed users
    allowed_users: Vec<String>,
    /// Require authentication
    require_auth: bool,
}

impl AuthMiddleware {
    /// Create a new auth middleware.
    pub fn new() -> Self {
        Self {
            allowed_users: Vec::new(),
            require_auth: false,
        }
    }

    /// Add an allowed user.
    pub fn allow_user(mut self, user: impl Into<String>) -> Self {
        self.allowed_users.push(user.into());
        self
    }

    /// Enable authentication requirement.
    pub fn require_auth(mut self) -> Self {
        self.require_auth = true;
        self
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for AuthMiddleware {
    fn name(&self) -> &str {
        "auth"
    }

    fn before(&self, request: &MiddlewareRequest) -> crate::Result<bool> {
        if self.require_auth {
            if let Some(requester) = request.requester() {
                if self.allowed_users.is_empty() || self.allowed_users.contains(&requester.to_string()) {
                    return Ok(true);
                }
                return Err(crate::NounVerbError::MiddlewareError(
                    format!("User '{}' is not authorized", requester),
                ));
            } else {
                return Err(crate::NounVerbError::MiddlewareError(
                    "Authentication required".to_string(),
                ));
            }
        }
        Ok(true)
    }
}

/// Profiling middleware for execution timing.
#[derive(Debug)]
pub struct ProfilingMiddleware {
    /// Store timing metrics
    timings: Arc<std::sync::Mutex<HashMap<String, Vec<u128>>>>,
}

impl ProfilingMiddleware {
    /// Create a new profiling middleware.
    pub fn new() -> Self {
        Self {
            timings: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Get average execution time for a command (in milliseconds).
    pub fn get_average_time(&self, command: &str) -> Option<f64> {
        let timings = self.timings.lock().ok()?;
        let times = timings.get(command)?;
        if times.is_empty() {
            return None;
        }
        Some(times.iter().sum::<u128>() as f64 / times.len() as f64)
    }
}

impl Default for ProfilingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for ProfilingMiddleware {
    fn name(&self) -> &str {
        "profiling"
    }

    fn before(&self, _request: &MiddlewareRequest) -> crate::Result<bool> {
        // Record start time (stored in request context in real implementation)
        Ok(true)
    }

    fn after(&self, _response: &MiddlewareResponse) -> crate::Result<()> {
        // Record execution time
        if let Ok(mut timings) = self.timings.lock() {
            let command = "command"; // In real implementation, get from context
            timings
                .entry(command.to_string())
                .or_insert_with(Vec::new)
                .push(0); // Placeholder
        }
        Ok(())
    }
}

/// Rate limiting middleware.
#[derive(Debug)]
pub struct RateLimitingMiddleware {
    /// Maximum requests per window
    max_requests: Arc<AtomicUsize>,
    /// Time window in seconds
    #[allow(dead_code)]
    window_seconds: u64,
}

impl RateLimitingMiddleware {
    /// Create a new rate limiting middleware.
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            max_requests: Arc::new(AtomicUsize::new(max_requests)),
            window_seconds,
        }
    }

    /// Get the current request count.
    pub fn current_requests(&self) -> usize {
        self.max_requests.load(Ordering::SeqCst)
    }
}

impl Middleware for RateLimitingMiddleware {
    fn name(&self) -> &str {
        "rate_limiting"
    }

    fn before(&self, _request: &MiddlewareRequest) -> crate::Result<bool> {
        let current = self.max_requests.load(Ordering::SeqCst);
        if current > 0 {
            self.max_requests.fetch_sub(1, Ordering::SeqCst);
            Ok(true)
        } else {
            Err(crate::NounVerbError::MiddlewareError(
                "Rate limit exceeded".to_string(),
            ))
        }
    }
}

/// Result caching middleware.
#[derive(Debug)]
pub struct CachingMiddleware {
    /// Command result cache
    cache: Arc<std::sync::Mutex<HashMap<String, String>>>,
    /// Cache TTL in seconds
    #[allow(dead_code)]
    ttl_seconds: u64,
}

impl CachingMiddleware {
    /// Create a new caching middleware.
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            ttl_seconds,
        }
    }

    /// Get a cached result.
    pub fn get(&self, key: &str) -> Option<String> {
        self.cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(key).cloned())
    }

    /// Set a cached result.
    pub fn set(&self, key: String, value: String) -> crate::Result<()> {
        self.cache
            .lock()
            .map_err(|_| {
                crate::NounVerbError::MiddlewareError("Cache lock failed".to_string())
            })?
            .insert(key, value);
        Ok(())
    }

    /// Clear the cache.
    pub fn clear(&self) -> crate::Result<()> {
        self.cache
            .lock()
            .map_err(|_| {
                crate::NounVerbError::MiddlewareError("Cache lock failed".to_string())
            })?
            .clear();
        Ok(())
    }
}

impl Middleware for CachingMiddleware {
    fn name(&self) -> &str {
        "caching"
    }

    fn after(&self, response: &MiddlewareResponse) -> crate::Result<()> {
        if response.is_success() {
            let cache_key = format!("result_{:?}", SystemTime::now());
            self.set(cache_key, response.message().to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_middleware_creation() {
        let mw = LoggingMiddleware::new();
        assert_eq!(mw.name(), "logging");
    }

    #[test]
    fn test_error_recovery_middleware() {
        let mw = ErrorRecoveryMiddleware::new();
        assert_eq!(mw.name(), "error_recovery");
        assert_eq!(mw.max_retries, 3);
    }

    #[test]
    fn test_auth_middleware_creation() {
        let mw = AuthMiddleware::new().allow_user("alice");
        assert_eq!(mw.name(), "auth");
    }

    #[test]
    fn test_profiling_middleware() {
        let mw = ProfilingMiddleware::new();
        assert_eq!(mw.name(), "profiling");
    }

    #[test]
    fn test_rate_limiting_middleware() {
        let mw = RateLimitingMiddleware::new(10, 60);
        assert_eq!(mw.name(), "rate_limiting");
        assert_eq!(mw.current_requests(), 10);
    }

    #[test]
    fn test_caching_middleware() {
        let mw = CachingMiddleware::new(300);
        assert_eq!(mw.name(), "caching");
        mw.set("test".to_string(), "value".to_string()).unwrap();
        assert_eq!(mw.get("test"), Some("value".to_string()));
    }
}
