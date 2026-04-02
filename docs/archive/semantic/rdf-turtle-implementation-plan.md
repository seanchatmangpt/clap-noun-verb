# RDF Turtle CLI Generation - Implementation Plan

**Project**: clap-noun-verb RDF Turtle to CLI Code Generation
**Architecture**: Type-first, Zero-cost abstractions, Chicago TDD
**Timeline**: 4 weeks (4 phases)

---

## Overview

This implementation plan details the step-by-step execution of the RDF Turtle CLI generation feature for clap-noun-verb v5.4+. The plan follows SPARC methodology and enforces Chicago TDD principles throughout.

---

## Phase 1: Core Parsing (Week 1)

### Goals
- Implement type-safe Turtle parsing with oxigraph
- Create generic storage backend with zero-cost abstractions
- Establish comprehensive error handling hierarchy
- Write Chicago TDD tests for all parsing scenarios

### Deliverables

#### 1.1 Storage Backend (`src/rdf/turtle/storage.rs`)

```rust
// Define zero-cost storage abstraction
pub trait StorageBackend: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    fn load_triples(&self) -> Result<impl Iterator<Item = RdfTriple>, Self::Error>;
    fn query_sparql(&self, query: &str) -> Result<QueryResults, Self::Error>;
}

// Implement in-memory storage with oxigraph
pub struct MemoryStorage {
    triples: Vec<RdfTriple>,
    graph: oxigraph::MemoryStore,
}
```

**Tests** (`tests/storage_tests.rs`):
- Test MemoryStorage creation and initialization
- Test triple loading and iteration
- Test SPARQL query execution
- Test error handling for invalid queries

#### 1.2 Turtle Parser (`src/rdf/turtle/parser.rs`)

```rust
// Turtle parsing with detailed error reporting
pub struct TurtleParser {
    namespaces: Vec<Namespace>,
}

impl TurtleParser {
    pub fn parse<S: StorageBackend>(&self, input: &str)
        -> Result<TurtleOntology<S>, ParseError>;
}

// Error types with line/column information
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    SyntaxError { line: usize, column: usize, message: String },
    UndefinedPrefix { prefix: String, line: usize },
    InvalidIri { iri: String, reason: String },
    DuplicateDefinition { uri: String },
}
```

**Tests** (`tests/parser_tests.rs`):
- Test valid Turtle parsing (AAA pattern)
- Test syntax error reporting with line/column
- Test namespace prefix resolution
- Test duplicate definition detection
- Test IRI validation

#### 1.3 TurtleOntology (`src/rdf/turtle/ontology.rs`)

```rust
// Core ontology type with const generic namespaces
pub struct TurtleOntology<S: StorageBackend, const N: usize = 5> {
    storage: S,
    namespaces: [Namespace; N],
    commands: CommandIndex,
    executor: SparqlExecutor<S>,
    _validation: PhantomData<Unvalidated>, // Initial state
}

impl<S: StorageBackend, const N: usize> TurtleOntology<S, N> {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ParseError>;
    pub fn from_str(turtle: &str) -> Result<Self, ParseError>;
    pub fn command_count(&self) -> usize;
    pub fn has_command(&self, name: &str) -> bool;
    pub fn get_command(&self, name: &str) -> Option<&Command>;
}
```

**Tests** (`tests/ontology_tests.rs`):
- Test ontology creation from file
- Test ontology creation from string
- Test command indexing and lookup
- Test namespace management
- Test const generic namespace count validation (compile-time)

### Validation Checklist (Phase 1)

- [ ] All code compiles with `cargo make check`
- [ ] No compiler warnings
- [ ] All tests pass with `cargo make test`
- [ ] No clippy warnings with `cargo make lint`
- [ ] Chicago TDD tests follow AAA pattern
- [ ] Tests verify observable state, not implementation
- [ ] Error messages include context (line numbers, entity names)
- [ ] Zero-cost abstractions verified (no vtables in objdump)

---

## Phase 2: Code Generation (Week 2)

