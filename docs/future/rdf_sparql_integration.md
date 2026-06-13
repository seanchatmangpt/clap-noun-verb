# RDF/SPARQL Semantic Integration Design Document

**Status:** Proposed for v6.0 (Phase 2–3)  
**Timeline:** 2026-06-16 – 2026-07-11

This design document outlines the architecture, semantic mappings, triple representations, and query engine interfaces for integrating Resource Description Framework (RDF) and SPARQL capabilities into the `clap-noun-verb` framework. By turning CLI structures and validation logic into a queryable semantic knowledge graph, `clap-noun-verb` transitions from a traditional parser to an agent-comprehensible, self-documenting, and self-validating command mesh.

---

## 1. Architectural Overview

The RDF/SPARQL integration bridges declarative Rust macros with an in-memory semantic database. Command metadata, parameter requirements, safety constraints, and side effects are extracted at compile-time and loaded into an Oxigraph-backed triple store at runtime.

```mermaid
graph TD
    subgraph Compile-Time Macro Expansion
        A[#[verb] Macro] -->|Analyzes Signature| B[RDF/SHACL Generator]
        B -->|Generates Turtle Strings| C[Distributed Linkme Slice]
    end

    subgraph Runtime Initialization
        C -->|Static Registration| D[__VERB_RDF Slice]
        D -->|On-Demand Load| E[SemanticEngine]
        E -->|Initializes & Caches| F[Oxigraph Store]
    end

    subgraph Interface & Query Layer
        F -->|SPARQL Engine| G[Introspection API]
        F -->|SHACL Validation| H[Semantic Validation Guard]
        G -->|CLI REPL / MCP| I[Agent / Human Interface]
    end
```

### 1.1 Key Goals
*   **Decoupled Introspection**: LLM agents and external services can query the tool capabilities tree directly via RDF formats (Turtle, JSON-LD) or SPARQL queries rather than parsing unstructured text.
*   **Dynamic Intent Resolution**: Translating natural language intents (fuzzy or abstract concepts) to concrete command invocations.
*   **Semantic Guardrails**: Validating overlapping side-effects (e.g. concurrent mutation of same resources) via graph queries prior to execution.
*   **No Runtime Overhead (Disabled State)**: Ensuring the semantic modules compile out completely when the `semantic` feature flag is inactive.

---

## 2. Vocabulary & Semantic Mappings

To represent command-line structures as standard RDF graphs, we map the components of the `clap-noun-verb` API to classes and properties in the `cnv` ontology namespace, aligned with popular W3C and community vocabularies.

### 2.1 Core Namespace and Prefix Mapping
All queries and triple files utilize the following standard vocabulary mappings:
*   `cnv`: `https://cnv.dev/ontology#` (Core framework ontology)
*   `cli`: `https://cnv.dev/cli#` (Generated CLI application instance graph)
*   `rdf`: `http://www.w3.org/1999/02/22-rdf-syntax-ns#`
*   `rdfs`: `http://www.w3.org/2000/01/rdf-schema#`
*   `xsd`: `http://www.w3.org/2001/XMLSchema#`
*   `sh`: `http://www.w3.org/ns/shacl#`

### 2.2 Semantic CLI Class Taxonomy

| Rust Concept | Ontology Class | Description |
| :--- | :--- | :--- |
| `Noun` | `cnv:Noun` | A logical category or domain grouping related operations (e.g., `papers`, `config`). |
| `Verb` / Handler | `cnv:Verb` | An actionable operation performed on a Noun (e.g., `generate`, `list`). |
| Command Path | `cnv:Command` | The concrete execution target formed by binding a Noun and a Verb. |
| Parameter / Argument | `cnv:Argument` | A flag, option, or positional parameter required or accepted by a Command. |
| Output Structure | `cnv:ReturnType` | The structured data schema representing the output of a Command. |
| Side Effects | `cnv:Effect` | The declarative impact category (ReadOnly, MutateState, MutateConfig, etc.). |

### 2.3 Semantic Property Mappings

