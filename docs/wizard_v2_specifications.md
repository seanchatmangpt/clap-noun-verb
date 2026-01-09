# Wizard v2 - Comprehensive System Requirements Specification

**Document Version**: 2.0.0
**Date**: 2026-01-09
**Status**: Specification Phase (SPARC)
**Author**: Specification Engineer

---

## Executive Summary

This document specifies requirements for Wizard v2, a production-ready enhancement of the multi-provider AI integration layer for clap-noun-verb. v2 focuses on production hardening, zero lint violations, enhanced features, and comprehensive observability.

**v1 Status**: 4,269 lines, 88+ tests, passes lint, excellent architecture
**v2 Goals**: Production-ready, zero violations, 95%+ coverage, enhanced features

---

## 1. Introduction

### 1.1 Purpose

Wizard v2 provides a type-safe, zero-cost abstraction layer over rust-genai for multi-provider LLM integration with production-grade reliability, observability, and performance.

### 1.2 Scope

**In Scope**:
- Enhanced streaming with backpressure control
- Advanced caching with TTL and eviction policies
- Token bucket rate limiting
- Exponential backoff retry logic
- Model fallback chains
- Circuit breaker pattern
- Comprehensive error handling
- Security hardening
- Observability (metrics, tracing, logging)
- 95%+ test coverage

**Out of Scope**:
- Custom model training
- Model fine-tuning
- Non-rust-genai providers
- GUI/Web interface

### 1.3 Definitions

- **Client**: Type-safe wrapper around rust-genai Client
- **Provider**: AI service (OpenAI, Anthropic, Gemini, etc.)
- **Model**: Specific AI model variant (GPT-4, Claude-3-Sonnet, etc.)
- **Prompt**: Input to AI model with optional system message and history
- **Response**: Output from AI model with metadata and usage statistics
- **Session**: Stateful conversation context with history
- **SLO**: Service Level Objective - measurable performance target

---

## 2. Functional Requirements

### 2.1 Streaming Responses (FR-STREAM)

#### FR-STREAM-001: Token-by-Token Streaming
**Priority**: HIGH
**Description**: Stream AI responses token-by-token as they are generated

**Acceptance Criteria**:
- [ ] Client supports streaming API calls
- [ ] Tokens arrive incrementally (not batched)
- [ ] Stream can be consumed via async iterator or callback
- [ ] Partial responses are valid WizardResponse objects
- [ ] Stream metadata includes timing information

**Test Scenarios**:
```rust
// Scenario: Successful token streaming
#[tokio::test]
async fn test_token_streaming() {
    // Arrange
    let client = create_test_client().await;
    let prompt = Prompt::new("Count to 10");

    // Act
    let mut stream = client.stream_generate(prompt).await.unwrap();
    let mut tokens = Vec::new();
    while let Some(chunk) = stream.next().await {
        tokens.push(chunk.unwrap());
    }

    // Assert
    assert!(tokens.len() > 1, "Should receive multiple chunks");
    assert!(tokens.iter().all(|t| !t.text.is_empty()));
}
```

#### FR-STREAM-002: Stream Cancellation
**Priority**: HIGH
**Description**: Allow cancellation of in-flight streaming requests

**Acceptance Criteria**:
- [ ] Stream supports graceful cancellation
- [ ] Cancellation stops token generation
- [ ] Resources are cleaned up on cancellation
- [ ] Cancellation returns partial response with metadata

#### FR-STREAM-003: Backpressure Control
**Priority**: MEDIUM
**Description**: Control token flow rate with configurable buffer size

**Acceptance Criteria**:
- [ ] Configurable buffer size (default: 256 tokens)
- [ ] Backpressure prevents memory overflow
- [ ] Buffer full behavior is configurable (block/drop/error)

### 2.2 Response Caching (FR-CACHE)

#### FR-CACHE-001: LRU Cache with TTL
**Priority**: HIGH
**Description**: Cache AI responses with configurable size and time-to-live

**Acceptance Criteria**:
- [ ] Configurable cache size (default: 100 entries)
- [ ] Configurable TTL (default: 1 hour)
- [ ] LRU eviction when cache is full
- [ ] TTL eviction removes stale entries
- [ ] Cache key includes prompt, system message, and model

**Configuration**:
```rust
CacheConfig {
    enabled: bool,
    max_size: NonZeroUsize,
    ttl: Duration,
    eviction_policy: EvictionPolicy, // LRU, LFU, FIFO
}
```

#### FR-CACHE-002: Cache Invalidation
**Priority**: MEDIUM
**Description**: Manually invalidate cache entries

**Acceptance Criteria**:
- [ ] Clear entire cache
- [ ] Invalidate by key prefix
- [ ] Invalidate by model
- [ ] Invalidate expired entries only

#### FR-CACHE-003: Cache Metrics
**Priority**: MEDIUM
**Description**: Track cache performance metrics

**Acceptance Criteria**:
- [ ] Hit rate (hits / total requests)
- [ ] Miss rate
- [ ] Eviction count
- [ ] Current size and capacity
- [ ] Average TTL utilization

### 2.3 Rate Limiting (FR-RATE)

#### FR-RATE-001: Token Bucket Algorithm
**Priority**: HIGH
**Description**: Implement token bucket rate limiting per provider

**Acceptance Criteria**:
- [ ] Configurable tokens per second (default: 10)
- [ ] Configurable burst size (default: 20)
- [ ] Per-provider rate limits
- [ ] Requests block when tokens exhausted
- [ ] Tokens refill at configured rate

