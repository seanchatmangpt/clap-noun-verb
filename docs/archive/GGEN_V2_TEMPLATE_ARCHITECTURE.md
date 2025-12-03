# ggen v2.0 Template Architecture: Pure RDF-Driven Generation

## Vision: RDF as Single Source of Truth

**Principle**: Templates are pure rendering logic. RDF files define what to generate. No hardcoded data.

---

## Core Principles

### 1. **Pure Templates**
Templates contain **only** rendering logic:
- ✅ SPARQL queries (SELECT or CONSTRUCT)
- ✅ Template syntax (Tera)
- ✅ Logic (conditionals, loops, filters)
- ❌ NO hardcoded RDF references (`rdf: domain.ttl`)
- ❌ NO hardcoded variables (`vars:` section)
- ❌ NO hardcoded data

### 2. **RDF Files as Input**
RDF files are provided via CLI, not embedded in templates:
- ✅ RDF files specified via `--rdf` or `--graph`
- ✅ Multiple RDF files can be merged
- ✅ RDF files loaded into graph store at runtime
- ❌ NO RDF hardcoded in template frontmatter

### 3. **Minimal CLI Arguments**
CLI has minimal arguments - RDF defines everything:
- ✅ Template path: `--template templates/verb.tmpl`
- ✅ RDF input: `--rdf domain.ttl` or `--graph ontology.ttl`
- ✅ Output directory: `--output src/` (optional, can infer from RDF)
- ❌ NO `--var` flags (data comes from RDF)
- ❌ NO `--name` flags (names come from RDF)
- ❌ NO complex option lists (everything in RDF)

### 4. **Separation of Concerns**
Clear boundaries:
- **RDF** → Defines what to generate (domain model)
- **Template** → Defines how to generate it (rendering logic)
- **CLI** → Orchestrates the process (minimal coordination)

---

## Architecture Pattern

### v2.0 Workflow

```
User Command
    ↓
ggen template generate --template verb.tmpl --rdf command.ttl
    ↓
1. Load RDF file(s) into graph store
    ↓
2. Load template (parse frontmatter - queries only, no data)
    ↓
3. Execute SPARQL queries from template against graph store
    ↓
4. Render template with query results
    ↓
5. Write generated code
```

### Template Structure (v2.0)

```yaml
---
# ✅ GOOD: Pure template - no hardcoded data
# Template: templates/verb.tmpl
# Description: Generate verb command from RDF

sparql:
  # SELECT query - extracts data for template
  verbs: |
    PREFIX nv: <http://clap-noun-verb.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT ?verb ?verbName ?nounName ?description ?outputType
    WHERE {
      ?verb a nv:Verb ;
            nv:name ?verbName ;
            nv:belongsTo ?noun .
      ?noun nv:name ?nounName .
      
      OPTIONAL { ?verb nv:description ?description }
      OPTIONAL { ?verb nv:returns ?output ; rdfs:label ?outputType }
    }
    ORDER BY ?nounName ?verbName

  # CONSTRUCT query - transforms RDF into template-ready structure
  command_structure: |
    PREFIX nv: <http://clap-noun-verb.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    CONSTRUCT {
      ?verb nv:hasCommandName ?commandName ;
            nv:hasFunctionName ?functionName ;
            nv:hasOutputType ?outputType ;
            nv:hasModulePath ?modulePath ;
            nv:hasArguments ?argsList .
    }
    WHERE {
      ?verb a nv:Verb ;
            nv:name ?verbName ;
            nv:belongsTo ?noun .
      ?noun nv:name ?nounName .
      
      # Generate derived properties
      BIND(CONCAT(?nounName, "_", ?verbName) AS ?commandName)
      BIND(CONCAT("create_", ?nounName, "_", ?verbName) AS ?functionName)
      BIND(CONCAT(UCASE(SUBSTR(?nounName, 1, 1)), SUBSTR(?nounName, 2),
                  UCASE(SUBSTR(?verbName, 1, 1)), SUBSTR(?verbName, 2), 
                  "Output") AS ?outputType)
      BIND(CONCAT("crate::commands::", ?nounName, "::", ?verbName) AS ?modulePath)
      
      # Aggregate arguments
      OPTIONAL {
        ?verb nv:hasArgument ?arg .
        ?arg nv:name ?argName ;
             nv:type ?argType .
      }
      BIND(GROUP_CONCAT(CONCAT(?argName, ":", ?argType); SEPARATOR=",") AS ?argsList)
    }
---

{# ✅ Template only contains rendering logic #}
{% for cmd in query('command_structure') %}
{# FILE: src/commands/{{ cmd.nounName }}/{{ cmd.verbName }}.rs #}
/// {{ cmd.description | default("Command description") }}

use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

#[verb("{{ cmd.verbName }}", "{{ cmd.nounName }}")]
pub fn {{ cmd.commandName }}() -> Result<{{ cmd.outputType }}> {
    Ok({{ cmd.functionName }}())
}

pub fn {{ cmd.functionName }}() -> {{ cmd.outputType }} {
    {{ cmd.outputType }} {
        // Generated from RDF
    }
}

#[derive(Debug, serde::Serialize)]
pub struct {{ cmd.outputType }} {
    // Fields from RDF
}
{% endfor %}
```

