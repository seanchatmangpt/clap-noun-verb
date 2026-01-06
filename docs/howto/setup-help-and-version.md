# How to Set Up Help and Version for Your CLI

**Goal**: Configure correct help output and version display for your clap-noun-verb CLI application.

**Prerequisites**:
- Basic Rust knowledge
- Existing Cargo project with clap-noun-verb dependency
- At least one `#[verb]` command defined

**Time**: 10 minutes

---

## Overview

This guide shows you how to:
1. Set up automatic help generation
2. Configure the correct version string
3. Handle help/version output correctly
4. Troubleshoot common issues

---

## Step 1: Choose Your Configuration Approach

clap-noun-verb offers two patterns:

### Pattern A: Zero-Config Auto-Discovery (Simplest)

**When to use**: Small CLIs, prototypes, version from Cargo.toml is acceptable

```rust
fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()  // Automatically uses CARGO_PKG_VERSION
}
```

**Result**:
- Help: Auto-generated from `#[verb]` attributes
- Version: Uses `env!("CARGO_PKG_VERSION")` from your CLI crate's Cargo.toml

### Pattern B: Explicit CliBuilder (Recommended for Production)

**When to use**: Need custom version string, custom about text, explicit control

```rust
use clap_noun_verb::CliBuilder;

fn main() -> clap_noun_verb::Result<()> {
    CliBuilder::new()
        .name("myapp")
        .about("My application description")
        .version(env!("CARGO_PKG_VERSION"))  // Explicit version
        .run()
}
```

**Choose Pattern B if you have any of these requirements**:
- Custom version string format
- Version from a different source than CLI crate's Cargo.toml
- Custom application name/description
- Need to register commands programmatically

---

## Step 2: Set the Correct Version String

### Problem: Version Mismatch

**Symptom**: Running `myapp --version` shows the wrong version (e.g., dependency version instead of your app version)

**Cause**: Using Pattern A without workspace version configuration

### Solution 2.1: Use CliBuilder with Explicit Version

```rust
use clap_noun_verb::CliBuilder;

fn main() -> clap_noun_verb::Result<()> {
    CliBuilder::new()
        .name("myapp")
        .version(env!("CARGO_PKG_VERSION"))  // ← This pulls from YOUR Cargo.toml
        .run()
}
```

### Solution 2.2: Workspace Version Inheritance

If using a Cargo workspace, ensure your CLI crate inherits the workspace version:

**Workspace root Cargo.toml**:
```toml
[workspace]
members = ["crates/*"]

[workspace.package]
version = "1.0.0"
```

**CLI crate Cargo.toml**:
```toml
[package]
name = "myapp"
version.workspace = true  # ← Inherit from workspace

[dependencies]
clap-noun-verb = "5.3"
```

**Verify**:
```bash
cargo build
./target/debug/myapp --version
# Should show: myapp 1.0.0
```

---

## Step 3: Configure Help Text

### Automatic Help Generation

clap-noun-verb auto-generates help from:
1. `#[verb]` attribute parameters
2. Function doc comments
3. Parameter names and types

**Example**:

```rust
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct SyncOutput {
    files_processed: usize,
}

/// Execute the complete code synchronization pipeline.
///
/// This command processes a ggen.toml manifest and generates
/// code from ontology definitions.
///
/// # Examples
///
/// Basic sync:
///   myapp sync
///
/// Dry-run to preview changes:
///   myapp sync --dry-run
#[verb("sync", "root")]
fn sync(
    /// Path to ggen.toml manifest file
    manifest: Option<String>,

    /// Output directory for generated code
    output_dir: Option<String>,

    /// Preview changes without writing files
    dry_run: Option<bool>,
) -> Result<SyncOutput> {
    // Implementation...
    Ok(SyncOutput { files_processed: 42 })
}
```

**Generated help output**:
```
$ myapp sync --help

Execute the complete code synchronization pipeline.

This command processes a ggen.toml manifest and generates
code from ontology definitions.

Usage: myapp sync [OPTIONS]

Options:
      --manifest <PATH>     Path to ggen.toml manifest file
      --output-dir <PATH>   Output directory for generated code
      --dry-run             Preview changes without writing files
  -h, --help                Print help
  -V, --version             Print version

Examples:
  Basic sync:
    myapp sync

  Dry-run to preview changes:
    myapp sync --dry-run
```

