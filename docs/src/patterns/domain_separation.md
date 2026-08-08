# Pattern: Domain Separation

## Context
When building command-line applications (CLIs) in Rust, developers often mix command-line parsing, argument validation, execution, and output formatting in the same functions or modules. The application is built around the CLI library's paradigms rather than the application's domain logic.

## Problem
Tightly coupling domain logic with command-line parsing and output presentation makes the codebase difficult to test, maintain, and reuse.

## Forces
* **Testability:** Testing domain logic that is mixed with CLI-specific arguments, standard output streaming, or exit codes requires complex mock environments and standard stream capturing.
* **Interface Flexibility (Multi-modal interfaces):** Exposing the same functionality to an interactive REPL, an HTTP API, an LLM agent, or standard automated testing tools becomes highly redundant or impossible if the logic is bound to command-line argument parsing.
* **Separation of Concerns:** Business logic should change only when business rules change. The CLI presentation layer should change only when the CLI interface changes. Mixing them violates this principle.
* **Developer Overhead:** Writing translation layers between CLI options and internal functions can feel like boilerplate, motivating developers to take the shortcut of mixing them together.

## Solution
Strictly isolate CLI wrappers from core business logic using a layered architecture:
1. **Core Domain Layer:** Implement domain logic in pure, CLI-agnostic functions and structures. These functions accept standard Rust types (e.g., primitives, structs) and return standard `Result<T, E>`. They must have no knowledge of `clap`, command-line flags, or printing to `stdout`.
2. **CLI Wrapper Layer:** Create thin CLI wrapper functions decorated with the `#[verb]` and `#[noun]` macros. These wrappers are solely responsible for mapping command-line arguments to domain arguments, invoking the domain function, and returning the domain result wrapped in a serializable structure.

### Example

```rust
// 1. Pure Domain Logic (No CLI details, easy to unit test)
pub fn calculate_sum(x: i32, y: i32) -> i32 {
    x + y
}

// 2. Thin CLI Wrapper (Solely handles command mapping)
#[derive(serde::Serialize)]
pub struct SumResult {
    pub result: i32,
}

#[clap_noun_verb_macros::verb("add")]
fn cmd_add(x: i32, y: i32) -> clap_noun_verb::Result<SumResult> {
    Ok(SumResult {
        result: calculate_sum(x, y),
    })
}
```

## Resulting Context / Connections
* **Isolability:** Domain functions can be unit-tested rapidly and reliably.
* **Reusability:** The `calculate_sum` function can be used in other crates, libraries, or service layers without pulling in CLI dependencies.
* **Agent Integration:** Because the CLI wrappers return type-safe, serializable Rust structs rather than printing directly, they feed directly into the **JSON by Default** pattern, making the CLI compatible with agent workflows.
* **Introspection:** Since wrappers are thin metadata declarations, they enable **Reflexive Introspection** to discover commands and generate schemas without running side-effect-heavy domain code.
