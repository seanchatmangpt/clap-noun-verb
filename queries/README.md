# SPARQL Queries for CLI Introspection & Code Generation

This directory contains SPARQL queries for introspecting and generating code from the clap-noun-verb ontology.

## Overview

These queries enable:
- **Discovery**: Find all verbs, nouns, arguments, and traits in the CLI definition
- **Validation**: Check that the noun-verb structure conforms to the pattern
- **Code Generation**: Extract structured data for generating Rust trait implementations
- **Specification**: Construct normalized specifications for tooling and documentation

## Query Files

### 1. find-all-verbs.rq

**Purpose**: Discover all verb definitions in the ontology.

**Type**: SELECT query

**Returns**: 
- `verb` - URI of the verb resource
- `verbName` - CLI identifier (e.g., "status")
- `verbAbout` - Description
- `nounName` - Parent noun (e.g., "services")
- `returnType` - Return type annotation (e.g., "ServiceStatus (JSON)")
- `argumentCount` - Number of arguments accepted

**Sample Results** (from cli-pattern.ttl):
```
verb                    verbName   verbAbout                          nounName   returnType                argumentCount
================================================================================================================================================
ex:BackupVerb           backup     Create database backup              database   BackupResult (JSON)       1
ex:StatusVerb           status     Show current status of all svc...   services   ServiceStatus (JSON)      1
ex:StartVerb            start      Start one or more services          services   Result<()>                1
ex:CreateUserVerb       create     Create a new user account           users      UserInfo (JSON)           1
ex:ListUserVerb         list       List all user accounts              users      Vec<UserInfo> (JSON)      1
```

**Use Cases**:
- Generate Rust enum variants for all verbs
- Build CLI help text
- Create verb registry
- Generate tests for each verb

---

### 2. extract-arguments.rq

**Purpose**: Extract detailed argument specifications for a specific verb.

**Type**: SELECT query

**Parameters**: 
- Hardcoded example uses `verbName "status"@en` (can be parameterized)

**Returns**:
- `argument` - URI of the argument
- `argumentName` - CLI identifier (e.g., "service", "verbose")
- `shortName` - Single-char short form (e.g., "v")
- `argumentAbout` - Help text
- `argumentTypeLabel` - Kind: Required, Optional, Flag, Repeating
- `valueType` - Rust type (String, u16, bool, etc.)
- `required` - Boolean: is this argument mandatory?
- `defaultValue` - Default value if optional

**Sample Query** (for "status" verb):
```sparql
SELECT ?argumentName ?shortName ?valueType ?required 
WHERE {
  ?verb cnv:hasVerbName "status"@en .
  ?verb cnv:hasArguments ?argument .
  ?argument cnv:hasArgumentName ?argumentName .
  ...
}
```

**Use Cases**:
- Generate function signatures
- Build clap::Command builders
- Generate help text
- Validate required vs optional arguments
- Type-check argument parsing

---

### 3. validate-cli-structure.rq

**Purpose**: Validate that the CLI definition conforms to the noun-verb pattern.

**Type**: SELECT query (returns issues when violations exist)

**Checks**:
1. **Orphaned verbs**: Verbs without a parent noun
2. **Unnamed verbs**: Verbs missing `cnv:hasVerbName`
3. **Unnamed nouns**: Nouns missing `cnv:hasNounName`
4. **Ambiguous verbs**: Verbs with multiple parent nouns (exactly 1 expected)
5. **Typed arguments**: Required arguments must specify `cnv:valueType`
6. **Send trait**: Verbs must implement Send
7. **Sync trait**: Verbs must implement Sync

**Returns**:
- `issue` - Human-readable issue title
- `affectedResource` - URI of the problematic resource
- `description` - Detailed explanation

**Sample Results** (from verb-traits.ttl):
```
issue                      affectedResource  description
========================================================================
Orphaned verb: AddVerb      AddVerb           Verb does not belong to any noun
Missing Send trait: AddVerb AddVerb           Verb should implement Send trait
Missing Sync trait: AddVerb AddVerb           Verb should implement Sync trait
```

**Use Cases**:
- CI/CD validation gate (fail if any issues exist)
- Pre-compilation checks
- Ontology quality assurance
- Generate detailed error reports

---

### 4. generate-trait-impls.rq

