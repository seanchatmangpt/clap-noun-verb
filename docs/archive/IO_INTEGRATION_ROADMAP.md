# I/O Integration Roadmap: Practical Implementation Guide

## Quick Reference

**What**: Add ecosystem I/O capabilities to clap-noun-verb while maintaining Typer-style simplicity.

**Why**:
- Replace custom I/O implementations with battle-tested crio crate
- Reduce maintenance burden
- Improve error handling with anyhow/thiserror
- Add structured logging with tracing
- Better testing infrastructure

**How**: Four-phase implementation following existing architecture patterns

**Effort**: ~4-6 weeks for full implementation

---

## Phase 1: Foundation (Week 1-2)

### 1.1 Add Dependencies

**File**: `Cargo.toml`

```toml
[dependencies]
# Existing
clap = { version = "4.5", features = ["derive", "env", "suggestions"] }
clap_complete = "4.5"
clap_mangen = "0.2"
# ... other existing deps ...

# NEW - I/O Handling
clio = { version = "0.3", features = ["clap-parse"] }

# NEW - Error Handling (applications)
anyhow = "1.0"

# NEW - Error Handling (libraries)
thiserror = "1.0"

# NEW - Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "ansi"] }

[dev-dependencies]
# Existing
chicago-tdd-tools = "1.0"
insta = "1.0"
proptest = "1.0"
criterion = "0.5"

# NEW - CLI Testing
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.0"
```

**Task**: Update Cargo.toml with new dependencies
- [ ] Add clio with clap-parse feature
- [ ] Add anyhow and thiserror
- [ ] Add tracing and tracing-subscriber
- [ ] Add test dependencies
- [ ] Run `cargo check` to verify
- [ ] Run `cargo tree` to review dependency tree

### 1.2 Create I/O Module

**File**: `src/io/mod.rs` (NEW)

```rust
//! I/O utilities and types for clap-noun-verb
//!
//! Provides ergonomic wrappers around clio for stdin/stdout/file handling.
//! Integrates with the #[verb] macro for automatic argument parsing.
//!
//! # Examples
//!
//! ```rust,ignore
//! use clap_noun_verb::io::Input;
//! use clap_noun_verb_macros::verb;
//!
//! #[verb]
//! fn process(input: Input) -> Result<Output> {
//!     let content = input.read_to_string()?;
//!     Ok(Output { ... })
//! }
//! ```

// Re-export clio types with clio-parse feature enabled
pub use clio::{Input, InputPath, Output, OutputPath};

// Error types
pub mod error;
pub use error::{IoError, Result as IoResult};

// Type detection helpers (for macro use)
pub(crate) mod types;
pub use types::{is_input_type, is_output_type};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_stdin() {
        // Test "-" maps to stdin
    }

    #[test]
    fn test_output_stdout() {
        // Test "-" maps to stdout
    }
}
```

**File**: `src/io/error.rs` (NEW)

```rust
//! I/O error types

use std::fmt;

#[derive(Debug)]
pub enum IoError {
    Io(std::io::Error),
    ClioInput(String),  // Placeholder until clio has proper error type
    ClioOutput(String),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IoError::Io(e) => write!(f, "I/O error: {}", e),
            IoError::ClioInput(e) => write!(f, "Input error: {}", e),
            IoError::ClioOutput(e) => write!(f, "Output error: {}", e),
        }
    }
}

impl std::error::Error for IoError {}

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        IoError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, IoError>;
```

**File**: `src/io/types.rs` (NEW)

```rust
//! Type detection helpers for macro expansion

use syn::Type;

/// Check if a type is clio::Input
pub fn is_input_type(ty: &Type) -> bool {
    match ty {
        syn::Type::Path(p) => {
            let last_segment = p.path.segments.last();
            last_segment.map(|seg| seg.ident == "Input").unwrap_or(false)
        }
        _ => false,
    }
}

/// Check if a type is clio::Output
pub fn is_output_type(ty: &Type) -> bool {
    match ty {
        syn::Type::Path(p) => {
            let last_segment = p.path.segments.last();
            last_segment.map(|seg| seg.ident == "Output").unwrap_or(false)
        }
        _ => false,
    }
}

