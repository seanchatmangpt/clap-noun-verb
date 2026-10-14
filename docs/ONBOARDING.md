# clap-noun-verb Onboarding Guide

Welcome to the **clap-noun-verb** project! This guide will take you from zero to productive contributor in stages, with clear milestones and checkpoints.

**Estimated time to complete full guide:** 2-3 hours  
**Estimated time to make your first contribution:** 1-2 hours

---

## Table of Contents

1. [Prerequisites & Setup](#prerequisites--setup)
2. [Environment Setup](#environment-setup)
3. [Project Orientation](#project-orientation)
4. [First Contribution](#first-contribution)
5. [Code Walkthrough](#code-walkthrough)
6. [Testing Strategy](#testing-strategy)
7. [Communication](#communication)
8. [Workflow](#workflow)
9. [Performance Expectations](#performance-expectations)
10. [Success Metrics](#success-metrics)

---

## Prerequisites & Setup

### What You Need

**Essential:**
- **Rust 1.74+** (check with `rustc --version`)
- **Git** (for version control)
- **A text editor or IDE** (VS Code recommended, see IDE setup below)
- **Basic CLI familiarity** (terminal, git commands)

**Optional but Helpful:**
- Familiarity with Rust procedural macros (we'll explain as needed)
- Experience with CLI tools (clap, Typer, Click)
- Understanding of JSON and serialization

### Rust Installation

If you don't have Rust installed, use **rustup**:

```bash
# Install Rust (macOS/Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Rust (Windows)
# Download installer from https://rustup.rs/

# Verify installation
rustc --version  # Should be 1.74+
cargo --version
```

### Required Tools

```bash
# cargo-make (task runner, required by project)
cargo install cargo-make

# Recommended tools
cargo install cargo-deny       # License compliance checker
cargo install cargo-outdated   # Dependency upgrade assistant
```

### IDE Setup

**VS Code (Recommended)**

1. Install Rust Analyzer extension: `rust-lang.rust-analyzer`
2. Install CodeLLDB extension (optional, for debugging): `vadimcn.vscode-lldb`
3. Add to `.vscode/settings.json`:
   ```json
   {
       "rust-analyzer.checkOnSave.command": "clippy",
       "[rust]": {
           "editor.formatOnSave": true,
           "editor.defaultFormatter": "rust-lang.rust-analyzer"
       }
   }
   ```

**Other IDEs:**
- **IntelliJ IDEA**: Use Rust plugin from JetBrains
- **Neovim**: Use rust-analyzer with nvim-lsp
- **Vim**: Use vim-rust or similar

### Check Your Setup

Run this command to verify everything is ready:

```bash
# Verify Rust version
rustc --version

# Verify Cargo Make is installed
cargo make --version

# Clone the project (if not already done)
git clone https://github.com/seanchatmangpt/clap-noun-verb.git
cd clap-noun-verb
```

**Expected output:**
```
rustc 1.74.0 (or later)
cargo-make 0.37.0 (or later)
```

---

## Environment Setup

### Clone & Initialize Repository

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/clap-noun-verb.git
cd clap-noun-verb

# Create your local branch (don't work on main!)
git checkout -b feat/your-feature-name
```

### Cargo Workspace Setup

The project is a **Cargo workspace** with multiple crates:

```
clap-noun-verb/
├── clap-noun-verb-macros/     # Proc-macro crate (published first)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── validation.rs
│   │   └── io_detection.rs
│   └── Cargo.toml
├── src/                         # Main library crate
│   ├── lib.rs
│   ├── cli/
│   ├── logic/
│   ├── error.rs
│   └── ...
├── examples/                    # Learn-by-example code
├── tests/                       # Integration and unit tests
├── Cargo.toml                   # Workspace manifest
└── Makefile.toml               # Task definitions
```

**Important:** Always use `cargo make` instead of `cargo` directly (see Build Commands below).

### First Build

```bash
# Download dependencies and build
cargo make build

# Run a quick test to verify everything works
cargo make test-lib-deterministic
```

**Expected output:**
```
Compiling clap-noun-verb-macros v26.9.1
Compiling clap-noun-verb v26.9.1
Finished dev [unoptimized + debuginfo] in X.XXs
```

If build fails:
- Check Rust version: `rustc --version` (need 1.74+)
- Clear cache: `cargo clean && cargo make build`
- Check internet connection (first build downloads ~50MB of deps)

### IDE Configuration

**Open VS Code at project root:**

```bash
code .
```

**Configure Rust Analyzer:**
1. Open command palette: `Ctrl+Shift+P` (macOS: `Cmd+Shift+P`)
2. Type `Rust Analyzer: Restart`
3. Wait for Rust Analyzer to initialize (watch status bar)

**Expected behavior:**
- Hover over identifiers → see type information
- `Ctrl+Click` on functions → jump to definition
- Save → auto-format code
- Build errors appear in Problems panel

### Verification Checklist

- [ ] Rust version 1.74+: `rustc --version`
- [ ] Cargo Make installed: `cargo make --version`
- [ ] Project builds: `cargo make build`
- [ ] Quick tests pass: `cargo make test-lib-deterministic`
- [ ] IDE opens without errors
- [ ] Can hover over code in editor and see types

---

## Project Orientation

### What is clap-noun-verb?

**clap-noun-verb** is a Rust CLI framework that:

1. **Simplifies CLI design** with noun-verb command patterns (e.g., `myapp services status`)
2. **Uses proc-macros** for declarative command registration (`#[verb]` attributes)
3. **Auto-discovers commands** at compile time using `linkme` distributed slices
4. **Outputs JSON by default** for integration with scripts and agents
5. **Stays minimal** — only 10 core dependencies by default

**Real-world example:**
```bash
# Instead of: myapp status_services
# Or: myapp services-status

# You write: myapp services status
#            ^noun ^verb
```

### Project Goals

- **Productivity**: Developers add commands with a single macro
- **Safety**: Errors caught at compile time, not runtime
- **Performance**: Incremental compilation <=2s, binary <=10MB
- **Reliability**: No panics in production code (enforced by Clippy)

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│ Your Rust Code                                          │
│ #[verb(name="status", noun="services")]                │
│ pub fn handle_services_status() -> Result<...> { }     │
└────────────────────┬────────────────────────────────────┘
                     │ Proc-macro expansion
┌────────────────────▼────────────────────────────────────┐
│ clap-noun-verb-macros (clap-noun-verb-macros/src/)     │
│ • Validates your code at compile time                  │
│ • Generates linkme distributed_slice entry             │
│ • Checks return types are Serializable                 │
└────────────────────┬────────────────────────────────────┘
                     │ Link-time discovery
┌────────────────────▼────────────────────────────────────┐
│ Main Library (src/)                                    │
│ • CommandRegistry - collects all verbs                 │
│ • CliBuilder - constructs clap Command tree            │
│ • CommandRouter - dispatches to handlers               │
│ • Output formatting to JSON                            │
└────────────────────┬────────────────────────────────────┘
                     │ Runtime
┌────────────────────▼────────────────────────────────────┐
│ Your CLI Application                                    │
│ $ myapp services status                                │
│ {"status": "healthy", "timestamp": "2026-06-14..."}    │
└─────────────────────────────────────────────────────────┘
```

### Key Files & Directories

**Essential to Know:**

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Public API exports (start here!) |
| `src/cli/mod.rs` | Entry point `run()` function |
| `src/cli/registry.rs` | Verb auto-discovery via linkme |
| `src/cli/router.rs` | Command dispatch logic |
| `src/verb.rs` | `VerbCommand` trait definition |
| `src/noun.rs` | `NounCommand` trait definition |
| `src/logic/mod.rs` | `HandlerInput`/`HandlerOutput` types |
| `src/error.rs` | `NounVerbError` type |
| `src/format.rs` | JSON output formatting |
| `clap-noun-verb-macros/src/lib.rs` | `#[verb]` macro implementation |
| `clap-noun-verb-macros/src/validation.rs` | Compile-time checks |
| `examples/` | Working examples you can run |
| `Makefile.toml` | Build task definitions |
| `Cargo.toml` | Workspace configuration |
| `CLAUDE.md` | Development guidelines (read this!) |

**Understand This Flow:**

1. **Macro time** (`clap-noun-verb-macros/src/lib.rs`):
   - Your `#[verb]` function is parsed
   - Return type checked for `Serialize`
   - Generates registration entry

2. **Compile time** (`src/lib.rs` + `linkme`):
   - All registration entries collected
   - Verified no duplicates
   - Linked into the binary

3. **Runtime** (`src/cli/mod.rs`):
   - `CommandRegistry::new()` reads all registered verbs
   - `CliBuilder::build_from_registry()` creates clap Command tree
   - `CommandRouter::route()` dispatches to handler

### Codebase Tour

**Start here:**

```bash
# Open and read these in order (10 min each)
code src/lib.rs              # Public API, module exports
code src/verb.rs             # VerbCommand trait
code src/noun.rs             # NounCommand trait
code src/logic/mod.rs        # HandlerInput/Output types
```

**Then explore examples:**

```bash
# Run a working example
cargo run --example ref_framework -- --help

# Run a simpler tutorial example
cargo run --example tutorial_basic -- --help
```

**Understand the macros:**

```bash
# Read the macro code (40 min)
code clap-noun-verb-macros/src/lib.rs
code clap-noun-verb-macros/src/validation.rs

# Bonus: See macro expansion
cargo expand --lib | head -100
```

### Build Commands

**Always use `cargo make` (never direct `cargo` unless testing):**

| Task | Command | Time |
|------|---------|------|
| Quick check | `cargo make check` | 2-5s |
| Format check | `cargo make format-check` | 3s |
| Lint | `cargo make lint` | 5s |
| Full lint | `cargo make lint-all` | 10s |
| **Test (quick)** | `cargo make test` | **5-10s** |
| Test (single-threaded) | `cargo make test-lib-deterministic` | 10s |
| Test (all features) | `cargo make test-all` | 20s |
| Build debug | `cargo make build` | 5-10s |
| Build release | `cargo make build-release` | 15-20s |
| Generate docs | `cargo make doc` | 10s |
| Full CI | `cargo make ci` | 45s |

**Typical workflow:**

```bash
# Make a change
nano src/some_file.rs

# Quick validation
cargo make format-check
cargo make clippy
cargo make test

# Before pushing
cargo make lint
cargo make test-lib-deterministic
```

---

## First Contribution

### Milestone 1: Find an Easy Starting Issue (15 min)

**Goal:** Pick a small, scoped task to understand the workflow.

**Look for issues labeled:**
- `good first issue`
- `help wanted`
- `documentation`
- `beginner-friendly`

**On GitHub:**
```
https://github.com/seanchatmangpt/clap-noun-verb/issues?q=label:"good+first+issue"
```

**If no issues yet, pick one of these starter tasks:**

1. **Add a missing test** (10 min)
   - Pick a public function with no test
   - Write a simple unit test following AAA pattern
   - Run `cargo test test_name --quiet`

2. **Improve documentation** (20 min)
   - Add missing doc comment to a public function
   - Include an example in the doc comment
   - Run `cargo make doc` and check it renders

3. **Fix a clippy warning** (15 min)
   - Run `cargo make clippy`
   - Pick a warning marked "allow"
   - Fix the underlying issue
   - Run tests to verify

4. **Add an example** (30 min)
   - Create `examples/tutorial_myfeature.rs`
   - Demonstrate a specific feature
   - Add to `Cargo.toml` as `[[example]]`
   - Run `cargo run --example tutorial_myfeature -- --help`

### Milestone 2: Create a Test (30 min)

**Task:** Write a test for `CommandRouter` dispatch.

1. **Find the module** to test:
   ```bash
   # Open the router module
   code src/cli/router.rs
   ```

2. **Understand the code:**
   - What does `CommandRouter` do?
   - What are public methods?
   - What inputs/outputs?

3. **Write a test:**
   ```bash
   # Create tests/my_first_test.rs
   cat > tests/my_first_test.rs << 'EOF'
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_command_router_basic() {
           // Arrange
           // (Set up your test data)
           
           // Act
           // (Call the function being tested)
           
           // Assert
           // (Verify the output)
           assert!(true); // TODO: Replace with real assertion
       }
   }
   EOF
   ```

4. **Run the test:**
   ```bash
   cargo test my_first_test --quiet
   ```

5. **Commit it:**
   ```bash
   git add tests/my_first_test.rs
   git commit -m "test: add basic command router test

   This test verifies that CommandRouter can dispatch
   a simple command to its handler without errors."
   ```

### Milestone 3: Make a Small Code Change (1 hour)

**Task:** Add a new helper function with tests.

1. **Pick a location:**
   ```bash
   # Pick a small module (not core router/registry)
   code src/logic/mod.rs
   # Or add to an existing test file
   code tests/
   ```

2. **Add the function:**
   ```rust
   /// Helper to check if a command name is valid
   ///
   /// A valid command name contains only alphanumeric chars, hyphens, underscores.
   ///
   /// # Examples
   /// ```
   /// assert!(is_valid_command_name("my-command"));
   /// assert!(!is_valid_command_name("my command")); // space not allowed
   /// ```
   pub fn is_valid_command_name(name: &str) -> bool {
       !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
   }
   ```

3. **Write tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_valid_command_name_with_hyphens() {
           assert!(is_valid_command_name("my-command"));
       }

       #[test]
       fn test_invalid_command_name_with_spaces() {
           assert!(!is_valid_command_name("my command"));
       }

       #[test]
       fn test_invalid_empty_name() {
           assert!(!is_valid_command_name(""));
       }
   }
   ```

4. **Run tests:**
   ```bash
   cargo make test
   ```

5. **Format and lint:**
   ```bash
   cargo make format
   cargo make lint
   ```

6. **Commit:**
   ```bash
   git add -A
   git commit -m "feat: add command name validation helper

   Adds is_valid_command_name() to validate CLI command names
   against allowed character set (alphanumeric, hyphens, underscores).
   Includes tests for valid and invalid cases."
   ```

### Milestone 4: Submit a Pull Request (1 hour)

**Checklist before submitting:**

```bash
# 1. Make sure tests pass
cargo make test
cargo make test-lib-deterministic

# 2. Run linters
cargo make format
cargo make lint

# 3. Verify no uncommitted changes except intentional ones
git status

# 4. Check your commits are clean
git log --oneline main..HEAD
```

**Create the PR:**

```bash
# Push your branch (first time)
git push -u origin feat/your-feature-name

# Go to GitHub and create PR
# https://github.com/seanchatmangpt/clap-noun-verb/pulls
```

**PR Description Template:**

```markdown
## Summary
Brief description of what this PR does (1-2 sentences)

## Changes
- Bullet point 1
- Bullet point 2
- Bullet point 3

## Testing
- [ ] Tested with `cargo make test`
- [ ] Tested with `cargo make lint`
- [x] Example: `cargo run --example X -- --help`

## Checklist
- [x] Code follows style guidelines
- [x] Tests added/updated
- [x] Documentation updated
- [x] No breaking changes

Closes #XXX (if applicable)
```

---

## Code Walkthrough

### How a Verb Gets Registered

**You write:**
```rust
#[verb(name = "status", noun = "services")]
pub async fn handle_services_status(
    service_name: String,
) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    Ok(HandlerOutput::new(json!({
        "status": "healthy"
    })))
}
```

**Macro expands to something like:**
```rust
pub async fn handle_services_status(...) -> Result<...> { ... }

// Generated by macro:
#[linkme::distributed_slice(clap_noun_verb::VERBS)]
static REGISTER_SERVICES_STATUS: clap_noun_verb::VerbRegistration = 
    clap_noun_verb::VerbRegistration {
        noun: "services",
        verb: "status",
        handler: wrap(handle_services_status),
    };
```

**At startup:**
```rust
// In src/cli/mod.rs, function run()
let registry = CommandRegistry::new();
// CommandRegistry::new() collects ALL distributed_slice entries
// ↓
let app = CliBuilder::new("myapp")
    .build_from_registry(&registry)?;
// CliBuilder builds a clap Command tree
// ↓
let matches = app.get_matches();
// Clap parses: myapp services status
// ↓
let router = CommandRouter::from_registry(&registry);
router.route(&matches).await?;
// CommandRouter finds handler and calls it
```

### Key Types and Traits

**`VerbCommand` trait** (`src/verb.rs`):
```rust
pub trait VerbCommand {
    /// Name of this verb (e.g., "status")
    fn name(&self) -> &'static str;
    
    /// Optional description for help text
    fn description(&self) -> &'static str {
        ""
    }
    
    /// Execute the verb with parsed arguments
    async fn execute(&self, input: HandlerInput) -> Result<HandlerOutput>;
}
```

**`NounCommand` trait** (`src/noun.rs`):
```rust
pub trait NounCommand {
    /// Name of this noun (e.g., "services")
    fn name(&self) -> &'static str;
    
    /// Optional description
    fn description(&self) -> &'static str {
        ""
    }
}
```

**`HandlerInput` type** (`src/logic/mod.rs`):
```rust
pub struct HandlerInput {
    /// Parsed CLI arguments
    pub args: HashMap<String, serde_json::Value>,
    
    /// Context (env vars, working directory, etc.)
    pub context: Context,
}
```

**`HandlerOutput` type** (`src/logic/mod.rs`):
```rust
pub struct HandlerOutput {
    /// The value to output (always JSON)
    value: serde_json::Value,
    
    /// Optional metadata
    metadata: Option<HashMap<String, String>>,
}
```

**`CommandRegistry` type** (`src/registry.rs`):
```rust
pub struct CommandRegistry {
    /// All registered verbs
    verbs: Vec<VerbRegistration>,
    
    /// All registered nouns
    nouns: Vec<NounRegistration>,
}

impl CommandRegistry {
    /// Discover all verbs and nouns from distributed slices
    pub fn new() -> Self { /* ... */ }
    
    /// Get verb by name
    pub fn get_verb(&self, noun: &str, verb: &str) -> Option<&VerbRegistration> { /* ... */ }
}
```

### Control Flow Diagram

```
User runs: $ myapp services status --verbose
                  ↓
         clap parses arguments
                  ↓
         Matches: {
             "noun": "services",
             "verb": "status",
             "--verbose": true
         }
                  ↓
         CommandRouter looks up handler
                  ↓
         Creates HandlerInput from parsed args
                  ↓
         Calls handler function:
         handle_services_status(input)
                  ↓
         Handler returns Result<HandlerOutput, _>
                  ↓
         Output formatting to JSON
                  ↓
         Prints to stdout:
         {
             "status": "healthy",
             ...
         }
```

### Common Patterns

**Pattern 1: Simple verb with no arguments**
```rust
#[verb(name = "version")]
pub async fn cmd_version() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    Ok(HandlerOutput::new(json!({
        "version": "1.0.0"
    })))
}
```

**Pattern 2: Verb with required argument**
```rust
#[verb(name = "greet", noun = "user")]
pub async fn cmd_greet(
    name: String,  // Required positional argument
) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    Ok(HandlerOutput::new(json!({
        "greeting": format!("Hello, {}", name)
    })))
}

// Usage: myapp user greet Alice
```

**Pattern 3: Verb with optional flags**
```rust
#[verb(name = "list", noun = "services")]
pub async fn cmd_list(
    #[arg(long)]
    filter: Option<String>,  // Optional flag: --filter <value>
    
    #[arg(short)]
    verbose: bool,  // Boolean flag: -v / --verbose
) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    Ok(HandlerOutput::new(json!({
        "services": vec!["api", "db", "cache"],
        "filter": filter,
        "verbose": verbose
    })))
}

// Usage: myapp services list --filter api -v
```

**Pattern 4: Error handling**
```rust
use crate::error::NounVerbError;

#[verb(name = "fetch", noun = "data")]
pub async fn cmd_fetch(
    url: String,
) -> Result<HandlerOutput, NounVerbError> {
    // Use ? operator for clean error propagation
    let data = fetch_url(&url)
        .await
        .map_err(|e| NounVerbError::ExecutionFailed(
            format!("Failed to fetch {}: {}", url, e)
        ))?;
    
    Ok(HandlerOutput::new(serde_json::to_value(data)?))
}
```

**Pattern 5: Serializable struct output**
```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceStatus {
    name: String,
    healthy: bool,
    uptime_seconds: u64,
}

#[verb(name = "status", noun = "service")]
pub async fn cmd_status(name: String) -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    let status = ServiceStatus {
        name: name.clone(),
        healthy: true,
        uptime_seconds: 3600,
    };
    
    Ok(HandlerOutput::new(serde_json::to_value(&status)?))
}
```

---

## Testing Strategy

### Running Tests Locally

**Quick test suite (most common):**
```bash
cargo make test
```

**Single-threaded (find flaky tests):**
```bash
cargo make test-lib-deterministic
```

**With all features enabled:**
```bash
cargo make test-all
```

**Single test by name:**
```bash
cargo test test_name --quiet
```

### Understanding Test Organization

```
tests/
├── cli/                    # CLI integration tests
│   ├── integration_cli_tests.rs
│   └── mod.rs
├── frontier/              # Frontier feature tests
│   └── mod.rs
├── acceptance/            # Full acceptance tests
│   └── mod.rs
├── common/                # Shared test utilities
│   ├── test_prelude.rs
│   └── mod.rs
└── *.rs                   # Individual test files
```

**In-file tests:**

Most modules have tests in `#[cfg(test)]` blocks at the bottom:

```bash
code src/verb.rs    # Scroll to bottom to see tests
```

### Test Patterns (AAA)

All tests follow **Arrange-Act-Assert** pattern:

```rust
#[test]
fn test_verb_creates_output_with_valid_input() {
    // ===== ARRANGE =====
    let input = HandlerInput::default();
    let expected_status = "healthy";
    
    // ===== ACT =====
    let result = handle_services_status(input);
    
    // ===== ASSERT =====
    assert!(result.is_ok());
    let output = result.unwrap();
    let value = output.to_json();
    assert_eq!(value["status"], expected_status);
}
```

**Good test names:** `test_verb_does_x_with_condition_y`
**Bad test names:** `test_it_works`, `test_basic`, `test1`

### Writing a Test

**Step 1: Identify what to test**
```rust
// What is the public behavior?
pub fn parse_command_name(input: &str) -> Result<String> {
    // Takes a string, returns parsed command name
}
```

**Step 2: Write Arrange-Act-Assert**
```rust
#[test]
fn test_parse_command_name_with_valid_input() {
    // Arrange: Set up input
    let input = "my-command";
    
    // Act: Call the function
    let result = parse_command_name(input);
    
    // Assert: Verify output
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my-command");
}

#[test]
fn test_parse_command_name_with_invalid_chars() {
    // Arrange
    let input = "my command";  // space not allowed
    
    // Act
    let result = parse_command_name(input);
    
    // Assert
    assert!(result.is_err());
}
```

**Step 3: Run and verify**
```bash
cargo test parse_command_name --quiet
```

### Common Test Utilities

**From `tests/common/test_prelude.rs`:**
```rust
// Use std testing utilities
use std::collections::HashMap;

// Use assertion helpers
assert_eq!(actual, expected);
assert!(condition);
assert_matches!(value, pattern);

// Create test fixtures
fn create_test_handler_input() -> HandlerInput {
    HandlerInput {
        args: HashMap::new(),
        context: Context::default(),
    }
}
```

**Testing async code:**
```rust
#[tokio::test]  // Use this instead of #[test]
async fn test_async_verb_execution() {
    let result = handle_services_status(input).await;
    assert!(result.is_ok());
}
```

### Test Size Guidelines

- **Unit tests**: <50ms each (pure logic)
- **Integration tests**: <100ms each (with I/O)
- **Full suite**: <1 second total (parallel execution)

**If a test is slow:**
1. Check for `std::thread::sleep()` calls
2. Look for network/file I/O
3. Consider using mocks (`mockall` crate)
4. Move to separate benchmark file if needed

### Debugging a Failing Test

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name --quiet

# Run single-threaded (easier to read output)
cargo test test_name -- --test-threads=1

# Run with println output visible
cargo test test_name -- --nocapture

# Use logging
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Test Coverage Expectations

**For public functions:**
- Happy path test: Does it work with valid input?
- Error case test: Does it fail gracefully with invalid input?
- Edge case test: Boundary values, empty inputs, etc.

**Example:**
```rust
pub fn validate_port(port: u16) -> Result<u16> {
    if port < 1024 {
        Err(Error::PortTooLow)
    } else if port > 65535 {
        Err(Error::PortTooHigh)
    } else {
        Ok(port)
    }
}

#[test]
fn test_valid_port() {
    assert_eq!(validate_port(8080).unwrap(), 8080);
}

#[test]
fn test_port_too_low() {
    assert!(validate_port(80).is_err());
}

#[test]
fn test_port_boundary() {
    assert_eq!(validate_port(1024).unwrap(), 1024);  // Edge: minimum valid
}
```

---

## Communication

### Where to Ask Questions

| Question Type | Where | Response Time |
|---------------|-------|----------------|
| How do I...? | GitHub Discussions | 24-48h |
| This doesn't work | GitHub Issues | 24-72h |
| Code review feedback | Pull Request comments | Real-time during review |
| Architecture questions | GitHub Discussions | 24-48h |
| Bug reports | GitHub Issues | 24-72h |

### Reporting Bugs

**Include these details:**

1. **What happened** (actual behavior)
2. **What should happen** (expected behavior)
3. **Steps to reproduce** (exact commands)
4. **Environment** (Rust version, OS, features)
5. **Minimal reproduction** (small code example)

**Example:**
```
**Bug:** Verb fails silently when JSON serialization fails

**Steps:**
1. Create verb returning non-Serialize struct
2. Run command
3. Observe: No error, empty output

**Expected:** Error message about serialization failure

**Environment:**
- Rust 1.74
- Ubuntu 22.04
- clap-noun-verb v26.9.1

**Code:**
#[verb(name = "test")]
pub fn cmd_test() -> Result<NonSerializeStruct> { ... }
```

### Contacting Maintainers

**Author:** Sean Chatman (`seanchatmangpt@gmail.com`)

**For urgent issues:**
- File a GitHub issue with `[URGENT]` prefix
- Include severity explanation
- Include workaround if possible

### Discussion Channels

- **GitHub Issues**: Bugs, feature requests, design decisions
- **GitHub Discussions**: Questions, how-tos, architecture
- **GitHub PR**: Code review, specific changes
- **Email**: Complex discussions requiring back-and-forth

---

## Workflow

### Branch Strategy

**Branch naming:** Always use clear prefixes

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feat/description` | `feat/add-verb-middleware` |
| Bugfix | `fix/description` | `fix/router-dispatch-panic` |
| Refactor | `refactor/description` | `refactor/simplify-registry` |
| Documentation | `docs/description` | `docs/onboarding-guide` |
| Test | `test/description` | `test/add-macro-validation-tests` |

**Rules:**
- Never work on `main` directly
- Always create a new branch
- Delete branch after merge: `git branch -d feat/your-feature`

### Commit Conventions

**Format:** `<type>: <subject> [optional body]`

**Types:** `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `perf`

**Examples:**

```bash
# Simple feature
git commit -m "feat: add verb middleware support"

# Detailed feature
git commit -m "feat: add verb middleware support

This allows users to register middleware that runs before/after
each verb handler, enabling logging, authentication, and tracing.

Includes tests for middleware ordering and error handling."

# Bugfix
git commit -m "fix: prevent panics in command router with invalid args

The router was calling unwrap() instead of handling None.
Now returns proper error message to user."

# Documentation
git commit -m "docs: improve verb macro API documentation

Added examples showing how to use middleware in verb definitions."
```

**Commit message guidelines:**
- Imperative mood: "add" not "added" or "adds"
- No period after subject line
- Explain **why**, not **what** (the code shows what)
- Keep subject under 50 characters
- Wrap body at 72 characters
- Reference issues: "Closes #123" or "Related to #456"

### Pre-Commit Checks

Before pushing, always run:

```bash
# Format code
cargo make format

# Run linter
cargo make lint

# Run tests
cargo make test

# Verify no issues
git status
```

**If pre-commit hooks fail:**
1. Fix the errors (usually formatting or missing docs)
2. Re-stage changes: `git add -A`
3. Commit again: `git commit -m "..."`
4. Hooks run again; if successful, commit proceeds

**Never skip hooks** (`--no-verify`). They protect code quality.

### Pull Request Process

**1. Before creating PR:**
```bash
cargo make lint
cargo make test-lib-deterministic
cargo make format
git status  # Ensure clean state
```

**2. Create PR on GitHub:**
- Give it a clear title
- Fill out description template
- Link related issues
- Request reviewers

**3. Respond to feedback:**
- Respond to all comments
- Push new commits (don't amend, don't rebase)
- Request re-review
- Be receptive and collaborative

**4. Merge process:**
- Maintainer will merge using "Squash and merge" or "Create merge commit"
- Your branch will be deleted
- CI must pass before merge

### Merge Protocol

**Important:** This project **never rebases**

- ✅ Do: Merge pull requests with `git merge`
- ✅ Do: Create new commits instead of amending
- ❌ Don't: Rebase branches
- ❌ Don't: Force push
- ❌ Don't: Skip pre-commit hooks

**Why?**
- Easier to track history
- Safer for collaborative work
- Clearer merge points
- Prevents accidental data loss

**If you need to fix a commit:**
```bash
# WRONG: Don't amend
git commit --amend

# RIGHT: Create a new commit
git commit -m "fix: address code review feedback"
```

---

## Performance Expectations

### Compile Times

**SLO Targets:**
- **Incremental build:** ≤2 seconds
- **Clean build:** ≤30 seconds
- **Full test suite:** ≤15 seconds

**Current performance (v26.9.1):**
- Incremental: 0.66s ✅
- Clean: 8s ✅
- Tests: 5-10s ✅

**If you notice slowdowns:**

```bash
# Measure your change
touch src/lib.rs
time cargo make build

# Profile build time
cargo build --release -Z timings

# Check dependency graph
cargo tree | grep -E "^[├└]"
```

### Binary Size

**SLO Target:** ≤10MB

**Current size:** 2.2MB ✅

**If adding a feature:**
1. Build release: `cargo make build-release`
2. Check size: `ls -lh target/release/clap_noun_verb`
3. If >2.5MB, optimize before merging

### Runtime Performance

**Verb dispatch latency:** Should be <1ms

**If implementing expensive operation:**
- Use async/await for I/O
- Add benchmarks in `benches/`
- Profile with: `cargo make bench`

---

## Success Metrics

### You Know the Codebase When You Can...

**Tier 1: Foundation (After 1 hour)**
- [ ] Clone project and run tests successfully
- [ ] Understand noun-verb command pattern
- [ ] Explain what `#[verb]` macro does
- [ ] Run an example and understand the output
- [ ] Find a public function and read its docs

**Tier 2: Contributor (After 2-3 hours)**
- [ ] Write a test following AAA pattern
- [ ] Add a simple function to the codebase
- [ ] Run the full linting pipeline (`cargo make lint`)
- [ ] Create a branch, commit, and submit a PR
- [ ] Understand the flow: macro → registry → router → handler

**Tier 3: Independent (After 4-5 hours)**
- [ ] Add a new verb command from scratch
- [ ] Understand error handling patterns (no unwrap!)
- [ ] Write multiple tests covering edge cases
- [ ] Debug a failing test or lint error
- [ ] Review someone else's PR and give feedback

**Tier 4: Deep Knowledge (After 8-10 hours)**
- [ ] Modify macro code or validation logic
- [ ] Understand distributed slice linking mechanism
- [ ] Optimize a slow compilation or test
- [ ] Design and implement a new feature
- [ ] Mentor another contributor

### Checklist for Success

After completing this onboarding:

- [ ] Development environment is fully set up
- [ ] Can run tests, lint, and format code
- [ ] Understand the project's architecture
- [ ] Made at least one small contribution
- [ ] Know how to submit a PR
- [ ] Can read and understand existing code
- [ ] Know where to find answers to questions
- [ ] Understand commit message conventions
- [ ] Can write tests following project patterns
- [ ] Know what "no unwrap in production" means

### Next Steps After Onboarding

1. **Pick a task** from GitHub Issues labeled `good first issue`
2. **Make it your own** - add tests, improve docs
3. **Ask questions** if stuck - use Discussions
4. **Submit a PR** - follow the workflow section
5. **Review code** from other contributors
6. **Keep learning** - read the CLAUDE.md for deeper knowledge

---

## Appendix: Troubleshooting

### Build Problems

**"error: could not compile `clap-noun-verb`"**
```bash
cargo update
cargo clean
cargo make build
```

**"error[E0425]: cannot find function `verb`"**
```rust
// Add to your file:
use clap_noun_verb_macros::verb;
```

**"error[E0308]: mismatched types in verb return"**
```rust
// Ensure return type is Result<HandlerOutput, ...>
// NOT just HandlerOutput
#[verb(name = "cmd")]
pub async fn handler() -> Result<HandlerOutput, Box<dyn std::error::Error>> {
    Ok(HandlerOutput::new(json!({})))
}
```

### Test Problems

**"test suite panicked"**
```bash
RUST_BACKTRACE=1 cargo test test_name --quiet
```

**"tests take >1 second"**
- Check for `std::thread::sleep()` in tests
- Look for network/file I/O
- Use mocks instead of real I/O

**"test fails sometimes, passes other times"**
```bash
cargo make test-lib-deterministic
```
This runs tests single-threaded to catch race conditions.

### Lint Problems

**"error: code has incorrect formatting"**
```bash
cargo make format
```

**"clippy: use of unwrap_used"**
```rust
// Bad
let value = result.unwrap();

// Good
let value = result.map_err(|e| NounVerbError::ExecutionFailed(e.to_string()))?;
```

**"warning: function is never used"**
```rust
#[allow(dead_code)]
fn helper() { }
```

### Git Problems

**"error: pre-commit hook failed"**
```bash
# Fix the issues
cargo make format
cargo make lint

# Stage again
git add -A

# Try commit again
git commit -m "your message"
```

**"I committed to main by accident"**
```bash
# Move commits to new branch
git branch new-branch
git reset --hard origin/main
git checkout new-branch
# Continue from here
```

---

## Quick Reference

### Essential Commands

```bash
# Development
cargo make build          # Compile
cargo make test           # Run tests
cargo make lint           # Check code quality
cargo make format         # Auto-format

# Before committing
cargo make format-check
cargo make lint
cargo make test

# Full CI
cargo make ci

# Git
git checkout -b feat/name
git add .
git commit -m "type: message"
git push -u origin feat/name
```

### Key Files to Know

```bash
src/lib.rs                    # Public API
src/cli/mod.rs                # Entry point
src/verb.rs                   # Verb trait
clap-noun-verb-macros/src/    # Macro code
examples/                     # Learn by example
tests/                        # Test patterns
```

### Important Concepts

| Term | Meaning |
|------|---------|
| **Noun** | Resource/entity (e.g., `services`) |
| **Verb** | Action (e.g., `status`, `create`) |
| **Handler** | Function implementing a verb |
| **Macro** | `#[verb]` attribute for registration |
| **Registry** | Collects all registered verbs |
| **Router** | Dispatches commands to handlers |
| **Distributed Slice** | Compile-time collection via `linkme` |

### Where to Get Help

- **Project questions:** GitHub Discussions
- **Bug reports:** GitHub Issues
- **Code review:** Pull Requests
- **Architecture:** CLAUDE.md (in repository)
- **Author:** Sean Chatman (seanchatmangpt@gmail.com)

---

**Welcome to the team! 🚀 Happy coding!**

Last updated: August 20, 2026
