# How-To: Domain Separation Patterns

**Problem:** How do I properly separate domain logic from CLI code in production?

## Overview

Domain separation is the practice of keeping business logic independent from CLI concerns. This guide shows production patterns for achieving clean separation in clap-noun-verb applications.

**Goal:** CLI layer handles I/O and validation; domain layer contains pure business logic.

## Pattern 1: Service Layer

**When to use:** Complex business operations with multiple steps, state management, or dependencies.

**Example: Service Management**

```rust
// ✅ GOOD: Domain layer - Pure business logic
pub struct ServiceManager {
    services: HashMap<String, Service>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self { services: HashMap::new() }
    }

    pub fn get_status(&self) -> ServiceStatus {
        ServiceStatus {
            services: self.services.values()
                .map(|s| ServiceInfo {
                    name: s.name.clone(),
                    state: s.state.to_string(),
                    port: s.port,
                })
                .collect()
        }
    }

    pub fn restart_service(&mut self, name: &str) -> Result<RestartResult> {
        let service = self.services.get_mut(name)
            .ok_or_else(|| Error::ServiceNotFound(name.to_string()))?;

        service.restart()?;
        Ok(RestartResult { service: name.to_string(), success: true })
    }
}

#[derive(Serialize, Debug)]
pub struct ServiceStatus {
    services: Vec<ServiceInfo>,
}

#[derive(Serialize, Debug)]
pub struct ServiceInfo {
    name: String,
    state: String,
    port: u16,
}

#[derive(Serialize, Debug)]
pub struct RestartResult {
    service: String,
    success: bool,
}

// ✅ GOOD: CLI layer - Thin wrapper handling I/O
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Show status of all services
#[verb]
fn show_status() -> Result<ServiceStatus> {
    let manager = ServiceManager::new();
    Ok(manager.get_status())
}

/// Restart a service
#[verb]
fn restart(service: String) -> Result<RestartResult> {
    let mut manager = ServiceManager::new();
    Ok(manager.restart_service(&service)?)
}
```

**Why this works:**
- Domain layer (`ServiceManager`) has zero CLI knowledge
- CLI functions are thin wrappers (2-3 lines)
- Domain logic is testable without CLI machinery
- Easy to reuse in GUI, web API, etc.

## Pattern 2: Pure Functions

**When to use:** Stateless transformations, calculations, or data processing.

**Example: Tax Calculation**

```rust
// ✅ GOOD: Domain layer - Pure function
use rust_decimal::Decimal;

pub fn calculate_tax(amount: Decimal, rate: Decimal) -> Decimal {
    amount * rate
}

pub fn calculate_total_with_tax(
    subtotal: Decimal,
    tax_rate: Decimal,
    discount: Option<Decimal>,
) -> TaxResult {
    let discount_amount = discount.unwrap_or(Decimal::ZERO);
    let taxable = subtotal - discount_amount;
    let tax = calculate_tax(taxable, tax_rate);

    TaxResult {
        subtotal,
        discount: discount_amount,
        tax,
        total: taxable + tax,
    }
}

#[derive(Serialize, Debug)]
pub struct TaxResult {
    subtotal: Decimal,
    discount: Decimal,
    tax: Decimal,
    total: Decimal,
}

// ✅ GOOD: CLI layer - Input parsing only
#[verb]
fn calculate(
    subtotal: Decimal,
    tax_rate: Decimal,
    #[arg(long)] discount: Option<Decimal>,
) -> Result<TaxResult> {
    Ok(calculate_total_with_tax(subtotal, tax_rate, discount))
}
```

**Why this works:**
- Pure functions have predictable outputs
- No side effects, no hidden state
- Trivial to test (no mocks needed)
- Can be used in const contexts
- Zero allocation overhead

## Pattern 3: Generic Over I/O

**When to use:** File processing, network I/O, or any operation involving streams.

**Example: Data Processing**

