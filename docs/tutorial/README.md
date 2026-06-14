# 🎓 Tutorials: Learn clap-noun-verb

**Welcome!** This tutorial series will take you from zero to productive with clap-noun-verb through hands-on, step-by-step guidance.

---

## What You'll Learn

By the end of this tutorial series, you'll be able to:
- ✅ Build production-ready CLIs with noun-verb patterns
- ✅ Separate domain logic from CLI layer (architecture principle)
- ✅ Test CLIs with Chicago TDD methodology
- ✅ Handle errors professionally with thiserror
- ✅ Use async operations for real-world CLIs

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

1. **[Domain Separation Architecture](01-domain-separation.md)**
 - The Golden Rule: CLI validates, domain computes
 - Why separation matters
 - Testable business logic
 - **Time**: 10-15 minutes

2. **[Adding Multiple Commands](02-adding-multiple.md)**
 - Multi-command CLIs
 - File organization patterns
 - Noun-verb naming conventions
 - **Time**: 10-15 minutes

### 📚 Core Features (1-2 hours)

**For**: Developers building production CLIs

3. **[Testing Basics](03-testing-basics.md)**
 - Chicago TDD fundamentals
 - State-based testing
 - Real collaborators vs mocks
 - **Time**: 20-30 minutes

4. **[Output Formats](04-output-formats.md)**
 - JSON, YAML, TOML outputs
 - ASCII table formatting
 - Custom format selection
 - **Time**: 15-20 minutes

5. **[Async Operations](05-async-operations.md)**
 - Async command handlers
 - Tokio runtime integration
 - HTTP requests and database queries
 - **Time**: 30-45 minutes

6. **[Error Handling](06-error-handling.md)**
 - Result<T,E> patterns
 - Custom error types
 - Structured error output
 - **Time**: 30-45 minutes

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

## Getting Help

### Stuck on a tutorial?
- **Re-read the instructions** - Most issues come from skipped steps
- **Check the examples** - Each tutorial has a working example in `examples/`
- **Ask for help** - [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb)

### Found a bug in a tutorial?
- **Report it** - [GitHub Issues](https://github.com/seanchatmangpt/clap-noun-verb/issues)
- **All examples are tested** - If something doesn't work, it's a bug!

---

## Alternative Learning Paths

### "I want to learn by example"
→ Check out [examples/](../../examples/) directory with working examples

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
| [01. Domain Separation](01-domain-separation.md) | ✅ **Available** | 10-15 min |
| [02. Adding Commands](02-adding-multiple.md) | ✅ **Available** | 10-15 min |
| [03. Testing Basics](03-testing-basics.md) | ✅ **Available** | 20-30 min |
| [04. Output Formats](04-output-formats.md) | ✅ **Available** | 15-20 min |
| [05. Async Operations](05-async-operations.md) | ✅ **Available** | 30-45 min |
| [06. Error Handling](06-error-handling.md) | ✅ **Available** | 30-45 min |

**Total Estimated Time**: 2-3 hours for complete series

---

## Ready to Start?

👉 **[Begin with Tutorial 01: Domain Separation](01-domain-separation.md)**

---

**Note**: Tutorials follow Diataxis principles with validated 26.6.13 code examples.
