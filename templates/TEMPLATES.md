# clap-noun-verb Code Generation Templates

This directory contains Jinja2 templates for scaffolding clap-noun-verb CLI projects. Use these templates to quickly bootstrap new noun commands, verb handlers, and library structures while maintaining consistent patterns and best practices.

## File Organization

```
templates/
├── noun.rs.jinja          # Template for noun command handlers
├── verb.rs.jinja          # Template for verb implementations
├── lib.rs.jinja           # Template for main library scaffolding
├── main.rs.jinja          # Template for binary entry point
└── TEMPLATES.md           # This file
```

---

## Template: `noun.rs.jinja`

**Purpose**: Scaffold a noun command handler module.

**Output**: `src/commands/{noun_name}.rs`

### Placeholder Syntax

| Placeholder | Type | Description | Example |
|---|---|---|---|
| `{{ module_name }}` | String | Module name for documentation | `users` |
| `{{ noun_name }}` | String | Kebab-case noun identifier | `user-management` |
| `{{ noun_name_title }}` | String | Title-case noun | `User Management` |
| `{{ noun_struct_name }}` | String | PascalCase struct name | `UserManagement` |
| `{{ noun_output_struct }}` | String | PascalCase output struct | `UserManagement` |
| `{{ domain_struct_name }}` | String | Domain layer struct | `UserState` |
| `{{ brief_description }}` | String | One-line description | `Manage user accounts and permissions` |

### Usage Pattern

```jinja2
// Generated struct name: {{ noun_struct_name }}
pub struct {{ noun_struct_name }} {
    // {{ brief_description }}
}

impl {{ noun_struct_name }} {
    pub fn new() -> Self {
        // PLACEHOLDER: Initialize {{ noun_name }}-specific state
    }
}
```

### Example Variable Set

```toml
[variables]
module_name = "users"
noun_name = "users"
noun_name_title = "Users"
noun_struct_name = "UsersCommand"
noun_output_struct = "UsersCommand"
domain_struct_name = "UserState"
brief_description = "Manage user accounts and permissions"
```

### Generated Output Example

```rust
pub struct UsersCommand {
    // Placeholder: Initialize users-specific state
}

impl Default for UsersCommand {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Template: `verb.rs.jinja`

**Purpose**: Scaffold a verb implementation with domain logic, CLI binding, and integration layers.

**Output**: `src/commands/{noun_name}/{verb_name}.rs`

### Placeholder Syntax

| Placeholder | Type | Description | Example |
|---|---|---|---|
| `{{ verb_name }}` | String | Kebab-case verb identifier | `create-user` |
| `{{ verb_name_title }}` | String | Title-case verb | `Create User` |
| `{{ brief_description }}` | String | One-line description | `Create a new user account` |
| `{{ noun_name }}` | String | Parent noun | `users` |
| `{{ verb_domain_struct }}` | String | Domain struct name | `CreateUserDomain` |
| `{{ verb_output_struct }}` | String | Output struct name | `CreateUserOutput` |
| `{{ verb_handler_name }}` | String | Function name for #[verb] | `create_user` |

### Architecture Pattern

The template enforces a **three-layer architecture**:

```
1. DOMAIN LAYER (Pure Logic)
   ↓
2. OUTPUT LAYER (Serialization)
   ↓
3. CLI LAYER (Validation & Binding)
```

### Layer Breakdown

**Domain Layer** (`{{ verb_domain_struct }}`):
- Pure business logic, no I/O
- `fn execute() -> Result<{{ verb_output_struct }}>`
- Fully testable, reusable

**Output Layer** (`{{ verb_output_struct }}`):
- `#[derive(Serialize, Deserialize)]`
- JSON-ready structure
- Agent-consumable format

**CLI Layer** (`#[verb]`):
- Thin validation wrapper
- Argument parsing & binding
- Error handling with exit codes

### Example Variable Set

```toml
[variables]
verb_name = "create"
verb_name_title = "Create"
brief_description = "Create a new user account"
noun_name = "users"
verb_domain_struct = "CreateUserDomain"
verb_output_struct = "CreateUserOutput"
verb_handler_name = "create_user"
```

### Generated Output Example

```rust
#[derive(Debug, Clone)]
pub struct CreateUserDomain {
    // Pure business logic
}

impl CreateUserDomain {
    pub fn execute(&self) -> Result<CreateUserOutput> {
        // Domain-only logic, no CLI or I/O
    }
}

#[derive(Serialize)]
pub struct CreateUserOutput {
    pub id: String,
    pub status: String,
}

#[verb("users", "create")]
fn create_user(
    #[arg(index = 1)] name: Option<String>,
) -> Result<CreateUserOutput> {
    // Validate CLI args
    let domain = CreateUserDomain::new(name);
    // Call pure domain logic
    domain.execute()
}
```

