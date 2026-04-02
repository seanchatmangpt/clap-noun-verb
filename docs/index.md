# clap-noun-verb Documentation Index

**Version**: 5.1.1
**Framework**: [Diataxis](https://diataxis.fr/)
**Last Updated**: 2025-12-02

---

## 📚 Quick Navigation

This documentation follows the **Diataxis framework**, organizing content into four quadrants based on your needs:

| Quadrant | When to use | Start here |
|----------|-------------|------------|
| 🎓 **[Tutorial](tutorial/)** | You're learning and want step-by-step guidance | [Your First CLI](tutorial/01-your-first-cli.md) |
| 📘 **[How-To](howto/)** | You have a specific problem to solve | [How-To Index](howto/README.md) |
| 📚 **[Reference](reference/)** | You need to look up API details | [API Overview](reference/api/overview.md) |
| 💡 **[Explanation](explanation/)** | You want to understand the "why" | [Architecture](explanation/architecture/) |

---

## 🎓 Tutorial (Learning-Oriented)

**Goal**: Take you from zero to productive with hands-on, step-by-step guidance.

### Getting Started (30 minutes)
- [01. Your First CLI in 5 Minutes](tutorial/01-your-first-cli.md) - Hello World to working CLI
- [02. Domain Separation Architecture](tutorial/02-domain-separation.md) - Separating CLI from business logic
- [03. Adding Multiple Commands](tutorial/03-adding-commands.md) - Multi-command CLIs

### Intermediate (1-2 hours)
- [04. Testing Basics](tutorial/04-testing-basics.md) - Chicago TDD fundamentals
- [05. Output Formats](tutorial/05-output-formats.md) - JSON, YAML, Table outputs
- [06. Autonomic Features](tutorial/06-autonomic-features.md) - Machine-grade introspection

### Advanced (2-3 hours)
- [07. Async Operations](tutorial/07-async-operations.md) - Async command handlers
- [08. Error Handling](tutorial/08-error-handling.md) - Result<T,E> patterns
- [09. Deployment Basics](tutorial/09-deployment-basics.md) - Getting to production
- [10. Next Steps](tutorial/10-next-steps.md) - Paths to mastery

---

## 📘 How-To Guides (Problem-Solving)

**Goal**: Solve specific problems with practical, production-ready recipes.

### Production Patterns
- [Deploy to Production](howto/production/deployment.md) - Deployment strategies
- [Monitor with OTEL](howto/production/monitoring.md) - Observability integration
- [Configure Applications](howto/production/configuration.md) - Config management
- [Secure Your CLI](howto/production/security.md) - Security hardening

### Testing Strategies
- [Chicago TDD in Rust](howto/testing/chicago-tdd.md) - State-based testing
- [Integration Tests](howto/testing/integration-tests.md) - End-to-end testing
- [Property Tests](howto/testing/property-tests.md) - Property-based testing
- [Snapshot Tests](howto/testing/snapshot-tests.md) - Regression testing

### Integration Recipes
- [MCP Server Setup](howto/integration/mcp-servers.md) - Model Context Protocol
- [RDF/SPARQL Integration](howto/integration/rdf-sparql.md) - Semantic CLI
- [Async I/O Patterns](howto/integration/async-io.md) - Async operations
- [Database Connections](howto/integration/databases.md) - Database integration

### Common Patterns
- [Argument Parsing](howto/patterns/argument-parsing.md) - Complex argument patterns
- [Error Recovery](howto/patterns/error-recovery.md) - Error handling strategies
- [Output Formatting](howto/patterns/output-formatting.md) - Custom formats
- [Context Sharing](howto/patterns/context-sharing.md) - AppContext patterns

### Troubleshooting
- [Common Errors](howto/troubleshooting/common-errors.md) - Compilation issues
- [Runtime Issues](howto/troubleshooting/runtime-issues.md) - Debugging guide
- [Performance Tuning](howto/troubleshooting/performance.md) - Optimization

---

## 📚 Reference (Information-Oriented)

**Goal**: Provide accurate, concise API information for quick lookups.

### Core API
- [API Overview](reference/api/overview.md) - Structure of the API
- [#[verb] Macro](reference/api/verb-macro.md) - Complete macro reference
- [#[arg] Attributes](reference/api/arg-attributes.md) - Argument attributes
- [Types Catalog](reference/api/types.md) - All public types
- [Traits Reference](reference/api/traits.md) - Trait requirements
- [Errors Catalog](reference/api/errors.md) - Error types

### Autonomic CLI Layer API
- [Introspection API](reference/autonomic/introspection.md) - `--capabilities`, `--introspect`
- [Effect Metadata](reference/autonomic/effects.md) - Effect types & sensitivity
- [Plane Interactions](reference/autonomic/planes.md) - O/Σ/Q/ΔΣ planes
- [Guards & Budgets](reference/autonomic/guards.md) - Resource constraints
- [Execution Receipts](reference/autonomic/receipts.md) - Structured audit logs

### RDF/SPARQL API
- [CLI Ontology](reference/rdf/ontology.md) - RDF schema reference
- [SPARQL Queries](reference/rdf/sparql-queries.md) - Query patterns
- [SHACL Shapes](reference/rdf/shacl-shapes.md) - Validation rules

### CLI Reference
- [CLI Commands](reference/cli-commands.md) - All flags & options
- [Environment Variables](reference/environment-vars.md) - Env var reference
- [Configuration Files](reference/configuration.md) - Config file format

---

## 💡 Explanation (Understanding-Oriented)

**Goal**: Explain the "why" behind design decisions and architecture.

### Architecture Philosophy
- [Domain Separation](explanation/architecture/domain-separation.md) - Why domain-first design
- [Type-First Thinking](explanation/architecture/type-first-thinking.md) - Type-driven development
- [Zero-Cost Abstractions](explanation/architecture/zero-cost-abstractions.md) - Performance philosophy
- [Chicago TDD Rationale](explanation/architecture/chicago-tdd.md) - Testing philosophy

### Autonomic CLI Design
- [Machine-Grade Interfaces](explanation/autonomic/machine-grade-cli.md) - CLI as contracts
- [MAPE-K Loop Integration](explanation/autonomic/mape-k-loops.md) - Autonomic computing
- [Agent2028 Vision](explanation/autonomic/agent2028.md) - Trillion-agent ecosystems
- [Deterministic Execution](explanation/autonomic/determinism.md) - Predictable behavior

### Semantic CLI Design
- [RDF Rationale](explanation/semantic/rdf-rationale.md) - Why RDF for CLIs
- [SPARQL Benefits](explanation/semantic/sparql-benefits.md) - Query advantages
- [Ontology Design Principles](explanation/semantic/ontology-design.md) - Schema design

### Framework Comparisons
- [vs Pure Clap](explanation/comparisons/vs-clap.md) - Comparison with clap
- [vs Python Typer](explanation/comparisons/vs-typer.md) - Rust vs Python
- [vs Go Cobra](explanation/comparisons/vs-cobra.md) - Rust vs Go

### Future Direction
- [Roadmap](explanation/roadmap.md) - Planned features and vision

---

## 📖 Additional Resources

### Quick References
- [README.md](../README.md) - Project overview and quick start
- [QUICKSTART.md](QUICKSTART.md) - 10-minute quick start guide *(legacy v4, being migrated)*
- [AUTONOMIC.md](../AUTONOMIC.md) - Autonomic layer overview *(legacy, use explanation/autonomic/ instead)*

### Legacy Documentation (v4)
These documents are being migrated to the Diataxis structure:
- [CLI_REFERENCE.md](CLI_REFERENCE.md) - v4.0.2 reference *(outdated, use reference/ instead)*
- [CLI_COOKBOOK.md](CLI_COOKBOOK.md) - v4 recipes *(being migrated to howto/)*
- [CLI_TROUBLESHOOTING.md](CLI_TROUBLESHOOTING.md) - v4 troubleshooting *(being migrated)*

### Meta Documentation
- [CAPABILITY_CATALOG.md](CAPABILITY_CATALOG.md) - Complete feature catalog (v5.1.1)
- [GAP_ANALYSIS.md](GAP_ANALYSIS.md) - Documentation gap analysis
- [DOC_VALIDATION_REPORT.md](DOC_VALIDATION_REPORT.md) - Documentation validation
- [DIATAXIS_ARCHITECTURE_V5.md](DIATAXIS_ARCHITECTURE_V5.md) - Architecture specification

### Development
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [CHANGELOG.md](../CHANGELOG.md) - Version history

---

## 🗺️ Finding What You Need

### "I want to learn clap-noun-verb from scratch"
→ Start with [Tutorial 01: Your First CLI](tutorial/01-your-first-cli.md)

### "I need to solve a specific problem"
→ Browse [How-To Guides](howto/README.md) by category

### "I need to look up an API signature"
→ Check [Reference](reference/) → find your topic

### "I want to understand why something works this way"
→ Read [Explanation](explanation/) → architecture & philosophy

### "I'm upgrading from v4 to v5"
→ See [Migration Guide](howto/production/migration-v4-to-v5.md) *(coming soon)*

### "I'm an AI agent bootstrapping a CLI"
→ Start with [Autonomic Features Tutorial](tutorial/06-autonomic-features.md)

---

## 📊 Documentation Status

**Overall Progress**: ⚠️ **Migration in Progress**

| Quadrant | Status | Files | Progress |
|----------|--------|-------|----------|
| 🎓 Tutorial | ⏳ **Planned** | 0/10 | 0% |
| 📘 How-To | ⏳ **Planned** | 0/21 | 0% |
| 📚 Reference | ⏳ **Planned** | 0/19 | 0% |
| 💡 Explanation | ⏳ **Planned** | 0/17 | 0% |

**Notes**:
- v4 documentation (QUICKSTART.md, CLI_REFERENCE.md) remains available during migration
- New v5.1.1 documentation follows Diataxis framework
- Target completion: 6 weeks (see [DIATAXIS_IMPLEMENTATION_GUIDE.md](DIATAXIS_IMPLEMENTATION_GUIDE.md))

---

## 🔗 External Resources

- **Diataxis Framework**: https://diataxis.fr/
- **GitHub Repository**: https://github.com/seanchatmangpt/clap-noun-verb
- **Crates.io**: https://crates.io/crates/clap-noun-verb
- **Docs.rs**: https://docs.rs/clap-noun-verb

---

**Need help?** Check the [troubleshooting guide](howto/troubleshooting/) or [open an issue](https://github.com/seanchatmangpt/clap-noun-verb/issues).
