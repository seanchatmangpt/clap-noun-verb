# RDF/SPARQL Integration Research Analysis
## Comprehensive Investigation of clap-noun-verb Semantic CLI Framework

**Research Date**: 2026-01-05
**Project**: clap-noun-verb v5.0.2
**Researcher**: AI Research Agent
**Memory Key**: `rdf_sparql_research`

---

## Executive Summary

clap-noun-verb implements a **graph-native machine interface** where CLI structures are represented as RDF triples, enabling semantic discovery, SPARQL queries, and Model Context Protocol (MCP) integration for autonomous agent coordination. The system converts noun-verb command structures into a queryable knowledge graph with full provenance tracking.

**Key Metrics**:
- **Triple Cardinality**: ~5-7 triples per command capability
- **SPARQL Performance**: <5ms cached queries, 20-50ms cold start
- **MCP Tools**: 4 primary tools (sparql_query, discover_commands, validate_invocation, record_receipt)
- **Ontology Classes**: 8 core classes (Command, Noun, Verb, Argument, Invocation, Receipt, Guard, EffectModel)
- **Query Patterns**: 6 semantic discovery patterns implemented

---

## 1. RDF Ontology Generation Architecture

### 1.1 Compile-Time Generation via Macros

The system uses **procedural macros** to generate RDF metadata at compile-time:

```rust
// Location: clap-noun-verb-macros/src/rdf_generation.rs
#[linkme::distributed_slice]
pub static __VERB_RDF: [fn() -> (&'static str, &'static str)];
```

**Architecture**:
- Each `#[verb]` macro registers an initialization function in a distributed slice
- Functions return `(rdf_triples, shacl_shapes)` as static strings
- Runtime registry collects all metadata using `linkme` for zero-cost abstraction

**Triple Generation Example**:
```turtle
cli:services-status rdf:type cnv:Command ;
    cnv:hasNoun "services" ;
    cnv:hasVerb "status" ;
    cnv:hasArgument cli:services-status_arg_format ;
    rdfs:description "Get service status" .

cli:services-status_arg_format rdf:type cnv:Argument ;
    cnv:name "format" ;
    cnv:datatype "string" ;
    cnv:required "false"^^xsd:boolean .
```

### 1.2 Triple Cardinality Analysis

**Per Command** (typical):
- 1 triple: Command type declaration (`rdf:type cnv:Command`)
- 1 triple: Command name (`cnv:name`)
- 1 triple: Noun association (`cnv:hasNoun`)
- 1 triple: Verb association (`cnv:hasVerb`)
- 1 triple: Description (`rdfs:description`)
- N triples: Arguments (3-5 triples per argument)
- M triples: Guards (1-2 triples per guard)

**Total**: ~5-7 base triples + 3N argument triples + M guard triples

**Example - services-start command**:
```
Base: 5 triples
Arguments: 1 arg × 3 triples = 3 triples
Guards: 2 guards × 1 triple = 2 triples
TOTAL: 10 triples
```

### 1.3 Turtle Format Generation

**Namespace Prefixes**:
```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix shacl: <http://www.w3.org/ns/shacl#> .
```

**Ontology Export**:
```rust
// Location: src/rdf/ontology.rs:118
pub fn to_turtle(&self) -> String {
    // Generates complete Turtle document
    // Includes all prefixes and triples
}
```

---

## 2. SPARQL Query Capabilities

### 2.1 Query Architecture

The system implements a **three-tier query stack**:

1. **Simple Pattern Queries** (Phase 3 - Current):
   - String-based query execution
   - Pattern matching: `LIST NOUNS`, `LIST VERBS`, `FIND COMMAND`
   - LRU cache with 1000-entry capacity

2. **Full SPARQL 1.1 Parser** (Phase 4 - Advanced):
   - Recursive descent parser (`src/rdf/sparql_parser.rs`)
   - Property paths (transitive, inverse, Kleene star)
   - Aggregations (COUNT, SUM, MIN, MAX, AVG)
   - FILTER expressions with logical operators

