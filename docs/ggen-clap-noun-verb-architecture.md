# ggen-clap-noun-verb: Type-First Architecture Design

**Version**: 1.0.0
**Date**: 2026-01-06
**Status**: Architecture Design Phase
**Methodology**: Type-First Design + Zero-Cost Abstractions + Chicago TDD

---

## Executive Summary

**ggen-clap-noun-verb** is a type-safe code generation tool that transforms Turtle DSL specifications into production-ready Rust CLI applications using the clap-noun-verb framework. It combines ggen's ontology-driven code generation with clap-noun-verb's ergonomic noun-verb command patterns.

### Core Value Proposition

```
Turtle Ontology → Type-Safe AST → Zero-Cost Rust CLI
     (Spec)      →   (Compiler)  →     (Execution)
```

**Key Innovation**: Make invalid CLI specifications unrepresentable at the type level.

---

## 1. High-Level System Architecture

### 1.1 Conceptual Architecture (ASCII)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     ggen-clap-noun-verb System                         │
│                                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐            │
│  │ Turtle DSL   │───▶│ Parser Layer │───▶│  AST Layer   │            │
│  │ (.ttl files) │    │ (RDF → AST)  │    │ (Type-Safe)  │            │
│  └──────────────┘    └──────────────┘    └──────────────┘            │
│                            │                      │                    │
│                            ▼                      ▼                    │
│                      ┌──────────────────────────────────┐             │
│                      │   Semantic Validator Layer       │             │
│                      │ (Invariants + Constraints)       │             │
│                      └──────────────────────────────────┘             │
│                                  │                                     │
│                                  ▼                                     │
│                      ┌──────────────────────────────────┐             │
│                      │  Code Generation Pipeline        │             │
│                      │  (AST → Rust CLI Code)           │             │
│                      └──────────────────────────────────┘             │
│                                  │                                     │
│         ┌────────────────────────┼────────────────────────┐           │
│         ▼                        ▼                        ▼           │
│  ┌─────────────┐        ┌─────────────┐        ┌─────────────┐      │
│  │ Noun Module │        │ Verb Module │        │ Type Module │      │
│  │  Generator  │        │  Generator  │        │  Generator  │      │
│  └─────────────┘        └─────────────┘        └─────────────┘      │
│         │                        │                        │           │
│         └────────────────────────┼────────────────────────┘           │
│                                  ▼                                     │
│                      ┌──────────────────────────────────┐             │
│                      │   Template Rendering Engine      │             │
│                      │   (Tera + Custom Filters)        │             │
│                      └──────────────────────────────────┘             │
│                                  │                                     │
│                                  ▼                                     │
│                      ┌──────────────────────────────────┐             │
│                      │   Output: Rust CLI Project       │             │
│                      │   (clap-noun-verb based)         │             │
│                      └──────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────────────────┘

Integration Points:
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│  ggen-core      │◄────────│ ggen-clap-      │────────▶│ clap-noun-verb  │
│  (RDF/SPARQL)   │         │  noun-verb      │         │  (CLI Runtime)  │
└─────────────────┘         └─────────────────┘         └─────────────────┘
        │                           │                             │
        │ Graph/Template            │ Code Generation             │ Macro Expansion
        │ Processing                │ Pipeline                    │ & Auto-Discovery
        └───────────────────────────┴─────────────────────────────┘
```

### 1.2 Data Flow Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Code Generation Pipeline                         │
└─────────────────────────────────────────────────────────────────────┘

INPUT: Turtle DSL
─────────────────
@prefix cli: <http://clap-noun-verb.io/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

cli:User a cli:Noun ;
    cli:hasVerb cli:Create, cli:List, cli:Delete .

cli:Create a cli:Verb ;
    cli:hasArg [ a cli:StringArg ; cli:name "name" ; cli:required true ] .

         ↓ PHASE 1: Parsing (RDF → Structured Data)

RDF Graph (Oxigraph)
────────────────────
Subject: cli:User
  Predicate: rdf:type
  Object: cli:Noun
Subject: cli:User
  Predicate: cli:hasVerb
  Object: cli:Create

         ↓ PHASE 2: AST Construction (Structured → Typed)

Type-Safe AST
─────────────
CliSpec {
    nouns: Vec<NounSpec<Validated>>,
    verbs: HashMap<NounId, Vec<VerbSpec<Validated>>>,
    types: TypeRegistry<Validated>,
}

NounSpec<Validated> {
    id: NounId,
    name: NonEmptyString,
    verbs: Vec<VerbId>,
    marker: PhantomData<Validated>,
}

         ↓ PHASE 3: Semantic Validation (Type-Level Guarantees)

Validated AST (Compile-Time Invariants)
────────────────────────────────────────
✓ All noun names are unique (enforced by HashMap<NounId, _>)
✓ All verb references exist (enforced by typestate pattern)
✓ All argument types are valid (enforced by ArgumentType enum)
✓ No circular dependencies (enforced by topological sort)
✓ Command depth ≤ MAX_DEPTH (enforced by const generic)

         ↓ PHASE 4: Code Generation (AST → Rust Code)

Generated Rust Code
───────────────────
#[noun(name = "user")]
struct User;

#[verb(noun = "user", name = "create")]
fn user_create(
    #[arg(long)] name: String,
) -> Result<(), CliError> {
    user_domain::create(name)
}

         ↓ PHASE 5: Template Rendering (Rust Code → Project)

Output Project Structure
────────────────────────
my-cli/
├── Cargo.toml
├── cli/
│   └── src/main.rs (clap-noun-verb integration)
└── domain/
    └── src/lib.rs (business logic)
```

