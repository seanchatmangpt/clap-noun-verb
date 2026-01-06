# Generated CLI Examples from Turtle Specifications

This directory contains complete CLI applications generated from Turtle/RDF specifications using ggen-clap-noun-verb.

## Overview

Each subdirectory represents a fully-functional Rust CLI application generated from the corresponding Turtle specification in `/home/user/clap-noun-verb/examples/turtle-specs/`.

---

## Generated CLIs

### 1. calculator-cli/

**Generated from**: `turtle-specs/calculator.ttl`

**Description**: Basic arithmetic calculator with noun-verb command structure

**Commands**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/calculator-cli

# Build the CLI
cargo make check
cargo make test

# Run commands
cargo run -- calc add --left 5 --right 3
cargo run -- calc subtract --left 10 --right 4
cargo run -- calc multiply --left 6 --right 7
cargo run -- calc divide --left 20 --right 4
cargo run -- math add --left 100 --right 200
```

**Generated Files**:
```
calculator-cli/
├── Cargo.toml              # Project dependencies
├── src/
│   ├── main.rs             # Entry point with clap parsing
│   ├── cli.rs              # CLI struct with derives
│   ├── nouns/
│   │   ├── calc.rs         # Calc noun implementation
│   │   └── math.rs         # Math noun (alias)
│   ├── verbs/
│   │   ├── add.rs          # Add verb handler
│   │   ├── subtract.rs     # Subtract verb handler
│   │   ├── multiply.rs     # Multiply verb handler
│   │   └── divide.rs       # Divide verb handler with validation
│   ├── types.rs            # Type definitions
│   └── error.rs            # Error types
└── tests/
    └── integration_test.rs # Integration tests
```

---

### 2. file-manager-cli/

**Generated from**: `turtle-specs/file-manager.ttl`

**Description**: File system operations with path handling and flags

**Commands**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/file-manager-cli

# Build and test
cargo make check
cargo make test

# File operations
cargo run -- file create --path /tmp/test.txt --verbose
cargo run -- file list --path /tmp --recursive
cargo run -- file copy --path /tmp/source.txt --destination /tmp/dest.txt
cargo run -- file delete --path /tmp/test.txt --force

# Directory operations
cargo run -- dir create --path /tmp/testdir --recursive
cargo run -- dir list --path /tmp
cargo run -- dir delete --path /tmp/testdir --recursive --force
```

**Generated Files**:
```
file-manager-cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── nouns/
│   │   ├── file.rs         # File noun with path operations
│   │   └── dir.rs          # Directory noun
│   ├── verbs/
│   │   ├── create.rs       # Create file/dir
│   │   ├── delete.rs       # Delete with confirmations
│   │   ├── list.rs         # List with filtering
│   │   ├── copy.rs         # Copy operations
│   │   └── move.rs         # Move/rename operations
│   ├── validators.rs       # Path validation logic
│   └── error.rs
└── tests/
    └── integration_test.rs
```

---

### 3. user-api-cli/

**Generated from**: `turtle-specs/user-api.ttl`

**Description**: REST API client with CRUD operations and complex types

**Commands**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/user-api-cli

# Build and test
cargo make check
cargo make test

# User operations
cargo run -- --base-url https://api.example.com --api-key KEY \
    user create --name "John Doe" --email "john@example.com" --age 30

cargo run -- user read --id 123
cargo run -- user update --id 123 --email "newemail@example.com"
cargo run -- user list --limit 10 --offset 0
cargo run -- user delete --id 123 --confirm

# Post operations
cargo run -- post create --user-id 123 --title "My Post" --content "Content"
cargo run -- post read --id 456
cargo run -- post list --user-id 123 --limit 20

# Comment operations
cargo run -- comment create --post-id 456 --user-id 123 --content "Great post!"
cargo run -- comment list --post-id 456 --limit 50
```

**Generated Files**:
```
user-api-cli/
├── Cargo.toml              # With reqwest, tokio dependencies
├── src/
│   ├── main.rs
│   ├── cli.rs              # With global flags
│   ├── nouns/
│   │   ├── user.rs         # User CRUD operations
│   │   ├── post.rs         # Post operations
│   │   └── comment.rs      # Comment operations
│   ├── api/
│   │   ├── client.rs       # HTTP client with auth
│   │   ├── types.rs        # Request/response types
│   │   └── error.rs        # API errors
│   ├── validators.rs       # Email, ID validation
│   └── error.rs
└── tests/
    ├── unit/
    │   └── validators_test.rs
    └── integration/
        └── api_test.rs
