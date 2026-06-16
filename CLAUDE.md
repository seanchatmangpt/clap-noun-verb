# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**clap-noun-verb** is a Rust CLI framework built on top of `clap`, providing noun-verb command patterns (e.g., `myapp services status`). It uses proc-macros (`#[noun]`, `#[verb]`) for declarative command registration with `linkme` distributed slices for compile-time auto-discovery. Version 26.6.1.

## Build Commands

**Always use `cargo make`, never direct `cargo` commands.**

| Task | Command |
|------|---------|
| Format check | `cargo make format-check` |
| Format | `cargo make format` |
| Clippy | `cargo make clippy` |
| Lint (all) | `cargo make lint` |
| Test (quick) | `cargo make test` |
| Test (single-threaded) | `cargo make test-lib-deterministic` |
| Test (all features) | `cargo make test-all` |
| Test (frontier features) | `cargo make test-frontier` |
| Check | `cargo make check` |
| Check (all features) | `cargo make check-all` |
| Build | `cargo make build` |
| Build (release) | `cargo make build-release` |
| Build examples | `cargo make build-examples` |
| Doc | `cargo make doc` |
| CI (full) | `cargo make ci` |
| Benchmarks | `cargo make bench` |

**Single test**: `cargo test test_name --quiet` (only use direct `cargo test` for single test runs).

## Crate Structure

Two crates in a workspace:

- **`clap-noun-verb`** (`src/`) — Main library crate. All core modules, optional feature-gated modules, examples, tests.
- **`clap-noun-verb-macros`** (`clap-noun-verb-macros/src/`) — Proc-macro crate providing `#[noun]`, `#[verb]`, `#[arg]`, `#[meta_aware]`, `#[federated]`, `#[spec]`, `#[semantic_composable]`, `#[competency]`, `#[assessment]`, `#[auto_test]`, and more. Published first before the main crate.

## Architecture

### Core Flow
1. `#[verb]` macro on a function generates a `linkme::distributed_slice` entry
2. At startup, `CommandRegistry` collects all registered verbs via the distributed slice
3. `CliBuilder` constructs the clap `Command` tree from the registry
4. `CommandRouter` dispatches parsed args to the registered handler

### Key Modules (`src/`)
- **`cli/`** — Entry point (`run()`), `CommandRegistry` (noun/verb registration), `ArgMetadata`
- **`builder.rs`** — `CliBuilder` API for constructing CLIs
- **`router.rs`** — `CommandRouter` for dispatching commands
- **`logic/`** — `HandlerInput`/`HandlerOutput` types bridging CLI to domain
- **`error.rs`** — `NounVerbError` and `Result<T>` type
- **`format.rs`** — Output formatting (JSON by default, agent-ready)
- **`noun.rs`** / **`verb.rs`** — Trait definitions (`NounCommand`, `VerbCommand`)
- **`registry.rs`** — `CommandRegistry` for noun/verb registration
- **`tree.rs`** — `CommandTree` for hierarchical command structure

### Feature-Gated Modules
- `async` → `async_verb.rs` (the only optional module still present after the minimalist refactor)
- `federated-network` → `federation/` (feature-gated; see `src/federation/`)
- Note: earlier optional modules (`io/`, `kernel/`, `rdf/`, `semantic/`, `ggen_integration/`, `agent2028/`, `agents/`, `wizard/`, `plugin/`, `middleware/`, `integration/`, `plugins/`) were removed in the minimalist refactor and no longer exist in `src/`. The frontier feature *names* survive only as `check-cfg` allowances in `Cargo.toml`, not as buildable modules.

### Macro Crate (`clap-noun-verb-macros/src/`)
- **`lib.rs`** — `#[noun]` (deprecated no-op), `#[verb]` (main macro), `#[arg]` (parameter attributes)
- **`validation.rs`** — Compile-time validation (return type Serialize, duplicate detection, complexity checks)
- **`io_detection.rs`** — Auto-detection of `clio::Input`/`clio::Output` types
- **`macros/`** — Frontier feature macros (fractal patterns, federated network, semantic composition, executable specs, learning trajectories, reflexive testing)

### Feature System
- **Default**: No features (10 core dependencies only)
- **`full`**: All optional modules
- **Frontier features** (v5.4+): `meta-framework`, `rdf-composition`, `executable-specs`, `fractal-patterns`, `discovery-engine`, `federated-network`, `learning-trajectories`, `reflexive-testing`, `economic-sim`, `quantum-ready`
- **Meta-features**: `frontier-semantic`, `frontier-intelligence`, `frontier-quality`, `frontier-all`
- **`wizard`**: AI integration with rust-genai for multi-provider LLM support

## Critical Rules

### Error Handling
- **NEVER** use `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production code
- Clippy denies these via `lints.clippy`: `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`, `exit`
- Always use `Result<T>` with `?` operator or `map_err()`

### Trait Design
- Keep traits `dyn` compatible — no async methods in traits
- Use `&'static str` for trait method returns
- Use sync methods; async is handled via the `async` feature module

