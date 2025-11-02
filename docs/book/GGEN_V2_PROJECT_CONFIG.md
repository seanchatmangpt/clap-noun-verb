# ggen v2.0 Project Configuration: ggen.toml

## Vision: Project-Based Generation

**Principle**: Generation is project-based, not template-based. All configuration in `ggen.toml`. Templates are pure.

---

## Core Insight

**If we think about it again, do we need frontmatter?**

❌ **No!** Frontmatter mixes concerns:
- Template metadata (what template does)
- Project configuration (where to output)
- RDF references (what data to use)
- Variables (project-specific data)

✅ **Better**: All configuration in `ggen.toml` at project root.

---

## Architecture Pattern

### Pure Templates (No Frontmatter)

Templates contain **only** rendering logic:

```rust
// templates/verb.tmpl - Pure rendering logic
// ✅ No frontmatter - just SPARQL queries and template syntax

{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.output_path }} #}
/// {{ cmd.description }}

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

**No frontmatter needed** - all configuration in `ggen.toml`.

### Project Configuration: `ggen.toml`

All project configuration in one place:

```toml
# ggen.toml - Project configuration
[project]
name = "clap-noun-verb"
version = "3.0.0"

[generation]
# RDF files to load (in order - later files override earlier)
rdf = [
    "domain/base-ontology.ttl",
    "domain/commands.ttl",
    "domain/types.ttl",
]

# Templates to use
templates = [
    { template = "templates/verb.tmpl", output = "src/commands" },
    { template = "templates/test.tmpl", output = "tests/commands" },
    { template = "templates/doc.tmpl", output = "docs/commands" },
]

# Default output directory (can be overridden per template)
output_dir = "generated"

# SPARQL queries (project-level - can be shared across templates)
[queries]
command_structure = """
PREFIX nv: <http://clap-noun-verb.org/schema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
CONSTRUCT {
  ?verb nv:hasCommandName ?commandName ;
        nv:hasFunctionName ?functionName ;
        nv:hasOutputType ?outputType ;
        nv:hasOutputPath ?outputPath .
}
WHERE {
  ?verb a nv:Verb ;
        nv:name ?verbName ;
        nv:belongsTo ?noun .
  ?noun nv:name ?nounName .
  
  BIND(CONCAT(?nounName, "_", ?verbName) AS ?commandName)
  BIND(CONCAT("create_", ?nounName, "_", ?verbName) AS ?functionName)
  BIND(CONCAT(UCASE(SUBSTR(?nounName, 1, 1)), SUBSTR(?nounName, 2),
              UCASE(SUBSTR(?verbName, 1, 1)), SUBSTR(?verbName, 2), 
              "Output") AS ?outputType)
  BIND(CONCAT("src/commands/", ?nounName, "/", ?verbName, ".rs") AS ?outputPath)
}
"""

# Project-specific variables (for template use)
[vars]
project_name = "clap-noun-verb"
crate_name = "clap_noun_verb"
version = "3.0.0"
```

---

## Workflow

### 1. Project Setup

```bash
# Initialize project with ggen.toml
cd clap-noun-verb
ggen project init

# Creates:
# - ggen.toml (project configuration)
# - domain/ (RDF files)
# - templates/ (pure templates)
```

### 2. Define Domain in RDF

```turtle
# domain/commands.ttl
@prefix nv: <http://clap-noun-verb.org/schema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:UtilsNoun a nv:Noun ;
    nv:name "utils" ;
    rdfs:label "utils" .

:DoctorVerb a nv:Verb ;
    nv:name "doctor" ;
    nv:belongsTo :UtilsNoun ;
    nv:description "Check system prerequisites" ;
    nv:returns :DoctorOutput .
```

### 3. Create Pure Templates

```rust
// templates/verb.tmpl - Pure rendering logic
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.outputPath }} #}
/// {{ cmd.description }}

use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

#[verb("{{ cmd.verbName }}", "{{ cmd.nounName }}")]
pub fn {{ cmd.commandName }}() -> Result<{{ cmd.outputType }}> {
    Ok({{ cmd.functionName }}())
}

pub fn {{ cmd.functionName }}() -> {{ cmd.outputType }} {
    {{ cmd.outputType }} {}
}

#[derive(Debug, serde::Serialize)]
pub struct {{ cmd.outputType }} {}
{% endfor %}
```

### 4. Generate from Project Config

```bash
# Generate from ggen.toml
cd clap-noun-verb
ggen generate

# Or generate specific template
ggen generate --template verb