/// Check if a type is Option<Output>
pub fn is_optional_output_type(ty: &Type) -> bool {
    match ty {
        syn::Type::Option(opt) => {
            if let syn::Type::Path(p) = &*opt.elem {
                let last_segment = p.path.segments.last();
                last_segment.map(|seg| seg.ident == "Output").unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_input_type() {
        // Test type detection for Input
    }
}
```

**Task**: Create I/O module
- [ ] Create `src/io/mod.rs`
- [ ] Create `src/io/error.rs`
- [ ] Create `src/io/types.rs`
- [ ] Update `src/lib.rs` to include `pub mod io`
- [ ] Update `src/lib.rs` to re-export `pub use io::{Input, Output}`
- [ ] Run `cargo build` to verify
- [ ] Run `cargo test --lib io` for initial tests

### 1.3 Update Library Exports

**File**: `src/lib.rs` (MODIFY)

Add to module declarations:
```rust
// New in v4.1.0 - I/O Handling
pub mod io;

// Also re-export for convenience
pub use io::{Input, Output};
```

**Task**: Update lib.rs
- [ ] Add `pub mod io;` declaration
- [ ] Add `pub use io::{Input, Output};` exports
- [ ] Verify docs build: `cargo doc --open`

### 1.4 Create Basic I/O Example

**File**: `examples/io_basic.rs` (NEW)

```rust
//! Basic I/O example using clio
//!
//! Usage:
//!   cargo run --example io_basic -- echo -i input.txt -o output.txt
//!   cargo run --example io_basic -- echo -i input.txt
//!   echo "hello" | cargo run --example io_basic -- echo

use clap_noun_verb::io::Input;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct EchoResult {
    bytes_read: usize,
}

#[verb]
fn echo(
    #[arg(short, long)]
    input: Input,
) -> Result<EchoResult> {
    let content = input.read_to_string()?;
    let bytes = content.len();
    println!("{}", content);
    Ok(EchoResult { bytes_read: bytes })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

**Task**: Create example
- [ ] Create `examples/io_basic.rs`
- [ ] Test: `cargo run --example io_basic -- tools echo -i Cargo.toml`
- [ ] Test stdin: `echo "test" | cargo run --example io_basic -- tools echo`

### 1.5 Documentation

**File**: `src/io/mod.rs` (Update doc comments)

Add comprehensive module documentation with examples.

**Task**: Document I/O module
- [ ] Add module-level doc comments to `src/io/mod.rs`
- [ ] Add examples to each public function
- [ ] Run `cargo doc --open` and verify rendering

---

## Phase 2: Macro Enhancement (Week 2-3)

### 2.1 Enhance Macro Type Detection

**File**: `clap-noun-verb-macros/src/lib.rs` (MODIFY)

Add type detection in `#[verb]` macro:

```rust
fn detect_param_type(ty: &syn::Type) -> ParamType {
    use clap_noun_verb::io::types::*;

    if is_input_type(ty) {
        return ParamType::Input;
    }
    if is_output_type(ty) {
        return ParamType::Output;
    }
    if is_optional_output_type(ty) {
        return ParamType::OutputOpt;
    }

    // ... existing type detection ...
    match ty {
        syn::Type::Path(p) => {
            let name = &p.path.segments.last().unwrap().ident;
            match name.to_string().as_str() {
                "String" => ParamType::String,
                "bool" => ParamType::Flag,
                "usize" => ParamType::Count,
                _ => ParamType::Custom,
            }
        }
        // ... other cases ...
    }
}
```

**Task**: Enhance macro type detection
- [ ] Update macro to import I/O type detection
- [ ] Add ParamType variants for Input/Output
- [ ] Test macro with I/O types: `cargo test --all`

### 2.2 Auto-Generate ValueParser for I/O

**File**: `clap-noun-verb-macros/src/lib.rs` (MODIFY)

In macro expansion, for I/O types:

```rust
fn expand_arg_for_param(param: &ParamType) -> TokenStream {
    match param {
        ParamType::Input => quote! {
            .value_parser(clap_noun_verb::io::Input::value_parser())
            .help("Input file (use '-' for stdin)")
        },
        ParamType::Output => quote! {
            .value_parser(clap_noun_verb::io::Output::value_parser())
            .help("Output file (use '-' for stdout)")
        },
        ParamType::OutputOpt => quote! {
            .value_parser(clap_noun_verb::io::Output::value_parser())
            .help("Output file (use '-' for stdout)")
        },
        // ... existing cases ...
    }
}
```

**Task**: Auto-generate ValueParser
- [ ] Update macro expansion for Input/Output types
- [ ] Auto-add help text for I/O args
- [ ] Test macro expansion with `cargo expand --example io_basic`
- [ ] Verify clap integration

### 2.3 Create Advanced I/O Example

**File**: `examples/io_advanced.rs` (NEW)

```rust
//! Advanced I/O example with error handling

use clap_noun_verb::io::Input;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use anyhow::Context;

#[derive(Serialize)]
struct ProcessResult {
    input_lines: usize,
    output_lines: usize,
}

#[verb("process", "data")]
fn process_data(
    #[arg(short, long, help = "Input file")]
    input: Input,

    #[arg(short, long, help = "Output file")]
    output: Option<clap_noun_verb::io::Output>,
) -> anyhow::Result<ProcessResult> {
    let content = input.read_to_string()
        .context("Failed to read input")?;

    let input_lines = content.lines().count();
    let output_lines = input_lines;  // Just example

    if let Some(out) = output {
        let result = serde_json::to_string(&ProcessResult {
            input_lines,
            output_lines,
        })?;
        out.write_all(result.as_bytes())?;
    }

    Ok(ProcessResult {
        input_lines,
        output_lines,
    })
}

fn main() -> anyhow::Result<()> {
    Ok(clap_noun_verb::run()?)
}
```

**Task**: Create advanced example
- [ ] Create `examples/io_advanced.rs`
- [ ] Test with files and pipes
- [ ] Verify error messages are helpful

### 2.4 Macro Tests

**File**: `clap-noun-verb-macros/tests/io_macros.rs` (NEW)

```rust
#[test]
fn test_macro_expands_input_type() {
    // Test Input type detection and expansion
}

#[test]
fn test_macro_expands_output_type() {
    // Test Output type detection and expansion
}

#[test]
fn test_macro_adds_value_parser() {
    // Verify ValueParser is added for I/O types
}

#[test]
fn test_macro_backward_compat_string() {
    // Verify String arguments still work
}
```

**Task**: Write macro tests
- [ ] Create integration tests for macro I/O handling
- [ ] Test backward compatibility
- [ ] Test help text generation

---

## Phase 3: Integration (Week 3-4)

### 3.1 Error Handling Integration

**File**: `src/error.rs` (MODIFY)

Add I/O error variants:

```rust
#[derive(Debug)]
pub enum NounVerbError {
    // ... existing ...
    Io(#[from] std::io::Error),
    // Add clio errors when needed
}

impl From<anyhow::Error> for NounVerbError {
    fn from(err: anyhow::Error) -> Self {
        NounVerbError::Custom(err.to_string())
    }
}
```

**Task**: Integrate error handling
- [ ] Add I/O error variants to NounVerbError
- [ ] Update error display formatting
- [ ] Test error handling in examples

### 3.2 TelemetryProfile Enhancement

**File**: `src/kernel/telemetry.rs` (MODIFY)

```rust
impl TelemetryProfile {
    /// Get optimal buffer size based on verbosity
    pub fn io_buffer_size(&self) -> usize {
        match self.verbosity() {
            VerbosityLevel::Debug => 4096,   // Smaller for testing
            _ => 65536,  // 64KB standard
        }
    }

    /// Check if path validation should be strict
    pub fn validate_paths(&self) -> bool {
        self.is_verbose()
    }
}
```

**Task**: Enhance TelemetryProfile
- [ ] Add I/O-related methods
- [ ] Test with different verbosity levels

### 3.3 OutputPipeline Integration

**File**: `src/kernel/output.rs` (MODIFY)

```rust
impl OutputPipeline {
    /// Write formatted output to a sink
    pub fn write_to<T: Serialize>(
        result: T,
        output: &io::Output,
        profile: &TelemetryProfile,
    ) -> Result<()> {
        let formatted = match profile.format() {
            OutputFormat::Json => serde_json::to_string(&result)?,
            OutputFormat::Yaml => serde_yaml::to_string(&result)?,
            // ... other formats ...
        };
        output.write_all(formatted.as_bytes())?;
        Ok(())
    }
}
```

**Task**: Integrate OutputPipeline
- [ ] Add write_to method for I/O output
- [ ] Test with various formats
- [ ] Verify error handling

### 3.4 Logging Integration

**File**: `src/lib.rs` (NEW module)

Add optional tracing integration:

```rust
//! Optional structured logging with tracing crate
//!
//! Enable with feature flag when needed

#[cfg(feature = "tracing")]
pub mod logging {
    pub use tracing::{info, warn, error, debug, trace};
}
```

**Task**: Add logging support
- [ ] Add tracing feature flag to Cargo.toml
- [ ] Create logging module
- [ ] Document usage
- [ ] Create example with logging

### 3.5 Integration Tests

**File**: `tests/io_integration.rs` (NEW)

```rust
#[test]
fn test_verb_with_file_input() {
    // Test verb receiving file input
}

#[test]
fn test_verb_with_stdout_output() {
    // Test verb writing to stdout
}

#[test]
fn test_stdin_stdout_pipe() {
    // Test stdin -> verb -> stdout pipe
}

#[test]
fn test_file_to_file_pipeline() {
    // Test file input to file output
}

#[test]
fn test_error_on_missing_file() {
    // Test proper error when file doesn't exist
}
```

**Task**: Write integration tests
- [ ] Use assert_cmd for CLI testing
- [ ] Use assert_fs for file operations
- [ ] Use predicates for output assertions
- [ ] Cover common I/O scenarios

---

## Phase 4: Documentation (Week 4-5)

### 4.1 Update README

**File**: `README.md` (MODIFY)

Add I/O section:

```markdown
### How to use file I/O

Use `Input` and `Output` types for stdin/stdout/file handling:

\`\`\`rust
#[verb]
fn process(
    #[arg(short, long)]
    input: Input,

    #[arg(short, long)]
    output: Option<Output>,
) -> Result<ProcessResult> {
    let content = input.read_to_string()?;
    // ... process ...
    Ok(ProcessResult { ... })
}
\`\`\`

The "-" convention is automatically supported:
- `--input -` reads from stdin
- `--output -` writes to stdout
\`\`\`
```

**Task**: Update README
- [ ] Add I/O section with examples
- [ ] Document "-" convention
- [ ] Add error handling examples
- [ ] Link to detailed guide

### 4.2 Create Book Chapter

**File**: `docs/book/src/io_guide.md` (NEW)

Comprehensive guide on:
- Basic file I/O
- stdin/stdout handling
- Error patterns
- Best practices
- Advanced patterns

**Task**: Write documentation
- [ ] Create detailed I/O chapter
- [ ] Add multiple examples
- [ ] Document error scenarios
- [ ] Provide troubleshooting

### 4.3 Update CHANGELOG

**File**: `CHANGELOG.md` (MODIFY)

Add entry for v4.1.0:

```markdown
## [4.1.0] - 2025-XX-XX

### Added
- I/O module with clio integration (Input, Output types)
- Auto-detection of I/O types in #[verb] macro
- Automatic ValueParser and help text for I/O arguments
- Integration with OutputPipeline and TelemetryProfile
- Error handling with anyhow/thiserror
- Structured logging with tracing crate
- CLI testing infrastructure (assert_cmd, predicates, assert_fs)

### Examples
- io_basic.rs - Simple file I/O example
- io_advanced.rs - Advanced I/O with error handling

### Dependencies
- Added: clio, anyhow, thiserror, tracing
- Dev: assert_cmd, predicates, assert_fs
```

**Task**: Update changelog
- [ ] Document new features
- [ ] List new dependencies
- [ ] Link to examples
- [ ] Note breaking changes (if any)

### 4.4 API Documentation

**File**: `src/io/mod.rs` (Enhance doc comments)

Comprehensive documentation with examples.

**Task**: Complete API documentation
- [ ] Add module-level examples
- [ ] Document each public type
- [ ] Add common patterns
- [ ] Run `cargo doc` and verify

### 4.5 Migration Guide

**File**: `docs/MIGRATION.md` (NEW)

Guide for upgrading existing code:

```markdown
## Upgrading to v4.1.0

### Migration: String paths to Input/Output

**Before:**
```rust
#[verb]
fn process(input: String) -> Result<Output> {
    let content = std::fs::read_to_string(&input)?;
    // ...
}
```

**After:**
```rust
#[verb]
fn process(input: Input) -> Result<Output> {
    let content = input.read_to_string()?;
    // ...
}
```
```

**Task**: Write migration guide
- [ ] Document common patterns
- [ ] Provide before/after examples
- [ ] List deprecations (if any)
- [ ] Note backward compatibility

---

## Phase 5: Polish & Release (Week 5-6)

### 5.1 Code Review Checklist

- [ ] All tests pass: `cargo test --all`
- [ ] All examples run correctly
- [ ] Documentation builds: `cargo doc --open`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatting: `cargo fmt --check`
- [ ] MSRV compatible (if applicable)
- [ ] Dependency audit: `cargo audit`

### 5.2 Release Preparation

- [ ] Update version in Cargo.toml (4.0.0 → 4.1.0)
- [ ] Update version in clap-noun-verb-macros/Cargo.toml
- [ ] Update CHANGELOG.md with release date
- [ ] Tag commit: `git tag v4.1.0`
- [ ] Create GitHub release notes

### 5.3 Post-Release

- [ ] Publish to crates.io: `cargo publish`
- [ ] Announce in release notes
- [ ] Update website/docs
- [ ] Monitor for issues/feedback

---

## Testing Strategy

### Unit Tests
- Type detection logic
- Error conversions
- Type helpers

### Integration Tests
- Verb with file input
- Verb with stdout output
- Verb with error conditions
- Backward compatibility

### Example Tests
- All examples build and run
- Examples produce correct output
- Examples handle errors gracefully

### Documentation Tests
- Doc examples compile and run
- Markdown code samples are valid

---

## Success Criteria

Phase 1 (Complete):
- [ ] Dependencies added
- [ ] I/O module created and exported
- [ ] Basic example works
- [ ] Documentation complete
- [ ] All tests pass

Phase 2 (Complete):
- [ ] Macro detects I/O types
- [ ] ValueParser auto-applied
- [ ] Advanced example works
- [ ] Macro tests pass

Phase 3 (Complete):
- [ ] Error handling integrated
- [ ] TelemetryProfile enhanced
- [ ] OutputPipeline integration works
- [ ] Integration tests pass

Phase 4 (Complete):
- [ ] README updated with I/O section
- [ ] Book chapter written
- [ ] Migration guide provided
- [ ] API docs complete

Phase 5 (Complete):
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Ready for release

---

## Notes & Considerations

1. **Clio Version**: Monitor clio crate for updates and API stability
2. **Error Types**: Be prepared for clio's error types if they change
3. **Async I/O**: Future enhancement to support async I/O operations
4. **HTTP Support**: Optional clio feature for HTTP input/output
5. **Performance**: Benchmark buffering strategies
6. **Backward Compat**: Ensure String arguments continue to work

---

## Timeline

- **Week 1-2**: Phase 1 (Foundation)
- **Week 2-3**: Phase 2 (Macro Enhancement)
- **Week 3-4**: Phase 3 (Integration)
- **Week 4-5**: Phase 4 (Documentation)
- **Week 5-6**: Phase 5 (Polish & Release)

**Total**: 4-6 weeks with one developer

---

## Quick Command Reference

```bash
# Development
cargo build
cargo test --all
cargo doc --open
cargo clippy --all-targets
cargo fmt

# Testing
cargo test --all --verbose
cargo test --example io_basic
cargo expand --example io_basic

# Documentation
cargo doc --open --no-deps
cargo test --doc

# Release
cargo publish --dry-run
cargo publish
git tag v4.1.0
```

---

## Questions & Support

- For architecture questions: See TYPER_STYLE_IO_INTEGRATION.md
- For ecosystem research: See CLAP_ECOSYSTEM_RESEARCH.md
- For implementation details: Reference specific files in this roadmap
