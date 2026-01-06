# RDF Turtle CLI Architecture - Visual Diagrams

---

## System Architecture (C4 Model - Container Level)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         clap-noun-verb System                             │
│                                                                           │
│  ┌─────────────────────┐       ┌──────────────────────┐                 │
│  │  Turtle Parser      │──────▶│  TurtleOntology<S,N> │                 │
│  │  (oxigraph)         │       │  (Validated)         │                 │
│  └─────────────────────┘       └──────────┬───────────┘                 │
│           │                               │                              │
│           │ RdfTriple                     │ Command                      │
│           ▼                               ▼                              │
│  ┌─────────────────────┐       ┌──────────────────────┐                 │
│  │  StorageBackend     │       │  CliGenerator        │                 │
│  │  - MemoryStorage    │       │  - RustCliGenerator  │                 │
│  │  - DiskStorage      │       │  (const FEATURES)    │                 │
│  └─────────────────────┘       └──────────┬───────────┘                 │
│           │                               │                              │
│           │ SPARQL Query                  │ TokenStream                  │
│           ▼                               ▼                              │
│  ┌─────────────────────┐       ┌──────────────────────┐                 │
│  │  SparqlExecutor     │       │  Generated CLI Code  │                 │
│  │  (oxigraph)         │       │  (Rust source)       │                 │
│  └─────────────────────┘       └──────────────────────┘                 │
│                                                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                      MCP Tool Layer                                │  │
│  │                                                                    │  │
│  │  ┌──────────────────────┐  ┌──────────────────────┐              │  │
│  │  │ GenerateCliFromTurtle│  │ QueryCapabilities    │              │  │
│  │  └──────────────────────┘  └──────────────────────┘              │  │
│  │  ┌──────────────────────┐  ┌──────────────────────┐              │  │
│  │  │ ExportToTurtle       │  │ RdfMcpServer         │              │  │
│  │  └──────────────────────┘  └──────────────────────┘              │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow Diagram

```
┌──────────────┐
│  Turtle File │
│  (.ttl)      │
└──────┬───────┘
       │
       │ Read
       ▼
┌──────────────────────────────────────────────────────────────┐
│  TurtleParser::parse()                                        │
│  - Lexical analysis (namespace prefixes, IRIs)               │
│  - Syntax validation (Turtle grammar)                        │
│  - Triple extraction (subject, predicate, object)            │
└──────┬───────────────────────────────────────────────────────┘
       │
       │ Vec<RdfTriple>
       ▼
┌──────────────────────────────────────────────────────────────┐
│  TurtleOntology::<MemoryStorage, N>::new()                   │
│  - Store triples in oxigraph MemoryStore                     │
│  - Build command index (BTreeMap<String, Command>)           │
│  - Initialize SPARQL executor                                │
│  - State: Unvalidated                                        │
└──────┬───────────────────────────────────────────────────────┘
       │
       │ TurtleOntology<S, Unvalidated>
       ▼
┌──────────────────────────────────────────────────────────────┐
│  TurtleOntology::validate()                                  │
│  - Check required properties (cnv:name, cnv:hasVerb)         │
│  - Validate type consistency (Command, Noun, Verb)           │
│  - Detect circular dependencies                              │
│  - State transition: Unvalidated → Validated                 │
└──────┬───────────────────────────────────────────────────────┘
       │
       │ TurtleOntology<S, Validated>
       ▼
┌──────────────────────────────────────────────────────────────┐
│  RustCliGenerator::generate()                                │
│  - Extract commands via SPARQL                               │
│  - Generate struct definitions (nouns)                       │
│  - Generate function signatures (verbs)                      │
│  - Generate argument parsing (clap derives)                  │
│  - Apply feature flags (async, completions)                  │
└──────┬───────────────────────────────────────────────────────┘
       │
       │ TokenStream
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Generated Rust Code                                         │
│  - #[noun("services")] struct Services;                      │
│  - #[verb("status")] fn status() -> Result<()> { ... }      │
│  - Clap derives for argument parsing                         │
└──────────────────────────────────────────────────────────────┘
```