3. **Oxigraph Integration** (Production):
   - Full SPARQL 1.1 compliance
   - Query timeout: 5000ms default
   - Cached global store (20-50ms initialization avoided)

### 2.2 Query Performance Characteristics

**Benchmarks** (from `/playground/src/integration/rdf.rs`):

| Query Type | Cold Start | Cached | Notes |
|-----------|-----------|--------|-------|
| Simple SELECT | 20-50ms | <5ms | LRU cache hit |
| COUNT aggregation | 30-60ms | <5ms | Store initialization overhead |
| Property path (Kleene*) | 100-200ms | 10-20ms | Transitive closure computation |
| FILTER with CONTAINS | 25-55ms | <5ms | String matching optimization |

**Optimization Strategies**:
- Global cached store (`lazy_static` in playground)
- LRU query cache (1000 entries)
- Predicate indexing for fast lookups
- Subject-based storage with BTreeMap

### 2.3 Query Timeout Handling

**Implementation** (`src/rdf/sparql.rs`):
```rust
pub fn execute(&self, sparql: &str) -> Result<Vec<String>, SparqlError> {
    // Check cache first
    {
        let mut cache = self.cache.lock();
        if let Some(results) = cache.get(&sparql.to_string()) {
            return Ok(results.clone());
        }
    }

    // Execute with graceful degradation
    let results = self.execute_query(sparql)?;

    // Cache results
    self.cache.lock().insert(sparql.to_string(), results.clone());
    Ok(results)
}
```

**Graceful Degradation** (FMEA-3):
- Query parse errors → empty result set
- Execution timeout → partial results returned
- Solution iteration timeout → processed rows returned

---

## 3. MCP (Model Context Protocol) Integration

### 3.1 rmcp Crate Usage

**Official Rust SDK Integration**:
```rust
// Location: src/rdf/rmcp_handler.rs
use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};
use rmcp::ServerHandler;

impl ServerHandler for RdfMcpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "clap-noun-verb-rdf".to_string(),
                version: "5.0.2".to_string(),
                // ...
            },
        }
    }
}
```

### 3.2 MCP Resources Exposed

**Resource URIs**:
1. `ontology:///types` - Available RDF classes and types
2. `ontology:///instances` - All command instances
3. `ontology:///query` - SPARQL query interface
4. `ontology:///receipts` - Execution audit trail with blake3 hashes

**Resource Example**:
```json
{
  "uri": "ontology:///types",
  "name": "Ontology Types",
  "description": "Available classes and types in the ontology",
  "mimeType": "application/sparql-results+json"
}
```

### 3.3 MCP Tools

**Tool Schema**:

#### 1. `sparql_query`
```json
{
  "name": "sparql_query",
  "description": "Execute SPARQL CONSTRUCT/SELECT query against ontology",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "SPARQL query to execute"
      }
    },
    "required": ["query"]
  }
}
```

#### 2. `discover_commands`
```json
{
  "name": "discover_commands",
  "description": "Discover commands matching intent",
  "inputSchema": {
    "type": "object",
    "properties": {
      "intent": {
        "type": "string",
        "description": "Intent description (e.g., 'show status')"
      }
    }
  }
}
```

#### 3. `validate_invocation`
```json
{
  "name": "validate_invocation",
  "description": "Validate command invocation against SHACL guards",
  "inputSchema": {
    "type": "object",
    "properties": {
      "command": { "type": "string" },
      "arguments": { "type": "object" }
    }
  }
}
```

#### 4. `record_receipt`
```json
{
  "name": "record_receipt",
  "description": "Record execution receipt in lockchain",
  "inputSchema": {
    "type": "object",
    "properties": {
      "receipt": {
        "type": "object",
        "description": "Receipt with blake3 hashes"
      }
    }
  }
}
```

### 3.4 Machine-Readable CLI Representation

