# Advanced Patterns

This chapter covers advanced patterns for porting complex CLI structures, including async/sync compatibility, nested commands, error handling, custom implementations, and conditional command registration.

## Async/Sync Compatibility Pattern

**Critical**: `clap-noun-verb` v3.0.0 uses **sync-only** functions, but ggen has **async business logic**. This pattern shows how to handle async operations with sync CLI wrappers.

### The Pattern

```rust,no_run
// commands/utils.rs - CLI Layer (Sync Wrappers)
use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
struct DoctorOutput {
    checks: Vec<CheckResult>,
    overall: String,
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    status: String,
    message: Option<String>,
}

// CLI Layer (Sync Wrapper - Delegates to Async Business Logic)
#[verb("doctor", "utils")]
fn utils_doctor() -> Result<DoctorOutput> {
    // Create runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    // Block on async business logic
    rt.block_on(async {
        crate::domain::utils::run_diagnostics().await
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))
    })
}

// domain/utils.rs - Business Logic Layer (Async - Reusable)
pub async fn run_diagnostics() -> Result<DoctorOutput> {
    // Async operations here - can be I/O, network, etc.
    use tokio::fs;
    
    // Check file system
    let rust_ok = fs::metadata("Cargo.toml").await.is_ok();
    
    // Check network services
    // ... async checks ...
    
    Ok(DoctorOutput {
        checks: vec![
            CheckResult {
                name: "Rust".to_string(),
                status: if rust_ok { "OK".to_string() } else { "ERROR".to_string() },
                message: None,
            },
        ],
        overall: "OK".to_string(),
    })
}
```

### Multiple Async Calls

```rust,no_run
#[verb("project", "ai")]
fn ai_project(name: String, description: Option<String>) -> Result<ProjectOutput> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    rt.block_on(async {
        // Multiple async operations
        let project_data = crate::domain::ai::fetch_project_data(name.clone()).await?;
        let template = crate::domain::ai::load_template("base.tmpl").await?;
        let generated = crate::domain::ai::generate_from_template(template, project_data).await?;
        
        Ok(ProjectOutput {
            name,
            description,
            files: generated,
        })
    })
}
```

### Error Handling with Async

```rust,no_run
#[verb("generate", "template")]
fn template_generate(template: String, rdf: String) -> Result<TemplateOutput> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(
            format!("Failed to create runtime: {}", e)
        ))?;
    
    rt.block_on(async {
        // Async operations with proper error handling
        crate::domain::template::generate_from_template(template, rdf).await
            .map_err(|e| {
                // Convert domain errors to CLI errors
                clap_noun_verb::NounVerbError::execution_error(
                    format!("Template generation failed: {}", e)
                )
            })
    })
}
```

## Nested command structures

For complex CLI hierarchies, `clap-noun-verb` supports nested noun structures using the compound noun pattern.

### Example: Dev Tools with Nested Structure

If ggen has development tools that need nesting:

```rust,no_run
noun!("dev", "Development tools", {
    noun!("test", "Testing utilities", [
        verb!("run", "Run tests", |args: &VerbArgs| {
            println!("Running tests...");
            // In real implementation: run test suite
            println!("✓ Tests completed");
            Ok(())
        }),
        verb!("watch", "Watch for changes", |args: &VerbArgs| {
            println!("Watching for test changes...");
            // In real implementation: start file watcher
            println!("✓ File watcher started");
            Ok(())
        }),
    ]),
    noun!("lint", "Code linting", [
        verb!("check", "Check code style", |args: &VerbArgs| {
            println!("Checking code style...");
            // In real implementation: run linter
            println!("✓ Code style check complete");
            Ok(())
        }),
        verb!("fix", "Auto-fix issues", |args: &VerbArgs| {
            println!("Auto-fixing linting issues...");
            // In real implementation: run auto-fix
            println!("✓ Linting issues fixed");
            Ok(())
        }),
    ]),
})
```

