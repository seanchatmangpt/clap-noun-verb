# ggen-clap-noun-verb: Quick Start Implementation Guide

**Version**: 1.0.0
**Date**: 2026-01-06

---

## Overview

This guide provides step-by-step instructions to implement the ggen-clap-noun-verb architecture. Follow the phases sequentially, using Chicago TDD and cargo make for all operations.

---

## Prerequisites

1. Rust 1.74+ installed
2. cargo-make installed: `cargo install cargo-make`
3. ggen repository cloned to `vendors/ggen`
4. clap-noun-verb project initialized

---

## Phase 1: Foundation (Week 1-2)

### Step 1.1: Create Project Structure

```bash
# Create crate directory
mkdir -p /home/user/clap-noun-verb/crates/ggen-clap-noun-verb

# Create module structure
cd /home/user/clap-noun-verb/crates/ggen-clap-noun-verb
mkdir -p src/{parser,ast,validator,generator,templates,integration}
mkdir -p tests/{unit,integration}
mkdir -p templates
```

### Step 1.2: Initialize Cargo.toml

```toml
[package]
name = "ggen-clap-noun-verb"
version = "0.1.0"
edition = "2021"
rust-version = "1.74"

[dependencies]
# Core dependencies
ggen-core = { path = "../../vendors/ggen/crates/ggen-core" }
clap-noun-verb = { path = "../.." }

# RDF processing
oxigraph = "0.5.1"

# Template engine (via ggen)
tera = "1.20"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
once_cell = "1.19"

[dev-dependencies]
# Testing
proptest = "1.0"
insta = { version = "1.0", features = ["json", "yaml"] }
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.8"

[features]
default = []
full = []
```

### Step 1.3: Implement Core AST Types

Create `src/ast/state.rs`:

```rust
//! State markers for typestate pattern

/// Unvalidated state - parsed but not validated
pub struct Unvalidated;

/// Validated state - all invariants checked
pub struct Validated;

/// State trait - sealed to prevent external implementations
pub trait State: sealed::Sealed {}

impl State for Unvalidated {}
impl State for Validated {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Unvalidated {}
    impl Sealed for super::Validated {}
}
```

Create `src/ast/types.rs`:

```rust
//! Newtype wrappers for type safety

use std::fmt;

/// Non-empty string - makes empty strings unrepresentable
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: String) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(NonEmptyString(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Noun identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NounId(u32);

/// Verb identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VerbId(u32);

/// Type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);
```

Create `src/ast/spec.rs`:

```rust
//! Core specification types with typestate pattern

use std::marker::PhantomData;
use std::collections::HashMap;

use super::state::{State, Unvalidated, Validated};
use super::types::{NonEmptyString, NounId, VerbId, TypeId};

/// CLI specification - top-level container
#[derive(Debug, Clone)]
pub struct CliSpec<S: State = Validated> {
    pub project_name: NonEmptyString,
    pub nouns: Vec<NounSpec<S>>,
    pub shared_types: TypeRegistry<S>,
    _state: PhantomData<S>,
}

/// Noun specification
#[derive(Debug, Clone)]
pub struct NounSpec<S: State = Validated> {
    pub id: NounId,
    pub name: NonEmptyString,
    pub description: Option<String>,
    pub verbs: Vec<VerbId>,
    pub aliases: Vec<NonEmptyString>,
    _state: PhantomData<S>,
}

/// Verb specification
#[derive(Debug, Clone)]
pub struct VerbSpec<S: State = Validated> {
    pub id: VerbId,
    pub name: NonEmptyString,
    pub noun_id: NounId,
    pub description: Option<String>,
    pub arguments: Vec<ArgumentSpec<S>>,
    pub async_handler: bool,
    _state: PhantomData<S>,
}

/// Argument specification
#[derive(Debug, Clone)]
pub struct ArgumentSpec<S: State = Validated> {
    pub name: NonEmptyString,
    pub arg_type: ArgumentType,
    pub required: bool,
    pub default: Option<String>,
    pub validator: Option<ValidatorSpec>,
    _state: PhantomData<S>,
}

/// Argument type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentType {
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Url,
    Custom(TypeId),
}

/// Validator specification
#[derive(Debug, Clone)]
pub enum ValidatorSpec {
    Regex(String),
    Range { min: i64, max: i64 },
    Length { min: usize, max: usize },
    OneOf(Vec<String>),
    Custom(String),
}

/// Type registry
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry<S: State = Validated> {
    types: HashMap<TypeId, TypeDefinition>,
    _state: PhantomData<S>,
}

/// Type definition
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub id: TypeId,
    pub name: NonEmptyString,
    pub base_type: RustType,
    pub validator: Option<ValidatorSpec>,
}

/// Rust type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustType {
    String,
    I32,
    I64,
    F64,
    Bool,
    PathBuf,
    Custom(String),
}
```

