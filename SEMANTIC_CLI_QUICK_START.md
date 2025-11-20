# Semantic CLI Quick Start - Implementation Guide

**Purpose**: Get semantic CLI working in clap-noun-verb by Phase 1
**Timeline**: 1-2 weeks
**Complexity**: Medium (builds on existing code)

---

## What is Semantic CLI Control?

Using RDF/SPARQL to represent CLI structure as a **machine-readable knowledge graph**:

```
Traditional:
myapp services status
→ Parsed by clap, executed

Semantic:
myapp services status
→ RDF triple: ex:services_status rdf:type cnv:Verb
→ SPARQL queryable: "Find all verbs with intent 'show-status'"
→ Agents can introspect: "What commands show status?"
```

---

## Why It Matters

| Problem | Solution |
|---------|----------|
| Help text is unstructured | RDF makes it queryable |
| Validation is hardcoded | SPARQL queries can validate |
| Error messages are static | SPARQL can suggest related commands |
| Agents must parse help | Agents can query RDF directly |
| Documentation gets stale | Documentation generated from RDF |

---

## Phase 1: Foundation (What to Build)

### Step 1: Create Semantic Module Structure

```
src/
├── semantic/
│   ├── mod.rs            # Public API
│   ├── schema.rs         # ClnvOntology (class definitions)
│   ├── engine.rs         # SemanticEngine (SPARQL interface)
│   └── builder.rs        # RDF triple builder
└── ... (existing modules)
```

### Step 2: Define ClnvOntology

```rust
// src/semantic/schema.rs

pub const CNV_NAMESPACE: &str = "http://clap-noun-verb.dev/ontology#";

pub struct ClnvOntology;

impl ClnvOntology {
    // Classes
    pub fn verb() -> String { format!("{}Verb", CNV_NAMESPACE) }
    pub fn noun() -> String { format!("{}Noun", CNV_NAMESPACE) }
    pub fn argument() -> String { format!("{}Argument", CNV_NAMESPACE) }
    pub fn return_type() -> String { format!("{}ReturnType", CNV_NAMESPACE) }

    // Properties
    pub fn name() -> String { format!("{}name", CNV_NAMESPACE) }
    pub fn noun_name() -> String { format!("{}nounName", CNV_NAMESPACE) }
    pub fn intent() -> String { format!("{}intent", CNV_NAMESPACE) }
    pub fn argument() -> String { format!("{}hasArgument", CNV_NAMESPACE) }
    pub fn returns_type() -> String { format!("{}returnsType", CNV_NAMESPACE) }
    pub fn related_to() -> String { format!("{}relatedTo", CNV_NAMESPACE) }
    pub fn conflicts_with() -> String { format!("{}conflictsWith", CNV_NAMESPACE) }
}
```

### Step 3: Update Macro to Generate RDF

**Current behavior (unchanged)**:
```rust
#[verb]
fn status() -> Result<Status> { ... }
```

**New behavior (added)**:
Macro generates BOTH:
1. Current code (unchanged)
2. RDF triples (new, stored in binary)

**Implementation approach**:
```rust
// clap-noun-verb-macros/src/lib.rs

pub fn verb(attrs: TokenStream, input: TokenStream) -> TokenStream {
    // 1. Parse input (existing code)
    // 2. Generate verb handler code (existing)
    // 3. GENERATE RDF TRIPLES (new)

    let rdf_triples = generate_rdf_triples(&fn_name, &attributes);
    // Store in __VERB_RDF section

    // 4. Return combined output
}

fn generate_rdf_triples(fn_name: &str, attrs: &VerbAttributes) -> String {
    // Generate Turtle format RDF
    format!(r#"
    @prefix cnv: <http://clap-noun-verb.dev/ontology#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

    ex:{fn_name} a cnv:Verb ;
        cnv:name "{fn_name}" ;
        cnv:noun "{noun}" ;
        rdfs:comment "{doc_comment}" .
    "#)
}
```

### Step 4: Create SemanticEngine

```rust
// src/semantic/engine.rs

#[cfg(feature = "semantic")]
pub struct SemanticEngine {
    graph: Option<oxigraph::store::Store>,
    cache: Arc<Mutex<LruCache<String, String>>>,
}

#[cfg(feature = "semantic")]
impl SemanticEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            graph: Some(oxigraph::store::Store::new()?),
            cache: Arc::new(Mutex::new(LruCache::new(1000.try_into()?))),
        })
    }

    pub fn load_cli_metadata(&mut self, rdf_data: &str) -> Result<()> {
        if let Some(graph) = &mut self.graph {
            graph.load_turtle(rdf_data.as_bytes())?;
        }
        Ok(())
    }

    pub fn discover_by_intent(&self, intent: &str) -> Result<Vec<String>> {
        let query = format!(r#"
            SELECT ?verb WHERE {{
                ?cmd cnv:name ?verb .
                ?cmd cnv:intent ?intent .
                FILTER(CONTAINS(?intent, "{}"))
            }}
        "#, intent);

        self.execute_query(&query)
    }
}
```

### Step 5: Add Feature Flag

```toml
# Cargo.toml

[features]
default = []
semantic = ["oxigraph"]

[dependencies.oxigraph]
version = "0.4"
optional = true
features = ["sparql"]
```

### Step 6: Documentation

Create `docs/SEMANTIC_CLI.md`:
- What it is and why it matters
- Example RDF triples
- SPARQL query patterns
- How to enable the feature
- Use cases (error recovery, discovery, validation)

---

## Phase 1 Checklist

