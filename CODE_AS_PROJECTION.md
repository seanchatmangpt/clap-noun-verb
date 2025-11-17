# Code-as-Projection Policy and Enforcement

## The Principle

In the graph-universe thesis, **code is not edited by humans—it is generated from the ontology**.

```
Ontology (Σ) → ggen (projection engine) → Code/Tests/CLI/Config
↑                                         ↓
Authoritative source              Read-only derived views
```

**You do not edit code. You edit the ontology (Σ). The code regenerates automatically.**

---

## Why Code-as-Projection?

### 1. Single Source of Truth
Without projection:
- Code changes are made directly
- Tests might be out of sync
- Documentation lags behind
- Configuration is separate from schema
- Conflicts multiply

With projection:
- One source of truth: the ontology (Σ)
- Code, tests, config all derive from Σ
- Consistency guaranteed by construction
- No divergence possible

### 2. Deterministic Regeneration
Given Σ, we can regenerate code at any time:
```bash
$ cargo run --bin ggen regenerate
# Reads src/autonomic/schema.rs
# Writes src/generated/*.rs, tests/generated/*.rs, docs/auto/*.md
# All outputs are deterministic: same Σ → same code
```

If code is lost, corrupted, or outdated, regeneration restores it.

### 3. Proof of Consistency
You can verify that committed code matches the ontology:
```bash
$ cargo run --bin ggen verify
# Regenerates code
# Compares with committed code
# Reports if any divergence exists
# Exit code 0: consistent, Exit code 1: divergence detected
```

---

## How to Edit Code the Right Way

### Scenario 1: Add a New Command

**Wrong**: Edit `src/commands/my_command.rs` directly
```rust
// ❌ Don't do this
pub fn my_command(args: Args) -> Result<()> {
    // ... your implementation
}
```

**Right**: Add to ontology, then regenerate

1. **Edit the ontology** (`src/autonomic/schema.rs`):
```rust
{
  "command_id": "storage.create",
  "noun": "storage",
  "verb": "create",
  "effect": "MutateState",
  "parameters": [
    { "name": "name", "type": "String", "required": true }
  ]
}
```

2. **Regenerate**:
```bash
cargo run --bin ggen regenerate
```

3. **Commit both**:
- `src/autonomic/schema.rs` (the edit)
- `src/generated/commands/storage_create.rs` (the generated code)
- `tests/generated/storage_create.rs` (generated tests)

### Scenario 2: Modify Command Behavior

**Wrong**: Hand-edit the generated code

**Right**: Change the schema, then regenerate

1. **Update schema**:
```rust
{
  "command_id": "storage.create",
  "effect": "MutateState",  // Changed from MutateConfig
  "parameters": [
    { "name": "name", "type": "String", "required": true },
    { "name": "ttl", "type": "Duration", "required": false }  // Added field
  ]
}
```

2. **Regenerate**:
```bash
cargo run --bin ggen regenerate
```

3. **Verify**:
```bash
cargo test
```

4. **Commit**:
```bash
git add src/autonomic/schema.rs src/generated/ tests/generated/
git commit -m "refactor: Add TTL parameter to storage.create command"
```

### Scenario 3: Fix a Bug in Generated Code

If you discover a bug in the generated code, **do not hand-fix it**. Instead:

1. **Identify the root cause**: Is it in the schema definition or the projection engine?

2. **If it's a schema issue**:
   - Fix `src/autonomic/schema.rs`
   - Regenerate

3. **If it's a ggen bug**:
   - File an issue in ggen repo
   - Workaround: patch the generated file with a comment explaining the ggen limitation
   - Mark with `@generated-but-patched` comment
   - Link to the ggen issue

Example workaround:
```rust
// @generated-but-patched
// Issue: ggen issue #123 - incorrectly infers type for large integers
// Workaround: manually adjusted type annotation
pub fn storage_create(args: Args) -> Result<()> {
    // @patch: type should be u128, not i64
    let size: u128 = args.size.parse()?;
    // ... rest of function
}
```

---

## Enforcement Mechanisms

### 1. Pre-Commit Hook

When you `git commit`, the hook checks:

```bash
$ git commit -m "fix: Update storage command"

Running pre-commit checks for code-as-projection policy...
Checking for hand-edit markers... ✓
Checking critical projection files... ✓
Checking generated test files... ✓
Checking schema consistency... ⚠ Code changed but schema.rs not modified
  Hint: If you changed command schemas, update src/autonomic/schema.rs

✗ Some checks failed
```

This prevents accidental commits that violate the policy.

**To install the hook**:
```bash
git config core.hooksPath .githooks
# Or manually: cp .githooks/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

### 2. GitHub Actions (CI)

When you push or create a PR, the `projection-verification.yml` workflow runs:

```bash
Check for suspicious hand-edit markers... ✓
Verify macro-generated code consistency... ✓
Document code generation policy... ✓
Verify deterministic build... ✓
```

If any check fails, the build is marked red (CI failure).

### 3. Static Analysis

Commit to marking generated sections:

```rust
// @generated-from: src/autonomic/schema.rs:42-67
// Command ID: storage.create
// DO NOT EDIT BY HAND

