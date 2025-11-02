****# Project-Embedded Templates

## Principle: Templates Live with the Project

**Concept**: Project repositories contain their own templates, enabling project-specific code generation that follows the project's conventions, patterns, and structure.

---

## Architecture Pattern

### Embedded Template Structure

```
clap-noun-verb/
├── templates/                    # ✅ Project-specific templates
│   ├── verb.tmpl                 # Generate new verb commands
│   ├── noun.tmpl                 # Generate new noun commands
│   ├── test.tmpl                  # Generate tests
│   ├── doc.tmpl                   # Generate documentation
│   └── example.tmpl                # Generate examples
├── .ggen/                        # ggen configuration
│   ├── project.yaml               # Template manifest
│   └── queries.yaml                # Shared SPARQL queries
├── domain.ttl                     # Project ontology (optional)
└── ... (project code)
```

### Template Discovery Hierarchy

ggen searches for templates in this order:

1. **Project templates** (`.ggen/templates/` or `templates/`)
2. **Local templates** (`~/.ggen/templates/`)
3. **Marketplace templates** (`ggen marketplace install`)
4. **Global templates** (`/usr/local/share/ggen/templates/`)

---

## Benefits

### 1. **Project-Specific Patterns**
Templates match the project's conventions:
- Code style and formatting
- Architecture patterns
- Testing conventions
- Documentation standards

### 2. **Version Control**
Templates are versioned with the project:
- Track template changes with code changes
- Review template updates in PRs
- Roll back templates with code rollbacks

### 3. **Onboarding**
Generate starter code for new contributors:
```bash
# New contributor generates a new verb command
cd clap-noun-verb
ggen template generate templates/verb.tmpl --var verb_name=create
```

### 4. **Consistency**
All generated code follows project patterns:
- Same file structure
- Same naming conventions
- Same testing patterns
- Same documentation style

### 5. **Project Evolution**
Templates evolve with the project:
- Add new patterns to templates
- Update templates when architecture changes
- Templates stay in sync with codebase

---

## Example: clap-noun-verb Templates

### Project Structure

```
clap-noun-verb/
├── templates/
│   ├── verb.tmpl                  # Generate verb commands
│   ├── noun.tmpl                  # Generate noun commands
│   ├── test.tmpl                  # Generate tests
│   ├── doc.tmpl                   # Generate docs
│   └── example.tmpl               # Generate examples
├── .ggen/
│   ├── project.yaml                # Template manifest
│   ├── queries.yaml                # Shared SPARQL queries
│   └── config.yaml                 # Project config
├── domain.ttl                      # Project ontology
└── ... (project code)
```

### Template: `templates/verb.tmpl`

```yaml
---
# Verb Command Generator for clap-noun-verb
# Generates: src/commands/{noun}/{verb}.rs

rdf: domain.ttl  # ✅ Required - RDF is source of truth for template variables

sparql:
  # SELECT query - extracts data for template
  nouns: |
    PREFIX nv: <http://clap-noun-verb.org/schema#>
    SELECT ?noun ?name ?description
    WHERE {
      ?noun a nv:Noun ;
            nv:name ?name ;
            nv:description ?description .
    }

  # CONSTRUCT query - transforms/generates RDF for template consumption
  verb_structure: |
    PREFIX nv: <http://clap-noun-verb.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    CONSTRUCT {
      ?verb nv:hasCommandName ?commandName ;
            nv:hasFunctionName ?functionName ;
            nv:hasOutputType ?outputType ;
            nv:hasDescription ?description .
    }
    WHERE {
      ?verb a nv:Verb ;
            nv:name ?name ;
            nv:belongsTo ?noun .
      ?noun nv:name ?nounName .
      
      # Generate command name (noun_verb)
      BIND(CONCAT(?nounName, "_", ?name) AS ?commandName)
      
      # Generate function name (noun_verb)
      BIND(CONCAT("create_", ?nounName, "_", ?name) AS ?functionName)
      
      # Generate output type (NounNameVerbNameOutput)
      BIND(CONCAT(UCASE(SUBSTR(?nounName, 1, 1)), SUBSTR(?nounName, 2), 
                  UCASE(SUBSTR(?name, 1, 1)), SUBSTR(?name, 2), "Output") AS ?outputType)
      
      OPTIONAL { ?verb nv:description ?description }
    }

# ✅ All template variables come from RDF queries above
# No hardcoded vars - RDF is source of truth
---
{% for verb_data in query('verb_structure') %}
{# FILE: src/commands/{{ verb_data.nounName }}/{{ verb_data.verbName }}.rs #}
//! {{ verb_data.verbName | capitalize }} command for {{ verb_data.nounName }} noun

use crate::commands::{{ verb_data.nounName }}::{{ verb_data.verbName }}::*;
use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

{% if verb_data.description %}
/// {{ verb_data.description }}
///
/// {{ verb_data.description }}
{% else %}
/// {{ verb_data.verbName | capitalize }} a new {{ verb_data.nounName }} item
///
/// This command creates a new {{ verb_data.nounName }} item.
{% endif %}
#[verb("{{ verb_data.verbName }}", "{{ verb_data.nounName }}")]
pub fn {{ verb_data.commandName }}(
    name: String,
    description: Option<String>,
) -> Result<{{ verb_data.outputType }}> {
    Ok({{ verb_data.functionName }}(name, description))
}

/// Business logic for creating a {{ verb_data.nounName }} item
pub fn {{ verb_data.functionName }}(
    name: String,
    description: Option<String>,
) -> {{ verb_data.outputType }} {
    {{ verb_data.outputType }} {
        name,
        description,
        created: true,
    }
}

#[derive(Debug, serde::Serialize)]
pub struct {{ verb_data.outputType }} {
    pub name: String,
    pub description: Option<String>,
    pub created: bool,
}
{% endfor %}
```