- [ ] Create `src/semantic/` module structure
- [ ] Implement `ClnvOntology` in `schema.rs`
- [ ] Update macro to generate RDF
- [ ] Implement basic `SemanticEngine` in `engine.rs`
- [ ] Add oxigraph as optional dependency
- [ ] Add feature flag `semantic`
- [ ] Create example: `examples/semantic_cli.rs`
- [ ] Write `docs/SEMANTIC_CLI.md`
- [ ] Add tests for RDF generation
- [ ] Verify zero overhead when disabled

**Estimated Lines of Code**:
- Rust: ~800 lines
- Tests: ~300 lines
- Docs: ~500 lines

---

## Code Example: What Gets Generated

### Input (Rust)
```rust
/// Show service status and health
#[verb]
fn status(
    #[arg(short, long)]
    verbose: bool,
) -> Result<Status> {
    Ok(Status { ok: true })
}
```

### Generated RDF (Turtle)
```turtle
@prefix cnv: <http://clap-noun-verb.dev/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:services_status a cnv:Verb ;
    cnv:name "status" ;
    cnv:nounName "services" ;
    cnv:intent "show-status, display-info, health-check" ;
    cnv:hasArgument ex:verbose_arg ;
    cnv:returnsType ex:Status ;
    rdfs:comment "Show service status and health" .

ex:verbose_arg a cnv:Argument ;
    cnv:name "verbose" ;
    cnv:type xsd:boolean ;
    cnv:isFlag true ;
    cnv:shortFlag "v" ;
    cnv:required false ;
    rdfs:comment "Enable verbose output" .
```

### SPARQL Queries on Generated RDF

**Find all verbs with health-related intent:**
```sparql
SELECT ?noun ?verb WHERE {
    ?cmd a cnv:Verb ;
         cnv:intent ?intent ;
         cnv:nounName ?noun ;
         cnv:name ?verb .
    FILTER(CONTAINS(?intent, "health"))
}
```

**Result:**
```
noun     | verb
---------|-------------
services | status
services | health-check
```

---

## Integration Points

### 1. Autonomic Layer

Update `src/autonomic/` to use semantic queries:

```rust
impl CapabilityGraph {
    pub fn discover_by_capability(&self, capability: &str) -> Result<Vec<VerbMetadata>> {
        #[cfg(feature = "semantic")]
        {
            self.semantic_engine.discover_by_intent(capability)
        }
        #[cfg(not(feature = "semantic"))]
        {
            // Fallback to original logic
            self.discover_builtin(capability)
        }
    }
}
```

### 2. Error Handling

Update error handler to suggest related commands:

```rust
fn handle_unknown_command(unknown: &str) -> Result<()> {
    #[cfg(feature = "semantic")]
    {
        let suggestions = semantic_engine.find_similar(unknown)?;
        eprintln!("Did you mean: {}", suggestions.join(", "));
    }
}
```

### 3. Help System

Generate help from RDF:

```rust
fn generate_help(noun: &str) -> String {
    #[cfg(feature = "semantic")]
    {
        semantic_engine.describe_noun(noun)?
    }
    #[cfg(not(feature = "semantic"))]
    {
        builtin_help(noun)
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdf_generation() {
        let rdf = generate_rdf_for_verb("status", "services");
        assert!(rdf.contains("cnv:Verb"));
        assert!(rdf.contains("services"));
    }

    #[test]
    #[cfg(feature = "semantic")]
    fn test_sparql_query() {
        let engine = SemanticEngine::new()?;
        let results = engine.discover_by_intent("status")?;
        assert!(!results.is_empty());
    }
}
```

### Integration Tests

```rust
#[test]
#[cfg(feature = "semantic")]
fn test_semantic_error_recovery() {
    let unknown = "servces";  // Typo
    let suggestions = semantic_engine.find_similar(unknown)?;
    assert!(suggestions.contains(&"services".to_string()));
}
```

---

## Build & Test Commands

```bash
# Build without semantic features (default)
cargo build

# Build with semantic features
cargo build --features semantic

# Test everything
cargo test

# Test only semantic features
cargo test --features semantic

# Check compile time
time cargo build --features semantic

# Verify zero overhead
cargo build
cargo build --features semantic
# Compare binary sizes
```

---

## Success Criteria for Phase 1

1. ✅ RDF generation works for all verb patterns
2. ✅ Can query generated RDF with SPARQL
3. ✅ Feature can be disabled (zero overhead)
4. ✅ Documentation is complete
5. ✅ Examples show real use cases
6. ✅ Tests pass for all patterns
7. ✅ No regression in existing functionality

---

## Next Steps After Phase 1

**Phase 2** (v4.2): Add actual SPARQL queries
- [ ] Discover commands by intent
- [ ] Semantic validation
- [ ] Query caching
- [ ] Performance optimization

**Phase 3** (v4.3): Autonomic integration
- [ ] Use SemanticEngine in autonomic layer
- [ ] Error recovery via SPARQL
- [ ] Automatic help generation

**Phase 4** (v5.0): Advanced features
- [ ] SHACL validation shapes
- [ ] MCP server for RDF
- [ ] Cross-crate semantic linking

---

## Resources

- **Main Architecture**: `SEMANTIC_CLI_RESEARCH_SYNTHESIS.md`
- **Reference Implementation**: `/Users/sac/ggen/crates/ggen-core/src/rdf/`
- **Oxigraph Docs**: https://oxigraph.org/
- **RDF Primer**: https://www.w3.org/TR/rdf11-primer/
- **SPARQL Guide**: https://www.w3.org/TR/sparql11-query/

---

**Ready to start Phase 1?** Begin with implementing `schema.rs` and `ClnvOntology`, then update the macro layer.
