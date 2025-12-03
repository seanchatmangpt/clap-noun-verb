# Diataxis Documentation Architecture for clap-noun-verb v5.0.0

**Version**: 1.0.0
**Author**: System Architect
**Date**: 2025-11-20
**Status**: Design Specification
**Audience**: AI Coding Agents (Claude Code, GitHub Copilot, MCP agents)

---

## Executive Summary

This document specifies a complete **Diataxis-based documentation architecture** for clap-noun-verb v5.0.0, targeting **AI coding agents** and senior engineers. The architecture emphasizes:

1. **Domain separation**: CLI code ≠ Business logic
2. **Agent-friendly patterns**: Production-ready, not toy examples
3. **Zero hand-holding**: Assumes agent understands Rust, CLIs, RDF, MCP
4. **Compilation-verified examples**: All code examples must compile with v5.0.0 API

### Core Principle: "CLI is Interface, Domain is Logic"

**Every example demonstrates:**
- CLI layer handles argument parsing, validation, routing
- Domain layer contains pure business logic, testable in isolation
- Integration layer provides minimal glue code

---

## Table of Contents

1. [Diataxis Framework Overview](#diataxis-framework-overview)
2. [Documentation Hierarchy](#documentation-hierarchy)
3. [Content Strategy by Quadrant](#content-strategy-by-quadrant)
4. [Document Inventory](#document-inventory)
5. [README.md Hub Design](#readmemd-hub-design)
6. [Code Example Requirements](#code-example-requirements)
7. [Migration Plan](#migration-plan)
8. [Agent Integration](#agent-integration)

---

## Diataxis Framework Overview

### Four Documentation Quadrants

```
                Learning-Oriented  |  Problem-Oriented
                                   |
    TUTORIALS                      |      HOW-TO GUIDES
    (Learning by doing)            |      (Achieving goals)
    - Hands-on lessons             |      - Step-by-step recipes
    - Guided path                  |      - Specific problems
    - Confidence building          |      - Practical solutions
                                   |
─────────────────────────────────────────────────────────────
                                   |
    EXPLANATION                    |      REFERENCE
    (Understanding)                |      (Information)
    - Concepts                     |      - API documentation
    - Architecture                 |      - Complete catalog
    - Philosophy                   |      - Technical specs
                                   |
    Understanding-Oriented         |  Information-Oriented
```

### Adaptation for AI Agents

**Traditional Diataxis** → **Agent-Focused Diataxis**:

| Quadrant | Traditional | Agent-Focused |
|----------|-------------|---------------|
| **Tutorial** | "What is a CLI?" | "Production CLI in 5 minutes" (assumes agent knows CLIs) |
| **How-To** | "Add a flag" | "Separate domain logic from CLI" (production patterns) |
| **Reference** | API docs | Complete type catalog, trait bounds, zero-cost abstractions |
| **Explanation** | "Why CLIs matter" | "Why domain separation, type-first thinking, semantic CLI control" |

---

## Documentation Hierarchy

```
docs/
├── tutorial/                        # LEARNING (Hands-on)
│   ├── 00-quickstart.md            # 5-min: Production CLI from scratch
│   ├── 01-domain-separation.md     # Pattern: CLI vs Domain
│   ├── 02-semantic-cli.md          # Pattern: RDF ontology for agents
│   ├── 03-mcp-integration.md       # Pattern: Expose CLI via MCP
│   └── 04-autonomic-control.md     # Pattern: Guards, receipts, planes
│
├── how-to/                          # PROBLEM-SOLVING (Goal-oriented)
│   ├── separate-domain-logic.md    # HOW-TO: Extract domain logic
│   ├── add-semantic-metadata.md    # HOW-TO: RDF metadata for discoverability
│   ├── implement-guards.md         # HOW-TO: Add runtime constraints
│   ├── expose-via-mcp.md           # HOW-TO: Create MCP server
│   ├── handle-async-operations.md  # HOW-TO: Async in sync handlers
│   ├── test-domain-layer.md        # HOW-TO: Chicago TDD for domain
│   ├── optimize-performance.md     # HOW-TO: Zero-cost abstractions
│   └── migrate-from-clap.md        # HOW-TO: Port existing clap CLI
│
├── reference/                       # INFORMATION (Complete catalog)
│   ├── api-index.md                # All public APIs
│   ├── attribute-macros.md         # #[verb], #[noun], #[arg]
│   ├── rdf-ontology.md             # CNV ontology definition
│   ├── sparql-queries.md           # Query patterns
│   ├── mcp-resources.md            # MCP resource URIs
│   ├── autonomic-types.md          # Effect, Guard, Receipt types
│   ├── error-catalog.md            # All error types
│   └── trait-reference.md          # VerbCommand, AutonomicVerbCommand
│
└── explanation/                     # UNDERSTANDING (Concepts)
    ├── architecture-overview.md    # System design
    ├── domain-separation.md        # Why separate CLI from logic
    ├── semantic-cli-philosophy.md  # Why RDF for CLIs
    ├── type-first-thinking.md      # Why types encode invariants
    ├── zero-cost-abstractions.md   # How generics/macros optimize
    ├── autonomic-control.md        # Why machine-grade interfaces
    └── agent-integration.md        # How agents discover/invoke
```

---

## Content Strategy by Quadrant

### 1. Tutorial (Learning-Oriented)

**Target**: Agent building first production CLI with clap-noun-verb.

**Tone**: Directive, practical, assumes Rust knowledge.

**Structure**:
- **Goal**: What agent will build
- **Prerequisites**: Rust, cargo, basic CLI concepts
- **Steps**: Numbered, compilation-verified
- **Code**: Complete, runnable, domain-separated
- **Checkpoints**: "Run `cargo make check` - should compile"

**Example Outline** (`tutorial/00-quickstart.md`):

```markdown
# Production CLI in 5 Minutes

**Goal**: Build a production-ready CLI with domain separation, semantic metadata, and MCP integration.

**Prerequisites**: Rust 1.74+, clap-noun-verb 5.0.0

## Step 1: Domain Layer (Pure Business Logic)

src/domain/user_service.rs:
\```rust
// Pure domain logic - no CLI dependencies
pub struct UserService { ... }

impl UserService {
    pub fn create_user(&self, email: &str) -> Result<User, DomainError> {
        // Validation, business rules
    }
}
\```

## Step 2: CLI Layer (Argument Parsing)

src/cli/user_commands.rs:
\```rust
use clap_noun_verb_macros::verb;

#[verb(noun = "user", name = "create")]
fn create_user_command(
    #[arg(long)] email: String
) -> Result<User> {
    // Delegate to domain
    let service = UserService::new();
    service.create_user(&email)
}
\```

## Step 3: Verify Separation

Checkpoint: Domain tests run without CLI
\```bash
cargo test --lib  # Domain tests only
\```
```

**Key Characteristics**:
- ✅ Compiles with v5.0.0
- ✅ Demonstrates domain separation from step 1
- ✅ No "Hello World" - realistic domain example
- ✅ Checkpoint after each step

### 2. How-To Guides (Problem-Oriented)

**Target**: Agent solving specific production problems.

**Tone**: Prescriptive, recipe-style, solution-focused.

**Structure**:
- **Problem**: Clear statement of what agent wants to achieve
- **Solution**: Step-by-step recipe
- **Code**: Focused on problem, not complete app
- **Alternatives**: Trade-offs between approaches
- **Anti-patterns**: What NOT to do

**Example Outline** (`how-to/separate-domain-logic.md`):

```markdown
# How to Separate Domain Logic from CLI

**Problem**: Mixing business logic in CLI handlers makes testing hard and violates separation of concerns.

**Solution**: Extract domain logic into pure functions/structs, keep CLI as thin routing layer.

## Pattern 1: Service Layer

\```rust
// Domain: src/domain/order_service.rs
pub struct OrderService { ... }

impl OrderService {
    pub fn place_order(&self, items: Vec<Item>) -> Result<Order, DomainError> {
        // All business logic here
    }
}

// CLI: src/cli/order_commands.rs
#[verb]
fn place_order(items: Vec<String>) -> Result<Order> {
    let service = OrderService::new();
    service.place_order(&parse_items(&items))  // Delegate
}
\```

## Pattern 2: Pure Functions

\```rust
// Domain: src/domain/pricing.rs
pub fn calculate_total(items: &[Item], discount: Option<f64>) -> f64 {
    // Pure function - no I/O, no state
}

// CLI: src/cli/pricing_commands.rs
#[verb]
fn calculate(items: Vec<String>, discount: Option<f64>) -> Result<f64> {
    Ok(calculate_total(&parse_items(&items), discount))
}
\```

## Anti-Pattern: Logic in CLI Handler

\```rust
// ❌ BAD: Business logic in CLI
#[verb]
fn place_order(items: Vec<String>) -> Result<Order> {
    // Validation here
    // Database access here
    // Business rules here
    // Impossible to test without CLI framework
}
\```

## Testing Strategy

\```rust
#[test]
fn test_domain_without_cli() {
    // Test domain logic directly
    let service = OrderService::new();
    assert!(service.place_order(vec![item]).is_ok());
}
\```
```

**Key Characteristics**:
- ✅ Solves ONE specific problem
- ✅ Shows multiple patterns (service layer, pure functions)
- ✅ Includes anti-patterns (what NOT to do)
- ✅ Testing strategy for domain layer

### 3. Reference (Information-Oriented)

**Target**: Agent looking up API details, type signatures, trait bounds.

**Tone**: Dry, precise, exhaustive.

**Structure**:
- **Type Signature**: Complete, with trait bounds
- **Description**: What it does (not how to use)
- **Parameters**: All parameters, types, constraints
- **Returns**: Return type, error cases
- **Examples**: Minimal, focused on API usage
- **Trait Implementations**: All implemented traits

**Example Outline** (`reference/attribute-macros.md`):

```markdown
# Attribute Macros Reference

## #[verb]

**Signature**:
\```rust
#[verb(
    noun: Option<&str>,
    name: Option<&str>,
    description: Option<&str>
)]
\```

**Purpose**: Registers a function as a verb command with compile-time metadata generation.

**Parameters**:
- `noun` (optional): Noun name override (defaults to filename stem)
- `name` (optional): Verb name override (defaults to function name)
- `description` (optional): Command description (defaults to doc comment)

**Generated Code**:
- `VerbMetadata` trait impl
- RDF triple generation (if `rdf-control` feature enabled)
- SHACL shape generation (if constraints present)
- linkme registration slice entry

**Constraints**:
- Function must return `Result<T>` where `T: Serialize`
- All arguments must implement `FromStr` or have `#[arg]` attribute
- Function must be `pub` or module-visible

**Example**:
\```rust
#[verb(noun = "services", name = "status")]
fn show_status() -> Result<ServiceStatus> {
    // Implementation
}
\```

**Compile-Time Expansion**:
\```rust
impl VerbMetadata for ShowStatus {
    fn verb_name() -> &'static str { "status" }
    fn noun_name() -> &'static str { "services" }
    fn to_rdf_triples() -> Vec<RdfTriple> { /* ... */ }
}
\```

**Feature Gates**:
- `rdf-control`: Enables RDF metadata generation
- `autonomic`: Enables autonomic control metadata
```

**Key Characteristics**:
- ✅ Complete type information
- ✅ All parameters documented
- ✅ Constraints explicitly stated
- ✅ Generated code visible
- ✅ Feature gates documented

### 4. Explanation (Understanding-Oriented)

**Target**: Agent understanding *why* architecture decisions were made.

**Tone**: Analytical, conceptual, trade-off focused.

**Structure**:
- **Concept**: What is being explained
- **Motivation**: Why this approach
- **Alternatives**: Other approaches considered
- **Trade-offs**: Benefits vs costs
- **Mental Model**: How to think about it
- **Examples**: Illustrative, not prescriptive

**Example Outline** (`explanation/domain-separation.md`):

```markdown
# Why Separate Domain Logic from CLI

## The Problem

**Mixed concerns** violate testability, reusability, and maintainability:

\```rust
// ❌ Violates separation
#[verb]
fn process_order(order_id: String) -> Result<()> {
    // CLI parsing
    let id = order_id.parse()?;

    // Business validation
    if !is_valid_order(id) { return Err(...); }

    // Database access
    let db = Database::connect()?;
    db.update_order(id)?;

    // Impossible to test without CLI framework
    // Impossible to reuse in non-CLI context
}
\```

## The Solution: Three-Layer Architecture

```
┌──────────────────────────────────────┐
│   CLI Layer (Parsing, Routing)       │
│   - clap argument parsing             │
│   - Validation of input format        │
│   - Routing to domain                 │
└──────────────────────────────────────┘
                 ▼
┌──────────────────────────────────────┐
│   Domain Layer (Business Logic)      │
│   - Pure functions                    │
│   - Business rules                    │
│   - Type-safe operations              │
└──────────────────────────────────────┘
                 ▼
┌──────────────────────────────────────┐
│   Infrastructure Layer (I/O)         │
│   - Database access                   │
│   - File system                       │
│   - Network calls                     │
└──────────────────────────────────────┘
```

## Correct Implementation

\```rust
// Domain Layer: src/domain/order_service.rs
pub struct OrderService {
    repo: Arc<OrderRepository>,
}

impl OrderService {
    pub fn process_order(&self, id: OrderId) -> Result<Order, DomainError> {
        // Pure business logic
        let order = self.repo.find(id)?;
        order.validate()?;
        order.process()?;
        self.repo.save(order)
    }
}

// CLI Layer: src/cli/order_commands.rs
#[verb]
fn process_order(order_id: String) -> Result<Order> {
    let id = OrderId::from_str(&order_id)?;  // Parse
    let service = OrderService::new();       // Dependency injection
    service.process_order(id)                // Delegate
}

// Test: tests/domain/order_service_test.rs
#[test]
fn test_process_order_without_cli() {
    let repo = MockRepository::new();
    let service = OrderService { repo };

    let result = service.process_order(OrderId(123));

    assert!(result.is_ok());
    // No CLI framework needed
}
\```

## Benefits

1. **Testability**: Domain layer tests run without CLI
2. **Reusability**: Domain logic usable from HTTP API, gRPC, CLI
3. **Maintainability**: Changes to CLI don't affect domain
4. **Type Safety**: Domain enforces invariants at type level

## Trade-offs

**Cost**: More files, indirection
**Benefit**: Testability, reusability, maintainability
**Verdict**: Always worth it for production code

## Mental Model

**CLI = User Interface**
**Domain = Core Business Logic**
**Infrastructure = External World**

CLI translates user intent → domain operations → infrastructure effects.
```

**Key Characteristics**:
- ✅ Explains *why*, not *how*
- ✅ Shows anti-pattern first (motivation)
- ✅ Analyzes trade-offs explicitly
- ✅ Provides mental model
- ✅ No step-by-step instructions (that's Tutorial/How-To)

---

## Document Inventory

### Tutorial Documents

| Document | Title | Goal | Audience | Lines |
|----------|-------|------|----------|-------|
| `tutorial/00-quickstart.md` | Production CLI in 5 Minutes | Build domain-separated CLI | New agents | ~300 |
| `tutorial/01-domain-separation.md` | Domain-First Development | Separate CLI from logic | All agents | ~400 |
| `tutorial/02-semantic-cli.md` | Semantic CLI with RDF | Add RDF metadata for agents | Advanced agents | ~500 |
| `tutorial/03-mcp-integration.md` | Expose CLI via MCP | Create MCP server | MCP agents | ~400 |
| `tutorial/04-autonomic-control.md` | Autonomic CLI Layer | Add guards, receipts, planes | Control system agents | ~450 |

### How-To Guides

| Document | Title | Problem | Solution Pattern | Lines |
|----------|-------|---------|------------------|-------|
| `how-to/separate-domain-logic.md` | Extract Domain Logic | Mixed concerns | Service layer, pure functions | ~350 |
| `how-to/add-semantic-metadata.md` | Add RDF Metadata | Agent discovery | RDF ontology, SPARQL | ~300 |
| `how-to/implement-guards.md` | Runtime Constraints | Resource budgets | Guard configuration | ~250 |
| `how-to/expose-via-mcp.md` | MCP Server | Agent integration | MCP resource/tool pattern | ~400 |
| `how-to/handle-async-operations.md` | Async in Sync Handlers | Async domain logic | run_async pattern | ~200 |
| `how-to/test-domain-layer.md` | Test Without CLI | Domain testing | Chicago TDD, mocks | ~300 |
| `how-to/optimize-performance.md` | Zero-Cost Patterns | Performance | Generics, const generics | ~350 |
| `how-to/migrate-from-clap.md` | Port from clap | Legacy migration | Step-by-step refactor | ~450 |

### Reference Documents

| Document | Title | Content | Lines |
|----------|-------|---------|-------|
| `reference/api-index.md` | API Index | All public types, functions | ~200 |
| `reference/attribute-macros.md` | Macro Reference | #[verb], #[noun], #[arg] complete spec | ~500 |
| `reference/rdf-ontology.md` | RDF Ontology | CNV namespace, classes, properties | ~400 |
| `reference/sparql-queries.md` | SPARQL Patterns | Common queries catalog | ~300 |
| `reference/mcp-resources.md` | MCP Resource URIs | All clnv:// resources | ~250 |
| `reference/autonomic-types.md` | Autonomic Types | Effect, Guard, Receipt complete spec | ~350 |
| `reference/error-catalog.md` | Error Types | All error variants | ~300 |
| `reference/trait-reference.md` | Trait Catalog | VerbCommand, AutonomicVerbCommand | ~400 |

### Explanation Documents

| Document | Title | Concept | Lines |
|----------|-------|---------|-------|
| `explanation/architecture-overview.md` | System Architecture | High-level design | ~400 |
| `explanation/domain-separation.md` | Why Domain Separation | Three-layer architecture | ~450 |
| `explanation/semantic-cli-philosophy.md` | Why RDF for CLIs | Machine-readable interfaces | ~500 |
| `explanation/type-first-thinking.md` | Type-Level Design | Invariants in types | ~400 |
| `explanation/zero-cost-abstractions.md` | Zero-Cost Principles | How macros/generics optimize | ~350 |
| `explanation/autonomic-control.md` | Machine-Grade Interfaces | Guards, receipts, planes | ~450 |
| `explanation/agent-integration.md` | Agent Discovery & Invocation | How agents use CLIs | ~400 |

**Total**: ~21 core documents, ~7,500 lines

---

## README.md Hub Design

### Current README Problems

1. **Too tutorial-focused**: Mixes tutorial, reference, explanation
2. **Beginner-centric**: "What is clap-noun-verb?" (agents don't need this)
3. **Poor examples**: Missing domain separation
4. **Navigation weak**: Links scattered, no clear quadrants

### New README Structure

```markdown
# clap-noun-verb v5.0.0

**Production-ready CLI framework for AI agents with semantic control.**

## What is clap-noun-verb?

A Rust framework for building **autonomic CLIs** with:
- **Domain separation**: CLI code ≠ business logic
- **Semantic discovery**: RDF ontology for agent introspection
- **Machine-grade control**: Guards, receipts, planes (O/Σ/Q/ΔΣ)
- **MCP integration**: Expose CLI as MCP server
- **Zero-cost abstractions**: Compile-time metadata generation

**Not**: A clap wrapper with sugar syntax
**Is**: A framework for building production CLIs agents can discover, reason about, and invoke

## Quick Start (5 minutes)

\```rust
// Domain logic (pure, testable)
pub struct UserService { ... }

// CLI layer (routing only)
#[verb(noun = "user", name = "create")]
fn create_user(email: String) -> Result<User> {
    UserService::new().create_user(&email)
}
\```

[Complete tutorial →](docs/tutorial/00-quickstart.md)

---

## Documentation by Quadrant

### 📘 Learning (Tutorials)

**New to clap-noun-verb?** Start here:

| Tutorial | Goal | Time |
|----------|------|------|
| [Production CLI in 5 Minutes](docs/tutorial/00-quickstart.md) | Build domain-separated CLI | 5 min |
| [Domain-First Development](docs/tutorial/01-domain-separation.md) | Separate CLI from logic | 10 min |
| [Semantic CLI with RDF](docs/tutorial/02-semantic-cli.md) | Add RDF metadata | 15 min |
| [MCP Integration](docs/tutorial/03-mcp-integration.md) | Expose via MCP | 10 min |
| [Autonomic Control](docs/tutorial/04-autonomic-control.md) | Add guards, receipts | 15 min |

**Total**: 55 minutes to production-ready autonomic CLI

### 🔧 Problem-Solving (How-To Guides)

**Common production problems:**

| Problem | Solution |
|---------|----------|
| Mixed CLI and domain logic | [Separate domain logic](docs/how-to/separate-domain-logic.md) |
| Agent can't discover commands | [Add RDF metadata](docs/how-to/add-semantic-metadata.md) |
| Need runtime constraints | [Implement guards](docs/how-to/implement-guards.md) |
| Want MCP integration | [Expose via MCP](docs/how-to/expose-via-mcp.md) |
| Async operations in CLI | [Handle async](docs/how-to/handle-async-operations.md) |
| Testing CLI commands | [Test domain layer](docs/how-to/test-domain-layer.md) |
| Performance optimization | [Zero-cost patterns](docs/how-to/optimize-performance.md) |
| Migrating from clap | [Migration guide](docs/how-to/migrate-from-clap.md) |

### 📚 Information (Reference)

**Complete API catalog:**

| Reference | Content |
|-----------|---------|
| [API Index](docs/reference/api-index.md) | All public types/functions |
| [Attribute Macros](docs/reference/attribute-macros.md) | #[verb], #[noun], #[arg] |
| [RDF Ontology](docs/reference/rdf-ontology.md) | CNV namespace, classes |
| [SPARQL Queries](docs/reference/sparql-queries.md) | Query patterns |
| [MCP Resources](docs/reference/mcp-resources.md) | clnv:// URI catalog |
| [Autonomic Types](docs/reference/autonomic-types.md) | Effect, Guard, Receipt |
| [Error Catalog](docs/reference/error-catalog.md) | All error types |
| [Trait Reference](docs/reference/trait-reference.md) | VerbCommand, etc. |

### 💡 Understanding (Explanation)

**Why clap-noun-verb works this way:**

| Concept | Explanation |
|---------|-------------|
| [Architecture](docs/explanation/architecture-overview.md) | System design overview |
| [Domain Separation](docs/explanation/domain-separation.md) | Why separate CLI from logic |
| [Semantic CLI](docs/explanation/semantic-cli-philosophy.md) | Why RDF for CLIs |
| [Type-First Thinking](docs/explanation/type-first-thinking.md) | Invariants in types |
| [Zero-Cost Abstractions](docs/explanation/zero-cost-abstractions.md) | How macros optimize |
| [Autonomic Control](docs/explanation/autonomic-control.md) | Machine-grade interfaces |
| [Agent Integration](docs/explanation/agent-integration.md) | How agents use CLIs |

---

## Architecture Principles

1. **CLI is Interface, Domain is Logic**: Always separate
2. **Types Encode Invariants**: Use Rust's type system
3. **Zero-Cost by Default**: Macros generate optimal code
4. **Agent-First Design**: RDF for discovery, MCP for invocation
5. **Semantic Control**: Guards, receipts, planes for autonomic systems

[Read full architecture →](docs/explanation/architecture-overview.md)

---

## Examples

All examples demonstrate **domain separation**:

\```bash
# Conference management (domain: scheduling, submissions)
cargo run --example conference_management

# Semantic submissions (domain: review, decision logic)
cargo run --example semantic_submissions

# Template generator (domain: template rendering)
cargo run --example template_generator
\```

[Browse examples →](examples/)

---

## Features

- ✅ **Domain separation**: Three-layer architecture
- ✅ **Attribute macros**: #[verb], #[noun], #[arg]
- ✅ **Semantic CLI**: RDF ontology, SPARQL queries
- ✅ **MCP integration**: Stdio MCP server
- ✅ **Autonomic control**: Guards, receipts, planes
- ✅ **Zero-cost abstractions**: Compile-time metadata
- ✅ **Chicago TDD**: Domain testable without CLI

---

## Installation

\```toml
[dependencies]
clap-noun-verb = "5.0.0"
clap-noun-verb-macros = "5.0.0"

# Optional features
rmcp = "0.9"  # MCP SDK
oxigraph = "0.5.1"  # SPARQL engine
\```

---

## License

MIT OR Apache-2.0
\```

**Key Changes**:
- ✅ Clear Diataxis quadrants in README
- ✅ Emphasizes domain separation upfront
- ✅ No "What is a CLI?" beginner content
- ✅ Links organized by problem type
- ✅ Architecture principles visible

---

## Code Example Requirements

### All Examples Must Demonstrate

1. **Domain Separation**:
   \```rust
   // ✅ CORRECT
   // Domain: src/domain/service.rs
   pub struct Service { ... }

   // CLI: src/cli/commands.rs
   #[verb]
   fn command() -> Result<T> {
       Service::new().operation()
   }

   // ❌ WRONG
   #[verb]
   fn command() -> Result<T> {
       // Business logic here - violates separation
   }
   \```

2. **Type-First Design**:
   \```rust
   // ✅ CORRECT: Types encode constraints
   pub struct Email(String);  // Validated at construction

   impl Email {
       pub fn new(s: &str) -> Result<Self, ValidationError> {
           // Validation here
       }
   }

   // ❌ WRONG: String everywhere, validation scattered
   pub fn create_user(email: String) { ... }
   \```

3. **Realistic Domain**:
   \```rust
   // ✅ CORRECT: Production-realistic
   pub struct OrderService {
       repo: Arc<OrderRepository>,
       payment: Arc<PaymentGateway>,
   }

   // ❌ WRONG: Toy example
   pub fn hello_world() { ... }
   \```

4. **Compilation-Verified**:
   - All examples must compile with `cargo make check`
   - All examples must have `cargo test` passing
   - No placeholder code, no TODO comments

5. **Zero-Cost Visible**:
   \```rust
   // ✅ CORRECT: Show zero-cost abstraction
   #[verb]  // Expands to zero-cost VerbMetadata impl
   fn status<T: Status>() -> Result<T> {
       // Generic monomorphizes - zero-cost
   }

   // Explain in comment:
   // "Macro generates static dispatch, no runtime cost"
   \```

---

## Migration Plan

### Phase 1: Analyze Current README (Complete)

**Current Problems Identified**:
1. Mixed quadrants (tutorial + reference + explanation)
2. Beginner-focused ("What is a CLI?")
3. Examples lack domain separation
4. Navigation unclear

### Phase 2: Create Skeleton (Week 1)

**Deliverables**:
- [ ] Create `docs/` directory structure
- [ ] Write empty files with headings
- [ ] Create README.md hub with Diataxis quadrants
- [ ] Add navigation links

### Phase 3: Populate Tutorial (Week 2)

**Deliverables**:
- [ ] `tutorial/00-quickstart.md` (domain-separated CLI)
- [ ] `tutorial/01-domain-separation.md` (three-layer pattern)
- [ ] `tutorial/02-semantic-cli.md` (RDF ontology)
- [ ] `tutorial/03-mcp-integration.md` (MCP server)
- [ ] `tutorial/04-autonomic-control.md` (guards, receipts)

**Verification**:
- All code examples compile
- All checkpoints pass
- Agent can build production CLI in 55 minutes

### Phase 4: Populate How-To (Week 3)

**Deliverables**:
- [ ] All 8 how-to guides (see inventory)
- [ ] Anti-patterns documented
- [ ] Trade-offs analyzed

**Verification**:
- Each guide solves ONE problem
- Code examples focused on problem
- Alternatives documented

### Phase 5: Populate Reference (Week 4)

**Deliverables**:
- [ ] Complete API catalog
- [ ] All type signatures
- [ ] All trait bounds
- [ ] Error catalog

**Verification**:
- Coverage: 100% of public APIs
- Precision: All signatures correct
- Completeness: All parameters documented

### Phase 6: Populate Explanation (Week 5)

**Deliverables**:
- [ ] All 7 explanation documents (see inventory)
- [ ] Mental models defined
- [ ] Trade-offs analyzed

**Verification**:
- Explains *why*, not *how*
- Analyzes alternatives
- Provides mental models

### Phase 7: Validation (Week 6)

**Validation Steps**:
1. Agent runs through all tutorials → CLI builds successfully
2. Agent solves problems using how-tos → Solutions work
3. Agent looks up APIs in reference → Information complete
4. Agent understands concepts from explanations → Design decisions clear

**Success Criteria**:
- Tutorial completion rate: >90%
- How-to effectiveness: Problems solved
- Reference completeness: No missing APIs
- Explanation clarity: Concepts understood

---

## Agent Integration

### How Agents Use This Documentation

#### 1. Discovery Phase (MCP Agent)

**Goal**: Understand what CLI can do

**Path**:
1. Read `explanation/architecture-overview.md` (understand design)
2. Read `reference/mcp-resources.md` (discover clnv:// URIs)
3. Query `clnv://ontology/commands` (get command list)
4. Use `how-to/expose-via-mcp.md` (integrate)

#### 2. Learning Phase (Claude Code Agent)

**Goal**: Build production CLI

**Path**:
1. Read `tutorial/00-quickstart.md` (5-min intro)
2. Follow `tutorial/01-domain-separation.md` (learn pattern)
3. Reference `reference/attribute-macros.md` (macro details)
4. Read `explanation/domain-separation.md` (understand why)

#### 3. Problem-Solving Phase (GitHub Copilot)

**Goal**: Fix specific issue

**Path**:
1. Identify problem (mixed concerns in CLI)
2. Search `how-to/separate-domain-logic.md` (solution)
3. Apply pattern from guide
4. Verify with `how-to/test-domain-layer.md`

#### 4. Maintenance Phase (Agent Swarm)

**Goal**: Understand existing code

**Path**:
1. Read `explanation/type-first-thinking.md` (understand design)
2. Reference `reference/trait-reference.md` (trait details)
3. Consult `reference/error-catalog.md` (error handling)

### Agent-Specific Features

**Claude Code**:
- Diataxis quadrants in sidebar
- "Run this example" links → direct cargo commands
- Checkpoint validation after each tutorial step

**GitHub Copilot**:
- Inline code examples from how-tos
- Type signatures from reference
- Anti-patterns from explanations

**MCP Agents**:
- Direct links to `clnv://` resources
- SPARQL query examples
- MCP tool invocation patterns

---

## Conclusion

This Diataxis architecture provides:

1. **Agent-First Documentation**: No hand-holding, production patterns
2. **Domain Separation Emphasis**: Every example demonstrates this
3. **Complete Coverage**: Tutorial, How-To, Reference, Explanation
4. **Compilation-Verified**: All examples compile with v5.0.0
5. **Agent Integration**: MCP, Claude Code, Copilot paths

**Next Steps**:
1. Review this specification
2. Begin Phase 2 (skeleton creation)
3. Iterate based on agent feedback

**Estimated Effort**:
- 6 weeks for complete documentation
- ~21 core documents
- ~7,500 lines of documentation
- ~50 code examples (all compile-verified)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-20
**Authors**: System Architecture Designer (Claude Code)
**References**:
- Diataxis Framework: https://diataxis.fr/
- Current README: /Users/sac/clap-noun-verb/README.md
- Semantic CLI Architecture: docs/SEMANTIC_CLI_ARCHITECTURE.md
- RDF v5 Architecture: docs/rdf-v5-architecture.md
- Autonomic Layer: AUTONOMIC.md
