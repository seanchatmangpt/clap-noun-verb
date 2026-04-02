# Wizard Package Documentation

Welcome to the Wizard package - an AI-powered CLI wizard framework with rust-genai integration for building intelligent, interactive command-line interfaces.

## 📚 Documentation Structure

The Wizard package documentation is organized into four sections following the [Diataxis](https://diataxis.fr/) framework:

### [🎓 Tutorials](./tutorials/README.md)
**Learning-oriented guides to get you started**

Start here if you're new to Wizard:
- [Getting Started](./tutorials/getting-started.md) - Set up your first project in 5 minutes
- [Your First Wizard Session](./tutorials/first-session.md) - Build a simple interactive wizard
- [Running Examples](./tutorials/examples.md) - Explore working examples

### [🔧 How-To Guides](./how-to/README.md)
**Task-oriented guides for solving specific problems**

Use these when you need to accomplish a specific task:
- [Streaming Responses](./how-to/streaming.md) - Implement token-by-token generation
- [Caching Responses](./how-to/caching.md) - Improve performance with LRU caching
- [Implementing Retry Logic](./how-to/retry-logic.md) - Handle transient failures gracefully
- [Using Model Fallbacks](./how-to/model-fallbacks.md) - Chain multiple AI models
- [Configuration Guide](./how-to/configuration.md) - Configure wizard for your needs
- [CLI Integration](./how-to/cli-integration.md) - Integrate with clap CLI framework

### [📖 Reference](./reference/README.md)
**Information-oriented technical documentation**

Look here for complete technical information:
- [API Reference](./reference/api.md) - Complete public API documentation
- [Configuration Reference](./reference/configuration.md) - All configuration options
- [Error Types](./reference/errors.md) - Error handling and recovery
- [Feature Flags](./reference/features.md) - Feature flag guide
- [Type Index](./reference/types.md) - All public types and traits

### [💡 Explanation](./explanation/README.md)
**Understanding-oriented conceptual documentation**

Read these to understand the design and concepts:
- [Architecture Overview](./explanation/architecture.md) - High-level system design
- [Design Decisions](./explanation/design-decisions.md) - Why we made key choices
- [How Streaming Works](./explanation/streaming-internals.md) - Deep dive into streaming
- [How Caching Works](./explanation/caching-internals.md) - Cache algorithm details
- [Type-Safe Design](./explanation/type-safety.md) - Zero-cost abstraction patterns
- [Concepts & Terminology](./explanation/concepts.md) - Key terms and ideas

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = { version = "5.5", features = ["wizard"] }
```

### First Wizard

```rust
use clap_noun_verb::wizard::{WizardBuilder, WizardConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WizardConfig::from_env()?;

    let mut wizard = WizardBuilder::new()
        .with_config(config)
        .build()
        .await?;

    // Use your wizard...
    Ok(())
}
```

See [Getting Started](./tutorials/getting-started.md) for more.

## 🎯 Common Tasks

- **I want to...** → See [How-To Guides](./how-to/README.md)
- **I need to understand...** → See [Explanation](./explanation/README.md)
- **I'm looking for API docs** → See [Reference](./reference/README.md)
- **I'm new to Wizard** → See [Tutorials](./tutorials/README.md)

## 📊 Key Features

- **Multi-Provider Support** - OpenAI, Anthropic, Gemini, custom providers
- **Streaming Responses** - Token-by-token generation with backpressure
- **Smart Caching** - LRU cache with TTL and automatic eviction
- **Automatic Retry** - Exponential backoff with jitter
- **Model Fallbacks** - Automatic failover between models
- **Type Safety** - Compile-time guarantees, zero unsafe code
- **Performance** - Optimized for latency and throughput
- **Observable** - Metrics, tracing, structured logging

## 🔗 Related Resources

- [clap-noun-verb Repository](https://github.com/seanchatmangpt/clap-noun-verb)
- [Diataxis Framework](https://diataxis.fr/)
- [rust-genai Documentation](https://docs.rs/genai/)
- [SPARC Methodology](https://docs.rs/chicago-tdd-tools/)

## 🤝 Contributing

Found an issue with the documentation? [Open an issue on GitHub](https://github.com/seanchatmangpt/clap-noun-verb/issues).

## 📄 License

Licensed under MIT OR Apache-2.0

---

**Last Updated:** January 2026
**Wizard Version:** 2.0.0
**Documentation Format:** [Diataxis](https://diataxis.fr/)
