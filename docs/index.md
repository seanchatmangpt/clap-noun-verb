# clap-noun-verb Documentation Index

**Version**: 26.5.28
**Framework**: [Diataxis](https://diataxis.fr/)
**Last Updated**: 2026-05-28

---

## 📚 Quick Navigation

This documentation follows the **Diataxis framework**, organizing content into four quadrants based on your needs:

| Quadrant | When to use | Start here |
|----------|-------------|------------|
| 🎓 **[Tutorial](tutorial/)** | You're learning and want step-by-step guidance | [Domain Separation](tutorial/02-domain-separation.md) |
| 📘 **[How-To](howto/)** | You have a specific problem to solve | [How-To Index](howto/README.md) |
| 📚 **[Reference](reference/)** | You need to look up API details | [API Catalog](reference/api-catalog.md) |
| 💡 **[Explanation](explanation/)** | You want to understand the "why" | [Architecture](explanation/architecture.md) |

---

## 🎓 Tutorial (Learning-Oriented)

**Goal**: Take you from zero to productive with hands-on, step-by-step guidance.

### Available Tutorials
- [02. Domain Separation Architecture](tutorial/02-domain-separation.md) - Separating CLI from business logic
- [03. Adding Multiple Commands](tutorial/03-adding-commands.md) - Multi-command CLIs
- [04. Testing Basics](tutorial/04-testing-basics.md) - Chicago TDD fundamentals
- [05. Output Formats](tutorial/05-output-formats.md) - JSON, YAML, Table outputs
- [07. Async Operations](tutorial/07-async-operations.md) - Async command handlers
- [08. Error Handling](tutorial/08-error-handling.md) - Result<T,E> patterns

---

## 📘 How-To Guides (Problem-Solving)

**Goal**: Solve specific problems with practical, production-ready recipes.

### Core Guides
- [Common Mistakes](howto/common-mistakes.md) - Top 10 errors and how to fix them
- [Debugging](howto/debugging.md) - Debugging techniques and tools
- [Performance Optimization](howto/performance-optimization.md) - Making CLIs fast
- [Setup Help and Version](howto/setup-help-and-version.md) - `--help` and `--version` setup
- [Testing](howto/testing.md) - Test strategies
- [Validation](howto/validation.md) - Input validation techniques

### Production Patterns
- [Configure Applications](howto/production/configuration.md) - Config management
- [Deploy to Production](howto/production/deployment.md) - Deployment strategies
- [Monitor with OTEL](howto/production/monitoring.md) - Observability integration
- [Secure Your CLI](howto/production/security.md) - Security hardening

---

## 📚 Reference (Information-Oriented)

**Goal**: Provide accurate, concise API information for quick lookups.

### Core API
- [API Catalog](reference/api-catalog.md) - Complete API catalog
- [#[verb] Macro](reference/api/verb-macro.md) - Complete macro reference
- [#[arg] Attributes](reference/api/arg-attributes.md) - Argument attributes
- [Types Catalog](reference/api/types.md) - All public types
- [Errors Catalog](reference/api/errors.md) - Error types
- [Advanced Features](reference/api/advanced-features.md) - Completions, chaining, stdin, REPL, and introspection
- [Parameter Preprocessor Guide](PREPROCESSOR_GUIDE.md) - Chaining, variable expansion, overrides, and recursion safety
- [Telemetry Reference](reference/api/telemetry.md) - Distributed tracing & context
- [#[noun] Macro (DEPRECATED)](reference/api/noun-macro.md) - Legacy noun subcommand definition macro

### CLI Reference
- [Error Codes](reference/error-codes.md) - Common errors and solutions
- [Performance SLOs](reference/performance-slos.md) - Performance targets and guarantees
- [Schema Validation & Introspection](SCHEMA_VALIDATION.md) - SHACL constraints, JSON schema introspection, and output verification hooks

---

## 💡 Explanation (Understanding-Oriented)

**Goal**: Explain the "why" behind design decisions and architecture.

### Architecture & Design
- [Architecture Overview](explanation/architecture.md) - System architecture and component design
- [Design Patterns](explanation/design-patterns.md) - Common patterns and best practices

---

## 📖 Additional Resources

### Quick References
- [README.md](../README.md) - Project overview and quick start
- [AUTONOMIC.md](../AUTONOMIC.md) - Autonomic layer overview
- [Autonomic CLI Layer Spec](future/autonomic_cli.md) - Self-healing commands, diagnostic reporting, and MAPE-K integration patterns
- [VERIFICATION_REPORT.md](VERIFICATION_REPORT.md) - Documentation verification report

### Development
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [CHANGELOG.md](../CHANGELOG.md) - Version history

---

## 🗺️ Finding What You Need

### "I want to learn clap-noun-verb from scratch"
→ Start with [Tutorial 02: Domain Separation](tutorial/02-domain-separation.md)

### "I need to solve a specific problem"
→ Browse [How-To Guides](howto/README.md) by category

### "I need to look up an API signature"
→ Check [Reference](reference/README.md) → find your topic

### "I want to understand why something works this way"
→ Read [Explanation](explanation/README.md) → architecture & philosophy

---

## 📊 Documentation Status

**Overall Progress**: ✅ **Migration Complete**

| Quadrant | Status | Files | Progress |
|----------|--------|-------|----------|
| 🎓 Tutorial | ✅ **Complete** | 6/6 | 100% |
| 📘 How-To | ✅ **Complete** | 11/11 | 100% |
| 📚 Reference | ✅ **Complete** | 11/11 | 100% |
| 💡 Explanation | ✅ **Complete** | 2/2 | 100% |

---

## 🔗 External Resources

- **Diataxis Framework**: https://diataxis.fr/
- **GitHub Repository**: https://github.com/seanchatmangpt/clap-noun-verb
- **Crates.io**: https://crates.io/crates/clap-noun-verb
- **Docs.rs**: https://docs.rs/clap-noun-verb

---

**Need help?** Check the [troubleshooting guide](howto/debugging.md) or [open an issue](https://github.com/seanchatmangpt/clap-noun-verb/issues).