This creates commands like:
- `ggen dev test run`
- `ggen dev test watch`
- `ggen dev lint check`
- `ggen dev lint fix`

### When to Use Nested Nouns

Use nested nouns when:
1. **Natural hierarchy exists**: Commands naturally group into sub-categories
2. **Scalability needed**: Anticipate many commands that need organization
3. **Consistency**: Similar patterns exist elsewhere in the CLI

### Before (Regular clap)

```rust,no_run
#[derive(Subcommand)]
enum Commands {
    Dev {
        #[command(subcommand)]
        command: DevCommands,
    },
}

#[derive(Subcommand)]
enum DevCommands {
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
    Lint {
        #[command(subcommand)]
        command: LintCommands,
    },
}

#[derive(Subcommand)]
enum TestCommands {
    Run,
    Watch,
}

#[derive(Subcommand)]
enum LintCommands {
    Check,
    Fix,
}
```

### After (clap-noun-verb)

```rust,no_run
noun!("dev", "Development tools", {
    noun!("test", "Testing utilities", [
        verb!("run", "Run tests", |_args: &VerbArgs| Ok(())),
        verb!("watch", "Watch for changes", |_args: &VerbArgs| Ok(())),
    ]),
    noun!("lint", "Code linting", [
        verb!("check", "Check code style", |_args: &VerbArgs| Ok(())),
        verb!("fix", "Auto-fix issues", |_args: &VerbArgs| Ok(())),
    ]),
})
```

Much cleaner and more maintainable!

## Argument extraction and validation

### Type-Safe Argument Extraction

The `VerbArgs` type provides type-safe methods for extracting arguments:

```rust,no_run
verb!("project", "Generate project", |args: &VerbArgs| {
    // Required string
    let name = args.get_one_str("name")?;
    
    // Optional string
    let description = args.get_one_str_opt("description");
    
    // Required typed argument (usize, PathBuf, etc.)
    let port = args.get_one::<u16>("port")?;
    
    // Optional typed argument
    let timeout = args.get_one_opt::<u64>("timeout");
    
    // Multiple values
    let tags = args.get_many::<String>("tags")?;
    
    // Optional multiple values (returns empty vec if missing)
    let keywords = args.get_many_opt::<String>("keywords");
    
    // PathBuf convenience methods
    let config_path = args.get_path("config")?;
    let output_dir = args.get_path_opt("output");
    
    // Flags
    let force = args.is_flag_set("force");
    let verbose_count = args.get_flag_count("verbose");  // For -v, -vv, -vvv
    
    // Context access
    let verb_name = args.verb();  // "project"
    let noun_name = args.noun();  // Some("ai")
    
    Ok(())
}, args: [
    Arg::new("name").required(true),
    Arg::new("description"),
    Arg::new("port").value_parser(clap::value_parser!(u16)),
    Arg::new("timeout").value_parser(clap::value_parser!(u64)),
    Arg::new("tags").num_args(1..),
    Arg::new("keywords").num_args(1..),
    Arg::new("config").value_name("FILE"),
    Arg::new("output").value_name("DIR"),
    Arg::new("force").short('f').long("force"),
    Arg::new("verbose").short('v').action(clap::ArgAction::Count),
])
```

### Custom Validation

For complex validation, perform checks in the handler:

```rust,no_run
verb!("project", "Generate project", |args: &VerbArgs| {
    let name = args.get_one_str("name")?;
    
    // Custom validation
    if name.len() < 3 {
        return Err(NounVerbError::argument_error(
            "Project name must be at least 3 characters"
        ));
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(NounVerbError::argument_error(
            "Project name can only contain alphanumeric characters, dashes, and underscores"
        ));
    }
    
    // Continue with validated input
    handle_project(name)?;
    Ok(())
})
```

### Using clap Validators

You can use clap's built-in validators:

```rust,no_run
verb!("project", "Generate project", |args: &VerbArgs| {
    // Validation already done by clap
    let name = args.get_one_str("name")?;
    let port = args.get_one::<u16>("port")?;
    // ...
}, args: [
    Arg::new("name")
        .required(true)
        .value_parser(clap::builder::NonEmptyStringValueParser::new()),
    Arg::new("port")
        .value_parser(clap::value_parser!(u16).range(1..=65535)),
])
```

## Error handling with NounVerbError

The framework uses `NounVerbError` for structured error handling.

### Error Types

```rust,no_run
pub enum NounVerbError {
    CommandNotFound(String),
    VerbNotFound(String, String),  // (noun, verb)
    InvalidStructure(String),
    ExecutionError(String),
    ArgumentError(String),
}
```

### Creating Errors

Use the convenience constructors:

```rust,no_run
use clap_noun_verb::NounVerbError;

// Command not found
return Err(NounVerbError::command_not_found("nonexistent"));

// Verb not found
return Err(NounVerbError::verb_not_found("ai", "unknown"));

// Invalid structure
return Err(NounVerbError::invalid_structure(
    "Duplicate noun name: ai"
));

// Execution error
return Err(NounVerbError::execution_error(
    format!("Failed to generate project: {}", error)
));

// Argument error
return Err(NounVerbError::argument_error(
    "Required argument 'name' is missing"
));
```

### Error Propagation

Handler functions should propagate errors:

```rust,no_run
verb!("project", "Generate project", |args: &VerbArgs| -> Result<()> {
    let name = args.get_one_str("name")?;
    
    // Operations that return Result propagate automatically with ?
    let project_dir = create_project_dir(&name)?;
    generate_files(&project_dir)?;
    initialize_git(&project_dir)?;
    
    Ok(())
})
```

### Custom Error Context

Add context to errors:

```rust,no_run
verb!("project", "Generate project", |args: &VerbArgs| -> Result<()> {
    let name = args.get_one_str("name")?;
    
    match generate_project(&name) {
        Ok(_) => Ok(()),
        Err(e) => Err(NounVerbError::execution_error(
            format!("Failed to generate project '{}': {}", name, e)
        )),
    }
})
```

### Error Display

Errors are automatically formatted for display. Users see helpful messages:

```
Error: Command not found: nonexistent

Usage: ggen <COMMAND>

For more information, try 'ggen --help'
```

## Custom command implementations

For advanced use cases, you can implement the traits directly instead of using macros.

### Custom Noun Implementation

```rust,no_run
use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs, Result};

struct AiNoun {
    config: AiConfig,
}

impl NounCommand for AiNoun {
    fn name(&self) -> &'static str {
        "ai"
    }
    
    fn about(&self) -> &'static str {
        "AI-powered generation"
    }
    
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(ProjectVerb { config: self.config.clone() }),
            Box::new(GenerateVerb { config: self.config.clone() }),
        ]
    }
}

struct ProjectVerb {
    config: AiConfig,
}

impl VerbCommand for ProjectVerb {
    fn name(&self) -> &'static str {
        "project"
    }
    
    fn about(&self) -> &'static str {
        "Generate complete projects"
    }
    
    fn run(&self, args: &VerbArgs) -> Result<()> {
        let name = args.get_one_str("name")?;
        generate_project_with_config(&name, &self.config)?;
        Ok(())
    }
    
    fn build_command(&self) -> clap::Command {
        let mut cmd = clap::Command::new(self.name())
            .about(self.about());
        
        // Custom argument configuration
        cmd = cmd.arg(
            clap::Arg::new("name")
                .required(true)
                .help("Project name")
        );
        
        if self.config.allow_rust_option {
            cmd = cmd.arg(
                clap::Arg::new("rust")
                    .long("rust")
                    .help("Generate Rust project")
            );
        }
        
        cmd
    }
}
```

### Custom Verb with Advanced Arguments

