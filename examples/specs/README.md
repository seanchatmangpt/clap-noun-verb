# clap-noun-verb-gen - CLI Generator

`clap-noun-verb-gen` is a code generator that creates ready-to-compile Rust CLI applications using the clap-noun-verb framework from specifications.

## Overview

The generator transforms declarative specifications into complete, working Rust CLI applications using the `#[verb]` macro pattern for noun-verb command structures.

Three generation modes:
1. **from-yaml** - Generate from YAML specifications
2. **from-ttl** - Generate from RDF/Turtle ontologies
3. **scaffold** - Create minimal CLI skeletons

## Installation

The generator is included in the clap-noun-verb workspace:

```bash
# Build the generator
cargo build --bin clap-noun-verb-gen

# Run generator
cargo run --bin clap-noun-verb-gen -- gen --help
```

## Usage Examples

### 1. Generate from YAML Specification

Create a YAML file defining your CLI structure:

```yaml
# services-cli.yaml
name: services
about: Manage application services
version: 0.1.0
author: Your Name

verbs:
  - name: status
    noun: services
    doc: Show status of all services
    returns: ServiceStatus
    args: []

  - name: logs
    noun: services
    doc: Show logs for a service
    returns: ServiceLogs
    args:
      - name: service
        arg_type: String
        doc: Service name
        required: true

  - name: restart
    noun: services
    doc: Restart a service
    returns: RestartResult
    args:
      - name: service
        arg_type: String
        doc: Service name to restart
        required: true
      - name: force
        arg_type: bool
        doc: Force restart without graceful shutdown
        is_flag: true
```

Generate the CLI:

```bash
clap-noun-verb-gen gen from-yaml services-cli.yaml -o ./my-services-cli
```

Output structure:
```
my-services-cli/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library root
│   └── commands/
│       ├── mod.rs              # Module exports
│       ├── status.rs           # status verb handler
│       ├── logs.rs             # logs verb handler
│       └── restart.rs          # restart verb handler
```

Then build it:

```bash
cd my-services-cli
cargo build
./target/debug/services services status
./target/debug/services services logs my-service
./target/debug/services services restart my-service --force
```

### 2. Generate from TTL (RDF) Ontology

Create an RDF/Turtle ontology:

```ttl
# database-cli.ttl
@prefix : <http://example.org/db/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix skos: <http://www.w3.org/2004/02/skos/core#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

:DatabaseCLI
    a owl:Class ;
    skos:prefLabel "database" ;
    rdfs:comment "Database management CLI" .

:StatusVerb
    a owl:Class ;
    rdfs:subClassOf :DatabaseVerb ;
    skos:prefLabel "status" ;
    rdfs:comment "Show database status" .

:BackupVerb
    a owl:Class ;
    rdfs:subClassOf :DatabaseVerb ;
    skos:prefLabel "backup" ;
    rdfs:comment "Create database backup" .

:MigrateVerb
    a owl:Class ;
    rdfs:subClassOf :DatabaseVerb ;
    skos:prefLabel "migrate" ;
    rdfs:comment "Run database migrations" .
```

Generate:

```bash
clap-noun-verb-gen gen from-ttl database-cli.ttl -o ./db-cli -n mydb
```

### 3. Create a Minimal Scaffold

For quick prototyping:

```bash
# Create basic scaffold
clap-noun-verb-gen gen scaffold my-cli -o ./my-cli --with-examples --with-cargo
```

Output:

```
my-cli/
├── Cargo.toml
└── src/
    ├── main.rs           # Entry point
    ├── lib.rs            # Library root
    └── commands/
        ├── mod.rs        # Module exports
        ├── status.rs     # Example: status command
        └── help.rs       # Example: help command
```

Build and run:

```bash
cd my-cli
cargo build
./target/debug/my-cli system status
./target/debug/my-cli system help
```

## Specification Format

### YAML Schema

