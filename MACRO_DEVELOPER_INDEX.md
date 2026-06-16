# Macro Developer Quick Reference

**For developers working on clap-noun-verb-macros**

## Start Here

1. **[MACRO_DEVELOPMENT_GUIDE.md](./MACRO_DEVELOPMENT_GUIDE.md)** (990 lines)
   - Complete skill guide with patterns, examples, and best practices
   - Covers: architecture, debugging, validation, testing, optimization
   - Contains code examples from actual codebase

## Quick Navigation

### If you're...

**New to proc macros in this codebase:**
→ Start with [Architecture Overview](./MACRO_DEVELOPMENT_GUIDE.md#macro-architecture-overview)

**Debugging a macro issue:**
→ Jump to [Common Debugging Techniques](./MACRO_DEVELOPMENT_GUIDE.md#common-macro-debugging-techniques)

**Adding validation:**
→ Read [Poka-Yoke Section](./MACRO_DEVELOPMENT_GUIDE.md#compile-time-validation-poka-yoke) + [Validation Patterns](./MACRO_DEVELOPMENT_GUIDE.md#validation-pattern-library)

**Writing tests for macros:**
→ See [Testing Strategies](./MACRO_DEVELOPMENT_GUIDE.md#testing-strategies)

**Optimizing performance:**
→ Check [Performance Optimization](./MACRO_DEVELOPMENT_GUIDE.md#performance-optimization)

**Stuck on a problem:**
→ Try [Troubleshooting](./MACRO_DEVELOPMENT_GUIDE.md#troubleshooting)

## Key Source Files

**Main macro implementation:**
```
clap-noun-verb-macros/src/lib.rs (2800+ lines)
  - #[verb] macro (line 330)
  - #[arg] macro (line 71)
  - #[meta_aware] macro (line 109)
  - Helper functions (lines 451-2100)
```

**Validation logic:**
```
clap-noun-verb-macros/src/validation.rs (908 lines)
  - Gap 1-4 validation checks
  - FM-1.1 & FM-1.2 guards
  - Unit tests (lines 679-907)
```

**I/O type detection:**
```
clap-noun-verb-macros/src/io_detection.rs (215 lines)
  - DetectedIoType enum
  - Auto-wiring logic
```

**Tests:**
```
tests/macros/federated_network_test.rs
  - 14 integration tests following AAA pattern
  - Performance benchmarking
```

## Essential Patterns

| Pattern | Purpose | Line Range |
|---------|---------|-----------|
| Token parsing | Parse attribute args | lib.rs:367-384 |
| Error handling | Meaningful errors | validation.rs:28-50 |
| Type inspection | Check types | lib.rs:1730-1754 |
| Code generation | Generate with quote! | lib.rs:1622-1725 |
| Duplicate detection | Compile-time safety | validation.rs:268-290 |
| Return validation | Serialize check | validation.rs:22-126 |
| Attribute syntax | Typo suggestions | validation.rs:128-236 |
| Complexity check | FM-1.1 guard | validation.rs:554-595 |
| CLI type check | FM-1.2 guard | validation.rs:468-543 |

## Command Reference

### Build & Test
```bash
# Build macro crate
cargo make build

# Test all macros
cargo make test

# Lint with clippy
cargo make clippy

# See expanded macros
cargo expand --test integration_test | head -100
```

### Debugging
```bash
# View compilation diagnostics
cargo check 2>&1 | grep -A 5 "error"

# Build with verbose output
RUST_LOG=debug cargo build 2>&1

# Profile macro expansion time
cargo build --message-format=short 2>&1 | grep macro
```

## Core Concepts

**Poka-Yoke**: Mistake-proofing via compile-time checks
- Gap 1: Forgotten `#[verb]` detection
- Gap 2: Duplicate verb detection
- Gap 3: Return type validation
- Gap 4: Attribute syntax validation

**FM-1.1**: CLI Layer Contamination Guard
- Prevents business logic in `#[verb]` functions
- Enforces cyclomatic complexity < 5

**FM-1.2**: CLI Type Contamination Guard
- Forbids `ArgMatches`, `Command`, `HandlerInput` parameters
- Domain functions stay CLI-independent

**Linkme Integration**: Distributed slice registration
- Verbs auto-discovered at link time
- Zero-runtime registration overhead

## Error Messages Quality

Good error messages should include:
1. **What went wrong** (clear title)
2. **Why it's a problem** (context)
3. **How to fix it** (solution)
4. **Example** (before/after)

Example from validation.rs:
```rust
"🛡️ Poka-Yoke Guard: CLI type contamination detected (FM-1.2)\n\
 \n\
 Problem: Domain functions should not depend on CLI types.\n\
 Solution: Use simple typed parameters instead:\n\
 ✅ GOOD:   fn calculate(x: i32, y: i32) -> Result<i32>\n\
 ❌ WRONG:  fn calculate(args: VerbArgs) -> Result<i32>"
```

## Common Tasks

### Adding a Validation Check
1. Write unit test in validation.rs (test section)
2. Implement validation function
3. Add to macro pipeline (lib.rs line 346-348)
4. Test with real examples

### Adding a Macro Feature
1. Create new submodule (e.g., `macros/my_feature.rs`)
2. Implement `#[proc_macro_attribute]` or `#[proc_macro]`
3. Add integration tests following AAA pattern
4. Document with inline examples

### Debugging a Macro Issue
1. Run `cargo expand` to see generated code
2. Check error spans with `syn::Error::new(span, msg)`
3. Inspect token streams with `eprintln!`
4. Write a failing test to reproduce

## Testing Guidelines

**Use AAA Pattern**:
- Arrange: Set up test input
- Act: Call macro/function
- Assert: Verify output

**Test These**:
- Happy path (valid input)
- Error paths (invalid input)
- Edge cases (empty, very large, special chars)
- Error messages (contain helpful hints)
- Performance (benchmarks for expensive ops)

**Example**:
```rust
#[test]
fn test_validates_return_type() {
    // Arrange
    let fn_item: ItemFn = parse_quote! { fn test() -> Result<String> { Ok("".into()) } };
    
    // Act
    let result = validate_return_type(&fn_item.sig.output, &fn_item.sig.ident);
    
    // Assert
    assert!(result.is_ok());
}
```

## Performance Targets

From CLAUDE.md SLOs:
- Incremental compilation: ≤ 2 seconds
- Current actual: 0.66 seconds
- Macro parsing: < 10ms per invocation

## Troubleshooting Flowchart

```
Macro not compiling?
├─ Check: Is macro crate built? → cargo build -p clap-noun-verb-macros
├─ Check: Are attributes spelled correctly? → See validation.rs:ALLOWED_ARG_KEYS
└─ Check: Does return type implement Serialize? → Add #[derive(Serialize)]

Duplicate verb error?
├─ Check: Same function registered twice? → Rename one
└─ Check: Two functions with same noun+verb? → Change verb name

Error message unclear?
├─ Run: cargo expand to see generated code
├─ Run: cargo check with RUST_LOG=debug
└─ File: Issue with error message → Update in lib.rs or validation.rs

Build too slow?
├─ Profile: cargo build --message-format=short
├─ Reduce: Number of macro invocations
└─ Check: For expensive validation in hot path
```

---

## File Map

```
clap-noun-verb-macros/
├── Cargo.toml
├── src/
│   ├── lib.rs (2800 lines) ← MAIN MACRO IMPLEMENTATION
│   ├── validation.rs (908 lines) ← POKA-YOKE CHECKS
│   ├── io_detection.rs (215 lines)
│   ├── meta_framework.rs
│   ├── rdf_generation.rs
│   ├── telemetry_validation.rs
│   └── macros/
│       ├── mod.rs
│       ├── federated_network.rs
│       ├── semantic_composition.rs
│       ├── executable_specs.rs
│       ├── fractal_patterns.rs
│       ├── learning_trajectories.rs
│       ├── economic_simulation.rs
│       ├── reflexive_testing.rs
│       └── reflexive_testing_macro.rs
└── tests/
    └── macros/
        └── federated_network_test.rs (318 lines) ← INTEGRATION TESTS
```

---

**Need help?** See MACRO_DEVELOPMENT_GUIDE.md sections:
- [Core Proc-Macro Patterns](./MACRO_DEVELOPMENT_GUIDE.md#core-proc-macro-patterns)
- [Compile-Time Validation](./MACRO_DEVELOPMENT_GUIDE.md#compile-time-validation-poka-yoke)
- [Testing Strategies](./MACRO_DEVELOPMENT_GUIDE.md#testing-strategies)
- [Troubleshooting](./MACRO_DEVELOPMENT_GUIDE.md#troubleshooting)

