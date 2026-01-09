# System Architect Brief - v6.0.0 Release

**Agent ID**: system-architect-v6
**Memory Key**: v6_architecture
**Priority**: CRITICAL - Foundation for all other agents
**Timeline**: Complete within 30 minutes

## Mission
Design the v6.0.0 API architecture with type-first thinking, encoding invariants at compile-time, and identifying zero-cost abstractions for major feature improvements.

## Current State Analysis (v5.5.0)
- Noun-verb CLI pattern builder on top of clap
- Agent-grade CLI generation with runtime features
- Kernel capabilities for deterministic CLIs
- Version: 5.5.0

## v6.0.0 Architectural Goals

### 1. Type-First API Design
**Question**: What invariants can be encoded at the type level?

Analyze current `clap-noun-verb`:
- Command builder ergonomics - can we improve with types?
- Argument parsing - where do runtime checks exist that should be compile-time?
- Noun-verb composition - can we enforce at type-level?

**Deliverable**: Type-level invariant design document

### 2. Breaking Changes That Enable Better Design
Identify 3-5 MAJOR breaking changes that:
- Improve type safety
- Reduce runtime panics/errors
- Enable better zero-cost abstractions
- Simplify public API surface

Examples (research in codebase):
- Change return types from `Result<T, String>` to proper error types
- Restructure builder pattern for better type inference
- Modify command composition API for type safety

**Deliverable**: Breaking Change ADRs (Architecture Decision Records)

### 3. Zero-Cost Abstraction Analysis
Review current code (`src/` directory):
- Which abstractions have runtime overhead?
- Where can const generics help?
- Can we use monomorphization for zero-cost?
- Identify optimization opportunities

**Deliverable**: Zero-Cost Abstraction Opportunities document

### 4. New Feature Architectures
For each planned v6.0.0 feature:
- How does it integrate with existing design?
- Type-level guarantees it should provide?
- Performance characteristics?

**Deliverable**: Feature architecture designs

## Work Steps

1. **Analyze Current Code** (5 min)
   - Review `/home/user/clap-noun-verb/src/` structure
   - Understand current API patterns
   - Identify pain points from issues/PRs

2. **Design Type-First Improvements** (10 min)
   - Sketch type-level invariant encodings
   - Design new type-safe APIs
   - Identify breaking changes justified by design

3. **Document ADRs** (8 min)
   - Create Architecture Decision Records
   - Justify each breaking change
   - Show before/after API comparison

4. **Store in Memory** (2 min)
   - Save findings to v6_architecture memory key
   - Include Type diagrams, code sketches
   - Summary for other agents

## Constraints & Rules

- **Type Safety First**: Every decision must improve type safety
- **Zero-Cost**: All abstractions must be zero-cost (generics, const generics)
- **Ergonomics**: APIs must be ergonomic (easy correct, hard to misuse)
- **Breaking Changes Justified**: Each breaking change must have clear benefit
- **Backward Compatibility**: Document migration paths needed (for Code Analyzer)

## Success Criteria

- ✅ 3-5 MAJOR breaking changes identified with justification
- ✅ Type-level invariant design documented
- ✅ Zero-cost abstraction opportunities identified
- ✅ New feature architectures designed
- ✅ Memory key v6_architecture populated with findings
- ✅ Specification Agent can use architecture as foundation

## Dependencies & Downstream
- **Blocks**: Specification Agent (waits for v6_architecture)
- **Blocks**: Code Analyzer (needs architecture for breaking changes)
- **Used by**: Test Engineer, Performance Benchmarker

## Notes
- Focus on TYPE DESIGN, not implementation
- Sketch pseudocode if helpful, but no actual coding yet
- Document trade-offs clearly
- Be bold - v6.0.0 allows breaking changes for better design
