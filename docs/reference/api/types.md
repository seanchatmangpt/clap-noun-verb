# Reference: Core Types

**File**: `src/lib.rs`

## HandlerInput

The wrapper type that provides CLI argument access to verb handlers.

**Signature**:
```rust
pub struct HandlerInput {
    pub args: clap::ArgMatches,
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
pub type Result<T> = std::result::Result<T, CliError>;
```

---

## CliError

Error type for CLI operations.

**Signature**:
```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(String),
}
```

**Usage**:
```rust
use clap_noun_verb::CliError;

#[verb("validate")]
fn validate(email: String) -> Result<ValidateResult> {
    if !email.contains('@') {
        return Err(CliError::ValidationError("Invalid email".to_string()));
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
- CliError - Error handling
- Serializable Types - Output serialization
- serde::Serialize - Trait for JSON serialization