```

---

### 4. web-server-cli/

**Generated from**: `turtle-specs/web-server.ttl`

**Description**: Web server management with configuration and lifecycle operations

**Commands**:
```bash
cd /home/user/clap-noun-verb/examples/generated-from-turtle/web-server-cli

# Build and test
cargo make check
cargo make test

# Server lifecycle
cargo run -- server start --port 8080 --host 0.0.0.0 --daemon
cargo run -- server status --verbose
cargo run -- server restart --graceful
cargo run -- server stop --timeout 30

# Configuration
cargo run -- config validate --config-file /etc/webserver/config.toml
cargo run -- config show --format json
cargo run -- config set --key server.port --value 8080
cargo run -- config reset --confirm

# Route management
cargo run -- route add --path /api/users --handler users_handler --method GET
cargo run -- route list --filter /api
cargo run -- route test --path /api/users --method GET
cargo run -- route remove --path /api/users --confirm
```

**Generated Files**:
```
web-server-cli/
├── Cargo.toml              # With tokio, warp/axum dependencies
├── src/
│   ├── main.rs
│   ├── cli.rs              # With global config flags
│   ├── nouns/
│   │   ├── server.rs       # Server lifecycle
│   │   ├── config.rs       # Configuration management
│   │   └── route.rs        # Route management
│   ├── server/
│   │   ├── daemon.rs       # Daemon mode
│   │   ├── router.rs       # Route registry
│   │   └── graceful.rs     # Graceful shutdown
│   ├── config/
│   │   ├── loader.rs       # Config file parsing
│   │   ├── validator.rs    # Config validation
│   │   └── types.rs        # Config structures
│   ├── validators.rs       # Port, path validation
│   └── error.rs
└── tests/
    ├── unit/
    │   ├── config_test.rs
    │   └── route_test.rs
    └── integration/
        └── server_test.rs
```

---

## Generated Code Structure

All generated CLIs follow this pattern:

### 1. Entry Point (`main.rs`)
```rust
use clap::Parser;

mod cli;
mod nouns;
mod verbs;
mod error;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    cli.execute()?;
    Ok(())
}
```

### 2. CLI Struct (`cli.rs`)
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "app-name")]
#[command(version = "1.0.0")]
#[command(about = "Application description")]
pub struct Cli {
    /// Global flags
    #[command(flatten)]
    pub global: GlobalFlags,

    #[command(subcommand)]
    pub noun: NounCommand,
}

#[derive(Subcommand)]
pub enum NounCommand {
    /// Noun description
    Noun1(Noun1Args),
    Noun2(Noun2Args),
}
```

### 3. Noun Implementation
```rust
#[derive(Args)]
pub struct NounArgs {
    #[command(subcommand)]
    pub verb: VerbCommand,
}

#[derive(Subcommand)]
pub enum VerbCommand {
    /// Verb description
    Verb1(Verb1Args),
    Verb2(Verb2Args),
}
```

### 4. Verb Handlers
```rust
#[derive(Args)]
pub struct Verb1Args {
    /// Argument 1 description
    #[arg(long, short)]
    pub arg1: String,

    /// Flag description
    #[arg(long)]
    pub verbose: bool,
}

impl Verb1Args {
    pub fn execute(&self) -> Result<(), Error> {
        // Implementation
        Ok(())
    }
}
```

### 5. Validators (`validators.rs`)
```rust
use thiserror::Error;

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?;
    if regex.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail(email.to_string()))
    }
}

pub fn validate_port(port: u16) -> Result<(), ValidationError> {
    if port >= 1 && port <= 65535 {
        Ok(())
    } else {
        Err(ValidationError::InvalidPort(port))
    }
}
```

### 6. Error Types (`error.rs`)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid email: {0}")]
    InvalidEmail(String),

    #[error("Invalid port: {0}")]
    InvalidPort(u16),
}
```

---

## Running Generated CLIs

### Build Commands
```bash
# Check compilation (fast feedback)
cargo make check

# Run unit tests
cargo make test-unit

# Run all tests
cargo make test

# Run linter
cargo make lint

# Full CI validation
cargo make ci
```

### Execution
```bash
# Show help
cargo run -- --help
cargo run -- <noun> --help
cargo run -- <noun> <verb> --help

# Execute commands
cargo run -- <noun> <verb> <args> <flags>