### Logging
- Library code: use `log!` macros (`log::error!`, `log::warn!`, `log::info!`, `log::debug!`)
- **NEVER** use `print!`/`println!` in library code (only in `src/bin/`, `build.rs`, and test code)

### Testing
- Follow AAA pattern (Arrange, Act, Assert)
- Test **behaviors** (observable outputs/state changes), not implementation details
- No tests that only check `assert!(result.is_ok())` — verify actual behavior
- Use descriptive test names: `test_verb_command_executes_successfully_with_required_args`
- Entire test suite must complete in <1 second with parallel execution

### Git
- **NEVER rebase** — only merge
- **NEVER** use `git reset --hard` — fix forward only
- **NEVER** use `--no-verify` — hooks are mandatory quality gates
- Branch prefixes: `claude/*`, `feat/*`, `fix/*`, `refactor/*`

## Formatting

- `rustfmt.toml`: `max_width = 100`, `tab_spaces = 4`, `use_small_heuristics = "Max"`
- `deny.toml`: Permissive licenses only (MIT, Apache-2.0, BSD, ISC). No copyleft (AGPL, GPL, LGPL denied)

## Publishing

Macros crate must be published before main crate:
```
cargo make publish-macros
cargo make publish
```

## SLOs

- Incremental compilation: <=2s (currently 0.66s)
- Binary size: <=10MB (currently 2.2MB)

---

## Development Workflows

### Adding a New Verb Command

1. **Create the verb function** in an appropriate module (e.g., `src/commands/my_feature.rs`):
   ```rust
   use crate::logic::{HandlerInput, HandlerOutput};
   use serde_json::json;
   
   #[clap_noun_verb::verb(name = "status", noun = "services")]
   pub async fn handle_services_status(
       input: HandlerInput,
   ) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
       // Implement command logic
       let response = json!({
           "status": "healthy",
           "timestamp": chrono::Utc::now().to_rfc3339()
       });
       Ok(HandlerOutput::new(response))
   }
   ```

2. **Register the module** in `src/lib.rs`:
   ```rust
   mod commands;
   pub use commands::*;
   ```

3. **Write tests** in the same file or a parallel test module:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[tokio::test]
       async fn test_services_status_returns_healthy() {
           let input = HandlerInput::default();
           let result = handle_services_status(input).await;
           assert!(result.is_ok());
           // Assert on actual output, not just Ok status
       }
   }
   ```

4. **Run tests locally**:
   ```bash
   cargo make test
   cargo test services_status --quiet
   ```

5. **Format and lint**:
   ```bash
   cargo make format
   cargo make lint
   ```

6. **Commit with descriptive message**:
   ```bash
   git add .
   git commit -m "feat: add services status verb command"
   ```

### Adding a Noun with Multiple Verbs

1. **Define the noun** as a trait in `src/nouns/my_noun.rs`:
   ```rust
   use crate::NounCommand;
   
   pub struct MyNoun;
   
   impl NounCommand for MyNoun {
       fn name(&self) -> &'static str {
           "mynoun"
       }
   }
   ```

2. **Register verbs under this noun** (they automatically associate via the `noun` parameter):
   ```rust
   #[clap_noun_verb::verb(name = "list", noun = "mynoun")]
   pub async fn handle_mynoun_list() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
       // Implementation
   }
   
   #[clap_noun_verb::verb(name = "create", noun = "mynoun")]
   pub async fn handle_mynoun_create() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
       // Implementation
   }
   ```

3. **Verify the command tree** builds:
   ```bash
   cargo make check
   ```

### Debugging a Command

1. **Enable verbose logging**:
   ```bash
   RUST_LOG=clap_noun_verb=debug cargo make build
   ```

2. **Use the debug runner**:
   ```bash
   cargo run --bin myapp -- --help
   cargo run --bin myapp -- mynoun list --verbose
   ```

3. **Inspect the command registry** by adding temporary debug output:
   ```rust
   log::debug!("Registered verbs: {:?}", registry.all_verbs());
   ```

4. **Run with backtrace for panics** (should be rare with our error handling):
   ```bash
   RUST_BACKTRACE=1 cargo run --bin myapp -- mynoun list
   ```

### Profiling Compilation Time

1. **Check incremental time** with an example change:
   ```bash
   touch src/lib.rs
   time cargo make build
   ```

2. **Profile with cargo-build-time**:
   ```bash
   cargo install cargo-build-time
   cargo build-time
   ```

3. **Identify slow dependencies**:
   ```bash
   cargo build --release --verbose 2>&1 | grep "Compiling"
   ```

4. **Check dependency tree**:
   ```bash
   cargo tree | head -30
   ```

### Running a Full Test Suite with Determinism Check

1. **Run quick tests** (parallelized):
   ```bash
   cargo make test
   ```

2. **Run deterministic tests** (single-threaded, for flaky test detection):
   ```bash
   cargo make test-lib-deterministic
   ```

3. **Run all features** to catch feature-gated issues:
   ```bash
   cargo make test-all
   ```

4. **Run frontier features** (if adding experimental functionality):
   ```bash
   cargo make test-frontier
   ```

### Upgrading Dependencies

1. **Check for outdated versions**:
   ```bash
   cargo outdated
   ```

2. **Update specific dependency**:
   ```bash
   cargo update -p dep_name
   ```

3. **Run full test suite** to catch breaking changes:
   ```bash
   cargo make test-all
   cargo make lint
   ```

4. **Check license compliance**:
   ```bash
   cargo deny check
   ```

### Adding a Feature Flag

1. **Define in `Cargo.toml`**:
   ```toml
   [features]
   my-feature = ["optional_dep"]
   full = ["async", "federated-network", "my-feature"]
   ```

2. **Conditionally compile code**:
   ```rust
   #[cfg(feature = "my-feature")]
   pub mod my_feature {
       // Implementation
   }
   ```

3. **Test with and without the feature**:
   ```bash
   cargo make test
   cargo test --features my-feature --quiet
   cargo test --all-features --quiet
   ```

### Publishing a Release

1. **Update version** in both `Cargo.toml` files (workspace crates):
   - `clap-noun-verb-macros/Cargo.toml`
   - `clap-noun-verb/Cargo.toml`

2. **Update `CHANGELOG.md`** with user-facing changes

3. **Run full CI suite**:
   ```bash
   cargo make ci
   ```

4. **Publish macros first** (dependency of main crate):
   ```bash
   cd clap-noun-verb-macros
   cargo publish
   ```

5. **Publish main crate**:
   ```bash
   cargo publish
   ```

6. **Create git tag**:
   ```bash
   git tag v26.6.1
   git push origin v26.6.1
   ```

---

## Common Recipes

### Recipe: Create a CLI Binary That Uses clap-noun-verb

```rust
// main.rs
use clap_noun_verb::{CommandRegistry, CliBuilder, CommandRouter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let registry = CommandRegistry::new();
    let app = CliBuilder::new("myapp")
        .version("1.0.0")
        .about("My awesome CLI")
        .build_from_registry(&registry)?;
    
    let matches = app.get_matches();
    let router = CommandRouter::from_registry(&registry);
    router.route(&matches).await?;
    
    Ok(())
}
```

### Recipe: Create a Verb with Input/Output Files

```rust
use clap_noun_verb::verb;
use clio::{Input, Output};