### Step 1.4: Implement Error Types

Create `src/error.rs`:

```rust
//! Error types for ggen-clap-noun-verb

use thiserror::Error;
use std::path::PathBuf;

/// Top-level error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parsing error: {0}")]
    Parse(#[from] ParseError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Code generation error: {0}")]
    Generation(#[from] GenerationError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template error: {0}")]
    Template(#[from] tera::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Parsing errors
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid Turtle syntax at line {line}: {message}")]
    InvalidTurtle { line: usize, message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid RDF structure: {0}")]
    InvalidRdf(String),

    #[error("SPARQL query failed: {0}")]
    SparqlError(String),
}

/// Validation errors
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Duplicate noun name: {name}")]
    DuplicateNoun { name: String },

    #[error("Duplicate verb name: {name} in noun {noun}")]
    DuplicateVerb { name: String, noun: String },

    #[error("Unresolved reference: {ref_type} {ref_name}")]
    UnresolvedReference { ref_type: String, ref_name: String },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Invalid argument type: {0}")]
    InvalidArgumentType(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
}

/// Generation errors
#[derive(Error, Debug)]
pub enum GenerationError {
    #[error("Template rendering failed: {0}")]
    TemplateRenderFailed(String),

    #[error("File write failed: {path}")]
    FileWriteFailed { path: PathBuf },

    #[error("Invalid output path: {path}")]
    InvalidOutputPath { path: PathBuf },

    #[error("Code formatting failed: {0}")]
    FormattingFailed(String),
}
```

### Step 1.5: Write Initial Tests

Create `tests/unit/ast_test.rs`:

```rust
//! Unit tests for AST types (Chicago TDD)

use ggen_clap_noun_verb::ast::*;

#[test]
fn test_non_empty_string_creation() {
    // ARRANGE & ACT
    let valid = NonEmptyString::new("test".into());
    let empty = NonEmptyString::new("".into());

    // ASSERT
    assert!(valid.is_some());
    assert_eq!(valid.unwrap().as_str(), "test");
    assert!(empty.is_none());
}

#[test]
fn test_cli_spec_construction() {
    // ARRANGE
    let project_name = NonEmptyString::new("my-cli".into()).unwrap();
    let nouns = vec![];
    let shared_types = TypeRegistry::default();

    // ACT
    let spec = CliSpec::<Unvalidated> {
        project_name,
        nouns,
        shared_types,
        _state: PhantomData,
    };

    // ASSERT
    assert_eq!(spec.project_name.as_str(), "my-cli");
    assert_eq!(spec.nouns.len(), 0);
}

#[test]
fn test_typestate_size_equality() {
    // ASSERT: Typestate pattern is zero-cost
    assert_eq!(
        std::mem::size_of::<CliSpec<Unvalidated>>(),
        std::mem::size_of::<CliSpec<Validated>>()
    );
}
```

### Step 1.6: Run Tests

```bash
# Check compilation
cargo make check

# Run tests
cargo make test-unit

# Run linting
cargo make lint
```

---

## Phase 2: Validation (Week 3)

### Step 2.1: Implement Validation Traits

Create `src/validator/traits.rs`:

