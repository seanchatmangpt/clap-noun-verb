# API Client Example - Domain Separation with Fault Tolerance

This example demonstrates **production-ready domain separation** in an async API client with circuit breaker and retry logic.

## Architecture

```
src/
├── cli/           # CLI layer - HTTP I/O, user output
│   └── commands.rs    - Request execution, retries, formatting
├── domain/        # Domain layer - business rules
│   └── client.rs      - Validation, circuit breaker, retry policy
└── main.rs        # Entry point
```

## Key Patterns

### 1. Domain Layer (Pure State Machines)
- **Circuit breaker logic** - pure state transitions
- **Validation rules** - business constraints
- **Retry policy** - domain configuration
- **No HTTP** - no reqwest, no tokio::sleep

### 2. CLI Layer (I/O Handling)
- **HTTP requests** - actual network calls
- **Async execution** - tokio runtime
- **User feedback** - progress, retries, results
- **Delegates validation** - to domain layer

### 3. Testability
- **Domain tests** - pure unit tests, instant
- **CLI tests** - mockito HTTP mocks
- **No integration tests needed** - logic in domain

## Circuit Breaker Pattern

### Domain Logic (Testable)
```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    success_threshold: u32,
    // ...
}

impl CircuitBreaker {
    pub fn record_failure(&mut self) { /* state transition */ }
    pub fn record_success(&mut self) { /* state transition */ }
    pub fn can_request(&self) -> Result<(), ApiError> { /* check */ }
}
```

**Key insight**: Circuit breaker is a **pure state machine** - no I/O, fully testable.

### CLI Integration
```rust
async fn execute_with_retry(circuit_breaker: &mut CircuitBreaker, ...) -> Result<Response> {
    circuit_breaker.can_request()?;  // Check domain logic

    match http_request().await {      // CLI does I/O
        Ok(resp) => {
            circuit_breaker.record_success();  // Update domain state
            Ok(resp)
        }
        Err(_) => {
            circuit_breaker.record_failure();  // Update domain state
            retry()
        }
    }
}
```

## Type-First Design

```rust
// Domain types - business models
pub struct ApiRequest {
    pub endpoint: String,
    pub query: String,
    pub max_results: usize,
}

pub struct ApiResponse {
    pub results: Vec<ResultItem>,
    pub total: usize,
    pub has_more: bool,
}

// Domain validation - pure function
pub fn validate_request(request: &ApiRequest) -> Result<(), ApiError>
pub fn validate_response(response: &ApiResponse) -> Result<(), ApiError>
```

**Key insight**: Validation is **data in, error out** - no side effects.

## Running the Example

```bash
# Start mock server (in another terminal)
python3 -m http.server 8080

# Run client
cargo run -- query --query "rust" --max 5

# Run tests
cargo test
```

## Chicago TDD Examples

### Domain Layer Test
```rust
#[test]
fn test_circuit_breaker_opens_after_failures() {
    // Arrange
    let mut cb = CircuitBreaker::new(3, 2, Duration::from_secs(1));

    // Act
    cb.record_failure();
    cb.record_failure();
    cb.record_failure();

    // Assert - verify state change
    assert!(matches!(cb.state(), CircuitState::Open { .. }));
    assert!(cb.can_request().is_err());
}
```

**No mocks, no async, instant execution.**

### CLI Layer Test
```rust
#[tokio::test]
async fn test_query_command_success() {
    // Arrange - mock HTTP server
    let _m = mock("GET", "/search")
        .with_status(200)
        .with_body(r#"{"results": [], "total": 0, "has_more": false}"#)
        .create();

    // Act
    let result = query(server_url(), "/search", "test", 10).await;

    // Assert
    assert!(result.is_ok());
}
```

**Real HTTP mocking with mockito - not a trait mock.**

## Anti-Patterns Avoided

❌ **Wrong**: HTTP client in domain
```rust
// BAD - domain knows about HTTP
pub async fn query(url: &str) -> Result<Response> {
    reqwest::get(url).await  // I/O in domain!
}
```

✅ **Right**: Domain validates, CLI executes
```rust
// GOOD - domain checks rules
pub fn validate_request(req: &ApiRequest) -> Result<()>

// CLI handles HTTP
async fn execute_with_retry(...) -> Result<Response>
```

❌ **Wrong**: Circuit breaker does I/O
```rust
// BAD - state machine with side effects
impl CircuitBreaker {
    pub async fn execute<F>(&mut self, f: F) -> Result<T> {
        // Mixing state logic with I/O!
    }
}
```

✅ **Right**: Circuit breaker is pure state
```rust
// GOOD - pure state transitions
impl CircuitBreaker {
    pub fn can_request(&self) -> Result<()>
    pub fn record_success(&mut self)
    pub fn record_failure(&mut self)
}
```

## Production Features

- **Exponential backoff** - configurable retry delays
- **Circuit breaker** - prevents cascade failures
- **Request validation** - catches errors early
- **Response validation** - ensures data quality
- **Progress feedback** - user sees retries
- **Configurable policies** - adjust for environment
