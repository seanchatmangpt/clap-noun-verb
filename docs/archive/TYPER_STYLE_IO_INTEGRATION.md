# Integrating I/O Capabilities into clap-noun-verb's Typer-Style Architecture

## Executive Summary

clap-noun-verb's design philosophy is **Typer-style**: write simple functions with attributes, let the framework handle all CLI complexity. This document shows how to integrate ecosystem I/O capabilities (primarily **clio**) while maintaining this philosophy.

**Core Principle**: Keep verb handlers focused on business logic by letting the macro layer handle I/O argument parsing and integration.

---

## Part 1: Understanding Typer-Style Design

### What Makes It "Typer-Style"?

Python's Typer achieves simplicity through:
1. **Function signatures are the contract** - Parameters define arguments
2. **Type inference** - `str` → required arg, `Optional[str]` → optional, `bool` → flag
3. **Automatic wiring** - No manual clap::Command building
4. **Zero boilerplate** - Just add decorator and write logic
5. **Natural return values** - Return data structures, framework handles serialization

### clap-noun-verb Currently Implements This For:
```rust
#[verb("status", "services")]
fn show_status(
    #[arg(short, long)]
    verbose: bool,
) -> Result<ServiceStatus> {
    // Business logic here
    Ok(ServiceStatus { ... })
}
```

The macro:
- ✅ Extracts parameters from function signature
- ✅ Infers types (bool → SetTrue flag)
- ✅ Applies #[arg(...)] attributes
- ✅ Builds clap::Command automatically
- ✅ Serializes return value to JSON

### What's Missing for I/O?

Currently, I/O requires manual handling:
```rust
#[verb]
fn process(
    #[arg(short, long)]
    input: String,  // Just a string path!

    #[arg(short, long)]
    output: Option<String>,  // Still just a string!
) -> Result<ProcessResult> {
    // Manual I/O handling ❌
    let content = std::fs::read_to_string(&input)?;
    let output_file = std::fs::File::create(&output)?;
    // ... process ...
}
```

**Desired Typer-style approach**:
```rust
#[verb]
fn process(
    #[arg(short, long)]
    input: Input,  // Automatically handles stdin/files! ✅

    #[arg(short, long)]
    output: Option<Output>,  // Automatically handles stdout/files! ✅
) -> Result<ProcessResult> {
    // Just do business logic
    let content = input.read_to_string()?;
    // ... process ...
    if let Some(out) = output {
        out.write_all(result.as_bytes())?;
    }
}
```

---

## Part 2: Architecture Design for I/O Integration

### 2.1 Type Hierarchy

**Goal**: Provide ergonomic I/O types that integrate seamlessly with clap and the verb macro.

```
┌─────────────────────────────────────────────────────┐
│  User Function Parameters                           │
├─────────────────────────────────────────────────────┤
│  input: Input              (Typer-style wrapper)    │
│  output: Option<Output>    (Typer-style wrapper)    │
│  data: String              (Existing: works as-is)  │
└──────────────┬──────────────────────────────────────┘
               │
               ▼ (Macro Layer)
┌─────────────────────────────────────────────────────┐
│  #[verb] Macro Processing                           │
├─────────────────────────────────────────────────────┤
│  1. Parse function signature                        │
│  2. Detect Input/Output types                       │
│  3. Apply special clap configuration for I/O        │
│  4. Build appropriate clap::Arg with:               │
│     - ValueParser integration                       │
│     - "-" as stdin/stdout detection                 │
│     - Optional validation                           │
└──────────────┬──────────────────────────────────────┘
               │
               ▼ (Generated Code)
┌─────────────────────────────────────────────────────┐
│  clap::Arg with ValueParserFactory                  │
├─────────────────────────────────────────────────────┤
│  Uses clio::Input/Output under the hood             │
│  Validates paths early                              │
│  Handles "-" → stdin/stdout                         │
│  Defers file operations                             │
└──────────────┬──────────────────────────────────────┘
               │
               ▼ (Runtime)
┌─────────────────────────────────────────────────────┐
│  Verb Handler Execution                             │
├─────────────────────────────────────────────────────┤
│  input: clio::Input (ready to use)                  │
│  Lazy file opening on first read                    │
│  Automatic buffering                                │
│  Proper error context                               │
└─────────────────────────────────────────────────────┘
```

### 2.2 Proposed Module Structure

