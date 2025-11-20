# Tutorial: Domain-Separated CLI in 5 Minutes

**Target:** AI coding agents bootstrapping new CLI projects
**Time:** 5 minutes
**Output:** Working CLI with proper domain separation

## Prerequisites

- Rust 1.74+
- cargo installed
- Understanding of Rust types and trait system

## Step 1: Project Structure (1 min)

Create the canonical structure:

```bash
cargo new my-cli
cd my-cli
mkdir -p src/{cli,domain,integration} tests
```

Your structure:
```
my-cli/
├── src/
│   ├── cli/          # CLI layer (clap-noun-verb)
│   │   └── commands.rs
│   ├── domain/       # Domain logic (pure Rust)
│   │   └── processor.rs
│   ├── integration/  # Glue code
│   │   └── mod.rs
│   └── main.rs
├── tests/
│   └── integration_tests.rs
└── Cargo.toml
```

Add to `Cargo.toml`:
```toml
[dependencies]
clap-noun-verb = "5.0.0"
clap = { version = "4.0", features = ["derive"] }
```

## Step 2: Domain Layer First (2 min)

**Always start with domain, not CLI.**

```rust
// src/domain/processor.rs

/// Pure business logic - zero CLI dependencies
pub fn process_data(input: &str) -> Result<String, ProcessError> {
    if input.is_empty() {
        return Err(ProcessError::EmptyInput);
    }
    Ok(input.to_uppercase())
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Input cannot be empty")]
    EmptyInput,
}
```

```rust
// src/domain/mod.rs
pub mod processor;
pub use processor::{process_data, ProcessError};
```

**Key principle:** Domain layer has NO knowledge of CLI framework. Test independently.

## Step 3: CLI Layer Second (1 min)

**Thin wrapper over domain.**

```rust
// src/cli/commands.rs
use crate::domain;

/// Process input data
#[clap_noun_verb::verb]
pub fn process(
    /// Input string to process
    input: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // Delegate immediately to domain
    domain::process_data(&input).map_err(Into::into)
}
```

```rust
// src/cli/mod.rs
pub mod commands;
```

**Key principle:** CLI layer delegates immediately. Zero business logic here.

## Step 4: Wire It Up (30s)

```rust
// src/main.rs
use clap::Parser;
use clap_noun_verb::prelude::*;

mod cli;
mod domain;
mod integration;

#[derive(Parser)]
#[command(name = "my-cli", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Process(cli::commands::ProcessArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Process(args) => {
            match cli::commands::process(args.input) {
                Ok(result) => println!("{}", result),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}
```

## Step 5: Verify (30s)

```bash
# Test domain logic (no CLI involved)
cargo test

# Test CLI interface
cargo run -- process "hello world"
# Expected output: HELLO WORLD

cargo run -- process ""
# Expected error: Input cannot be empty
```

## What You Built

✅ **Domain logic separate** (testable, reusable, framework-agnostic)
✅ **CLI as thin interface** (delegates immediately to domain)
✅ **Zero business logic in CLI layer** (only arg parsing + delegation)
✅ **Production-ready structure** (scales to complex applications)

## Test Your Understanding

**Anti-pattern (❌):**
```rust
// Business logic in CLI layer - BAD
#[clap_noun_verb::verb]
pub fn process(input: String) -> Result<String> {
    if input.is_empty() {  // ← Domain logic leaked into CLI
        return Err("empty".into());
    }
    Ok(input.to_uppercase())  // ← Business logic in CLI
}
```

**Correct pattern (✅):**
```rust
// Thin delegation - GOOD
#[clap_noun_verb::verb]
pub fn process(input: String) -> Result<String> {
    domain::process_data(&input).map_err(Into::into)  // ← Immediate delegation
}
```

## Next Steps

**How-To Guides** (task-oriented):
- [Error Handling Patterns](../how-to/error-handling.md) - Add rich error types
- [Chicago TDD Testing](../how-to/testing-patterns.md) - Test domain + CLI layers
- [Multi-Command CLIs](../how-to/multi-command.md) - Scale to complex commands

**Reference** (information-oriented):
- [Macro System](../reference/macros.md) - `#[verb]` attribute details
- [Architecture](../reference/architecture.md) - Domain separation patterns
- [Error Types](../reference/errors.md) - Error handling reference

**Explanation** (understanding-oriented):
- [Why Domain Separation?](../explanation/domain-separation.md) - Design rationale
- [Semantic CLI Design](../explanation/semantic-design.md) - Noun-verb philosophy

## Common Agent Questions

**Q: Why domain first, CLI second?**
A: Domain = portable business logic. CLI = disposable interface. Keep them separate.

**Q: Can I put validation in CLI layer?**
A: Only format validation (e.g., parse args). Business validation goes in domain.

**Q: What if my CLI is trivial (no domain logic)?**
A: Structure is still valuable. Today's script is tomorrow's library.

**Q: How do I handle async domain logic?**
A: See [How-To: Async Commands](../how-to/async-commands.md)

---

**Time check:** If you followed this in <5 minutes, you have proper domain separation. If not, review Step 2 (domain first) and Step 3 (thin CLI wrapper).
