# clap-noun-verb Troubleshooting Guide

Solutions to common problems when building CLI applications with clap-noun-verb.

## Table of Contents

- [Installation & Setup Issues](#installation--setup-issues)
- [Command Discovery Issues](#command-discovery-issues)
- [Argument Parsing Errors](#argument-parsing-errors)
- [Compilation Errors](#compilation-errors)
- [Runtime Errors](#runtime-errors)
- [Performance Issues](#performance-issues)
- [Output Format Issues](#output-format-issues)
- [Async/Await Issues](#asyncawait-issues)
- [Context & State Issues](#context--state-issues)
- [Platform-Specific Issues](#platform-specific-issues)

---

## Installation & Setup Issues

### Error: "failed to resolve: use of undeclared crate or module"

**Symptoms:**
```
error[E0433]: failed to resolve: use of undeclared crate or module `clap_noun_verb_macros`
```

**Cause:** Missing macro crate dependency.

**Solution:**
```toml
# Add to Cargo.toml
[dependencies]
clap-noun-verb = "4.0.2"
clap-noun-verb-macros = "4.0.2"  # ← Don't forget this!
```

---

### Error: "no method named `run` found"

**Symptoms:**
```
error[E0599]: no method named `run` found for fn item in the current scope
```

**Cause:** Wrong import or calling pattern.

**Solution:**
```rust
// ✅ Correct
use clap_noun_verb;

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()  // Function call, not method
}

// ❌ Wrong
fn main() {
    clap_noun_verb::run;  // Missing ()
}
```

---

### Cargo Build Hangs

**Symptoms:** `cargo build` hangs indefinitely or takes very long time.

**Cause:** Network issues or crates.io being slow.

**Solution:**
```bash
# Use sparse registry (Rust 1.68+)
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Or use a mirror
# In ~/.cargo/config.toml
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"

# Clear cache and retry
cargo clean
rm -rf ~/.cargo/registry/index/*
cargo build
```

---

## Command Discovery Issues

### Error: "unknown command 'mycommand'"

**Symptoms:**
```bash
$ myapp services status
error: unknown command 'services'
```

**Cause 1:** Module not declared in main.rs

**Solution:**
```rust
// ✅ Correct - main.rs
mod services;  // ← Must declare module
mod database;

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
```

**Cause 2:** File naming doesn't match noun

**Solution:**
```bash
# ✅ Correct
src/services.rs    # Creates 'services' noun
src/database.rs    # Creates 'database' noun

# ❌ Wrong
src/my_services.rs  # Creates 'my_services' noun (with underscore)
```

---

### Commands Work in Debug but Not Release

**Symptoms:** Commands discovered in `cargo run` but not in `cargo build --release`.

**Cause:** Link-time optimization (LTO) removing unused code.

**Solution:**
```toml
# Cargo.toml
[profile.release]
lto = "thin"  # Use thin LTO instead of fat
```

Or ensure verbs are actually used:
```rust
// Force inclusion
#[used]
#[verb]
fn my_verb() -> Result<Output> { ... }
```

---

### No Verbs Discovered

**Symptoms:**
```
error: no verb functions found in noun 'services'
```

**Cause:** Missing `#[verb]` macro or wrong syntax.

**Solution:**
```rust
// ✅ Correct
#[verb]
fn status() -> Result<Status> { ... }

// ❌ Wrong - missing macro
fn status() -> Result<Status> { ... }

// ❌ Wrong - wrong attribute
#[derive(Verb)]
fn status() -> Result<Status> { ... }
```

---

## Argument Parsing Errors

### Error: "required arguments were not provided"

**Symptoms:**
```bash
$ myapp deploy
error: the following required arguments were not provided:
  --environment <ENVIRONMENT>
```

**Cause:** Argument is `String` type (required) but user didn't provide it.

**Solution 1:** Make it optional
```rust
// Before
#[verb]
fn deploy(environment: String) -> Result<DeployResult>

// After - optional with default
#[verb]
fn deploy(
    #[arg(default_value = "staging")]
    environment: String
) -> Result<DeployResult>

// Or - truly optional
#[verb]
fn deploy(environment: Option<String>) -> Result<DeployResult>
```

**Solution 2:** Use positional argument
```rust
#[verb]
fn deploy(
    #[arg(index = 0)]  // Makes it positional
    environment: String
) -> Result<DeployResult>

// Usage: myapp deploy production
```

---

### Error: "invalid value for argument"

**Symptoms:**
```bash
$ myapp serve --port abc
error: invalid value 'abc' for '--port <PORT>': invalid digit found in string
```

**Cause:** Type mismatch between CLI input and Rust type.

**Solution:** Add validation and better error messages
```rust
#[verb]
fn serve(
    #[arg(
        help = "Port number (1024-65535)",
        validator = validate_port
    )]
    port: u16
) -> Result<ServerStatus>

fn validate_port(s: &str) -> Result<(), String> {
    let port: u16 = s.parse()
        .map_err(|_| format!("'{}' is not a valid port number", s))?;

    if port < 1024 {
        return Err("Port must be >= 1024 (unprivileged)".to_string());
    }

    Ok(())
}
```

---

### Vec<String> Not Accepting Multiple Values

**Symptoms:**
```bash
$ myapp tag --tags one two three
error: unexpected argument 'two' found
```

**Cause:** Missing `multiple` attribute or wrong action.

**Solution:**
```rust
// ✅ Correct
#[verb]
fn tag(
    #[arg(multiple)]  // ← Add this
    tags: Vec<String>
) -> Result<TagResult>

// Or be explicit about action
#[verb]
fn tag(
    #[arg(action = "append")]
    tags: Vec<String>
) -> Result<TagResult>
```

---

### Boolean Flags Not Working

**Symptoms:**
```bash
$ myapp build --release
# Flag is false even though provided
```

**Cause:** Wrong type or action.

**Solution:**
```rust
// ✅ Correct - auto SetTrue
#[verb]
fn build(release: bool) -> Result<BuildResult>
// release is true if --release present

// ✅ Explicit action
#[verb]
fn build(
    #[arg(action = "set_true")]
    release: bool
) -> Result<BuildResult>

// ❌ Wrong - Option<bool> doesn't work as flag
#[verb]
fn build(release: Option<bool>) -> Result<BuildResult>
```

---

## Compilation Errors

### Error: "trait bound `T: Serialize` is not satisfied"

**Symptoms:**
```
error[E0277]: the trait bound `MyStruct: Serialize` is not satisfied
```

**Cause:** Return type doesn't implement `Serialize`.

**Solution:**
```rust
// Add #[derive(Serialize)]
use serde::Serialize;

#[derive(Serialize)]  // ← Add this
struct MyStruct {
    field: String,
}

#[verb]
fn my_verb() -> Result<MyStruct> { ... }
```

---

### Error: "cannot find macro `verb` in this scope"

**Symptoms:**
```
error: cannot find attribute `verb` in this scope
```

**Cause:** Missing import or wrong crate name.

**Solution:**
```rust
// ✅ Correct
use clap_noun_verb_macros::verb;

#[verb]
fn status() -> Result<Status> { ... }

// ❌ Wrong - importing from wrong crate
use clap_noun_verb::verb;  // Wrong!
```

---

### Error: "expected `Result<...>`, found `MyStruct`"

**Symptoms:**
```
error[E0308]: mismatched types
  expected enum `Result<MyStruct>`
     found struct `MyStruct`
```

**Cause:** Verb function must return `Result<T>`.

**Solution:**
```rust
// ❌ Wrong
#[verb]
fn status() -> Status {
    Status { ... }
}

// ✅ Correct
#[verb]
fn status() -> Result<Status> {
    Ok(Status { ... })  // Wrap in Ok()
}
```

---

### Slow Compile Times

**Symptoms:** `cargo build` takes minutes even for small changes.

**Solution 1:** Use incremental compilation
```toml
# Cargo.toml
[profile.dev]
incremental = true
```

**Solution 2:** Reduce dependencies
```toml
# Use default-features = false
[dependencies]
serde = { version = "1.0", features = ["derive"] }
# Don't import unused features
```

**Solution 3:** Use cargo-watch for incremental builds
```bash
cargo install cargo-watch
cargo watch -x build
```

---

## Runtime Errors

### Error: "No JSON output"

**Symptoms:** CLI prints nothing or prints debug format instead of JSON.

**Cause:** Missing `Serialize` implementation or output not being captured.

**Solution:**
```rust
// Ensure Serialize is derived
#[derive(Serialize)]
struct Output { ... }

// Ensure you're returning the value
#[verb]
fn status() -> Result<Output> {
    Ok(Output { ... })  // ✅ Return it
    // Don't just create it and drop
}
```

---

### Error: "thread 'main' panicked at 'unwrap failed'"

**Symptoms:** Panic instead of proper error.

**Cause:** Using `.unwrap()` instead of `?` operator.

**Solution:**
```rust
// ❌ Wrong - panics on error
#[verb]
fn read_file(path: String) -> Result<FileContents> {
    let contents = std::fs::read_to_string(path).unwrap();  // Panic!
    Ok(FileContents { contents })
}

// ✅ Correct - propagates error
#[verb]
fn read_file(path: String) -> Result<FileContents> {
    let contents = std::fs::read_to_string(path)?;  // Returns error
    Ok(FileContents { contents })
}
```

---

### Context Type Not Found

**Symptoms:**
```
error: no method named `get` found for type `AppContext`
```

**Cause:** State type not inserted into context.

**Solution:**
```rust
// ✅ Insert state before running
fn main() -> Result<()> {
    let context = AppContext::new();
    context.insert(AppState { ... })?;  // ← Insert first!

    clap_noun_verb::run_with_context(context)
}

// In verb
#[verb]
fn query(args: &VerbArgs) -> Result<QueryResult> {
    let state: AppState = args.context.get()?;  // ← Now works
    // ...
}
```

---

## Performance Issues

### CLI Startup is Slow

**Symptoms:** 1-2 second delay before command execution.

**Solution 1:** Profile startup
```bash
cargo build --release
time ./target/release/myapp --help  # Measure baseline
```

**Solution 2:** Lazy-load heavy dependencies
```rust
// ❌ Wrong - loads at startup
use heavy_dependency::*;

// ✅ Correct - load only when needed
#[verb]
fn heavy_operation() -> Result<Output> {
    use heavy_dependency::*;  // Load here
    // ...
}
```

**Solution 3:** Use dynamic linking in development
```toml
# Cargo.toml
[profile.dev]
opt-level = 1  # Some optimization
```

---

### High Memory Usage

**Symptoms:** CLI uses hundreds of MB of memory.

**Solution 1:** Use streaming for large data
```rust
// ❌ Wrong - loads everything into memory
let data = std::fs::read_to_string(path)?;
let lines: Vec<_> = data.lines().collect();

// ✅ Correct - streams line by line
use std::io::{BufReader, BufRead};
let file = std::fs::File::open(path)?;
let reader = BufReader::new(file);
for line in reader.lines() {
    process_line(&line?)?;
}
```

**Solution 2:** Drop values early
```rust
{
    let large_data = load_data()?;
    process(&large_data)?;
}  // large_data dropped here
// Continue without large_data in memory
```

---

## Output Format Issues

### Table Output is Garbled

**Symptoms:** Box-drawing characters display incorrectly.

**Cause:** Terminal encoding issues.

**Solution:**
```bash
# Set UTF-8 encoding
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Or use JSON instead
myapp services status --format json
```

---

### YAML Output Has Extra Quotes

**Symptoms:**
```yaml
name: "my-value"  # Has quotes
value: "42"       # Number as string
```

**Cause:** Fields are String type instead of native types.

**Solution:**
```rust
// Use proper types
#[derive(Serialize)]
struct Output {
    name: String,      // Will have quotes (correct)
    value: u32,        // No quotes (correct)
    enabled: bool,     // true/false (correct)
}
```

---

### JSON Output Not Pretty-Printed

**Symptoms:** JSON is all on one line.

**Solution:**
```rust
// Custom format function
use serde_json;

#[verb]
fn status(
    #[arg(long)]
    pretty: bool,
) -> Result<Status> {
    let status = Status { ... };

    if pretty {
        let json = serde_json::to_string_pretty(&status)?;
        println!("{}", json);
    }

    Ok(status)  // Framework will also output
}
```

---

## Async/Await Issues

### Error: "no method named `await` found"

**Symptoms:**
```
error[E0728]: `await` is only allowed inside `async` functions and blocks
```

**Cause:** Using `await` in non-async context.

**Solution:**
```rust
// ✅ Correct - use run_async
use clap_noun_verb::async_verb::run_async;

#[verb]
fn fetch(url: String) -> Result<FetchResult> {
    run_async(async move {  // ← Wrap in run_async
        let response = reqwest::get(&url).await?;
        let data = response.text().await?;
        Ok(FetchResult { data })
    })
}
```

---

### Tokio Runtime Error

**Symptoms:**
```
thread 'main' panicked at 'Cannot start a runtime from within a runtime'
```

**Cause:** Multiple tokio runtimes.

**Solution:**
```rust
// ✅ Use framework's runtime
use clap_noun_verb::async_verb::run_async;

// ❌ Don't create your own runtime
// tokio::runtime::Runtime::new()
```

---

## Context & State Issues

### State Not Shared Between Commands

**Symptoms:** Each command has fresh state.

**Cause:** State not properly inserted into shared context.

**Solution:**
```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<i32>>,  // Use Arc for shared state
}

fn main() -> Result<()> {
    let state = AppState {
        counter: Arc::new(Mutex::new(0)),
    };

    let context = AppContext::new();
    context.insert(state)?;

    clap_noun_verb::run_with_context(context)
}
```

---

## Platform-Specific Issues

### Windows: "command not found"

**Symptoms:** Binary not found after `cargo build`.

**Solution:**
```bash
# Windows uses .exe extension
./target/debug/myapp.exe services status

# Or add to PATH
set PATH=%PATH%;%CD%\target\debug
myapp.exe services status
```

---

### macOS: "cannot be opened because the developer cannot be verified"

**Symptoms:** macOS blocks execution.

**Solution:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ./target/release/myapp

# Or sign the binary
codesign -s - ./target/release/myapp
```

---

### Linux: "permission denied"

**Symptoms:** Cannot execute binary.

**Solution:**
```bash
# Add execute permission
chmod +x ./target/release/myapp

# Or
cargo build --release
./target/release/myapp  # Already has +x
```

---

## Getting Help

### Still Stuck?

1. **Check Examples**: See [examples/](../examples/) for working code
2. **Search Issues**: [GitHub Issues](https://github.com/ruvnet/clap-noun-verb/issues)
3. **Ask Questions**: [GitHub Discussions](https://github.com/ruvnet/clap-noun-verb/discussions)
4. **Read Docs**: [Full Documentation](../README.md)

### Reporting Bugs

When reporting issues, include:

```bash
# Version info
cargo --version
rustc --version

# Your Cargo.toml [dependencies]
cat Cargo.toml

# Full error message
cargo build 2>&1 | tee error.log

# Minimal reproduction
# Share smallest code that reproduces issue
```

---

## Compatibility Matrix

### Rust Versions

| clap-noun-verb | Minimum Rust | Recommended |
|----------------|--------------|-------------|
| 4.0.x | 1.70.0 | 1.75.0+ |
| 3.x | 1.65.0 | 1.70.0+ |

### Operating Systems

| OS | Support | Notes |
|----|---------|-------|
| Linux | ✅ Full | All features supported |
| macOS | ✅ Full | All features supported |
| Windows | ✅ Full | PowerShell recommended |
| BSD | ⚠️ Partial | Community-tested |

### Dependencies

| Dependency | Version | Required |
|------------|---------|----------|
| clap | 4.x | Yes |
| serde | 1.x | Yes |
| tokio | 1.x | For async |

---

## See Also

- [Quick Start Guide](./QUICKSTART.md) - Get started in 10 minutes
- [CLI Reference](./CLI_REFERENCE.md) - Complete API reference
- [CLI Cookbook](./CLI_COOKBOOK.md) - Common recipes and patterns
- [Examples](../examples/) - Working code examples

---

**Version:** 4.0.2
**Last Updated:** 2024-11-18
**License:** MIT OR Apache-2.0
