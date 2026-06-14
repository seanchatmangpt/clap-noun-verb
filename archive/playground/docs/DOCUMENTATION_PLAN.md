# Documentation Implementation Plan

**Phase**: Phase 4 (after critical fixes, code quality, and testing)
**Duration**: 2-3 days
**Target**: Diataxis score from 65/100 to 90/100
**Critical Gap**: Tutorial quadrant (45/100 - blocks onboarding)

---

## Diataxis Framework Assessment

| Quadrant | Current | Target | Gap | Priority |
|----------|---------|--------|-----|----------|
| **Tutorial** | 45/100 | 95/100 | 50 | CRITICAL |
| **How-To** | 85/100 | 95/100 | 10 | HIGH |
| **Reference** | 75/100 | 90/100 | 15 | MEDIUM |
| **Explanation** | 55/100 | 85/100 | 30 | HIGH |
| **Overall** | 65/100 | 90/100 | 25 | **CRITICAL** |

---

## Directory Structure

```
docs/
├── INDEX.md                                    (Synthesis navigation)
├── GETTING_STARTED.md              ← Tutorial (NEW - CRITICAL)
├── DESIGN_PHILOSOPHY.md            ← Explanation (NEW)
├── TROUBLESHOOTING.md              ← How-To (NEW)
├── USER_GUIDE.md                   ← How-To (existing, expand)
├── REFERENCE.md                    ← Reference (NEW, comprehensive)
├── API_REFERENCE.md                ← Reference (existing, improve)
├── CHANGELOG.md                    ← Reference (existing)
├── CONTRIBUTING.md                 ← How-To (existing)
├── examples/
│   ├── simple_paper.md             ← Tutorial example
│   ├── thesis_workflow.md           ← How-To example
│   └── advanced_sparql.md           ← Explanation example
└── diagrams/
    ├── architecture.svg            ← Explanation (NEW)
    ├── workflow.svg                ← Explanation (NEW)
    └── component_interaction.svg   ← Reference (NEW)
```

---

## 1. Getting Started Tutorial (CRITICAL - 4 hours)

**Location**: `docs/GETTING_STARTED.md`
**Audience**: Absolute beginners
**Goal**: New users can run their first command in <5 minutes

### Outline

```markdown
# Getting Started with Playground

## What is Playground?
[1 paragraph explaining thesis management with CLI]

## Prerequisites (2 min)
- Rust 1.70+
- Basic CLI knowledge
- (Optional: Understanding of academic thesis structure)

## Installation (5 min)
- Clone repository
- Build: cargo build --release
- Verify: ./target/release/htf --help

## Your First Paper (5 min)

### Step 1: Create a paper
$ htf papers add "My Research" imrad
✓ Paper added

### Step 2: List papers
$ htf papers list
- My Research [imrad]

### Step 3: Export to JSON
$ htf papers export My\ Research.json

### Step 4: View the output
$ cat My\ Research.json

## Your First Thesis (10 min)

### Step 1: Schedule
$ htf thesis schedule MyThesis

### Step 2: View schedule
$ htf thesis list

### Step 3: Check status
$ htf thesis check MyThesis

## Common Tasks (Reference to How-To)
- [Switching families](USER_GUIDE.md#switching-family)
- [Using SPARQL queries](USER_GUIDE.md#sparql-queries)
- [Generating PDF](USER_GUIDE.md#pdf-generation)

## Next Steps
→ Read [USER_GUIDE.md](USER_GUIDE.md) for deeper topics
→ See [REFERENCE.md](REFERENCE.md) for complete command list
→ Read [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) to understand why
```

**Content Checklist**:
- [ ] What & Why section (context)
- [ ] Installation steps (clear, tested)
- [ ] 3-5 minute first task (quick win)
- [ ] Build confidence (success is visible)
- [ ] Pointer to next learning step
- [ ] Screenshots/examples (visual proof)

---

## 2. Design Philosophy Document (2 hours)

**Location**: `docs/DESIGN_PHILOSOPHY.md`
**Audience**: Developers, architects, decision-makers
**Goal**: Explain "why" behind playground design

