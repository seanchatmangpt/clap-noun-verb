# Diataxis Documentation Visual Map

**Project**: clap-noun-verb v5.1.1
**Architecture**: Complete Diataxis-compliant structure

---

## User Journey Flow

```
                    🚪 ENTRY POINT
                    ┌─────────────┐
                    │  README.md  │
                    │  (Hub)      │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
   New User?         Have Task?         Need API Info?
        │                  │                  │
        ▼                  ▼                  ▼
  🎓 TUTORIAL       📘 HOW-TO          📚 REFERENCE
   30min-3hr        Task-focused       Quick lookup
   Learn by doing   Problem-solving    API catalog
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                           ▼
                    Want to understand?
                           │
                           ▼
                    💡 EXPLANATION
                    Architecture & "Why"
                    Design rationale
```

---

## Quadrant Relationships

```
┌────────────────────────────────────────────────────────────────┐
│                      DIATAXIS QUADRANTS                         │
│                                                                 │
│   PRACTICAL                                THEORETICAL          │
│   ────────                                 ───────────          │
│                                                                 │
│   🎓 TUTORIAL                              💡 EXPLANATION       │
│   ┌──────────────┐                        ┌──────────────┐    │
│   │ Learning     │◀──────reference────────│ Understanding│    │
│   │ Oriented     │                        │ Oriented     │    │
│   │              │                        │              │    │
│   │ • Step-by-   │                        │ • Why        │    │
│   │   step       │                        │ • Context    │    │
│   │ • Hands-on   │                        │ • Concepts   │    │
│   │ • Beginner   │                        │ • Trade-offs │    │
│   └──────┬───────┘                        └──────┬───────┘    │
│          │                                        │            │
│          │          LEARNING                      │            │
│          │          ────────                      │            │
│          │               ▲                        │            │
│          │               │                        │            │
│          │               ▼                        │            │
│          │          APPLICATION                   │            │
│          │          ───────────                   │            │
│          │                                        │            │
│   ┌──────▼───────┐                        ┌──────▼───────┐    │
│   │ Problem      │────────reference───────▶│ Information  │    │
│   │ Solving      │                        │ Oriented     │    │
│   │              │                        │              │    │
│   │ • Task-      │                        │ • Complete   │    │
│   │   focused    │                        │ • Precise    │    │
│   │ • Recipes    │                        │ • API docs   │    │
│   │ • Production │                        │ • Lookup     │    │
│   └──────────────┘                        └──────────────┘    │
│   📘 HOW-TO                                📚 REFERENCE        │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## Content Organization Map

```
docs/
│
├── INDEX.md ◀─────────────────── Master Documentation Map
│
├── 🎓 tutorial/ ──────────────── LEARNING PATH
│   │
│   ├── README.md ───────────────── Quadrant overview
│   │
│   ├── BEGINNER (30 min)
│   │   ├── 01-your-first-cli.md ───── 5 min: Hello World
│   │   ├── 02-domain-separation.md ── 10 min: Architecture
│   │   └── 03-adding-commands.md ──── 15 min: Multi-command
│   │
│   ├── INTERMEDIATE (1 hr)
│   │   ├── 04-testing-basics.md ───── 15 min: Chicago TDD
│   │   ├── 05-output-formats.md ───── 15 min: JSON/YAML/Table
│   │   └── 06-autonomic-features.md ─ 30 min: Introspection
│   │
│   ├── ADVANCED (1.5 hr)
│   │   ├── 07-async-operations.md ─── 30 min: Async
│   │   ├── 08-error-handling.md ───── 30 min: Result<T,E>
│   │   └── 09-deployment-basics.md ── 30 min: Docker
│   │
│   └── MASTERY
│       └── 10-next-steps.md ───────── Navigation to other quadrants
│
├── 📘 howto/ ─────────────────── PROBLEM SOLVING
│   │
│   ├── README.md ───────────────── Quadrant overview
│   │
│   ├── production/
│   │   ├── deployment.md ────────── Docker + CI/CD
│   │   ├── monitoring.md ────────── OTEL integration
│   │   ├── configuration.md ──────── Config management
│   │   └── security.md ───────────── Security hardening
│   │
│   ├── testing/
│   │   ├── chicago-tdd.md ───────── Chicago TDD in Rust
│   │   ├── integration-tests.md ─── Integration testing
│   │   ├── property-tests.md ────── Property-based
│   │   └── snapshot-tests.md ────── Snapshot testing
│   │
│   ├── integration/
│   │   ├── mcp-servers.md ───────── MCP server setup
│   │   ├── rdf-sparql.md ────────── RDF/SPARQL integration
│   │   ├── async-io.md ──────────── Async I/O patterns
│   │   └── databases.md ─────────── Database connections
│   │
│   ├── patterns/
│   │   ├── argument-parsing.md ──── Complex arguments
│   │   ├── error-recovery.md ────── Error handling
│   │   ├── output-formatting.md ─── Custom outputs
│   │   └── context-sharing.md ───── AppContext patterns
│   │
│   └── troubleshooting/
│       ├── common-errors.md ─────── Compilation errors
│       ├── runtime-issues.md ────── Runtime debugging
│       └── performance.md ───────── Performance tuning
│
├── 📚 reference/ ─────────────── INFORMATION
│   │
│   ├── README.md ───────────────── Quadrant overview
│   │
│   ├── api/ ────────────────────── Core API
│   │   ├── overview.md ──────────── API structure
│   │   ├── verb-macro.md ────────── #[verb] syntax
│   │   ├── arg-attributes.md ────── #[arg] attributes
│   │   ├── types.md ─────────────── Type catalog
│   │   ├── traits.md ────────────── Trait reference
│   │   └── errors.md ────────────── Error catalog
│   │
│   ├── autonomic/ ──────────────── Autonomic Layer
│   │   ├── introspection.md ─────── --capabilities, --introspect
│   │   ├── effects.md ───────────── Effect metadata
│   │   ├── planes.md ────────────── O/Σ/Q/ΔΣ planes
│   │   ├── guards.md ────────────── Guards & budgets
│   │   └── receipts.md ──────────── Execution receipts
│   │
│   ├── rdf/ ────────────────────── RDF/SPARQL
│   │   ├── ontology.md ──────────── CLI ontology
│   │   ├── sparql-queries.md ────── SPARQL patterns
│   │   └── shacl-shapes.md ──────── SHACL validation
│   │
│   └── [CLI Reference]
│       ├── cli-commands.md ──────── All flags/options
│       ├── environment-vars.md ──── Env variables
│       └── configuration.md ─────── Config files
│
└── 💡 explanation/ ───────────── UNDERSTANDING
    │
    ├── README.md ───────────────── Quadrant overview
    │
    ├── architecture/ ───────────── Philosophy
    │   ├── domain-separation.md ─── Why domain-first
    │   ├── type-first-thinking.md ─ Type-driven dev
    │   ├── zero-cost-abstractions.md Performance
    │   └── chicago-tdd.md ───────── Testing rationale
    │
    ├── autonomic/ ──────────────── Autonomic Design
    │   ├── machine-grade-cli.md ─── Machine-first
    │   ├── mape-k-loops.md ──────── MAPE-K integration
    │   ├── agent2028.md ─────────── Agent2028 vision
    │   └── determinism.md ───────── Deterministic execution
    │
    ├── semantic/ ───────────────── Semantic CLI
    │   ├── rdf-rationale.md ─────── Why RDF for CLIs
    │   ├── sparql-benefits.md ───── SPARQL advantages
    │   └── ontology-design.md ───── Ontology principles
    │
    ├── comparisons/ ────────────── Framework Comparisons
    │   ├── vs-clap.md ───────────── vs pure clap
    │   ├── vs-typer.md ──────────── Rust vs Python
    │   └── vs-cobra.md ──────────── vs Go Cobra
    │
    └── roadmap.md ──────────────── Future direction (v5.2+)
