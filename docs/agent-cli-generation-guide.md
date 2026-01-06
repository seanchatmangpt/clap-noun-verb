# MCP Agent CLI Generation Guide

## Overview

MCP agents can now write their own CLIs by defining command structures in RDF Turtle format. The clap-noun-verb framework provides three powerful MCP tools that enable agents to:

1. **Parse and validate** RDF Turtle ontologies
2. **Generate production-grade** Rust CLI code automatically
3. **Query capabilities** via SPARQL for intelligent discovery

This guide shows agents how to leverage these tools.

## Quick Start

### Step 1: Define Your CLI in RDF Turtle

Create an ontology that describes your CLI commands using the `clap-noun-verb` vocabulary:

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Define a noun (command category)
cnv:Services a cnv:Noun ;
    cnv:name "services" ;
    rdfs:comment "Service management commands" .

# Define verbs (commands under the noun)
cnv:StatusVerb a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Check service status" ;
    cnv:handler "status_service" .

cnv:StartVerb a cnv:Verb ;
    cnv:name "start" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Start a service" ;
    cnv:handler "start_service" .

cnv:StopVerb a cnv:Verb ;
    cnv:name "stop" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Stop a service" ;
    cnv:handler "stop_service" .
```

### Step 2: Call GenerateCliFromTurtle MCP Tool

Use the MCP tool to transform your Turtle ontology into Rust code:

```json
{
  "tool": "generate_cli_from_turtle",
  "input": {
    "turtle_definition": "<your Turtle content here>",
    "ontology_iri": "https://example.com/my-cli"
  }
}
```

**Response:**
```json
{
  "rust_code": "#[noun(\"services\", \"Service management commands\")]\npub struct Services;\n\n#[verb(Services, \"status\")]\npub async fn status_service(args: &StatusArgs) -> Result<StatusResponse> {\n    // Generated handler\n}\n...",
  "diagnostics": [
    {
      "level": "info",
      "message": "Generated 1 nouns and 3 verbs",
      "line": 1
    }
  ]
}
```

### Step 3: Query Capabilities via SPARQL

Agents can discover what CLIs are available or what operations they support:

```json
{
  "tool": "query_capabilities",
  "input": {
    "sparql_query": "PREFIX cnv: <https://cnv.dev/ontology#> SELECT ?verb WHERE { ?verb a cnv:Verb }",
    "operation": "list_commands"
  }
}
```

**Response:**
```json
{
  "results": [
    "cnv:StatusVerb",
    "cnv:StartVerb",
    "cnv:StopVerb"
  ],
  "found": true
}
```

## Detailed Usage

### MCP Tool #1: GenerateCliFromTurtle

**Purpose:** Transform RDF Turtle ontology definitions into production-grade Rust CLI code.

**Input:**
- `turtle_definition` (string): Complete RDF Turtle document
- `ontology_iri` (string, optional): Ontology IRI for validation

**Output:**
- `rust_code` (string): Generated Rust code with #[noun] and #[verb] macros
- `diagnostics` (array): Parse/generation warnings and info messages
  - `level` (string): "error" | "warning" | "info"
  - `message` (string): Human-readable message
  - `line` (number, optional): Line number for context

**Success Criteria:**
- Turtle parses without syntax errors
- All nouns and verbs are valid identifiers
- Generated code follows Rust syntax rules
- Returns valid Rust module that compiles with clap-noun-verb

**Error Handling:**
- Invalid Turtle syntax → diagnostic with line number
- Invalid Rust identifier → suggests sanitization
- Circular verb definitions → detects and reports
- Missing required fields → diagnostic with suggestion

**Example Agent Usage:**

```python
def generate_my_cli(description: str) -> str:
    """Agent writes CLI description in natural language, convert to Turtle."""
    turtle = convert_to_turtle(description)  # Agent's logic

    response = mcp.call_tool("generate_cli_from_turtle", {
        "turtle_definition": turtle,
        "ontology_iri": "https://my-agent.ai/cli"
    })

    if response["diagnostics"]:
        for diag in response["diagnostics"]:
            if diag["level"] == "error":
                raise Exception(f"Failed: {diag['message']}")

    return response["rust_code"]