**MCP Handler Output**:
```rust
pub struct DiscoverCommandsResult {
    pub commands: Vec<String>,
    pub count: usize,
}

pub struct ValidateInvocationResult {
    pub valid: bool,
    pub message: String,
}

pub struct RecordReceiptResult {
    pub receipt_id: String,
    pub command: String,
}
```

**Integration with Claude/LLMs**:
- Stdio-based JSON-RPC protocol
- Structured schema via `schemars::JsonSchema`
- Type-safe tool invocation through rmcp SDK

---

## 4. Semantic Discovery Patterns

### 4.1 Discovery Workflows

**Pattern 1: Intent-Based Discovery**
```rust
pub fn discover_by_intent(&self, intent: &str) -> Result<Vec<String>, SparqlError> {
    let query = format!(
        r#"SELECT ?cmd WHERE {{ ?cmd rdfs:comment ?desc . FILTER(CONTAINS(?desc, "{}")) }}"#,
        intent
    );
    self.execute_raw(&query)
}
```

**Example**:
- Intent: `"show status"`
- SPARQL: `FILTER(CONTAINS(?desc, "show status"))`
- Results: `["services-status", "config-show", "health-status"]`

**Pattern 2: Argument-Based Discovery**
```rust
pub fn find_commands_by_args(&self, arg_names: &[&str]) -> Result<Vec<String>, SparqlError> {
    let query = format!(
        r#"SELECT ?cmd WHERE {{ ?cmd cnv:hasArgument ?arg . ?arg cnv:name "{}" }}"#,
        arg
    );
    self.execute_raw(&query)
}
```

**Pattern 3: Related Command Discovery**
```rust
pub fn get_related_commands(&self, command: &str) -> Result<Vec<String>, SparqlError> {
    let query = format!(
        r#"SELECT ?related WHERE {{
            {{ <{}> cnv:hasNoun ?noun . ?related cnv:hasNoun ?noun }}
            UNION
            {{ <{}> cnv:hasVerb ?verb . ?related cnv:hasVerb ?verb }}
        }}"#,
        command, command
    );
    self.execute_raw(&query)
}
```

**Pattern 4: Effect-Based Discovery**
```sparql
SELECT ?cmd ?label ?effect
WHERE {
    ?cmd rdfs:label ?label .
    ?cmd cnv:hasEffect ?effect .
    FILTER (?effect = "state-change")
}
ORDER BY ?label
```

**Pattern 5: Guard-Based Discovery**
```sparql
SELECT ?guard (COUNT(?cmd) AS ?count)
WHERE {
    ?cmd cnv:requiresGuard ?guard .
}
GROUP BY ?guard
ORDER BY DESC(?count)
```

**Pattern 6: Noun-Verb Matrix**
```sparql
SELECT ?noun ?verb (COUNT(?cmd) AS ?count)
WHERE {
    ?cmd cnv:noun ?noun .
    ?cmd cnv:verb ?verb .
}
GROUP BY ?noun ?verb
ORDER BY ?noun ?verb
```

### 4.2 Capability Exploration for Agents

**Autonomous Agent Workflow**:
1. Query ontology types: `ontology:///types`
2. Discover available commands: `discover_commands("intent")`
3. Validate invocation: `validate_invocation(command, args)`
4. Execute command (external)
5. Record receipt: `record_receipt(receipt)`

**SPARQL Agent Example** (from `examples/playground/semantic_cli_hello_world.rs`):
```rust
// Phase 1: Queen initializes ontology
let ontology = Arc::new(builder.build()?);
let handler = RdfMcpHandler::new(ontology);

// Phase 2: Scouts discover commands
let discovery = handler.discover_commands("greeting")?;

// Phase 3: Workers validate and execute
let validation = handler.validate_invocation("hello-world", &None)?;
let receipt = handler.record_receipt("hello-world", 0)?;
```

---

## 5. RDF Schema Design

### 5.1 Ontology Vocabulary

