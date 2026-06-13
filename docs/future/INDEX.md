# Future Specifications Index: clap-noun-verb v6.0 Roadmap

**Status:** Proposed for v6.0  
**Timeline:** Phase 1–3, 2026-06-01 through 2026-07-11 (5 weeks)  
**Last Updated:** 2026-06-01

---

## Overview

This index organizes the 10 frontier feature specifications that define the architectural vision for **clap-noun-verb v6.0**. Each spec has been evaluated for scope, implementation complexity, and dependencies. The proposed timeline spans three phases:

| Phase | Dates | Focus |
|-------|-------|-------|
| **Phase 1** | 2026-06-01 – 2026-06-15 | Cleanups: Remove deprecated macros/APIs; prepare semantic foundation |
| **Phase 2** | 2026-06-16 – 2026-06-29 | Core Infrastructure: Config engine unification; RDF/SPARQL integration |
| **Phase 3** | 2026-06-30 – 2026-07-11 | Advanced Features: Autonomic loops; MCP integration; SHACL validation |

---

## Feature Areas & Specifications

### 1. **Core Framework Evolution**

#### [v6_breaking_plan.md](./v6_breaking_plan.md)
**Status:** Proposed for v6.0 (Phase 1–2)  
**Summary:** Removal of deprecated `#[noun]` macro, `arg_names()` API, and strict `#[arg]` whitelist enforcement. Introduction of unified configuration engine with environment variable standardization and strict schema validation.  
**Key Deliverables:**
- Remove deprecated `#[noun]` attribute macro entirely
- Replace `VerbArgs::arg_names()` with zero-allocation `arg_names_refs()`
- Standardize environment variable prefixing (default: `CNV_`)
- Implement strict schema validation for config files
- Native MCP Tool Schema generation
- Compile-time and runtime SHACL validation
- Second-order autonomic tuning via Meta-MAPE-K loop

**Dependencies:** None (foundational)

---

### 2. **Semantic & Knowledge Graph Integration**

#### [ontology_design_guide.md](./ontology_design_guide.md)
**Status:** Proposed for v6.0 (Phase 2)  
**Summary:** Formal specification for representing CLI commands, arguments, and safety constraints as RDF/OWL ontology triples. Enables intent-based discovery, decoupled policy enforcement via SHACL shapes, and agentic introspection.  
**Key Deliverables:**
- Define `cnv` namespace classes: `Noun`, `Verb`, `Command`, `Argument`, `ReturnType`, `Capability`
- Integrate external ontologies (SWO, PROV-O, OWL-Time, ODRL, ORG)
- Separate epistemic vs. kinetic state representations
- SHACL integrity shapes for compile-time validation
- Hierarchical command taxonomy rules with strict two-level depth
- Intent mapping via descriptive verb tags

**Dependencies:** Phase 1 cleanup; must precede RDF integration

---

#### [semantic_cli_patterns.md](./semantic_cli_patterns.md)
**Status:** Proposed for v6.0 (Phase 2)  
**Summary:** Comprehensive guide for transitioning CLIs from syntactic string parsing to semantic knowledge graphs. Models CLI structures as RDF triples queryable via SPARQL. Defines compile-time macro pipeline and runtime control plane.  
**Key Deliverables:**
- CLI-as-a-Knowledge-Graph conceptual framework
- `clap:CliApplication`, `clap:Noun`, `clap:Verb`, `clap:Argument`, `clap:Execution` class definitions
- Oxigraph in-memory RDF store with SPARQL 1.1 engine
- SHACL shapes for input validation
- JSON-LD and Model Context Protocol (MCP) export
- Intent-based discovery SPARQL patterns
- Automated error-correction suggestion queries
- Compile-time Turtle generation from `#[verb]` macros

**Dependencies:** Ontology Design Guide; Phase 2

---

#### [rdf_sparql_integration.md](./rdf_sparql_integration.md)
**Status:** Proposed for v6.0 (Phase 2–3)  
**Summary:** Detailed architecture for embedding RDF and SPARQL capabilities. Describes compile-time triple generation, Oxigraph-backed semantic engine, and query endpoints for autonomic controllers.  
**Key Deliverables:**
- `cnv` ontology namespace with property mappings
- Turtle representation of commands with SHACL validation shapes
- `linkme` distributed slice for static RDF registration
- `SemanticEngine` structure with thread-safe caching
- 5-second timeout guard rails for SPARQL queries
- Intent resolution, side-effect overlap detection, CLI interface verification queries
- Roadmap for v5.5–v5.7 extensions (SPARQL HTTP endpoint, MCP integration, swarm orchestration)

**Dependencies:** Semantic CLI Patterns; Ontology Design Guide

---

### 3. **Safety, Execution, & Autonomic Control**

