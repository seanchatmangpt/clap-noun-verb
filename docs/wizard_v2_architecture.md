# Wizard Package v2 Architecture Design

**Status**: Design Phase
**Version**: 2.0.0
**Date**: 2026-01-09
**Author**: System Architect
**Branch**: claude/wizard-package-launch-Qumas

---

## Executive Summary

The Wizard package v2 is a comprehensive refactoring of the AI integration layer, transitioning from prototype (v1) to production-ready implementation. This architecture addresses 23 lint violations, completes missing error infrastructure, enhances reliability features, and optimizes performance while maintaining backwards compatibility.

**Key Metrics**:
- Current: ~4,704 lines across 16 modules, 88 tests
- Lint Violations: 23 instances (unwrap/expect/panic/unsafe/todo)
- Missing Infrastructure: 9 error variants
- Target: Zero violations, complete type-safe error handling, <100ms p99 latency

---

## 1. Lint Violation Remediation Strategy

### 1.1 Current Violation Inventory

**Distribution by Type** (23 total):
- streaming.rs: 3 violations
- session.rs: 1 violation
- prompt.rs: 3 violations
- interactive.rs: 5 violations
- genai.rs: 1 violation
- config.rs: 3 violations
- client.rs: 1 violation
- cli.rs: 2 violations
- builder.rs: 4 violations

**Critical Issues**:
1. **Missing Error Variants**: error.rs lacks 9 error types referenced throughout codebase
2. **expect() in safety-critical code**: client.rs line 51, config.rs line 155
3. **unwrap() in stream processing**: streaming.rs lines 180, 184
4. **unwrap_or chains**: Multiple defensive unwrap_or patterns indicate missing Result types

### 1.2 Error Infrastructure Completion

**Phase 1: Complete WizardError Enum**

Add missing error variants to `/src/wizard/error.rs`:

```rust
/// Error type for wizard operations (v2 - Production Ready)
#[derive(Debug)]
pub enum WizardError {
    // === Existing variants ===
    ClientError(String),
    InvalidStateTransition { from: String, to: String },
    InvalidPrompt(String),
    SessionNotInitialized,
    ConfigError(String),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    EnvVarError(std::env::VarError),
    Other(String),

    // === NEW v2 variants ===
    /// Authentication failure (invalid API key, expired token)
    Auth(String),

    /// Token limit exceeded for model
    TokenLimit { requested: usize, max: usize },

    /// API request failure (network, HTTP errors)
    Request(String),

    /// Response parsing failure
    Parse(String),

    /// Configuration validation failure
    Config(String),

    /// Rate limit exceeded
    RateLimit(String),

    /// Request timeout
    Timeout(String),

    /// Network connectivity error
    Network(String),

    /// Fallback chain exhausted
    Fallback(String),
}
```

**Implementation Strategy**:
- Add Display, Error trait implementations for new variants
- Add From<T> conversions for common error types
- Maintain source() chain for error context
- Add builder methods for detailed error construction

### 1.3 Violation Resolution Patterns

**Pattern 1: Replace expect() with ?**

```rust
// ❌ BEFORE (client.rs line 51)
Some(LruCache::new(NonZeroUsize::new(100).expect("100 is non-zero")))

// ✅ AFTER
NonZeroUsize::new(100)
    .ok_or_else(|| WizardError::Config("Invalid cache size".to_string()))
    .map(LruCache::new)?
```

**Pattern 2: Replace unwrap() in streaming with Result propagation**

```rust
// ❌ BEFORE (streaming.rs line 180)
let text = chunk.content.as_ref()
    .and_then(|c| c.text_as_str())
    .unwrap_or("");

// ✅ AFTER
let text = chunk.content
    .as_ref()
    .and_then(|c| c.text_as_str())
    .ok_or_else(|| WizardError::Parse("No text content in chunk".to_string()))?;
```

**Pattern 3: Replace unwrap_or with explicit error handling**

```rust
// ❌ BEFORE (retry.rs line 240)
Err(last_error.unwrap_or_else(|| WizardError::Other("All retry attempts failed".to_string())))

// ✅ AFTER
Err(last_error.ok_or_else(|| {
    WizardError::Retry {
        attempts: context.attempts,
        last_error: "Unknown error".to_string(),
    }
})?)
```

**Pattern 4: Type-safe NonZeroUsize construction**

```rust
// Helper function for validated cache sizes
const fn validated_cache_size(size: usize) -> Result<NonZeroUsize, WizardError> {
    match NonZeroUsize::new(size) {
        Some(nz) => Ok(nz),
        None => Err(WizardError::Config("Cache size must be non-zero".to_string())),
    }
}
```