**Core Classes**:
```turtle
cnv:Command      rdf:type rdfs:Class .
cnv:Noun         rdf:type rdfs:Class .
cnv:Verb         rdf:type rdfs:Class .
cnv:Argument     rdf:type rdfs:Class .
cnv:Invocation   rdf:type rdfs:Class .
cnv:Receipt      rdf:type rdfs:Class .
cnv:Guard        rdf:type rdfs:Class .
cnv:EffectModel  rdf:type rdfs:Class .
```

**Core Properties**:
```turtle
cnv:name            rdf:type rdf:Property .
cnv:nounName        rdf:type rdf:Property .
cnv:verbName        rdf:type rdf:Property .
cnv:hasVerb         rdf:type rdf:Property .
cnv:hasArgument     rdf:type rdf:Property .
cnv:argumentType    rdf:type rdf:Property .
cnv:isOptional      rdf:type rdf:Property .
cnv:invokesCommand  rdf:type rdf:Property .
cnv:exitCode        rdf:type rdf:Property .
cnv:resultHash      rdf:type rdf:Property .
cnv:timestamp       rdf:type rdf:Property .
```

### 5.2 SHACL Shape Validation

**Shape Example**:
```turtle
cnv:CommandShape a sh:NodeShape ;
    sh:targetClass cnv:Command ;
    sh:property [
        sh:path cnv:name ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:datatype xsd:string ;
    ] ;
    sh:property [
        sh:path cnv:hasNoun ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path cnv:hasVerb ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .
```

**Constraint Types Supported**:
- `sh:minCount` / `sh:maxCount` - Cardinality
- `sh:minLength` / `sh:maxLength` - String length
- `sh:minInclusive` / `sh:maxInclusive` - Numeric ranges
- `sh:pattern` - Regular expression matching
- `sh:datatype` - Type validation

### 5.3 Extensibility Strategy

**Custom Domain Integration**:
```rust
// Add domain-specific prefixes
ontology.add_prefix("myapp", "https://myapp.example/");

// Add custom command types
builder.add_command(
    "database-migrate",
    "database",
    "migrate",
    "Run database migrations"
)?;

// Add custom SHACL constraints
builder.add_shape("DatabaseCommandShape", "myapp:DatabaseCommand")?;
```

### 5.4 Schema Versioning

**Approach**: Namespace-based versioning
```turtle
@prefix cnv5: <https://cnv.dev/ontology/v5#> .
@prefix cnv4: <https://cnv.dev/ontology/v4#> .

# Backward compatibility mapping
cnv5:Command owl:equivalentClass cnv4:Command .
```

---

## 6. Real-World Usage Examples

### 6.1 Playground CLI Ontology

**Generated Ontology** (from `examples/playground/rdf_oxigraph_sparql.rs`):

```rust
let commands = vec![
    ("services-list", "services", "list", "List all available services",
     vec![("filter", "string", false), ("verbose", "boolean", false)],
     vec!["authenticated"],
     vec!["read-only"]),

    ("services-start", "services", "start", "Start a service",
     vec![("name", "string", true)],
     vec!["authenticated", "authorized"],
     vec!["state-change", "idempotent"]),

    ("config-set", "config", "set", "Set configuration value",
     vec![("key", "string", true), ("value", "string", true)],
     vec!["authenticated", "authorized"],
     vec!["state-change", "idempotent"]),
];
```

**Generated Triples** (5 commands × ~10 triples = 50 triples):
```turtle
cnv:services-list rdf:type cnv:Command ;
    rdfs:label "services list" ;
    rdfs:description "List all available services" ;
    cnv:noun "services" ;
    cnv:verb "list" ;
    cnv:hasParameter cnv:services-list_param_0 ;
    cnv:requiresGuard "authenticated" ;
    cnv:hasEffect "read-only" .

cnv:services-list_param_0
    cnv:paramName "filter" ;
    cnv:paramType "string" ;
    cnv:required "false" .
```

### 6.2 SPARQL Query Examples

