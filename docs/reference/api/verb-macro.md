# Reference: #[verb] Macro

**File**: `clap-noun-verb-macros/src/lib.rs`

## Signature

```rust
#[verb(name, "about")]
fn command_name(arg1: Type1, arg2: Type2) -> Result<Output> { }
```

## Description

The `#[verb]` macro registers a command as a verb under its parent noun. It automatically extracts CLI arguments, validates them, and routes execution to the wrapped function.

## Parameters

### Required:
- `name` - String literal naming the verb command (e.g., `"list"`, `"create"`, `"delete"`)
- `about` - String literal describing the verb's purpose (shown in help)

### Optional:
- `"noun_name"` - Specify parent noun explicitly (inferred from `#[noun]` by default)

## Macro Expansion

The macro generates:

1. **Static registration**: Adds verb to `linkme` distributed slice
2. **Argument extraction**: Creates `HandlerInput` wrapper from CLI args
3. **Type coercion**: Converts clap values to Rust types
4. **Error handling**: Maps parse errors to `CliError`
5. **Handler dispatch**: Routes to wrapped function

## Examples

### Basic Verb

```rust
#[noun("status", "Status commands")]
#[verb("check")]
fn check_status() -> Result<Status> {
    Ok(Status::Running)
}
```

**Usage**:
```bash
$ myapp status check
```

### Verb with Arguments

```rust
#[noun("user", "User management")]
#[verb("create")]
fn create_user(username: String, email: String) -> Result<UserId> {
    // Implementation
    Ok(UserId(1))
}
```

**Usage**:
```bash
$ myapp user create alice alice@example.com
```

### Verb with Optional Arguments

```rust
#[noun("config", "Configuration")]
#[verb("set")]
fn set_config(key: String, value: Option<String>) -> Result<Output> {
    // Implementation
    Ok(Output::default())
}
```

**Usage**:
```bash
$ myapp config set debug_mode
$ myapp config set log_level info
```

### Verb with Doc Comments

```rust
/// Create a new user account
///
/// # Arguments
/// * `username` - User login name
/// * `email` - User email address
#[noun("user", "User management")]
#[verb("create")]
fn create_user(username: String, email: String) -> Result<UserId> {
    Ok(UserId(1))
}
```

## Return Type

The wrapped function must return `Result<T>` where `T` implements `Serialize`. The output is automatically JSON-serialized.

```rust
#[verb("list")]
fn list_items() -> Result<Vec<Item>> {  // OK: Vec<Item> is serializable
    Ok(vec![])
}

#[verb("get")]
fn get_item(id: u32) -> Result<Item> {  // OK: Item is serializable
    Ok(Item::default())
}

#[verb("count")]
fn count_items() -> Result<u32> {  // OK: u32 is serializable
    Ok(0)
}
```

## Handler Input Structure

The macro provides access to CLI arguments via `HandlerInput` (internally created):

```rust
#[verb("process")]
fn process(input_file: Option<String>) -> Result<ProcessResult> {
    // `input_file` parameter automatically extracted from CLI
    match input_file {
        Some(file) => /* process file */,
        None => /* use stdin */,
    }
}
```

## Error Handling

The wrapped function should return errors as `Result<T, E>` where `E` implements `Into<CliError>`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("Invalid email")]
    InvalidEmail,
}

#[verb("create")]
fn create_user(email: String) -> Result<UserId, UserError> {
    if !email.contains('@') {
        return Err(UserError::InvalidEmail);
    }
    Ok(UserId(1))
}
```

## Doc Comment Syntax

Verbs support Typer-like doc comment tags for argument relationships:

```rust
/// Export data in multiple formats
///
/// # Arguments
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
/// * `output` - Output file [requires: format]
#[verb("export")]
fn export_data(json: bool, yaml: bool, output: Option<String>) -> Result<Output> {
    // Implementation
}
```

### Supported Tags:
- `[group: name]` - Argument belongs to exclusive group
- `[requires: arg]` - Argument requires another
- `[conflicts: arg]` - Argument conflicts with another
- `[env: VAR]` - Read from environment variable
- `[default: value]` - Default value
- `[value_hint: type]` - Completion hint
- `[hide]` - Hide from help
- `[help_heading: name]` - Group in help output
- `[global]` - Propagate to subcommands
- `[exclusive]` - Cannot combine with other args

## Async Functions

Verbs support async handlers via `tokio::main`:

```rust
#[verb("fetch")]
async fn fetch_data(url: String) -> Result<Data> {
    let response = reqwest::get(&url).await?;
    let data = response.json().await?;
    Ok(data)
}
```

## Advanced: Multiple Verbs Under One Noun

```rust
#[noun("database", "Database operations")]
#[verb("create")]
fn db_create(name: String) -> Result<DatabaseId> { }

#[noun("database", "Database operations")]
#[verb("drop")]
fn db_drop(name: String) -> Result<Output> { }

#[noun("database", "Database operations")]
#[verb("list")]
fn db_list() -> Result<Vec<Database>> { }
```

## See Also

- `#[noun]` - Parent command container
- `HandlerInput` - CLI argument access
- `Result<T>` - Required return type
- Doc Comment Tags - Relationship declarations
