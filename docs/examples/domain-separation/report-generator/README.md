# Report Generator Example - Multiple Output Formats

This example demonstrates **production-ready domain separation** with multiple output formats and comprehensive aggregation logic.

## Architecture

```
src/
├── cli/           # CLI layer - file I/O, format parsing
│   └── commands.rs
├── domain/        # Domain layer - aggregation, formatting
│   └── report.rs
└── main.rs
```

## Key Patterns

### 1. Domain Layer (Pure Transformations)
- **Aggregation logic** - statistical calculations
- **Multiple formatters** - JSON, CSV, Markdown
- **Type-rich models** - SalesStats, CategoryStats, ProductStats
- **Zero I/O** - takes Vec, returns String

### 2. CLI Layer (I/O Only)
- **File loading** - CSV parsing
- **File writing** - output to file or stdout
- **Format selection** - user choice
- **Summary display** - user feedback

### 3. Snapshot Testing with insta
- **Deterministic outputs** - format functions are pure
- **Visual diffs** - easy to review changes
- **Multiple formats** - test all formatters

## Type-First Design

```rust
// Domain models encode business concepts
pub struct SalesStats {
    pub total_revenue: f64,
    pub total_quantity: u32,
    pub by_category: HashMap<String, CategoryStats>,
    pub by_product: HashMap<String, ProductStats>,
}

// Pure aggregation function
pub fn aggregate_sales(records: Vec<SalesRecord>) -> Result<SalesStats, ReportError>

// Pure formatting functions
pub fn format_json(stats: &SalesStats) -> Result<String, ReportError>
pub fn format_csv(stats: &SalesStats) -> Result<String, ReportError>
pub fn format_markdown(stats: &SalesStats) -> Result<String, ReportError>
```

**Key insight**: All formatters are **pure functions** - same input always produces same output.

## Running the Example

```bash
# Create sample data
echo "id,product,category,amount,quantity,date" > sales.csv
echo "1,Laptop,Electronics,1200.0,1,2024-01-01T00:00:00Z" >> sales.csv
echo "2,Mouse,Electronics,25.0,2,2024-01-01T00:00:00Z" >> sales.csv
echo "3,Novel,Books,15.0,3,2024-01-01T00:00:00Z" >> sales.csv

# Generate JSON report
cargo run -- generate --input sales.csv --format json

# Generate Markdown report
cargo run -- generate --input sales.csv --output report.md --format markdown

# Run tests (including snapshot tests)
cargo test
```

## Chicago TDD with Snapshot Testing

### Domain Layer Tests
```rust
#[test]
fn test_aggregate_sales_success() {
    // Arrange
    let records = vec![
        SalesRecord { id: "1".to_string(), ... },
        SalesRecord { id: "2".to_string(), ... },
    ];

    // Act
    let stats = aggregate_sales(records).unwrap();

    // Assert - verify state
    assert_eq!(stats.total_revenue, 1240.0);
    assert_eq!(stats.by_category.len(), 2);
}
```

### Snapshot Tests
```rust
#[test]
fn test_format_markdown_snapshot() {
    // Arrange
    let stats = create_sample_stats();

    // Act
    let markdown = format_markdown(&stats).unwrap();

    // Assert - snapshot testing with insta
    insta::assert_snapshot!(markdown);
}
```

**Benefits:**
- Catch formatting regressions
- Visual diff of changes
- Easy to review expected output

## Production Features

### 1. Rich Aggregations
- Total revenue and quantity
- Per-category breakdowns
- Per-product statistics
- Average prices
- Time period tracking

### 2. Multiple Output Formats
- **JSON** - machine-readable, API integration
- **CSV** - Excel, data analysis
- **Markdown** - documentation, reports

### 3. Error Handling
- Empty data validation
- Format parsing errors
- File I/O errors
- Aggregation errors

### 4. Testability
- Pure aggregation functions
- Pure formatting functions
- Snapshot testing for outputs
- State-based assertions

## Anti-Patterns Avoided

❌ **Wrong**: Formatting in CLI
```rust
// BAD - formatting logic in CLI
pub fn generate(input: PathBuf, output: PathBuf) -> Result<()> {
    let stats = aggregate(...)?;

    let mut output = String::new();
    output.push_str("# Report\n");  // Formatting in CLI!
    output.push_str(&format!("Total: {}\n", stats.total));
    // ...
}
```

✅ **Right**: Domain handles formatting
```rust
// GOOD - domain formats, CLI writes
pub fn generate(input: PathBuf, output: PathBuf, format: String) -> Result<()> {
    let stats = domain::aggregate(...)?;
    let report = domain::format_report(&stats, format)?;  // Domain formats!
    std::fs::write(output, report)?;  // CLI writes!
}
```

❌ **Wrong**: File paths in domain
```rust
// BAD - domain knows about files
pub fn generate_report(input_path: PathBuf, format: Format) -> Result<String>
```

✅ **Right**: Domain takes data
```rust
// GOOD - domain takes data structures
pub fn aggregate_sales(records: Vec<SalesRecord>) -> Result<SalesStats>
pub fn format_report(stats: &SalesStats, format: Format) -> Result<String>
```

## Extending the Example

### Adding New Formats
```rust
// 1. Add variant to domain enum
pub enum ReportFormat {
    Json,
    Csv,
    Markdown,
    Html,  // New!
}

// 2. Implement pure formatter in domain
pub fn format_html(stats: &SalesStats) -> Result<String, ReportError> {
    // Pure formatting logic
}

// 3. Add to dispatcher
pub fn format_report(stats: &SalesStats, format: ReportFormat) -> Result<String> {
    match format {
        ReportFormat::Html => format_html(stats),
        // ...
    }
}

// 4. CLI parses format
fn parse_format(s: &str) -> Result<ReportFormat> {
    match s {
        "html" => Ok(ReportFormat::Html),
        // ...
    }
}
```

**No changes to aggregation logic or other formatters!**

## Summary

This example shows:
- Pure aggregation functions (testable, fast)
- Multiple output formats (extensible)
- Snapshot testing (catch regressions)
- Clean separation (domain vs CLI)
- Type-rich models (CategoryStats, ProductStats)
- Production-ready error handling
