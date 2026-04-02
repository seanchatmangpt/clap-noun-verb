# Tutorial 4: Query Ontologies with SPARQL

**Duration**: 20-25 minutes
**Level**: Intermediate
**Prerequisites**: Completed [Tutorial 3: Generate Your First CLI](tutorial-3-first-cli.md)
**Goals**:
- Learn SPARQL query language
- Query your ontology to discover capabilities
- Filter and join data semantically
- Use results to guide code generation

## What is SPARQL?

SPARQL (SPARQL Protocol and RDF Query Language) is the standard way to query RDF graphs. It's like SQL for RDF data.

**SQL example**:
```sql
SELECT name FROM users WHERE age > 21
```

**SPARQL equivalent**:
```sparql
SELECT ?name WHERE {
    ?user rdf:type foaf:Person ;
          foaf:age ?age ;
          foaf:name ?name .
    FILTER (?age > 21)
}
```

## Step 1: Set Up SPARQL Executor

Update `src/lib.rs` to create a SPARQL executor:

```rust
use clap_noun_verb::rdf::sparql_executor::SparqlExecutor;
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
use std::fs;

pub fn query_ontology(turtle_file: &str, query: &str) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    // Load and parse ontology
    let turtle_content = fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&turtle_content)?;

    // Create executor
    let executor = SparqlExecutor::new(&ontology)?;

    // Execute query
    let results = executor.execute_query(query)?;

    // Convert results to rows
    let rows = results.into_iter()
        .map(|r| r.bindings.values().cloned().collect())
        .collect();

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_all_verbs() {
        let query = "SELECT ?verb WHERE { ?verb a cnv:Verb }";
        let result = query_ontology("ontology/services-cli.ttl", query);
        assert!(result.is_ok());
    }
}
```

## Step 2: Basic SPARQL Queries

### Query 1: List All Verbs

```sparql
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
}
```

**Result** (from services-cli.ttl):
```
verb                      | name
--------------------------|--------
cnv:StatusVerb            | status
cnv:StartVerb             | start
cnv:StopVerb              | stop
cnv:RestartVerb           | restart
```

### Query 2: Find Verbs Under Services

```sparql
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:hasNoun cnv:Services ;
          cnv:name ?name .
}
```

**Result**:
```
verb                      | name
--------------------------|--------
cnv:StatusVerb            | status
cnv:StartVerb             | start
cnv:StopVerb              | stop
cnv:RestartVerb           | restart
```

## Step 3: Using FILTER for Conditions

Find verbs that start with "s":

```sparql
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    FILTER (STRSTARTS(?name, "s"))
}
```

**Result**:
```
verb                      | name
--------------------------|--------
cnv:StatusVerb            | status
cnv:StartVerb             | start
cnv:StopVerb              | stop
```

## Step 4: JOIN Operations

Find nouns and their verbs:

```sparql
SELECT ?noun ?nounName ?verb ?verbName WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName .
}
ORDER BY ?nounName ?verbName
```

**Result**:
```
noun            | nounName | verb              | verbName
----------------|----------|-------------------|----------
cnv:Services    | services | cnv:StatusVerb    | status
cnv:Services    | services | cnv:StartVerb     | start
cnv:Services    | services | cnv:StopVerb      | stop
cnv:Services    | services | cnv:RestartVerb   | restart
```

## Step 5: COUNT Aggregations

Count verbs per noun:

```sparql
SELECT ?nounName (COUNT(?verb) as ?verbCount) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun .
}
GROUP BY ?nounName
```

## Step 6: OPTIONAL for Nullable Data

Get all verbs with optional descriptions:

```sparql
SELECT ?verb ?name ?description WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    OPTIONAL {
        ?verb cnv:description ?description .
    }
}
```

## Step 7: Advanced - UNION for Alternatives

Get all classes (both nouns and verbs):

```sparql
SELECT ?resource WHERE {
    {
        ?resource a cnv:Noun
    }
    UNION
    {
        ?resource a cnv:Verb
    }
}
```

## Step 8: Use Query Results for Code Generation

Create a helper to generate code from query results:

```rust
pub fn generate_cli_from_query(turtle_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Query all verbs with their nouns
    let query = r#"
    SELECT ?nounName ?verbName WHERE {
        ?noun a cnv:Noun ; cnv:name ?nounName .
        ?verb a cnv:Verb ;
              cnv:hasNoun ?noun ;
              cnv:name ?verbName .
    }
    ORDER BY ?nounName ?verbName
    "#;

    let results = query_ontology(turtle_file, query)?;

    // Generate code from results
    let mut code = String::new();
    code.push_str("// Generated from SPARQL query results\n\n");

    let mut current_noun = String::new();
    for row in results {
        let noun = row.get(0).cloned().unwrap_or_default();
        let verb = row.get(1).cloned().unwrap_or_default();

        // Start new noun block
        if noun != current_noun {
            if !current_noun.is_empty() {
                code.push_str("}\n\n");
            }
            code.push_str(&format!("#[noun(\"{}\")]\npub struct {};\n\n", noun, noun));
            current_noun = noun;
        }

        // Add verb
        code.push_str(&format!(
            "#[verb({}, \"{}\")]\npub async fn handle_{}_{}_() {{}}\n\n",
            current_noun, verb, current_noun.to_lowercase(), verb
        ));
    }

    if !current_noun.is_empty() {
        code.push_str("}\n");
    }

    Ok(code)
}
```

## Practical Examples

### Example 1: Capability Discovery

An agent can discover what commands are available:

```sparql
SELECT ?nounName (COUNT(?verb) AS ?count) WHERE {
    ?noun a cnv:Noun ; cnv:name ?nounName .
    ?verb a cnv:Verb ; cnv:hasNoun ?noun .
}
GROUP BY ?nounName
ORDER BY DESC(?count)
```

### Example 2: Find Commands by Description

Search for service-related verbs:

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name ;
          cnv:description ?desc .
    FILTER (CONTAINS(?desc, "service"))
}
```

### Example 3: Validate Ontology Structure

Ensure all verbs have required properties:

```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER NOT EXISTS {
        ?verb cnv:name ?name .
    }
}
```

If this returns no results, your ontology is valid!

## Common SPARQL Patterns

### Pattern 1: Filter by Property Value
```sparql
WHERE {
    ?thing ?prop ?value .
    FILTER (?value = "some-value")
}
```

### Pattern 2: Check Property Existence
```sparql
WHERE {
    ?thing a cnv:Verb ;
           cnv:name ?name .
    FILTER EXISTS {
        ?thing cnv:description ?desc .
    }
}
```

### Pattern 3: String Matching
```sparql
WHERE {
    ?thing cnv:name ?name .
    FILTER (REGEX(?name, "^st", "i"))  # Case-insensitive regex
}
```

### Pattern 4: Numeric Comparisons
```sparql
WHERE {
    ?thing cnv:priority ?p .
    FILTER (?p > 5 && ?p < 10)
}
```

## Debugging SPARQL Queries

If a query returns no results:

1. **Check syntax**: Use validator at https://www.w3.org/2001/sw/wiki/SparqlQueryValidator
2. **Verify prefix declarations**: Are `cnv:`, `rdf:`, `rdfs:` defined?
3. **Check data**: Run `SELECT * WHERE { ?s ?p ?o . } LIMIT 10` to see what's in the graph
4. **Validate patterns**: Each `WHERE` block should match triples in your data

## What You Learned

✅ SPARQL query language basics
✅ SELECT queries with filtering
✅ JOIN operations across triples
✅ Aggregation with COUNT and GROUP BY
✅ OPTIONAL for nullable data
✅ Using query results to guide code generation
✅ Ontology validation through SPARQL

## Next Steps

Ready to deploy your generated CLIs to production?

**→ [Tutorial 5: Deploy Production CLIs](tutorial-5-deployment.md)**

In the next tutorial, you'll:
- Package generated code as distributable binaries
- Configure agents to auto-deploy
- Monitor CLI performance in production
- Update ontologies without recompilation

---

## Reference

- [Reference: SPARQL Query Patterns](../reference/sparql-patterns.md)
- [How-to: Query with SPARQL](../howto/sparql-queries.md)
- [Explanation: Semantic Web Fundamentals](../explanation/semantic-web.md)
