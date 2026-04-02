# Domain Separation Template

Use this template to start new CLI projects with proper domain logic separation.

## Quick Start

```bash
# Copy template
cp -r docs/examples/domain-separation/template my-new-cli

# Update Cargo.toml
cd my-new-cli
# Edit name, authors, description

# Start implementing
cargo build
cargo test
```

## Project Structure

```
my-cli/
├── Cargo.toml           # Dependencies and metadata
├── README.md            # Project documentation
├── src/
│   ├── main.rs         # Entry point (minimal)
│   ├── cli/            # CLI layer
│   │   ├── mod.rs
│   │   └── commands.rs  # Command implementations
│   ├── domain/         # Domain layer
│   │   ├── mod.rs
│   │   └── logic.rs    # Business logic
│   └── lib.rs          # Library root (optional)
├── tests/              # Integration tests
│   └── integration_test.rs
└── examples/           # Usage examples
    └── basic.rs
```

## Layer Responsibilities

### Domain Layer (`src/domain/`)
**What goes here:**
- Business logic
- Data transformations
- Validation rules
- Pure functions
- Domain models (structs, enums)
- Error types

**What does NOT go here:**
- `std::path::PathBuf`
- File I/O (`File::open`, `File::create`)
- Network I/O (`reqwest`, HTTP calls)
- User output (`println!`, progress bars)
- CLI arguments parsing

**Example:**
```rust
// src/domain/logic.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}

pub struct Input {
    pub data: String,
}

pub struct Output {
    pub result: String,
}

/// Pure business logic - takes data, returns data
pub fn process(input: Input) -> Result<Output, DomainError> {
    if input.data.is_empty() {
        return Err(DomainError::ValidationFailed("Empty input".to_string()));
    }

    let result = input.data.to_uppercase();

    Ok(Output { result })
}
```

### CLI Layer (`src/cli/`)
**What goes here:**
- Argument parsing
- File I/O
- Network requests
- User output (println!, progress)
- Error conversion (domain → CLI)
- Configuration loading

**What does NOT go here:**
- Business logic
- Data validation (beyond parsing)
- Complex transformations
- Algorithmic code

**Example:**
```rust
// src/cli/commands.rs

use crate::domain::{self, Input};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn process_file(input_path: PathBuf) -> Result<()> {
    // CLI: Load file
    let data = std::fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read: {:?}", input_path))?;

    // Domain: Process
    let input = Input { data };
    let output = domain::process(input)
        .context("Processing failed")?;

    // CLI: Display result
    println!("✓ Result: {}", output.result);

    Ok(())
}
```

### Main Entry (`src/main.rs`)
**Keep it minimal:**
```rust
mod cli;
mod domain;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    Process {
        #[arg(short, long)]
        input: std::path::PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process { input } => {
            cli::commands::process_file(input)?;
        }
    }

    Ok(())
}
```

## Testing Strategy

### Domain Tests (Fast, Isolated)
```rust
// src/domain/logic.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_success() {
        // Arrange
        let input = Input { data: "hello".to_string() };

        // Act
        let output = process(input).unwrap();

        // Assert
        assert_eq!(output.result, "HELLO");
    }

    #[test]
    fn test_process_empty_input_fails() {
        // Arrange
        let input = Input { data: "".to_string() };

        // Act
        let result = process(input);

        // Assert
        assert!(result.is_err());
    }
}
```

### CLI Tests (Integration)
```rust
// tests/integration_test.rs

use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_process_file_command() {
    // Arrange
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "test data").unwrap();

    // Act
    let result = cli::commands::process_file(input_file.path().to_path_buf());

    // Assert
    assert!(result.is_ok());
}
```

## Checklist for New Features

When adding a new feature, ask:

### Domain Layer
- [ ] Does this function take data (not paths)?
- [ ] Does it return Result<Data, DomainError>?
- [ ] Is it pure (no I/O, no randomness)?
- [ ] Can I test it with in-memory data?
- [ ] Does it have no CLI dependencies?

### CLI Layer
- [ ] Does this function only handle I/O?
- [ ] Does it delegate logic to domain?
- [ ] Does it format output for users?
- [ ] Does it convert domain errors to CLI errors?

### Tests
- [ ] Are domain tests fast (< 10ms each)?
- [ ] Do domain tests use AAA pattern?
- [ ] Do domain tests verify state changes?
- [ ] Are CLI tests integration tests?
- [ ] Do CLI tests use temp files/mocks?

## Common Patterns

### Pattern 1: Streaming Data
```rust
// Domain: Generic over I/O
pub fn process_stream<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
) -> Result<Stats, DomainError>

// CLI: Concrete files
pub fn process_file(input: PathBuf, output: PathBuf) -> Result<()> {
    let reader = BufReader::new(File::open(input)?);
    let mut writer = BufWriter::new(File::create(output)?);
    domain::process_stream(reader, &mut writer)?;
    Ok(())
}
```

### Pattern 2: Configuration
```rust
// Domain: Config type
pub struct Config {
    pub threshold: f64,
    pub max_retries: u32,
}

pub fn process(input: Input, config: &Config) -> Result<Output, DomainError>

// CLI: Load and parse config
pub fn process_cmd(input_path: PathBuf, config_path: PathBuf) -> Result<()> {
    let config = load_config(&config_path)?;
    let input = load_input(&input_path)?;
    let output = domain::process(input, &config)?;
    println!("✓ Done");
    Ok(())
}
```

### Pattern 3: Multiple Outputs
```rust
// Domain: Enum for format
pub enum OutputFormat {
    Json,
    Csv,
    Markdown,
}

pub fn format_output(data: &Data, format: OutputFormat) -> Result<String, DomainError>

// CLI: Parse format string
pub fn export_cmd(input: PathBuf, format: String) -> Result<()> {
    let output_format = parse_format(&format)?;
    let data = load_data(&input)?;
    let formatted = domain::format_output(&data, output_format)?;
    println!("{}", formatted);
    Ok(())
}
```

## Dependencies

### Cargo.toml Template
```toml
[package]
name = "my-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI parsing
clap = { version = "4.4", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Serialization (if needed)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
# Testing
tempfile = "3.8"
proptest = "1.4"        # Property testing
insta = "1.34"          # Snapshot testing
```

## Getting Help

- **Data Processor Example** - Streaming, CSV processing
- **API Client Example** - Async, retries, circuit breaker
- **Report Generator Example** - Multiple formats, aggregations
- **Anti-Patterns** - What NOT to do

Read the READMEs in each example for detailed explanations.
