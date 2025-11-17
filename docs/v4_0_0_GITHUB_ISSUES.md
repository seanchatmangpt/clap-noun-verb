# v4.0.0 Release - GitHub Issues Template

Copy-paste these into GitHub Issues to track P0 blockers.

---

## Issue 1: Fix Cargo.toml Lint Violations (P0 BLOCKER)

**Title**: [P0] Fix 50+ unwrap/expect/panic violations breaking lint policy

**Labels**: `P0-blocker`, `code-quality`, `v4.0.0`

**Description**:

### Problem
Cargo.toml declares strict deny-level lints for `unwrap_used`, `expect_used`, and `panic`, but the codebase contains 50+ violations, causing builds to fail with `-D warnings`.

### Impact
- Builds fail in CI/CD with `-D warnings`
- Safety guarantees not enforced
- Blocks v4.0.0 release

### Affected Files
20+ files including:
- `src/config.rs` (8 violations in tests)
- `src/context.rs` (15 violations in tests)
- `src/kernel/clnrm.rs` (6 violations)
- `src/io/async_io.rs` (12 violations)
- See `docs/v4_0_0_VALIDATION_REPORT.md` for full list

### Proposed Fix
Add `#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` to test modules:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    // Tests are allowed to panic
    #[test]
    fn test_something() {
        let value = something.unwrap();
    }
}
```

### Acceptance Criteria
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All test code has appropriate `#[allow]` attributes
- [ ] No production code uses unwrap/expect/panic
- [ ] CI updated to enforce this check

### Estimated Effort
8-12 hours

### Related
- Validation report: `docs/v4_0_0_VALIDATION_REPORT.md`
- Action items: `docs/v4_0_0_ACTION_ITEMS.md`

---

## Issue 2: Fix Example Compilation Failures (P0 BLOCKER)

**Title**: [P0] Fix io_advanced.rs and autonomic_example.rs compilation errors

**Labels**: `P0-blocker`, `examples`, `v4.0.0`

**Description**:

### Problem
2 out of 18 examples fail to compile, blocking release:

1. **io_advanced.rs**: `error[E0382]: borrow of moved value: 'inputs'`
2. **autonomic_example.rs**: Missing `LoggingMiddleware` and `ReadOnlyFS` types

### Impact
- Documentation examples don't work
- Users copy non-compiling code
- Reduces trust in v4.0.0
- Blocks cargo package publication

### Reproduction
```bash
cargo build --example io_advanced
cargo build --example autonomic_example
```

### Proposed Fix
**Option 1**: Fix the ownership/import issues
**Option 2**: Remove incomplete examples temporarily
**Option 3**: Mark as work-in-progress with TODO comments

### Acceptance Criteria
- [ ] All 18 examples compile successfully
- [ ] `cargo build --examples --all-features` passes
- [ ] Examples run without errors
- [ ] CI check added to prevent regressions

### Estimated Effort
2-3 hours

### Related
- Validation report: `docs/v4_0_0_VALIDATION_REPORT.md`

---

## Issue 3: Fix Doc Test Failures (P0 BLOCKER)

**Title**: [P0] Fix 3 failing documentation tests in kernel modules

**Labels**: `P0-blocker`, `documentation`, `v4.0.0`

**Description**:

### Problem
3 documentation tests fail, indicating broken API examples in docs:

1. `src/kernel/simd.rs - kernel::simd::FrameSerializer (line 84)`
2. `src/kernel/const_caps.rs - kernel::const_caps::require_agent_safe (line 255)`
3. `src/kernel/typestate.rs - kernel::typestate::TypedSession (line 43)`

### Impact
- API documentation contains non-working examples
- docs.rs will show failures
- Users copy broken code from documentation
- Reduces credibility

### Reproduction
```bash
cargo test --doc
```

Expected output should show failures.

### Proposed Fix
**Option 1**: Fix examples to compile and work
```rust
/// ```
/// use clap_noun_verb::kernel::simd::FrameSerializer;
/// // Add missing imports and context
/// ```
```

**Option 2**: Mark as conceptual (if not meant to compile)
```rust
/// ```no_run
/// // Conceptual example
/// ```
```

**Option 3**: Temporarily ignore
```rust
/// ```ignore
/// // TODO: Fix this example
/// ```
```

### Acceptance Criteria
- [ ] `cargo test --doc` passes with 23/23 tests
- [ ] All doc examples compile or are properly marked
- [ ] Documentation is accurate and helpful
- [ ] CI check added for doc tests

### Estimated Effort
3-4 hours

### Related
- Validation report: `docs/v4_0_0_VALIDATION_REPORT.md`

---

## Issue 4: Fix Vec<String> Parsing Bug in Proc Macro (P1 HIGH)

**Title**: [P1] Proc macro fails to parse Vec<String> parameters

**Labels**: `P1-high`, `bug`, `proc-macro`, `v4.0.0`

**Description**:

### Problem
The `#[verb]` proc macro fails to parse `Vec<String>` function parameters despite documentation claiming support.