| RDF Property | Domain | Range | Description |
| :--- | :--- | :--- | :--- |
| `cnv:noun` | `cnv:Command` | `xsd:string` | Links a Command to its Noun namespace. |
| `cnv:verb` | `cnv:Command` | `xsd:string` | Links a Command to its action Verb. |
| `cnv:hasArgument` | `cnv:Command` | `cnv:Argument` | Relates a Command to its parameter options. |
| `cnv:returnsType` | `cnv:Command` | `cnv:ReturnType` | Declares the semantic schema of the return output. |
| `cnv:effectType` | `cnv:Command` | `xsd:string` | Declares the side-effect category. |
| `cnv:resourceTarget`| `cnv:Command` | `xsd:anyURI` | Defines the specific resource path mutated (e.g., `db://users`). |
| `cnv:required` | `cnv:Argument` | `xsd:boolean` | Indicates if an argument must be supplied. |
| `cnv:minValue` | `cnv:Argument` | `xsd:integer` | Specifies numeric minimum boundary constraints. |
| `cnv:maxValue` | `cnv:Argument` | `xsd:integer` | Specifies numeric maximum boundary constraints. |
| `cnv:pattern` | `cnv:Argument` | `xsd:string` | Defines regular expression pattern matches. |

---

## 3. Triple Representation of Commands

When commands are declared via Rust macros, they produce Turtle RDF representing their capabilities and structural layout.

### 3.1 Concrete Command Mapping (Turtle Example)
The following example demonstrates the RDF mapping generated for a database configuration command (`cli:config-set`):

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix cli: <https://cnv.dev/cli#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Command Definition
cli:config-set a cnv:Command ;
    cnv:noun "config" ;
    cnv:verb "set" ;
    rdfs:comment "Set a configuration value for the application." ;
    cnv:effectType "MutateConfig" ;
    cnv:resourceTarget "config://settings.toml" ;
    cnv:hasArgument cli:arg-key, cli:arg-value .

# Key Argument Definition
cli:arg-key a cnv:Argument ;
    cnv:name "key" ;
    cnv:type xsd:string ;
    cnv:required true ;
    rdfs:comment "The configuration key to update." .

# Value Argument Definition
cli:arg-value a cnv:Argument ;
    cnv:name "value" ;
    cnv:type xsd:string ;
    cnv:required true ;
    cnv:minLength 1 ;
    cnv:maxLength 256 ;
    rdfs:comment "The configuration value to store." .
```

### 3.2 SHACL Validation Shape Representation
Complementing the raw data representation, validation shapes are generated to ensure command inputs are validated declaratively at runtime before hitting business logic.

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix cli: <https://cnv.dev/cli#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

cli:config-set-shape a sh:NodeShape ;
    sh:targetNode cli:config-set ;
    sh:property [
        sh:path cnv:hasArgument ;
        sh:name "key" ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] ;
    sh:property [
        sh:path cnv:hasArgument ;
        sh:name "value" ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:minLength 1 ;
        sh:maxLength 256 ;
    ] .
```

---

## 4. Rust Integration & Runtime Architecture

To compile and load these semantic statements efficiently, `clap-noun-verb` utilizes compile-time code generation and link-time assembly.

### 4.1 Static Distributed Slice (`linkme`)
Triples are registered statically during compilation using the `linkme` crate, ensuring that no file system lookups are needed to locate turtle files.

```rust
// In `clap-noun-verb` core:
pub mod rdf {
    pub mod macro_integration {
        use linkme::distributed_slice;

        /// Compile-time registry for CLI RDF triples
        #[distributed_slice]
        pub static __VERB_RDF: [fn() -> (&'static str, &'static str)];
    }
}
```

```rust
// Macro-generated expansion snippet for `#[verb]`:
#[allow(non_upper_case_globals)]
#[linkme::distributed_slice(::clap_noun_verb::rdf::macro_integration::__VERB_RDF)]
static __RDF_CONFIG_SET: fn() -> (&'static str, &'static str) = || {
    (
        // RDF Triples String (Turtle format)
        "cli:config-set a cnv:Command ; cnv:noun \"config\" ; cnv:verb \"set\" .",
        // SHACL Shapes String
        "cli:config-set-shape a sh:NodeShape ; sh:targetNode cli:config-set ."
    )
};
```

### 4.2 SemanticEngine Structure
The `SemanticEngine` manages the `oxigraph::store::Store` instance. To prevent expensive instantiation cycles (~20-50ms) per command execution or query, the ontology store is globally cached using thread-safe structures (`lazy_static` / `once_cell`).

```rust
use std::sync::Arc;
use oxigraph::store::Store;
use parking_lot::RwLock;

