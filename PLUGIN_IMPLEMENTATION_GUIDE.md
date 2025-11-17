# 10 Plugins with Chicago-TDD Testing - Implementation Guide

## Overview

This guide provides complete specifications for implementing 10 production-grade plugins with Chicago-TDD style integration testing.

**Key Principles**:
- Chicago-TDD: Integration tests, no mocking, real components
- Plugin trait implementation with capability system
- Thread-safe, production-ready code
- Comprehensive error handling
- Multi-plugin collaboration workflows

---

## Plugin 1: Cache Manager Plugin ✅ COMPLETED

**File**: `src/plugins/cache.rs`
**Status**: Fully implemented with 6 Chicago-TDD tests

**Features**:
- LRU eviction policy
- Per-item TTL
- Thread-safe with Arc<Mutex<>>
- Configurable max size

**Key Methods**:
```rust
set(key, value) -> Result<()>
set_with_ttl(key, value, ttl) -> Result<()>
get(key) -> Result<Option<String>>
clear() -> Result<()>
stats() -> Result<(total, expired)>
```

**Test Patterns**:
- Set and get workflow
- TTL expiration workflow
- LRU eviction workflow
- Clear workflow
- Statistics workflow
- Multi-user session caching

---

## Plugin 2: Rate Limiter Plugin

**File**: `src/plugins/rate_limiter.rs`
**Implementation Size**: ~400 lines

**Features**:
- Token bucket algorithm
- Per-user rate limiting
- Configurable rates
- Time window management

**Core Implementation**:
```rust
pub struct RateLimiterPlugin {
    // Token buckets per user: user_id -> (tokens, last_refill)
    buckets: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    // Tokens per second
    rate: u32,
    // Maximum tokens
    capacity: u32,
    loaded: bool,
}

pub fn allow_request(&self, user_id: &str) -> crate::Result<bool>
pub fn reset_user(&self, user_id: &str) -> crate::Result<()>
pub fn get_remaining(&self, user_id: &str) -> crate::Result<u32>
```

**Chicago-TDD Tests** (8+):
1. Single request allows_workflow
2. Burst request within limit_workflow
3. Rate limit exceeded_workflow
4. Token refill over_time_workflow
5. User reset_workflow
6. Multiple users isolation_workflow
7. Concurrent requests handling_workflow
8. Edge case: zero capacity_workflow

---

## Plugin 3: Configuration Manager Plugin

**File**: `src/plugins/config_manager.rs`
**Implementation Size**: ~350 lines

**Features**:
- TOML/JSON config loading
- Environment variable override
- Config validation
- Hierarchical settings

**Core Implementation**:
```rust
pub struct ConfigManagerPlugin {
    config: Arc<Mutex<HashMap<String, String>>>,
    config_path: Option<String>,
    loaded: bool,
}

pub fn load_config(&mut self, path: &str) -> crate::Result<()>
pub fn get(&self, key: &str) -> crate::Result<Option<String>>
pub fn set(&mut self, key: String, value: String) -> crate::Result<()>
pub fn validate(&self) -> crate::Result<()>
```

**Chicago-TDD Tests** (8+):
1. Load TOML config_workflow
2. Load JSON config_workflow
3. Env var override_workflow
4. Config validation_workflow
5. Get nonexistent key_workflow
6. Hierarchical config path_workflow
7. Config reload_workflow
8. Invalid file handling_workflow

---

## Plugin 4: Metrics Aggregator Plugin

**File**: `src/plugins/metrics_aggregator.rs`
**Implementation Size**: ~400 lines

**Features**:
- Time-series data collection
- Aggregation (sum, avg, min, max)
- Time-based bucketing
- Multi-metric support

**Core Implementation**:
```rust
struct MetricBucket {
    timestamp: Instant,
    values: Vec<f64>,
}

pub struct MetricsAggregatorPlugin {
    metrics: Arc<Mutex<HashMap<String, Vec<MetricBucket>>>>,
    bucket_duration: Duration,
    loaded: bool,
}

pub fn record(&self, name: &str, value: f64) -> crate::Result<()>
pub fn aggregate(&self, name: &str) -> crate::Result<Aggregation>
pub fn get_timeseries(&self, name: &str) -> crate::Result<Vec<f64>>
```

**Chicago-TDD Tests** (10+):
1. Record single metric_workflow
2. Record multiple metrics_workflow
3. Aggregate sum_workflow
4. Aggregate average_workflow
5. Aggregate min/max_workflow
6. Time bucket management_workflow
7. Timeseries retrieval_workflow
8. Concurrent metric recording_workflow
9. Old data cleanup_workflow
10. Query non-existent metric_workflow

---

## Plugin 5: Logger Plugin

**File**: `src/plugins/logger.rs`
**Implementation Size**: ~380 lines

