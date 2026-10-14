> Archived 2026-08-20: superseded/stale as of v26.8.20.

# GGen Manufacturing System

**Version:** 26.6.1
**Status:** Implemented
**Last Updated:** 2026-06-02

## Overview

GGen (Code Generation) is the **manufacturing system** for clap-noun-verb CLI applications. It transforms domain ontologies (RDF/Turtle definitions) into production-ready Rust code through a **proof-gated pipeline**.

The system implements the **CodeManufactory doctrine**: artifacts are produced by applying manufacturing operators (code generation templates) to source ontologies, with receipts proving provenance and proof gates ensuring quality.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ ONTOLOGY SOURCES (RDF/Turtle definitions)                       │
│ ├── ontology/clap-noun-verb-ontology.ttl (framework)            │
│ ├── ontology/cli-pattern.ttl (CLI patterns)                     │
│ ├── ontology/cargo-cicd.ttl (cargo-cicd domain)                 │
│ └── ... other domain ontologies ...                             │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
        ┌────────────────────────────────┐
        │ SPARQL Query Execution         │
        │ ├── cargo-cicd-commands.rq     │
        │ ├── generate-trait-impls.rq    │
        │ └── validate-cli-structure.rq  │
        └────────────────────────────────┘
                         │
                         ▼
        ┌────────────────────────────────┐
        │ MANUFACTURING STAGES           │
        │ ├── Stage 1: Spec Extraction   │
        │ ├── Stage 2: Validation        │
        │ ├── Stage 3: Code Generation   │
        │ ├── Stage 4: Documentation     │
        │ └── Stage 5: Test Generation   │
        └────────────────────────────────┘
                         │
                         ▼
        ┌────────────────────────────────┐
        │ PROOF GATES (Quality Checks)   │
        │ ├── Ontology Consistency       │
        │ ├── Code Compilation           │
        │ ├── Test Coverage              │
        │ └── Documentation Completeness │
        └────────────────────────────────┘
                         │
                         ▼
    ┌────────────────────────────────────────┐
    │ ARTIFACTS (Generated Code)             │
    │ ├── src/generated/verbs/                │
    │ ├── src/generated/nouns/                │
    │ ├── src/generated/cli_spec.json         │
    │ ├── docs/generated/commands/            │
    │ └── tests/generated/                    │
    └────────────────────────────────────────┘
                         │
                         ▼
        ┌────────────────────────────────┐
        │ RECEIPTS (Proof of Provenance) │
        │ ├── receipts/ggen/             │
        │ └── Hash chain validation      │
        └────────────────────────────────┘
```

## Core Concepts

### 1. Ontology (Source Definition)

**Location:** `ontology/cargo-cicd.ttl`

Domain model expressed in RDF/Turtle format. Defines:

- **Nouns:** Domain entities (e.g., `target`, `test`, `workspace`, `git`)
- **Verbs:** Actions on nouns (e.g., `show`, `run`, `prune`, `publish`)
- **Arguments:** CLI parameters for verbs (with types and validation)
- **Relationships:** Links between nouns, verbs, and arguments
- **Metadata:** Help text, output formats, environment requirements

Example:
```turtle
cicd:TargetNoun a cnv:Noun ;
    cnv:hasNounName "target"@en ;
    cnv:nounAbout "Manage build targets, architectures, and platform configurations"@en ;
    cnv:hasVerbs cicd:TargetShowVerb, cicd:TargetInstallVerb .

cicd:TargetShowVerb a cnv:Verb ;
    cnv:hasVerbName "show"@en ;
    cnv:verbAbout "Display all installed targets and their details"@en ;
    cnv:returnType "List<String>"@en ;
    cnv:outputFormat "json" .