### Outline

```markdown
# Design Philosophy

## Core Principles

### 1. Type-First Design
- Why: Encode invariants in types
- Example: PaperFamily enum prevents invalid values
- Benefit: Compile-time guarantees

### 2. Determinism & Reproducibility
- Why: Same input should always produce same output
- Example: SPARQL queries return consistent results
- Benefit: Scriptable, testable, predictable

### 3. Semantic Web Integration
- Why: RDF enables flexible, extensible data
- Example: 26 family types encoded as RDF triples
- Benefit: Interoperable, queryable, standardized

### 4. Zero-Copy When Possible
- Why: Performance without sacrifice
- Example: Templates parsed once, reused
- Benefit: Fast, memory-efficient

## Why 26 Family Types?

[Detailed explanation of how theses map to 26 RDF concepts]

## How RDF Models Thesis Structure

[Example RDF triples showing thesis family encoding]

## Command Structure Rationale

[Explanation of noun-verb design]

## Extension Points

[How to add new families, templates, commands]
```

**Content Checklist**:
- [ ] Core 4-5 principles explained
- [ ] "Why" for each major design choice
- [ ] RDF/semantic web context
- [ ] Connections to academic standards
- [ ] Extension/modification guidelines
- [ ] References to research/standards

---

## 3. Troubleshooting Guide (3 hours)

**Location**: `docs/TROUBLESHOOTING.md`
**Audience**: Users encountering issues
**Goal**: Self-serve resolution for common problems

### Problem Categories

```markdown
# Troubleshooting Guide

## Installation Issues

### "htf: command not found"
**Problem**: Binary not in PATH
**Solutions**:
1. Check installation: `ls -la ~/.cargo/bin/htf`
2. Add to PATH: `export PATH=$PATH:~/.cargo/bin`
3. Verify: `htf --version`

### "error: could not compile playground"
**Problem**: Rust version incompatible
**Solutions**:
1. Check version: `rustc --version`
2. Update: `rustup update`
3. Required: Rust 1.70+

## Command Issues

### "Invalid family error"
**Problem**: Used invalid family name
**Valid families**:
- imrad, argument, contribution, monograph
- dsr, narrative, papers
**Fix**: `htf papers add "My Paper" imrad`

### "SPARQL query times out"
**Problem**: Complex query on large ontology
**Solutions**:
1. Simplify query (reduce wildcards)
2. Add LIMIT clause: `LIMIT 100`
3. Filter by family: `?x rdf:type MyFamily`

### "PDF export is blank"
**Problem**: LaTeX not installed
**Solutions**:
1. Install LaTeX: `brew install basictex`  (macOS)
2. Update templates: `tlmgr update --all`
3. Verify: `pdflatex --version`

## Data Issues

### "Paper not found"
**Problem**: Typo in paper name
**Solution**: List papers: `htf papers list`

### "Memory usage is high"
**Problem**: Large ontology loaded
**Solutions**:
1. Reduce ontology size
2. Query specific subset
3. Use LIMIT in SPARQL

## Performance Issues

### "CLI startup is slow"
**Problem**: System is busy
**Solutions**:
1. Check system resources: `top`
2. Close other applications
3. Profile: `cargo flamegraph`

## Getting Help

If not listed above:
1. Check error message carefully
2. Search GitHub issues
3. Create new issue with:
   - Rust version
   - Exact command run
   - Full error message
   - System info (OS, RAM, etc.)
```

**Content Checklist**:
- [ ] 10+ common issues covered
- [ ] Problem → Solution format
- [ ] Clear, actionable steps
- [ ] Verified solutions work
- [ ] Links to deeper docs
- [ ] Escalation path for unsupported issues

---

## 4. Comprehensive Reference Guide (3 hours)

**Location**: `docs/REFERENCE.md`
**Audience**: Users who know what they need but need syntax
**Goal**: Complete command/option reference

### Outline