---

## 2. Module Structure

### 2.1 Crate Organization

```
ggen-clap-noun-verb/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API surface
│   ├── parser/                   # Phase 1: RDF → Structured Data
│   │   ├── mod.rs
│   │   ├── turtle.rs             # Turtle DSL parsing
│   │   ├── rdf_extractor.rs      # RDF graph extraction (via ggen)
│   │   └── sparql_queries.rs     # SPARQL query definitions
│   ├── ast/                      # Phase 2: Typed AST
│   │   ├── mod.rs
│   │   ├── spec.rs               # CliSpec<S: State>
│   │   ├── noun.rs               # NounSpec<S: State>
│   │   ├── verb.rs               # VerbSpec<S: State>
│   │   ├── argument.rs           # ArgumentSpec<S: State>
│   │   ├── types.rs              # Type definitions
│   │   └── state.rs              # Typestate pattern (Unvalidated/Validated)
│   ├── validator/                # Phase 3: Semantic Validation
│   │   ├── mod.rs
│   │   ├── invariants.rs         # Compile-time invariants
│   │   ├── constraints.rs        # Runtime constraints
│   │   ├── uniqueness.rs         # Name uniqueness validation
│   │   └── references.rs         # Reference resolution
│   ├── generator/                # Phase 4: Code Generation
│   │   ├── mod.rs
│   │   ├── pipeline.rs           # Generation pipeline
│   │   ├── noun_gen.rs           # Noun module generation
│   │   ├── verb_gen.rs           # Verb module generation
│   │   ├── type_gen.rs           # Type module generation
│   │   ├── workspace_gen.rs      # Workspace structure
│   │   └── domain_gen.rs         # Domain layer generation
│   ├── templates/                # Phase 5: Template Rendering
│   │   ├── mod.rs
│   │   ├── filters.rs            # Custom Tera filters
│   │   ├── functions.rs          # Custom Tera functions
│   │   └── registry.rs           # Template registry
│   ├── integration/              # Integration Layer
│   │   ├── mod.rs
│   │   ├── ggen.rs               # ggen-core integration
│   │   └── clap_noun_verb.rs     # clap-noun-verb integration
│   ├── error.rs                  # Error hierarchy
│   └── config.rs                 # Configuration types
├── templates/                     # Tera templates
│   ├── workspace.tmpl
│   ├── cli_main.tmpl
│   ├── domain_lib.tmpl
│   ├── noun.tmpl
│   └── verb.tmpl
└── tests/
    ├── integration/
    │   ├── end_to_end_test.rs
    │   └── golden_test.rs
    └── unit/
        ├── parser_test.rs
        ├── validator_test.rs
        └── generator_test.rs
```

### 2.2 Module Responsibilities

| Module | Responsibility | Key Types | Dependencies |
|--------|---------------|-----------|-------------|
| `parser` | Parse Turtle DSL → RDF Graph | `TurtleParser`, `RdfExtractor` | ggen-core, oxigraph |
| `ast` | Type-safe AST with typestate | `CliSpec<S>`, `NounSpec<S>`, `VerbSpec<S>` | None (core types) |
| `validator` | Semantic validation & invariants | `Validator`, `InvariantChecker` | ast |
| `generator` | Code generation pipeline | `Pipeline`, `NounGenerator`, `VerbGenerator` | ast, templates |
| `templates` | Template rendering engine | `TemplateRegistry`, `CustomFilters` | ggen-core (Tera) |
| `integration` | Bridge to external crates | `GgenAdapter`, `ClapAdapter` | ggen-core, clap-noun-verb |
| `error` | Error handling hierarchy | `Error`, `Result<T>` | thiserror |
| `config` | Configuration types | `GeneratorConfig`, `OutputConfig` | serde |

---

## 3. Type System Design

### 3.1 Core Types (Typestate Pattern)