### 1.4 Remediation Roadmap

**Sprint 1: Error Infrastructure** (2 days)
1. Add missing error variants to error.rs
2. Implement Display/Error traits for new variants
3. Add From<T> conversions
4. Update Result type alias
5. Add error context builders

**Sprint 2: Critical Path Fixes** (3 days)
1. Fix client.rs cache initialization (expect removal)
2. Fix streaming.rs text extraction (unwrap removal)
3. Fix retry.rs error propagation
4. Fix config.rs validation (expect removal)
5. Fix builder.rs error handling

**Sprint 3: Comprehensive Validation** (2 days)
1. Fix interactive.rs user input handling (5 violations)
2. Fix remaining builder patterns
3. Run clippy with --deny warnings
4. Validate all error paths with tests
5. Property-based testing for error recovery

---

## 2. v2 Feature Architecture

### 2.1 Streaming Responses (Enhancement)

**Current State**: Basic implementation exists (streaming.rs), but has unwrap violations

**v2 Design**:

```rust
/// Enhanced streaming with backpressure and cancellation
pub struct StreamingClient {
    client: genai::Client,
    config: ModelConfig,
    streaming_config: StreamingConfig,
    // NEW: Circuit breaker for error recovery
    circuit_breaker: Arc<CircuitBreaker>,
}

impl StreamingClient {
    /// Generate stream with enhanced error recovery
    pub async fn generate_stream(
        &self,
        prompt: impl Into<Prompt>,
    ) -> WizardResult<impl Stream<Item = WizardResult<StreamChunk>>> {
        // Check circuit breaker state
        if self.circuit_breaker.is_open().await {
            return Err(WizardError::CircuitOpen("Too many recent failures".to_string()));
        }

        // Build request with validation
        let request = self.build_request(prompt.into())?;

        // Execute with error tracking
        let stream = self.client
            .exec_chat_stream(self.config.model.model_id(), request, None)
            .await
            .map_err(|e| {
                self.circuit_breaker.record_failure();
                WizardError::Request(e.to_string())
            })?;

        // Wrap with error recovery
        Ok(stream.map(move |result| {
            match result {
                Ok(chunk) => self.parse_chunk(chunk),
                Err(e) => {
                    self.circuit_breaker.record_failure();
                    Err(WizardError::Request(e.to_string()))
                }
            }
        }))
    }

    /// Parse chunk with proper error handling (no unwrap!)
    fn parse_chunk(&self, chunk: genai::chat::ChatStreamResponse) -> WizardResult<StreamChunk> {
        let text = chunk.content
            .as_ref()
            .and_then(|c| c.text_as_str())
            .ok_or_else(|| WizardError::Parse("No text content in stream chunk".to_string()))?;

        let usage = chunk.usage
            .map(|u| TokenUsage::new(
                u.prompt_tokens.unwrap_or(0),
                u.completion_tokens.unwrap_or(0)
            ));

        Ok(StreamChunk {
            text: text.to_string(),
            is_final: chunk.is_final(),
            usage,
        })
    }
}
```

**Key Improvements**:
- ✅ Zero unwrap/expect calls
- ✅ Circuit breaker integration
- ✅ Explicit error types for all failure modes
- ✅ Backpressure via tokio channel bounds

### 2.2 Response Caching (Enhancement)

**Current State**: LRU cache with TTL implemented, but uses expect()

**v2 Design**:

```rust
/// Enhanced caching with async invalidation and warming
pub struct CachedClient {
    client: GenAiClient,
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<AtomicCacheStats>,
}

impl CachedClient {
    /// Create with validated configuration
    pub async fn new(wizard_config: WizardConfig, cache_config: CacheConfig) -> WizardResult<Self> {
        cache_config.validate()?;

        let cache_size = NonZeroUsize::new(cache_config.max_entries)
            .ok_or_else(|| WizardError::Config("Cache size must be non-zero".to_string()))?;

        Ok(Self {
            client: GenAiClient::new(wizard_config).await?,
            cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            config: cache_config,
            stats: Arc::new(AtomicCacheStats::default()),
        })
    }

    /// NEW: Warm cache with common prompts
    pub async fn warm_cache(&mut self, prompts: Vec<Prompt>) -> WizardResult<usize> {
        let mut warmed = 0;
        for prompt in prompts {
            match self.generate(prompt).await {
                Ok(_) => warmed += 1,
                Err(e) => log::warn!("Cache warming failed: {}", e),
            }
        }
        Ok(warmed)
    }

    /// NEW: Async cache invalidation by pattern
    pub async fn invalidate_by_pattern(&mut self, pattern: &str) -> usize {
        let mut cache = self.cache.write().await;
        let mut invalidated = 0;

        let keys_to_remove: Vec<_> = cache.iter()
            .filter_map(|(k, _)| {
                if k.contains(pattern) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            cache.pop(&key);
            invalidated += 1;
        }

        invalidated
    }
}

/// Atomic cache statistics (lock-free)
struct AtomicCacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    insertions: AtomicU64,
}
```