---

## Type System Diagram (Zero-Cost Abstractions)

```
┌─────────────────────────────────────────────────────────────┐
│                   Type-Level State Machine                   │
│                                                              │
│  ┌────────────────────────┐                                 │
│  │ TurtleOntology<S, N>   │                                 │
│  │   Unvalidated          │                                 │
│  └───────────┬────────────┘                                 │
│              │                                               │
│              │ .validate()                                   │
│              │ (consumes self)                               │
│              ▼                                               │
│  ┌────────────────────────┐                                 │
│  │ TurtleOntology<S, N>   │                                 │
│  │   Validated            │ ◀───────────────────┐           │
│  └───────────┬────────────┘                     │           │
│              │                                   │           │
│              │ .generate()                       │           │
│              │ (only valid for Validated)        │           │
│              ▼                                   │           │
│  ┌────────────────────────┐                     │           │
│  │   TokenStream          │                     │           │
│  │   (Rust code)          │                     │           │
│  └────────────────────────┘                     │           │
│                                                  │           │
│  PhantomData<Validated>: Zero runtime cost ─────┘           │
│  Compile-time validation enforced by types                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              Generic Parameter Monomorphization              │
│                                                              │
│  StorageBackend<S>:                                          │
│    - S = MemoryStorage   → Direct function calls            │
│    - S = DiskStorage     → Direct function calls            │
│    No vtable, no dynamic dispatch                           │
│                                                              │
│  const N: usize (namespace count):                          │
│    - N = 5  → [Namespace; 5]  (stack allocated)            │
│    - N = 10 → [Namespace; 10] (stack allocated)            │
│    Array size validated at compile time                     │
│                                                              │
│  const FEATURES: u32 (feature flags):                       │
│    - if FEATURES & FLAG != 0 { ... } → const-folded        │
│    - Dead code elimination by compiler                      │
│    Zero runtime overhead for unused features                │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Dependency Graph

```
                    ┌─────────────┐
                    │   error.rs  │
                    │ (TurtleErr) │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│  turtle/      │  │  codegen/     │  │  mcp/         │
│  - parser     │  │  - generator  │  │  - tools      │
│  - ontology   │  │  - commands   │  │  - server     │
│  - storage    │  │  - templates  │  │  - types      │
│  - validation │  │  - optimizer  │  │               │
└───────┬───────┘  └───────┬───────┘  └───────┬───────┘
        │                  │                  │
        │   Turtle         │   Command        │   Input/
        │   Ontology       │   Extraction     │   Output
        │                  │                  │   Types
        ▼                  ▼                  ▼
┌───────────────────────────────────────────────────────┐
│                    types.rs                           │
│  RdfTriple, RdfValue, Invocation, Command, etc.      │
└───────────────────────────────────────────────────────┘
        ▲
        │
        │ Uses
        │
┌───────┴───────┐
│  sparql/      │
│  - executor   │
│  - optimizer  │
│  - planner    │
└───────────────┘
```

---

## Error Handling Flow

```
                        ┌────────────────────┐
                        │  TurtleCliError    │
                        │  (Root error)      │
                        └─────────┬──────────┘
                                  │
    ┌─────────────────────────────┼─────────────────────────────┐
    │                             │                             │
    ▼                             ▼                             ▼
┌───────────┐             ┌───────────────┐           ┌──────────────┐
│ParseError │             │GeneratorError │           │  QueryError  │
└───────────┘             └───────────────┘           └──────────────┘
    │                             │                             │
    ├─ SyntaxError               ├─ MissingProperty            ├─ ParseError
    ├─ UndefinedPrefix           ├─ InvalidStructure           ├─ ExecutionError
    ├─ InvalidIri                ├─ TypeMismatch               └─ SerializationError
    └─ DuplicateDefinition       └─ SynthesisError