**Query 1: Count All Commands**
```sparql
PREFIX cnv: <https://cnv.dev/ontology#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT (COUNT(?cmd) AS ?count)
WHERE {
    ?cmd rdf:type cnv:Command .
}
```
**Result**: `5`

**Query 2: State-Changing Commands**
```sparql
PREFIX cnv: <https://cnv.dev/ontology#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?cmd ?label ?effect
WHERE {
    ?cmd rdfs:label ?label .
    ?cmd cnv:hasEffect ?effect .
    FILTER (?effect = "state-change")
}
ORDER BY ?label
```
**Results**:
```
services-start | services start | state-change
services-stop  | services stop  | state-change
config-set     | config set     | state-change
```

**Query 3: Commands with Required Parameters**
```sparql
SELECT ?cmd ?label ?paramName
WHERE {
    ?cmd rdfs:label ?label .
    ?cmd cnv:hasParameter ?param .
    ?param cnv:paramName ?paramName .
    ?param cnv:required "true" .
}
ORDER BY ?label ?paramName
```

**Query 4: Idempotent State Changes**
```sparql
SELECT ?label ?noun ?verb
WHERE {
    ?cmd rdfs:label ?label .
    ?cmd cnv:noun ?noun .
    ?cmd cnv:verb ?verb .
    ?cmd cnv:hasEffect "state-change" .
    ?cmd cnv:hasEffect "idempotent" .
}
```

### 6.3 Integration Patterns

**Pattern: Queen-Orchestrated Discovery**
```rust
// 1. Queen initializes ontology
let mut builder = OntologyBuilder::new();
builder.add_command("hello-world", "greeting", "hello", "Display hello")?;
let ontology = Arc::new(builder.build()?);
let handler = RdfMcpHandler::new(ontology);

// 2. Queen queries for greeting commands
let discovery = handler.discover_commands("greeting")?;
println!("Found {} greeting commands", discovery.count);

// 3. Scouts validate each command
for cmd in &discovery.commands {
    let validation = handler.validate_invocation(cmd, &None)?;
    println!("Command {}: {}", cmd, validation.message);
}

// 4. Workers execute with receipt tracking
let receipt = handler.record_receipt("hello-world", 0)?;
println!("Receipt ID: {}", receipt.receipt_id);
```

**Pattern: SPARQL-Driven Workflow**
```rust
// Use Oxigraph for complex semantic queries
let store = Store::new()?;
load_ontology(&store, &capabilities)?;

let query = r#"
    SELECT ?noun ?verb (COUNT(?cmd) AS ?count)
    WHERE {
        ?cmd cnv:noun ?noun .
        ?cmd cnv:verb ?verb .
    }
    GROUP BY ?noun ?verb
"#;

let results = execute_sparql(&store, query)?;
```

---

## 7. Advanced Features

### 7.1 Graph Reasoning Support

**Future Implementation** (not yet supported):
```turtle
# Inference rules (future)
cnv:write-verb rdfs:subClassOf cnv:state-changing-verb .
cnv:read-verb rdfs:subClassOf cnv:read-only-verb .

# OWL reasoning
?cmd cnv:hasVerb ?verb .
?verb rdf:type cnv:write-verb .
=>
?cmd cnv:hasEffect "state-change" .
```

### 7.2 Custom Predicates

**Extensible Property System**:
```rust
// Add custom properties to ontology
ontology.add_triple(RdfTriple::new(
    "cli:database-migrate",
    "myapp:requiresDatabaseConnection",
    RdfValue::typed_literal("true", "xsd:boolean")
));

// Query custom properties
let query = r#"
    SELECT ?cmd WHERE {
        ?cmd myapp:requiresDatabaseConnection "true"^^xsd:boolean .
    }
"#;
```

### 7.3 Cross-CLI Ontology Federation

