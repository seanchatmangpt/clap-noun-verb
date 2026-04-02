# Domain Separation Examples - Production-Ready Patterns

This directory contains **three complete, production-ready examples** demonstrating how to properly separate domain logic from CLI code in Rust applications.

## 📚 Examples Overview

### 1. Data Processor - Streaming & Error Handling
**Location:** `data-processor/`

**Demonstrates:**
- Streaming CSV processing (handles large datasets)
- Generic I/O (testable with in-memory buffers)
- Comprehensive error handling and statistics
- Chicago TDD with state-based testing

**Use case:** Real-world data ETL pipeline with transformation logic separated from file I/O.

**Key takeaway:** Domain layer processes streams, CLI layer opens files.

---

### 2. API Client - Circuit Breaker & Retries
**Location:** `api-client/`

**Demonstrates:**
- Circuit breaker pattern (pure state machine)
- Exponential backoff retry logic
- Async domain validation with sync testing
- HTTP mocking with mockito

**Use case:** Fault-tolerant API client with business rules separated from network I/O.

**Key takeaway:** Domain logic = state transitions, CLI layer = HTTP execution.

---

### 3. Report Generator - Multiple Output Formats
**Location:** `report-generator/`

**Demonstrates:**
- Rich aggregation logic (statistics, grouping)
- Multiple output formats (JSON, CSV, Markdown)
- Snapshot testing with insta
- Type-rich domain models

**Use case:** Business intelligence reporting with formatting logic separated from file I/O.

**Key takeaway:** Domain formats data, CLI layer writes files.

---

## 🎯 Template Project
**Location:** `template/`

Ready-to-use template with:
- Complete project structure
- Domain/CLI separation boilerplate
- Testing examples
- Comprehensive checklist
- Common patterns

**Use this to start new projects!**

---

## 🚫 Anti-Patterns Guide
**Location:** `anti-patterns/`

Comprehensive guide showing **what NOT to do**:
- CLI logic in domain layer
- Domain logic in CLI layer
- Untestable code patterns
- Type confusion
- God functions

**Read this to avoid common mistakes!**

---

## 🧠 Core Principles

### Domain Layer Rules
✅ **DO:**
- Take data structures (Vec, String, structs)
- Return Result<Data, DomainError>
- Use generic I/O traits (BufRead, Write)
- Implement pure functions (no side effects)
- Define rich domain models

❌ **DON'T:**
- Use PathBuf or File
- Call println! or other user output
- Make HTTP requests or network calls
- Parse command-line arguments
- Access environment variables

### CLI Layer Rules
✅ **DO:**
- Open/close files
- Parse command-line arguments
- Make HTTP requests
- Format output for users
- Convert domain errors to user messages

❌ **DON'T:**
- Implement business logic
- Validate domain rules
- Transform data (beyond parsing)
- Compute aggregations or statistics

---

## 🧪 Testing Patterns

### Domain Tests (Fast, Pure)
```rust
#[test]
fn test_domain_logic() {
    // Arrange - pure data
    let input = DomainInput { data: "test".to_string() };

    // Act - call domain function
    let output = domain::process(input).unwrap();

    // Assert - verify state
    assert_eq!(output.result, "PROCESSED: TEST");
}
```

**Characteristics:**
- ⚡ **Fast** - no I/O, runs in microseconds
- 🎯 **Focused** - tests single function
- 🔄 **Deterministic** - same input, same output
- 🧪 **Pure** - no mocks, no stubs, real data

### CLI Tests (Integration)
```rust
#[test]
fn test_cli_command() {
    // Arrange - create temp file
    let input_file = create_temp_file("test data");

    // Act - call CLI function
    let result = cli::process_file(input_file.path());

    // Assert - verify files/output
    assert!(result.is_ok());
    assert!(output_file_exists());
}
```

**Characteristics:**
- 💾 **I/O-heavy** - uses temp files, HTTP mocks
- 🔗 **Integration** - tests full workflow
- 🧰 **Tools** - uses tempfile, mockito
- 🐢 **Slower** - milliseconds per test

---

## 🏗️ Architecture Comparison

### ❌ WRONG: Mixed Concerns
```
src/
└── main.rs  (500 lines of mixed CLI + domain logic)
```

**Problems:**
- Can't test without file system
- Can't reuse logic
- Hard to maintain

### ✅ RIGHT: Clean Separation
```
src/
├── cli/           # Thin I/O layer
│   └── commands.rs
├── domain/        # Pure business logic
│   └── logic.rs
└── main.rs        # Minimal entry point
```

**Benefits:**
- ✅ Fast unit tests (domain)
- ✅ Reusable logic
- ✅ Easy to extend
- ✅ Clear responsibilities

---

## 📖 Learning Path

**1. Start with Template**
- Copy `template/` to new project
- Read `template/README.md`
- Understand structure

