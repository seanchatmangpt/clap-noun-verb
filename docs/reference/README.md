# 📚 Reference: Complete API Documentation

**Welcome!** This reference provides accurate, concise API information for quick lookups.

---

## When to Use Reference Documentation

Use the reference when you:
- ✅ Need to look up API signatures and types
- ✅ Want to check what arguments a macro accepts
- ✅ Need to understand error types
- ✅ Are looking for trait requirements
- ✅ Want complete documentation of the API surface

---

## Reference Sections

### Core API

**Complete catalog of the core clap-noun-verb API**

- **[API Overview](api/overview.md)** - API structure and organization
- **[#[verb] Macro](api/verb-macro.md)** - Complete macro reference with all syntax variations
- **[#[arg] Attributes](api/arg-attributes.md)** - All argument attributes and their effects
- **[Types Catalog](api/types.md)** - All public types with descriptions and examples
- **[Traits Reference](api/traits.md)** - Trait requirements and implementations
- **[Errors Catalog](api/errors.md)** - Complete error type hierarchy

### Autonomic CLI Layer API

**Machine-grade interface for agents and autonomous systems**

- **[Introspection API](autonomic/introspection.md)** - `--capabilities`, `--introspect`, `--graph` flags
- **[Effect Metadata](autonomic/effects.md)** - Effect types, sensitivity levels, idempotency
- **[Plane Interactions](autonomic/planes.md)** - O/Σ/Q/ΔΣ plane interactions
- **[Guards & Budgets](autonomic/guards.md)** - Resource constraints and enforcement
- **[Execution Receipts](autonomic/receipts.md)** - Structured audit log format

### RDF/SPARQL API

**Semantic CLI layer for intent-based command discovery**

- **[CLI Ontology](rdf/ontology.md)** - RDF schema reference for CLI commands
- **[SPARQL Queries](rdf/sparql-queries.md)** - Query patterns for command discovery
- **[SHACL Shapes](rdf/shacl-shapes.md)** - Validation rules and constraints

### CLI Reference

**Command-line interface and configuration**

- **[CLI Commands](cli-commands.md)** - All CLI flags, options, and subcommands
- **[Environment Variables](environment-vars.md)** - Environment variable reference
- **[Configuration Files](configuration.md)** - Config file format and schema

---

## Reference Format

Each reference page provides:

1. **Signatures** - Complete type signatures and syntax
2. **Parameters** - All parameters with types and defaults
3. **Return Types** - What the API returns
4. **Examples** - Minimal examples showing usage
5. **Related APIs** - Links to related references

---

## Quick Lookup

### By Component

| Component | Reference |
|-----------|-----------|
| **#[verb] macro** | [api/verb-macro.md](api/verb-macro.md) |
| **#[arg] attributes** | [api/arg-attributes.md](api/arg-attributes.md) |
| **Result<T,E>** | [api/types.md](api/types.md#result-type) |
| **NounVerbError** | [api/errors.md](api/errors.md) |
| **OutputFormat** | [api/types.md](api/types.md#outputformat) |
| **AppContext** | [api/types.md](api/types.md#appcontext) |

### By Feature

| Feature | Reference |
|---------|-----------|
| **Introspection** | [autonomic/introspection.md](autonomic/introspection.md) |
| **Effect Metadata** | [autonomic/effects.md](autonomic/effects.md) |
| **Guards** | [autonomic/guards.md](autonomic/guards.md) |
| **Receipts** | [autonomic/receipts.md](autonomic/receipts.md) |
| **RDF Ontology** | [rdf/ontology.md](rdf/ontology.md) |
| **SPARQL** | [rdf/sparql-queries.md](rdf/sparql-queries.md) |

### By Use Case

| Use Case | Reference |
|----------|-----------|
| **Define a command** | [api/verb-macro.md](api/verb-macro.md) |
| **Parse arguments** | [api/arg-attributes.md](api/arg-attributes.md) |
| **Handle errors** | [api/errors.md](api/errors.md) |
| **Format output** | [api/types.md](api/types.md#outputformat) |
| **Introspect CLI** | [autonomic/introspection.md](autonomic/introspection.md) |
| **Query commands** | [rdf/sparql-queries.md](rdf/sparql-queries.md) |

---

## Reference Status

| Section | Files | Status |
|---------|-------|--------|
| **Core API** | 6 files | ⏳ **Planned** |
| **Autonomic** | 5 files | ⏳ **Planned** |
| **RDF/SPARQL** | 3 files | ⏳ **Planned** |
| **CLI** | 3 files | ⏳ **Planned** |
| **TOTAL** | **19 files** | ⏳ **0% complete** |

---

## API Stability Guarantees

### Stable APIs (v5.1.1)

These APIs are stable and follow semantic versioning:
- ✅ **Core API**: `#[verb]`, `#[arg]`, `Result<T,E>`, `OutputFormat`
- ✅ **Autonomic Layer**: Introspection, effects, planes, guards, receipts
- ✅ **RDF Layer**: Ontology, SPARQL queries, SHACL validation

### Experimental APIs

These APIs are experimental and may change:
- ⚠️ **Agent2028**: Trust networks, quantum-safe crypto (simulated)
- ⚠️ **KGC Integration**: Knowledge graph integration (partial)

### Planned APIs (v5.2+)

These APIs are planned for future versions:
- ⏳ **Delegation Chains**: Capability transfer (Q1 2026)
- ⏳ **Deterministic Execution**: Guard enforcement (Q1 2026)

---

## Alternative Resources

### "I'm learning from scratch"
→ Start with [Tutorial](../tutorial/README.md) for step-by-step guidance

### "I need to solve a problem"
→ Check [How-To Guides](../howto/README.md) for production patterns

### "I want to understand the architecture"
→ Read [Explanation](../explanation/README.md) for design philosophy

---

## Contributing to Reference Documentation

Reference documentation must be:
- ✅ **Accurate** - All signatures and types must be correct
- ✅ **Complete** - Document all public APIs
- ✅ **Concise** - Minimal examples, no explanations
- ✅ **Tested** - All examples must compile and work

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## Getting Help

- **Can't find an API?** - [Request documentation](https://github.com/seanchatmangpt/clap-noun-verb/issues/new?template=doc_request.md)
- **API doesn't work as documented?** - [Report a bug](https://github.com/seanchatmangpt/clap-noun-verb/issues)
- **Have a question?** - [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb/discussions)

---

**Note**: During migration from v4 to v5.1.1 documentation, the legacy [CLI_REFERENCE.md](../CLI_REFERENCE.md) remains available. New reference documentation follows Diataxis principles with validated v5.1.1 APIs.
