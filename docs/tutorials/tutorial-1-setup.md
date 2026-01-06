# Tutorial 1: Set Up Your First MCP Agent

**Duration**: 10-15 minutes
**Level**: Beginner
**Goals**:
- Install clap-noun-verb with RDF support
- Verify installation with a test
- Understand basic project structure

## Prerequisites

- Rust 1.74 or later (`rustc --version`)
- Cargo installed (`cargo --version`)
- Git installed (`git --version`)
- Text editor or IDE (VS Code recommended)

## Step 1: Create a New Rust Project

```bash
cargo new my-first-agent --lib
cd my-first-agent
```

This creates a new library project. We use `--lib` because MCP agents are typically libraries that the MCP server loads.

**Expected output**:
```
Created library package `my-first-agent`
```

## Step 2: Add clap-noun-verb Dependency

Edit `Cargo.toml` and add the following under `[dependencies]`:

```toml
[dependencies]
clap-noun-verb = { version = "5.3.4", features = ["rdf-composition"] }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

The `rdf-composition` feature enables RDF/Turtle and SPARQL support.

## Step 3: Set Up Your First Agent Module

Create `src/lib.rs` with the following:

```rust
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
use clap_noun_verb::rdf::sparql_executor_oxigraph::SparqlExecutor;

/// Our first MCP agent
pub struct MyFirstAgent {
    name: String,
}

impl MyFirstAgent {
    /// Create a new agent
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }

    /// Get agent name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Test Turtle parsing
    pub fn test_parsing(&self) -> Result<(), Box<dyn std::error::Error>> {
        let turtle = r#"
            @prefix cnv: <https://cnv.dev/ontology#> .
            cnv:Services a cnv:Noun ;
                cnv:name "services" .
        "#;

        let parser = TurtleParser::new();
        let parsed = parser.parse(turtle)?;

        println!("✓ Agent {} parsed Turtle successfully!", self.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = MyFirstAgent::new("TestBot");
        assert_eq!(agent.name(), "TestBot");
    }

    #[test]
    fn test_turtle_parsing() {
        let agent = MyFirstAgent::new("TurtleBot");
        assert!(agent.test_parsing().is_ok());
    }
}
```

## Step 4: Build and Test

```bash
cargo build --features rdf-composition
cargo test --lib
```

**Expected output**:
```
   Compiling my-first-agent v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs

running 2 tests
test tests::test_agent_creation ... ok
test tests::test_turtle_parsing ... ok
```

Congratulations! You've successfully:
- ✅ Set up a Rust project with clap-noun-verb
- ✅ Enabled RDF/Turtle support
- ✅ Created your first agent
- ✅ Parsed RDF Turtle syntax
- ✅ Written tests

## Step 5: Understanding the Code

### What We Did

```
Project Structure:
my-first-agent/
├── Cargo.toml           # Project manifest with dependencies
├── src/
│   └── lib.rs          # Our agent implementation
└── target/             # Build artifacts
```

### Key Components

1. **TurtleParser**: Parses RDF Turtle documents into an in-memory graph
   - Feature-gated: `#[cfg(feature = "rdf-composition")]`
   - Returns `Result<ParsedTurtle, TurtleError>`

2. **MyFirstAgent**: Our agent struct
   - Holds agent metadata (name)
   - Methods to interact with Turtle/SPARQL

3. **Tests**: Validation using standard Rust testing
   - Unit tests with `#[test]`
   - No external dependencies needed

## Step 6: Run Your Agent

In a separate window, create `examples/demo.rs`:

```rust
use my_first_agent::MyFirstAgent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = MyFirstAgent::new("DemoBot");
    println!("Welcome! I'm {}", agent.name());

    agent.test_parsing()?;
    println!("\n✓ Setup complete! Ready for Tutorial 2.");

    Ok(())
}
```

Run it:

```bash
cargo run --example demo --features rdf-composition
```

**Expected output**:
```
Welcome! I'm DemoBot
✓ Agent DemoBot parsed Turtle successfully!

✓ Setup complete! Ready for Tutorial 2.
```

## Troubleshooting

### Compilation Error: "feature `rdf-composition` not found"

**Solution**: Make sure you added `features = ["rdf-composition"]` to Cargo.toml:

```toml
clap-noun-verb = { version = "5.3.4", features = ["rdf-composition"] }
                                                   ^^^^^^^^^^^^^^
```

### Error: "failed to parse Turtle"

**Solution**: Check Turtle syntax. Valid Turtle requires:
- Prefixes with `@prefix`
- Statements ending with `.`
- Valid IRIs in angle brackets `<...>`

### Test timeout or hangs

**Solution**: Ensure oxigraph dependency builds successfully. Try:

```bash
cargo clean
cargo build --features rdf-composition
```

## What You Learned

✅ How to set up a Rust project with clap-noun-verb
✅ How to enable RDF/Turtle support via feature flags
✅ How to parse RDF Turtle documents
✅ How to write tests for agent functionality
✅ How to run and verify your agent works

## Next Steps

Ready to create actual ontologies? Continue to:

**→ [Tutorial 2: Create Your First RDF Ontology](tutorial-2-first-rdf.md)**

In the next tutorial, you'll:
- Learn RDF concepts (triples, namespaces, predicates)
- Create a real ontology for a service management CLI
- Understand the cnv: vocabulary
- Validate your ontology

---

**Questions?** Check the [How-to: Validate Ontologies](../howto/validation.md) guide or the [FAQ](../reference/faq.md).

**Need reference?** See [API Reference](../reference/api.md) for all types and functions.

**Want to understand RDF?** Read [Explanation: What is RDF and Why Use It?](../explanation/rdf-basics.md)