### Goals
- Implement CliGenerator trait with type-safe output
- Create RustCliGenerator with const generic feature flags
- Build command extraction from ontology
- Generate valid Rust code (TokenStream)

### Deliverables

#### 2.1 CliGenerator Trait (`src/rdf/codegen/generator.rs`)

```rust
// Type-safe code generation trait
pub trait CliGenerator<Output, Config = DefaultConfig> {
    type Error: std::error::Error + Send + Sync + 'static;

    fn generate<S: StorageBackend, const N: usize>(
        &self,
        ontology: &TurtleOntology<S, N>,
    ) -> Result<Output, Self::Error>;

    fn validate<S: StorageBackend, const N: usize>(
        &self,
        ontology: &TurtleOntology<S, N>,
    ) -> Result<ValidationReport, Self::Error>;
}

// Rust CLI generator with feature flags
pub struct RustCliGenerator<const FEATURES: u32 = 0> {
    config: GeneratorConfig,
    _features: PhantomData<ConstU32<FEATURES>>,
}

// Feature flags (const generics)
pub mod features {
    pub const ASYNC_SUPPORT: u32 = 1 << 0;
    pub const COMPLETIONS: u32 = 1 << 1;
    pub const MAN_PAGES: u32 = 1 << 2;
    pub const COLORED_HELP: u32 = 1 << 3;
}
```

**Tests** (`tests/generator_tests.rs`):
- Test basic CLI generation (struct + functions)
- Test feature flag compilation (async, completions)
- Test generated code is syntactically valid (syn::parse_file)
- Test generator error handling
- Test const generic feature flag optimization

#### 2.2 Command Extraction (`src/rdf/codegen/commands.rs`)

```rust
// Extract commands from RDF ontology
pub struct CommandExtractor<'ont, S: StorageBackend> {
    ontology: &'ont TurtleOntology<S>,
}

impl<'ont, S: StorageBackend> CommandExtractor<'ont, S> {
    pub fn extract(&self) -> Result<Vec<Command<'ont>>, GeneratorError>;
    pub fn extract_nouns(&self) -> Result<Vec<Noun<'ont>>, GeneratorError>;
    pub fn extract_verbs(&self, noun: &str) -> Result<Vec<Verb<'ont>>, GeneratorError>;
}

// Command representation
pub struct Command<'ont> {
    pub noun: String,
    pub verbs: Vec<Verb<'ont>>,
    pub args: Vec<Argument>,
    pub metadata: CommandMetadata,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    MissingProperty { command: String, property: String },
    InvalidStructure { command: String, reason: String },
    TypeMismatch { entity: String, expected: String, found: String },
    SynthesisError { message: String },
}
```

**Tests** (`tests/command_extraction_tests.rs`):
- Test command extraction from valid ontology
- Test noun extraction
- Test verb extraction for specific noun
- Test error handling for missing properties
- Test error handling for invalid structures

#### 2.3 Code Templates (`src/rdf/codegen/templates.rs`)

```rust
// Generate TokenStream for CLI code
pub struct CodeTemplates;

impl CodeTemplates {
    pub fn generate_noun_struct(&self, noun: &Noun) -> TokenStream;
    pub fn generate_verb_function(&self, verb: &Verb) -> TokenStream;
    pub fn generate_argument(&self, arg: &Argument) -> TokenStream;
    pub fn generate_module(&self, commands: &[Command]) -> TokenStream;
}
```

**Tests** (`tests/template_tests.rs`):
- Test noun struct generation
- Test verb function generation
- Test argument generation
- Test full module generation
- Test generated code compiles (snapshot tests with insta)

### Validation Checklist (Phase 2)

- [ ] Generated code is syntactically valid Rust
- [ ] Feature flags compile out when not used (verify with asm)
- [ ] All tests pass with `cargo make test`
- [ ] Snapshot tests capture generated code examples
- [ ] Error messages provide recovery suggestions
- [ ] Zero allocations in hot paths (verified with benchmarks)

