# clap-noun-verb v6.0.0 - Specification Summary

**Version**: 6.0.0
**Status**: SPARC Specification Phase - Executive Summary
**Date**: 2026-01-08
**Methodology**: SPARC + Chicago TDD + Toyota Production System (TPS)

---

## Strategic Vision

**Primary Goal**: Create the most type-safe, deterministic, and extensible noun-verb CLI framework for agent-grade applications.

**Three Core Pillars**:
1. **Type-Safety-First**: Compile-time guarantees for capabilities, constraints, and validity
2. **Deterministic Execution**: Unforgeable kernel receipts proving execution happened
3. **Unified Frontier Integration**: 10 frontier features with stable, composable APIs

---

## Key Changes from v5.5.0

### Architectural Changes
| Change | Impact | Rationale |
|--------|--------|-----------|
| Handler trait: `Handler<A>` → `Handler<A, E, O>` | HIGH | Separate error from output for better Result semantics |
| Phantom type parameters added | MEDIUM | Encode capability constraints at compile-time |
| Feature restructuring (3-tier system) | MEDIUM | Clear stability expectations (Stable/Experimental/Frontier) |
| Default features: enabled → empty | LOW | Faster compilation, explicit opt-in |
| Unwrap/expect elimination | LOW | Production-safe error handling |

### Problem Statement

**Problem 1**: v5.5.0 had runtime capability checks → v6.0.0 compile-time type constraints
**Problem 2**: Unclear feature maturity levels → v6.0.0 explicit tiers
**Problem 3**: Associated types hard to work with in bounds → v6.0.0 generic parameters
**Problem 4**: Incomplete error handling in places → v6.0.0 100% Result<T, E>
**Problem 5**: No determinism guarantees → v6.0.0 kernel receipts + policy traces

---

## Functional Requirements Matrix

### Tier 1: Core (CRITICAL - Must Have)

| ID | Requirement | Success Metric |
|----|-------------|----------------|
| FR-C.1 | Noun-verb command parsing with macro support | 100% of test cases pass |
| FR-C.2 | Type-safe handler registration | Compiler errors prevent invalid handlers |
| FR-C.3 | Ergonomic API for CLI creation | Common case <10 lines |
| FR-T.1 | Capability trait bounds | Impossible to violate capability constraints |
| FR-T.2 | Delegation chain validation | No circular delegations possible |
| FR-D.1 | Kernel receipts for every execution | 100% of executions produce valid receipts |
| FR-D.2 | Policy evaluation traces | All policy decisions traceable |

### Tier 2: Frontier Features (EXPERIMENTAL)

| ID | Feature | Target Stable | Status |
|----|---------|----------------|--------|
| FR-F.1 | Meta-framework (typetag dispatch) | v6.2 | Type-erased reflection |
| FR-F.2 | RDF composition (Turtle → Rust) | v6.3+ | Semantic CLI generation |
| FR-F.3 | Fractal patterns (self-similar hierarchies) | v6.2 | Const-generic depth limits |
| FR-F.4 | Discovery engine (dynamic capabilities) | v6.2 | Microservices support |
| FR-F.5 | Federated network (multi-host) | v6.3+ | Byzantine fault tolerance |

---

## Non-Functional Requirements

### Performance SLOs

| Metric | Target | Acceptance | Measurement |
|--------|--------|-----------|-------------|
| Incremental compile | ≤ 2s | ≤ 3s | `cargo make check` |
| CLI execution (p95) | ≤ 100ms | ≤ 150ms | Benchmark suite |
| Receipt generation | ≤ 10ms | ≤ 20ms | Crypto benchmarks |
| Memory peak | ≤ 10MB | ≤ 15MB | RSS under load |
| Test coverage (critical) | ≥ 85% | ≥ 80% | `cargo tarpaulin` |

### Security Requirements

- Zero unsafe code (except trait object dispatch)
- BLAKE3 hashing, Ed25519 signatures, timing-safe comparisons
- Policy enforcement with audit trails
- Type-safe capability constraints

### Reliability

- 99.9% uptime SLA
- Chicago TDD compliance (state-based, AAA pattern, ≥85% coverage)
- No panics in production code (0 unwrap/expect)
- Deterministic policy evaluation

---

## Type System Architecture

### Handler Trait (Core API)