### Testing Pattern (Included)

Uses **AAA (Arrange-Act-Assert)** pattern:

```rust
#[test]
fn test_create_user_with_valid_input() {
    // Arrange
    let domain = CreateUserDomain::new("alice".to_string());

    // Act
    let result = domain.execute();

    // Assert
    assert!(result.is_ok());
}
```

---

## Template: `lib.rs.jinja`

**Purpose**: Scaffold the main library crate entry point with module organization.

**Output**: `src/lib.rs`

### Placeholder Syntax

| Placeholder | Type | Description | Example |
|---|---|---|---|
| `{{ crate_name }}` | String | Crate name | `my-cli` |
| `{{ crate_description }}` | String | Crate description | `A command-line tool for project management` |

### Module Structure

```rust
pub mod domain;         // Pure business logic
pub mod integration;    // Glue code & I/O
pub mod commands;       // CLI layer & #[verb] handlers
pub mod outputs;        // JSON serialization types

pub fn run() -> Result<()> {
    clap_noun_verb::run()  // Auto-discovers all #[verb] commands
}
```

### Architecture Diagram (Included)

```
┌──────────────┐
│  CLI Layer   │  ← commands/ (#[verb] attributes)
└────────┬─────┘
         │
┌────────▼──────────┐
│ Integration       │  ← Tera, file I/O, external services
└────────┬──────────┘
         │
┌────────▼──────────┐
│  Domain Layer     │  ← Pure business logic (testable)
└──────────────────┘
```

### Example Variable Set

```toml
[variables]
crate_name = "my-cli"
crate_description = "A command-line tool for project management"
```

### Generated Module Tree

```
src/
├── lib.rs              # Main crate entry
├── main.rs             # Binary wrapper
├── commands/
│   ├── mod.rs          # Command module index
│   ├── users.rs        # #[verb] handlers for users
│   └── projects.rs     # #[verb] handlers for projects
├── domain/
│   ├── mod.rs          # Domain module index
│   ├── users.rs        # User business logic
│   └── projects.rs     # Project business logic
├── integration/
│   ├── mod.rs          # Integration module index
│   ├── file_system.rs  # File I/O operations
│   └── http.rs         # HTTP client
└── outputs/
    ├── mod.rs          # Output types
    ├── users.rs        # User output types
    └── projects.rs     # Project output types
```

---

## Template: `main.rs.jinja`

**Purpose**: Scaffold the binary entry point with error handling and initialization.

**Output**: `src/main.rs`

### Placeholder Syntax

| Placeholder | Type | Description | Example |
|---|---|---|---|
| `{{ crate_lib }}` | String | Library crate name (module path) | `my_cli` |
| `{{ crate_name }}` | String | Display name | `my-cli` |
| `{{ crate_description }}` | String | Purpose description | `Project management CLI` |
| `{{ example_noun }}` | String | Example noun for docs | `users` |
| `{{ example_verb }}` | String | Example verb for docs | `create` |

### Exit Code Semantics

| Exit Code | Error Type | Recovery |
|---|---|---|
| 0 | Success | N/A |
| 1 | **ValidationError** | Fix input and retry |
| 2 | **ExecutionError** | Check logs, retry |
| 3 | **ConfigurationError** | Fix config, retry |

### Error Formatting Function

Included `format_error_with_code()` maps error types to exit codes and user-friendly messages:

```rust
fn format_error_with_code(error: &NounVerbError) -> (i32, String)
```

### Example Variable Set

```toml
[variables]
crate_lib = "my_cli"
crate_name = "my-cli"
crate_description = "A command-line tool for project management"
example_noun = "users"
example_verb = "create"
```

### Generated Entry Point

```rust
fn main() {
    match my_cli::run() {
        Ok(()) => process::exit(0),
        Err(e) => {
            let (exit_code, msg) = format_error_with_code(&e);
            eprintln!("{}", msg);
            process::exit(exit_code);
        }
    }
}
```

---

## Integration with Playground Example

The **playground** example demonstrates all template patterns in practice:

```bash
# Source: /Users/sac/clap-noun-verb/examples/playground/

# Structure matches templates:
src/
├── main.rs           # → main.rs.jinja template
├── lib.rs            # → lib.rs.jinja template
├── commands/
│   ├── papers.rs     # → verb.rs.jinja template
│   └── thesis.rs     # → verb.rs.jinja template
├── domain/           # Pure business logic
├── integration/      # Tera, RDF, file I/O
└── outputs/          # JSON output types
```