```rust,no_run
struct CustomGenerateVerb;

impl VerbCommand for CustomGenerateVerb {
    fn name(&self) -> &'static str {
        "generate"
    }
    
    fn about(&self) -> &'static str {
        "Generate templates from descriptions"
    }
    
    fn additional_args(&self) -> Vec<clap::Arg> {
        vec![
            clap::Arg::new("description")
                .short('d')
                .long("description")
                .required(true)
                .help("Template description"),
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path"),
            clap::Arg::new("template")
                .short('t')
                .long("template")
                .value_name("TEMPLATE")
                .help("Base template to use"),
        ]
    }
    
    fn run(&self, args: &VerbArgs) -> Result<()> {
        use std::fs;
        
        let description = args.get_one_str("description")?;
        let output = args.get_one_str_opt("output");
        let template = args.get_one_str_opt("template");
        
        println!("Generating template from description: {}", description);
        if let Some(ref t) = template {
            println!("Using base template: {}", t);
        }
        
        let content = format!("// Generated from: {}\n// Template: {:?}\n\nCode here...\n", 
            description, template);
        
        if let Some(output_path) = output {
            fs::write(&output_path, content)?;
            println!("✓ Template written to: {}", output_path);
        } else {
            println!("{}", content);
        }
        
        Ok(())
    }
}
```

### When to Use Custom Implementations

Use custom implementations when:
1. **Complex configuration**: Commands need configuration passed from outside
2. **Conditional arguments**: Arguments depend on runtime state
3. **Shared state**: Multiple commands need shared state
4. **Advanced clap features**: Need fine-grained control over command building

## Global arguments pattern

Global arguments are available to all verbs through `VerbArgs`.

### Defining Global Arguments

```rust,no_run
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
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be done without making changes"),
        ])
        // ... nouns
})
```

### Accessing Global Arguments

All verbs can access global arguments:

```rust,no_run
verb!("project", "Generate project", |args: &VerbArgs| {
    // Verb-specific arguments
    let name = args.get_one_str("name")?;
    
    // Global arguments
    let verbose = args.get_global_flag_count("verbose");
    let config = args.get_global_str("config");
    let dry_run = args.is_global_flag_set("dry-run");
    
    if verbose > 0 {
        println!("[Verbose level: {}] Generating project: {}", verbose, name);
    }
    
    if let Some(config_file) = config {
        load_config(config_file)?;
    }
    
    if dry_run {
        println!("[DRY RUN] Would generate project: {}", name);
        return Ok(());
    }
    
    // Actual generation
    generate_project(&name)?;
    Ok(())
})
```

### Common Global Arguments

Common patterns for global arguments:

```rust,no_run
// Verbosity (multiple levels)
Arg::new("verbose")
    .short('v')
    .long("verbose")
    .action(clap::ArgAction::Count)
    .help("Increase verbosity (-v, -vv, -vvv)")

// Configuration file
Arg::new("config")
    .short('c')
    .long("config")
    .value_name("FILE")
    .help("Configuration file path")

// Output directory
Arg::new("output")
    .short('o')
    .long("output")
    .value_name("DIR")
    .help("Output directory")

// Quiet mode
Arg::new("quiet")
    .short('q')
    .long("quiet")
    .help("Suppress output")

// Debug mode
Arg::new("debug")
    .long("debug")
    .help("Enable debug output")
```

## Conditional command registration

You can conditionally register commands based on features or configuration.

### Feature-Based Commands

```rust,no_run
fn build_cli() -> CliBuilder {
    let mut cli = CliBuilder::new()
        .name("ggen")
        .about("Rust Template Generator");
    
    // Always available
    cli = cli.noun(noun!("marketplace", "Template marketplace", [
        // ... verbs
    ]));
    
    // Only if AI feature is enabled
    #[cfg(feature = "ai")]
    {
        cli = cli.noun(build_ai_noun());
    }
    
    // Only if template feature is enabled
    #[cfg(feature = "templates")]
    {
        cli = cli.noun(noun!("template", "Template operations", [
            // ... verbs
        ]));
    }
    
    cli
}
```

### Runtime Configuration-Based Commands

