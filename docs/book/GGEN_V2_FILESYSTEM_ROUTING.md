# ggen v2.0 Filesystem Routing: Convention Over Configuration

## Vision: Filesystem as Configuration

**Principle**: Use filesystem structure for routing and discovery. Minimal hardcoding. Maximum convention.

---

## Core Insight

**Why hardcode paths when the filesystem already tells us where everything is?**

Instead of:
```toml
# ❌ Hardcoding paths
templates = [
    { template = "templates/verb.tmpl", output = "src/commands" },
]
```

Use filesystem structure:
```
templates/
  verb.tmpl
  noun.tmpl
  test.tmpl
```

Filesystem routing discovers templates automatically.

---

## Architecture Pattern

### Minimal Configuration: `ggen.toml`

Only essential configuration:

```toml
# ggen.toml - Minimal configuration
[project]
name = "clap-noun-verb"

[generation]
# RDF files (filesystem order determines merge order)
rdf = [
    "domain/base-ontology.ttl",
    "domain/commands.ttl",
]

# ✅ No template paths - discovered from filesystem
# ✅ No output paths - convention-based routing

# Optional: Override defaults
[generation]
output_dir = "generated"  # Default convention
```

**Everything else inferred from filesystem structure.**

### Filesystem Conventions

```
clap-noun-verb/
├── ggen.toml              # Minimal config
├── domain/                 # RDF files (auto-discovered)
│   ├── base-ontology.ttl
│   ├── commands.ttl
│   └── types.ttl
├── templates/              # Templates (auto-discovered)
│   ├── verb.tmpl          # Generates: src/commands/{noun}/{verb}.rs
│   ├── noun.tmpl          # Generates: src/commands/{noun}/mod.rs
│   ├── test.tmpl          # Generates: tests/commands/{noun}/{verb}_test.rs
│   └── doc.tmpl           # Generates: docs/commands/{noun}/{verb}.md
└── queries/                # Shared SPARQL queries (optional)
    ├── command_structure.sparql
    └── verb_list.sparql
```

### Automatic Discovery

**Templates**: Discovered from `templates/` directory
```bash
# ggen discovers:
templates/verb.tmpl      → Generate from verb template
templates/noun.tmpl      → Generate from noun template
templates/test.tmpl      → Generate from test template
```

**RDF Files**: Discovered from `domain/` directory
```bash
# ggen discovers:
domain/base-ontology.ttl → Base ontology
domain/commands.ttl      → Command definitions
domain/types.ttl         → Type definitions
```

**Queries**: Discovered from `queries/` directory (optional)
```bash
# ggen discovers:
queries/command_structure.sparql → Named query: command_structure
queries/verb_list.sparql        → Named query: verb_list
```

### Output Path Convention

Templates use **convention-based output paths**:

```
Template Name          →  Output Convention
templates/verb.tmpl   →  src/commands/{noun}/{verb}.rs
templates/noun.tmpl   →  src/commands/{noun}/mod.rs
templates/test.tmpl   →  tests/commands/{noun}/{verb}_test.rs
templates/doc.tmpl    →  docs/commands/{noun}/{verb}.md
```

Or templates can infer from RDF:
```rust
// templates/verb.tmpl
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.outputPath }} #}  // From RDF CONSTRUCT query
// ... template code ...
{% endfor %}
```

---

## Filesystem Routing Rules

### Rule 1: Template Discovery

**Location**: `templates/` directory

**Pattern**: `templates/{name}.tmpl`

**Discovery**: All `.tmpl` files in `templates/` are discovered automatically

```bash
# Filesystem:
templates/
  verb.tmpl
  noun.tmpl
  test.tmpl

# ggen discovers all three templates
```

### Rule 2: RDF Discovery

**Location**: `domain/` directory

**Pattern**: `domain/{name}.ttl` or `domain/{name}.rdf`

**Discovery**: All `.ttl` and `.rdf` files discovered, ordered by filename

```bash
# Filesystem:
domain/
  01-base-ontology.ttl
  02-commands.ttl
  03-types.ttl

# ggen discovers and merges in order
```

### Rule 3: Query Discovery

**Location**: `queries/` directory (optional)

**Pattern**: `queries/{name}.sparql`

**Discovery**: All `.sparql` files become named queries

