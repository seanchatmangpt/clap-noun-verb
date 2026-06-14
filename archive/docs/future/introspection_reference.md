# Introspection API Design & Reference Specification

**Status:** Proposed for v6.0 (Phase 2–3)  
**Timeline:** 2026-06-16 – 2026-07-11

This document defines the formal reference specification for the **Introspection API** in the `clap-noun-verb` framework. It outlines the schema formats, metadata structures, CLI command maps, and machine-readable output schemas designed for autonomic controllers, orchestrators, Model Context Protocol (MCP) servers, and LLM agents.

---

## 1. CLI Command Mapping

The Introspection API is exposed via global flags and subcommands. The table below maps these CLI commands to their designated targets, schema models, and purposes:

| CLI Command / Option | Target | Schema Model | Purpose |
|----------------------|--------|--------------|---------|
| `--capabilities` | Application-level capabilities | `CapabilitiesMetadata` | Returns CLI/schema versions and active features |
| `--introspect` | Universal LLM tool registry | `Vec<ToolDefinition>` | Outputs a standard JSON Schema array for LLMs and MCP |
| `--introspect-noun <noun>` | Noun-scoped subset of tools | `Vec<ToolDefinition>` | Filters LLM tool schema to a specific noun namespace |
| `--introspect-domain <domain>` | Domain-scoped tool subset | `Vec<ToolDefinition>` | Filters LLM tool schema by functional domain groups |
| `--graph` | Command dependency topology | `CommandGraphSchema` | Outputs execution preconditions, constraints, and relationships |
| `--receipt-only` | Dry-run structured execution | `ExecutionReceipt` | Runs validations/guards and emits execution receipts without executing |

---

## 2. Core Metadata Schemas

The Introspection API models commands and execution boundaries using structured schemas. Below are the key Rust struct definitions and their JSON equivalents.

### 2.1. Standard LLM Tool Definitions (`--introspect`)

Used to dynamically generate tool definitions compatible with OpenAI, Anthropic, and the Model Context Protocol (MCP).

#### Rust Schema
```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: std::collections::BTreeMap<String, PropertySchema>,
    pub required: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "items")]
    pub items: Option<Box<PropertySchema>>,
}
```

#### JSON Output Example
```json
[
  {
    "name": "services_status",
    "description": "Get the status of a registered service",
    "parameters": {
      "type": "object",
      "properties": {
        "service_name": {
          "type": "string",
          "description": "Name of the target service to query"
        },
        "verbose": {
          "type": "boolean",
          "description": "Enable verbose output logging",
          "default": "false"
        },
        "timeout": {
          "type": "string",
          "description": "Timeout duration in seconds",
          "default": "30"
        }
      },
      "required": [
        "service_name"
      ]
    }
  }
]
```

---

### 2.2. Autonomic Introspection Response (`--capabilities`)

Provides a system-level representation of metadata for MAPE-K loops.