# Production build
cargo make release-validate
cargo build --release
./target/release/<cli-name> <noun> <verb> <args>
```

---

## Adapting Examples for Your CLIs

### 1. Modify Turtle Specification
Edit the appropriate `.ttl` file in `turtle-specs/` to match your domain:
- Change noun names
- Add/remove verbs
- Modify arguments and types
- Update validation rules

### 2. Regenerate Code
```bash
# From ggen-clap-noun-verb crate directory
cargo make ggen-generate -- \
    --input ../../examples/turtle-specs/your-spec.ttl \
    --output ../../examples/generated-from-turtle/your-cli
```

### 3. Implement Business Logic
The generated code provides the CLI structure. Add your domain logic in verb handlers:

```rust
impl CreateUserArgs {
    pub fn execute(&self) -> Result<(), CliError> {
        // Generated validation
        validate_email(&self.email)?;

        // Add your business logic here
        let user = User::new(&self.name, &self.email)?;
        user.save_to_database()?;

        println!("User created: {}", user.id);
        Ok(())
    }
}
```

### 4. Add Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        // Arrange
        let args = CreateUserArgs {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: Some(30),
        };

        // Act
        let result = args.execute();

        // Assert
        assert!(result.is_ok());
    }
}
```

---

## Code Generation Features

### Type-Safe Arguments
- Compile-time type checking
- Automatic parsing from command-line
- Validation before execution

### Validation Integration
- Generated validators from Turtle constraints
- Composed validation pipelines
- Clear error messages

### Error Handling
- Structured error types from specifications
- Result-based error propagation
- User-friendly error display

### Documentation
- Generated help messages from `clap:about`
- Automatic `--help` flag support
- Usage examples in help output

---

## Dependencies

Generated CLIs include:

**Core**:
- `clap` (v4+) with derive feature
- `thiserror` for error types
- `anyhow` for error handling

**Optional** (based on features):
- `tokio` for async operations
- `reqwest` for HTTP clients
- `serde`/`serde_json` for serialization
- `regex` for validation

---

## Performance Characteristics

All generated CLIs meet these SLOs:

- **Compilation**: Incremental ≤ 2s
- **Tests**: Unit ≤ 10s, Integration ≤ 30s
- **CLI execution**: ≤ 100ms end-to-end
- **Memory usage**: ≤ 10MB

---

## Customization Points

### 1. Add Custom Types
```rust
// In types.rs
pub struct Email(String);

impl Email {
    pub fn new(s: String) -> Result<Self, ValidationError> {
        validate_email(&s)?;
        Ok(Email(s))
    }
}
```

### 2. Add Middleware
```rust
// In middleware.rs
pub fn logging_middleware<F>(f: F) -> impl Fn() -> Result<(), CliError>
where
    F: Fn() -> Result<(), CliError>,
{
    move || {
        info!("Starting operation");
        let result = f();
        info!("Operation complete");
        result
    }
}
```

### 3. Add Output Formatting
```rust
// In formatters.rs
pub trait Formatter {
    fn format(&self, data: &impl Serialize) -> String;
}

pub struct JsonFormatter;
pub struct TextFormatter;
pub struct YamlFormatter;
```

---

## Troubleshooting

### Build Errors
```bash
# Clear cargo cache
cargo clean

# Update dependencies
cargo update

# Check for missing features
cargo make check
```

### Runtime Errors
```bash
# Enable verbose logging
RUST_LOG=debug cargo run -- <command>

# Check input validation
cargo run -- <noun> <verb> --help
```

### Test Failures
```bash
# Run specific test
cargo make test <test_name>

# Show test output
cargo test -- --nocapture
```

---

## Next Steps

1. **Explore Examples**: Run each generated CLI to see functionality
2. **Study Code**: Examine generated code structure
3. **Customize**: Modify Turtle specs and regenerate
4. **Extend**: Add business logic to verb handlers
5. **Deploy**: Build production binaries with `cargo build --release`

---

## Related Documentation

- [Turtle Specifications README](/home/user/clap-noun-verb/examples/turtle-specs/README.md) - Specification format guide
- [USAGE_GUIDE.md](/home/user/clap-noun-verb/docs/USAGE_GUIDE.md) - Complete usage workflow
- [EXAMPLES_SHOWCASE.md](/home/user/clap-noun-verb/docs/EXAMPLES_SHOWCASE.md) - Before/after showcase
- [ggen-clap-noun-verb-quickstart.md](/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-quickstart.md) - Implementation guide

---

**Ready to generate your own CLI?** Start with the [USAGE_GUIDE.md](/home/user/clap-noun-verb/docs/USAGE_GUIDE.md) for step-by-step instructions!