#### [effect_metadata.md](./effect_metadata.md)
**Status:** Proposed for v6.0 (Phase 2)  
**Summary:** Declarative system for annotating side effects of CLI commands. Enables compile-time validation, runtime safety boundaries, and command graph conflict detection.  
**Key Deliverables:**
- `EffectType` enum: `ReadOnly`, `MutateState`, `MutateConfig`, `MutateOntology`, `MutateSecurity`
- `Sensitivity` enum: `Low`, `Medium`, `High`, `Critical`
- `ResourceTarget` for identifying specific system resources
- Programmatic `EffectMetadata` builder
- Macro-based declarative annotation support
- Compile-time validation rules (resource consistency, missing metadata, safety requirements)
- `SafetyError` enum and `RuntimeSafetyGate` for execution enforcement
- `ExecutionPlanner` for command graph conflict detection
- `--sandbox` and `--read-only` flag support

**Dependencies:** Phase 1; precedes Autonomic CLI integration

---

#### [autonomic_cli.md](./autonomic_cli.md)
**Status:** Proposed for v6.0 (Phase 3)  
**Summary:** Complete specification for self-healing, diagnostic-enabled autonomic CLI layer. Defines machine-grade interfaces for autonomous agents via MAPE-K loop closure.  
**Key Deliverables:**
- O-Σ-Q-ΔΣ Framework (Observations, Ontology, Invariants, Overlays planes)
- Three execution flags: `--autonomic`, `--enforce-guards`, `--receipt-only`
- Three recovery actions: Command Spellcheck, Dynamic Timeout Adaptation, Budget Remediation
- `StructuredError` JSON schema with `action_templates`
- MAPE-K loop closure (Monitor–Analyze–Plan–Execute–Knowledge)
- Self-healing Rust patterns for orchestrator agents
- Concrete diagnostic report examples (spellcheck, latency guard breach)

**Dependencies:** Effect Metadata; Introspection Reference; Phase 2 completion

---

#### [introspection_reference.md](./introspection_reference.md)
**Status:** Proposed for v6.0 (Phase 2–3)  
**Summary:** Formal reference specification for Introspection API. Defines CLI command maps, metadata schemas, and machine-readable output formats for autonomic controllers, orchestrators, MCP servers, and LLM agents.  
**Key Deliverables:**
- CLI command mapping: `--capabilities`, `--introspect`, `--introspect-noun`, `--introspect-domain`, `--graph`, `--receipt-only`
- Standard LLM Tool Definitions schema (OpenAI/Anthropic/MCP compatible)
- `IntrospectionResponse` and metadata structures
- Effect profiles and sensitivity levels
- Conceptual plane interactions (O/Σ/Q/ΔΣ)
- Guards and resource budgets
- Execution Receipts and Structured Error schemas
- SHACL Shapes for parameter validation
- Execution Contracts and isolation levels

**Dependencies:** Effect Metadata; Phase 2 preparation

---

### 4. **Data Flow & Serialization**

#### [custom_serialization.md](./custom_serialization.md)
**Status:** Proposed for v6.0 (Phase 2)  
**Summary:** Comprehensive reference for complex CLI argument parsing, JSON output formatting, and custom error formats. Bridges human-facing and machine-facing serialization needs.  
**Key Deliverables:**
- Range-bounded parsing with `clap-num` wrappers and `value_parser` mechanisms
- Custom string format parsers: `parse_percentage`, `parse_bytes`, `parse_duration`
- Flattening and normalization (dot notation, environment variables)
- `OutputFormat` enum: `Json`, `JsonPretty`, `Yaml`, `Table`, `Plain`, `Tsv`, `Quiet`
- Output validation hooks for security compliance
- Deep serialization bounds mitigation
- `NounVerbError` vs `StructuredError` distinction
- Error mapping matrix for autonomic loop integration
- JSON representation examples

**Dependencies:** Phase 1; foundational for introspection

---

### 5. **Observability & Performance**

#### [performance_profiling.md](./performance_profiling.md)
**Status:** Proposed for v6.0 (Phase 1)  
**Summary:** Methodologies and tools for profiling and optimizing CLI applications. Focuses on sub-10ms startup latency, memory footprint control, and allocation minimization.  
**Key Deliverables:**
- Architecture profile of `linkme` compile-time registration and metadata leakage via `Box::leak`
- Deferred async runtime initialization patterns
- Lazy configuration parsing
- Stripping and link-time optimization configuration
- Memory allocator selection (mimalloc vs jemalloc)
- Static string reuse strategies
- Profiling tools: `hyperfine` (startup), `samply` (CPU), `dhat` (allocation), `tracing-chrome` (trace analysis)
- Performance checklist for new verbs

**Dependencies:** None (tooling-focused)

---

#### [testing_strategies.md](./testing_strategies.md)
**Status:** Proposed for v6.0 (Phase 1–2)  
**Summary:** Advanced testing strategies for production-ready CLI systems. Covers reflexive testing, interactive prompt testing, environment mocking, and compliance with Chicago TDD.  
**Key Deliverables:**
- Reflexive testing (self-testing systems) with `TestCase<T>`, `CoverageMask`, `RegressionBaseline`
- `SemanticTestGenerator` for Turtle ontology-driven test generation
- Interactive session testing with mock readers/writers
- Partial streaming inputs and interruption simulation
- Thread-safe environment variable mocking
- Dependency injection pattern for env settings
- Serialized execution with test mutexes
- File system mocking with `tempfile`
- Terminal/TTY capability mocking
- Best practices checklist

