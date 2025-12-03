# ggen v2.0 RDF Engine Analysis: Oxigraph, SPARQL, SHACL, OWL

## Overview

Comprehensive analysis of the RDF, OWL, SPARQL, and SHACL engines in ggen-core, including current implementation, LOC statistics, and v2.0 migration requirements.

**Total LOC in RDF Stack**: ~2,081 lines (excluding tests)  
**Dependencies**: Oxigraph 0.5.1 (SPARQL engine), custom SHACL validator

---

## Core Components

### 1. Graph Store (`graph.rs`) - 657 LOC

**Purpose**: Thread-safe Oxigraph wrapper with SPARQL query caching

**Key Features**:
- ✅ **RDF Loading**: Turtle (.ttl), N-Triples (.nt), RDF/XML (.rdf)
- ✅ **SPARQL Query Execution**: SELECT, CONSTRUCT, ASK queries
- ✅ **Query Caching**: LRU cache for query plans and results
- ✅ **Graph Operations**: Insert triples, filter patterns, clear graph
- ✅ **Thread-Safe**: Arc-based cloning, epoch-based cache invalidation

**Implementation Details**:
```rust
// Core structure
pub struct Graph {
    inner: Store,                          // Oxigraph store
    epoch: Arc<AtomicU64>,                 // Cache invalidation epoch
    plan_cache: Arc<Mutex<LruCache<u64, String>>>,      // Query plan cache
    result_cache: Arc<Mutex<LruCache<(u64, u64), CachedResult>>>, // Result cache
}

// Key methods
- load_from_file()      // Load RDF file
- insert_turtle()       // Insert Turtle RDF
- query_cached()         // Execute SPARQL with caching
- query()                // Direct SPARQL query
- quads_for_pattern()    // Filter triples by pattern
```

**LOC Breakdown**:
- Graph structure and initialization: ~100 LOC
- RDF loading and parsing: ~150 LOC
- SPARQL query execution: ~200 LOC
- Caching logic: ~150 LOC
- Tests: ~57 LOC

**v2.0 Changes**:
- ✅ **KEEP**: All existing functionality (core engine)
- ✅ **ENHANCE**: Support for SPARQL CONSTRUCT queries (already supported)
- ✅ **ENHANCE**: Better error messages for RDF loading failures
- ⚠️ **DEPRECATED**: Using deprecated `Store::query()` API (line 207, 234, 256)
  - Migration note: "will migrate to SparqlEvaluator post-1.0"

---

### 2. Template Metadata (`rdf/template_metadata.rs`) - 601 LOC

**Purpose**: RDF-based metadata storage and querying for templates

**Key Features**:
- ✅ **Metadata Extraction**: Extract metadata from template frontmatter
- ✅ **RDF Serialization**: Convert metadata to Turtle RDF format
- ✅ **SPARQL Querying**: Query template metadata using SPARQL
- ✅ **Relationship Management**: Template dependencies, extensions, overrides

**Implementation Details**:
```rust
// Core structure
pub struct TemplateMetadata {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub variables: Vec<TemplateVariable>,
    pub generated_files: Vec<String>,
    pub dependencies: Vec<String>,
    // ... more fields
}

pub struct TemplateMetadataStore {
    graph: Arc<Graph>,                    // Shared graph store
    // ... metadata storage
}
```

**LOC Breakdown**:
- Metadata structures: ~100 LOC
- RDF serialization (to_turtle): ~200 LOC
- SPARQL querying: ~150 LOC
- Relationship management: ~100 LOC
- Tests: ~50 LOC

**v2.0 Changes**:
- ✅ **KEEP**: All existing functionality
- ✅ **ENHANCE**: Support for RDF-based template discovery (filesystem routing)
- ⚠️ **REMOVE**: Dependencies on frontmatter parsing (moved to CLI)

---

### 3. SHACL Validation (`rdf/validation.rs`) - 521 LOC

**Purpose**: SHACL-based validation for template metadata

**Key Features**:
- ✅ **Shape Definitions**: SHACL shape definitions for templates
- ✅ **Property Constraints**: Min/max count, datatype, pattern validation
- ✅ **Validation Reports**: Detailed error and warning reporting
- ✅ **Severity Levels**: Error, Warning, Info classifications

**Implementation Details**:
```rust
// Core structures
pub struct Validator {
    shapes: HashMap<String, Shape>,       // SHACL shapes
}

struct Shape {
    target_class: String,
    properties: Vec<PropertyConstraint>,  // Property constraints
}

pub struct ValidationReport {
    pub template_id: String,
    pub result: ValidationResult,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}
```

