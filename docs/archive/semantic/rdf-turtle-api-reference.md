# RDF Turtle CLI Generation - API Reference

**Quick reference for type signatures and core APIs**

---

## Core Types

### TurtleOntology - Parsed RDF Ontology

```rust
pub struct TurtleOntology<S: StorageBackend, const N: usize = 5> {
    storage: S,
    namespaces: [Namespace; N],
    commands: CommandIndex,
    executor: SparqlExecutor<S>,
    _validation: PhantomData<Validated>,
}

impl<S: StorageBackend, const N: usize> TurtleOntology<S, N> {
    // Parse from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ParseError>;

    // Parse from string
    pub fn from_str(turtle: &str) -> Result<Self, ParseError>;

    // Validate ontology (state transition)
    pub fn validate(self) -> Result<TurtleOntology<S, N>, ValidationError>;

    // Query with SPARQL
    pub fn query(&self, sparql: &str) -> Result<QueryResults, QueryError>;

    // Get command count
    pub fn command_count(&self) -> usize;

    // Check if command exists
    pub fn has_command(&self, name: &str) -> bool;

    // Get command by name
    pub fn get_command(&self, name: &str) -> Option<&Command>;
}
```

### StorageBackend - Zero-Cost Storage Abstraction

```rust
pub trait StorageBackend: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    fn load_triples(&self) -> Result<impl Iterator<Item = RdfTriple>, Self::Error>;
    fn query_sparql(&self, query: &str) -> Result<QueryResults, Self::Error>;
}

// In-memory storage (default)
pub struct MemoryStorage {
    triples: Vec<RdfTriple>,
    graph: oxigraph::MemoryStore,
}

impl MemoryStorage {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
}
```

### CliGenerator - Type-Safe Code Generation

```rust
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

// Rust CLI code generator
pub struct RustCliGenerator<const FEATURES: u32 = 0> {
    config: GeneratorConfig,
}

impl<const FEATURES: u32> RustCliGenerator<FEATURES> {
    pub fn new(config: GeneratorConfig) -> Self;

    // Generate with default config
    pub fn default() -> Self;
}

// Feature flags (const generics)
pub mod features {
    pub const ASYNC_SUPPORT: u32 = 1 << 0;
    pub const COMPLETIONS: u32 = 1 << 1;
    pub const MAN_PAGES: u32 = 1 << 2;
    pub const COLORED_HELP: u32 = 1 << 3;
}
```

---

## MCP Tool Types

### GenerateCliFromTurtle - Turtle to Rust CLI

```rust
pub struct GenerateCliFromTurtle<G: CliGenerator<TokenStream>> {
    generator: G,
    cache: Arc<GenerationCache>,
}

impl<G: CliGenerator<TokenStream>> GenerateCliFromTurtle<G> {
    pub fn new() -> Self;
    pub fn with_generator(generator: G) -> Self;
    pub fn execute(&self, input: &GenerateCliInput) -> Result<GenerateCliOutput, McpError>;
}

// Tool input
#[derive(Debug, Clone, serde::Deserialize)]
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

// Tool output
#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerateCliOutput {
    pub code: String,
    pub commands: Vec<CommandSummary>,
    pub metadata: GenerationMetadata,
}
```

### QueryCapabilities - SPARQL Queries

```rust
pub struct QueryCapabilities<S: StorageBackend> {
    ontology: Arc<TurtleOntology<S>>,
    query_optimizer: SparqlOptimizer,
}

impl<S: StorageBackend> QueryCapabilities<S> {
    pub fn new(ontology: Arc<TurtleOntology<S>>) -> Self;
    pub fn execute(&self, input: &QueryInput) -> Result<QueryOutput, McpError>;
}

// Tool input
#[derive(Debug, Clone, serde::Deserialize)]
pub struct QueryInput {
    pub query: String,

    #[serde(default)]
    pub format: QueryFormat,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryFormat {
    Json,
    Xml,
    Csv,
    Turtle,
}

// Tool output
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryOutput {
    pub results: QueryResults,
    pub execution_time_us: u64,
    pub result_count: usize,
}
```

### ExportToTurtle - CLI to RDF

```rust
pub struct ExportToTurtle {
    serializer: TurtleSerializer,
}

impl ExportToTurtle {
    pub fn new() -> Self;
    pub fn export(&self, input: &ExportInput) -> Result<ExportOutput, McpError>;
}

// Tool input
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExportInput {
    pub cli_definition: CliDefinition,

    #[serde(default)]
    pub options: ExportOptions,
}

// Tool output
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportOutput {
    pub turtle: String,
    pub triple_count: usize,
    pub namespaces: Vec<String>,
}
```

---

## Error Types

### TurtleCliError - Root Error

```rust
#[derive(Debug, thiserror::Error)]
pub enum TurtleCliError {
    #[error("Failed to parse Turtle ontology: {source}")]
    ParseError {
        #[from]
        source: ParseError,
    },

    #[error("Failed to generate CLI code: {source}")]
    GeneratorError {
        #[from]
        source: GeneratorError,
    },

    #[error("SPARQL query failed: {source}")]
    QueryError {
        #[from]
        source: QueryError,
    },

    #[error("Ontology validation failed: {source}")]
    ValidationError {
        #[from]
        source: ValidationError,
    },

    #[error("MCP tool error: {source}")]
    McpError {
        #[from]
        source: McpError,
    },

    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

pub type Result<T, E = TurtleCliError> = std::result::Result<T, E>;
```

