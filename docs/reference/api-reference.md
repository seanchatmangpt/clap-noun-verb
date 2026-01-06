# API Reference

Complete reference for all public types and functions in clap-noun-verb with RDF support.

## Core Types

### `TurtleParser`

Parses RDF 1.1 Turtle documents.

```rust
pub struct TurtleParser;

impl TurtleParser {
    /// Create a new Turtle parser
    pub fn new() -> Self { ... }

    /// Parse Turtle document
    pub fn parse(&self, turtle: &str) -> Result<ParsedTurtle, TurtleError>

    /// Parse Turtle from file
    pub fn parse_file(&self, path: &str) -> Result<ParsedTurtle, TurtleError>
}
```

**Example**:
```rust
let parser = TurtleParser::new();
let parsed = parser.parse(turtle_string)?;
```

### `ParsedTurtle`

Represents a parsed RDF ontology.

```rust
pub struct ParsedTurtle {
    // Private fields
}

impl ParsedTurtle {
    /// Validate ontology structure
    pub fn validate_ontology(&self) -> Result<(), TurtleError>

    /// Get all prefixes
    pub fn resolve_prefixes(&self) -> Result<HashMap<String, String>, TurtleError>

    /// Get RDF store for SPARQL queries
    pub fn store(&self) -> &Store
}
```

### `CliCodeGenerator`

Generates Rust CLI code from RDF ontologies.

```rust
pub struct CliCodeGenerator;

impl CliCodeGenerator {
    /// Create new code generator
    pub fn new() -> Result<Self, CodeGenError>

    /// Generate CLI code from ontology
    pub fn generate_from_ontology(
        &self,
        ontology: &ParsedTurtle,
    ) -> Result<GeneratedCli, CodeGenError>

    /// Generate noun macro
    pub fn generate_noun_macro(
        &self,
        name: &str,
        description: &str,
    ) -> String

    /// Generate verb macro
    pub fn generate_verb_macro(
        &self,
        name: &str,
        noun: &str,
        handler: &str,
    ) -> String
}
```

**Example**:
```rust
let generator = CliCodeGenerator::new()?;
let generated = generator.generate_from_ontology(&ontology)?;
println!("{}", generated.rust_code());
```

### `GeneratedCli`

Result of code generation.

```rust
pub struct GeneratedCli {
    // Private fields
}

impl GeneratedCli {
    /// Get generated Rust code
    pub fn rust_code(&self) -> &str

    /// Get number of nouns generated
    pub fn noun_count(&self) -> usize

    /// Get number of verbs generated
    pub fn verb_count(&self) -> usize

    /// Get generation diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic]
}
```

### `SparqlExecutor`

Executes SPARQL 1.1 queries on RDF graphs.

```rust
pub struct SparqlExecutor {
    // Private fields
}

impl SparqlExecutor {
    /// Create executor from parsed ontology
    pub fn new(ontology: &ParsedTurtle) -> Result<Self, SparqlError>

    /// Execute SPARQL query
    pub fn execute_query(
        &self,
        sparql: &str,
    ) -> Result<Vec<QueryResult>, SparqlError>

    /// List all classes in ontology
    pub fn list_classes(&self) -> Result<Vec<String>, SparqlError>

    /// List all properties in ontology
    pub fn list_properties(&self) -> Result<Vec<String>, SparqlError>
}
```

**Example**:
```rust
let executor = SparqlExecutor::new(&ontology)?;
let results = executor.execute_query(
    "SELECT ?verb WHERE { ?verb a cnv:Verb }"
)?;
```

### `QueryResult`

Result from SPARQL query.

```rust
pub struct QueryResult {
    pub bindings: HashMap<String, String>,
}

impl QueryResult {
    /// Get binding by variable name
    pub fn get(&self, var: &str) -> Option<&String>

    /// Get all variable names
    pub fn variables(&self) -> impl Iterator<Item = &str>
}
```

## Error Types

### `TurtleError`

Parsing errors.