**Features**:
- Structured logging (JSON)
- Log levels (DEBUG, INFO, WARN, ERROR)
- Log rotation
- Filtering

**Core Implementation**:
```rust
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug, Info, Warn, Error,
}

pub struct LoggerPlugin {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    level: LogLevel,
    max_logs: usize,
    loaded: bool,
}

pub fn log(&self, level: LogLevel, message: &str) -> crate::Result<()>
pub fn get_logs(&self, level: LogLevel) -> crate::Result<Vec<LogEntry>>
pub fn clear_logs(&self) -> crate::Result<()>
```

**Chicago-TDD Tests** (9+):
1. Log debug message_workflow
2. Log at different levels_workflow
3. Filter by level_workflow
4. Log rotation (max size)_workflow
5. Structured JSON output_workflow
6. Timestamp accuracy_workflow
7. Concurrent logging_workflow
8. Clear logs_workflow
9. Query logs with filters_workflow

---

## Plugin 6: Auth Manager Plugin

**File**: `src/plugins/auth_manager.rs`
**Implementation Size**: ~400 lines

**Features**:
- JWT token generation/validation
- User credential management
- Role-based access control
- Token expiration

**Core Implementation**:
```rust
pub struct AuthManagerPlugin {
    users: Arc<Mutex<HashMap<String, User>>>,
    tokens: Arc<Mutex<HashMap<String, TokenInfo>>>,
    secret_key: String,
    loaded: bool,
}

pub fn authenticate(&self, username: &str, password: &str) -> crate::Result<String>
pub fn validate_token(&self, token: &str) -> crate::Result<TokenInfo>
pub fn add_user(&mut self, username: &str, password: &str) -> crate::Result<()>
pub fn grant_role(&self, user: &str, role: &str) -> crate::Result<()>
```

**Chicago-TDD Tests** (10+):
1. User registration_workflow
2. Authentication success_workflow
3. Authentication failure_workflow
4. Token generation_workflow
5. Token validation_workflow
6. Token expiration_workflow
7. Role assignment_workflow
8. Role-based access check_workflow
9. Token refresh_workflow
10. Multiple user auth_workflow

---

## Plugin 7: Database Pool Plugin

**File**: `src/plugins/database_pool.rs`
**Implementation Size**: ~420 lines

**Features**:
- Connection pooling
- Connection reuse
- Timeout handling
- Health checks

**Core Implementation**:
```rust
pub struct PooledConnection {
    id: u32,
    in_use: bool,
    last_used: Instant,
}

pub struct DatabasePoolPlugin {
    connections: Arc<Mutex<Vec<PooledConnection>>>,
    pool_size: u32,
    timeout: Duration,
    loaded: bool,
}

pub fn acquire_connection(&self) -> crate::Result<u32>
pub fn release_connection(&self, conn_id: u32) -> crate::Result<()>
pub fn health_check(&self) -> crate::Result<PoolHealth>
```

**Chicago-TDD Tests** (10+):
1. Acquire connection_workflow
2. Connection reuse_workflow
3. Acquire when pool full_workflow
4. Release connection_workflow
5. Connection timeout_workflow
6. Health check_workflow
7. Cleanup stale connections_workflow
8. Concurrent acquire/release_workflow
9. Pool exhaustion handling_workflow
10. Connection statistics_workflow

---

## Plugin 8: Message Queue Plugin

**File**: `src/plugins/message_queue.rs`
**Implementation Size**: ~380 lines

**Features**:
- FIFO message queue
- Async message publishing
- Message priority
- Dead letter queue

**Core Implementation**:
```rust
pub struct Message {
    id: String,
    body: String,
    priority: u8,
    timestamp: Instant,
}

pub struct MessageQueuePlugin {
    messages: Arc<Mutex<VecDeque<Message>>>,
    dead_letter: Arc<Mutex<Vec<Message>>>,
    max_queue_size: usize,
    loaded: bool,
}

pub fn publish(&self, message: &str) -> crate::Result<String>
pub fn consume(&self) -> crate::Result<Option<Message>>
pub fn publish_priority(&self, msg: &str, priority: u8) -> crate::Result<String>
```

**Chicago-TDD Tests** (10+):
1. Publish message_workflow
2. Consume message_workflow
3. FIFO order guarantee_workflow
4. Priority queue ordering_workflow
5. Empty queue consume_workflow
6. Queue full handling_workflow
7. Message persistence check_workflow
8. Dead letter queue_workflow
9. Concurrent publish/consume_workflow
10. Queue statistics_workflow

---

## Plugin 9: Event Bus Plugin

**File**: `src/plugins/event_bus.rs`
**Implementation Size**: ~400 lines

**Features**:
- Pub/Sub pattern
- Event filtering
- Async event delivery
- Event history