### ParseError - Turtle Parsing

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid Turtle syntax at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Undefined namespace prefix '{prefix}' at line {line}")]
    UndefinedPrefix {
        prefix: String,
        line: usize,
    },

    #[error("Invalid IRI '{iri}': {reason}")]
    InvalidIri {
        iri: String,
        reason: String,
    },

    #[error("Duplicate definition for '{uri}'")]
    DuplicateDefinition {
        uri: String,
    },
}
```

### GeneratorError - Code Generation

```rust
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("Missing required property '{property}' for command '{command}'")]
    MissingProperty {
        command: String,
        property: String,
    },

    #[error("Invalid command structure for '{command}': {reason}")]
    InvalidStructure {
        command: String,
        reason: String,
    },

    #[error("Type mismatch for '{entity}': expected {expected}, found {found}")]
    TypeMismatch {
        entity: String,
        expected: String,
        found: String,
    },

    #[error("Code synthesis failed: {message}")]
    SynthesisError {
        message: String,
    },
}
```

---

## Usage Examples

### Basic: Parse and Generate

```rust
use clap_noun_verb::rdf::prelude::*;

fn main() -> Result<()> {
    // Parse Turtle
    let ontology = TurtleOntology::<MemoryStorage>::from_file("cli.ttl")?;

    // Validate
    let validated = ontology.validate()?;

    // Generate CLI code
    let generator = RustCliGenerator::default();
    let code = generator.generate(&validated)?;

    println!("{}", code.to_string());
    Ok(())
}
```

### Advanced: Feature Flags

```rust
use clap_noun_verb::rdf::prelude::*;

fn main() -> Result<()> {
    let ontology = TurtleOntology::<MemoryStorage>::from_file("cli.ttl")?.validate()?;

    // Generator with async + completions
    let generator = RustCliGenerator::<
        { features::ASYNC_SUPPORT | features::COMPLETIONS }
    >::new(GeneratorConfig::default());

    let code = generator.generate(&ontology)?;
    Ok(())
}
```

### SPARQL Queries

```rust
use clap_noun_verb::rdf::prelude::*;

fn main() -> Result<()> {
    let ontology = Arc::new(TurtleOntology::<MemoryStorage>::from_file("cli.ttl")?);
    let query_tool = QueryCapabilities::new(ontology);

    let input = QueryInput {
        query: r#"
            PREFIX cnv: <https://cnv.dev/ontology#>
            SELECT ?name WHERE {
                ?cmd a cnv:Command ;
                     cnv:name ?name .
            }
        "#.to_string(),
        format: QueryFormat::Json,
    };

    let output = query_tool.execute(&input)?;
    println!("Found {} commands", output.result_count);
    Ok(())
}
```

---

## Module Structure

```
src/rdf/
├── turtle/
│   ├── parser.rs         # TurtleParser, ParseError
│   ├── ontology.rs       # TurtleOntology<S, N>
│   ├── storage.rs        # StorageBackend, MemoryStorage
│   └── validation.rs     # Validator, ValidationError
│
├── codegen/
│   ├── generator.rs      # CliGenerator, RustCliGenerator
│   ├── commands.rs       # Command extraction
│   ├── templates.rs      # TokenStream generation
│   └── optimizer.rs      # Code optimization
│
├── mcp/
│   ├── tools.rs          # MCP tool implementations
│   ├── server.rs         # Enhanced RdfMcpServer
│   └── types.rs          # Tool input/output types
│
├── sparql/               # SPARQL execution (existing)
├── error.rs              # Error hierarchy
├── types.rs              # Core RDF types (existing)
└── prelude.rs            # Convenient imports
```

---

## Performance SLOs

- Turtle parsing: ≤ 50ms for 1000 triples
- CLI generation: ≤ 100ms for 10 commands
- SPARQL queries: ≤ 10ms for simple queries
- Memory usage: ≤ 20MB for typical ontology

---

## Key Design Decisions

1. **Generic Storage Backend**: Zero-cost abstraction via monomorphization
2. **Const Generic Namespaces**: Compile-time validation of namespace count
3. **State Machine Types**: `Validated` vs `Unvalidated` enforced by types
4. **TokenStream Output**: Native Rust representation for generated code
5. **Oxigraph Integration**: Full SPARQL 1.1 support with type safety

---

## Implementation Phases

1. **Phase 1**: Core Parsing (`TurtleOntology`, `MemoryStorage`, `ParseError`)
2. **Phase 2**: Code Generation (`CliGenerator`, `RustCliGenerator`, command extraction)
3. **Phase 3**: MCP Tools (`GenerateCliFromTurtle`, `QueryCapabilities`, `ExportToTurtle`)
4. **Phase 4**: Integration & Optimization (benchmarks, SLO validation, documentation)

---

**See**: `/home/user/clap-noun-verb/docs/rdf-turtle-cli-architecture.md` for complete architecture documentation.