```rust
//! Type-first design: Use typestate pattern to enforce invariants at compile time

// ============================================================================
// STATE MARKERS - Zero-cost type-level state tracking
// ============================================================================

/// Unvalidated state - parsed but not yet validated
pub struct Unvalidated;

/// Validated state - all invariants checked
pub struct Validated;

/// State trait - constrains valid states
pub trait State: sealed::Sealed {}
impl State for Unvalidated {}
impl State for Validated {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Unvalidated {}
    impl Sealed for super::Validated {}
}

// ============================================================================
// CORE SPEC TYPES - Parameterized by state
// ============================================================================

/// CLI specification - top-level container
///
/// Type invariants:
/// - S=Unvalidated: No guarantees, may contain invalid data
/// - S=Validated: All semantic invariants hold, ready for codegen
#[derive(Debug, Clone)]
pub struct CliSpec<S: State = Validated> {
    pub project_name: NonEmptyString,
    pub nouns: Vec<NounSpec<S>>,
    pub shared_types: TypeRegistry<S>,
    _state: PhantomData<S>,
}

/// Noun specification - represents a CLI noun (e.g., "user", "project")
#[derive(Debug, Clone)]
pub struct NounSpec<S: State = Validated> {
    pub id: NounId,
    pub name: NonEmptyString,
    pub description: Option<String>,
    pub verbs: Vec<VerbId>,
    pub aliases: Vec<NonEmptyString>,
    _state: PhantomData<S>,
}

/// Verb specification - represents a CLI verb (e.g., "create", "list")
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

/// Argument specification - represents a CLI argument
#[derive(Debug, Clone)]
pub struct ArgumentSpec<S: State = Validated> {
    pub name: NonEmptyString,
    pub arg_type: ArgumentType,
    pub required: bool,
    pub default: Option<String>,
    pub validator: Option<ValidatorSpec>,
    _state: PhantomData<S>,
}

// ============================================================================
// TYPE REGISTRY - Shared type definitions
// ============================================================================

/// Type registry - manages custom types used across CLI
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry<S: State = Validated> {
    types: HashMap<TypeId, TypeDefinition>,
    _state: PhantomData<S>,
}

/// Type definition - custom type with validation
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub id: TypeId,
    pub name: NonEmptyString,
    pub base_type: RustType,
    pub validator: Option<ValidatorSpec>,
}

// ============================================================================
// ARGUMENT TYPES - Strongly typed CLI arguments
// ============================================================================

/// Argument type - represents Rust types for CLI arguments
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

/// Rust type - base Rust types for code generation
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

// ============================================================================
// VALIDATION TYPES - Type-safe validators
// ============================================================================

/// Validator specification
#[derive(Debug, Clone)]
pub enum ValidatorSpec {
    Regex(String),
    Range { min: i64, max: i64 },
    Length { min: usize, max: usize },
    OneOf(Vec<String>),
    Custom(String), // Custom validator function name
}

// ============================================================================
// NEWTYPE PATTERNS - Prevent primitive obsession
// ============================================================================

/// Non-empty string - makes empty strings unrepresentable
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Create a NonEmptyString - returns None if empty
    pub fn new(s: String) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(NonEmptyString(s))
        }
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Noun identifier - newtype for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NounId(u32);

/// Verb identifier - newtype for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VerbId(u32);

/// Type identifier - newtype for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

// ============================================================================
// CONST GENERICS - Zero-cost compile-time constraints
// ============================================================================

/// Command depth limit - enforced at compile time
pub const MAX_COMMAND_DEPTH: usize = 5;

/// Maximum number of arguments per verb
pub const MAX_ARGS_PER_VERB: usize = 20;

/// Bounded command hierarchy - enforced via const generics
#[derive(Debug)]
pub struct CommandHierarchy<const DEPTH: usize> {
    levels: [Vec<NounId>; DEPTH],
}

// Compile-time assertion: DEPTH must be <= MAX_COMMAND_DEPTH
impl<const DEPTH: usize> CommandHierarchy<DEPTH> {
    pub fn new() -> Self
    where
        [(); DEPTH]: ,  // Assert DEPTH is valid const
    {
        // Compile-time check (will fail compilation if violated)
        const _: () = assert!(DEPTH <= MAX_COMMAND_DEPTH);

        Self {
            levels: std::array::from_fn(|_| Vec::new()),
        }
    }
}
```

### 3.2 Trait Hierarchy

```rust
//! Trait-based polymorphism for generators and validators

// ============================================================================
// GENERATOR TRAITS - Code generation abstraction
// ============================================================================

/// Generator trait - abstract code generation
pub trait Generator {
    type Input;
    type Output;
    type Error;

    /// Generate code from input
    fn generate(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Noun generator - generates noun modules
pub trait NounGenerator: Generator<Input = NounSpec<Validated>, Output = GeneratedCode> {
    /// Generate noun struct definition
    fn generate_noun_struct(&self, noun: &NounSpec<Validated>) -> String;

    /// Generate noun module
    fn generate_noun_module(&self, noun: &NounSpec<Validated>) -> GeneratedCode;
}

/// Verb generator - generates verb functions
pub trait VerbGenerator: Generator<Input = VerbSpec<Validated>, Output = GeneratedCode> {
    /// Generate verb function signature
    fn generate_verb_signature(&self, verb: &VerbSpec<Validated>) -> String;

    /// Generate verb handler
    fn generate_verb_handler(&self, verb: &VerbSpec<Validated>) -> GeneratedCode;
}

// ============================================================================
// VALIDATOR TRAITS - Semantic validation
// ============================================================================

/// Validator trait - abstract validation
pub trait Validator<T> {
    type Error;

    /// Validate input
    fn validate(&self, input: &T) -> Result<(), Self::Error>;
}

/// Invariant checker - compile-time invariants
pub trait InvariantChecker<T> {
    /// Check invariants - must be const fn where possible
    fn check(&self, input: &T) -> Result<(), InvariantViolation>;
}

// ============================================================================
// TRANSFORMATION TRAITS - State transitions
// ============================================================================

/// Transition from Unvalidated to Validated state
pub trait Validate {
    type Error;

    /// Validate and transition to Validated state
    fn validate(self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

// Implementation example
impl Validate for CliSpec<Unvalidated> {
    type Error = ValidationError;

    fn validate(self) -> Result<CliSpec<Validated>, Self::Error> {
        // Perform validation
        // ...

        // Transition to Validated state (zero-cost - just marker change)
        Ok(CliSpec {
            project_name: self.project_name,
            nouns: self.nouns.into_iter()
                .map(|n| n.validate())
                .collect::<Result<Vec<_>, _>>()?,
            shared_types: self.shared_types.validate()?,
            _state: PhantomData,
        })
    }
}

// ============================================================================
// TEMPLATE TRAITS - Template rendering abstraction
// ============================================================================

/// Template renderer - abstract template rendering
pub trait TemplateRenderer {
    type Context;
    type Output;
    type Error;

    /// Render template with context
    fn render(&self, template: &str, context: &Self::Context) -> Result<Self::Output, Self::Error>;
}

/// Custom filter - extend Tera with custom filters
pub trait CustomFilter {
    /// Filter name
    fn name(&self) -> &'static str;

    /// Apply filter
    fn apply(&self, value: &tera::Value, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value>;
}
```