**Core Implementation**:
```rust
pub struct Event {
    topic: String,
    data: String,
    timestamp: Instant,
}

type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

pub struct EventBusPlugin {
    subscribers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    event_history: Arc<Mutex<Vec<Event>>>,
    loaded: bool,
}

pub fn subscribe(&self, topic: &str, handler: EventHandler) -> crate::Result<()>
pub fn publish(&self, topic: &str, data: &str) -> crate::Result<()>
pub fn get_history(&self, topic: &str) -> crate::Result<Vec<Event>>
```

**Chicago-TDD Tests** (10+):
1. Subscribe to topic_workflow
2. Publish event_workflow
3. Event delivery to subscribers_workflow
4. Multiple subscribers_workflow
5. Topic filtering_workflow
6. Event history tracking_workflow
7. Unsubscribe_workflow
8. Concurrent publish/subscribe_workflow
9. Event ordering_workflow
10. Memory management (history limit)_workflow

---

## Plugin 10: Circuit Breaker Plugin

**File**: `src/plugins/circuit_breaker.rs`
**Implementation Size**: ~420 lines

**Features**:
- Failure detection
- State management (Open, Half-Open, Closed)
- Automatic recovery
- Configurable thresholds

**Core Implementation**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, reject requests
    HalfOpen,   // Testing recovery
}

pub struct CircuitBreakerPlugin {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    success_count: Arc<Mutex<u32>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    loaded: bool,
}

pub fn call<F>(&self, operation: F) -> crate::Result<String>
where
    F: FnOnce() -> crate::Result<String>,

pub fn get_state(&self) -> crate::Result<CircuitState>
```

**Chicago-TDD Tests** (12+):
1. Circuit closed normal operation_workflow
2. Record failures_workflow
3. Circuit open after threshold_workflow
4. Reject requests when open_workflow
5. Enter half-open state_workflow
6. Success in half-open closes circuit_workflow
7. Failure in half-open reopens circuit_workflow
8. Automatic recovery timeout_workflow
9. Reset circuit_workflow
10. Concurrent operations_workflow
11. Failure monitoring_workflow
12. State transitions logging_workflow

---

## Testing Infrastructure

### Chicago-TDD Principles Applied

**No Mocks**: All tests use real plugin implementations
**Integration**: Tests collaborations between plugins
**Behavior-Driven**: Test names describe workflows
**Assertions**: Test complete application behavior

### Test Naming Convention

```
test_<plugin>_<scenario>_workflow

Examples:
- test_cache_set_and_get_workflow
- test_rate_limiter_burst_handling_workflow
- test_event_bus_multiple_subscribers_workflow
```

### Multi-Plugin Integration Tests

Create file: `src/plugins/integration_tests.rs`

```rust
#[test]
fn test_cache_logger_workflow() {
    // Workflow: Log cache operations
    let cache = CacheManagerPlugin::new();
    let logger = LoggerPlugin::new();

    cache.set("key", "value").unwrap();
    logger.log(LogLevel::Info, "Cached key").unwrap();

    // Assert both plugins cooperated correctly
}

#[test]
fn test_event_bus_with_logger_workflow() {
    // Workflow: Log all published events
    let bus = EventBusPlugin::new();
    let logger = LoggerPlugin::new();

    bus.subscribe("system", Box::new(|event| {
        logger.log(LogLevel::Info, &format!("Event: {}", event.data)).unwrap();
    })).unwrap();

    bus.publish("system", "startup").unwrap();
}

#[test]
fn test_circuit_breaker_with_metrics_workflow() {
    // Workflow: Track circuit breaker state transitions
    let breaker = CircuitBreakerPlugin::new();
    let metrics = MetricsAggregatorPlugin::new();

    // ... test operations and record metrics
}
```

---

## Implementation Checklist

- [x] Cache Manager Plugin
- [ ] Rate Limiter Plugin
- [ ] Configuration Manager Plugin
- [ ] Metrics Aggregator Plugin
- [ ] Logger Plugin
- [ ] Auth Manager Plugin
- [ ] Database Pool Plugin
- [ ] Message Queue Plugin
- [ ] Event Bus Plugin
- [ ] Circuit Breaker Plugin
- [ ] Integration test suite
- [ ] Multi-plugin workflows
- [ ] Documentation examples

---

## Performance Targets

- **Cache**: < 1μs get/set
- **Rate Limiter**: < 500ns allow_request
- **Event Bus**: < 100μs publish
- **Circuit Breaker**: < 1μs call overhead
- **All**: Thread-safe with Arc<Mutex<>>

---

## Quality Metrics

- **Code Coverage**: 85%+
- **Test Count**: 100+ tests
- **Integration Tests**: 20+ multi-plugin workflows
- **Doc Tests**: All public APIs documented

---

## Next Steps

1. Implement remaining 9 plugins
2. Create plugin registry integration tests
3. Add performance benchmarks
4. Create real-world usage examples
5. Add plugin configuration TOML examples