# Or generate with different RDF
ggen generate --rdf domain/commands-new.ttl
```

---

## Benefits

### 1. **Separation of Concerns**
- **Template** = Pure rendering logic (no configuration)
- **ggen.toml** = Project configuration (all settings)
- **RDF** = Domain model (what to generate)

### 2. **Project-Based Generation**
- Generation happens at project level
- All configuration in one place
- Easy to version control
- Clear project boundaries

### 3. **Change-Based Generation**
- Generate based on RDF changes
- Watch mode: regenerate on RDF changes
- Delta generation: only generate changed files

### 4. **Template Reusability**
- Templates are completely pure
- No project-specific data
- Reusable across projects
- Easy to share via marketplace

### 5. **Configuration Management**
- All config in `ggen.toml`
- Version controlled with project
- Easy to review changes
- Clear project structure

---

## ggen.toml Structure

### Minimal Configuration

```toml
# Minimal ggen.toml
[generation]
rdf = ["domain/commands.ttl"]
templates = ["templates/verb.tmpl"]
output_dir = "src/commands"
```

### Full Configuration

```toml
# Full ggen.toml
[project]
name = "clap-noun-verb"
version = "3.0.0"
description = "Framework for composable CLI patterns"

[generation]
# RDF files (merged in order)
rdf = [
    "domain/base-ontology.ttl",  # Base patterns
    "domain/commands.ttl",        # Command definitions
    "domain/types.ttl",          # Type definitions
    "domain/relationships.ttl",  # Relationships
]

# Templates and their output paths
templates = [
    { template = "templates/verb.tmpl", output = "src/commands" },
    { template = "templates/noun.tmpl", output = "src/commands" },
    { template = "templates/test.tmpl", output = "tests/commands" },
    { template = "templates/doc.tmpl", output = "docs/commands" },
]

# Default output directory
output_dir = "generated"

# SPARQL queries (shared across templates)
[queries]
command_structure = """
PREFIX nv: <http://clap-noun-verb.org/schema#>
CONSTRUCT {
  ?verb nv:hasCommandName ?commandName ;
        nv:hasOutputPath ?outputPath .
}
WHERE {
  ?verb a nv:Verb ;
        nv:name ?verbName ;
        nv:belongsTo ?noun .
  ?noun nv:name ?nounName .
  
  BIND(CONCAT(?nounName, "_", ?verbName) AS ?commandName)
  BIND(CONCAT("src/commands/", ?nounName, "/", ?verbName, ".rs") AS ?outputPath)
}
"""

verb_list = """
PREFIX nv: <http://clap-noun-verb.org/schema#>
SELECT ?verb ?noun ?name
WHERE {
  ?verb a nv:Verb ;
        nv:name ?name ;
        nv:belongsTo ?noun .
}
"""

# Project variables (available in templates)
[vars]
project_name = "clap-noun-verb"
crate_name = "clap_noun_verb"
author = "ggen team"
license = "MIT"
version = "3.0.0"

# Generation settings
[settings]
watch = false              # Watch RDF files for changes
delta = true               # Only generate changed files
validate = true            # Validate generated code
format = true              # Format generated code
```

---

## Advanced Patterns

### Pattern 1: Change-Based Generation

```toml
# ggen.toml
[generation]
# Generate only from changes
delta = true
watch = true

# RDF files to watch
rdf = ["domain/commands.ttl"]

# Templates to regenerate on changes
templates = ["templates/verb.tmpl"]
```

Usage:
```bash
# Watch mode - regenerate on RDF changes
ggen generate --watch

# Delta mode - only generate changed files
ggen generate --delta
```

### Pattern 2: Multi-Stage Generation

```toml
# ggen.toml
[generation.stages]
stage1 = { rdf = ["domain/base.ttl"], templates = ["templates/base.tmpl"] }
stage2 = { rdf = ["domain/commands.ttl"], templates = ["templates/commands.tmpl"] }
stage3 = { rdf = ["domain/tests.ttl"], templates = ["templates/tests.tmpl"] }
```

Usage:
```bash
# Generate stage 1
ggen generate --stage stage1

# Generate all stages
ggen generate --all-stages
```

### Pattern 3: Template Profiles

```toml
# ggen.toml
[profiles.dev]
templates = [
    { template = "templates/verb.tmpl", output = "src/commands" },
    { template = "templates/test.tmpl", output = "tests/commands" },
]

[profiles.prod]
templates = [
    { template = "templates/verb.tmpl", output = "src/commands" },
    { template = "templates/optimized.tmpl", output = "src/commands" },
]
```

Usage:
```bash
# Generate with dev profile
ggen generate --profile dev

