# Porting Commands Step-by-Step

This chapter provides detailed examples of porting each command group from regular clap to clap-noun-verb, with before/after comparisons.

## Porting AI commands

The AI commands are perfect candidates for the noun-verb pattern. They all share the `ai` prefix and perform related actions.

### AI project command

#### Before (Regular clap)

```rust,no_run
use clap::{Parser, Subcommand, Arg};

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
}

#[derive(Subcommand)]
enum AiCommands {
    Project {
        name: String,
        description: Option<String>,
        #[arg(long)]
        rust: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ai { command } => match command {
            AiCommands::Project { name, description, rust } => {
                handle_ai_project(name, description, rust);
            }
            // ... other commands
        },
        // ... other commands
    }
}

fn handle_ai_project(name: String, description: Option<String>, rust: bool) {
    println!("Generating AI project: {}", name);
    if let Some(desc) = description {
        println!("Description: {}", desc);
    }
    if rust {
        println!("Generating Rust project structure...");
    }
    println!("Project '{}' generated successfully!", name);
}
```

#### After (clap-noun-verb v3.0.0 with async/sync pattern)

```rust,no_run
// commands/ai.rs - CLI Layer (Sync Wrappers)
//! AI-powered generation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct ProjectOutput {
    name: String,
    description: Option<String>,
    rust: bool,
}

// CLI Layer (Sync Wrapper - Delegates to Async Business Logic)
#[verb("project", "ai")] // Verb "project", noun "ai"
fn ai_project(name: String, description: Option<String>, rust: bool) -> Result<ProjectOutput> {
    // Create runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    // Block on async business logic
    rt.block_on(async {
        crate::domain::ai::generate_project(name, description, rust).await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))
    })
}

// domain/ai.rs - Business Logic Layer (Async - Reusable)
use tokio::fs;
use std::path::PathBuf;

pub async fn generate_project(
    name: String,
    description: Option<String>,
    rust: bool
) -> Result<ProjectOutput> {
    // Async business logic here - can be I/O, network, AI APIs, etc.
    let project_dir = PathBuf::from(&name);
    fs::create_dir_all(&project_dir).await
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create project directory: {}", e)
        ))?;
    
    // Write project description if provided
    if let Some(desc) = &description {
        let desc_file = project_dir.join("DESCRIPTION.txt");
        fs::write(&desc_file, format!("Project: {}\nDescription: {}", name, desc)).await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write description file: {}", e)
            ))?;
    }
    
    // Generate Rust project structure if requested
    if rust {
        let cargo_toml = project_dir.join("Cargo.toml");
        fs::write(&cargo_toml, format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#, name
        )).await
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create Cargo.toml: {}", e)
        ))?;
        
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to create src directory: {}", e)
            ))?;
        
        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {\n    println!(\"Hello, world!\");\n}\n").await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to create main.rs: {}", e)
            ))?;
    }
    
    Ok(ProjectOutput { name, description, rust })
}
```

#### Key Changes

1. **Eliminated nested enums**: No need for `Commands` and `AiCommands` enums
2. **Co-located handler**: Handler is defined inline with the command
3. **Type-safe argument extraction**: `get_one_str()` returns `Result<String>`, preventing panics
4. **Cleaner structure**: Related commands grouped together

### AI generate command

#### Before (Regular clap)

```rust,no_run
#[derive(Subcommand)]
enum AiCommands {
    // ...
    Generate {
        #[arg(short, long)]
        description: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    // ...
    match command {
        AiCommands::Generate { description, output } => {
            handle_ai_generate(description, output);
        }
        // ...
    }
}
```

#### After (clap-noun-verb v3.0.0)

```rust,no_run
// ai.rs
//! AI-powered generation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Business Logic Layer
fn generate_template(description: String) -> String {
    format!("// Generated template\n// Description: {}\n\nTemplate code here...\n", description)
}

// CLI Layer
#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred from filename
fn ai_generate(description: String, output: Option<String>) -> Result<String> {
    // Arguments automatically inferred: --description (required), --output (optional)
    let template_content = generate_template(description.clone());
    
    // Write to output file or return for JSON output
    if let Some(output_path) = output {
        use std::fs;
        fs::write(&output_path, &template_content)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write output file: {}", e)
            ))?;
        Ok(format!("Template written to: {}", output_path))
    } else {
        Ok(template_content)
    }
}
```

#### Complete AI Commands Example (v3.0.0)

Here's the complete AI noun with all verbs using the zero-args pattern:

```rust,no_run
// ai.rs
//! AI-powered generation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct ProjectOutput {
    name: String,
    description: Option<String>,
    rust: bool,
}

#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(name: String, description: Option<String>, rust: bool) -> Result<ProjectOutput> {
    // Arguments automatically inferred: --name (required), --description (optional), --rust (flag)
    Ok(ProjectOutput { name, description, rust })
}

#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred from filename
fn ai_generate(description: String, output: Option<String>) -> Result<String> {
    // --description (required), --output (optional)
    Ok(format!("Template generated: {}", description))
}

#[verb] // Verb "graph" auto-inferred, noun "ai" auto-inferred from filename
fn ai_graph(description: String, output: Option<String>) -> Result<String> {
    // --description (required), --output (optional)
    Ok(format!("RDF graph generated: {}", description))
}

#[verb] // Verb "sparql" auto-inferred, noun "ai" auto-inferred from filename
fn ai_sparql(description: String, graph: String, output: Option<String>) -> Result<String> {
    // --description (required), --graph (required), --output (optional)
    Ok(format!("SPARQL query generated: {} for graph: {}", description, graph))
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

### AI graph command

Similar pattern to `generate`, but for RDF graph generation:

```rust,no_run
// ai.rs (continued)

// Business Logic Layer
fn generate_rdf_graph(description: String) -> String {
    format!(
        r#"@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

# Generated ontology
# Description: {}

ex:ontology rdf:type ex:KnowledgeGraph ;
    ex:description "{}" .
"#, description, description
    )
}

// CLI Layer
#[verb] // Verb "graph" auto-inferred, noun "ai" auto-inferred
fn ai_graph(description: String, output: Option<String>) -> Result<String> {
    // Arguments automatically inferred: --description (required), --output (optional)
    let rdf_content = generate_rdf_graph(description);
    
    if let Some(output_path) = output {
        use std::fs;
        fs::write(&output_path, &rdf_content)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write RDF file: {}", e)
            ))?;
        Ok(format!("RDF ontology written to: {}", output_path))
    } else {
        Ok(rdf_content)
    }
}
```

### AI sparql command

Includes an additional required `graph` argument:

```rust,no_run
// ai.rs (continued)

// Business Logic Layer
fn generate_sparql_query(description: String, graph: String) -> String {
    format!(
        r#"# Generated SPARQL query
# Description: {}
# Graph: {}

PREFIX ex: <http://example.org/>
SELECT ?subject ?predicate ?object
WHERE {{
    GRAPH <{}> {{
        ?subject ?predicate ?object .
    }}
}}
"#, description, graph, graph
    )
}

// CLI Layer
#[verb] // Verb "sparql" auto-inferred, noun "ai" auto-inferred
fn ai_sparql(description: String, graph: String, output: Option<String>) -> Result<String> {
    // Arguments automatically inferred: --description (required), --graph (required), --output (optional)
    let sparql_query = generate_sparql_query(description, graph.clone());
    
    if let Some(output_path) = output {
        use std::fs;
        fs::write(&output_path, &sparql_query)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write SPARQL file: {}", e)
            ))?;
        Ok(format!("SPARQL query written to: {}", output_path))
    } else {
        Ok(sparql_query)
    }
}
```

## Porting marketplace commands

Marketplace commands manage template packages. They naturally group under a `marketplace` noun.

### Search command

#### Before (Regular clap)

```rust,no_run
#[derive(Subcommand)]
enum Commands {
    // ...
    Search {
        query: String,
    },
}

fn main() {
    match cli.command {
        Commands::Search { query } => {
            handle_search(query);
        }
        // ...
    }
}
```

#### After (clap-noun-verb v3.0.0 with async/sync pattern - v2.0: market → marketplace)

```rust,no_run
// commands/marketplace.rs - CLI Layer (Sync Wrappers)
//! Template marketplace (v2.0: market → marketplace)

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct SearchResult {
    packages: Vec<String>,
}

// CLI Layer (Sync Wrapper - Delegates to Async Business Logic)
#[verb("search", "marketplace")] // Verb "search", noun "marketplace" (v2.0: market → marketplace)
fn marketplace_search(query: String) -> Result<SearchResult> {
    // Create runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    // Block on async business logic
    rt.block_on(async {
        crate::domain::marketplace::search_packages(query).await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))
    })
}

// domain/marketplace.rs - Business Logic Layer (Async - Reusable)
pub async fn search_packages(query: String) -> Result<SearchResult> {
    // Async business logic here - can be network, database, etc.
    // In real implementation: query marketplace API
    Ok(SearchResult {
        packages: vec![
            "io.ggen.rust.axum - Rust Axum web framework template".to_string(),
            "io.ggen.rust.cli - Rust CLI application template".to_string(),
            "io.ggen.python.flask - Python Flask web application".to_string(),
        ],
    })
}
```

### Add command

```rust,no_run
// marketplace.rs (continued)

use serde::Serialize;

#[derive(Serialize)]
struct AddResult {
    package: String,
    location: String,
    success: bool,
}

