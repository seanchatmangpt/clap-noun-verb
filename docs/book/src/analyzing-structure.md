# Analyzing ggen's Current Structure

Before porting, we need to understand ggen's current CLI structure and map it to the noun-verb pattern. This chapter analyzes the existing commands and identifies how they should be organized.

## Overview of ggen commands

Based on the ggen documentation, here are the main command categories:

### AI Commands

```bash
ggen ai project "E-commerce API with Stripe" --name shop-api --rust
ggen ai generate -d "Database repository pattern" -o repo.tmpl
ggen ai graph -d "User management ontology" -o users.ttl
ggen ai sparql -d "Find all active users" -g schema.ttl
```

### Marketplace Commands (v2.0: market → marketplace)

```bash
ggen marketplace search "rust web"    # v2.0: market → marketplace
ggen marketplace add io.ggen.rust.axum
ggen marketplace list
ggen marketplace update
```

### Template Generation Commands (v2.0: gen → template generate)

```bash
ggen template generate --template example.tmpl --rdf domain.ttl  # v2.0: gen → template generate, --vars → --rdf
```

### Utility Commands (v2.0: root-level → utils noun)

```bash
ggen utils doctor      # v2.0: doctor → utils doctor
ggen utils help-me     # v2.0: help-me → utils help-me
```

## Identifying nouns

**Nouns** are entities or concepts that group related actions. In ggen's case, we can identify:

### 1. `ai` - AI-powered generation

This groups all AI-related commands:
- `project` - Generate complete projects
- `generate` - Generate templates from descriptions
- `graph` - Generate RDF ontologies
- `sparql` - Generate SPARQL queries

### 2. `marketplace` - Template marketplace

This groups marketplace-related operations:
- `search` - Find packages
- `add` - Install package
- `list` - List installed packages
- `update` - Update packages

### 3. `template` - Template operations (potential)

If there are template-specific commands beyond generation:
- `generate` - Generate from template
- `validate` - Validate template
- `list` - List templates

### 4. Root-level commands

Some commands might not need a noun wrapper if they're standalone:
- `doctor` - Diagnostics
- `help-me` - Get tips

However, we could group them under:
- `utils` - Utility commands

Or keep them as root-level if that makes more sense.

## Identifying verbs

**Verbs** are actions performed on nouns. For each noun identified above, here are the verbs:

### AI Verbs

| Verb | Description | Arguments |
|------|-------------|-----------|
| `project` | Generate complete projects | `name`, `--rust` flag |
| `generate` | Generate templates | `-d/--description`, `-o/--output` |
| `graph` | Generate RDF ontologies | `-d/--description`, `-o/--output` |
| `sparql` | Generate SPARQL queries | `-d/--description`, `-g/--graph`, `-o/--output` |

### Marketplace Verbs

| Verb | Description | Arguments |
|------|-------------|-----------|
| `search` | Find packages | `query` (required) |
| `add` | Install package | `package` (required) |
| `list` | List installed packages | None |
| `update` | Update packages | None |

### Template Verbs (v2.0)

| Verb | Description | Arguments |
|------|-------------|-----------|
| `generate` | Generate from template | `--template`, `--rdf` (v2.0: `--vars` → `--rdf`) |
| `validate` | Validate template | `template` |
| `list` | List templates | None |

## Command hierarchy analysis

Let's analyze the current structure and how it maps to noun-verb:

### Current Structure (Regular clap)

Assuming a typical clap structure, ggen likely uses:

```
ggen (root)
├── ai (subcommand)
│   ├── project (subcommand)
│   ├── generate (subcommand)
│   ├── graph (subcommand)
│   └── sparql (subcommand)
├── marketplace (subcommand)  # v2.0: market → marketplace
│   ├── search (subcommand)
│   ├── add (subcommand)
│   ├── list (subcommand)
│   └── update (subcommand)
├── utils (subcommand)        # v2.0: root-level → utils noun
│   ├── doctor (subcommand)
│   └── help-me (subcommand)
└── template (subcommand)     # v2.0: gen → template generate
    └── generate (subcommand)
```

### Target Structure (clap-noun-verb)

```
ggen (root)
├── ai (noun)
│   ├── project (verb)
│   ├── generate (verb)
│   ├── graph (verb)
│   └── sparql (verb)
├── marketplace (noun)
│   ├── search (verb)
│   ├── add (verb)
│   ├── list (verb)
│   └── update (verb)
└── template (noun, optional)
    ├── generate (verb)
    ├── validate (verb)
    └── list (verb)
```

### Decisions to Make

1. **Marketplace grouping**: Should `search`, `add`, `list`, `update` be grouped under `marketplace` noun, or stay as root-level commands?

   **Recommendation**: Group under `marketplace` for consistency and scalability. If users prefer root-level, we can alias them.