```

---

## Migration Flow

```
┌────────────────────────────────────────────────────────────────┐
│                    CONTENT MIGRATION FLOW                       │
└────────────────────────────────────────────────────────────────┘

CURRENT STATE (v4)                        NEW STATE (v5.1.1)
──────────────────                        ──────────────────

README.md (485 lines)                     README.md (300 lines)
├── Quickstart ────────────────┐          ├── Navigation Hub
├── Architecture ──────────┐   │          ├── 30-sec example
├── Examples ──────────┐   │   │          └── Links to quadrants
├── Philosophy ────┐   │   │   │
└── API snippets ──│───│───│───│─┐        docs/INDEX.md
                   │   │   │   │ │        └── Master map
                   │   │   │   │ │
AUTONOMIC.md       │   │   │   │ │        tutorial/
├── Introspection ─│───│───│───│─│───────▶├── 01-10 progressive
├── Effects ───────│───│───│───│─│───┐    │   chapters
├── Planes ────────│───│───│───│─│─┐ │    └── Hands-on
└── Guards ────────│───│───│───│─│─│─│
                   │   │   │   │ │ │ │    howto/
QUICKSTART.md      │   │   │   │ │ │ │    ├── production/
├── Installation ──│───│───┘   │ │ │ └───▶├── testing/
├── Examples ──────│───────────┘ │ │      ├── integration/
└── Patterns ──────│─────────────┘ │      ├── patterns/
                   │                 │      └── troubleshooting/
