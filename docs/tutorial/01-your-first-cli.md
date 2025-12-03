# Tutorial 01: Your First CLI in 5 Minutes

**Learning Path:** Beginner → First Working CLI
**Time:** 5 minutes
**Prerequisites:** Rust installed, basic Rust knowledge

---

## What You'll Build

A simple calculator CLI that demonstrates clap-noun-verb's core concepts:
- Domain-separated architecture
- Automatic command discovery
- JSON output for AI agents

```bash
$ calculator math add --x 5 --y 3
{"result": 8}
```

---

## Step 1: Create New Project (1 min)

```bash
cargo new calculator
cd calculator
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "5.2"
clap-noun-verb-macros = "5.2"
serde = { version = "1.0", features = ["derive"] }
```

---

## Step 2: Write Domain Logic (1 min)

Create `src/domain.rs` - pure business logic:

```rust
//! Pure domain logic - no CLI dependencies

pub fn add(x: i32, y: i32) -> i32 {
    x + y
}

pub fn subtract(x: i32, y: i32) -> i32 {
    x - y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
    }
}
```

**Why separate domain logic?**
- ✅ Testable without CLI framework
- ✅ Reusable in other contexts
- ✅ Pure Rust functions, no dependencies

---

## Step 3: Create CLI Commands (2 min)

Create `src/commands.rs` - thin CLI wrapper:

```rust
//! CLI commands - thin wrappers over domain logic

use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct MathResult {
    pub result: i32,
}

/// Add two numbers
#[verb(
    help = "Add two numbers together"
)]
pub fn add(
    #[arg(help = "First number")] x: i32,
    #[arg(help = "Second number")] y: i32,
) -> Result<MathResult, Box<dyn std::error::Error>> {
    let result = crate::domain::add(x, y);
    Ok(MathResult { result })
}

/// Subtract two numbers
#[verb(
    help = "Subtract second number from first"
)]
pub fn subtract(
    #[arg(help = "First number")] x: i32,
    #[arg(help = "Second number")] y: i32,
) -> Result<MathResult, Box<dyn std::error::Error>> {
    let result = crate::domain::subtract(x, y);
    Ok(MathResult { result })
}
```

**Key Pattern:**
1. CLI function delegates to domain immediately
2. Zero business logic in CLI layer
3. Returns structured JSON for agents

---

## Step 4: Wire Up Main (1 min)

Update `src/main.rs`:

```rust
mod domain;
mod commands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    clap_noun_verb::run()
}
```

**That's it!** Auto-discovery finds all `#[verb]` functions.

---

## Step 5: Run Your CLI

```bash
# Build and test
cargo build

# See available commands
cargo run -- --help

# Add numbers
cargo run -- math add --x 5 --y 3
{"result":8}

# Subtract numbers
cargo run -- math subtract --x 10 --y 4
{"result":6}
```

---

## What Just Happened?

### Architecture Layers

```
┌─────────────────┐
│  CLI Commands   │  ← src/commands.rs (thin validation)
└────────┬────────┘
         │
┌────────▼────────┐
│  Domain Logic   │  ← src/domain.rs (pure, testable)
└─────────────────┘
```

### Auto-Discovery Magic

The `#[verb]` macro:
1. Registers `add` as command `math add`
2. Parses arguments from function signature
3. Serializes result to JSON
4. Handles errors automatically

**Noun inference:** Function `add` in `commands.rs` → noun `math` (from module)

---

## Exercise: Add Multiplication

**Goal:** Add a `multiply` command

**Arrange:** Add domain function
```rust
// In src/domain.rs
pub fn multiply(x: i32, y: i32) -> i32 {
    x * y
}
```

**Act:** Add CLI command
```rust
// In src/commands.rs
#[verb(help = "Multiply two numbers")]
pub fn multiply(
    #[arg(help = "First number")] x: i32,
    #[arg(help = "Second number")] y: i32,
) -> Result<MathResult, Box<dyn std::error::Error>> {
    let result = crate::domain::multiply(x, y);
    Ok(MathResult { result })
}
```

**Assert:** Test it works
```bash
cargo run -- math multiply --x 6 --y 7
# Expected: {"result":42}
```

---

## Key Takeaways

✅ **Domain-separated architecture** - CLI validates, domain computes
✅ **Auto-discovery** - Just add `#[verb]`, framework does the rest
✅ **JSON output** - Perfect for AI agents and MCP integration
✅ **Type-safe** - Compile-time validation of arguments
✅ **Zero boilerplate** - No manual command registration

---

## Next Steps

- **[Tutorial 02: Domain Separation](02-domain-separation.md)** - Deep dive into architecture
- **[Tutorial 03: Adding Commands](03-adding-commands.md)** - Advanced command patterns
- **[Tutorial 04: Testing Basics](04-testing-basics.md)** - Chicago TDD approach

**Estimated time to next tutorial:** 10 minutes

---

## Quick Reference

**Project structure:**
```
calculator/
├── src/
│   ├── main.rs       # Entry point
│   ├── domain.rs     # Pure business logic
│   └── commands.rs   # CLI wrappers
└── Cargo.toml
```

**Common commands:**
- `cargo build` - Compile
- `cargo run -- --help` - Show help
- `cargo test` - Run tests
- `cargo run -- <noun> <verb> [args]` - Execute command

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