### RDF File Structure

```turtle
# command.ttl - Defines what to generate
@prefix nv: <http://clap-noun-verb.org/schema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Define noun
:UtilsNoun a nv:Noun ;
    nv:name "utils" ;
    rdfs:label "utils" ;
    nv:description "Utility commands" .

# Define verb
:DoctorVerb a nv:Verb ;
    nv:name "doctor" ;
    nv:belongsTo :UtilsNoun ;
    nv:description "Check system prerequisites" ;
    nv:returns :DoctorOutput ;
    nv:hasArgument :DoctorVerb_format .

# Define output type
:DoctorOutput a nv:OutputType ;
    rdfs:label "DoctorOutput" ;
    nv:hasField :DoctorOutput_rust_ok, :DoctorOutput_git_ok .

:DoctorOutput_rust_ok a nv:Field ;
    nv:name "rust_ok" ;
    nv:type "bool" .

:DoctorOutput_git_ok a nv:Field ;
    nv:name "git_ok" ;
    nv:type "bool" .

# Define argument
:DoctorVerb_format a nv:Argument ;
    nv:name "format" ;
    nv:type "String" ;
    nv:required false ;
    nv:defaultValue "json" .
```

### CLI Usage (v2.0)

```bash
# ✅ GOOD: Minimal arguments, RDF-driven
ggen template generate \
  --template templates/verb.tmpl \
  --rdf command.ttl

# ✅ GOOD: Multiple RDF files merged
ggen template generate \
  --template templates/verb.tmpl \
  --rdf command.ttl \
  --rdf args.ttl \
  --rdf types.ttl

# ✅ GOOD: Output directory inferred from RDF or explicitly set
ggen template generate \
  --template templates/verb.tmpl \
  --rdf command.ttl \
  --output src/commands

# ❌ BAD: No variables via CLI
# ggen template generate --template verb.tmpl --var noun=utils --var verb=doctor

# ❌ BAD: No hardcoded RDF in template
# Template should NOT have: rdf: domain.ttl
```

---

## Benefits of Pure RDF-Driven Approach

### 1. **Separation of Concerns**
- **RDF** = Data (what to generate)
- **Template** = Logic (how to generate it)
- **CLI** = Coordination (orchestration)

### 2. **Flexibility**
Same template works with different RDF files:
```bash
# Generate utils/doctor from command1.ttl
ggen template generate --template verb.tmpl --rdf command1.ttl

# Generate ai/project from command2.ttl
ggen template generate --template verb.tmpl --rdf command2.ttl
```

### 3. **Reusability**
Templates are pure, reusable across projects:
```
clap-noun-verb/templates/verb.tmpl  # Pure template
ggen/templates/verb.tmpl            # Same pattern, different RDF
my-project/templates/verb.tmpl       # Reuse same pattern
```

### 4. **Testability**
Easy to test templates with different RDF inputs:
```bash
# Test template with test RDF
ggen template generate --template verb.tmpl --rdf test-command.ttl
```

### 5. **Composability**
Multiple RDF files can be merged:
```bash
# Merge base ontology + command-specific data
ggen template generate \
  --template verb.tmpl \
  --rdf base-ontology.ttl \
  --rdf command-data.ttl
```

### 6. **Version Control**
RDF files tracked separately from templates:
```
repo/
├── templates/          # Pure templates (stable)
├── domain/             # RDF domain models (evolving)
│   ├── commands.ttl
│   ├── types.ttl
│   └── relationships.ttl
```

---

## Migration Path: v1.x → v2.0

### v1.x (Current - Deprecated Patterns)

```yaml
---
# ❌ BAD: Hardcoded RDF in frontmatter
rdf: domain.ttl

# ❌ BAD: Hardcoded variables
vars:
  noun: "utils"
  verb: "create"

sparql:
  # Queries
---
```

### v2.0 (Pure Pattern)

```yaml
---
# ✅ GOOD: Pure template - no hardcoded data
# RDF provided via CLI: --rdf domain.ttl

sparql:
  # Queries only - no hardcoded data
---
```

### Breaking Changes

1. **Remove `rdf:` from frontmatter**
   - Old: `rdf: domain.ttl` in template
   - New: `--rdf domain.ttl` via CLI

2. **Remove `vars:` from frontmatter**
   - Old: `vars: { noun: "utils" }` in template
   - New: All data from RDF queries

3. **Remove `--var` CLI flags**
   - Old: `ggen template generate --var noun=utils`
   - New: Data comes from RDF only

4. **Require `--rdf` flag**
   - Old: Optional RDF reference
   - New: Required RDF input (`--rdf` or `--graph`)

---

## Template Manifest Pattern (v2.0)

For multi-template projects, use a manifest that references RDF:

