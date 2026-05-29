# Schema Validation & Introspection in clap-noun-verb

This document provides a comprehensive overview of the three schema validation and introspection layers in the `clap-noun-verb` framework:

1. **SHACL Schema Integration** – Compile-time generation and runtime validation of CLI invocations against SHACL shapes.
2. **JSON Schema Introspection (`--introspect`)** – Exporting command definitions as JSON Schema-compliant tool definitions for LLMs and MCP (Model Context Protocol).
3. **Dynamic Schema Verification** – Post-execution validation of command output serialization using registered hooks.

---

## 1. SHACL Schema Integration

The framework implements a semantic control plane using the Resource Description Framework (RDF) and Shapes Constraint Language (SHACL) to validate CLI command invocations. 

### Compilation Flow
When a command is annotated with the `#[verb]` macro, the macro generates RDF triples representing the verb metadata and SHACL shapes representing the input constraint definitions.

```
┌─────────────────────────────────┐
│     Rust #[verb] Annotation     │
└────────────────┬────────────────┘
                 │
                 ▼ (Macro expansion / compile-time)
┌─────────────────────────────────┐
│ RDF Triples & SHACL shapes      │
│ registered via distributed slice│
└────────────────┬────────────────┘
                 │
                 ▼ (Runtime instantiation)
┌─────────────────────────────────┐
│ RdfRegistry & ShapeValidator    │
│ build RDF Ontology / validate   │
└─────────────────────────────────┘
```

### Complete SHACL shape example (Turtle format)
Below is an example of the generated Turtle schema defining a SHACL NodeShape to validate arguments passed to the `services-status` command.

```turtle
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sh:    <http://www.w3.org/ns/shacl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix cnv:   <https://cnv.dev/ontology#> .
@prefix cli:   <https://cnv.dev/cli#> .

# NodeShape defining constraints for the services-status command
cli:services-status-shape a sh:NodeShape ;
    sh:targetNode cli:services-status ;
    
    # Validation constraint for argument: service_name
    sh:property [
        sh:path cnv:argument ;
        sh:name "service_name" ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:minLength 3 ;
        sh:maxLength 64 ;
        sh:pattern "^[a-zA-Z0-9_.-]+$" ;
        sh:description "Name of the target service to query" 
    ] ;
    
    # Validation constraint for argument: verbose
    sh:property [
        sh:path cnv:argument ;
        sh:name "verbose" ;
        sh:datatype xsd:boolean ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
        sh:description "Enable verbose output logging" 
    ] ;

    # Validation constraint for argument: timeout
    sh:property [
        sh:path cnv:argument ;
        sh:name "timeout" ;
        sh:datatype xsd:integer ;
        sh:minCount 0 ;
        sh:maxCount 1 ;
        sh:minInclusive 1 ;
        sh:maxInclusive 300 ;
        sh:description "Timeout duration in seconds" 
    ] .
```

### Runtime Validation using ShapeValidator
The `ShapeValidator` in the RDF control layer parses `ParsedInvocation` structs and validates them against the compiled shapes.

```rust
use std::collections::BTreeMap;
use clap_noun_verb::rdf::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize ShapeValidator & register the shape
    let mut validator = ShapeValidator::new();
    
    let shape = ShaclShape::new("cli:services-status-shape")
        .with_constraints(vec![
            Constraint::Required(true),
            Constraint::MinCount(1),
            Constraint::DataType("xsd:string".to_string()),
            Constraint::MinLength(3),
            Constraint::MaxLength(64),
            Constraint::Pattern("^[a-zA-Z0-9_.-]+$".to_string()),
        ]);
        
    validator.add_shape(shape)?;

    // 2. Mock a parsed user invocation (e.g. services status --service_name database)
    let mut args = BTreeMap::new();
    args.insert("service_name".to_string(), "database".to_string());
    
    let invocation = ParsedInvocation {
        command: "services-status".to_string(),
        args,
        output_format: Some("json".to_string()),
    };

    // 3. Execute validation check
    match validator.validate(&invocation) {
        Ok(_) => println!("✅ Invocation matches SHACL schema boundaries!"),
        Err(e) => eprintln!("❌ Shape constraint violation: {}", e),
    }

    Ok(())
}
```

---

## 2. JSON Schema Introspection (`--introspect`)

The framework allows LLM agents and orchestrators to discover tool structures directly from the CLI. Passing the global `--introspect` flag instructs the CLI to output all registered commands as a standard JSON Schema array of tools.

### Query Command
```bash
myapp --introspect
```

### JSON Schema Output Example
The generated output is fully compliant with OpenAI, Anthropic, and Model Context Protocol (MCP) tool-calling schemas:

```json
[
  {
    "name": "services_status",
    "description": "Get the status of a registered service",
    "parameters": {
      "type": "object",
      "properties": {
        "service_name": {
          "type": "string",
          "description": "Name of the target service to query"
        },
        "verbose": {
          "type": "boolean",
          "description": "Enable verbose output logging",
          "default": "false"
        },
        "timeout": {
          "type": "string",
          "description": "Timeout duration in seconds",
          "default": "30"
        }
      },
      "required": [
        "service_name"
      ]
    }
  },
  {
    "name": "calculator_add",
    "description": "Perform basic integer addition",
    "parameters": {
      "type": "object",
      "properties": {
        "left": {
          "type": "string",
          "description": "The left operand value"
        },
        "right": {
          "type": "string",
          "description": "The right operand value"
        }
      },
      "required": [
        "left",
        "right"
      ]
    }
  }
]
```

---

## 3. Dynamic Schema Verification

Dynamic Schema Verification allows developers to register global output validation hooks. These hooks run after a command execution finishes and before formatting the output, ensuring that serialized JSON values do not violate size, length, security, or domain-specific invariants.

### Registering an Output Validation Hook
A validation hook is registered globally and executes sequentially on all formatted payloads.

```rust
use serde::Serialize;
use clap_noun_verb::format::{
    register_output_validation_hook,
    format_output,
    OutputFormat,
};

#[derive(Serialize)]
struct SystemStatus {
    service: String,
    status: String,
    load_factor: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register output verification hook to enforce limits and security bounds
    register_output_validation_hook(|value| {
        // Enforce that load_factor does not exceed a critical ceiling limit
        if let Some(load) = value.get("load_factor").and_then(|v| v.as_f64()) {
            if load < 0.0 || load > 10.0 {
                return Err("load_factor value out of legal bounds [0.0, 10.0]".into());
            }
        }
        
        // Enforce maximum length constraint on string values
        if let Some(status_str) = value.get("status").and_then(|v| v.as_str()) {
            if status_str.len() > 256 {
                return Err("status payload length exceeds 256 character limit".into());
            }
        }

        Ok(())
    });

    // Mock output serialization
    let payload = SystemStatus {
        service: "web-portal".to_string(),
        status: "operational".to_string(),
        load_factor: 0.72,
    };

    // Format output (runs validation hooks internally)
    let formatted_str = OutputFormat::JsonPretty.format(&payload)?;
    println!("{}", formatted_str);

    Ok(())
}
```
