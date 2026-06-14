# Code Review Standards and Checklist for clap-noun-verb

A comprehensive guide for conducting thorough, constructive code reviews of the clap-noun-verb Rust CLI framework. This document establishes expectations for both reviewers and submitters to maintain high code quality, safety, and maintainability.

---

## Table of Contents

1. [Review Scope](#review-scope)
2. [Correctness & Safety](#correctness--safety)
3. [Performance](#performance)
4. [Security](#security)
5. [Maintainability](#maintainability)
6. [Architecture](#architecture)
7. [Testing](#testing)
8. [Style & Idioms](#style--idioms)
9. [Compatibility](#compatibility)
10. [Tone & Culture](#tone--culture)
11. [Review Checklist](#review-checklist)
12. [Common Anti-Patterns](#common-anti-patterns)
13. [Example Comments](#example-comments)

---

## Review Scope

### In Scope

Reviewers **SHOULD** check:

- **Correctness**: Logic errors, edge cases, off-by-one errors, logic inversions
- **Safety**: Unsafe blocks, panics, unwrap/expect usage, data races
- **Error handling**: Proper Result/Option usage, error propagation, meaningful error messages
- **Performance**: Unnecessary allocations, algorithmic complexity, compilation time impact
- **Testing**: Coverage, test quality, determinism, flakiness prevention
- **Documentation**: API clarity, examples, rustdoc compliance, comment accuracy
- **Style**: Formatting, idioms, naming, Clippy warnings, adherence to project standards
- **Architecture**: Alignment with ADL (Architecture Decision Log), no unexpected patterns
- **Compatibility**: Breaking changes, deprecation strategy, version bumping

### Out of Scope

Reviewers **SHOULD NOT** enforce or debate:

- **Personal style preferences** unrelated to project standards (e.g., variable naming conventions that don't violate patterns)
- **"Nice to have" optimizations** that don't significantly impact performance (unless they block SLOs)
- **Bike-shedding** on colors, naming debates without consensus guidance
- **Complete rewrites** of working code without justification
- **Features or design decisions** already approved in the PR description or ADL
- **Third-party dependencies** beyond license and security concerns (those are handled by `cargo deny`)

### Pre-Review Checklist for Reviewers

Before starting a review, check:

- [ ] **Self-review first?** The submitter should have done this; if not, ask gently
- [ ] **Tests passing locally?** Run `cargo make ci` or equivalent
- [ ] **All conversations resolved?** Check for unresolved threads
- [ ] **Scope reasonable?** Is this PR focused (one feature/fix) or does it drift?
- [ ] **Any previous reviews?** Read earlier feedback to see if it's addressed

---

## Correctness & Safety

### Logic Errors

Verify that the logic implements the intended behavior correctly.

#### Example: Off-by-One Error

```rust
// BAD: Loop range off by one
for i in 0..verbs.len() - 1 {
    verbs[i].register();
}
// Last verb is not registered!

// GOOD: Correct loop range
for verb in &verbs {
    verb.register();
}
```

**Review comment:**
```
The loop omits the last verb in the slice. Did you intend to skip the final verb,
or should this be `0..verbs.len()`? Using an iterator (`for verb in &verbs`) is
more idiomatic and avoids this class of error.
```

---

#### Example: Logic Inversion

```rust
// BAD: Condition inverted
if !is_valid_verb_name(name) {
    registry.register(name);  // Registers invalid names!
}

// GOOD: Correct condition
if is_valid_verb_name(name) {
    registry.register(name);
}
```

**Review comment:**
```
The condition appears inverted. You're registering the verb only when the name is
NOT valid. Did you mean to use `if is_valid_verb_name(name)`?
```

---

### Edge Cases

Check for boundary conditions, empty inputs, None/None/zero values.

#### Example: Unhandled Empty Input

```rust
// BAD: Panics on empty input
pub fn first_verb(verbs: &[VerbCommand]) -> &VerbCommand {
    &verbs[0]  // Panics if empty
}

// GOOD: Returns Option
pub fn first_verb(verbs: &[VerbCommand]) -> Option<&VerbCommand> {
    verbs.first()
}
```

**Review comment:**
```
What should happen if `verbs` is empty? This will panic. Consider returning
`Option<&VerbCommand>` to handle the edge case gracefully. Callers can then
decide whether to use `.unwrap_or_default()`, return an error, or take another path.
```

---

#### Example: None Not Handled

```rust
// BAD: Ignores None case
fn maybe_route(input: Option<String>) {
    let value = input.unwrap();  // Denies!
    router.route(&value);
}

// GOOD: Handles None explicitly
fn maybe_route(input: Option<String>) -> Result<()> {
    let value = input.ok_or(NounVerbError::MissingInput)?;
    router.route(&value)
}
```

**Review comment:**
```
This uses `unwrap()` which will panic if `input` is `None`. The error lints deny
`unwrap_used` in production code. Replace with `?` operator and return a `Result`,
or use `map_err()` to convert the error gracefully.
```

---

### Error Handling

Ensure errors are properly propagated, meaningful, and actionable.

#### Example: Lost Error Context

```rust
// BAD: Error context lost
pub fn load_config() -> Result<Config> {
    let json = std::fs::read_to_string("config.json")
        .map_err(|_| NounVerbError::Generic("failed".to_string()))?;
    parse_config(&json)
}
// User sees "failed" but not the underlying fs::Error

// GOOD: Preserve error context
pub fn load_config() -> Result<Config> {
    let json = std::fs::read_to_string("config.json")
        .map_err(|e| NounVerbError::Generic(
            format!("Failed to read config.json: {}", e)
        ))?;
    parse_config(&json)
}
```

**Review comment:**
```
The error message "failed" doesn't tell the user what went wrong (permission denied?
file not found?). Include the underlying error details: 
`format!("Failed to read config.json: {}", e)`. This makes debugging much easier.
```

---

#### Example: Swallowing Errors Silently

```rust
// BAD: Error is silently dropped
pub fn register_all_verbs(registry: &mut CommandRegistry) {
    for verb in get_verbs() {
        let _ = registry.register(verb);  // Ignores registration failures
    }
}

// GOOD: Propagate or log the error
pub fn register_all_verbs(registry: &mut CommandRegistry) -> Result<()> {
    for verb in get_verbs() {
        registry.register(verb)?;  // Propagates failure
    }
    Ok(())
}

// OR, if partial failure is acceptable:
pub fn register_all_verbs(registry: &mut CommandRegistry) {
    for verb in get_verbs() {
        match registry.register(verb) {
            Ok(_) => {},
            Err(e) => log::warn!("Failed to register verb: {}", e),
        }
    }
}
```

**Review comment:**
```
Registration errors are silently dropped with `let _ = ...`. If registration fails,
should the entire operation fail, or is partial failure acceptable? If it's an error
condition, propagate with `?`. If it's acceptable but should be noted, add a
warning log: `log::warn!("Failed to register verb: {}", e)`.
```

---

### Resource Leaks & Lifetimes

Check for missing cleanup, dangling references, or lifetime violations.

#### Example: Missing Cleanup

```rust
// BAD: File handle not closed
pub fn read_config(path: &str) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    // File is dropped here, but implicit; not obvious
    Ok(buf)
}

// GOOD: Explicit cleanup or RAII scope
pub fn read_config(path: &str) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut buf = String::new();
    std::io::Read::read_to_string(&mut io::BufReader::new(file), &mut buf)?;
    Ok(buf)
}

// BEST: Use helper function
pub fn read_config(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| NounVerbError::Generic(format!("Failed to read {}: {}", path, e)))
}
```

**Review comment:**
```
The file handle is dropped at the end of the function, which works, but it's not
explicit. Rust handles cleanup via RAII, so this is safe, but for clarity consider
using `std::fs::read_to_string()` directly, which handles this pattern. If you're
using a file for multiple operations, make the scope explicit with braces:
`{ let mut file = File::open(...)?; ... }`.
```

---

## Performance

### Algorithmic Complexity

Check for unnecessary nested loops, redundant traversals, or high-order operations.

#### Example: Quadratic Complexity

```rust
// BAD: O(n²) complexity
pub fn find_duplicate_verbs(verbs: &[VerbCommand]) -> Vec<String> {
    let mut duplicates = Vec::new();
    for i in 0..verbs.len() {
        for j in (i + 1)..verbs.len() {
            if verbs[i].name() == verbs[j].name() {
                duplicates.push(verbs[i].name().to_string());
            }
        }
    }
    duplicates
}

// GOOD: O(n) complexity with HashMap
pub fn find_duplicate_verbs(verbs: &[VerbCommand]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    verbs.iter()
        .filter_map(|verb| {
            if !seen.insert(verb.name()) {
                Some(verb.name().to_string())
            } else {
                None
            }
        })
        .collect()
}
```

**Review comment:**
```
This has O(n²) complexity due to nested loops. For a registry with many verbs,
this will be slow. Consider using a `HashSet` to track seen verbs: insert each verb
name, and return those that fail insertion (already seen). This reduces to O(n).
```

---

### Memory Usage

Check for unnecessary allocations, large copies, or unbounded collections.

#### Example: Unnecessary String Allocation

```rust
// BAD: Allocates a new String unnecessarily
pub fn verb_description(name: &str) -> String {
    format!("Verb: {}", name)
}

// GOOD: Return &str or avoid allocation
pub fn verb_description(name: &str) -> Cow<'static, str> {
    Cow::Borrowed(name)
}

// OR, if formatting is required:
pub fn verb_description<'a>(name: &'a str) -> String {
    format!("Verb: {}", name)
}
```

**Review comment:**
```
This allocates a new `String` every call. If the result is short-lived or
frequently called, consider returning `&str` or using `Cow<str>` to avoid
allocation when possible. If you must format, that's fine, but document why.
```

---

#### Example: Large Stack Allocation

```rust
// BAD: Allocates 1MB on the stack
pub fn process_chunk() -> Result<()> {
    let buffer: [u8; 1_000_000] = [0; 1_000_000];
    // ...
}

// GOOD: Allocate on the heap via Vec
pub fn process_chunk() -> Result<()> {
    let buffer = vec![0u8; 1_000_000];
    // ...
}
```

**Review comment:**
```
This allocates 1MB on the stack, which may overflow the stack (typical ~8MB limit).
Move this to the heap using `vec!`: `let buffer = vec![0u8; 1_000_000];`.
```

---

### Compilation Time Impact

Check for heavy proc-macro expansion, excessive generics, or monomorphization.

#### Example: Excessive Macro Expansion

```rust
// BAD: Expanded macro is expensive to compile
#[verb]
#[verb]
#[verb]
pub fn handle_something() { }
// Duplicate macros expand multiple times!

// GOOD: Single annotation
#[verb]
pub fn handle_something() { }
```

**Review comment:**
```
This has the `#[verb]` attribute twice, causing the macro to expand twice and
slowing compilation. Keep a single attribute. Run `cargo make build` to verify
incremental build time stays under 2s (current SLO).
```

---

## Security

### Unsafe Blocks

Verify that unsafe code is necessary, well-justified, and properly guarded.

#### Example: Unnecessary Unsafe

```rust
// BAD: Unsafe without justification
unsafe {
    let ptr = some_value as *const u8;
    let slice = std::slice::from_raw_parts(ptr, 10);
}

// GOOD: Safe alternative
let slice = &some_value[..10];
```

**Review comment:**
```
This unsafe block can be removed. You're converting to a pointer and back; the
compiler can already verify slice bounds safely. Use `&some_value[..10]` directly.
Unsafe should be avoided unless necessary. If it is necessary, document why with
a comment explaining the safety invariants.
```

---

#### Example: Unsafe Without Safety Documentation

```rust
// BAD: Unsafe without explaining why it's safe
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}

// GOOD: Unsafe with safety documentation
// SAFETY: We've verified that:
// - src points to valid, initialized memory of at least `len` bytes
// - dst points to valid, uninitialized memory of at least `len` bytes
// - src and dst do not overlap
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
```

**Review comment:**
```
This unsafe block doesn't document why it's safe. Per the Rust reference, each
unsafe block must have a comment explaining the safety invariants. Add a `// SAFETY:`
comment detailing why the preconditions are met.
```

---

### Panics & Unwrap

Ensure no panics or unwrap-like operations (these are denied in production code).

#### Example: Panic in Production Code

```rust
// BAD: Denied by Clippy
pub fn route_command(input: String) {
    let parts: Vec<&str> = input.split(' ').collect();
    let noun = parts[0];  // Panics if empty!
    let verb = parts[1];  // Panics if fewer than 2 parts!
}

// GOOD: Handle the error
pub fn route_command(input: &str) -> Result<()> {
    let mut parts = input.split(' ');
    let noun = parts.next().ok_or(NounVerbError::MissingNoun)?;
    let verb = parts.next().ok_or(NounVerbError::MissingVerb)?;
    Ok(())
}
```

**Review comment:**
```
This will panic if `input` is empty or has fewer than 2 parts. The lints deny
`expect_used` and array indexing panics. Use `parts.next()` with error handling:
`let noun = parts.next().ok_or(NounVerbError::MissingNoun)?;`.
```

---

### Input Validation

Check that user input is validated before use.

#### Example: Unvalidated String Input

```rust
// BAD: No validation
pub fn add_verb(name: String) -> Result<()> {
    if name.is_empty() {
        return Err(NounVerbError::InvalidName);
    }
    // But other invalid characters are not checked!
}

// GOOD: Comprehensive validation
pub fn add_verb(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(NounVerbError::InvalidName("Name cannot be empty".into()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(NounVerbError::InvalidName(
            "Name must contain only alphanumerics and underscores".into()
        ));
    }
    Ok(())
}
```

**Review comment:**
```
The validation only checks for empty names. What about invalid characters? If the
verb name is used in command parsing or file paths, ensure it's validated against
the full set of allowed characters. Consider a regex or explicit character check.
```

---

### Dependency Security

Check for known vulnerabilities and license compliance.

#### Example: Outdated Dependency

```toml
# BAD: Known vulnerability in an old version
[dependencies]
serde_json = "1.0.0"

# GOOD: Updated to patched version
[dependencies]
serde_json = "1.0.94"
```

**Review comment:**
```
This dependency has a known vulnerability (CVE-XXXX). Update to the latest patch
version. Run `cargo deny check` to verify no other vulnerabilities are introduced.
```

---

## Maintainability

### Naming

Check for clear, descriptive names that reflect intent.

#### Example: Vague Names

```rust
// BAD: Names don't reflect purpose
pub fn do_thing(x: String, y: Vec<u8>) -> Result<()> {
    let z = format!("{:?}", y);
    Ok(())
}

// GOOD: Clear, descriptive names
pub fn serialize_registry(verb_name: String, data: Vec<u8>) -> Result<()> {
    let serialized = format!("{:?}", data);
    store_serialized(&serialized)
}
```

**Review comment:**
```
Variable names `do_thing`, `x`, `y`, `z` don't convey intent. Rename to be more
descriptive: `serialize_registry`, `verb_name`, `data`, `serialized`. The code
should be readable without needing comments.
```

---

### Code Structure

Check for high cyclomatic complexity, deeply nested conditions, or functions that do too many things.

#### Example: High Cyclomatic Complexity

```rust
// BAD: Many branches, hard to follow
pub fn handle_verb(noun: &str, verb: &str) -> Result<()> {
    if noun == "services" {
        if verb == "start" {
            // ...
        } else if verb == "stop" {
            // ...
        } else if verb == "status" {
            // ...
        }
    } else if noun == "config" {
        if verb == "get" {
            // ...
        } else if verb == "set" {
            // ...
        }
    }
    // ... more branches
}

// GOOD: Dispatch to separate handlers
pub fn handle_verb(noun: &str, verb: &str) -> Result<()> {
    match (noun, verb) {
        ("services", cmd) => handle_services_cmd(cmd),
        ("config", cmd) => handle_config_cmd(cmd),
        _ => Err(NounVerbError::UnknownCommand),
    }
}
```

**Review comment:**
```
This function has high cyclomatic complexity (many branches). The logic would be
clearer dispatched to separate handler functions. Consider:
- Using a match statement instead of nested if-else
- Extracting noun/verb handlers to separate functions
- Using a trait-based dispatch system

This improves testability and readability.
```

---

### Documentation Clarity

Check that public APIs are documented and examples are correct.

#### Example: Missing Documentation

```rust
// BAD: No docs
pub fn register_verb(name: &str, handler: Box<dyn VerbHandler>) {
    // ...
}

// GOOD: Comprehensive doc comment
/// Register a new verb handler with the command registry.
///
/// # Arguments
/// * `name` - The verb name (e.g., "status"). Must be lowercase alphanumeric + underscores.
/// * `handler` - The verb handler function that will be invoked.
///
/// # Returns
/// Returns an error if the verb name is invalid or already registered.
///
/// # Examples
/// ```
/// let registry = CommandRegistry::new();
/// registry.register_verb("status", Box::new(handle_status))?;
/// ```
pub fn register_verb(name: &str, handler: Box<dyn VerbHandler>) -> Result<()> {
    // ...
}
```

**Review comment:**
```
Public function `register_verb` has no documentation. Add a doc comment including:
- A one-liner summary of what it does
- Explanation of arguments and their constraints
- Return value and error conditions
- A working example users can copy-paste

Run `cargo make doc` to verify the documentation renders correctly.
```

---

#### Example: Incorrect Documentation Example

```rust
/// Process a verb and return the result.
///
/// # Examples
/// ```
/// let result = process_verb("services".to_string());
/// assert!(result.is_ok());
/// ```
pub fn process_verb(noun: &str) -> Result<Output> {
    // ...
}
```

**Review comment:**
```
The documentation example doesn't compile: `process_verb` takes `&str`, not
`String`. Change to `process_verb("services")?;` or use the correct type. The
rustdoc examples should compile and run successfully (they're checked by `cargo test --doc`).
```

---

## Architecture

### Alignment with ADL

Verify changes align with documented Architecture Decision Log (see CLAUDE.md).

#### Example: ADL-005 Violation (No Panics)

```rust
// BAD: Violates ADL-005 (No Panics)
pub fn get_first_verb(verbs: &[VerbCommand]) -> &VerbCommand {
    &verbs[0]  // Panics if empty; violates ADL-005
}
```

**Review comment:**
```
Per ADL-005 (No Panics in Production Code), this will panic if `verbs` is empty.
The lints deny panics. Return `Option` or `Result` instead:
```rust
pub fn get_first_verb(verbs: &[VerbCommand]) -> Option<&VerbCommand> {
    verbs.first()
}
```
This aligns with our no-panic guarantee.
```

---

#### Example: ADL-010 Violation (Trait Design)

```rust
// BAD: Violates ADL-010 (dyn compatible, sync-only traits)
pub trait VerbHandler: Send + Sync {
    async fn execute(&mut self, input: HandlerInput) -> Result<Output>;
}
// Async in trait = not object-safe!

// GOOD: Sync methods, dyn compatible
pub trait VerbHandler: Send + Sync {
    fn execute(&self, input: &HandlerInput) -> Result<Output>;
}
```

**Review comment:**
```
Per ADL-010, traits must be `dyn`-compatible and sync-only. Async methods in
traits make them non-object-safe. Remove the `async` keyword. If async is needed,
use the `async_verb.rs` module (feature-gated) instead.
```

---

### Unexpected Patterns

Check for code that violates project conventions without justification.

#### Example: Using println! in Library Code

```rust
// BAD: Using println! in library code
pub fn log_verb_execution(verb: &VerbCommand) {
    println!("Executing: {}", verb.name());
}

// GOOD: Use log macros
pub fn log_verb_execution(verb: &VerbCommand) {
    log::info!("Executing: {}", verb.name());
}
```

**Review comment:**
```
Library code should use `log::info!()` instead of `println!()`. Output goes to
stderr or files, not stdout, and respects the logging level (users can filter via
`RUST_LOG`). The project guidelines (CLAUDE.md) reserve `println!` for binaries
and tests only.
```

---

#### Example: Direct `cargo` Commands Instead of `cargo make`

```
BAD (in Makefile or docs):
  cargo build
  cargo test
  
GOOD:
  cargo make build
  cargo make test
```

**Review comment:**
```
Always use `cargo make` for consistency. These commands abstract away the specific
flags and features used (e.g., `cargo make test` handles features, timeouts,
determinism checks). Direct `cargo` commands bypass these safeguards.
```

---

### Future-Proofing

Check that changes don't make future extensibility harder.

#### Example: Hardcoded Values That Should Be Configurable

```rust
// BAD: Magic numbers, hard to change
const MAX_VERBS: usize = 100;

pub fn initialize_registry() -> CommandRegistry {
    CommandRegistry::with_capacity(100)
}

// GOOD: Configuration, easy to extend
const MAX_VERBS: usize = 256;

pub fn initialize_registry() -> CommandRegistry {
    CommandRegistry::with_capacity(MAX_VERBS)
}
```

**Review comment:**
```
The magic number `100` is hardcoded. Extract it to a named constant so it's easy
to change globally. Better yet, make `CommandRegistry::with_capacity()` accept a
parameter or use a default that's easily configurable (e.g., via `const MAX_VERBS`).
```

---

## Testing

### Coverage

Verify that public functions and key paths have tests.

#### Example: Missing Test for Public Function

```rust
// BAD: No test for public function
pub fn validate_verb_name(name: &str) -> bool {
    name.len() > 0 && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// GOOD: Comprehensive tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_verb_name_with_valid_name() {
        assert!(validate_verb_name("valid_name"));
    }

    #[test]
    fn test_validate_verb_name_with_empty_string() {
        assert!(!validate_verb_name(""));
    }

    #[test]
    fn test_validate_verb_name_with_invalid_chars() {
        assert!(!validate_verb_name("invalid-name"));
    }
}
```

**Review comment:**
```
The public function `validate_verb_name` has no tests. Add tests covering:
- Valid names (alphanumeric + underscores)
- Empty string (should fail)
- Invalid characters (should fail)
Follow the AAA pattern: Arrange, Act, Assert.
```

---

### Edge Cases in Tests

Verify that tests cover boundary conditions, not just happy paths.

#### Example: Only Happy Path Tested

```rust
// BAD: Only tests success case
#[test]
fn test_find_verb() {
    let registry = CommandRegistry::new();
    registry.register("services", "status");
    let result = registry.find_verb("services", "status");
    assert!(result.is_ok());  // Vague assertion
}

// GOOD: Tests multiple cases
#[test]
fn test_find_verb_with_registered_verb() {
    let registry = CommandRegistry::new();
    registry.register("services", "status");
    let result = registry.find_verb("services", "status");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "status");
}

#[test]
fn test_find_verb_with_unregistered_noun() {
    let registry = CommandRegistry::new();
    let result = registry.find_verb("invalid", "status");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), NounVerbError::CommandNotFound { ... });
}

#[test]
fn test_find_verb_with_unregistered_verb() {
    let registry = CommandRegistry::new();
    registry.register("services", "start");
    let result = registry.find_verb("services", "stop");
    assert!(result.is_err());
}
```

**Review comment:**
```
The test only covers the success case. Add tests for:
- Unregistered nouns (should return error)
- Unregistered verbs under a registered noun (should return error)
- Empty registry (should return error)
- Verify error details, not just `is_ok()` / `is_err()`

Use `assert_eq!(result.unwrap_err(), expected_error)` to verify exact error conditions.
```

---

### Test Determinism

Verify tests pass consistently, without random failures.

#### Example: Test with Time Dependency

```rust
// BAD: Test result depends on timing
#[tokio::test]
async fn test_command_executes_quickly() {
    let start = std::time::Instant::now();
    execute_command().await;
    let elapsed = start.elapsed();
    assert!(elapsed < std::time::Duration::from_secs(1));
    // Flaky! May fail on slow CI machines
}

// GOOD: Test logic, not timing
#[tokio::test]
async fn test_command_executes() {
    let result = execute_command().await;
    assert!(result.is_ok());
}

// OR, if performance matters:
#[bench]
fn bench_command_execution(b: &mut Bencher) {
    b.iter(|| execute_command());
}
```

**Review comment:**
```
This test checks execution time, which is environment-dependent and causes flaky
failures on slow CI machines. Either:
1. Remove the timing assertion and test the logic instead
2. Move it to a benchmark (`benches/` directory) using Criterion
For deterministic tests, focus on behavior, not performance. Performance testing
is handled separately via `cargo make bench`.
```

---

#### Example: Test with Shared Mutable State

```rust
// BAD: Unsafe shared state in tests
static mut COUNTER: i32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_double() {
    unsafe { COUNTER *= 2; }
    assert_eq!(unsafe { COUNTER }, 2);  // Fails if test_increment ran first!
}

// GOOD: Use thread-local or per-test state
#[test]
fn test_increment() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}

#[test]
fn test_double() {
    let mut counter = 0;
    counter *= 2;
    assert_eq!(counter, 0);
}
```

**Review comment:**
```
Tests share mutable state (`unsafe COUNTER`), causing non-deterministic failures
depending on test execution order. Each test must be independent. Use thread-local
storage or per-test local variables instead.

Run `cargo make test-lib-deterministic` to force single-threaded execution and
catch these issues.
```

---

### Test Quality

Verify tests check actual behavior, not just "is_ok()".

#### Example: Trivial Assertion

```rust
// BAD: Only checks Ok/Err, not behavior
#[test]
fn test_parse_command() {
    let result = parse_command("services status");
    assert!(result.is_ok());  // Vague!
}

// GOOD: Verify actual parsed values
#[test]
fn test_parse_command_extracts_noun_and_verb() {
    let result = parse_command("services status");
    assert!(result.is_ok());
    
    let parsed = result.unwrap();
    assert_eq!(parsed.noun, "services");
    assert_eq!(parsed.verb, "status");
}
```

**Review comment:**
```
This test only checks that parsing succeeded, not that the result is correct. What
if `parse_command` always returns `Ok` with garbage values? Assert on the actual
values: `assert_eq!(parsed.noun, "services")`, etc. The test should verify the
observable output, not just success/failure.
```

---

## Style & Idioms

### Formatting

Check that code is formatted per `rustfmt.toml` (100-char line width, 4-space tabs).

#### Example: Formatting Issue

```rust
// BAD: Line too long, inconsistent indentation
pub fn register_verb_with_long_name_and_complicated_signature(name: &str, handler: Box<dyn VerbHandler>, additional_param: String) -> Result<()> {
    Ok(())
}

// GOOD: Properly formatted
pub fn register_verb_with_long_name(
    name: &str,
    handler: Box<dyn VerbHandler>,
    additional_param: String,
) -> Result<()> {
    Ok(())
}
```

**Review comment:**
```
This line exceeds 100 characters. Run `cargo make format` to auto-format.
```

---

### Clippy Warnings

Verify no Clippy warnings are introduced.

#### Example: Clippy Warning

```rust
// BAD: Clippy warns (unnecessary_wraps)
pub fn get_config() -> Result<Config, String> {
    Ok(Config::default())
    // Never returns Err!
}

// GOOD: Remove unnecessary Result
pub fn get_config() -> Config {
    Config::default()
}

// OR, if errors are possible:
pub fn get_config() -> Result<Config, ConfigError> {
    // ... can return Err
}
```

**Review comment:**
```
Clippy warns: `unnecessary_wraps`. Since this function always returns `Ok`, don't
use `Result`. Either:
1. Remove the `Result` wrapper: `pub fn get_config() -> Config`
2. Make it actually error-prone: `pub fn get_config() -> Result<Config, ConfigError>`

Run `cargo make clippy` before committing to catch these.
```

---

### Rustdoc Compliance

Check that doc examples compile and run.

#### Example: Broken Doc Example

```rust
/// Returns the configuration.
///
/// # Examples
/// ```
/// let config = get_config();
/// ```
pub fn get_config() -> Config {
    Config::default()
}
// Doc example doesn't assert anything; it's incomplete
```

**Review comment:**
```
The doc example compiles but doesn't verify anything. Add an assertion:
```rust
/// # Examples
/// ```
/// let config = get_config();
/// assert!(config.is_valid());
/// ```
```
Or mark it as `ignore` if it's not runnable: `` ```ignore ``.
```

---

## Compatibility

### Breaking Changes

Verify that changes don't break the public API without deprecation.

#### Example: Breaking Change Without Deprecation

```rust
// BAD: Public API changed without warning
// v1.0.0:
pub fn register_verb(name: &str) -> Result<()> { ... }

// v1.1.0:
pub fn register_verb(name: &str, priority: u8) -> Result<()> { ... }
// Existing code breaks!

// GOOD: Add new function, deprecate old one
pub fn register_verb(name: &str) -> Result<()> {
    register_verb_with_priority(name, 0)
}

#[deprecated(since = "1.1.0", note = "use register_verb_with_priority instead")]
pub fn register_verb_old(name: &str) -> Result<()> { ... }

pub fn register_verb_with_priority(name: &str, priority: u8) -> Result<()> { ... }
```

**Review comment:**
```
This changes the signature of a public function, breaking existing code. You need:
1. A new function with the new signature: `register_verb_with_priority(...)`
2. Mark the old one as deprecated: `#[deprecated(...)]`
3. Bump the minor or major version (depending on severity)

See the ADL and CLAUDE.md for our deprecation strategy.
```

---

### Version Bumping

Verify that `Cargo.toml` versions are bumped appropriately.

#### Example: Forgetting to Bump Version

```
BAD:
- Added breaking change, but version is still 26.6.14

GOOD:
- Bug fix: 26.6.14 -> 26.6.15 (patch)
- New feature: 26.6.14 -> 26.7.0 (minor)
- Breaking change: 26.6.14 -> 27.0.0 (major)
```

**Review comment:**
```
You've added a new public function, but didn't bump the version in `Cargo.toml`.
Update to 26.7.0 (minor version) since this is a new, non-breaking feature.
Maintain semver: patch for bug fixes, minor for new features, major for breaking changes.
```

---

## Tone & Culture

### Constructive Feedback

Reviews should be **kind, specific, and actionable**.

#### Example: Harsh Comment ❌

```
"This code is terrible. Why would you write it like this? Use a HashMap."
```

#### Example: Constructive Comment ✅

```
"This uses nested loops which gives O(n²) complexity. For large registries, this
could be slow. Consider using a HashMap instead: `verbs.iter().find(|v| v.name() == name)`.
Here's an example: [code]. This would be O(n) on average."
```

---

### Acknowledging Good Code

Celebrate patterns, optimizations, or clarity that stands out.

#### Good Comment

```
"I like how you're using pattern matching here—it's much clearer than the nested
if-else it replaces. The error messages are also very helpful for debugging!"
```

---

### Learning Opportunities

Frame feedback as a chance to learn, not a failure.

#### Good Comment

```
"I notice you're using `format!()` here. In this case, since the string is only
used once, it might be more efficient to use `write!()` directly to a buffer.
No urgency to change, but it's a pattern that can help with performance-critical
code. See the RFC for more details: [link]."
```

---

### Separating Nits from Substantive Feedback

Use clear labels to distinguish between different feedback types.

Good Practice:

```
**Must-fix**: This violates ADL-005 (no panics). [details]

**Should-fix**: This has O(n²) complexity and might affect performance. [details]

**Nice-to-have**: Consider using `Cow<str>` here to avoid allocation. [details]

**Nit**: Variable name `x` could be `index` for clarity. [details]
```

---

## Review Checklist

Use this checklist during code review. Not all items apply to every PR; use judgment.

### Pre-Review
- [ ] PR description is clear and complete
- [ ] Submitter has done self-review (check comment history)
- [ ] All conversations resolved from previous feedback
- [ ] PR is focused (one feature/fix, not sprawling)
- [ ] Tests pass locally (`cargo make ci`)

### Correctness
- [ ] No logic errors or off-by-one bugs
- [ ] Edge cases handled (empty input, None, zero, etc.)
- [ ] Errors properly handled and propagated
- [ ] No resource leaks or lifetime violations
- [ ] Return types correct (Result vs Option)
- [ ] No panics (unwrap, expect, assert, array indexing on unknown bounds)

### Performance
- [ ] Algorithmic complexity appropriate (O(n) not O(n²) for large data)
- [ ] No unnecessary allocations
- [ ] No large stack allocations (>1KB)
- [ ] Compilation time impact checked (incremental build <2s)
- [ ] No premature optimization without justification

### Security
- [ ] No unsafe code without justification and safety comments
- [ ] User input is validated
- [ ] No panics in production code
- [ ] Dependencies checked for vulnerabilities (`cargo deny check`)
- [ ] Licenses are permissive (MIT, Apache-2.0, BSD, ISC)

### Maintainability
- [ ] Names are clear and descriptive
- [ ] Functions do one thing (not too complex)
- [ ] Cyclomatic complexity reasonable
- [ ] Public APIs documented with examples
- [ ] Comments explain "why", not "what"
- [ ] Code is DRY (no unnecessary duplication)

### Architecture
- [ ] Aligns with ADL (see CLAUDE.md)
- [ ] Follows project conventions (e.g., `cargo make`, not direct `cargo`)
- [ ] No unexpected patterns (e.g., println! in library code)
- [ ] Future extensibility considered
- [ ] Trait design (if applicable) is dyn-compatible and sync-only

### Testing
- [ ] Public functions have tests
- [ ] Edge cases are tested (not just happy path)
- [ ] Tests are deterministic (no timing/ordering dependencies)
- [ ] Tests verify behavior, not just Ok/Err status
- [ ] No flaky tests
- [ ] Test names are descriptive

### Style
- [ ] Code is formatted (`cargo make format` passes)
- [ ] No Clippy warnings (`cargo make clippy` passes)
- [ ] Doc examples compile (`cargo test --doc` passes)
- [ ] Line width <=100 chars, 4-space tabs
- [ ] Imports are organized

### Compatibility
- [ ] No breaking changes without deprecation
- [ ] Version bumped appropriately (semver)
- [ ] Public API changes documented
- [ ] Deprecation strategy followed (if applicable)
- [ ] CHANGELOG updated (if applicable)

### Documentation
- [ ] Public APIs have rustdoc comments
- [ ] Examples are correct and runnable
- [ ] Error cases explained
- [ ] Panics documented (though should be zero)
- [ ] Complex logic has comments

---

## Common Anti-Patterns

### Anti-Pattern 1: Unwrap-Heavy Code

**What it is**: Excessive use of `unwrap()`, `expect()`, or panic-prone patterns.

**Why it's bad**:
- Crashes on edge cases
- No error context for debugging
- Violates ADL-005 (no panics)

**Example**:
```rust
// BAD
let value = result.unwrap();
let first = vec.get(0).unwrap();
let parsed = json_str.parse::<Value>().unwrap();

// GOOD
let value = result.map_err(|e| NounVerbError::Generic(e.to_string()))?;
let first = vec.first().ok_or(NounVerbError::EmptyList)?;
let parsed = serde_json::from_str::<Value>(json_str)
    .map_err(|e| NounVerbError::InvalidJson(e.to_string()))?;
```

---

### Anti-Pattern 2: Print-Based Debugging Left in Code

**What it is**: `println!()` or `dbg!()` macros left in production code.

**Why it's bad**:
- Pollutes stdout/stderr
- Hard to disable
- Should use `log::` crate instead

**Example**:
```rust
// BAD
pub fn route_command(cmd: &str) {
    println!("Routing: {}", cmd);  // Debug output in library code!
    // ...
}

// GOOD
pub fn route_command(cmd: &str) {
    log::debug!("Routing: {}", cmd);  // Respects RUST_LOG level
    // ...
}
```

---

### Anti-Pattern 3: Silent Error Swallowing

**What it is**: Errors caught but ignored with `let _ = result`.

**Why it's bad**:
- No visibility into failures
- Hard to debug
- May hide bugs

**Example**:
```rust
// BAD
for verb in verbs {
    let _ = registry.register(verb);  // Failures are silently dropped
}

// GOOD
for verb in verbs {
    registry.register(verb)
        .map_err(|e| log::warn!("Failed to register verb: {}", e))
        .ok();  // Only if partial failure is acceptable
}

// OR, better:
for verb in verbs {
    registry.register(verb)?;  // Propagate the error
}
```

---

### Anti-Pattern 4: Magic Numbers

**What it is**: Hardcoded constants without explanation.

**Why it's bad**:
- Hard to understand intent
- Hard to change globally
- Easy to introduce bugs

**Example**:
```rust
// BAD
if verbs.len() > 100 {
    log::warn!("Too many verbs");
}

// GOOD
const MAX_VERBS: usize = 100;

if verbs.len() > MAX_VERBS {
    log::warn!("Too many verbs (max: {})", MAX_VERBS);
}
```

---

### Anti-Pattern 5: No Error Context

**What it is**: Errors with generic messages like "failed" or "error".

**Why it's bad**:
- Hard to debug
- User can't tell what went wrong
- Makes support harder

**Example**:
```rust
// BAD
file.read_to_string(&mut buf)
    .map_err(|_| NounVerbError::Generic("failed".to_string()))?;

// GOOD
file.read_to_string(&mut buf)
    .map_err(|e| NounVerbError::Generic(
        format!("Failed to read config.json: {}", e)
    ))?;
```

---

### Anti-Pattern 6: Mixing Levels of Abstraction

**What it is**: Code that jumps between high-level logic and low-level details.

**Why it's bad**:
- Hard to follow
- Hard to test
- Violates single responsibility

**Example**:
```rust
// BAD: Mixes business logic with I/O
pub fn process_verbs(path: &str) -> Result<Output> {
    let file = std::fs::File::open(path)?;  // Low-level I/O
    let verbs: Vec<VerbCommand> = serde_json::from_reader(file)?;  // Parsing
    
    let mut total = 0;
    for verb in verbs {
        total += verb.compute_score();  // Business logic
    }
    
    println!("Total: {}", total);  // Low-level output
    Ok(Output { total })
}

// GOOD: Separate concerns
pub fn process_verbs(verbs: &[VerbCommand]) -> Result<u32> {
    verbs.iter()
        .map(|v| v.compute_score())
        .sum()
}

pub fn load_and_process(path: &str) -> Result<Output> {
    let file = std::fs::File::open(path)?;
    let verbs = serde_json::from_reader(file)?;
    let total = process_verbs(&verbs)?;
    Ok(Output { total })
}
```

---

### Anti-Pattern 7: Tight Coupling to External Dependencies

**What it is**: Direct usage of specific libraries in core logic, making it hard to swap implementations.

**Why it's bad**:
- Hard to test (requires the actual dependency)
- Hard to swap implementations
- Violates dependency inversion

**Example**:
```rust
// BAD: Tightly coupled to serde_json
pub fn serialize_config(cfg: &Config) -> Result<String> {
    serde_json::to_string(cfg)
        .map_err(|e| NounVerbError::Generic(e.to_string()))
}

// BETTER: Parameterized or abstracted
pub fn serialize_config<S: Serializer>(cfg: &Config, serializer: S) -> Result<()> {
    cfg.serialize(serializer)
}

// OR, wrap the dependency:
pub fn serialize_config_to_json(cfg: &Config) -> Result<String> {
    serde_json::to_string(cfg)
        .map_err(|e| NounVerbError::SerializationError(e.to_string()))
}
```

---

### Anti-Pattern 8: Tests That Only Check is_ok()

**What it is**: Tests that verify success/failure but don't verify the actual output.

**Why it's bad**:
- Doesn't catch logic bugs
- Allows any output as long as it's "Ok"
- False sense of coverage

**Example**:
```rust
// BAD
#[test]
fn test_parse_verb() {
    let result = parse_verb("services status");
    assert!(result.is_ok());  // What if parse_verb returns Ok(garbage)?
}

// GOOD
#[test]
fn test_parse_verb_extracts_noun_and_verb() {
    let result = parse_verb("services status");
    assert!(result.is_ok());
    
    let parsed = result.unwrap();
    assert_eq!(parsed.noun, "services");
    assert_eq!(parsed.verb, "status");
}
```

---

### Anti-Pattern 9: Trait Methods with Lifetimes Beyond 'static

**What it is**: Traits with methods that return non-'static lifetimes.

**Why it's bad**:
- Makes traits non-object-safe (`dyn Trait` doesn't work)
- Hard to use as trait objects
- Violates ADL-010

**Example**:
```rust
// BAD: Non-object-safe trait
pub trait VerbCommand {
    fn name<'a>(&self, ctx: &'a Context) -> &'a str;  // Non-'static return
}

// GOOD: Object-safe, 'static returns
pub trait VerbCommand {
    fn name(&self) -> &'static str;
}

// OR, if dynamic, use owned types:
pub trait VerbCommand {
    fn name(&self) -> String;
}
```

---

### Anti-Pattern 10: Circular Dependencies Between Modules

**What it is**: Module A imports from B, B imports from A (or longer cycles).

**Why it's bad**:
- Hard to compile (cycles often cause errors)
- Hard to reason about dependencies
- Violates separation of concerns

**Example**:
```rust
// BAD: Circular dependency
// registry.rs imports from router.rs:
use crate::router::CommandRouter;

// router.rs imports from registry.rs:
use crate::registry::CommandRegistry;

// GOOD: Dependency flow is one-directional
// router.rs imports from registry.rs:
use crate::registry::CommandRegistry;

// registry.rs does NOT import from router.rs
```

To detect: Run `cargo tree --duplicates` and look for dependency cycles.

---

## Example Comments

Real-world examples of review comments applying the standards above.

### Example 1: Comprehensive Correctness Review

```
Title: Check: `find_verb` logic and error handling

**Concern**: Logic appears correct, but error context is thin.

The function correctly searches for verbs in the registry. However, consider these:

1. **Error message quality**: If no verb is found, return a more helpful error:
   ```rust
   Err(NounVerbError::VerbNotFound {
       noun: noun.to_string(),
       verb: verb.to_string(),
       suggestion: suggest_similar(verb),
   })
   ```
   This gives users a hint for typos.

2. **Test coverage**: The implementation looks good, but tests should cover:
   - Unregistered noun (should return error)
   - Registered noun but unregistered verb (should return error)
   - Valid noun+verb pair (should return handler)

Otherwise, this looks solid!
```

---

### Example 2: Performance Feedback

```
Title: Performance: Possible O(n²) in verb validation

This looks fine, but I want to flag a potential performance issue for discussion.

**Current**: You're iterating all verbs and checking validity for each one.
**Impact**: For a registry with 1000+ verbs, this could be slow.

**Suggestion** (not urgent):
Could we validate once during registration and store the flag, rather than
re-validating on each lookup? That shifts the cost to add-time (one-time) instead
of lookup-time (frequent).

Not a must-fix for this PR, but worth keeping in mind as we grow.

Example:
```rust
pub struct Verb {
    name: String,
    is_valid: bool,  // Computed once at registration
}
```

What do you think?
```

---

### Example 3: Architecture Alignment

```
Title: Architecture: Trait object compatibility

Per ADL-010, our traits must be dyn-compatible (object-safe). I noticed this:

```rust
pub trait VerbHandler {
    async fn execute(&self, input: Input) -> Result<Output>;
}
```

**Issue**: Async methods in traits aren't object-safe. This breaks `dyn VerbHandler`.

**Fix**: Remove the `async` keyword. Async is handled via the `async_verb.rs`
module when needed.

```rust
pub trait VerbHandler {
    fn execute(&self, input: Input) -> Result<Output>;
}
```

If you need async, use the `async_verb.rs` feature-gated module instead.

Thanks!
```

---

### Example 4: Testing Feedback

```
Title: Tests: Missing edge case coverage

The tests cover the happy path well! But I'm noticing a gap:

**Missing**:
- What happens if the input verb string is empty?
- What if the verb name has special characters?
- What if there are duplicate verb registrations?

Let's add tests for these:

```rust
#[test]
fn test_register_duplicate_verb_returns_error() {
    let mut registry = CommandRegistry::new();
    registry.register("services", "status").unwrap();
    
    let result = registry.register("services", "status");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        NounVerbError::DuplicateVerbRegistration { ... }
    );
}
```

This ensures we handle duplicates correctly (vs. silently overwriting).

Good work on the foundation—just a bit more coverage to be complete.
```

---

### Example 5: Documentation Feedback

```
Title: Docs: Please add rustdoc and examples

Great implementation! Now let's document the public API:

```rust
/// Register a verb command with the given noun.
///
/// # Arguments
/// * `noun` - The noun grouping (e.g., "services")
/// * `verb` - The verb name (e.g., "status")
/// * `handler` - The verb handler function
///
/// # Returns
/// - `Ok(())` if registration succeeds
/// - `Err(NounVerbError::DuplicateVerbRegistration)` if already registered
/// - `Err(NounVerbError::InvalidName)` if the name is invalid
///
/// # Examples
/// ```
/// let mut registry = CommandRegistry::new();
/// registry.register("services", "status", handle_services_status)?;
/// assert!(registry.find_verb("services", "status").is_ok());
/// ```
pub fn register(&mut self, noun: &str, verb: &str, handler: Box<dyn VerbHandler>) -> Result<()> {
    // ...
}
```

This helps users understand:
- What the function does
- What errors can occur (and when)
- How to use it

Run `cargo make doc` to verify it renders correctly!
```

---

## Summary

Code reviews should be:

1. **Thorough**: Check all 10 categories (correctness, performance, security, maintainability, architecture, testing, style, compatibility, documentation, tone)
2. **Constructive**: Frame feedback as learning, not criticism
3. **Specific**: Point to exact lines and explain why
4. **Actionable**: Provide examples or links to solutions
5. **Kind**: Acknowledge good work, use clear labels for priority

Reviewers and submitters both play a role in creating a culture of high quality and continuous improvement.

---

**Last Updated**: 2026-06-14
**Version**: 1.0
**Maintainer**: Code Review Committee

For questions or updates, open an issue with the label `code-review-standards`.
