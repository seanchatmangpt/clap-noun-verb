# Testing and Validation

This chapter covers testing strategies for ported CLI commands, including async/sync compatibility testing, ensuring backward compatibility, validating CLI structure, and integration testing.

## Testing Async/Sync Compatibility

**Critical**: Since ggen uses async business logic with sync CLI wrappers, we need to test both layers.

### Testing Sync CLI Wrappers

```rust,no_run
#[cfg(test)]
mod tests {
    use super::*;
    use clap_noun_verb::Result;

    #[test]
    fn test_utils_doctor_sync_wrapper() -> Result<()> {
        // Test that sync wrapper correctly spawns runtime and calls async function
        let result = utils_doctor()?;
        assert_eq!(result.overall, "OK");
        assert!(!result.checks.is_empty());
        Ok(())
    }
}
```

### Testing Async Business Logic Directly

```rust,no_run
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_run_diagnostics_async() -> Result<()> {
        // Test async business logic directly
        let result = crate::domain::utils::run_diagnostics().await?;
        assert_eq!(result.overall, "OK");
        Ok(())
    }

    #[test]
    async fn test_generate_project_async() -> Result<()> {
        // Test async project generation
        let result = crate::domain::ai::generate_project(
            "test-project".to_string(),
            Some("Test description".to_string()),
            true
        ).await?;
        
        assert_eq!(result.name, "test-project");
        assert!(result.rust);
        Ok(())
    }
}
```

### Testing Runtime Spawning

```rust,no_run
#[test]
fn test_runtime_creation() {
    // Verify runtime creation doesn't panic
    let rt = tokio::runtime::Runtime::new();
    assert!(rt.is_ok());
}

#[test]
fn test_block_on_execution() {
    // Test that block_on correctly executes async code
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result: i32 = rt.block_on(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        42
    });
    assert_eq!(result, 42);
}
```

## Testing ported commands

Testing CLI commands requires validating both the command structure and the handler logic.

### Unit Testing Handlers

Test handlers directly without CLI parsing:

```rust,no_run
#[cfg(test)]
mod tests {
    use super::*;
    use clap_noun_verb::{VerbArgs, VerbContext};
    use clap::ArgMatches;
    use std::collections::HashMap;

    fn create_test_args(arg_map: HashMap<&str, &str>) -> VerbArgs {
        // Create minimal ArgMatches for testing
        let matches = ArgMatches::default();
        let context = VerbContext::new("test_verb").with_noun("test_noun");
        VerbArgs::new(matches).with_context(context)
    }

    #[test]
    fn test_handle_project() -> Result<()> {
        // Arrange
        let args = create_test_args([("name", "my-project")].iter().cloned().collect());
        
        // Act
        let result = handle_project(&args);
        
        // Assert
        assert!(result.is_ok());
        // Verify project was created, etc.
        Ok(())
    }
}
```

### Testing with run_cli_with_args

Use `run_cli_with_args` to test the full CLI flow:

```rust,no_run
#[cfg(test)]
mod tests {
    use clap_noun_verb::run_cli_with_args;

    #[test]
    fn test_ai_project_command() -> Result<()> {
        // Arrange & Act
        let result = run_cli_with_args(
            vec!["ggen", "ai", "project", "test-project"],
            |cli| {
                cli.name("ggen")
                    .noun(noun!("ai", "AI-powered generation", [
                        verb!("project", "Generate project", |args: &VerbArgs| {
                            let name = args.get_one_str("name")?;
                            assert_eq!(name, "test-project");
                            Ok(())
                        }, args: [
                            Arg::new("name").required(true),
                        ]),
                    ]))
            }
        );
        
        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_ai_project_with_rust_flag() -> Result<()> {
        let result = run_cli_with_args(
            vec!["ggen", "ai", "project", "test-project", "--rust"],
            |cli| {
                cli.name("ggen")
                    .noun(noun!("ai", "AI-powered generation", [
                        verb!("project", "Generate project", |args: &VerbArgs| {
                            let name = args.get_one_str("name")?;
                            let rust = args.is_flag_set("rust");
                            assert_eq!(name, "test-project");
                            assert!(rust);
                            Ok(())
                        }, args: [
                            Arg::new("name").required(true),
                            Arg::new("rust").long("rust"),
                        ]),
                    ]))
            }
        );
        
        assert!(result.is_ok());
        Ok(())
    }
}
```

