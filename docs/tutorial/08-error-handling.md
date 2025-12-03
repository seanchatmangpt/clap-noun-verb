# Tutorial 08: Error Handling - Result Types and thiserror

**Learning Path:** Basic Error Handling → Production-Grade Errors
**Time:** 25 minutes
**Prerequisites:** [Tutorial 07: Async Operations](07-async-operations.md)

---

## What You'll Learn

How to handle errors professionally in clap-noun-verb:
- Using `Result<T, E>` everywhere
- Creating custom error types with `thiserror`
- Error propagation with `?` operator
- User-friendly error messages
- Error context and debugging

---

## The Rust Error Philosophy

**❌ Never panic in production code:**
```rust
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        panic!("Division by zero!"); // ❌ BAD: Crashes entire program
    }
    a / b
}
```

**✅ Always use Result types:**
```rust
pub fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        return Err(MathError::DivisionByZero); // ✅ GOOD: Recoverable error
    }
    Ok(a / b)
}
```

---

## Custom Error Types with thiserror

### Step 1: Define Error Enum

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Database connection failed")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Network request failed")]
    NetworkError(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

**thiserror features:**
- `#[error("...")]` - Error message template
- `{0}`, `{1}` - Positional field formatting
- `#[from]` - Automatic conversion from other error types

---

### Step 2: Use in Domain Logic

```rust
// domain/users.rs
pub fn create_user(email: &str, username: &str) -> Result<User, DomainError> {
    // Validate email
    if !email.contains('@') {
        return Err(DomainError::InvalidEmail(email.to_string()));
    }

    // Check if user exists
    if user_exists(username)? {
        return Err(DomainError::UserNotFound(username.to_string()));
    }

    // Create user
    let user = User {
        email: email.to_string(),
        username: username.to_string(),
    };

    Ok(user)
}

fn user_exists(username: &str) -> Result<bool, DomainError> {
    // Database query (sqlx::Error automatically converts to DomainError::DatabaseError)
    let exists = sqlx::query!("SELECT COUNT(*) FROM users WHERE username = $1", username)
        .fetch_one(&pool)
        .await?;

    Ok(exists.count > 0)
}
```

---

### Step 3: Handle in CLI Layer

```rust
// commands/users.rs
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserCreated {
    username: String,
    email: String,
    status: String,
}

#[verb(help = "Create a new user")]
pub fn create(
    #[arg(help = "User email address")] email: String,
    #[arg(help = "Username")] username: String,
) -> Result<UserCreated, Box<dyn std::error::Error>> {
    // Error propagates up automatically
    let user = crate::domain::users::create_user(&email, &username)?;

    Ok(UserCreated {
        username: user.username,
        email: user.email,
        status: "created".to_string(),
    })
}
```

**Error output (automatic):**
```bash
$ myapp users create --email invalid --username alice
Error: Invalid email format: invalid
```

---

## Error Propagation with ?

The `?` operator propagates errors up the call stack:

```rust
pub fn process_payment(amount: u64) -> Result<Payment, PaymentError> {
    // Each ? either:
    // - Unwraps Ok value
    // - Returns Err immediately

    let user = get_current_user()?;           // ← Returns early if error
    let balance = check_balance(&user.id)?;   // ← Returns early if error
    let payment = charge_account(amount)?;    // ← Returns early if error

    Ok(payment)
}
```

**Without ?:**
```rust
pub fn process_payment(amount: u64) -> Result<Payment, PaymentError> {
    // ❌ Verbose and error-prone
    let user = match get_current_user() {
        Ok(u) => u,
        Err(e) => return Err(e),
    };

    let balance = match check_balance(&user.id) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };

    // ...
}
```

---

## Multiple Error Types

### Pattern 1: Box<dyn Error>

Simple but less type-safe:

```rust
#[verb]
pub fn command() -> Result<Output, Box<dyn std::error::Error>> {
    // Any error type works
    let data = fetch_data()?;        // reqwest::Error
    let user = query_user()?;         // sqlx::Error
    let config = load_config()?;      // std::io::Error

    Ok(Output { data, user, config })
}
```

**Pros:** Simple, works with any error
**Cons:** Less type-safe, harder to match specific errors

---

### Pattern 2: Custom Error with From

Type-safe error handling:

```rust
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
}

#[verb]
pub fn command() -> Result<Output, CommandError> {
    // All errors convert automatically via From trait
    let data = fetch_data()?;        // reqwest::Error → CommandError::Network
    let user = query_user()?;         // sqlx::Error → CommandError::Database
    let config = load_config()?;      // std::io::Error → CommandError::Io

    Ok(Output { data, user, config })
}
```

**Pros:** Type-safe, can match specific errors
**Cons:** More code upfront

---

## Error Context

Add context to errors for debugging:

```rust
use anyhow::{Context, Result};

#[verb]
pub fn load_config(
    #[arg(help = "Config file path")] path: String,
) -> Result<ConfigData> {
    let contents = std::fs::read_to_string(&path)
        .context(format!("Failed to read config file: {}", path))?;

    let config: ConfigData = serde_json::from_str(&contents)
        .context("Failed to parse JSON config")?;

    Ok(config)
}
```

**Error output with context:**
```bash
$ myapp config load --path /missing/config.json
Error: Failed to read config file: /missing/config.json

Caused by:
    No such file or directory (os error 2)
```

---

## Exercise: Build Robust File Processor

**Goal:** Create a file processor with comprehensive error handling

**Arrange:** Define custom error types

```rust
// domain/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileProcessorError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },

    #[error("Unsupported file extension: {0}")]
    UnsupportedExtension(String),
}
```

**Act:** Implement domain logic with error handling

```rust
// domain/file_processor.rs
use super::errors::FileProcessorError;

const MAX_FILE_SIZE: u64 = 10_000_000; // 10MB

pub fn process_file(path: &str) -> Result<ProcessedFile, FileProcessorError> {
    // Check file exists
    if !std::path::Path::new(path).exists() {
        return Err(FileProcessorError::FileNotFound(path.to_string()));
    }

    // Check file size
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(FileProcessorError::FileTooLarge {
            size: metadata.len(),
            max: MAX_FILE_SIZE,
        });
    }

    // Check extension
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| FileProcessorError::UnsupportedExtension("none".to_string()))?;

    match extension {
        "json" => process_json_file(path),
        "txt" => process_text_file(path),
        _ => Err(FileProcessorError::UnsupportedExtension(extension.to_string())),
    }
}

fn process_json_file(path: &str) -> Result<ProcessedFile, FileProcessorError> {
    let contents = std::fs::read_to_string(path)?;
    let data: serde_json::Value = serde_json::from_str(&contents)?;

    Ok(ProcessedFile {
        path: path.to_string(),
        format: "json".to_string(),
        size: contents.len(),
        records: count_json_records(&data),
    })
}

fn process_text_file(path: &str) -> Result<ProcessedFile, FileProcessorError> {
    let contents = std::fs::read_to_string(path)?;

    Ok(ProcessedFile {
        path: path.to_string(),
        format: "text".to_string(),
        size: contents.len(),
        records: contents.lines().count(),
    })
}
```

**Act:** Create CLI command

```rust
// commands/files.rs
#[verb(help = "Process a file")]
pub fn process(
    #[arg(help = "File path to process")] path: String,
) -> Result<ProcessResult, Box<dyn std::error::Error>> {
    match crate::domain::file_processor::process_file(&path) {
        Ok(processed) => Ok(ProcessResult {
            path: processed.path,
            format: processed.format,
            size: processed.size,
            records: processed.records,
            status: "success".to_string(),
        }),
        Err(e) => {
            // Log error for debugging
            eprintln!("Error processing file: {}", e);

            // Return user-friendly error
            Err(e.into())
        }
    }
}
```

**Assert:** Test error cases

```bash
# File not found
$ myapp files process --path /missing/file.json
Error: File not found: /missing/file.json

# File too large
$ myapp files process --path /huge/file.json
Error: File too large: 20000000 bytes (max: 10000000 bytes)

# Unsupported format
$ myapp files process --path data.xml
Error: Unsupported file extension: xml

# Success
$ myapp files process --path data.json
{"path":"data.json","format":"json","size":1234,"records":42,"status":"success"}
```

---

## Testing Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found() {
        // Arrange
        let path = "/nonexistent/file.json";

        // Act
        let result = process_file(path);

        // Assert
        assert!(matches!(result, Err(FileProcessorError::FileNotFound(_))));
    }

    #[test]
    fn test_unsupported_extension() {
        // Arrange
        let path = "test.xml";

        // Act
        let result = process_file(path);

        // Assert
        assert!(matches!(result, Err(FileProcessorError::UnsupportedExtension(_))));
    }

    #[test]
    fn test_json_parsing_error() {
        // Arrange
        let path = "tests/fixtures/invalid.json"; // Contains malformed JSON

        // Act
        let result = process_file(path);

        // Assert
        assert!(matches!(result, Err(FileProcessorError::JsonError(_))));
    }
}
```

---

## Best Practices

### ✅ Do:
1. **Use Result types everywhere** - Never panic in production
2. **Create domain-specific errors** - Clear, actionable messages
3. **Use thiserror** - Ergonomic error definitions
4. **Add context** - Use `anyhow::Context` for debugging
5. **Test error cases** - Verify all error paths

### ❌ Don't:
1. **Don't use `unwrap()`** - Causes panics
2. **Don't use `expect()` in production** - Same as unwrap
3. **Don't swallow errors** - Always propagate or handle
4. **Don't use string errors** - Use typed errors
5. **Don't expose internal errors** - Wrap implementation details

---

## Key Takeaways

✅ **Result<T, E>** - Rust's error handling foundation
✅ **thiserror** - Ergonomic custom error types
✅ **Error propagation** - `?` operator for clean code
✅ **Error context** - anyhow for debugging information
✅ **Test error paths** - Verify error handling works

---

## Next Steps

- **[Tutorial 09: Deployment Basics](09-deployment-basics.md)** - Production deployment
- **[Tutorial 10: Next Steps](10-next-steps.md)** - Advanced topics
- **[How-To: Error Strategies](../howto/advanced/error-strategies.md)** - Production error patterns

**Estimated time to next tutorial:** 20 minutes

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