```bash
# Filesystem:
queries/
  command_structure.sparql
  verb_list.sparql

# ggen loads:
# query('command_structure') → queries/command_structure.sparql
# query('verb_list')         → queries/verb_list.sparql
```

### Rule 4: Output Path Convention

**Pattern**: Template name → Output convention

```
templates/verb.tmpl     →  src/commands/{noun}/{verb}.rs
templates/noun.tmpl     →  src/commands/{noun}/mod.rs
templates/test.tmpl     →  tests/commands/{noun}/{verb}_test.rs
templates/doc.tmpl      →  docs/commands/{noun}/{verb}.md
templates/route.tmpl    →  src/routes/{noun}/{verb}.rs
templates/schema.tmpl   →  schemas/{noun}/{verb}.json
```

Or inferred from RDF:
```sparql
# CONSTRUCT query generates outputPath
CONSTRUCT {
  ?verb nv:hasOutputPath ?outputPath .
} WHERE {
  ?verb a nv:Verb .
  BIND(CONCAT("src/commands/", ?nounName, "/", ?verbName, ".rs") AS ?outputPath)
}
```

---

## Minimal ggen.toml

### Minimal Configuration

```toml
# ggen.toml - Minimal configuration
[project]
name = "clap-noun-verb"

# ✅ Everything else discovered from filesystem
```

**That's it!** All templates, RDF files, and queries discovered automatically.

### Optional Overrides

```toml
# ggen.toml - Optional overrides
[project]
name = "clap-noun-verb"

[generation]
# Override default RDF directory
rdf_dir = "ontologies"  # Default: domain

# Override default template directory
template_dir = "templates"  # Default: templates

# Override default query directory
query_dir = "sparql"  # Default: queries

# Override default output directory
output_dir = "generated"  # Default: src (convention-based)

# Specify RDF merge order (if non-alphabetical)
rdf_order = [
    "domain/base-ontology.ttl",
    "domain/commands.ttl",
]
```

---

## Pure Templates (No Path Hardcoding)

### Template: `templates/verb.tmpl`

```rust
// templates/verb.tmpl - Pure rendering, no path hardcoding
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.outputPath }} #}  // From RDF CONSTRUCT query

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

**No frontmatter, no hardcoded paths, no configuration** - pure rendering logic.

### Query: `queries/command_structure.sparql`

```sparql
# queries/command_structure.sparql
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
  
  # ✅ Output path generated from convention
  BIND(CONCAT("src/commands/", ?nounName, "/", ?verbName, ".rs") AS ?outputPath)
}
```

**Query file separate from template** - discovered from `queries/` directory.

---

## Workflow

### 1. Project Structure

```bash
cd clap-noun-verb
mkdir -p domain templates queries
touch ggen.toml
```

### 2. Define RDF

```bash
# domain/commands.ttl
cat > domain/commands.ttl <<EOF
@prefix nv: <http://clap-noun-verb.org/schema#> .
:UtilsNoun a nv:Noun ; nv:name "utils" .
:DoctorVerb a nv:Verb ; nv:name "doctor" ; nv:belongsTo :UtilsNoun .
EOF
```

### 3. Create Pure Template

```bash
# templates/verb.tmpl
cat > templates/verb.tmpl <<EOF
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.outputPath }} #}
// Generated code
{% endfor %}
EOF
```

### 4. Create Query

```bash
# queries/command_structure.sparql
cat > queries/command_structure.sparql <<EOF
PREFIX nv: <http://clap-noun-verb.org/schema#>
CONSTRUCT {
  ?verb nv:hasOutputPath ?outputPath .
} WHERE {
  ?verb a nv:Verb ; nv:name ?verbName ; nv:belongsTo ?noun .
  ?noun nv:name ?nounName .
  BIND(CONCAT("src/commands/", ?nounName, "/", ?verbName, ".rs") AS ?outputPath)
}
EOF
```

### 5. Generate

```bash
# ✅ Everything discovered automatically
ggen generate