```rust
pub trait Handler<A: Args, E: Error, O: Output>: Send + Sync {
    fn execute(&self, args: A) -> Result<O, E>;
}
```

**Benefits**:
- Generic parameters enable trait bounds: `T: Handler<A, E, O>`
- Type system enforces error handling
- Output type separate from error for better semantics
- Composition: `impl<H> Feature for H where H: Handler<...>`

### Capability System (Type-Safe)

```rust
pub trait CapabilityBound<C: Capability>: Sized {
    type Output;
    fn execute(self, cap: C) -> Result<Self::Output, CliError>;
}

// Usage: Handler requires AdminCap
impl Handler<Args, CliError, Output> for MyHandler
where
    Self: CapabilityBound<AdminCap>,
{ ... }
```

**Benefits**:
- Impossible to create handler without required capabilities
- Zero-cost abstraction (compile-time only)
- Compiler prevents capability violations

### Resource Budgets (Const Generics)

```rust
pub struct BudgetedExecution<const MAX_MEMORY: usize, const MAX_LATENCY_MS: u64> {
    // Type parameters prevent overspending
}
```

**Benefits**:
- Stack overflow impossible (depth bounded)
- OOM prevented (allocation size known at compile-time)
- No runtime overhead (zero-cost)

---

## Breaking Changes Summary

### Top 5 Changes (Must Handle)

1. **Handler Signature** (HIGH): `impl Handler<A>` → `impl Handler<A, E, O>`
   - All custom handlers need update
   - Separate error from output type
   - Effort: 30 min per handler type

2. **Error Type Consolidation** (MEDIUM): Multiple error types → `CliError`
   - Use unified enum for all CLI errors
   - Better pattern matching and serialization
   - Effort: Depends on error handling scope

3. **Feature Flags** (LOW): Update Cargo.toml with explicit features
   - `default = ["autonomic"]` → `features = ["autonomic"]`
   - Add frontier features explicitly
   - Effort: 5-10 minutes

4. **Capability Constraints** (MEDIUM): Runtime checks → Type-level bounds
   - Add trait bounds where capabilities needed
   - Remove `if !has_capability()` checks
   - Effort: 1 hour for complex handlers

5. **Default Features** (LOW): Were enabled, now explicitly required
   - Add `autonomic`, `crypto` to features list if needed
   - Eliminates unexpected compile-time impact
   - Effort: 5 minutes

### Deprecated APIs Removed

| Item | Removed | Migration |
|------|---------|-----------|
| `Handler::Output` associated type | Yes | Use `O` generic parameter |
| `Handler::Error` associated type | Yes | Use `E` generic parameter |
| `legacy_handler_registry()` | Yes | Use `register_handler()` |
| `unsafe_execute()` | Yes | Use `execute()` |

---

## Feature Tier Stability

### Tier 1: Stable (Guaranteed Support)
- ✅ Available by default or via simple flag
- ✅ Backwards compatible for ≥2 major versions
- ✅ Full test coverage (≥90%)
- ✅ Production-ready with SLA
- Examples: `noun`, `verb`, core CLI framework

### Tier 2: Experimental (Provisional)
- ⚠️ Available via feature flag
- ⚠️ May have breaking changes next minor version
- ⚠️ Sufficient test coverage (≥70%)
- ⚠️ Documented limitations
- Examples: `learning-trajectories`, `reflexive-testing`

### Tier 3: Frontier (Research Grade)
- 🔬 Available via feature flag
- 🔬 Breaking changes expected
- 🔬 Limited test coverage
- 🔬 Explicitly marked unstable
- Examples: `federated-network`, `quantum-ready`

**Feature Status Table**:
| Feature | Tier | Target Stable | Path |
|---------|------|----------------|------|
| meta-framework | Frontier | v6.2 | Stabilizing |
| rdf-composition | Frontier | v6.3+ | Research phase |
| executable-specs | Experimental | v6.1 | Ready to stabilize |
| fractal-patterns | Frontier | v6.2 | Type-level work |
| discovery-engine | Frontier | v6.2 | Service discovery |
| federated-network | Frontier | v6.3+ | Byzantine consensus |
| learning-trajectories | Experimental | v6.1 | ML integration |
| reflexive-testing | Experimental | v6.1 | Property testing |
| economic-sim | Frontier | v6.3+ | ECS simulation |
| quantum-ready | Frontier | v7.0+ | NIST standards |