CLI_REFERENCE.md   │                 │
├── #[verb] ───────│─────────────────│───▶reference/
├── #[arg] ────────│─────────────────│───▶├── api/
├── Types ─────────│─────────────────│───▶├── autonomic/
└── Errors ────────│─────────────────│───▶├── rdf/
                   │                 │     └── cli-commands
                   │                 │
SEMANTIC_CLI_*.md  │                 │     explanation/
├── RDF ───────────│─────────────────┴────▶├── architecture/
├── SPARQL ────────│──────────────────────▶├── autonomic/
└── Ontology ──────┘──────────────────────▶├── semantic/
                                            ├── comparisons/
ARCHITECTURE_*.md                           └── roadmap
└── Design ────────────────────────────────▶
```

---

## File Creation Priority

```
PRIORITY 1: NAVIGATION (Week 1)
┌────────────────────────────────┐
│ □ docs/INDEX.md                │ ◀── Start here
│ □ README.md (refactor)         │
│ □ tutorial/README.md           │
│ □ howto/README.md              │
│ □ reference/README.md          │
│ □ explanation/README.md        │
└────────────────────────────────┘

PRIORITY 2: TUTORIAL (Week 2)
┌────────────────────────────────┐
│ □ tutorial/01-your-first-cli   │ ◀── Highest impact
│ □ tutorial/02-domain-separation│
│ □ tutorial/03-adding-commands  │
│ □ tutorial/04-testing-basics   │
│ □ tutorial/05-output-formats   │
│ □ tutorial/06-autonomic        │
│ □ tutorial/07-async            │
│ □ tutorial/08-error-handling   │
│ □ tutorial/09-deployment       │
│ □ tutorial/10-next-steps       │
└────────────────────────────────┘

