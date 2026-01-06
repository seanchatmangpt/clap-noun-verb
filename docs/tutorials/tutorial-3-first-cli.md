# Tutorial 3: Generate Your First CLI

**Duration**: 10-15 minutes
**Level**: Beginner
**Prerequisites**: Completed [Tutorial 2: Create RDF Ontology](tutorial-2-first-rdf.md)
**Goals**:
- Use the code generator on your ontology
- Generate production-grade Rust CLI code
- Compile and run the generated code

## Step 1: Use the Code Generator

Update `src/lib.rs` to add code generation:

```rust
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
use clap_noun_verb::rdf::code_generator::CliCodeGenerator;
use std::fs;

pub fn generate_cli_code(turtle_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Load the Turtle file
    let turtle_content = fs::read_to_string(turtle_file)?;

    // Step 2: Parse the Turtle into an ontology
    let parser = TurtleParser::new();
    let ontology = parser.parse(&turtle_content)?;

    // Step 3: Validate the ontology
    ontology.validate_ontology()?;

    // Step 4: Generate Rust CLI code
    let generator = CliCodeGenerator::new()?;
    let generated = generator.generate_from_ontology(&ontology)?;

    // Get the generated code
    let rust_code = generated.rust_code();

    println!("✓ Generated CLI code ({} commands)", generated.verb_count());
    println!("✓ Output: {} lines of Rust", rust_code.lines().count());

    Ok(rust_code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_services_cli() {
        let result = generate_cli_code("ontology/services-cli.ttl");
        assert!(result.is_ok(), "Code generation failed: {:?}", result);

        let code = result.unwrap();
        assert!(code.contains("noun"), "Generated code missing noun macro");
        assert!(code.contains("verb"), "Generated code missing verb macro");
    }
}
```

## Step 2: Run Code Generation

```bash
cargo test test_generate_services_cli --lib --features rdf-composition -- --nocapture
```

**Expected output**:
```
✓ Generated CLI code (4 commands)
✓ Output: 156 lines of Rust

test test_generate_services_cli ... ok
```

## Step 3: Save Generated Code

Create a utility to save the generated code to a file:

```rust
pub fn save_cli_code(turtle_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let code = generate_cli_code(turtle_file)?;
    fs::write(output_file, code)?;
    println!("✓ Saved generated code to {}", output_file);
    Ok(())
}
```

## Step 4: Understanding Generated Code

The generator produces code like this:

```rust
// Generated from your ontology
#[noun("services", "Commands for managing services")]
pub struct Services;

#[verb(Services, "status")]
pub async fn status_service(args: &StatusArgs) -> Result<StatusResponse> {
    // Handler implementation
    Ok(StatusResponse::default())
}

#[verb(Services, "start")]
pub async fn start_service(args: &StartArgs) -> Result<StartResponse> {
    // Handler implementation
    Ok(StartResponse::default())
}

// ... more verbs generated from your ontology
```

### What Each Macro Does

**`#[noun(...)]`**: Defines a command category
- First argument: command name (lowercase)
- Second argument: description
- Creates a struct representing the noun

**`#[verb(...)]`**: Defines a command under a noun
- First argument: noun type
- Second argument: command name
- Generates handler function with proper types

## Step 5: Create a Full CLI Example

Create `examples/generated-cli.rs`:

```rust
// This example shows what generated code looks like
// In practice, you'd use the actual generated code

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate CLI code from ontology
    let turtle = fs::read_to_string("ontology/services-cli.ttl")?;

    println!("=== Generated CLI Code ===\n");

    // Show what gets generated
    let code_snippet = r#"
#[noun("services", "Commands for managing services")]
pub struct Services;

#[verb(Services, "status")]
pub async fn status_service(args: &StatusArgs) -> Result<StatusResponse> {
    println!("Checking service status...");
    Ok(StatusResponse::default())
}

#[verb(Services, "start")]
pub async fn start_service(args: &StartArgs) -> Result<StartResponse> {
    println!("Starting service...");
    Ok(StartResponse::default())
}

#[verb(Services, "stop")]
pub async fn stop_service(args: &StopArgs) -> Result<StopResponse> {
    println!("Stopping service...");
    Ok(StopResponse::default())
}

#[verb(Services, "restart")]
pub async fn restart_service(args: &RestartArgs) -> Result<RestartResponse> {
    println!("Restarting service...");
    Ok(RestartResponse::default())
}
"#;

    println!("{}", code_snippet);
    println!("\n✓ This code compiles and runs as a CLI!");
    println!("✓ All types are automatically inferred from your ontology");

    Ok(())
}
```