// Business Logic Layer
fn add_package(package: String) -> AddResult {
    use std::path::PathBuf;
    let package_dir = PathBuf::from("~/.ggen/packages").join(&package);
    AddResult {
        package: package.clone(),
        location: package_dir.display().to_string(),
        success: true,
    }
}

// CLI Layer
#[verb] // Verb "add" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_add(package: String) -> Result<AddResult> {
    // Arguments automatically inferred: --package (required)
    Ok(add_package(package))
}
```

### List command

```rust,no_run
// marketplace.rs (continued)

use serde::Serialize;

#[derive(Serialize)]
struct PackageInfo {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct ListResult {
    packages: Vec<PackageInfo>,
}

// Business Logic Layer
fn list_packages() -> ListResult {
    ListResult {
        packages: vec![
            PackageInfo { name: "io.ggen.rust.axum".to_string(), version: "v1.2.0".to_string() },
            PackageInfo { name: "io.ggen.rust.cli".to_string(), version: "v2.0.1".to_string() },
        ],
    }
}

// CLI Layer
#[verb] // Verb "list" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_list() -> Result<ListResult> {
    Ok(list_packages())
}
```

### Update command

```rust,no_run
// marketplace.rs (continued)

use serde::Serialize;

#[derive(Serialize)]
struct UpdateResult {
    checked: usize,
    updated: usize,
    message: String,
}

// Business Logic Layer
fn update_packages() -> UpdateResult {
    UpdateResult {
        checked: 2,
        updated: 1,
        message: "Package update check complete".to_string(),
    }
}

// CLI Layer
#[verb] // Verb "update" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_update() -> Result<UpdateResult> {
    Ok(update_packages())
}
```

#### Complete Marketplace Commands Example

```rust,no_run
// marketplace.rs
//! Template marketplace

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
fn search_packages(query: String) -> Vec<String> {
    vec![
        "io.ggen.rust.axum - Rust Axum web framework template".to_string(),
        "io.ggen.rust.cli - Rust CLI application template".to_string(),
        "io.ggen.python.flask - Python Flask web application".to_string(),
    ]
}

fn add_package(package: String) -> String {
    format!("Package '{}' installed successfully", package)
}

fn list_packages() -> Vec<(String, String)> {
    vec![
        ("io.ggen.rust.axum".to_string(), "v1.2.0".to_string()),
        ("io.ggen.rust.cli".to_string(), "v2.0.1".to_string()),
    ]
}

fn update_packages() -> String {
    "Package update check complete".to_string()
}

// CLI Layer (Input Validation + Output Shaping Only)
#[verb] // Verb "search" auto-inferred, noun "marketplace" auto-inferred from filename
fn marketplace_search(query: String) -> Result<Vec<String>> {
    Ok(search_packages(query))
}

#[verb] // Verb "add" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_add(package: String) -> Result<String> {
    Ok(add_package(package))
}

#[verb] // Verb "list" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_list() -> Result<Vec<(String, String)>> {
    Ok(list_packages())
}

#[verb] // Verb "update" auto-inferred, noun "marketplace" auto-inferred
fn marketplace_update() -> Result<String> {
    Ok(update_packages())
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

## Porting template commands (v2.0: Pure RDF-Driven)

Template commands in v2.0 are **pure RDF-driven** - all data comes from RDF ontologies via SPARQL queries. No `--vars` flags - use `--rdf` instead.

```rust,no_run
// commands/template.rs - CLI Layer (Sync Wrappers)
//! Template operations (v2.0: Pure RDF-driven)

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct TemplateGenerateOutput {
    template: String,
    rdf: String,
    files_generated: Vec<String>,
}

// CLI Layer (Sync Wrapper - Delegates to Async Business Logic)
#[verb("generate", "template")] // Verb "generate", noun "template" (v2.0: gen → template generate)
fn template_generate(template: String, rdf: String) -> Result<TemplateGenerateOutput> {
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

// domain/template.rs - Business Logic Layer (Async - Reusable)
pub async fn generate_from_template(
    template: String,
    rdf: String
) -> Result<TemplateGenerateOutput> {
    // v2.0: All data comes from RDF
    // 1. Load RDF file into graph store
    // 2. Execute SPARQL queries from template
    // 3. Generate files from template with RDF data
    
    Ok(TemplateGenerateOutput {
        template,
        rdf,
        files_generated: vec!["generated_code.rs".to_string()],
    })
}

// Additional template commands
#[verb("validate", "template")]
fn template_validate(template: String) -> Result<bool> {
    // Sync validation - no async needed
    Ok(true)
}

#[verb("list", "template")]
fn template_list() -> Result<Vec<String>> {
    // Sync listing - no async needed
    Ok(vec!["template1".to_string(), "template2".to_string()])
}
```

## Handling global arguments

**Note:** Global arguments are not directly supported in v3.0.0 attribute macro API with `clap_noun_verb::run()`. If you need global arguments, you would need to use the builder API instead of auto-discovery.

However, the framework automatically provides JSON output, which is often more useful than global verbosity flags.

### After (clap-noun-verb v3.0.0)

With the attribute macro API, commands automatically return structured data as JSON:

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
    success: bool,
    message: String,
}