pub fn storage_create(args: Args) -> Result<()> {
    // ... auto-generated implementation
}

// @end-generated
```

The `@generated-from` annotation links code back to its source in the ontology.

---

## Annotations

### Code Sections

Use these annotations in generated code:

```rust
// @generated-from: src/autonomic/schema.rs:42-67
// Command ID: storage.create
// DO NOT EDIT BY HAND

pub fn my_function() {
    // ...
}

// @end-generated
```

### Files

Mark generated files at the top:

```rust
//! @generated-file
//! This file is generated from src/autonomic/schema.rs
//! DO NOT EDIT by hand. Run `cargo run --bin ggen regenerate` instead.
//!
//! Last generated: 2025-11-17T12:34:56Z
//! Schema version: 2.0.0
```

### Patched Sections

If you must patch generated code (workaround for ggen bug):

```rust
// @generated-but-patched
// Issue: https://github.com/seanchatmangpt/ggen/issues/123
// Workaround: manually fixed type inference
// TODO(ggen): remove this patch when ggen issue #123 is fixed

pub fn storage_create(args: Args) -> Result<()> {
    // @patch: type should be u128, not i64
    let size: u128 = args.size.parse()?;
    // ... rest
}
```

---

## Workflow: Editing Code Properly

### Step 1: Understand What You're Changing

Is it a change to:
- Command structure? → Edit `src/autonomic/schema.rs`
- Verification logic? → Edit `src/autonomic/contracts.rs`
- Governance policy? → Edit `src/kernel/ahi_policy.rs`
- Telemetry? → Edit `src/autonomic/telemetry.rs`

### Step 2: Make the Change in the Ontology

```bash
# 1. Edit the schema
$EDITOR src/autonomic/schema.rs

# 2. Verify schema is valid (if ggen has a validate command)
cargo run --bin ggen validate
```

### Step 3: Regenerate Code

```bash
# 3. Regenerate all derived code
cargo run --bin ggen regenerate

# This updates:
# - src/generated/*.rs (command implementations)
# - tests/generated/*.rs (test suites)
# - docs/auto/*.md (documentation)
# - examples/auto/*.rs (example code)
```

### Step 4: Run Tests

```bash
# 4. Verify everything still works
cargo test

# Run specific tests if needed
cargo test --test cnv4_advanced
```

### Step 5: Commit

```bash
# 5. Stage and commit
git add src/autonomic/schema.rs src/generated/ tests/generated/ docs/auto/
git commit -m "feat: Add 'ttl' parameter to storage.create command

- Updated schema.rs with new field definition
- Regenerated command implementations and tests
- All 191 tests pass"
```

---

## FAQ

**Q: Can I ever hand-edit code?**

A: Only in these cases:
1. **Non-generated files** (like `src/autonomic/schema.rs` itself)
2. **Emergency patches** marked with `@generated-but-patched` (with issue reference)
3. **Tests/examples** not in the generated folder (like custom integration tests)

**Q: What if the generated code is wrong?**

A: This indicates a ggen bug. Options:
1. Fix the schema to work around the bug
2. File an issue in ggen repo
3. Temporarily patch with `@generated-but-patched` comment

**Q: What if I disagree with a schema change?**

A: The schema is the authority. If you think it's wrong:
1. Discuss in a comment on the PR
2. If consensus is the schema needs revision, update it
3. Regenerate

There is no "hand-edit override."

**Q: How do I test changes locally before committing?**

A:

```bash
# 1. Edit the schema
$EDITOR src/autonomic/schema.rs

# 2. Regenerate
cargo run --bin ggen regenerate

# 3. Test
cargo test

# 4. Review changes
git diff src/generated/ tests/generated/

# 5. If satisfied, commit
git add . && git commit -m "..."
```

---

## The Principle: A = μ(O)

Remember: **Application = μ-kernel(Ontology)**

```
Ontology (Σ)    Schema, types, contracts, policies
     ↓
  ggen           Projection engine: deterministic code generation
     ↓
  Code (A)       Command implementations, tests, configs
     ↓
  μ-kernel       Executes with timing bounds and receipts
     ↓
  Results        Observable behavior, audit trail
```

If you edit the code (A) directly, you break the relationship. Δ = μ(Σ), meaning changes must flow through the chain.

By editing Σ and regenerating, you maintain the mathematical invariant.

---

## References

- **PHILOSOPHY.md** — Why the graph-universe thesis requires code-as-projection
- **MU_KERNEL.md** — How μ(O) = A is executed
- **src/autonomic/schema.rs** — The ontology (Σ) you should edit
- **.githooks/pre-commit** — Enforcement hook
- **.github/workflows/projection-verification.yml** — CI enforcement