```rust
//! Validation traits

use crate::error::{Result, ValidationError};

/// Validator trait
pub trait Validator<T> {
    fn validate(&self, input: &T) -> Result<()>;
}

/// Validate trait - typestate transition
pub trait Validate {
    type Error;
    fn validate(self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
```

### Step 2.2: Implement Semantic Validators

Create `src/validator/uniqueness.rs`:

```rust
//! Uniqueness validation

use std::collections::HashSet;
use crate::ast::*;
use crate::error::{Result, ValidationError};
use super::traits::Validator;

/// Uniqueness validator - ensures noun/verb names are unique
pub struct UniquenessValidator;

impl Validator<CliSpec<Unvalidated>> for UniquenessValidator {
    fn validate(&self, spec: &CliSpec<Unvalidated>) -> Result<()> {
        // Check noun name uniqueness
        let mut noun_names = HashSet::new();
        for noun in &spec.nouns {
            if !noun_names.insert(noun.name.as_str()) {
                return Err(ValidationError::DuplicateNoun {
                    name: noun.name.as_str().to_string(),
                }.into());
            }
        }

        // Check verb name uniqueness within each noun
        for noun in &spec.nouns {
            let mut verb_names = HashSet::new();
            for verb_id in &noun.verbs {
                // Would lookup verb by ID and check name
                // (implementation depends on full spec structure)
            }
        }

        Ok(())
    }
}
```

### Step 2.3: Implement Typestate Transition

Create `src/validator/transition.rs`:

```rust
//! Typestate transition implementation

use std::marker::PhantomData;
use crate::ast::*;
use crate::error::{Result, ValidationError};
use super::traits::{Validate, Validator};
use super::uniqueness::UniquenessValidator;

impl Validate for CliSpec<Unvalidated> {
    type Error = crate::error::Error;

    fn validate(self) -> Result<CliSpec<Validated>> {
        // Run all validators
        let uniqueness = UniquenessValidator;
        uniqueness.validate(&self)?;

        // Validate nested components
        let validated_nouns = self.nouns.into_iter()
            .map(|n| n.validate())
            .collect::<Result<Vec<_>>>()?;

        let validated_types = self.shared_types.validate()?;

        // Transition to Validated state
        Ok(CliSpec {
            project_name: self.project_name,
            nouns: validated_nouns,
            shared_types: validated_types,
            _state: PhantomData,
        })
    }
}

impl Validate for NounSpec<Unvalidated> {
    type Error = crate::error::Error;

    fn validate(self) -> Result<NounSpec<Validated>> {
        // Validation logic for noun
        Ok(NounSpec {
            id: self.id,
            name: self.name,
            description: self.description,
            verbs: self.verbs,
            aliases: self.aliases,
            _state: PhantomData,
        })
    }
}

// Similar implementations for VerbSpec, ArgumentSpec, TypeRegistry
```

---

## Phase 3: Code Generation (Week 4-5)

### Step 3.1: Implement Generator Traits

Create `src/generator/traits.rs`:

```rust
//! Generator traits

use crate::error::Result;

/// Generator trait
pub trait Generator {
    type Input;
    type Output;
    type Error;

    fn generate(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Noun generator
pub trait NounGenerator: Generator {
    fn generate_noun_struct(&self, noun: &NounSpec<Validated>) -> String;
    fn generate_noun_module(&self, noun: &NounSpec<Validated>) -> GeneratedCode;
}

/// Verb generator
pub trait VerbGenerator: Generator {
    fn generate_verb_signature(&self, verb: &VerbSpec<Validated>) -> String;
    fn generate_verb_handler(&self, verb: &VerbSpec<Validated>) -> GeneratedCode;
}
```

### Step 3.2: Create Tera Templates

Create `templates/noun.tmpl`:

```rust
// Generated noun: {{ noun.name }}
{% if noun.description %}
/// {{ noun.description }}
{% endif %}
#[noun(name = "{{ noun.name }}")]
pub struct {{ noun.struct_name }};
```

Create `templates/verb.tmpl`:

```rust
// Generated verb: {{ verb.name }}
{% if verb.description %}
/// {{ verb.description }}
{% endif %}
#[verb(noun = "{{ verb.noun_name }}", name = "{{ verb.name }}")]
pub {% if verb.async %}async {% endif %}fn {{ verb.fn_name }}(
    {% for arg in verb.arguments %}
    #[arg(long{% if arg.required %}, required = true{% endif %})] {{ arg.name }}: {{ arg.rust_type }},
    {% endfor %}
) -> Result<(), CliError> {
    {{ verb.noun_name }}_domain::{{ verb.name }}(
        {% for arg in verb.arguments %}{{ arg.name }},{% endfor %}
    )
}
```

---

## Testing Checklist

### Phase 1 Tests
- [ ] NonEmptyString creation and validation
- [ ] Typestate size equality (zero-cost)
- [ ] AST type construction
- [ ] Error type construction

### Phase 2 Tests
- [ ] Uniqueness validation
- [ ] Reference validation
- [ ] Typestate transitions
- [ ] Property-based tests (proptest)

### Phase 3 Tests
- [ ] Code generation (noun/verb)
- [ ] Template rendering
- [ ] Golden tests (insta)
- [ ] End-to-end (Turtle → Project)

---

## Validation Commands

```bash
# Phase 1 validation
cargo make check
cargo make test-unit
cargo make lint

# Phase 2 validation
cargo make test
cargo make slo-check

# Phase 3 validation
cargo make test
cargo make bench
cd output/my-cli && cargo make check

# Full validation
cargo make ci
cargo make pre-commit
```

---

## Next Steps

1. Follow Phase 1 implementation
2. Run tests after each step
3. Commit with descriptive messages
4. Move to Phase 2 after Phase 1 complete

---

---

## Examples

### Example 1: Calculator CLI

See a complete working example in:
- **Turtle Spec**: `/home/user/clap-noun-verb/examples/turtle-specs/calculator.ttl`
- **Generated Code**: `/home/user/clap-noun-verb/examples/generated-from-turtle/calculator-cli/`

**Key Features**:
- Simple arithmetic operations (add, subtract, multiply, divide)
- Integer type handling
- Division by zero validation
- Clear error messages

**Try it**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/calculator-cli
cargo make check
cargo make test
cargo run -- calc add --left 5 --right 3
```

### Example 2: File Manager CLI

See a complete working example in:
- **Turtle Spec**: `/home/user/clap-noun-verb/examples/turtle-specs/file-manager.ttl`
- **Generated Code**: `/home/user/clap-noun-verb/examples/generated-from-turtle/file-manager-cli/`

**Key Features**:
- File and directory operations
- PathBuf type handling
- Boolean flags (--force, --verbose, --recursive)
- Path validation
- Confirmation prompts

**Try it**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/file-manager-cli
cargo make check
cargo run -- file create --path /tmp/test.txt --verbose
cargo run -- dir create --path /tmp/testdir --recursive
```

### Example 3: User API CLI

See a complete working example in:
- **Turtle Spec**: `/home/user/clap-noun-verb/examples/turtle-specs/user-api.ttl`
- **Generated Code**: `/home/user/clap-noun-verb/examples/generated-from-turtle/user-api-cli/`

**Key Features**:
- CRUD operations for multiple resources (user, post, comment)
- Email validation with regex
- Pagination with limit/offset
- Global flags with environment variables
- Complex return types

**Try it**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/user-api-cli
cargo make check
export API_KEY=test123
cargo run -- user create --name "John Doe" --email "john@example.com" --age 30
cargo run -- user list --limit 10
```

### Example 4: Web Server CLI

See a complete working example in:
- **Turtle Spec**: `/home/user/clap-noun-verb/examples/turtle-specs/web-server.ttl`
- **Generated Code**: `/home/user/clap-noun-verb/examples/generated-from-turtle/web-server-cli/`

**Key Features**:
- Server lifecycle management
- Configuration validation
- Port range validation
- Daemon mode
- Multiple output formats (JSON, YAML, TOML)
- Route management

**Try it**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/web-server-cli
cargo make check
cargo run -- server start --port 8080 --host 0.0.0.0
cargo run -- server status --verbose
cargo run -- config show --format json
```