---

## 4. Code Generation Pipeline

### 4.1 Pipeline Architecture

```rust
//! Code generation pipeline - orchestrates all phases

use crate::ast::*;
use crate::error::Result;
use crate::generator::*;
use crate::parser::*;
use crate::validator::*;

/// Generation pipeline - orchestrates parsing, validation, and code generation
pub struct GenerationPipeline {
    parser: TurtleParser,
    validator: SemanticValidator,
    noun_gen: Box<dyn NounGenerator>,
    verb_gen: Box<dyn VerbGenerator>,
    workspace_gen: WorkspaceGenerator,
}

impl GenerationPipeline {
    /// Execute full pipeline: Turtle → Validated AST → Generated Code
    pub fn execute(&self, turtle_path: &Path) -> Result<GeneratedProject> {
        // Phase 1: Parse Turtle DSL → Unvalidated AST
        let unvalidated_spec = self.parser.parse(turtle_path)?;

        // Phase 2: Validate AST → Validated AST (typestate transition)
        let validated_spec = self.validator.validate(unvalidated_spec)?;

        // Phase 3: Generate code from validated AST
        let generated_code = self.generate_code(&validated_spec)?;

        // Phase 4: Render project structure
        let project = self.workspace_gen.generate(generated_code)?;

        Ok(project)
    }

    /// Generate code from validated specification
    fn generate_code(&self, spec: &CliSpec<Validated>) -> Result<GeneratedCode> {
        let mut code = GeneratedCode::new();

        // Generate noun modules (parallel - no dependencies)
        for noun in &spec.nouns {
            let noun_code = self.noun_gen.generate(noun)?;
            code.add_noun_module(noun_code);
        }

        // Generate verb handlers (parallel - no dependencies)
        for noun in &spec.nouns {
            for verb_id in &noun.verbs {
                if let Some(verb) = spec.get_verb(*verb_id) {
                    let verb_code = self.verb_gen.generate(verb)?;
                    code.add_verb_handler(verb_code);
                }
            }
        }

        Ok(code)
    }
}
```

### 4.2 Template Structure

```rust
//! Template definitions for code generation

/// Generated code container
#[derive(Debug, Default)]
pub struct GeneratedCode {
    pub cli_main: String,
    pub domain_lib: String,
    pub noun_modules: HashMap<NounId, String>,
    pub verb_handlers: HashMap<VerbId, String>,
    pub type_definitions: HashMap<TypeId, String>,
}

/// Generated project - complete project structure
#[derive(Debug)]
pub struct GeneratedProject {
    pub root_path: PathBuf,
    pub workspace_toml: String,
    pub cli_crate: CrateStructure,
    pub domain_crate: CrateStructure,
}

/// Crate structure - individual crate files
#[derive(Debug)]
pub struct CrateStructure {
    pub cargo_toml: String,
    pub src_files: HashMap<PathBuf, String>,
}
```

---

## 5. Integration Strategy

### 5.1 ggen-core Integration

```rust
//! Integration with ggen-core for RDF processing and template rendering

use ggen_core::{Graph, Template, Pipeline as GgenPipeline};

/// Adapter for ggen-core Graph
pub struct GgenGraphAdapter {
    graph: Graph,
}

impl GgenGraphAdapter {
    /// Create adapter from Turtle file
    pub fn from_turtle(path: &Path) -> Result<Self> {
        let graph = Graph::new()?;
        let turtle_content = std::fs::read_to_string(path)?;
        graph.insert_turtle(&turtle_content)?;
        Ok(Self { graph })
    }

    /// Execute SPARQL query
    pub fn query(&self, sparql: &str) -> Result<Vec<QuerySolution>> {
        Ok(self.graph.query(sparql)?)
    }

    /// Extract CLI specification from RDF graph
    pub fn extract_cli_spec(&self) -> Result<CliSpec<Unvalidated>> {
        // Query for nouns
        let nouns_query = r#"
            PREFIX cli: <http://clap-noun-verb.io/ontology#>
            SELECT ?noun ?name ?description WHERE {
                ?noun a cli:Noun ;
                      cli:name ?name .
                OPTIONAL { ?noun cli:description ?description }
            }
        "#;

        let nouns = self.query(nouns_query)?;

        // Query for verbs
        let verbs_query = r#"
            PREFIX cli: <http://clap-noun-verb.io/ontology#>
            SELECT ?verb ?name ?noun ?async WHERE {
                ?verb a cli:Verb ;
                      cli:name ?name ;
                      cli:belongsTo ?noun .
                OPTIONAL { ?verb cli:async ?async }
            }
        "#;

        let verbs = self.query(verbs_query)?;

        // Build unvalidated AST from query results
        // ...

        Ok(cli_spec)
    }
}

/// Adapter for ggen-core Template rendering
pub struct GgenTemplateAdapter {
    pipeline: GgenPipeline,
}

impl GgenTemplateAdapter {
    /// Render template with context
    pub fn render(&self, template_name: &str, context: &tera::Context) -> Result<String> {
        // Use ggen's template rendering infrastructure
        Ok(self.pipeline.render(template_name, context)?)
    }
}
```

### 5.2 clap-noun-verb Integration