2. **Template vs Generation**: Is there a distinction between AI generation (`ai generate`) and template generation (`template generate`), or should they be unified?

   **Recommendation**: Keep separate - `ai generate` uses LLMs, while `template generate` uses existing templates with RDF data.

3. **Global arguments**: What global arguments does ggen support?
   - `--verbose` / `-v` (verbosity)
   - `--config` / `-c` (config file)
   - Potentially others

## Mapping commands to noun-verb structure

Here's the complete mapping using v3.0.0 attribute macro API:

### AI Commands

```rust,no_run
// ai.rs
//! AI-powered generation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct ProjectOutput {
    name: String,
    rust: bool,
}

// Business Logic Layer
fn create_project(name: String, rust: bool) -> ProjectOutput {
    ProjectOutput { name, rust }
}

// CLI Layer
#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    // Arguments automatically inferred: --name (required), --rust (flag)
    Ok(create_project(name, rust))
}

#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred
fn ai_generate(description: String, output: Option<String>) -> Result<String> {
    // --description (required), --output (optional)
    Ok(format!("Generated: {}", description))
}

#[verb] // Verb "graph" auto-inferred, noun "ai" auto-inferred
fn ai_graph(description: String, output: Option<String>) -> Result<String> {
    // --description (required), --output (optional)
    Ok(format!("RDF graph generated: {}", description))
}

#[verb] // Verb "sparql" auto-inferred, noun "ai" auto-inferred
fn ai_sparql(description: String, graph: String, output: Option<String>) -> Result<String> {
    // --description (required), --graph (required), --output (optional)
    Ok(format!("SPARQL query generated: {} for graph: {}", description, graph))
}
```

### Marketplace Commands

```rust,no_run
// marketplace.rs
//! Template marketplace

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct SearchResult {
    packages: Vec<String>,
}

// Business Logic Layer
fn search_packages(query: String) -> SearchResult {
    SearchResult { packages: vec!["package1".to_string(), "package2".to_string()] }
}

// CLI Layer
#[verb] // Verb "search" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_search(query: String) -> Result<SearchResult> {
    Ok(search_packages(query))
}

#[verb] // Verb "add" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_add(package: String) -> Result<String> {
    Ok(format!("Added: {}", package))
}

#[verb] // Verb "list" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_list() -> Result<Vec<String>> {
    Ok(vec!["package1".to_string(), "package2".to_string()])
}

#[verb] // Verb "update" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_update() -> Result<String> {
    Ok("Packages updated".to_string())
}
```

### Template Commands (v2.0: Pure RDF-Driven)

```rust,no_run
// template.rs
//! Template operations (v2.0: Pure RDF-driven)

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Business Logic Layer (Async - Reusable)
async fn generate_from_template_async(template: String, rdf: String) -> Result<TemplateOutput> {
    // v2.0: All data comes from RDF, no --vars
    // Load RDF, execute SPARQL queries, generate from template
    Ok(TemplateOutput {
        template,
        rdf,
        files_generated: vec![],
    })
}

// CLI Layer (Sync Wrapper - Delegates to Async Business Logic)
#[verb("generate", "template")] // Verb "generate", noun "template"
fn template_generate(template: String, rdf: String) -> Result<TemplateOutput> {
    // v2.0: --template and --rdf required (no --vars)
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    rt.block_on(async {
        crate::domain::template::generate_from_template(template, rdf).await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))
    })
}

#[derive(Serialize)]
struct TemplateOutput {
    template: String,
    rdf: String,
    files_generated: Vec<String>,
}
```

## Command grouping strategy

When grouping commands, consider:

### 1. Semantic Cohesion

Commands should naturally belong together:
- ✅ `ai project`, `ai generate` - All use AI/LLMs
- ✅ `marketplace search`, `marketplace add` - All manage marketplace packages
- ❌ Don't force unrelated commands into the same noun

### 2. User Mental Model

How do users think about the commands?
- "I want to use AI to generate something" → `ai` noun
- "I want to manage marketplace packages" → `marketplace` noun

### 3. Scalability

Consider future commands:
- `ai` might get `refine`, `explain`, `debug` verbs
- `marketplace` might get `remove`, `info`, `publish` verbs

### 4. Consistency

Use consistent naming patterns:
- All verbs within a noun should follow similar patterns
- Similar verbs across nouns should have similar behavior

## Next Steps

Now that we've analyzed the structure, we're ready to:

1. [Getting Started with Porting](getting-started.md) - Set up the project and understand the framework APIs
2. [Porting Commands Step-by-Step](porting-commands.md) - Implement each command group