```

### 2. SPARQL Queries (Specification Extraction)

**Location:** `queries/cargo-cicd-commands.rq`

SPARQL queries select noun-verb combinations and extract their specifications into JSON-LD or other formats.

Purpose:
- Extract structured command metadata from ontology
- Generate CLI specifications
- Validate ontology consistency

Example query output:
```json
{
  "noun_name": "target",
  "verb_name": "show",
  "full_command_name": "target show",
  "return_type": "List<String>",
  "output_format": "json",
  "arguments": []
}
```

### 3. Manufacturing Templates

**Location:** `templates/`

Jinja2 templates that transform query results into Rust code:

- **`verb_generated.rs.jinja`** — Handler function for each verb
- **`noun.rs.jinja`** — Noun module aggregating verbs
- **`command_doc.md.jinja`** — Auto-generated documentation
- **`cli_test.rs.jinja`** — Integration tests (AAA pattern)
- **`cicd_schema.rs.jinja`** — Type definitions

Templates receive context:
```jinja
{{ noun_name }}           # "target"
{{ verb_name }}           # "show"
{{ verb_description }}    # "Display all installed targets..."
{{ return_type }}         # "List<String>"
{{ arguments }}           # [{ name: "...", type: "...", ... }]
```

### 4. Manufacturing Stages

**Configuration:** `ggen.toml`

Pipeline stages execute in dependency order:

#### Stage 1: Specification Extraction
- **Query:** `queries/cargo-cicd-commands.rq`
- **Output:** `src/generated/cli_spec.json`
- **Purpose:** Extract normalized command specifications from ontology

#### Stage 2: Validation
- **Query:** `queries/validate-cli-structure.rq`
- **Output:** `src/generated/validation_report.json`
- **Purpose:** Check ontology consistency (no orphaned terms, duplicates, etc.)

#### Stage 3: Code Generation
- **Template:** `templates/verb_generated.rs.jinja`
- **Output:** `src/generated/verbs/{noun}/{verb}.rs`
- **Purpose:** Generate Rust handler code

#### Stage 4: Documentation
- **Template:** `templates/command_doc.md.jinja`
- **Output:** `docs/generated/commands/{noun}/{verb}.md`
- **Purpose:** Auto-generate command documentation

#### Stage 5: Test Generation
- **Template:** `templates/cli_test.rs.jinja`
- **Output:** `tests/generated/{noun}_{verb}_test.rs`
- **Purpose:** Generate integration tests (AAA pattern)

### 5. Proof Gates (Quality Checks)

Each artifact must pass proof gates before release:

| Gate | Severity | Check |
|------|----------|-------|
| Ontology Consistency | ERROR | No orphaned terms, all nouns/verbs have descriptions |
| Code Compilation | ERROR | Generated code must compile with `cargo check` |
| Test Coverage | WARNING | Minimum 50% coverage on generated code |
| Documentation | WARNING | All verbs must have doc comments |

**Proof Gate Result:** Each gate produces a receipt (JSON-LD) proving compliance.

### 6. Receipts (Proof of Manufacturing)

**Location:** `receipts/ggen/`

Each artifact receives a receipt (receipt.jsonld) containing:

- **artifact_id:** SHA256 hash of generated code
- **sources:** Hash of ontology, queries, templates used
- **stage:** Manufacturing stage that produced it
- **proof_gates:** List of proof gates passed
- **timestamp:** ISO8601 timestamp
- **version:** GGen version and configuration

Example:
```json
{
  "@context": "https://chatmangpt.com/ontologies/codemanufactory#",
  "artifact_id": "sha256:abc123...",
  "artifact_type": "rust_verb_handler",
  "noun": "target",
  "verb": "show",
  "sources": {
    "ontology": "sha256:xyz789...",
    "template": "sha256:def456..."
  },
  "proof_gates_passed": [
    "ontology_consistency",
    "code_compilation",
    "test_coverage"
  ],
  "timestamp": "2026-06-02T14:30:00Z",
  "ggen_version": "26.6.1"
}
```

## Configuration (ggen.toml)

**Location:** `ggen.toml`

Master configuration file for the manufacturing pipeline:

```toml
[ggen]
name = "clap-noun-verb"
version = "26.6.1"
ontology_sources = ["ontology/cargo-cicd.ttl", ...]
query_registry = ["queries/cargo-cicd-commands.rq", ...]
template_dir = "templates"
output_dir = "src/generated"

[ggen.stages]
spec_extraction = { query = "...", output = "..." }
validation = { ... }
trait_generation = { ... }
documentation = { ... }
test_generation = { ... }

[ggen.proof_gates]
ontology_consistency = { enabled = true, severity = "error" }
generated_code_compilation = { enabled = true, severity = "error" }
test_coverage = { enabled = true, severity = "warning" }
documentation_completeness = { enabled = true, severity = "warning" }

[ggen.receipts]
enabled = true
receipt_format = "jsonld"
receipt_dir = "receipts/ggen"
hash_algorithm = "sha256"
```

## Using GGen

### 1. Define Domain Ontology

Create or extend `ontology/cargo-cicd.ttl`:

```turtle
@prefix cnv: <http://clap-noun-verb.io/ontology#> .
@prefix cicd: <http://cargo-cicd.io/ontology#> .

cicd:MyNoun a cnv:Noun ;
    cnv:hasNounName "my-noun"@en ;
    cnv:nounAbout "Description"@en ;
    cnv:hasVerbs cicd:MyVerb .

cicd:MyVerb a cnv:Verb ;
    cnv:hasVerbName "my-verb"@en ;
    cnv:verbAbout "What it does"@en ;
    cnv:returnType "OutputType"@en .
```

### 2. Write SPARQL Query (Optional)

If `cargo-cicd-commands.rq` doesn't cover your domain, write a new query:

```sparql
PREFIX cnv: <http://clap-noun-verb.io/ontology#>
PREFIX cicd: <http://cargo-cicd.io/ontology#>