```yaml
name: string              # CLI name (kebab-case recommended)
about: string             # Short description
version: string           # Semantic version
author: string            # Author name

verbs:
  - name: string          # Verb name (function name)
    noun: string?         # Optional noun namespace
    doc: string           # Help documentation
    returns: string       # Return type (default: serde_json::Value)
    args:
      - name: string      # Argument name
        arg_type: string  # Type: String, i32, bool, f64, etc.
        doc: string?      # Help text
        required: bool    # Default: true
        default: string?  # Default value
        short: char?      # Short flag (-x)
        long: string?     # Long flag (--flag)
        values: [string]  # Enum values
        is_flag: bool     # Is a boolean flag
```

### Supported Argument Types

```
string, text, str       -> String
int, i32, integer       -> i32
i64, u32, u64           -> i64, u32, u64
f32, float, f64, double -> f32, f64
bool, boolean           -> bool
json, value             -> serde_json::Value
path, pathbuf           -> std::path::PathBuf
```

## Generated Code Structure

Generated verbs follow this pattern:

```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct StatusOutput {
    pub message: String,
}

/// Show system status
#[verb]
pub fn status() -> Result<serde_json::Value> {
    // TODO: Implement logic
    Ok(serde_json::json!({
        "message": "Command status executed successfully",
        "status": "unimplemented"
    }))
}
```

The `#[verb]` macro:
- Auto-discovers the function at compile time
- Infers verb name from function name
- Extracts documentation from doc comments
- Parses arguments from function signature

## Command Reference

```bash
# Show help for all generators
clap-noun-verb-gen gen --help

# From YAML
clap-noun-verb-gen gen from-yaml <FILE> -o <DIR> [--verify]

  <FILE>              Path to YAML specification
  -o, --output <DIR>  Output directory
  --verify            Verify compilation with cargo check

# From TTL
clap-noun-verb-gen gen from-ttl <FILE> -o <DIR> [--name <NAME>] [--with-cargo] [--verify]

  <FILE>              Path to TTL file
  -o, --output <DIR>  Output directory
  -n, --name <NAME>   Override CLI name from TTL
  --with-cargo        Generate Cargo.toml
  --verify            Verify compilation

# Scaffold
clap-noun-verb-gen gen scaffold <NAME> -o <DIR> [--with-examples] [--with-cargo]

  <NAME>              CLI name
  -o, --output <DIR>  Output directory
  --with-examples     Include example verbs
  --with-cargo        Generate Cargo.toml
```

## Workflow

### 1. Define Specification

Create YAML or TTL defining commands and arguments.

### 2. Generate Code

Run generator to create project structure and boilerplate.

### 3. Implement Verbs

Edit generated verb functions in `src/commands/` to add business logic.

### 4. Build & Test

```bash
cargo build
./target/debug/my-cli <noun> <verb> [args]
```

## Generated Features

- ✅ Automatic command discovery via `#[verb]` macro
- ✅ JSON output by default
- ✅ Type-safe arguments with derive macros
- ✅ Integration with clap 4.5+
- ✅ Minimal dependencies (10 core crates)
- ✅ Ready-to-compile boilerplate
- ✅ Example-driven documentation

## Advanced: Custom Return Types

Modify generated verb return types:

```rust
#[derive(Serialize, Debug)]
pub struct MyOutput {
    pub result: String,
    pub count: u32,
}

/// Show detailed status
#[verb]
pub fn status() -> Result<MyOutput> {
    Ok(MyOutput {
        result: "success".to_string(),
        count: 42,
    })
}
```

## Integration with Existing Projects

Add to your `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap-noun-verb = "5.6"
clap-noun-verb-macros = "5.6"
linkme = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Then use the generated code as a reference for your own verbs.

## See Also

- `clap-noun-verb` - Core framework
- `examples/specs/` - Example specifications
- `examples/tutorial/` - Tutorial examples