// Business Logic Layer
fn create_project(name: String, rust: bool) -> ProjectOutput {
    // Business logic here
    ProjectOutput {
        name: name.clone(),
        rust,
        success: true,
        message: format!("Project '{}' created successfully", name),
    }
}

// CLI Layer
#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    // Arguments automatically inferred: --name (required), --rust (flag)
    // Output automatically serialized to JSON
    Ok(create_project(name, rust))
}

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all commands!
}
```

**Usage:**
```bash
$ ggen ai project my-app --rust
{"name":"my-app","rust":true,"success":true,"message":"Project 'my-app' created successfully"}
```

The structured JSON output provides all information, eliminating the need for verbose flags in most cases.

### Structured Output Instead of Global Arguments

Instead of global verbosity flags, use structured return types:

```rust,no_run
// ai.rs (continued)

use serde::Serialize;

#[derive(Serialize)]
struct GenerateOutput {
    description: String,
    content: String,
    output_path: Option<String>,
    success: bool,
}

// Business Logic Layer
fn generate_template(description: String, output: Option<String>) -> GenerateOutput {
    let content = format!(
        "// Generated template\n// Description: {}\n\nTemplate code here...\n",
        description
    );
    
    GenerateOutput {
        description: description.clone(),
        content: content.clone(),
        output_path: output.clone(),
        success: true,
    }
}

// CLI Layer
#[verb] // Verb "generate" auto-inferred, noun "ai" auto-inferred
fn ai_generate(description: String, output: Option<String>) -> Result<GenerateOutput> {
    // Arguments automatically inferred: --description (required), --output (optional)
    // Output automatically serialized to JSON with all information
    Ok(generate_template(description, output))
}
```

**Usage:**
```bash
$ ggen ai generate -d "Database pattern" -o output.tmpl
{"description":"Database pattern","content":"// Generated template...","output_path":"output.tmpl","success":true}
```

All information is available in the structured JSON output, providing better integration with scripts and automation tools.

## Complete Example

Here's a complete example combining all command groups using v3.0.0 attribute macro API:

**File Structure:**
```
src/
├── main.rs          # Entry point
├── ai.rs            # AI commands
├── marketplace.rs   # Marketplace commands
└── template.rs      # Template commands
```

**src/ai.rs:**
```rust,no_run
//! AI-powered generation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Business Logic Layer
fn create_project(name: String, rust: bool) -> ProjectOutput {
    ProjectOutput { name, rust, success: true }
}

// CLI Layer
#[verb] // Auto-inferred: verb="project", noun="ai"
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, rust))
}

#[derive(Serialize)]
struct ProjectOutput {
    name: String,
    rust: bool,
    success: bool,
}
```

**src/marketplace.rs:**
```rust,no_run
//! Template marketplace

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct SearchResult {
    packages: Vec<String>,
}

#[verb] // Auto-inferred: verb="search", noun="marketplace"
fn marketplace_search(query: String) -> Result<SearchResult> {
    Ok(SearchResult {
        packages: vec!["package1".to_string(), "package2".to_string()],
    })
}

#[verb] // Auto-inferred: verb="add", noun="marketplace"
fn marketplace_add(package: String) -> Result<String> {
    Ok(format!("Added: {}", package))
}
```

**src/main.rs:**
```rust,no_run
mod ai;
mod marketplace;

use clap_noun_verb::Result;

fn main() -> Result<()> {
    // Auto-discovers all #[verb] functions in ai.rs and marketplace.rs
    clap_noun_verb::run()
}
```

**Usage:**
```bash
$ ggen ai project my-app --rust
{"name":"my-app","rust":true,"success":true}

$ ggen marketplace search rust
{"packages":["package1","package2"]}
```

**Key Features:**
1. **Auto-discovery**: All `#[verb]` functions automatically discovered
2. **Auto-inference**: Verb and noun names inferred from function names and filenames
3. **Type inference**: Arguments inferred from function signatures
4. **JSON output**: All return types automatically serialized to JSON
5. **Separation of concerns**: Business logic separated from CLI layer

## Next Steps

Now that you've seen how to port commands, learn about:

1. [Advanced Patterns](advanced-patterns.md) - Nested commands, custom implementations, and more
2. [Testing and Validation](testing-validation.md) - How to test your ported CLI

