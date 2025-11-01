# Introduction

This guide provides a comprehensive walkthrough for porting the `ggen` CLI application from regular `clap` to the `clap-noun-verb` framework. Whether you're new to `clap-noun-verb` or an experienced Rust developer, this guide will help you understand the porting process step-by-step.

## What is clap-noun-verb?

`clap-noun-verb` is a framework for building composable CLI patterns on top of `clap`. It provides a high-level, ergonomic API that organizes commands using the **noun-verb pattern** (e.g., `services status`, `collector up`), similar to how Python's Typer provides a simpler interface over Click.

### Key Features

- **Composable Command Structure**: Easy composition of nouns and verbs
- **Framework-Level APIs**: APIs that make it easy to build CLI frameworks
- **Type-Safe Composition**: Compile-time verification of command structure
- **Zero-Cost Abstractions**: Thin wrapper over clap with no runtime overhead
- **Convenience Macros**: Reduce boilerplate with `noun!` and `verb!` macros
- **Multiple Composition Methods**: Choose the approach that fits your needs

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

#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    // name: String → Required argument --name
    // rust: bool → Flag --rust
    Ok(ProjectOutput { name, rust })
}

#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred from filename
fn ai_generate(description: String, output: Option<String>) -> Result<String> {
    Ok(format!("Generated: {}", description))
}

// marketplace.rs
//! Template marketplace

#[verb] // Verb "search" auto-inferred, noun "marketplace" auto-inferred from filename
fn marketplace_search(query: String) -> Result<Vec<String>> {
    Ok(vec!["package1".to_string(), "package2".to_string()])
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

The `clap-noun-verb` approach provides:

- **Cleaner structure**: Commands naturally group by functionality
- **Less boilerplate**: Macros handle repetitive patterns
- **Better organization**: Related commands are co-located
- **Easier to extend**: Adding new verbs to a noun is simple
- **Type safety**: Compile-time verification of command structure
- **Multiple composition methods**: Choose what fits your project

## About ggen

`ggen` is a Rust Template Generator with Frontmatter & RDF Support. It provides:

- **AI-Powered Generation**: Generate templates, projects, and ontologies using LLMs
- **Deterministic & Reproducible**: Generate byte-identical output every time
- **Knowledge Graph-Driven**: Embed RDF and query with SPARQL
- **Marketplace Integration**: Reusable template packages (gpacks)
- **Production-Ready Testing**: Hermetic, deterministic test environments

From the ggen documentation, we can see it has commands like:

```bash
# AI commands
ggen ai project "E-commerce API with Stripe" --name shop-api --rust
ggen ai generate -d "Database repository pattern" -o repo.tmpl
ggen ai graph -d "User management ontology" -o users.ttl
ggen ai sparql -d "Find all active users" -g schema.ttl

# Marketplace commands
ggen search "rust web"
ggen add io.ggen.rust.axum
ggen list
ggen update
```

This is a perfect candidate for the noun-verb pattern, as commands naturally group into:
- `ai` noun with `project`, `generate`, `graph`, `sparql` verbs
- `marketplace` noun with `search`, `add`, `list`, `update` verbs
- And potentially other groups

## Benefits of the noun-verb pattern

The noun-verb pattern provides several advantages for CLI applications:

### 1. Intuitive Structure

Users quickly understand the command hierarchy:

```bash
ggen ai project <name>        # Generate an AI project
ggen ai generate <description> # Generate a template
ggen marketplace search <query> # Search marketplace
```

The structure is self-documenting and follows natural language patterns.

### 2. Scalable Organization

Adding new commands is straightforward:

```rust,no_run
noun!("ai", "AI-powered generation", [
    verb!("project", ...),
    verb!("generate", ...),
    verb!("new-command", ...), // Easy to add!
])
```

No need to modify enums or add match cases—just add the verb to the appropriate noun.

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

