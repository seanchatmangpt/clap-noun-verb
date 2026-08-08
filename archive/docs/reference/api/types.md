# Reference: Core Types

**File**: `src/lib.rs`

## HandlerInput

The wrapper type that provides CLI argument access to verb handlers.

**Signature**:
```rust
pub struct HandlerInput {
    pub args: clap_noun_verb::ArgMatches,  // Re-exported from clap
    pub context: Option<AppContext>,
}
```

**Usage**:
```rust
#[verb("process")]
fn process(input_file: String, verbose: bool) -> Result<Output> {
    // Macro automatically extracts `input_file` and `verbose` from CLI
    // HandlerInput is created internally by macro
    Ok(Output::default())
}
```

**Internal Details**:
- Created by `#[verb]` macro during expansion
- Contains parsed clap `ArgMatches`
- Holds optional application context
- Argument values extracted via clap APIs

---

## HandlerOutput

Represents command execution output.

**Signature**:
```rust
pub struct HandlerOutput {
    pub data: serde_json::Value,
    pub status_code: u32,
}
```

**Implementation**:
```rust
impl HandlerOutput {
    pub fn from_data<T: Serialize>(data: T) -> Result<Self> {
        Ok(HandlerOutput {
            data: serde_json::to_value(data)?,
            status_code: 0,
        })
    }

    pub fn success() -> Self {
        HandlerOutput {
            data: json!({}),
            status_code: 0,
        }
    }
}
```

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

Application-level context passed to handlers.

**Signature**:
```rust
pub struct AppContext {
    pub name: String,
    pub version: String,
    pub metadata: HashMap<String, String>,
}
```

**Usage**:
```rust
#[verb("info")]
fn show_info() -> Result<AppInfo> {
    // Context available via HandlerInput if set
    Ok(AppInfo {
        app_name: "myapp".to_string(),
    })
}
```

---

## ArgMetadata

Metadata about function arguments (internal use).

**Signature**:
```rust
pub struct ArgMetadata {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub value_name: Option<String>,
    pub help: Option<String>,
    pub group: Option<String>,
    pub requires: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub is_flag: bool,
    pub is_global: bool,
}
```

---

## CommandRegistry

Registry of all commands available in the CLI.

**Signature**:
```rust
pub struct CommandRegistry {
    commands: HashMap<String, CommandDefinition>,
}

impl CommandRegistry {
    pub fn get(&self, name: &str) -> Option<&CommandDefinition> { }
    pub fn list_all(&self) -> Vec<&CommandDefinition> { }
    pub fn find_by_verb(&self, verb: &str) -> Vec<&CommandDefinition> { }
}
```

**Usage**:
```rust
// Built automatically by macro registration
// Available via clap_noun_verb::CommandRegistry
```

---

## CommandDefinition

Definition of a single command.

**Signature**:
```rust
pub struct CommandDefinition {
    pub noun: String,
    pub verb: String,
    pub about: String,
    pub args: Vec<ArgMetadata>,
    pub handler: fn(HandlerInput) -> Result<HandlerOutput>,
}
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