# Discovered:
# - templates/verb.tmpl
# - domain/commands.ttl
# - queries/command_structure.sparql
# → Generates: src/commands/utils/doctor.rs
```

---

## Benefits

### 1. **Zero Hardcoding**
- ✅ No template paths in config
- ✅ No output paths in config
- ✅ Everything discovered from filesystem

### 2. **Convention Over Configuration**
- ✅ Standard directory structure
- ✅ Predictable output locations
- ✅ Easy to understand

### 3. **Automatic Discovery**
- ✅ Templates auto-discovered
- ✅ RDF files auto-discovered
- ✅ Queries auto-discovered

### 4. **Minimal Configuration**
- ✅ `ggen.toml` can be very minimal
- ✅ Or even optional (use defaults)
- ✅ Maximum convention, minimum config

### 5. **Filesystem as Source of Truth**
- ✅ Structure is configuration
- ✅ Easy to version control
- ✅ Clear project organization

---

## Advanced Patterns

### Pattern 1: Nested Templates

```
templates/
  commands/
    verb.tmpl
    noun.tmpl
  tests/
    verb_test.tmpl
    noun_test.tmpl
```

**Convention**: `templates/{category}/{name}.tmpl`

### Pattern 2: RDF Organization

```
domain/
  base/
    ontology.ttl
  commands/
    utils.ttl
    ai.ttl
  types/
    output.ttl
```

**Convention**: Merge all `.ttl` files recursively

### Pattern 3: Query Organization

```
queries/
  commands/
    structure.sparql
    list.sparql
  types/
    definition.sparql
```

**Convention**: `queries/{category}/{name}.sparql` → `query('{category}_{name}')`

---

## CLI Commands (v2.0)

### Generation

```bash
# Generate (discovers everything from filesystem)
ggen generate

# Generate specific template (discovered from filesystem)
ggen generate --template verb

# Generate with different RDF (still uses filesystem discovery)
ggen generate --rdf domain/new-commands.ttl
```

### Discovery

```bash
# List discovered templates
ggen template list

# List discovered RDF files
ggen rdf list

# List discovered queries
ggen query list
```

---

## Comparison: Hardcoded vs Filesystem Routing

| Aspect | Hardcoded (v1.x) | Filesystem Routing (v2.0) |
|--------|------------------|---------------------------|
| **Template Paths** | ❌ Hardcoded in config | ✅ Discovered from filesystem |
| **Output Paths** | ❌ Hardcoded in config | ✅ Convention-based |
| **RDF Paths** | ❌ Hardcoded in config | ✅ Discovered from filesystem |
| **Query Paths** | ❌ Inline in templates | ✅ Separate files, discovered |
| **Configuration** | ❌ Lots of paths | ✅ Minimal (project name) |
| **Maintainability** | ❌ Update config for paths | ✅ Add files, auto-discovered |
| **Clarity** | ❌ Config scattered | ✅ Filesystem is config |

---

## Migration Path

### v1.x → v2.0

1. **Move templates to filesystem**
   ```bash
   # Old: config specifies paths
   templates = ["templates/verb.tmpl"]
   
   # New: filesystem structure
   templates/verb.tmpl
   ```

2. **Move queries to separate files**
   ```bash
   # Old: queries inline in template frontmatter
   
   # New: separate query files
   queries/command_structure.sparql
   ```

3. **Use convention-based output paths**
   ```bash
   # Old: hardcoded output paths
   { template = "verb.tmpl", output = "src/commands" }
   
   # New: convention-based
   templates/verb.tmpl → src/commands/{noun}/{verb}.rs
   ```

---

## Conclusion

**Key Insight**: **Filesystem structure IS the configuration.**

**Result**: Zero hardcoding, maximum convention, automatic discovery.

**Benefits**: Minimal config, clear organization, easy maintenance.

---

**Last Updated**: ggen v2.0 filesystem routing pattern documented.

---

## See Also

- **[GGEN_V2_TEMPLATE_ARCHITECTURE.md](GGEN_V2_TEMPLATE_ARCHITECTURE.md)** - Pure RDF-driven template architecture
- **[GGEN_V2_BUSINESS_LOGIC_SEPARATION.md](GGEN_V2_BUSINESS_LOGIC_SEPARATION.md)** - Business logic separation & frozen sections
- **[GGEN_V2_PROJECT_CONFIG.md](GGEN_V2_PROJECT_CONFIG.md)** - Project configuration with `ggen.toml`
- **[GGEN_V2_ARCHITECTURE_DIAGRAMS.puml](GGEN_V2_ARCHITECTURE_DIAGRAMS.puml)** - C4 architecture diagrams