┌────────────────────────────────────────────────────────────────┐
│  Error Handling Strategy:                                      │
│  1. All errors implement std::error::Error                     │
│  2. Errors carry context (line numbers, entity names)         │
│  3. Conversion via #[from] for ergonomic propagation           │
│  4. No panic paths (all operations return Result)             │
│  5. Error messages include recovery suggestions               │
└────────────────────────────────────────────────────────────────┘
```

---

## Performance Optimization Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                  Hot Path Optimization                       │
│                                                              │
│  Parsing (50ms target for 1000 triples):                    │
│    - Zero-copy string slicing (no allocations)              │
│    - oxigraph native parsing (C++ optimized)                │
│    - Incremental namespace resolution (BTreeMap)            │
│                                                              │
│  Generation (100ms target for 10 commands):                 │
│    - SPARQL query result caching (Arc<QueryResults>)        │
│    - TokenStream builder (no string concatenation)          │
│    - Parallel command generation (rayon, future)            │
│                                                              │
│  SPARQL Queries (10ms target):                              │
│    - oxigraph in-memory store (optimized B-trees)           │
│    - Query plan optimization (SparqlOptimizer)              │
│    - Result streaming (no intermediate buffering)           │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  Memory Layout Optimization                  │
│                                                              │
│  TurtleOntology<MemoryStorage, 5>:                          │
│    - storage: MemoryStorage        (24 bytes + triples)     │
│    - namespaces: [Namespace; 5]    (80 bytes, stack)        │
│    - commands: CommandIndex        (24 bytes + commands)    │
│    - executor: SparqlExecutor      (16 bytes + graph ref)   │
│    - _validation: PhantomData      (0 bytes)                │
│    Total: ~144 bytes + dynamic data                         │
│                                                              │
│  Zero-cost guarantees:                                       │
│    - No heap allocation for PhantomData                     │
│    - No vtable for generic types (monomorphized)            │
│    - No reference counting for owned data                   │
└─────────────────────────────────────────────────────────────┘
```

---

