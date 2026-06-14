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
- **[API Catalog](api-catalog.md)** - Quick reference for all APIs
- **[Types Catalog](api/types.md)** - All public types with descriptions
- **[Errors Catalog](api/errors.md)** - Complete error type hierarchy
- **[Advanced Features](api/advanced-features.md)** - Completions, chaining, stdin, REPL, and introspection
- **[Telemetry Reference](api/telemetry.md)** - Distributed tracing context, W3C traceparents, and manager
- **[#[noun] Macro (DEPRECATED)](api/noun-macro.md)** - Legacy noun subcommand definition macro
- **[Error Codes](error-codes.md)** - Common errors and solutions

### CLI Reference

**Command-line interface and configuration**

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

| Component | Reference |
|-----------|-----------|
| **#[verb] macro** | [api/verb-macro.md](api/verb-macro.md) |
| **#[arg] attributes** | [api/arg-attributes.md](api/arg-attributes.md) |
| **Result<T,E>** | [api/types.md](api/types.md) |
| **NounVerbError** | [api/errors.md](api/errors.md) |
| **TraceContext / Telemetry** | [api/telemetry.md](api/telemetry.md) |
| **OutputFormat** | [api/types.md](api/types.md) |
| **Advanced Features** | [api/advanced-features.md](api/advanced-features.md) |
| **#[noun] macro (DEPRECATED)** | [api/noun-macro.md](api/noun-macro.md) |
| **API Catalog** | [api-catalog.md](api-catalog.md) |

---

## Reference Status

| Section | Files | Status |
|---------|-------|--------|
| **Core API** | 9 files | ✅ **Available** |
| **CLI** | 1 file | ✅ **Available** |
| **TOTAL** | **10 files** | ✅ **Available** |

---

## API Stability Guarantees

### Stable APIs (26.6.13)

These APIs are stable and follow semantic versioning:
- ✅ **Core API**: `#[verb]`, `#[arg]`, `Result<T,E>`, `OutputFormat`
- ✅ **Autonomic Layer**: Introspection, effects, guards, receipts

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
- **Have a question?** - [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb)

---

**Note**: Reference documentation follows Diataxis principles with validated 26.6.13 APIs.
