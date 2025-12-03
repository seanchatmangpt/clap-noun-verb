# Playground Examples

**Experimental and cutting-edge features for exploration**

These examples showcase experimental RDF/semantic features, MCP integration, and research-oriented patterns. Use these for exploring advanced concepts before they become stable APIs.

## Quick Start

```bash
# Try the interactive RDF playground
cargo run --example rdf_interactive_playground

# Explore SPARQL with Oxigraph
cargo run --example rdf_oxigraph_sparql

# Generate an academic paper structure
cargo run --example arxiv_paper_generator
```

## Examples

### rdf_interactive_playground.rs - Interactive RDF Exploration
**Entry point for understanding semantic CLI patterns**

```bash
cargo run --example rdf_interactive_playground
```

**Demonstrates:**
- Building CLI command ontologies with RDF types
- Converting command structures to RDF triples
- SPARQL query patterns for discovering relationships
- SHACL validation for command structure
- MCP memory integration concepts

### rdf_oxigraph_sparql.rs - Production SPARQL Queries
**Real SPARQL 1.1 queries with Oxigraph**

```bash
cargo run --example rdf_oxigraph_sparql
```

**Demonstrates:**
- Production SPARQL 1.1 queries using Oxigraph store
- Complex queries with GROUP BY, COUNT aggregations
- Filtering and pattern matching
- Semantic command relationship analysis

### rdf_mcp_lean.rs - Minimal MCP Integration
**80/20 approach to MCP with RDF**

```bash
cargo run --example rdf_mcp_lean
```

**Demonstrates:**
- Lean MCP integration patterns
- Minimal dependencies approach
- Quick prototyping techniques

### rdf_mcp_core.rs - Full MCP Integration
**Complete MCP + RDF coordination**

```bash
cargo run --example rdf_mcp_core
```

**Demonstrates:**
- Full MCP server integration
- Swarm coordination with RDF agents
- Memory management for ontologies

### thesis_rdf_mcp_80_20.rs - Research Framework
**Academic research CLI patterns**

```bash
cargo run --example thesis_rdf_mcp_80_20
```

**Demonstrates:**
- Thesis/dissertation management
- Citation and reference handling
- Academic workflow patterns

### arxiv_paper_generator.rs - Paper Generation
**Automated academic paper structure**

```bash
cargo run --example arxiv_paper_generator
```

**Demonstrates:**
- ArXiv paper structure generation
- LaTeX template integration
- Metadata management

### conference_management.rs - Conference CLI
**Academic conference management**

```bash
cargo run --example playground_conference
```

### semantic_submissions.rs - Semantic Submissions
**Semantic document handling**

```bash
cargo run --example playground_semantic_submissions
```

### template_generator.rs - Template Generation
**Dynamic template generation**

```bash
cargo run --example playground_template_gen
```

### semantic_cli_hello_world.rs - Semantic Hello World
**Entry point to semantic CLIs**

```bash
cargo run --example playground_semantic_hello
```

### rdf_mcp_server.rs - MCP Server
**Full MCP server implementation**

```bash
cargo run --example playground_rdf_mcp_server
```

### claude_md_config_cli.rs - Claude Config CLI
**Configuration management CLI**

```bash
cargo run --example playground_claude_md
```

### compile_time_validation_demo.rs - Compile-Time Validation
**Validation at compile time**

```bash
cargo run --example playground_compile_validation
```

### telemetry_validation.rs - Telemetry
**Telemetry and observability**

```bash
cargo run --example playground_telemetry
```

## Experimental Status

| Example | Stability | Use Case |
|---------|-----------|----------|
| `rdf_interactive_playground` | Beta | Learning RDF concepts |
| `rdf_oxigraph_sparql` | Beta | Production SPARQL patterns |
| `rdf_mcp_lean` | Alpha | Quick MCP prototyping |
| `rdf_mcp_core` | Alpha | Full MCP integration |
| `thesis_rdf_mcp_80_20` | Alpha | Academic workflows |
| `arxiv_paper_generator` | Alpha | Paper generation |
| `conference_management` | Alpha | Conference workflows |
| `semantic_submissions` | Alpha | Semantic documents |
| `template_generator` | Alpha | Dynamic templates |
| `rdf_mcp_server` | Alpha | MCP server patterns |
| `compile_time_validation` | Alpha | Compile-time checks |
| `telemetry_validation` | Alpha | Observability |

## MCP Integration

The playgrounds use **claude-flow MCP** for coordination:

### Swarm Setup
```rust
// Initialize mesh topology for RDF agents
mcp__claude-flow__swarm_init {
    topology: "mesh",
    maxAgents: 5,
    strategy: "adaptive"
}

// Spawn specialized RDF agents
mcp__claude-flow__agent_spawn {
    type: "researcher",
    name: "rdf-explorer",
    capabilities: ["rdf-analysis", "semantic-web", "graph-exploration"]
}

mcp__claude-flow__agent_spawn {
    type: "coder",
    name: "rdf-builder",
    capabilities: ["rdf-generation", "oxigraph", "sparql"]
}
```