## Testing Strategy Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    Chicago TDD Approach                       │
│                                                               │
│  ┌────────────────┐     ┌────────────────┐                  │
│  │  Arrange       │────▶│     Act        │────┐             │
│  │  (Setup state) │     │ (Execute code) │    │             │
│  └────────────────┘     └────────────────┘    │             │
│                                                │             │
│                                                ▼             │
│                                         ┌────────────────┐   │
│                                         │    Assert      │   │
│                                         │ (Verify state) │   │
│                                         └────────────────┘   │
│                                                               │
│  Test Types:                                                 │
│    - Unit tests:       Individual function behavior          │
│    - Integration tests: Component interaction                │
│    - Property tests:    Invariant verification (proptest)    │
│    - Snapshot tests:    Generated code validation (insta)    │
│                                                               │
│  Principles:                                                 │
│    - State-based testing (not interaction-based)             │
│    - Real collaborators (no mocks)                           │
│    - Behavior verification (observable outputs)              │
│    - AAA pattern (Arrange-Act-Assert)                        │
└──────────────────────────────────────────────────────────────┘
```

---

## Andon Signal Flow (Stop the Line)

```
┌──────────────────────────────────────────────────────────────┐
│                  Development Workflow                         │
│                                                               │
│  1. Write Code                                               │
│     │                                                         │
│     ▼                                                         │
│  2. cargo make check ────────────┐                          │
│     │                              │                          │
│     ├─ Compiler Errors? ──────────┤─▶ STOP (Red Signal)     │
│     ├─ Compiler Warnings? ────────┤─▶ STOP (Yellow Signal)  │
│     │                              │                          │
│     ▼                              │                          │
│  3. cargo make test ──────────────┤                          │
│     │                              │                          │
│     ├─ Test Failures? ────────────┤─▶ STOP (Red Signal)     │
│     │                              │                          │
│     ▼                              │                          │
│  4. cargo make lint ──────────────┤                          │
│     │                              │                          │
│     ├─ Clippy Warnings? ──────────┤─▶ STOP (Yellow Signal)  │
│     │                              │                          │
│     ▼                              ▼                          │
│  5. All Clear ─────────────────▶ Proceed                     │
│                                                               │
│  Andon Signal Response:                                      │
│    1. STOP immediately (no further work)                     │
│    2. INVESTIGATE root cause (5 Whys)                        │
│    3. FIX root cause (not symptom)                           │
│    4. VERIFY signal cleared                                  │
│    5. PROCEED with confidence                                │
└──────────────────────────────────────────────────────────────┘
```

---

## MCP Tool Integration Flow

```
┌──────────────────────────────────────────────────────────────┐
│                   MCP Client (Agent)                          │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            │ JSON-RPC Request
                            │ (stdio)
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                   RdfMcpServer                                │
│                                                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Request Router                                         │  │
│  │  - resources/list   → List available resources        │  │
│  │  - resources/read   → Read resource content           │  │
│  │  - tools/list       → List available tools            │  │
│  │  - tools/call       → Execute tool                    │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │                                   │
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Tool Dispatcher                                        │  │
│  │  - GenerateCliFromTurtle                              │  │
│  │  - QueryCapabilities                                  │  │
│  │  - ExportToTurtle                                     │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │                                   │
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Tool Execution                                         │  │
│  │  1. Validate input (serde deserialize)                │  │
│  │  2. Execute tool logic                                │  │
│  │  3. Serialize output (serde serialize)                │  │
│  │  4. Handle errors (convert to JSON-RPC error)         │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │                                   │
└──────────────────────────┼───────────────────────────────────┘
                           │
                           │ JSON-RPC Response
                           │ (stdio)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   MCP Client (Agent)                          │
│  - Receives generated CLI code                               │
│  - Receives SPARQL query results                             │
│  - Receives Turtle export                                    │
└──────────────────────────────────────────────────────────────┘
```

---

## Architecture Decision Records (Visual)

```
┌────────────────────────────────────────────────────────────┐
│  ADR-001: Generic Storage Backend vs Trait Objects         │
│                                                             │
│  Decision: Use generic StorageBackend<S> trait             │
│                                                             │
│  Trade-offs:                                                │
│    Performance ██████████ (Zero-cost)                      │
│    Binary Size ████────── (Larger)                         │
│    Flexibility ████████── (Compile-time only)              │
│                                                             │
│  Rationale: Performance > Binary size for CLI tools        │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  ADR-002: Const Generic Namespace Count                    │
│                                                             │
│  Decision: Use const N: usize generic parameter            │
│                                                             │
│  Trade-offs:                                                │
│    Type Safety ██████████ (Compile-time validation)        │
│    Flexibility ████────── (Static only)                    │
│    Performance ██████████ (Zero-cost)                      │
│                                                             │
│  Rationale: Static namespace count sufficient for CLIs     │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  ADR-003: State Machine Types (Validated/Unvalidated)      │
│                                                             │
│  Decision: Use PhantomData marker types for validation     │
│                                                             │
│  Trade-offs:                                                │
│    Type Safety ██████████ (Compile-time enforcement)       │
│    API Clarity ████████── (More types)                     │
│    Performance ██████████ (Zero runtime cost)              │
│                                                             │
│  Rationale: Type safety > API simplicity                   │
└────────────────────────────────────────────────────────────┘
```

---

**See also**:
- Architecture Document: `/home/user/clap-noun-verb/docs/rdf-turtle-cli-architecture.md`
- API Reference: `/home/user/clap-noun-verb/docs/rdf-turtle-api-reference.md`
- Implementation Plan: `/home/user/clap-noun-verb/docs/rdf-turtle-implementation-plan.md`