**Configuration**:
```rust
RateLimitConfig {
    tokens_per_second: f64,
    burst_size: usize,
    per_provider: HashMap<Provider, (f64, usize)>,
}
```

#### FR-RATE-002: Rate Limit Errors
**Priority**: HIGH
**Description**: Proper error handling for rate limit exceeded

**Acceptance Criteria**:
- [ ] Returns `WizardError::RateLimitExceeded`
- [ ] Error includes retry-after duration
- [ ] Error includes current rate limit configuration
- [ ] Non-blocking check for available tokens

#### FR-RATE-003: Dynamic Rate Adjustment
**Priority**: LOW
**Description**: Adjust rate limits based on provider responses

**Acceptance Criteria**:
- [ ] Parse rate limit headers from provider
- [ ] Auto-adjust to provider limits
- [ ] Emit warning when approaching limits
- [ ] Configurable auto-adjustment (opt-in)

### 2.4 Retry Logic (FR-RETRY)

#### FR-RETRY-001: Exponential Backoff with Jitter
**Priority**: HIGH
**Description**: Retry failed requests with exponential backoff and jitter

**Acceptance Criteria**:
- [ ] Configurable max attempts (default: 3)
- [ ] Exponential backoff (base: 2^attempt seconds)
- [ ] Jitter to prevent thundering herd (±25%)
- [ ] Configurable backoff multiplier
- [ ] Only retry transient errors (network, timeout, 5xx)

**Configuration**:
```rust
RetryConfig {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter_factor: f64, // 0.0-1.0
    retryable_errors: Vec<ErrorKind>,
}
```

**Backoff Formula**:
```
delay = min(base_delay * multiplier^attempt, max_delay)
jittered_delay = delay * (1.0 ± jitter_factor * random())
```

#### FR-RETRY-002: Retry Predicate
**Priority**: MEDIUM
**Description**: Configurable retry decision logic

**Acceptance Criteria**:
- [ ] Default predicate retries transient errors
- [ ] Custom predicate function support
- [ ] Context includes attempt number and error
- [ ] Non-retryable errors fail immediately

#### FR-RETRY-003: Retry Metrics
**Priority**: MEDIUM
**Description**: Track retry statistics

**Acceptance Criteria**:
- [ ] Total retry attempts
- [ ] Success after retry count
- [ ] Failure after max retries count
- [ ] Average retry delay
- [ ] Per-error-type retry distribution

### 2.5 Model Fallback Chains (FR-FALLBACK)

#### FR-FALLBACK-001: Sequential Model Fallback
**Priority**: MEDIUM
**Description**: Try alternative models when primary fails

**Acceptance Criteria**:
- [ ] Configurable fallback chain (list of models)
- [ ] Try models in order until success
- [ ] Return error only if all models fail
- [ ] Metadata indicates which model succeeded
- [ ] Configurable fallback triggers (error types)

**Configuration**:
```rust
FallbackConfig {
    chain: Vec<Model>,
    triggers: Vec<ErrorKind>, // Which errors trigger fallback
    preserve_context: bool,   // Pass conversation history
}
```

#### FR-FALLBACK-002: Fallback Selection Strategy
**Priority**: LOW
**Description**: Intelligent model selection for fallback

**Acceptance Criteria**:
- [ ] Sequential (default): try in order
- [ ] Cost-optimized: try cheaper models first
- [ ] Latency-optimized: try faster models first
- [ ] Custom selection function

#### FR-FALLBACK-003: Fallback Metrics
**Priority**: MEDIUM
**Description**: Track fallback performance

**Acceptance Criteria**:
- [ ] Fallback trigger count per model
- [ ] Success rate by fallback position
- [ ] Average fallback depth (how far down chain)
- [ ] Per-model reliability score

### 2.6 Session Persistence (FR-SESSION)

#### FR-SESSION-001: Session Serialization
**Priority**: MEDIUM
**Description**: Serialize and deserialize session state

**Acceptance Criteria**:
- [ ] Session can be serialized to JSON
- [ ] Session can be restored from JSON
- [ ] Conversation history is preserved
- [ ] Model configuration is preserved
- [ ] Session ID for tracking