```yaml
# .ggen/project.yaml
project_name: clap-noun-verb

# RDF files used by this project
rdf:
  - domain/commands.ttl
  - domain/types.ttl
  - domain/relationships.ttl

# Templates and their RDF requirements
templates:
  - name: verb
    template: templates/verb.tmpl
    rdf: domain/commands.ttl
    output: src/commands
  
  - name: noun
    template: templates/noun.tmpl
    rdf: domain/commands.ttl
    output: src/commands
  
  - name: test
    template: templates/test.tmpl
    rdf: domain/commands.ttl
    output: tests/commands
```

Usage:
```bash
# Generate from manifest
ggen project generate --manifest .ggen/project.yaml

# Generate specific template
ggen project generate --manifest .ggen/project.yaml --template verb
```

---

## Advanced Patterns

### Pattern 1: Multiple RDF Sources

```bash
# Merge base ontology + project-specific data
ggen template generate \
  --template verb.tmpl \
  --rdf ~/.ggen/base-ontology.ttl \
  --rdf project/commands.ttl
```

Templates query merged graph - get both base patterns and project data.

### Pattern 2: RDF Composition

```turtle
# base-commands.ttl - Base patterns
@prefix nv: <http://clap-noun-verb.org/schema#> .

:CommandPattern a nv:Pattern ;
    nv:structure "CLI Layer → Business Logic" .

# project-commands.ttl - Extends base
@prefix base: <http://base.example.org/> .
@prefix nv: <http://clap-noun-verb.org/schema#> .

:DoctorCommand a nv:Verb ;
    nv:follows base:CommandPattern ;
    nv:name "doctor" .
```

### Pattern 3: Template Composition

```bash
# Generate complete feature from multiple templates + RDF
ggen project generate \
  --template verb.tmpl \
  --template test.tmpl \
  --template doc.tmpl \
  --rdf command.ttl
```

Each template queries the same RDF, generates different files.

---

## Comparison: v1.x vs v2.0

| Aspect | v1.x | v2.0 |
|--------|------|------|
| **RDF Location** | ❌ Hardcoded in template | ✅ Provided via CLI |
| **Variables** | ❌ Hardcoded `vars:` section | ✅ From RDF queries only |
| **CLI Arguments** | ❌ Many `--var` flags | ✅ Minimal: `--template`, `--rdf` |
| **Template Reusability** | ❌ Tightly coupled to data | ✅ Pure, reusable |
| **Separation of Concerns** | ❌ Mixed (data + logic) | ✅ Clear (data vs logic) |
| **Testability** | ❌ Hard to test with different data | ✅ Easy - different RDF files |
| **Composability** | ❌ Single RDF source | ✅ Multiple RDF files merged |

---

## Implementation Plan

### Phase 1: Deprecate Old Patterns (v1.4.0)

- ⚠️ Deprecate `rdf:` in frontmatter
- ⚠️ Deprecate `vars:` in frontmatter
- ⚠️ Deprecate `--var` CLI flags
- ✅ Document new pure pattern

### Phase 2: Implement v2.0 (v2.0.0-beta)

- ✅ Remove `rdf:` support in frontmatter
- ✅ Remove `vars:` support in frontmatter
- ✅ Remove `--var` CLI flags
- ✅ Require `--rdf` flag
- ✅ Support multiple `--rdf` flags (merge)
- ✅ Pure template validation

### Phase 3: Tooling (v2.0.0)

- ✅ Template migration tool
- ✅ RDF validation
- ✅ Template manifest support
- ✅ Documentation

---

## Success Metrics

- ✅ **Zero hardcoded data in templates** - All data from RDF
- ✅ **Minimal CLI arguments** - Only template, RDF, output
- ✅ **100% template reusability** - Pure templates work with any RDF
- ✅ **Clear separation** - Data (RDF) vs Logic (template) vs Coordination (CLI)
- ✅ **Easy composition** - Multiple RDF files, multiple templates

---

## Conclusion

**v2.0 Principle**: Templates are pure. RDF defines everything. CLI orchestrates minimally.

**Key Insight**: Separate what (RDF) from how (template) from when (CLI).

**Result**: Maximum flexibility, reusability, and composability.

---

**Last Updated**: ggen v2.0 pure RDF-driven template architecture documented.

---

## See Also

- **[GGEN_V2_BUSINESS_LOGIC_SEPARATION.md](GGEN_V2_BUSINESS_LOGIC_SEPARATION.md)** - Business logic separation & frozen sections
- **[GGEN_V2_PROJECT_CONFIG.md](GGEN_V2_PROJECT_CONFIG.md)** - Project configuration with `ggen.toml`
- **[GGEN_V2_FILESYSTEM_ROUTING.md](GGEN_V2_FILESYSTEM_ROUTING.md)** - Filesystem-based routing conventions
- **[GGEN_V2_ARCHITECTURE_DIAGRAMS.puml](GGEN_V2_ARCHITECTURE_DIAGRAMS.puml)** - C4 architecture diagrams