**2. Study Data Processor**
- Read `data-processor/README.md`
- Run tests: `cd data-processor && cargo test`
- See streaming pattern

**3. Study API Client**
- Read `api-client/README.md`
- Understand circuit breaker
- See async testing

**4. Study Report Generator**
- Read `report-generator/README.md`
- Understand formatters
- See snapshot testing

**5. Review Anti-Patterns**
- Read `anti-patterns/README.md`
- Understand what to avoid
- Learn from mistakes

---

## 🎓 Key Concepts

### Type-First Design
```rust
// Domain takes data, not paths
pub fn process(input: Input) -> Result<Output, DomainError>

// CLI takes paths, opens files
pub fn process_cmd(input_path: PathBuf) -> Result<()> {
    let input = load_from_file(&input_path)?;
    let output = domain::process(input)?;
    println!("Done: {:?}", output);
    Ok(())
}
```

### Generic I/O
```rust
// Domain: Generic over readers/writers
pub fn process_stream<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
) -> Result<Stats>

// Test: Use in-memory buffers
#[test]
fn test_process() {
    let input = "data".as_bytes();
    let mut output = Vec::new();
    let stats = process_stream(input, &mut output).unwrap();
    assert_eq!(stats.processed, 1);
}
```

### Pure State Machines
```rust
// Domain: State transitions (no I/O)
impl CircuitBreaker {
    pub fn record_failure(&mut self) { /* update state */ }
    pub fn can_request(&self) -> Result<()> { /* check state */ }
}

// CLI: Execute actions based on state
async fn make_request(cb: &mut CircuitBreaker) -> Result<Response> {
    cb.can_request()?;  // Check domain state
    let resp = http_call().await?;  // CLI does I/O
    cb.record_success();  // Update domain state
    Ok(resp)
}
```

---

## 🚀 Running Examples

### Compile All Examples
```bash
cd data-processor && cargo build --release && cd ..
cd api-client && cargo build --release && cd ..
cd report-generator && cargo build --release && cd ..
cd template && cargo build --release && cd ..
```

### Run All Tests
```bash
cd data-processor && cargo test && cd ..
cd api-client && cargo test && cd ..
cd report-generator && cargo test && cd ..
cd template && cargo test && cd ..
```

### Try Examples
```bash
# Data Processor
cd data-processor
echo "id,name,value,category" > test.csv
echo "1,Test,10.0,A" >> test.csv
cargo run -- process --input test.csv --output result.csv

# Report Generator
cd report-generator
echo "id,product,category,amount,quantity,date" > sales.csv
echo "1,Laptop,Electronics,1200.0,1,2024-01-01T00:00:00Z" >> sales.csv
cargo run -- generate --input sales.csv --format markdown
```

---

## 📊 Comparison Matrix

| Example | Domain Logic | I/O Pattern | Testing | Best For |
|---------|--------------|-------------|---------|----------|
| **Data Processor** | Transformations | Streaming | State-based | ETL, data pipelines |
| **API Client** | Validation, circuit breaker | Async HTTP | Mocked HTTP | API clients, resilience |
| **Report Generator** | Aggregation, formatting | File I/O | Snapshot | Reports, analytics |
| **Template** | Generic example | File I/O | Both | New projects |

---

## 🎯 When to Use Each Pattern

### Use Data Processor Pattern When:
- Processing large files (streaming)
- Transforming data line-by-line
- Need error recovery and statistics

### Use API Client Pattern When:
- Making network requests
- Need fault tolerance (retries, circuit breaker)
- Complex state management

### Use Report Generator Pattern When:
- Multiple output formats needed
- Rich aggregations and statistics
- Want snapshot testing

---

## 💡 Pro Tips

1. **Start with types** - Define domain models first
2. **Make it generic** - Use traits for I/O (BufRead, Write)
3. **Test domain first** - Fast feedback loop
4. **Keep CLI thin** - If CLI > 50 lines, extract to domain
5. **Use Result everywhere** - No unwrap() in production
6. **Document with examples** - Show usage in doc comments
7. **Snapshot complex outputs** - Use insta for formatters

---

## 🤝 Contributing

Found a better pattern? See an anti-pattern we missed?

1. Add example to appropriate directory
2. Include comprehensive tests
3. Document key patterns
4. Update this README

---

## 📚 Further Reading

- **Rust Book** - Chapters on error handling, testing
- **Clean Architecture** - Robert Martin's principles
- **Domain-Driven Design** - Eric Evans
- **Chicago TDD** - State-based testing approach

---

## Summary

These examples demonstrate **production-ready domain separation** with:

✅ Pure domain logic (fast tests, reusable)
✅ Thin CLI layer (I/O only)
✅ Chicago TDD patterns (state-based, real collaborators)
✅ Type-first design (invalid states impossible)
✅ Multiple testing strategies (unit, integration, snapshot)
✅ Real-world use cases (not toy examples)

**Use the template to start your next project with clean separation from day one!**
