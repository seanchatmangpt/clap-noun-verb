# Reference: Core Types

**File**: `src/lib.rs`

## HandlerInput

Input to a `CommandHandler`, holding CLI arguments already validated by the
CLI layer (`src/logic/handler.rs`).

**Signature**:
```rust
pub struct HandlerInput {
    /// Validated arguments as key-value pairs
    pub args: std::collections::HashMap<String, String>,
    /// Validated options as key-value pairs
    pub opts: std::collections::HashMap<String, String>,
    /// Context information (noun, verb names, etc.)
    pub context: HandlerContext,
}

pub struct HandlerContext {
    pub noun: Option<String>,
    pub verb: String,
    pub data: std::collections::HashMap<String, String>,
}
```

**Internal Details**:
- Consumed by the `CommandHandler` trait's `execute(&self, input: HandlerInput) -> Result<HandlerOutput>`
- Arguments and options are already validated string key-value pairs, not raw clap `ArgMatches`
- `HandlerContext::new`/`with_noun` build the context fluently

---

## HandlerOutput

Output from a `CommandHandler` (`src/logic/handler.rs`); data is auto-serialized
to JSON for agent/MCP consumption.

**Signature**:
```rust
pub struct HandlerOutput {
    /// Result data (auto-serialized to JSON)
    pub data: serde_json::Value,
    /// Success message (optional)
    pub message: Option<String>,
}
```

**Implementation**:
```rust
impl HandlerOutput {
    pub fn from_data<T: serde::Serialize>(data: T) -> Result<Self> { /* ... */ }
    pub fn with_message(mut self, message: String) -> Self { /* ... */ }
    pub fn to_json(&self) -> Result<String> { /* ... */ }
}
```

There is no `status_code` field and no `success()` constructor -- success is
implicit (an `Ok(HandlerOutput)` return); failures propagate as `Err(NounVerbError)`.

**Usage**:
```rust
#[verb("create")]
fn create_user(name: String) -> Result<User> {
    // Macro automatically wraps return value
    Ok(User { id: 1, name })
}
// Output: {"id": 1, "name": "Alice"}
```

---

## Result<T>

Standard Rust `Result` type used for error handling.

**Usage in Verbs**:
```rust
#[verb("process")]
fn process(file: String) -> Result<ProcessResult> {
    // Return Ok for success
    Ok(ProcessResult { lines: 42 })
}

// With custom error type
#[verb("parse")]
fn parse(input: String) -> Result<ParsedData, ParseError> {
    Err(ParseError::InvalidSyntax)?
}
```

**Type Alias**:
```rust
pub type Result<T> = std::result::Result<T, NounVerbError>;
```

---

## NounVerbError

Error type for CLI operations.

**Signature**:
```rust
#[derive(Error, Debug)]
pub enum NounVerbError {
    #[error("Command '{noun}' not found{suggestion}")]
    CommandNotFound { noun: String, suggestion: String },

    #[error("Verb '{verb}' not found for noun '{noun}'{suggestion}")]
    VerbNotFound { noun: String, verb: String, suggestion: String },

    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },

    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    #[error("Telemetry error: {0}")]
    TelemetryError(String),

    #[error("Error: {0}")]
    Generic(String),
}
```

**Usage**:
```rust
use clap_noun_verb::NounVerbError;

#[verb("validate")]
fn validate(email: String) -> Result<ValidateResult> {
    if !email.contains('@') {
        return Err(NounVerbError::ValidationFailed("Invalid email".to_string()));
    }
    Ok(ValidateResult { valid: true })
}
```

---

## AppContext

Type-erased, thread-safe application context (`src/context.rs`) -- not a
plain name/version/metadata struct. Holds arbitrary `Send + Sync + 'static`
values behind an `Arc<RwLock<ContextData>>`.

**Signature**:
```rust
pub struct AppContext {
    state: Arc<RwLock<ContextData>>, // private; no public name/version/metadata fields
}

impl AppContext {
    pub fn insert<T: Send + Sync + 'static>(&self, value: T) -> Result<(), ContextError> { /* ... */ }
    pub fn with<T, F, R>(&self, f: F) -> Result<R, ContextError>
    where
        F: FnOnce(&T) -> R,
        T: Send + Sync + 'static,
    { /* ... */ }
}
```

**Usage**:
```rust
let context = AppContext::default();
context.insert(MyConfig { threshold: 42 })?;
let value = context.with::<MyConfig, _, _>(|cfg| cfg.threshold)?;
```

---

## ArgMetadata

Metadata about a registered `#[arg]`-declared argument (`src/cli/registry.rs`).
The real struct has ~20 fields covering validation, clap wiring, and
telemetry -- the ones below are the ones most commonly read; see
`src/cli/registry.rs` for the full list (which also includes `min_value`,
`max_value`, `min_length`, `max_length`, `env`, `multiple`, `positional`,
`action`, `group`, `requires`, `conflicts_with`, and more).