```
src/
├── kernel/
│   ├── io.rs                 # Current (can coexist with clio)
│   └── ... (existing)
│
├── io/                       # NEW MODULE
│   ├── mod.rs                # Public API
│   ├── input.rs              # Input wrapper
│   ├── output.rs             # Output wrapper
│   ├── typed_io.rs           # Type detection helpers
│   └── error.rs              # I/O-specific errors
│
└── ... (existing modules)
```

### 2.3 New Public API

```rust
// src/io/mod.rs

pub use clio::{Input, Output};

/// Convenience wrapper for optional output
pub type OutputOpt = Option<Output>;

/// I/O error handling aligned with Result<T>
pub use crate::error::Result;

// Re-exported utilities
pub use clio::{InputPath, OutputPath};
```

**Principle**: Lightweight wrappers, expose clio directly for users who need it.

---

## Part 3: Integration with #[verb] Macro

### 3.1 Macro Enhancement Strategy

The macro needs to:
1. **Detect I/O types** in function parameters
2. **Auto-apply ValueParser** for I/O types
3. **Add appropriate help text** ("use '-' for stdin/stdout")
4. **Maintain backward compatibility** (String arguments still work)

### 3.2 Detection Pattern

```rust
// In macro expansion: detect parameter types

fn detect_param_type(ty: &Type) -> ParamType {
    match ty {
        // I/O Detection
        Type::Path(p) if is_ident(p, "Input") => ParamType::Input,
        Type::Path(p) if is_ident(p, "Output") => ParamType::Output,
        Type::Option(opt) if is_inner_type(opt, "Output") => ParamType::OutputOpt,

        // Existing types
        Type::Path(p) if is_ident(p, "String") => ParamType::String,
        Type::Path(p) if is_ident(p, "bool") => ParamType::Flag,
        Type::Path(p) if is_ident(p, "usize") => ParamType::Count,

        // ... existing handling ...
        _ => ParamType::Custom,
    }
}
```

### 3.3 Macro Expansion

**Current behavior for String**:
```rust
#[verb]
fn process(
    #[arg(short, long)]
    input: String,
) -> Result<Output> { ... }

// Expands to something like:
fn process(input: String) -> Result<Output> { ... }
// with clap arg: Arg::new("input").short('i').long("input")
```

**Enhanced behavior for Input**:
```rust
#[verb]
fn process(
    #[arg(short, long)]
    input: Input,  // ← Detected by macro
) -> Result<Output> { ... }

// Expands to:
fn process(input: clio::Input) -> Result<Output> { ... }
// with clap arg: Arg::new("input")
//     .short('i')
//     .long("input")
//     .value_parser(Input::value_parser())  // ← Auto-added!
//     .help("Input file (use '-' for stdin)")  // ← Auto-added!
```

### 3.4 Macro Implementation Sketch

```rust
// In clap-noun-verb-macros/src/lib.rs

#[proc_macro_attribute]
pub fn verb(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut fn_item = parse_macro_input!(input as ItemFn);

    for param in &mut fn_item.sig.inputs {
        if let FnArg::Typed(PatType { ty, .. }) = param {
            match detect_param_type(ty) {
                ParamType::Input => {
                    // Add #[arg(..., value_parser = ...)] with Input::value_parser()
                    enhance_for_input(param);
                }
                ParamType::Output => {
                    // Add #[arg(..., value_parser = ...)] with Output::value_parser()
                    enhance_for_output(param);
                }
                ParamType::OutputOpt => {
                    // Add with Option handling
                    enhance_for_output_opt(param);
                }
                _ => {} // No enhancement needed
            }
        }
    }

    // ... rest of macro expansion ...
}
```

---

## Part 4: Error Handling Integration

### 4.1 Current Error Strategy

```rust
// src/error.rs - Existing
pub type Result<T> = std::result::Result<T, NounVerbError>;

pub enum NounVerbError {
    // ... existing variants ...
}
```

### 4.2 Enhanced Error Strategy

**Option A: Integrate anyhow** (Recommended for verbs)
```rust
// In verb handlers - use anyhow for ergonomics
#[verb]
fn process(input: Input) -> anyhow::Result<Output> {
    // clio errors convert automatically
    let content = input.read_to_string()?;

    // Custom errors work with ?
    let parsed = parse_content(&content)?;

    Ok(Output { ... })
}

// Wrapper converts anyhow::Result to clap_noun_verb::Result in CLI layer
```

**Option B: Extend NounVerbError** (Compatible with existing API)
```rust
pub enum NounVerbError {
    // ... existing ...
    Io(#[from] std::io::Error),
    ClioInput(#[from] clio::InputError),
    ClioOutput(#[from] clio::OutputError),
}
```