```rust
use std::io::{BufRead, Write};

// ✅ GOOD: Domain layer - Generic over I/O sources
pub fn process_stream<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
) -> Result<ProcessStats> {
    let mut stats = ProcessStats::default();
    let mut buffer = String::new();

    while reader.read_line(&mut buffer)? > 0 {
        let processed = transform_line(&buffer)?;
        writeln!(writer, "{}", processed)?;
        stats.lines_processed += 1;
        buffer.clear();
    }

    Ok(stats)
}

fn transform_line(line: &str) -> Result<String> {
    // Pure transformation logic
    Ok(line.trim().to_uppercase())
}

#[derive(Serialize, Debug, Default)]
pub struct ProcessStats {
    lines_processed: usize,
}

// ✅ GOOD: CLI layer - Handles file I/O
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[verb]
fn process(input: PathBuf, output: PathBuf) -> Result<ProcessStats> {
    let input_file = File::open(input)
        .map_err(|e| Error::FileOpen(e))?;
    let output_file = File::create(output)
        .map_err(|e| Error::FileCreate(e))?;

    let reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);

    process_stream(reader, writer)
}
```

**Why this works:**
- Domain layer knows nothing about files
- Can test with in-memory buffers (fast)
- Works with stdin/stdout, sockets, etc.
- Composable with other stream processors
- Type system enforces separation

**Testing example:**

```rust
#[test]
fn test_process_stream() {
    let input = b"hello\nworld\n";
    let mut output = Vec::new();

    let stats = process_stream(
        &input[..],
        &mut output,
    ).unwrap();

    assert_eq!(stats.lines_processed, 2);
    assert_eq!(output, b"HELLO\nWORLD\n");
}
```

## Pattern 4: State Machines

**When to use:** Complex state transitions, protocol implementations, or workflow engines.

**Example: Connection State Machine**

```rust
// ✅ GOOD: Domain layer - Type-safe state machine
pub enum ConnectionState {
    Idle,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { reason: String },
}

impl ConnectionState {
    pub fn handle_event(self, event: ConnectionEvent) -> Self {
        match (self, event) {
            (Self::Idle, ConnectionEvent::Connect) => {
                Self::Connecting { attempt: 1 }
            }
            (Self::Connecting { attempt }, ConnectionEvent::Success(session_id)) => {
                Self::Connected { session_id }
            }
            (Self::Connecting { attempt }, ConnectionEvent::Retry) if attempt < 3 => {
                Self::Connecting { attempt: attempt + 1 }
            }
            (Self::Connecting { .. }, ConnectionEvent::Fail(reason)) => {
                Self::Failed { reason }
            }
            (Self::Connected { .. }, ConnectionEvent::Disconnect) => {
                Self::Idle
            }
            // Invalid transitions return current state
            (state, _) => state,
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. })
    }
}

pub enum ConnectionEvent {
    Connect,
    Success(String),
    Fail(String),
    Retry,
    Disconnect,
}

#[derive(Serialize)]
pub struct ConnectionStatus {
    state: String,
    details: Option<String>,
}

impl From<&ConnectionState> for ConnectionStatus {
    fn from(state: &ConnectionState) -> Self {
        match state {
            ConnectionState::Idle => ConnectionStatus {
                state: "idle".to_string(),
                details: None,
            },
            ConnectionState::Connecting { attempt } => ConnectionStatus {
                state: "connecting".to_string(),
                details: Some(format!("attempt {}", attempt)),
            },
            ConnectionState::Connected { session_id } => ConnectionStatus {
                state: "connected".to_string(),
                details: Some(session_id.clone()),
            },
            ConnectionState::Failed { reason } => ConnectionStatus {
                state: "failed".to_string(),
                details: Some(reason.clone()),
            },
        }
    }
}

// ✅ GOOD: CLI layer - Drives state machine
#[verb]
fn connect(endpoint: String) -> Result<ConnectionStatus> {
    let mut state = ConnectionState::Idle;
    state = state.handle_event(ConnectionEvent::Connect);

    // Simulate connection attempt
    state = if endpoint.starts_with("valid") {
        state.handle_event(ConnectionEvent::Success("session-123".to_string()))
    } else {
        state.handle_event(ConnectionEvent::Fail("invalid endpoint".to_string()))
    };

    Ok(ConnectionStatus::from(&state))
}

#[verb]
fn disconnect() -> Result<ConnectionStatus> {
    let mut state = ConnectionState::Connected {
        session_id: "session-123".to_string()
    };
    state = state.handle_event(ConnectionEvent::Disconnect);
    Ok(ConnectionStatus::from(&state))
}
```

