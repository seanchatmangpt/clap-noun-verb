# 📘 How-To Guides: Problem-Solving with clap-noun-verb

**Welcome!** These guides help you solve specific problems with practical, production-ready recipes.

---

## When to Use How-To Guides

Use these guides when you:
- ✅ Have a specific problem to solve
- ✅ Need production-ready patterns
- ✅ Want to implement a particular feature
- ✅ Are looking for best practices

---

## Guide Categories

### 🚀 Production Patterns

**Goal**: Deploy and run CLIs in production environments

- **[Deploy to Production](production/deployment.md)** - Deployment strategies, cross-compilation, distribution
- **[Monitor with OTEL](production/monitoring.md)** - OpenTelemetry integration, distributed tracing
- **[Configure Applications](production/configuration.md)** - Config files, environment variables, secrets
- **[Secure Your CLI](production/security.md)** - Input validation, sandboxing, credential management

### 🧪 Testing Strategies

**Goal**: Test CLIs comprehensively with Chicago TDD

- **[Chicago TDD in Rust](testing/chicago-tdd.md)** - State-based testing, real collaborators, AAA pattern
- **[Integration Tests](testing/integration-tests.md)** - End-to-end CLI testing with assert_cmd
- **[Property Tests](testing/property-tests.md)** - Property-based testing with proptest
- **[Snapshot Tests](testing/snapshot-tests.md)** - Regression testing with insta

### 🔌 Integration Recipes

**Goal**: Integrate with external systems and protocols

- **[MCP Server Setup](integration/mcp-servers.md)** - Model Context Protocol server implementation
- **[RDF/SPARQL Integration](integration/rdf-sparql.md)** - Semantic CLI with RDF and SPARQL
- **[Async I/O Patterns](integration/async-io.md)** - Async operations, HTTP requests, database queries
- **[Database Connections](integration/databases.md)** - SQLite, PostgreSQL, async database drivers

### 🎨 Common Patterns

**Goal**: Implement common CLI patterns correctly

- **[Argument Parsing](patterns/argument-parsing.md)** - Complex argument patterns, subcommands, flags
- **[Error Recovery](patterns/error-recovery.md)** - Error handling strategies, retry logic, fallbacks
- **[Output Formatting](patterns/output-formatting.md)** - Custom formats, tables, progress bars
- **[Context Sharing](patterns/context-sharing.md)** - AppContext patterns, state management

### 🔧 Troubleshooting

**Goal**: Diagnose and fix common issues

- **[Common Errors](troubleshooting/common-errors.md)** - Compilation errors, macro issues, type mismatches
- **[Runtime Issues](troubleshooting/runtime-issues.md)** - Debugging strategies, logging, profiling
- **[Performance Tuning](troubleshooting/performance.md)** - Optimization strategies, benchmarking, profiling

---

## How-To Format

Each guide follows this structure:

1. **Problem Statement** - What problem does this solve?
2. **Solution Overview** - High-level approach
3. **Step-by-Step Instructions** - Detailed implementation
4. **Complete Example** - Working code you can copy
5. **Variations** - Alternative approaches
6. **Troubleshooting** - Common issues and fixes
7. **Related Guides** - Where to go next

---

## Finding the Right Guide

### By Task

| Task | Guide |
|------|-------|
| **Deploy my CLI** | [production/deployment.md](production/deployment.md) |
| **Add monitoring** | [production/monitoring.md](production/monitoring.md) |
| **Test my CLI** | [testing/chicago-tdd.md](testing/chicago-tdd.md) |
| **Connect to database** | [integration/databases.md](integration/databases.md) |
| **Parse complex arguments** | [patterns/argument-parsing.md](patterns/argument-parsing.md) |
| **Handle errors gracefully** | [patterns/error-recovery.md](patterns/error-recovery.md) |
| **Optimize performance** | [troubleshooting/performance.md](troubleshooting/performance.md) |

### By Technology

| Technology | Guide |
|------------|-------|
| **MCP** | [integration/mcp-servers.md](integration/mcp-servers.md) |
| **RDF/SPARQL** | [integration/rdf-sparql.md](integration/rdf-sparql.md) |
| **Tokio/Async** | [integration/async-io.md](integration/async-io.md) |
| **SQLx/Diesel** | [integration/databases.md](integration/databases.md) |
| **OTEL** | [production/monitoring.md](production/monitoring.md) |

### By Production Concern

| Concern | Guide |
|---------|-------|
| **Security** | [production/security.md](production/security.md) |
| **Configuration** | [production/configuration.md](production/configuration.md) |
| **Monitoring** | [production/monitoring.md](production/monitoring.md) |
| **Testing** | [testing/chicago-tdd.md](testing/chicago-tdd.md) |
| **Performance** | [troubleshooting/performance.md](troubleshooting/performance.md) |

---

## Guide Status

| Category | Guides | Status |
|----------|--------|--------|
| **Production** | 4 guides | ⏳ **Planned** |
| **Testing** | 4 guides | ⏳ **Planned** |
| **Integration** | 4 guides | ⏳ **Planned** |
| **Patterns** | 4 guides | ⏳ **Planned** |
| **Troubleshooting** | 3 guides | ⏳ **Planned** |
| **TOTAL** | **21 guides** | ⏳ **0% complete** |

---

## Contributing a How-To Guide

Have a recipe you want to share?

1. **Check existing guides** - Avoid duplicates
2. **Follow the format** - Use the structure above
3. **Test your code** - All examples must compile and work
4. **Submit a PR** - See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

## Alternative Resources

### "I'm learning from scratch"
→ Start with [Tutorial](../tutorial/README.md) for step-by-step guidance

### "I need API documentation"
→ Check [Reference](../reference/README.md) for API signatures

### "I want to understand the architecture"
→ Read [Explanation](../explanation/README.md) for design philosophy

---

## Getting Help

- **Can't find a guide?** - [Request a guide](https://github.com/seanchatmangpt/clap-noun-verb/issues/new?template=guide_request.md)
- **Guide doesn't work?** - [Report a bug](https://github.com/seanchatmangpt/clap-noun-verb/issues)
- **Have a question?** - [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb/discussions)

---

**Note**: During migration from v4 to v5.1.1 documentation, the legacy [CLI_COOKBOOK.md](../CLI_COOKBOOK.md) remains available. New how-to guides follow Diataxis principles with validated v5.1.1 code examples.