---

## Phase 3: MCP Tools (Week 3)

### Goals
- Implement GenerateCliFromTurtle MCP tool
- Implement QueryCapabilities MCP tool
- Implement ExportToTurtle MCP tool
- Integrate with existing RdfMcpServer

### Deliverables

#### 3.1 MCP Tool Types (`src/rdf/mcp/types.rs`)

```rust
// Tool input/output types (strongly typed)
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateCliInput {
    #[serde(flatten)]
    pub source: TurtleSource,
    #[serde(default)]
    pub options: GenerationOptions,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum TurtleSource {
    Content { turtle: String },
    Path { path: PathBuf },
    Url { url: Url },
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerateCliOutput {
    pub code: String,
    pub commands: Vec<CommandSummary>,
    pub metadata: GenerationMetadata,
}

// Query tool types
#[derive(Debug, Clone, serde::Deserialize)]
pub struct QueryInput {
    pub query: String,
    #[serde(default)]
    pub format: QueryFormat,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryOutput {
    pub results: QueryResults,
    pub execution_time_us: u64,
    pub result_count: usize,
}

// Export tool types
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExportInput {
    pub cli_definition: CliDefinition,
    #[serde(default)]
    pub options: ExportOptions,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportOutput {
    pub turtle: String,
    pub triple_count: usize,
    pub namespaces: Vec<String>,
}
```

**Tests** (`tests/mcp_types_tests.rs`):
- Test input deserialization (valid and invalid)
- Test output serialization
- Test unknown field rejection (serde)
- Test default values

#### 3.2 MCP Tools (`src/rdf/mcp/tools.rs`)

```rust
// GenerateCliFromTurtle tool
pub struct GenerateCliFromTurtle<G: CliGenerator<TokenStream>> {
    generator: G,
    cache: Arc<GenerationCache>,
    _phantom: PhantomData<fn() -> TokenStream>,
}

impl<G: CliGenerator<TokenStream>> GenerateCliFromTurtle<G> {
    pub fn new() -> Self;
    pub fn with_generator(generator: G) -> Self;
    pub fn execute(&self, input: &GenerateCliInput)
        -> Result<GenerateCliOutput, McpError>;
}

// QueryCapabilities tool
pub struct QueryCapabilities<S: StorageBackend> {
    ontology: Arc<TurtleOntology<S>>,
    query_optimizer: SparqlOptimizer,
}

impl<S: StorageBackend> QueryCapabilities<S> {
    pub fn new(ontology: Arc<TurtleOntology<S>>) -> Self;
    pub fn execute(&self, input: &QueryInput)
        -> Result<QueryOutput, McpError>;
}

// ExportToTurtle tool
pub struct ExportToTurtle {
    serializer: TurtleSerializer,
}

impl ExportToTurtle {
    pub fn new() -> Self;
    pub fn export(&self, input: &ExportInput)
        -> Result<ExportOutput, McpError>;
}
```

**Tests** (`tests/mcp_tools_tests.rs`):
- Test GenerateCliFromTurtle with valid input
- Test QueryCapabilities with SPARQL queries
- Test ExportToTurtle with CLI definitions
- Test error handling for all tools
- Test tool integration with RdfMcpServer

#### 3.3 Enhanced MCP Server (`src/rdf/mcp/server.rs`)

```rust
// Enhanced RdfMcpServer with Turtle tools
impl RdfMcpServer {
    pub fn with_turtle_tools(self) -> Self {
        self.with_tool(GenerateCliFromTurtle::new())
            .with_tool(QueryCapabilities::new(self.ontology.clone()))
            .with_tool(ExportToTurtle::new())
    }
}
```

**Tests** (`tests/mcp_server_integration_tests.rs`):
- Test server initialization with tools
- Test tool invocation via MCP protocol
- Test error propagation
- Test resource listing includes new tools

### Validation Checklist (Phase 3)