---

## TPS Standardization (Operational Excellence)

### Naming Conventions

| Category | Pattern | Examples | Rationale |
|----------|---------|----------|-----------|
| Traits | `{Concept}` | `Handler`, `Capability`, `Registry` | Business logic |
| Type Params | A, E, O, T, C | Args, Error, Output, Type, Capability | Standard abbreviations |
| Associated Types | `type {Concept}` | `type Output`, `type Error` | Clear intent |
| Const Generics | `const {QUAL}_{METRIC}` | `const MAX_DEPTH`, `const MAX_MEMORY` | Compile-time constants |
| Error Types | `{Context}Error` | `ParseError`, `CapabilityError` | Namespacing |
| Macros | `#[{action}]` | `#[noun]`, `#[verb]` | Compile-time transformation |

### Error Handling Standard

**Rule 1**: `Result<T, E>` everywhere
```rust
✅ pub fn do_something() -> Result<Output, MyError> { ... }
❌ pub fn do_something() -> Option<Output> { ... }
❌ pub fn do_something() -> Output { ... }  // Panics!
```

**Rule 2**: Descriptive error messages with context
```rust
✅ Err(CliError::CapabilityMissing("ReadCap required".into()))
❌ Err(CliError::CapabilityMissing("cap error".into()))
```

**Rule 3**: Use `?` operator for propagation
```rust
✅ let value = parse()?.validate()?;
❌ let value = match parse() { Ok(v) => ..., Err(e) => return Err(e) };
```

### Type-First Design Patterns

**Pattern 1**: Make invalid states unrepresentable
```rust
// ✅ GOOD: Type ensures validity
pub struct ValidatedArgs { value: String }  // Constructor checks non-empty

// ❌ BAD: Runtime validation needed
pub struct Args { value: String }  // Need is_valid() check
```

**Pattern 2**: Phantom types for zero-cost encoding
```rust
// ✅ GOOD: Phantom types encode properties
pub struct Token<const VALID: bool> { data: String }

// ❌ BAD: Runtime checks instead
pub struct Token { data: String, validated: bool }
```

---

## Acceptance Criteria

### AC-GROUP-1: Type Safety (All CRITICAL)
- [ ] AC-1.1: Invalid handler signature fails to compile
- [ ] AC-1.2: Capability violation caught at compile-time
- [ ] AC-1.3: Delegation chains prevent cycles
- [ ] AC-1.4: Resource budgets enforced by types

### AC-GROUP-2: Determinism (All HIGH)
- [ ] AC-2.1: Every execution generates receipt with signature
- [ ] AC-2.2: Receipts are immutable (write-once)
- [ ] AC-2.3: Policy decisions traced completely
- [ ] AC-2.4: Policy evaluation is deterministic

### AC-GROUP-3: Error Handling (All CRITICAL)
- [ ] AC-3.1: Zero unwrap/expect in library code
- [ ] AC-3.2: All errors return Result<T, E>
- [ ] AC-3.3: Error messages are helpful (explain + solution)

### AC-GROUP-4: Frontier Integration (All EXPERIMENTAL)
- [ ] AC-4.1: Meta-framework type dispatch works
- [ ] AC-4.2: RDF composition round-trips perfectly
- [ ] AC-4.3: Fractal patterns generate valid commands
- [ ] AC-4.4: Federated network handles partitions

---

## Success Metrics

### Quantitative

| Metric | Target | Acceptance | Measurement |
|--------|--------|-----------|-------------|
| Incremental compile | ≤ 2s | ≤ 3s | `cargo make check` |
| CLI execution (p95) | ≤ 100ms | ≤ 150ms | Benchmark suite |
| Test coverage (critical) | ≥ 85% | ≥ 80% | `cargo tarpaulin` |
| Clippy warnings | 0 | 0 | `cargo make lint` |
| Panics in lib | 0 | 0 | Grep + clippy |

### Qualitative

- ✅ Type safety: Compile-time guarantee for capability constraints
- ✅ Determinism: Same inputs → same policy output
- ✅ Ergonomics: Simple case <10 lines
- ✅ Documentation: Every public API documented + examples
- ✅ Maintainability: New contributors understand within 1 day

---

## Definition of Done