```

---

### MCP Tool #2: QueryCapabilities

**Purpose:** Query ontologies using SPARQL to discover available commands and capabilities.

**Input:**
- `sparql_query` (string): SPARQL 1.1 query
- `operation` (string): Query operation hint
  - `"list_commands"`: Find all Verb instances
  - `"find_verb"`: Find specific verb by name
  - `"describe"`: Get full description with properties
  - `"custom"`: Execute arbitrary SPARQL

**Output:**
- `results` (array of strings): Query result bindings or term descriptions
- `found` (boolean): Whether query returned results

**Supported SPARQL Features:**
- SELECT queries with variable binding
- FILTER conditions (regex, string functions)
- OPTIONAL patterns
- UNION alternatives
- COUNT aggregation
- Prefix support (rdf, rdfs, cnv)

**Performance Characteristics:**
- Simple queries: < 10ms
- Complex joins: < 50ms
- Large ontologies: Linear with triple count

**Example Agent Usage:**

```python
def discover_available_commands(ontology_uri: str) -> List[str]:
    """Find all available commands in an ontology."""
    response = mcp.call_tool("query_capabilities", {
        "sparql_query": """
            PREFIX cnv: <https://cnv.dev/ontology#>
            SELECT ?cmd WHERE {
                ?cmd a cnv:Verb ;
                     cnv:name ?cmdName .
            }
        """,
        "operation": "list_commands"
    })
    return response["results"]

def find_verb_by_handler(handler_name: str) -> Optional[str]:
    """Find verb with specific handler."""
    response = mcp.call_tool("query_capabilities", {
        "sparql_query": f"""
            PREFIX cnv: <https://cnv.dev/ontology#>
            SELECT ?verb WHERE {{
                ?verb cnv:handler "{handler_name}" .
            }}
        """,
        "operation": "find_verb"
    })
    return response["results"][0] if response["found"] else None
```

---

### MCP Tool #3: ExportToTurtle

**Purpose:** Convert Rust CLI code back to RDF Turtle (future implementation).

**Status:** Currently returns `ExportNotImplemented` error.

**Expected Input (when implemented):**
- `cli_source_code` (string): Rust CLI source with #[noun] and #[verb] macros
- `cli_name` (string): Name for generated ontology

**Expected Output (when implemented):**
- `turtle_ontology` (string): RDF Turtle representation of CLI

**Use Cases:**
- Round-trip validation: Turtle → Code → Turtle
- Version control: Track CLI changes in RDF format
- Semantic analysis: Analyze CLIs with standard RDF tools
- Integration: Convert existing CLIs to semantic format

---

## Vocabulary Reference

### Core Classes

```turtle
# Noun: A command category/group
cnv:Noun
  a rdf:Class ;
  rdfs:label "Noun" ;
  rdfs:comment "A command category (e.g., 'services', 'config')" ;
  cnv:name "Category name (string)" ;
  cnv:verbs "Associated verbs (range cnv:Verb)" .

# Verb: A command within a noun
cnv:Verb
  a rdf:Class ;
  rdfs:label "Verb" ;
  rdfs:comment "An action/command under a noun" ;
  cnv:name "Command name (string)" ;
  cnv:hasNoun "Associated noun (range cnv:Noun)" ;
  cnv:description "Command description (string)" ;
  cnv:handler "Handler function name (string)" .