**Default Shapes**:
- `TemplateShape`: Validates template metadata structure
- `VariableShape`: Validates template variable definitions
- `FileShape`: Validates generated file metadata

**LOC Breakdown**:
- Validator structure: ~50 LOC
- Shape definitions: ~200 LOC
- Validation logic: ~150 LOC
- Error reporting: ~100 LOC
- Tests: ~20 LOC

**v2.0 Changes**:
- ✅ **KEEP**: All existing functionality
- ✅ **ENHANCE**: Support for validating RDF data from CLI (not just frontmatter)
- ✅ **ENHANCE**: Add shape definitions for v2.0 RDF structures (commands, business logic, etc.)

---

### 4. RDF Schema (`rdf/schema.rs`) - 225 LOC

**Purpose**: Ggen ontology definitions and namespace management

**Key Features**:
- ✅ **Namespace Definitions**: Ggen, RDF, RDFS, XSD, OWL namespaces
- ✅ **Ontology Classes**: Template, File, Variable, Directory, Artifact, etc.
- ✅ **Ontology Properties**: Relationships, metadata properties
- ✅ **Type Definitions**: Property URIs for RDF serialization

**Implementation Details**:
```rust
// Core structure
pub const GGEN_NAMESPACE: &str = "http://ggen.dev/ontology#";
pub const OWL_NAMESPACE: &str = "http://www.w3.org/2002/07/owl#";

pub struct GgenOntology;

impl GgenOntology {
    // Classes
    pub fn template() -> String;
    pub fn file() -> String;
    pub fn variable() -> String;
    
    // Properties
    pub fn has_variable() -> String;
    pub fn generates_file() -> String;
    // ... more properties
}
```

**Namespace Support**:
- `http://ggen.dev/ontology#` - Ggen ontology
- `http://www.w3.org/1999/02/22-rdf-syntax-ns#` - RDF
- `http://www.w3.org/2000/01/rdf-schema#` - RDFS
- `http://www.w3.org/2001/XMLSchema#` - XSD
- `http://www.w3.org/2002/07/owl#` - OWL

**LOC Breakdown**:
- Namespace constants: ~20 LOC
- Class definitions: ~100 LOC
- Property definitions: ~100 LOC

**v2.0 Changes**:
- ✅ **KEEP**: All existing functionality
- ✅ **ENHANCE**: Add v2.0-specific ontology classes:
  - `Noun`, `Verb` classes for clap-noun-verb commands
  - `BusinessLogic` class for business logic files
  - `FrozenSection` class for frozen sections
  - `TemplateQuery` class for SPARQL queries in templates

---

### 5. Template Processing (`template.rs`) - 882 LOC (RDF-related)

**Purpose**: Template rendering with RDF/SPARQL integration

**Key Features** (RDF-related):
- ✅ **RDF Loading**: Load RDF from frontmatter or files
- ✅ **SPARQL Execution**: Execute SPARQL queries in templates
- ✅ **Query Results**: Inject SPARQL results into template context
- ⚠️ **Frontmatter Dependencies**: Currently loads RDF from frontmatter (v2.0: remove)

**Current Implementation** (Lines 200-294):
```rust
// Current: RDF loading from frontmatter
for rdf_file in &self.front.rdf {  // ❌ v2.0: Remove this
    // Load RDF file
    graph.insert_turtle(&ttl_content)?;
}

// Current: SPARQL execution
for (name, q) in &self.front.sparql {
    let results = graph.query(&final_q)?;
    // Inject into template context
    self.front.sparql_results.insert(name.clone(), json_result);
}
```

**v2.0 Changes**:
- ❌ **REMOVE**: Lines 73-75 (`rdf: Vec<String>`) from Frontmatter struct
- ❌ **REMOVE**: Lines 200-255 (RDF loading from frontmatter)
- ✅ **KEEP**: Lines 257-291 (SPARQL query execution) - move to API parameter
- ✅ **ENHANCE**: Add `render_with_rdf(rdf_files: Vec<PathBuf>) -> Result<String>` method

**LOC to Remove**:
- Frontmatter RDF fields: ~10 LOC
- RDF loading logic: ~55 LOC
- Total: ~65 LOC to remove

**LOC to Keep**:
- SPARQL execution: ~35 LOC (refactor to accept RDF via API)

---

## SPARQL Query Support

### Query Types Supported

1. **SELECT Queries** ✅
   - Extract data for template rendering
   - Most common query type
   - Results: JSON array of objects

