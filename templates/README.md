# clap-noun-verb Code Generation Templates

Professional Jinja2 templates for scaffolding clap-noun-verb CLI projects with clean architecture, comprehensive documentation, and production-ready patterns.

## Overview

This directory contains four core templates and comprehensive documentation for generating clap-noun-verb CLIs:

| File | Purpose | Output |
|---|---|---|
| **`noun.rs.jinja`** | Noun command handler skeleton | `src/commands/{noun_name}.rs` |
| **`verb.rs.jinja`** | Verb handler with three-layer architecture | `src/commands/{noun}/{verb}.rs` |
| **`lib.rs.jinja`** | Main library crate entry point | `src/lib.rs` |
| **`main.rs.jinja`** | Binary entry point with error handling | `src/main.rs` |

## Documentation

### For Getting Started
- **[QUICK_START.md](QUICK_START.md)** - Step-by-step guide to build a task management CLI using templates

### For Reference
- **[TEMPLATES.md](TEMPLATES.md)** - Comprehensive template documentation, placeholder syntax, and best practices

---

## Quick Reference

### Template Files Summary

#### 1. **noun.rs.jinja** (125 lines)
Scaffolds noun command handlers with optional state management.

**Key Placeholders:**
- `{{ noun_name }}` - Kebab-case noun (e.g., `users`)
- `{{ noun_struct_name }}` - PascalCase struct (e.g., `UsersCommand`)
- `{{ brief_description }}` - One-line description

**Includes:**
- Default trait implementation
- Prerequisite validation method
- Test stubs (AAA pattern)

**Example Output:**
```rust
pub struct UsersCommand { ... }
impl Default for UsersCommand { ... }
impl UsersCommand {
    pub fn new() -> Self { ... }
    fn validate_prerequisites(&self) -> Result<()> { ... }
}
```

---

#### 2. **verb.rs.jinja** (279 lines)
Comprehensive verb handler with three-layer architecture.

**Architecture:**
1. **Domain Layer** - Pure business logic (`{{ verb_domain_struct }}`)
2. **Output Layer** - JSON serialization (`{{ verb_output_struct }}`)
3. **CLI Layer** - Validation and binding (`#[verb]` handler)

**Key Placeholders:**
- `{{ verb_name }}` - Kebab-case verb (e.g., `create`)
- `{{ verb_handler_name }}` - Function name (e.g., `create_user`)
- `{{ brief_description }}` - Purpose description

**Includes:**
- Full domain logic skeleton
- Output serialization struct
- Validation wrapper
- Dry-run support
- Exit code documentation
- AAA-pattern tests

**Example Output:**
```rust
// Domain Layer (pure logic, no I/O)
pub struct CreateUserDomain { ... }
impl CreateUserDomain {
    pub fn new(name: String) -> Self { ... }
    pub fn execute(&self) -> Result<CreateUserOutput> { ... }
}

// Output Layer (JSON-ready)
#[derive(Serialize)]
pub struct CreateUserOutput { ... }

// CLI Layer (validation + dispatch)
#[verb("users", "create")]
pub fn create_user(name: Option<String>) -> Result<CreateUserOutput> { ... }
```

---

#### 3. **lib.rs.jinja** (158 lines)
Main library crate structure with module organization.

**Module Layout:**
```rust
pub mod domain;      // Pure business logic
pub mod integration; // Glue code & I/O
pub mod commands;    // CLI layer & #[verb] handlers
pub mod outputs;     // JSON serialization types

pub fn run() -> Result<()> { ... }  // Auto-discover commands
```

**Key Placeholders:**
- `{{ crate_name }}` - Crate display name
- `{{ crate_description }}` - Purpose description

**Includes:**
- Architecture diagram (ASCII)
- Module documentation
- Public API re-exports
- Entry point function
- Initialization hooks
- Integration tests

---

#### 4. **main.rs.jinja** (280 lines)
Binary entry point with structured error handling.

**Exit Codes:**
| Code | Error Type | User Action |
|---|---|---|
| 0 | Success | — |
| 1 | ValidationError | Fix input |
| 2 | ExecutionError | Check logs |
| 3 | ConfigurationError | Fix setup |