**Key Improvements**:
- ✅ Validated NonZeroUsize construction
- ✅ Async RwLock for concurrent access
- ✅ Atomic statistics (zero-cost reads)
- ✅ Cache warming for hot paths
- ✅ Pattern-based invalidation

### 2.3 Rate Limiting (Production Ready)

**Current State**: Token bucket implemented correctly

**v2 Enhancements**:

```rust
/// Enhanced rate limiter with adaptive limits
pub struct RateLimitedClient {
    client: GenAiClient,
    bucket: Arc<Mutex<TokenBucket>>,
    config: RateLimitConfig,
    // NEW: Adaptive rate adjustment based on API feedback
    adaptive_limiter: Option<AdaptiveRateLimiter>,
}

/// Adaptive rate limiter that adjusts based on X-RateLimit-* headers
struct AdaptiveRateLimiter {
    current_limit: AtomicU64,
    reset_time: Arc<RwLock<Option<Instant>>>,
}

impl AdaptiveRateLimiter {
    /// Update limits from response headers
    fn update_from_headers(&self, headers: &HeaderMap) {
        if let Some(remaining) = headers.get("X-RateLimit-Remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_val) = remaining_str.parse::<u64>() {
                    self.current_limit.store(remaining_val, Ordering::Relaxed);
                }
            }
        }

        if let Some(reset) = headers.get("X-RateLimit-Reset") {
            // Parse reset timestamp and update
        }
    }
}
```

**Key Improvements**:
- ✅ Adaptive rate limiting from API headers
- ✅ Per-endpoint rate limits
- ✅ Burst allowance tracking
- ✅ Rate limit metrics export

### 2.4 Retry Logic (Enhanced)

**Current State**: Exponential backoff with jitter implemented, has unwrap issue

**v2 Design**:

```rust
/// Enhanced retry with circuit breaker integration
pub struct RetryClient {
    client: GenAiClient,
    config: RetryConfig,
    // NEW: Circuit breaker to stop retries when service is down
    circuit_breaker: Arc<CircuitBreaker>,
}

impl RetryClient {
    pub async fn generate_with_context(
        &mut self,
        prompt: Prompt,
    ) -> WizardResult<(WizardResponse, RetryContext)> {
        let mut context = RetryContext::new();
        let mut last_error = None;

        for attempt in 0..self.config.max_attempts {
            // Check circuit breaker before attempting
            if self.circuit_breaker.is_open().await {
                return Err(WizardError::CircuitOpen(
                    "Circuit breaker open, too many failures".to_string()
                ));
            }

            match self.client.generate(prompt.clone()).await {
                Ok(response) => {
                    self.circuit_breaker.record_success();
                    return Ok((response, context));
                }
                Err(error) => {
                    self.circuit_breaker.record_failure();

                    if !Self::should_retry(&error, &self.config) {
                        return Err(error);
                    }

                    last_error = Some(error.clone());

                    if attempt + 1 < self.config.max_attempts {
                        let delay = self.config.calculate_delay(attempt);
                        context.record_attempt(delay.as_millis() as u64, &error);
                        tokio::time::sleep(delay).await;
                    } else {
                        context.record_attempt(0, &error);
                    }
                }
            }
        }

        // All attempts failed - use ok_or_else instead of unwrap_or_else
        Err(last_error.ok_or_else(|| {
            WizardError::Retry {
                attempts: self.config.max_attempts,
                context: format!("All {} attempts failed", self.config.max_attempts),
            }
        })?)
    }
}
```

**Key Improvements**:
- ✅ Circuit breaker integration
- ✅ Structured retry context
- ✅ No unwrap() calls
- ✅ Configurable per-error-type strategies

### 2.5 Session Persistence (NEW)

**Design**:

```rust
/// Session persistence for stateful conversations
pub struct PersistentSession {
    session: InteractiveSession,
    storage: Box<dyn SessionStorage>,
}

/// Trait for session storage backends
#[async_trait]
pub trait SessionStorage: Send + Sync {
    async fn save(&self, session_id: &str, data: &SessionData) -> WizardResult<()>;
    async fn load(&self, session_id: &str) -> WizardResult<SessionData>;
    async fn delete(&self, session_id: &str) -> WizardResult<()>;
}

/// Serializable session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub conversation_history: Vec<Message>,
    pub context: HashMap<String, serde_json::Value>,
}

/// File-based session storage
pub struct FileSessionStorage {
    base_path: PathBuf,
}

#[async_trait]
impl SessionStorage for FileSessionStorage {
    async fn save(&self, session_id: &str, data: &SessionData) -> WizardResult<()> {
        let path = self.session_path(session_id);
        let json = serde_json::to_string_pretty(data)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    async fn load(&self, session_id: &str) -> WizardResult<SessionData> {
        let path = self.session_path(session_id);
        let json = tokio::fs::read_to_string(path).await?;
        let data = serde_json::from_str(&json)?;
        Ok(data)
    }

    async fn delete(&self, session_id: &str) -> WizardResult<()> {
        let path = self.session_path(session_id);
        tokio::fs::remove_file(path).await?;
        Ok(())
    }
}
```

**Key Features**:
- ✅ Trait-based storage backends (file, memory, database)
- ✅ Async I/O for persistence
- ✅ Serializable session state
- ✅ Session expiration policies

### 2.6 Model Fallback (Enhancement)

**Current State**: Sequential fallback implemented

**v2 Enhancements**:

```rust
/// Enhanced fallback with parallel attempts and priority
pub struct FallbackClient {
    wizard_config: WizardConfig,
    fallback_config: FallbackConfig,
    stats: FallbackStats,
    // NEW: Parallel fallback strategy
    strategy: FallbackStrategy,
}

/// Fallback execution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// Try models sequentially (current)
    Sequential,
    /// Try top N models in parallel, return first success
    Parallel { concurrency: usize },
    /// Try models based on historical success rate
    Adaptive,
}

impl FallbackClient {
    /// NEW: Parallel fallback for low-latency scenarios
    pub async fn generate_parallel(
        &mut self,
        prompt: Prompt,
        concurrency: usize,
    ) -> WizardResult<WizardResponse> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let mut futures = FuturesUnordered::new();

        for model_config in self.fallback_config.model_chain.iter().take(concurrency) {
            let mut temp_config = self.wizard_config.clone();
            temp_config.model_config = model_config.clone();

            let prompt_clone = prompt.clone();
            futures.push(async move {
                let mut client = GenAiClient::new(temp_config).await?;
                client.generate(prompt_clone).await
            });
        }

        // Return first success
        while let Some(result) = futures.next().await {
            match result {
                Ok(response) => return Ok(response),
                Err(e) => continue, // Try next
            }
        }

        Err(WizardError::Fallback("All parallel attempts failed".to_string()))
    }
}
```

**Key Improvements**:
- ✅ Parallel fallback for reduced latency
- ✅ Adaptive model selection based on history
- ✅ Configurable fallback strategies
- ✅ Cost-aware fallback (prefer cheaper models)

---

## 3. Error Recovery & Resilience

### 3.1 Circuit Breaker Pattern

**Design**:

```rust
/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    config: CircuitBreakerConfig,
    metrics: Arc<CircuitMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Debug, Clone, Copy)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening
    pub failure_threshold: usize,
    /// Time window for counting failures
    pub window_duration: Duration,
    /// Time to wait before attempting half-open
    pub timeout: Duration,
    /// Success threshold in half-open before closing
    pub success_threshold: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            window_duration: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            success_threshold: 2,
        }
    }
}

impl CircuitBreaker {
    pub async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Open { opened_at } => {
                // Check if timeout has passed
                if opened_at.elapsed() >= self.config.timeout {
                    drop(state);
                    self.transition_to_half_open().await;
                    false // Allow half-open attempt
                } else {
                    true // Still open
                }
            }
            CircuitState::HalfOpen => false, // Allow attempt
            CircuitState::Closed => false,
        }
    }

    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::HalfOpen => {
                let successes = self.metrics.record_success();
                if successes >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    self.metrics.reset();
                }
            }
            CircuitState::Closed => {
                self.metrics.record_success();
            }
            CircuitState::Open { .. } => {
                // Ignore success in open state
            }
        }
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let failures = self.metrics.record_failure();

        match *state {
            CircuitState::Closed => {
                if failures >= self.config.failure_threshold {
                    *state = CircuitState::Open { opened_at: Instant::now() };
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open { opened_at: Instant::now() };
                self.metrics.reset();
            }
            CircuitState::Open { .. } => {
                // Already open, no action needed
            }
        }
    }
}
```

