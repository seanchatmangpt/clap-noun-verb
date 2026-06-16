# Documentation Quick Reference

**One-page cheat sheet for common documentation tasks.**

> **New:** For detailed task-oriented workflows, see [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md).

---

## Commands at a Glance

```bash
# Generate and view documentation
cargo make doc && open target/doc/clap_noun_verb/index.html

# Test all doc examples
cargo test --doc

# Format code and docs
cargo make format

# Lint everything
cargo make lint

# Build all examples
cargo make build-examples

# Run a specific example
cargo run --example tutorial_basic -- --help
```

---

## Documentation Comment Templates

### Module Documentation

```rust
//! # Module Name
//!
//! Brief one-liner.
//!
//! ## Examples
//!
//! ```rust
//! # use clap_noun_verb::module;
//! // Your example here
//! ```
```

### Type Documentation

```rust
/// Brief one-liner.
///
/// Detailed explanation.
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::MyType;
/// let value = MyType::new();
/// ```
///
/// # See Also
///
/// - [`RelatedType`]
pub struct MyType {
    /// Field doc
    pub field: String,
}
```

### Function with Errors

```rust
/// Brief one-liner.
///
/// Detailed explanation.
///
/// # Errors
///
/// Returns `Err` when:
/// - Condition 1
/// - Condition 2
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::my_function;
/// # fn main() -> clap_noun_verb::Result<()> {
/// let result = my_function("input")?;
/// # Ok(())
/// # }
/// ```
pub fn my_function(input: &str) -> Result<Output> {
    todo!()
}
```

### Macro Documentation

```rust
/// Registers a new command.
///
/// # Syntax
///
/// ```ignore
/// #[verb("name")]
/// fn handler(arg: Type) -> Result<Output>
/// ```
///
/// # Examples
///
/// ```ignore
/// #[verb("greet")]
/// fn cmd_greet(name: String) -> Result<Greeting> {
///     Ok(Greeting { message: format!("Hello, {}", name) })
/// }
/// ```
pub use clap_noun_verb_macros::verb;
```

---

## Example Quality Checklist

Before committing an example:

- [ ] Runnable without modification
- [ ] Meaningful output
- [ ] Compiles: `cargo build --example <name>`
- [ ] Runs: `cargo run --example <name> --`
- [ ] Listed in `examples/README.md`
- [ ] Module doc comment with learning goals
- [ ] Shows common patterns, not edge cases

---

## Diataxis Quick Reference

| Type | Purpose | Length | Tone | Example |
|------|---------|--------|------|---------|
| **Tutorial** | Learn from scratch | 1500-3000 words | Friendly, step-by-step | [tutorial/basic.rs](../examples/tutorial/basic.rs) |
| **How-To** | Solve a problem | 800-2000 words | Practical, direct | [docs/howto/arg_groups.md](howto/arg_groups.md) |
| **Reference** | Look up API | 500-1500 words | Accurate, concise | [docs/reference/api/verb-macro.md](reference/api/verb-macro.md) |
| **Explanation** | Understand design | 1000-2000 words | Conceptual, context | [docs/explanation/architecture.md](explanation/architecture.md) |

---

## Updating When You Change Code

```
1. Update src/file.rs (your code change)
2. Update doc comments (same file)
3. Test: cargo test --doc
4. If new API/feature:
   - Create docs/howto/feature.md
   - Create examples/howto/feature.rs
   - Create docs/reference/api/feature.md
5. Update README.md if user-facing
6. Run cargo make doc
7. Commit with clear message
```

---

## Common Doc Test Patterns

### ✅ Good Example

```rust
/// ```rust
/// use clap_noun_verb::builder::CliBuilder;
///
/// # fn main() -> clap_noun_verb::Result<()> {
/// let builder = CliBuilder::new();
/// let command = builder.build()?;
/// assert_eq!(command.get_name(), "myapp");
/// # Ok(())
/// # }
/// ```
```

### ❌ Bad Examples

```rust
// Incomplete - doesn't compile
/// ```rust
/// let builder = CliBuilder::new();
/// builder.build()  // Missing error handling
/// ```

// Stub - teaches nothing
/// ```rust
/// assert!(builder.build().is_ok());  // Tautological
/// ```

// Has unwrap - violates guidelines
/// ```rust
/// let result = builder.build().unwrap();  // Bad!
/// ```
```

---

## Doc Comment Stubs to Hide

Use `#` prefix to hide setup code:

```rust
/// ```rust
/// # use clap_noun_verb::MyType;
/// # fn main() -> clap_noun_verb::Result<()> {
/// let value = MyType::new();
/// println!("{:?}", value);
/// # Ok(())
/// # }
/// ```
```

---

## Project Structure Map

```
docs/
  ├─ DOCUMENTATION_GUIDE.md      ← You are here (full guide)
  ├─ DOCUMENTATION_QUICK_REFERENCE.md ← This file
  ├─ tutorial/                    Tutorial docs (learning)
  ├─ howto/                       How-to guides (solving problems)
  ├─ reference/                   Reference docs (looking up)
  └─ explanation/                 Explanation docs (understanding)

