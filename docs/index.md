# Wizard Package Documentation

Welcome to the Wizard package - an AI-powered CLI wizard framework with rust-genai integration for building intelligent, interactive command-line interfaces.

## 📚 Documentation Structure

The Wizard package documentation is organized into four sections following the [Diataxis](https://diataxis.fr/) framework:

### [🎓 Tutorials](../tutorial/README.md)
**Learning-oriented guides to get you started**

Start here if you're new to clap-noun-verb:
- [Your First CLI](../tutorial/01-your-first-cli.md) - Build a CLI in 5 minutes
- [Domain Separation](../tutorial/02-domain-separation.md) - Architecture principles
- [Adding Commands](../tutorial/03-adding-commands.md) - Multi-command CLIs
- [Testing Basics](../tutorial/04-testing-basics.md) - Chicago TDD methodology

### [🔧 How-To Guides](../howto/README.md)
**Task-oriented guides for solving specific problems**

Use these when you need to accomplish a specific task:
- [Production Deployment](../howto/production/deployment.md) - Deploy your CLI
- [Production Monitoring](../howto/production/monitoring.md) - Observability
- [Production Configuration](../howto/production/configuration.md) - Config management
- [Production Security](../howto/production/security.md) - Security best practices
- [Testing](../howto/testing.md) - Test strategies
- [Validation](../howto/validation.md) - Input validation

### [📖 Reference](../reference/README.md)
**Information-oriented technical documentation**

Look here for complete technical information:
- [Verb Macro](../reference/api/verb-macro.md) - `#[verb]` attribute reference
- [Arg Attributes](../reference/api/arg-attributes.md) - Argument attributes
- [Types](../reference/api/types.md) - Public types
- [Errors](../reference/api/errors.md) - Error types

### [💡 Explanation](../explanation/README.md)
**Understanding-oriented conceptual documentation**

Read these to understand the design and concepts:
- [Architecture](../explanation/architecture.md) - System architecture
- [Agent Architecture](../explanation/agent-architecture.md) - Multi-agent design
- [AUTONOMIC.md](../AUTONOMIC.md) - Autonomic CLI layer

> **Note:** The wizard package (requires "wizard" feature) has specialized documentation. See [wizard_quick_reference.md](wizard_quick_reference.md) for wizard-specific features.

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