**Integration Points**:
- StreamingClient
- RetryClient
- FallbackClient
- All network-facing operations

### 3.2 Graceful Degradation

**Strategy**:

```rust
/// Unified client with graceful degradation
pub struct ResilientClient {
    client: GenAiClient,
    cache: Option<CachedClient>,
    retry: Option<RetryClient>,
    fallback: Option<FallbackClient>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ResilientClient {
    /// Generate with all resilience features
    pub async fn generate(&mut self, prompt: Prompt) -> WizardResult<WizardResponse> {
        // 1. Try cache first
        if let Some(cache) = &mut self.cache {
            if let Ok(response) = cache.generate(prompt.clone()).await {
                return Ok(response);
            }
        }

        // 2. Check circuit breaker
        if self.circuit_breaker.is_open().await {
            // Attempt fallback instead
            if let Some(fallback) = &mut self.fallback {
                return fallback.generate(prompt).await;
            }
            return Err(WizardError::CircuitOpen("Service unavailable".to_string()));
        }

        // 3. Try with retry
        if let Some(retry) = &mut self.retry {
            return retry.generate(prompt).await;
        }

        // 4. Direct client call
        self.client.generate(prompt).await
    }
}
```

---

## 4. Performance Optimization

### 4.1 Hot Path Analysis

**Identified Hot Paths**:
1. Cache key generation (called on every request)
2. Token bucket refill calculation
3. Prompt serialization for caching
4. Stream chunk parsing

**Optimization Strategy**:

```rust
/// Zero-allocation cache key generation
#[inline]
fn cache_key_fast(prompt: &Prompt) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = ahash::AHasher::default();
    prompt.text.hash(&mut hasher);
    if let Some(system) = &prompt.system {
        system.hash(&mut hasher);
    }
    hasher.finish()
}

/// Const-evaluated token bucket parameters
const REFILL_INTERVAL_NANOS: u64 = 1_000_000_000; // 1 second in nanoseconds

#[inline]
const fn tokens_per_nanos(rate: f64) -> f64 {
    rate / REFILL_INTERVAL_NANOS as f64
}
```

### 4.2 Memory Pooling

**Design**:

```rust
/// String buffer pool for response building
pub struct ResponseBufferPool {
    pool: Arc<Mutex<Vec<String>>>,
    max_size: usize,
}

impl ResponseBufferPool {
    pub fn acquire(&self) -> String {
        self.pool.lock()
            .and_then(|mut pool| pool.pop())
            .unwrap_or_else(|| String::with_capacity(4096))
    }

    pub fn release(&self, mut buffer: String) {
        buffer.clear();
        if let Ok(mut pool) = self.pool.lock() {
            if pool.len() < self.max_size {
                pool.push(buffer);
            }
        }
    }
}
```

**Usage in Streaming**:

```rust
pub async fn collect_stream(&self, stream: impl Stream<Item = StreamChunk>) -> String {
    let mut buffer = self.buffer_pool.acquire();

    while let Some(chunk) = stream.next().await {
        buffer.push_str(&chunk.text);
    }

    let result = buffer.clone();
    self.buffer_pool.release(buffer);
    result
}
```

### 4.3 Async Concurrency Optimization

**Parallel Request Batching**:

```rust
/// Batch multiple prompts for parallel processing
pub async fn generate_batch(
    &mut self,
    prompts: Vec<Prompt>,
) -> Vec<WizardResult<WizardResponse>> {
    use futures::stream::{FuturesUnordered, StreamExt};

    let futures: FuturesUnordered<_> = prompts
        .into_iter()
        .map(|prompt| self.generate(prompt))
        .collect();

    futures.collect().await
}
```

### 4.4 Performance SLOs

**Target Metrics**:
- Cache hit: <1ms p99
- Cache miss: <100ms p99
- Streaming first token: <200ms p99
- Retry overhead: <5ms per attempt
- Memory usage: <50MB for 1000-entry cache

---

## 5. Module Reorganization

### 5.1 Current Structure

```
src/wizard/
├── mod.rs              (58 lines)
├── error.rs            (121 lines)
├── types.rs            (284 lines)
├── config.rs           (488 lines)
├── client.rs           (272 lines)
├── builder.rs          (192 lines)
├── genai.rs            (254 lines)
├── prompt.rs           (~150 lines, estimated)
├── session.rs          (~100 lines, estimated)
├── cli.rs              (~200 lines, estimated)
├── interactive.rs      (~250 lines, estimated)
├── streaming.rs        (339 lines)
├── cache.rs            (373 lines)
├── rate_limit.rs       (349 lines)
├── retry.rs            (409 lines)
└── fallback.rs         (368 lines)
```