```rust
//! Integration with clap-noun-verb runtime

/// Generate clap-noun-verb compatible code
pub struct ClapNounVerbCodegen {
    config: CodegenConfig,
}

impl NounGenerator for ClapNounVerbCodegen {
    fn generate_noun_struct(&self, noun: &NounSpec<Validated>) -> String {
        format!(
            r#"
#[noun(name = "{}")]
pub struct {};
"#,
            noun.name.as_str(),
            self.struct_name(noun)
        )
    }

    fn generate_noun_module(&self, noun: &NounSpec<Validated>) -> GeneratedCode {
        let mut code = String::new();

        // Add imports
        code.push_str("use clap_noun_verb::prelude::*;\n\n");

        // Add noun struct
        code.push_str(&self.generate_noun_struct(noun));

        GeneratedCode {
            noun_modules: [(noun.id, code)].into_iter().collect(),
            ..Default::default()
        }
    }
}

impl VerbGenerator for ClapNounVerbCodegen {
    fn generate_verb_signature(&self, verb: &VerbSpec<Validated>) -> String {
        let args = verb.arguments.iter()
            .map(|arg| self.format_argument(arg))
            .collect::<Vec<_>>()
            .join(",\n    ");

        let async_keyword = if verb.async_handler { "async " } else { "" };

        format!(
            r#"
#[verb(noun = "{}", name = "{}")]
pub {}fn {}_{}(
    {}
) -> Result<(), CliError>
"#,
            self.noun_name(verb.noun_id),
            verb.name.as_str(),
            async_keyword,
            self.noun_name(verb.noun_id),
            verb.name.as_str(),
            args
        )
    }

    fn format_argument(&self, arg: &ArgumentSpec<Validated>) -> String {
        let required = if arg.required { "required = true" } else { "" };
        format!(
            r#"#[arg(long, {})] {}: {}"#,
            required,
            arg.name.as_str(),
            self.rust_type(&arg.arg_type)
        )
    }
}
```

---

## 6. Error Handling Strategy

### 6.1 Error Hierarchy

```rust
//! Error types - comprehensive error handling with Result<T, E>

use thiserror::Error;

/// Top-level error type - all errors in the system
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

    #[error("ggen integration error: {0}")]
    Ggen(#[from] ggen_core::Error),
}

/// Result type alias - simplifies error handling
pub type Result<T> = std::result::Result<T, Error>;

// ============================================================================
// PARSING ERRORS
// ============================================================================

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

// ============================================================================
// VALIDATION ERRORS
// ============================================================================

#[derive(Error, Debug)]
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

    #[error("Command depth exceeds maximum ({max}): {actual}")]
    CommandDepthExceeded { max: usize, actual: usize },

    #[error("Too many arguments ({count}) for verb {verb}. Maximum: {max}")]
    TooManyArguments { verb: String, count: usize, max: usize },

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
}

// ============================================================================
// GENERATION ERRORS
// ============================================================================

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

// ============================================================================
// ERROR RECOVERY STRATEGIES
// ============================================================================

/// Error recovery - attempt to recover from errors
pub trait ErrorRecovery {
    /// Attempt recovery - returns Ok(()) if recovered, Err if unrecoverable
    fn try_recover(&self) -> Result<()>;
}

impl ErrorRecovery for ValidationError {
    fn try_recover(&self) -> Result<()> {
        match self {
            // Some validation errors can be auto-fixed
            ValidationError::DuplicateNoun { name } => {
                // Could auto-rename with suffix
                Err(Error::Validation(self.clone()))
            }
            // Most validation errors are unrecoverable
            _ => Err(Error::Validation(self.clone())),
        }
    }
}
```

---

## 7. Performance Characteristics

### 7.1 Zero-Cost Abstractions

```rust
//! Performance guarantees - zero-cost abstractions

/// ZERO-COST GUARANTEE #1: Typestate Pattern
///
/// The typestate pattern (CliSpec<S: State>) has ZERO runtime cost.
/// The state parameter S is a marker type (zero-sized) and is optimized
/// away by the compiler. There is NO difference in memory layout or
/// runtime behavior between CliSpec<Unvalidated> and CliSpec<Validated>.
///
/// Evidence:
/// ```
/// assert_eq!(
///     std::mem::size_of::<CliSpec<Unvalidated>>(),
///     std::mem::size_of::<CliSpec<Validated>>()
/// );
/// ```

/// ZERO-COST GUARANTEE #2: Newtype Pattern
///
/// Newtypes like NonEmptyString, NounId, VerbId have ZERO runtime cost.
/// They are transparent wrappers and compile to the same representation
/// as their inner type.
///
/// Evidence:
/// ```
/// assert_eq!(
///     std::mem::size_of::<NonEmptyString>(),
///     std::mem::size_of::<String>()
/// );
/// ```

/// ZERO-COST GUARANTEE #3: Const Generics
///
/// Const generic constraints (CommandHierarchy<const DEPTH: usize>)
/// are checked at compile time and have ZERO runtime overhead.
///
/// Evidence: Compiler will reject invalid DEPTH values at compile time,
/// not runtime.

/// ZERO-COST GUARANTEE #4: Trait Objects Avoided
///
/// Where possible, we use static dispatch (generics) instead of dynamic
/// dispatch (trait objects). Generator traits use generics for zero-cost
/// polymorphism.
///
/// Example:
/// ```rust
/// // Zero-cost (monomorphization)
/// fn generate<G: NounGenerator>(gen: &G, noun: &NounSpec) -> String {
///     gen.generate_noun_struct(noun)
/// }
///
/// // Has cost (dynamic dispatch)
/// fn generate_dyn(gen: &dyn NounGenerator, noun: &NounSpec) -> String {
///     gen.generate_noun_struct(noun)
/// }
/// ```
```

### 7.2 Performance SLOs

```toml
# Performance Service Level Objectives

[slo.compilation]
incremental = "≤ 2s"   # Incremental compilation
full = "≤ 15s"         # Full compilation from clean

