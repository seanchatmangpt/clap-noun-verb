# ggen-clap-noun-verb Integration Architecture

**Version:** 1.0.0
**Date:** 2026-01-06
**Status:** Design Phase Complete

## Executive Summary

This document defines the type-first integration architecture for bringing ggen's ontology-driven code generation capabilities into the clap-noun-verb ecosystem.

## Architecture Principles

### 1. Type-First Thinking
- Types encode invariants at compile time
- Invalid states are unrepresentable
- Compiler as design tool for correctness

### 2. Zero-Cost Abstractions
- Use generics for monomorphization (zero-cost)
- Const generics where applicable
- References over owned values

### 3. Deterministic Outputs
- All code generation must be reproducible
- Receipt-based validation
- Immutable data structures

### 4. Feature-Gated Integration
- Optional ggen feature for minimal dependency burden
- Clean separation of concerns
- No runtime overhead when disabled

## System Architecture

### Component Overview

```
clap-noun-verb/
├── src/
│   ├── integration/
│   │   ├── mod.rs                    # Existing integration layer
│   │   ├── ggen/                     # NEW: ggen integration module
│   │   │   ├── mod.rs                # Public API & re-exports
│   │   │   ├── core.rs               # ggen-core wrapper
│   │   │   ├── domain.rs             # ggen-domain wrapper
│   │   │   ├── generator.rs          # High-level Generator API
│   │   │   ├── graph.rs              # RDF Graph integration
│   │   │   ├── pipeline.rs           # Template pipeline integration
│   │   │   ├── config.rs             # Configuration types
│   │   │   └── error.rs              # Error types (Result-based)
│   │   └── ...
│   └── ...
└── vendors/
    └── ggen/                         # ggen workspace (v5.2.0)
        ├── crates/
        │   ├── ggen-core/            # Core code generation
        │   ├── ggen-domain/          # Domain logic
        │   └── ...
        └── ...
```

### Dependency Graph

```
clap-noun-verb (v5.3.4)
└── [optional: ggen feature]
    ├── ggen-core (v5.0.0) from vendors/ggen
    └── ggen-domain (v5.0.0) from vendors/ggen
        └── ggen-core (v5.0.0)
```

### Integration Layers

#### Layer 1: Core Integration (`src/integration/ggen/core.rs`)
Wraps `ggen-core` types with clap-noun-verb ergonomics:
- `Generator` - Code generation engine
- `Pipeline` - Template processing pipeline
- `Template` - Template representation
- `Graph` - RDF graph management

#### Layer 2: Domain Integration (`src/integration/ggen/domain.rs`)
Wraps `ggen-domain` business logic:
- `ProjectGenerator` - Project scaffolding
- `TemplateOperations` - Template CRUD operations
- `OntologyOperations` - RDF/SPARQL operations

#### Layer 3: High-Level API (`src/integration/ggen/generator.rs`)
Provides ergonomic, type-safe API:
- Builder pattern for configuration
- Result-based error handling (no unwrap/expect)
- Async-first design
- Zero-copy where possible

## Feature Gates

### New Cargo Feature: `ggen`

```toml
[features]
# ggen integration - ontology-driven code generation
ggen = [
    "dep:ggen-core",
    "dep:ggen-domain",
    "rdf",           # Requires RDF support
    "crypto",        # Requires cryptographic receipts
]
```

### Conditional Compilation

```rust
#[cfg(feature = "ggen")]
pub mod ggen;
```

## Type Design

### Core Types (Type-First Design)

```rust
/// Generator configuration (builder pattern)
#[derive(Debug, Clone)]
pub struct GgenConfig {
    template_path: NonEmptyPath,
    output_path: NonEmptyPath,
    variables: BTreeMap<String, String>,
    rdf_graph: Option<Graph>,
}

/// Generation result (Result-based error handling)
pub type GgenResult<T> = Result<T, GgenError>;

/// Generation receipt (deterministic proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationReceipt {
    pub template_hash: Blake3Hash,
    pub output_hash: Blake3Hash,
    pub timestamp: Timestamp,
    pub variables: BTreeMap<String, String>,
}

/// Generator (zero-cost abstraction)
pub struct GgenGenerator<S: State = Configured> {
    config: GgenConfig,
    _state: PhantomData<S>,
}

/// Type-state pattern for compile-time validation
pub trait State {}
pub struct Configured;
pub struct Generated;
impl State for Configured {}
impl State for Generated {}
```

### Error Handling (Result-Based)

```rust
#[derive(Debug, thiserror::Error)]
pub enum GgenError {
    #[error("Template not found: {0}")]
    TemplateNotFound(PathBuf),

    #[error("Invalid RDF graph: {0}")]
    InvalidGraph(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Core error: {0}")]
    Core(#[from] ggen_core::Error),

    #[error("Domain error: {0}")]
    Domain(#[from] ggen_domain::Error),
}
```

## API Design

### Builder Pattern (Ergonomic)

```rust
use clap_noun_verb::integration::ggen::GgenGenerator;

let receipt = GgenGenerator::new()
    .template("templates/rust-cli.tera")?
    .output("output/my-cli")?
    .variable("name", "my-cli")?
    .variable("author", "Alice")?
    .with_rdf_graph(graph)?
    .generate()
    .await?;
```

### Functional API (Type-Safe)

```rust
use clap_noun_verb::integration::ggen::{generate_from_template, TemplateConfig};

let config = TemplateConfig::builder()
    .template("templates/rust-cli.tera")
    .output("output/my-cli")
    .build()?;

let receipt = generate_from_template(config).await?;
```

## Integration Points

### 1. CLI Commands (Noun-Verb Pattern)

