# 🎓 Tutorial: Learning clap-noun-verb

**Welcome!** This tutorial series will take you from zero to productive with clap-noun-verb through hands-on, step-by-step guidance.

---

## What You'll Learn

By the end of this tutorial series, you'll be able to:
- ✅ Build production-ready CLIs with noun-verb patterns
- ✅ Separate domain logic from CLI layer (architecture principle)
- ✅ Test CLIs with Chicago TDD methodology
- ✅ Use autonomic features for machine-grade interfaces
- ✅ Deploy CLIs to production with confidence

---

## Prerequisites

- **Rust 1.74+** installed ([rustup.rs](https://rustup.rs/))
- **Basic Rust knowledge** (variables, functions, Result type)
- **10 minutes** for the first tutorial
- **2-3 hours** for the complete series

---

## Tutorial Path

### 🚀 Getting Started (30 minutes)

**For**: Absolute beginners who want to build their first CLI

1. **[Your First CLI in 5 Minutes](01-your-first-cli.md)**
   - Hello World to working CLI
   - Auto-discovery magic
   - JSON output by default
   - **Time**: 5-10 minutes

2. **[Domain Separation Architecture](02-domain-separation.md)**
   - The Golden Rule: CLI validates, domain computes
   - Why separation matters
   - Testable business logic
   - **Time**: 10-15 minutes

3. **[Adding Multiple Commands](03-adding-commands.md)**
   - Multi-command CLIs
   - File organization patterns
   - Noun-verb naming conventions
   - **Time**: 10-15 minutes

### 📚 Intermediate Features (1-2 hours)

**For**: Developers building production CLIs

4. **[Testing Basics](04-testing-basics.md)**
   - Chicago TDD fundamentals
   - State-based testing
   - Real collaborators vs mocks
   - **Time**: 20-30 minutes

5. **[Output Formats](05-output-formats.md)**
   - JSON, YAML, TOML outputs
   - ASCII table formatting
   - Custom format selection
   - **Time**: 15-20 minutes

6. **[Autonomic Features](06-autonomic-features.md)**
   - Machine-grade introspection
   - `--capabilities`, `--introspect` flags
   - Effect metadata and guards
   - **Time**: 20-30 minutes

### 🔥 Advanced Topics (2-3 hours)

**For**: Developers building complex, production-grade CLIs

7. **[Async Operations](07-async-operations.md)**
   - Async command handlers
   - Tokio runtime integration
   - HTTP requests and database queries
   - **Time**: 30-45 minutes

8. **[Error Handling](08-error-handling.md)**
   - Result<T,E> patterns
   - Custom error types
   - Structured error output
   - **Time**: 30-45 minutes

9. **[Deployment Basics](09-deployment-basics.md)**
   - Building release binaries
   - Cross-compilation
   - Distribution strategies
   - **Time**: 20-30 minutes

10. **[Next Steps](10-next-steps.md)**
    - Paths to mastery
    - Production patterns
    - Community resources
    - **Time**: 10 minutes

---

## Learning Approach

This tutorial series follows **learning-oriented** principles:

### ✅ Do's
- **Hands-on**: You'll write code from the first tutorial
- **Progressive**: Each tutorial builds on the previous one
- **Safe**: You can't break anything - experiment freely!
- **Validated**: All code examples compile and work (tested in CI)

### ❌ Don'ts
- **No theory dumps**: We learn by doing, not reading
- **No skipping steps**: Each tutorial assumes completion of previous ones
- **No production shortcuts**: We teach the right way from the start

---

## Tutorial Format

Each tutorial follows this structure:

1. **What You'll Build** - Clear learning outcomes
2. **Prerequisites** - What you need before starting
3. **Step-by-Step Instructions** - Hands-on coding
4. **What You Learned** - Summary of concepts
5. **Next Steps** - Where to go from here

---

## Getting Help

### Stuck on a tutorial?
- **Re-read the instructions** - Most issues come from skipped steps
- **Check the examples** - Each tutorial has a working example in `examples/`
- **Ask for help** - [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb/discussions)

### Found a bug in a tutorial?
- **Report it** - [GitHub Issues](https://github.com/seanchatmangpt/clap-noun-verb/issues)
- **All examples are tested** - If something doesn't work, it's a bug!

---

## Alternative Learning Paths

### "I want to learn by example"
→ Check out [examples/](../../examples/) directory with 50+ working examples

### "I have a specific problem to solve"
→ Skip to [How-To Guides](../howto/README.md) for production patterns

### "I need API documentation"
→ Check [Reference](../reference/README.md) for complete API catalog

### "I want to understand the architecture"
→ Read [Explanation](../explanation/README.md) for design philosophy

---

## Tutorial Status

| Tutorial | Status | Estimated Time |
|----------|--------|----------------|
| [01. Your First CLI](01-your-first-cli.md) | ⏳ **Planned** | 5-10 min |
| [02. Domain Separation](02-domain-separation.md) | ⏳ **Planned** | 10-15 min |
| [03. Adding Commands](03-adding-commands.md) | ⏳ **Planned** | 10-15 min |
| [04. Testing Basics](04-testing-basics.md) | ⏳ **Planned** | 20-30 min |
| [05. Output Formats](05-output-formats.md) | ⏳ **Planned** | 15-20 min |
| [06. Autonomic Features](06-autonomic-features.md) | ⏳ **Planned** | 20-30 min |
| [07. Async Operations](07-async-operations.md) | ⏳ **Planned** | 30-45 min |
| [08. Error Handling](08-error-handling.md) | ⏳ **Planned** | 30-45 min |
| [09. Deployment Basics](09-deployment-basics.md) | ⏳ **Planned** | 20-30 min |
| [10. Next Steps](10-next-steps.md) | ⏳ **Planned** | 10 min |

**Total Estimated Time**: 2-3 hours for complete series

---

## Ready to Start?

👉 **[Begin with Tutorial 01: Your First CLI](01-your-first-cli.md)**

---

**Note**: During migration from v4 to v5.1.1 documentation, the legacy [QUICKSTART.md](../QUICKSTART.md) remains available. New tutorials follow Diataxis principles with validated v5.1.1 code examples.