[slo.parsing]
turtle_small = "≤ 50ms"     # < 100 triples
turtle_medium = "≤ 200ms"   # 100-1000 triples
turtle_large = "≤ 1s"       # > 1000 triples

[slo.validation]
spec_small = "≤ 10ms"    # < 10 nouns
spec_medium = "≤ 50ms"   # 10-50 nouns
spec_large = "≤ 200ms"   # > 50 nouns

[slo.generation]
code_gen = "≤ 100ms"     # Code generation per noun
template_render = "≤ 50ms"  # Template rendering per file

[slo.memory]
peak_usage = "≤ 100MB"   # Peak memory during generation
```

---

## 8. Implementation Plan

### Phase 1: Foundation (Week 1-2)

**Goal**: Establish type system and basic parsing

**Tasks**:
1. Define core AST types with typestate pattern
   - CliSpec<S>, NounSpec<S>, VerbSpec<S>, ArgumentSpec<S>
   - State markers (Unvalidated, Validated)
   - Newtype wrappers (NonEmptyString, NounId, VerbId)

2. Implement RDF parser
   - TurtleParser using ggen-core Graph
   - SPARQL query definitions for CLI ontology
   - Extraction logic: RDF → CliSpec<Unvalidated>

3. Error types and Result<T>
   - Error hierarchy (ParseError, ValidationError, GenerationError)
   - Result type alias
   - Error recovery strategies

4. Basic unit tests (Chicago TDD)
   - Parser tests with sample Turtle files
   - AST construction tests
   - Error handling tests

**Deliverables**:
- Working parser: Turtle → CliSpec<Unvalidated>
- Complete type system
- Test coverage ≥ 80%

**Validation**:
```bash
cargo make test-unit
cargo make lint
```

### Phase 2: Validation (Week 3)

**Goal**: Semantic validation with compile-time guarantees

**Tasks**:
1. Implement semantic validators
   - Uniqueness validator (noun/verb names)
   - Reference validator (verb → noun references)
   - Constraint validator (depth limits, arg counts)
   - Invariant checker (type-level guarantees)

2. Typestate transitions
   - Validate trait implementation
   - CliSpec<Unvalidated> → CliSpec<Validated>
   - Validation pipeline orchestration

3. Property-based tests
   - QuickCheck/proptest for invariants
   - Fuzzing invalid Turtle inputs
   - Edge case discovery

**Deliverables**:
- Complete validation system
- Typestate transitions working
- Property-based test suite

**Validation**:
```bash
cargo make test
cargo make slo-check  # Validation SLOs
```

### Phase 3: Code Generation (Week 4-5)

**Goal**: Generate clap-noun-verb compatible code

**Tasks**:
1. Implement generators
   - NounGenerator (noun structs + modules)
   - VerbGenerator (verb functions + handlers)
   - TypeGenerator (custom type definitions)
   - WorkspaceGenerator (Cargo.toml + project structure)

2. Template system
   - Tera template definitions
   - Custom filters for Rust code
   - Template registry and loader

3. clap-noun-verb integration
   - Generate #[noun] and #[verb] attributes
   - Generate argument parsing code
   - Generate domain function calls

4. Integration tests
   - End-to-end: Turtle → Generated Project
   - Golden tests (snapshot testing with insta)
   - Compilation tests (generated code compiles)

**Deliverables**:
- Complete code generation pipeline
- Working Tera templates
- Generated projects compile successfully

**Validation**:
```bash
cargo make test
cargo make bench  # Generation performance
cd output/my-cli && cargo make check
```

### Phase 4: Integration & Polish (Week 6)

**Goal**: ggen integration and production readiness

**Tasks**:
1. ggen-core integration
   - Adapter for ggen Graph
   - Adapter for ggen Template
   - Integration with ggen CLI

2. Documentation
   - API documentation (rustdoc)
   - Architecture documentation
   - Usage examples and tutorials

3. CLI tool
   - Command-line interface for generator
   - Configuration file support
   - Watch mode for incremental generation

4. Performance optimization
   - Benchmark suite with criterion
   - Profile and optimize hot paths
   - Verify SLOs met

**Deliverables**:
- Published crate on crates.io
- Complete documentation
- CLI tool ready for use

**Validation**:
```bash
cargo make ci          # Full CI pipeline
cargo make slo-check   # All SLOs met
cargo make pre-commit  # Production readiness
```

### Phase 5: Advanced Features (Week 7+)

**Goal**: Advanced features and ecosystem integration

**Tasks**:
1. Advanced validation
   - SHACL shape validation
   - Custom validator plugins
   - Cross-field validation

2. Advanced code generation
   - Middleware generation
   - Test generation
   - Documentation generation

3. Ecosystem integration
   - MCP server integration
   - Agent coordination
   - Marketplace integration

**Deliverables**:
- Advanced feature set
- Ecosystem integrations
- Production deployments

---

## 9. Architecture Decision Records (ADRs)

### ADR-001: Typestate Pattern for AST

**Status**: Accepted
**Date**: 2026-01-06

**Context**:
We need to enforce semantic correctness of CLI specifications before code generation. Invalid specifications should be caught early and prevented from reaching the code generation phase.

**Decision**:
Use the typestate pattern to encode validation state in the type system. AST types are parameterized by a state marker (Unvalidated, Validated), and code generation only accepts Validated types.

**Consequences**:
- **Positive**: Compile-time enforcement of validation - impossible to generate code from unvalidated AST
- **Positive**: Zero runtime cost - state markers are zero-sized types
- **Positive**: Clear API - users know whether they have validated data
- **Negative**: Slightly more complex API with generic parameters

**Alternatives Considered**:
1. Runtime validation checks - rejected due to runtime cost and possibility of forgetting checks
2. Sealed traits - rejected due to less explicit type signatures

### ADR-002: ggen-core for RDF Processing

**Status**: Accepted
**Date**: 2026-01-06

**Context**:
We need to parse Turtle DSL and extract CLI specifications. We could implement our own parser or use existing infrastructure.

**Decision**:
Integrate with ggen-core's Graph module (Oxigraph) for RDF processing and SPARQL queries. Use ggen's template rendering infrastructure (Tera).

**Consequences**:
- **Positive**: Reuse battle-tested RDF processing code
- **Positive**: Consistent with existing ggen ecosystem
- **Positive**: SPARQL provides powerful query capabilities
- **Negative**: Dependency on ggen-core (acceptable trade-off)

**Alternatives Considered**:
1. Custom Turtle parser - rejected due to complexity and maintenance burden
2. Other RDF libraries - rejected due to ggen ecosystem consistency

### ADR-003: Const Generics for Command Depth

**Status**: Accepted
**Date**: 2026-01-06

**Context**:
We need to enforce maximum command depth to prevent deeply nested CLI structures that are hard to use and maintain.

**Decision**:
Use const generics to enforce command depth limits at compile time via CommandHierarchy<const DEPTH: usize> with compile-time assertion.

**Consequences**:
- **Positive**: Zero runtime cost - checked at compile time
- **Positive**: Clear compile errors if depth exceeded
- **Negative**: Requires Rust 1.51+ for const generics

**Alternatives Considered**:
1. Runtime depth checking - rejected due to runtime cost
2. No depth limit - rejected due to usability concerns

### ADR-004: Result<T, E> over Panics

**Status**: Accepted
**Date**: 2026-01-06

**Context**:
Error handling strategy for parser, validator, and generator. We need predictable error handling for library code.

**Decision**:
Use Result<T, E> for all fallible operations. Never use unwrap/expect in production code. Use thiserror for error types.

**Consequences**:
- **Positive**: Explicit error handling - callers must handle errors
- **Positive**: Composable error handling with ?operator
- **Positive**: Detailed error messages with context
- **Negative**: More verbose code (acceptable trade-off)

**Alternatives Considered**:
1. Panic on errors - rejected due to lack of recoverability
2. Option<T> - rejected due to loss of error context

---

## 10. Security Considerations

### 10.1 Input Validation

```rust
//! Security: Input validation and sanitization

