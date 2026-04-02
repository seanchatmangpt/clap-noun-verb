# Diataxis Documentation Architecture for clap-noun-verb v5.1.1

**Version**: 5.1.1
**Date**: 2025-12-02
**Status**: Architecture Specification
**Framework**: [Diataxis](https://diataxis.fr/)

---

## Executive Summary

This document specifies a complete Diataxis-compliant documentation architecture for clap-noun-verb v5.1.1, transforming fragmented v4 documentation into a systematic, user-journey-optimized structure.

**Key Goals**:
- **Tutorial**: Learning-oriented paths for beginners (agents bootstrapping CLI projects)
- **How-To**: Problem-solving guides for production patterns
- **Reference**: Complete API catalog for quick lookups
- **Explanation**: Architecture and philosophy for understanding "why"

**Migration Strategy**: Incremental refactor preserving existing v4 content while creating new v5 Diataxis structure.

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Diataxis Directory Structure](#2-diataxis-directory-structure)
3. [Tutorial Quadrant](#3-tutorial-quadrant-learning)
4. [How-To Quadrant](#4-how-to-quadrant-problem-solving)
5. [Reference Quadrant](#5-reference-quadrant-information)
6. [Explanation Quadrant](#6-explanation-quadrant-understanding)
7. [README.md as Navigation Hub](#7-readmemd-as-navigation-hub)
8. [docs/INDEX.md Structure](#8-docsindexmd-structure)
9. [Content Migration Plan](#9-content-migration-plan)
10. [Quality Metrics](#10-quality-metrics)

---

## 1. Current State Analysis

### 1.1 Existing Documentation Inventory

**Root Level**:
- `README.md` (485 lines) - Mixed content: quickstart, architecture, examples, philosophy
- `AUTONOMIC.md` (346 lines) - Autonomic CLI layer explanation
- `CONTRIBUTING.md`, `CHANGELOG.md`, `LICENSE.md`

**docs/ Directory** (100+ files):
- **Tutorial-like**: `QUICKSTART.md` (466 lines), `tutorial/quickstart.md`
- **Reference-like**: `CLI_REFERENCE.md` (932 lines), `ARG_ATTRIBUTES.md`
- **How-To-like**: `CLI_COOKBOOK.md`, `CLI_TROUBLESHOOTING.md`
- **Explanation-like**: `SEMANTIC_CLI_ARCHITECTURE.md`, `ARCHITECTURE_V5_COMPLETE.md`
- **Mixed/Legacy**: `book/` (80+ planning docs), `architecture/`, `hive-mind-swarm/`

### 1.2 Content Gaps Identified

**Tutorial Gaps**:
- ❌ No "Your First CLI in 5 Minutes" for absolute beginners
- ❌ No progressive tutorial series (Basic → Intermediate → Advanced)
- ❌ No hands-on exercises with solutions
- ❌ QUICKSTART.md assumes prior Rust/CLI knowledge

**How-To Gaps**:
- ❌ No production deployment patterns
- ❌ No testing strategies guide (Chicago TDD mentioned but not explained)
- ❌ No error handling cookbook
- ❌ No MCP server integration guide

**Reference Gaps**:
- ❌ No complete type catalog (all public types and traits)
- ❌ No macro syntax reference
- ❌ No error type catalog
- ❌ CLI_REFERENCE.md is v4.0.2 (not v5.1.1)

**Explanation Gaps**:
- ❌ No "Domain Separation Architecture" deep dive
- ❌ No "Agent2028" design rationale
- ❌ No RDF/SPARQL decision context
- ❌ Autonomic layer "why" is scattered across multiple docs

### 1.3 Content Redundancies

**Duplicated Content**:
- Domain separation principle: README.md + ARCHITECTURE_V5_COMPLETE.md
- Autonomic CLI: README.md + AUTONOMIC.md + SEMANTIC_CLI_ARCHITECTURE.md
- Quickstart content: README.md + QUICKSTART.md + tutorial/quickstart.md

**Inconsistent Versions**:
- README.md claims v5.0.0 features
- CLI_REFERENCE.md is v4.0.2
- Some docs reference v4 syntax, others v5

---

## 2. Diataxis Directory Structure

### 2.1 Complete Directory Tree

```
clap-noun-verb/
├── README.md                          # Navigation hub (Diataxis entry point)
├── CONTRIBUTING.md                     # Contributor guide
├── CHANGELOG.md                        # Version history
├── LICENSE.md                          # License
├── AUTONOMIC.md                        # (Legacy - merge into docs/explanation/)
│
├── docs/
│   ├── INDEX.md                       # Master documentation map
│   │
│   ├── tutorial/                      # 🎓 Learning-Oriented
│   │   ├── README.md                  # Tutorial quadrant overview
│   │   ├── 01-your-first-cli.md       # 5-minute hello world
│   │   ├── 02-domain-separation.md    # Domain-separated architecture
│   │   ├── 03-adding-commands.md      # Multi-command CLIs
│   │   ├── 04-testing-basics.md       # Chicago TDD fundamentals
│   │   ├── 05-output-formats.md       # JSON/YAML/Table outputs
│   │   ├── 06-autonomic-features.md   # Machine-grade introspection
│   │   ├── 07-async-operations.md     # Async command handlers
│   │   ├── 08-error-handling.md       # Result<T,E> patterns
│   │   ├── 09-deployment-basics.md    # Basic deployment
│   │   └── 10-next-steps.md           # Paths to mastery
│   │
│   ├── howto/                         # 📘 Problem-Solving
│   │   ├── README.md                  # How-to quadrant overview
│   │   ├── production/                # Production patterns
│   │   │   ├── deployment.md          # Deploy to prod
│   │   │   ├── monitoring.md          # OTEL integration
│   │   │   ├── configuration.md       # Config management
│   │   │   └── security.md            # Security hardening
│   │   ├── testing/                   # Testing strategies
│   │   │   ├── chicago-tdd.md         # Chicago TDD in Rust
│   │   │   ├── integration-tests.md   # Integration testing
│   │   │   ├── property-tests.md      # Property-based testing
│   │   │   └── snapshot-tests.md      # Snapshot testing
│   │   ├── integration/               # Integration patterns
│   │   │   ├── mcp-servers.md         # MCP server setup
│   │   │   ├── rdf-sparql.md          # RDF/SPARQL integration
│   │   │   ├── async-io.md            # Async I/O patterns
│   │   │   └── databases.md           # Database connections
│   │   ├── patterns/                  # Common patterns
│   │   │   ├── argument-parsing.md    # Complex argument patterns
│   │   │   ├── error-recovery.md      # Error handling strategies
│   │   │   ├── output-formatting.md   # Custom output formats
│   │   │   └── context-sharing.md     # AppContext patterns
│   │   └── troubleshooting/           # Problem resolution
│   │       ├── common-errors.md       # Common compilation errors
│   │       ├── runtime-issues.md      # Runtime debugging
│   │       └── performance.md         # Performance tuning
│   │
│   ├── reference/                     # 📚 Information-Oriented
│   │   ├── README.md                  # Reference quadrant overview
│   │   ├── api/                       # API reference
│   │   │   ├── overview.md            # API structure overview
│   │   │   ├── verb-macro.md          # #[verb] macro reference
│   │   │   ├── arg-attributes.md      # #[arg(...)] attributes
│   │   │   ├── types.md               # Core types catalog
│   │   │   ├── traits.md              # Trait reference
│   │   │   └── errors.md              # Error type catalog
│   │   ├── autonomic/                 # Autonomic layer API
│   │   │   ├── introspection.md       # --capabilities, --introspect
│   │   │   ├── effects.md             # Effect metadata
│   │   │   ├── planes.md              # O/Σ/Q/ΔΣ planes
│   │   │   ├── guards.md              # Guards & budgets
│   │   │   └── receipts.md            # Execution receipts
│   │   ├── rdf/                       # RDF/SPARQL API
│   │   │   ├── ontology.md            # CLI ontology reference
│   │   │   ├── sparql-queries.md      # SPARQL query patterns
│   │   │   └── shacl-shapes.md        # SHACL validation
│   │   ├── cli-commands.md            # All CLI flags/options
│   │   ├── environment-vars.md        # Environment variable reference
│   │   └── configuration.md           # Config file reference
│   │
│   ├── explanation/                   # 💡 Understanding-Oriented
│   │   ├── README.md                  # Explanation quadrant overview
│   │   ├── architecture/              # Architecture philosophy
│   │   │   ├── domain-separation.md   # Why domain-first design
│   │   │   ├── type-first-thinking.md # Type-driven development
│   │   │   ├── zero-cost-abstractions.md # Performance philosophy
│   │   │   └── chicago-tdd.md         # Chicago TDD rationale
│   │   ├── autonomic/                 # Autonomic layer design
│   │   │   ├── machine-grade-cli.md   # Machine-first interfaces
│   │   │   ├── mape-k-loops.md        # MAPE-K integration
│   │   │   ├── agent2028.md           # Agent2028 vision
│   │   │   └── determinism.md         # Deterministic execution
│   │   ├── semantic/                  # Semantic CLI design
│   │   │   ├── rdf-rationale.md       # Why RDF for CLIs
│   │   │   ├── sparql-benefits.md     # SPARQL query advantages
│   │   │   └── ontology-design.md     # Ontology principles
│   │   ├── comparisons/               # Framework comparisons
│   │   │   ├── vs-clap.md             # clap-noun-verb vs pure clap
│   │   │   ├── vs-typer.md            # Rust vs Python Typer
│   │   │   └── vs-cobra.md            # Comparison with Go Cobra
│   │   └── roadmap.md                 # Future direction
│   │
│   ├── examples/                      # Code examples (organized by category)
│   │   ├── README.md                  # Examples index
│   │   ├── basic/                     # Basic examples
│   │   ├── autonomic/                 # Autonomic features
│   │   └── rdf/                       # RDF/SPARQL examples
│   │
│   └── archive/                       # Legacy documentation
│       ├── v4/                        # v4 documentation
│       ├── book/                      # Old mdBook content
│       └── planning/                  # Planning artifacts
│
├── examples/                          # Runnable example code
│   ├── basic.rs
│   ├── autonomic_example.rs
│   └── ...
│
└── src/                               # Source code
    └── ...
```

### 2.2 Quadrant Characteristics

| Quadrant | Audience | Goal | Format | Examples |
|----------|----------|------|--------|----------|
| **Tutorial** | Beginners, agents bootstrapping | Learn by doing | Step-by-step, hands-on | "Build your first CLI in 5 minutes" |
| **How-To** | Practitioners | Solve specific problem | Task-focused, recipes | "Deploy CLI to production" |
| **Reference** | All users | Look up information | Exhaustive, precise | "All #[arg] attributes" |
| **Explanation** | Architects, contributors | Understand concepts | Conceptual, design-focused | "Why domain separation matters" |

---

## 3. Tutorial Quadrant (Learning)

### 3.1 docs/tutorial/README.md

**Purpose**: Gateway to learning clap-noun-verb
**Audience**: New users (humans and AI agents)
**Prerequisites**: Basic Rust knowledge

**Content Structure**:
```markdown
# Tutorial: Learn clap-noun-verb

Welcome! This tutorial series takes you from zero to production-ready CLIs.

## Learning Path

### Beginner Track (30 minutes)
1. [Your First CLI](01-your-first-cli.md) - 5 minutes
2. [Domain Separation](02-domain-separation.md) - 10 minutes
3. [Adding Commands](03-adding-commands.md) - 15 minutes

### Intermediate Track (1 hour)
4. [Testing Basics](04-testing-basics.md) - 15 minutes
5. [Output Formats](05-output-formats.md) - 15 minutes
6. [Autonomic Features](06-autonomic-features.md) - 30 minutes

### Advanced Track (1.5 hours)
7. [Async Operations](07-async-operations.md) - 30 minutes
8. [Error Handling](08-error-handling.md) - 30 minutes
9. [Deployment Basics](09-deployment-basics.md) - 30 minutes

### Mastery
10. [Next Steps](10-next-steps.md) - Paths to mastery

## Prerequisites
- Rust 1.74+ installed
- Basic Rust syntax knowledge
- Familiarity with `cargo`

## What You'll Build
By the end, you'll have:
- Production-ready CLI with domain-separated architecture
- Comprehensive Chicago TDD test suite
- Machine-grade introspection (autonomic layer)
- JSON/YAML/Table output formats
- Deployment configuration

## How to Use This Tutorial
- **Sequential**: Follow 1→10 for complete learning
- **Skip ahead**: Jump to topics you need
- **Hands-on**: Type every example (don't copy-paste)
- **Experiments**: Break things and explore
```

### 3.2 Tutorial Files Content Outlines

#### 3.2.1 `01-your-first-cli.md` (5 minutes)

**Learning Objectives**:
- Create new Cargo project
- Add clap-noun-verb dependency
- Define first verb with `#[verb]`
- Run CLI and see JSON output

**Content Structure**:
```markdown
# Your First CLI in 5 Minutes

By the end, you'll have a working CLI that outputs JSON.

## Step 1: Create Project (1 minute)
[cargo new commands]

## Step 2: Add Dependencies (1 minute)
[Cargo.toml snippet]

## Step 3: Write Code (2 minutes)
[Minimal working example with #[verb]]

## Step 4: Run It (1 minute)
[cargo run commands]

## What You Learned
- #[verb] macro basics
- Automatic JSON serialization
- Command discovery

## Next Steps
Learn domain-separated architecture →
```

#### 3.2.2 `02-domain-separation.md` (10 minutes)

**Learning Objectives**:
- Understand CLI vs domain logic separation
- Create separate domain module
- Test domain logic independently
- Keep CLI layer thin

**Content Structure**:
```markdown
# Domain-Separated Architecture

The Golden Rule: CLI validates, domain computes.

## The Problem
[Anti-pattern: business logic in CLI handlers]

## The Solution
[Show 3-layer architecture diagram]

## Hands-On Example (8 minutes)
[Refactor previous example into domain/cli split]

## Testing Domain Logic
[Chicago TDD tests for domain, not CLI]

## What You Learned
- 3-layer architecture
- Domain purity
- Testability benefits

## Next Steps
Add more commands →
```

#### 3.2.3 `03-adding-commands.md` (15 minutes)

**Learning Objectives**:
- Add multiple verbs to single noun
- Create multi-noun CLI
- Use file-based noun inference
- Add command arguments

**Content Structure**:
```markdown
# Adding Multiple Commands

Build multi-command CLI with nouns and verbs.

## Single Noun, Multiple Verbs (5 min)
[services.rs with status/restart/logs verbs]

## Multiple Nouns (5 min)
[Add database.rs, config.rs modules]

## Adding Arguments (5 min)
[Arguments with #[arg] attributes]

## Running Your CLI
[cargo run examples]

## What You Learned
- Multi-command organization
- File-based noun inference
- Argument parsing

## Next Steps
Add tests →
```

#### 3.2.4 `04-testing-basics.md` (15 minutes)

**Learning Objectives**:
- Write Chicago TDD tests
- Test domain logic (not CLI parsing)
- Use AAA pattern (Arrange-Act-Assert)
- Verify behavior, not implementation

**Content Structure**:
```markdown
# Testing with Chicago TDD

Test-Driven Development: Write tests first.

## Chicago vs London School
[Comparison table]

## Writing Your First Test (5 min)
[Domain function test with AAA pattern]

## Testing CLI Integration (5 min)
[Integration test for command execution]

## Running Tests (5 min)
[cargo make test commands]

## What You Learned
- Chicago TDD principles
- AAA pattern
- Domain testing

## Next Steps
Add output formats →
```

#### 3.2.5 `05-output-formats.md` (15 minutes)
#### 3.2.6 `06-autonomic-features.md` (30 minutes)
#### 3.2.7 `07-async-operations.md` (30 minutes)
#### 3.2.8 `08-error-handling.md` (30 minutes)
#### 3.2.9 `09-deployment-basics.md` (30 minutes)
#### 3.2.10 `10-next-steps.md` (Reference to other quadrants)

---

## 4. How-To Quadrant (Problem-Solving)

### 4.1 docs/howto/README.md

**Purpose**: Task-focused guides for specific problems
**Audience**: Practitioners implementing features
**Prerequisites**: Completed tutorial or equivalent knowledge

**Content Structure**:
```markdown
# How-To Guides

Production-ready recipes for common tasks.

## Production Patterns
- [Deploy to Production](production/deployment.md)
- [Add Monitoring](production/monitoring.md)
- [Configure Environments](production/configuration.md)
- [Security Hardening](production/security.md)

## Testing Strategies
- [Chicago TDD in Rust](testing/chicago-tdd.md)
- [Integration Testing](testing/integration-tests.md)
- [Property-Based Testing](testing/property-tests.md)
- [Snapshot Testing](testing/snapshot-tests.md)

## Integration Patterns
- [MCP Server Setup](integration/mcp-servers.md)
- [RDF/SPARQL Integration](integration/rdf-sparql.md)
- [Async I/O Patterns](integration/async-io.md)
- [Database Connections](integration/databases.md)

## Common Patterns
- [Complex Arguments](patterns/argument-parsing.md)
- [Error Recovery](patterns/error-recovery.md)
- [Custom Outputs](patterns/output-formatting.md)
- [Context Sharing](patterns/context-sharing.md)

## Troubleshooting
- [Common Errors](troubleshooting/common-errors.md)
- [Runtime Issues](troubleshooting/runtime-issues.md)
- [Performance Tuning](troubleshooting/performance.md)
```

### 4.2 How-To Files Content Outlines

#### 4.2.1 `production/deployment.md`

**Problem**: Deploy CLI to production
**Solution**: Docker containerization + GitHub Actions CI/CD

**Content Structure**:
```markdown
# Deploy CLI to Production

Deploy clap-noun-verb CLI with Docker and CI/CD.

## Problem
You have a working CLI locally, need production deployment.

## Solution Overview
Docker + GitHub Actions + Release automation

## Prerequisites
- Working CLI with tests
- GitHub repository
- Docker installed

## Step 1: Create Dockerfile (5 min)
[Multi-stage Dockerfile for Rust]

## Step 2: GitHub Actions Workflow (10 min)
[CI/CD pipeline YAML]

## Step 3: Release Automation (5 min)
[cargo-release configuration]

## Step 4: Deploy (5 min)
[Deploy commands]

## Verification
[Test deployed CLI]

## Troubleshooting
Common issues and solutions

## Related
- [Configuration](configuration.md)
- [Monitoring](monitoring.md)
```

#### 4.2.2 `testing/chicago-tdd.md`

**Problem**: Implement Chicago TDD in Rust CLI project
**Solution**: State-based testing with real collaborators

**Content Structure**:
```markdown
# Chicago TDD in Rust

Implement state-based TDD for CLI projects.

## Problem
Need testable CLI without mocks.

## Solution
Chicago School: Test observable outputs, use real objects.

## Chicago vs London
[Detailed comparison]

## Pattern: AAA (Arrange-Act-Assert)
[Example with domain function]

## Pattern: Real Collaborators
[Example avoiding mocks]

## Pattern: State Verification
[Test state changes, not calls]

## Example: Service Manager
[Complete example with tests]

## Common Pitfalls
- Testing implementation details
- Over-mocking
- Meaningless assertions

## Related
- [Integration Tests](integration-tests.md)
```

#### 4.2.3 `integration/mcp-servers.md`

**Problem**: Integrate CLI with Model Context Protocol servers
**Solution**: Use rmcp crate for MCP integration

**Content Structure**:
```markdown
# MCP Server Integration

Add MCP capabilities to your CLI.

## Problem
Need AI agents to discover and use CLI commands.

## Solution
MCP server exposing CLI as MCP tools.

## Prerequisites
- Working CLI with autonomic layer
- rmcp dependency

## Step 1: Define MCP Tools (10 min)
[Map verbs to MCP tool definitions]

## Step 2: Implement Tool Handlers (15 min)
[MCP tool handler code]

## Step 3: Start MCP Server (5 min)
[Server startup code]

## Step 4: Test with Claude Code (10 min)
[Test CLI via MCP]

## Complete Example
[Full working MCP server example]

## Related
- [RDF Integration](rdf-sparql.md)
- [Autonomic Features](../tutorial/06-autonomic-features.md)
```

#### 4.2.4 Additional How-To Files
[Similar detailed outlines for remaining how-to guides]

---

## 5. Reference Quadrant (Information)

### 5.1 docs/reference/README.md

**Purpose**: Exhaustive API documentation
**Audience**: All users looking up specific information
**Prerequisites**: None (designed for quick lookups)

**Content Structure**:
```markdown
# API Reference

Complete reference documentation for clap-noun-verb v5.1.1.

## Core API
- [API Overview](api/overview.md) - Architecture and modules
- [#[verb] Macro](api/verb-macro.md) - Verb registration
- [#[arg] Attributes](api/arg-attributes.md) - Argument configuration
- [Types](api/types.md) - All public types
- [Traits](api/traits.md) - All public traits
- [Errors](api/errors.md) - Error type catalog

## Autonomic Layer API
- [Introspection](autonomic/introspection.md) - --capabilities, --introspect
- [Effects](autonomic/effects.md) - Effect metadata
- [Planes](autonomic/planes.md) - O/Σ/Q/ΔΣ planes
- [Guards](autonomic/guards.md) - Resource budgets
- [Receipts](autonomic/receipts.md) - Execution receipts

## RDF/SPARQL API
- [Ontology](rdf/ontology.md) - CLI ontology reference
- [SPARQL Queries](rdf/sparql-queries.md) - Query patterns
- [SHACL Shapes](rdf/shacl-shapes.md) - Validation shapes

## CLI Reference
- [CLI Commands](cli-commands.md) - All flags and options
- [Environment Variables](environment-vars.md) - Env var reference
- [Configuration](configuration.md) - Config file format

## Quick Lookup Tables
- [All Attributes](quick-ref-attributes.md)
- [All Types](quick-ref-types.md)
- [All Errors](quick-ref-errors.md)
```

### 5.2 Reference Files Content Outlines

#### 5.2.1 `api/verb-macro.md`

**Purpose**: Complete `#[verb]` macro syntax reference

**Content Structure**:
```markdown
# #[verb] Macro Reference

Complete syntax reference for verb registration.

## Syntax

### Auto-Inferred
`#[verb]`

### Custom Verb Name
`#[verb("custom-name")]`

### Explicit Noun
`#[verb("verb", "noun")]`

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| verb_name | &str | No | Custom verb name (default: function name) |
| noun_name | &str | No | Explicit noun (default: filename) |

## Function Requirements

### Return Type
Must return `Result<T>` where `T: Serialize`

### Arguments
Function arguments become CLI arguments:
- `String` → Required positional/flag
- `Option<T>` → Optional argument
- `bool` → Boolean flag
- `Vec<T>` → Multiple values

## Examples

### Basic
[Code example]

### With Arguments
[Code example]

### Custom Names
[Code example]

## Related
- [#[arg] Attributes](arg-attributes.md)
- [Types](types.md)
```

#### 5.2.2 `api/arg-attributes.md`

**Purpose**: Exhaustive `#[arg(...)]` attribute catalog

**Content Structure**:
```markdown
# #[arg] Attributes Reference

Complete catalog of argument attributes.

## Attribute Index

### Core Attributes
- [short](#short) - Short flag (-s)
- [long](#long) - Long flag (--long)
- [default_value](#default_value) - Default value
- [env](#env) - Environment variable

### Validation
- [required](#required) - Required argument
- [value_name](#value_name) - Value placeholder
- [multiple](#multiple) - Accept multiple values

### Relationships
- [group](#group) - Argument group
- [requires](#requires) - Dependency
- [conflicts_with](#conflicts_with) - Conflict

### Display
- [help](#help) - Help text
- [long_help](#long_help) - Extended help
- [hide](#hide) - Hide from help
- [display_order](#display_order) - Display order

### Actions
- [action](#action) - Argument action (count, append, etc.)

## Detailed Reference

### `short`
**Syntax**: `#[arg(short = 'c')]`
**Type**: `char`
**Description**: Single-character flag

**Examples**:
[Code examples]

**Related**: [long](#long)

---

[Continue for all attributes...]

## Attribute Combinations

Common patterns:
[Example combinations]

## Related
- [#[verb] Macro](verb-macro.md)
- [Types](types.md)
```

#### 5.2.3 `api/types.md`

**Purpose**: Catalog of all public types

**Content Structure**:
```markdown
# Type Reference

Complete catalog of clap-noun-verb types.

## Core Types

### VerbCommand
[Detailed documentation]

### CommandRegistry
[Detailed documentation]

### VerbArgs
[Detailed documentation]

## Autonomic Types

### CommandMetadata
[Detailed documentation]

### EffectMetadata
[Detailed documentation]

### PlaneInteraction
[Detailed documentation]

### GuardConfig
[Detailed documentation]

### ExecutionReceipt
[Detailed documentation]

## RDF Types

### CliOntology
[Detailed documentation]

### SparqlQuery
[Detailed documentation]

## Type Relationships

[Diagram showing type relationships]

## Related
- [Traits](traits.md)
- [Errors](errors.md)
```

#### 5.2.4 Additional Reference Files
[Detailed outlines for all reference documentation]

---

## 6. Explanation Quadrant (Understanding)

### 6.1 docs/explanation/README.md

**Purpose**: Conceptual understanding and design rationale
**Audience**: Architects, contributors, deep learners
**Prerequisites**: Completed tutorial and some how-to guides

**Content Structure**:
```markdown
# Explanation: Understanding clap-noun-verb

Deep dives into architecture, philosophy, and design decisions.

## Architecture Philosophy
- [Domain Separation](architecture/domain-separation.md) - Why CLI ≠ Application
- [Type-First Thinking](architecture/type-first-thinking.md) - Types encode invariants
- [Zero-Cost Abstractions](architecture/zero-cost-abstractions.md) - Performance philosophy
- [Chicago TDD](architecture/chicago-tdd.md) - Testing philosophy

## Autonomic Layer Design
- [Machine-Grade CLIs](autonomic/machine-grade-cli.md) - Why agents need different interfaces
- [MAPE-K Loops](autonomic/mape-k-loops.md) - Autonomic computing integration
- [Agent2028](autonomic/agent2028.md) - Trillion-agent ecosystems
- [Determinism](autonomic/determinism.md) - Deterministic execution

## Semantic CLI Design
- [RDF Rationale](semantic/rdf-rationale.md) - Why RDF for CLIs
- [SPARQL Benefits](semantic/sparql-benefits.md) - Query advantages
- [Ontology Design](semantic/ontology-design.md) - Ontology principles

## Framework Comparisons
- [vs. Pure Clap](comparisons/vs-clap.md) - What clap-noun-verb adds
- [vs. Python Typer](comparisons/vs-typer.md) - Rust vs Python
- [vs. Go Cobra](comparisons/vs-cobra.md) - Comparison with Cobra

## Future Direction
- [Roadmap](roadmap.md) - v5.2+ plans
```

### 6.2 Explanation Files Content Outlines

#### 6.2.1 `architecture/domain-separation.md`

**Purpose**: Explain why domain-first architecture matters

**Content Structure**:
```markdown
# Domain Separation Architecture

Why your CLI should not be your application.

## The Problem

Most CLIs mix concerns:
[Anti-pattern example with business logic in CLI]

**Consequences**:
- Untestable business logic
- Duplication across commands
- Cannot reuse logic outside CLI
- Brittle to UI changes

## The Solution: 3-Layer Architecture

[Diagram: CLI → Integration → Domain]

**Layer 1: CLI (Thin)**
- Argument parsing
- Validation
- Output formatting
- NO business logic

**Layer 2: Integration (Glue)**
- Maps CLI to domain
- Converts types
- Handles I/O

**Layer 3: Domain (Pure)**
- Business logic
- Pure functions
- Testable
- Reusable

## Benefits

### Testability
[Example: Testing domain vs CLI]

### Reusability
[Example: Using domain from CLI, API, WASM]

### Maintainability
[Example: Changing CLI without changing logic]

## Real-World Example

[Complete example: Service manager with 3 layers]

## Trade-Offs

**Pros**:
- Testable
- Maintainable
- Reusable

**Cons**:
- More files
- Type conversions
- Initial overhead

**When to Use**:
- ✅ Non-trivial business logic
- ✅ Multiple interfaces planned
- ✅ Team projects
- ❌ Simple scripts (<100 LOC)

## Related Patterns

- [Type-First Thinking](type-first-thinking.md)
- [Chicago TDD](chicago-tdd.md)

## Further Reading

- Hexagonal Architecture (Alistair Cockburn)
- Clean Architecture (Robert Martin)
```

#### 6.2.2 `autonomic/machine-grade-cli.md`

**Purpose**: Explain machine-first CLI design

**Content Structure**:
```markdown
# Machine-Grade CLIs

Why AI agents need different interfaces than humans.

## The Problem

Traditional CLIs are human-first:
- Help text for reading
- Flags for typing
- Errors for debugging

**But agents need**:
- Structured introspection (JSON)
- Effect metadata (safety analysis)
- Execution receipts (audit logs)
- Deterministic behavior

## Machine-Grade Principles

### 1. Introspectable
[Explain --capabilities, --introspect]

### 2. Analyzable
[Explain effect metadata, guards]

### 3. Observable
[Explain execution receipts]

### 4. Deterministic
[Explain deterministic execution]

## Real-World Use Cases

### MAPE-K Loops
[Example: Autonomic controller using introspection]

### Agent Orchestration
[Example: Multi-agent CLI coordination]

### Audit & Compliance
[Example: Execution receipts for compliance]

## Design Evolution

**v4**: Human-first CLI
**v5.0**: + Autonomic layer (introspection)
**v5.1**: + Agent2028 (delegation chains)
**v5.2 (planned)**: + Distributed execution

## Related

- [MAPE-K Loops](mape-k-loops.md)
- [Agent2028](agent2028.md)
- [RDF Rationale](../semantic/rdf-rationale.md)
```

#### 6.2.3 Additional Explanation Files
[Detailed outlines for all explanation documentation]

---

## 7. README.md as Navigation Hub

### 7.1 New README.md Structure

**Purpose**: Entry point for all users → direct to appropriate quadrant
**Length**: ~300 lines (down from 485)
**Tone**: Welcoming, clear, actionable

**Content Structure**:

```markdown
# clap-noun-verb

**Machine-grade CLI framework for AI agents and autonomous systems**

> ⚠️ **Architecture First:** This framework enforces domain logic separation.
> Your CLI is an interface, not your application.

[![Crates.io](https://img.shields.io/crates/v/clap-noun-verb.svg)](https://crates.io/crates/clap-noun-verb)
[![Documentation](https://docs.rs/clap-noun-verb/badge.svg)](https://docs.rs/clap-noun-verb)
[![License](https://img.shields.io/crates/l/clap-noun-verb.svg)](LICENSE.md)

---

## Quick Navigation (Diataxis)

### 🎓 [Tutorial](docs/tutorial/) - Get Started in 30 Minutes
**For:** Beginners, agents bootstrapping new CLIs
**Start here:** [Your First CLI in 5 Minutes](docs/tutorial/01-your-first-cli.md)

### 📘 [How-To Guides](docs/howto/) - Production Patterns
**For:** Practitioners solving specific problems
**Popular guides:**
- [Deploy to Production](docs/howto/production/deployment.md)
- [Chicago TDD in Rust](docs/howto/testing/chicago-tdd.md)
- [MCP Server Integration](docs/howto/integration/mcp-servers.md)

### 📚 [Reference](docs/reference/) - Complete API Catalog
**For:** Quick lookups and exhaustive documentation
**Key references:**
- [#[verb] Macro](docs/reference/api/verb-macro.md)
- [#[arg] Attributes](docs/reference/api/arg-attributes.md)
- [Autonomic API](docs/reference/autonomic/)

### 💡 [Explanation](docs/explanation/) - Architecture & Philosophy
**For:** Understanding the "why" behind design decisions
**Deep dives:**
- [Domain Separation Architecture](docs/explanation/architecture/domain-separation.md)
- [Machine-Grade CLIs](docs/explanation/autonomic/machine-grade-cli.md)
- [RDF Rationale](docs/explanation/semantic/rdf-rationale.md)

**📖 Full Documentation Map:** [docs/INDEX.md](docs/INDEX.md)

---

## 30-Second Example

```rust
// domain/calculator.rs - Pure business logic
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}

// cli/commands.rs - Thin CLI wrapper
use clap_noun_verb_macros::verb;

#[verb]
fn add(x: i32, y: i32) -> Result<AddResult> {
    let result = crate::domain::calculator::add(x, y);
    Ok(AddResult { result })
}
```

**Run it:**
```bash
myapp add --x 2 --y 3
{"result": 5}
```

**[Full Tutorial →](docs/tutorial/01-your-first-cli.md)**

---

## Installation

```toml
[dependencies]
clap-noun-verb = "5.1.1"
clap-noun-verb-macros = "5.1.1"
```

---

## Why clap-noun-verb?

### For Humans
- **Intuitive structure:** `noun verb` pattern (e.g., `services status`)
- **Zero boilerplate:** `#[verb]` macro does the work
- **Better errors:** Type-safe validation

### For AI Agents
- **Machine-readable:** JSON output by default
- **Introspectable:** `--capabilities`, `--introspect` flags
- **Semantic:** RDF/SPARQL layer for intent-based discovery
- **Autonomous:** MAPE-K loop integration

### For Developers
- **Type-first:** Encode invariants in types
- **Zero-cost:** No runtime overhead
- **Domain-separated:** CLI validates, domain computes
- **Production-ready:** Chicago TDD, comprehensive testing

**[Learn More →](docs/explanation/)**

---

## v5.1.1 Highlights

**Release Quality:** ✅ **PRODUCTION READY**
- **Documentation Quality:** 76% risk reduction (RPN: 4,848 → 1,152)
- **Test Coverage:** 100% pass rate on validation suite
- **Examples:** 100% compilation (25+ examples)

**Key Features:**
- ✅ **Autonomic CLI Layer** - Machine-grade introspection
- ✅ **MCP Integration** - Native Model Context Protocol support
- ✅ **Agent2028** - Cryptographic receipts for trillion-agent ecosystems
- ✅ **Type-Safe & Zero-Cost** - Compile-time validation

**[Full Changelog →](CHANGELOG.md)**

---

## Examples

```bash
# Basic noun-verb pattern
cargo run --example basic -- services status

# Autonomic CLI features
cargo run --example autonomic_example -- --capabilities
cargo run --example autonomic_example -- --introspect

# See all examples
ls examples/
```

**[Example Code →](examples/)** | **[Example Tutorials →](docs/tutorial/)**

---

## Community & Contributing

- **Repository:** [github.com/seanchatmangpt/clap-noun-verb](https://github.com/seanchatmangpt/clap-noun-verb)
- **Issues:** [Report bugs](https://github.com/seanchatmangpt/clap-noun-verb/issues)
- **Discussions:** [Ask questions](https://github.com/seanchatmangpt/clap-noun-verb/discussions)
- **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)

---

## License

MIT OR Apache-2.0

---

## Acknowledgments

Built with ❤️ by the Rust community.
Inspired by [Python Typer](https://typer.tiangolo.com/) | Built on [clap](https://crates.io/crates/clap)
```

---

## 8. docs/INDEX.md Structure

### 8.1 Complete Documentation Map

**Purpose**: Master index for all documentation
**Audience**: All users navigating documentation
**Format**: Hierarchical with descriptions

**Content Structure**:

```markdown
# clap-noun-verb Documentation Index

Complete documentation map for v5.1.1.

---

## Diataxis Navigation

### 🎓 Tutorial (Learning-Oriented)
**Start here if you're new to clap-noun-verb**

| Document | Time | Description |
|----------|------|-------------|
| [Your First CLI](tutorial/01-your-first-cli.md) | 5 min | Hello World in clap-noun-verb |
| [Domain Separation](tutorial/02-domain-separation.md) | 10 min | Separate CLI from business logic |
| [Adding Commands](tutorial/03-adding-commands.md) | 15 min | Multi-command CLIs |
| [Testing Basics](tutorial/04-testing-basics.md) | 15 min | Chicago TDD fundamentals |
| [Output Formats](tutorial/05-output-formats.md) | 15 min | JSON/YAML/Table outputs |
| [Autonomic Features](tutorial/06-autonomic-features.md) | 30 min | Machine-grade introspection |
| [Async Operations](tutorial/07-async-operations.md) | 30 min | Async command handlers |
| [Error Handling](tutorial/08-error-handling.md) | 30 min | Result<T,E> patterns |
| [Deployment Basics](tutorial/09-deployment-basics.md) | 30 min | Basic deployment |
| [Next Steps](tutorial/10-next-steps.md) | 5 min | Paths to mastery |

**Total Learning Time:** ~3 hours for complete tutorial series

---

### 📘 How-To Guides (Problem-Solving)
**Use when you have a specific task to accomplish**

#### Production Patterns
- [Deploy to Production](howto/production/deployment.md)
- [Add Monitoring](howto/production/monitoring.md)
- [Configure Environments](howto/production/configuration.md)
- [Security Hardening](howto/production/security.md)

#### Testing Strategies
- [Chicago TDD in Rust](howto/testing/chicago-tdd.md)
- [Integration Testing](howto/testing/integration-tests.md)
- [Property-Based Testing](howto/testing/property-tests.md)
- [Snapshot Testing](howto/testing/snapshot-tests.md)

#### Integration Patterns
- [MCP Server Setup](howto/integration/mcp-servers.md)
- [RDF/SPARQL Integration](howto/integration/rdf-sparql.md)
- [Async I/O Patterns](howto/integration/async-io.md)
- [Database Connections](howto/integration/databases.md)

#### Common Patterns
- [Complex Arguments](howto/patterns/argument-parsing.md)
- [Error Recovery](howto/patterns/error-recovery.md)
- [Custom Outputs](howto/patterns/output-formatting.md)
- [Context Sharing](howto/patterns/context-sharing.md)

#### Troubleshooting
- [Common Errors](howto/troubleshooting/common-errors.md)
- [Runtime Issues](howto/troubleshooting/runtime-issues.md)
- [Performance Tuning](howto/troubleshooting/performance.md)

---

### 📚 Reference (Information-Oriented)
**Use for quick lookups and exhaustive documentation**

#### Core API
- [API Overview](reference/api/overview.md)
- [#[verb] Macro](reference/api/verb-macro.md)
- [#[arg] Attributes](reference/api/arg-attributes.md)
- [Types Catalog](reference/api/types.md)
- [Traits Reference](reference/api/traits.md)
- [Error Types](reference/api/errors.md)

#### Autonomic Layer
- [Introspection](reference/autonomic/introspection.md)
- [Effects](reference/autonomic/effects.md)
- [Planes (O/Σ/Q/ΔΣ)](reference/autonomic/planes.md)
- [Guards & Budgets](reference/autonomic/guards.md)
- [Execution Receipts](reference/autonomic/receipts.md)

#### RDF/SPARQL
- [CLI Ontology](reference/rdf/ontology.md)
- [SPARQL Queries](reference/rdf/sparql-queries.md)
- [SHACL Shapes](reference/rdf/shacl-shapes.md)

#### CLI Reference
- [CLI Commands](reference/cli-commands.md)
- [Environment Variables](reference/environment-vars.md)
- [Configuration Files](reference/configuration.md)

---

### 💡 Explanation (Understanding-Oriented)
**Use to understand concepts and design decisions**

#### Architecture Philosophy
- [Domain Separation](explanation/architecture/domain-separation.md)
- [Type-First Thinking](explanation/architecture/type-first-thinking.md)
- [Zero-Cost Abstractions](explanation/architecture/zero-cost-abstractions.md)
- [Chicago TDD](explanation/architecture/chicago-tdd.md)

#### Autonomic Layer Design
- [Machine-Grade CLIs](explanation/autonomic/machine-grade-cli.md)
- [MAPE-K Loops](explanation/autonomic/mape-k-loops.md)
- [Agent2028 Vision](explanation/autonomic/agent2028.md)
- [Deterministic Execution](explanation/autonomic/determinism.md)

#### Semantic CLI Design
- [RDF Rationale](explanation/semantic/rdf-rationale.md)
- [SPARQL Benefits](explanation/semantic/sparql-benefits.md)
- [Ontology Design](explanation/semantic/ontology-design.md)

#### Framework Comparisons
- [vs. Pure Clap](explanation/comparisons/vs-clap.md)
- [vs. Python Typer](explanation/comparisons/vs-typer.md)
- [vs. Go Cobra](explanation/comparisons/vs-cobra.md)

#### Future Direction
- [Roadmap v5.2+](explanation/roadmap.md)

---

## Quick Links

### Getting Started
1. [Install](tutorial/01-your-first-cli.md#installation)
2. [Your First CLI](tutorial/01-your-first-cli.md)
3. [Domain Separation](tutorial/02-domain-separation.md)

### Common Tasks
- [Add command](tutorial/03-adding-commands.md)
- [Parse arguments](reference/api/arg-attributes.md)
- [Handle errors](tutorial/08-error-handling.md)
- [Deploy to prod](howto/production/deployment.md)

### API Lookups
- [#[verb] syntax](reference/api/verb-macro.md)
- [#[arg] attributes](reference/api/arg-attributes.md)
- [All types](reference/api/types.md)
- [All errors](reference/api/errors.md)

### Understanding
- [Why domain separation](explanation/architecture/domain-separation.md)
- [Why machine-grade](explanation/autonomic/machine-grade-cli.md)
- [Why RDF](explanation/semantic/rdf-rationale.md)

---

## Documentation by Version

- **v5.1.1** (current) - This documentation
- **v5.0.0** - [archive/v5.0/](archive/v5.0/)
- **v4.0.2** - [archive/v4/](archive/v4/)

---

## Contributing to Documentation

See [CONTRIBUTING.md](../CONTRIBUTING.md) for documentation guidelines.

**Documentation follows Diataxis principles:**
- **Tutorial**: Learning by doing
- **How-To**: Task-focused solutions
- **Reference**: Exhaustive information
- **Explanation**: Conceptual understanding

---

**Last Updated:** 2025-12-02
**Version:** 5.1.1
```

---

## 9. Content Migration Plan

### 9.1 Migration Strategy

**Principle**: Incremental migration preserving v4 content

**Phases**:
1. **Phase 1: Create Structure** (Week 1)
   - Create Diataxis directory structure
   - Write quadrant README files
   - Create INDEX.md

2. **Phase 2: Migrate Tutorial** (Week 2)
   - Write new tutorial series (01-10)
   - Archive old QUICKSTART.md

3. **Phase 3: Migrate How-To** (Week 3)
   - Extract how-to content from existing docs
   - Create production/testing/integration guides

4. **Phase 4: Migrate Reference** (Week 4)
   - Update CLI_REFERENCE.md to v5.1.1
   - Create API reference structure
   - Document autonomic/RDF APIs

5. **Phase 5: Migrate Explanation** (Week 5)
   - Extract conceptual content from README/AUTONOMIC
   - Write architecture deep dives
   - Create comparison docs

6. **Phase 6: Refactor README** (Week 6)
   - Reduce README to navigation hub
   - Link to Diataxis quadrants

### 9.2 Content Mapping

| Current Location | Content Type | New Location | Action |
|------------------|--------------|--------------|--------|
| README.md (L1-124) | Tutorial-like | tutorial/01-your-first-cli.md | **Extract** |
| README.md (L46-64) | Explanation | explanation/architecture/domain-separation.md | **Extract** |
| README.md (L126-156) | Explanation | explanation/autonomic/machine-grade-cli.md | **Extract** |
| README.md (L165-267) | Explanation | explanation/architecture/type-first-thinking.md | **Extract** |
| README.md (L303-336) | Tutorial | tutorial/01-your-first-cli.md | **Extract** |
| README.md (L398-451) | Reference | explanation/comparisons/vs-clap.md | **Extract** |
| QUICKSTART.md | Tutorial | tutorial/01-05 series | **Migrate & Expand** |
| CLI_REFERENCE.md | Reference | reference/api/ (multiple files) | **Update v4→v5 & Split** |
| AUTONOMIC.md | Explanation + Reference | explanation/autonomic/ + reference/autonomic/ | **Split** |
| SEMANTIC_CLI_ARCHITECTURE.md | Explanation | explanation/semantic/ | **Migrate** |
| docs/ARG_ATTRIBUTES.md | Reference | reference/api/arg-attributes.md | **Migrate** |
| docs/CLI_COOKBOOK.md | How-To | howto/patterns/ | **Migrate** |
| docs/CLI_TROUBLESHOOTING.md | How-To | howto/troubleshooting/ | **Migrate** |

### 9.3 Content Gaps to Fill

**Tutorial (New Content)**:
- ✅ 01-your-first-cli.md (NEW - extract from README examples)
- ✅ 02-domain-separation.md (NEW - expand README L46-64)
- ✅ 03-adding-commands.md (NEW - expand QUICKSTART patterns)
- ✅ 04-testing-basics.md (NEW - Chicago TDD intro)
- ✅ 05-output-formats.md (NEW - JSON/YAML/Table)
- ✅ 06-autonomic-features.md (NEW - --capabilities intro)
- ✅ 07-async-operations.md (NEW - async patterns)
- ✅ 08-error-handling.md (NEW - Result<T,E> guide)
- ✅ 09-deployment-basics.md (NEW - Docker basics)
- ✅ 10-next-steps.md (NEW - navigation to other quadrants)

**How-To (New Content)**:
- ✅ production/deployment.md (NEW - Docker + CI/CD)
- ✅ production/monitoring.md (NEW - OTEL integration)
- ✅ testing/chicago-tdd.md (NEW - Chicago TDD in Rust)
- ✅ integration/mcp-servers.md (NEW - MCP setup)
- ✅ integration/rdf-sparql.md (NEW - RDF integration)
- ⚠️ troubleshooting/* (MIGRATE from CLI_TROUBLESHOOTING.md)

**Reference (Update + New)**:
- ⚠️ api/verb-macro.md (NEW - from CLI_REFERENCE + v5 features)
- ⚠️ api/arg-attributes.md (MIGRATE + UPDATE from ARG_ATTRIBUTES.md)
- ✅ api/types.md (NEW - catalog all types)
- ✅ api/traits.md (NEW - catalog all traits)
- ✅ api/errors.md (NEW - error catalog)
- ⚠️ autonomic/* (EXTRACT from AUTONOMIC.md reference sections)
- ✅ rdf/* (NEW - RDF/SPARQL/SHACL API reference)

**Explanation (New Content)**:
- ⚠️ architecture/domain-separation.md (EXTRACT + EXPAND from README L46-64)
- ⚠️ architecture/type-first-thinking.md (EXTRACT from README L165-267)
- ✅ autonomic/machine-grade-cli.md (EXTRACT from README + AUTONOMIC)
- ✅ autonomic/mape-k-loops.md (NEW - MAPE-K integration)
- ✅ autonomic/agent2028.md (NEW - Agent2028 vision)
- ⚠️ semantic/rdf-rationale.md (EXTRACT from SEMANTIC_CLI_ARCHITECTURE.md)
- ✅ comparisons/vs-clap.md (EXTRACT from README L398-451)
- ✅ comparisons/vs-typer.md (NEW - Rust vs Python comparison)
- ✅ comparisons/vs-cobra.md (NEW - vs Go Cobra)

**Legend**:
- ✅ NEW - Create from scratch
- ⚠️ EXTRACT - Extract from existing docs
- ⚠️ MIGRATE - Move and update existing content
- ⚠️ UPDATE - Update v4 → v5 versions

---

## 10. Quality Metrics

### 10.1 Documentation Quality Targets

**Completeness Metrics**:
- ✅ 100% of public API documented (reference quadrant)
- ✅ 100% of v5 features documented (tutorial + reference)
- ✅ 10 tutorial chapters (complete learning path)
- ✅ 20+ how-to guides (production-ready recipes)

**Clarity Metrics** (Measured via user testing):
- ✅ Tutorial completion rate >80%
- ✅ Time to first working CLI <10 minutes
- ✅ "Find what I need" success rate >90%

**Consistency Metrics**:
- ✅ All code examples compile and run
- ✅ All references to API match v5.1.1
- ✅ Cross-references between quadrants functional

**Maintenance Metrics**:
- ✅ Documentation updated within 1 week of code changes
- ✅ All examples in CI/CD pipeline
- ✅ Broken links checked automatically

### 10.2 Validation Criteria

**Tutorial Validation**:
- [ ] Complete tutorial series (10 chapters)
- [ ] All examples compile and run
- [ ] Progressive difficulty (beginner → advanced)
- [ ] Hands-on exercises with solutions
- [ ] Time estimates accurate (<10% variance)

**How-To Validation**:
- [ ] 20+ task-focused guides
- [ ] All production patterns documented
- [ ] Troubleshooting covers common issues
- [ ] Each guide standalone (no required sequence)

**Reference Validation**:
- [ ] 100% API coverage
- [ ] All v5.1.1 features documented
- [ ] Quick lookup tables functional
- [ ] Cross-references complete

**Explanation Validation**:
- [ ] Architecture rationale explained
- [ ] Design decisions justified
- [ ] Trade-offs documented
- [ ] Comparisons with alternatives

---

## Appendix A: File Creation Checklist

### A.1 Tutorial Files (10 files)
- [ ] docs/tutorial/README.md
- [ ] docs/tutorial/01-your-first-cli.md
- [ ] docs/tutorial/02-domain-separation.md
- [ ] docs/tutorial/03-adding-commands.md
- [ ] docs/tutorial/04-testing-basics.md
- [ ] docs/tutorial/05-output-formats.md
- [ ] docs/tutorial/06-autonomic-features.md
- [ ] docs/tutorial/07-async-operations.md
- [ ] docs/tutorial/08-error-handling.md
- [ ] docs/tutorial/09-deployment-basics.md
- [ ] docs/tutorial/10-next-steps.md

### A.2 How-To Files (20+ files)
- [ ] docs/howto/README.md
- [ ] docs/howto/production/deployment.md
- [ ] docs/howto/production/monitoring.md
- [ ] docs/howto/production/configuration.md
- [ ] docs/howto/production/security.md
- [ ] docs/howto/testing/chicago-tdd.md
- [ ] docs/howto/testing/integration-tests.md
- [ ] docs/howto/testing/property-tests.md
- [ ] docs/howto/testing/snapshot-tests.md
- [ ] docs/howto/integration/mcp-servers.md
- [ ] docs/howto/integration/rdf-sparql.md
- [ ] docs/howto/integration/async-io.md
- [ ] docs/howto/integration/databases.md
- [ ] docs/howto/patterns/argument-parsing.md
- [ ] docs/howto/patterns/error-recovery.md
- [ ] docs/howto/patterns/output-formatting.md
- [ ] docs/howto/patterns/context-sharing.md
- [ ] docs/howto/troubleshooting/common-errors.md
- [ ] docs/howto/troubleshooting/runtime-issues.md
- [ ] docs/howto/troubleshooting/performance.md

### A.3 Reference Files (20+ files)
- [ ] docs/reference/README.md
- [ ] docs/reference/api/overview.md
- [ ] docs/reference/api/verb-macro.md
- [ ] docs/reference/api/arg-attributes.md
- [ ] docs/reference/api/types.md
- [ ] docs/reference/api/traits.md
- [ ] docs/reference/api/errors.md
- [ ] docs/reference/autonomic/introspection.md
- [ ] docs/reference/autonomic/effects.md
- [ ] docs/reference/autonomic/planes.md
- [ ] docs/reference/autonomic/guards.md
- [ ] docs/reference/autonomic/receipts.md
- [ ] docs/reference/rdf/ontology.md
- [ ] docs/reference/rdf/sparql-queries.md
- [ ] docs/reference/rdf/shacl-shapes.md
- [ ] docs/reference/cli-commands.md
- [ ] docs/reference/environment-vars.md
- [ ] docs/reference/configuration.md

### A.4 Explanation Files (15+ files)
- [ ] docs/explanation/README.md
- [ ] docs/explanation/architecture/domain-separation.md
- [ ] docs/explanation/architecture/type-first-thinking.md
- [ ] docs/explanation/architecture/zero-cost-abstractions.md
- [ ] docs/explanation/architecture/chicago-tdd.md
- [ ] docs/explanation/autonomic/machine-grade-cli.md
- [ ] docs/explanation/autonomic/mape-k-loops.md
- [ ] docs/explanation/autonomic/agent2028.md
- [ ] docs/explanation/autonomic/determinism.md
- [ ] docs/explanation/semantic/rdf-rationale.md
- [ ] docs/explanation/semantic/sparql-benefits.md
- [ ] docs/explanation/semantic/ontology-design.md
- [ ] docs/explanation/comparisons/vs-clap.md
- [ ] docs/explanation/comparisons/vs-typer.md
- [ ] docs/explanation/comparisons/vs-cobra.md
- [ ] docs/explanation/roadmap.md

### A.5 Navigation Files (3 files)
- [ ] README.md (refactor to ~300 lines)
- [ ] docs/INDEX.md
- [ ] docs/tutorial/README.md (quadrant overview)
- [ ] docs/howto/README.md (quadrant overview)
- [ ] docs/reference/README.md (quadrant overview)
- [ ] docs/explanation/README.md (quadrant overview)

**Total New/Updated Files**: ~70 files

---

## Appendix B: Memory Storage Keys

Store design decisions in memory for agent coordination:

```bash
# Store directory structure
memory store diataxis/structure "$(cat directory-tree.txt)"

# Store content mapping
memory store diataxis/migration-map "$(cat content-mapping-table.json)"

# Store file creation checklist
memory store diataxis/file-checklist "$(cat file-checklist.json)"

# Store tutorial outline
memory store diataxis/tutorial-outline "$(cat tutorial-outline.json)"

# Store how-to outline
memory store diataxis/howto-outline "$(cat howto-outline.json)"

# Store reference outline
memory store diataxis/reference-outline "$(cat reference-outline.json)"

# Store explanation outline
memory store diataxis/explanation-outline "$(cat explanation-outline.json)"

# Store README refactor
memory store diataxis/readme-structure "$(cat readme-new-structure.md)"
```

---

## Next Steps

### For Implementers:
1. **Review this architecture** - Validate structure and content plan
2. **Create directory structure** - `mkdir -p docs/{tutorial,howto,reference,explanation}`
3. **Start with Tutorial quadrant** - Highest user impact
4. **Parallel work possible** - Each quadrant can be done independently

### For Reviewers:
1. **Validate Diataxis compliance** - Check each quadrant follows principles
2. **Check content coverage** - Verify no gaps in v5.1.1 documentation
3. **Review migration plan** - Ensure smooth v4 → v5 transition

### For Project Leads:
1. **Approve architecture** - Green-light implementation
2. **Assign resources** - Allocate writer/reviewer capacity
3. **Set milestones** - 6-week incremental delivery

---

**Status**: Ready for implementation
**Confidence Level**: High (based on proven Diataxis framework)
**Risk Level**: Low (incremental migration, v4 content preserved)