### Custom About Text (Application-Level)

```rust
use clap_noun_verb::CliBuilder;

fn main() -> clap_noun_verb::Result<()> {
    CliBuilder::new()
        .name("myapp")
        .about("Language-agnostic code generator using ontologies and RDF")
        .version(env!("CARGO_PKG_VERSION"))
        .run()
}
```

**Generated main help**:
```
$ myapp --help

myapp 1.0.0
Language-agnostic code generator using ontologies and RDF

Usage: myapp [COMMAND]

Commands:
  sync  Execute the complete code synchronization pipeline
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

---

## Step 4: Handle Help/Version Output Correctly

### Problem: "ERROR: CLI execution failed" on --help

**Symptom**:
```bash
$ myapp --help
ERROR: CLI execution failed: Argument parsing failed: [help text]
```

**Cause**: Wrapping ALL clap-noun-verb errors as failures, including successful help/version displays

### Solution: Distinguish Between Error Types

clap-noun-verb returns help/version as errors with **exit code 0** (success). Your main function must handle this:

#### ❌ WRONG: Wrap all errors

```rust
// DON'T DO THIS
fn main() {
    match cli_runner() {
        Err(e) => {
            eprintln!("ERROR: {}", e);  // ← Prints error for help too!
            std::process::exit(1);
        }
        Ok(_) => {}
    }
}

fn cli_runner() -> Result<()> {
    clap_noun_verb::run()
        .map_err(|e| MyError::new(&format!("CLI execution failed: {}", e)))?;
    Ok(())
}
```

#### ✅ CORRECT: Let clap-noun-verb handle it

```rust
fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()  // ← Direct return, no wrapping
}
```

**Why this works**: clap-noun-verb's `Result` type handles help/version gracefully:
- Exit code 0 for `--help` and `--version`
- Exit code 1 for actual errors
- Clean output to stdout (not stderr) for help/version

#### Alternative: Custom Error Handling with CliBuilder

If you need custom error handling:

```rust
use clap_noun_verb::CliBuilder;
use std::process::exit;

fn main() {
    if let Err(e) = run_cli() {
        // clap-noun-verb errors include exit code information
        eprintln!("{}", e);  // Error message only, no "ERROR:" prefix
        exit(1);
    }
}

fn run_cli() -> clap_noun_verb::Result<()> {
    CliBuilder::new()
        .name("myapp")
        .version(env!("CARGO_PKG_VERSION"))
        .run()
}
```

---

## Step 5: Verify Your Configuration

### Test Checklist

Run these commands and verify output:

```bash
# 1. Version displays correctly
$ cargo build
$ ./target/debug/myapp --version
myapp 1.0.0  # ← Should match your Cargo.toml version

# 2. Short version flag works
$ ./target/debug/myapp -V
myapp 1.0.0

# 3. Help displays without errors
$ ./target/debug/myapp --help
# Should show clean help text, NO "ERROR:" prefix

# 4. Short help flag works
$ ./target/debug/myapp -h
# Should show same help as --help

# 5. Command-specific help works
$ ./target/debug/myapp sync --help
# Should show sync command help with all options

# 6. Missing required arguments show help
$ ./target/debug/myapp
# Should show main help (no error prefix)
```

### Expected vs Actual Output

| Command | Expected | Common Issue |
|---------|----------|--------------|
| `myapp --version` | `myapp 1.0.0` | Shows `cli 5.3.4` (dependency version) |
| `myapp --help` | Clean help text | `ERROR: CLI execution failed: ...` |
| `myapp sync --help` | Sync command help | Shows main help instead |
| `myapp` (no args) | Main help | Error message |

---

## Common Issues and Solutions

### Issue 1: Version Shows Dependency Version

**Symptom**: `myapp --version` → `cli 5.3.4`

**Fix**: Use CliBuilder with explicit version:

```rust
CliBuilder::new()
    .version(env!("CARGO_PKG_VERSION"))
    .run()