2. **CONSTRUCT Queries** ✅
   - Transform RDF into new graph
   - Generate derived properties
   - Results: RDF graph (not currently exposed to templates)

3. **ASK Queries** ✅
   - Boolean queries
   - Results: true/false

4. **UPDATE Operations** ⚠️
   - Not currently used in templates
   - Supported by Oxigraph

### Current Usage in Templates

**Pattern**:
```yaml
---
sparql:
  get_classes: |
    PREFIX owl: <http://www.w3.org/2002/07/owl#>
    SELECT ?class WHERE { ?class a owl:Class }
  get_properties: |
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT ?property ?type WHERE {
      ?property a rdf:Property ;
                rdfs:range ?type .
    }
---
{{ sparql_results.get_classes | length }} classes found
{% for class in sparql_results.get_classes %}
- {{ class.class }}
{% endfor %}
```

**v2.0 Pattern** (No frontmatter):
```rust
// Template file (no frontmatter)
{% for class in query('get_classes') %}
- {{ class.class }}
{% endfor %}

// SPARQL queries in separate file or ggen.toml
[queries]
get_classes = """
PREFIX owl: <http://www.w3.org/2002/07/owl#>
SELECT ?class WHERE { ?class a owl:Class }
"""
```

---

## OWL Support

### Current OWL Usage