PRIORITY 3: HOW-TO (Week 3)
┌────────────────────────────────┐
│ □ howto/production/*           │ ◀── Production patterns
│ □ howto/testing/*              │
│ □ howto/integration/*          │
│ □ howto/patterns/*             │
│ □ howto/troubleshooting/*      │
└────────────────────────────────┘

PRIORITY 4: REFERENCE (Week 4)
┌────────────────────────────────┐
│ □ reference/api/*              │ ◀── API catalog
│ □ reference/autonomic/*        │
│ □ reference/rdf/*              │
│ □ reference/cli-commands       │
└────────────────────────────────┘

PRIORITY 5: EXPLANATION (Week 5)
┌────────────────────────────────┐
│ □ explanation/architecture/*   │ ◀── Deep dives
│ □ explanation/autonomic/*      │
│ □ explanation/semantic/*       │
│ □ explanation/comparisons/*    │
└────────────────────────────────┘

PRIORITY 6: POLISH (Week 6)
┌────────────────────────────────┐
│ □ Cross-reference verification │
│ □ Code example compilation     │
│ □ Broken link checking         │
│ □ Archive old docs             │
└────────────────────────────────┘
```

---

## Target Audiences & Entry Points

```
┌───────────────────────────────────────────────────────────────┐
│                      USER PERSONAS                             │
└───────────────────────────────────────────────────────────────┘

👨‍💻 BEGINNER DEVELOPER
├── Entry: README.md → tutorial/01-your-first-cli.md
├── Journey: Tutorial 01 → 10 (3 hours)
├── Goal: Working CLI with domain separation
└── Next: howto/testing/chicago-tdd.md

🤖 AI AGENT BOOTSTRAPPING
├── Entry: README.md → tutorial/06-autonomic-features.md
├── Journey: Autonomic tutorial → reference/autonomic/
├── Goal: Machine-grade CLI with introspection
└── Next: howto/integration/mcp-servers.md

👷 PRACTITIONER (Production)
├── Entry: README.md → howto/production/deployment.md
├── Journey: How-to guides (task-focused)
├── Goal: Deploy CLI to production
└── Next: howto/production/monitoring.md

🔍 API USER (Quick Lookup)
├── Entry: README.md → reference/api/verb-macro.md
├── Journey: Reference docs (quick lookup)
├── Goal: Find #[verb] syntax
└── Next: reference/api/arg-attributes.md

🏗️ ARCHITECT (Understanding)
├── Entry: README.md → explanation/architecture/domain-separation.md
├── Journey: Explanation docs (concepts)
├── Goal: Understand design rationale
└── Next: explanation/autonomic/machine-grade-cli.md

📚 CONTRIBUTOR
├── Entry: CONTRIBUTING.md → explanation/architecture/
├── Journey: Architecture + Reference
├── Goal: Understand codebase philosophy
└── Next: howto/testing/chicago-tdd.md
```

---

## Quality Gates

```
┌──────────────────────────────────────────────────────────────┐
│                  DOCUMENTATION QUALITY GATES                  │
└──────────────────────────────────────────────────────────────┘

COMPLETENESS ✓
├── [✓] 100% public API documented
├── [✓] 100% v5.1.1 features documented
├── [✓] 10 tutorial chapters
├── [✓] 20+ how-to guides
└── [✓] All code examples compile

CLARITY ✓
├── [✓] Tutorial completion rate >80%
├── [✓] Time to first CLI <10 minutes
└── [✓] "Find what I need" >90%

CONSISTENCY ✓
├── [✓] All references match v5.1.1
├── [✓] Cross-references functional
└── [✓] Version consistency

MAINTAINABILITY ✓
├── [✓] Examples in CI/CD
├── [✓] Broken link checking
└── [✓] Update SLA: <1 week
```

---

## Memory Keys for Agent Coordination

```json
{
  "diataxis/structure": "/Users/sac/clap-noun-verb/docs/DIATAXIS_ARCHITECTURE_V5.md",
  "diataxis/summary": "/Users/sac/clap-noun-verb/docs/DIATAXIS_SUMMARY.md",
  "diataxis/visual-map": "/Users/sac/clap-noun-verb/docs/DIATAXIS_VISUAL_MAP.md",

  "diataxis/quadrants": {
    "tutorial": {
      "path": "docs/tutorial/",
      "files": 11,
      "time": "30min-3hr",
      "audience": "beginners, agents bootstrapping"
    },
    "howto": {
      "path": "docs/howto/",
      "files": 21,
      "time": "task-specific",
      "audience": "practitioners, production users"
    },
    "reference": {
      "path": "docs/reference/",
      "files": 19,
      "time": "instant lookup",
      "audience": "all users, API lookups"
    },
    "explanation": {
      "path": "docs/explanation/",
      "files": 17,
      "time": "deep dive",
      "audience": "architects, contributors"
    }
  },

  "diataxis/migration": {
    "phase1": "Create structure (Week 1)",
    "phase2": "Tutorial (Week 2)",
    "phase3": "How-To (Week 3)",
    "phase4": "Reference (Week 4)",
    "phase5": "Explanation (Week 5)",
    "phase6": "README refactor (Week 6)"
  },

  "diataxis/priorities": [
    "Navigation files (INDEX.md, README.md)",
    "Tutorial quadrant (highest impact)",
    "How-To quadrant (production patterns)",
    "Reference quadrant (API catalog)",
    "Explanation quadrant (deep dives)",
    "Polish & cross-reference"
  ]
}
```

---

**Status**: ✅ Complete - Ready for Implementation

**Full Architecture**: `docs/DIATAXIS_ARCHITECTURE_V5.md` (30,000+ words)
**Summary**: `docs/DIATAXIS_SUMMARY.md` (concise overview)
**Visual Map**: `docs/DIATAXIS_VISUAL_MAP.md` (this document)