### Testing Argument Extraction

Test all argument extraction methods:

```rust,no_run
#[test]
fn test_argument_extraction() -> Result<()> {
    run_cli_with_args(
        vec![
            "ggen", "ai", "generate",
            "-d", "test description",
            "-o", "output.tmpl",
        ],
        |cli| {
            cli.name("ggen")
                .noun(noun!("ai", "AI-powered generation", [
                    verb!("generate", "Generate template", |args: &VerbArgs| {
                        // Test required string
                        let description = args.get_one_str("description")?;
                        assert_eq!(description, "test description");
                        
                        // Test optional string
                        let output = args.get_one_str_opt("output");
                        assert_eq!(output, Some("output.tmpl".to_string()));
                        
                        Ok(())
                    }, args: [
                        Arg::new("description").short('d').long("description").required(true),
                        Arg::new("output").short('o').long("output"),
                    ]),
                ]))
        }
    )?;
    Ok(())
}
```

### Testing Global Arguments

Test that global arguments are accessible:

```rust,no_run
#[test]
fn test_global_arguments() -> Result<()> {
    run_cli_with_args(
        vec![
            "ggen",
            "-vv",  // Global verbose flag
            "--config", "config.toml",  // Global config
            "ai", "project", "test-project",
        ],
        |cli| {
            cli.name("ggen")
                .global_args(vec![
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(clap::ArgAction::Count),
                    Arg::new("config")
                        .short('c')
                        .long("config"),
                ])
                .noun(noun!("ai", "AI-powered generation", [
                    verb!("project", "Generate project", |args: &VerbArgs| {
                        // Test global flag count
                        let verbose = args.get_global_flag_count("verbose");
                        assert_eq!(verbose, 2);
                        
                        // Test global string
                        let config = args.get_global_str("config");
                        assert_eq!(config, Some("config.toml".to_string()));
                        
                        Ok(())
                    }, args: [
                        Arg::new("name").required(true),
                    ]),
                ]))
        }
    )?;
    Ok(())
}
```

### Testing Error Cases

Test error handling:

```rust,no_run
#[test]
fn test_missing_required_argument() {
    let result = run_cli_with_args(
        vec!["ggen", "ai", "project"],  // Missing required "name"
        |cli| {
            cli.name("ggen")
                .noun(noun!("ai", "AI-powered generation", [
                    verb!("project", "Generate project", |args: &VerbArgs| {
                        let _name = args.get_one_str("name")?;
                        Ok(())
                    }, args: [
                        Arg::new("name").required(true),
                    ]),
                ]))
        }
    );
    
    // Should fail with argument error
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("name") || e.to_string().contains("required"));
    }
}

#[test]
fn test_invalid_command() {
    let result = run_cli_with_args(
        vec!["ggen", "nonexistent", "command"],
        |cli| {
            cli.name("ggen")
                .noun(noun!("ai", "AI-powered generation", [
                    // ... verbs
                ]))
        }
    );
    
    // Should fail with command not found
    assert!(result.is_err());
}
```

## Ensuring backward compatibility

When porting, ensure existing command invocations still work.

### Command Structure Compatibility

Ensure the command structure matches:

```bash
# Before (regular clap)
ggen ai project my-project

# After (clap-noun-verb) - Must work the same way
ggen ai project my-project
```

### Argument Compatibility

Ensure all arguments work the same way:

```bash
# Short flags
ggen ai generate -d "description" -o output.tmpl

# Long flags
ggen ai generate --description "description" --output output.tmpl

# Mixed
ggen ai generate -d "description" --output output.tmpl
```

### Help Text Compatibility

Verify help output matches expected format:

```rust,no_run
#[test]
fn test_help_output() -> Result<()> {
    // Test that help text is generated correctly
    let cli = CliBuilder::new()
        .name("ggen")
        .noun(/* ... */)
        .build_command();
    
    let help_text = cli.render_help().to_string();
    
    // Verify expected content
    assert!(help_text.contains("AI-powered generation"));
    assert!(help_text.contains("project"));
    assert!(help_text.contains("generate"));
    
    Ok(())
}
```

