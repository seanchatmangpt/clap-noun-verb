# Semantic Bridge Integration Guide

**Status:** v26.6.1 Design Phase  
**Version:** 1.0  
**Date:** 2026-06-01

---

## Quick Start

The semantic bridge connects ggen (CLI code generator) with Open Ontologies (RDF/SPARQL specifications). This guide explains how to use the bridge to query, generate, and validate CLIs.

### For CLI Designers (Using Existing Ontologies)

**Goal:** Generate a CLI from an RDF specification

**Steps:**

1. **Find the ontology spec** (e.g., `ontology/examples/graph-commands.nt`)

2. **Query to extract command specs:**
   ```bash
   sparql \
     --data ontology/ggen-command-vocab.nt \
     --data ontology/examples/graph-commands.nt \
     --query ontology/queries/extract-command-specs.rq \
     > /tmp/cli-spec.rdf
   ```

3. **Convert RDF to ggen YAML** (pseudo-code; implementation in v26.7.0):
   ```bash
   rdf-to-yaml /tmp/cli-spec.rdf > cli-spec.yaml
   ```

4. **Generate Rust code:**
   ```bash
   clap-noun-verb-gen --spec cli-spec.yaml --output src/
   ```

5. **Build and test:**
   ```bash
   cargo make build && cargo make test
   ```

### For Ontology Authors (Creating New Specs)

**Goal:** Define a new CLI in RDF

**Steps:**

1. **Create instance data file** (e.g., `ontology/examples/my-cli.nt`)

2. **Define nouns:**
   ```ntriples
   <urn:myapp:database> rdf:type ggen:NounSpec .
   <urn:myapp:database> ggen:nounName "database"^^xsd:string .
   <urn:myapp:database> ggen:nounDocstring "Database management commands"@en .
   ```

3. **Define verbs:**
   ```ntriples
   <urn:myapp:database-migrate> rdf:type ggen:VerbSpec .
   <urn:myapp:database-migrate> ggen:verbName "migrate"^^xsd:string .
   <urn:myapp:database-migrate> ggen:belongsToNoun <urn:myapp:database> .
   ```

4. **Define parameters:**
   ```ntriples
   <urn:myapp:database-migrate-version> rdf:type ggen:ParameterSpec .
   <urn:myapp:database-migrate-version> ggen:paramName "version"^^xsd:string .
   <urn:myapp:database-migrate-version> ggen:paramType "String"^^xsd:string .
   <urn:myapp:database-migrate-version> ggen:isRequired "true"^^xsd:boolean .
   ```

5. **Define output type:**
   ```ntriples
   <urn:myapp:database-migrate-output> rdf:type ggen:OutputTypeSpec .
   <urn:myapp:database-migrate-output> ggen:outputName "MigrationResult"^^xsd:string .
   <urn:myapp:database-migrate-output> ggen:hasField "status: String"^^xsd:string .
   ```

6. **Link them together:**
   ```ntriples
   <urn:myapp:database-migrate> ggen:hasParameter <urn:myapp:database-migrate-version> .
   <urn:myapp:database-migrate> ggen:returnsType <urn:myapp:database-migrate-output> .
   <urn:myapp:database> ggen:hasVerb <urn:myapp:database-migrate> .
   ```

7. **Validate with SPARQL query:**
   ```bash
   sparql \
     --data ontology/ggen-command-vocab.nt \
     --data ontology/examples/my-cli.nt \
     --query ontology/queries/extract-command-specs.rq
   ```

### For ggen Authors (Implementation)

**Goal:** Implement forward/reverse mappings

#### v26.6.1 (Now)

