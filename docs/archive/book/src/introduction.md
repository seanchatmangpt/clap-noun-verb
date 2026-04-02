# Introduction

This guide provides a comprehensive walkthrough for porting the `ggen` CLI application from regular `clap` to the `clap-noun-verb` framework. Whether you're new to `clap-noun-verb` or an experienced Rust developer, this guide will help you understand the porting process step-by-step.

## What is clap-noun-verb?

`clap-noun-verb` is a framework for building composable CLI patterns on top of `clap`. It provides a high-level, ergonomic API that organizes commands using the **noun-verb pattern** (e.g., `services status`, `collector up`), similar to how Python's Typer provides a simpler interface over Click.

### Key Features

- **Attribute Macros**: `#[verb]` attribute for zero-boilerplate command registration
- **Auto-Discovery**: Commands automatically discovered at compile time
- **Auto-Inference**: Verb names from function names, noun names from filenames
- **Type Inference**: Arguments inferred from function signatures
- **JSON Output**: All output automatically serialized to JSON (perfect for agents/MCP)
- **Composable Command Structure**: Easy composition of nouns and verbs
- **Separation of Concerns**: Business logic separated from CLI layer

### The Noun-Verb Pattern

The noun-verb pattern structures commands hierarchically:

```
myapp
├── services
│   ├── status
│   ├── logs
│   └── restart
├── collector
│   ├── up
│   ├── down
│   └── status
└── dev
    ├── test
    │   ├── run
    │   └── watch
    └── lint
        ├── check
        └── fix
```

Where:
- **Nouns** are entities or concepts (e.g., `services`, `collector`, `dev`)
- **Verbs** are actions performed on nouns (e.g., `status`, `logs`, `up`)

This creates an intuitive, scalable command structure that's easy to understand and extend.

## Why port from regular clap?

While `clap` is powerful and flexible, building complex CLI structures with traditional `clap` often results in:

1. **Verbose enum-based structures**: Deeply nested enums and match statements
2. **Boilerplate code**: Repetitive command definitions and handlers
3. **Scattered logic**: Command definitions spread across multiple files
4. **Less intuitive organization**: Commands don't naturally group related functionality
5. **Harder maintenance**: Adding new commands requires touching multiple places

### Example: Before (Regular clap)

```rust,no_run
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },
    Search { query: String },
    Add { package: String },
    List,
    Update,
}

#[derive(Subcommand)]
enum AiCommands {
    Project {
        name: String,
        #[arg(long)]
        rust: bool,
    },
    Generate {
        #[arg(short, long)]
        description: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    Graph {
        #[arg(short, long)]
        description: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    Sparql {
        #[arg(short, long)]
        description: String,
        #[arg(short, long)]
        graph: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ai { command } => match command {
            AiCommands::Project { name, rust } => {
                // Handle ai project
            }
            AiCommands::Generate { description, output } => {
                // Handle ai generate
            }
            // ... more matches
        },
        Commands::Search { query } => {
            // Handle search
        },
        // ... more matches
    }
}
```

### Example: After (clap-noun-verb v3.0.0)

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

// Business Logic Layer (Pure Functions - Reusable)
fn create_project(name: String, rust: bool) -> ProjectOutput {
    ProjectOutput { name, rust }
}

// CLI Layer (Input Validation + Output Shaping Only)
#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    // name: String → Required argument --name
    // rust: bool → Flag --rust
    // Output automatically serialized to JSON
    Ok(create_project(name, rust))
}

#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred from filename
fn ai_generate(description: String, output: Option<String>) -> Result<String> {
    Ok(format!("Generated: {}", description))
}

// marketplace.rs
//! Template marketplace

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

// Business Logic Layer
fn search_packages(query: String) -> Vec<String> {
    vec!["package1".to_string(), "package2".to_string()]
}

// CLI Layer
#[verb] // Verb "search" auto-inferred, noun "marketplace" auto-inferred from filename
fn marketplace_search(query: String) -> Result<Vec<String>> {
    Ok(search_packages(query))
}