#[verb(name = "process", noun = "files")]
pub async fn handle_files_process(
    input: Input,
    output: Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = input.read()?;
    let processed = transform(&content);
    output.write(&processed)?;
    Ok(())
}

fn transform(content: &str) -> String {
    content.to_uppercase()
}
```

### Recipe: Create a Verb with Serializable Output

```rust
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ServiceStatus {
    name: String,
    healthy: bool,
    uptime_seconds: u64,
}

#[verb(name = "status", noun = "service")]
pub async fn handle_service_status(
    name: String,
) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    let status = ServiceStatus {
        name: name.clone(),
        healthy: true,
        uptime_seconds: 3600,
    };
    
    let output = serde_json::to_value(&status)?;
    Ok(HandlerOutput::new(output))
}
```

### Recipe: Create an Async Verb with Error Handling

```rust
use crate::error::NounVerbError;

#[verb(name = "fetch", noun = "data")]
pub async fn handle_data_fetch(
    url: String,
) -> Result<HandlerOutput, NounVerbError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| NounVerbError::ExecutionFailed(
            format!("Failed to fetch {}: {}", url, e)
        ))?;
    
    let json = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| NounVerbError::ExecutionFailed(
            format!("Failed to parse response: {}", e)
        ))?;
    
    Ok(HandlerOutput::new(json))
}
```

### Recipe: Create a Noun Grouping Commands

```rust
use crate::NounCommand;

pub struct Database;

impl NounCommand for Database {
    fn name(&self) -> &'static str {
        "db"
    }
    
    fn description(&self) -> &'static str {
        "Database management commands"
    }
}

#[verb(name = "init", noun = "db")]
pub async fn handle_db_init() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    // Initialize database
    Ok(HandlerOutput::new(json!({ "status": "initialized" })))
}

#[verb(name = "migrate", noun = "db")]
pub async fn handle_db_migrate(
    version: Option<String>,
) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    // Run migrations
    Ok(HandlerOutput::new(json!({ "status": "migrated", "version": version })))
}
```

### Recipe: Parse Custom Arguments with Validation

```rust
use clap::Parser;

#[derive(Parser)]
struct MyArgs {
    #[arg(long, value_parser = parse_port)]
    port: u16,
    
    #[arg(long, value_parser = validate_email)]
    email: String,
}

fn parse_port(s: &str) -> Result<u16, String> {
    s.parse::<u16>()
        .map_err(|_| "Port must be a valid u16".to_string())
        .and_then(|p| {
            if p > 1024 {
                Ok(p)
            } else {
                Err("Port must be > 1024".to_string())
            }
        })
}