```markdown
# Complete Reference

## Papers Commands

### papers add
Add a new paper to the registry

**Syntax**: `htf papers add <title> <family> [options]`

**Arguments**:
- `<title>`: Paper title (string, required)
- `<family>`: Paper family (enum, required)
  - Valid: imrad, argument, contribution, monograph, dsr, narrative, papers

**Options**:
- `-a, --author <name>`: Author name (default: current user)
- `-d, --description <text>`: Paper description
- `--tags <tag1>,<tag2>`: Comma-separated tags

**Examples**:
```bash
htf papers add "My Research" imrad
htf papers add "Theory" argument --author "Jane Doe"
htf papers add "Study" dsr --description "Design Science Research"
```

### papers list
List all papers

**Syntax**: `htf papers list [options]`

**Options**:
- `-f, --family <family>`: Filter by family
- `--format <format>`: Output format (json, yaml, table, plain)
- `--sort <field>`: Sort by (title, family, date)

**Examples**:
```bash
htf papers list --family imrad
htf papers list --format json > papers.json
htf papers list --sort date
```

### papers export
Export paper to PDF/JSON

**Syntax**: `htf papers export <paper-name> <output-file>`

**Examples**:
```bash
htf papers export "My Paper" output.pdf
htf papers export "My Paper" output.json
```

## Thesis Commands

[Similar format for thesis, config, meta commands]

## SPARQL Commands

### sparql
Execute SPARQL query

**Syntax**: `htf sparql "<query>" [options]`

**Options**:
- `--timeout <seconds>`: Query timeout (default: 5)
- `--limit <n>`: Limit results

**Examples**:
```bash
htf sparql "SELECT ?x WHERE { ?x rdf:type Paper }"
htf sparql "SELECT COUNT(*) WHERE { ?x ?y ?z }" --limit 1000
```

## Configuration

### config get
Get configuration value

**Syntax**: `htf config get <key>`

**Examples**:
```bash
htf config get output.format
htf config get profile:research
```

### config set
Set configuration value

**Syntax**: `htf config set <key> <value>`

**Examples**:
```bash
htf config set output.format json
htf config set profile:research family=imrad
```

## Meta Commands

### meta introspect
Get system introspection

### meta ontology
Get RDF ontology

### meta completions
Generate shell completions

## Global Options

- `-h, --help`: Show help
- `-v, --verbose`: Verbose output
- `--version`: Show version
- `--config <path>`: Use custom config file

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Resource not found |
| 4 | Timeout |
| 5 | Internal error |
```

**Content Checklist**:
- [ ] All 17 commands documented
- [ ] Syntax format consistent
- [ ] Arguments and options listed
- [ ] 2-3 examples per command
- [ ] Exit codes explained
- [ ] Cross-references between commands

---

## 5. Architecture Diagrams (4 hours)

Create 3 visual diagrams:

### 5A: System Architecture Diagram
```
File: docs/diagrams/architecture.svg

┌─────────────────────────────────────────────┐
│          User (Developer/Researcher)         │
└────────────┬────────────────────────────────┘
             │
             ▼
    ┌────────────────────┐
    │   CLI Layer        │
    │ (clap-noun-verb)   │
    └────────┬───────────┘
             │
     ┌───────┴────────┬──────────────┐
     ▼                ▼              ▼
┌─────────────┐ ┌─────────┐ ┌──────────┐
│Papers       │ │Thesis   │ │Config    │
│Commands     │ │Commands │ │Commands  │
└─────┬───────┘ └────┬────┘ └────┬─────┘
      │              │           │
      └──────┬───────┴───────────┘
             ▼
    ┌────────────────────┐
    │  Domain Layer      │
    │ (types, logic)     │
    └────────┬───────────┘
             │
     ┌───────┴────────┬──────────────┐
     ▼                ▼              ▼
┌──────────┐  ┌──────────┐   ┌──────────────┐
│Templates │  │Registry  │   │SPARQL        │
│(Tera)    │  │(In-mem)  │   │(Oxigraph RDF)│
└──────────┘  └──────────┘   └──────────────┘
```