- [ ] All MCP tools registered and discoverable
- [ ] Tool input validation rejects invalid data
- [ ] Tool outputs are correctly serialized
- [ ] Integration tests cover end-to-end flows
- [ ] Error messages are actionable

---

## Phase 4: Integration & Optimization (Week 4)

### Goals
- Performance benchmarks and SLO validation
- End-to-end integration tests
- Documentation (rustdoc + user guide)
- Production readiness validation

### Deliverables

#### 4.1 Performance Benchmarks (`benches/turtle_benchmarks.rs`)

```rust
// Criterion benchmarks for all operations
fn bench_turtle_parsing(c: &mut Criterion) {
    let turtle = include_str!("../testdata/cli-ontology.ttl");
    c.bench_function("parse_1000_triples", |b| {
        b.iter(|| TurtleOntology::<MemoryStorage>::from_str(black_box(turtle)))
    });
}

fn bench_cli_generation(c: &mut Criterion) {
    let ontology = setup_ontology_with_10_commands();
    let generator = RustCliGenerator::default();
    c.bench_function("generate_10_commands", |b| {
        b.iter(|| generator.generate(black_box(&ontology)))
    });
}

fn bench_sparql_queries(c: &mut Criterion) {
    let ontology = setup_ontology();
    c.bench_function("simple_sparql_query", |b| {
        b.iter(|| ontology.query(black_box(SIMPLE_QUERY)))
    });
}
```

**SLO Targets**:
- Turtle parsing: ≤ 50ms for 1000 triples
- CLI generation: ≤ 100ms for 10 commands
- SPARQL queries: ≤ 10ms for simple queries
- Memory usage: ≤ 20MB for typical ontology

#### 4.2 Integration Tests (`tests/integration_tests.rs`)

```rust
// End-to-end integration tests
#[test]
fn test_full_pipeline_turtle_to_cli() {
    // Arrange: Create Turtle ontology
    let turtle = create_test_ontology_turtle();

    // Act: Parse, validate, generate
    let ontology = TurtleOntology::<MemoryStorage>::from_str(&turtle).unwrap();
    let validated = ontology.validate().unwrap();
    let generator = RustCliGenerator::default();
    let code = generator.generate(&validated).unwrap();

    // Assert: Generated code is valid and complete
    assert!(syn::parse_file(&code.to_string()).is_ok());
    assert!(code.to_string().contains("struct Services"));
    assert!(code.to_string().contains("fn status("));
}

#[test]
fn test_mcp_tool_integration() {
    // Arrange: Create MCP server with tools
    let ontology = Arc::new(TurtleOntology::<MemoryStorage>::new());
    let mut server = RdfMcpServer::new(ontology).with_turtle_tools();

    // Act: Invoke tool via MCP protocol
    let request = create_generate_cli_request();
    let response = server.handle_request(&request).unwrap();

    // Assert: Response is valid and contains generated code
    assert!(response["result"]["code"].is_string());
}
```

#### 4.3 Documentation (`docs/turtle-cli-guide.md`)

```markdown
# RDF Turtle CLI Generation Guide

## Quick Start

### 1. Create Turtle Ontology

\`\`\`turtle
@prefix cnv: <https://cnv.dev/ontology#> .

cnv:ServicesCommand a cnv:Command ;
    cnv:name "services" ;
    cnv:hasVerb cnv:StatusVerb .

cnv:StatusVerb a cnv:Verb ;
    cnv:name "status" ;
    cnv:description "Show service status" .
\`\`\`

### 2. Generate CLI Code

\`\`\`rust
use clap_noun_verb::rdf::prelude::*;

let ontology = TurtleOntology::<MemoryStorage>::from_file("cli.ttl")?;
let validated = ontology.validate()?;
let generator = RustCliGenerator::default();
let code = generator.generate(&validated)?;
\`\`\`

### 3. Use Generated CLI

\`\`\`bash
$ mycli services status
All systems operational
\`\`\`

## Advanced Topics

- Feature flags (async, completions, man pages)
- SPARQL queries for introspection
- Custom storage backends
- MCP tool integration
\`\`\`

**Tests** (`tests/doc_tests.rs`):
- Compile all documentation code examples
- Verify examples produce expected output

#### 4.4 Example Programs (`examples/turtle_generation_demo.rs`)

```rust
//! Demonstrates RDF Turtle to CLI code generation
//!
//! Run: cargo run --example turtle_generation_demo --features rdf-composition

