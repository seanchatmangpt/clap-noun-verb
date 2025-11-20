//! Domain logic for API interaction - ZERO CLI dependencies
//!
//! This module contains pure business logic with no knowledge of:
//! - Command-line arguments
//! - User interaction
//! - Terminal output formatting

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Domain error types - business logic failures only
#[derive(Debug, Error, PartialEq)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
}

/// API request - domain model
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ApiRequest {
    pub endpoint: String,
    pub query: String,
    pub max_results: usize,
}

/// API response - domain model
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ApiResponse {
    pub results: Vec<ResultItem>,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResultItem {
    pub id: String,
    pub title: String,
    pub score: f64,
}

/// Retry policy - domain rules
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

/// Circuit breaker state - domain model
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open { until: std::time::Instant },
    HalfOpen,
}

/// Circuit breaker - domain logic for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    consecutive_failures: u32,
    consecutive_successes: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_threshold,
            success_threshold,
            timeout,
            consecutive_failures: 0,
            consecutive_successes: 0,
        }
    }

    /// Check if request can proceed - pure logic
    pub fn can_request(&self) -> Result<(), ApiError> {
        match &self.state {
            CircuitState::Closed => Ok(()),
            CircuitState::HalfOpen => Ok(()),
            CircuitState::Open { until } => {
                if std::time::Instant::now() >= *until {
                    // Transition to half-open handled by caller
                    Ok(())
                } else {
                    Err(ApiError::CircuitBreakerOpen)
                }
            }
        }
    }

    /// Record success - state transition logic
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.consecutive_successes += 1;

        match self.state {
            CircuitState::HalfOpen => {
                if self.consecutive_successes >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.consecutive_successes = 0;
                }
            }
            CircuitState::Open { until } => {
                if std::time::Instant::now() >= until {
                    self.state = CircuitState::HalfOpen;
                    self.consecutive_successes = 1;
                }
            }
            CircuitState::Closed => {}
        }
    }

    /// Record failure - state transition logic
    pub fn record_failure(&mut self) {
        self.consecutive_successes = 0;
        self.consecutive_failures += 1;

        if self.consecutive_failures >= self.failure_threshold {
            self.state = CircuitState::Open {
                until: std::time::Instant::now() + self.timeout,
            };
            self.consecutive_failures = 0;
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }
}

/// Validation logic - pure business rules
pub fn validate_request(request: &ApiRequest) -> Result<(), ApiError> {
    if request.endpoint.is_empty() {
        return Err(ApiError::ValidationFailed("Endpoint cannot be empty".to_string()));
    }

    if request.query.is_empty() {
        return Err(ApiError::ValidationFailed("Query cannot be empty".to_string()));
    }

    if request.max_results == 0 {
        return Err(ApiError::ValidationFailed("Max results must be greater than 0".to_string()));
    }

    if request.max_results > 1000 {
        return Err(ApiError::ValidationFailed("Max results cannot exceed 1000".to_string()));
    }

    Ok(())
}

/// Validate response - business rules
pub fn validate_response(response: &ApiResponse) -> Result<(), ApiError> {
    if response.results.is_empty() && response.total > 0 {
        return Err(ApiError::InvalidResponse(
            "Total > 0 but results empty".to_string()
        ));
    }

    for item in &response.results {
        if item.id.is_empty() {
            return Err(ApiError::InvalidResponse(
                "Result item missing ID".to_string()
            ));
        }
        if item.score < 0.0 || item.score > 1.0 {
            return Err(ApiError::InvalidResponse(
                format!("Invalid score: {}", item.score)
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_success() {
        // Arrange
        let request = ApiRequest {
            endpoint: "/search".to_string(),
            query: "test".to_string(),
            max_results: 10,
        };

        // Act
        let result = validate_request(&request);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_request_empty_query_fails() {
        // Arrange
        let request = ApiRequest {
            endpoint: "/search".to_string(),
            query: "".to_string(),
            max_results: 10,
        };

        // Act
        let result = validate_request(&request);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ApiError::ValidationFailed("Query cannot be empty".to_string())
        );
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        // Arrange
        let mut cb = CircuitBreaker::new(3, 2, Duration::from_secs(1));

        // Act - record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Closed);

        cb.record_failure();

        // Assert - circuit should be open
        assert!(matches!(cb.state(), CircuitState::Open { .. }));
        assert!(cb.can_request().is_err());
    }

    #[test]
    fn test_circuit_breaker_closes_after_successes() {
        // Arrange
        let mut cb = CircuitBreaker::new(2, 2, Duration::from_millis(1));
        cb.record_failure();
        cb.record_failure(); // Opens circuit

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(2));

        // Act - transition to half-open, then closed
        cb.record_success();
        assert!(matches!(cb.state(), CircuitState::HalfOpen));

        cb.record_success();

        // Assert - circuit should be closed
        assert_eq!(cb.state(), &CircuitState::Closed);
    }

    #[test]
    fn test_validate_response_success() {
        // Arrange
        let response = ApiResponse {
            results: vec![
                ResultItem {
                    id: "1".to_string(),
                    title: "Test".to_string(),
                    score: 0.95,
                }
            ],
            total: 1,
            has_more: false,
        };

        // Act
        let result = validate_response(&response);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_response_invalid_score_fails() {
        // Arrange
        let response = ApiResponse {
            results: vec![
                ResultItem {
                    id: "1".to_string(),
                    title: "Test".to_string(),
                    score: 1.5, // Invalid!
                }
            ],
            total: 1,
            has_more: false,
        };

        // Act
        let result = validate_response(&response);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::InvalidResponse(_)));
    }
}
