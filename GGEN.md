# ggen: General Projection Engine (Σ + Q → Code)

## What is ggen?

**ggen** (graph generator) is the **projection engine** that implements the core thesis of the graph-universe:

```
Σ (Ontology) + Q (Invariants) → ggen → A (Application)
    Code, Tests, CLI, Configs
```

**ggen takes:**
- **Σ** (Ontology): Schema definitions, types, capabilities, policies
- **Q** (Invariants): Constraints, guards, verification rules

**ggen produces:**
- **Code** (Rust implementations of capabilities)
- **Tests** (Comprehensive test suites for generated code)
- **CLI** (Command definitions and help text)
- **Configs** (TOML/YAML configuration templates)
- **Docs** (Auto-generated documentation)

**Why?** Because A = μ(Σ) — the application is derived from the ontology by the μ-kernel. ggen is how we make that relationship concrete.

---

## The Problem: Hand-Written Code

Traditional approach:
```
Engineer ← writes code → repository
  ↓
Code is source of truth
Code changes → must update tests, docs, CLI separately
  ↓
Inconsistency: code drifts from reality
```

Issues:
- Code and tests get out of sync
- Documentation lags behind implementation
- CLI help is incomplete or outdated
- No single source of truth
- Breaking changes aren't caught early

## The Solution: Projection-Based Code Generation

ggen approach:
```
Schema (Σ) ← single source of truth
  ↓
ggen processes schema
  ├─ Generates code/ (implementations)
  ├─ Generates tests/ (test suites)
  ├─ Generates docs/ (auto-generated docs)
  └─ Generates examples/ (working examples)
  ↓
All outputs consistent by construction
All derived from single source
All updated together when schema changes
```

Benefits:
- **One source of truth**: Schema is the authority
- **Consistency**: Code, tests, docs always match
- **Automation**: No manual synchronization needed
- **Reversibility**: Delete generated files, regenerate from schema
- **Safety**: Breaking changes caught by compiler

---

## Architecture

### Input: Σ (Ontology) + Q (Invariants)

**From KNHK:**

```rust
pub struct Ontology {
    pub capabilities: Vec<Capability>,
    pub types: Vec<Type>,
    pub policies: Vec<Policy>,
    pub version: String,
}

pub struct Capability {
    pub id: String,           // "storage.create"
    pub noun: String,         // "storage"
    pub verb: String,         // "create"
    pub signature: Signature, // input/output types
    pub effects: Vec<Effect>, // MutateState, ReadOnly, etc.
    pub guards: Vec<Guard>,   // constraints
}

pub struct Signature {
    pub inputs: Vec<Parameter>,
    pub output: Type,
}

pub struct Parameter {
    pub name: String,
    pub type_: Type,
    pub required: bool,
    pub default: Option<String>,
}
```

**From Contracts:**

```rust
pub struct Invariants {
    pub timing_bounds: TimingBound,  // τ <= 8 ticks
    pub memory_limits: MemoryLimit,  // max allocation
    pub quota_rules: Vec<QuotaRule>,
    pub safety_checks: Vec<SafetyCheck>,
}

pub struct TimingBound {
    pub max_ns: u64,        // 100 nanoseconds
    pub percentile: f64,    // p99 constraint
}
```

### Processing: Projection Profiles

ggen uses **projection profiles** to customize output:

```rust
pub struct ProjectionProfile {
    pub name: String,  // "rust-native", "wasm", "cloud", etc.

    // Code generation
    pub code_lang: String,           // "rust", "go", "python"
    pub async_support: bool,         // Generate async versions?
    pub error_handling: ErrorStyle,  // Result<T>, Option<T>, exceptions

    // Testing
    pub test_frameworks: Vec<String>,  // ["criterion", "proptest"]
    pub test_coverage: f64,            // 0.95 = 95% coverage target

    // Documentation
    pub doc_format: String,  // "markdown", "asciidoc"
    pub examples_per_capability: usize,  // How many examples to generate

    // Other
    pub with_validation: bool,       // Add input validation code?
    pub with_metrics: bool,          // Add telemetry/metrics?
    pub optimization_level: String,  // "debug", "balanced", "aggressive"
}
```

### Output: Generated Code

For a single capability definition in Σ:

```rust
// Input (schema.rs):
{
  "command_id": "storage.create",
  "noun": "storage",
  "verb": "create",
  "signature": {
    "inputs": [
      {"name": "key", "type": "String"},
      {"name": "value", "type": "Bytes"}
    ],
    "output": "StorageRef"
  },
  "effects": ["MutateState"],
  "guards": [
    {"type": "WriteQuota", "limit": "1MB"}
  ]
}
```