**Session Schema**:
```rust
#[derive(Serialize, Deserialize)]
struct SessionState {
    id: Uuid,
    model: Model,
    history: Vec<Message>,
    metadata: SessionMetadata,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

#### FR-SESSION-002: Session Storage Backend
**Priority**: LOW
**Description**: Pluggable session storage (filesystem, database, memory)

**Acceptance Criteria**:
- [ ] SessionStore trait for custom backends
- [ ] In-memory store (default)
- [ ] Filesystem store (JSON files)
- [ ] Optional database store (SQLite)

#### FR-SESSION-003: Session Expiration
**Priority**: MEDIUM
**Description**: Automatic session expiration and cleanup

**Acceptance Criteria**:
- [ ] Configurable session TTL (default: 24 hours)
- [ ] Auto-cleanup of expired sessions
- [ ] Session can be explicitly closed
- [ ] Expired session error with recovery suggestion

### 2.7 Circuit Breaker (FR-CIRCUIT)

#### FR-CIRCUIT-001: Failure Threshold Detection
**Priority**: MEDIUM
**Description**: Detect repeated failures and open circuit

**Acceptance Criteria**:
- [ ] Configurable failure threshold (default: 5 failures)
- [ ] Configurable time window (default: 1 minute)
- [ ] Circuit opens after threshold exceeded
- [ ] Circuit states: Closed, Open, HalfOpen

**Configuration**:
```rust
CircuitBreakerConfig {
    failure_threshold: usize,
    time_window: Duration,
    timeout: Duration,        // How long to stay open
    half_open_max_calls: usize, // Calls to try in half-open
}
```

#### FR-CIRCUIT-002: Circuit States
**Priority**: MEDIUM
**Description**: Implement circuit breaker state machine

**State Transitions**:
```
Closed --[threshold exceeded]--> Open
Open --[timeout elapsed]--> HalfOpen
HalfOpen --[success]--> Closed
HalfOpen --[failure]--> Open
```

**Acceptance Criteria**:
- [ ] Closed: normal operation, count failures
- [ ] Open: reject requests immediately with CircuitOpen error
- [ ] HalfOpen: allow limited requests to test recovery
- [ ] State transitions are thread-safe

#### FR-CIRCUIT-003: Circuit Metrics
**Priority**: MEDIUM
**Description**: Track circuit breaker metrics

**Acceptance Criteria**:
- [ ] Current state (Closed/Open/HalfOpen)
- [ ] State transition count
- [ ] Time in each state
- [ ] Failure rate in time window
- [ ] Circuit open event notifications

---

## 3. Non-Functional Requirements

### 3.1 Performance (NFR-PERF)

#### NFR-PERF-001: Response Latency
**Description**: API response time targets

**SLOs**:
- [ ] p50 latency < 100ms (excluding LLM time)
- [ ] p95 latency < 200ms (excluding LLM time)
- [ ] p99 latency < 500ms (excluding LLM time)

**Measurement**: All latencies measured from client.generate() call to first byte

#### NFR-PERF-002: Streaming Latency
**Description**: Streaming first token time

**SLOs**:
- [ ] Time to first token < 1 second (p95)
- [ ] Token delivery rate > 50 tokens/second (average)
- [ ] Stream overhead < 50ms (vs non-streaming)

#### NFR-PERF-003: Memory Usage
**Description**: Memory footprint limits

**SLOs**:
- [ ] Client memory < 10MB (without cache)
- [ ] Cache memory < 100MB (default config)
- [ ] Session memory < 1MB per session
- [ ] No memory leaks (stable under load)

#### NFR-PERF-004: Compilation Time
**Description**: Build time targets

**SLOs**:
- [ ] Clean build < 60 seconds (release mode)
- [ ] Incremental build < 2 seconds
- [ ] Check (cargo make check) < 5 seconds

### 3.2 Reliability (NFR-REL)

#### NFR-REL-001: Error Handling
**Description**: All errors are handled with Result<T, E>

**Acceptance Criteria**:
- [ ] Zero unwrap() in production code
- [ ] Zero expect() in production code
- [ ] Zero panic!() in production code
- [ ] All public APIs return Result<T, WizardError>
- [ ] Errors include recovery suggestions

#### NFR-REL-002: Resilience
**Description**: Graceful degradation under failure

**Acceptance Criteria**:
- [ ] Retry logic for transient failures
- [ ] Circuit breaker prevents cascade failures
- [ ] Fallback models for high availability
- [ ] Timeouts prevent indefinite hangs
- [ ] Resource cleanup on errors

#### NFR-REL-003: Data Integrity
**Description**: No data corruption or loss

**Acceptance Criteria**:
- [ ] Session persistence is atomic
- [ ] Cache consistency under concurrent access
- [ ] Conversation history is append-only
- [ ] No race conditions (pass miri tests)

### 3.3 Security (NFR-SEC)

#### NFR-SEC-001: Input Validation
**Description**: Validate all inputs to prevent injection

**Acceptance Criteria**:
- [ ] Prompt length validation (max: model context window)
- [ ] System message length validation
- [ ] Model selection validation (enum only)
- [ ] Configuration validation (ranges, non-negative)
- [ ] Reject null bytes and control characters

#### NFR-SEC-002: Secret Management
**Description**: Secure handling of API keys

**Acceptance Criteria**:
- [ ] API keys never logged
- [ ] API keys never in error messages
- [ ] API keys loaded from environment only
- [ ] No hardcoded API keys
- [ ] Redacted in Debug output

#### NFR-SEC-003: Prompt Injection Defense
**Description**: Mitigate prompt injection attacks

**Acceptance Criteria**:
- [ ] System message is protected from user input
- [ ] Clear separation of user/system messages
- [ ] Optional prompt sanitization (configurable)
- [ ] Fuzz testing for injection vectors

#### NFR-SEC-004: Rate Limiting
**Description**: Prevent abuse and DoS

**Acceptance Criteria**:
- [ ] Per-session rate limits
- [ ] Global rate limits
- [ ] Configurable limits per provider
- [ ] Rate limit headers respected

### 3.4 Observability (NFR-OBS)

#### NFR-OBS-001: Structured Logging
**Description**: Comprehensive structured logging

**Acceptance Criteria**:
- [ ] All operations logged at appropriate level
- [ ] JSON structured logs (tracing-subscriber)
- [ ] Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- [ ] No sensitive data in logs (API keys, prompts)
- [ ] Correlation IDs for request tracing

**Log Events**:
```rust
// INFO: Normal operations
- "client_created"
- "request_started"
- "request_completed"
- "stream_started"
- "stream_completed"

// WARN: Degraded performance
- "cache_miss"
- "retry_attempt"
- "fallback_triggered"
- "rate_limit_approaching"

