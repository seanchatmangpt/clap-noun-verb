# ggen Architecture Overview - Template Engine & RDF Stack

This document provides C4 architecture diagrams for understanding ggen's template rendering engine, RDF, OWL, SHACL, and SPARQL integration.

**Based on**: [ggen GitHub Repository](https://github.com/seanchatmangpt/ggen)

## Overview

ggen is a **Rust Template Generator with Frontmatter & RDF Support** that combines:
- **Template Rendering**: Tera-based templating with SPARQL integration
- **RDF/OWL/SHACL**: Full semantic web stack support
- **SPARQL**: Query language for knowledge graphs
- **AI Integration**: LLM-powered generation with ontology context

---

## C4 Diagrams

All diagrams are available in `ARCHITECTURE_DIAGRAMS.puml`:

### 1. System Context Diagram (`c4_context_ggen`)

**Level**: Context (C4 Level 1)

Shows ggen's place in the overall system, including:
- **Users**: Developers and AI assistants
- **External Systems**: AI providers, marketplace, file system
- **Key Interactions**: CLI commands, JSON output, LLM API calls

**Key Relationships**:
- Users → ggen: CLI commands (ai project, template generate, etc.)
- ggen → AI Providers: LLM API calls for generation
- ggen → Marketplace: Package operations
- ggen → File System: Template files, generated code, RDF graphs

---

### 2. Container Diagram - Template & RDF (`c4_containers_template_rdf`)

**Level**: Container (C4 Level 2)

Shows the main containers/subsystems:

**Containers:**
- **CLI** (`clap-noun-verb`): Command-line interface with auto-discovery
- **Template Engine** (`ggen-core`): Template rendering, frontmatter parsing, variable substitution
- **RDF Processor** (`ggen-core`): RDF/OWL/SHACL support, graph parsing, SPARQL execution
- **AI Integration** (`ggen-ai`): LLM providers (OpenAI, Anthropic, Ollama)
- **Marketplace** (`ggen-marketplace`): Package management
- **Knowledge Hooks** (`ggen-core`): Autonomic regeneration triggers

**Key Flows**:
1. CLI delegates to template engine for rendering
2. Template engine queries RDF processor via SPARQL
3. AI integration uses RDF context for ontology embedding
4. Knowledge hooks monitor RDF changes and trigger regeneration

---

### 3. Component Diagram - Template Engine (`c4_components_template_engine`)

**Level**: Component (C4 Level 3)

Detailed view of the template rendering engine:

**Components:**
- **Frontmatter Parser**: Parses YAML frontmatter (metadata, variables, config)
- **Tera Engine**: Rust templating engine (variable substitution, control structures)
- **SPARQL Integration**: Executes SPARQL queries within templates (`{{ sparql(query="...") }}`)
- **File Tree Generator**: Generates directory structure and nested templates
- **Variable Resolver**: Resolves variables from user input, RDF data, computed values
- **Template Loader**: Loads templates from files, marketplace, with caching
- **Output Writer**: Writes generated code with proper directory structure

**Key Feature**: **SPARQL Integration in Templates**
```
// Template example with SPARQL
{{ sparql(query="SELECT ?type WHERE {...}") }}
pub struct {{name | capitalize}} {
    {% for field in sparql(query="get_fields") %}
    {{ field.name }}: {{ field.type }},
    {% endfor %}
}
```

---

### 4. Component Diagram - RDF Processor (`c4_components_rdf_processor`)

**Level**: Component (C4 Level 3)

Complete RDF processing stack:

**Components:**
- **RDF Parser**: Parses Turtle (.ttl), N-Triples, RDF/XML, JSON-LD
- **Graph Store**: In-memory RDF triple store (Subject-Predicate-Object)
- **OWL Processor**: OWL ontology support (classes, properties, inheritance, reasoning)
- **SHACL Validator**: SHACL shape validation (constraints, validation reports)
- **SPARQL Engine**: SPARQL query execution (SELECT, CONSTRUCT, ASK, UPDATE)
- **Ontology Manager**: Ontology management (load/save, merge, namespaces)
- **RDF Serializer**: Serializes to Turtle, N-Triples, RDF/XML

**Technology Stack:**
- **RDF**: Resource Description Framework (triple-based data model)
- **OWL**: Web Ontology Language (semantic modeling, reasoning)
- **SHACL**: Shapes Constraint Language (data validation)
- **SPARQL**: SPARQL Protocol and RDF Query Language (querying knowledge graphs)

---

### 5. Template + SPARQL Integration Detail (`c4_template_sparql_interaction`)

**Level**: Component Detail (C4 Level 4)

Shows how templates integrate with SPARQL:

**Flow:**
1. Tera engine processes template
2. Encounters `{{ sparql(query="...") }}` filter
3. SPARQL filter executes query via SPARQL engine
4. SPARQL engine queries graph store
5. Graph store uses OWL reasoner for inference
6. Results injected back into template

**Example:**
```tera
//! Type from RDF: {{ sparql(query="get_type") }}

pub struct {{name | capitalize}} {
    name: String,
}

impl {{name | capitalize}} {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
```

---

### 6. SHACL Validation Flow (`c4_shacl_validation_flow`)

**Level**: Component Detail (C4 Level 4)

SHACL (Shapes Constraint Language) validation process:

**Components:**
- **SHACL Loader**: Loads SHACL shape definitions
- **SHACL Validator**: Validates RDF data against shapes
- **Constraint Checker**: Evaluates constraints (minCount, maxCount, datatype, pattern)
- **Validation Report**: Generates validation reports (violations, warnings)

**SHACL Example:**
```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/> .

ex:UserShape
    a sh:NodeShape ;
    sh:targetClass ex:User ;
    sh:property [
        sh:path ex:name ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
    ] .
```

**Validation Process:**
1. Load SHACL shapes
2. Match data nodes to target classes
3. Check property constraints
4. Generate validation report

---

### 7. OWL Ontology Processing (`c4_owl_ontology_processing`)

**Level**: Component Detail (C4 Level 4)

OWL (Web Ontology Language) processing:

**Components:**
- **OWL Parser**: Parses OWL ontologies (classes, properties, individuals, axioms)
- **Ontology Reasoner**: Performs reasoning (class hierarchy, property chains, restrictions)
- **Class Hierarchy**: Manages class inheritance (SubClassOf, EquivalentClass, DisjointWith)
- **Property Manager**: Manages properties (ObjectProperty, DataProperty, InverseOf, Transitive)
- **Individual Tracker**: Tracks instances (Type assertions, property assertions)

**OWL Example:**
```turtle
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix ex: <http://example.org/> .

ex:Developer
    a owl:Class ;
    rdfs:subClassOf ex:Person .

ex:worksOn
    a owl:ObjectProperty ;
    rdfs:domain ex:Developer ;
    rdfs:range ex:Project .
```

**Reasoning Process:**
1. Parse OWL axioms into graph store
2. Perform reasoning (inference)
3. Build class hierarchy
4. Infer property relations
5. Type instances
6. Add inferred triples to graph store

---

### 8. Complete Generation Flow (`c4_full_generation_flow`)

**Level**: Container (C4 Level 2)

Shows the complete generation pipeline:

**Flow:**
1. User commands via CLI
2. CLI delegates to AI generator
3. AI generator calls LLM API
4. LLM returns generated template
5. Template engine processes template
6. Template engine queries RDF processor via SPARQL
7. RDF processor returns ontology data
8. Template engine renders with RDF data
9. File generator writes output

**Key Integration Points:**
- **AI Generation**: Natural language → Template with RDF hooks
- **Template Rendering**: Template + RDF data → Generated code
- **RDF Support**: OWL ontologies, SHACL validation, SPARQL queries

---

### 9. Data Flow - Template with RDF Knowledge Graph (`c4_data_flow_template_rdf`)

**Level**: Data Flow (C4 Level 4)

Detailed data flow showing template rendering with RDF:

**Steps:**
1. Load Tera template
2. Execute SPARQL queries in template
3. SPARQL executor queries graph store
4. OWL reasoner performs inference
5. SHACL validator validates data
6. Query results returned to template
7. Template renders with RDF data
8. Output generator writes final code

**Persistent Storage**: RDF graphs stored as Turtle/N-Triples files

---

## Key Architectural Patterns

### 1. Separation of Concerns

**CLI Layer** (Validation Only):
- Validates arguments/options
- Delegates to business logic
- **NO business logic**

**Business Logic Layer** (Reusable):
- Pure functions
- Can be used by CLI, API, Web, etc.
- Template rendering
- RDF processing

### 2. Template + RDF Integration

**SPARQL in Templates**:
- Templates can execute SPARQL queries
- Results injected into template variables
- Enables knowledge graph-driven code generation

**Example:**
```tera
{% for class in sparql(query="SELECT ?class WHERE { ?class a owl:Class }") %}
pub struct {{ class.name }} {
    // ...
}
{% endfor %}
```

### 3. Ontology-Driven Generation

**OWL Ontologies**:
- Define domain models
- Enable reasoning
- Type checking

**SHACL Validation**:
- Ensures generated code matches constraints
- Validates data shapes
- Provides error reports

**SPARQL Queries**:
- Extract knowledge from graphs
- Enable flexible querying
- Support complex patterns

### 4. AI + Knowledge Graph Integration

**Flow:**
1. AI generates template from natural language
2. Template includes RDF hooks (`{{ sparql(...) }}`)
3. Template queries knowledge graph
4. Generated code is ontology-aware

---

## Technology Stack

### Template Engine
- **Tera**: Rust templating engine
- **Frontmatter**: YAML metadata
- **SPARQL Integration**: Queries in templates

### RDF Stack
- **RDF**: Triple-based data model
- **OWL**: Ontology language (OWL 2)
- **SHACL**: Shape constraints
- **SPARQL**: Query language (SPARQL 1.1)

### AI Integration
- **OpenAI**: GPT-4o
- **Anthropic**: Claude 3.5
- **Ollama**: Local models
- **Streaming**: Real-time generation

---

## Data Formats

### Template Files
- **Format**: Tera templates with YAML frontmatter
- **Extension**: `.tmpl`, `.tera`
- **Example**:
  ```yaml
  ---
  name: "Module Template"
  description: "Generate a Rust module"
  ---
  pub struct {{name}} {
      // ...
  }
  ```

### RDF Files
- **Turtle** (`.ttl`): Human-readable RDF
- **N-Triples** (`.nt`): Line-based format
- **RDF/XML** (`.rdf`): XML format
- **JSON-LD** (`.jsonld`): JSON format

### OWL Ontologies
- **Format**: Turtle or RDF/XML
- **Extension**: `.owl`, `.ttl`, `.rdf`
- **Standards**: OWL 2 DL, OWL 2 EL, OWL 2 QL

### SHACL Shapes
- **Format**: Turtle
- **Extension**: `.shacl`, `.ttl`
- **Standards**: SHACL Core, SHACL-SPARQL

---

## Generation Workflow

### Typical Flow:

1. **User Command**:
   ```bash
   ggen ai generate -d "Database repository pattern"
   ```

2. **AI Generation**:
   - LLM generates template with RDF hooks
   - Includes SPARQL queries for knowledge graph

3. **Template Processing**:
   - Frontmatter parsed (metadata, variables)
   - SPARQL queries executed
   - RDF data injected into template
   - Variables resolved

4. **RDF Processing**:
   - OWL reasoning (if needed)
   - SHACL validation (if configured)
   - Graph store queries

5. **Code Generation**:
   - Template rendered with data
   - File tree generated
   - Output written

6. **Validation**:
   - SHACL validation of generated data
   - Error reporting

---

## Query Patterns

### Common SPARQL Patterns in Templates

**1. Get Class Properties:**
```sparql
SELECT ?property ?type WHERE {
    ?class a owl:Class ;
        ?property ?type .
}
```

**2. Get Subclasses:**
```sparql
SELECT ?subclass WHERE {
    ?subclass rdfs:subClassOf ex:BaseClass .
}
```

**3. Get Instances:**
```sparql
SELECT ?instance WHERE {
    ?instance a ex:MyClass .
}
```

**4. Get Related Entities:**
```sparql
SELECT ?related WHERE {
    ?entity ex:relation ?related .
}
```

---

## Validation Patterns

### SHACL Validation Examples

**1. String Property (Required):**
```turtle
ex:NameProperty
    sh:path ex:name ;
    sh:datatype xsd:string ;
    sh:minCount 1 ;
    sh:maxCount 1 .
```

**2. Integer Range:**
```turtle
ex:AgeProperty
    sh:path ex:age ;
    sh:datatype xsd:integer ;
    sh:minInclusive 0 ;
    sh:maxInclusive 120 .
```

**3. Pattern Matching:**
```turtle
ex:EmailProperty
    sh:path ex:email ;
    sh:datatype xsd:string ;
    sh:pattern "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$" .
```

---

## Summary

ggen's architecture integrates:
- **Template Engine** (Tera) with SPARQL support
- **RDF Stack** (RDF, OWL, SHACL, SPARQL)
- **AI Integration** (LLM providers)
- **Knowledge Graph** (persistent RDF storage)

**Key Innovation**: Templates can query knowledge graphs via SPARQL, enabling ontology-driven code generation.

**Benefits**:
- **Semantic Generation**: Code generated from ontologies
- **Type Safety**: OWL classes provide type information
- **Validation**: SHACL ensures generated code matches constraints
- **Flexibility**: SPARQL enables complex queries
- **AI Context**: LLMs use ontology context for better generation

---

**References:**
- [ggen GitHub Repository](https://github.com/seanchatmangpt/ggen)
- [RDF Specification](https://www.w3.org/RDF/)
- [OWL 2 Specification](https://www.w3.org/TR/owl2-overview/)
- [SHACL Specification](https://www.w3.org/TR/shacl/)
- [SPARQL 1.1 Specification](https://www.w3.org/TR/sparql11-overview/)