SELECT ?noun_name ?verb_name ?description
WHERE {
  ?noun a cnv:Noun ;
    cnv:hasNounName ?noun_name .
  ?noun cnv:hasVerbs ?verb .
  ?verb cnv:hasVerbName ?verb_name ;
    cnv:verbAbout ?description .
}
ORDER BY ?noun_name ?verb_name
```

### 3. Run Manufacturing Pipeline

Command (when ggen CLI is available):

```bash
# Full pipeline
ggen sync

# Specific stage
ggen stage spec_extraction
ggen stage validation
ggen stage trait_generation

# Show what would be generated
ggen sync --dry-run

# Regenerate with cache clear
ggen sync --no-cache
```

**Current Status:** GGen framework is defined in `ggen.toml`. Execution requires ggen CLI tool implementation or integration with existing code generation system.

### 4. Verify Proof Gates

```bash
# Check all proof gates
ggen proof-gate check

# List failed gates
ggen proof-gate status --failed-only

# Re-run specific gate
ggen proof-gate run ontology_consistency
```

### 5. Retrieve Receipts

```bash
# List all receipts
ggen receipt list

# Get receipt for specific artifact
ggen receipt show --artifact "target_show"

# Verify receipt chain
ggen receipt verify --all
```

## File Structure

After manufacturing completes:

```
clap-noun-verb/
├── ggen.toml                          # Manufacturing configuration
├── ontology/
│   ├── cargo-cicd.ttl                 # Domain ontology (SOURCE)
│   └── ...
├── queries/
│   ├── cargo-cicd-commands.rq        # Specification extraction query
│   └── ...
├── templates/
│   ├── verb_generated.rs.jinja        # Code templates
│   ├── command_doc.md.jinja
│   ├── cli_test.rs.jinja
│   └── cicd_schema.rs.jinja
├── src/generated/                     # ARTIFACTS (generated)
│   ├── cli_spec.json
│   ├── verbs/
│   │   ├── target/
│   │   │   ├── show.rs
│   │   │   └── install.rs
│   │   ├── test/
│   │   │   └── run.rs
│   │   └── ...
│   └── nouns/
│       ├── target.rs
│       ├── test.rs
│       └── ...
├── docs/generated/                    # Documentation (generated)
│   └── commands/
│       ├── target/
│       │   ├── show.md
│       │   └── install.md
│       └── ...
├── tests/generated/                   # Tests (generated)
│   ├── target_show_test.rs
│   ├── target_install_test.rs
│   └── ...
└── receipts/ggen/                     # RECEIPTS (proof of manufacturing)
    ├── target_show.receipt.jsonld
    ├── target_install.receipt.jsonld
    └── ...
```

## Theorem: Manufacturing Correctness

**Claim:** If all proof gates pass, then the generated code correctly implements the ontology.

**Proof Sketch:**
1. Ontology defines contract (nouns, verbs, arguments, output types)
2. SPARQL queries extract specifications from ontology (preserving contract)
3. Templates apply transformation rules deterministically
4. Proof gate: Code compilation verifies Rust syntax/type safety
5. Proof gate: Test coverage verifies behavior coverage
6. Therefore: Artifact code = Ontology contract + Proof gates ✓

## Extensions & Future Work

### Domain-Specific Ontologies

Create additional ontologies for different CLIs:

- `ontology/my-project.ttl` — Your project domain
- `queries/my-project-commands.rq` — Your project queries
- `templates/my-project-*.jinja` — Custom templates

### Custom Proof Gates

Extend `ggen.toml` with domain-specific proof gates:

```toml
[ggen.proof_gates]
my_custom_check = {
    enabled = true,
    severity = "error",
    check = "Custom validation rule"
}
```

### Feedback Loops

Use receipts to drive optimization:

1. Collect artifact metrics (execution time, coverage, etc.)
2. Correlate with ontology decisions
3. Update ontology for better manufacturing outcomes
4. Re-run pipeline with new receipt chain

### Integration with LSP

GGen could integrate with Language Server Protocol:

- On-save SPARQL validation
- Real-time template preview
- Ontology completion suggestions

## References

- **CodeManufactory Doctrine:** Artifacts are manufactured from sources via operators with receipt proof
- **Linked Data (RDF/Turtle):** Ontology language (W3C standards)
- **SPARQL:** Query language for RDF graphs (W3C standards)
- **Jinja2:** Template engine for code generation
- **clap-noun-verb:** Underlying CLI framework

## See Also

- [`ontology/cargo-cicd.ttl`](../ontology/cargo-cicd.ttl) — Domain ontology
- [`queries/cargo-cicd-commands.rq`](../queries/cargo-cicd-commands.rq) — Specification query
- [`templates/`](../templates/) — Manufacturing templates
- [`ggen.toml`](../ggen.toml) — Manufacturing configuration