fn validate_email(s: &str) -> Result<String, String> {
    if s.contains('@') {
        Ok(s.to_string())
    } else {
        Err("Invalid email format".to_string())
    }
}
```

### Recipe: Write Tests Following AAA Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_status_with_valid_name() {
        // Arrange
        let service_name = "api-server".to_string();
        let expected_name = service_name.clone();

        // Act
        let result = handle_service_status(service_name).await;

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        let value = output.to_json();
        assert_eq!(value["name"], expected_name);
        assert_eq!(value["healthy"], true);
    }

    #[tokio::test]
    async fn test_service_status_records_uptime() {
        // Arrange
        let uptime_start = std::time::Instant::now();

        // Act
        let result = handle_service_status("svc".to_string()).await;

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        let value = output.to_json();
        let uptime = value["uptime_seconds"].as_u64().unwrap();
        assert!(uptime > 0);
    }
}
```

### Recipe: Setup Logging in Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn init_logger() {
        let _ = env_logger::builder()
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_with_logging() {
        init_logger();
        log::info!("Test running with logging enabled");
        
        // Your test assertions here
        assert!(true);
    }
}
```

---

## Troubleshooting

### Build Issues

#### "error: could not compile `clap-noun-verb`"
- **Likely cause**: Missing or incompatible `clap` version
- **Solution**: Run `cargo update` and `cargo make clean`, then rebuild:
  ```bash
  cargo update
  cargo make clean
  cargo make build
  ```

#### "error[E0425]: cannot find function `verb` in this scope"
- **Likely cause**: Not importing the macro
- **Solution**: Add to your crate root:
  ```rust
  use clap_noun_verb::verb;
  ```

#### "error[E0308]: mismatched types in verb return"
- **Likely cause**: Verb function doesn't return `Result<HandlerOutput, ...>`
- **Solution**: Ensure your function signature is:
  ```rust
  pub async fn handle_xxx() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
  ```

### Runtime Issues

#### "CommandRouter failed to dispatch: verb not found"
- **Likely cause**: Verb name doesn't match registration name
- **Solution**: Verify the `#[verb(name = "...", noun = "...")]` matches the CLI args

#### "JSON serialization failed"
- **Likely cause**: Struct doesn't implement `Serialize`
- **Solution**: Add `#[derive(Serialize)]` to your struct:
  ```rust
  #[derive(Serialize)]
  struct MyOutput {
      field: String,
  }
  ```

#### "async runtime panic: no runtime found"
- **Likely cause**: Missing `#[tokio::main]` or `#[tokio::test]`
- **Solution**: Mark your main and test functions:
  ```rust
  #[tokio::main]
  async fn main() { }
  
  #[tokio::test]
  async fn test_something() { }
  ```

### Testing Issues

#### "test suite panicked: assertion failed"
- **Solution**: 
  1. Run with backtrace: `RUST_BACKTRACE=1 cargo test --quiet`
  2. Review the full assertion message
  3. Check that test follows AAA pattern

#### "tests take too long (>1 second)"
- **Likely cause**: Tests using I/O, network, or heavy computation
- **Solution**: 
  - Mock external dependencies with `mockall` or similar
  - Use `#[tokio::test]` for parallel execution
  - Check for `std::thread::sleep()` calls in tests

#### "test is flaky (passes sometimes, fails randomly)"
- **Solution**: Run deterministic tests to isolate:
  ```bash
  cargo make test-lib-deterministic
  ```
  - Look for thread-safety issues (shared mutable state)
  - Check for time-dependent logic
  - Remove any randomness sources in deterministic tests

### Feature-Related Issues

#### "error[E0433]: cannot find crate `xxx` in this scope"
- **Likely cause**: Feature-gated dependency not enabled
- **Solution**: Enable the feature in your `Cargo.toml`:
  ```toml
  [dependencies]
  clap-noun-verb = { version = "26.6.1", features = ["full"] }
  ```

#### "warning: unexpected `cfg` condition: `feature = \"my-feature\"`"
- **Likely cause**: Feature not declared in `Cargo.toml`
- **Solution**: Add to `[features]` section in `Cargo.toml`:
  ```toml
  [features]
  my-feature = []
  ```

### Formatting & Linting Issues

#### "error: code has incorrect formatting"
- **Solution**: Run formatter:
  ```bash
  cargo make format
  ```

#### "warning: function is never used"
- **Solution**: Either use the function or mark it:
  ```rust
  #[allow(dead_code)]
  fn unused_helper() { }
  ```

#### "clippy: use of unwrap_used"
- **Solution**: Replace with proper error handling:
  ```rust
  // Bad
  let value = result.unwrap();
  
  // Good
  let value = result.map_err(|e| NounVerbError::ExecutionFailed(e.to_string()))?;
  ```

### Git Issues

#### "error: pre-commit hook failed"
- **Likely cause**: Code doesn't pass lint checks
- **Solution**: Run checks locally first:
  ```bash
  cargo make lint
  cargo make format
  ```

