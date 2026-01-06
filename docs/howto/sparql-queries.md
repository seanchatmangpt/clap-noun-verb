# How-to: Query with SPARQL

**Problem**: You need to discover CLI capabilities, validate ontologies, and generate code dynamically from RDF

**Solution**: Use SPARQL 1.1 queries to introspect and manipulate ontologies

## Essential SPARQL Queries for CLI Development

### Query 1: Discover All Commands

```sparql
SELECT ?nounName ?verbName ?description WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName .

    OPTIONAL { ?verb cnv:description ?description }
}
ORDER BY ?nounName ?verbName
```

**Use case**: Agents discovering what commands are available
**Expected output**: Complete command hierarchy

### Query 2: Find Commands by Keyword

```sparql
SELECT DISTINCT ?nounName ?verbName WHERE {
    ?noun a cnv:Noun ; cnv:name ?nounName .
    ?verb a cnv:Verb ; cnv:hasNoun ?noun ; cnv:name ?verbName .

    {
        ?verb cnv:description ?desc .
        FILTER (CONTAINS(?desc, "status"))
    }
    UNION
    {
        ?verb cnv:name ?name .
        FILTER (CONTAINS(?name, "status"))
    }
}
```

**Use case**: Agents searching for specific capabilities
**Expected output**: Matching commands

### Query 3: List Commands by Noun

```sparql
SELECT ?nounName (GROUP_CONCAT(?verbName; separator=", ") as ?commands) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName .
}
GROUP BY ?nounName
ORDER BY ?nounName
```

**Use case**: Showing command categories and their verbs
**Expected output**: Grouped command list

### Query 4: Validate Ontology Structure

```sparql
# Find verbs without nouns (orphaned verbs)
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER NOT EXISTS { ?verb cnv:hasNoun ?noun }
}
```

**Use case**: Ensuring ontology consistency
**Expected output**: Should be empty for valid ontologies

### Query 5: Dependency Chain

```sparql
# Find all commands that depend on a specific noun
SELECT ?nounName ?verbName WHERE {
    ?targetNoun a cnv:Noun ;
                cnv:name "services" .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?targetNoun ;
          cnv:name ?verbName .

    ?targetNoun cnv:name ?nounName .
}
```

**Use case**: Understanding command relationships
**Expected output**: All commands under "services"

## Common SPARQL Patterns

### Pattern 1: Count Resources

```sparql
SELECT (COUNT(DISTINCT ?verb) as ?total_commands)
       (COUNT(DISTINCT ?noun) as ?total_nouns) WHERE {
    ?noun a cnv:Noun .
    ?verb a cnv:Verb .
}
```

### Pattern 2: Optional Properties

```sparql
SELECT ?name ?handler ?description WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    OPTIONAL { ?verb cnv:handler ?handler }
    OPTIONAL { ?verb cnv:description ?description }
}
```

### Pattern 3: Filter by Property

```sparql
# Find all commands that have handlers
SELECT ?name ?handler WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name ;
          cnv:handler ?handler .
}
```

### Pattern 4: Complex Filters

```sparql
# Find verbs whose name starts with "get"
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    FILTER (STRSTARTS(?name, "get"))
}
```

### Pattern 5: Regular Expression Matching

```sparql
# Find commands containing "service"
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    FILTER (REGEX(?name, "^serv"))
}
```

### Pattern 6: Union Queries

```sparql
# Get all resources (both nouns and verbs)
SELECT ?resource ?type WHERE {
    {
        ?resource a cnv:Noun .
        BIND("Noun" as ?type)
    }
    UNION
    {
        ?resource a cnv:Verb .
        BIND("Verb" as ?type)
    }
}
```

### Pattern 7: Aggregation

```sparql
# Count verbs per noun
SELECT ?nounName (COUNT(?verb) as ?verb_count) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .
    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun .
}
GROUP BY ?nounName
ORDER BY DESC(?verb_count)
```

### Pattern 8: Path Queries

```sparql
# Find all verbs connected to a noun
SELECT ?verb ?verbName WHERE {
    cnv:Services ^cnv:hasNoun ?verb .
    ?verb cnv:name ?verbName .
}
```

