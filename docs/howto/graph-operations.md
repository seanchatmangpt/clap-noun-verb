# How-To: Working with RDF Graphs

**Goal**: Load, query, and validate RDF graphs using the graph module.

## Overview

The graph module provides tools for working with RDF (Resource Description Framework) data:
- **Load** RDF files in multiple formats (Turtle, N-Triples, RDF/XML)
- **Query** graphs using SPARQL-like syntax
- **Validate** graph structure and semantic constraints

This is useful for:
- Semantic knowledge representation
- Ontology management
- Data validation against RDF schemas
- Knowledge graph operations

## Loading RDF Files

### Basic Usage

```bash
# Load an RDF file in Turtle format
myapp graph load --file ontology.ttl

# Load an RDF file in N-Triples format
myapp graph load --file data.nt

# Load and display statistics
myapp graph load --file ontology.ttl --format json
```

### Output

The `graph load` command returns a `GraphLoadedOutput` with:
```json
{
  "file": "ontology.ttl",
  "format": "turtle",
  "triples_loaded": 245,
  "namespaces": ["http://example.org/", "http://www.w3.org/2002/07/owl#"],
  "status": "success"
}
```

## Querying Graphs

### SPARQL-Like Queries

```bash
# Find all subjects and their properties
myapp graph query --sparql "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"

# Find specific types
myapp graph query --sparql "SELECT ?class WHERE { ?class rdf:type rdfs:Class }"

# Find resources with specific properties
myapp graph query --sparql "SELECT ?resource WHERE { ?resource rdfs:label ?label }"
```

### Query Results

Output formats supported:
- `--format json` - Structured JSON results
- `--format table` - ASCII table display
- `--format yaml` - YAML serialization
- `--format tsv` - Tab-separated values (for spreadsheets)

Example query with JSON output:
```bash
myapp graph query --sparql "SELECT ?s ?label WHERE { ?s rdfs:label ?label }" --format json
```

Returns:
```json
{
  "query": "SELECT ?s ?label WHERE { ?s rdfs:label ?label }",
  "results": [
    {"s": "http://example.org/Thing1", "label": "Example Thing"},
    {"s": "http://example.org/Thing2", "label": "Another Thing"}
  ],
  "count": 2
}
```

## Validating Graphs

### Basic Validation

```bash
# Validate a graph against constraints
myapp graph validate --file ontology.ttl

# Validate with detailed reporting
myapp graph validate --file ontology.ttl --strict

# Check specific constraints
myapp graph validate --file ontology.ttl --constraints "no-dangling-refs"
```

### Validation Output

```json
{
  "file": "ontology.ttl",
  "valid": true,
  "issues": [],
  "checks": {
    "referential_integrity": "passed",
    "type_consistency": "passed",
    "property_domains": "passed"
  },
  "warnings": []
}
```

### Common Validation Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| `dangling-reference` | URI referenced but not defined | Add missing resource definition |
| `type-mismatch` | Property used with wrong type | Update type annotations |
| `missing-label` | Resource without rdfs:label | Add label annotation |
| `namespace-collision` | Same URI in multiple namespaces | Consolidate namespace definitions |

## Practical Examples

### Example 1: Load and Inspect an Ontology

```bash
# Load the ontology
myapp graph load --file company-ontology.ttl

# Find all classes defined
myapp graph query --sparql "SELECT ?class WHERE { ?class rdf:type rdfs:Class }"

# Find all properties
myapp graph query --sparql "SELECT ?prop WHERE { ?prop rdf:type rdf:Property }"
```

### Example 2: Validate a Knowledge Graph

```bash
# Load knowledge graph
myapp graph load --file kg.ttl

# Validate structure
myapp graph validate --file kg.ttl --strict

# Find consistency issues
myapp graph query --sparql "SELECT ?s WHERE { ?s rdf:type ?t . ?s ?p ?o FILTER NOT EXISTS { ?t rdfs:domain ?p } }"
```

### Example 3: Export Graph Data

```bash
# Load graph
myapp graph load --file ontology.ttl

# Query and export as table for spreadsheet
myapp graph query --sparql "SELECT ?resource ?label ?type WHERE { ?resource rdfs:label ?label . ?resource rdf:type ?type }" --format table

# Export as TSV
myapp graph query --sparql "SELECT ?s ?p ?o WHERE { ?s ?p ?o LIMIT 100 }" --format tsv > export.tsv
```

## Integration with Other Commands

### Combining with Capability Packing

```bash
# Load domain ontology
myapp graph load --file domain.ttl

# Register as capability pack
myapp pack add --name "domain-ontology" --file domain.ttl

# List all registered ontologies
myapp pack list
```

### System Diagnostics

```bash
# Check graph module health
myapp doctor check --component graph

# View diagnostic details
myapp doctor check --format json
```

## Troubleshooting

### "File not found" Error

```bash
# Ensure file path is correct
ls -l ontology.ttl

# Use absolute path if relative fails
myapp graph load --file /absolute/path/to/ontology.ttl
```

### "Invalid RDF Format" Error

```bash
# Specify format explicitly
myapp graph load --file ontology.rdf --format rdf-xml

# Convert file format first
# (Use external tools like `rapper`)
```

### "Query syntax error" Error

```bash
# Check SPARQL syntax
# Use simple queries first
myapp graph query --sparql "SELECT ?s ?p ?o WHERE { ?s ?p ?o LIMIT 10 }"

# Build up complexity gradually
myapp graph query --sparql "SELECT ?class WHERE { ?class rdf:type rdfs:Class }"
```

## Performance Tips

1. **Load once, query many times**: Load large graphs once and cache results
2. **Use filters**: Add `FILTER` clauses to reduce result sets
3. **Limit results**: Use `LIMIT` in queries for large graphs
4. **Index common patterns**: Pre-compute frequently-used query results

## See Also

- [Capability Packing](capability-packing.md) - Managing graph-based capabilities
- [System Diagnostics](diagnostics.md) - Health checks for graph module
- [Reference: Graph API](../reference/api/graph.md) - Complete API documentation
