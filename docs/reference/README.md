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

- **[#[verb] Macro](api/verb-macro.md)** - Complete macro reference with all syntax variations
- **[#[arg] Attributes](api/arg-attributes.md)** - All argument attributes and their effects
- **[#[noun] Macro](api/noun-macro.md)** - **DEPRECATED** - See deprecation notice
- **[Types Catalog](api/types.md)** - All public types with descriptions and examples
- **[Errors Catalog](api/errors.md)** - Complete error type hierarchy
- **[CLI Runner](api/cli-runner.md)** - `run()` function and entry point

> **Note:** Autonomic CLI Layer API and RDF/SPARQL API documentation is planned for future releases. See [AUTONOMIC.md](../AUTONOMIC.md) for current capabilities.

### CLI Reference

**Command-line interface and configuration**

- **[Configuration](configuration.md)** - Config file format and schema
- **[Performance SLOs](performance-slos.md)** - Performance targets and guarantees

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
| **#[noun] macro** | [api/noun-macro.md](api/noun-macro.md) (deprecated) |
| **Result<T,E>** | [api/types.md](api/types.md) |
| **NounVerbError** | [api/errors.md](api/errors.md) |
| **OutputFormat** | [api/types.md](api/types.md) |

### By Use Case

| Use Case | Reference |
|----------|-----------|
| **Define a command** | [api/verb-macro.md](api/verb-macro.md) |
| **Parse arguments** | [api/arg-attributes.md](api/arg-attributes.md) |
| **Handle errors** | [api/errors.md](api/errors.md) |
| **Format output** | [api/types.md](api/types.md) |
| **Run CLI** | [api/cli-runner.md](api/cli-runner.md) |

---

## Reference Status

| Section | Files | Status |
|---------|-------|--------|
| **Core API** | 6 files | ✅ **Available** |
| **CLI** | 2 files | ✅ **Available** |
| **TOTAL** | **8 files** | ✅ **Available** |

> **Planned:** Autonomic API, RDF/SPARQL API coming in future releases.

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

**Note**: Reference documentation follows Diataxis principles with validated v5.6.1 APIs.