### Template: `templates/test.tmpl`

```yaml
---
# Test Generator for clap-noun-verb
# Generates: tests/commands/{noun}/{verb}_test.rs

rdf: domain.ttl

vars:
  noun: "utils"
  verb: "create"
  output_dir: "tests/commands"
---
{% if noun and verb %}
{# FILE: tests/commands/{{ noun }}/{{ verb }}_test.rs #}
#[cfg(test)]
mod tests {
    use super::*;
    use clap_noun_verb::Result;

    #[test]
    fn test_{{ noun }}_{{ verb }}() -> Result<()> {
        // Arrange
        let name = "test".to_string();
        let description = Some("Test description".to_string());

        // Act
        let output = create_{{ noun }}_{{ verb }}(name.clone(), description.clone())?;

        // Assert
        assert_eq!(output.name, name);
        assert_eq!(output.description, description);
        assert!(output.created);

        Ok(())
    }
}
{% endif %}
```

### Template Manifest: `.ggen/project.yaml`

```yaml
# clap-noun-verb Template Manifest
project_name: clap-noun-verb
rdf: domain.ttl  # Optional

templates:
  # Verb command templates
  - name: verb
    template: templates/verb.tmpl
    description: Generate a new verb command
    vars:
      - noun
      - verb
  
  # Noun command templates
  - name: noun
    template: templates/noun.tmpl
    description: Generate a new noun command group
    vars:
      - noun
  
  # Test templates
  - name: test
    template: templates/test.tmpl
    description: Generate tests for a command
    vars:
      - noun
      - verb
  
  # Documentation templates
  - name: doc
    template: templates/doc.tmpl
    description: Generate documentation for a command
    vars:
      - noun
      - verb
  
  # Example templates
  - name: example
    template: templates/example.tmpl
    description: Generate example code
    vars:
      - noun
      - verb
```

---

## Usage Patterns

### Pattern 1: Generate Command from Project Template

```bash
# Generate a new verb command using project templates
cd clap-noun-verb
ggen template generate templates/verb.tmpl \
  --var noun=utils \
  --var verb=create

# Output: src/commands/utils/create.rs
```

### Pattern 2: Generate from Template Manifest

```bash
# Generate using template manifest
cd clap-noun-verb
ggen project generate \
  --manifest .ggen/project.yaml \
  --template verb \
  --var noun=utils \
  --var verb=create
```

### Pattern 3: Generate Multiple Files

```bash
# Generate command + test + doc + example
cd clap-noun-verb
ggen project generate \
  --manifest .ggen/project.yaml \
  --template verb \
  --template test \
  --template doc \
  --template example \
  --var noun=utils \
  --var verb=create
```

### Pattern 4: Generate from RDF