Run it:

```bash
cargo run --example generated-cli
```

## Generated Code Features

### ✅ Automatic Features

The code generator automatically:

1. **Creates proper types** for each verb
   - `StatusArgs` for command arguments
   - `StatusResponse` for output

2. **Handles async execution** (ready for agents)
   - All handlers are `async fn`
   - Compatible with tokio runtime

3. **Validates arguments** (compile-time)
   - Uses Rust's type system
   - Impossible to pass wrong arguments

4. **Generates JSON serialization**
   - Output serializable to JSON
   - Perfect for agent communication

5. **Creates documentation**
   - Doc comments generated from your ontology
   - Descriptions from rdfs:comment

### Example with Full Type Info

```rust
// Generated types based on your ontology
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusArgs {
    service_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    verbose: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    status: String,
    running: bool,
    uptime_seconds: u64,
}

// Handler function generated with proper signature
#[verb(Services, "status")]
pub async fn status_service(args: &StatusArgs) -> Result<StatusResponse> {
    // Implement your logic here
    Ok(StatusResponse {
        status: "running".to_string(),
        running: true,
        uptime_seconds: 3600,
    })
}
```

## Step 6: Integration with MCP

The generated code works perfectly with MCP (Model Context Protocol):

```rust
// MCP agents can:
1. Call GenerateCliFromTurtle to produce this code
2. Compile it into a plugin
3. Invoke the CLI commands
4. Parse JSON responses
5. Chain multiple agents together
```

## Customizing Generated Code

The generated code is a starting point. You can customize:

### 1. Handler Implementation

Replace the auto-generated logic:

```rust
#[verb(Services, "status")]
pub async fn status_service(args: &StatusArgs) -> Result<StatusResponse> {
    // Your custom implementation
    let actual_status = check_service(&args.service_name)?;
    Ok(StatusResponse {
        status: actual_status,
        running: true,
        uptime_seconds: get_uptime()?,
    })
}
```

### 2. Additional Methods

Add helper functions:

```rust
fn check_service(name: &str) -> Result<String> {
    // Your logic
    Ok("running".to_string())
}

fn get_uptime() -> Result<u64> {
    // Your logic
    Ok(3600)
}
```

### 3. Error Handling

Implement custom error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Service not found: {0}")]
    NotFound(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ServiceError> for Box<dyn std::error::Error> {
    fn from(err: ServiceError) -> Self {
        Box::new(err)
    }
}
```

## Workflow: Ontology → Code → Binary

```
ontology/
  services-cli.ttl
       ↓
  [Code Generator]
       ↓
src/
  generated/
    mod.rs (generated code)
       ↓
  lib.rs (your custom logic)
       ↓
  [Compiler]
       ↓
target/release/
  my_cli (executable)
```

## What You Learned

✅ How to use the code generator
✅ How to generate Rust CLI code from RDF ontologies
✅ What the generated code looks like
✅ How to customize generated code
✅ MCP integration for agent systems
✅ Type safety and compile-time validation

## Next Steps

Ready to query your ontologies with SPARQL?

**→ [Tutorial 4: Query Ontologies with SPARQL](tutorial-4-sparql.md)**

In the next tutorial, you'll:
- Learn SPARQL query language
- Query your ontology to discover capabilities
- Filter and join data semantically
- Use results to guide code generation

---

## Reference

- [Reference: API - CliCodeGenerator](../reference/api.md#CliCodeGenerator)
- [How-to: Build Multi-Level CLIs](../howto/multi-level.md)
- [Explanation: Design Patterns](../explanation/patterns.md)