/// Core engine managing the RDF triple store
pub struct SemanticEngine {
    store: Arc<Store>,
}

impl SemanticEngine {
    /// Creates and populates a new RDF engine using the compile-time slice
    pub fn new() -> Result<Self, String> {
        let store = Store::new().map_err(|e| e.to_string())?;
        
        // Load registered slices
        for init_fn in crate::rdf::macro_integration::__VERB_RDF {
            let (rdf, shacl) = init_fn();
            store.load_turtle(rdf.as_bytes()).map_err(|e| e.to_string())?;
            store.load_turtle(shacl.as_bytes()).map_err(|e| e.to_string())?;
        }
        
        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Execute a SPARQL query on the cached store
    pub fn execute_query(&self, sparql: &str) -> Result<Vec<Vec<String>>, String> {
        // Implement timeout-aware execution and parse logic using oxigraph::sparql
        // Reference timeout guard rails in section 4.3 below
        self.execute_query_with_timeout(sparql, 5000)
    }
}
```

### 4.3 Fault Tolerance & Timeout Guard Rails
To prevent runaway queries or hung CLI states, queries executed in `SemanticEngine` implement rigid time boundaries (FMEA-5) and fail-safe defaults (FMEA-3):

> [!IMPORTANT]
> **Timeout Boundary**: No SPARQL query may run for more than 5000 milliseconds. If the execution limit is hit, the system triggers graceful degradation, logging warnings and returning empty/partial result arrays instead of crashing or locking the CLI runtime.

---

## 5. Query Endpoints & SPARQL Operations

The CLI runtime exposes key programmatic functions to retrieve schema definitions and resolve execution intents.

### 5.1 Intent Resolution & Discovery
Resolves abstract intents by checking command comments and metadata fields for matches.

*   **SPARQL Query Pattern**:
```sparql
PREFIX cnv: <https://cnv.dev/ontology#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?noun ?verb ?comment
WHERE {
    ?cmd rdf:type cnv:Command ;
         cnv:noun ?noun ;
         cnv:verb ?verb ;
         rdfs:comment ?comment .
    FILTER(CONTAINS(LCASE(?comment), "configuration") || CONTAINS(LCASE(?comment), "update"))
}
```

### 5.2 Side-Effect Overlap & Lock Checking
Detects concurrency risks where two scheduled commands plan to mutate the same physical resource paths.

*   **SPARQL Query Pattern**:
```sparql
PREFIX cnv: <https://cnv.dev/ontology#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?cmdA ?cmdB ?target
WHERE {
    ?cmdA rdf:type cnv:Command ;
          cnv:effectType ?effectA ;
          cnv:resourceTarget ?target .
    ?cmdB rdf:type cnv:Command ;
          cnv:effectType ?effectB ;
          cnv:resourceTarget ?target .
    
    FILTER(?cmdA != ?cmdB)
    FILTER(?effectA IN ("MutateConfig", "MutateState") && ?effectB IN ("MutateConfig", "MutateState"))
}
```

### 5.3 CLI Interface Verification
Queries command parameters, required states, and datatypes.

*   **SPARQL Query Pattern**:
```sparql
PREFIX cnv: <https://cnv.dev/ontology#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?argName ?type ?required
WHERE {
    <https://cnv.dev/cli#config-set> cnv:hasArgument ?arg .
    ?arg cnv:name ?argName ;
         cnv:type ?type ;
         cnv:required ?required .
}
ORDER BY ?argName
```

---

## 6. Future Extensions & Roadmaps

The RDF/SPARQL capabilities lay the path for advanced autonomic CLI operation:
1.  **v5.5: Embedded SPARQL HTTP Endpoint**: Exposing a local HTTP service directly from the interactive REPL shell (`clap-noun-verb repl --sparql-port 8080`) to allow browser-based visualization of command graphs.
2.  **v5.6: Model Context Protocol (MCP) Integration**: Seamless exporting of CLI ontology models to connected AI agents, enabling them to discover tools and construct commands on-the-fly.
3.  **v5.7: Distributed Swarm Orchestration**: Sharing CLI metadata graphs between networked nodes to coordinate command execution across swarm topologies dynamically.