**Purpose**: Extract trait implementation requirements for code generation.

**Type**: SELECT query

**Returns**:
- `resource` - URI of the noun or verb
- `resourceType` - "Noun" or "Verb"
- `resourceName` - CLI identifier
- `traitRequirement` - URI of the required trait
- `traitLabel` - Human-readable trait name (e.g., "Send", "Serialize")
- `returnType` - Return type (verbs only)
- `handlerSignature` - Handler function signature (verbs only)
- `argumentCount` - Number of arguments (aggregated)

**Sample Results**:
```
resourceType resourceName returnType          traitRequirement  traitLabel
================================================================================
Verb         start       Result<()>           Send              Send marker trait
Verb         start       Result<()>           Sync              Sync marker trait
Verb         status      ServiceStatus (JSON) Serialize         Serialize trait
Noun         services    (null)               (null)            (null)
Noun         users       (null)               (null)            (null)
```

**Use Cases**:
- Generate `impl NounCommand for MyNoun` blocks
- Generate `impl VerbCommand for MyVerb` blocks
- Verify trait bounds at ontology level
- Create trait wrapper code

---

### 5. generate-cli-spec.rq

**Purpose**: Construct a normalized CLI specification in RDF format, ready for code generation.

**Type**: CONSTRUCT query (returns RDF triples with `spec:` predicates)

**Output Predicates**:
- `spec:NounSpec` - Class for normalized nouns
- `spec:VerbSpec` - Class for normalized verbs
- `spec:nounName`, `spec:verbName` - String identifiers
- `spec:hasVerbSpec` - Relationship from noun to verbs
- `spec:returnType` - Return type for verbs

**Sample Output** (as N-Triples):
```
<http://clap-noun-verb.io/spec#services> a <http://clap-noun-verb.io/spec#NounSpec> .
<http://clap-noun-verb.io/spec#services> <http://clap-noun-verb.io/spec#nounName> "services" .
<http://clap-noun-verb.io/spec#services> <http://clap-noun-verb.io/spec#hasVerbSpec> <http://clap-noun-verb.io/spec#services:status> .
<http://clap-noun-verb.io/spec#services:status> a <http://clap-noun-verb.io/spec#VerbSpec> .
<http://clap-noun-verb.io/spec#services:status> <http://clap-noun-verb.io/spec#verbName> "status" .
```

**Use Cases**:
- Serialize CLI definition to JSON (via RDF→JSON-LD)
- Feed into code generation templates
- Version and diff CLI specifications
- Export to OpenAPI/AsyncAPI schemas
- Document command hierarchy

---

## Running the Queries

### Prerequisites

```bash
pip install rdflib
```

### Execute All Queries

```bash
python3 run-sparql-queries.py
```

This script:
1. Loads all `.ttl` files from `ontology/`
2. Executes each `.rq` query
3. Displays formatted results
4. Reports total result counts

### Execute a Single Query (Python)

```python
from rdflib import Graph
from rdflib.plugins.sparql import prepareQuery

graph = Graph()
graph.parse("ontology/cli-pattern.ttl", format="turtle")

with open("queries/find-all-verbs.rq") as f:
    query = prepareQuery(f.read())

results = graph.query(query)
for row in results:
    print(row)
```

### Execute a Query with Parameters

To pass parameters (e.g., filter by specific verb):

```sparql
# Parameterized query
SELECT ?argumentName ?valueType 
WHERE {
  ?verb cnv:hasVerbName ?targetVerbName .
  ?verb cnv:hasArguments ?argument .
  ?argument cnv:hasArgumentName ?argumentName .
  ?argument cnv:valueType ?valueType .
}
```

Pass as:
```python
results = graph.query(query, initBindings={
    Variable("targetVerbName"): Literal("status")
})
```

---

## Query Patterns & Best Practices

### Pattern 1: Optional Properties

Use `OPTIONAL` blocks to handle missing data:

```sparql
OPTIONAL { ?verb cnv:verbAbout ?description . }
OPTIONAL { ?verb cnv:returnType ?returnType . }
```

### Pattern 2: Aggregation

Count related resources:

```sparql
SELECT ?verb (COUNT(?arg) AS ?argumentCount)
WHERE {
  ?verb cnv:hasArguments ?arg .
}
GROUP BY ?verb
```

