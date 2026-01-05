# RDF Package Integration - Quick Start Guide

**For:** Developers implementing RDF/SPARQL integration
**Based on:** [RDF_PACKAGE_INTEGRATION_REPORT.md](./RDF_PACKAGE_INTEGRATION_REPORT.md)
**Timeline:** 4-6 weeks (Phase 1)

---

## Quick Reference

### Recommended Packages

```toml
[dependencies]
# Feature-gated RDF dependencies
oxigraph = { version = "0.5.1", optional = true, features = ["http-client"] }
json-ld = { version = "0.18", optional = true }
json-ld-serialization = { version = "0.3", optional = true }

[features]
rdf-composition = ["oxigraph", "json-ld", "json-ld-serialization"]
```

### Implementation Priority

| Package | Status | Priority | Use Case |
|---------|--------|----------|----------|
| **oxigraph** | ADOPT | CRITICAL | Primary RDF store + SPARQL engine |
| **json-ld** | ADOPT | HIGH | JSON-LD serialization for MCP |
| **sophia** | EVALUATE | LOW | Optional: Advanced I/O, isomorphism |
| **rio/oxttl** | SKIP | N/A | Use oxigraph's built-in parsers |

---

## Phase 1 Implementation (v5.4.0)

### Step 1: Add Feature Flag

**File:** `Cargo.toml`

```toml
[dependencies]
oxigraph = { version = "0.5.1", optional = true, features = ["http-client"] }
json-ld = { version = "0.18", optional = true }
json-ld-serialization = { version = "0.3", optional = true }

[features]
rdf-composition = ["oxigraph", "json-ld", "json-ld-serialization"]

# Full feature set includes new RDF backend
full = [
    # ... existing features ...
    "rdf-composition"
]
```

### Step 2: Create Oxigraph Backend

**File:** `src/rdf/oxigraph_backend.rs`

```rust
//! Oxigraph-based semantic discovery backend
//!
//! Production-grade RDF store with full SPARQL 1.1 support

use oxigraph::store::Store;
use oxigraph::model::*;
use oxigraph::sparql::QueryResults;
use crate::agents::semantic::{Capability, RdfTriple};
use crate::error::Result;

/// Semantic discovery engine using oxigraph
pub struct SemanticDiscoveryV2 {
    /// Oxigraph RDF store
    store: Store,
    /// Namespace prefix
    namespace: String,
}

impl SemanticDiscoveryV2 {
    /// Create new in-memory semantic discovery engine
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: Store::new()?,
            namespace: "https://cnv.dev/ontology#".to_string(),
        })
    }

    /// Register agent with capabilities
    pub fn register_agent(&mut self, agent_id: &str, capabilities: Vec<Capability>)
        -> Result<()>
    {
        let agent_uri = NamedNode::new(format!("{}{}", self.namespace, agent_id))?;
        let has_cap = NamedNode::new(format!("{}hasCapability", self.namespace))?;
        let has_tag = NamedNode::new(format!("{}hasTag", self.namespace))?;

        for capability in capabilities {
            // Agent hasCapability capability_id
            self.store.insert(&Quad::new(
                agent_uri.clone(),
                has_cap.clone(),
                Literal::new_simple_literal(&capability.id),
                GraphName::DefaultGraph
            ))?;

            // Agent hasTag tag (for each tag)
            for tag in &capability.tags {
                self.store.insert(&Quad::new(
                    agent_uri.clone(),
                    has_tag.clone(),
                    Literal::new_simple_literal(tag),
                    GraphName::DefaultGraph
                ))?;
            }
        }

        Ok(())
    }

    /// Execute SPARQL query
    pub fn query(&self, sparql: &str) -> Result<Vec<String>> {
        let results = self.store.query(sparql)?;

        let mut agent_ids = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;

                // Extract first variable (assume ?agent or ?s)
                if let Some(term) = solution.iter().next() {
                    if let Some(value) = term.1 {
                        // Extract local name from URI
                        let uri_str = value.to_string();
                        if let Some(local) = uri_str.strip_prefix(&self.namespace) {
                            agent_ids.push(local.to_string());
                        } else {
                            agent_ids.push(uri_str);
                        }
                    }
                }
            }
        }

        Ok(agent_ids)
    }

    /// Get agent capabilities (compatibility with v1)
    pub fn get_agent_capabilities(&self, agent_id: &str) -> Result<Vec<Capability>> {
        let query = format!(r#"
            PREFIX cnv: <{}>
            SELECT ?cap WHERE {{
                cnv:{} cnv:hasCapability ?cap .
            }}
        "#, self.namespace, agent_id);

        let results = self.store.query(&query)?;
        let mut capabilities = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution?;
                if let Some(cap_term) = solution.get("cap") {
                    capabilities.push(Capability::new(
                        cap_term.to_string(),
                        String::new() // Description not stored in this version
                    ));
                }
            }
        }

        Ok(capabilities)
    }

    /// Export ontology as Turtle
    pub fn to_turtle(&self) -> Result<String> {
        use oxigraph::io::GraphFormat;
        let mut buffer = Vec::new();
        self.store.dump_graph(
            &mut buffer,
            GraphFormat::Turtle,
            &GraphName::DefaultGraph
        )?;
        Ok(String::from_utf8(buffer)?)
    }
}
```

