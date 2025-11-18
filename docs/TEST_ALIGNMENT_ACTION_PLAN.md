# Test-README Alignment: Action Plan

**Project:** clap-noun-verb v4.0.1
**Priority:** High
**Estimated Effort:** 2-3 days
**Impact:** Critical for user trust and library adoption

## Executive Summary

This action plan addresses critical gaps between README documentation and actual test coverage. The project has 285+ tests but lacks coverage for 4 README how-to guides, has 2 broken examples, and poor test-to-README linkage.

**Goal:** Achieve 90%+ alignment between README claims and verified test coverage.

---

## Phase 1: Critical Fixes (Day 1) - P0

### 1.1 Fix Broken async_example.rs (2 hours)

**Current Issue:**
```bash
$ cargo run --example async_example -- users profile --user_id 1
Error: thread 'tokio-runtime-worker' panicked at macro code
```

**Root Cause:** Macro expansion issue with async handling

**Action Steps:**
```bash
# 1. Investigate panic
cd /Users/sac/clap-noun-verb
RUST_BACKTRACE=full cargo run --example async_example -- users profile --user_id 1

# 2. Create test case
cat > tests/run_async_tests.rs << 'EOF'
//! Tests for README "How to use async operations" (lines 162-190)

use clap_noun_verb::async_verb::run_async;
use clap_noun_verb::Result;

#[test]
fn test_run_async_basic() {
    let result: Result<String> = run_async(async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok("success".to_string())
    });
    assert!(result.is_ok());
}

#[test]
fn test_run_async_database_simulation() {
    // Matches README example pattern
    let result = run_async(async {
        // Simulate async DB query
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(vec!["user1".to_string(), "user2".to_string()])
    });
    assert!(result.is_ok());
}
EOF

# 3. Run test to verify fix
cargo test run_async_tests

# 4. Update example to match README
# Either fix macro or update example pattern
```

**Acceptance Criteria:**
- [ ] Example runs without panic
- [ ] Example matches README pattern exactly
- [ ] New test suite passes (tests/run_async_tests.rs)

---

### 1.2 Fix Misleading context_example.rs (1 hour)

**Current Issue:**
```rust
fn show_config() -> Result<ConfigInfo> {
    // In a real app, you'd pass context...
    // But example doesn't actually use AppContext!
    let config = AppConfig { ... }; // Creates fresh config
}
```

**Action Steps:**
```bash
# 1. Read current example
cat examples/context_example.rs

# 2. Create proper AppContext example
cat > examples/context_example.rs << 'EOF'
//! Example: Global Application Context
//! Demonstrates AppContext<T> to share state across commands

use clap_noun_verb::{AppContext, Result};
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;
use std::sync::Arc;

// Shared state
#[derive(Clone)]
struct AppState {
    config: Config,
    db: Arc<Database>,
}

static CONTEXT: once_cell::sync::Lazy<AppContext> =
    once_cell::sync::Lazy::new(|| {
        let ctx = AppContext::new();
        let state = AppState {
            config: Config::load(),
            db: Arc::new(Database::connect()),
        };
        ctx.insert(state).unwrap();
        ctx
    });

#[noun("app", "Application management")]
#[verb("config")]
fn show_config() -> Result<ConfigInfo> {
    // Get shared state from context
    let state: AppState = CONTEXT.get()?;
    Ok(ConfigInfo {
        database_url: state.config.db_url.clone(),
        debug: state.config.debug,
    })
}
EOF

# 3. Create test
cat > tests/app_context_tests.rs << 'EOF'
//! Tests for README "How to share state across commands" (lines 193-217)

use clap_noun_verb::AppContext;

#[test]
fn test_app_context_insert_and_get() {
    let ctx = AppContext::new();
    ctx.insert("test_value".to_string()).unwrap();
    let value: String = ctx.get().unwrap();
    assert_eq!(value, "test_value");
}

#[test]
fn test_app_context_multiple_types() {
    let ctx = AppContext::new();
    ctx.insert(42_i32).unwrap();
    ctx.insert("hello".to_string()).unwrap();

    let num: i32 = ctx.get().unwrap();
    let text: String = ctx.get().unwrap();

    assert_eq!(num, 42);
    assert_eq!(text, "hello");
}
EOF

cargo test app_context_tests
```

