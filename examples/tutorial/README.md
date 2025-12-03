# Tutorial Examples

**Learning-oriented examples for developers new to clap-noun-verb**

These examples are designed to be worked through in order, building your understanding progressively.

## Learning Path (30 minutes total)

### 1. basic.rs (5 min)
**Your first CLI** - The simplest possible clap-noun-verb application.

```bash
cargo run --example tutorial_basic -- services status
```

**What you'll learn:**
- Basic `#[noun]` and `#[verb]` macro usage
- Domain separation principle
- JSON output format

### 2. arguments.rs (10 min)
**Adding typed arguments** - Working with command-line arguments.

```bash
cargo run --example tutorial_arguments -- user create alice alice@example.com
```

**What you'll learn:**
- Required vs optional arguments
- Type inference from function signature
- Multiple arguments with different types

### 3. positional.rs (5 min)
**Positional vs named arguments** - Understanding argument positioning.

```bash
cargo run --example tutorial_positional -- config.toml --verbose
```

**What you'll learn:**
- Positional arguments (no `--` flag)
- Named arguments (`--flag`)
- Combining both styles

### 4. services.rs (10 min)
**Multi-noun CLI** - Building a CLI with multiple command groups.

```bash
cargo run --example tutorial_services -- services status
cargo run --example tutorial_services -- config get timeout
```

**What you'll learn:**
- Multiple nouns in one CLI
- Command organization patterns
- Help text generation

## Key Concepts

### Domain Separation
```rust
// ✅ GOOD: Domain logic separate from CLI
fn add(x: i32, y: i32) -> i32 { x + y }  // Pure function

#[verb("add")]
fn cmd_add(x: i32, y: i32) -> Result<Output> {
    Ok(Output { result: add(x, y) })  // CLI delegates to domain
}
```

### JSON Output
All commands return JSON by default for agent consumption:
```bash
$ myapp calc add 2 3
{"result": 5}
```

### Type Inference
Arguments types are inferred from the function signature:
```rust
#[verb("greet")]
fn greet(name: String, times: u32) -> Result<Output> { }
//       ^^^^^^^       ^^^^
//       Required      Required
//       String        u32 (validated)
```

## Next Steps

After completing these tutorials:
1. Try the [how-to examples](../howto/) to solve specific problems
2. Read [docs/tutorial/](../../docs/tutorial/) for in-depth explanations
3. Explore the [reference examples](../reference/) for complete API coverage