examples/
  ├─ README.md                    Navigation guide
  ├─ tutorial/                    Tutorial examples
  ├─ howto/                       How-to examples
  └─ reference/                   Reference examples

src/
  └─ lib.rs                       Main module docs
```

---

## Documentation Checklist by Type

### ✅ Before Committing Doc Comments

- [ ] Doc test compiles (`cargo test --doc`)
- [ ] Example is realistic and useful
- [ ] No unwrap() or expect() in examples
- [ ] Links use `[`Type`]` syntax
- [ ] Related items linked in "See Also"
- [ ] Error cases documented in `# Errors`

### ✅ Before Committing How-To Guide

- [ ] Solves ONE specific problem
- [ ] Step-by-step walkthrough
- [ ] Includes troubleshooting section
- [ ] ~800-2000 words
- [ ] Real, tested code examples
- [ ] Links to reference docs

### ✅ Before Committing Example Code

- [ ] Module doc comment present
- [ ] Runs without modification
- [ ] Produces meaningful output
- [ ] Listed in examples/README.md
- [ ] ~50-200 lines (not too long)
- [ ] Shows patterns, not edge cases

### ✅ Before Committing Reference Page

- [ ] API signatures are accurate
- [ ] All parameters documented
- [ ] Examples are minimal
- [ ] Links to related items
- [ ] ~500-1500 words
- [ ] No explanatory prose

---

## Quick File Locations

| What | Where |
|------|-------|
| Main lib docs | `src/lib.rs` |
| Module docs | `src/module.rs` (top of file) |
| Type docs | `src/types.rs` (above type definition) |
| Macro docs | `clap-noun-verb-macros/src/lib.rs` |
| How-to guides | `docs/howto/*.md` |
| Reference pages | `docs/reference/api/*.md` |
| Tutorial docs | `docs/tutorial/*.md` |
| Examples | `examples/{tutorial,howto,reference}/*.rs` |
| ADRs | `docs/adr/*.md` |

---

## Doc Test Special Syntax

```rust
/// Compile, run, and test to pass:
/// ```rust
/// let x = 2 + 2;
/// assert_eq!(x, 4);
/// ```

/// Compile and run, but ignore output:
/// ```rust,ignore
/// # Pseudo-code example that doesn't actually compile
/// ```

/// Text block (doesn't compile or run):
/// ```text
/// This is documentation, not code
/// ```

/// Compile only, don't run:
/// ```rust,no_run
/// fn long_operation() { /* ... */ }
/// ```

/// Expected to panic:
/// ```rust,should_panic
/// panic!("This will panic");
/// ```
```

---

## Terminology Consistency

Use these terms consistently across documentation:

| Concept | Use | Not |
|---------|-----|-----|
| Handler function | "verb handler" or "command handler" | "verb function", "handler fn" |
| Attributes | "#[verb]", "#[arg]" | "@verb", "[verb]" |
| Arguments | "arguments" or "args" | "parameters", "options" (unless they use `--`) |
| Flags | "flags" (for boolean `--flag`) | "arguments" |
| Feature | `feature = "name"` in code | "feature:name", "feature-name" (lowercase) |
| Type | `TypeName` (CamelCase) | "the TypeName type" |
| Function | `function_name()` | "the function_name function" |
| Error | `MyError::Variant` or "MyError" | "the error" (unless context is clear) |

---

## ADR Quick Template

```markdown
# ADR NNNN: [Decision Title]

**Status:** Accepted | Proposed

**Date:** YYYY-MM-DD

## Context

[What problem/question prompted this?]

## Decision

[What did we decide?]

## Consequences

**Positive:**
- [Benefit 1]
- [Benefit 2]

**Negative:**
- [Trade-off 1]
- [Trade-off 2]

## Alternatives Considered

### Alternative A
[Approach and trade-offs]

### Alternative B
[Approach and trade-offs]
```

---

## Common Mistakes

❌ Don't:
- Use `unwrap()` in doc examples
- Write examples that don't compile
- Use `assert!(x.is_ok())` as only test
- Document private APIs
- Mix Diataxis categories
- Leave broken links
- Forget "See Also" sections

✅ Do:
- Test with `cargo test --doc`
- Write realistic examples
- Use `?` operator and `# fn main()`
- Update docs when APIs change
- Use consistent terminology
- Link between sections
- Review before committing

---

## Quick Validation

Before committing, run:

```bash
cargo test --doc           # All doc examples pass
cargo make format-check    # Code is formatted
cargo make lint            # No linting errors
cargo make build-examples  # All examples compile
cargo make doc             # Docs generate cleanly
```

---

## Resources

- **Full Guide**: [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md)
- **Diataxis Framework**: https://diataxis.fr/
- **Rust Doc Comments**: https://doc.rust-lang.org/rustdoc/
- **Example Directory**: [examples/README.md](../examples/README.md)
- **Contributing**: [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

**Last Updated:** 2026-06-14

**See Also:** [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md) for complete documentation