**Key Placeholders:**
- `{{ crate_lib }}` - Library crate module path
- `{{ example_noun }}` / `{{ example_verb }}` - Usage examples
- `{{ crate_description }}` - Purpose

**Includes:**
- Error formatting function
- Exit code mapping
- Initialization hooks
- User-friendly error messages
- Logger integration stubs
- Config loader stubs

---

## Placeholder Syntax Reference

### Universal Placeholders

```jinja2
{{ variable_name }}                      # Direct substitution
{{ variable_name|default("fallback") }}  # With fallback
{{ variable_name|replace("a", "b") }}    # With filter

// PLACEHOLDER: Comment style
// Indicates where code generation can occur
```

### Naming Conventions

| Format | Usage | Example |
|---|---|---|
| **kebab-case** | CLI identifiers | `user-management`, `create-task` |
| **PascalCase** | Rust types | `UserManagement`, `CreateTask` |
| **snake_case** | Functions | `create_user`, `list_projects` |
| **UPPERCASE** | Constants | `DEFAULT_TIMEOUT`, `MAX_RETRIES` |

---

## Architecture Pattern

All templates enforce this three-layer architecture:

```
┌────────────────────────────────────────┐
│          CLI LAYER                     │
│  #[verb] handlers, validation, binding │
│  (thin wrapper, arg parsing)           │
└────────────────┬───────────────────────┘
                 │
┌────────────────▼───────────────────────┐
│      INTEGRATION LAYER                 │
│  Tera, file I/O, external services     │
│  (side effects, dependencies)          │
└────────────────┬───────────────────────┘
                 │
┌────────────────▼───────────────────────┐
│       DOMAIN LAYER                     │
│  Pure business logic, testable         │
│  (no I/O, no CLI, no dependencies)     │
└────────────────────────────────────────┘
```

**Benefits:**
- ✓ Pure domain logic is fully testable
- ✓ Easy to reuse business logic
- ✓ CLI validation is explicit
- ✓ Integration layer is isolated
- ✓ Changes in one layer don't affect others

---

## Example Usage

### Scenario: Create a User Management CLI

**Step 1: Use `lib.rs.jinja`**
```toml
[variables]
crate_name = "user-cli"
crate_description = "Manage users and permissions"
```

Generates `src/lib.rs` with module structure.

**Step 2: Use `noun.rs.jinja`**
```toml
[variables]
noun_name = "users"
noun_struct_name = "UsersCommand"
brief_description = "Manage user accounts and permissions"
```

Generates `src/commands/users.rs` noun handler.

**Step 3: Use `verb.rs.jinja`** (multiple times)
```toml
[variables]
noun_name = "users"
verb_name = "create"
verb_handler_name = "create_user"
verb_domain_struct = "CreateUserDomain"
verb_output_struct = "CreateUserOutput"
brief_description = "Create a new user account"
```

Generates `src/commands/users/create.rs` with full domain logic.

**Step 4: Use `main.rs.jinja`**
```toml
[variables]
crate_lib = "user_cli"
example_noun = "users"
example_verb = "create"
```

Generates `src/main.rs` with error handling.

---

## Testing Pattern (Included)

All templates include **AAA (Arrange-Act-Assert)** tests:

```rust
#[test]
fn test_create_user_with_valid_input() {
    // ARRANGE: Set up test data
    let domain = CreateUserDomain::new("alice".to_string());

    // ACT: Execute the operation
    let result = domain.execute();

    // ASSERT: Verify behavior
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.name, "alice");
}
```

**Test Coverage:**
- Valid input paths
- Invalid input handling
- Error cases
- Edge cases (empty strings, None values)
- Integration scenarios

---

## Real-World Examples

The templates are used in the **playground** example:

```bash
# Real implementation using template patterns
/Users/sac/clap-noun-verb/examples/playground/

# Structure:
src/
├── main.rs           ← main.rs.jinja pattern
├── lib.rs            ← lib.rs.jinja pattern
├── commands/
│   ├── papers.rs     ← verb.rs.jinja pattern
│   ├── thesis.rs     ← verb.rs.jinja pattern
│   └── config.rs     ← verb.rs.jinja pattern
├── domain/           ← Pure business logic
├── integration/      ← Tera, RDF, file I/O
└── outputs/          ← JSON serialization
```