use clap_noun_verb::rdf::prelude::*;

fn main() -> Result<()> {
    // Load Turtle ontology
    let turtle = include_str!("../testdata/example-cli.ttl");
    let ontology = TurtleOntology::<MemoryStorage>::from_str(turtle)?;

    // Validate ontology
    let validated = ontology.validate()?;

    // Generate CLI code with async support
    let generator = RustCliGenerator::<{ features::ASYNC_SUPPORT }>::new(
        GeneratorConfig::default()
    );
    let code = generator.generate(&validated)?;

    // Print generated code
    println!("Generated CLI code:");
    println!("{}", code.to_string());

    // Query commands via SPARQL
    let query_tool = QueryCapabilities::new(Arc::new(validated));
    let query_result = query_tool.execute(&QueryInput {
        query: "SELECT ?name WHERE { ?cmd a cnv:Command ; cnv:name ?name }".to_string(),
        format: QueryFormat::Json,
    })?;

    println!("\nFound {} commands", query_result.result_count);

    Ok(())
}
```

### Validation Checklist (Phase 4)

- [ ] All benchmarks meet SLO targets
- [ ] Integration tests cover happy path and error cases
- [ ] Documentation examples compile and run
- [ ] rustdoc coverage 100% for public APIs
- [ ] Example programs demonstrate all features
- [ ] `cargo make ci` passes all checks
- [ ] Production readiness validated

---

## Testing Strategy (Chicago TDD)

### Principles
1. **State-based testing**: Verify observable outputs, not implementation
2. **Real collaborators**: Use real TurtleOntology, real MemoryStorage
3. **AAA pattern**: Arrange-Act-Assert structure
4. **Behavior verification**: Test what code does, not how it does it

### Test Categories

#### Unit Tests
- Colocated with source (`src/rdf/turtle/parser_tests.rs`)
- Test individual functions/methods
- Focus on edge cases and error handling

#### Integration Tests
- In `/tests` directory
- Test component interactions
- Verify end-to-end workflows

#### Property Tests
- Using `proptest` for command generation invariants
- Verify parse-generate-parse roundtrips

#### Snapshot Tests
- Using `insta` for generated code validation
- Capture expected code output
- Detect regressions in code generation

### Test Execution
```bash
# Run all tests
cargo make test

# Run unit tests only
cargo make test-unit

# Run integration tests only
cargo test --test integration_tests

# Run specific test
cargo make test test_turtle_parsing_valid_input