**Output (generated/storage_create.rs):**

```rust
// @generated-file
// Generated from: src/autonomic/schema.rs:42
// Last generated: 2025-11-17T12:34:56Z
//! storage.create capability
//!
//! # Description
//! Creates a new storage entry
//!
//! # Effect
//! Mutates system state
//!
//! # Example
//! ```rust
//! let result = storage_create("my-key", b"my-value")?;
//! assert!(result.exists());
//! ```

use crate::kernel::{CapabilityContext, GuardEnforcer};
use crate::types::{StorageRef, GuardError};

/// Create a new storage entry
///
/// # Parameters
/// - `key`: Storage key (required)
/// - `value`: Storage value (required)
///
/// # Returns
/// - `Ok(StorageRef)` on success
/// - `Err(GuardError)` if quota/permission violated
///
/// # Timing Guarantee
/// Latency <= 100ns (p99)
///
/// # Safety
/// - All inputs are validated
/// - Quota enforcement is automatic
/// - Operation is deterministic
pub fn storage_create(
    context: &CapabilityContext,
    key: String,
    value: Vec<u8>,
) -> Result<StorageRef, GuardError> {
    // @generated
    // Validate inputs
    validate_key(&key)?;
    validate_value(&value)?;

    // Enforce guards (write quota)
    context.check_quota("write", value.len())?;

    // Record effect
    context.record_effect("MutateState");

    // Implementation (from schema)
    let ref_id = uuid::Uuid::new_v4().to_string();
    let storage_ref = StorageRef {
        id: ref_id,
        key: key.clone(),
        created_at: std::time::SystemTime::now(),
    };

    // Record receipt (audit trail)
    context.record_receipt(&storage_ref)?;

    Ok(storage_ref)
    // @end-generated
}

// Private helpers (generated)
#[inline]
fn validate_key(key: &str) -> Result<(), GuardError> {
    if key.is_empty() {
        return Err(GuardError::InvalidInput("key cannot be empty".into()));
    }
    if key.len() > 256 {
        return Err(GuardError::InvalidInput("key too long".into()));
    }
    Ok(())
}

#[inline]
fn validate_value(value: &[u8]) -> Result<(), GuardError> {
    if value.len() > 1_000_000 {
        return Err(GuardError::InvalidInput("value exceeds 1MB limit".into()));
    }
    Ok(())
}
```

**Output (generated/tests/storage_create_test.rs):**

```rust
// @generated-file
// Property-based tests generated by chicago-tdd-tools
// Last generated: 2025-11-17T12:34:56Z

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn storage_create_basic() {
        let context = test_context();
        let result = storage_create(&context, "key".into(), b"value".to_vec());
        assert!(result.is_ok());
    }

    // Property: storage.create is deterministic
    proptest! {
        #[test]
        fn prop_storage_create_deterministic(
            key in ".*",
            value in prop::collection::vec(0u8.., 0..1000),
        ) {
            let context = test_context();
            let result1 = storage_create(&context, key.clone(), value.clone());
            let result2 = storage_create(&context, key.clone(), value.clone());
            // Same input should give results with same hash
            match (result1, result2) {
                (Ok(ref1), Ok(ref2)) => {
                    // Both succeeded; hashes should match
                    prop_assert_eq!(hash(&ref1), hash(&ref2));
                }
                (Err(e1), Err(e2)) => {
                    // Both failed; errors should be same type
                    prop_assert_eq!(std::mem::discriminant(&e1), std::mem::discriminant(&e2));
                }
                _ => prop_assert!(false, "Inconsistent results"),
            }
        }
    }

    // Property: quota enforcement
    proptest! {
        #[test]
        fn prop_storage_create_quota_enforced(
            value_size in 0usize..=1_100_000,
        ) {
            let context = test_context_with_quota(1_000_000);
            let result = storage_create(&context, "key".into(), vec![0; value_size]);

            if value_size > 1_000_000 {
                prop_assert!(result.is_err(), "Should reject oversized value");
            } else {
                prop_assert!(result.is_ok(), "Should accept value within quota");
            }
        }
    }
}
```

---

## ggen Processing Pipeline

```
1. Load Ontology (Σ)
   ├─ Parse KNHK graph
   ├─ Extract capabilities, types, policies
   └─ Validate completeness

2. Load Invariants (Q)
   ├─ Parse contracts
   ├─ Extract timing bounds, quotas, guards
   └─ Validate consistency with Σ

3. Select Projection Profile
   ├─ Choose target language (Rust, Go, Python)
   ├─ Choose test framework (criterion, proptest)
   └─ Choose documentation style