**Run it:**
```bash
cd examples/playground
cargo run -- --help
```

---

## Integration with IDEs

### VS Code Snippet

Add to `.vscode/my-snippets.code-snippets`:

```json
{
  "clap-noun-verb: New Verb": {
    "prefix": "cnv-verb",
    "body": [
      "#[verb(\"${1:noun}\", \"${2:verb}\")]",
      "pub fn ${3:handler_name}(",
      "    ${4:param}: ${5:Type},",
      ") -> Result<${6:Output}> {",
      "    ${0:unimplemented!()}",
      "}"
    ],
    "description": "New clap-noun-verb handler"
  }
}
```

### IntelliJ Live Template

Create `User.xml`:

```xml
<template name="cnv-verb" value="#[verb(&quot;$NOUN$&quot;, &quot;$VERB$&quot;)]&#10;pub fn $FUNC$($PARAM$: $TYPE$) -> Result&lt;$OUTPUT$&gt; {&#10;    $END$&#10;}" ...>
  <variable name="NOUN" expression="" />
  <variable name="VERB" expression="" />
  <variable name="FUNC" expression="camelCase($VERB)" />
  <!-- ... -->
</template>
```

---

## Development Tips

### 1. Customize for Your Domain

Edit placeholders to match your terminology:

```jinja2
// Standard:
{{ noun_name }} → users
{{ verb_name }} → create

// Your project:
{{ noun_name }} → accounts
{{ verb_name }} → onboard
```

### 2. Add Domain-Specific Validation

Extend `verb.rs.jinja` validation section:

```rust
// CLI LAYER - Add custom validation
let email = email.ok_or_else(|| {
    NounVerbError::validation_error(
        "email".to_string(),
        "<missing>".to_string(),
        Some("Valid email format: user@example.com"),
    )
})?;

// Validate email format
if !email.contains('@') {
    return Err(NounVerbError::validation_error(
        "email".to_string(),
        email,
        Some("Email must contain @ symbol"),
    ));
}
```

### 3. Add Pre/Post Hooks

Extend `lib.rs.jinja` initialization:

```rust
pub fn run_with_config(config: MyConfig) -> Result<()> {
    // Pre-dispatch setup
    setup_logging(&config)?;
    setup_database(&config)?;

    // Run auto-discovered commands
    clap_noun_verb::run()?;

    // Post-dispatch cleanup
    shutdown_database()?;
    Ok(())
}
```

### 4. Add Middleware

Create integration layer modules:

```rust
// src/integration/middleware.rs
pub fn log_execution(handler: &str, args: &[String]) {
    log::info!("Executing: {} with args: {:?}", handler, args);
}

pub fn rate_limit_check(user: &str) -> Result<()> {
    // Check rate limits
    Ok(())
}
```

---

## See Also

- `examples/howto/` - Documented examples of specific features
  - `arg_actions.rs` - Enhanced ArgAction support
  - `validation.rs` - Automatic validation
  - `arg_groups.rs` - Argument grouping
  - `env_vars.rs` - Environment variables

- `examples/playground/` - Full-featured reference implementation
  - Complete three-layer architecture
  - Tera template rendering
  - RDF/Ontology integration
  - Multi-format output (JSON, YAML, Table)

- `examples/reference/` - Command patterns and argument types

- `CLAUDE.md` - Project architecture and testing guidelines

---

## File Statistics

| Template | Lines | Comment % | Purpose |
|---|---|---|---|
| `noun.rs.jinja` | 125 | 40% | Noun handlers |
| `verb.rs.jinja` | 279 | 35% | Verb + domain + output |
| `lib.rs.jinja` | 158 | 45% | Library entry |
| `main.rs.jinja` | 280 | 38% | Binary entry + errors |
| **Total** | **842** | **38%** | Full scaffolding suite |

**Documentation:**
- `TEMPLATES.md` - 547 lines (comprehensive reference)
- `QUICK_START.md` - 543 lines (hands-on tutorial)
- `README.md` - This file

---

## License

Copyright (c) 2024 Sean Chatman
SPDX-License-Identifier: MIT OR Apache-2.0

All templates in this directory are provided under the same license as the clap-noun-verb project, with full permission to customize, extend, and redistribute as needed.

