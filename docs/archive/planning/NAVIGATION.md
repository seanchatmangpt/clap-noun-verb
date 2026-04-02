# Documentation Navigation Graph

**Purpose:** Complete navigation map for AI coding agents using clap-noun-verb v5.

---

## Diataxis Framework

This documentation follows the [Diataxis](https://diataxis.fr/) systematic approach:

```
                Learning-oriented │ Understanding-oriented
                                 │
        ─────────────────────────┼─────────────────────────
                                 │
         Practical & Problem     │  Theoretical & Explanation
                                 │
```

---

## Quick Start (By Use Case)

### 🆕 "I want to build a new CLI from scratch"
1. **START:** [Tutorial: Quickstart](tutorial/quickstart.md) (5 min)
2. **THEN:** [How-To: Domain Separation Patterns](how-to/domain-separation-patterns.md)
3. **THEN:** [Examples: Complete Projects](examples/domain-separation/)

### 🔧 "I need to solve a specific problem"
1. **START:** [How-To Guides](how-to/) - Find your problem
2. **REFERENCE:** [API Catalog](reference/api-catalog.md) - Look up types
3. **EXAMPLES:** [Domain Separation Examples](examples/domain-separation/)

### 📚 "I need to look up a type/function"
1. **START:** [Reference: API Catalog](reference/api-catalog.md)
2. **DETAILS:** Check specific sections (macros, errors, telemetry)

### 💡 "I want to understand WHY it's designed this way"
1. **START:** [Explanation: Architecture](explanation/architecture.md)
2. **DEEP DIVE:** [AUTONOMIC.md](../AUTONOMIC.md) - v5 machine interface
3. **RESEARCH:** [PhD_THESIS.md](../PhD_THESIS.md) - Agent2028 vision

---

## Documentation Status

### Current Documentation (v5.0.0+)
All documentation in this directory is current and maintained for v5.0.0+ unless marked otherwise.

**Historical Documentation Policy:**
- Historical release documentation (v4.0.0, v3.x) has been archived/removed
- Release history is preserved in `CHANGELOG.md` in the project root
- Session summaries and ephemeral notes are not kept in documentation
- Roadmaps for completed versions are removed (see CHANGELOG.md for history)

**If you find outdated documentation:**
1. Check if it's referenced in current docs
2. If not referenced and clearly outdated, it should be removed
3. Historical information belongs in CHANGELOG.md, not separate files

---

## Document Index

### 🎓 Tutorial (Learning-Oriented)
**Goal:** Get agents up and running fast

| Document | Time | Purpose |
|----------|------|---------|
| [Quickstart](tutorial/quickstart.md) | 5 min | Build first domain-separated CLI |

### 📘 How-To Guides (Problem-Oriented)
**Goal:** Solve specific implementation problems

| Document | Purpose |
|----------|---------|
| [Domain Separation Patterns](how-to/domain-separation-patterns.md) | 5 production patterns for separating concerns |

**Coming Soon in v5 docs:**
- Error Handling Patterns
- Testing Strategies (Chicago TDD)
- MCP Server Integration
- Deployment Patterns
- Performance Optimization

### 📚 Reference (Information-Oriented)
**Goal:** Complete technical reference for lookup

| Document | Purpose |
|----------|---------|
| [API Catalog](reference/api-catalog.md) | Complete type signatures and API surface |

**Coming Soon in v5 docs:**
- Macro System Deep Dive
- Error Type Catalog
- v5 Autonomic API Reference
- Feature Flag Matrix

### 💡 Explanation (Understanding-Oriented)
**Goal:** Understand concepts and design rationale

| Document | Purpose |
|----------|---------|
| [Architecture & Philosophy](explanation/architecture.md) | Why clap-noun-verb is designed this way |
| [AUTONOMIC.md](../AUTONOMIC.md) | Machine-grade interface design |
| [PhD_THESIS.md](../PhD_THESIS.md) | Agent2028 ecosystem vision |

---

## Cross-References

### Domain Separation
- **Tutorial:** [Quickstart](tutorial/quickstart.md) § Step 2-3
- **How-To:** [Domain Separation Patterns](how-to/domain-separation-patterns.md)
- **Reference:** [API Catalog](reference/api-catalog.md) § Macro System
- **Explanation:** [Architecture](explanation/architecture.md) § Core Philosophy
- **Examples:** [Complete Projects](examples/domain-separation/)

### v5 Autonomic Features
- **Reference:** [API Catalog](reference/api-catalog.md) § v5 Autonomic API
- **Explanation:** [AUTONOMIC.md](../AUTONOMIC.md)
- **Architecture:** [Architecture](explanation/architecture.md) § Machine-Grade Interface

### Testing
- **How-To:** [Domain Separation Patterns](how-to/domain-separation-patterns.md) § Testing
- **Explanation:** [Architecture](explanation/architecture.md) § Why Chicago TDD
- **Examples:** All examples include test suites

### MCP Integration
- **Explanation:** [Architecture](explanation/architecture.md) § Integration with Agent Systems
- **Reference:** [AUTONOMIC.md](../AUTONOMIC.md) § MCP Protocol

---

## Learning Paths

### Path 1: New Project (Agent Bootstrapping)
```
Quickstart (5 min)
    ↓
Domain Separation Patterns (read patterns)
    ↓
Copy Template Example
    ↓
API Catalog (reference as needed)
```

### Path 2: Add Features to Existing CLI
```
Identify Problem
    ↓
How-To Guides (find pattern)
    ↓
API Catalog (look up types)
    ↓
Examples (see complete code)
```

### Path 3: Understand Design
```
Architecture Philosophy
    ↓
AUTONOMIC.md (v5 features)
    ↓
PhD_THESIS.md (future vision)
```

### Path 4: Agent Integration (MCP)
```
AUTONOMIC.md (introspection API)
    ↓
API Catalog (type signatures)
    ↓
Architecture (integration patterns)
```

---

## External Resources

### Official Documentation
- **Main README:** [README.md](../README.md) - Entry point
- **Migration:** [MIGRATION_V4_TO_V5.md](MIGRATION_V4_TO_V5.md) - Upgrade from v4
- **Changelog:** [CHANGELOG.md](../CHANGELOG.md) - Release history

### Research & Vision
- **Agent2028:** [PhD_THESIS.md](../PhD_THESIS.md)
- **Autonomic Layer:** [AUTONOMIC.md](../AUTONOMIC.md)
- **Semantic CLI:** [SEMANTIC_CLI_ARCHITECTURE.md](SEMANTIC_CLI_ARCHITECTURE.md)

### Examples
- **Domain Separation:**
  - [Data Processor](examples/domain-separation/data-processor/)
  - [API Client](examples/domain-separation/api-client/)
  - [Report Generator](examples/domain-separation/report-generator/)
  - [Template](examples/domain-separation/template/)
- **Anti-Patterns:** [Anti-Patterns Guide](examples/domain-separation/anti-patterns/)

---

## Quick Lookup

### By Concept
- **Domain Separation:** Tutorial § 2-3, How-To § Patterns, Explanation § Philosophy
- **Type Safety:** Explanation § Type-First Design, Reference § Type Signatures
- **Zero-Cost:** Explanation § Zero-Cost Abstractions, Architecture § Performance
- **Testing:** How-To § Testing Strategies, Explanation § Chicago TDD
- **MCP:** AUTONOMIC.md, Architecture § Integration

### By Task
- **Create new CLI:** Tutorial → How-To → Examples
- **Add commands:** API Catalog § Macros → Examples
- **Handle errors:** Reference § Error Types → How-To § Error Patterns
- **Add telemetry:** Reference § v5 Autonomic → AUTONOMIC.md
- **Integrate MCP:** AUTONOMIC.md → Architecture § MCP

### By Role
- **Coding Agent:** Tutorial → How-To → API Catalog
- **Architect:** Explanation → AUTONOMIC.md → PhD_THESIS.md
- **Debugger:** API Catalog § Errors → How-To § Anti-Patterns
- **Maintainer:** Explanation → Migration Guide → Changelog

---

## Meta

**Framework:** Diataxis v2.0
**Target Audience:** AI coding agents, senior Rust engineers
**Completeness:**
- ✅ Tutorial (complete)
- ✅ How-To (1/5 guides, more coming in v5 docs)
- ✅ Reference (1/4 catalogs, more coming in v5 docs)
- ✅ Explanation (complete)

**Last Updated:** 2025-11-20
**Version:** v5.0.0