### 4.3 Integration with StructuredError

The kernel's `StructuredError` should wrap I/O errors properly:

```rust
// kernel/output.rs - Enhancement
impl From<std::io::Error> for StructuredError {
    fn from(err: std::io::Error) -> Self {
        StructuredError {
            code: "io_error".to_string(),
            message: err.to_string(),
            context: None,
            exit_code: 1,
        }
    }
}

impl From<clio::InputError> for StructuredError {
    fn from(err: clio::InputError) -> Self {
        StructuredError {
            code: "input_error".to_string(),
            message: format!("Failed to open input: {}", err),
            context: Some(vec![("type".to_string(), "input".to_string())]),
            exit_code: 1,
        }
    }
}
```

---

## Part 5: Integration with Kernel & Telemetry

### 5.1 TelemetryProfile Enhancement

Current: Handles verbosity, color, format
New: Add I/O-related telemetry

```rust
// kernel/telemetry.rs - Enhancement

pub struct TelemetryProfile {
    // ... existing fields ...

    // New fields
    pub buffer_size: usize,  // For BufferedReader/Writer
    pub validate_paths: bool,  // Pre-validate with clio
}

impl TelemetryProfile {
    pub fn io_buffer_size(&self) -> usize {
        if self.is_debug() {
            4096  // Smaller for testing
        } else {
            65536  // 64KB standard
        }
    }
}
```

### 5.2 Integration Pattern

```rust
#[verb]
fn process(
    #[arg(short, long)]
    input: Input,
) -> Result<Output> {
    // Access telemetry in verb context (optional)
    let args = get_verb_args();  // Contains verbosity info

    // Or just use clio directly
    let content = input.read_to_string()?;

    Ok(Output { ... })
}
```

### 5.3 OutputPipeline Integration

```rust
// kernel/output.rs - Enhancement

impl OutputPipeline {
    /// Write output to sink (file or stdout)
    pub fn write_to<T: Serialize>(
        result: T,
        output: &Output,  // clio::Output
        profile: &TelemetryProfile,
    ) -> Result<()> {
        let formatted = Self::format(&result, profile)?;
        output.write_all(formatted.as_bytes())?;
        Ok(())
    }
}

// Usage in verb:
#[verb]
fn process(
    input: Input,
    #[arg(short, long)]
    output: Option<Output>,
) -> Result<ProcessOutput> {
    let result = process_data(&input)?;

    if let Some(out) = output {
        OutputPipeline::write_to(&result, &out, &profile)?;
    } else {
        // Auto stdout via OutputPipeline
        OutputPipeline::render(Ok(result), &profile)?;
    }

    Ok(result)
}
```

---

## Part 6: Complete Example

### 6.1 Before (Current Pattern)

```rust
// services.rs
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct ProcessResult {
    lines: usize,
    status: String,
}

#[verb]
fn process(
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
) -> Result<ProcessResult> {
    // Manual I/O handling ❌
    let content = fs::read_to_string(&input)
        .map_err(|e| clap_noun_verb::NounVerbError::from(e))?;

    let lines = content.lines().count();

    if let Some(out_path) = output {
        let result = format!(r#"{{"lines":{}}}"#, lines);
        fs::write(&out_path, result)
            .map_err(|e| clap_noun_verb::NounVerbError::from(e))?;
    } else {
        println!(r#"{{"lines":{}}}"#, lines);
    }

    Ok(ProcessResult {
        lines,
        status: "success".to_string(),
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

### 6.2 After (With I/O Integration)

```rust
// services.rs
use clap_noun_verb::Result;
use clap_noun_verb::io::{Input, Output};  // ← New I/O types
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct ProcessResult {
    lines: usize,
    status: String,
}