### Step 3: Update Module Exports

**File:** `src/rdf/mod.rs`

```rust
// Existing modules
pub mod types;
pub mod ontology;

// New oxigraph backend
#[cfg(feature = "rdf-composition")]
pub mod oxigraph_backend;

// Type alias for default backend
#[cfg(feature = "rdf-composition")]
pub use oxigraph_backend::SemanticDiscoveryV2 as DefaultSemanticBackend;

#[cfg(not(feature = "rdf-composition"))]
pub use crate::agents::semantic::SemanticDiscovery as DefaultSemanticBackend;
```

### Step 4: Add Tests

**File:** `tests/oxigraph_integration_tests.rs`

```rust
#![cfg(feature = "rdf-composition")]

use clap_noun_verb::rdf::oxigraph_backend::SemanticDiscoveryV2;
use clap_noun_verb::agents::semantic::Capability;

#[test]
fn test_oxigraph_basic_registration() {
    // Arrange
    let mut discovery = SemanticDiscoveryV2::new().unwrap();
    let caps = vec![Capability::new("nlp", "Natural Language Processing")];

    // Act
    discovery.register_agent("agent-001", caps).unwrap();

    // Assert
    let results = discovery.get_agent_capabilities("agent-001").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "nlp");
}

#[test]
fn test_oxigraph_sparql_query() {
    // Arrange
    let mut discovery = SemanticDiscoveryV2::new().unwrap();
    discovery.register_agent("agent-001", vec![
        Capability::new("nlp", "NLP")
    ]).unwrap();
    discovery.register_agent("agent-002", vec![
        Capability::new("vision", "Vision")
    ]).unwrap();

    // Act
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        SELECT ?agent WHERE {
            ?agent cnv:hasCapability "nlp" .
        }
    "#;
    let results = discovery.query(query).unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "agent-001");
}

#[test]
fn test_oxigraph_complex_sparql() {
    // Arrange
    let mut discovery = SemanticDiscoveryV2::new().unwrap();
    discovery.register_agent("agent-001", vec![
        Capability::new("nlp", "NLP").with_tag("language")
    ]).unwrap();
    discovery.register_agent("agent-002", vec![
        Capability::new("nlp", "NLP").with_tag("language")
    ]).unwrap();

    // Act - Complex query with aggregation
    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        SELECT ?cap (COUNT(?agent) as ?count)
        WHERE {
            ?agent cnv:hasCapability ?cap .
        }
        GROUP BY ?cap
        ORDER BY DESC(?count)
    "#;
    let results = discovery.query(query).unwrap();

    // Assert - Should return aggregated results
    assert!(!results.is_empty());
}

#[test]
fn test_oxigraph_turtle_export() {
    // Arrange
    let mut discovery = SemanticDiscoveryV2::new().unwrap();
    discovery.register_agent("agent-001", vec![
        Capability::new("nlp", "NLP")
    ]).unwrap();

    // Act
    let turtle = discovery.to_turtle().unwrap();

    // Assert
    assert!(turtle.contains("agent-001"));
    assert!(turtle.contains("hasCapability"));
    assert!(turtle.contains("nlp"));
}
```

### Step 5: Add Benchmarks

**File:** `benches/oxigraph_benchmarks.rs`

