# Macro Development Skill Guide for clap-noun-verb-macros

**Version**: 26.6.1  
**Target Audience**: Developers extending the clap-noun-verb-macros procedural macro crate  
**Last Updated**: 2026-06-14

## Table of Contents

1. [Macro Architecture Overview](#macro-architecture-overview)
2. [Core Proc-Macro Patterns](#core-proc-macro-patterns)
3. [Compile-Time Validation (Poka-Yoke)](#compile-time-validation-poka-yoke)
4. [Common Macro Debugging Techniques](#common-macro-debugging-techniques)
5. [Validation Pattern Library](#validation-pattern-library)
6. [Testing Strategies](#testing-strategies)
7. [Performance Optimization](#performance-optimization)
8. [Real-World Examples](#real-world-examples)
9. [Troubleshooting](#troubleshooting)

---

## Macro Architecture Overview

### High-Level Design

The clap-noun-verb-macros crate provides procedural attribute macros for declarative CLI command registration. The architecture follows these principles:

- **Compile-Time Registration**: Verbs are registered at macro expansion time using `linkme` distributed slices
- **Type-Safe Code Generation**: Generates wrapper functions that adapt `HandlerInput` to function signatures
- **Layered Validation**: Multiple validation passes catch errors early (return types, syntax, complexity)
- **Zero-Runtime Overhead**: All validation happens at compile time

### Macro Entry Points

Located in `clap-noun-verb-macros/src/lib.rs`:

| Macro | Type | Purpose | Entry Point |
|-------|------|---------|------------|
| `#[verb]` | proc_macro_attribute | Register command handler | Line 330 |
| `#[noun]` | proc_macro_attribute | (Deprecated) Register noun | Line 285 |
| `#[arg]` | proc_macro_attribute | Parameter metadata | Line 71 |
| `#[meta_aware]` | proc_macro_attribute | Self-introspection | Line 109 |
| `declare_span!` | proc_macro | Telemetry declaration | Line 139 |
| `span!` | proc_macro | Telemetry instrumentation | Line 217 |

### Module Organization

```
clap-noun-verb-macros/src/
├── lib.rs                 # Main macro definitions (2800+ lines)
├── validation.rs          # Poka-Yoke validation checks (700+ lines)
├── io_detection.rs        # I/O type detection (215 lines)
├── telemetry_validation.rs # Span validation
├── rdf_generation.rs      # RDF triple generation
├── meta_framework.rs      # Meta-aware introspection
└── macros/                # Frontier feature macros
    ├── fractal_patterns.rs
    ├── federated_network.rs
    ├── semantic_composition.rs
    ├── executable_specs.rs
    ├── learning_trajectories.rs
    └── reflexive_testing.rs
```

---

## Core Proc-Macro Patterns

### Pattern 1: Token Stream Parsing with `syn`

The `syn` crate is essential for parsing Rust tokens into AST structures.

```rust
use syn::{parse_macro_input, ItemFn, Signature};
use quote::quote;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn my_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse attribute arguments
    let args_tokens = proc_macro2::TokenStream::from(args);
    
    // Parse input as a function
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Work with the AST
    let fn_name = &input_fn.sig.ident;
    
    // Generate code with quote!
    let expanded = quote! {
        // Generated code here
        #input_fn
    };
    
    expanded.into()
}
```

**Key Points**:
- Use `parse_macro_input!` for standard items (ItemFn, ItemStruct, etc.)
- Use `proc_macro2::TokenStream::from()` to work with token streams in proc_macro2
- Always convert back with `.into()` when returning from proc_macro context

### Pattern 2: Attribute Argument Parsing

From `lib.rs` lines 367-384:

```rust
// Parse verb name from args using Punctuated parser
let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
let args_vec: syn::punctuated::Punctuated<_, _> = 
    match Parser::parse2(parser, args_tokens.clone()) {
        Ok(args) => args,
        Err(_) => {
            // Fallback: extract verb name from function name
            let verb_name = extract_verb_name_from_fn_name(&input_fn);
            return generate_verb_registration(input_fn, verb_name, None, None, HashMap::new());
        }
    };

// Extract individual arguments
let verb_name = match &args_vec[0] {
    syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.value(),
    _ => return error_token("First argument must be a string literal"),
};
```

**Pattern for Safe Extraction**:
1. Use `Punctuated::parse_terminated` for comma-separated values
2. Check bounds before indexing: `if args_vec.len() > 2 { error })`
3. Match on expression types to validate (Lit::Str, Lit::Int, etc.)
4. Provide meaningful error messages with `syn::Error`

### Pattern 3: Error Handling with `syn::Error`

From `validation.rs` lines 28-50:

```rust
pub fn validate_return_type(return_type: &ReturnType, fn_name: &syn::Ident) -> syn::Result<()> {
    match return_type {
        ReturnType::Default => {
            return Err(syn::Error::new(
                fn_name.span(),
                format!(
                    "Function '{}' must return a value that implements serde::Serialize\n\
                     \n\
                     Expected return type patterns:\n\
                     - Result<T> where T: Serialize\n\
                     - Option<T> where T: Serialize\n\
                     - T where T: Serialize",
                    fn_name
                ),
            ));
        }
        ReturnType::Type(_, ty) => {
            validate_type_is_serializable(ty, fn_name)?;
        }
    }
    Ok(())
}
```

**Best Practices**:
- Use `syn::Error::new(span, message)` to attach location info
- Include multi-line error messages with expected vs. found patterns
- Use `.span()` method to pinpoint errors in source code
- Chain with `?` operator for composition: `validate_x()?.validate_y()?`

### Pattern 4: Type Inspection and Validation

From `lib.rs` lines 1730-1754:

```rust
/// Check if type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

/// Extract inner type from Option<T>
fn extract_inner_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    return inner.clone();
                }
            }
        }
    }
    ty.clone()
}
```

**Type Checking Strategy**:
1. Match on `syn::Type::Path` for most types
2. Access last segment: `.path.segments.last()`
3. For generics, check `.arguments` as `AngleBracketed`
4. Extract first generic arg from `.args.first()`

### Pattern 5: Code Generation with `quote!`

From `lib.rs` lines 1622-1725:

```rust
let expanded = quote! {
    // Original function (annotated) stays
    #output_fn
    
    // Duplicate detection for compile-time safety
    #duplicate_check
    
    // Wrapper adapts HandlerInput to function signature
    fn #wrapper_name(__handler_input: ::clap_noun_verb::logic::HandlerInput) 
        -> ::clap_noun_verb::error::Result<::clap_noun_verb::logic::HandlerOutput> 
    {
        // Extract arguments from HandlerInput
        #(#arg_extractions)*
        
        // Call original function
        let result = #fn_name(#(#arg_calls),*)?;
        
        // Convert to HandlerOutput
        ::clap_noun_verb::logic::HandlerOutput::from_data(result)
    }
    
    // Auto-registration with linkme distributed slice
    #[allow(non_upper_case_globals)]
    #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__VERB_REGISTRY)]
    static #init_fn_name: fn() = {
        fn __register_impl() {
            // Registration logic
            ::clap_noun_verb::cli::registry::CommandRegistry::register_verb_with_args::<_>(
                noun_name_static,
                verb_name_final,
                #about_str,
                args,
                #wrapper_name,
            );
        }
        __register_impl
    };
};
```

**Code Generation Patterns**:
- Use `#variable` to interpolate identifiers and expressions
- Use `#(#vec_variable),*` for iterating vectors
- Fully qualify types: `::crate::path::Type` (no relative paths in generated code)
- Return function pointers, not closures, for `linkme` compatibility

---

## Compile-Time Validation (Poka-Yoke)

Poka-Yoke is a manufacturing principle: "mistake-proofing" by making errors impossible at the source.

### The Four Validation Gaps

From `validation.rs`:

#### Gap 1: Forgotten `#[verb]` Detection

**Problem**: Developer forgets `#[verb]` on handler function

**Solution**: Document expected patterns (currently manual, but could be automated)

```rust
// BAD: Missing #[verb]
fn show_status() -> Result<Status> { ... }

// GOOD: Marked with #[verb]
#[verb("status")]
fn show_status() -> Result<Status> { ... }
```

#### Gap 2: Duplicate Verb Registration

From `validation.rs` lines 268-290:

**Problem**: Same verb registered twice causes silent shadowing

**Solution**: Compile-time const collision detection

```rust
pub fn generate_duplicate_detection(
    verb_name: &str,
    noun_name: &str,
    fn_name: &syn::Ident,
) -> TokenStream {
    let duplicate_check_ident = quote::format_ident!(
        "__VERB_DUPLICATE_CHECK_{}_{}_{}",
        sanitize_ident(noun_name),
        sanitize_ident(verb_name),
        fn_name
    );

    // This const will conflict if duplicate noun+verb registered
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #duplicate_check_ident: () = ();
    }
}
```

**How It Works**:
- Generates a const with name derived from (noun, verb, function)
- Two functions with same noun+verb → const name collision → compilation error
- Error message: "duplicate definitions with name..."

#### Gap 3: Return Type Validation

From `validation.rs` lines 22-126:

**Problem**: Return type doesn't implement Serialize

**Solution**: Recursive type validation at compile time

```rust
pub fn validate_return_type(return_type: &ReturnType, fn_name: &syn::Ident) 
    -> syn::Result<()> 
{
    match return_type {
        ReturnType::Default => {
            Err(syn::Error::new(
                fn_name.span(),
                "Function must return a value that implements serde::Serialize",
            ))
        }
        ReturnType::Type(_, ty) => {
            validate_type_is_serializable(ty, fn_name)?;
        }
    }
    Ok(())
}

// Recursive validation handles Result<T>, Option<T>, nested types
fn validate_type_is_serializable(ty: &Type, fn_name: &syn::Ident) 
    -> syn::Result<()> 
{
    match ty {
        Type::Path(type_path) => {
            let type_name = type_path.path.segments.last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            match type_name.as_str() {
                "Result" => {
                    // Extract T from Result<T, E> and validate recursively
                    if let Some(inner) = extract_inner_type(type_path) {
                        return validate_type_is_serializable(&inner, fn_name);
                    }
                }
                "Option" => {
                    // Extract T from Option<T> and validate recursively
                    if let Some(inner) = extract_inner_type(type_path) {
                        return validate_type_is_serializable(&inner, fn_name);
                    }
                }
                _ => Ok(())  // Let compiler check trait impl
            }
        }
        _ => Ok(())
    }
}
```

#### Gap 4: Attribute Syntax Validation

From `validation.rs` lines 128-236:

**Problem**: Typos in attribute arguments cause confusion

**Solution**: Validate argument count, types, and suggest corrections

```rust
pub fn validate_verb_attribute_syntax(
    args: &TokenStream, 
    input_fn: &ItemFn
) -> syn::Result<()> {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args_vec = parser.parse2(args.clone())?;

    // Check argument count
    if args_vec.len() > 2 {
        return Err(syn::Error::new(
            args.span(),
            format!("Too many arguments: expected 0-2, found {}", args_vec.len()),
        ));
    }

    // Check all arguments are string literals
    for (idx, arg) in args_vec.iter().enumerate() {
        match arg {
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(_), .. }) => {},
            syn::Expr::Path(path) => {
                // Common mistake: identifier instead of string
                let ident = path.path.get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "...".to_string());
                return Err(syn::Error::new(
                    arg.span(),
                    format!("Argument {} must be a string literal \"{}\" not {}",
                        idx + 1, ident, ident),
                ));
            }
            _ => return Err(syn::Error::new(
                arg.span(),
                "Argument must be a string literal",
            ))
        }
    }

    Ok(())
}
```

### FM-1.1 & FM-1.2: Architecture Guards

From `validation.rs` lines 545-595:

These Poka-Yoke guards prevent architectural violations:

**FM-1.1: CLI Layer Contamination**
```rust
pub fn validate_verb_complexity(input_fn: &ItemFn) -> syn::Result<()> {
    let complexity = calculate_cyclomatic_complexity(input_fn);

    // Threshold: 5 prevents business logic in CLI layer
    if complexity > 5 {
        return Err(syn::Error::new(
            input_fn.sig.ident.span(),
            "Verb function too complex (max cyclomatic complexity: 5)\n\
             Problem: Verb functions should delegate to domain logic\n\
             Solution: Extract business logic into separate function",
        ));
    }

    Ok(())
}
```

**FM-1.2: CLI Type Contamination**
```rust
pub fn validate_no_cli_types_in_params(sig: &syn::Signature) -> syn::Result<()> {
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            // Forbidden: ArgMatches, Command, HandlerInput, VerbArgs
            if let Some(error) = check_for_cli_types(&pat_type.ty) {
                return Err(error);
            }
        }
    }
    Ok(())
}
```

---

## Common Macro Debugging Techniques

### Technique 1: Expanded Code Inspection

Use `cargo expand` to see generated code:

```bash
# Install cargo-expand if needed
cargo install cargo-expand

# View expanded macros for the main crate
cd /home/user/clap-noun-verb
cargo expand --test integration_test | head -100

# View expanded macros for specific module
cargo expand clap_noun_verb::examples | grep -A 20 "verb_registry"
```

### Technique 2: Compile-Time Diagnostics

Enable detailed error messages:

```bash
# Build with verbose error output
RUST_LOG=debug cargo build 2>&1 | head -50

# Check for compiler diagnostics
cargo check 2>&1 | grep -A 5 "error\["

# View macro expansion errors
cargo build --message-format=short 2>&1 | grep "macro"
```

### Technique 3: Synthetic Span Debugging

In `lib.rs`, spans tell you exactly where errors occur:

```rust
// Error at function name location
let err = syn::Error::new(
    input_fn.sig.ident.span(),  // Points to function name
    "Error message",
);

// Error at return type location
let err = syn::Error::new(
    return_type_span,  // Points to exact return type
    "Invalid return type",
);

// Error at attribute location
let err = syn::Error::new(
    attr.span(),  // Points to the attribute itself
    "Invalid attribute syntax",
);
```

### Technique 4: Token Stream Inspection

Debug what's being parsed:

```rust
// Print tokens for debugging (disable in production)
let tokens_str = args_tokens.to_string();
eprintln!("Tokens: {}", tokens_str);

// Inspect specific expression types
if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = arg {
    eprintln!("String literal value: {}", s.value());
}

// Check type path segments
if let syn::Type::Path(type_path) = ty {
    let segments: Vec<_> = type_path.path.segments.iter()
        .map(|s| s.ident.to_string())
        .collect();
    eprintln!("Type path: {:?}", segments);
}
```

### Technique 5: Test-Driven Debugging

From `validation.rs` lines 679-907 (tests):

```rust
#[test]
fn test_validate_verb_syntax_invalid_identifier() {
    let tokens = quote! { status };  // Missing quotes
    let fn_item: ItemFn = parse_quote! {
        fn test_fn() -> Result<()> { Ok(()) }
    };
    
    let result = validate_verb_attribute_syntax(&tokens, &fn_item);
    assert!(result.is_err());
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be a string literal"));
    assert!(err_msg.contains("Add double quotes"));
}
```

---

## Validation Pattern Library

### Reusable Validation Functions

#### Pattern: Safe Vector Extraction

```rust
// For each parameter in function signature
for input in &input_fn.sig.inputs {
    if let syn::FnArg::Typed(pat_type) = input {
        // Get parameter name
        let arg_name = match &*pat_type.pat {
            syn::Pat::Ident(ident) => &ident.ident,
            _ => continue,  // Skip non-identifier patterns
        };

        // Check for raw identifier prefix (e.g., r#type)
        let arg_name_str = arg_name.to_string();
        let arg_name_str = arg_name_str
            .strip_prefix("r#")
            .map(str::to_string)
            .unwrap_or(arg_name_str);

        // Type-specific handling
        let is_option = is_option_type(&pat_type.ty);
        let is_flag = is_bool_type(&pat_type.ty);
        let is_vec = is_vec_type(&pat_type.ty);

        // ... per-type extraction logic
    }
}
```

#### Pattern: Levenshtein Distance for Typo Suggestions

From `validation.rs` lines 345-370:

```rust
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut dp = vec![vec![0; b_len + 1]; a_len + 1];
    
    for i in 0..=a_len {
        dp[i][0] = i;
    }
    for j in 0..=b_len {
        dp[0][j] = j;
    }
    
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            dp[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(dp[i][j + 1] + 1, dp[i + 1][j] + 1),
                dp[i][j] + cost,
            );
        }
    }
    
    dp[a_len][b_len]
}

// Usage: Find best matching allowed key for typo'd input
let mut best_suggestion = None;
let mut min_distance = usize::MAX;

for &allowed_key in ALLOWED_KEYS {
    let dist = levenshtein_distance(&typo, allowed_key);
    if dist < min_distance {
        min_distance = dist;
        best_suggestion = Some(allowed_key);
    }
}

if let Some(suggestion) = best_suggestion {
    if min_distance <= 3 {
        // Likely typo - suggest correction
    }
}
```

---

## Testing Strategies

### Strategy 1: Unit Testing Validation Functions

Location: `validation.rs` lines 679-907

```rust
#[test]
fn test_validate_return_type_result() {
    let fn_item: ItemFn = parse_quote! {
        fn test_fn() -> Result<String> {
            Ok("test".to_string())
        }
    };
    assert!(validate_return_type(&fn_item.sig.output, &fn_item.sig.ident).is_ok());
}
```

**AAA Pattern** (Arrange-Act-Assert):
1. **Arrange**: Create test input using `parse_quote!`
2. **Act**: Call validation function
3. **Assert**: Check result matches expectations

### Strategy 2: Integration Testing Macro Expansion

Location: `/home/user/clap-noun-verb/tests/macros/federated_network_test.rs`

```rust
use clap_noun_verb_macros::macros::federated_network::parse_capability_config;
use quote::quote;

#[test]
fn test_advertise_capability_macro_compiles() {
    // Arrange: Create macro arguments
    let args = quote! {
        capability_id = "test-capability",
        description = "Test capability description",
        inputs = ["input1:string", "input2:int"],
        outputs = ["output1:json"]
    };

    // Act: Parse configuration
    let result = parse_capability_config(args);

    // Assert: Verify structure
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.capability_id, "test-capability");
    assert_eq!(config.inputs.len(), 2);
    assert_eq!(config.outputs.len(), 1);
}
```

### Strategy 3: Error Message Validation

Ensure error messages are helpful:

```rust
#[test]
fn test_validate_arg_attribute_syntax_typo_suggests_correction() {
    let attrs: Vec<syn::Attribute> = parse_quote! {
        #[arg(shrt = 'v')]  // Typo: should be "short"
    };
    
    let result = validate_arg_attribute_syntax(&attrs);
    assert!(result.is_err());
    
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown argument parameter `shrt`"));
    assert!(err_msg.contains("Did you mean `short`?"));
    assert!(err_msg.contains("Valid parameters are:"));
}
```

### Strategy 4: Performance Testing

```rust
#[test]
fn bench_macro_parsing_performance() {
    use std::time::Instant;

    // Arrange
    let args = quote! {
        capability_id = "bench-capability",
        description = "Benchmark test",
        inputs = ["input1:string", "input2:int", "input3:bool"],
        outputs = ["output1:json", "output2:xml"]
    };

    // Act
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = parse_capability_config(args.clone());
    }
    let elapsed = start.elapsed();
    let avg_per_parse = elapsed.as_micros() / iterations;

    // Assert: Must complete in < 10ms per iteration
    assert!(
        avg_per_parse < 10_000,
        "Average parsing time should be < 10ms, got {}μs",
        avg_per_parse
    );
}
```

---

## Performance Optimization

### SLO: Incremental Compilation <= 2s

From CLAUDE.md: Target is 0.66s for incremental builds

### Optimization 1: Minimize Token Stream Cloning

**Problem**: TokenStream cloning is expensive

**Pattern**: Clone only when necessary

```rust
// BAD: Clone multiple times
let tokens1 = args.clone();
let tokens2 = args.clone();
let tokens3 = args.clone();

// GOOD: Use borrowed references
fn parse_args(args: &TokenStream) -> Result<...> {
    // Work with borrowed reference
}
```

### Optimization 2: Early Exit in Validation

**Pattern**: Return errors as soon as found

```rust
// BAD: Collect all errors
let mut all_errors = Vec::new();
for input in &inputs {
    if let Err(e) = validate(input) {
        all_errors.push(e);
    }
}

// GOOD: Return on first critical error
for input in &inputs {
    validate_critical(input)?;  // Return immediately on error
}
```

### Optimization 3: Lazy Type Inspection

**Pattern**: Inspect types only when needed

```rust
// BAD: Always check
for param in params {
    let is_cli_type = check_for_cli_types(&param.ty);
    let is_option = is_option_type(&param.ty);
}

// GOOD: Check only when needed
for param in params {
    if is_custom_type(&param.ty) {
        check_for_cli_types(&param.ty)?;
    }
}
```

---

## Real-World Examples

### Example 1: Adding a New Validation Check

**Scenario**: Prevent functions from accepting `String` (should use `&str`)

```rust
/// Prefer &str over String for CLI arguments
pub fn validate_string_types_in_params(sig: &syn::Signature) -> syn::Result<()> {
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if is_owned_string_type(&pat_type.ty) {
                return Err(syn::Error::new(
                    pat_type.ty.span(),
                    "Prefer &str over String for CLI arguments",
                ));
            }
        }
    }
    Ok(())
}

fn is_owned_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}
```

### Example 2: Extending Argument Metadata

**Scenario**: Add custom validation to argument attributes

```rust
// Add to parse_arg_attributes (lib.rs line 1981)
"value_parser" => {
    let ts = quote::quote! { #nv.value };
    let ts_string = ts.to_string();
    config.value_parser = Some(proc_macro2::TokenStream::from_iter(
        std::iter::once(proc_macro2::TokenTree::Literal(
            proc_macro2::Literal::string(&ts_string),
        )),
    ));
}
```

---

## Troubleshooting

### Problem: "Cannot find proc_macro in registry"

**Cause**: Macro crate not compiled yet

**Solution**:
```bash
cd clap-noun-verb-macros && cargo build
cd .. && cargo build
```

### Problem: "Duplicate definitions with name `__VERB_DUPLICATE_CHECK_...`"

**Cause**: Two functions registered with same noun+verb

**Solution**: Rename one function or use different verb name

```rust
// BAD
#[verb("status")]
fn show_status() -> Result<Status> { ... }

#[verb("status")]
fn display_status() -> Result<Status> { ... }

// GOOD
#[verb("status")]
fn show_status() -> Result<Status> { ... }

#[verb("info")]
fn display_status() -> Result<Status> { ... }
```

### Problem: "Function must return a value that implements serde::Serialize"

**Cause**: Return type doesn't implement Serialize

**Solution**: Derive Serialize on return type

```rust
use serde::Serialize;

#[derive(Serialize)]
struct TestOutput {
    value: String,
}

#[verb("test")]
fn test_verb() -> Result<TestOutput> { ... }
```

### Problem: "Unknown argument parameter `shrt`"

**Cause**: Typo in #[arg] attribute

**Solution**: Use correct parameter name

```rust
// BAD
#[arg(shrt = 'v')]

// GOOD
#[arg(short = 'v')]
```

---

## Summary Checklist

When extending the macro crate:

- [ ] **Understanding**: Read `lib.rs` lines 1-450 (architecture)
- [ ] **Validation**: Add validation check to `validation.rs` + tests
- [ ] **Error Messages**: Include "Expected vs Found" patterns
- [ ] **Type Checking**: Use proper `syn` patterns for type inspection
- [ ] **Code Generation**: Use fully qualified paths in `quote!`
- [ ] **Testing**: Add unit test + integration test + error case test
- [ ] **Performance**: Benchmark if adding expensive operation
- [ ] **Documentation**: Add examples in macro doc comments
- [ ] **Compatibility**: Test with raw identifiers (`r#type`)
- [ ] **Error Handling**: No unwrap/expect in macro code

---

## Additional Resources

**Key Files**:
- `clap-noun-verb-macros/src/lib.rs` (Main macro)
- `clap-noun-verb-macros/src/validation.rs` (Validation)
- `tests/macros/federated_network_test.rs` (Integration tests)
- `CLAUDE.md` (Project guidelines)

**External Resources**:
- [syn crate documentation](https://docs.rs/syn/)
- [quote crate documentation](https://docs.rs/quote/)
- [proc_macro API](https://doc.rust-lang.org/proc_macro/)

**Commands**:
```bash
# See expanded macros
cargo expand --test integration_test | head -100

# Run all macro tests
cargo make test

# Lint macro code
cargo make clippy
```

