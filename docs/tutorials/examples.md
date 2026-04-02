# Running Examples

Explore working examples to learn Wizard in action.

## Available Examples

### Basic Examples

**`examples/wizard_basic.rs`** - Hello Wizard
- Simple single-turn interaction
- Loading configuration from environment
- Basic prompt and response
- Error handling
- **Run:** `cargo run --example wizard_basic --features wizard`

**`examples/wizard_interactive.rs`** - Interactive Session
- Multi-turn conversation
- Session history
- Special commands (exit, clear, help)
- Async/await patterns
- **Run:** `cargo run --example wizard_interactive --features wizard`

### Feature Examples

**`examples/wizard_streaming.rs`** - Streaming Responses
- Token-by-token output
- Buffer management
- Streaming configuration
- Response accumulation
- **Run:** `cargo run --example wizard_streaming --features wizard`

**`examples/wizard_caching.rs`** - Response Caching
- Cache configuration
- Cache hits and misses
- TTL expiration
- Performance improvement
- **Run:** `cargo run --example wizard_caching --features wizard`

**`examples/wizard_retry.rs`** - Retry Logic
- Exponential backoff
- Jitter calculation
- Retry configuration
- Failure handling
- **Run:** `cargo run --example wizard_retry --features wizard`

**`examples/wizard_fallback.rs`** - Model Fallback
- Multiple models
- Fallback strategies
- Cost-aware selection
- Latency tracking
- **Run:** `cargo run --example wizard_fallback --features wizard`

## Running Examples

### Prerequisites

1. Set your API key:
```bash
export ANTHROPIC_API_KEY="your-key"
# OR
export OPENAI_API_KEY="your-key"
# OR
export GEMINI_API_KEY="your-key"
```

2. Have Rust 1.74+ installed

### Run Any Example

```bash
cargo run --example <name> --features wizard
```

Replace `<name>` with the example name (without `.rs`).

## Example Walkthrough

Each example is designed to be understood in sequence:

1. **Start with `wizard_basic`** - Understand the fundamentals
2. **Try `wizard_interactive`** - Learn about sessions
3. **Explore feature examples** - Learn each feature independently

## Modifying Examples

All examples are self-contained and can be modified:

1. Copy the example to your own project
2. Make changes
3. Run with `cargo run --features wizard`

## Learning from Code

When reading examples:

1. **Start at main()** - Understand the flow
2. **Follow the happy path** - See the basic usage
3. **Check error handling** - See how errors are caught
4. **Read comments** - Understand the why

## Common Modifications to Try

### Basic Example

- Change the system prompt
- Try different models
- Add custom configuration

### Interactive Example

- Add new commands (e.g., `stats`, `model`)
- Save history to a file
- Add command-line arguments

### Streaming Example

- Change buffer size
- Experiment with different models
- Measure streaming speed

### Caching Example

- Change cache size
- Modify TTL
- Track cache efficiency

### Retry Example

- Adjust backoff parameters
- Try different jitter factors
- Measure retry overhead

### Fallback Example

- Add more models
- Try different selection strategies
- Compare fallback efficiency

## Troubleshooting Examples

**API key not found?**
```bash
# Check environment variable
echo $ANTHROPIC_API_KEY
```

**Compilation errors?**
```bash
# Ensure wizard feature is enabled
cargo build --example wizard_basic --features wizard
```

**Runtime errors?**
- Check your internet connection
- Verify API key is valid
- Check API provider status

## Next Steps

- Modify an example to suit your needs
- Check [How-To Guides](../how-to/README.md) for specific patterns
- Read [Reference](../reference/README.md) for detailed APIs

---

[Back to Tutorials →](./README.md)