### Behavior Compatibility

Ensure command behavior matches:

```rust,no_run
#[test]
fn test_command_behavior_matches() -> Result<()> {
    // Test that the new implementation produces the same results
    // as the old implementation for the same inputs
    
    let inputs = vec![
        ("test-project", false),
        ("my-app", true),
    ];
    
    for (name, rust_flag) in inputs {
        // Run old implementation (if available)
        // let old_result = old_handle_project(name, rust_flag)?;
        
        // Run new implementation
        let new_result = run_cli_with_args(
            vec!["ggen", "ai", "project", name]
                .into_iter()
                .chain(if rust_flag { vec!["--rust"] } else { vec![] })
                .collect(),
            |cli| {
                cli.name("ggen")
                    .noun(noun!("ai", "AI-powered generation", [
                        verb!("project", "Generate project", handle_project, args: [
                            Arg::new("name").required(true),
                            Arg::new("rust").long("rust"),
                        ]),
                    ]))
            }
        )?;
        
        // Assert results match (compare outputs, file creation, etc.)
        // assert_eq!(old_result, new_result);
    }
    
    Ok(())
}
```

## Validating CLI structure

Use `auto_validate` to catch structural issues early:

### Enable Auto-Validation

```rust,no_run
run_cli(|cli| {
    cli.name("ggen")
        .auto_validate(true)  // Enable automatic validation
        .noun(/* ... */)
})
```

### Manual Validation

You can also validate manually:

```rust,no_run
use clap_noun_verb::{CliBuilder, CommandRegistry};

let registry = CliBuilder::new()
    .name("ggen")
    .noun(/* ... */)
    .registry();

// Manual validation
registry.validate()?;
```

### What Validation Catches

- **Duplicate noun names**: Multiple nouns with the same name
- **Empty nouns**: Nouns with no verbs or sub-nouns
- **Duplicate verb names**: Multiple verbs with the same name within a noun
- **Duplicate sub-noun names**: Multiple sub-nouns with the same name
- **Verb/sub-noun conflicts**: Name conflicts between verbs and sub-nouns

### Testing Validation

```rust,no_run
#[test]
fn test_validation_catches_duplicate_nouns() {
    let result = CliBuilder::new()
        .name("ggen")
        .auto_validate(true)
        .noun(noun!("ai", "AI commands", [/* ... */]))
        .noun(noun!("ai", "Duplicate AI", [/* ... */]))  // Duplicate!
        .run();
    
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().to_lowercase().contains("duplicate"));
    }
}

#[test]
fn test_validation_catches_duplicate_verbs() {
    let result = CliBuilder::new()
        .name("ggen")
        .auto_validate(true)
        .noun(noun!("ai", "AI commands", [
            verb!("project", "Generate project", |_| Ok(())),
            verb!("project", "Duplicate project", |_| Ok(())),  // Duplicate!
        ]))
        .run();
    
    assert!(result.is_err());
}
```

## Integration testing

Test the full CLI end-to-end:

### Basic Integration Test

```rust,no_run
#[test]
fn test_full_cli_workflow() -> Result<()> {
    // Test complete workflow
    run_cli_with_args(
        vec!["ggen", "marketplace", "search", "rust"],
        |cli| {
            cli.name("ggen")
                .noun(noun!("marketplace", "Template marketplace", [
                    verb!("search", "Find packages", |args: &VerbArgs| {
                        let query = args.get_one_str("query")?;
                        // Verify search works
                        assert_eq!(query, "rust");
                        Ok(())
                    }, args: [
                        Arg::new("query").required(true),
                    ]),
                ]))
        }
    )?;
    Ok(())
}
```

### Multi-Command Integration Test

