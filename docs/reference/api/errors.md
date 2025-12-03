# Reference: Error Types

**File**: `src/error.rs`

## CliError Enum

The primary error type for clap-noun-verb operations.

**Definition**:
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

## Error Variants

### ParseError

Raised when CLI argument parsing fails.

**When It Happens**:
- Invalid argument type (e.g., "abc" as u32)
- Missing required argument
- Invalid argument format

**Example**:
```rust
#[verb("list")]
fn list_items(page: u32) -> Result<Vec<Item>> {
    // If user provides: myapp list --page abc
    // Error: Parse error: invalid digit found in string
    Ok(vec![])
}
```

### ValidationError

Raised when argument values fail validation.

**When It Happens**:
- Custom validation rules fail
- Value constraints violated
- Domain-specific rules broken

**Example**:
```rust
#[verb("create")]
fn create_user(email: String) -> Result<UserId> {
    if !email.contains('@') {
        return Err(CliError::ValidationError("Invalid email format".to_string()));
    }
    Ok(UserId(1))
}
```

### IoError

Raised during file I/O operations.

**When It Happens**:
- File not found
- Permission denied
- Disk full
- Broken pipe

**Example**:
```rust
#[verb("read")]
fn read_file(path: String) -> Result<FileContents> {
    let contents = std::fs::read_to_string(&path)?;  // Converts io::Error
    Ok(FileContents(contents))
}
```

### SerializationError

Raised when output cannot be serialized to JSON.

**When It Happens**:
- Circular references in data
- Non-serializable types in output
- serde serialization failure

**Example**:
```rust
#[verb("export")]
fn export_data() -> Result<DataToExport> {
    Ok(DataToExport {
        values: vec![1, 2, 3],  // OK: Vec<i32> is serializable
    })
}
```

### Custom

Generic error for application-specific failures.

**Usage**:
```rust
#[verb("process")]
fn process(file: String) -> Result<ProcessResult> {
    let result = do_complex_operation(&file)
        .map_err(|e| CliError::Custom(format!("Processing failed: {}", e)))?;
    Ok(result)
}
```

## Creating Custom Error Types

**With thiserror**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(u32),

    #[error("Invalid email: {0}")]
    InvalidEmail(String),

    #[error("Duplicate user: {0}")]
    DuplicateUser(String),
}

// Implement conversion to CliError
impl From<UserError> for CliError {
    fn from(err: UserError) -> Self {
        CliError::Custom(err.to_string())
    }
}

#[verb("create")]
fn create_user(email: String) -> Result<UserId, UserError> {
    if !email.contains('@') {
        return Err(UserError::InvalidEmail(email));
    }
    Ok(UserId(1))
}
```

**With anyhow**:
```rust
use anyhow::Result;

#[verb("fetch")]
fn fetch_data(url: String) -> Result<Data> {
    let response = reqwest::blocking::get(&url)?;
    let data = response.json()?;
    Ok(data)
}
```

## Error Handling Patterns

### Pattern 1: Conversion with `?` Operator

```rust
#[verb("process")]
fn process(path: String) -> Result<ProcessResult> {
    let input = std::fs::read_to_string(&path)?;  // Converts io::Error
    let parsed = serde_json::from_str::<Data>(&input)?;  // Converts serde_json::Error
    Ok(ProcessResult { count: parsed.items.len() })
}
```

### Pattern 2: Explicit Conversion with `map_err`

```rust
#[verb("validate")]
fn validate(value: String) -> Result<ValidateResult> {
    check_constraint(&value)
        .map_err(|e| CliError::ValidationError(format!("Constraint failed: {}", e)))?;
    Ok(ValidateResult { valid: true })
}
```

### Pattern 3: Custom Error Context

```rust
#[verb("import")]
fn import_data(file: String) -> Result<ImportResult> {
    let contents = std::fs::read_to_string(&file)
        .map_err(|e| CliError::Custom(
            format!("Failed to read import file '{}': {}", file, e)
        ))?;
    Ok(ImportResult { imported: 42 })
}
```

### Pattern 4: Nested Results

```rust
#[verb("complex")]
fn complex_operation(arg: String) -> Result<ComplexResult> {
    let step1 = step_one(&arg)?;
    let step2 = step_two(step1)?;
    let step3 = step_three(step2)?;
    Ok(ComplexResult { result: step3 })
}

fn step_one(arg: &str) -> Result<String> {
    Ok(arg.to_uppercase())
}

fn step_two(input: String) -> Result<String> {
    Ok(input)  // Could return Err(...)
}

fn step_three(input: String) -> Result<Vec<String>> {
    Ok(vec![input])
}
```

## Error Output Format

Errors are automatically formatted for CLI:

**Example Error Output**:
```bash
$ myapp user create invalid-email
Error: Validation error: Invalid email format
```

**Exit Codes**:
- `0` - Success
- `1` - Generic error
- `2` - Parse/usage error
- Custom codes supported via `HandlerOutput::status_code`

## HTTP Status Code Mapping

For APIs, map CLI errors to HTTP:

```rust
impl From<CliError> for u16 {
    fn from(error: CliError) -> u16 {
        match error {
            CliError::ParseError(_) => 400,      // Bad Request
            CliError::ValidationError(_) => 422, // Unprocessable Entity
            CliError::IoError(_) => 500,         // Internal Server Error
            CliError::SerializationError(_) => 500,
            CliError::Custom(_) => 500,
        }
    }
}
```

## Best Practices

1. **Be Specific**: Use concrete error variants when possible
2. **Provide Context**: Include relevant information in error messages
3. **Use Custom Types**: For domain-specific errors, create custom error enums
4. **Chain Errors**: Use `?` operator to preserve error chains
5. **User-Friendly Messages**: Write errors users can act on
6. **Document Errors**: Note what errors your command can return

## Testing Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let result = create_user("no-at-sign".to_string());
        assert!(result.is_err());
        match result {
            Err(CliError::ValidationError(msg)) => {
                assert!(msg.contains("email"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_io_error() {
        let result = read_file("/nonexistent/file");
        assert!(matches!(result, Err(CliError::IoError(_))));
    }
}
```

## See Also

- Result<T> - Return type wrapper
- thiserror - Custom error types crate
- anyhow - Flexible error handling
- Error Messages - User-facing error output