**Federation Architecture** (future):
```turtle
# Import external CLI ontologies
@prefix kubectl: <https://kubernetes.io/ontology#> .
@prefix docker: <https://docker.com/ontology#> .

# Map relationships
cnv:deploy-service cnv:invokesExternal kubectl:apply .
cnv:build-image cnv:invokesExternal docker:build .
```

**SPARQL Federated Query** (future):
```sparql
SELECT ?localCmd ?externalCmd
WHERE {
    # Local ontology
    ?localCmd cnv:invokesExternal ?externalCmd .

    # Federated query to Kubernetes ontology
    SERVICE <https://kubernetes.io/sparql> {
        ?externalCmd kubectl:apiVersion ?version .
    }
}
```

### 7.4 Provenance Tracking with Blake3 Hashes

**Lockchain Receipt Structure**:
```rust
pub struct LockchainReceipt {
    pub invocation_hash: Blake3Hash,  // Hash of command + args
    pub result_hash: Blake3Hash,       // Hash of execution output
    pub metadata: ReceiptMetadata,
}

pub struct LockchainEntry {
    pub receipt: LockchainReceipt,
    pub chain_hash: Blake3Hash,        // Hash of (prev_hash + receipt)
    pub prev_hash: Option<Blake3Hash>, // Previous chain entry
    pub timestamp: u64,
    pub index: usize,
}
```

**RDF Representation**:
```turtle
receipt:001 rdf:type cnv:Receipt ;
    cnv:invocationHash "blake3:a7f8..." ;
    cnv:resultHash "blake3:3e2d..." ;
    cnv:chainHash "blake3:9c4a..." ;
    cnv:prevHash "blake3:5b1f..." ;
    cnv:timestamp "1704499200"^^xsd:integer ;
    cnv:agentId "worker-007" .
```

**Provenance Query**:
```sparql
SELECT ?receipt ?cmd ?timestamp ?agent
WHERE {
    ?receipt rdf:type cnv:Receipt .
    ?receipt cnv:invocationHash ?hash .
    ?receipt cnv:timestamp ?timestamp .
    ?receipt cnv:agentId ?agent .
    # Reconstruct command from hash
}
ORDER BY DESC(?timestamp)
LIMIT 10
```

---

## 8. Integration Guide for Autonomous Agents

### 8.1 Agent Workflow

**Step 1: Initialize MCP Connection**
```rust
use clap_noun_verb::rdf::{RdfMcpHandler, OntologyBuilder};

let ontology = /* load or generate */;
let handler = RdfMcpHandler::new(ontology);

// Get server info
let info = handler.get_server_info();
println!("Connected to: {} v{}", info.server_info.name, info.server_info.version);
```

**Step 2: Discover Available Commands**
```rust
// Option A: Intent-based discovery
let discovery = handler.discover_commands("show configuration")?;
for cmd in &discovery.commands {
    println!("Found: {}", cmd);
}

// Option B: SPARQL-based discovery
let query = r#"
    SELECT ?cmd ?desc WHERE {
        ?cmd rdfs:description ?desc .
        FILTER(CONTAINS(?desc, "configuration"))
    }
"#;
let results = handler.execute_sparql(query)?;
```

**Step 3: Validate Before Execution**
```rust
let command = "config-set";
let args = json!({
    "key": "timeout",
    "value": "30"
});

let validation = handler.validate_invocation(command, &Some(args))?;
if !validation.valid {
    eprintln!("Validation failed: {}", validation.message);
    return;
}
```

**Step 4: Execute and Record**
```rust
// Execute command (external to MCP handler)
let exit_code = execute_command_externally(command, &args)?;

// Record receipt in lockchain
let receipt = handler.record_receipt(command, exit_code)?;
println!("Execution recorded: {}", receipt.receipt_id);
```

### 8.2 Multi-Agent Coordination