**Dependencies:** Phase 1; preparation for Phase 2 integration

---

---

## Timeline Gantt Chart

```
Phase 1: Cleanups & Foundation (2026-06-01 to 2026-06-15)
├── Remove deprecated #[noun] macro
├── Remove arg_names() API
├── Enforce #[arg] whitelist
├── Performance profiling tools & baseline
└── Testing strategies framework
    ↓
Phase 2: Semantic Foundation & Config (2026-06-16 to 2026-06-29)
├── Ontology design guide implementation
├── Semantic CLI patterns (Oxigraph/SPARQL)
├── RDF/SPARQL integration engine
├── Effect metadata system
├── Unified configuration engine
├── Introspection API formalization
├── Custom serialization framework
└── Documentation & examples
    ↓
Phase 3: Autonomic & Advanced Features (2026-06-30 to 2026-07-11)
├── Autonomic CLI layer (MAPE-K loops)
├── Self-healing diagnostics
├── Native MCP integration
├── Compile-time SHACL validation
├── Second-order autonomic tuning
├── Integration tests
└── Final release preparation
```

---

## Dependency Graph

```
┌─────────────────────────────────────────────────────────┐
│ Phase 1: Cleanups (Non-Blocking)                        │
│ • v6_breaking_plan (cleanup section)                    │
│ • performance_profiling                                 │
│ • testing_strategies                                    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│ Phase 2: Semantic & Config Infrastructure               │
│ • ontology_design_guide (foundation)                    │
│ • semantic_cli_patterns (Oxigraph engine)               │
│ • rdf_sparql_integration (query endpoints)              │
│ • effect_metadata (safety framework)                    │
│ • introspection_reference (API spec)                    │
│ • custom_serialization (data format)                    │
│ • v6_breaking_plan (config section)                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│ Phase 3: Autonomic & Advanced                           │
│ • autonomic_cli (MAPE-K loops)                          │
│ • MCP schema generation                                 │
│ • SHACL validation hooks                                │
│ • Second-order tuning (meta-MAPE-K)                     │
└─────────────────────────────────────────────────────────┘
```

---

## Specification Document Purposes

| Spec | Purpose | Audience |
|------|---------|----------|
| **v6_breaking_plan.md** | Migration guide + feature proposals | Maintainers, contributors |
| **ontology_design_guide.md** | Formal vocabulary + semantics | RDF/knowledge-graph specialists |
| **semantic_cli_patterns.md** | Architectural patterns + integration | Framework architects, LLM integrators |
| **rdf_sparql_integration.md** | Implementation details + query patterns | Backend engineers, knowledge engineers |
| **effect_metadata.md** | Safety specification + compile-time validation | Safety-critical CLI designers |
| **autonomic_cli.md** | MAPE-K specifications + diagnostic schemas | Autonomous agent orchestrators |
| **introspection_reference.md** | API contracts + metadata schemas | Tool integration specialists, MCP servers |
| **custom_serialization.md** | Data format patterns + validation hooks | Serialization & I/O specialists |
| **performance_profiling.md** | Optimization methodologies + tooling | Performance engineers |
| **testing_strategies.md** | Test design patterns + mocking strategies | QA engineers, test architects |

---

## Key Integration Points

### Compile-Time (Macro Expansion)
- `#[verb]` macro expands to: Rust code + Turtle RDF strings + SHACL shapes → distributed `linkme` slices
- `#[arg]` macro validates against strict whitelist; emits effect metadata

### Runtime Initialization
- Semantic engine loads embedded RDF/SHACL from distributed slices
- Effect metadata populates safety registry
- Configuration engine reads and merges from 5 sources with strict schema validation

### Execution
- Introspection API exposed via `--capabilities`, `--introspect`, `--graph` flags
- Effect metadata guards enforce `--sandbox`, `--read-only`, confirmation gates
- Autonomic flags (`--autonomic`, `--enforce-guards`) trigger MAPE-K loop integration

### Observability
- Structured error output for machine consumption
- Execution receipts with guard enforcement telemetry
- Event logs for second-order autonomic tuning

---

## Success Metrics (v6.0 Completion)

- [ ] All 10 specs implemented and integrated
- [ ] Backwards-incompatible changes documented; migration guide published
- [ ] Test coverage >85% for autonomic layer
- [ ] Startup latency <10ms (micro-benchmark validated)
- [ ] SPARQL query latency <100ms (P95, 1000-triple graph)
- [ ] MCP schema generation produces OpenAI/Anthropic-compatible tool definitions
- [ ] SHACL validation gates block 100% of structurally invalid inputs
- [ ] MAPE-K loop closure demonstrated with self-healing example

---

## References

- **RDF/SPARQL Standards:** W3C SPARQL 1.1, RDF 1.1 Semantics
- **Knowledge Graphs:** OWL 2, SHACL (W3C Shape Constraint Language)
- **Agent Integration:** Model Context Protocol (MCP) v1.0
- **CLI Design:** POSIX.1-2017, clap v4.x API
- **Testing Philosophy:** Chicago TDD, property-based testing (proptest)
- **Process Mining:** van der Aalst's event log analysis methodology