4. Validate Σ + Q
   ├─ Check no missing types
   ├─ Check all capabilities have timing bounds
   ├─ Check all guards have implementations
   └─ Check invariants don't conflict

5. Generate Code
   ├─ For each capability:
   │  ├─ Generate implementation
   │  ├─ Add input validation
   │  ├─ Add quota enforcement
   │  ├─ Add receipt recording
   │  └─ Add documentation
   └─ Generate module structure

6. Generate Tests
   ├─ For each capability:
   │  ├─ Generate unit tests
   │  ├─ Generate property-based tests
   │  ├─ Generate edge case tests
   │  └─ Generate timing benchmark
   └─ Generate integration tests

7. Generate Documentation
   ├─ API docs from capability signatures
   ├─ Example code for each capability
   ├─ Constraint documentation
   └─ CLI help text

8. Emit Output
   ├─ Write src/generated/ (code)
   ├─ Write tests/generated/ (tests)
   ├─ Write docs/generated/ (docs)
   └─ Write examples/generated/ (examples)

9. Verify
   ├─ Compile generated code
   ├─ Run generated tests
   ├─ Check code coverage
   └─ Report any generation errors
```

---

## Projection Profiles: Multiple Targets

### Profile: "rust-native"
```rust
ProjectionProfile {
    code_lang: "rust",
    async_support: true,
    error_handling: Result,
    test_frameworks: ["criterion", "proptest"],
    test_coverage: 0.95,
    doc_format: "markdown",
    examples_per_capability: 3,
    with_validation: true,
    with_metrics: true,
    optimization_level: "balanced",
}
```

Output: Native Rust code, property-based tests, comprehensive docs

### Profile: "cloud-functions"
```rust
ProjectionProfile {
    code_lang: "rust",
    async_support: true,
    error_handling: JsonError,
    test_frameworks: ["tokio", "proptest"],
    test_coverage: 0.90,
    doc_format: "markdown",
    examples_per_capability: 1,
    with_validation: true,
    with_metrics: true,
    optimization_level: "aggressive",
}
```

Output: Cloud-ready async code, HTTP error responses, minimal docs

### Profile: "python-stub"
```rust
ProjectionProfile {
    code_lang: "python",
    async_support: false,
    error_handling: Exceptions,
    test_frameworks: ["pytest"],
    test_coverage: 0.80,
    doc_format: "rst",
    examples_per_capability: 2,
    with_validation: true,
    with_metrics: false,  // Python doesn't need telemetry
    optimization_level: "debug",
}
```

Output: Python stubs for integration, pytest tests

---

## Template System

ggen uses **templates** to customize code generation:

```rust
pub struct Template {
    pub name: String,        // "capability_impl", "test_suite", etc.
    pub format: TemplateFormat,  // Handlebars, Askama, etc.
    pub source: String,      // Template file path
    pub variables: Vec<String>,  // Variables available to template
}
```

Example template (`templates/capability_impl.rs.hbs`):

```handlebars
// @generated-file
//! {{capability.noun}}.{{capability.verb}} capability
//!
//! # Description
//! {{capability.description}}