```rust
pub enum TurtleError {
    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid prefix: {0}")]
    InvalidPrefix(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### `CodeGenError`

Code generation errors.

```rust
pub enum CodeGenError {
    #[error("Parse error: {0}")]
    ParseError(#[from] TurtleError),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Missing noun reference: {0}")]
    MissingNounReference(String),

    #[error("Code generation failed: {0}")]
    GenerationFailed(String),
}
```

### `SparqlError`

Query execution errors.

```rust
pub enum SparqlError {
    #[error("Query parse error: {0}")]
    QueryParseError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Binding error: {0}")]
    BindingError(String),
}
```

## MCP Tool Schemas

### `GenerateCliFromTurtle`

**Input**:
```json
{
  "turtle_definition": "string (Turtle document)",
  "ontology_iri": "string (optional, ontology identifier)"
}
```

**Output**:
```json
{
  "rust_code": "string (generated Rust code)",
  "diagnostics": [
    {
      "level": "error|warning|info",
      "message": "string",
      "line": "number (optional)"
    }
  ]
}
```

**Errors**:
- `ParseError`: Invalid Turtle syntax
- `GenerationError`: Code generation failed
- `ValidationError`: Ontology validation failed

### `QueryCapabilities`

**Input**:
```json
{
  "sparql_query": "string (SPARQL 1.1 query)",
  "operation": "list_commands|find_verb|describe|custom"
}
```

**Output**:
```json
{
  "results": ["string", ...],
  "found": "boolean"
}
```

**Supported Operations**:
- `list_commands`: Find all verbs
- `find_verb`: Find specific verb
- `describe`: Get full description
- `custom`: Execute arbitrary SPARQL

### `ExportToTurtle`

**Input**:
```json
{
  "cli_source_code": "string (Rust CLI code)",
  "cli_name": "string (CLI name)"
}
```

**Output**:
```json
{
  "turtle_ontology": "string (RDF Turtle)"
}
```

**Status**: Future implementation

## RDF Vocabulary (cnv: Namespace)

**Namespace**: `https://cnv.dev/ontology#`

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `cnv:Noun` | (class) | - | Defines command category |
| `cnv:Verb` | (class) | - | Defines command under noun |
| `cnv:name` | Noun, Verb | xsd:string | Command name |
| `cnv:hasNoun` | Verb | Noun | Parent noun reference |
| `cnv:description` | Noun, Verb | xsd:string | Description |
| `cnv:handler` | Verb | xsd:string | Handler function name |

**Standard Prefixes**:

```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
```

## Configuration Options

### Feature Flags

```toml
[dependencies]
clap-noun-verb = { version = "5.3.4", features = [
    "rdf-composition",      # RDF/Turtle support
    "async",               # Async runtime
    "crypto",              # Cryptographic features
] }
```

### Environment Variables

```bash
RUST_LOG=debug              # Enable debug logging
CLAP_NOUN_VERB_CACHE=1     # Enable SPARQL caching
```

## Performance Notes

| Operation | Time (100 triples) | Scaling |
|-----------|-------------------|---------|
| Turtle parsing | ~18 ms | O(n) |
| SPARQL simple | ~5 ms | O(n) |
| SPARQL JOIN | ~15 ms | O(n log n) |
| Code generation | ~76 ms (100 commands) | O(n) |

## Trait Implementations

### `Display` for Error Types

All error types implement `Display` for pretty printing:

```rust
match parser.parse(turtle) {
    Ok(ontology) => println!("✓ Parsed"),
    Err(e) => eprintln!("Error: {}", e),  // Nice error message
}
```

### `Debug` for All Types

Full debug output with `{:?}`:

```rust
println!("{:?}", ontology);
```

### `Serialize`/`Deserialize`

Integration with serde:

```rust
let json = serde_json::to_string(&results)?;
```

## Thread Safety

All types are `Send` and `Sync` (safe to use across threads).

## Lifetime Parameters

Most types have no lifetime parameters, allowing for simple usage.

---

**See also**:
- [MCP Tool Schemas](#mcp-tool-schemas)
- [RDF Vocabulary](#rdf-vocabulary-cnv-namespace)
- [Error Handling](../howto/error-handling.md)

