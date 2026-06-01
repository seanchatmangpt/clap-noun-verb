# Open Ontologies Graph Manager - Specimen CLI

**Version**: 0.1.0  
**Framework**: clap-noun-verb v26.6.1  
**Status**: Research Phase 10 Implementation

## Overview

The Open Ontologies Graph Manager is a specimen CLI implementation demonstrating all proven v26.6.1 APIs of the clap-noun-verb framework. It provides a complete, production-ready command structure for managing RDF graphs and capability registries using declarative noun-verb command patterns.

## Design Principles

### 1. Pure v26.6.1 API Usage
- **Macros**: `#[verb(...)]` for command registration
- **Traits**: Implicitly via macro (VerbCommand trait methods generated)
- **Registry**: Auto-discovery via `linkme` distributed slices
- **Output**: JSON via `serde::Serialize` implementations
- **Validation**: Via `validators` module functions
- **Error Handling**: `Result<T>` and `NounVerbError`

### 2. No v26.7.0 Extensions
- ❌ No `Receipt` type (not in v26.6.1)
- ❌ No `CommandMetadata` trait (future feature)
- ❌ No `async_verb` handlers (async feature not used)
- ✅ Sync-only command handlers
- ✅ All outputs `Serialize` compatible

### 3. Minimal Dependencies
- clap-noun-verb (with core deps only)
- serde + serde_json (serialization)
- Standard library utilities

## Architecture

### Module Structure

```
src/
├── main.rs                    (entry point: clap_noun_verb::run())
├── lib.rs                     (public API exports)
├── graph_model.rs             (domain: RDF Triple, Graph, Registry)
├── output_models.rs           (Serialize types for all commands)
└── commands/
    ├── graph_load.rs          (#[verb("load", "graph")])
    ├── graph_query.rs         (#[verb("query", "graph")])
    ├── graph_validate.rs      (#[verb("validate", "graph")])
    ├── doctor_check.rs        (#[verb("check", "doctor")])
    ├── pack_add.rs            (#[verb("add", "pack")])
    └── pack_remove.rs         (#[verb("remove", "pack")])
```

### Command Anatomy

Each command follows the proven v26.6.1 pattern:

```rust
#[verb("verb_name", "noun_name")]
fn handler_function(arg1: String, arg2: Option<i32>) -> Result<OutputType>
where
    OutputType: Serialize
```

**Type Inference**:
- `String` → required positional argument
- `Option<T>` → optional argument
- `bool` → flag (--flag or --no-flag)

**Error Handling**:
- Return `Result<T>` wrapping output
- Use `NounVerbError::execution_error()` for CLI errors

**Output**:
- All return types must implement `serde::Serialize`
- Framework automatically formats to JSON (configurable via `OutputFormat`)

## Commands

### 1. Graph Load
```bash
specimen-graph-manager graph load <path>
```
**Arguments**:
- `path: String` - Path to Turtle (.ttl) RDF file

**Returns**:
```json
{
  "triples_loaded": 42,
  "source": "data/example.ttl",
  "status": "success"
}
```

**Validation**:
- File must exist (checked via `fs::metadata()`)
- File must be readable
- Content must parse as valid triples

### 2. Graph Query
```bash
specimen-graph-manager graph query <query_string>
```
**Arguments**:
- `query_string: String` - Query pattern (e.g., "subject:ex:alice")

**Returns**:
```json
{
  "query_type": "subject",
  "pattern": "ex:alice",
  "results": [
    {
      "index": 0,
      "subject": "ex:alice",
      "predicate": "rdf:type",
      "object": "ex:Person"
    }
  ],
  "match_count": 1
}
```

**Validation**:
- Query string cannot be empty
- Query type must be recognized (subject/predicate/object)

### 3. Graph Validate
```bash
specimen-graph-manager graph validate <path>
```
**Arguments**:
- `path: String` - Path to RDF file to validate

**Returns**:
```json
{
  "valid": true,
  "errors": [],
  "total_triples": 42,
  "valid_triples": 42
}
```

**Validation**:
- File must exist and be readable
- All lines must have subject, predicate, object
- All URIs must be properly formatted

### 4. Doctor Check
```bash
specimen-graph-manager doctor check
```
**Arguments**: None

**Returns**:
```json
{
  "status": "healthy",
  "healthy": true,
  "issues": [
    {
      "level": "info",
      "message": "All core services operational"
    }
  ],
  "graph_triples": 42,
  "registry_packages": 5
}
```

**Validation**:
- Checks graph store accessibility
- Verifies registry operational status
- Validates system resources