# Generate with prod profile
ggen generate --profile prod
```

---

## Comparison: Frontmatter vs ggen.toml

| Aspect | Frontmatter (v1.x) | ggen.toml (v2.0) |
|--------|-------------------|------------------|
| **Location** | ❌ In each template | ✅ Project root |
| **Template Purity** | ❌ Mixed concerns | ✅ Pure rendering |
| **Configuration** | ❌ Scattered | ✅ Centralized |
| **Reusability** | ❌ Project-specific | ✅ Pure, reusable |
| **Version Control** | ❌ Mixed with templates | ✅ Separate, clear |
| **Change Management** | ❌ Hard to track | ✅ Easy to review |
| **Project Context** | ❌ Template-level | ✅ Project-level |

---

## Migration Path

### v1.x → v2.0

1. **Extract frontmatter to ggen.toml**
   ```toml
   # ggen.toml
   [generation]
   rdf = ["domain/commands.ttl"]  # From template frontmatter
   templates = ["templates/verb.tmpl"]
   ```

2. **Remove frontmatter from templates**
   ```rust
   // templates/verb.tmpl - Pure rendering logic only
   {% for cmd in query('command_structure') %}
   // ... template code ...
   {% endfor %}
   ```

3. **Move SPARQL queries to ggen.toml**
   ```toml
   # ggen.toml
   [queries]
   command_structure = """
   PREFIX nv: <http://clap-noun-verb.org/schema#>
   CONSTRUCT { ... }
   WHERE { ... }
   """
   ```

---

## CLI Commands (v2.0)

### Project Commands

```bash
# Initialize project
ggen project init

# Generate from ggen.toml
ggen template generate

# Generate specific template
ggen template generate --template verb.tmpl

# Generate with different RDF
ggen template generate --template verb.tmpl --rdf domain/new-commands.ttl

# Watch mode (regenerate on changes)
ggen template generate --template verb.tmpl --watch

# Delta mode (only changed files)
ggen template generate --template verb.tmpl --delta
```

### Template Commands

```bash
# Validate templates
ggen template validate

# List templates
ggen template list

# Show template (pure rendering logic)
ggen template show verb
```

---

## Project Structure

```
clap-noun-verb/
├── ggen.toml                 # ✅ Project configuration
├── domain/                    # RDF domain models
│   ├── base-ontology.ttl
│   ├── commands.ttl
│   ├── types.ttl
│   └── relationships.ttl
├── templates/                 # Pure templates (no frontmatter)
│   ├── verb.tmpl
│   ├── noun.tmpl
│   ├── test.tmpl
│   └── doc.tmpl
├── src/                       # Generated code
│   └── commands/
└── tests/                     # Generated tests
    └── commands/
```

---

## Benefits Summary

### 1. **Pure Templates**
- ✅ No frontmatter
- ✅ No configuration
- ✅ Pure rendering logic
- ✅ Completely reusable

### 2. **Project-Based**
- ✅ All configuration in `ggen.toml`
- ✅ Generation at project level
- ✅ Clear project boundaries
- ✅ Easy to version control

### 3. **Change-Based**
- ✅ Watch mode (regenerate on changes)
- ✅ Delta generation (only changed files)
- ✅ Clear change tracking
- ✅ Efficient regeneration

### 4. **Separation of Concerns**
- ✅ Templates = Logic (how to generate)
- ✅ ggen.toml = Configuration (what to generate)
- ✅ RDF = Data (domain model)
- ✅ Clear boundaries

---

## Conclusion

**Key Insight**: Generation should be **project-based**, not template-based.

**Result**: Pure templates, centralized configuration, clear separation of concerns.

**Benefits**: Maximum reusability, clarity, and maintainability.

---

**Last Updated**: ggen v2.0 project-based configuration with ggen.toml documented.

---

## See Also

- **[GGEN_V2_TEMPLATE_ARCHITECTURE.md](GGEN_V2_TEMPLATE_ARCHITECTURE.md)** - Pure RDF-driven template architecture
- **[GGEN_V2_BUSINESS_LOGIC_SEPARATION.md](GGEN_V2_BUSINESS_LOGIC_SEPARATION.md)** - Business logic separation & frozen sections
- **[GGEN_V2_FILESYSTEM_ROUTING.md](GGEN_V2_FILESYSTEM_ROUTING.md)** - Filesystem-based routing conventions
- **[GGEN_V2_ARCHITECTURE_DIAGRAMS.puml](GGEN_V2_ARCHITECTURE_DIAGRAMS.puml)** - C4 architecture diagrams