**Why this works:**
- State transitions are type-safe
- Impossible states are unrepresentable
- Pure state logic (no side effects)
- Easy to test all transitions
- Can visualize as state diagram

## Pattern 5: Builder Pattern with Validation

**When to use:** Complex object construction with validation rules.

**Example: Configuration Builder**

```rust
// ✅ GOOD: Domain layer - Builder with validation
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            timeout: None,
        }
    }

    pub fn host(mut self, host: String) -> Result<Self> {
        if host.is_empty() {
            return Err(Error::InvalidHost("host cannot be empty".to_string()));
        }
        self.host = Some(host);
        Ok(self)
    }

    pub fn port(mut self, port: u16) -> Result<Self> {
        if port < 1024 {
            return Err(Error::InvalidPort("port must be >= 1024".to_string()));
        }
        self.port = Some(port);
        Ok(self)
    }

    pub fn timeout(mut self, seconds: u64) -> Result<Self> {
        if seconds == 0 || seconds > 300 {
            return Err(Error::InvalidTimeout("timeout must be 1-300 seconds".to_string()));
        }
        self.timeout = Some(Duration::from_secs(seconds));
        Ok(self)
    }

    pub fn build(self) -> Result<Config> {
        Ok(Config {
            host: self.host.ok_or(Error::MissingHost)?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

pub struct Config {
    host: String,
    port: u16,
    timeout: Duration,
}

// ✅ GOOD: CLI layer - Collects inputs and delegates
#[verb]
fn create(
    host: String,
    #[arg(long, default_value = "8080")] port: u16,
    #[arg(long, default_value = "30")] timeout: u64,
) -> Result<Config> {
    ConfigBuilder::new()
        .host(host)?
        .port(port)?
        .timeout(timeout)?
        .build()
}
```

## Anti-Patterns to Avoid

### ❌ Domain Logic in CLI

```rust
// ❌ BAD: Business logic mixed with CLI
#[verb]
fn calculate_price(quantity: i32, unit_price: Decimal) -> Result<Decimal> {
    // Discount logic HERE - not reusable
    let subtotal = Decimal::from(quantity) * unit_price;
    let discount = if quantity > 10 {
        subtotal * Decimal::new(10, 2)
    } else {
        Decimal::ZERO
    };
    Ok(subtotal - discount)
}
```

**Fix:** Move logic to domain function:

```rust
// ✅ GOOD: Domain layer
pub fn calculate_price(quantity: i32, unit_price: Decimal) -> Decimal {
    let subtotal = Decimal::from(quantity) * unit_price;
    let discount = if quantity > 10 {
        subtotal * Decimal::new(10, 2)
    } else {
        Decimal::ZERO
    };
    subtotal - discount
}

// ✅ GOOD: CLI layer
#[verb]
fn calculate(quantity: i32, unit_price: Decimal) -> Result<Decimal> {
    Ok(domain::calculate_price(quantity, unit_price))
}
```

### ❌ CLI Types Leaking into Domain

```rust
// ❌ BAD: Domain knows about clap types
pub fn process(args: &ArgMatches) -> Result<Output> {
    let value = args.get_one::<String>("input").unwrap();
    // Domain logic using clap types
}
```

**Fix:** Extract values in CLI layer:

```rust
// ✅ GOOD: Domain layer - no CLI knowledge
pub fn process(input: &str) -> Result<Output> {
    // Pure logic
}

// ✅ GOOD: CLI layer
#[verb]
fn process_cmd(input: String) -> Result<Output> {
    domain::process(&input)
}
```