**Acceptance Criteria:**
- [ ] Example actually uses AppContext.get()/insert()
- [ ] Example runs successfully
- [ ] Test suite validates AppContext behavior
- [ ] README example pattern matches

---

### 1.3 Add OutputFormat Test Suite (2 hours)

**Current Gap:** README claims 5 output formats, no tests exist

**Action Steps:**
```bash
cat > tests/output_format_tests.rs << 'EOF'
//! Tests for README "How to format output" (lines 220-252)

use clap_noun_verb::OutputFormat;
use serde::Serialize;

#[derive(Serialize)]
struct TestData {
    name: String,
    value: i32,
}

#[test]
fn test_output_format_json() {
    let data = TestData { name: "test".into(), value: 42 };
    let json = OutputFormat::Json.format(&data).unwrap();
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"test\""));
    assert!(json.contains("\"value\""));
    assert!(json.contains("42"));
}

#[test]
fn test_output_format_yaml() {
    let data = TestData { name: "test".into(), value: 42 };
    let yaml = OutputFormat::Yaml.format(&data).unwrap();
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("test"));
}

#[test]
fn test_output_format_toml() {
    let data = TestData { name: "test".into(), value: 42 };
    let toml = OutputFormat::Toml.format(&data).unwrap();
    assert!(toml.contains("name"));
    assert!(toml.contains("test"));
}

#[test]
fn test_output_format_table() {
    let data = vec![
        TestData { name: "alice".into(), value: 1 },
        TestData { name: "bob".into(), value: 2 },
    ];
    let table = OutputFormat::Table.format(&data).unwrap();
    assert!(table.contains("alice"));
    assert!(table.contains("bob"));
}

#[test]
fn test_output_format_tsv() {
    let data = vec![
        TestData { name: "alice".into(), value: 1 },
        TestData { name: "bob".into(), value: 2 },
    ];
    let tsv = OutputFormat::Tsv.format(&data).unwrap();
    assert!(tsv.contains("\t")); // Tab-separated
    assert!(tsv.contains("alice"));
}

#[test]
fn test_output_format_all_formats_work() {
    let data = TestData { name: "test".into(), value: 42 };

    // All formats should succeed
    assert!(OutputFormat::Json.format(&data).is_ok());
    assert!(OutputFormat::Yaml.format(&data).is_ok());
    assert!(OutputFormat::Toml.format(&data).is_ok());
    assert!(OutputFormat::Table.format(&data).is_ok());
    assert!(OutputFormat::Tsv.format(&data).is_ok());
}
EOF

cargo test output_format_tests
```

**Acceptance Criteria:**
- [ ] All 5 formats tested (JSON, YAML, TOML, Table, TSV)
- [ ] Tests verify actual output content
- [ ] Tests pass consistently

---

### 1.4 Add Shell Completion Test Suite (2 hours)

**Current Gap:** README claims 5 shell completions, no tests

**Action Steps:**
```bash
cat > tests/shell_completion_tests.rs << 'EOF'
//! Tests for README "How to generate shell completions" (lines 254-279)

use clap::Command;
use clap_noun_verb::{generate_completion, Shell};

#[test]
fn test_generate_completion_bash() {
    let mut cmd = Command::new("test");
    let completion = generate_completion(&mut cmd, Shell::Bash, "test");
    assert!(!completion.is_empty());
    assert!(completion.contains("bash") || completion.contains("complete"));
}

#[test]
fn test_generate_completion_zsh() {
    let mut cmd = Command::new("test");
    let completion = generate_completion(&mut cmd, Shell::Zsh, "test");
    assert!(!completion.is_empty());
    assert!(completion.contains("zsh") || completion.contains("compdef"));
}

#[test]
fn test_generate_completion_fish() {
    let mut cmd = Command::new("test");
    let completion = generate_completion(&mut cmd, Shell::Fish, "test");
    assert!(!completion.is_empty());
    assert!(completion.contains("fish") || completion.contains("complete"));
}

#[test]
fn test_generate_completion_powershell() {
    let mut cmd = Command::new("test");
    let completion = generate_completion(&mut cmd, Shell::PowerShell, "test");
    assert!(!completion.is_empty());
}

#[test]
fn test_generate_completion_elvish() {
    let mut cmd = Command::new("test");
    let completion = generate_completion(&mut cmd, Shell::Elvish, "test");
    assert!(!completion.is_empty());
}

#[test]
fn test_shell_file_extension() {
    assert_eq!(Shell::Bash.file_extension(), "bash");
    assert_eq!(Shell::Zsh.file_extension(), "zsh");
    assert_eq!(Shell::Fish.file_extension(), "fish");
}

#[test]
fn test_shell_install_instructions() {
    let instructions = Shell::Bash.install_instructions("myapp");
    assert!(instructions.contains("bash") || instructions.contains("source"));
}
EOF

cargo test shell_completion_tests
```

