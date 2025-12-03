# ggen v2.0 Business Logic Separation & Frozen Sections

## Vision: CLI Layer Delegates to Business Logic

**Principle**: Templates generate CLI layer that delegates to business logic files. Business logic files are editable by coding agents. Frozen sections preserve human edits in generated code.

---

## Core Architecture

### Separation of Concerns

```
Generated Code (CLI Layer)
    ↓ delegates to
Business Logic File (Editable by Agent)
    ↓ uses
Domain Logic (Reusable)
```

### Template Pattern

**Template generates**:
- CLI layer (thin wrapper)
- Reference to business logic file
- Business logic file skeleton (if doesn't exist)

**Agent edits**:
- Business logic file only
- Never regenerated code

**Frozen sections**:
- Preserve human edits in generated code
- Mark sections as editable/human-maintained

---

## Template Structure

### Template: `templates/verb.tmpl`

```rust
// templates/verb.tmpl - Generates CLI layer with business logic reference
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.cliPath }} #}
/// {{ cmd.description }}

use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

// ✅ Reference to business logic file
use crate::domain::{{ cmd.nounName }}::{{ cmd.verbName }}::{
    {{ cmd.functionName }},
    {{ cmd.outputType }},
};

#[verb("{{ cmd.verbName }}", "{{ cmd.nounName }}")]
pub fn {{ cmd.commandName }}(
    name: String,
    format: Option<String>,
) -> Result<{{ cmd.outputType }}> {
    // ✅ Thin CLI layer - delegates to business logic
    Ok({{ cmd.functionName }}(name, format))
}

{# FILE: {{ cmd.businessLogicPath }} #}
/// Business logic for {{ cmd.description }}
///
/// ✅ This file is EDITABLE by coding agents
/// ✅ Never regenerated - only CLI layer is regenerated
/// ✅ Agent edits this file to implement business logic

use crate::domain::types::{{ cmd.outputType }};

/// Business logic implementation
pub fn {{ cmd.functionName }}(
    name: String,
    format: Option<String>,
) -> {{ cmd.outputType }} {
    // ✅ TODO: Implement business logic
    // ✅ Agent edits this function
    
    {{ cmd.outputType }} {
        name,
        format: format.unwrap_or_else(|| "json".to_string()),
        // ... more fields from RDF
    }
}

#[derive(Debug, serde::Serialize)]
pub struct {{ cmd.outputType }} {
    pub name: String,
    pub format: String,
    // ... more fields from RDF
}
{% endfor %}
```

**Generated Structure**:
```
src/
├── commands/              # Generated CLI layer
│   └── utils/
│       └── doctor.rs      # Thin wrapper, delegates to business logic
└── domain/                # Business logic (editable by agent)
    └── utils/
        └── doctor.rs      # Agent edits this file
```

### CLI Layer (Generated)

```rust
// src/commands/utils/doctor.rs - GENERATED (regenerated on template changes)
use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

// ✅ Reference to business logic file
use crate::domain::utils::doctor::{
    run_diagnostics,
    DoctorOutput,
};

#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    // ✅ Thin wrapper - delegates to business logic
    Ok(run_diagnostics())
}
```

### Business Logic File (Editable by Agent)

```rust
// src/domain/utils/doctor.rs - EDITABLE (never regenerated)
// ✅ Agent edits this file to implement business logic
// ✅ CLI layer delegates to this file

use crate::domain::types::DoctorOutput;

/// Business logic for diagnostics
pub fn run_diagnostics() -> DoctorOutput {
    // ✅ Agent implements business logic here
    // ✅ This file is never regenerated
    
    DoctorOutput {
        rust_ok: check_rust(),
        git_ok: check_git(),
        ollama_ok: check_ollama(),
        docker_ok: check_docker(),
        message: Some("All checks passed".to_string()),
    }
}

fn check_rust() -> bool {
    // Agent implements check logic
    true
}

fn check_git() -> bool {
    // Agent implements check logic
    true
}

fn check_ollama() -> bool {
    // Agent implements check logic
    false
}

fn check_docker() -> bool {
    // Agent implements check logic
    true
}
```

---

## Frozen Sections

### Problem: Preserving Edits in Generated Code

**Challenge**: Some parts of generated code need to be editable, but we still want to regenerate other parts.

**Solution**: Frozen sections are defined directly in templates, marking editable parts of generated code.

### Frozen Section Syntax in Templates

Templates can mark sections as frozen using special template syntax:

```rust
// templates/verb.tmpl - Template with frozen section
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.cliPath }} #}
use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

use crate::domain::{{ cmd.nounName }}::{{ cmd.verbName }}::{
    {{ cmd.functionName }},
    {{ cmd.outputType }},
};

#[verb("{{ cmd.verbName }}", "{{ cmd.nounName }}")]
pub fn {{ cmd.commandName }}() -> Result<{{ cmd.outputType }}> {
    // ✅ Generated code - can be regenerated
    
    {% frozen %}
    // 🔒 FROZEN START: Human-editable section
    // This section is preserved during regeneration
    // Agent can edit this section without losing changes
    
    let format = std::env::var("GGEN_FORMAT")
        .unwrap_or_else(|| "json".to_string());
    
    let result = {{ cmd.functionName }}();
    
    if format == "text" {
        println!("Rust: {}", result.rust_ok);
        println!("Git: {}", result.git_ok);
    }
    
    Ok(result)
    // 🔒 FROZEN END
    {% endfrozen %}
}
{% endfor %}
```

**Template Syntax**:
- `{% frozen %}` - Marks beginning of frozen section in template
- `{% endfrozen %}` - Marks end of frozen section in template

**Generated Code Markers**:
- `🔒 FROZEN START` - Marks beginning of frozen section in generated code
- `🔒 FROZEN END` - Marks end of frozen section in generated code

**Behavior**:
- Frozen sections are defined in templates, not via separate commands
- Frozen sections are preserved during regeneration
- Everything else can be regenerated
- Agent can edit frozen sections freely

### Frozen Section in Templates

Frozen sections are defined directly in templates using `{% frozen %}` tags:

```rust
// templates/verb.tmpl - Template with frozen section
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.cliPath }} #}
use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

use crate::domain::{{ cmd.nounName }}::{{ cmd.verbName }}::{
    {{ cmd.functionName }},
    {{ cmd.outputType }},
};

#[verb("{{ cmd.verbName }}", "{{ cmd.nounName }}")]
pub fn {{ cmd.commandName }}() -> Result<{{ cmd.outputType }}> {
    {% frozen %}
    // 🔒 FROZEN START: Human-editable section
    // This section is preserved during regeneration
    // Agent can edit this section
    
    // ✅ Default implementation (can be edited)
    Ok({{ cmd.functionName }}())
    
    // 🔒 FROZEN END
    {% endfrozen %}
}
{% endfor %}
```

**Key Points**:
- Frozen sections are defined in templates using `{% frozen %}` / `{% endfrozen %}` tags
- No separate freeze command needed - it's built into the template syntax
- During regeneration, ggen detects frozen sections and preserves them

### Frozen Section Detection

ggen detects frozen sections during regeneration by parsing the `🔒 FROZEN START` / `🔒 FROZEN END` markers in generated code:

```rust
// Before regeneration
#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    // 🔒 FROZEN START
    let custom_logic = do_custom_check();  // Human edit
    Ok(custom_logic)
    // 🔒 FROZEN END
}

// After regeneration - frozen section preserved
#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    // 🔒 FROZEN START
    let custom_logic = do_custom_check();  // ✅ Preserved!
    Ok(custom_logic)
    // 🔒 FROZEN END
}
```

**How It Works**:
1. Template uses `{% frozen %}` tags to mark sections
2. Generated code includes `🔒 FROZEN START` / `🔒 FROZEN END` markers
3. During regeneration, ggen detects these markers and preserves the content
4. Everything between markers is preserved, even if template changes

---

## Agent Workflow

### 1. Initial Generation

```bash
# Generate CLI layer + business logic skeleton
cd clap-noun-verb
ggen template generate --template verb.tmpl --rdf command.ttl

# Generates:
# - src/commands/utils/doctor.rs (CLI layer)
# - src/domain/utils/doctor.rs (business logic skeleton)
```

### 2. Agent Implements Business Logic

```rust
// src/domain/utils/doctor.rs - Agent edits this file
pub fn run_diagnostics() -> DoctorOutput {
    // ✅ Agent implements business logic
    DoctorOutput {
        rust_ok: check_rust(),
        git_ok: check_git(),
        // ... implementation
    }
}
```

### 3. Regeneration (Preserves Business Logic)

```bash
# Regenerate CLI layer (preserves business logic)
ggen template generate --template verb.tmpl --rdf command.ttl --regenerate

# ✅ CLI layer regenerated
# ✅ Business logic file preserved (never regenerated)
# ✅ Frozen sections in CLI layer preserved
```

---

## Project Structure

```
clap-noun-verb/
├── ggen.toml
├── domain/
│   └── commands.ttl
├── templates/
│   └── verb.tmpl
└── src/
    ├── commands/              # Generated CLI layer (regenerated)
    │   └── utils/
    │       └── doctor.rs       # Thin wrapper, delegates to business logic
    └── domain/                # Business logic (editable by agent)
        └── utils/
            └── doctor.rs       # ✅ Agent edits this file
```

---

## Frozen Section Patterns

### Pattern 1: Complete Function Frozen

```rust
#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    // 🔒 FROZEN START
    // Entire function is frozen - agent can edit freely
    let result = run_diagnostics();
    
    // Custom logic added by agent
    if should_format_as_text() {
        print_text_format(result);
    }
    
    Ok(result)
    // 🔒 FROZEN END
}
```

### Pattern 2: Partial Function Frozen

```rust
#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    // ✅ Generated code - regenerated
    let diagnostics = run_diagnostics();
    
    // 🔒 FROZEN START
    // Only this section is frozen
    let formatted = format_diagnostics(diagnostics);
    // 🔒 FROZEN END
    
    // ✅ Generated code - regenerated
    Ok(formatted)
}
```

### Pattern 3: Multiple Frozen Sections

```rust
#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    // 🔒 FROZEN START
    // First frozen section
    let format = get_format_preference();
    // 🔒 FROZEN END
    
    let diagnostics = run_diagnostics();
    
    // 🔒 FROZEN START
    // Second frozen section
    if format == "text" {
        print_text(diagnostics);
    }
    // 🔒 FROZEN END
    
    Ok(diagnostics)
}
```

---

## Template Configuration

### Template: `templates/verb.tmpl`

```rust
{% for cmd in query('command_structure') %}
{# FILE: {{ cmd.cliPath }} #}
use crate::error::Result;
use clap_noun_verb::{verb, VerbArgs};

// ✅ Reference to business logic file
use crate::domain::{{ cmd.nounName }}::{{ cmd.verbName }}::{
    {{ cmd.functionName }},
    {{ cmd.outputType }},
};

#[verb("{{ cmd.verbName }}", "{{ cmd.nounName }}")]
pub fn {{ cmd.commandName }}() -> Result<{{ cmd.outputType }}> {
    // 🔒 FROZEN START: Human-editable section
    // This section is preserved during regeneration
    // Agent can edit this section
    
    // ✅ Default: Delegate to business logic
    Ok({{ cmd.functionName }}())
    
    // 🔒 FROZEN END
}

{# FILE: {{ cmd.businessLogicPath }} #}
/// Business logic for {{ cmd.description }}
///
/// ✅ This file is EDITABLE by coding agents
/// ✅ Never regenerated - only CLI layer is regenerated
/// ✅ Agent edits this file to implement business logic

use crate::domain::types::{{ cmd.outputType }};

/// Business logic implementation
pub fn {{ cmd.functionName }}() -> {{ cmd.outputType }} {
    // ✅ TODO: Implement business logic
    // ✅ Agent edits this function
    
    {{ cmd.outputType }} {
        // ... fields from RDF
    }
}

#[derive(Debug, serde::Serialize)]
pub struct {{ cmd.outputType }} {
    // ... fields from RDF
}
{% endfor %}
```

---

## RDF Schema Extension

### Business Logic Reference in RDF

Paths can be defined in RDF or derived from filesystem routing conventions. See [GGEN_V2_FILESYSTEM_ROUTING.md](GGEN_V2_FILESYSTEM_ROUTING.md) for convention-based paths.

```turtle
# domain/commands.ttl
@prefix nv: <http://clap-noun-verb.org/schema#> .

:DoctorVerb a nv:Verb ;
    nv:name "doctor" ;
    nv:belongsTo :UtilsNoun ;
    nv:description "Check system prerequisites" ;
    nv:hasCLIPath "src/commands/utils/doctor.rs" ;
    nv:hasBusinessLogicPath "src/domain/utils/doctor.rs" ;
    nv:hasBusinessLogicFunction "run_diagnostics" ;
    nv:returns :DoctorOutput .
```

**CONSTRUCT Query** (paths from RDF):
```sparql
CONSTRUCT {
  ?verb nv:hasCLIPath ?cliPath ;
        nv:hasBusinessLogicPath ?businessLogicPath ;
        nv:hasBusinessLogicFunction ?businessLogicFunction .
} WHERE {
  ?verb a nv:Verb ;
        nv:hasCLIPath ?cliPath ;  # From RDF
        nv:hasBusinessLogicPath ?businessLogicPath ;  # From RDF
        nv:hasBusinessLogicFunction ?businessLogicFunction .  # From RDF
}
```

**Alternative**: Use filesystem routing conventions to derive paths automatically.

---

## Benefits

### 1. **Clear Separation**
- ✅ CLI layer: Generated, regenerated
- ✅ Business logic: Editable, never regenerated
- ✅ Clear boundaries

### 2. **Agent-Friendly**
- ✅ Agent edits business logic files only
- ✅ No risk of losing agent edits
- ✅ Clear where to make changes

### 3. **Frozen Sections**
- ✅ Preserve human edits in generated code
- ✅ Selective regeneration
- ✅ Flexible customization

### 4. **Maintainability**
- ✅ Business logic isolated
- ✅ CLI layer simple and regenerable
- ✅ Clear responsibility boundaries

---

## CLI Commands

### Generation

```bash
# Generate CLI layer + business logic skeleton
ggen template generate --template verb.tmpl --rdf command.ttl

# Regenerate CLI layer (preserves frozen sections and business logic)
ggen template generate --template verb.tmpl --rdf command.ttl --regenerate
```

**Frozen Sections**: Frozen sections are defined directly in templates using `{% frozen %}` tags. No separate command needed - ggen automatically detects and preserves frozen sections during regeneration.

**When to Use**:
- **Frozen Sections**: For small edits within generated CLI layer (e.g., formatting, conditional logic)
- **Business Logic Files**: For complete implementation logic that shouldn't be regenerated

---

## See Also

- **[GGEN_V2_TEMPLATE_ARCHITECTURE.md](GGEN_V2_TEMPLATE_ARCHITECTURE.md)** - Pure RDF-driven template architecture
- **[GGEN_V2_PROJECT_CONFIG.md](GGEN_V2_PROJECT_CONFIG.md)** - Project configuration with `ggen.toml`
- **[GGEN_V2_FILESYSTEM_ROUTING.md](GGEN_V2_FILESYSTEM_ROUTING.md)** - Filesystem-based routing conventions
- **[GGEN_V2_ARCHITECTURE_DIAGRAMS.puml](GGEN_V2_ARCHITECTURE_DIAGRAMS.puml)** - C4 architecture diagrams

---

## Conclusion

**Key Insight**: CLI layer delegates to business logic files. Agent edits business logic only. Frozen sections preserve edits in generated code.

**Result**: Clear separation, agent-friendly, flexible customization.

**Benefits**: Maintainability, clarity, edit preservation.

---

**Last Updated**: Business logic separation and frozen sections pattern documented.

