# How-to: Debug RDF Issues

**Problem**: Your ontology isn't parsing, queries return empty results, or generated code has errors

**Solution**: Use systematic debugging techniques to identify and fix RDF issues

## Debugging Strategy

### 1. Enable Debug Logging

```rust
// In main.rs or lib.rs
pub fn setup_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}

// In your code
log::debug!("Parsing ontology: {}", path);
log::trace!("Parsed triple: {:?}", triple);
```

Enable at runtime:
```bash
RUST_LOG=debug cargo run
```

### 2. Print Ontology Contents

```rust
pub fn inspect_ontology(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;

    // List all resources
    println!("\n=== Ontology Contents ===\n");

    // Query all subjects
    let query = "SELECT DISTINCT ?s WHERE { ?s ?p ?o }";
    let executor = SparqlExecutor::new(&ontology)?;
    let results = executor.execute_query(query)?;

    println!("Resources found: {}", results.len());
    for result in results {
        if let Some(s) = result.get("s") {
            println!("  - {}", s);
        }
    }

    Ok(())
}
```

## Common Issues and Fixes

### Issue 1: Parse Error - "Unexpected token"

**Symptom**:
```
Parse error at line 15: Unexpected token ';'
```

**Causes**:
1. Missing period at end of statement
2. Missing semicolon between predicates
3. Undefined prefix

**Debug**:
```bash
# Check line 15
sed -n '14,16p' ontology/services-cli.ttl

# Look for syntax issues
grep -n "^[^#]*$" ontology/services-cli.ttl | tail -5
```

**Fix**:
```turtle
# ❌ WRONG - missing period
cnv:Services a cnv:Noun

# ✅ CORRECT
cnv:Services a cnv:Noun .
```

### Issue 2: Undefined Prefix

**Symptom**:
```
Error: Unknown prefix: 'foo'
```

**Debug**:
```rust
pub fn debug_prefixes(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(turtle_file)?;

    // Find used prefixes
    let used: Vec<_> = content.lines()
        .filter_map(|line| {
            if let Some(idx) = line.find(':') {
                Some(&line[..idx])
            } else {
                None
            }
        })
        .collect();

    // Find declared prefixes
    let declared: Vec<_> = content.lines()
        .filter_map(|line| {
            if line.starts_with("@prefix") {
                Some(line)
            } else {
                None
            }
        })
        .collect();

    println!("Declared prefixes:");
    for p in declared {
        println!("  {}", p);
    }

    println!("\nUsed prefixes (first 20):");
    for p in used.iter().take(20) {
        println!("  {}", p);
    }

    Ok(())
}
```

**Fix**:
```turtle
# ❌ WRONG - foo prefix not declared
foo:Services a cnv:Noun .

# ✅ CORRECT - declare prefix first
@prefix foo: <https://example.com/foo#> .
foo:Services a cnv:Noun .
```

### Issue 3: Query Returns Empty Results

**Symptom**:
```
Query executed successfully but no results returned
```

**Debug**:
```rust
pub fn debug_query(
    ontology: &ParsedTurtle,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let executor = SparqlExecutor::new(ontology)?;

    // First, see what data exists
    println!("=== Data in ontology ===");
    let everything = executor.execute_query("SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 20")?;
    for result in everything {
        println!("  {:?}", result.bindings);
    }

    // Now run your query
    println!("\n=== Your query ===");
    println!("Query: {}", query);
    let results = executor.execute_query(query)?;
    println!("Results: {}", results.len());
    for result in results {
        println!("  {:?}", result.bindings);
    }

    Ok(())
}
```

**Common causes**:
1. Wrong variable names (case-sensitive!)
2. Missing namespace prefixes
3. Inverted relationship direction

**Fix**:
```sparql
# ❌ WRONG - cnv:Verb doesn't exist (it's a namespace, not data)
SELECT ?v WHERE { ?v a cnv:Verb }

# ✅ CORRECT - use full IRI or defined prefix
SELECT ?v WHERE { ?v a <https://cnv.dev/ontology#Verb> }
```

### Issue 4: Type Validation Fails

**Symptom**:
```
Validation failed: Invalid type for predicate
```

