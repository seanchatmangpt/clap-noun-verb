# clap-noun-verb v6.0.0 - Breaking Changes & Migration Guide

**Version**: 6.0.0
**Status**: SPARC Specification - Migration Guidance
**Previous Version**: 5.5.0
**Release**: Q1 2026

---

## Quick Reference

| Change | Impact | Migration | Complexity |
|--------|--------|-----------|-----------|
| Handler trait signature | HIGH | Update all handlers | Medium |
| Error type separation | HIGH | Add explicit error types | Medium |
| Feature restructuring | MEDIUM | Update Cargo.toml | Low |
| Capability constraints | MEDIUM | Add trait bounds | Medium |
| Phantom type parameters | MEDIUM | Add generics | Medium |
| Default features empty | LOW | Enable features explicitly | Low |
| Unwrap/expect elimination | LOW | Already Result-based API | Low |

---

## Table of Contents

1. [Migration Summary](#1-migration-summary)
2. [Handler API Changes](#2-handler-api-changes)
3. [Type System Changes](#3-type-system-changes)
4. [Feature Flag Changes](#4-feature-flag-changes)
5. [Error Handling Changes](#5-error-handling-changes)
6. [Public API Removals](#6-public-api-removals)
7. [Detailed Migration Examples](#7-detailed-migration-examples)
8. [Troubleshooting Guide](#8-troubleshooting-guide)

---

## 1. Migration Summary

### 1.1 Why These Changes?

**Reason 1: Type Safety**
- v5.5.0: Handler signatures had single output type, errors bundled
- v6.0.0: Separate error type from output for better Result semantics
- Benefit: Impossible to forget error handling at compile-time

**Reason 2: Determinism**
- v5.5.0: Policy checks were runtime-only
- v6.0.0: Type-level constraints prevent invalid states
- Benefit: More errors caught before execution

**Reason 3: Clarity**
- v5.5.0: Frontier features had unclear maturity
- v6.0.0: Explicit tiers (Stable, Experimental, Frontier)
- Benefit: Clear expectations for production use

### 1.2 Three-Step Migration Path

**Step 1: Update Cargo.toml** (5 minutes)
- Remove `default = ["agent2028", "autonomic"]`
- Add explicit features: `features = ["async", "rdf"]`

**Step 2: Update Handler Implementations** (30 minutes)
- Change from `Handler<Args>` to `Handler<Args, Error, Output>`
- Separate error type from output type
- Run tests after each change

**Step 3: Update Capability Bounds** (1 hour)
- Add trait bounds to generics
- Use `where H: CapabilityBound<AdminCap>`
- Verify compiler accepts new constraints

---

## 2. Handler API Changes

### 2.1 Change: Handler Trait Signature

**v5.5.0 Signature**
```rust
pub trait Handler<A: Args>: Send + Sync {
    type Output;
    type Error: Error;

    fn execute(&self, args: A) -> Result<Self::Output, Self::Error>;
}

// Implementation
impl Handler<CreateArgs> for CreateHandler {
    type Output = CreateResult;
    type Error = CliError;

    fn execute(&self, args: CreateArgs) -> Result<CreateResult, CliError> {
        // ...
    }
}
```

**v6.0.0 Signature**
```rust
pub trait Handler<A: Args, E: Error, O: Output>: Send + Sync {
    fn execute(&self, args: A) -> Result<O, E>;
}

// Implementation
impl Handler<CreateArgs, CliError, CreateResult> for CreateHandler {
    fn execute(&self, args: CreateArgs) -> Result<CreateResult, CliError> {
        // ...
    }
}
```

**Why This Change**
- Associated types in v5.5.0 made bounds harder to express
- Generic parameters in v6.0.0 fit better with trait bounds
- Enables: `T: Handler<A, E, O>` trait bounds
- Matches Rust idiom: type parameter first, then associated types

**Migration Impact**
- All custom handler implementations must be updated
- Built-in handlers updated automatically in v6.0.0
- Tests require matching signature update
- No functional behavior changes

**Code Diff**
```diff
- impl Handler<CreateArgs> for CreateHandler {
-     type Output = CreateResult;
-     type Error = CliError;
+ impl Handler<CreateArgs, CliError, CreateResult> for CreateHandler {
-     fn execute(&self, args: CreateArgs) -> Result<CreateResult, CliError> {
+     fn execute(&self, args: CreateArgs) -> Result<CreateResult, CliError> {
          // implementation unchanged
      }
  }
```

### 2.2 Change: Generic Handler Factory

**v5.5.0 Pattern** (Not directly supported)
```rust
// No standard way to create generic handlers in v5.5.0
fn create_handler<A>() -> Box<dyn Handler<A>> { ... }  // Not possible
```

**v6.0.0 Pattern** (Standard with trait bounds)
```rust
// Now possible with explicit type parameters
pub fn create_handler<A, E, O>() -> Box<dyn Handler<A, E, O>>
where
    A: Args + 'static,
    E: Error + 'static,
    O: Output + 'static,
{
    // Create handler with all types constrained
}
```

**Benefits**
- Factory functions are now type-safe
- Generics compose with trait bounds
- Error types are explicit at factory level

---

## 3. Type System Changes

### 3.1 Change: Error Type Separation

**The Problem**
```rust
// v5.5.0: Handler<A> with associated Output and Error
trait Handler<A> {
    type Output;
    type Error;
}

// Hard to express in bounds:
// "I want handlers that return Json with CliError"
// Best we could do:
// where H: Handler<Args> (but then what's the output?)
```

**The Solution**
```rust
// v6.0.0: Handler<A, E, O> with explicit generics
trait Handler<A, E, O> {
    fn execute(&self, args: A) -> Result<O, E>;
}

// Now easy to express in bounds:
// "I want handlers that return Json with CliError"
pub fn register<H>(h: H)
where
    H: Handler<Args, CliError, Json>,
{
    // H must have exactly those types
}
```

**Migration Pattern**
```rust
// ❌ OLD: Infer from implementation
impl Handler<Args> for MyHandler {
    type Output = Json;
    type Error = CliError;
    // ...
}

// ✅ NEW: Explicit in impl header
impl Handler<Args, CliError, Json> for MyHandler {
    // ...
}
```

### 3.2 Change: Phantom Type Parameters

**v5.5.0** (No phantom type support)
```rust
// No way to express "this handler requires AdminCap" in types
pub struct AdminHandler {
    // Runtime check in execute():
    // if !has_admin_capability() { return Err(...) }
}
```

**v6.0.0** (Type-level capability encoding)
```rust
// Phantom type parameter encodes capability at compile-time
pub struct AdminHandler<C: Capability = AdminCap> {
    _cap: PhantomData<C>,
}

impl Handler<Args, CliError, Output> for AdminHandler<AdminCap> {
    fn execute(&self, args: Args) -> Result<Output, CliError> {
        // Compiler guarantees C = AdminCap
        // No runtime check needed
    }
}

// Handler without AdminCap won't compile:
// impl Handler for AdminHandler<ReadCap> { ... }  // Compiler error!
```

**Benefits**
- Impossible to instantiate AdminHandler without AdminCap
- Zero runtime overhead (zero-cost abstraction)
- Compiler prevents capability violations

**Migration Example**
```rust
// ❌ OLD: Runtime capability check
pub struct MyHandler;
impl Handler<Args> for MyHandler {
    type Output = Output;
    type Error = CliError;

    fn execute(&self, args: Args) -> Result<Output, CliError> {
        // Runtime check
        if !has_capability("admin") {
            return Err(CliError::Unauthorized);
        }
        // ...
    }
}

// ✅ NEW: Compile-time capability constraint
pub struct MyHandler;
impl Handler<Args, CliError, Output> for MyHandler
where
    Self: CapabilityBound<AdminCap>,
{
    fn execute(&self, args: Args) -> Result<Output, CliError> {
        // Compiler proves capability exists
        // No runtime check needed
    }
}
```

### 3.3 Change: Const Generic Constraints

**Depth Limits (Prevents Stack Overflow)**

v5.5.0 (Runtime bounds checking):
```rust
pub fn create_tree(depth: usize) -> Result<CommandTree, Error> {
    if depth > 100 {
        return Err(Error::DepthTooDeep);
    }
    // Recursive creation with runtime checks
}
```

v6.0.0 (Compile-time bounds):
```rust
pub struct CommandTree<const MAX_DEPTH: usize> {
    // Depth bounded at compile-time
}

impl<const D: usize> CommandTree<D> {
    pub fn new() -> Self {
        // Compiler proves D fits in stack
        // const D limits maximum recursion
    }
}

// Usage:
let tree: CommandTree<10> = CommandTree::new(); // Compile-time verified
let deep: CommandTree<1000> = CommandTree::new(); // Compiler error if stack too small
```

---

## 4. Feature Flag Changes

### 4.1 Change: Feature Restructuring

**v5.5.0 Features**
```toml
[features]
default = ["autonomic", "crypto"]  # Always enabled
agent2028 = ["async", "crypto", ...]  # One-level
rdf = ["crypto", ...]              # No hierarchy
```

**v6.0.0 Features** (Three-tier system)
```toml
[features]
default = []  # Explicitly empty!

# Tier 1: Stability levels
frontier-all = ["meta-framework", "rdf-composition", ...]
frontier-semantic = ["meta-framework", "rdf-composition", ...]

# Tier 2: Individual features
meta-framework = [...]
rdf-composition = ["rdf", ...]
executable-specs = [...]

# Tier 3: Base features (unchanged)
async = [...]
rdf = [...]
```

**Why This Change**
- v5.5.0: Users didn't know maturity level of features
- v6.0.0: Clear tiers (Stable, Experimental, Frontier)
- v5.5.0: Default features increased compile time
- v6.0.0: Empty defaults, users choose what they need

**Migration Steps**

1. **Check current Cargo.toml**
   ```toml
   # v5.5.0
   [dependencies]
   clap-noun-verb = "5.5"
   # Gets default = ["autonomic", "crypto"]
   ```

2. **Update to v6.0.0 with explicit features**
   ```toml
   # v6.0.0
   [dependencies]
   clap-noun-verb = {
       version = "6.0",
       features = ["autonomic", "crypto"]  # Explicit!
   }
   ```

3. **If using frontier features**
   ```toml
   # Get convenience bundle
   clap-noun-verb = {
       version = "6.0",
       features = ["frontier-all"]
   }

   # OR pick specific features
   clap-noun-verb = {
       version = "6.0",
       features = [
           "meta-framework",
           "rdf-composition",
           "fractal-patterns",
       ]
   }
   ```

### 4.2 Change: Autonomic Feature

**What Changed**
- v5.5.0: Autonomic always enabled by default
- v6.0.0: Autonomic is optional feature
- Impact: Slightly faster compilation if not needed

**Migration**
```toml
# v5.5.0 (implicit autonomic)
clap-noun-verb = "5.5"

# v6.0.0 (explicit autonomic)
clap-noun-verb = { version = "6.0", features = ["autonomic"] }
```

**Who Needs It**
- ✅ Using `#[verb]` macro with policy enforcement
- ✅ Using kernel receipts or certificates
- ✅ Using telemetry/observability
- ❌ Simple CLI without these features

### 4.3 Feature Matrix

| Feature | v5.5.0 | v6.0.0 | Status | Migration |
|---------|--------|--------|--------|-----------|
| Core CLI | Required | Required | Stable | None |
| async | Optional | Optional | Stable | No change |
| crypto | Optional | Optional | Stable | No change |
| autonomic | Default | Optional | Stable | Add explicitly |
| agent2028 | Optional | → frontier-intelligence | Frontier | Use new name |
| rdf | Optional | rdf-composition | Frontier | Feature-renamed |
| kernel | New | Stable | v6.0 | Add explicitly |

---

## 5. Error Handling Changes

### 5.1 Change: Unified Error Type

**v5.5.0** (Multiple error sources)
```rust
// Different errors from different modules
pub enum ParseError { ... }
pub enum PolicyError { ... }
pub enum CapabilityError { ... }

// Hard to unify in Result:
pub fn handle(args: &str) -> Result<Output, Box<dyn Error>> { ... }
```

**v6.0.0** (Single unified error)
```rust
pub enum CliError {
    ParseError(String),
    PolicyError(String),
    CapabilityError(String),
    // ...
}

// Easy to unify:
pub fn handle(args: &str) -> Result<Output, CliError> { ... }
```

**Benefits**
- Pattern matching on error type is type-safe
- Error messages are consistent
- Serialization for audit trails simplified

**Migration**
```rust
// ❌ OLD: Pattern match on error type
match operation() {
    Ok(v) => ...,
    Err(e) if e.is::<ParseError>() => ...,
    Err(e) if e.is::<PolicyError>() => ...,
}

// ✅ NEW: Simple enum matching
match operation() {
    Ok(v) => ...,
    Err(CliError::ParseError(e)) => ...,
    Err(CliError::PolicyError(e)) => ...,
}
```

### 5.2 Change: No Unwrap/Expect in Library

**v5.5.0** (Some unwrap calls)
```rust
// Occasionally present:
let value = operation().unwrap();  // Panic if error!
```

**v6.0.0** (100% Result-based)
```rust
// All operations return Result:
let value = operation()?;  // Propagate error safely
```

**Impact on Library Users**
- ✅ Library never panics unexpectedly
- ✅ All errors can be handled with Result<T, E>
- ✅ Production-safe error semantics

---

## 6. Public API Removals

### 6.1 Deprecated Items Removed

These items were deprecated in v5.5.0 and are removed in v6.0.0:

| Item | Removed | Replacement | Migration |
|------|---------|------------|-----------|
| `Handler::Output` assoc type | Yes | Use generic `O` parameter | Update impl |
| `Handler::Error` assoc type | Yes | Use generic `E` parameter | Update impl |
| `legacy_handler_registry()` | Yes | Use `register_handler()` | Update registration |
| `unsafe_execute()` | Yes | Use `execute()` | Remove unsafe code |

### 6.2 Detailed Removal Guide

**Removal 1: Associated Type Removal**

Deprecated in v5.5.0:
```rust
impl Handler<Args> for MyHandler {
    type Output = Json;  // ⚠️ Deprecated in 5.5.0
    type Error = CliError;  // ⚠️ Deprecated in 5.5.0
}
```

Removed in v6.0.0 (compile error):
```rust
// This no longer compiles in v6.0.0!
impl Handler<Args> for MyHandler {
    type Output = Json;
    type Error = CliError;
}

// Error: trait Handler takes 3 type parameters but 1 were supplied
```

Fix:
```rust
// Update to v6.0.0 form
impl Handler<Args, CliError, Json> for MyHandler {
    fn execute(&self, args: Args) -> Result<Json, CliError> {
        // implementation
    }
}
```

**Removal 2: Legacy Registry Function**

Deprecated in v5.5.0:
```rust
legacy_handler_registry::register(handler);  // ⚠️ Deprecated
```

Removed in v6.0.0:
```rust
// Compile error: function not found
legacy_handler_registry::register(handler);  // ❌ Error!
```

Fix:
```rust
// Use new registration API
handler_registry::register(handler);  // ✅ Works
```

---

## 7. Detailed Migration Examples

### 7.1 Example 1: Simple Handler Migration

**v5.5.0 Code**
```rust
use clap_noun_verb::{Handler, Args, Noun, Verb};
use serde_json::json;

#[derive(clap::Parser)]
struct MyArgs {
    #[arg(help = "Resource name")]
    name: String,
}

struct MyHandler;

impl Handler<MyArgs> for MyHandler {
    type Output = serde_json::Value;
    type Error = anyhow::Error;

    fn execute(&self, args: MyArgs) -> Result<Self::Output, Self::Error> {
        Ok(json!({
            "status": "ok",
            "name": args.name
        }))
    }
}
```

**v6.0.0 Equivalent**
```rust
use clap_noun_verb::{Handler, Args, Noun, Verb};
use serde_json::json;

#[derive(clap::Parser)]
struct MyArgs {
    #[arg(help = "Resource name")]
    name: String,
}

struct MyHandler;

impl Handler<MyArgs, CliError, serde_json::Value> for MyHandler {
    fn execute(&self, args: MyArgs) -> Result<serde_json::Value, CliError> {
        Ok(json!({
            "status": "ok",
            "name": args.name
        }))
    }
}
```

**Changes Made**
1. Handler signature: `Handler<A>` → `Handler<A, E, O>`
2. Associated types removed (now generic parameters)
3. Error type from `anyhow::Error` → `CliError` (for consistency)
4. Output type from `Self::Output` → explicit in generics

### 7.2 Example 2: Handler with Capability Constraint

**v5.5.0 Pattern** (Runtime check)
```rust
struct AdminHandler;

impl Handler<AdminArgs> for AdminHandler {
    type Output = AdminResult;
    type Error = CliError;

    fn execute(&self, args: AdminArgs) -> Result<AdminResult, CliError> {
        // Runtime capability check
        if !args.user.has_admin_capability() {
            return Err(CliError::Unauthorized);
        }
        // ... admin operation
    }
}
```

**v6.0.0 Pattern** (Type-safe)
```rust
struct AdminHandler;

impl Handler<AdminArgs, CliError, AdminResult> for AdminHandler
where
    Self: CapabilityBound<AdminCap>,
{
    fn execute(&self, args: AdminArgs) -> Result<AdminResult, CliError> {
        // Compiler guarantees AdminCap exists
        // No runtime check needed!
        // ... admin operation
    }
}
```

**Benefits**
- No runtime capability check needed
- Compiler prevents unauthorized handlers
- Type-safe capability expression

### 7.3 Example 3: Generic Handler Factory

**v5.5.0** (Not easily done)
```rust
// Hard to create generic handlers with trait bounds
fn create_handler<A>() -> Box<dyn Handler<A>> {
    // What's the Output and Error? Unclear!
}
```

**v6.0.0** (Clean with generics)
```rust
fn create_handler<A, E, O>() -> Box<dyn Handler<A, E, O>>
where
    A: Args + 'static,
    E: Error + 'static,
    O: Output + 'static,
{
    // All types explicit and constrained
}

// Usage:
let h: Box<dyn Handler<MyArgs, CliError, Json>> =
    create_handler::<MyArgs, CliError, Json>();
```

---

## 8. Troubleshooting Guide

### Issue 1: "trait Handler takes 3 type parameters but 1 were supplied"

**Problem**
```rust
impl Handler<MyArgs> for MyHandler {  // ❌ Error!
    // ...
}
```

**Cause**
- v6.0.0 Handler has 3 type parameters: `Handler<A, E, O>`
- You provided only 1: `Handler<MyArgs>`

**Solution**
```rust
impl Handler<MyArgs, CliError, JsonOutput> for MyHandler {  // ✅ Fixed!
    fn execute(&self, args: MyArgs) -> Result<JsonOutput, CliError> {
        // ...
    }
}
```

### Issue 2: "type mismatch: expected `Result<O, E>`, found `O`"

**Problem**
```rust
impl Handler<Args, CliError, Output> for MyHandler {
    fn execute(&self, args: Args) -> Output {  // ❌ Missing Result!
        // ...
    }
}
```

**Cause**
- Handler must return `Result<O, E>`, not just `O`

**Solution**
```rust
impl Handler<Args, CliError, Output> for MyHandler {
    fn execute(&self, args: Args) -> Result<Output, CliError> {  // ✅ Fixed!
        // ...
    }
}
```

### Issue 3: "the trait bound `Self: CapabilityBound<AdminCap>` is not satisfied"

**Problem**
```rust
impl Handler<Args, CliError, Output> for MyHandler
where
    Self: CapabilityBound<AdminCap>,  // ❌ Not satisfied!
{
    // ...
}
```

**Cause**
- `MyHandler` doesn't implement `CapabilityBound<AdminCap>`
- Need to explicitly declare it

**Solution**
```rust
// Implement the capability trait
impl CapabilityBound<AdminCap> for MyHandler {
    // ...
}

// Then use in Handler
impl Handler<Args, CliError, Output> for MyHandler
where
    Self: CapabilityBound<AdminCap>,  // ✅ Now satisfied!
{
    // ...
}
```

### Issue 4: "feature `frontier-all` is not found"

**Problem**
```toml
[dependencies]
clap-noun-verb = { version = "6.0", features = ["frontier-all"] }
```

**Cause**
- Feature name changed from `frontier-all` to more specific names
- Or using v5.5.0-style feature names

**Solution**
```toml
# Use actual feature names
clap-noun-verb = { version = "6.0", features = [
    "meta-framework",
    "rdf-composition",
    "fractal-patterns",
    # etc
] }

# OR use convenience bundle (if available)
clap-noun-verb = { version = "6.0", features = ["frontier-intelligence"] }
```

### Issue 5: "use of `unwrap_used` is denied"

**Problem**
```rust
let handler = registry.get("cmd").unwrap();  // ❌ Denied!
```

**Cause**
- v6.0.0 denies unwrap in production code via clippy

**Solution**
```rust
// Explicit error handling
let handler = registry.get("cmd")
    .ok_or(CliError::NotFound)?;  // ✅ Works!

// Or with context
let handler = registry.get("cmd")
    .context("Handler not found")?;  // ✅ Also works!
```

---

## Checklist for Migration

Use this checklist to track your migration progress:

### Phase 1: Preparation (1 day)
- [ ] Read v6_0_0_SPECIFICATION.md completely
- [ ] Read this migration guide completely
- [ ] Identify all custom Handler implementations
- [ ] Identify all feature flag usage
- [ ] Back up current Cargo.toml

### Phase 2: Dependency Update (1 hour)
- [ ] Update clap-noun-verb to 6.0.0 in Cargo.toml
- [ ] Add explicit features to features list
- [ ] Run `cargo check` and note errors
- [ ] Review migration guide for each error

### Phase 3: Code Update (2-4 hours)
- [ ] Update all Handler impl blocks (handler trait)
- [ ] Update all error handling (use CliError)
- [ ] Add capability bounds where needed (where clauses)
- [ ] Update feature gates in code (#[cfg] attrs)
- [ ] Remove deprecated API calls

### Phase 4: Testing (2-3 hours)
- [ ] Run `cargo test` - fix failing tests
- [ ] Run `cargo clippy` - fix warnings
- [ ] Run `cargo fmt` - fix formatting
- [ ] Test manually with key use cases

### Phase 5: Validation (1 hour)
- [ ] Code review of changes
- [ ] Run benchmarks - check for regressions
- [ ] Document any custom changes in CHANGELOG
- [ ] Verify production readiness

---

## Support & Resources

**Documentation**
- v6.0.0 Specification: `/docs/v6_0_0_SPECIFICATION.md`
- Migration Guide: This document
- API Documentation: https://docs.rs/clap-noun-verb/6.0

**Community Support**
- GitHub Issues: Report bugs/questions with `[v6-migration]` tag
- RFC Process: Propose major changes via RFC
- Examples: `/examples` contains migration examples

**Timing Estimates**

| Project Size | Estimated Time | Complexity |
|--------------|----------------|-----------|
| Small (<10 handlers) | 1-2 hours | Low |
| Medium (10-50 handlers) | 4-8 hours | Medium |
| Large (50+ handlers) | 1-2 days | High |
| Complex (with frontier) | 2-5 days | Very High |

---

## Success Criteria

Your migration is successful when:

1. ✅ Code compiles with zero clippy warnings
2. ✅ All tests pass with 100% success rate
3. ✅ No unwrap/expect in production code
4. ✅ Benchmarks show <2% performance regression
5. ✅ All functionality works identically to v5.5.0

---

**Last Updated**: 2026-01-08
**Version**: 1.0 FINAL
**Status**: COMPLETE - Ready for Implementation Phase