### Additional Examples

For more examples and detailed explanations, see:
- [Turtle Specifications README](/home/user/clap-noun-verb/examples/turtle-specs/README.md)
- [Generated CLI Examples](/home/user/clap-noun-verb/examples/generated-from-turtle/README.md)
- [Examples Showcase](/home/user/clap-noun-verb/docs/EXAMPLES_SHOWCASE.md)

---

## Troubleshooting

### Common Issues and Solutions

#### Issue 1: Compiler Errors After Generation

**Symptom**:
```
error[E0425]: cannot find value `validate_email` in scope
```

**Solution**:
- Ensure all validators referenced in Turtle spec are implemented in `src/validators.rs`
- Check that validator names match between spec and implementation
- Verify imports are correct

**Fix**:
```bash
# Check generated validators
cat src/validators.rs

# Implement missing validator
# Add to src/validators.rs:
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    // Implementation
}
```

#### Issue 2: Turtle Syntax Errors

**Symptom**:
```
Parsing error at line 42: Expected '.' after triple
```

**Solution**:
- Every RDF triple must end with a period (`.`)
- Lists must use parentheses: `(item1 item2)`
- Semicolons (`;`) separate properties for the same subject
- Check for missing quotes around strings

**Fix**:
```turtle
# ❌ Incorrect
my:Noun a clap:Noun ;
    clap:name "noun"      # Missing period!

# ✅ Correct
my:Noun a clap:Noun ;
    clap:name "noun" .    # Period at end
```

#### Issue 3: Missing Dependencies

**Symptom**:
```
error: no matching package named `reqwest` found
```

**Solution**:
- Add missing dependencies to `Cargo.toml`
- Common dependencies for generated CLIs:
  - `clap` (CLI parsing)
  - `thiserror` (error types)
  - `reqwest` (HTTP clients)
  - `tokio` (async runtime)
  - `serde`/`serde_json` (serialization)

**Fix**:
```toml
# Add to Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "blocking"] }
tokio = { version = "1.0", features = ["full"] }
```

#### Issue 4: Test Failures

**Symptom**:
```
test test_create_user ... FAILED
```

**Solution**:
- Read the test failure output carefully
- Check that business logic is implemented
- Verify test assertions match expected behavior
- Use `cargo test -- --nocapture` to see print statements

**Fix**:
```bash
# Run specific test with output
cargo test test_create_user -- --nocapture

# Debug with logging
RUST_LOG=debug cargo test test_create_user -- --nocapture
```

#### Issue 5: Validation Not Working

**Symptom**:
Invalid input passes through without error.

**Solution**:
- Verify validation is referenced in verb definition
- Check validator implementation
- Ensure validator is called in execute() method

**Fix**:
```turtle
# In Turtle spec - ensure validation is referenced
my:CreateVerb a clap:Verb ;
    clap:validation my:ValidEmail .  # Must reference validator

# In Rust code - ensure validator is called
impl CreateUserArgs {
    pub fn execute(&self) -> Result<UserResponse, CliError> {
        validate_email(&self.email)?;  // Must call validator
        // ... rest of implementation
    }
}
```

#### Issue 6: Cargo Make Commands Not Found

**Symptom**:
```
cargo make: command not found
```

**Solution**:
Install cargo-make:
```bash
cargo install cargo-make
```

Verify installation:
```bash
cargo make --version
```

#### Issue 7: ggen Not Found

**Symptom**:
```
ggen: command not found
```

**Solution**:
Build ggen from vendors directory:
```bash
cd /home/user/clap-noun-verb/vendors/ggen
cargo build --release
export PATH=$PATH:/home/user/clap-noun-verb/vendors/ggen/target/release
```

Or use from crate directory:
```bash
cd /home/user/clap-noun-verb/crates/ggen-clap-noun-verb
cargo run -- generate --input ../../examples/turtle-specs/spec.ttl --output ../../output
```

#### Issue 8: Type Mismatch Errors

**Symptom**:
```
error[E0308]: mismatched types
expected `i32`, found `i64`
```