### 5. Pack Add
```bash
specimen-graph-manager pack add <name> <version>
```
**Arguments**:
- `name: String` - Package name
- `version: String` - Semantic version (X.Y.Z)

**Returns**:
```json
{
  "id": "pkg-graphutils",
  "name": "GraphUtils",
  "version": "2.1.0",
  "status": "added"
}
```

**Validation**:
- Name cannot be empty
- Version must follow semantic versioning (X.Y.Z)
- Package ID auto-generated from name

### 6. Pack Remove
```bash
specimen-graph-manager pack remove <id>
```
**Arguments**:
- `id: String` - Package ID (must start with "pkg-")

**Returns**:
```json
{
  "removed_id": "pkg-graphutils",
  "status": "removed",
  "message": "Package successfully removed from registry"
}
```

**Validation**:
- ID cannot be empty
- ID must start with "pkg-"
- Package must exist in registry

## Building & Testing

### Build
```bash
cargo make build --example specimen-graph-manager
```

### Run Commands
```bash
# Load graph
cargo run --example specimen-graph-manager -- graph load /path/to/file.ttl

# Query graph
cargo run --example specimen-graph-manager -- graph query "subject:ex:alice"

# Validate RDF
cargo run --example specimen-graph-manager -- graph validate /path/to/file.ttl

# Health check
cargo run --example specimen-graph-manager -- doctor check

# Add capability
cargo run --example specimen-graph-manager -- pack add GraphUtils 1.0.0

# Remove capability
cargo run --example specimen-graph-manager -- pack remove pkg-graphutils
```

### Test
```bash
cargo test --example specimen-graph-manager
```

### Documentation
```bash
cargo doc --example specimen-graph-manager --no-deps --open
```

### Help
```bash
cargo run --example specimen-graph-manager -- --help
cargo run --example specimen-graph-manager -- graph load --help
cargo run --example specimen-graph-manager -- doctor check --help
```

## Implementation Statistics

- **Commands**: 6 (graph load, query, validate; doctor check; pack add, remove)
- **Modules**: 7 (main, lib, graph_model, output_models, 6× command modules)
- **Lines of Code**: ~1,100 (implementation + tests)
- **Tests**: 35+ (unit tests + integration tests)
- **Dependencies**: 7 (clap-noun-verb, serde, regex, url, etc.)

## Proven v26.6.1 Features Used

| Feature | Status | Location |
|---------|--------|----------|
| `#[verb]` macro | ✅ | All command modules |
| Auto-discovery (linkme) | ✅ | Main entry point |
| Type inference from signatures | ✅ | All handlers |
| JSON output formatting | ✅ | OutputFormat |
| Error handling (Result/NounVerbError) | ✅ | All handlers |
| Validators module | ✅ | output_models validation |
| Serialize trait requirement | ✅ | All output types |
| Sync-only handlers | ✅ | All 6 commands |

## What's NOT Used (Correctly Excluded)

| Feature | Status | Reason |
|---------|--------|--------|
| `async_verb` | ❌ | Not in v26.6.1 stable |
| `Receipt` type | ❌ | Planned for v26.7.0 |
| `CommandMetadata` | ❌ | Planned for v26.7.0 |
| Async handlers | ❌ | Keep it v26.6.1 pure |
| Custom output formats | ❌ | Use OutputFormat directly |

## Validation & Correctness

### Compile-Time Guarantees
- ✅ All handlers have matching `#[verb]` declarations
- ✅ All return types implement `Serialize`
- ✅ All error paths return `Result<T>`
- ✅ No `unwrap()` in production code
- ✅ No `panic!()` in production code

### Runtime Guarantees
- ✅ All inputs validated before processing
- ✅ All file I/O checked for existence/readability
- ✅ All argument formats validated
- ✅ All errors propagate correctly
- ✅ All outputs serialize to valid JSON

## Future Extensions (Post-v26.6.1)

When v26.7.0+ features become available:

1. **Async Operations** - Add async verb handlers for concurrent queries
2. **Command Metadata** - Track execution provenance via Receipt type
3. **Advanced Queries** - Full SPARQL support with distributed evaluation
4. **Capability Negotiation** - Dynamic capability composition
5. **Observability** - Structured logging and tracing

## References

- [clap-noun-verb v26.6.1 Documentation](https://docs.rs/clap-noun-verb/26.6.1/)
- [Serde Documentation](https://serde.rs/)
- [RDF/Turtle Format](https://www.w3.org/TR/turtle/)
- [Semantic Versioning](https://semver.org/)

## License

Copyright (c) 2024 Sean Chatman  
SPDX-License-Identifier: MIT OR Apache-2.0