```

**Verify in Cargo.toml**:
```toml
[package]
name = "myapp"
version = "1.0.0"  # ← This should be shown
```

---

### Issue 2: Help Shows "ERROR: CLI execution failed"

**Symptom**:
```
ERROR: CLI execution failed: Argument parsing failed: [help text]
```

**Fix**: Don't wrap clap-noun-verb errors. Change:

```rust
// Before (WRONG)
clap_noun_verb::run()
    .map_err(|e| MyError::new(&format!("CLI failed: {}", e)))?;

// After (CORRECT)
clap_noun_verb::run()?;  // Direct return
```

---

### Issue 3: Custom Version Handling Conflicts

**Symptom**: You have custom version handling code that conflicts with clap's automatic handling

**Example of problematic code**:
```rust
// This creates conflicts!
if args.iter().any(|arg| arg == "--version" || arg == "-V") {
    println!("myapp {}", env!("CARGO_PKG_VERSION"));
    return Ok(());
}
clap_noun_verb::run()?;  // clap also handles --version!
```

**Fix**: Remove custom handling, let clap do it:

```rust
// Just this:
clap_noun_verb::run()?;
```

Or if you MUST customize version output format:

```rust
use clap_noun_verb::CliBuilder;

CliBuilder::new()
    .version(format!("v{} (build {})",
        env!("CARGO_PKG_VERSION"),
        env!("BUILD_ID")))
    .run()
```

---

### Issue 4: Help Text Missing Documentation

**Symptom**: Help shows command names but no descriptions

**Fix**: Add doc comments to `#[verb]` functions:

```rust
/// This description appears in help output
#[verb("sync", "root")]
fn sync() -> Result<Output> { ... }
```

---

## Complete Working Example

**File structure**:
```
myapp/
├── Cargo.toml
└── src/
    ├── main.rs
    └── commands/
        └── sync.rs
```

**Cargo.toml**:
```toml
[package]
name = "myapp"
version = "1.0.0"
edition = "2021"

[dependencies]
clap-noun-verb = "5.3"
clap-noun-verb-macros = "5.3"
serde = { version = "1", features = ["derive"] }
```

**src/main.rs**:
```rust
use clap_noun_verb::CliBuilder;

mod commands;

fn main() -> clap_noun_verb::Result<()> {
    // Explicit configuration ensures correct version
    CliBuilder::new()
        .name("myapp")
        .about("Language-agnostic code generator")
        .version(env!("CARGO_PKG_VERSION"))  // Uses 1.0.0 from Cargo.toml
        .run()  // Automatic command discovery + help/version handling
}
```

**src/commands/sync.rs**:
```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct SyncOutput {
    pub success: bool,
    pub files_processed: usize,
}

/// Execute the complete code synchronization pipeline.
///
/// Processes a ggen.toml manifest and generates code from ontology definitions.
#[verb("sync", "root")]
pub fn sync(
    /// Path to ggen.toml manifest file
    manifest: Option<String>,

    /// Preview changes without writing files
    dry_run: Option<bool>,
) -> Result<SyncOutput> {
    let manifest_path = manifest.unwrap_or_else(|| "ggen.toml".to_string());

    println!("Syncing from manifest: {}", manifest_path);

    if dry_run.unwrap_or(false) {
        println!("DRY RUN: No files will be modified");
    }

    Ok(SyncOutput {
        success: true,
        files_processed: 42,
    })
}
```

**Test it**:
```bash
cargo build
./target/debug/myapp --version   # → myapp 1.0.0
./target/debug/myapp --help      # → Clean help, no errors
./target/debug/myapp sync --help # → Sync command help
./target/debug/myapp sync        # → {"success":true,"files_processed":42}
```

---

## Next Steps

- **Add more commands**: Follow the same `#[verb]` pattern
- **Customize help categories**: Use the HelpSystem API (advanced)
- **Add shell completions**: See [How to Add Shell Completions](./add-shell-completions.md)
- **Configure output formats**: See [How to Configure Output Formats](./configure-output-formats.md)

---

## Reference

- [API Reference: CliBuilder](../reference/api/cli-builder.md)
- [Tutorial: Your First CLI](../tutorial/01-your-first-cli.md)
- [Explanation: Architecture Philosophy](../explanation/architecture.md)