// ERROR: Failures
- "request_failed"
- "circuit_opened"
- "max_retries_exceeded"
```

#### NFR-OBS-002: Metrics
**Description**: Prometheus-compatible metrics

**Acceptance Criteria**:
- [ ] Counter: total requests, errors, retries
- [ ] Histogram: latency, token count
- [ ] Gauge: active requests, cache size, circuit state
- [ ] Metrics per provider and model
- [ ] Configurable metrics export (optional)

**Key Metrics**:
```rust
wizard_requests_total{provider, model, status}
wizard_request_duration_seconds{provider, model}
wizard_tokens_total{provider, model, type}
wizard_cache_hits_total
wizard_cache_misses_total
wizard_retry_attempts_total{provider, error_type}
wizard_circuit_state{provider}
```

#### NFR-OBS-003: Distributed Tracing
**Description**: OpenTelemetry tracing support

**Acceptance Criteria**:
- [ ] Spans for all async operations
- [ ] Parent-child span relationships
- [ ] Span attributes: model, provider, token count
- [ ] Trace context propagation
- [ ] Optional OTLP export (feature-gated)

**Spans**:
```
wizard.generate
  ├─ wizard.cache.lookup
  ├─ wizard.rate_limit.acquire
  ├─ wizard.provider.request
  │   ├─ wizard.retry.attempt (if retry)
  │   └─ wizard.fallback.try_model (if fallback)
  └─ wizard.cache.store