/// Validate Turtle input for security issues
pub fn validate_turtle_input(content: &str) -> Result<()> {
    // Check file size limit (prevent DoS)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    if content.len() > MAX_FILE_SIZE {
        return Err(Error::Parse(ParseError::InvalidTurtle {
            line: 0,
            message: "File size exceeds 10MB limit".into(),
        }));
    }

    // Validate Turtle syntax before processing
    // (prevents injection attacks via malformed RDF)
    oxigraph::model::Dataset::from_str(content, oxigraph::model::GraphFormat::Turtle)
        .map_err(|e| Error::Parse(ParseError::InvalidTurtle {
            line: 0,
            message: e.to_string(),
        }))?;

    Ok(())
}

/// Sanitize output paths (prevent directory traversal)
pub fn sanitize_output_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .map_err(|e| Error::Generation(GenerationError::InvalidOutputPath {
            path: path.to_path_buf(),
        }))?;

    // Ensure path is within allowed output directory
    // (prevent writing to sensitive locations)
    if !canonical.starts_with(std::env::current_dir()?) {
        return Err(Error::Generation(GenerationError::InvalidOutputPath {
            path: canonical,
        }));
    }

    Ok(canonical)
}
```

### 10.2 Code Injection Prevention

```rust
//! Prevent code injection in generated code

/// Sanitize identifier names (prevent code injection)
pub fn sanitize_identifier(name: &str) -> Result<String> {
    // Only allow valid Rust identifiers
    let valid_chars = name.chars().all(|c| {
        c.is_alphanumeric() || c == '_'
    });

    if !valid_chars {
        return Err(Error::Validation(ValidationError::InvalidArgumentType(
            format!("Invalid identifier: {}", name)
        )));
    }

    // Check for Rust keywords
    const RUST_KEYWORDS: &[&str] = &[
        "fn", "let", "mut", "const", "static", "if", "else", "match",
        "loop", "while", "for", "return", "break", "continue", "as",
        "unsafe", "mod", "use", "pub", "crate", "super", "self", "Self",
    ];

    if RUST_KEYWORDS.contains(&name) {
        return Err(Error::Validation(ValidationError::InvalidArgumentType(
            format!("Identifier cannot be Rust keyword: {}", name)
        )));
    }

    Ok(name.to_string())
}
```

---

## 11. Testing Strategy

### 11.1 Test Pyramid

```
              ┌─────────────────┐
              │  E2E Tests (5%) │  ← Full pipeline: Turtle → Generated Project
              └─────────────────┘
                    ▲
              ┌─────────────────────┐
              │ Integration Tests   │  ← Module integration
              │      (15%)          │
              └─────────────────────┘
                    ▲
              ┌─────────────────────────┐
              │   Unit Tests (80%)      │  ← Individual functions
              │  (Chicago TDD)          │
              └─────────────────────────┘
