# clap-noun-verb: Design Principles

## Core Principle: CLI as Interface Layer

The CLI is **only an interface layer** that validates arguments and delegates to reusable business logic. The business logic layer is completely independent and can be used by CLI, API, web apps, or any other interface.

## Key Design Principles

### 1. CLI Code ONLY Validates Arguments

The CLI layer contains **NO business logic**. It only:
- Validates argument types and formats
- Extracts validated arguments
- Delegates to business logic functions
- Shapes output for JSON

**Enforcement**: The type system and attribute macros enforce this pattern.

### 2. Business Logic is Reusable

Business logic functions are:
- Pure functions (no side effects unless intentional)
- Independent of CLI implementation
- Callable from any interface (CLI, API, Web, etc.)
- Testable in isolation

### 3. Clear Separation of Concerns

```
CLI Layer:     Argument validation → Delegation → Output shaping
Logic Layer:   Pure business functions
Runtime Layer: Execution infrastructure
```

### 4. Type System Enforcement

The type system enforces the pattern:
- Attribute macros generate validation-only wrappers
- Business logic functions have separate types
- Return types must implement `Serialize` for JSON output

## Architecture Layers

### Layer 1: CLI Layer (`src/cli/`)

**Responsibility**: Argument validation and routing only
- `registry.rs` - Command registration with linkme
- `router.rs` - Command routing
- `validator.rs` - Argument validation
- **NO business logic allowed**

### Layer 2: Business Logic Layer (`src/logic/`)

**Responsibility**: Reusable business functions
- `handler.rs` - Handler input/output types
- **Independent of CLI** - can be used by any interface

### Layer 3: Runtime Layer (`src/runtime/`)

**Responsibility**: Execution infrastructure
- `executor.rs` - Executes business logic
- `interceptor.rs` - Cross-cutting concerns (logging, tracing, etc.)

## Example: v3.0.0 Pattern

### Business Logic (Pure, Reusable)

```rust
fn get_service_status() -> Status {
    // Pure function - can be used by CLI, API, Web, etc.
    Status {
        services: vec!["api".to_string()],
        healthy: true,
    }
}
```

### CLI Layer (Validation + Delegation Only)

```rust
use clap_noun_verb_macros::{noun, verb};
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    services: Vec<String>,
    healthy: bool,
}

/// Show status of all services
#[noun("services", "Manage services")]
#[verb("status")]
fn show_status() -> Result<Status> {
    // 1. Validate inputs (auto-validated from signature - none here)
    // 2. Delegate to business logic
    Ok(get_service_status())
    // 3. Output shaping (auto-serializes to JSON)
}
```

## Benefits

1. **Reusability** - Business logic can be used by CLI, API, Web, etc.
2. **Testability** - Business logic functions can be tested independently
3. **Maintainability** - Clear separation makes code easier to understand
4. **Enforcement** - Attribute macros and type system prevent mixing concerns
5. **Scalability** - Easy to add new interfaces without duplicating logic