**Queen-Scout-Worker Pattern**:
```rust
// Queen: Orchestrate semantic discovery
let handler = RdfMcpHandler::new(ontology);
let all_commands = handler.discover_commands("")?;

// Scouts: Explore command space in parallel
let scouts: Vec<_> = all_commands.commands
    .chunks(10)
    .map(|chunk| {
        tokio::spawn(async move {
            for cmd in chunk {
                validate_and_explore(cmd).await?;
            }
            Ok(())
        })
    })
    .collect();

// Workers: Execute validated commands
let workers: Vec<_> = validated_commands
    .into_iter()
    .map(|cmd| {
        tokio::spawn(async move {
            execute_with_receipt(cmd).await
        })
    })
    .collect();
```

### 8.3 Semantic Query Examples for Agents

**Agent Decision-Making Query**:
```sparql
# Find safe, read-only commands for exploration
SELECT ?cmd ?desc
WHERE {
    ?cmd rdf:type cnv:Command .
    ?cmd rdfs:description ?desc .
    ?cmd cnv:hasEffect "read-only" .
    FILTER NOT EXISTS { ?cmd cnv:requiresGuard ?guard }
}
```

**Agent Learning Query**:
```sparql
# Discover command relationships for learning
SELECT ?cmd1 ?cmd2 ?sharedNoun
WHERE {
    ?cmd1 cnv:hasNoun ?sharedNoun .
    ?cmd2 cnv:hasNoun ?sharedNoun .
    FILTER(?cmd1 != ?cmd2)
}
GROUP BY ?sharedNoun
```

---

## 9. Performance Optimization Recommendations

### 9.1 Current Bottlenecks

1. **Cold Start Overhead**: 20-50ms store initialization
   - **Solution**: Global cached store with `lazy_static`
   - **Impact**: Reduces to <5ms for subsequent queries

2. **Property Path Queries**: 100-200ms for transitive closure
   - **Solution**: Pre-compute common paths
   - **Impact**: 10x speedup for frequent patterns

3. **Large Result Sets**: Linear serialization cost
   - **Solution**: Streaming results with pagination
   - **Impact**: Constant memory usage

### 9.2 Optimization Strategies

**Strategy 1: Query Plan Optimization**
```rust
// Location: src/rdf/sparql_optimizer.rs
pub struct QueryPlan {
    pub steps: Vec<ExecutionStep>,
    pub estimated_cost: f64,
    pub cardinality_estimates: HashMap<String, usize>,
}

// Reorder triple patterns by selectivity
pub fn optimize_triple_order(patterns: &[TriplePattern]) -> Vec<TriplePattern>;
```

**Strategy 2: Index Selection**
```rust
// Choose optimal join method based on statistics
match stats.estimate_join_cardinality(left, right) {
    n if n < 1000 => JoinMethod::NestedLoop,
    _ => JoinMethod::HashJoin { hash_on: var },
}
```

**Strategy 3: Caching Layers**
```
L1: In-memory LRU cache (1000 entries) - <1ms
L2: Global ontology store (lazy_static) - <5ms
L3: Disk-backed store (optional) - <50ms
```

---

## 10. Conclusion & Recommendations

### Key Findings

1. **Semantic Discovery**: RDF ontology enables rich semantic queries beyond simple text matching
2. **MCP Integration**: Full protocol support for autonomous agent coordination
3. **Performance**: Sub-5ms cached queries suitable for real-time agent decision-making
4. **Extensibility**: Clean separation of schema and implementation allows domain customization
5. **Provenance**: Blake3 hash chains provide cryptographic audit trails

### Recommended Use Cases

**Optimal**:
- Multi-agent CLI discovery and coordination
- Semantic command search and recommendation
- Audit trail and compliance tracking
- Cross-CLI federation and integration

**Suboptimal** (consider alternatives):
- Simple single-command CLIs (overhead not justified)
- Performance-critical hot paths (<1ms latency required)
- Embedded systems with limited memory

### Future Enhancements

1. **Graph Reasoning**: OWL inference for semantic relationships
2. **Federation**: Cross-CLI ontology queries via SPARQL SERVICE
3. **Streaming**: Incremental SPARQL results for large graphs
4. **Compression**: Compact RDF serialization for network transfer