**Acceptance Criteria:**
- [ ] All 5 shells tested (Bash, Zsh, Fish, PowerShell, Elvish)
- [ ] Completion generation produces non-empty output
- [ ] Helper methods tested (file_extension, install_instructions)

---

## Phase 2: High Priority Fixes (Day 2) - P1

### 2.1 Fix 3 Failing Examples (3 hours)

**Failing Examples:**
1. `async_io_example.rs` - BackpressureError doesn't impl Error
2. `io_advanced.rs` - Borrow checker error
3. `integration_layer_example.rs` - Missing types

**Action Steps:**
```bash
# 1. Fix async_io_example.rs
# Add Error impl to BackpressureError
cat >> src/kernel/session_streaming.rs << 'EOF'
impl std::error::Error for BackpressureError {}
EOF

# 2. Fix io_advanced.rs
# Investigate borrow checker issue
cargo build --example io_advanced 2>&1 | grep "error\["
# Fix based on specific error

# 3. Fix integration_layer_example.rs
# Add missing types or feature-gate example
grep -n "LoggingMiddleware\|ReadOnlyFS\|Safe" examples/integration_layer_example.rs
# Either implement missing types or add #[cfg(feature = "integration")]

# Verify all compile
cargo build --examples
```

**Acceptance Criteria:**
- [ ] All 29 examples compile
- [ ] cargo build --examples succeeds with 0 errors

---

### 2.2 Add README Linkage to All Test Files (2 hours)

**Action Steps:**
```bash
# Script to add module docs to test files
cat > scripts/add_readme_links.sh << 'EOF'
#!/bin/bash

# Tests mapping to README sections
declare -A TEST_SECTIONS=(
    ["attribute_macro_acceptance.rs"]="Attribute Macros (lines 10-11)"
    ["arg_actions.rs"]="Type Inference & Argument Actions (lines 110-159, 305-338)"
    ["env_vars.rs"]="Environment Variable Fallback (lines 119-121)"
    ["positional_args.rs"]="Positional Arguments (lines 123-125)"
    ["app_context_tests.rs"]="How to Share State (lines 193-217)"
    ["output_format_tests.rs"]="How to Format Output (lines 220-252)"
    ["shell_completion_tests.rs"]="How to Generate Completions (lines 254-279)"
    ["run_async_tests.rs"]="How to Use Async Operations (lines 162-190)"
    ["autonomic_tests.rs"]="Autonomic CLI Layer (AUTONOMIC.md)"
)

for file in "${!TEST_SECTIONS[@]}"; do
    section="${TEST_SECTIONS[$file]}"

    # Check if file exists
    if [ -f "tests/$file" ]; then
        # Add module doc if not present
        if ! grep -q "//! Tests for README" "tests/$file"; then
            cat > "tests/${file}.tmp" << HEADER
//! Tests for README "$section"
//!
//! Validates the examples and claims made in README.md.
//! Each test corresponds to a specific README section or example.

HEADER
            cat "tests/$file" >> "tests/${file}.tmp"
            mv "tests/${file}.tmp" "tests/$file"
            echo "✅ Updated tests/$file"
        fi
    fi
done
EOF

chmod +x scripts/add_readme_links.sh
./scripts/add_readme_links.sh
```

**Acceptance Criteria:**
- [ ] All test files have module-level README references
- [ ] Links include specific line numbers
- [ ] Purpose of each test file is clear

---

### 2.3 Add Deprecation Test Suite (1 hour)