```

### Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `cnv:name` | Noun, Verb | xsd:string | Command/noun name (must be valid Rust identifier) |
| `cnv:hasNoun` | Verb | Noun | Parent noun reference |
| `cnv:description` | Noun, Verb | xsd:string | Human-readable description |
| `cnv:handler` | Verb | xsd:string | Function name for implementation |
| `rdfs:comment` | any | xsd:string | Additional documentation |

### Validation Rules

1. **Names must be valid Rust identifiers:**
   - Start with letter or underscore
   - Contain only alphanumeric and underscore
   - Not a Rust keyword (self, type, impl, etc.)

2. **All verbs must reference a noun:**
   - `?verb cnv:hasNoun ?noun` must be present

3. **Names must be unique within scope:**
   - Noun names globally unique
   - Verb names unique per noun

4. **No circular references:**
   - Handler functions must not create verb cycles

---

## Advanced Examples

### Example 1: Multi-Level CLI

Agent creates a CLI with multiple noun categories:

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .

# Configuration management
cnv:Config a cnv:Noun ; cnv:name "config" .
cnv:ShowConfig a cnv:Verb ; cnv:name "show" ; cnv:hasNoun cnv:Config .
cnv:SetConfig a cnv:Verb ; cnv:name "set" ; cnv:hasNoun cnv:Config .

# Data operations
cnv:Data a cnv:Noun ; cnv:name "data" .
cnv:ExportData a cnv:Verb ; cnv:name "export" ; cnv:hasNoun cnv:Data .
cnv:ImportData a cnv:Verb ; cnv:name "import" ; cnv:hasNoun cnv:Data .

# Monitoring
cnv:Monitor a cnv:Noun ; cnv:name "monitor" .
cnv:StatusMonitor a cnv:Verb ; cnv:name "status" ; cnv:hasNoun cnv:Monitor .
cnv:AlertMonitor a cnv:Verb ; cnv:name "alert" ; cnv:hasNoun cnv:Monitor .
```

Result: CLI with commands like:
- `mycli config show`
- `mycli config set`
- `mycli data export`
- `mycli data import`
- `mycli monitor status`
- `mycli monitor alert`

### Example 2: Query to Find Dangerous Operations

Agent discovers which commands modify state:

```python
# Find all verbs that might be dangerous
response = mcp.call_tool("query_capabilities", {
    "sparql_query": """
        PREFIX cnv: <https://cnv.dev/ontology#>
        SELECT ?verb ?name ?desc WHERE {
            ?verb a cnv:Verb ;
                  cnv:name ?name ;
                  cnv:description ?desc .
            FILTER(regex(?desc, "delete|remove|drop|destroy", "i"))
        }
    """,
    "operation": "custom"
})

dangerous_verbs = response["results"]
# Agent can log these for audit trails
```

### Example 3: Dynamic CLI Generation

Agent generates a unique CLI for each user:

```python
def generate_user_cli(user_id: str, permissions: List[str]) -> str:
    """Generate personalized CLI based on user permissions."""

    # Build custom Turtle based on permissions
    turtle = "@prefix cnv: <https://cnv.dev/ontology#> .\n"
    turtle += f"cnv:UserCli_{user_id} a cnv:Noun ; cnv:name \"user{user_id}\" .\n"

    for perm in permissions:
        turtle += f"""
        cnv:{perm}_Verb_{user_id} a cnv:Verb ;
            cnv:name "{perm}" ;
            cnv:hasNoun cnv:UserCli_{user_id} ;
            cnv:description "User-scoped {perm}" .
        """

    # Generate code
    response = mcp.call_tool("generate_cli_from_turtle", {
        "turtle_definition": turtle,
        "ontology_iri": f"https://my-system/{user_id}"
    })

    return response["rust_code"]
```

---

## Type System

### Input Type: TurtleDefinition

```typescript
interface TurtleDefinition {
  turtle_definition: string;      // RDF 1.1 Turtle document
  ontology_iri?: string;          // Optional IRI for metadata
}
```

### Input Type: SparqlQuery

```typescript
interface SparqlQuery {
  sparql_query: string;           // SPARQL 1.1 query string
  operation: "list_commands" | "find_verb" | "describe" | "custom";
}
```

### Output Type: GeneratedCli

