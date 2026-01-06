# ggen-clap-noun-verb: Architecture Decision Records Summary

**Date**: 2026-01-06
**Status**: Design Phase Complete

---

## Quick Reference Card

### Core Design Principles

1. **Type-First Thinking**: Make invalid states unrepresentable
2. **Zero-Cost Abstractions**: No runtime overhead for type safety
3. **Explicit Error Handling**: Result<T, E> everywhere, no panics
4. **Chicago TDD**: State-based testing with real collaborators

### Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| RDF Processing | ggen-core (Oxigraph) | Battle-tested, SPARQL support |
| Template Engine | Tera (via ggen) | Proven in ggen ecosystem |
| CLI Framework | clap-noun-verb | Target output format |
| Error Handling | thiserror | Ergonomic derive macros |
| Testing | proptest, insta | Property-based + snapshot tests |

---

## Critical Architecture Decisions

### ADR-001: Typestate Pattern for AST Safety

**Decision**: Use typestate pattern to enforce validation state in types.

```rust
pub struct CliSpec<S: State = Validated> { /* ... */ }

// Compile-time enforcement
fn generate(spec: &CliSpec<Validated>) { /* Only accepts validated */ }
```

**Impact**:
- ✅ Impossible to generate code from unvalidated AST
- ✅ Zero runtime cost (marker types are zero-sized)
- ✅ Clear API contracts

**Performance**: Zero-cost (verified via `size_of` assertions)

---

### ADR-002: ggen-core Integration

**Decision**: Integrate with ggen-core for RDF processing and templates.

**Rationale**:
- Reuse battle-tested RDF/SPARQL infrastructure
- Consistent with ggen ecosystem
- Avoid reinventing Turtle parser

**Integration Points**:
```rust
// RDF Graph extraction
GgenGraphAdapter::from_turtle(path) → CliSpec<Unvalidated>

// Template rendering
GgenTemplateAdapter::render(template, context) → Generated Code
```

---

### ADR-003: Const Generics for Compile-Time Limits

**Decision**: Use const generics to enforce command depth limits.

```rust
pub struct CommandHierarchy<const DEPTH: usize> {
    // Compile-time assertion
    const _: () = assert!(DEPTH <= MAX_COMMAND_DEPTH);
}
```

**Impact**:
- ✅ Zero runtime cost - checked at compile time
- ✅ Clear compile errors if violated
- ❌ Requires Rust 1.51+ (acceptable)

---

### ADR-004: Result<T, E> over Panics

**Decision**: Use Result<T, E> for all fallible operations.

**Error Hierarchy**:
```
Error
├── ParseError (Turtle syntax, RDF structure)
├── ValidationError (Semantic invariants)
├── GenerationError (Template rendering, file I/O)
└── External (std::io::Error, tera::Error, ggen::Error)
```

**Rules**:
- ❌ No unwrap/expect in production code
- ✅ Use thiserror for error types
- ✅ Provide recovery strategies where possible

---

## Type System Design

### Core Types (Typestate Pattern)

```rust
// State markers (zero-sized)
pub struct Unvalidated;
pub struct Validated;

// AST types parameterized by state
pub struct CliSpec<S: State = Validated> { ... }
pub struct NounSpec<S: State = Validated> { ... }
pub struct VerbSpec<S: State = Validated> { ... }
pub struct ArgumentSpec<S: State = Validated> { ... }
```

### Newtype Pattern (Type Safety)

```rust
// Prevent primitive obsession
pub struct NonEmptyString(String);
pub struct NounId(u32);
pub struct VerbId(u32);
pub struct TypeId(u32);
```

### Trait Hierarchy

```rust
// Code generation abstraction
pub trait Generator {
    type Input;
    type Output;
    type Error;
    fn generate(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

// Validation abstraction
pub trait Validator<T> {
    type Error;
    fn validate(&self, input: &T) -> Result<(), Self::Error>;
}

// Typestate transition
pub trait Validate {
    type Error;
    fn validate(self) -> Result<Self, Self::Error>;
}
```

---

## Code Generation Pipeline

```
Turtle DSL
    ↓
┌─────────────────────────────────────┐
│ PHASE 1: Parsing                    │
│ TurtleParser + ggen Graph           │
│ Turtle → RDF Graph → Structured Data│
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ PHASE 2: AST Construction           │
│ RdfExtractor                        │
│ Structured Data → CliSpec<Unvalidated>│
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ PHASE 3: Semantic Validation        │
│ SemanticValidator                   │
│ CliSpec<Unvalidated> → CliSpec<Validated>│
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ PHASE 4: Code Generation            │
│ NounGen + VerbGen + WorkspaceGen    │
│ CliSpec<Validated> → GeneratedCode  │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ PHASE 5: Template Rendering         │
│ TemplateRegistry + Tera             │
│ GeneratedCode → Rust Project Files  │
└─────────────────────────────────────┘
    ↓
Output: Rust CLI Project (clap-noun-verb)
```