```bash
# Generate from RDF ontology
cd clap-noun-verb
ggen template generate templates/verb.tmpl \
  --rdf domain.ttl \
  --var noun=utils
```

---

## Template Discovery

### Discovery Order

ggen searches for templates in this order:

1. **Project templates** (highest priority)
   - `.ggen/templates/`
   - `templates/`
   - `src/templates/`

2. **Local user templates**
   - `~/.ggen/templates/`

3. **Marketplace templates**
   - `~/.ggen/marketplace/`

4. **Global templates** (lowest priority)
   - `/usr/local/share/ggen/templates/`
   - `/usr/share/ggen/templates/`

### Template Resolution

```bash
# Relative path - searches project templates first
ggen template generate verb.tmpl

# Absolute path - uses exact path
ggen template generate ~/.ggen/templates/verb.tmpl

# Marketplace template - uses marketplace
ggen template generate marketplace:clap-noun-verb/verb.tmpl

# Priority resolution
# 1. Check project templates/
# 2. Check .ggen/templates/
# 3. Check ~/.ggen/templates/
# 4. Check marketplace
# 5. Check global templates
```

---

## Configuration

### Project Config: `.ggen/config.yaml`

```yaml
# clap-noun-verb ggen configuration
project_name: clap-noun-verb

# Template directories
template_dirs:
  - templates/
  - .ggen/templates/

# RDF configuration
rdf:
  default_ontology: domain.ttl
  prefixes:
    nv: http://clap-noun-verb.org/schema#

# Output configuration
output:
  default_dir: src/commands
  test_dir: tests/commands
  doc_dir: docs/commands

# Project-specific variables
vars:
  project_name: clap-noun-verb
  author: "ggen team"
  license: "MIT"
```

---

## CLI Integration

### New Commands for Project Templates

```bash
# Generate from project templates
ggen project generate \
  --template verb \
  --var noun=utils \
  --var verb=create

# List project templates
ggen template list --project

# Show project template manifest
ggen template show --manifest

# Validate project templates
ggen template validate --project
```

---

## Examples

### Example 1: Add New Verb to clap-noun-verb

```bash
cd clap-noun-verb

# Generate new verb command
ggen project generate \
  --template verb \
  --var noun=utils \
  --var verb=doctor

# Generates:
# - src/commands/utils/doctor.rs
```

### Example 2: Generate Complete Command (Command + Test + Doc)

```bash
cd clap-noun-verb

# Generate everything for a new command
ggen project generate \
  --template verb \
  --template test \
  --template doc \
  --template example \
  --var noun=utils \
  --var verb=help-me

# Generates:
# - src/commands/utils/help-me.rs
# - tests/commands/utils/help-me_test.rs
# - docs/commands/utils/help-me.md
# - examples/help-me.rs
```

### Example 3: Generate from RDF with CONSTRUCT

```bash
cd clap-noun-verb

# Define command in RDF
cat > new-command.ttl <<EOF
@prefix nv: <http://clap-noun-verb.org/schema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:DoctorCommand a nv:Verb ;
    rdfs:label "doctor" ;
    nv:name "doctor" ;
    nv:belongsTo :UtilsNoun ;
    nv:description "Check system prerequisites" ;
    nv:hasArgument :DoctorVerb_name, :DoctorVerb_format ;
    nv:returns :DoctorOutput .

:DoctorVerb_name a nv:Argument ;
    nv:name "name" ;
    nv:type "String" ;
    nv:required true .

:DoctorVerb_format a nv:Argument ;
    nv:name "format" ;
    nv:type "String" ;
    nv:required false ;
    nv:defaultValue "json" .

:UtilsNoun a nv:Noun ;
    nv:name "utils" ;
    rdfs:label "utils" .
EOF

# Template uses CONSTRUCT to transform RDF into template-friendly structure
# CONSTRUCT generates: command names, function names, output types, etc.
# Then template iterates over CONSTRUCT results
ggen template generate templates/verb.tmpl \
  --rdf new-command.ttl

# Output: Generated code from RDF structure via CONSTRUCT transformation
```

---

## RDF as Source of Truth

### Principle: All Template Variables from RDF

**Key Insight**: Template variables should come from RDF queries, not hardcoded `vars:` section.

### RDF-Driven Variables