**Action Steps:**
```bash
cat > tests/deprecation_tests.rs << 'EOF'
//! Tests for README "How to mark commands as deprecated" (lines 281-300)

use clap_noun_verb::deprecation::{Deprecation, DeprecationType};

#[test]
fn test_deprecation_basic() {
    let dep = Deprecation::new(DeprecationType::Verb)
        .since("3.5.0")
        .removed_in("4.0.0");

    let warning = dep.warning_message("old-verb");
    assert!(warning.contains("deprecated"));
    assert!(warning.contains("3.5.0"));
    assert!(warning.contains("4.0.0"));
}

#[test]
fn test_deprecation_with_suggestion() {
    let dep = Deprecation::new(DeprecationType::Verb)
        .suggestion("Use 'new-verb' instead");

    let warning = dep.warning_message("old-verb");
    assert!(warning.contains("Suggestion"));
    assert!(warning.contains("new-verb"));
}

#[test]
fn test_deprecation_with_note() {
    let dep = Deprecation::new(DeprecationType::Verb)
        .note("This feature is being replaced");

    let warning = dep.warning_message("old-verb");
    assert!(warning.contains("This feature is being replaced"));
}

#[test]
fn test_deprecation_noun_type() {
    let dep = Deprecation::new(DeprecationType::Noun);
    let warning = dep.warning_message("old-noun");
    assert!(warning.contains("Noun"));
}
EOF

cargo test deprecation_tests
```

**Acceptance Criteria:**
- [ ] Deprecation system tested
- [ ] All builder methods verified
- [ ] Warning message format validated

---

## Phase 3: Test Organization (Day 3) - P2

### 3.1 Create Test Discovery Guide (1 hour)

**Action Steps:**
```bash
cat > docs/TEST_DISCOVERY.md << 'EOF'
# Test Discovery Guide

**Quick reference:** Which test file covers which README feature?

## Core Features

| Feature | README Section | Test File | Example |
|---------|----------------|-----------|---------|
| Attribute Macros | Lines 10-11 | `attribute_macro_acceptance.rs` | `attribute_macro.rs` |
| Auto-Discovery | Line 12 | `acceptance/attribute_macro.rs` | `basic.rs` |
| Type Inference | Line 14 | `arg_actions.rs`, `positional_args.rs` | `arguments.rs` |
| Async Support | Line 16 | `run_async_tests.rs` | `async_example.rs` |
| AppContext | Line 17 | `app_context_tests.rs` | `context_example.rs` |
| Output Formats | Line 18 | `output_format_tests.rs` | `format_example.rs` |
| Shell Completions | Line 19 | `shell_completion_tests.rs` | `completion_example.rs` |
| Autonomic Layer | Line 20 | `autonomic_tests.rs` | `autonomic_example.rs` |

## How-to Guides

| Guide | README Lines | Test File | Key Tests |
|-------|--------------|-----------|-----------|
| Configure Arguments | 110-159 | `arg_actions.rs`, `env_vars.rs`, `positional_args.rs` | test_count_action, test_env_fallback |
| Async Operations | 162-190 | `run_async_tests.rs` | test_run_async_basic, test_database_simulation |
| Share State | 193-217 | `app_context_tests.rs` | test_insert_and_get, test_multiple_types |
| Format Output | 220-252 | `output_format_tests.rs` | test_all_formats_work |
| Shell Completions | 254-279 | `shell_completion_tests.rs` | test_generate_completion_* |
| Deprecation | 281-300 | `deprecation_tests.rs` | test_deprecation_with_suggestion |

## Finding Tests for Your Use Case

**Want to test attribute macros?**
→ `tests/attribute_macro_acceptance.rs`

**Want to test async operations?**
→ `tests/run_async_tests.rs`

**Want to test argument parsing?**
→ `tests/arg_actions.rs` (actions), `tests/env_vars.rs` (environment), `tests/positional_args.rs` (positional)

**Want to test output formatting?**
→ `tests/output_format_tests.rs`

**Want to test autonomic features?**
→ `tests/autonomic_tests.rs`

## Running Specific Test Suites

```bash
# Test async support
cargo test run_async_tests

# Test output formats
cargo test output_format_tests

# Test shell completions
cargo test shell_completion_tests

# Test all README features
cargo test --test '*_tests'