### Pattern 3: Validation (FILTER NOT EXISTS)

Check for missing required properties:

```sparql
?verb a cnv:Verb .
FILTER NOT EXISTS { ?verb cnv:belongsToNoun ?noun . }
BIND("Orphaned verb" AS ?issue)
```

### Pattern 4: Case/When Logic

Map ontology values to normalized values:

```sparql
BIND(CASE
  WHEN ?argType = cnv:ArgumentType_Flag THEN spec:BoolFlag
  WHEN ?argType = cnv:ArgumentType_Required THEN spec:PositionalRequired
  ELSE spec:Unknown
END AS ?argumentKind)
```

### Pattern 5: URI Construction

Generate new URIs from components:

```sparql
BIND(IRI(CONCAT(STR(spec:), ?nounName, ":", ?verbName)) AS ?verbRef)
```

---

## Ontology Namespaces

| Prefix | Namespace | Purpose |
|--------|-----------|---------|
| `cnv:` | `http://clap-noun-verb.io/ontology#` | Core noun-verb ontology |
| `ex:` | `http://clap-noun-verb.io/examples#` | Example instances (services, users, database) |
| `spec:` | `http://clap-noun-verb.io/spec#` | Normalized specification format |
| `rdf:` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | RDF primitives |
| `rdfs:` | `http://www.w3.org/2000/01/rdf-schema#` | RDF Schema |
| `owl:` | `http://www.w3.org/2002/07/owl#` | OWL ontology language |
| `xsd:` | `http://www.w3.org/2001/XMLSchema#` | XML Schema datatypes |

---

## Integration with Code Generation

### Workflow: Ontology → Rust Code

1. **Introspection**: Run `find-all-verbs.rq` to discover all verbs
2. **Validation**: Run `validate-cli-structure.rq` to ensure correctness
3. **Extraction**: Run `generate-trait-impls.rq` to get implementation data
4. **Specification**: Run `generate-cli-spec.rq` to produce normalized RDF
5. **Code Gen**: Transform spec RDF to Rust code via templates

### Example: Generate Verb Enum

```rust
// Generated from find-all-verbs.rq results
pub enum Verb {
    Status,     // "status" -> ServiceStatus (JSON)
    Start,      // "start" -> Result<()>
    Stop,       // "stop" -> Result<()>
    Restart,    // "restart" -> Result<()>
    Logs,       // "logs" -> LogOutput (streamed)
}
```

### Example: Generate Argument Structs

```rust
// Generated from extract-arguments.rq
pub struct StatusArgs {
    #[arg(short, long)]
    service: Option<String>,    // Optional
    #[arg(short, long)]
    verbose: bool,              // Flag
}
```

---

## Troubleshooting

### No Results from Query

1. **Check namespace prefixes**: Verify that all IRIs use correct namespaces
2. **Inspect loaded data**: `SELECT ?s ?p ?o LIMIT 10` to see what's in the graph
3. **Test simpler queries**: Start with `?s a cnv:Verb` to verify data is present
4. **Check optional handling**: Add debug OPTIONAL blocks to trace missing properties

### Syntax Errors

- CONSTRUCT queries cannot use `BIND()` in the WHERE clause (only in SELECT)
- Use full IRIs in the CONSTRUCT template, not prefix abbreviations
- Ensure all PREFIX declarations come before the query

### Performance

- Filter early with `VALUES` for large datasets
- Use `LIMIT` to cap result size during development
- Consider pre-filtering in Python before querying

---

## Files Reference

| File | Type | Purpose |
|------|------|---------|
| `find-all-verbs.rq` | SELECT | Discover all verbs in ontology |
| `extract-arguments.rq` | SELECT | Get argument specs for a verb |
| `validate-cli-structure.rq` | SELECT | Check pattern conformance |
| `generate-trait-impls.rq` | SELECT | Extract trait requirements |
| `generate-cli-spec.rq` | CONSTRUCT | Build normalized RDF spec |
| `run-sparql-queries.py` | Python | Execute all queries and display results |
| `README.md` | Documentation | This file |

---

## Related Documentation

- Ontology: `../ontology/clap-noun-verb-ontology.ttl`
- Examples: `../ontology/cli-pattern.ttl`
- Traits: `../ontology/verb-traits.ttl`
- Project CLAUDE.md: `../CLAUDE.md`