```typescript
interface GeneratedCli {
  rust_code: string;              // Generated Rust CLI module
  diagnostics: Diagnostic[];      // Parse/generation warnings
}

interface Diagnostic {
  level: "error" | "warning" | "info";
  message: string;
  line?: number;                  // Optional line reference
  entity?: string;                // Optional entity reference
}
```

### Output Type: QueryResults

```typescript
interface QueryResults {
  results: string[];              // Query result bindings or descriptions
  found: boolean;                 // Whether query returned results
}
```

---

## Error Handling

### Common Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| `ParseError` | Invalid Turtle syntax | Check syntax against W3C Turtle spec |
| `InvalidIdentifier` | Name not valid Rust identifier | Use only alphanumeric + underscore |
| `MissingNoun` | Verb without hasNoun | Add `cnv:hasNoun` property |
| `DuplicateName` | Name used twice in scope | Rename one instance |
| `QueryParseError` | Invalid SPARQL syntax | Validate against SPARQL 1.1 spec |

### Error Response Pattern

```json
{
  "error": {
    "type": "ParseError",
    "message": "Invalid Turtle syntax at line 5: expected '.'",
    "context": {
      "line": 5,
      "input": "cnv:Services a cnv:Noun",
      "suggestion": "Add '.' at end of statement"
    }
  }
}
```

---

## Performance Characteristics

| Operation | Complexity | Typical Time |
|-----------|-----------|--------------|
| Parse Turtle | O(n) where n=triples | < 50ms for typical ontologies |
| Generate CLI code | O(n) where n=commands | < 100ms for 100 commands |
| SPARQL query | O(n) where n=triples | < 10ms for simple queries |
| Export to Turtle | O(n) where n=AST nodes | < 30ms (not yet implemented) |

---

## Best Practices for Agents

### 1. Validate Before Generating

```python
# Check if ontology is valid before code generation
response = mcp.call_tool("query_capabilities", {
    "sparql_query": "SELECT (COUNT(?v) as ?count) WHERE { ?v a ?type }",
    "operation": "custom"
})
if not response["found"]:
    print("Ontology is empty!")
```

### 2. Handle Diagnostics

```python
response = mcp.call_tool("generate_cli_from_turtle", {...})

for diag in response["diagnostics"]:
    if diag["level"] == "error":
        raise GenerationError(diag["message"])
    elif diag["level"] == "warning":
        log.warning(f"Warning: {diag['message']}")
```

### 3. Cache Generated Code

```python
generated_code_cache = {}

def get_or_generate_cli(ontology_hash: str, turtle: str) -> str:
    if ontology_hash in generated_code_cache:
        return generated_code_cache[ontology_hash]

    response = mcp.call_tool("generate_cli_from_turtle", {
        "turtle_definition": turtle
    })

    generated_code_cache[ontology_hash] = response["rust_code"]
    return response["rust_code"]
```

### 4. Query for Validation

```python
# Verify generated CLI has expected commands
created_verbs = mcp.call_tool("query_capabilities", {
    "sparql_query": "SELECT ?v WHERE { ?v a cnv:Verb }",
    "operation": "custom"
})
assert len(created_verbs["results"]) > 0, "No verbs generated!"
```

---

## Future Enhancements

- [ ] ExportToTurtle implementation (reverse engineering)
- [ ] Template-based code generation (custom handlers)
- [ ] Persistent RDF storage (beyond in-memory graphs)
- [ ] Schema validation with SHACL shapes
- [ ] Graphical ontology editor integration
- [ ] Federated SPARQL query support
- [ ] Performance optimization for large ontologies

---

## Support and Resources

- **API Reference**: `/docs/rdf-turtle-api-reference.md`
- **Architecture Guide**: `/docs/rdf-turtle-cli-architecture.md`
- **Examples**: `/examples/playground/rdf-cli-generation/`
- **Test Suite**: `/tests/mcp_turtle_tools_test.rs`

---

Generated for clap-noun-verb v5.3.4
Last updated: 2026-01-06