**Namespace Support**:
- ✅ OWL namespace defined in `schema.rs`
- ✅ OWL classes and properties accessible in SPARQL queries
- ⚠️ No explicit OWL reasoning (relies on Oxigraph's implicit support)

**Common OWL Patterns**:
```sparql
# Get classes
SELECT ?class WHERE { ?class a owl:Class }

# Get subclasses
SELECT ?subclass WHERE {
  ?subclass rdfs:subClassOf ?superclass .
}

# Get properties
SELECT ?property WHERE {
  ?property a owl:ObjectProperty .
}
```

**v2.0 Enhancements**:
- ✅ **KEEP**: Existing OWL namespace support
- ✅ **ENHANCE**: Document OWL patterns in templates
- ✅ **ENHANCE**: Support OWL ontologies in RDF files loaded via CLI

---

## SHACL Validation

### Current SHACL Implementation

**Custom Validator** (not full SHACL engine):
- ✅ Shape definitions (hardcoded in Rust)
- ✅ Property constraints (min/max count, datatype, pattern)
- ⚠️ Not a full SHACL processor (no SHACL-SPARQL, no advanced features)

**Validation Shapes**:
1. **TemplateShape**: Validates template metadata
   - Required: `templateName`
   - Optional: `templateVersion` (with pattern validation)
   - Optional: `templateDescription`

2. **VariableShape**: Validates template variables
   - Required: `variableName`
   - Optional: `variableType`, `variableDefault`

3. **FileShape**: Validates generated file metadata
   - Optional: `filePath`, `fileExtension`

**v2.0 Enhancements**:
- ✅ **KEEP**: Existing validation logic
- ✅ **ENHANCE**: Add shapes for v2.0 structures:
  - `NounShape`: Validates noun command structure
  - `VerbShape`: Validates verb command structure
  - `BusinessLogicShape`: Validates business logic file references
- ✅ **ENHANCE**: Support validating RDF data from CLI (not just metadata)

---

## Dependencies

### External Crates

1. **oxigraph** (0.5.1)
   - SPARQL engine
   - RDF store
   - Turtle/N-Triples/RDF/XML parsing

2. **shacl_validation** (0.1) ⚠️
   - Listed in Cargo.toml but not used
   - Current SHACL validation is custom implementation

3. **srdf** (0.1) ⚠️
   - Listed in Cargo.toml but not used
   - Might be for future RDF serialization

### Internal Dependencies

- `anyhow` - Error handling
- `serde` / `serde_json` - Serialization
- `lru` - LRU cache for query results
- `ahash` - Fast hashing for cache keys

---

## LOC Statistics Summary

**Total RDF Engine LOC**: ~2,081 lines (excluding tests)

| Component | LOC | Tests | Total |
|-----------|-----|-------|-------|
| `graph.rs` | 600 | 57 | 657 |
| `rdf/template_metadata.rs` | 551 | 50 | 601 |
| `rdf/validation.rs` | 501 | 20 | 521 |
| `rdf/schema.rs` | 225 | 0 | 225 |
| `template.rs` (RDF parts) | 65 | 0 | 65 |
| **Total** | **1,942** | **127** | **2,069** |

**v2.0 Changes**:
- **Remove**: ~65 LOC (frontmatter RDF loading)
- **Add**: ~100 LOC (RDF via CLI, enhanced shapes)
- **Net Change**: +35 LOC (enhancements exceed removals)

---

## v2.0 Migration Requirements

### Phase 1: Remove Frontmatter RDF Dependencies

**Files to Modify**:
1. `ggen-core/src/template.rs`
   - Remove `rdf: Vec<String>` from Frontmatter struct (line 73-75)
   - Remove RDF loading logic (lines 200-255)
   - Add `render_with_rdf(rdf_files: Vec<PathBuf>) -> Result<String>` method

**Action Items**:
- [ ] Remove `rdf:` field from `Frontmatter` struct
- [ ] Remove RDF file loading from `load_rdf_data()` method
- [ ] Refactor SPARQL execution to accept RDF via API parameter
- [ ] Update tests to use RDF via CLI instead of frontmatter

### Phase 2: Enhance RDF Engine for v2.0

**Files to Enhance**:
1. `ggen-core/src/rdf/schema.rs`
   - Add v2.0 ontology classes (Noun, Verb, BusinessLogic, FrozenSection)
   - Add v2.0 property definitions

2. `ggen-core/src/rdf/validation.rs`
   - Add v2.0 SHACL shapes (NounShape, VerbShape, BusinessLogicShape)
   - Support validating RDF data from CLI

3. `ggen-core/src/graph.rs`
   - Update deprecated `Store::query()` API usage
   - Add support for multiple RDF file merging

**Action Items**:
- [ ] Add v2.0 ontology classes to `GgenOntology`
- [ ] Add v2.0 SHACL shapes to `Validator`
- [ ] Migrate to `SparqlEvaluator` API (post-1.0)
- [ ] Add RDF file merging support

### Phase 3: Template Engine Integration

**Files to Modify**:
1. `ggen-core/src/templates/generator.rs`
   - Add RDF loading via CLI parameter
   - Support filesystem routing for RDF discovery
   - Integrate SPARQL queries from `ggen.toml` or separate files

**Action Items**:
- [ ] Add RDF loading via CLI parameter to template generation
- [ ] Support RDF discovery from `domain/` directory
- [ ] Support SPARQL query discovery from `queries/` directory
- [ ] Integrate query execution with template rendering

---

## Performance Considerations

### Query Caching

**Current Implementation**:
- LRU cache for query plans (100 entries)
- LRU cache for query results (1,000 entries)
- Epoch-based cache invalidation

**Performance Impact**:
- ✅ Caching significantly improves repeated query performance
- ✅ Plan caching reduces query parsing overhead
- ✅ Result caching eliminates redundant SPARQL execution

**v2.0 Enhancements**:
- Consider increasing cache sizes for large projects
- Add cache hit/miss metrics
- Support cache warming for common queries

### Graph Store Operations

**Current Implementation**:
- In-memory Oxigraph store
- Thread-safe with Arc-based cloning
- Efficient triple storage and indexing

**Performance Impact**:
- ✅ Fast query execution for moderate-sized graphs
- ⚠️ Memory usage scales with graph size
- ⚠️ Large graphs (>1M triples) may require optimization

**v2.0 Considerations**:
- Support persistent graph stores for large projects
- Add graph size limits and warnings
- Optimize for common query patterns (noun-verb structure)

---

## Testing Status

### Current Test Coverage

**graph.rs**: 9 tests
- Insert and query operations
- Caching behavior
- Pattern filtering

**rdf/template_metadata.rs**: Tests present but not detailed
- RDF serialization
- SPARQL querying

**rdf/validation.rs**: Basic validation tests

### v2.0 Testing Requirements

- [ ] Test RDF loading via CLI (not frontmatter)
- [ ] Test SPARQL CONSTRUCT queries in templates
- [ ] Test v2.0 ontology classes
- [ ] Test v2.0 SHACL shapes
- [ ] Test multiple RDF file merging
- [ ] Test filesystem routing for RDF discovery

---

## Summary

**Current State**: ✅ Robust RDF/SPARQL/SHACL engine built on Oxigraph

**v2.0 Requirements**:
- ✅ **KEEP**: Core engine functionality (~1,900 LOC)
- ❌ **REMOVE**: Frontmatter RDF dependencies (~65 LOC)
- ✅ **ADD**: v2.0 enhancements (~100 LOC)
- **Net Change**: +35 LOC (minimal impact)

**Migration Risk**: **LOW** - Most changes are API refactoring, core engine remains stable.

---

**Last Updated**: RDF engine analysis for v2.0 migration