#### "cannot rebase: history rewritten"
- **Solution**: Do NOT rebase. Merge instead:
  ```bash
  git merge main
  ```

---

## Performance Optimization

### Reducing Compile Time

#### 1. Check Incremental Build Performance
Current SLO: <=2 seconds

```bash
# Touch a file and measure
touch src/lib.rs
time cargo make build
```

If incremental builds exceed 2s:

#### 2. Identify Slow Dependencies
```bash
# Check build timeline
cargo build --release -Z timings

# Check dependency graph
cargo tree | grep -E "^[├└].*\(."
```

#### 3. Reduce Dependency Features
In `Cargo.toml`, minimize feature usage:
```toml
[dependencies]
# Bad: enables many features
serde = "1.0"

# Good: only needed features
serde = { version = "1.0", features = ["derive"] }
```

#### 4. Use Workspace Opt Levels
In `Cargo.toml`:
```toml
[profile.dev.package."*"]
opt-level = 2  # Optimize dependencies in debug builds

[profile.dev]
opt-level = 0  # But not our crate (for faster iteration)
```

#### 5. Use `cargo-nextest` for Faster Tests
```bash
cargo install cargo-nextest
cargo nextest run
```

### Reducing Binary Size

Current SLO: <=10MB

#### 1. Build with Release Profile
```bash
cargo make build-release
ls -lh target/release/myapp
```

#### 2. Strip Symbols
```bash
strip target/release/myapp
```

#### 3. Enable LTO
In `Cargo.toml`:
```toml
[profile.release]
lto = true
codegen-units = 1
```

#### 4. Remove Unused Features
Check what's actually used:
```bash
cargo tree --duplicates
```

Remove unused features from transitive dependencies.

#### 5. Check Binary Composition
```bash
cargo install cargo-bloat
cargo bloat --release -n 20
```

### Memory Usage Optimization

#### 1. Profile Memory Usage
```bash
cargo install valgrind
valgrind --leak-check=full ./target/debug/myapp
```

#### 2. Avoid Large Stack Allocations
```rust
// Bad: allocates large struct on stack
let big: [u8; 1_000_000] = [0; 1_000_000];

// Good: allocate on heap
let big = vec![0u8; 1_000_000];
```

#### 3. Use References in Hot Paths
```rust
// Bad: clones on each call
pub fn process(data: Vec<u8>) { }

// Good: borrows
pub fn process(data: &[u8]) { }
```

### Benchmarking

To identify performance bottlenecks:

```bash
cargo make bench
```

Create benchmark in `benches/my_bench.rs`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_verb_execution(c: &mut Criterion) {
    c.bench_function("execute_verb", |b| {
        b.iter(|| {
            black_box(handle_my_verb(black_box("input".to_string())))
        })
    });
}

criterion_group!(benches, benchmark_verb_execution);
criterion_main!(benches);
```

### Caching & Memoization

For expensive operations in verbs:

```rust
use std::sync::OnceLock;

static CACHED_CONFIG: OnceLock<Config> = OnceLock::new();