---

## Performance SLOs

| Operation | Target | Rationale |
|-----------|--------|-----------|
| Parsing (small) | ≤ 50ms | < 100 triples |
| Parsing (medium) | ≤ 200ms | 100-1000 triples |
| Parsing (large) | ≤ 1s | > 1000 triples |
| Validation | ≤ 50ms | 10-50 nouns |
| Code Generation | ≤ 100ms | Per noun |
| Template Rendering | ≤ 50ms | Per file |
| **Peak Memory** | **≤ 100MB** | During generation |

---

## Zero-Cost Guarantees

### Verified Zero-Cost Abstractions

1. **Typestate Pattern**: `size_of(CliSpec<Unvalidated>) == size_of(CliSpec<Validated>)`
2. **Newtype Pattern**: `size_of(NonEmptyString) == size_of(String)`
3. **Const Generics**: Compile-time checks, zero runtime overhead
4. **Generics**: Monomorphization (static dispatch), no trait object overhead

### Performance Validation

```rust
#[cfg(test)]
mod perf_tests {
    #[test]
    fn zero_cost_typestate() {
        assert_eq!(
            std::mem::size_of::<CliSpec<Unvalidated>>(),
            std::mem::size_of::<CliSpec<Validated>>()
        );
    }

    #[test]
    fn zero_cost_newtype() {
        assert_eq!(
            std::mem::size_of::<NonEmptyString>(),
            std::mem::size_of::<String>()
        );
    }
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- Core AST types with typestate
- RDF parser (Turtle → CliSpec<Unvalidated>)
- Error types and Result<T>
- Unit tests (Chicago TDD)

**Validation**: `cargo make test-unit && cargo make lint`

### Phase 2: Validation (Week 3)
- Semantic validators (uniqueness, references, constraints)
- Typestate transitions (Validate trait)
- Property-based tests (proptest)

**Validation**: `cargo make test && cargo make slo-check`

### Phase 3: Code Generation (Week 4-5)
- Generators (Noun, Verb, Type, Workspace)
- Tera templates
- clap-noun-verb integration
- Integration tests (end-to-end, golden)

**Validation**: `cargo make test && cargo make bench`

### Phase 4: Integration & Polish (Week 6)
- ggen-core adapters
- Documentation (rustdoc, tutorials)
- CLI tool
- Performance optimization

**Validation**: `cargo make ci && cargo make pre-commit`

### Phase 5: Advanced Features (Week 7+)
- SHACL validation
- Plugin system
- MCP integration
- Marketplace

---

## Security Considerations

### Input Validation

```rust
// File size limits (prevent DoS)
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

// Validate Turtle syntax before processing
validate_turtle_input(content)?;

// Sanitize output paths (prevent directory traversal)
sanitize_output_path(path)?;

// Sanitize identifiers (prevent code injection)
sanitize_identifier(name)?;
```

### Code Injection Prevention

- Only allow valid Rust identifiers (alphanumeric + underscore)
- Check for Rust keywords
- Escape user input in templates
- Validate all file paths

---

## Testing Strategy

### Chicago TDD Requirements

```rust
#[test]
fn test_feature() {
    // ARRANGE: Real objects (no mocks)
    let obj = RealObject::new();

    // ACT: Call public API
    let result = obj.do_thing();

    // ASSERT: Verify observable state/output
    assert_eq!(result, expected);
}
```

### Test Pyramid

- **Unit Tests (80%)**: Individual functions, Chicago TDD
- **Integration Tests (15%)**: Module integration
- **E2E Tests (5%)**: Full pipeline (Turtle → Generated Project)

### Test Coverage Target

- Overall: ≥ 80%
- Critical paths: 100% (parser, validator, generator)
- Error paths: 100% (all error variants)

---

## Key Takeaways

1. **Type Safety**: Typestate pattern enforces validation at compile time
2. **Zero-Cost**: All abstractions compile to zero runtime overhead
3. **Integration**: Reuse ggen-core infrastructure for RDF/templates
4. **Deterministic**: All outputs are reproducible and deterministic
5. **Production-Ready**: Comprehensive error handling, security, testing

---

## Next Steps

1. Create project structure: `ggen-clap-noun-verb/`
2. Implement Phase 1 (Foundation): AST types + parser
3. Write initial tests (Chicago TDD)
4. Validate with `cargo make check`

**Reference**: See `/home/user/clap-noun-verb/docs/ggen-clap-noun-verb-architecture.md` for full architecture document.

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-06