**Debug**:
```rust
pub fn debug_type_issues(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;

    // Try to validate
    match ontology.validate_ontology() {
        Ok(_) => println!("✅ Validation passed"),
        Err(e) => {
            println!("❌ Validation failed: {}", e);

            // Inspect the problematic triples
            let executor = SparqlExecutor::new(&ontology)?;
            let triples = executor.execute_query("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")?;

            println!("\nFirst 20 triples in graph:");
            for (i, result) in triples.iter().take(20).enumerate() {
                println!("  {}: {:?}", i, result.bindings);
            }
        }
    }

    Ok(())
}
```

### Issue 5: Generated Code Has Errors

**Symptom**:
```
Generated code doesn't compile
```

**Debug**:
```rust
pub fn debug_generation(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;

    // Generate code
    let generator = CliCodeGenerator::new()?;
    let generated = generator.generate_from_ontology(&ontology)?;

    // Show generated code
    println!("=== Generated Code ===");
    println!("{}", generated.rust_code());

    // Check diagnostics
    if !generated.diagnostics().is_empty() {
        println!("\n=== Diagnostics ===");
        for diag in generated.diagnostics() {
            println!("  {} - {}", diag.level, diag.message);
        }
    }

    Ok(())
}
```

## Systematic Debugging Process

### Step 1: Isolate the Problem

```rust
pub async fn debug_workflow(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging workflow...\n");

    // Step 1: Can we read the file?
    println!("1️⃣  Reading file...");
    let content = std::fs::read_to_string(turtle_file)?;
    println!("   ✅ File read successfully ({} bytes)", content.len());

    // Step 2: Can we parse it?
    println!("\n2️⃣  Parsing Turtle...");
    let parser = TurtleParser::new();
    let ontology = match parser.parse(&content) {
        Ok(o) => {
            println!("   ✅ Parse successful");
            o
        }
        Err(e) => {
            println!("   ❌ Parse failed: {}", e);
            return Err(e.into());
        }
    };

    // Step 3: Can we validate?
    println!("\n3️⃣  Validating...");
    match ontology.validate_ontology() {
        Ok(_) => println!("   ✅ Validation passed"),
        Err(e) => {
            println!("   ⚠️  Validation warning: {}", e);
        }
    }

    // Step 4: Can we create executor?
    println!("\n4️⃣  Creating SPARQL executor...");
    let executor = SparqlExecutor::new(&ontology)?;
    println!("   ✅ Executor created");

    // Step 5: Can we query?
    println!("\n5️⃣  Testing SPARQL query...");
    let results = executor.execute_query("SELECT ?v WHERE { ?v a cnv:Verb }")?;
    println!("   ✅ Found {} verbs", results.len());

    // Step 6: Can we generate code?
    println!("\n6️⃣  Generating code...");
    let generator = CliCodeGenerator::new()?;
    let generated = generator.generate_from_ontology(&ontology)?;
    println!("   ✅ Generated {} lines of code", generated.rust_code().lines().count());

    println!("\n✅ All steps completed successfully!");
    Ok(())
}
```

### Step 2: Use Logging

```bash
RUST_LOG=trace cargo test --lib -- --nocapture
```

### Step 3: Create Minimal Reproduction

```turtle
# Start with minimal example
@prefix cnv: <https://cnv.dev/ontology#> .

# Just one noun and verb
cnv:Services a cnv:Noun ; cnv:name "services" .
cnv:Status a cnv:Verb ; cnv:name "status" ; cnv:hasNoun cnv:Services .
```

Test this minimal version, then gradually add complexity.

## Debugging Tools

### Visual RDF Viewer

Create an HTML viewer:

```html
<!DOCTYPE html>
<html>
<head>
    <title>RDF Viewer</title>
</head>
<body>
    <h1>RDF Graph Viewer</h1>
    <pre id="graph"></pre>
    <script>
        // Load Turtle
        fetch('ontology/services-cli.ttl')
            .then(r => r.text())
            .then(text => {
                document.getElementById('graph').textContent = text;
            });
    </script>
</body>
</html>
```

### SPARQL Testing in Browser

Use online SPARQL endpoint: https://dbpedia.org/sparql

## Debugging Checklist

- ✅ File exists and is readable
- ✅ Turtle syntax is valid (period at end of each statement)
- ✅ All prefixes are declared
- ✅ Validation passes
- ✅ Queries return expected results
- ✅ Code generation succeeds
- ✅ Generated code compiles
- ✅ Tests pass

---

**Related**:
- [How-to: Validate Ontologies](validation.md)
- [How-to: Query with SPARQL](sparql-queries.md)
- [Explanation: Semantic Web Fundamentals](../explanation/semantic-web.md)