**Solution**:
- Check Turtle spec uses correct XSD types
- Ensure Rust type matches spec
- Use appropriate type conversions

**Fix**:
```turtle
# Use correct XSD type
# For i32:
clap:valueType xsd:integer .

# For i64:
clap:valueType xsd:long .

# For u16 (ports):
clap:valueType xsd:unsignedShort .
```

#### Issue 9: Andon Signals (Warnings)

**Symptom**:
```
warning: unused variable: `verbose`
```

**Solution**:
- Fix all clippy warnings immediately (Stop the Line)
- Never proceed with warnings present
- Use variables or prefix with underscore if intentionally unused

**Fix**:
```bash
# Check for warnings
cargo make lint

# Fix unused variables
# If intentionally unused:
fn execute(&self, _verbose: bool) { ... }

# If should be used:
fn execute(&self, verbose: bool) {
    if verbose {
        println!("Verbose output");
    }
}
```

#### Issue 10: Performance Issues

**Symptom**:
CLI takes > 100ms to execute simple commands.

**Solution**:
- Check for unnecessary allocations
- Profile with `cargo make profile`
- Verify SLOs with `cargo make slo-check`
- Use release builds for benchmarking

**Fix**:
```bash
# Profile execution
cargo make profile

# Check SLOs
cargo make slo-check

# Use release build
cargo build --release
./target/release/my-cli <command>
```

---

## Getting Help

### Documentation Resources

1. **Complete Usage Guide**: [USAGE_GUIDE.md](/home/user/clap-noun-verb/docs/USAGE_GUIDE.md)
2. **Turtle Syntax Reference**: [TURTLE_SPECIFICATION_GUIDE.md](/home/user/clap-noun-verb/docs/TURTLE_SPECIFICATION_GUIDE.md)
3. **Examples Showcase**: [EXAMPLES_SHOWCASE.md](/home/user/clap-noun-verb/docs/EXAMPLES_SHOWCASE.md)
4. **Example Specifications**: [examples/turtle-specs/README.md](/home/user/clap-noun-verb/examples/turtle-specs/README.md)
5. **Generated CLIs**: [examples/generated-from-turtle/README.md](/home/user/clap-noun-verb/examples/generated-from-turtle/README.md)

### Debug Checklist

When encountering issues:

- [ ] Run `cargo make check` - Any compiler errors?
- [ ] Run `cargo make test` - All tests passing?
- [ ] Run `cargo make lint` - Any clippy warnings?
- [ ] Check Turtle syntax with `rapper -i turtle spec.ttl`
- [ ] Verify all dependencies in Cargo.toml
- [ ] Check that all validators are implemented
- [ ] Review generated code structure
- [ ] Enable debug logging: `RUST_LOG=debug cargo run`
- [ ] Check timeout command exists: `cargo make timeout-check`
- [ ] Verify all Andon signals cleared

### Common Patterns

**Pattern 1: Add New Verb**
1. Add verb to Turtle spec
2. Regenerate code
3. Implement execute() method
4. Add tests
5. Verify with `cargo make test`

**Pattern 2: Add Validation**
1. Define validation in Turtle spec
2. Reference in verb definition
3. Implement validator in validators.rs
4. Call in execute() method
5. Add test for validation

**Pattern 3: Add Optional Argument**
1. Define argument with `clap:required false`
2. Add `clap:defaultValue` if appropriate
3. Use `Option<T>` in Rust code
4. Handle None case in execute()

---

## Reference Documents

- **Full Architecture**: `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-architecture.md`
- **ADR Summary**: `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-adr-summary.md`
- **Usage Guide**: `/home/user/clap-noun-verb/docs/USAGE_GUIDE.md`
- **Turtle Specification Guide**: `/home/user/clap-noun-verb/docs/TURTLE_SPECIFICATION_GUIDE.md`
- **Examples Showcase**: `/home/user/clap-noun-verb/docs/EXAMPLES_SHOWCASE.md`
- **This Guide**: `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-quickstart.md`

---

**Happy Coding!**