```rust
#![cfg(feature = "rdf-composition")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use clap_noun_verb::rdf::oxigraph_backend::SemanticDiscoveryV2;
use clap_noun_verb::agents::semantic::Capability;

fn bench_oxigraph_registration(c: &mut Criterion) {
    c.bench_function("oxigraph_register_single_agent", |b| {
        b.iter(|| {
            let mut discovery = SemanticDiscoveryV2::new().unwrap();
            let caps = vec![Capability::new("nlp", "NLP")];
            discovery.register_agent(black_box("agent-001"), caps).unwrap();
            black_box(discovery)
        });
    });
}

fn bench_oxigraph_query(c: &mut Criterion) {
    // Setup
    let mut discovery = SemanticDiscoveryV2::new().unwrap();
    for i in 0..100 {
        discovery.register_agent(
            &format!("agent-{}", i),
            vec![Capability::new(format!("cap-{}", i % 10), "Capability")]
        ).unwrap();
    }

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        SELECT ?agent WHERE {
            ?agent cnv:hasCapability "cap-0" .
        }
    "#;

    c.bench_function("oxigraph_query_100_agents", |b| {
        b.iter(|| {
            let results = discovery.query(black_box(query)).unwrap();
            black_box(results)
        });
    });
}

fn bench_oxigraph_complex_query(c: &mut Criterion) {
    // Setup
    let mut discovery = SemanticDiscoveryV2::new().unwrap();
    for i in 0..100 {
        discovery.register_agent(
            &format!("agent-{}", i),
            vec![
                Capability::new(format!("cap-{}", i % 10), "Cap")
                    .with_tag("tag1")
                    .with_tag("tag2")
            ]
        ).unwrap();
    }

    let query = r#"
        PREFIX cnv: <https://cnv.dev/ontology#>
        SELECT ?cap (COUNT(?agent) as ?count)
        WHERE {
            ?agent cnv:hasCapability ?cap .
            ?agent cnv:hasTag "tag1" .
        }
        GROUP BY ?cap
        ORDER BY DESC(?count)
    "#;

    c.bench_function("oxigraph_complex_query_aggregation", |b| {
        b.iter(|| {
            let results = discovery.query(black_box(query)).unwrap();
            black_box(results)
        });
    });
}

criterion_group!(
    oxigraph_benches,
    bench_oxigraph_registration,
    bench_oxigraph_query,
    bench_oxigraph_complex_query
);
criterion_main!(oxigraph_benches);
```

---

## Testing Checklist

### Build Verification

```bash
# Verify feature compiles
cargo make check --features rdf-composition

# Run tests with new feature
cargo make test --features rdf-composition

# Run benchmarks
cargo make bench --features rdf-composition
```

### Integration Tests

- [ ] Basic agent registration works
- [ ] Simple SPARQL queries execute
- [ ] Complex SPARQL (GROUP BY, ORDER BY) works
- [ ] Error handling is proper (Result types)
- [ ] Turtle export is valid RDF
- [ ] Performance meets SLOs (<100ms)

### Compatibility Tests

- [ ] Existing code works without feature flag
- [ ] Migration from v1 to v2 is seamless
- [ ] API signatures are compatible
- [ ] Error types are consistent

---

## Performance Targets

### Latency SLOs

| Operation | Target | Acceptable |
|-----------|--------|------------|
| Agent registration | <10μs | <50μs |
| Simple SPARQL query | <5μs | <20μs |
| Complex SPARQL query | <50μs | <100μs |
| Turtle export | <100μs | <500μs |

### Verification

```bash
# Run benchmarks and check output
cargo make bench --features rdf-composition | grep "time:"

# Should see:
# oxigraph_register_single_agent   time: [<10 μs]
# oxigraph_query_100_agents        time: [<5 μs]
# oxigraph_complex_query           time: [<50 μs]
```

---

## Common Issues & Solutions

### Issue 1: Compilation Errors

**Error:** `cannot find type Store in module oxigraph::store`

**Solution:**
```toml
# Ensure oxigraph version is correct
oxigraph = { version = "0.5.1", features = ["http-client"] }
```

### Issue 2: SPARQL Query Syntax Errors

**Error:** `SPARQL query failed: syntax error`

**Solution:**
```rust
// Always use proper SPARQL syntax with PREFIX declarations
let query = r#"
    PREFIX cnv: <https://cnv.dev/ontology#>
    SELECT ?agent WHERE {
        ?agent cnv:hasCapability "nlp" .
    }
"#;
```

### Issue 3: Result Extraction

**Error:** `Failed to extract agent IDs from query results`

**Solution:**
```rust
// Extract results correctly based on query variables
if let QueryResults::Solutions(solutions) = results {
    for solution in solutions {
        // Get variable by name (e.g., ?agent)
        if let Some(term) = solution?.get("agent") {
            // Extract value
        }
    }
}
```

---

## Next Steps

### After Phase 1 Completion

1. **Update Examples:**
   - Convert `/examples/semantic_coordinator.rs` to use oxigraph
   - Add new SPARQL query examples

2. **Documentation:**
   - Write migration guide for users
   - Add SPARQL query cookbook
   - Update API documentation

3. **Phase 2 Preparation:**
   - Deprecate `SemanticDiscovery` (v1)
   - Add deprecation warnings
   - Create migration tool/script

---

## Resources

- **Full Report:** [RDF_PACKAGE_INTEGRATION_REPORT.md](./RDF_PACKAGE_INTEGRATION_REPORT.md)
- **Oxigraph Docs:** https://docs.rs/oxigraph
- **SPARQL 1.1 Spec:** https://www.w3.org/TR/sparql11-query/
- **RDF 1.1 Primer:** https://www.w3.org/TR/rdf11-primer/

---

**Quick Start Complete!**

For detailed information, see the full integration report.