### 5.2 Proposed v2 Structure

**Option A: Flat Structure (Recommended for <5000 lines)**

Keep current flat structure, add new modules:

```
src/wizard/
├── mod.rs              # Public API exports
├── error.rs            # Complete error types (v2 additions)
├── types.rs            # Core types (Prompt, Response, Message)
├── config.rs           # Configuration types
├── client.rs           # Base GenAiClient
├── streaming.rs        # Streaming client
├── cache.rs            # Caching wrapper
├── rate_limit.rs       # Rate limiting wrapper
├── retry.rs            # Retry logic wrapper
├── fallback.rs         # Fallback chain
├── resilient.rs        # NEW: Unified resilient client
├── circuit_breaker.rs  # NEW: Circuit breaker implementation
├── session.rs          # Session management
├── persistence.rs      # NEW: Session persistence
├── builder.rs          # Builder patterns
├── cli.rs              # CLI integration
├── interactive.rs      # Interactive REPL
└── metrics.rs          # NEW: Performance metrics
```

**Option B: Nested Structure (For >10000 lines)**

```
src/wizard/
├── mod.rs
├── core/               # Core functionality
│   ├── mod.rs
│   ├── error.rs
│   ├── types.rs
│   └── config.rs
├── clients/            # Client implementations
│   ├── mod.rs
│   ├── base.rs         # GenAiClient
│   ├── streaming.rs
│   ├── cached.rs
│   └── resilient.rs
├── middleware/         # Request middleware
│   ├── mod.rs
│   ├── rate_limit.rs
│   ├── retry.rs
│   └── fallback.rs
├── resilience/         # Resilience patterns
│   ├── mod.rs
│   └── circuit_breaker.rs
├── session/            # Session management
│   ├── mod.rs
│   ├── session.rs
│   └── persistence.rs
└── ui/                 # User interfaces
    ├── mod.rs
    ├── cli.rs
    └── interactive.rs
```

**Recommendation**: Use **Option A** (flat structure) for v2. The codebase is ~4,700 lines and flat structure maintains simplicity while providing clear organization.

### 5.3 Module Dependencies (DAG)

```
error.rs
  ↓
types.rs, config.rs
  ↓
client.rs
  ↓
streaming.rs, cache.rs, rate_limit.rs, retry.rs, fallback.rs
  ↓
circuit_breaker.rs
  ↓
resilient.rs
  ↓
session.rs, persistence.rs
  ↓
builder.rs, cli.rs, interactive.rs
```

---

## 6. Backwards Compatibility Strategy

### 6.1 API Stability Guarantees

**Breaking Changes** (require semver major bump):
- Error enum additions (technically breaking)
- Required method parameter changes
- Public trait changes

**Non-Breaking Changes** (semver minor):
- New modules (resilient.rs, circuit_breaker.rs)
- New optional features
- Default impl additions

### 6.2 Migration Path

**Phase 1: Add v2 alongside v1** (Deprecation Phase)

```rust
// Old API (deprecated but functional)
#[deprecated(since = "2.0.0", note = "Use GenAiClient::new instead")]
pub async fn from_env(model: String) -> Result<GenAiClient> {
    let config = WizardConfig::from_env()?;
    GenAiClient::new(config).await
}

// New API (v2)
pub async fn new(config: WizardConfig) -> WizardResult<Self> {
    // v2 implementation
}
```

**Phase 2: Compatibility Shims**

```rust
/// Legacy error conversion
impl From<WizardError> for String {
    fn from(err: WizardError) -> String {
        err.to_string()
    }
}

/// Legacy result type alias (for gradual migration)
#[deprecated(since = "2.0.0", note = "Use WizardResult instead")]
pub type Result<T> = std::result::Result<T, WizardError>;
```

### 6.3 Feature Flags

```toml
[features]
default = ["resilience", "caching"]

# Core features
resilience = ["circuit-breaker", "retry", "fallback"]
caching = ["lru", "ahash"]
streaming = []
rate-limiting = []

# v2 features
session-persistence = ["tokio/fs", "chrono"]
metrics = ["prometheus"]

# Backwards compatibility
v1-compat = []  # Enable v1 deprecated APIs
```

### 6.4 Migration Guide

**Example Migration**:

```rust
// v1 (Old)
let client = GenAiClient::from_env("gpt-4".to_string())?;
let response = client.generate(&prompt).await?;

// v2 (New - Recommended)
let config = WizardConfig::from_env()?;
let mut client = GenAiClient::new(config).await?;
let response = client.generate(prompt).await?;

// v2 (With Resilience)
let config = WizardConfig::from_env()?;
let mut client = ResilientClient::builder()
    .with_config(config)
    .with_retry(RetryConfig::default())
    .with_fallback(fallback_chain)
    .with_circuit_breaker(CircuitBreakerConfig::default())
    .build()
    .await?;
let response = client.generate(prompt).await?;
```

---

## 7. Testing Strategy

### 7.1 Test Coverage Goals

**Current**: 88 tests
**Target v2**: 200+ tests, 85%+ coverage

**Breakdown**:
- Unit tests: 120 tests (error handling, validation, algorithms)
- Integration tests: 50 tests (client interactions, API mocking)
- Property-based tests: 20 tests (fuzzing, edge cases)
- Performance tests: 10 benchmarks (hot paths, SLO validation)

### 7.2 Chicago TDD Compliance

All tests must follow AAA (Arrange-Act-Assert) pattern with behavior verification:

```rust
#[tokio::test]
async fn test_circuit_breaker_opens_after_threshold() {
    // Arrange
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);

    // Act - trigger failures
    for _ in 0..3 {
        breaker.record_failure().await;
    }

    // Assert - verify circuit is open
    assert!(breaker.is_open().await);
}
```

### 7.3 Error Path Testing

**Every Error Variant Must Be Tested**:

```rust
#[tokio::test]
async fn test_all_error_variants() {
    // Test Auth error
    let auth_err = WizardError::Auth("invalid key".to_string());
    assert!(matches!(auth_err, WizardError::Auth(_)));

    // Test TokenLimit error
    let limit_err = WizardError::TokenLimit { requested: 10000, max: 8192 };
    assert!(matches!(limit_err, WizardError::TokenLimit { .. }));

    // ... test all variants
}
```

### 7.4 Andon Signal Integration

**Pre-commit Hook**:

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running Andon signal checks..."

# Check for compiler errors
cargo make check || exit 1

# Check for lint violations
cargo make lint || exit 1

# Run unit tests
cargo make test-unit || exit 1