## Using SPARQL from Rust

### Basic Query Execution

```rust
use clap_noun_verb::rdf::sparql_executor::SparqlExecutor;

pub fn execute_sparql_query(
    ontology: &ParsedTurtle,
    query: &str,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let executor = SparqlExecutor::new(ontology)?;
    let results = executor.execute_query(query)?;

    let rows: Vec<_> = results.into_iter()
        .map(|r| r.bindings)
        .collect();

    Ok(rows)
}
```

### Processing Results

```rust
pub fn list_all_commands(
    ontology: &ParsedTurtle,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let query = r#"
    SELECT ?nounName ?verbName WHERE {
        ?noun a cnv:Noun ; cnv:name ?nounName .
        ?verb a cnv:Verb ; cnv:hasNoun ?noun ; cnv:name ?verbName .
    }
    ORDER BY ?nounName ?verbName
    "#;

    let executor = SparqlExecutor::new(ontology)?;
    let results = executor.execute_query(query)?;

    let commands: Vec<_> = results.into_iter()
        .filter_map(|r| {
            let noun = r.get("nounName")?.clone();
            let verb = r.get("verbName")?.clone();
            Some((noun, verb))
        })
        .collect();

    Ok(commands)
}
```

### Error Handling

```rust
match executor.execute_query(query) {
    Ok(results) => {
        println!("Query returned {} results", results.len());
        for row in results {
            println!("{:?}", row.bindings);
        }
    }
    Err(e) => {
        eprintln!("Query error: {}", e);
    }
}
```

## Query Debugging

### Check Query Syntax

```bash
# Use W3C SPARQL validator
# https://www.w3.org/2001/sw/wiki/SparqlQueryValidator

# Or test locally
cargo test -- test_sparql_query_syntax
```

### Inspect Data

Before writing complex queries, see what data exists:

```sparql
# See first 10 triples
SELECT ?s ?p ?o WHERE {
    ?s ?p ?o .
} LIMIT 10
```

### Step-by-Step Testing

```rust
#[test]
fn test_sparql_progressive() {
    // Step 1: Can we find any nouns?
    assert!(query_nouns().is_ok());

    // Step 2: Can we find any verbs?
    assert!(query_verbs().is_ok());

    // Step 3: Can we join them?
    assert!(query_join().is_ok());
}
```

## Performance Optimization

### Index Frequently Queried Properties

```sparql
# Fast: Known pattern
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
}

# Slower: Scan all triples
SELECT ?verb WHERE {
    ?verb ?p ?o .
    FILTER (?p = cnv:Verb)
}
```

### Use Specific Filters

```sparql
# Good: FILTER after pattern matching
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (STRLEN(?name) > 3)
}

# Better: Pattern matching
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (STRSTARTS(?name, "st"))
}
```

## Real-World Examples

### Agent: Discover Available Commands

```rust
pub async fn agent_discover_commands(
    ontology: &ParsedTurtle,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let query = r#"
    SELECT DISTINCT ?verbName WHERE {
        ?verb a cnv:Verb ; cnv:name ?verbName .
    }
    "#;

    let executor = SparqlExecutor::new(ontology)?;
    let results = executor.execute_query(query)?;

    let commands = results.into_iter()
        .filter_map(|r| r.get("verbName").cloned())
        .collect();

    Ok(commands)
}
```

### Agent: Validate Before Generation

```rust
pub async fn agent_validate_ontology(
    ontology: &ParsedTurtle,
) -> Result<bool, Box<dyn std::error::Error>> {
    let executor = SparqlExecutor::new(ontology)?;

    // Check for orphaned verbs
    let orphaned = executor.execute_query(
        "SELECT ?verb WHERE { ?verb a cnv:Verb . FILTER NOT EXISTS { ?verb cnv:hasNoun ?n } }"
    )?;

    Ok(orphaned.is_empty())
}
```

---

**Related**:
- [Tutorial 4: Query Ontologies with SPARQL](../tutorials/tutorial-4-sparql.md)
- [Reference: SPARQL Patterns](../reference/sparql-patterns.md)
- [Explanation: Semantic Web Fundamentals](../explanation/semantic-web.md)