### Real Usage Example

To generate a new `users` noun with `create` verb:

```bash
# 1. Create module structure
mkdir -p src/commands/users
mkdir -p src/domain/users
mkdir -p src/integration/users
mkdir -p src/outputs

# 2. Render noun.rs.jinja template
# Variables: module_name="users", noun_struct_name="UsersCommand", etc.
# Output: src/commands/users.rs

# 3. Render verb.rs.jinja template
# Variables: verb_name="create", verb_handler_name="create_user", etc.
# Output: src/commands/users/create.rs

# 4. Update lib.rs to export new modules
# Add: pub mod users;

# 5. Update mod.rs to register module
# Add: mod users;
```

---

## Placeholder Conventions

All templates follow consistent placeholder syntax:

```jinja2
{{ variable_name }}        # Simple substitution (required)
{{ variable_name|default("fallback") }}  # With default value

# PLACEHOLDER: comment style
// PLACEHOLDER: Inline comment indicating where code generation can occur
```

### Naming Conventions

1. **Kebab-case** for CLI identifiers: `user-management`, `create-project`
2. **PascalCase** for Rust types: `UserManagement`, `CreateProject`
3. **snake_case** for functions: `create_user`, `list_projects`
4. **UPPERCASE** for constants: `DEFAULT_TIMEOUT`, `MAX_RETRIES`

---

## Example: Complete Generation Workflow

### Step 1: Define Noun

```bash
# Create users command handler
# Variables:
noun_name = "users"
noun_struct_name = "UsersCommand"
brief_description = "Manage user accounts and permissions"
```

**Input**: `noun.rs.jinja`
**Output**: `src/commands/users.rs`

### Step 2: Define Verb under Noun

```bash
# Create user verb (create_user verb)
# Variables:
noun_name = "users"
verb_name = "create"
verb_handler_name = "create_user"
brief_description = "Create a new user account"
```

**Input**: `verb.rs.jinja`
**Output**: `src/commands/users/create.rs`

### Step 3: Update Library Module

Use `lib.rs.jinja` to ensure module structure is correct:

```rust
pub mod domain;
pub mod integration;
pub mod commands;    // Auto-discovers #[verb] in all submodules
pub mod outputs;
```

### Step 4: Create Binary Entry

Use `main.rs.jinja` to handle errors and initialization:

```rust
// Auto-routes all discovered #[verb] commands to handlers
// Formats errors with appropriate exit codes
```

---

## Best Practices

### 1. Domain Logic Isolation

Keep domain logic in **separate modules** from CLI:

```
domain/
├── user.rs          # struct User, impl User { ... }
└── user_commands.rs # fn create_user_logic() { ... }
```

### 2. Three-Layer Pattern

Always maintain separation:

1. **CLI** - Validates input, calls domain
2. **Domain** - Pure logic, no I/O
3. **Integration** - Calls external services

### 3. Error Handling

Use explicit error types with context:

```rust
NounVerbError::validation_error(field, value, hint)
NounVerbError::execution_error(reason)
NounVerbError::configuration_error(reason)
```

### 4. Testing

Test each layer independently:

```rust
// Domain tests (pure logic)
#[test]
fn test_user_creation() { ... }

// CLI tests (argument parsing)
#[test]
fn test_create_user_handler() { ... }

// Integration tests (end-to-end)
#[test]
fn test_create_user_with_db() { ... }
```

### 5. JSON Serialization

Always derive `Serialize` on output types:

```rust
#[derive(Debug, Serialize)]
pub struct UserOutput {
    pub id: String,
    pub name: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
}
```

---

## Template Rendering Tools

These templates are designed for:

- **Manual code generation** - Copy/paste and customize
- **Code generator tools** - Tera, Askama, Jinja2 (Python)
- **IDE code templates** - VS Code snippets, IntelliJ live templates

### VS Code Snippet Integration

To use as VS Code snippet:

```json
{
  "clap-noun-verb: Create Verb": {
    "prefix": "cnv-verb",
    "body": [
      "// Use ${1:verb_name} as variable",
      "#[verb(\"${2:noun}\", \"${1:verb}\")]",
      "fn ${3:handler_name}(/* args */) -> Result<${4:Output}> {",
      "    ${0:unimplemented!()}",
      "}"
    ],
    "description": "Generate a new verb handler"
  }
}
```

---

## See Also

- `examples/howto/` - Documented examples of specific features
- `examples/playground/` - Full-featured CLI project
- `examples/reference/` - Command and argument reference
- `src/CLAUDE.md` - Architecture and testing guidelines