{{#each imports}}
use {{this}};
{{/each}}

/// {{capability.description}}
{{#each capability.parameters}}
/// # Parameters
/// - `{{name}}`: {{type}}
{{/each}}
pub fn {{capability.noun}}_{{capability.verb}}(
    context: &CapabilityContext,
    {{#each capability.parameters}}
    {{name}}: {{type}},
    {{/each}}
) -> Result<{{capability.output}}, GuardError> {
    // Validate inputs
    {{#each capability.parameters}}
    validate_{{name}}(&{{name}})?;
    {{/each}}

    // Enforce guards
    {{#each capability.guards}}
    context.check_{{guard_type}}("{{guard_name}}", {{guard_params}})?;
    {{/each}}

    // Implementation
    // TODO: Implement capability logic

    // Record receipt
    context.record_receipt(&result)?;

    Ok(result)
}
```

---

## Integration with Code-as-Projection

ggen enforces CODE_AS_PROJECTION policy:

1. **Mark generated code** with `@generated-file` and `@generated` annotations
2. **Link to source**: `// Generated from: src/autonomic/schema.rs:42`
3. **Timestamp**: `// Last generated: 2025-11-17T12:34:56Z`
4. **Regeneration**:
   ```bash
   $ cargo run --bin ggen regenerate
   # Reads schema from src/autonomic/schema.rs
   # Overwrites src/generated/*
   # All changes are from Σ, not hand-edits
   ```

---

## Implementation Roadmap

### Phase 1: Core Engine (2-3 weeks)
- [ ] Ontology parser
  - [ ] KNHK schema loader
  - [ ] Type resolution
  - [ ] Capability extraction

- [ ] Code generator foundation
  - [ ] Template engine integration
  - [ ] Basic Rust code generation
  - [ ] Module structure generation

### Phase 2: Complete Language Support (2-3 weeks)
- [ ] Rust generation (full featured)
  - [ ] Input validation generation
  - [ ] Guard enforcement
  - [ ] Receipt recording

- [ ] Test generation
  - [ ] Unit test generation
  - [ ] Property-based test generation (proptest)
  - [ ] Benchmark generation (criterion)

### Phase 3: Projection Profiles (2 weeks)
- [ ] Profile system
  - [ ] Built-in profiles (rust-native, cloud, python)
  - [ ] Profile serialization

- [ ] Multi-language backends
  - [ ] Python stub generation
  - [ ] Go generation
  - [ ] TypeScript/JavaScript generation

### Phase 4: Documentation & Examples (1-2 weeks)
- [ ] Auto-doc generation
  - [ ] Capability documentation
  - [ ] Type documentation
  - [ ] Constraint documentation

- [ ] Example generation
  - [ ] Working examples for each capability
  - [ ] Integration examples

### Phase 5: Verification & Integration (2 weeks)
- [ ] Verification
  - [ ] Compile check
  - [ ] Test execution
  - [ ] Code coverage reporting

- [ ] Integration with build system
  - [ ] Cargo build hooks
  - [ ] CI/CD integration
  - [ ] Incremental generation

### Phase 6: Optimization (1-2 weeks)
- [ ] Performance
  - [ ] Generation speed optimization
  - [ ] Output size optimization
  - [ ] Compile time reduction

- [ ] Advanced features
  - [ ] Custom generators
  - [ ] Plugin system
  - [ ] Custom templates

---

## Extensibility: Custom Generators

Users can create custom generators:

```rust
pub trait ProjectionGenerator {
    fn name(&self) -> &str;
    fn generate_capability(&self, cap: &Capability) -> Result<String>;
    fn generate_test(&self, cap: &Capability) -> Result<String>;
    fn generate_docs(&self, cap: &Capability) -> Result<String>;
}

// Custom generator for domain-specific language
pub struct DslGenerator;
impl ProjectionGenerator for DslGenerator {
    fn name(&self) -> &str { "dsl-gen" }

    fn generate_capability(&self, cap: &Capability) -> Result<String> {
        // Custom code generation logic
        Ok(format!("command {} {{\n  // ...\n}}", cap.id))
    }
    // ...
}
```

---

## Testing ggen

ggen itself is tested with golden files:

```
tests/
├── golden/
│  ├── input/
│  │  ├── ontology_simple.json   // Simple test schema
│  │  ├── ontology_complex.json  // Complex schema
│  │  └── invariants.json        // Test invariants
│  └── expected_output/
│     ├── rust-native/
│     │  ├── storage_create.rs
│     │  └── test_storage_create.rs
│     ├── python-stub/
│     │  └── storage_create.py
│     └── docs/
│        └── storage_create.md
```

Test process:
```
1. Load test ontology
2. Run ggen with profile
3. Compare output against golden files
4. If different: either golden files outdated OR ggen bug
5. Update goldens (with human review) or fix bug
```

---

## Future: Bidirectional Mapping

Long-term, ggen could support **reverse projection**:

```
Code → ggen.invert() → Updated Σ

Scenario: Someone hand-edits code (against policy), then:
$ cargo run --bin ggen invert
# Analyzes hand-edited code
# Generates new schema (Σ') that would produce it
# Shows diff: Σ → Σ'
# User decides: accept (update schema) or discard (regenerate from Σ)
```

This allows recovery from accidental hand-edits or reasoning about what changed.

---

## Summary: ggen as Core to the Thesis

ggen is the **proof** of the thesis **A = μ(O)**:

- **O** (Ontology) is primary: defined in KNHK
- **ggen** is the machine that applies μ-kernel semantics
- **A** (Application) is the output: code, tests, docs
- **Verification**: compile and test generated code to prove consistency

Without ggen, the thesis is philosophical. With ggen, it's demonstrated in every build.

---

## References

- **PHILOSOPHY.md** — Why A = μ(O) and why ggen is needed
- **CODE_AS_PROJECTION.md** — How ggen enforces projection model
- **KNHK.md** — Ontology that ggen consumes
- **Cargo build hooks** — How ggen integrates with Rust build system
- **Chicago TDD Tools** — How generated tests are validated