#### Rust Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    pub cli_name: String,
    pub version: String,
    pub description: String,
    pub nouns: Vec<NounMetadata>,
    pub total_capabilities: usize,
    pub autonomic_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NounMetadata {
    pub name: String,
    pub description: String,
    pub verbs: Vec<VerbMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbMetadata {
    pub name: String,
    pub description: String,
    pub effect: String,
    pub args: Vec<ArgSpec>,
    pub capability_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    pub name: String,
    pub arg_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub help: String,
}
```

#### JSON Output Example
```json
{
  "cli_version": "3.8.0",
  "schema_version": "1.0.0",
  "features": ["introspect", "capabilities", "effects", "planes", "guards", "receipts"],
  "app": {
    "name": "myapp",
    "version": "1.0.0",
    "about": "My application"
  }
}
```

---

## 3. Metadata Formats & Behavioral Contracts

Autonomic commands implement the `AutonomicVerbCommand` trait to declare their resource limits, dependency graphs, effects, and plane interactions.

### 3.1. Effect Profiles & Sensitivity Levels
Commands specify their side effects and blast radius so orchestrators can reason about safety before dispatching.

* **Effect Types**:
  * `ReadOnly`: No modifications to system state or configurations.
  * `MutateState`: Modifies runtime/memory system state.
  * `MutateConfig`: Modifies persistent application configs.
  * `MutateOntology`: Modifies schema, models, or types.
  * `MutateSecurity`: Alters access control rules or credentials.
* **Sensitivity Levels**:
  * `Low`: Trivial impact; safe for automatic execution.
  * `Medium`: Moderate impact; logging required.
  * `High`: High impact; verification recommended.
  * `Critical`: Severe impact; user confirmation or air-gapped execution required.

```json
{
  "effect_type": "MutateConfig",
  "sensitivity": "High",
  "idempotent": false
}
```

### 3.2. Conceptual Plane Interactions (O/Σ/Q/ΔΣ)
Reflecting the formal Autonomic Control Plane design, commands document their read/write interactions across four planes:

* **O (Observations)**: Reading telemetry or emitting performance events.
* **Σ (Ontology)**: Interacting with vocabulary schemas or semantic types.
* **Q (Invariants)**: Querying shape constraints or enforcing safety guards.
* **ΔΣ (Overlays)**: Proposing transitions or structural delta changes.

```json
"planes": {
  "O": ["read", "emit"],
  "Σ": ["read"],
  "Q": ["check"],
  "ΔΣ": []
}
```

### 3.3. Guards & Budgets
Resource consumption constraints validated prior to and during command execution:
* **Max Latency**: Deadline boundary in milliseconds.
* **Max Memory**: Heap allocation boundary in kilobytes.
* **Max CPU**: CPU execution time slice in milliseconds.

---

## 4. Execution Receipts & Structured Errors

Upon completing an execution in autonomic mode, the CLI returns structured logs instead of unstructured stdout.

### 4.1. Execution Receipts
A complete record of the resource lifecycle, correlation IDs, and safety checks.

```json
{
  "command": "services status",
  "timestamp": "2026-05-28T19:43:50Z",
  "duration_ms": 42,
  "guard": {
    "enforced": true,
    "latency_ms": 42,
    "max_latency_ms": 100,
    "status": "within_budget"
  },
  "planes": {
    "O": ["read"],
    "Σ": ["read"]
  },
  "correlation_id": "8c0a37db-b7a4-4a41-8608-e8cb9b55ad25",
  "success": true
}
```

### 4.2. Structured Errors
When guards fail or execution invariants are breached, errors are serialized to JSON with relevant debugging contexts.

```json
{
  "error": {
    "kind": "GuardExceeded",
    "message": "Latency guard exceeded threshold",
    "details": {
      "limit_ms": 100,
      "actual_ms": 154
    }
  }
}
```

---

## 5. Schema Validation & SHACL Shapes

The framework compiles Rust macro annotations to RDF triples and SHACL NodeShapes to enforce parameter boundaries.

```turtle
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sh:    <http://www.w3.org/ns/shacl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix cnv:   <https://cnv.dev/ontology#> .
@prefix cli:   <https://cnv.dev/cli#> .

cli:services-status-shape a sh:NodeShape ;
    sh:targetNode cli:services-status ;
    
    sh:property [
        sh:path cnv:argument ;
        sh:name "service_name" ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:minLength 3 ;
        sh:maxLength 64 ;
        sh:pattern "^[a-zA-Z0-9_.-]+$" ;
        sh:description "Name of the target service to query" 
    ] .
```

---

## 6. Execution Contracts & Isolation

For autonomous agent negotiation, commands support execution contracts defining isolation boundaries:

```json
{
  "capability_id": "cnv:services_restart",
  "effect_type": "MutateState",
  "isolation_level": "ReadCommitted",
  "idempotent": false,
  "timeout_ms": 30000
}
```