```rust
#[noun]
struct Template;

#[verb(Template, "generate")]
async fn generate_template(
    #[arg] template: PathBuf,
    #[arg] output: PathBuf,
) -> Result<()> {
    let receipt = GgenGenerator::new()
        .template(&template)?
        .output(&output)?
        .generate()
        .await?;

    println!("Generated: {}", receipt.output_hash);
    Ok(())
}
```

### 2. RDF Graph Integration

```rust
use clap_noun_verb::rdf::OntologyGraph;
use clap_noun_verb::integration::ggen::GgenGenerator;

let graph = OntologyGraph::new()?;
graph.load_turtle("schema.ttl")?;

let receipt = GgenGenerator::new()
    .template("templates/from-ontology.tera")?
    .output("output")?
    .with_rdf_graph(graph.inner())?
    .generate()
    .await?;
```

### 3. Middleware Integration

```rust
use clap_noun_verb::integration::ggen::middleware::GgenMiddleware;

let executor = CommandExecutor::builder()
    .middleware(GgenMiddleware::new())
    .build();
```

## Testing Strategy (Chicago TDD)

### Unit Tests (State-Based)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_builder() {
        // Arrange
        let generator = GgenGenerator::new()
            .template("templates/test.tera")
            .unwrap()
            .output("output")
            .unwrap();

        // Act
        let config = generator.config();

        // Assert
        assert_eq!(config.template_path.as_ref(), Path::new("templates/test.tera"));
        assert_eq!(config.output_path.as_ref(), Path::new("output"));
    }
}
```

### Integration Tests (Behavior Verification)

```rust
#[tokio::test]
async fn test_end_to_end_generation() {
    // Arrange
    let temp_dir = tempfile::tempdir().unwrap();
    let template_path = temp_dir.path().join("template.tera");
    let output_path = temp_dir.path().join("output");

    std::fs::write(&template_path, "Hello {{ name }}!").unwrap();

    // Act
    let receipt = GgenGenerator::new()
        .template(&template_path).unwrap()
        .output(&output_path).unwrap()
        .variable("name", "World").unwrap()
        .generate()
        .await
        .unwrap();

    // Assert
    let output_content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(output_content, "Hello World!");
    assert!(receipt.output_hash.is_valid());
}
```

## Performance SLOs

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Template load | ≤10ms | In-memory caching |
| RDF graph query | ≤50ms | SPARQL optimization |
| Code generation | ≤100ms | Template rendering |
| Receipt generation | ≤5ms | Blake3 hashing |

## Security Considerations

### 1. Path Validation
- Use `NonEmptyPath` type for compile-time validation
- Prevent path traversal attacks
- Validate output paths

### 2. Template Sandboxing
- Restrict Tera template capabilities
- No arbitrary code execution
- Whitelist allowed functions

### 3. RDF Graph Security
- SPARQL query timeouts
- Memory limits for large graphs
- Input validation

## Migration Path

### Phase 1: Foundation (Week 1)
- [ ] Create `src/integration/ggen/` module structure
- [ ] Implement core wrapper types
- [ ] Add `ggen` feature to Cargo.toml

### Phase 2: API Implementation (Week 1)
- [ ] Implement `GgenGenerator` with builder pattern
- [ ] Implement error types
- [ ] Add type-state pattern

### Phase 3: Testing (Week 1)
- [ ] Unit tests for all public APIs
- [ ] Integration tests for code generation
- [ ] Property-based tests with proptest

### Phase 4: Documentation (Week 1)
- [ ] API documentation (rustdoc)
- [ ] Examples in `/examples/integration/ggen/`
- [ ] User guide

### Phase 5: Validation (Week 1)
- [ ] Andon signal checks (cargo make check/test/lint)
- [ ] Performance benchmarking
- [ ] Security audit

## Success Criteria

### Compile-Time Safety
- ✅ All paths validated with `NonEmptyPath`
- ✅ No `unwrap()` or `expect()` in production code
- ✅ Type-state pattern prevents invalid states

### Runtime Performance
- ✅ All SLOs met (≤100ms code generation)
- ✅ Zero-cost abstractions (no heap allocations in hot path)
- ✅ Efficient memory usage (≤10MB)

### Quality Assurance
- ✅ 100% of public API covered by tests
- ✅ All Andon signals green (check, test, lint)
- ✅ No compiler warnings or errors

### Developer Experience
- ✅ Ergonomic builder pattern API
- ✅ Clear error messages
- ✅ Comprehensive documentation

## References

- [ggen Documentation](https://github.com/seanchatmangpt/ggen)
- [clap-noun-verb API](https://docs.rs/clap-noun-verb)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Zero-Cost Abstractions](https://doc.rust-lang.org/book/ch19-00-patterns.html)

## Appendix A: Type Hierarchy

```
GgenGenerator<S: State>
├── GgenConfig
│   ├── NonEmptyPath (template_path)
│   ├── NonEmptyPath (output_path)
│   ├── BTreeMap<String, String> (variables)
│   └── Option<Graph> (rdf_graph)
└── PhantomData<S>

GenerationReceipt
├── Blake3Hash (template_hash)
├── Blake3Hash (output_hash)
├── Timestamp
└── BTreeMap<String, String> (variables)
```

## Appendix B: Feature Matrix

| Feature | Base clap-noun-verb | +ggen | +ggen+rdf |
|---------|---------------------|-------|-----------|
| Template generation | ❌ | ✅ | ✅ |
| RDF graph queries | ❌ | ❌ | ✅ |
| Ontology validation | ❌ | ❌ | ✅ |
| Code generation receipts | ❌ | ✅ | ✅ |
| SPARQL queries | ❌ | ❌ | ✅ |

---

**Architecture Status:** ✅ Design Complete
**Next Phase:** Implementation
**Orchestrator:** Task Orchestrator Agent
**Date:** 2026-01-06
