# Data Processor Example - Domain Separation Pattern

This example demonstrates **production-ready domain logic separation** in a realistic data processing CLI.

## Architecture

```
src/
├── cli/           # CLI layer - thin wrappers
│   └── commands.rs    - Argument parsing, file I/O, user output
├── domain/        # Domain layer - pure business logic
│   └── transform.rs   - Data transformation, validation, streaming
└── main.rs        # Entry point
```

## Key Patterns

### 1. Domain Layer (Pure Logic)
- **Zero CLI dependencies** - no PathBuf, no clap, no println!
- **Generic over I/O** - testable with in-memory buffers
- **Type-first design** - invalid states unrepresentable
- **Result-based errors** - no panics

### 2. CLI Layer (Thin Wrapper)
- **File handling only** - opens/closes files
- **Error conversion** - domain errors → CLI errors
- **User output** - formatting, progress, messages
- **Delegates to domain** - no business logic

### 3. Chicago TDD
- **State-based testing** - verify outputs, not implementation
- **Real collaborators** - no mocks, use real types
- **AAA pattern** - Arrange, Act, Assert
- **Observable outputs** - test what code does

## Type-First Design

```rust
// Domain function - pure, testable
pub fn transform_record(
    record: Record,
    config: &TransformConfig,
) -> Result<TransformedRecord, TransformError>

// CLI function - thin wrapper
pub fn process(
    input_path: PathBuf,
    output_path: PathBuf,
    scale: Option<f64>,
    multiplier: Option<f64>,
) -> Result<()>
```

**Key insight**: Domain function takes **data**, CLI function takes **paths**.

## Running the Example

```bash
# Create sample input
echo "id,name,value,category" > input.csv
echo "1,Test Item,10.0,A" >> input.csv
echo "2,Example,20.0,B" >> input.csv

# Run processor
cargo run -- process --input input.csv --output output.csv --scale 2.0

# Run tests
cargo test
```

## What Makes This Production-Ready?

1. **Streaming processing** - handles large files efficiently
2. **Error handling** - continues on errors, reports stats
3. **Observable outputs** - ProcessingStats for monitoring
4. **Zero-cost abstractions** - generic readers/writers
5. **Comprehensive tests** - unit + integration coverage

## Testing Strategy

### Domain Layer Tests
```rust
#[test]
fn test_transform_record_success() {
    // Arrange - pure data
    let record = Record { id: 1, name: "Test".to_string(), value: 10.0, category: "A".to_string() };
    let config = TransformConfig::default();

    // Act - call domain function
    let result = transform_record(record, &config);

    // Assert - verify state changes
    assert!(result.is_ok());
    assert_eq!(result.unwrap().scaled_value, 10.0);
}
```

### CLI Layer Tests
```rust
#[test]
fn test_process_command_success() {
    // Arrange - create temp files
    let input_file = create_temp_csv();
    let output_file = NamedTempFile::new().unwrap();

    // Act - call CLI function
    let result = process(input_file.path(), output_file.path(), Some(2.0), Some(5.0));

    // Assert - verify files created
    assert!(result.is_ok());
    assert!(output_file.path().exists());
}
```

## Anti-Patterns Avoided

❌ **Wrong**: Domain function takes PathBuf
```rust
// BAD - couples domain to file system
pub fn transform(input_path: PathBuf, output_path: PathBuf) -> Result<()>
```

✅ **Right**: Domain function takes readers/writers
```rust
// GOOD - generic, testable
pub fn process_stream<R: BufRead, W: Write>(reader: R, writer: &mut W) -> Result<Stats>
```

❌ **Wrong**: CLI logic in domain
```rust
// BAD - println! in domain layer
pub fn transform_record(record: Record) -> Result<TransformedRecord> {
    println!("Processing record {}", record.id);  // CLI concern!
    // ...
}
```

✅ **Right**: Domain returns data, CLI formats
```rust
// GOOD - domain returns stats
pub fn process_stream(...) -> Result<ProcessingStats>

// CLI formats for user
let stats = process_stream(...)?;
println!("Processed: {}", stats.processed);
```