echo "All Andon signals clear ✓"
```

---

## 8. Implementation Roadmap

### Phase 1: Foundation (Week 1)

**Sprint 1: Error Infrastructure** (Days 1-2)
- [ ] Add missing error variants to error.rs
- [ ] Implement Display/Error traits
- [ ] Add From<T> conversions
- [ ] Test all error paths

**Sprint 2: Critical Violations** (Days 3-5)
- [ ] Fix client.rs cache initialization
- [ ] Fix streaming.rs unwrap calls
- [ ] Fix retry.rs error propagation
- [ ] Fix config.rs validation
- [ ] Run clippy with --deny warnings

### Phase 2: Resilience Features (Week 2)

**Sprint 3: Circuit Breaker** (Days 6-8)
- [ ] Implement CircuitBreaker
- [ ] Integrate with RetryClient
- [ ] Integrate with StreamingClient
- [ ] Add circuit breaker tests

**Sprint 4: Enhanced Features** (Days 9-10)
- [ ] Add session persistence
- [ ] Enhance caching (warming, invalidation)
- [ ] Add parallel fallback
- [ ] Adaptive rate limiting

### Phase 3: Performance & Polish (Week 3)

**Sprint 5: Performance** (Days 11-13)
- [ ] Implement buffer pooling
- [ ] Optimize hot paths
- [ ] Add performance benchmarks
- [ ] Validate SLOs

**Sprint 6: Documentation & Testing** (Days 14-15)
- [ ] Complete integration tests
- [ ] Write migration guide
- [ ] Update API documentation
- [ ] Final Andon signal validation

---

## 9. Success Metrics

### 9.1 Code Quality

- ✅ Zero lint violations (unwrap/expect/panic/unsafe/todo)
- ✅ 85%+ test coverage
- ✅ All error types have Display + Error implementations
- ✅ Clippy passes with --deny warnings

### 9.2 Performance

- ✅ Cache hit latency: <1ms p99
- ✅ Streaming first token: <200ms p99
- ✅ Memory usage: <50MB per 1000 cache entries
- ✅ Zero allocations in hot paths

### 9.3 Reliability

- ✅ Circuit breaker prevents cascading failures
- ✅ Retry logic handles transient errors
- ✅ Fallback succeeds >95% when primary fails
- ✅ Graceful degradation under load

### 9.4 Developer Experience

- ✅ Clear error messages with context
- ✅ Type-safe configuration builders
- ✅ Comprehensive examples
- ✅ Migration guide <500 lines

---

## 10. Architecture Decision Records (ADRs)

### ADR-001: Flat Module Structure

**Context**: Need to organize ~4,700 lines of wizard code
**Decision**: Use flat module structure (Option A) rather than nested
**Rationale**: Codebase size doesn't justify nested hierarchy; flat structure is more discoverable
**Consequences**: If code exceeds 10,000 lines, consider refactoring to nested structure

### ADR-002: Circuit Breaker Integration

**Context**: Need to prevent cascading failures in AI service calls
**Decision**: Integrate circuit breaker into retry/streaming/fallback clients
**Rationale**: Circuit breaker is essential for production resilience; prevents waste of resources
**Consequences**: Adds ~300 lines of code; requires testing across all clients

### ADR-003: Error Enum Expansion

**Context**: 9 error variants missing, causing unwrap/expect violations
**Decision**: Add all missing variants to WizardError enum
**Rationale**: Complete error coverage enables proper error propagation
**Consequences**: Technically breaking change; requires semver major bump

### ADR-004: Backwards Compatibility via Deprecation

**Context**: Need to introduce v2 features without breaking existing code
**Decision**: Deprecate v1 APIs rather than remove; provide migration period
**Rationale**: Gradual migration reduces risk for existing users
**Consequences**: Maintain deprecated code for one major version cycle

### ADR-005: Performance Optimization Strategy

**Context**: Need to meet <100ms p99 latency SLO
**Decision**: Focus on hot paths (cache key gen, token bucket, streaming)
**Rationale**: 80/20 rule - optimize the 20% that matters
**Consequences**: Requires benchmarking infrastructure and profiling

---

## 11. Risk Assessment

### High-Priority Risks

1. **Error Migration Breaking Changes**
   - **Risk**: Adding error variants breaks pattern matching
   - **Mitigation**: Use non_exhaustive attribute; provide compatibility shim
   - **Timeline**: Address in Sprint 1

2. **Performance Regression**
   - **Risk**: New features (circuit breaker, metrics) add latency
   - **Mitigation**: Benchmark all changes; optimize hot paths
   - **Timeline**: Continuous through Phase 3

3. **Test Coverage Gaps**
   - **Risk**: Complex async code has hidden edge cases
   - **Mitigation**: Property-based testing; chaos engineering
   - **Timeline**: Address in Sprint 6

### Medium-Priority Risks

1. **API Complexity Growth**
   - **Risk**: Too many client types (Cached, RateLimited, Retry, etc.)
   - **Mitigation**: Provide ResilientClient as unified interface
   - **Timeline**: Address in Sprint 4

2. **Documentation Debt**
   - **Risk**: New features not documented for migration
   - **Mitigation**: Write docs alongside implementation
   - **Timeline**: Address in Sprint 6

---

## 12. Next Steps

**Immediate Actions** (This Week):
1. Review and approve architecture design
2. Create GitHub issues for each sprint
3. Set up performance benchmarking infrastructure
4. Begin Sprint 1: Error Infrastructure

**Success Criteria for v2 Launch**:
- All 23 lint violations resolved
- 200+ tests passing
- Performance SLOs met
- Migration guide published
- Zero Andon signals on main branch

---

## Appendix A: Error Variant Matrix

| Error Variant | Usage Count | Critical Path | Resolution Priority |
|--------------|-------------|---------------|-------------------|
| Auth | ~5 | Yes (client init) | P0 |
| TokenLimit | ~3 | Yes (validation) | P0 |
| Request | ~15 | Yes (network) | P0 |
| Parse | ~8 | Yes (streaming) | P0 |
| Config | ~12 | Yes (initialization) | P0 |
| RateLimit | ~6 | No (middleware) | P1 |
| Timeout | ~4 | No (retry) | P1 |
| Network | ~5 | Yes (connection) | P0 |
| Fallback | ~3 | No (fallback) | P1 |

---

## Appendix B: Performance Benchmark Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_cache_key_generation(c: &mut Criterion) {
    let prompt = Prompt::new("What is Rust?")
        .with_system("You are a helpful assistant");

    c.bench_function("cache_key_fast", |b| {
        b.iter(|| {
            cache_key_fast(black_box(&prompt))
        })
    });
}

criterion_group!(benches, bench_cache_key_generation);
criterion_main!(benches);
```

---

**Document Version**: 1.0
**Last Updated**: 2026-01-09
**Next Review**: 2026-01-16 (Sprint 2 completion)
