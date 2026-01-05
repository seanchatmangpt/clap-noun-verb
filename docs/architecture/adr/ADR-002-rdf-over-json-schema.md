# ADR-002: RDF/OWL for Capability Description Over JSON Schema

## Status
Accepted

## Context

CLI capabilities need formal, machine-readable descriptions for:
- Type-safe cross-boundary invocation
- Semantic discovery queries
- Compatibility verification
- Schema evolution

Two primary schema languages were considered:
1. **JSON Schema**: Widely adopted, simple, JSON-native
2. **RDF/OWL**: W3C standard, semantic reasoning, linked data

## Decision

We will use **RDF/OWL with SHACL constraints** for capability descriptions.

## Rationale

### Capability Requirements

| Requirement              | JSON Schema | RDF/OWL | Winner  |
|--------------------------|-------------|---------|---------|
| Type validation          | ✓           | ✓       | Tie     |
| Semantic queries         | ✗           | ✓       | RDF     |
| Reasoning/inference      | ✗           | ✓       | RDF     |
| Schema composition       | Limited     | Native  | RDF     |
| Tool ecosystem           | Excellent   | Good    | JSON    |
| Linked data              | ✗           | ✓       | RDF     |
| Formal semantics         | Weak        | Strong  | RDF     |
| Learning curve           | Low         | High    | JSON    |

### Key Decision Factors

1. **Semantic Queries**: Need to ask "find all CLIs that accept image files and output JSON" - requires reasoning over type hierarchies
2. **Schema Composition**: CLIs extend base ontologies; RDF naturally supports composition via imports
3. **Formal Semantics**: OWL provides formal logic for compatibility checking
4. **Linked Data**: RDF URIs enable global namespacing without collisions
5. **Inference**: OWL reasoners can derive type compatibility (e.g., PNG ⊆ Image)

### Example: Why JSON Schema Falls Short

**JSON Schema** (limited semantics):
```json
{
  "command": "convert",
  "input": {"type": "string", "format": "uri"},
  "output": {"type": "string"}
}
```

**RDF/OWL** (rich semantics):
```turtle
:ConvertCommand a clicap:Command ;
  clicap:accepts [ a clicap:ImageFile ;
                   clicap:format :PNG, :JPEG ] ;
  clicap:produces [ a clicap:StructuredData ;
                    clicap:serialization :JSON ] ;
  clicap:typeSignature "ImageFile -> JSON"^^xsd:string .

:PNG rdfs:subClassOf :ImageFile .
:JSON rdfs:subClassOf :StructuredData .
```

With RDF, we can query: "Find commands that accept any image format" (inference: PNG ⊆ ImageFile).

### Trade-offs

**Costs**:
- Higher learning curve (RDF/OWL/SPARQL)
- More complex tooling (reasoners, triple stores)
- Larger serialization size (Turtle/RDF-XML vs JSON)
- Slower parsing (triple parsing vs. JSON)

**Benefits**:
- Semantic discovery (find by meaning, not just name)
- Formal type compatibility proofs
- Global namespacing (URIs prevent collisions)
- Schema evolution via versioned ontologies
- Reasoning (infer new facts from published data)

## Consequences

### Positive
- Enables expressive semantic queries across federated network
- Type compatibility provably correct via OWL reasoning
- Natural extension mechanism via ontology imports
- Global CLI capability marketplace possible (linked data web)

### Negative
- Requires SPARQL knowledge for discovery queries
- RDF parsing slower than JSON (mitigated by caching)
- Tooling less mature than JSON ecosystem
- Debugging ontologies requires specialized tools (Protégé)

### Mitigation Strategies

1. **Learning Curve**: Provide "RDF for CLI Developers" tutorial
2. **Tooling**: Use Oxigraph (Rust-native SPARQL engine)
3. **Performance**: Cache compiled schemas; use binary RDF (HDT format)
4. **Debugging**: Integrate SHACL validation with clear error messages

## Alternatives Considered

### 1. Hybrid Approach (JSON Schema + RDF Mapping)
- Developers write JSON Schema
- Automated translation to RDF

**Rejected because**:
- Mapping loses semantic richness
- Two sources of truth (synchronization issues)
- Complexity without semantic benefits

### 2. Protocol Buffers with Annotations
- Use protobuf for types + custom annotations for semantics

**Rejected because**:
- Annotations not standardized
- No querying capability
- Manual semantic interpretation required

## Validation

Success metrics:
- 90% of CLI developers can write basic RDF ontologies after 2-hour tutorial
- SPARQL query latency p99 < 50ms on 10K ontologies
- Zero namespace collision incidents across federated network
- 100+ semantic discovery queries successfully answered

## References

- [OWL 2 Web Ontology Language](https://www.w3.org/TR/owl2-overview/)
- [SHACL Shapes Constraint Language](https://www.w3.org/TR/shacl/)
- [JSON Schema Limitations](https://json-schema.org/understanding-json-schema/about.html)
- [Semantic Web for the Working Ontologist](https://www.amazon.com/Semantic-Web-Working-Ontologist-Effective/dp/0123859654)
