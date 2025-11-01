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

#### After (clap-noun-verb v3.0.0)

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
    // Arguments automatically inferred from function signature
    handle_ai_project(name.clone(), description.clone(), rust)?;
    Ok(ProjectOutput { name, description, rust })
}

fn handle_ai_project(name: String, description: Option<String>, rust: bool) -> Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    println!("Generating AI project: {}", name);
    
    // Create project directory
    let project_dir = PathBuf::from(&name);
    fs::create_dir_all(&project_dir)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create project directory: {}", e)
        ))?;
    
    // Write project description if provided
    if let Some(desc) = description {
        let desc_file = project_dir.join("DESCRIPTION.txt");
        fs::write(&desc_file, format!("Project: {}\nDescription: {}", name, desc))
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
        )).map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create Cargo.toml: {}", e)
        ))?;
        
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        
        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {\n    println!(\"Hello, world!\");\n}\n")?;
    }
    
    println!("✓ Project '{}' generated successfully!", name);
    Ok(())
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

#### After (clap-noun-verb)

```rust,no_run
noun!("ai", "AI-powered generation", [
    // ... project verb
    verb!("generate", "Generate templates from descriptions", |args: &VerbArgs| {
        let description = args.get_one_str("description")?;
        let output = args.get_one_str_opt("output");
        
        // Generate template content
        let template_content = format!(
            "// Generated template\n// Description: {}\n\nTemplate code here...\n",
            description
        );
        
        // Write to output file or stdout
        if let Some(output_path) = output {
            use std::fs;
            fs::write(&output_path, &template_content)
                .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                    format!("Failed to write output file: {}", e)
                ))?;
            println!("✓ Template written to: {}", output_path);
        } else {
            println!("{}", template_content);
        }
        
        Ok(())
    }, args: [
        Arg::new("description")
            .short('d')
            .long("description")
            .required(true)
            .help("Template description"),
        Arg::new("output")
            .short('o')
            .long("output")
            .help("Output file path"),
    ]),
])
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
verb!("graph", "Generate RDF ontologies", |args: &VerbArgs| {
    let description = args.get_one_str("description")?;
    let output = args.get_one_str_opt("output");
    
    // Generate RDF/Turtle content
    let rdf_content = format!(
        r#"@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

# Generated ontology
# Description: {}

ex:ontology rdf:type ex:KnowledgeGraph ;
    ex:description "{}" .
"#, description, description
    );
    
    // Write to output file or stdout
    if let Some(output_path) = output {
        use std::fs;
        fs::write(&output_path, rdf_content)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write RDF file: {}", e)
            ))?;
        println!("✓ RDF ontology written to: {}", output_path);
    } else {
        println!("{}", rdf_content);
    }
    
    Ok(())
}, args: [
    Arg::new("description").short('d').long("description").required(true),
    Arg::new("output").short('o').long("output"),
]),
```

### AI sparql command

Includes an additional required `graph` argument:

