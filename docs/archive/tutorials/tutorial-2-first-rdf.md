# Tutorial 2: Create Your First RDF Ontology

**Duration**: 15-20 minutes
**Level**: Beginner
**Prerequisites**: Completed [Tutorial 1: Setup](tutorial-1-setup.md)
**Goals**:
- Understand RDF concepts (triples, subjects, predicates, objects)
- Learn Turtle syntax
- Create your first ontology file
- Validate the ontology

## What is RDF?

RDF (Resource Description Framework) represents data as **triples**:

```
Subject    Predicate       Object
┌──────────┬──────────────┬─────────┐
│ services │ is-a         │ Noun    │
│ status   │ is-a         │ Verb    │
│ status   │ belongs-to   │ services│
└──────────┴──────────────┴─────────┘
```

Each triple answers: **Who (subject) did What (predicate) to Whom (object)?**

## RDF as a Graph

Triples form a directed graph:

```
    services ──(is-a)──> Noun
        │
        └──(has-verb)──> status ──(is-a)──> Verb
```

## Turtle Syntax

Turtle is a human-friendly RDF syntax. Here's the same data in Turtle:

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

cnv:Services rdf:type cnv:Noun ;
    cnv:name "services" .

cnv:Status rdf:type cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services .
```

## Step 1: Create Your Ontology File

Create `my-first-agent/ontology/services-cli.ttl`:

```turtle
# ============================================================================
# Service Management CLI Ontology
# ============================================================================

@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# ============================================================================
# Define the "Services" Noun (command category)
# ============================================================================

cnv:Services a cnv:Noun ;
    cnv:name "services" ;
    rdfs:comment "Commands for managing services" ;
    rdfs:label "Services" .

# ============================================================================
# Define Verbs (commands under Services)
# ============================================================================

# Status command: shows service status
cnv:StatusVerb a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Check status of a service" ;
    cnv:handler "status_service" ;
    rdfs:label "Status" .

# Start command: starts a service
cnv:StartVerb a cnv:Verb ;
    cnv:name "start" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Start a service" ;
    cnv:handler "start_service" ;
    rdfs:label "Start" .

# Stop command: stops a service
cnv:StopVerb a cnv:Verb ;
    cnv:name "stop" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Stop a service" ;
    cnv:handler "stop_service" ;
    rdfs:label "Stop" .

# Restart command: restarts a service
cnv:RestartVerb a cnv:Verb ;
    cnv:name "restart" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Restart a service" ;
    cnv:handler "restart_service" ;
    rdfs:label "Restart" .

# ============================================================================
# End of Ontology
# ============================================================================
```

## Step 2: Understanding the Syntax

Let's break down one triple:

```turtle
cnv:StatusVerb a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services .
```

**Breaks down to:**

```
Subject:        cnv:StatusVerb
Predicate 1:    rdf:type (shorthand: "a")
Object 1:       cnv:Verb

Predicate 2:    cnv:name
Object 2:       "status"

Predicate 3:    cnv:hasNoun
Object 3:       cnv:Services
```

**In English**: "StatusVerb is a Verb, has name 'status', and belongs to the Services noun."

## Step 3: Validate Your Ontology

Update your `src/lib.rs` to test the ontology:

```rust
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
use std::fs;

pub fn load_ontology(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Turtle file
    let turtle_content = fs::read_to_string(file_path)?;

    // Parse it
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle_content)?;

    // Validate it
    parsed.validate_ontology()?;

    println!("✓ Ontology loaded and validated successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_services_ontology() {
        let result = load_ontology("ontology/services-cli.ttl");
        assert!(result.is_ok(), "Failed to load ontology: {:?}", result);
    }
}
```

## Step 4: Build and Test

```bash
cargo test --lib --features rdf-composition
```

**Expected output**:
```
test test_load_services_ontology ... ok
```

## Understanding RDF Namespace Prefixes

### What are Prefixes?

Prefixes are shorthand for long IRIs (Internationalized Resource Identifiers).

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
         ^^^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        prefix      full IRI namespace
```

So `cnv:StatusVerb` expands to: `https://cnv.dev/ontology#StatusVerb`

### Standard Prefixes

| Prefix | Namespace | Purpose |
|--------|-----------|---------|
| `cnv:` | `https://cnv.dev/ontology#` | clap-noun-verb vocabulary |
| `rdf:` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | RDF core |
| `rdfs:` | `http://www.w3.org/2000/01/rdf-schema#` | RDF schema |
| `xsd:` | `http://www.w3.org/2001/XMLSchema#` | Data types |

## RDF Data Types

Turtle supports different data types:

```turtle
cnv:MyNoun a cnv:Noun ;
    cnv:name "Example" ;              # String (default)
    cnv:priority 1 ;                  # Integer
    cnv:enabled true ;                # Boolean
    cnv:version "1.0"^^xsd:string ;   # Explicit string
    cnv:weight 3.14 ;                 # Float
    cnv:created "2026-01-06"^^xsd:date . # Date
```

## Step 5: Explore Your Ontology

Create a simple example that shows what you created:

```rust
pub fn describe_ontology(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let turtle_content = fs::read_to_string(file_path)?;
    let parser = TurtleParser::new();
    let parsed = parser.parse(&turtle_content)?;

    // Validate
    parsed.validate_ontology()?;

    // Describe
    println!("Ontology loaded with {}", file_path);
    println!("  ✓ Valid RDF Turtle");
    println!("  ✓ Prefixes resolved");
    println!("  ✓ Ready for code generation");

    Ok(())
}
```

## Common Mistakes

### ❌ Missing prefix declaration

```turtle
# WRONG: Uses cnv: without declaring it
MyNoun a cnv:Noun .
```

**FIX**: Add the prefix declaration

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
MyNoun a cnv:Noun .
```

### ❌ Missing dots or semicolons

```turtle
# WRONG: No period at end
cnv:Services a cnv:Noun
```

**FIX**: Add period

```turtle
cnv:Services a cnv:Noun .
```

### ❌ Invalid characters in names

```turtle
# WRONG: Spaces and special chars
cnv:My Service Name a cnv:Noun .
```

**FIX**: Use camelCase or underscores

```turtle
cnv:MyServiceName a cnv:Noun .
```

## Debugging

If validation fails, check:

1. **Prefix declarations**: Are all prefixes defined?
   ```bash
   grep "@prefix" ontology/services-cli.ttl
   ```

2. **Syntax**: Are all statements ending with `.`?
   ```bash
   grep -v "\.$" ontology/services-cli.ttl | grep -v "^#" | grep -v "^$"
   ```

3. **IRIs**: Are all IRIs valid?
   - Prefixed: `cnv:Name` ✓
   - Full: `<https://example.com#Name>` ✓
   - Invalid: `Name` ✗

## What You Learned

✅ RDF concepts: triples, subjects, predicates, objects
✅ Turtle syntax: prefixes, statements, semicolons
✅ RDF namespaces and prefixes
✅ cnv: vocabulary for CLI definitions
✅ How to validate ontologies
✅ Common mistakes and how to fix them

## Next Steps

Ready to generate code from your ontology?

**→ [Tutorial 3: Generate Your First CLI](tutorial-3-first-cli.md)**

In the next tutorial, you'll:
- Use the code generator on your ontology
- Produce Rust CLI code
- Compile and run the generated code
- Understand generated code patterns

---

## Reference

- [Explanation: What is RDF and Why Use It?](../explanation/rdf-basics.md) - Deeper understanding
- [Reference: RDF Vocabulary](../reference/vocabulary.md) - All cnv: properties
- [How-to: Validate Ontologies](../howto/validation.md) - Advanced validation