### ❌ File Paths in Domain

```rust
// ❌ BAD: Domain tied to filesystem
pub fn analyze(file_path: &Path) -> Result<Analysis> {
    let content = std::fs::read_to_string(file_path)?;
    // Analysis logic
}
```

**Fix:** Use generic I/O:

```rust
// ✅ GOOD: Domain layer
pub fn analyze<R: BufRead>(reader: R) -> Result<Analysis> {
    // Analysis logic using reader
}

// ✅ GOOD: CLI layer
#[verb]
fn analyze_file(file: PathBuf) -> Result<Analysis> {
    let reader = BufReader::new(File::open(file)?);
    domain::analyze(reader)
}
```

## Decision Matrix

Choose the right pattern based on your needs:

| Pattern | Use When | Testing Strategy | Performance |
|---------|----------|------------------|-------------|
| Service Layer | Complex operations, state management | Integration tests with real services | Moderate (allocations) |
| Pure Functions | Stateless transformations | Fast unit tests (no mocks) | Excellent (zero-cost) |
| Generic I/O | File/stream processing | In-memory buffers | Excellent (zero-cost abstractions) |
| State Machines | Complex state transitions | Test all transitions | Excellent (zero allocation) |
| Builder Pattern | Complex construction with validation | Test validation rules | Good (builder allocates) |

## Testing Benefits by Pattern

### Service Layer
```rust
#[test]
fn test_service_manager() {
    let mut manager = ServiceManager::new();
    let result = manager.restart_service("web-server").unwrap();
    assert_eq!(result.success, true);
}
```

### Pure Functions
```rust
#[test]
fn test_tax_calculation() {
    let result = calculate_tax(
        Decimal::new(100, 0),
        Decimal::new(10, 2), // 0.10
    );
    assert_eq!(result, Decimal::new(10, 0));
}
```

### Generic I/O
```rust
#[test]
fn test_process_stream() {
    let input = b"test data\n";
    let mut output = Vec::new();
    process_stream(&input[..], &mut output).unwrap();
    assert_eq!(output, b"PROCESSED: test data\n");
}
```

### State Machines
```rust
#[test]
fn test_connection_state_machine() {
    let state = ConnectionState::Idle;
    let state = state.handle_event(ConnectionEvent::Connect);
    assert!(matches!(state, ConnectionState::Connecting { .. }));
}
```

## Complete Examples

See production examples in the repository:

- **Service Management**: [`examples/services.rs`](../../examples/services.rs) - Service layer pattern
- **Framework Composition**: [`examples/framework.rs`](../../examples/framework.rs) - Multi-noun CLI
- **ggen CLI**: [`examples/ggen_cli.rs`](../../examples/ggen_cli.rs) - Complete production app
  - Error handling: [`examples/ggen/errors.rs`](../../examples/ggen/errors.rs)
  - Validators: [`examples/ggen/validators.rs`](../../examples/ggen/validators.rs)
  - Commands: [`examples/ggen/ai_commands.rs`](../../examples/ggen/ai_commands.rs)

## Best Practices

1. **Start with Pure Functions**: Default to stateless transformations
2. **Add State When Needed**: Use service layer for complex operations
3. **Generic Over Concrete**: Prefer `impl Read` over `File`
4. **Test Domain First**: Domain tests should be fast and comprehensive
5. **CLI as Thin Wrapper**: Keep CLI functions to 2-5 lines
6. **Error Handling in Domain**: Domain returns `Result`, CLI converts to user messages
7. **Type-Driven Design**: Use types to enforce invariants

## Next Steps

- **Reference**: [Architecture Overview](../reference/architecture.md)
- **Tutorial**: [Building Your First CLI](../tutorials/getting-started.md)
- **Explanation**: [Why Domain Separation Matters](../explanation/domain-separation.md)
- **How-To**: [Error Handling Patterns](error-handling.md)