```rust,no_run
verb!("sparql", "Generate SPARQL queries", |args: &VerbArgs| {
    let description = args.get_one_str("description")?;
    let graph = args.get_one_str("graph")?;
    let output = args.get_one_str_opt("output");
    
    // Generate SPARQL query
    let sparql_query = format!(
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
    );
    
    // Write to output file or stdout
    if let Some(output_path) = output {
        use std::fs;
        fs::write(&output_path, sparql_query)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write SPARQL file: {}", e)
            ))?;
        println!("✓ SPARQL query written to: {}", output_path);
    } else {
        println!("{}", sparql_query);
    }
    
    Ok(())
}, args: [
    Arg::new("description").short('d').long("description").required(true),
    Arg::new("graph").short('g').long("graph").required(true),
    Arg::new("output").short('o').long("output"),
]),
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

#### After (clap-noun-verb)

```rust,no_run
noun!("marketplace", "Template marketplace", [
    verb!("search", "Find packages", |args: &VerbArgs| {
        let query = args.get_one_str("query")?;
        // Search marketplace packages (simulated)
        println!("Searching marketplace for: '{}'", query);
        println!("");
        println!("Found packages:");
        println!("  1. io.ggen.rust.axum - Rust Axum web framework template");
        println!("  2. io.ggen.rust.cli - Rust CLI application template");
        println!("  3. io.ggen.python.flask - Python Flask web application");
        println!("");
        println!("Use 'ggen marketplace add <package>' to install a package");
        Ok(())
    }, args: [
        Arg::new("query").required(true).help("Search query"),
    ]),
])
```

### Add command

```rust,no_run
verb!("add", "Install package", |args: &VerbArgs| {
    let package = args.get_one_str("package")?;
    // Install package from marketplace
    println!("Installing package: {}", package);
    
    // In real implementation, download and install package
    use std::path::PathBuf;
    let package_dir = PathBuf::from("~/.ggen/packages").join(&package);
    println!("  Package location: {}", package_dir.display());
    println!("✓ Package '{}' installed successfully", package);
    Ok(())
}, args: [
    Arg::new("package").required(true).help("Package name (e.g., io.ggen.rust.axum)"),
]),
```

### List command

```rust,no_run
verb!("list", "List installed packages", |args: &VerbArgs| {
    // List installed packages
    println!("Installed packages:");
    println!("  io.ggen.rust.axum - v1.2.0");
    println!("  io.ggen.rust.cli - v2.0.1");
    println!("");
    println!("Use 'ggen marketplace update' to update packages");
    Ok(())
}),
```

### Update command

```rust,no_run
verb!("update", "Update packages", |args: &VerbArgs| {
    // Update installed packages
    println!("Checking for package updates...");
    println!("  io.ggen.rust.axum: v1.2.0 -> v1.3.0 (update available)");
    println!("  io.ggen.rust.cli: v2.0.1 -> v2.0.1 (up to date)");
    println!("");
    println!("✓ Package update check complete");
    Ok(())
}),
```

#### Complete Marketplace Commands Example

```rust,no_run
noun!("marketplace", "Template marketplace", [
    verb!("search", "Find packages", |args: &VerbArgs| {
        let query = args.get_one_str("query")?;
        // Search marketplace packages (simulated)
        println!("Searching marketplace for: '{}'", query);
        println!("");
        println!("Found packages:");
        println!("  1. io.ggen.rust.axum - Rust Axum web framework template");
        println!("  2. io.ggen.rust.cli - Rust CLI application template");
        println!("  3. io.ggen.python.flask - Python Flask web application");
        println!("");
        println!("Use 'ggen marketplace add <package>' to install a package");
        Ok(())
    }, args: [
        Arg::new("query").required(true),
    ]),
    verb!("add", "Install package", |args: &VerbArgs| {
        let package = args.get_one_str("package")?;
        
        // Install package from marketplace
        println!("Installing package: {}", package);
        
        // In real implementation, download and install package
        use std::path::PathBuf;
        let package_dir = PathBuf::from("~/.ggen/packages").join(&package);
        println!("  Package location: {}", package_dir.display());
        println!("✓ Package '{}' installed successfully", package);
        
        Ok(())
    }, args: [
        Arg::new("package").required(true),
    ]),
    verb!("list", "List installed packages", |args: &VerbArgs| {
        // List installed packages
        println!("Installed packages:");
        println!("  io.ggen.rust.axum - v1.2.0");
        println!("  io.ggen.rust.cli - v2.0.1");
        println!("");
        println!("Use 'ggen marketplace update' to update packages");
        
        Ok(())
    }),
    verb!("update", "Update packages", |args: &VerbArgs| {
        // Update installed packages
        println!("Checking for package updates...");
        println!("  io.ggen.rust.axum: v1.2.0 -> v1.3.0 (update available)");
        println!("  io.ggen.rust.cli: v2.0.1 -> v2.0.1 (up to date)");
        println!("");
        println!("✓ Package update check complete");
        
        Ok(())
    }),
])
```

## Porting template commands

If ggen has template-specific commands (separate from AI generation), they can be grouped under a `template` noun:

```rust,no_run
noun!("template", "Template operations", [
    verb!("generate", "Generate from template", |args: &VerbArgs| {
        let template = args.get_one_str("template")?;
        let vars = args.get_many_opt::<String>("vars");
        handle_template_generate(template, vars)?;
        Ok(())
    }, args: [
        Arg::new("template").required(true),
        Arg::new("vars").long("vars").num_args(1..),
    ]),
    verb!("validate", "Validate template", |args: &VerbArgs| {
        let template = args.get_one_str("template")?;
        handle_template_validate(template)?;
        Ok(())
    }, args: [
        Arg::new("template").required(true),
    ]),
    verb!("list", "List templates", |args: &VerbArgs| {
        handle_template_list()?;
        Ok(())
    }),
])
```

## Handling global arguments

Global arguments (like `--verbose` and `--config`) are available to all verbs through `VerbArgs`.

### Before (Regular clap)

```rust,no_run
#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    #[arg(short, long)]
    config: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    // Pass verbose/config to handlers manually
}
```

### After (clap-noun-verb)

```rust,no_run
use clap::Arg;

