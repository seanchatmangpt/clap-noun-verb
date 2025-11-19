# ggen CLI - Enhanced Error Handling Example

This example demonstrates production-grade error handling for a CLI application built with clap-noun-verb.

## Overview

The ggen (General Projection Engine) CLI shows how to implement user-friendly error messages that:
- Clearly explain what went wrong
- Provide actionable recovery steps
- Include working example commands
- Link to relevant documentation

## Features

### Error Handling (`errors.rs`)
- Structured error types with problem/solution/docs
- Error categorization for metrics
- Pretty formatting with emoji markers
- 11 specialized error constructors

### Input Validation (`validators.rs`)
- Comprehensive validation with helpful errors
- Typo detection and suggestions
- Format validation with examples
- 6 validation functions

### Commands

#### AI Commands (`ai_commands.rs`)
```bash
ggen ai generate -d "Create REST API handler" --model gpt-4-turbo
ggen ai project MyApp --description "Web service"
ggen ai graph -d "E-commerce ontology" --format turtle
ggen ai sparql -d "Find all users" --graph data.ttl
```

#### Marketplace Commands (`marketplace_commands.rs`)
```bash
ggen marketplace search "rust web"
ggen marketplace install io.ggen.rust.axum
ggen marketplace list
ggen marketplace publish ./my-package
```

#### Template Commands (`template_commands.rs`)
```bash
ggen template generate rust-lib.tmpl name=mylib author="John"
ggen template render rust-bin.tmpl name=mycli
ggen template validate my-template.tmpl
ggen template list
```

## Building

```bash
# Build the example
cargo build --example ggen_cli

# Run with help
cargo run --example ggen_cli -- --help

# Try a command (will show helpful error for missing API key)
cargo run --example ggen_cli -- ai generate -d "Create handler"
```

## Testing

```bash
# Run all error handling tests
cargo test --test ggen_error_handling_tests

# Run specific test
cargo test --test ggen_error_handling_tests test_validate_model_name

# Run with output
cargo test --test ggen_error_handling_tests -- --nocapture
```

**Expected**: 58 tests, all passing

## Error Message Examples

### Invalid Model Name
```
❌ Problem: Model 'gpt5' not recognized. Did you mean 'gpt-4-turbo'?
💡 Solution: Use the correct model name:
  ggen ai generate --model gpt-4-turbo -d 'your prompt'

  Supported models:
  - gpt-4-turbo (recommended for complex tasks)
  - gpt-3.5-turbo (faster, good for simple tasks)
  - claude-3-opus (best quality, slower)
📚 Learn more: https://docs.ggen.io/models
```

### Missing Template Variables
```
❌ Problem: Template 'rust-lib.tmpl' requires 1 variable(s) that were not provided
💡 Solution: Provide the following variables:
  author=<value>

  Example: ggen template generate rust-lib.tmpl name=mylib author=example
📚 Learn more: https://docs.ggen.io/templates
```

### Empty Search Results
```
❌ Problem: No packages found matching 'nonexistent'
💡 Solution: Try these alternatives:
  - Broaden your search terms
  - Browse all packages: ggen marketplace list
  - Search by category: ggen marketplace search --category rust
📚 Learn more: https://marketplace.ggen.io
```

## Code Structure

```
ggen/
├── mod.rs                 # Module exports
├── errors.rs              # Error types and constructors (280 lines)
├── validators.rs          # Input validation (350 lines)
├── ai_commands.rs         # AI generation commands (250 lines)
├── marketplace_commands.rs # Package marketplace (320 lines)
└── template_commands.rs   # Template operations (280 lines)
```

## Implementation Patterns

### Error Construction
```rust
UserError::new(
    ErrorCategory::Validation,
    "Clear problem statement",
    "Actionable recovery steps"
).with_docs("https://docs.ggen.io/topic")
```

### Validation Flow
```rust
// 1. Validate input
let validated = validate_model_name(&model)?;

// 2. Execute business logic (cannot fail due to validation)
let result = generate_code(&validated)?;

// 3. Return result
Ok(result)
```

### Command Pattern
```rust
#[verb("generate", "ai")]
pub fn ai_generate(
    #[arg(short, long)] description: String,
    #[arg(short, long)] model: String,
) -> CnvResult<GenerateOutput> {
    // Validate
    let validated_prompt = validate_prompt(&description)?;
    let validated_model = validate_model_name(&model)?;

    // Execute
    generate_ai_content(&validated_prompt, &validated_model)
}
```

## Best Practices Demonstrated

1. **No `unwrap()` or `panic!()`** - All errors handled gracefully
2. **Comprehensive validation** - Catch errors early with good messages
3. **Actionable errors** - Always provide recovery steps
4. **Working examples** - Include copy-paste commands
5. **Documentation links** - Point to relevant help
6. **Test coverage** - 58 tests covering all scenarios

## Documentation

- [Error Handling Improvements](../../docs/ggen_error_handling_improvements.md) - Before/after examples
- [Implementation Summary](../../docs/ggen_implementation_summary.md) - Technical details

## License

MIT OR Apache-2.0 (same as clap-noun-verb)