A feature is DONE when ALL of these pass:

### Code Quality
- ✅ `cargo make check` - Zero compiler errors
- ✅ `cargo make lint` - Zero clippy warnings
- ✅ `cargo make test` - 100% tests passing
- ✅ Coverage - Critical paths ≥85%

### Quality Assurance
- ✅ Chicago TDD - AAA pattern + behavior verification
- ✅ Error paths - All tested explicitly
- ✅ Edge cases - Documented and tested

### Documentation
- ✅ Public APIs - 100% documented with examples
- ✅ Trait bounds - Documented with use cases
- ✅ CHANGELOG - Entry with breaking changes note

### Production Readiness
- ✅ No panics - `--deny=panic` passes
- ✅ No unwrap - `--deny=unwrap_used` passes
- ✅ No expect - `--deny=expect_used` passes
- ✅ Performance - <2% regression vs baseline

---

## Implementation Roadmap

### Phase 1: Architecture & Pseudocode (Week 1)
- [ ] SPARC Pseudocode phase
- [ ] Type-level design documentation
- [ ] Trait hierarchy diagrams

### Phase 2: Core Implementation (Weeks 2-3)
- [ ] Handler trait refactoring
- [ ] Capability type system
- [ ] Kernel receipts implementation

### Phase 3: Frontier Integration (Weeks 4-5)
- [ ] Meta-framework integration
- [ ] RDF composition layer
- [ ] Fractal pattern generation

### Phase 4: Testing & Validation (Week 6)
- [ ] Chicago TDD test suite
- [ ] Performance benchmarks
- [ ] Security audit

### Phase 5: Documentation & Release (Week 7)
- [ ] Complete API documentation
- [ ] Migration guide refinement
- [ ] Release candidate testing

---

## Critical Success Factors

1. **Type-Level Design**: All capability constraints encoded in types
2. **Zero-Cost Abstractions**: Phantom types, const generics, monomorphization
3. **Determinism**: Every execution produces verifiable receipts
4. **Frontier Stability**: Clear maturity expectations (3-tier system)
5. **Production Quality**: Chicago TDD + FMEA + SLO verification
6. **Clear Migration Path**: Detailed guide with examples for all breaking changes

---

## Key Decisions & Rationale

**Decision 1**: Handler<A, E, O> instead of associated types
- Rationale: Generic parameters compose better with trait bounds
- Enables: Type-safe handler factories and composition

**Decision 2**: Phantom type parameters for capabilities
- Rationale: Zero-cost compile-time constraints
- Benefit: Impossible to violate capability constraints

**Decision 3**: Empty default features
- Rationale: Explicit opt-in reduces surprise compile-time costs
- Benefit: Faster compilation for basic use cases

**Decision 4**: 3-tier feature stability system
- Rationale: Clear expectations for production use
- Benefit: Users know which features are safe for production

**Decision 5**: 100% Result<T, E> error handling
- Rationale: Production-safe semantics, no panics
- Benefit: Reliable execution in mission-critical applications

---

## Risks & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Migration complexity deters adoption | Medium | HIGH | Detailed migration guide + examples |
| Type-level design has compilation cost | Low | MEDIUM | Const generics, monomorphization |
| Frontier features delay stable release | Medium | MEDIUM | Clear timeline, phased rollout |
| Breaking changes reduce adoption | Medium | MEDIUM | Deprecation warnings 1 release prior |
| Security audit uncovers issues | Low | HIGH | Security audit in Phase 4 |

---

## Success Definition

**v6.0.0 is successful when**:

1. ✅ All Andon signals cleared (compiler, tests, warnings)
2. ✅ All acceptance criteria met
3. ✅ Performance SLOs achieved (p95 latency verified)
4. ✅ Chicago TDD compliance verified (≥85% coverage)
5. ✅ Zero known critical bugs
6. ✅ Migration guide enables upgrade in <1 day for most projects
7. ✅ Frontier features have clear upgrade path to stable
8. ✅ Security audit completed and passed
9. ✅ 2+ external projects successfully upgraded
10. ✅ Community feedback positive on new APIs

---

**Document Version**: 1.0 FINAL
**Status**: SPARC SPECIFICATION PHASE COMPLETE
**Next Phase**: SPARC Pseudocode Phase
**Date**: 2026-01-08