```rust,no_run
#[test]
fn test_multiple_commands_in_sequence() -> Result<()> {
    // Test marketplace search
    run_cli_with_args(
        vec!["ggen", "marketplace", "search", "rust"],
        build_cli
    )?;
    
    // Test marketplace add
    run_cli_with_args(
        vec!["ggen", "marketplace", "add", "io.ggen.rust.axum"],
        build_cli
    )?;
    
    // Test AI project with added package
    run_cli_with_args(
        vec!["ggen", "ai", "project", "test-app", "--rust"],
        build_cli
    )?;
    
    Ok(())
}

fn build_cli(cli: CliBuilder) -> CliBuilder {
    cli.name("ggen")
        .noun(/* ... */)
        // ... all commands
}
```

### Testing Help Output

```rust,no_run
#[test]
fn test_help_display() -> Result<()> {
    let cli = CliBuilder::new()
        .name("ggen")
        .about("Rust Template Generator")
        .noun(/* ... */)
        .build_command();
    
    // Test root help
    let root_help = cli.render_help().to_string();
    assert!(root_help.contains("ggen"));
    assert!(root_help.contains("Rust Template Generator"));
    assert!(root_help.contains("AI-powered generation"));
    
    // Test noun help
    let ai_cmd = cli
        .get_subcommands()
        .find(|c| c.get_name() == "ai")
        .expect("AI command should exist");
    
    let ai_help = ai_cmd.render_help().to_string();
    assert!(ai_help.contains("AI-powered generation"));
    assert!(ai_help.contains("project"));
    assert!(ai_help.contains("generate"));
    
    Ok(())
}
```

## Performance testing

Ensure the port doesn't introduce performance regressions:

### CLI Building Performance

```rust,no_run
use std::time::Instant;

#[test]
fn test_cli_build_performance() {
    let start = Instant::now();
    
    let _cli = CliBuilder::new()
        .name("ggen")
        .noun(/* ... */)
        // ... all commands
        .build_command();
    
    let duration = start.elapsed();
    
    // Should build quickly (adjust threshold as needed)
    assert!(duration.as_millis() < 100, "CLI build took too long: {:?}", duration);
}
```

### Command Parsing Performance

```rust,no_run
#[test]
fn test_command_parsing_performance() {
    let cli = build_cli().build_command();
    
    let start = Instant::now();
    
    // Parse multiple times
    for _ in 0..1000 {
        let _matches = cli.clone().try_get_matches_from(vec![
            "ggen", "ai", "project", "test"
        ]).unwrap();
    }
    
    let duration = start.elapsed();
    
    // Average should be fast (adjust as needed)
    let avg = duration.as_millis() as f64 / 1000.0;
    assert!(avg < 1.0, "Average parsing took too long: {}ms", avg);
}
```

## Testing best practices

### 1. Test Behavior, Not Implementation

```rust,no_run
// ✅ Good: Test what the command does
#[test]
fn test_project_generation_creates_files() -> Result<()> {
    // Verify files are created
}

// ❌ Bad: Test implementation details
#[test]
fn test_project_uses_generator_function() {
    // Don't test internal function calls
}
```

### 2. Use Descriptive Test Names

```rust,no_run
// ✅ Good
#[test]
fn test_ai_project_with_rust_flag_generates_rust_project() -> Result<()> {
    // Clear what is being tested
}

// ❌ Bad
#[test]
fn test_project() {
    // Too vague
}
```

### 3. Test Both Success and Error Cases

```rust,no_run
#[test]
fn test_valid_input() -> Result<()> {
    // Test valid inputs
    Ok(())
}

#[test]
fn test_invalid_input_returns_error() {
    // Test error cases
    assert!(result.is_err());
}
```

### 4. Use Helper Functions

```rust,no_run
fn create_test_cli() -> CliBuilder {
    CliBuilder::new()
        .name("ggen")
        .noun(/* ... */)
}

#[test]
fn test_command_1() -> Result<()> {
    run_cli_with_args(vec!["ggen", "command1"], create_test_cli)?;
    Ok(())
}
```

### 5. Test Incrementally

Start with simple commands and build up:

1. Test individual handlers
2. Test simple commands
3. Test commands with arguments
4. Test global arguments
5. Test complex nested structures
6. Test integration scenarios

## Next Steps

With testing complete, proceed to:

1. [Migration Checklist](migration-checklist.md) - Final checklist and best practices for completing the migration