```yaml
---
# ✅ GOOD: Variables from RDF
rdf: domain.ttl

sparql:
  nouns: |
    PREFIX nv: <http://clap-noun-verb.org/schema#>
    SELECT ?noun ?name ?description
    WHERE {
      ?noun a nv:Noun ;
            nv:name ?name ;
            nv:description ?description .
    }

# ✅ Template uses query results, not hardcoded vars
---

{% for noun in query('nouns') %}
{# Generated code uses {{ noun.name }}, {{ noun.description }} from RDF #}
{% endfor %}
```

### SPARQL CONSTRUCT for Data Transformation

**CONSTRUCT queries** transform/generate RDF data into template-friendly structures:

```yaml
---
rdf: domain.ttl

sparql:
  # CONSTRUCT query - transforms RDF into template-ready structure
  command_structure: |
    PREFIX nv: <http://clap-noun-verb.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    CONSTRUCT {
      ?command nv:hasCommandName ?commandName ;
               nv:hasFunctionName ?functionName ;
               nv:hasOutputType ?outputType ;
               nv:hasModulePath ?modulePath ;
               nv:hasDescription ?description ;
               nv:hasArguments ?argsList .
    }
    WHERE {
      ?command a nv:Verb ;
               nv:name ?verbName ;
               nv:belongsTo ?noun .
      ?noun nv:name ?nounName .
      
      # Generate derived properties for template
      BIND(CONCAT(?nounName, "_", ?verbName) AS ?commandName)
      BIND(CONCAT("create_", ?nounName, "_", ?verbName) AS ?functionName)
      BIND(CONCAT(UCASE(SUBSTR(?nounName, 1, 1)), SUBSTR(?nounName, 2),
                  UCASE(SUBSTR(?verbName, 1, 1)), SUBSTR(?verbName, 2), 
                  "Output") AS ?outputType)
      BIND(CONCAT("crate::commands::", ?nounName, "::", ?verbName) AS ?modulePath)
      
      # Collect all arguments
      OPTIONAL {
        ?command nv:hasArgument ?arg .
        ?arg nv:name ?argName ;
             nv:type ?argType .
      }
      BIND(GROUP_CONCAT(CONCAT(?argName, ":", ?argType); SEPARATOR=",") AS ?argsList)
      
      OPTIONAL { ?command nv:description ?description }
    }

# Template uses CONSTRUCT results directly
---

{% for cmd in query('command_structure') %}
// Generated from RDF via CONSTRUCT
// Command: {{ cmd.commandName }}
// Function: {{ cmd.functionName }}
// Output: {{ cmd.outputType }}
// Module: {{ cmd.modulePath }}
// Args: {{ cmd.argsList }}
{% endfor %}
```

### Benefits of CONSTRUCT

1. **Data Transformation**: Reshape RDF into template-friendly structures
2. **Derived Properties**: Generate computed values (function names, types, etc.)
3. **Aggregation**: Combine multiple RDF triples into single template variables
4. **Normalization**: Standardize data formats across different RDF sources
5. **Composition**: Merge data from multiple sources into unified structure

### CONSTRUCT Workflow

```
RDF Domain Model (domain.ttl)
    ↓
SPARQL CONSTRUCT Query (transform)
    ↓
Transformed RDF (template-ready structure)
    ↓
Template Rendering (uses transformed RDF)
    ↓
Generated Code
```

### Example: CONSTRUCT for FastAPI Generation