```rust,no_run
fn build_cli_with_config(config: &AppConfig) -> CliBuilder {
    let mut cli = CliBuilder::new()
        .name("ggen")
        .about("Rust Template Generator");
    
    // Always available
    cli = cli.noun(noun!("marketplace", "Template marketplace", [
        // ... verbs
    ]));
    
    // Conditional based on config
    if config.enable_ai {
        cli = cli.noun(build_ai_noun_with_config(&config.ai_config));
    }
    
    if config.enable_dev_tools {
        cli = cli.noun(noun!("dev", "Development tools", [
            // ... verbs
        ]));
    }
    
    cli
}
```

### Plugin-Based Commands

For dynamic command loading:

```rust,no_run
fn build_cli_with_plugins(plugins: &[Plugin]) -> CliBuilder {
    let mut cli = CliBuilder::new()
        .name("ggen")
        .about("Rust Template Generator");
    
    // Base commands
    cli = cli.noun(noun!("marketplace", "Template marketplace", [
        // ... verbs
    ]));
    
    // Load commands from plugins
    for plugin in plugins {
        if let Some(noun) = plugin.build_noun() {
            cli = cli.noun(noun);
        }
    }
    
    cli
}
```

## Organizing code with modules

For large CLIs, organize code into modules:

### Project Structure

```
src/
├── main.rs           # CLI builder
├── commands/
│   ├── mod.rs        # Re-export all commands
│   ├── ai.rs         # AI noun implementation
│   ├── marketplace.rs # Marketplace noun implementation
│   └── template.rs   # Template noun implementation
└── handlers/
    ├── mod.rs        # Re-export all handlers
    ├── ai_handlers.rs # AI command handlers
    └── marketplace_handlers.rs # Marketplace handlers
```

### Module Example

```rust,no_run
// src/commands/ai.rs
use clap_noun_verb::{noun, verb, VerbArgs, Result};
use clap::Arg;
use crate::handlers::ai_handlers;

pub fn build_ai_noun() -> impl NounCommand + 'static {
    noun!("ai", "AI-powered generation", [
        verb!("project", "Generate complete projects", ai_handlers::handle_project, args: [
            Arg::new("name").required(true),
        ]),
        verb!("generate", "Generate templates", ai_handlers::handle_generate, args: [
            Arg::new("description").short('d').long("description").required(true),
            Arg::new("output").short('o').long("output"),
        ]),
    ])
}

// src/handlers/ai_handlers.rs
use clap_noun_verb::{VerbArgs, Result};

pub fn handle_project(args: &VerbArgs) -> Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    let name = args.get_one_str("name")?;
    let project_dir = PathBuf::from(&name);
    
    println!("Generating project: {}", name);
    fs::create_dir_all(&project_dir)?;
    
    // Create Cargo.toml for Rust project
    let cargo_toml = project_dir.join("Cargo.toml");
    fs::write(&cargo_toml, format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#, name
    ))?;
    
    println!("✓ Project '{}' generated", name);
    Ok(())
}

pub fn handle_generate(args: &VerbArgs) -> Result<()> {
    use std::fs;
    
    let description = args.get_one_str("description")?;
    let output = args.get_one_str_opt("output");
    
    println!("Generating template: {}", description);
    
    let content = format!("// Generated template\n// Description: {}\n\nCode here...\n", description);
    
    if let Some(output_path) = output {
        fs::write(&output_path, content)?;
        println!("✓ Template written to: {}", output_path);
    } else {
        println!("{}", content);
    }
    
    Ok(())
}

// src/main.rs
mod commands;
mod handlers;

use clap_noun_verb::run_cli;

fn main() -> Result<()> {
    run_cli(|cli| {
        cli.name("ggen")
            .about("Rust Template Generator")
            .noun(commands::ai::build_ai_noun())
            .noun(commands::marketplace::build_marketplace_noun())
    })
}
```

## Next Steps

Now that you understand advanced patterns, proceed to:

1. [Testing and Validation](testing-validation.md) - Learn how to test your ported CLI
2. [Migration Checklist](migration-checklist.md) - Final checklist before completing the migration