- [x] Vocabulary defined in `ontology/ggen-command-vocab.nt`
- [x] Example instances in `ontology/examples/graph-commands.nt`
- [x] SPARQL query for extraction (`extract-command-specs.rq`)
- [ ] Implement reverse mapper (RDF → code generation)
- [ ] Implement forward mapper (#[verb] macros → RDF triples)

#### v26.7.0 (Q3 2026)

- [ ] Create `Receipt` struct for proof emission
- [ ] Implement `emit_receipt()` in verb handlers
- [ ] Implement `--introspect` flag for metadata export
- [ ] Create `CommandMetadata` struct for runtime introspection
- [ ] Implement serialization of CommandRegistry to JSON

#### v26.8.0+ (Q4 2026+)

- [ ] Full SPARQL query support via triplestore
- [ ] Process Mining Chicago TDD conformance validator
- [ ] Bidirectional sync (RDF ↔ Rust)
- [ ] Autonomous CLI generation from graph law

---

## File Reference

### Core Ontology

| File | Purpose | Size |
|------|---------|------|
| `ontology/ggen-command-vocab.nt` | RDF vocabulary definitions (classes, properties) | 21 KB |
| `ontology/examples/graph-commands.nt` | Example instances: graph noun, load/query/validate verbs | 12 KB |

### Queries

| File | Purpose | Type |
|------|---------|------|
| `ontology/queries/extract-command-specs.rq` | Extract verb specs for code generation | CONSTRUCT |
| `ontology/queries/conformance-check.rq` | Validate receipts against model (Process Mining) | ASK |

### Documentation

| File | Purpose |
|------|---------|
| `docs/semantic-bridge-ggen-ontologies.md` | Full 600+ line specification document |
| `docs/semantic-bridge-integration-guide.md` | This file |
| `docs/ggen-integration-contract-v26.6.1.md` | Stable API contract for ggen |
| `research/clap-noun-verb-v2661/intel/ggen-producer-needs.md` | Requirements analysis |

---

## RDF Data Model

### Class Hierarchy

```
CommandSpec (abstract)
├── NounSpec
│   └── (contains verbs, sub-nouns)
└── VerbSpec
    ├── (has parameters)
    └── (returns OutputTypeSpec)

ParameterSpec (standalone)
OutputTypeSpec (standalone)
Receipt (proof records)
```

### Key Relationships

| From | Property | To | Meaning |
|------|----------|----|---------| 
| NounSpec | hasVerb | VerbSpec | Noun contains these actions |
| NounSpec | hasSubNoun | NounSpec | Noun contains these nested nouns |
| VerbSpec | belongsToNoun | NounSpec | Verb operates on this noun |
| VerbSpec | hasParameter | ParameterSpec | Verb accepts these arguments |
| VerbSpec | returnsType | OutputTypeSpec | Verb returns this type |

---

## Namespace Management

### Ontology Namespace

All vocabulary is defined under:
```
https://chatmangpt.com/ontologies/ggen/command-vocab#
```

### Instance Namespaces

All example instances use local identifiers:
```
urn:ggen:specimen:  (specimen CLI instances)
urn:ggen:receipt:   (execution receipt identifiers)
urn:myapp:          (user-defined app instances)
```

**Never use `example.org` or invent URIs for domains you don't control.**

---

## Validation Checklist

When creating a new RDF CLI specification, verify:

- [ ] All nouns have `nounName` and `nounDocstring`
- [ ] All verbs have `verbName`, `verbDocstring`, `belongsToNoun`
- [ ] All parameters have `paramName`, `paramType`, `isRequired`
- [ ] All output types have `outputName` and at least one `hasField`
- [ ] All verbs link to their parameters via `hasParameter`
- [ ] All verbs link to their output type via `returnsType`
- [ ] All nouns link to their verbs via `hasVerb`
- [ ] SPARQL query returns non-empty result set
- [ ] No unresolved IRIs (all predicates/classes fully qualified)

---

## Process Mining Chicago TDD

### Receipt Emission (v26.7.0+)

Every CLI execution should emit a Receipt:

```rust
Receipt {
    id: "uuid-or-hash",
    timestamp: 1748866800000,  // Unix ms
    command_path: vec!["graph", "load"],
    exit_code: 0,
    input_args: json!({"path": "file.ttl"}),
    output: json!({"triples_loaded": 42, ...}),
    duration_ms: 125,
    stderr: None,
}
```

### Conformance Validation

Receipts must conform to declared command structure:

```bash
# Run conformance query
sparql \
  --data ontology/ggen-command-vocab.nt \
  --data ontology/examples/graph-commands.nt \
  --data event-logs.nt \
  --query ontology/queries/conformance-check.rq
```

**Result:**
- `true` = all receipts conform to model ✓
- `false` = at least one receipt violates model ✗

---

## Common Tasks

### Add a New Verb to Existing Noun

1. Create VerbSpec instance
2. Set `verbName`, `verbDocstring`, `belongsToNoun`
3. Create parameters and output type
4. Link via `hasParameter`, `returnsType`
5. Add noun's `hasVerb` reference
6. Test with SPARQL query

### Change Parameter Type

1. Update `paramType` property
2. Update example instances (if applicable)
3. Re-run SPARQL query to verify
4. Regenerate code (in v26.7.0+)

### Add Validation Rule

Set `validation` property on ParameterSpec:
```ntriples
<urn:myapp:param-1> ggen:validation "path_exists"^^xsd:string .
<urn:myapp:param-2> ggen:validation "regex: [a-z]+"^^xsd:string .
<urn:myapp:param-3> ggen:validation "enum: prod,staging,dev"^^xsd:string .
```

### Generate Documentation

```bash
# Extract all verbs and parameters
sparql \
  --data ontology/ggen-command-vocab.nt \
  --data ontology/examples/my-cli.nt \
  --query ontology/queries/extract-command-specs.rq \
  --output-format json > docs/api.json

# Generate HTML/Markdown from JSON (implementation in v26.7.0+)
ggen-doc-gen docs/api.json --format markdown > API.md
```

---

## Troubleshooting

### SPARQL Query Returns Empty Results

1. Check ontology file is loaded: `sparql --data ontology/ggen-command-vocab.nt --query 'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 1'`
2. Check instance file syntax: `sparql --data ontology/examples/my-cli.nt --query 'SELECT ?s WHERE { ?s rdf:type ?t } LIMIT 5'`
3. Verify namespace URIs match in both files
4. Check for typos in property names (case-sensitive)

### Invalid N-Triples Syntax

N-Triples format is strict:
- Every line must be `<subject> <predicate> <object> .`
- All IRIs must be in angle brackets: `<http://example.org/something>`
- All literals must be quoted: `"value"`
- Language tags: `"value"@en`
- Datatypes: `"value"^^<http://www.w3.org/2001/XMLSchema#string>`
- Comment lines start with `#`

Validate with any RDF parser:
```bash
python3 -c "from rdflib import Graph; Graph().parse('file.nt', format='nt'); print('Valid')"
```

### Conformance Check Always Fails

1. Verify receipts have correct `commandPath` format
2. Check verbs are declared in ontology
3. Verify `inputArguments` JSON contains required parameters
4. Look for typos in verb/parameter names

---

## Future Extensions

### SHACL Validation Shapes

(Planned v26.8.0+)

Define validation constraints in SHACL:
```ntriples
<urn:ggen:shapes:ParamShape> a sh:NodeShape ;
  sh:targetClass ggen:ParameterSpec ;
  sh:property [
    sh:path ggen:paramName ;
    sh:minCount 1 ;
    sh:datatype xsd:string
  ] .
```

### Semantic Composition

(Planned v26.9.0+)

Compose commands based on semantic compatibility:
```sparql
# Find all verbs that output type matches another verb's input
CONSTRUCT {
  ?verb1 ggen:canComposeTo ?verb2
}
WHERE {
  ?verb1 ggen:returnsType ?output1 .
  ?verb2 ggen:hasParameter ?param .
  ?param ggen:paramType ?output1Type .
  ?output1 ggen:outputName ?output1Type .
}
```

### Event Log Mining

(Planned v27.0.0+)

Discover process models from execution receipts using pm4py:
```python
from pm4py.discovery import discover_process_tree
log = import_receipts_as_ocel('event-logs.nt')
model = discover_process_tree(log)
print(model.to_eps())  # Export to Petri net
```

---

## References

- **RDF Specification:** https://www.w3.org/RDF/
- **SPARQL 1.1:** https://www.w3.org/TR/sparql11-query/
- **OWL 2.0:** https://www.w3.org/TR/owl2-overview/
- **N-Triples Format:** https://www.w3.org/TR/n-triples/
- **Process Mining Chicago TDD:** `~/.claude/rules/process-mining-chicago-tdd.md`

---

**End of Integration Guide**