**Signature (selected fields)**:
```rust
pub struct ArgMetadata {
    pub name: String,
    pub required: bool,
    pub is_flag: bool,
    pub help: Option<String>,
    pub short: Option<char>,
    pub default_value: Option<String>,
    pub value_name: Option<String>,
    pub aliases: Vec<String>,
    // ...and ~10 more validation/clap-wiring fields; see src/cli/registry.rs
}
```

There is no `long` or `is_global` field -- the long flag is derived from
`name`, and there is no per-argument global-flag concept in this struct.

A second, differently-shaped `ArgMetadata` also exists in
`clap-noun-verb-macros/src/rdf_generation.rs`, used only for RDF/ontology
projection during macro expansion -- do not confuse the two.

---

## CommandRegistry

Registry of all noun commands available in the CLI. Nouns are typically registered
automatically via the `#[verb]` macro's `linkme` distributed slice; `register_noun` /
`register_nouns` are the manual escape hatch (see [Verb Macro](verb-macro.md)).

**Signature (selected methods)**:
```rust
pub struct CommandRegistry { /* ... */ }

impl CommandRegistry {
    pub fn new() -> Self
    pub fn register_noun(self, noun: impl NounCommand + 'static) -> Self
    pub fn register_nouns<I>(self, nouns: I) -> Self
    pub fn get_noun(&self, name: &str) -> Option<&dyn NounCommand>
    pub fn noun_names(&self) -> Vec<&str>
    pub fn has_noun(&self, name: &str) -> bool
    pub fn command_structure(&self) -> HashMap<String, Vec<String>>
    pub fn validate(&self) -> Result<()>
    pub fn build_command(&self) -> Command
    pub fn route(&self, matches: &ArgMatches) -> Result<()>
    pub fn run(self) -> Result<()>
    pub fn run_with_args(self, args: Vec<String>) -> Result<()>
}
```

**Usage**:
```rust
// Built automatically by macro registration
// Available via clap_noun_verb::CommandRegistry
```

---

## Serializable Types

Return types from verbs must implement `serde::Serialize`.

**Built-in support**:
- Primitive types: `u32`, `i32`, `f64`, `bool`, `String`
- Collections: `Vec<T>`, `HashMap<K, V>`, `Option<T>`
- Structs with `#[derive(Serialize)]`
- Enums with `#[derive(Serialize)]`

**Example**:
```rust
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    username: String,
    email: String,
}

#[verb("create")]
fn create_user(username: String, email: String) -> Result<User> {
    Ok(User {
        id: 1,
        username,
        email,
    })
}
```

## OutputFormat

Enum representing the supported output rendering styles for CLI execution results.

**Signature**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Compact JSON (machine-readable)
    Json,
    /// Pretty-printed JSON (human-readable; default)
    #[default]
    JsonPretty,
    /// YAML format
    Yaml,
    /// Pretty-printed ASCII table format
    Table,
    /// Plain text (key: value pairs)
    Plain,
    /// Tab-separated values
    Tsv,
    /// Quiet mode (silences stdout output entirely)
    Quiet,
}
```

**Methods**:
- `pub fn format<S: Serialize>(self, value: &S) -> Result<String, Box<dyn std::error::Error>>`: Formats a serializable value into the selected output format.
- `pub fn available_formats() -> &'static [&'static str]`: Returns a slice of static string slices representing all valid format flags (`"json"`, `"json-pretty"`, `"yaml"`, `"table"`, `"plain"`, `"tsv"`, `"quiet"`).
- `pub fn description(&self) -> &'static str`: Returns a human-readable description of the output format variant.

**Usage**:
```rust
use clap_noun_verb::format::OutputFormat;

let data = vec![("key", "value")];
let formatted = OutputFormat::Yaml.format(&data)?;
println!("{}", formatted);
```

**Quiet Mode**:
When `OutputFormat::Quiet` is specified, the CLI execution engine will suppress all standard output printing to standard out (`stdout`), returning an empty string. This is designed for high-performance automation scripting or CI/CD pipelines where outputs should be kept silent unless an error occurs.

---

## Type Conversions

**From CLI to Rust**:
- `String` ← CLI argument
- `u32` ← parsed CLI argument
- `bool` ← flag presence
- `Option<T>` ← optional CLI argument
- `Vec<T>` ← repeated CLI arguments

**From Rust to Output**:
- All `Serialize` types → JSON
- Errors → CLI error output
- Status codes → exit codes

---

## See Also

- Result<T> - Return type requirements
- NounVerbError - Error handling
- Serializable Types - Output serialization
- serde::Serialize - Trait for JSON serialization
- OutputFormat - Output formatting system