```

### 11.2 Chicago TDD Requirements

```rust
//! Chicago TDD - State-based testing with real collaborators

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noun_parsing_state_verification() {
        // ARRANGE: Set up real objects (no mocks)
        let turtle = r#"
            @prefix cli: <http://clap-noun-verb.io/ontology#> .
            cli:User a cli:Noun ;
                cli:name "user" ;
                cli:description "User management" .
        "#;
        let parser = TurtleParser::new();

        // ACT: Call public API
        let result = parser.parse_string(turtle);

        // ASSERT: Verify observable state/output
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert_eq!(spec.nouns.len(), 1);
        assert_eq!(spec.nouns[0].name.as_str(), "user");
        assert_eq!(spec.nouns[0].description, Some("User management".into()));
    }

    #[test]
    fn test_validation_transition_behavior() {
        // ARRANGE
        let unvalidated = CliSpec::<Unvalidated> {
            project_name: NonEmptyString::new("test".into()).unwrap(),
            nouns: vec![/* ... */],
            shared_types: TypeRegistry::default(),
            _state: PhantomData,
        };

        // ACT
        let validated = unvalidated.validate();

        // ASSERT: Verify state transition occurred
        assert!(validated.is_ok());
        let spec = validated.unwrap();
        // Type system guarantees spec is now CliSpec<Validated>
        // Can only call methods that accept Validated state
    }
}
```

---

## 12. Future Enhancements

### 12.1 Planned Features (Post-MVP)

1. **Interactive Mode**: TUI for building CLI specifications interactively
2. **Hot Reload**: Watch mode with incremental regeneration
3. **Plugin System**: Custom generators and validators
4. **SHACL Validation**: Advanced RDF shape validation
5. **Multi-Language**: Generate CLIs in Python, Go, TypeScript
6. **Agent Integration**: MCP server for agent-driven CLI generation
7. **Marketplace**: Template marketplace for common CLI patterns

### 12.2 Research Areas

1. **Formal Verification**: Prove typestate invariants with Kani
2. **Query Optimization**: Optimize SPARQL queries for large ontologies
3. **Incremental Generation**: Only regenerate changed modules
4. **Parallel Code Generation**: Parallelize generation across nouns/verbs

---

## Appendix A: Turtle DSL Ontology

```turtle
@prefix cli: <http://clap-noun-verb.io/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

# ============================================================================
# CLI Ontology - Noun-Verb Command Structure
# ============================================================================

cli:Noun a owl:Class ;
    rdfs:label "CLI Noun" ;
    rdfs:comment "Represents a CLI noun (entity or resource)" .

cli:Verb a owl:Class ;
    rdfs:label "CLI Verb" ;
    rdfs:comment "Represents a CLI verb (action or operation)" .

cli:Argument a owl:Class ;
    rdfs:label "CLI Argument" ;
    rdfs:comment "Represents a CLI argument (parameter)" .

# Properties
cli:name a owl:DatatypeProperty ;
    rdfs:domain [ owl:unionOf (cli:Noun cli:Verb cli:Argument) ] ;
    rdfs:range xsd:string ;
    rdfs:label "Name" .

cli:description a owl:DatatypeProperty ;
    rdfs:domain [ owl:unionOf (cli:Noun cli:Verb cli:Argument) ] ;
    rdfs:range xsd:string ;
    rdfs:label "Description" .

cli:hasVerb a owl:ObjectProperty ;
    rdfs:domain cli:Noun ;
    rdfs:range cli:Verb ;
    rdfs:label "Has Verb" .

cli:hasArg a owl:ObjectProperty ;
    rdfs:domain cli:Verb ;
    rdfs:range cli:Argument ;
    rdfs:label "Has Argument" .

cli:argType a owl:DatatypeProperty ;
    rdfs:domain cli:Argument ;
    rdfs:range xsd:string ;
    rdfs:label "Argument Type" .

cli:required a owl:DatatypeProperty ;
    rdfs:domain cli:Argument ;
    rdfs:range xsd:boolean ;
    rdfs:label "Required" .

cli:async a owl:DatatypeProperty ;
    rdfs:domain cli:Verb ;
    rdfs:range xsd:boolean ;
    rdfs:label "Async Handler" .
```

## Appendix B: Example Turtle Specification

```turtle
@prefix cli: <http://clap-noun-verb.io/ontology#> .
@prefix : <http://example.org/my-cli#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# ============================================================================
# Example: User Management CLI
# ============================================================================

:User a cli:Noun ;
    cli:name "user" ;
    cli:description "User management operations" ;
    cli:hasVerb :UserCreate, :UserList, :UserDelete .

:UserCreate a cli:Verb ;
    cli:name "create" ;
    cli:description "Create a new user" ;
    cli:async "false"^^xsd:boolean ;
    cli:hasArg :UserCreateName, :UserCreateEmail .

:UserCreateName a cli:Argument ;
    cli:name "name" ;
    cli:description "User's full name" ;
    cli:argType "String" ;
    cli:required "true"^^xsd:boolean .

:UserCreateEmail a cli:Argument ;
    cli:name "email" ;
    cli:description "User's email address" ;
    cli:argType "String" ;
    cli:required "true"^^xsd:boolean .

:UserList a cli:Verb ;
    cli:name "list" ;
    cli:description "List all users" ;
    cli:async "false"^^xsd:boolean .

:UserDelete a cli:Verb ;
    cli:name "delete" ;
    cli:description "Delete a user" ;
    cli:async "false"^^xsd:boolean ;
    cli:hasArg :UserDeleteId .

:UserDeleteId a cli:Argument ;
    cli:name "id" ;
    cli:description "User ID to delete" ;
    cli:argType "Integer" ;
    cli:required "true"^^xsd:boolean .
```

---

## Document Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-06 | System Architect | Initial architecture design |

---

**End of Architecture Document**