# Run with coverage
cargo make coverage
```

---

## Andon Signal Workflow

### Signals (Stop the Line)
1. **CRITICAL (Red)**: Compiler errors, test failures
2. **HIGH (Yellow)**: Compiler warnings, clippy warnings

### Workflow
1. **Monitor**: `cargo make check`, `cargo make test`, `cargo make lint`
2. **Stop**: When signal appears, immediately stop work
3. **Investigate**: Use 5 Whys root cause analysis
4. **Fix**: Address root cause, not symptom
5. **Verify**: Re-run checks to confirm signal cleared

### Never Proceed With:
- Compiler errors (`error[E...]`)
- Test failures (`test ... FAILED`)
- Clippy warnings (`warning: ...`)
- Unhandled `Result` types

---

## Definition of Done (Each Phase)

### Before Marking Phase Complete:
1. [ ] Run `cargo make timeout-check`
2. [ ] Run `cargo make check` - no compiler errors or warnings
3. [ ] Run `cargo make test` - all tests pass
4. [ ] Run `cargo make lint` - no clippy warnings
5. [ ] Run `cargo make slo-check` - all SLOs met (Phase 4)
6. [ ] Review all error messages for clarity
7. [ ] Verify zero-cost abstractions (objdump/asm)
8. [ ] Update documentation for new APIs
9. [ ] Create snapshot tests for generated code
10. [ ] All Andon signals cleared

---

## Risk Mitigation

### Technical Risks

1. **Risk**: oxigraph dependency size increases binary size
   - **Mitigation**: Feature-gate rdf-composition, use conditional compilation
   - **Acceptance**: Binary size < 10MB for default features

2. **Risk**: SPARQL query performance doesn't meet SLOs
   - **Mitigation**: Implement query optimizer, use HNSW indexing
   - **Acceptance**: 95th percentile < 10ms for simple queries

3. **Risk**: Generated code has compilation errors
   - **Mitigation**: Validate with syn::parse_file, snapshot tests
   - **Acceptance**: 100% of generated code compiles

### Process Risks

1. **Risk**: Scope creep during implementation
   - **Mitigation**: Strict adherence to phase deliverables
   - **Acceptance**: No new features mid-phase

2. **Risk**: Test coverage insufficient
   - **Mitigation**: Chicago TDD mandatory, 80%+ coverage target
   - **Acceptance**: All public APIs tested

---

## Success Criteria

### Compile-Time Guarantees
- [ ] Invalid namespace counts rejected at compile time
- [ ] Unvalidated ontologies cannot generate code (type error)
- [ ] Feature flags const-folded (verified with asm)
- [ ] Zero trait object allocations (verified with objdump)

### Performance SLOs
- [ ] Turtle parsing: ≤ 50ms for 1000 triples
- [ ] CLI generation: ≤ 100ms for 10 commands
- [ ] SPARQL queries: ≤ 10ms for simple queries
- [ ] Memory usage: ≤ 20MB for typical ontology

### Code Quality
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] All Andon signals cleared
- [ ] 100% rustdoc coverage for public APIs
- [ ] 80%+ test coverage

### Chicago TDD Compliance
- [ ] All tests follow AAA pattern
- [ ] Tests verify observable state/behavior
- [ ] No mocks (real collaborators used)
- [ ] Tests pass before feature completion

---

## Tools & Commands

### Development
```bash
# Quick feedback loop
cargo make check          # Compilation check (5s timeout)
cargo make test-unit      # Unit tests only (10s timeout)
cargo make lint           # Clippy linting

# Full validation
cargo make test           # All tests
cargo make ci             # Full CI pipeline
cargo make release-validate  # Release checks

# Performance
cargo make slo-check      # Verify SLOs
cargo make bench          # Run benchmarks
cargo make profile        # Performance profiling
```

### Debugging
```bash
# Assembly inspection
cargo rustc --release -- --emit asm

# Zero-cost verification
objdump -d target/release/clap-noun-verb | grep -A10 "function_name"

# Test specific module
cargo test --package clap-noun-verb --lib rdf::turtle::parser
```

---

## Timeline Summary

| Phase | Duration | Deliverables | Validation |
|-------|----------|--------------|------------|
| 1: Core Parsing | Week 1 | TurtleOntology, StorageBackend, ParseError | All tests pass, zero-cost verified |
| 2: Code Generation | Week 2 | CliGenerator, RustCliGenerator, command extraction | Generated code compiles |
| 3: MCP Tools | Week 3 | 3 MCP tools, enhanced server | End-to-end integration tests pass |
| 4: Integration & Optimization | Week 4 | Benchmarks, docs, examples | SLOs met, production ready |

**Total Duration**: 4 weeks
**Methodology**: SPARC + Chicago TDD + DfLSS
**Quality Gates**: Andon signals, Definition of Done per phase

---

**Related Documents**:
- Architecture: `/home/user/clap-noun-verb/docs/rdf-turtle-cli-architecture.md`
- API Reference: `/home/user/clap-noun-verb/docs/rdf-turtle-api-reference.md`