```yaml
---
rdf: api-domain.ttl

sparql:
  # Transform RDF API model into template-ready structure
  api_structure: |
    PREFIX api: <http://api.example.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    CONSTRUCT {
      ?model api:hasPydanticSchema ?schemaName ;
             api:hasSQLAlchemyModel ?modelName ;
             api:hasRouteModule ?routeModule ;
             api:hasTestModule ?testModule ;
             api:hasFields ?fieldsList ;
             api:hasRelationships ?relsList .
    }
    WHERE {
      ?model a api:Model ;
             rdfs:label ?label .
      
      # Generate Python names
      BIND(CONCAT(UCASE(SUBSTR(?label, 1, 1)), SUBSTR(?label, 2)) AS ?modelName)
      BIND(CONCAT(?modelName, "Schema") AS ?schemaName)
      BIND(CONCAT("api.routes.", LCASE(?label), "s") AS ?routeModule)
      BIND(CONCAT("tests.test_", LCASE(?label), "s") AS ?testModule)
      
      # Aggregate fields
      OPTIONAL {
        ?model api:hasField ?field .
        ?field api:fieldName ?fieldName ;
               api:pythonType ?pythonType .
      }
      BIND(GROUP_CONCAT(CONCAT(?fieldName, ":", ?pythonType); SEPARATOR=",") AS ?fieldsList)
      
      # Aggregate relationships
      OPTIONAL {
        ?model api:hasRelationship ?rel .
        ?rel api:relationshipName ?relName ;
             api:targetModel ?targetModel .
        ?targetModel rdfs:label ?targetLabel .
      }
      BIND(GROUP_CONCAT(CONCAT(?relName, "->", ?targetLabel); SEPARATOR=",") AS ?relsList)
    }

---
{% for model in query('api_structure') %}
{# Generate Pydantic schema: {{ model.schemaName }} #}
{# Generate SQLAlchemy model: {{ model.modelName }} #}
{# Generate route module: {{ model.routeModule }} #}
{# Generate test module: {{ model.testModule }} #}
{# Fields: {{ model.fieldsList }} #}
{# Relationships: {{ model.relsList }} #}
{% endfor %}
```

### SELECT vs CONSTRUCT

| Query Type | Use Case | Output |
|------------|----------|--------|
| **SELECT** | Extract data for direct template use | Table of variables |
| **CONSTRUCT** | Transform/generate RDF structure | New RDF graph |

**SELECT Example**:
```sparql
SELECT ?noun ?name WHERE {
  ?noun a nv:Noun ; nv:name ?name .
}
```
→ Returns: `?noun`, `?name` directly to template

**CONSTRUCT Example**:
```sparql
CONSTRUCT {
  ?verb nv:hasCommandName ?commandName .
} WHERE {
  ?verb a nv:Verb ; nv:name ?name .
  BIND(CONCAT("prefix_", ?name) AS ?commandName)
}
```
→ Returns: New RDF with `nv:hasCommandName` property that template can query

### When to Use CONSTRUCT

Use CONSTRUCT when you need to:
- ✅ Transform RDF structure (reshape data)
- ✅ Generate derived properties (computed values)
- ✅ Aggregate multiple triples (combine data)
- ✅ Normalize data formats (standardize)
- ✅ Merge multiple sources (composition)

Use SELECT when you need to:
- ✅ Simple data extraction (direct use)
- ✅ No transformation needed (as-is)
- ✅ Single triple pattern (direct mapping)

---

## Integration with Existing Patterns

### Works with Modular Templates

Project templates can be modular:
```
clap-noun-verb/templates/
├── verb/
│   ├── command.tmpl              # Command implementation
│   ├── test.tmpl                 # Tests
│   ├── doc.tmpl                  # Documentation
│   └── example.tmpl              # Examples
├── noun/
│   ├── group.tmpl                # Noun group
│   └── module.tmpl               # Module structure
└── manifest.yaml                 # Template manifest
```

### Works with RDF-Driven Generation

Project can have its own ontology:
```turtle
@prefix nv: <http://clap-noun-verb.org/schema#> .

:VerbCommand a nv:Command ;
    nv:hasPattern :CommandPattern ;
    nv:requires :BusinessLogic .

:CommandPattern a nv:Pattern ;
    nv:structure "CLI Layer → Business Logic" .
```

---

## Benefits Summary

| Aspect | Benefits |
|--------|----------|
| **Onboarding** | ✅ New contributors generate starter code |
| **Consistency** | ✅ All code follows project patterns |
| **Evolution** | ✅ Templates evolve with project |
| **Version Control** | ✅ Templates tracked with code |
| **Maintainability** | ✅ Templates maintained with project |
| **Reusability** | ✅ Share templates via marketplace |

---

## Next Steps

1. ✅ Document this pattern (this file)
2. ⏳ Implement template discovery in ggen
3. ⏳ Add project template manifest support
4. ⏳ Add `.ggen/config.yaml` support
5. ⏳ Update clap-noun-verb to use project templates
6. ⏳ Create template examples for clap-noun-verb
7. ⏳ Update documentation with project template examples

---

**Last Updated**: Project-embedded templates pattern documented. Ready for implementation.

