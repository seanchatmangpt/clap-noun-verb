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

## Reference Documents

- **Full Architecture**: `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-architecture.md`
- **ADR Summary**: `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-adr-summary.md`
- **This Guide**: `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-quickstart.md`

---

**Happy Coding!**
