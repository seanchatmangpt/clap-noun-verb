# Explanation: Semantic Web Fundamentals

**Purpose**: Understand the semantic web and why it matters for CLI generation

## What is the Semantic Web?

The "semantic" web is the web of meaning, not just syntax.

### Web 1.0: Documents

Traditional web:
```html
<h1>Services</h1>
<p>Check status of a service</p>
<button onclick="status()">Click here</button>
```

Meaning is implicit. Browser shows HTML. Only humans understand what it means.

### Web 2.0: Data

Structured data:
```json
{
  "nouns": [
    {
      "name": "services",
      "verbs": ["status", "start", "stop"]
    }
  ]
}
```

Structure exists. Computers can parse it. But meaning requires documentation.

### Semantic Web (Web 3.0): Meaning

Machine-readable semantics:
```turtle
cnv:Services a cnv:Noun ;
    cnv:name "services" .

cnv:Status a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Check service status" .
```

**Meaning is explicit**. Machines understand relationships, not just data types.

## RDF: The Foundation

RDF (Resource Description Framework) enables semantic meaning through triples:

```
Subject  Predicate  Object
Services is-a       Noun
Status   is-a       Verb
Status   belongs-to Services
```

**Why triples work**:

1. **Universal**: Any fact can be expressed
2. **Composable**: Multiple triples form graphs
3. **Queryable**: SPARQL queries find patterns
4. **Extensible**: Add new properties without changing schema
5. **Interoperable**: W3C standard

## Ontologies: Models of Reality

An ontology is a machine-readable model of a domain.

### Example: CLI Ontology

```turtle
# Define classes (types of things)
cnv:Noun a rdfs:Class .       # Command categories
cnv:Verb a rdfs:Class .       # Commands

# Define relationships
cnv:hasNoun a rdf:Property ;
    rdfs:domain cnv:Verb ;
    rdfs:range cnv:Noun .

# Define instances (actual things)
cnv:Services a cnv:Noun .
cnv:Status a cnv:Verb ; cnv:hasNoun cnv:Services .
```

### Why Ontologies Matter

**For humans**: Clear specification of domain knowledge
```
"Services is a command category" (explicit)
vs.
"services" (ambiguous - could be anything)
```

**For machines**: Enable reasoning
```sparql
# Query: What are all the commands?
SELECT ?verb WHERE { ?verb a cnv:Verb }

# Query: What commands belong to Services?
SELECT ?verb WHERE {
    ?verb a cnv:Verb ; cnv:hasNoun cnv:Services .
}

# Query: Commands that don't have descriptions
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER NOT EXISTS { ?verb cnv:description ?d }
}
```

## Knowledge Graphs

RDF ontologies form knowledge graphs - connected representations of facts:

```
       Services ──(name)──> "services"
          │
          ├─(type)──> Noun
          │
          ├─(has-verb)──> Status ──(name)──> "status"
          │                   │
          │                   ├─(type)──> Verb
          │                   │
          │                   └─(description)──> "Check service status"
          │
          └─(has-verb)──> Start ──(name)──> "start"
```

**Queries traverse the graph**:
- "Find all verbs" → Follow `has-verb` edges
- "Find command names" → Follow `name` properties
- "Find descriptions" → Follow `description` properties

## Semantic Web Stack

The semantic web has layers:

```
┌─────────────────────────────┐
│ OWL (Web Ontology Language) │ ← Express complex logic
├─────────────────────────────┤
│ SPARQL (Query Language)     │ ← Query the knowledge graph
├─────────────────────────────┤
│ RDF (Data Format)           │ ← Represent facts
├─────────────────────────────┤
│ XML (Serialization)         │ ← Text representation
└─────────────────────────────┘

clap-noun-verb uses:
- RDF (triples)
- Turtle (RDF syntax)
- SPARQL (querying)
```

## SPARQL: Asking Questions

SPARQL is like SQL for knowledge graphs:

```sparql
-- SQL: "Get all users"
SELECT name FROM users

-- SPARQL: "Get all verbs"
SELECT ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
}
```

### SPARQL Powers

**Question 1**: What verbs exist?
```sparql
SELECT ?verb WHERE { ?verb a cnv:Verb }
```

**Question 2**: Which verbs belong to Services?
```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb ; cnv:hasNoun cnv:Services
}
```

**Question 3**: Validate - are all verbs valid?
```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER NOT EXISTS { ?verb cnv:hasNoun ?n }
}
# Should return empty for valid ontology
```

**Question 4**: Count commands by noun
```sparql
SELECT ?noun (COUNT(?verb) as ?count) WHERE {
    ?noun a cnv:Noun .
    ?verb cnv:hasNoun ?noun .
} GROUP BY ?noun
```

## Interoperability Through Semantics

### Linked Data Principles

1. **Use URIs as identifiers** (not string names)
   ```
   ❌ "Status"
   ✅ https://cnv.dev/ontology#Status
   ```