run_cli(|cli| {
    cli.name("ggen")
        .global_args(vec![
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count)
                .help("Increase verbosity"),
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file"),
        ])
        .noun(noun!("ai", "AI-powered generation", [
            verb!("project", "Generate complete projects", |args: &VerbArgs| {
                // Access global arguments
                let verbose = args.get_global_flag_count("verbose");
                let config = args.get_global_str("config");
                
                // Access verb-specific arguments
                let name = args.get_one_str("name")?;
                
                if verbose > 1 {
                    println!("[DEBUG] Config: {:?}", config);
                }
                
                // Project generation logic would go here
                println!("Generating project: {} (verbose: {}, config: {:?})", name, verbose, config);
                Ok(())
            }, args: [
                Arg::new("name").required(true),
            ]),
        ]))
})
```

### Using Global Arguments in Handlers

All verbs can access global arguments:

```rust,no_run
verb!("generate", "Generate templates", |args: &VerbArgs| {
    // Verb-specific
    let description = args.get_one_str("description")?;
    
    // Global arguments
    let verbose = args.get_global_flag_count("verbose");
    let config = args.get_global_str("config");
    
    if verbose > 0 {
        println!("[Verbose] Generating: {}", description);
    }
    
    if let Some(config_file) = config {
        load_config(config_file)?;
    }
    
    // Generate template content
    let template_content = format!(
        "// Generated template\n// Description: {}\n\nTemplate code here...\n",
        description
    );
    
    // Write to output file or stdout
    if let Some(output_path) = output {
        use std::fs;
        fs::write(&output_path, template_content)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write output file: {}", e)
            ))?;
        println!("✓ Template written to: {}", output_path);
    } else {
        println!("{}", template_content);
    }
    
    Ok(())
})
```

## Complete Example

Here's a complete example combining all command groups:

```rust,no_run
use clap_noun_verb::{noun, run_cli, verb, VerbArgs, Result};
use clap::Arg;