#[verb]
fn process(
    #[arg(short, long)]
    input: Input,  // ← Automatically handles stdin/files

    #[arg(short, long)]
    output: Option<Output>,  // ← Automatically handles stdout/files
) -> Result<ProcessResult> {
    // Pure business logic ✅
    let content = input.read_to_string()?;
    let lines = content.lines().count();

    // Automatic stdout via OutputPipeline
    Ok(ProcessResult {
        lines,
        status: "success".to_string(),
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

**Benefits**:
- No manual I/O handling
- stdin/stdout handled automatically
- "-" convention supported
- Early validation of paths
- Works with OutputPipeline format selection
- Consistent with Typer philosophy

### 6.3 Advanced Example

```rust
#[derive(Serialize)]
struct TransformResult {
    input_lines: usize,
    output_lines: usize,
    duration_ms: u64,
}

#[verb]
fn transform(
    #[arg(short, long, help = "Input file or '-' for stdin")]
    input: Input,

    #[arg(short, long, help = "Output file or '-' for stdout")]
    output: Option<Output>,

    #[arg(short, long, default_value = "utf8")]
    encoding: String,

    #[arg(short, long)]
    verbose: bool,
) -> anyhow::Result<TransformResult> {
    let start = std::time::Instant::now();

    // Read with error context
    let content = input.read_to_string()
        .context("Failed to read input")?;

    // Process
    let transformed = process_content(&content, &encoding)?;

    // Write output (or use default pipeline)
    if let Some(out) = output {
        out.write_all(transformed.as_bytes())?;
    } else {
        // Using OutputPipeline for automatic format handling
        println!("{}", transformed);
    }

    Ok(TransformResult {
        input_lines: content.lines().count(),
        output_lines: transformed.lines().count(),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}
```

---

## Part 7: Backward Compatibility

### 7.1 String Arguments Still Work

```rust
#[verb]
fn old_style(
    #[arg(short, long)]
    path: String,  // ← Still works as before
) -> Result<Output> {
    // Manual handling if needed, but could use Input now
    Ok(Output { ... })
}
```

### 7.2 Gradual Migration Path

1. **Phase 1**: Add `clap_noun_verb::io` module with re-exports
2. **Phase 2**: Update macro to detect I/O types
3. **Phase 3**: Migrate examples to new pattern
4. **Phase 4**: Document in book and README

---

## Part 8: Implementation Plan

### Phase 1: Foundation (1-2 weeks)
- [ ] Create `src/io/mod.rs` with re-exports of clio types
- [ ] Add `clio` to Cargo.toml with `clap-parse` feature
- [ ] Document I/O module in lib.rs
- [ ] Create basic example: `examples/io_basic.rs`
- [ ] Add tests for I/O types

### Phase 2: Macro Enhancement (2-3 weeks)
- [ ] Enhance `#[verb]` macro to detect I/O types
- [ ] Auto-apply `ValueParser` for Input/Output
- [ ] Add auto-generated help text for I/O args
- [ ] Update macro tests
- [ ] Create example: `examples/io_advanced.rs`

### Phase 3: Integration (1-2 weeks)
- [ ] Integrate with kernel/TelemetryProfile
- [ ] Integrate with OutputPipeline
- [ ] Error handling integration
- [ ] Test with real-world workflows

### Phase 4: Documentation (1 week)
- [ ] Update README.md with I/O section
- [ ] Add book chapter on I/O patterns
- [ ] Document error handling best practices
- [ ] Migration guide from manual I/O
- [ ] Advanced examples

---

## Part 9: Dependency Changes

### Cargo.toml Addition

```toml
[dependencies]
clap-noun-verb = "4.0"
clap-noun-verb-macros = "4.0"

# I/O Handling
clio = { version = "0.3", features = ["clap-parse"] }

# Error Handling
anyhow = "1.0"  # For verb handlers
thiserror = "1.0"  # For custom error types

# Logging (optional but recommended)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

---

## Part 10: Design Principles Summary

1. **Typer Philosophy Maintained**
   - Zero boilerplate in function signatures
   - Type inference for I/O types
   - Automatic framework wiring

2. **Ergonomics First**
   - clio types directly in signatures
   - Automatic "-" handling for stdin/stdout
   - No manual file opening

3. **Backward Compatible**
   - Existing String arguments still work
   - Existing verbs don't need changes
   - Opt-in integration

4. **Composable**
   - I/O with output formats
   - I/O with telemetry
   - I/O with error handling

5. **Minimal Dependencies**
   - Lean on well-maintained crates (clio, anyhow)
   - No extra heavy dependencies
   - Optional logging (tracing)

---

## Conclusion

This design maintains clap-noun-verb's Typer-style philosophy while adding industrial-strength I/O capabilities. Verb authors focus on business logic; the framework handles:

- ✅ CLI argument wiring
- ✅ Input validation
- ✅ stdin/stdout detection
- ✅ File path handling
- ✅ Buffering and performance
- ✅ Error context and reporting
- ✅ Output format selection

The result is code that's as simple as Python Typer but with Rust's type safety and performance.