2. **Use HTTP URIs** (dereferenceable)
   ```
   Fetch https://cnv.dev/ontology#Status
   → Get RDF describing Status
   ```

3. **Link to other resources**
   ```
   cnv:Status cnv:hasNoun cnv:Services .
   cnv:Services owl:sameAs service:ServiceCommands .
   ```

4. **Use standard vocabularies**
   ```
   rdf:type       (what is this thing?)
   rdfs:label     (human name)
   rdfs:comment   (description)
   ```

### Benefits of Linked Data

**Systems understand each other**:
- Agent A: "I need to generate commands"
- Agent B: "Here's an RDF ontology"
- Agent A: "Perfect, I understand your ontology because it uses standard semantics"

No custom parsers needed!

## Reasoning and Inference

Semantic systems can make deductions:

```turtle
# Rule 1: If X is a Verb and has a Noun, then X belongs to that domain
cnv:Status a cnv:Verb ; cnv:hasNoun cnv:Services .

# Question: What verbs belong to Services?
# Answer: cnv:Status (inferred automatically)
```

Advanced example with SHACL (Shape Constraint Language):

```turtle
# Rule: Every Verb must have exactly one name
cnv:VerbShape a sh:NodeShape ;
    sh:targetClass cnv:Verb ;
    sh:property [
        sh:path cnv:name ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
    ] .
```

## Why This Matters for Agents

### Problem: Agents need to understand domains

**Without semantics**:
```json
{
  "commands": [
    {
      "id": "svc_status",
      "name": "status",
      "parent": "svc_services",
      "help": "Check service status"
    }
  ]
}
```

Agent sees data but doesn't know:
- Is "status" related to "services"?
- What does "parent" mean?
- What are valid operations?

**With semantics**:
```turtle
cnv:Status a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Check service status" .
```

Agent knows:
- `Status` is definitionally a `Verb`
- `Status` belongs to `Services` (relationship is explicit)
- Purpose is clear from description
- Can query for patterns (all Verbs, all Nouns, etc.)

### Problem: Systems need to work together

**Without semantics**:
```
System A's CLI: "service list"
System B's CLI: "svc ls"
System C's CLI: "list-services"

How do they talk? Custom integration code!
```

**With semantics**:
```turtle
# All systems use RDF
sysA:ServiceList a cnv:Verb ; cnv:name "list" ; cnv:hasNoun cnv:Services .
sysB:ServiceList a cnv:Verb ; cnv:name "ls" ; cnv:hasNoun cnv:Services .
sysC:ServiceList a cnv:Verb ; cnv:name "list-services" ; cnv:hasNoun cnv:Services .

# They can be linked:
sysA:ServiceList owl:sameAs sysB:ServiceList .
sysB:ServiceList owl:sameAs sysC:ServiceList .

# Systems understand each other automatically!
```

## Use Cases for clap-noun-verb

### 1. Agent Capability Discovery

Agents query what's available:
```sparql
SELECT ?capability WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?capability .
}
```

No hardcoded capabilities. Fully extensible.

### 2. Semantic Code Generation

Generate code from meaning, not syntax:
```
RDF ontology
    ↓ (encoded as triples)
SPARQL query capabilities
    ↓ (discover patterns)
Generate Rust code
    ↓ (with guarantee of correctness)
```

### 3. Ontology Evolution

Update specs without breaking clients:
```turtle
# Version 1
cnv:Status a cnv:Verb ; cnv:name "status" .

# Version 2 - just add properties, don't remove
cnv:Status a cnv:Verb ;
    cnv:name "status" ;
    cnv:documentation "Check service status" ;
    cnv:version "2.0" .

# Old clients ignore new properties
# New clients use them
# Both work!
```

### 4. Multi-Agent Systems

Agents coordinate through shared semantics:
```
Agent 1: "I need services to have a 'health' command"
Agent 2: "I understand. Adding to RDF..."
Agent 3: "I detected change in RDF, redeploying..."

All without custom protocol!
```

## Comparison: Semantic vs. Non-Semantic

| Aspect | Without Semantics | With RDF Semantics |
|--------|-------------------|-------------------|
| Adding new property | Update schema | Add triple, no schema change |
| Systems understanding each other | Custom integration | SPARQL queries |
| Validation | Custom code | SPARQL queries + SHACL |
| Code generation | Template strings | Type-safe from types |
| Extensibility | Forced versioning | Backward compatible |
| Reasoning | Hard-coded | Automatic inference |

## The Future of Semantic CLIs

As AI agents become more prevalent:

1. **Agents discover capabilities**: Query RDF ontologies
2. **Agents validate**: SPARQL queries check constraints
3. **Agents generate**: Type-safe code from RDF
4. **Agents coordinate**: Share ontologies, understand each other
5. **Systems evolve**: Add features without breaking clients

---

**Related**:
- [Explanation: RDF Basics](rdf-basics.md)
- [Explanation: Agent Architecture Patterns](agent-architecture.md)
- [How-to: Query with SPARQL](../howto/sparql-queries.md)