fn main() -> Result<()> {
    run_cli(|cli| {
        cli.name("ggen")
            .about("Rust Template Generator with Frontmatter & RDF Support")
            .version(env!("CARGO_PKG_VERSION"))
            .global_args(vec![
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::Count)
                    .help("Increase verbosity"),
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Configuration file"),
            ])
            .noun(noun!("ai", "AI-powered generation", [
                verb!("project", "Generate complete projects", |args: &VerbArgs| {
                    let name = args.get_one_str("name")?;
                    let description = args.get_one_str_opt("description");
                    let rust = args.is_flag_set("rust");
                    // Use the complete implementation shown below
                    // This calls the full handle_ai_project function
                    handle_ai_project(name, description, rust, args)?;
                    Ok(())
                }, args: [
                    Arg::new("name").required(true),
                    Arg::new("description"),
                    Arg::new("rust").long("rust"),
                ]),
                verb!("generate", "Generate templates from descriptions", |args: &VerbArgs| {
                    let description = args.get_one_str("description")?;
                    let output = args.get_one_str_opt("output");
                    handle_ai_generate(description, output, args)?;
                    Ok(())
                }, args: [
                    Arg::new("description").short('d').long("description").required(true),
                    Arg::new("output").short('o').long("output"),
                ]),
                verb!("graph", "Generate RDF ontologies", |args: &VerbArgs| {
                    let description = args.get_one_str("description")?;
                    let output = args.get_one_str_opt("output");
                    handle_ai_graph(description, output, args)?;
                    Ok(())
                }, args: [
                    Arg::new("description").short('d').long("description").required(true),
                    Arg::new("output").short('o').long("output"),
                ]),
                verb!("sparql", "Generate SPARQL queries", |args: &VerbArgs| {
                    let description = args.get_one_str("description")?;
                    let graph = args.get_one_str("graph")?;
                    let output = args.get_one_str_opt("output");
                    handle_ai_sparql(description, graph, output, args)?;
                    Ok(())
                }, args: [
                    Arg::new("description").short('d').long("description").required(true),
                    Arg::new("graph").short('g').long("graph").required(true),
                    Arg::new("output").short('o').long("output"),
                ]),
            ]))
            .noun(noun!("marketplace", "Template marketplace", [
                verb!("search", "Find packages", |args: &VerbArgs| {
                    let query = args.get_one_str("query")?;
                    handle_search(query, args)?;
                    Ok(())
                }, args: [
                    Arg::new("query").required(true),
                ]),
                verb!("add", "Install package", |args: &VerbArgs| {
                    let package = args.get_one_str("package")?;
                    handle_add(package, args)?;
                    Ok(())
                }, args: [
                    Arg::new("package").required(true),
                ]),
                verb!("list", "List installed packages", |args: &VerbArgs| {
                    handle_list(args)?;
                    Ok(())
                }),
                verb!("update", "Update packages", |args: &VerbArgs| {
                    handle_update(args)?;
                    Ok(())
                }),
            ]))
    })
}

// Handler functions receive VerbArgs for global argument access
fn handle_ai_project(name: String, description: Option<String>, rust: bool, args: &VerbArgs) -> Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    let verbose = args.get_global_flag_count("verbose");
    let config = args.get_global_str("config");
    
    if verbose > 0 {
        println!("[Verbose level {}] Starting project generation", verbose);
        println!("  Project name: {}", name);
        if let Some(ref desc) = description {
            println!("  Description: {}", desc);
        }
        println!("  Rust project: {}", rust);
        if let Some(ref cfg) = config {
            println!("  Using config: {}", cfg);
        }
    }
    
    // Load config if provided
    if let Some(config_file) = config {
        // In real implementation, load and parse config file
        if verbose > 1 {
            println!("[DEBUG] Loading config from: {}", config_file);
        }
    }
    
    // Create project directory
    let project_dir = PathBuf::from(&name);
    if verbose > 0 {
        println!("Creating project directory: {}", project_dir.display());
    }
    
    fs::create_dir_all(&project_dir)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create project directory: {}", e)
        ))?;
    
    // Write project description if provided
    if let Some(desc) = description {
        let desc_file = project_dir.join("DESCRIPTION.txt");
        fs::write(&desc_file, format!("Project: {}\nDescription: {}", name, desc))
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
                format!("Failed to write description file: {}", e)
            ))?;
        
        if verbose > 0 {
            println!("✓ Wrote description file");
        }
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
        )).map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create Cargo.toml: {}", e)
        ))?;
        
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        
        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {\n    println!(\"Hello, world!\");\n}\n")?;
        
        if verbose > 0 {
            println!("✓ Created Rust project structure");
        }
    }
    
    println!("✓ Project '{}' generated successfully!", name);
    Ok(())
}
```

## Next Steps

Now that you've seen how to port commands, learn about:

1. [Advanced Patterns](advanced-patterns.md) - Nested commands, custom implementations, and more
2. [Testing and Validation](testing-validation.md) - How to test your ported CLI