---

## Appendix A: Complete Type Reference

### RDF Types

```rust
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: RdfValue,
}

pub enum RdfValue {
    Uri(String),
    Literal(String),
    TypedLiteral { value: String, datatype: String },
    LangLiteral { value: String, lang: String },
    BlankNode(String),
}

pub struct Invocation {
    pub command: String,
    pub args: BTreeMap<String, String>,
    pub output_format: Option<String>,
    pub metadata: BTreeMap<String, String>,
}
```

### SPARQL Types

```rust
pub struct ParsedQuery {
    pub select_vars: Vec<String>,
    pub where_patterns: Vec<TriplePattern>,
    pub filters: Vec<FilterExpression>,
    pub optional: Vec<TriplePattern>,
    pub unions: Vec<Vec<TriplePattern>>,
    pub group_by: Vec<String>,
    pub aggregations: Vec<Aggregation>,
}

pub enum PropertyPath {
    Direct(String),                           // :p
    Inverse(Box<PropertyPath>),               // ^:p
    Sequence(Box<PropertyPath>, Box<PropertyPath>), // :p/:q
    Alternative(Box<PropertyPath>, Box<PropertyPath>), // :p|:q
    ZeroOrMore(Box<PropertyPath>),            // :p*
    OneOrMore(Box<PropertyPath>),             // :p+
    ZeroOrOne(Box<PropertyPath>),             // :p?
}
```

---

## Appendix B: File Locations

| Component | File Path | Lines |
|-----------|-----------|-------|
| Ontology Core | `/src/rdf/ontology.rs` | 388 |
| Builder | `/src/rdf/builder.rs` | 225 |
| SPARQL Planner | `/src/rdf/sparql.rs` | 583 |
| SPARQL Parser | `/src/rdf/sparql_parser.rs` | 666 |
| SPARQL Executor | `/src/rdf/sparql_executor.rs` | 652 |
| SPARQL Optimizer | `/src/rdf/sparql_optimizer.rs` | ~400 (estimated) |
| MCP Handler (rmcp) | `/src/rdf/rmcp_handler.rs` | 264 |
| MCP Server (stdio) | `/src/rdf/mcp_server.rs` | 537 |
| Macro Integration | `/src/rdf/macro_integration.rs` | 453 |
| RDF Types | `/src/rdf/types.rs` | 208 |
| Playground Example | `/examples/playground/rdf_oxigraph_sparql.rs` | 314 |
| Semantic Hello World | `/examples/playground/semantic_cli_hello_world.rs` | 186 |
| Playground Integration | `/playground/src/integration/rdf.rs` | 251 |
| Tests | `/tests/rdf_ontology_tests.rs` | 271 |

**Total LOC**: ~5,000+ lines of RDF/SPARQL implementation

---

## Appendix C: Memory Storage Format

**Key**: `rdf_sparql_research`

**Value Structure**:
```json
{
  "timestamp": "2026-01-05T00:00:00Z",
  "ontology_architecture": {
    "triple_cardinality": "5-7 base + 3N args + M guards",
    "namespaces": ["cnv", "rdf", "rdfs", "xsd", "shacl"],
    "classes": 8,
    "properties": 11
  },
  "sparql_capabilities": {
    "query_types": ["SELECT", "CONSTRUCT", "ASK"],
    "property_paths": ["transitive", "inverse", "kleene"],
    "aggregations": ["COUNT", "SUM", "MIN", "MAX", "AVG"],
    "performance": {
      "cold_start_ms": "20-50",
      "cached_ms": "<5",
      "timeout_ms": 5000
    }
  },
  "mcp_integration": {
    "sdk": "rmcp",
    "version": "5.0.2",
    "resources": 4,
    "tools": 4,
    "protocol": "stdio JSON-RPC"
  },
  "discovery_patterns": 6,
  "file_locations": { /* ... */ }
}
```

---

**End of Research Report**