#[verb(name = "status", noun = "app")]
pub async fn handle_app_status() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    let config = CACHED_CONFIG.get_or_init(|| Config::load().unwrap());
    // Use config
    Ok(HandlerOutput::new(json!({})))
}
```

---

## Contributing Guidelines

### Before You Start

1. **Check existing issues** to avoid duplicate work
2. **Discuss major features** in an issue before implementing
3. **Follow the branch naming convention**:
   - Feature: `feat/description-of-feature`
   - Bugfix: `fix/description-of-bug`
   - Refactor: `refactor/description`
   - Docs: `docs/description`

### Code Quality Standards

#### 1. Error Handling
- **NEVER use**: `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`
- **ALWAYS use**: `Result<T>`, `?` operator, `map_err()`
- Clippy enforces these rules via deny lints

#### 2. Testing Requirements
- **Every public function** must have tests
- **Follow AAA pattern**: Arrange, Act, Assert
- **Test behaviors, not implementations**: Verify outputs/side-effects, not code paths
- **Avoid trivial assertions**: Don't just check `is_ok()`, verify actual values
- **Descriptive names**: `test_verb_creates_output_with_valid_input` not `test_it_works`

Example:
```rust
#[test]
fn test_parse_url_with_valid_scheme() {
    // Arrange
    let url = "https://example.com";
    
    // Act
    let result = parse_url(url);
    
    // Assert
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.scheme, "https");
    assert_eq!(parsed.host, "example.com");
}
```

#### 3. Documentation
- **Document public APIs** with doc comments:
  ```rust
  /// Processes input and returns formatted output.
  ///
  /// # Arguments
  /// * `input` - The input data to process
  ///
  /// # Returns
  /// A Result containing the formatted output or an error
  ///
  /// # Examples
  /// ```
  /// let result = process("data".to_string())?;
  /// ```
  pub fn process(input: String) -> Result<String, MyError> {
  ```

- **Include examples** in doc comments for public functions
- **Document panics** if any (should be none!)
- **Document safety concerns** for unsafe code (avoid when possible)

#### 4. Formatting & Style
- **Always run** `cargo make format` before committing
- **Always run** `cargo make lint` before pushing
- Line width: 100 characters (enforced by rustfmt)
- Tab size: 4 spaces

#### 5. Licensing
- **All contributions** must be under permissive licenses: MIT, Apache-2.0, BSD, ISC
- **No copyleft**: AGPL, GPL, LGPL are denied by `deny.toml`
- **Check new dependencies**: `cargo deny check`

### Commit Guidelines

1. **Commit frequently** with atomic, logical changes
2. **Write clear commit messages**:
   ```
   <type>: <subject>
   
   <body explaining why, not what>
   ```
   
   Example:
   ```
   feat: add services status verb command
   
   This enables users to check the health status of all registered
   services with a single command. The verb returns JSON output
   compatible with CI/CD pipelines for automated health checks.
   ```

3. **Types**: `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `perf`
4. **Subject**: lowercase, no period, <50 characters
5. **Body**: explain WHY, not WHAT; wrap at 72 characters

### Pull Request Process

1. **Create a focused PR** - one feature or fix per PR
2. **Self-review first** before requesting review:
   ```bash
   git diff main...HEAD
   cargo make ci
   ```

3. **Write a clear PR description**:
   ```markdown
   ## Summary
   Add new `services status` verb for health monitoring
   
   ## Changes
   - New verb `services status` returns service health
   - Adds JSON serializable ServiceStatus struct
   - 3 new tests covering happy path and edge cases
   
   ## Testing
   - [ ] Tested with `cargo make test`
   - [ ] Tested with `cargo make test-all`
   - [ ] Manual testing: `cargo run -- services status`
   
   ## Checklist
   - [x] Code follows style guidelines
   - [x] Tests added/updated
   - [x] Documentation updated
   - [x] No breaking changes
   ```

4. **Link related issues**: "Closes #123"
5. **Request review** from maintainers

### Code Review Expectations

When your PR is reviewed:

- **Be receptive to feedback** - reviewers are helping you
- **Respond to all comments** - don't leave threads unresolved
- **Push fixes as new commits** - don't amend/rebase
- **Request re-review** after addressing feedback
- **Maintain the merge protocol**: Merge only (never rebase/force-push)

### Reporting Bugs

Include:
1. **Clear description** of what's broken
2. **Steps to reproduce** - exact commands
3. **Expected behavior** - what should happen
4. **Actual behavior** - what happens instead
5. **Environment**: Rust version, OS, feature flags
6. **Minimal reproduction** - isolated code example

Example:
```markdown
**Bug**: Verb fails silently on malformed JSON input

**Steps**:
1. Run: `myapp data process "{invalid"` 
2. Observe: No error message, exits with code 0

**Expected**: Clear error message about invalid JSON, exit code 1

**Actual**: Silent failure, exit code 0

**Environment**: 
- Rust 1.75
- Ubuntu 22.04
- clap-noun-verb v26.6.1
```

### Asking Questions

Before asking:
- Check the CLAUDE.md documentation (you're reading it!)
- Search existing issues for similar questions
- Review relevant source code

Ask on:
- **GitHub Issues** for bugs/features
- **Discussions** for how-to questions
- **PRs** for code review feedback

---

## Architecture Decision Log (ADL)

### ADL-001: Noun-Verb Command Pattern

**Status**: Implemented (v1.0+)

**Decision**: Use a noun-verb pattern (e.g., `myapp services status`) instead of flat commands or full nesting.

**Rationale**:
- More intuitive for users coming from cloud CLIs (AWS, GCP, Azure patterns)
- Scales better than flat commands as more features are added
- Allows logical grouping of related operations (all service commands under `services`)
- Reduces command name collisions

**Alternatives Considered**:
1. Flat commands: `myapp status-services` - less intuitive
2. Full nesting: `myapp config services credentials delete` - overly complex
3. Single-level: `myapp services` (no verbs) - insufficient expressiveness

**Trade-offs**:
- Requires users to learn two levels of commands
- Slightly more complex internal routing

**Related Files**:
- `src/router.rs` - CommandRouter implementation
- `src/registry.rs` - CommandRegistry tracks noun-verb relationships

---

### ADL-002: Proc-Macro Over Derive-Based Registration

**Status**: Implemented (v1.0+)

**Decision**: Use `#[verb]` and `#[noun]` proc-macros for declarative registration with `linkme` distributed slices.

**Rationale**:
- Compile-time discovery: no runtime registration overhead
- Distributed slices eliminate the need for a central registry module
- Macros reduce boilerplate: single annotation instead of manual registration
- Errors caught at compile time, not runtime

**Alternatives Considered**:
1. Runtime registration: User calls `registry.register()` - runtime overhead, easy to forget
2. Convention-based (e.g., module names) - magic, hard to debug
3. Attribute derives on impl blocks - more complex macro code
4. YAML/TOML config files - external dependency, harder to keep in sync

**Trade-offs**:
- Macro code is harder to understand than straightforward Rust
- Requires `linkme` as a dependency
- Compilation might be slightly slower due to macro expansion

**Related Files**:
- `clap-noun-verb-macros/src/lib.rs` - `#[verb]` macro
- `src/cli/mod.rs` - CommandRegistry uses `linkme` distributed slices

---

### ADL-003: JSON-First Output Format

**Status**: Implemented (v1.0+)

**Decision**: Default output format is JSON; other formats (YAML, text) are optional.

**Rationale**:
- JSON is machine-parseable for scripting and CI/CD integration
- Standard in modern APIs and cloud CLIs
- Serializable types automatically give free JSON support
- Compatible with `jq` for filtering/transformation
- Agents and automation tools expect JSON

**Alternatives Considered**:
1. Human-readable text by default - less machine-friendly
2. YAML by default - less widely supported in scripts
3. Format negotiation - adds complexity

**Trade-offs**:
- Verbosity: JSON output is larger than human-readable text
- Readability: Less convenient for direct human consumption (users should pipe to `jq` or format tools)

**Related Files**:
- `src/format.rs` - OutputFormatter implementation
- `src/logic/mod.rs` - HandlerOutput always serializes to JSON

---

### ADL-004: Async-First Verbs with Feature-Gate for Sync

**Status**: Implemented (v2.0+)

**Decision**: Verbs are async by default; sync support is feature-gated.

**Rationale**:
- Modern Rust prefers async for I/O-bound operations (network, file, DB)
- Tokio is ubiquitous, adding minimal overhead
- Allows verbs to perform concurrent operations (e.g., parallel API calls)
- Sync-only verbs become a legacy concern; feature-gate reduces surface area

**Alternatives Considered**:
1. Sync-first with optional async - forces users to use `block_on()` wrappers
2. Support both equally - API explosion, complex routing
3. Sync-only - poor performance for modern workloads

**Trade-offs**:
- Users must understand async/await if they write custom verbs
- Adds dependency on Tokio (unavoidable for modern async)

**Implementation**:
- `#[verb]` generates async functions by default
- `#[verb(sync)]` for sync verbs (requires `async` feature disabled)

**Related Files**:
- `src/async_verb.rs` - Async verb handling
- `clap-noun-verb-macros/src/lib.rs` - Macro handles both async and sync

---

### ADL-005: No Panics in Production Code

**Status**: Enforced (v3.0+)

**Decision**: Panic-like operations (`unwrap()`, `expect()`, `panic!()`, `todo!()`) are forbidden in production code via Clippy deny lints.

**Rationale**:
- Panics crash the entire application; libraries should never panic
- CLIs need graceful error handling for user-facing failures
- `Result<T>` and `?` operator provide better error information
- Consistent error handling across the codebase

**Alternatives Considered**:
1. Allow panics - easier to write, but crashes on edge cases
2. Soft lint warnings - too easy to ignore
3. Runtime checks - too late to catch

**Trade-offs**:
- More verbose error handling code
- Requires developers to think about error cases upfront
- Impossible to use `unwrap()` even in obviously-safe cases (e.g., const initialization)

**Enforcement**:
```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
exit = "deny"
```

**Related Files**:
- `Cargo.toml` - Lint configuration
- `src/error.rs` - NounVerbError type

---

### ADL-006: Feature-Gated Experimental Modules

**Status**: Implemented (v5.4+)

**Decision**: Experimental/frontier features are feature-gated to keep the core minimal (10 core dependencies).

**Rationale**:
- Default crate remains lightweight and fast to compile
- Users opt-in to experimental features
- Unstable code doesn't affect the stable API
- Easy to iterate on frontier ideas without destabilizing releases

**Feature Categories**:
1. **Core** (always included): CLI framework, verb routing
2. **Optional** (feature flags): `async`, `federated-network`
3. **Frontier** (experimental): `meta-framework`, `executable-specs`, `fractal-patterns`, etc.
4. **Meta** (convenience): `frontier-all` enables multiple frontier features

**Trade-offs**:
- Code duplication if stable features want to use frontier features
- Users must understand feature flags to enable experimental functionality

**Related Files**:
- `Cargo.toml` - Feature definitions
- `src/lib.rs` - Conditional module inclusion

---

### ADL-007: Minimalist Core After v26 Refactor

**Status**: Implemented (v26.0+)

**Decision**: Remove non-core optional modules (`io/`, `kernel/`, `rdf/`, `semantic/`, `agents/`, `wizard/`, `plugin/`, `middleware/`, `integration/`) from the main crate; preserve frontier feature declarations in Cargo.toml for forward compatibility.

**Rationale**:
- Maintenance burden: large number of optional modules
- Unclear ownership: some modules were exploratory/dead code
- Compilation time: optional modules add to check time even when unused
- Focus: clarify what the core library does (noun-verb CLI framework)
- Frontier features can be experimental without bloating the main crate

**What Remains**:
- Core CLI framework (verb routing, command registry, output formatting)
- One optional module: `async_verb.rs` (essential for async support)
- One optional module: `federation/` (for `federated-network` feature)

**Alternatives Considered**:
1. Keep all modules - easier for users, harder to maintain
2. Remove frontier features entirely - lose experimental capabilities
3. Separate into multiple crates - more to publish, harder to coordinate versions

**Trade-offs**:
- Users who depended on removed modules must migrate or fork
- Frontier features remain as Cargo.toml declarations without implementations
- Clearer but smaller scope for the main crate

**Related Files**:
- `Cargo.toml` - Feature declarations remain for forward compatibility
- `src/lib.rs` - Only core and two optional modules are included

---

### ADL-008: Distributed Slices for Verb Registration

**Status**: Implemented (v1.0+)

**Decision**: Use `linkme::distributed_slice!` to automatically discover and register verbs at compile time.

**Rationale**:
- **Decentralized**: Each verb module declares its own registration; no central registry
- **Zero runtime cost**: Discovery happens at link time, not startup
- **Compile-time safety**: Errors if registration is malformed
- **Easy to add**: Single macro attribute, automatic discovery

**How It Works**:
1. `#[verb]` macro generates a `distributed_slice!` entry
2. `CommandRegistry` collects entries via the distributed slice
3. No explicit "register with CommandRegistry" call needed

**Alternatives Considered**:
1. Manual registration: `registry.register(verb)` - error-prone, boilerplate
2. Reflection/type registry - requires heap allocation, runtime cost
3. YAML/TOML registry file - external file, out-of-sync risk
4. Convention-based discovery - magic, hard to debug

**Trade-offs**:
- Requires `linkme` dependency
- Distributed slice behavior is not obvious to new users
- Debugging registration issues requires understanding linker behavior

**Related Files**:
- `clap-noun-verb-macros/src/lib.rs` - Macro generates distributed_slice entry
- `src/registry.rs` - CommandRegistry collects distributed_slice entries

---

### ADL-009: SLO Targets (Compilation & Binary Size)

**Status**: Implemented (v25.0+)

**Decision**: Enforce SLO targets to keep the framework lightweight and developer-friendly.

**Rationale**:
- Incremental compilation <=2s: Developers iterate quickly
- Binary size <=10MB: Easy to distribute, reasonable in containers
- These targets align with industry standards for CLI tools

**Current Performance**:
- Incremental compilation: 0.66s (well within SLO)
- Binary size: 2.2MB (well within SLO)

**Enforcement**:
- CI checks against these targets
- Performance regressions must be justified and approved
- Developers should periodically profile and optimize

**Trade-offs**:
- Constraints on dependency additions (can't add heavy crates)
- Requires conscious effort to maintain performance as features grow

**Related Files**:
- `Makefile.toml` - CI includes compilation and size checks
- `benches/` - Benchmarks track performance over time

---

### ADL-010: Trait Design: Sync-Only, dyn Compatible

**Status**: Implemented (v2.0+)

**Decision**: Trait methods are synchronous and trait objects (`dyn Trait`) are supported; async is handled via feature-gated module.

**Rationale**:
- Async in traits complicates object safety and lifetime bounds
- Keep the core API simple for library users
- Async verbs are handled in `async_verb.rs` module (feature-gated)
- Library code should be flexible; async is an implementation detail

**Constraints**:
- `NounCommand`, `VerbCommand` traits are object-safe
- No `async` methods in core traits
- No lifetimes beyond `'static` in trait method signatures

**Example**:
```rust
// Good: sync, object-safe
pub trait NounCommand {
    fn name(&self) -> &'static str;
    fn execute(&self, args: &[String]) -> Result<Output>;
}

// Bad: async, not object-safe
pub trait NounCommand {
    async fn name(&self) -> &'static str;
}
```

**Trade-offs**:
- Users needing async must use the `async_verb.rs` module or wrap in async runtime
- Slightly more manual setup for async scenarios

**Related Files**:
- `src/noun.rs` - `NounCommand` trait definition
- `src/verb.rs` - `VerbCommand` trait definition
- `src/async_verb.rs` - Async handling module

---

## Glossary

- **Noun**: A logical grouping of related commands (e.g., `services`, `database`)
- **Verb**: An action or operation (e.g., `status`, `create`, `delete`)
- **Handler**: The function implementing a verb's logic
- **CommandRegistry**: Central registry collecting all verbs at startup
- **CliBuilder**: API for constructing the clap `Command` tree
- **CommandRouter**: Dispatches parsed CLI args to the appropriate handler
- **Distributed Slice**: Compile-time collection mechanism via `linkme`
- **HandlerInput**: Input types passed to verb handlers
- **HandlerOutput**: Output types returned from verb handlers
- **Verb Macro**: `#[verb]` attribute for declarative verb registration
- **Frontier Features**: Experimental/unstable feature set (v5.4+)