# Test all examples
cargo test --test integration_examples
```

## Adding New Tests

When adding tests for a new feature:

1. Create test file: `tests/<feature>_tests.rs`
2. Add module doc: `//! Tests for README "<feature>" (lines X-Y)`
3. Link to this guide: Update table above
4. Add to CI: Ensure `cargo test` runs it
EOF
```

**Acceptance Criteria:**
- [ ] Guide created
- [ ] All test files listed with README mappings
- [ ] Examples provided for common use cases

---

### 3.2 Add CI Validation for Examples (1 hour)

**Action Steps:**
```bash
# Add to .github/workflows/ci.yml or create new file
cat > .github/workflows/examples.yml << 'EOF'
name: Validate Examples

on: [push, pull_request]

jobs:
  examples:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Build all examples
        run: cargo build --examples

      - name: Test examples compile count
        run: |
          EXPECTED=29
          ACTUAL=$(cargo build --examples 2>&1 | grep "Compiling.*example" | wc -l)
          if [ "$ACTUAL" -ne "$EXPECTED" ]; then
            echo "Expected $EXPECTED examples, got $ACTUAL"
            exit 1
          fi

      - name: Run example help commands
        run: |
          for example in examples/*.rs; do
            name=$(basename "$example" .rs)
            echo "Testing: $name --help"
            cargo run --example "$name" -- --help || exit 1
          done
EOF

# Test locally
act -j examples  # If using act for local CI testing
```

**Acceptance Criteria:**
- [ ] CI validates all examples compile
- [ ] CI checks example count (should be 29)
- [ ] CI runs --help for all examples

---

### 3.3 Reorganize Test Structure (2 hours)

**Action Steps:**
```bash
# Create feature-based structure
mkdir -p tests/features
mkdir -p tests/howto

# Move tests to organized locations
mv tests/app_context_tests.rs tests/features/
mv tests/output_format_tests.rs tests/features/
mv tests/shell_completion_tests.rs tests/features/
mv tests/deprecation_tests.rs tests/features/
mv tests/run_async_tests.rs tests/howto/

# Update Cargo.toml if needed for test organization
cat >> Cargo.toml << 'EOF'
[[test]]
name = "features"
path = "tests/features/mod.rs"

[[test]]
name = "howto"
path = "tests/howto/mod.rs"
EOF

# Create module files
cat > tests/features/mod.rs << 'EOF'
//! Feature tests - Core library features

mod app_context_tests;
mod output_format_tests;
mod shell_completion_tests;
mod deprecation_tests;
EOF

cat > tests/howto/mod.rs << 'EOF'
//! How-to guide tests - Validate README tutorials

mod run_async_tests;
EOF
```

**Acceptance Criteria:**
- [ ] Clear directory structure
- [ ] Tests grouped by feature
- [ ] cargo test still works

---

## Validation Checklist

### After Phase 1 (P0 - Critical)
- [ ] All README how-to guides have test coverage
- [ ] All examples run without crashes
- [ ] No misleading examples
- [ ] AppContext, OutputFormat, Completions tested

### After Phase 2 (P1 - High Priority)
- [ ] All 29 examples compile
- [ ] All test files have README references
- [ ] Deprecation system tested
- [ ] No compilation failures

### After Phase 3 (P2 - Organization)
- [ ] Test discovery guide created
- [ ] CI validates examples
- [ ] Test structure reorganized
- [ ] Documentation complete

---

## Success Metrics

**Before:**
- Test coverage: 75% of README claims
- Example accuracy: 55% (16/29)
- Examples compiling: 69% (20/29)
- Documentation quality: 3/10

**After (Target):**
- Test coverage: 90%+ of README claims
- Example accuracy: 100% (29/29)
- Examples compiling: 100% (29/29)
- Documentation quality: 8/10

---

## Timeline

| Phase | Duration | Priority | Effort |
|-------|----------|----------|--------|
| Phase 1 | Day 1 | P0 | 8 hours |
| Phase 2 | Day 2 | P1 | 6 hours |
| Phase 3 | Day 3 | P2 | 4 hours |
| **Total** | **3 days** | - | **18 hours** |

---

## Resources

- **Full validation report:** `TEST_ALIGNMENT_VALIDATION.md`
- **Summary:** `VALIDATION_SUMMARY.md`
- **Test discovery:** `TEST_DISCOVERY.md` (to be created)
- **README:** `README.md`
- **Autonomic docs:** `AUTONOMIC.md`

---

**Generated:** 2025-11-18
**Status:** Ready for implementation
**Assignee:** TBD
**Review by:** Project maintainers