### Memory Storage
```rust
// Store ontology metadata
mcp__claude-flow__memory_usage {
    action: "store",
    key: "rdf/ontology/playground-demo",
    value: {
        "commands": 5,
        "nouns": ["services", "config"],
        "verbs": ["list", "start", "stop", "get", "set"],
        "guards": ["authenticated", "authorized"],
        "effects": ["read-only", "state-change", "idempotent"],
        "triples_count": 72
    },
    namespace: "clap-noun-verb"
}

// Store SPARQL query results
mcp__claude-flow__memory_usage {
    action: "store",
    key: "rdf/playground/sparql-results",
    value: {
        "total_commands": 5,
        "state_change_commands": 3,
        "required_params": 5,
        "guards": {
            "authenticated": 5,
            "authorized": 3
        },
        "noun_verb_matrix": {
            "config": ["get", "set"],
            "services": ["list", "start", "stop"]
        },
        "idempotent_state_changes": 2,
        "queries_executed": 6
    },
    namespace: "clap-noun-verb"
}
```

## 📊 RDF Ontology Structure

The playground builds a semantic CLI ontology using:

### Namespaces
- `cnv:` - https://cnv.dev/ontology# (CLI command ontology)
- `rdf:` - http://www.w3.org/1999/02/22-rdf-syntax-ns#
- `rdfs:` - http://www.w3.org/2000/01/rdf-schema#

### Core Types
- `cnv:Command` - CLI command entity
- `cnv:noun` - Command noun (e.g., "services")
- `cnv:verb` - Command verb (e.g., "list")
- `cnv:hasParameter` - Links to parameter entities
- `cnv:requiresGuard` - Security/precondition guards
- `cnv:hasEffect` - Side effects (state-change, idempotent, read-only)

### Example RDF Triple
```turtle
cnv:services-list rdf:type cnv:Command .
cnv:services-list rdfs:label "services list" .
cnv:services-list cnv:noun "services" .
cnv:services-list cnv:verb "list" .
cnv:services-list cnv:hasEffect "read-only" .
cnv:services-list cnv:requiresGuard "authenticated" .
```

## 🔍 SPARQL Query Patterns

### Pattern 1: Find Commands by Effect
```sparql
SELECT ?cmd WHERE {
    ?cmd cnv:hasEffect "state-change" .
}
```

### Pattern 2: Aggregate Guard Usage
```sparql
SELECT ?guard (COUNT(?cmd) AS ?count) WHERE {
    ?cmd cnv:requiresGuard ?guard .
}
GROUP BY ?guard
ORDER BY DESC(?count)
```

### Pattern 3: Noun-Verb Matrix
```sparql
SELECT ?noun ?verb (COUNT(?cmd) AS ?count) WHERE {
    ?cmd cnv:noun ?noun .
    ?cmd cnv:verb ?verb .
}
GROUP BY ?noun ?verb
```

### Pattern 4: Complex Filtering
```sparql
SELECT ?label WHERE {
    ?cmd rdfs:label ?label .
    ?cmd cnv:hasEffect "state-change" .
    ?cmd cnv:hasEffect "idempotent" .
}
```

## 💡 Use Cases

### 1. Agent Introspection
Agents can query the CLI ontology to discover available commands:
```sparql
SELECT ?cmd ?description WHERE {
    ?cmd rdf:type cnv:Command .
    ?cmd rdfs:description ?description .
}
```

### 2. Guard Validation
Validate that commands meet security requirements:
```sparql
SELECT ?cmd WHERE {
    ?cmd cnv:hasEffect "state-change" .
    FILTER NOT EXISTS { ?cmd cnv:requiresGuard "authorized" }
}
```

### 3. Parameter Discovery
Find all commands requiring specific parameters:
```sparql
SELECT ?cmd ?paramName WHERE {
    ?cmd cnv:hasParameter ?param .
    ?param cnv:paramName ?paramName .
    ?param cnv:required "true" .
}
```

## 🚀 Next Steps

1. **Add More Commands**: Extend `build_cli_ontology()` with your own CLI structure
2. **Custom SPARQL Queries**: Create domain-specific queries for your use case
3. **SHACL Validation**: Define custom SHACL shapes for command validation
4. **MCP Swarm Distribution**: Distribute SPARQL queries across MCP swarm agents
5. **Neural Pattern Training**: Use MCP neural features to learn command patterns

## 📚 References

- **clap-noun-verb RDF docs**: `src/rdf/mod.rs`
- **Oxigraph**: https://github.com/oxigraph/oxigraph
- **SPARQL 1.1**: https://www.w3.org/TR/sparql11-query/
- **RDF 1.1**: https://www.w3.org/TR/rdf11-primer/
- **Claude Flow MCP**: See `.claude-flow/` directory
- **Tutorial Failure Scenarios**: `tutorial-failure-scenarios.md`

## 🎯 Key Insights

1. **RDF enables semantic reasoning** - Commands become queryable knowledge graphs
2. **SPARQL reveals relationships** - Complex patterns emerge through queries
3. **MCP coordinates distributed reasoning** - Swarms can collaboratively explore ontologies
4. **Oxigraph provides production-ready storage** - Full SPARQL 1.1 compliance
5. **Type-safe integration** - Rust's type system ensures correct RDF operations

---

## Next Steps

1. See [tutorial examples](../tutorial/) to learn clap-noun-verb basics
2. Check [advanced examples](../advanced/) for production patterns
3. Explore [reference examples](../reference/) for API details
4. Read [AUTONOMIC.md](../../AUTONOMIC.md) for agent documentation

---

**Status**: ✅ Experimental playground examples
**Version**: clap-noun-verb v5.2.0
**MCP Swarm**: Coordinated with claude-flow@alpha