**Error**:
```
error: expected one of `!`, `(`, `+`, `::`, `;`, `<`, or `=`, found `:`
  --> src/cli/core.rs:87:10
   |
85 | #[verb]
86 | fn exec(
87 |     names: Vec<String>,
   |          ^ expected one of 7 possible tokens
```

### Impact
- Documented feature doesn't work
- Users must use workaround (manual string splitting)
- Type safety lost
- Documentation mismatch

### Workaround
Currently users must do:
```rust
#[verb]
fn exec(names: String) -> Result<()> {
    let names: Vec<String> = names.split_whitespace().map(String::from).collect();
}
```

### Proposed Fix
Enhance the proc macro parser in `clap-noun-verb-macros` to handle generic types like `Vec<T>`.

### Detailed Analysis
See `docs/VEC_STRING_PARSING_ISSUE.md` for full root cause analysis and fix recommendations.

### Acceptance Criteria
- [ ] `Vec<String>` parameters work in `#[verb]` functions
- [ ] `Vec<T>` for other types works (PathBuf, u32, etc.)
- [ ] Tests added for Vec parameter parsing
- [ ] Documentation updated if needed

### Estimated Effort
6-8 hours

### Related
- Full analysis: `docs/VEC_STRING_PARSING_ISSUE.md`
- Validation report: `docs/v4_0_0_VALIDATION_REPORT.md`

---

## Issue 5: Remove Dead Code in io_detection Module (P1 HIGH)

**Title**: [P1] Remove or complete unused io_detection module (10 warnings)

**Labels**: `P1-high`, `code-quality`, `dead-code`, `v4.0.0`

**Description**:

### Problem
Entire `clap-noun-verb-macros/src/io_detection.rs` module is unused, generating 10 compiler warnings:

- Unused imports: `DetectedIoType`, `IoArgConfig`, `detect_io_type`
- Unused enum: `DetectedIoType`
- Unused functions: 5 functions
- Unused struct: `IoArgConfig`
- Unused methods: 5 methods

### Impact
- Code bloat
- Maintenance burden
- Confusing for contributors
- Suggests incomplete v4.0 I/O integration feature

### Proposed Fix
**Option 1** (Recommended if unused): Remove module
```bash
rm clap-noun-verb-macros/src/io_detection.rs
# Remove from lib.rs imports
```

**Option 2** (If needed for v4.0): Complete the feature and integrate it

**Option 3** (Temporary): Add `#[allow(dead_code)]` if work in progress

### Acceptance Criteria
- [ ] No dead code warnings in macros crate
- [ ] `cargo build -p clap-noun-verb-macros` clean
- [ ] Decision documented (removed vs. to be completed)

### Estimated Effort
- Removal: 1-2 hours
- Completion: 8-12 hours (if feature needed)

### Related
- Validation report: `docs/v4_0_0_VALIDATION_REPORT.md`

---

## Issue 6: Document Kani Formal Verification Configuration (P1 HIGH)

**Title**: [P1] Add Kani cfg documentation and suppress warnings

**Labels**: `P1-high`, `documentation`, `build-config`, `v4.0.0`

**Description**:

### Problem
10+ warnings about unexpected `cfg` condition name `kani` (formal verification tool):

```
warning: unexpected `cfg` condition name: `kani`
```

### Impact
- Build output cluttered with warnings
- Contributors don't understand what Kani is
- Missing Cargo.toml configuration
- Formal verification tooling undocumented

### Proposed Fix
Add to `Cargo.toml`:
```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(kani)'] }
```

And/or add documentation:
```rust
//! # Formal Verification
//!
//! This crate uses Kani for formal verification.
//! Install: cargo install kani-verifier
//! Run: cargo kani
#![cfg_attr(kani, feature(kani))]
```

### Acceptance Criteria
- [ ] No warnings about unexpected `cfg(kani)`
- [ ] Kani usage documented in README or module docs
- [ ] Optional: Add CI job for Kani verification

### Estimated Effort
1 hour

### Related
- Validation report: `docs/v4_0_0_VALIDATION_REPORT.md`
- Kani: https://github.com/model-checking/kani

---

## Quick Copy-Paste Commands for Issue Creation

```bash
# Create all issues at once (GitHub CLI required)
gh issue create --title "[P0] Fix 50+ unwrap/expect/panic violations" --label "P0-blocker,code-quality,v4.0.0" --body-file issue1.md

gh issue create --title "[P0] Fix io_advanced.rs and autonomic_example.rs compilation errors" --label "P0-blocker,examples,v4.0.0" --body-file issue2.md

gh issue create --title "[P0] Fix 3 failing documentation tests" --label "P0-blocker,documentation,v4.0.0" --body-file issue3.md

gh issue create --title "[P1] Proc macro fails to parse Vec<String>" --label "P1-high,bug,proc-macro,v4.0.0" --body-file issue4.md

gh issue create --title "[P1] Remove or complete io_detection module" --label "P1-high,code-quality,dead-code,v4.0.0" --body-file issue5.md

gh issue create --title "[P1] Document Kani configuration" --label "P1-high,documentation,build-config,v4.0.0" --body-file issue6.md
```

---

## Labels to Create

```bash
gh label create "P0-blocker" --color "d73a4a" --description "Blocks v4.0.0 release"
gh label create "P1-high" --color "ff9800" --description "High priority for v4.0.0"
gh label create "P2-medium" --color "fbca04" --description "Medium priority"
gh label create "v4.0.0" --color "0052cc" --description "v4.0.0 release"
```

---

**Ready to track**: Copy these issue templates to GitHub Issues and start fixing!
