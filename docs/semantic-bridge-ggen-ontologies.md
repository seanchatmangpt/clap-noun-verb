# Semantic Bridge: ggen ↔ Open Ontologies

**Version:** 26.6.1  
**Date:** 2026-06-01  
**Status:** Design Specification  
**Authors:** Sean Chatman

---

## Executive Summary

This document defines the semantic bridge architecture between **ggen** (CLI code generator) and **Open Ontologies** (RDF/SPARQL-based command vocabularies). The bridge enables:

1. **RDF → ggen** (reverse): Query ontologies, synthesize CLI specifications, generate code
2. **ggen → RDF** (forward): Introspect generated CLIs, emit provable event logs
3. **Bidirectional sync** (future): Keep RDF and generated code in lockstep

The bridge leverages:
- **RDF Turtle/N-Triples** for command vocabulary definition
- **SPARQL** for querying command patterns
- **Rust macros** (`#[verb]`, `#[arg]`) for compile-time code generation
- **clap-noun-verb v26.6.1** stable APIs (CommandRegistry, #[verb], OutputFormat)
- **Process Mining Chicago TDD** for proof/receipt emission

---

## PART 1: Command Vocabulary Ontology (RDF)

### 1.1 Core Ontology Namespace

**File:** `ontology/ggen-command-vocab.nt`  
**Namespace URI:** `https://chatmangpt.com/ontologies/ggen/command-vocab#`  
**Prefix:** `ggen:`

```ntriples
# Ontology declaration
<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#Ontology> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "ggen Command Vocabulary Ontology"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Defines semantic vocabulary for CLI commands, their parameters, outputs, and proof receipts. Bridges ggen code generation with RDF-based ontologies."@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://purl.org/dc/terms/creator> 
  "Sean Chatman" .

<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://purl.org/dc/terms/created> 
  "2026-06-01"^^<http://www.w3.org/2001/XMLSchema#date> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://purl.org/dc/terms/license> 
  <https://opensource.org/licenses/MIT> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#> 
  <http://www.w3.org/2002/07/owl#versionInfo> 
  "5.6.0" .
```

### 1.2 CLI Command Classes

```ntriples
# CommandSpec: Abstract container for noun-verb definitions
<https://chatmangpt.com/ontologies/ggen/command-vocab#CommandSpec> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2000/01/rdf-schema#Class> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#CommandSpec> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "Command Specification"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#CommandSpec> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Specification for a CLI command; parent container for nouns and verbs"@en .

# NounSpec: Domain entity container (services, users, database)
<https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2000/01/rdf-schema#Class> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> 
  <http://www.w3.org/2000/01/rdf-schema#subClassOf> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#CommandSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "Noun Specification"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Specification for a noun (domain entity); contains verbs and sub-nouns"@en .

# VerbSpec: Action on a noun (load, query, validate, check)
<https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2000/01/rdf-schema#Class> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> 
  <http://www.w3.org/2000/01/rdf-schema#subClassOf> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#CommandSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "Verb Specification"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Specification for a verb (action); always a child of a noun"@en .

# ParameterSpec: CLI argument (flag, option, positional)
<https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2000/01/rdf-schema#Class> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "Parameter Specification"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Specification for a command-line parameter (argument, flag, option)"@en .

# OutputTypeSpec: Serializable return value (JSON, YAML)
<https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2000/01/rdf-schema#Class> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "Output Type Specification"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Specification for a verb's return type; must implement serde::Serialize"@en .

# Receipt: Proof of execution (immutable event record)
<https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2000/01/rdf-schema#Class> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "Execution Receipt"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> 
  <http://www.w3.org/2000/01/rdf-schema#comment> 
  "Immutable record of command execution; contains proof data for Process Mining Chicago TDD conformance"@en .
```

### 1.3 Noun Properties

```ntriples
# nounName: CLI identifier (e.g., "graph", "services")
<https://chatmangpt.com/ontologies/ggen/command-vocab#nounName> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#nounName> 
  <http://www.w3.org/2000/01/rdf-schema#label> 
  "noun name"@en .

<https://chatmangpt.com/ontologies/ggen/command-vocab#nounName> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#nounName> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# nounDocstring: Human-readable description (for --help)
<https://chatmangpt.com/ontologies/ggen/command-vocab#nounDocstring> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#nounDocstring> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#nounDocstring> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .

# hasVerb: Links noun to verbs
<https://chatmangpt.com/ontologies/ggen/command-vocab#hasVerb> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#ObjectProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasVerb> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasVerb> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

# hasSubNoun: Links noun to child nouns (for nesting)
<https://chatmangpt.com/ontologies/ggen/command-vocab#hasSubNoun> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#ObjectProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasSubNoun> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasSubNoun> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .
```

### 1.4 Verb Properties

```ntriples
# verbName: Action identifier (e.g., "load", "query")
<https://chatmangpt.com/ontologies/ggen/command-vocab#verbName> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#verbName> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#verbName> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# verbDocstring: Human-readable description
<https://chatmangpt.com/ontologies/ggen/command-vocab#verbDocstring> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#verbDocstring> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#verbDocstring> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# belongsToNoun: Parent noun
<https://chatmangpt.com/ontologies/ggen/command-vocab#belongsToNoun> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#ObjectProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#belongsToNoun> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#belongsToNoun> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .

# hasParameter: Parameters accepted by the verb
<https://chatmangpt.com/ontologies/ggen/command-vocab#hasParameter> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#ObjectProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasParameter> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasParameter> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

# returnsType: Output type specification
<https://chatmangpt.com/ontologies/ggen/command-vocab#returnsType> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#ObjectProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#returnsType> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#returnsType> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> .

# canFail: Whether verb may emit errors
<https://chatmangpt.com/ontologies/ggen/command-vocab#canFail> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#canFail> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#canFail> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#boolean> .
```

### 1.5 Parameter Properties

```ntriples
# paramName: CLI argument name (--verbose, path, --output)
<https://chatmangpt.com/ontologies/ggen/command-vocab#paramName> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#paramName> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#paramName> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# paramType: Rust type (String, u32, PathBuf, bool)
<https://chatmangpt.com/ontologies/ggen/command-vocab#paramType> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#paramType> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#paramType> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# isRequired: Whether parameter is mandatory
<https://chatmangpt.com/ontologies/ggen/command-vocab#isRequired> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#isRequired> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#isRequired> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#boolean> .

# paramDocstring: Help text for --help
<https://chatmangpt.com/ontologies/ggen/command-vocab#paramDocstring> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#paramDocstring> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#paramDocstring> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# shortForm: Short flag (e.g., -v for --verbose)
<https://chatmangpt.com/ontologies/ggen/command-vocab#shortForm> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#shortForm> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#shortForm> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# defaultValue: Default value if not provided
<https://chatmangpt.com/ontologies/ggen/command-vocab#defaultValue> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#defaultValue> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#defaultValue> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# validation: Validation rule (regex, enum values, range)
<https://chatmangpt.com/ontologies/ggen/command-vocab#validation> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#validation> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#validation> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .
```

### 1.6 Output Type Properties

```ntriples
# outputName: Struct name for the return type
<https://chatmangpt.com/ontologies/ggen/command-vocab#outputName> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#outputName> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#outputName> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# outputDescription: What this output represents
<https://chatmangpt.com/ontologies/ggen/command-vocab#outputDescription> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#outputDescription> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#outputDescription> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# hasField: Properties in the output struct
<https://chatmangpt.com/ontologies/ggen/command-vocab#hasField> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasField> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#hasField> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .
```

### 1.7 Receipt Properties (Proof)

```ntriples
# receiptId: Unique identifier (UUID or hash)
<https://chatmangpt.com/ontologies/ggen/command-vocab#receiptId> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#receiptId> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#receiptId> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# executionTimestamp: Unix milliseconds
<https://chatmangpt.com/ontologies/ggen/command-vocab#executionTimestamp> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#executionTimestamp> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#executionTimestamp> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#long> .

# commandPath: Noun/verb sequence (["graph", "load"])
<https://chatmangpt.com/ontologies/ggen/command-vocab#commandPath> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#commandPath> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#commandPath> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# exitCode: Exit status (0 = success)
<https://chatmangpt.com/ontologies/ggen/command-vocab#exitCode> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#exitCode> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#exitCode> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#integer> .

# inputArguments: JSON-serialized args
<https://chatmangpt.com/ontologies/ggen/command-vocab#inputArguments> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#inputArguments> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#inputArguments> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# outputData: JSON-serialized output
<https://chatmangpt.com/ontologies/ggen/command-vocab#outputData> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#outputData> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#outputData> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .

# durationMillis: Execution time
<https://chatmangpt.com/ontologies/ggen/command-vocab#durationMillis> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#durationMillis> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#durationMillis> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#long> .

# stderrOutput: Error messages if any
<https://chatmangpt.com/ontologies/ggen/command-vocab#stderrOutput> 
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> 
  <http://www.w3.org/2002/07/owl#DatatypeProperty> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#stderrOutput> 
  <http://www.w3.org/2000/01/rdf-schema#domain> 
  <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .

<https://chatmangpt.com/ontologies/ggen/command-vocab#stderrOutput> 
  <http://www.w3.org/2000/01/rdf-schema#range> 
  <http://www.w3.org/2001/XMLSchema#string> .
```

---

## PART 2: Example RDF Command Instance

**File:** `ontology/examples/graph-commands.nt`

```ntriples
# Example: "graph load" command specification

# Define the noun "graph"
<urn:ggen:specimen:graph>
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#NounSpec> .

<urn:ggen:specimen:graph>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#nounName>
  "graph"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#nounDocstring>
  "Manage RDF graph files and queries"@en .

# Define the verb "load"
<urn:ggen:specimen:graph-load-verb>
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#VerbSpec> .

<urn:ggen:specimen:graph-load-verb>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#verbName>
  "load"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-verb>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#verbDocstring>
  "Load an RDF file from disk"@en .

<urn:ggen:specimen:graph-load-verb>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#belongsToNoun>
  <urn:ggen:specimen:graph> .

<urn:ggen:specimen:graph-load-verb>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#canFail>
  "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .

# Parameter: "path"
<urn:ggen:specimen:graph-load-path>
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#ParameterSpec> .

<urn:ggen:specimen:graph-load-path>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#paramName>
  "path"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-path>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#paramType>
  "String"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-path>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#isRequired>
  "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .

<urn:ggen:specimen:graph-load-path>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#paramDocstring>
  "Path to RDF file (.ttl, .nt, .rdf)"@en .

<urn:ggen:specimen:graph-load-verb>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#hasParameter>
  <urn:ggen:specimen:graph-load-path> .

# Output type: "GraphLoadedOutput"
<urn:ggen:specimen:graph-load-output>
  <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#OutputTypeSpec> .

<urn:ggen:specimen:graph-load-output>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#outputName>
  "GraphLoadedOutput"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-output>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#outputDescription>
  "Result of loading an RDF file"@en .

<urn:ggen:specimen:graph-load-output>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#hasField>
  "triples_loaded: u64"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-output>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#hasField>
  "source: String"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-output>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#hasField>
  "status: String"^^<http://www.w3.org/2001/XMLSchema#string> .

<urn:ggen:specimen:graph-load-verb>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#returnsType>
  <urn:ggen:specimen:graph-load-output> .

# Link noun to verb
<urn:ggen:specimen:graph>
  <https://chatmangpt.com/ontologies/ggen/command-vocab#hasVerb>
  <urn:ggen:specimen:graph-load-verb> .
```

---

## PART 3: Forward Mapping Algorithm (ggen → RDF)

### 3.1 Rust Macro to RDF Triples

When `#[verb]` macro is invoked on a function, generate RDF triples:

**Input Rust Code:**
```rust
#[verb("load", "graph")]
pub fn load_graph(path: String) -> Result<GraphLoadedOutput> {
    // ... implementation
}

#[derive(Serialize)]
pub struct GraphLoadedOutput {
    pub triples_loaded: u64,
    pub source: String,
    pub status: String,
}
```

**Generated RDF (simplified):**
```
macro_input:
  - verb_name = "load"
  - noun_name = "graph"
  - function_name = "load_graph"
  - parameters = ["path: String"]
  - return_type = "GraphLoadedOutput"
  - docstring (extracted from /// comments)
  - struct_fields (extracted from return type)

output_triples:
  <urn:ggen:app:graph>
    rdf:type ggen:NounSpec ;
    ggen:nounName "graph" .

  <urn:ggen:app:graph-load>
    rdf:type ggen:VerbSpec ;
    ggen:verbName "load" ;
    ggen:belongsToNoun <urn:ggen:app:graph> ;
    ggen:returnsType <urn:ggen:app:graph-load-output> .

  <urn:ggen:app:graph-load-path>
    rdf:type ggen:ParameterSpec ;
    ggen:paramName "path" ;
    ggen:paramType "String" ;
    ggen:isRequired "true" .

  <urn:ggen:app:graph-load-verb>
    ggen:hasParameter <urn:ggen:app:graph-load-path> .

  <urn:ggen:app:graph-load-output>
    rdf:type ggen:OutputTypeSpec ;
    ggen:outputName "GraphLoadedOutput" ;
    ggen:hasField "triples_loaded: u64" ;
    ggen:hasField "source: String" ;
    ggen:hasField "status: String" .
```

### 3.2 Algorithm Pseudocode

```python
def rust_macro_to_rdf(verb_name, noun_name, function, return_type, docstring, params):
    """
    Convert parsed #[verb] metadata to RDF triples.
    """
    triples = []
    
    # 1. Create noun IRI
    noun_iri = f"urn:ggen:app:{noun_name}"
    triples.append((noun_iri, RDF.type, GGEN.NounSpec))
    triples.append((noun_iri, GGEN.nounName, noun_name))
    
    # 2. Create verb IRI
    verb_iri = f"urn:ggen:app:{noun_name}-{verb_name}"
    triples.append((verb_iri, RDF.type, GGEN.VerbSpec))
    triples.append((verb_iri, GGEN.verbName, verb_name))
    triples.append((verb_iri, GGEN.belongsToNoun, noun_iri))
    
    if docstring:
        triples.append((verb_iri, GGEN.verbDocstring, docstring))
    
    # 3. Create parameters
    for param_name, param_type in params:
        param_iri = f"urn:ggen:app:{noun_name}-{verb_name}-{param_name}"
        triples.append((param_iri, RDF.type, GGEN.ParameterSpec))
        triples.append((param_iri, GGEN.paramName, param_name))
        triples.append((param_iri, GGEN.paramType, param_type))
        triples.append((param_iri, GGEN.isRequired, is_required(param_type)))
        triples.append((verb_iri, GGEN.hasParameter, param_iri))
    
    # 4. Create output type
    output_iri = f"urn:ggen:app:{noun_name}-{verb_name}-output"
    triples.append((output_iri, RDF.type, GGEN.OutputTypeSpec))
    triples.append((output_iri, GGEN.outputName, return_type.__name__))
    
    for field_name, field_type in return_type.__fields__:
        field_decl = f"{field_name}: {field_type}"
        triples.append((output_iri, GGEN.hasField, field_decl))
    
    triples.append((verb_iri, GGEN.returnsType, output_iri))
    
    # 5. Link noun to verb
    triples.append((noun_iri, GGEN.hasVerb, verb_iri))
    
    return triples
```

---

## PART 4: Reverse Mapping Algorithm (RDF → ggen)

### 4.1 SPARQL Query to Extract Command Specs

**File:** `ontology/queries/extract-command-specs.rq`

```sparql
PREFIX ggen: <https://chatmangpt.com/ontologies/ggen/command-vocab#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# Extract all verb specifications with their parameters and return types
CONSTRUCT {
  ?verb
    ggen:verbName ?verbName ;
    ggen:verbDocstring ?verbDocstring ;
    ggen:belongsToNoun ?noun ;
    ggen:hasParameter ?param ;
    ggen:returnsType ?outputType .
    
  ?noun
    ggen:nounName ?nounName ;
    ggen:nounDocstring ?nounDocstring .
    
  ?param
    ggen:paramName ?paramName ;
    ggen:paramType ?paramType ;
    ggen:isRequired ?isRequired ;
    ggen:paramDocstring ?paramDocstring .
    
  ?outputType
    ggen:outputName ?outputName ;
    ggen:hasField ?field .
}
WHERE {
  ?verb rdf:type ggen:VerbSpec ;
        ggen:verbName ?verbName ;
        ggen:belongsToNoun ?noun .
        
  OPTIONAL { ?verb ggen:verbDocstring ?verbDocstring }
  
  ?noun rdf:type ggen:NounSpec ;
        ggen:nounName ?nounName .
  OPTIONAL { ?noun ggen:nounDocstring ?nounDocstring }
  
  OPTIONAL {
    ?verb ggen:hasParameter ?param .
    ?param ggen:paramName ?paramName ;
           ggen:paramType ?paramType .
    OPTIONAL { ?param ggen:isRequired ?isRequired }
    OPTIONAL { ?param ggen:paramDocstring ?paramDocstring }
  }
  
  ?verb ggen:returnsType ?outputType .
  ?outputType ggen:outputName ?outputName .
  OPTIONAL { ?outputType ggen:hasField ?field }
}
```

### 4.2 Algorithm: RDF → Rust ggen Spec

```python
def rdf_to_ggen_spec(rdf_graph, target_namespace):
    """
    Query RDF ontology for command definitions and generate ggen specification.
    """
    # 1. SPARQL query for all verbs
    results = sparql_query(rdf_graph, EXTRACT_COMMAND_SPECS_QUERY)
    
    commands = {}
    for row in results:
        verb_name = row['verbName']
        noun_name = row['nounName']
        docstring = row.get('verbDocstring', '')
        
        # 2. Aggregate parameters
        params = []
        for param_row in rows_with_same_verb(results, verb_name):
            if param_row['paramName']:
                params.append({
                    'name': param_row['paramName'],
                    'type': param_row['paramType'],
                    'required': param_row['isRequired'] == 'true',
                    'doc': param_row.get('paramDocstring', '')
                })
        
        # 3. Extract output type
        output_type = {
            'name': row['outputName'],
            'fields': extract_fields(row['field'])
        }
        
        # 4. Create ggen spec
        if noun_name not in commands:
            commands[noun_name] = {
                'name': noun_name,
                'doc': row.get('nounDocstring', ''),
                'verbs': []
            }
        
        commands[noun_name]['verbs'].append({
            'name': verb_name,
            'doc': docstring,
            'params': params,
            'returns': output_type
        })
    
    # 5. Generate ggen-compatible YAML/Turtle spec
    return generate_ggen_spec(commands)
```

### 4.3 Generated ggen Spec Output

**File:** `target/generated-spec.yaml`

```yaml
app_name: specimen_graph_manager
version: "1.0.0"

nouns:
  - name: graph
    docstring: "Manage RDF graph files and queries"
    verbs:
      - name: load
        docstring: "Load an RDF file from disk"
        params:
          - name: path
            type: String
            required: true
            doc: "Path to RDF file (.ttl, .nt, .rdf)"
        returns:
          name: GraphLoadedOutput
          fields:
            - triples_loaded: u64
            - source: String
            - status: String
      
      - name: query
        docstring: "Query graph with SPARQL pattern"
        params:
          - name: pattern
            type: String
            required: true
            doc: "SPARQL query pattern"
        returns:
          name: QueryResultOutput
          fields:
            - query_type: String
            - pattern: String
            - results: Vec
            - match_count: u64
```

---

## PART 5: Bidirectional Sync Protocol

### 5.1 Sync Workflow

```
┌─────────────────────────────────────────────────────────────┐
│  Source: Rust Code (#[verb] macros)                         │
│  ↓ (emit_rdf phase)                                         │
│  RDF Instance Data (ontology/examples/)                     │
│  ↓ (introspect phase)                                       │
│  Stored in triplestore or file                              │
│  ↓ (query phase)                                            │
│  SPARQL query results                                        │
│  ↓ (generate phase)                                         │
│  ggen Specification (YAML/Turtle)                           │
│  ↓ (codegen phase)                                          │
│  Generated Rust Code (#[verb], tests, docs)                │
│  ↓ (validation phase)                                       │
│  Compile check, test suite                                  │
│  ↓ (emit_receipts phase)                                    │
│  Execution logs (Process Mining Chicago TDD)                │
│  ↓ (proof_conformance phase)                                │
│  SPARQL queries to validate event log against model         │
│  ✓ CONFORMANCE VERIFIED                                     │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Sync Invariants

**After any change, these must hold:**

1. **RDF completeness:** Every `#[verb]` in Rust has a corresponding VerbSpec in RDF
2. **Parameter consistency:** Parameter name/type in RDF matches Rust function signature
3. **Output consistency:** Return type in RDF matches Serialize struct in Rust
4. **Docstring consistency:** Doc comments in Rust match docstring properties in RDF
5. **Receipt conformance:** Execution events match declared command structure

### 5.3 Conflict Resolution

| Conflict Type | Resolution | Rationale |
|---|---|---|
| RDF says param required, Rust has default | Trust RDF; update Rust to require | RDF is truth |
| Rust adds new verb, RDF outdated | Emit RDF triple; update store | Macro-time is canonical |
| RDF param name changes | Regenerate Rust; update macro call | RDF drives codegen |
| Execution receipt violates model | Reject execution as non-conformant | Process Mining Chicago TDD |

---

## PART 6: Receipt Emission (Proof Generation)

### 6.1 Receipt Structure

**File:** `src/proof.rs` (planned v26.7.0)

```rust
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub timestamp: i64,  // Unix milliseconds
    pub command_path: Vec<String>,  // ["graph", "load"]
    pub exit_code: i32,  // 0 = success
    pub input_args: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub stderr: Option<String>,
}

impl Receipt {
    pub fn to_rdf_triples(&self) -> Vec<String> {
        // Convert Receipt to RDF N-Triples for Process Mining
        vec![
            format!(
                "<urn:receipt:{}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> \
                 <https://chatmangpt.com/ontologies/ggen/command-vocab#Receipt> .",
                self.id
            ),
            format!(
                "<urn:receipt:{}> <https://chatmangpt.com/ontologies/ggen/command-vocab#receiptId> \
                 \"{}\" .",
                self.id, self.id
            ),
            // ... more triples
        ]
    }
}
```

### 6.2 Receipt Emission in Verb Handler

```rust
#[verb("load", "graph")]
pub fn load_graph(args: VerbArgs) -> Result<GraphLoadedOutput> {
    let start = std::time::Instant::now();
    
    // ... handler implementation
    
    let output = GraphLoadedOutput { /* ... */ };
    let duration_ms = start.elapsed().as_millis() as u64;
    
    // Emit receipt for Process Mining Chicago TDD
    let receipt = Receipt {
        id: generate_receipt_id(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
        command_path: vec!["graph".to_string(), "load".to_string()],
        exit_code: 0,
        input_args: serde_json::json!({ "path": args.matches.get_one::<String>("path") }),
        output: serde_json::to_value(&output)?,
        duration_ms,
        stderr: None,
    };
    
    // Write receipt to event log
    emit_receipt(&receipt)?;
    
    Ok(output)
}
```

---

## PART 7: SPARQL Conformance Queries

### 7.1 Process Mining Chicago TDD Validation

**File:** `ontology/queries/conformance-check.rq`

```sparql
PREFIX ggen: <https://chatmangpt.com/ontologies/ggen/command-vocab#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

# ASK query: Is execution log conformant with declared model?
ASK {
  # For each receipt in the event log
  ?receipt rdf:type ggen:Receipt .
  ?receipt ggen:commandPath ?path .
  ?receipt ggen:exitCode ?exitCode .
  
  # Extract noun/verb from path
  BIND(SUBSTR(?path, 1, STRLEN(?path) - 5) AS ?noun)
  
  # Verify the verb is declared in the ontology
  ?verb rdf:type ggen:VerbSpec .
  ?verb ggen:belongsToNoun ?nounSpec .
  ?nounSpec ggen:nounName ?noun .
  
  # Verify parameters in receipt match declared parameters
  ?receipt ggen:inputArguments ?args .
  ?verb ggen:hasParameter ?param .
  ?param ggen:paramName ?paramName .
}
```

### 7.2 Fitness Metric

```sparql
# Count conformant executions vs. total
SELECT (COUNT(?conformant) AS ?fitness) 
       (COUNT(?total) AS ?total_executions)
WHERE {
  ?receipt rdf:type ggen:Receipt .
  BIND(?receipt AS ?total)
  
  OPTIONAL {
    # This receipt conforms to model (ASK query succeeded)
    INCLUDE <conformance-check.rq>
    BIND(?receipt AS ?conformant)
  }
}
```

---

## PART 8: Implementation Roadmap

### v26.6.1 (Now)
- [ ] Create `ontology/ggen-command-vocab.nt` (DONE: RDF vocabulary)
- [ ] Create example instance `ontology/examples/graph-commands.nt`
- [ ] Document forward/reverse mapping algorithms (THIS DOCUMENT)

### v26.7.0 (Q3 2026)
- [ ] Implement `Receipt` struct in `src/proof.rs`
- [ ] Add `emit_receipt()` functionality to verb handlers
- [ ] Implement `--introspect` flag in CommandRegistry
- [ ] Export command metadata as JSON Schema

### v26.8.0+ (Q4 2026+)
- [ ] Implement SPARQL query support
- [ ] Add Process Mining Chicago TDD conformance checker
- [ ] Create `ggen` Python/Rust tool for RDF ↔ CLI generation
- [ ] Enable bidirectional sync (RDF ↔ Rust code)

---

## PART 9: Example: End-to-End Workflow

### Scenario: Generate CLI from RDF Ontology

**Input:** `ontology/examples/web-server.nt` (RDF spec)

**Step 1: Query RDF**
```bash
sparql \
  --data ontology/ggen-command-vocab.nt \
  --data ontology/examples/web-server.nt \
  --query ontology/queries/extract-command-specs.rq \
  > /tmp/specs.rdf
```

**Step 2: Convert to ggen Spec**
```python
from rdflib import Graph

g = Graph()
g.parse('/tmp/specs.rdf', format='nt')

spec = rdf_to_ggen_spec(g, 'web_server')
spec.to_yaml('target/web-server-spec.yaml')
```

**Output:** `target/web-server-spec.yaml`
```yaml
app_name: web_server
verbs:
  - noun: server
    verb: start
    params:
      - name: port
        type: u16
        required: true
```

**Step 3: Generate Rust Code**
```bash
clap-noun-verb-gen \
  --spec target/web-server-spec.yaml \
  --output src/commands/
```

**Step 4: Compile & Test**
```bash
cargo make build
cargo make test
```

**Step 5: Run & Emit Receipts**
```bash
cargo run -- server start --port 8080
```

**Step 6: Validate Conformance**
```sparql
# Query event logs against model
sparql \
  --data ontology/ggen-command-vocab.nt \
  --data ontology/examples/web-server.nt \
  --data event-logs.nt \
  --query ontology/queries/conformance-check.rq
```

**Output:** `true` (conforms) or `false` (violates model)

---

## Summary

The semantic bridge connects:

| Component | Role | Technology |
|---|---|---|
| **RDF Vocabulary** | Define command semantics | Turtle/N-Triples |
| **SPARQL Queries** | Extract specs, validate | SPARQL 1.1 |
| **Forward Mapper** | Rust → RDF | Macro introspection |
| **Reverse Mapper** | RDF → Rust | Code generation |
| **Receipts** | Proof of execution | Immutable event logs |
| **Process Mining** | Conformance validation | SPARQL + pm4py |

This enables **graph law manufacturing** where CLIs are declaratively specified, code-generated, tested, and validated against formal models—all driven by open RDF ontologies.

---

**End of Specification**