#[verb] // Verb "add" auto-inferred, noun "marketplace" auto-inferred from filename
fn marketplace_add(package: String) -> Result<String> {
    Ok(format!("Added: {}", package))
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

### Benefits

The `clap-noun-verb` v3.0.0 approach provides:

- **Auto-discovery**: No manual registration needed - commands are discovered automatically
- **Auto-inference**: Verb and noun names inferred from function names and filenames
- **Type inference**: Arguments inferred from function signatures automatically
- **JSON output**: All output automatically serialized to JSON (perfect for agents/MCP servers)
- **Cleaner structure**: Commands naturally group by functionality
- **Less boilerplate**: Attribute macros handle repetitive patterns
- **Better organization**: Related commands are co-located in files
- **Separation of concerns**: Business logic separated from CLI layer
- **Easier to extend**: Just add a function with `#[verb]` attribute
- **Type safety**: Compile-time verification of command structure

## About ggen v2.0

`ggen` is a Rust Template Generator with Pure RDF-Driven Architecture (v2.0). It provides:

- **Pure RDF-Driven Templates**: All data comes from RDF ontologies via SPARQL queries
- **AI-Powered Generation**: Generate templates, projects, and ontologies using LLMs
- **Business Logic Separation**: CLI layer automatically separated from editable business logic
- **Deterministic & Reproducible**: Generate byte-identical output every time
- **Knowledge Graph-Driven**: Embed RDF and query with SPARQL
- **Marketplace Integration**: Reusable template packages (gpacks)
- **Production-Ready Testing**: Hermetic, deterministic test environments

From ggen v2.0, commands follow the noun-verb pattern:

```bash
# AI commands
ggen ai project shop-api --rust
ggen ai generate --description "Database repository pattern" --output repo.tmpl
ggen ai graph --description "User management ontology" --output users.ttl
ggen ai sparql --description "Find all active users" --graph schema.ttl

# Marketplace commands (v2.0: market → marketplace)
ggen marketplace search "rust web"
ggen marketplace add io.ggen.rust.axum
ggen marketplace list
ggen marketplace update

# Utility commands (v2.0: doctor → utils doctor)
ggen utils doctor
ggen utils help-me

# Template commands (v2.0: gen → template generate)
ggen template generate --template verb.tmpl --rdf command.ttl
```

This follows the noun-verb pattern perfectly:
- `ai` noun with `project`, `generate`, `graph`, `sparql` verbs
- `marketplace` noun with `search`, `add`, `list`, `update` verbs
- `utils` noun with `doctor`, `help-me` verbs
- `template` noun with `generate`, `list`, `validate` verbs

## Benefits of the noun-verb pattern

The noun-verb pattern provides several advantages for CLI applications:

### 1. Intuitive Structure

Users quickly understand the command hierarchy:

```bash
ggen ai project <name>              # Generate an AI project
ggen ai generate --description <desc> # Generate a template
ggen marketplace search <query>     # Search marketplace (v2.0: market → marketplace)
ggen utils doctor                   # Run diagnostics (v2.0: doctor → utils doctor)
ggen template generate --template <tmpl> --rdf <rdf> # Generate from template (v2.0: gen → template generate)
```

The structure is self-documenting and follows natural language patterns.

### 2. Scalable Organization

Adding new commands is straightforward - just add a new function with `#[verb]`:

```rust,no_run
// ai.rs
//! AI-powered generation

#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(name: String) -> Result<ProjectOutput> { ... }

#[verb] // Verb "generate" auto-inferred
fn ai_generate(description: String) -> Result<String> { ... }

#[verb] // Verb "new-command" auto-inferred - Easy to add!
fn ai_new_command() -> Result<String> { ... }
```

No need to modify enums or add match cases—just add the function with `#[verb]`!

### 3. Consistent UX

Users learn one pattern and can apply it everywhere. Once they understand `noun verb`, they can predict how new commands work.

### 4. Type Safety

The framework provides compile-time verification of command structure, catching errors before runtime.

### 5. Better Code Organization

Commands are organized by functionality, making the codebase easier to navigate and maintain.

## Next Steps

Now that you understand the motivation and benefits, let's proceed to:

1. [Analyzing ggen's Current Structure](analyzing-structure.md) - Understanding how to map ggen's commands to the noun-verb pattern
2. [Getting Started with Porting](getting-started.md) - Setting up the project and understanding the framework APIs
3. [Porting Commands Step-by-Step](porting-commands.md) - Detailed examples of porting each command group