```

### 3.5 Maintainability (NFR-MAINT)

#### NFR-MAINT-001: Code Quality
**Description**: Zero lint violations

**Acceptance Criteria**:
- [ ] cargo clippy --all-features -- -D warnings (pass)
- [ ] cargo fmt --check (pass)
- [ ] No TODO comments (only FUTURE:)
- [ ] No commented-out code
- [ ] All public items documented

#### NFR-MAINT-002: Test Coverage
**Description**: Comprehensive test coverage

**Acceptance Criteria**:
- [ ] 95%+ line coverage (measured with tarpaulin)
- [ ] 100% public API coverage
- [ ] Unit tests for all modules
- [ ] Integration tests for workflows
- [ ] Property tests for prompt parsing
- [ ] Fuzz tests for security

**Test Categories**:
```
Unit Tests (src/wizard/*_test.rs):
- Pure functions
- State transitions
- Error cases
- Edge cases

Integration Tests (tests/wizard_*_test.rs):
- End-to-end workflows
- Provider interactions (mocked)
- Configuration scenarios
- Error recovery paths

Property Tests:
- Prompt parsing invariants
- Cache consistency
- Rate limiter fairness

Fuzz Tests:
- Prompt injection
- Invalid JSON
- Malformed inputs
```

#### NFR-MAINT-003: Documentation
**Description**: Complete documentation

**Acceptance Criteria**:
- [ ] All public APIs have rustdoc
- [ ] Examples for common use cases
- [ ] README with quickstart
- [ ] Architecture documentation
- [ ] Migration guide from v1
- [ ] API reference (docs.rs)

---

## 4. API Specification

### 4.1 Client Types

#### StreamingClient
```rust
pub struct StreamingClient {
    inner: GenAiClient,
    config: StreamConfig,
}

impl StreamingClient {
    pub async fn stream_generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<impl Stream<Item = WizardResult<ResponseChunk>>>;

    pub async fn stream_with_callback<F>(
        &mut self,
        prompt: impl Into<Prompt>,
        callback: F,
    ) -> WizardResult<WizardResponse>
    where
        F: FnMut(ResponseChunk) -> ControlFlow<(), ()>;
}

pub struct StreamConfig {
    pub buffer_size: usize,
    pub timeout: Option<Duration>,
    pub backpressure: BackpressureStrategy,
}

pub enum BackpressureStrategy {
    Block,          // Wait for consumer
    DropOldest,     // Drop old chunks
    Error,          // Return error
}
```

#### CachedClient
```rust
pub struct CachedClient {
    inner: GenAiClient,
    cache: Arc<RwLock<Cache>>,
    config: CacheConfig,
}

impl CachedClient {
    pub async fn generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<WizardResponse>;

    pub fn clear_cache(&mut self);

    pub fn invalidate(&mut self, key: &str);

    pub fn stats(&self) -> CacheStats;
}

pub struct CacheConfig {
    pub max_size: NonZeroUsize,
    pub ttl: Duration,
    pub eviction_policy: EvictionPolicy,
}

pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    FIFO, // First In First Out
}

pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub capacity: usize,
}
```

#### RateLimitedClient
```rust
pub struct RateLimitedClient {
    inner: GenAiClient,
    limiter: Arc<RwLock<RateLimiter>>,
    config: RateLimitConfig,
}

impl RateLimitedClient {
    pub async fn generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<WizardResponse>;

    pub async fn try_generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<Option<WizardResponse>>;

    pub fn tokens_available(&self) -> usize;
}

pub struct RateLimitConfig {
    pub tokens_per_second: f64,
    pub burst_size: usize,
    pub per_provider: HashMap<Provider, ProviderRateLimit>,
}

pub struct ProviderRateLimit {
    pub tokens_per_second: f64,
    pub burst_size: usize,
}
```

#### RetryClient
```rust
pub struct RetryClient {
    inner: GenAiClient,
    config: RetryConfig,
}

impl RetryClient {
    pub async fn generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<WizardResponse>;

    pub fn stats(&self) -> RetryStats;
}

pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
    pub retry_predicate: Arc<dyn Fn(&WizardError, usize) -> bool + Send + Sync>,
}

pub struct RetryStats {
    pub total_attempts: u64,
    pub successful_retries: u64,
    pub failed_after_retries: u64,
    pub average_delay: Duration,
}
```

#### FallbackClient
```rust
pub struct FallbackClient {
    clients: Vec<GenAiClient>,
    config: FallbackConfig,
}

impl FallbackClient {
    pub async fn generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<WizardResponse>;

    pub fn stats(&self) -> FallbackStats;
}

pub struct FallbackConfig {
    pub chain: Vec<Model>,
    pub triggers: Vec<ErrorKind>,
    pub preserve_context: bool,
    pub selection_strategy: SelectionStrategy,
}

pub enum SelectionStrategy {
    Sequential,
    CostOptimized,
    LatencyOptimized,
    Custom(Arc<dyn Fn(&[Model], &WizardError) -> Option<usize> + Send + Sync>),
}

pub struct FallbackStats {
    pub triggers: HashMap<Model, u64>,
    pub successes_by_position: Vec<u64>,
    pub average_depth: f64,
}
```

#### CircuitBreakerClient
```rust
pub struct CircuitBreakerClient {
    inner: GenAiClient,
    breaker: Arc<RwLock<CircuitBreaker>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreakerClient {
    pub async fn generate(
        &mut self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<WizardResponse>;

    pub fn state(&self) -> CircuitState;

    pub fn reset(&mut self);
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub time_window: Duration,
    pub timeout: Duration,
    pub half_open_max_calls: usize,
}

pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

### 4.2 Builder Pattern

#### CompletionOptions
```rust
pub struct CompletionOptions {
    // Model parameters
    pub model: Model,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,

    // Feature flags
    pub enable_streaming: bool,
    pub enable_caching: bool,
    pub enable_retry: bool,
    pub enable_fallback: bool,
    pub enable_circuit_breaker: bool,

    // Feature configurations
    pub stream_config: Option<StreamConfig>,
    pub cache_config: Option<CacheConfig>,
    pub rate_limit_config: Option<RateLimitConfig>,
    pub retry_config: Option<RetryConfig>,
    pub fallback_config: Option<FallbackConfig>,
    pub circuit_breaker_config: Option<CircuitBreakerConfig>,
}

impl CompletionOptions {
    pub fn builder() -> CompletionOptionsBuilder;
}

pub struct CompletionOptionsBuilder {
    // Implements fluent builder pattern
}
```

### 4.3 Error Types

#### Enhanced WizardError
```rust
#[derive(Debug, thiserror::Error)]
pub enum WizardError {
    // Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String, suggestion: Option<String> },

    #[error("Token limit exceeded: requested {requested}, max {max}")]
    TokenLimit { requested: usize, max: usize },

    #[error("Authentication failed: {0}")]
    Auth(String),

    // Request errors
    #[error("API request failed: {message}")]
    Request { message: String, provider: Provider, retryable: bool },

    #[error("Response parsing failed: {0}")]
    Parse(String),

    #[error("Request timeout after {duration:?}")]
    Timeout { duration: Duration },

    // Rate limiting
    #[error("Rate limit exceeded, retry after {retry_after:?}")]
    RateLimitExceeded { retry_after: Duration, limit: RateLimitConfig },

    // Circuit breaker
    #[error("Circuit breaker open for {provider:?}, retry after {retry_after:?}")]
    CircuitOpen { provider: Provider, retry_after: Duration },

    // Retry exhausted
    #[error("Max retry attempts ({attempts}) exceeded")]
    MaxRetriesExceeded { attempts: usize, last_error: Box<WizardError> },

    // Fallback errors
    #[error("All fallback models failed")]
    AllFallbacksFailed { errors: Vec<(Model, WizardError)> },

    // Session errors
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session expired at {expired_at}")]
    SessionExpired { expired_at: DateTime<Utc> },

    // Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    // Stream errors
    #[error("Stream cancelled")]
    StreamCancelled,

    #[error("Stream buffer full")]
    StreamBufferFull,

    // Validation errors
    #[error("Invalid prompt: {0}")]
    InvalidPrompt(String),

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: String, to: String },

    // I/O and serialization
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    // Generic
    #[error("{0}")]
    Other(String),
}

impl WizardError {
    /// Get recovery suggestion for this error
    pub fn suggestion(&self) -> Option<&str>;

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool;

    /// Get error kind for categorization
    pub fn kind(&self) -> ErrorKind;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Configuration,
    Authentication,
    Network,
    Timeout,
    RateLimit,
    CircuitBreaker,
    Validation,
    Parse,
    Session,
    Cache,
    Stream,
    Unknown,
}
```

---

## 5. Configuration Specification

### 5.1 Unified Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardConfig {
    // Core configuration
    pub model_config: ModelConfig,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,

    // Feature configurations
    #[serde(default)]
    pub streaming: StreamConfig,

    #[serde(default)]
    pub caching: CacheConfig,

    #[serde(default)]
    pub rate_limiting: RateLimitConfig,

    #[serde(default)]
    pub retry: RetryConfig,

    #[serde(default)]
    pub fallback: FallbackConfig,

    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    #[serde(default)]
    pub observability: ObservabilityConfig,
}

impl WizardConfig {
    pub fn from_env() -> WizardResult<Self>;
    pub fn from_file(path: impl AsRef<Path>) -> WizardResult<Self>;
    pub fn from_json(json: &str) -> WizardResult<Self>;
    pub fn validate(&self) -> WizardResult<()>;
}
```

### 5.2 Configuration Validation

```rust
impl WizardConfig {
    pub fn validate(&self) -> WizardResult<()> {
        // Model config validation
        self.model_config.validate()?;

        // API key validation (except Ollama)
        if self.model_config.model.provider() != Provider::Ollama {
            self.api_key.as_ref()
                .ok_or_else(|| WizardError::Config {
                    message: "API key required".to_string(),
                    suggestion: Some("Set ANTHROPIC_API_KEY environment variable".to_string()),
                })?;
        }

        // Stream config validation
        if self.streaming.buffer_size == 0 {
            return Err(WizardError::Config {
                message: "Stream buffer size must be > 0".to_string(),
                suggestion: Some("Set buffer_size >= 64".to_string()),
            });
        }

        // Cache config validation
        if self.caching.ttl.is_zero() {
            return Err(WizardError::Config {
                message: "Cache TTL must be > 0".to_string(),
                suggestion: Some("Set ttl >= 60 seconds".to_string()),
            });
        }

        // Rate limit validation
        if self.rate_limiting.tokens_per_second <= 0.0 {
            return Err(WizardError::Config {
                message: "Rate limit must be > 0".to_string(),
                suggestion: Some("Set tokens_per_second >= 1.0".to_string()),
            });
        }

        // Retry config validation
        if self.retry.max_attempts == 0 {
            return Err(WizardError::Config {
                message: "Max retry attempts must be > 0".to_string(),
                suggestion: Some("Set max_attempts >= 1".to_string()),
            });
        }

        // Fallback chain validation
        if !self.fallback.chain.is_empty() {
            // Ensure fallback models are different from primary
            if self.fallback.chain.contains(&self.model_config.model) {
                return Err(WizardError::Config {
                    message: "Fallback chain cannot include primary model".to_string(),
                    suggestion: Some("Remove primary model from fallback chain".to_string()),
                });
            }
        }

        Ok(())
    }
}
```

### 5.3 Environment Variables

```bash
# Core configuration
WIZARD_MODEL=anthropic:claude-3-sonnet
ANTHROPIC_API_KEY=sk-ant-...
WIZARD_ENDPOINT=https://api.anthropic.com/v1 # Optional

# Feature flags
WIZARD_ENABLE_STREAMING=true
WIZARD_ENABLE_CACHING=true
WIZARD_ENABLE_RETRY=true
WIZARD_ENABLE_FALLBACK=false
WIZARD_ENABLE_CIRCUIT_BREAKER=true

# Streaming configuration
WIZARD_STREAM_BUFFER_SIZE=256
WIZARD_STREAM_TIMEOUT_MS=30000

# Cache configuration
WIZARD_CACHE_MAX_SIZE=100
WIZARD_CACHE_TTL_SECONDS=3600
WIZARD_CACHE_EVICTION_POLICY=lru

# Rate limiting
WIZARD_RATE_LIMIT_TPS=10.0
WIZARD_RATE_LIMIT_BURST=20

# Retry configuration
WIZARD_RETRY_MAX_ATTEMPTS=3
WIZARD_RETRY_BASE_DELAY_MS=1000
WIZARD_RETRY_MAX_DELAY_MS=30000
WIZARD_RETRY_BACKOFF_MULTIPLIER=2.0
WIZARD_RETRY_JITTER_FACTOR=0.25

# Fallback configuration
WIZARD_FALLBACK_CHAIN=anthropic:claude-3-haiku,openai:gpt-3.5-turbo

# Circuit breaker
WIZARD_CIRCUIT_FAILURE_THRESHOLD=5
WIZARD_CIRCUIT_TIME_WINDOW_MS=60000
WIZARD_CIRCUIT_TIMEOUT_MS=30000

# Observability
WIZARD_LOG_LEVEL=info
WIZARD_ENABLE_METRICS=true
WIZARD_ENABLE_TRACING=false
WIZARD_OTLP_ENDPOINT=http://localhost:4317 # Optional
```

---

## 6. Test Strategy

### 6.1 Test Coverage Requirements

#### Coverage Targets
- [ ] Overall line coverage: ≥ 95%
- [ ] Public API coverage: 100%
- [ ] Error path coverage: ≥ 90%
- [ ] Edge case coverage: ≥ 80%

#### Coverage by Module
```
src/wizard/
├── builder.rs        100% (all builders must be tested)
├── client.rs         ≥95% (core functionality)
├── config.rs         100% (all config paths)
├── error.rs          100% (all error types)
├── types.rs          100% (all type constructors)
├── streaming.rs      ≥95% (critical streaming logic)
├── cache.rs          ≥95% (cache operations)
├── rate_limit.rs     ≥95% (rate limiting)
├── retry.rs          ≥95% (retry logic)
├── fallback.rs       ≥95% (fallback chains)
├── session.rs        ≥90% (session management)
├── prompt.rs         ≥90% (prompt processing)
└── interactive.rs    ≥85% (UI logic)
```

### 6.2 Chicago TDD Requirements

All tests MUST follow Chicago TDD (state-based testing):

**Required Patterns**:
1. **AAA Pattern**: Arrange - Act - Assert
2. **State-based**: Verify outputs and state changes, not implementation
3. **Real collaborators**: Use real objects, minimize mocks
4. **Behavior verification**: Test what code does, not how it does it
5. **Observable outputs**: Test return values, side effects, state mutations

**Example**:
```rust
#[test]
fn test_cache_stores_and_retrieves_responses() {
    // Arrange
    let mut cache = Cache::new(NonZeroUsize::new(10).unwrap());
    let key = "test_key";
    let response = WizardResponse::new("test response", "gpt-4");

    // Act
    cache.put(key.to_string(), response.clone());
    let retrieved = cache.get(key);

    // Assert
    assert_eq!(retrieved, Some(&response));
    assert_eq!(cache.len(), 1);
}
```

### 6.3 Test Categories

#### Unit Tests (src/wizard/*_test.rs or #[cfg(test)] mod tests)
```rust
// Pure function tests
#[test]
fn test_exponential_backoff_calculation() {
    // Test mathematical correctness of backoff formula
}

// State transition tests
#[test]
fn test_circuit_breaker_state_transitions() {
    // Test state machine transitions
}

// Error case tests
#[test]
fn test_invalid_config_returns_error() {
    // Test error handling paths
}

// Edge case tests
#[test]
fn test_cache_eviction_at_capacity() {
    // Test boundary conditions
}
```

#### Integration Tests (tests/wizard_*_test.rs)
```rust
// End-to-end workflow tests
#[tokio::test]
async fn test_complete_generation_workflow() {
    // Create client → generate → verify response
}

// Provider interaction tests (mocked)
#[tokio::test]
async fn test_anthropic_provider_integration() {
    // Test with mocked anthropic API
}

// Configuration scenario tests
#[test]
fn test_config_from_env_variables() {
    // Test environment-based configuration
}

// Error recovery tests
#[tokio::test]
async fn test_retry_after_transient_failure() {
    // Test retry logic with simulated failures
}
```

#### Property Tests (proptest)
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_cache_size_never_exceeds_capacity(
        ops in prop::collection::vec(cache_operation(), 1..100)
    ) {
        let mut cache = Cache::new(NonZeroUsize::new(10).unwrap());
        for op in ops {
            apply_operation(&mut cache, op);
            assert!(cache.len() <= cache.capacity());
        }
    }

    #[test]
    fn test_rate_limiter_fairness(
        requests in prop::collection::vec(1..100usize, 10..50)
    ) {
        let limiter = RateLimiter::new(10.0, 20);
        // Verify fair distribution of tokens
    }
}
```

#### Fuzz Tests (cargo-fuzz)
```rust
// Fuzz prompt parsing for injection vulnerabilities
#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = Prompt::new(s);
        // Should never panic, regardless of input
    }
});

// Fuzz configuration parsing
#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = WizardConfig::from_json(s);
        // Should return error for invalid JSON, never panic
    }
});
```

#### Benchmark Tests (criterion)
```rust
fn bench_streaming_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    group.bench_function("token_streaming_1000_tokens", |b| {
        b.iter(|| {
            // Measure streaming throughput
        });
    });

    group.finish();
}
```

### 6.4 Mock Strategy

**Minimal Mocking** (Chicago TDD principle):
- Use real implementations where possible
- Mock only external dependencies (network, filesystem)
- Use test doubles for slow/non-deterministic operations

**Mock External Services**:
```rust
struct MockGenAiClient {
    responses: VecDeque<Result<ChatResponse, genai::Error>>,
}

impl MockGenAiClient {
    fn with_responses(responses: Vec<Result<ChatResponse, genai::Error>>) -> Self {
        Self { responses: responses.into() }
    }

    async fn exec_chat(&mut self, ...) -> Result<ChatResponse, genai::Error> {
        self.responses.pop_front().unwrap_or_else(|| {
            Err(genai::Error::new("No more mock responses"))
        })
    }
}
```

### 6.5 Test Organization

```
tests/
├── wizard_integration_test.rs       # Core integration tests
├── wizard_streaming_test.rs         # Streaming feature tests
├── wizard_caching_test.rs           # Caching feature tests
├── wizard_rate_limit_test.rs        # Rate limiting tests
├── wizard_retry_test.rs             # Retry logic tests
├── wizard_fallback_test.rs          # Fallback chain tests
├── wizard_circuit_breaker_test.rs   # Circuit breaker tests
├── wizard_session_test.rs           # Session persistence tests
├── wizard_e2e_workflow_test.rs      # End-to-end workflows
├── wizard_property_based_test.rs    # Property-based tests
└── wizard_security_test.rs          # Security and fuzz tests
```

---

## 7. Acceptance Criteria

### 7.1 Definition of Done

Before marking v2 as complete, ALL criteria must be met:

#### Code Quality
- [ ] Zero clippy warnings (`cargo make lint`)
- [ ] Zero compiler warnings
- [ ] Zero unwrap/expect/panic in production code
- [ ] All code formatted (`cargo make format`)
- [ ] No TODO comments (only FUTURE:)
- [ ] No commented-out code

#### Testing
- [ ] 95%+ line coverage (measured with `cargo tarpaulin`)
- [ ] 100% public API coverage
- [ ] All tests pass (`cargo make test`)
- [ ] Property tests pass (`cargo test --features proptest`)
- [ ] Benchmark tests compile and run
- [ ] No flaky tests (100 consecutive runs pass)

#### Performance
- [ ] All SLOs met (verified with benchmarks)
- [ ] No performance regressions vs v1
- [ ] Memory usage within targets
- [ ] Compilation time within targets

#### Documentation
- [ ] All public APIs documented
- [ ] Examples compile and run
- [ ] README.md updated
- [ ] CHANGELOG.md updated
- [ ] Migration guide from v1
- [ ] docs.rs builds successfully

#### Security
- [ ] Input validation complete
- [ ] No secrets in logs/errors
- [ ] Fuzz tests pass
- [ ] Dependency audit clean (`cargo audit`)

#### Observability
- [ ] Logging implemented
- [ ] Metrics exposed
- [ ] Tracing spans complete
- [ ] No PII in telemetry

### 7.2 Validation Checklist

Run this checklist before release:

```bash
# 1. Check for compiler errors
cargo make check

# 2. Run all tests
cargo make test

# 3. Check linting
cargo make lint

# 4. Run benchmarks
cargo make bench

# 5. Check coverage
cargo tarpaulin --out Html

# 6. Security audit
cargo audit

# 7. Check documentation
cargo doc --no-deps --open

# 8. Verify SLOs
cargo make slo-check

# 9. Run property tests
cargo test --features proptest

# 10. Check for forbidden patterns
rg -i 'unwrap\(\)|expect\(|panic!|todo!|unimplemented!' src/wizard --type rust --files-with-matches
```

**Expected Results**:
- All commands exit with code 0
- Coverage report shows ≥95%
- Audit shows no vulnerabilities
- Forbidden patterns only in test code

### 7.3 Release Criteria

**v2.0.0 Release Requirements**:

1. **Functional Completeness**:
   - [ ] All FR requirements implemented
   - [ ] All NFR requirements met
   - [ ] API stable and backward-compatible (within v2)

2. **Quality Gates**:
   - [ ] Definition of Done checklist 100% complete
   - [ ] Validation checklist passes
   - [ ] No known critical bugs
   - [ ] No known security vulnerabilities

3. **Documentation**:
   - [ ] User guide complete
   - [ ] API reference complete
   - [ ] Examples for all features
   - [ ] Migration guide from v1

4. **Stakeholder Approval**:
   - [ ] Code review approved
   - [ ] Architecture review approved
   - [ ] Performance review approved
   - [ ] Security review approved

---

## 8. Success Metrics

### 8.1 Development Metrics

**Velocity**:
- Sprint duration: 2 weeks
- Expected completion: 3-4 sprints (6-8 weeks)
- Features per sprint: 2-3 major features

**Quality**:
- Bug escape rate: < 5% (bugs found post-merge)
- Test coverage: ≥ 95%
- Code review approval rate: 100% (no merges without review)

### 8.2 Performance Metrics

**Latency** (measured with benchmarks):
- p50 < 100ms
- p95 < 200ms
- p99 < 500ms

**Throughput**:
- Streaming: > 50 tokens/second
- Concurrent sessions: > 100 simultaneous
- Requests per second: > 1000 (without rate limiting)

**Resource Usage**:
- Memory: < 10MB (client) + < 100MB (cache)
- CPU: < 5% idle, < 50% under load
- Build time: < 60s clean, < 2s incremental

### 8.3 Reliability Metrics

**Error Rates**:
- Client errors: < 0.1% (errors from wizard code)
- Retry success rate: > 80% (successful after retry)
- Circuit breaker trips: < 1 per hour (under normal operation)

**Availability**:
- Client uptime: 99.9% (excluding provider outages)
- Recovery time: < 30s (from circuit breaker open to closed)

### 8.4 Adoption Metrics

**v1 → v2 Migration**:
- Migration effort: < 4 hours (for typical usage)
- Breaking changes: Minimize (document all)
- Backward compatibility: Maintain where possible

**Community**:
- Documentation completeness: 100% public APIs
- Example coverage: 100% common use cases
- Issue response time: < 24 hours

---

## 9. Risk Analysis

### 9.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| rust-genai API changes | Medium | High | Version pinning, integration tests |
| Performance regressions | Low | High | Continuous benchmarking, SLO enforcement |
| Memory leaks in cache | Low | Medium | Property tests, valgrind/miri checks |
| Race conditions | Low | High | Miri tests, careful synchronization |
| Breaking changes from v1 | Medium | Medium | Deprecation warnings, migration guide |

### 9.2 Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Scope creep | Medium | Medium | Strict prioritization, MVP focus |
| Testing takes longer | Medium | Low | Parallel testing, incremental coverage |
| Integration complexity | Low | Medium | Phased integration, feature flags |

### 9.3 Quality Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Insufficient test coverage | Low | High | Coverage gates, mandatory reviews |
| Undetected security issues | Low | Critical | Fuzz testing, security review |
| Poor documentation | Medium | Medium | Docs in Definition of Done |

---

## 10. Appendices

### 10.1 Glossary

- **Andon Signal**: Visual problem indicator (compiler errors, test failures, warnings) that requires immediate attention
- **Chicago TDD**: State-based testing approach using real collaborators and behavior verification
- **Circuit Breaker**: Design pattern that prevents cascade failures by detecting and isolating faults
- **DfLSS**: Design for Lean Six Sigma - methodology for defect prevention
- **Exponential Backoff**: Retry strategy where delay increases exponentially with each attempt
- **Jitter**: Random variance added to retry delays to prevent thundering herd
- **LRU**: Least Recently Used - cache eviction policy
- **Property Testing**: Testing approach that verifies invariants across many random inputs
- **SLO**: Service Level Objective - measurable performance target
- **Token Bucket**: Rate limiting algorithm using token consumption/refill
- **TTL**: Time To Live - duration before data expires
- **Type-First Thinking**: Design approach using types to encode invariants at compile time
- **Zero-Cost Abstraction**: Abstraction with no runtime overhead

### 10.2 References

- [SPARC Methodology](https://github.com/ruvnet/sparc)
- [Chicago TDD](http://www.natpryce.com/articles/000772.html)
- [rust-genai Documentation](https://docs.rs/genai)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Circuit Breaker Pattern](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Token Bucket Algorithm](https://en.wikipedia.org/wiki/Token_bucket)

### 10.3 Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 2.0.0 | 2026-01-09 | Specification Engineer | Initial v2 specification |

---

## 11. Specification Approval

**Stakeholders**:
- [ ] Technical Lead
- [ ] Product Owner
- [ ] Security Engineer
- [ ] Quality Assurance
- [ ] DevOps Engineer

**Approval Date**: _____________
**Signed**: _____________

---

*End of Specification Document*
