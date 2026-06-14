# Macro Development Skill Guide for clap-noun-verb

**A comprehensive guide for developers working on the clap-noun-verb-macros crate**

**Version**: 26.6.1 | **Last Updated**: 2025-02-14 | **Target Audience**: Macro developers

---

## Table of Contents

1. [Proc-Macro Architecture](#proc-macro-architecture)
2. [Core Macro Patterns](#core-macro-patterns)
3. [Compile-Time Validation (Poka-Yoke)](#compile-time-validation-poka-yoke)
4. [Debugging Techniques](#debugging-techniques)
5. [Testing Strategies](#testing-strategies)
6. [Performance Considerations](#performance-considerations)
7. [Frontier Features](#frontier-features)
8. [Best Practices & Anti-Patterns](#best-practices--anti-patterns)

---

## Proc-Macro Architecture

### Project Structure

The `clap-noun-verb-macros` crate is organized as follows:

```
clap-noun-verb-macros/src/
├── lib.rs                    # Main macro entry points
├── validation.rs             # Poka-yoke compile-time checks
├── io_detection.rs           # Type detection for clio::Input/Output
├── meta_framework.rs         # Self-introspecting meta-aware macro
├── telemetry_validation.rs   # Span declaration & usage tracking
├── rdf_generation.rs         # RDF serialization helpers
└── macros/                   # Frontier feature macros
    ├── mod.rs
    ├── fractal_patterns.rs
    ├── federated_network.rs
    ├── semantic_composition.rs
    ├── executable_specs.rs
    ├── learning_trajectories.rs
    ├── reflexive_testing.rs
    └── economic_simulation.rs
```

### Macro Types in the Project

| Macro | Type | Purpose | Input | Output |
|-------|------|---------|-------|--------|
| `#[verb]` | Attribute | Command registration | Function | Wrapper + Registry code |
| `#[noun]` | Attribute | Noun declaration (deprecated) | Function | Deprecation warning |
| `#[arg]` | Attribute | Parameter metadata | Function parameter attrs | Compile error if misused |
| `#[meta_aware]` | Attribute | Self-introspection | Struct | RDF + capability methods |
| `declare_span!` | Declarative | Telemetry span creation | Ident, string | Const assertion |
| `span!` | Procedural | Span instrumentation | Ident, block | Instrumented code |

### The `#[verb]` Macro Flow

```
                    ┌─────────────────────┐
                    │  Function Input     │
                    │  #[verb("status")]  │
                    │  fn show_status()   │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │  Parse Arguments    │
                    │  & Attributes       │
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┬──────────────┬──────────────┐
                    │                     │              │              │
                    ▼                     ▼              ▼              ▼
          ┌──────────────────┐  ┌──────────────┐ ┌──────────────┐ ┌──────┐
          │ Validate Return  │  │ Validate no  │ │ Check Verb   │ │Parse │
          │ Type (Serialize) │  │ CLI Types    │ │ Complexity   │ │Docs  │
          └──────────┬───────┘  └──────────┬───┘ └──────────┬───┘ └──┬───┘
                     │                     │              │         │
                     └─────────────┬───────┴──────────────┴─────────┘
                                   │
                                   ▼
                    ┌─────────────────────────┐
                    │ Signature Analysis:     │
                    │ - Extract arguments     │
                    │ - Determine types       │
                    │ - Check Option<T>/Vec   │
                    │ - Infer actions (count) │
                    └──────────┬──────────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
                    ▼                     ▼
          ┌──────────────────┐  ┌──────────────────┐
          │ Generate Arg     │  │ Generate Wrapper │
          │ Metadata         │  │ Function         │
          │ (ArgMetadata)    │  │ Handler code     │
          └──────────┬───────┘  └──────────┬───────┘
                     │                     │
                     └─────────────┬───────┘
                                   │
                                   ▼
                    ┌─────────────────────────┐
                    │ Emit Distributed Slice  │
                    │ for Runtime Discovery   │
                    │ (linkme integration)    │
                    └──────────┬──────────────┘
                               │
                               ▼
                    ┌─────────────────────────┐
                    │ Generated Code:         │
                    │ - Original function     │
                    │ - Wrapper adapter       │
                    │ - Registry entry        │
                    │ - Duplicate check const │
                    └─────────────────────────┘
```

---

## Core Macro Patterns

### Pattern 1: The #[verb] Macro - Command Registration

The `#[verb]` macro is the heart of the system. It transforms a simple function into a CLI-registered command.

#### Example Usage

```rust
//! A module for managing services
//!
//! This module contains verbs for starting, stopping, and checking service status.

use clap_noun_verb::prelude::*;

/// Show the status of a service
///
/// Checks if the service is running and displays its current state.
///
/// # Arguments
/// * `name` - Service name [requires: "config"]
/// * `config` - Config file path [env: SERVICE_CONFIG] [default: service.toml]
/// * `verbose` - Show detailed output [hide]
#[verb("status")]
fn show_status(
    #[arg(help = "Service name to check")]
    name: String,
    #[arg(env = "SERVICE_CONFIG", default_value = "service.toml")]
    config: String,
    #[arg(short = 'v')]
    verbose: bool,
) -> Result<StatusOutput> {
    // Implementation validates args, calls domain logic, returns serializable output
    Ok(StatusOutput { name, running: true })
}

#[derive(Serialize)]
struct StatusOutput {
    name: String,
    running: bool,
}
```

#### Generated Code (Conceptual)

```rust
// 1. Original function unchanged
fn show_status(name: String, config: String, verbose: bool) -> Result<StatusOutput> {
    Ok(StatusOutput { name, running: true })
}

// 2. Duplicate detection const
#[doc(hidden)]
const __VERB_DUPLICATE_CHECK_services_status_show_status: () = ();

// 3. Wrapper function adapting HandlerInput to function signature
fn __show_status_wrapper(
    __handler_input: HandlerInput
) -> Result<HandlerOutput> {
    let name = __handler_input.args.get("name")
        .ok_or_else(|| NounVerbError::missing_argument("name"))?
        .parse::<String>()?;
    
    let config = __handler_input.args.get("config")
        .map(|v| v.parse::<String>().ok())
        .flatten()
        .unwrap_or_else(|| "service.toml".to_string());
    
    let verbose = __handler_input.args.get("verbose")
        .map(|v| v.parse::<bool>().unwrap_or(false))
        .unwrap_or(false);

    let result = show_status(name, config, verbose)?;
    HandlerOutput::from_data(result)
}

// 4. Registry entry with linkme distributed slice
#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__VERB_REGISTRY)]
static __init_show_status: fn() = {
    fn __register_impl() {
        let noun_name = "services";  // auto-inferred from filename
        let args = vec![
            ArgMetadata {
                name: "name".to_string(),
                required: true,
                is_flag: false,
                help: Some("Service name to check".to_string()),
                // ... more metadata
            },
            // ... more arguments
        ];
        
        CommandRegistry::register_verb_with_args::<_>(
            noun_name,
            "status",
            "Show the status of a service",
            args,
            __show_status_wrapper,
        );
    }
    __register_impl
};
```

### Pattern 2: Raw Identifier Handling (Critical Fix A.5)

The macro strips `r#` prefix from raw identifiers to prevent CLI flag pollution.

```rust
// Parameter name with raw identifier (because `type` is a keyword)
fn configure(#[arg(short = 't')] r#type: String) -> Result<()> {
    // In macro expansion:
    // arg_name_str = "r#type".strip_prefix("r#") → "type"
    // So clap receives --type flag, not --r#type
}
```

#### Implementation Details

```rust
let arg_name_str = arg_name.to_string();
let arg_name_str = arg_name_str
    .strip_prefix("r#")
    .map(str::to_string)
    .unwrap_or(arg_name_str);
```

### Pattern 3: Type Inference for Arguments

The macro infers clap configurations from Rust types without explicit attributes.

```rust
#[verb("process")]
fn process_data(
    // String → required text argument
    input: String,
    
    // Option<T> → optional argument
    filter: Option<String>,
    
    // bool → SetTrue flag
    force: bool,
    
    // usize → Count action (for -v, -vv, -vvv)
    verbosity: usize,
    
    // Vec<T> → multiple values
    tags: Vec<String>,
) -> Result<()> {
    Ok(())
}
```

**Type Mapping Table:**

| Rust Type | Inferred Action | Example CLI |
|-----------|-----------------|------------|
| `String` | Set (required) | `--name "value"` |
| `Option<String>` | Set (optional) | `--name "value"` or omitted |
| `bool` | SetTrue flag | `--force` |
| `usize` | Count | `-vvv` becomes 3 |
| `Vec<T>` | Append/multiple | `--tag a --tag b` |

### Pattern 4: Documentation-Driven Configuration

Doc comments provide clap configuration without attributes.

```rust
/// Sync changes with remote repository
///
/// Connects to the specified remote and synchronizes local changes.
///
/// # Arguments
/// * `remote` - Remote repository URL [requires: "branch"]
/// * `branch` - Branch to sync [default: main] [env: GIT_BRANCH]
/// * `force` - Force sync without confirmation [conflicts: "dry_run"]
/// * `dry_run` - Show what would be synced [group: mode]
/// * `interactive` - Interactive mode [group: mode] [conflicts: "force"]
#[verb]
fn sync_repo(
    remote: String,
    #[arg(default_value = "main")]
    branch: String,
    force: bool,
    dry_run: bool,
    interactive: bool,
) -> Result<SyncResult> {
    Ok(SyncResult { synced: 0 })
}
```

**Doc Comment Tags:**

| Tag | Format | Purpose |
|-----|--------|---------|
| `[group: name]` | `[group: format]` | Mutually exclusive arguments |
| `[requires: arg]` | `[requires: "config"]` | Required dependency |
| `[conflicts: arg]` | `[conflicts: "force"]` | Mutual exclusion |
| `[env: VAR]` | `[env: PORT]` | Environment variable |
| `[default: value]` | `[default: 8080]` | Default value |
| `[hide]` | `[hide]` | Hide from help |
| `[global]` | `[global]` | Propagate to subcommands |
| `[exclusive]` | `[exclusive]` | Cannot use with other args |
| `[value_hint: type]` | `[value_hint: file_path]` | Shell completion hint |

---

## Compile-Time Validation (Poka-Yoke)

The macro implements four critical "error-proofing" checks to prevent common mistakes.

### Gap 1: Forgotten #[verb] Detection (Planned)

**Status**: Framework in place, activation pending.

```rust
// Future: Developers will call check_verb_registration!() at module end
// This will detect functions returning Result<T> without #[verb]

check_verb_registration!();  // Will error if any public functions missed
```

### Gap 2: Duplicate Verb Detection

**Implementation**: Compile-time constant name collision.

When two functions register the same noun+verb combination, the compiler generates conflicting const names:

```rust
// Function 1: services.rs
#[verb("status")]
fn show_status() -> Result<()> { }

// Generates:
const __VERB_DUPLICATE_CHECK_services_status_show_status: () = ();

// Function 2: services.rs (DUPLICATE!)
#[verb("status")]
fn get_status() -> Result<()> { }

// Generates:
const __VERB_DUPLICATE_CHECK_services_status_get_status: () = ();

// ❌ COMPILATION ERROR: duplicate items
```

**Testing Duplicate Detection:**

```rust
#[test]
fn test_duplicate_verb_detection() {
    let tokens = generate_duplicate_detection("status", "services", &parse_quote! { test_fn });
    let tokens_str = tokens.to_string();
    assert!(tokens_str.contains("__VERB_DUPLICATE_CHECK_"));
}
```

### Gap 3: Return Type Serialization Check

**Implementation**: Type path analysis with helpful error messages.

The macro validates that return types implement `serde::Serialize`:

```rust
pub fn validate_return_type(
    return_type: &ReturnType,
    fn_name: &syn::Ident
) -> syn::Result<()> {
    match return_type {
        ReturnType::Default => {
            // Error: no return type
            Err(syn::Error::new(fn_name.span(),
                "Function must return Result<T> where T: Serialize"
            ))
        }
        ReturnType::Type(_, ty) => {
            validate_type_is_serializable(ty, fn_name)?
        }
    }
    Ok(())
}
```

**Error Message Example:**

```
error: Function 'show_status' must return a value that implements serde::Serialize

   Expected return type patterns:
   - Result<T> where T: Serialize
   - Option<T> where T: Serialize
   - T where T: Serialize

   Hint: Add a return type like `Result<Status>` where Status derives Serialize
```

### Gap 4: Attribute Syntax Validation with Typo Suggestions

**Implementation**: Levenshtein distance for helpful error suggestions.

```rust
pub fn validate_arg_attribute_syntax(
    attrs: &[syn::Attribute]
) -> syn::Result<()> {
    // Parses #[arg(...)] and validates all keys
    // If typo detected (Levenshtein distance ≤ 3), suggests correct name
}
```

**Example Error with Suggestion:**

```
error: Unknown argument parameter `shrt` in `#[arg]`. Did you mean `short`?

   Valid parameters are: short, default_value, env, multiple, value_name, ...
```

### Gap 4B: Poka-Yoke FM-1.1 & FM-1.2 Guards

**FM-1.1 - CLI Layer Contamination Guard:**

Prevents business logic in verb functions by enforcing cyclomatic complexity ≤ 5.

```rust
pub fn validate_verb_complexity(input_fn: &ItemFn) -> syn::Result<()> {
    let complexity = calculate_cyclomatic_complexity(input_fn);
    
    if complexity > 5 {
        return Err(syn::Error::new(
            input_fn.sig.ident.span(),
            "Verb function too complex (FM-1.1)\n\
             Problem: Verb functions should delegate to domain logic, \
             not implement it."
        ));
    }
    Ok(())
}
```

**FM-1.2 - Domain Dependency on CLI Types Guard:**

Prevents verb functions from accepting CLI types (ArgMatches, Command, HandlerInput, etc.):

```rust
pub fn validate_no_cli_types_in_params(sig: &syn::Signature) -> syn::Result<()> {
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let Some(error) = check_for_cli_types(&pat_type.ty) {
                return Err(error);
            }
        }
    }
    Ok(())
}
```

**Error Example:**

```
🛡️ Poka-Yoke Guard: CLI type contamination detected (FM-1.2)

   Forbidden types: ArgMatches, Command, VerbContext, VerbArgs, HandlerInput
   Found: VerbArgs

   Problem: Domain functions should not depend on CLI types.
   
   Solution: Use simple typed parameters instead:
   ✅ GOOD:   fn calculate(x: i32, y: i32) -> Result<i32>
   ❌ WRONG:  fn calculate(args: VerbArgs) -> Result<i32>
```

---

## Debugging Techniques

### Technique 1: Macro Expansion Inspection

View generated code with `cargo expand`:

```bash
# Install cargo-expand (one-time)
cargo install cargo-expand

# Expand macro for specific function
cargo expand --lib test_module::show_status

# Expand entire file
cargo expand --lib test_module
```

**Output Example:**

```rust
fn show_status(name: String) -> Result<StatusOutput> {
    Ok(StatusOutput { name, running: true })
}

const __VERB_DUPLICATE_CHECK_services_status_show_status: () = ();

fn __show_status_wrapper(
    __handler_input: ::clap_noun_verb::logic::HandlerInput
) -> ::clap_noun_verb::error::Result<::clap_noun_verb::logic::HandlerOutput> {
    let name = __handler_input.args.get("name")
        .ok_or_else(|| ::clap_noun_verb::error::NounVerbError::missing_argument("name"))?
        .parse::<String>()?;
    let result = show_status(name)?;
    ::clap_noun_verb::logic::HandlerOutput::from_data(result)
}

// ... linkme distributed slice registration
```

### Technique 2: Syn Debugging with quote!

Use `quote!` to pretty-print TokenStreams for inspection:

```rust
// In macro code
let my_tokens = quote! { /* ... */ };
eprintln!("Generated tokens:\n{}", my_tokens);  // Pretty prints

// In tests
#[test]
fn debug_token_generation() {
    let tokens = quote! { 
        const EXAMPLE: () = (); 
    };
    println!("Tokens:\n{}", tokens.pretty()); // Pretty print in test
}
```

### Technique 3: Compile Error Messages

The macro includes detailed, multi-paragraph error messages to guide fixes:

```rust
return Err(syn::Error::new(
    fn_name.span(),
    "Function 'show_status' must return a value that implements serde::Serialize\n\
     \n\
     Expected return type patterns:\n\
     - Result<T> where T: Serialize\n\
     - Option<T> where T: Serialize\n\
     - T where T: Serialize\n\
     \n\
     Hint: Add a return type like `Result<Status>` where Status derives Serialize"
));
```

**Best Practices:**
- Include context (what went wrong)
- List valid patterns
- Provide actionable hints
- Use formatting for clarity

### Technique 4: Testing with Trybuild

Use the `trybuild` crate to test macro error messages:

```rust
// tests/ui/expand_tests.rs
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
    t.compile_succeed("tests/ui/pass/*.rs");
}
```

**Test File Structure:**

```
tests/ui/
├── pass/
│   ├── simple_verb.rs
│   ├── optional_args.rs
│   └── complex_types.rs
└── fail/
    ├── no_return_type.rs
    ├── duplicate_verb.rs
    └── missing_serialize.rs
```

### Technique 5: Proc-Macro Debug Logging

Add debug output in macro code with `proc_macro_error::abort!`:

```rust
use proc_macro_error::abort;

pub fn verb(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Log function name
    eprintln!("🔍 Processing verb: {}", input_fn.sig.ident);
    
    // Log parsed arguments
    eprintln!("📋 Args: {:?}", args.to_string());
    
    // ... macro logic ...
}
```

**Run with output:**

```bash
RUST_LOG=debug cargo build 2>&1 | grep "🔍"
```

---

## Testing Strategies

### Strategy 1: Unit Tests in validation.rs

Test compile-time validation functions in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_validate_return_type_result() {
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<String> {
                Ok("test".to_string())
            }
        };
        assert!(validate_return_type(
            &fn_item.sig.output,
            &fn_item.sig.ident
        ).is_ok());
    }

    #[test]
    fn test_validate_return_type_missing() {
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() {
                println!("test");
            }
        };
        let result = validate_return_type(
            &fn_item.sig.output,
            &fn_item.sig.ident
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string()
            .contains("must return a value"));
    }
}
```

**Test Organization:**

```rust
// Tests for specific validation gaps
#[test]
fn test_gap_2_duplicate_detection() { }

#[test]
fn test_gap_3_return_type_validation() { }

#[test]
fn test_gap_4_attribute_syntax() { }

#[test]
fn test_gap_4b_fm_1_1_complexity() { }

#[test]
fn test_gap_4b_fm_1_2_cli_types() { }
```

### Strategy 2: Integration Tests with Actual Macro Usage

Test macros in the context of real functions:

```rust
// tests/integration_tests.rs
#[test]
fn test_simple_verb_expansion() {
    let input = quote! {
        #[verb("status")]
        fn show_status(name: String) -> Result<StatusOutput> {
            Ok(StatusOutput { name, running: true })
        }
    };
    
    let result = verb(TokenStream::from(quote! { "status" }), 
                     TokenStream::from(input));
    
    // Assert generated code contains expected elements
    let result_str = result.to_string();
    assert!(result_str.contains("__show_status_wrapper"));
    assert!(result_str.contains("__VERB_DUPLICATE_CHECK_"));
    assert!(result_str.contains("__VERB_REGISTRY"));
}
```

### Strategy 3: Compile Fail Tests (UI Tests)

Test that invalid code produces expected compile errors:

```rust
// tests/ui/fail/no_return_type.rs
#[verb("status")]
fn show_status(name: String) {  // ❌ Missing return type!
    println!("{}", name);
}
```

**Running UI Tests:**

```bash
cargo test --test ui_tests
```

**Expected Output:**

```
error: Function 'show_status' must return a value that implements serde::Serialize
   |
   | fn show_status(name: String) {
   |    ^^^^^^^^^^^
   |
   = note: Expected return type patterns:
           - Result<T> where T: Serialize
           ...
```

### Strategy 4: Snapshot Testing

Use `insta` crate for testing macro output:

```rust
#[test]
fn test_arg_metadata_snapshot() {
    let input = quote! {
        fn test(
            #[arg(short = 'v', env = "PORT")]
            port: u16
        ) -> Result<()> { }
    };
    
    let metadata = parse_arg_metadata(&input);
    
    // Automatically creates/updates snapshot
    insta::assert_debug_snapshot!(metadata);
}
```

### Strategy 5: AAA Pattern for Macro Tests

Follow Arrange-Act-Assert:

```rust
#[test]
fn test_verb_with_optional_args() {
    // Arrange: Setup input and expected behavior
    let fn_item: ItemFn = parse_quote! {
        #[verb("process")]
        fn process_data(
            input: String,
            filter: Option<String>,
        ) -> Result<Output> {
            Ok(Output { success: true })
        }
    };
    
    // Act: Execute macro
    let result = verb_macro(&fn_item);
    
    // Assert: Verify output
    let output_str = result.to_string();
    assert!(output_str.contains("filter")); // Argument included
    assert!(output_str.contains("Option"));  // Type preserved
    assert!(!output_str.contains("unwrap")); // No panics
}
```

### Strategy 6: Performance Testing

Verify macros don't exceed SLOs:

```rust
#[test]
#[ignore]  // Only run with --ignored
fn test_macro_performance_slo() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Generate complex verb with many arguments
    for _ in 0..100 {
        let _result = verb(TokenStream::from(quote! { "test" }), 
                          TokenStream::from(complex_fn()));
    }
    
    let elapsed = start.elapsed();
    
    // SLO: <2s for 100 macro expansions
    assert!(elapsed.as_millis() < 2000,
           "Macro expansion too slow: {}ms", elapsed.as_millis());
}
```

### Strategy 7: Fuzz Testing Macro Inputs

Test edge cases with random inputs:

```rust
#[test]
fn test_validate_verb_attribute_syntax_random() {
    let test_cases = vec![
        ("", false),              // Empty args
        ("\"status\"", true),     // Valid
        ("status", false),        // Missing quotes
        ("\"status\", \"noun\"", true),  // Two args
        ("\"a\", \"b\", \"c\"", false),  // Too many
    ];
    
    for (input, should_pass) in test_cases {
        let tokens = proc_macro2::TokenStream::from_str(input).unwrap();
        let fn_item = parse_quote! { fn test() -> Result<()> { Ok(()) } };
        
        let result = validate_verb_attribute_syntax(&tokens, &fn_item);
        assert_eq!(result.is_ok(), should_pass, 
                   "Failed for input: {}", input);
    }
}
```

### Test Execution Commands

```bash
# Run all macro tests
cargo test --lib -p clap-noun-verb-macros

# Run specific test
cargo test test_validate_return_type_result --lib

# Run tests with output
cargo test --lib -- --nocapture

# Run deterministic (single-threaded)
cargo test --lib --quiet -- --test-threads=1

# Run with specific filter
cargo test validation --lib
```

---

## Performance Considerations

### 1. Token Stream Processing Efficiency

**Pattern: Minimize TokenStream conversions**

```rust
// ❌ INEFFICIENT: Multiple to_string() calls
fn bad_check(ty: &Type) -> bool {
    let s1 = ty.to_token_stream().to_string();
    let s2 = ty.to_token_stream().to_string();  // Duplicate!
    s1.contains("Option") || s2.contains("Vec")
}

// ✅ EFFICIENT: Single conversion
fn good_check(ty: &Type) -> bool {
    let type_str = ty.to_token_stream().to_string();
    type_str.contains("Option") || type_str.contains("Vec")
}
```

### 2. Symbol Lookup Optimization

**Pattern: Cache repeated lookups**

```rust
// ❌ INEFFICIENT: Walking attribute list multiple times
fn bad_validation(attrs: &[syn::Attribute]) -> Option<String> {
    // First pass: find #[arg]
    let arg_attr = attrs.iter().find(|a| a.path().is_ident("arg"));
    
    // Second pass: find #[doc]
    let doc_attr = attrs.iter().find(|a| a.path().is_ident("doc"));
    
    // Third pass: find #[validate]
    let val_attr = attrs.iter().find(|a| a.path().is_ident("validate"));
}

// ✅ EFFICIENT: Single pass
fn good_validation(attrs: &[syn::Attribute]) -> (Option<&Attribute>, Option<&Attribute>) {
    let mut arg_attr = None;
    let mut doc_attr = None;
    
    for attr in attrs {
        if arg_attr.is_none() && attr.path().is_ident("arg") {
            arg_attr = Some(attr);
        }
        if doc_attr.is_none() && attr.path().is_ident("doc") {
            doc_attr = Some(attr);
        }
        if arg_attr.is_some() && doc_attr.is_some() {
            break;  // Early exit
        }
    }
    (arg_attr, doc_attr)
}
```

### 3. SLO Targets

From CLAUDE.md:

- **Incremental compilation**: ≤2s (currently 0.66s)
- **Binary size**: ≤10MB (currently 2.2MB)
- **CLI generation**: ≤100ms

**Monitoring Performance:**

```bash
# Measure incremental compile time
time cargo check

# Measure macro expansion time
cargo build -p clap-noun-verb-macros --release -j1

# Profile with flamegraph (requires cargo-flamegraph)
cargo flamegraph --bin my_cli
```

### 4. Memory-Efficient Pattern Matching

**Pattern: Use string matching for heuristic checks**

```rust
// ❌ OVER-ENGINEERED: Full parse for simple check
fn bad_has_option(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        // Complex nested matching
        if let Some(segment) = path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                // ...
            }
        }
    }
    false
}

// ✅ EFFICIENT: String pattern for heuristic
fn good_has_option(ty: &Type) -> bool {
    let type_str = ty.to_token_stream().to_string();
    type_str.starts_with("Option <")  // Fast string check
}
```

### 5. Avoid Unnecessary TokenStream Allocations

```rust
// ❌ Creates new TokenStream in loop
fn bad_loop() {
    for field in fields {
        let ts = quote! { let #field = /* ... */; };  // New allocation each iteration
        results.push(ts);
    }
}

// ✅ Single concatenation
fn good_loop() {
    let mut all_tokens = TokenStream::new();
    for field in fields {
        let ts = quote! { let #field = /* ... */; };
        all_tokens.extend(ts);  // Reuse buffer
    }
    all_tokens
}
```

---

## Frontier Features

The macro crate includes opt-in frontier features for advanced use cases. These are feature-gated and may have limited support.

### Feature: meta-framework → #[meta_aware]

**Purpose**: Self-introspecting code generation for agents and knowledge bases.

**Location**: `clap-noun-verb-macros/src/meta_framework.rs`

**Example:**

```rust
#[meta_aware]
struct AgentCapabilities {
    name: String,
    max_concurrency: usize,
    supports_async: bool,
}

let caps = AgentCapabilities::new("worker".to_string(), 10, true);

// Auto-generated methods:
let rdf = caps.introspect_capabilities();      // RDF triples
let opts = caps.query_optimizations();         // Optimization hints
let schema = AgentCapabilities::introspect_schema();  // Type schema
```

**Generated Output:**

```text
:instance a :AgentCapabilities ;
    cnv:hasField [ cnv:name "name" ; cnv:type xsd:string ; cnv:value "worker" ] ;
    cnv:hasField [ cnv:name "max_concurrency" ; cnv:type xsd:integer ; cnv:value "10" ] ;
    cnv:fieldCount "3" .
```

### Feature: federated-network → #[federated]

**Purpose**: Distributed compute topology specification.

**Status**: Placeholder macros in `src/macros/federated_network.rs`

### Feature: executable-specs → #[spec]

**Purpose**: Executable specification generation.

**Status**: Placeholder macros in `src/macros/executable_specs.rs`

### Feature: fractal-patterns → #[semantic_composable]

**Purpose**: Recursive, composable type patterns.

**Status**: Placeholder macros in `src/macros/fractal_patterns.rs`

### Feature: reflexive-testing → #[auto_test]

**Purpose**: Automatic test generation from type structure.

**Status**: Placeholder macros in `src/macros/reflexive_testing.rs`

### Feature: learning-trajectories → #[competency]

**Purpose**: Learning path and skill tree specification.

**Status**: Placeholder macros in `src/macros/learning_trajectories.rs`

**Example Skeleton:**

```rust
// Future use
#[competency("cli_development")]
#[requires("rust_basics")]
#[requires("proc_macro_foundations")]
fn expert_macro_developer() {
    // Competency definition
}
```

---

## Best Practices & Anti-Patterns

### ✅ Best Practices

#### 1. Always Use Helper Errors with Context

```rust
// ✅ GOOD: Multi-line error with context
return Err(syn::Error::new(
    arg.span(),
    "Argument 'shrt' in #[verb] must be a string literal for function 'show_status'\n\
     \n\
     Found: identifier 'shrt'\n\
     Expected: string like \"status\"\n\
     \n\
     Did you mean: short?"
));

// ❌ BAD: Single-line error
return Err(syn::Error::new(arg.span(), "Invalid arg"));
```

#### 2. Process Tokens Efficiently

```rust
// ✅ GOOD: Single pass with early exit
for attr in attrs {
    if attr.path().is_ident("arg") {
        return parse_arg_attribute(attr);
    }
    if attr.path().is_ident("doc") {
        // Handle docs
    }
}

// ❌ BAD: Multiple iterations
let arg_attr = attrs.iter().find(|a| a.path().is_ident("arg"));
let doc_attr = attrs.iter().find(|a| a.path().is_ident("doc"));
// ...more iterations...
```

#### 3. Preserve Source Span Information

```rust
// ✅ GOOD: Use span() to point to exact error location
let error = syn::Error::new(
    invalid_arg.span(),  // Points to user's code
    "Invalid configuration"
);

// ❌ BAD: Generic span
let error = syn::Error::new(
    proc_macro2::Span::call_site(),  // Points to macro invocation
    "Invalid configuration"
);
```

#### 4. Test Edge Cases and Error Paths

```rust
// ✅ GOOD: Comprehensive test coverage
#[test]
fn test_parse_arg_attributes() {
    // Test valid cases
    assert!(parse_valid_arg_attr().is_ok());
    
    // Test error cases
    assert!(parse_missing_quotes().is_err());
    assert!(parse_invalid_key().is_err());
    assert!(parse_empty_args().is_ok());
    
    // Test edge cases
    assert!(parse_unicode_in_help().is_ok());
    assert!(parse_very_long_names().is_ok());
}
```

#### 5. Document Macro Behavior Thoroughly

```rust
/// Attribute macro for registering a verb command
///
/// # Usage
///
/// ```rust,ignore
/// #[verb("status")]
/// fn show_status(name: String) -> Result<StatusOutput> {
///     Ok(StatusOutput { name, running: true })
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// 1. Duplicate detection const
/// 2. Wrapper function adapting HandlerInput
/// 3. linkme distributed slice entry
///
/// # Arguments
///
/// - First arg (optional): verb name - auto-inferred from function name if omitted
/// - Second arg (optional): noun name - auto-detected from filename if omitted
///
/// # Validation
///
/// The macro performs compile-time checks:
/// - Return type must implement Serialize
/// - Verb function complexity ≤ 5 (FM-1.1)
/// - No CLI types in parameters (FM-1.2)
///
/// # Errors
///
/// Compilation fails if:
/// - Return type doesn't implement Serialize (Gap 3)
/// - Verb attribute syntax is invalid (Gap 4)
/// - Duplicate verb+noun combination detected (Gap 2)
#[proc_macro_attribute]
pub fn verb(args: TokenStream, input: TokenStream) -> TokenStream { }
```

#### 6. Use Builder Patterns for Complex Code Generation

```rust
// ✅ GOOD: Structured code generation
struct WrapperGenerator {
    fn_name: syn::Ident,
    args: Vec<ArgExtraction>,
    arg_calls: Vec<TokenStream>,
}

impl WrapperGenerator {
    fn generate(&self) -> TokenStream {
        let fn_name = &self.fn_name;
        let extractions = &self.args;
        let calls = &self.arg_calls;
        
        quote! {
            fn wrapper(__handler_input: HandlerInput) -> Result<HandlerOutput> {
                #(#extractions)*
                let result = #fn_name(#(#calls),*)?;
                HandlerOutput::from_data(result)
            }
        }
    }
}
```

### ❌ Anti-Patterns to Avoid

#### 1. Never Panic in Macro Code

```rust
// ❌ BAD: Will crash user's compiler
if args_vec.len() > 2 {
    panic!("Too many arguments!");  // NEVER!
}

// ✅ GOOD: Return Err for compile-time error
if args_vec.len() > 2 {
    return Err(syn::Error::new(
        args.span(),
        "Too many arguments in #[verb]"
    )).to_compile_error().into();
}
```

#### 2. Never Use unwrap() in Macro Code

```rust
// ❌ BAD: Will panic if attribute parsing fails
let segment = type_path.path.segments.last().unwrap();  // CRASH!

// ✅ GOOD: Handle None gracefully
let segment = match type_path.path.segments.last() {
    Some(s) => s,
    None => {
        return Err(syn::Error::new(
            type_path.span(),
            "Invalid type path"
        ));
    }
};
```

#### 3. Don't Ignore Error Types

```rust
// ❌ BAD: Silently ignores parsing errors
let _ = parser.parse2(tokens);  // Error lost!

// ✅ GOOD: Handle errors explicitly
match parser.parse2(tokens) {
    Ok(args) => process_args(args),
    Err(e) => {
        return Err(syn::Error::new(
            e.span(),
            format!("Failed to parse arguments: {}", e)
        ));
    }
}
```

#### 4. Don't Generate Code with Hardcoded Types

```rust
// ❌ BAD: Only works for String, hardcoded
let arg_extraction = quote! {
    let #arg_name = __handler_input.args.get(#arg_name_str)
        .unwrap()
        .parse::<String>()?;
};

// ✅ GOOD: Uses actual inferred type
let inner_ty = extract_inner_type(&pat_type.ty);
let arg_extraction = quote! {
    let #arg_name = __handler_input.args.get(#arg_name_str)
        .ok_or_else(|| NounVerbError::missing_argument(#arg_name_str))?
        .parse::<#inner_ty>()?;
};
```

#### 5. Don't Silently Truncate Data

```rust
// ❌ BAD: Silently skips invalid attributes
for attr in attrs {
    if let Ok(config) = parse_arg_config(attr) {
        // Only valid ones processed - invalid ones ignored
    }
}

// ✅ GOOD: Report issues
for attr in attrs {
    let config = parse_arg_config(attr)?;  // Propagate error
    // Process config
}
```

---

## Quick Reference

### Macro File Organization

| File | Purpose | Key Functions |
|------|---------|---------------|
| `lib.rs` | Entry points | `#[verb]`, `#[noun]`, `#[arg]`, `#[meta_aware]` |
| `validation.rs` | Compile-time checks | `validate_return_type()`, `validate_verb_complexity()` |
| `io_detection.rs` | Type inference | `detect_io_type()`, `is_option_type()` |
| `meta_framework.rs` | Introspection | `generate_meta_aware()` |
| `telemetry_validation.rs` | Span tracking | `generate_span_declaration()` |
| `rdf_generation.rs` | RDF output | RDF serialization helpers |

### Common Type Checks

```rust
is_option_type(&ty)           // Option<T>?
is_bool_type(&ty)             // bool?
is_vec_type(&ty)              // Vec<T>?
extract_inner_type(&ty)       // Get T from Option<T>
extract_option_inner(&ty)     // Get T from Option<T>
detect_io_type(&ty)           // Input/Output/etc?
```

### Common Parsing Patterns

```rust
// Parse comma-separated expressions
let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
let args = parser.parse2(tokens)?;

// Parse #[attr(...)] syntax
if let syn::Meta::List(list) = &attr.meta {
    let meta_list = Punctuated::<syn::Meta, Token![,]>::parse_terminated
        .parse2(list.tokens.clone())?;
}

// Extract ident from path
if let Some(ident) = path.get_ident() {
    let name = ident.to_string();
}
```

### Code Generation Snippets

```rust
// Check if type is serializable
if !is_serializable_type(ty) {
    return Err(error);
}

// Generate type coercion
let coercion = if is_option {
    quote! { Some(#inner_ty) }
} else {
    quote! { #inner_ty }
};

// Generate error handling
let error_handling = quote! {
    .ok_or_else(|| NounVerbError::missing_argument(#name))?
};

// Generate argument extraction
let extraction = quote! {
    let #name = __handler_input.args.get(#name_str)
        #error_handling
        .parse::<#inner_ty>()?;
};
```

---

## Additional Resources

### Internal Documentation
- **CLAUDE.md** - Project guidelines and build commands
- **Cargo.toml** - Dependencies and feature gates
- **syn crate docs** - https://docs.rs/syn/latest/syn/
- **quote crate docs** - https://docs.rs/quote/latest/quote/

### External Learning Resources
- **Proc Macro Workshop** - https://github.com/dtolnay/proc-macro-workshop
- **Little Book of Macros** - https://veykril.github.io/tlbom/
- **Rust Macro Design** - https://rustyyato.github.io/type-level-programming/

### Testing & Debugging Tools
- `cargo expand` - View macro-generated code
- `cargo-tree` - Visualize dependency tree
- `trybuild` - UI testing for macros
- `insta` - Snapshot testing
- `cargo-flamegraph` - Performance profiling

---

## Conclusion

The clap-noun-verb macro system combines compile-time validation, automated code generation, and architectural patterns to provide a safe, efficient CLI development experience. When extending or modifying these macros:

1. **Respect the Poka-Yoke gaps** - These prevent common mistakes
2. **Test thoroughly** - Unit, integration, and UI tests catch regressions
3. **Document behavior** - Clear docs help future maintainers
4. **Monitor performance** - Keep incremental compile times fast
5. **Preserve span information** - Helpful errors guide users to fixes

Questions? Refer to specific sections above or check the source code comments for additional context.