### 5B: Paper Export Workflow
```
File: docs/diagrams/workflow.svg

Paper → Templates → Tera → LaTeX → pdflatex → PDF
          (7 types)        Render      CLI
```

### 5C: Component Interaction
```
File: docs/diagrams/interaction.svg

CLI Input
    ↓
[Argument Parsing]
    ↓
[Domain Handler]
    ├→ [Paper Service] ←→ [Registry]
    ├→ [Thesis Service] ←→ [RDF Store]
    └→ [Config Service] ←→ [Config File]
    ↓
[Output Formatter]
    ├→ JSON
    ├→ YAML
    ├→ Table
    └→ Plain
    ↓
CLI Output
```

---

## 6. Examples Section Expansion (2 hours)

Expand `examples/` with real-world workflows:

### Example 1: Simple Paper (`examples/simple_paper.md`)
```markdown
# Example: Create and Export a Simple Paper

## Goal
Create an academic paper using IMRaD structure and export to PDF.

## Steps

### 1. Create the paper
```bash
htf papers add "Machine Learning Applications" imrad \
  --author "Dr. Jane Smith" \
  --description "Survey of ML applications in healthcare"
```

### 2. View the paper
```bash
htf papers info "Machine Learning Applications"
```

### 3. Export to PDF
```bash
htf papers export "Machine Learning Applications" ml_survey.pdf
```

### 4. Result
Now you have `ml_survey.pdf` with your paper!

## What Happened

1. **Title & Family**: "Machine Learning Applications" (imrad = Introduction, Methods, Results, Discussion)
2. **Metadata**: Author and description stored
3. **Template**: Used standard IMRaD template
4. **LaTeX**: Template rendered to LaTeX, compiled to PDF
```

### Example 2: Thesis Workflow (`examples/thesis_workflow.md`)
### Example 3: Advanced SPARQL (`examples/advanced_sparql.md`)

---

## Implementation Checklist

- [ ] **Getting Started** (4 hours)
  - [ ] Create docs/GETTING_STARTED.md
  - [ ] Add screenshots/examples
  - [ ] Test with new user
  - [ ] Verify <5 min first task

- [ ] **Design Philosophy** (2 hours)
  - [ ] Create docs/DESIGN_PHILOSOPHY.md
  - [ ] Explain core principles
  - [ ] Justify design choices
  - [ ] Link to research/standards

- [ ] **Troubleshooting** (3 hours)
  - [ ] Create docs/TROUBLESHOOTING.md
  - [ ] Cover 10+ common issues
  - [ ] Test solutions work
  - [ ] Add diagnostic steps

- [ ] **Reference Guide** (3 hours)
  - [ ] Create docs/REFERENCE.md
  - [ ] Document all 17 commands
  - [ ] Add examples
  - [ ] Document exit codes

- [ ] **Diagrams** (4 hours)
  - [ ] Create architecture.svg
  - [ ] Create workflow.svg
  - [ ] Create interaction.svg
  - [ ] Verify accuracy

- [ ] **Examples** (2 hours)
  - [ ] Expand examples/ directory
  - [ ] Create 3+ real workflows
  - [ ] Verify examples work
  - [ ] Add explanations

- [ ] **Integration**
  - [ ] Update README.md links
  - [ ] Add docs to CI/CD
  - [ ] Verify all links work
  - [ ] Check spelling/grammar

---

## Success Metrics

- [ ] Diataxis score: 65 → 90/100
- [ ] Tutorial score: 45 → 95/100 (CRITICAL)
- [ ] How-To score: 85 → 95/100
- [ ] Reference score: 75 → 90/100
- [ ] Explanation score: 55 → 85/100
- [ ] New user can complete first task in <5 minutes
- [ ] All 17 commands documented with examples
- [ ] 3+ visual diagrams included
- [ ] 10+ troubleshooting issues resolved
- [ ] Documentation builds without warnings

---

**Status**: Implementation plan complete
**Next**: Execute Phase 4 documentation work
**Expected Duration**: 2-3 days

