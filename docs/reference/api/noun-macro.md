# Reference: #[noun] Macro

**File**: `clap-noun-verb-macros/src/lib.rs`

## Signature

```rust
#[noun("name", "about")]
```

## Description

The `#[noun]` macro defines a top-level command container that groups related verbs. It sets up the command structure that appears in help and handles routing to verb handlers.

## Parameters

### Required:
- `name` - String literal naming the noun command (e.g., `"user"`, `"database"`, `"config"`)
- `about` - String literal describing the noun's purpose (shown in help)

## Macro Expansion

The macro:

1. Creates a command group with the specified name
2. Registers itself for auto-discovery
3. Sets up verb routing
4. Generates help text
5. Implements command parsing

## Examples

### Basic Noun

```rust
#[noun("user", "User management commands")]
#[verb("list")]
fn list_users() -> Result<Vec<User>> {
    Ok(vec![])
}

#[noun("user", "User management commands")]
#[verb("create")]
fn create_user(username: String) -> Result<UserId> {
    Ok(UserId(1))
}
```

**Usage**:
```bash
$ myapp user list
$ myapp user create alice
$ myapp user --help
```

### Multiple Nouns

```rust
#[noun("user", "User management")]
#[verb("list")]
fn user_list() -> Result<Vec<User>> { }

#[noun("database", "Database operations")]
#[verb("list")]
fn db_list() -> Result<Vec<Database>> { }

#[noun("config", "Configuration")]
#[verb("get")]
fn config_get(key: String) -> Result<String> { }
```

**Usage**:
```bash
$ myapp user list          # User commands
$ myapp database list      # Database commands
$ myapp config get timeout # Config commands
```

## Verb Organization

All verbs under a noun should use the same noun name:

```rust
#[noun("database", "Database operations")]
#[verb("create")]
fn db_create(name: String) -> Result<DatabaseId> { }

#[noun("database", "Database operations")]
#[verb("drop")]
fn db_drop(name: String) -> Result<Output> { }

#[noun("database", "Database operations")]
#[verb("migrate")]
fn db_migrate(target_version: u32) -> Result<Output> { }
```

This generates:
```bash
$ myapp database create mydb
$ myapp database drop mydb
$ myapp database migrate 5
```

## Help Output

The noun name and about string appear in main help:

```bash
$ myapp --help

COMMANDS:
  config      Configuration management
  database    Database operations
  user        User management
  help        Print this message or the help of the given subcommand(s)
```

Each noun shows its verbs:

```bash
$ myapp user --help

User management commands

COMMANDS:
  create    Create new user
  delete    Delete user
  list      List all users
  update    Update user
  help      Print this message or the help of the given subcommand(s)
```

## Argument Inheritance

Arguments declared at noun level with `[global]` tag are available to all verbs:

```rust
#[noun("user", "User management")]
/// # Arguments
/// * `config` - Config file [global]
fn user_ops() -> Result<()> { }

#[noun("user", "User management")]
#[verb("create")]
fn create_user(username: String, config: Option<String>) -> Result<UserId> {
    // `config` available here via [global] tag
}
```

## Naming Conventions

**Best Practices**:
- Use lowercase noun names: `"user"`, `"database"`, `"config"`
- Use snake_case for multi-word nouns: `"api_key"`, `"ssh_config"`
- Keep nouns singular or noun-like: `"user"` not `"users"`, `"config"` not `"configurations"`
- Make about strings descriptive (50-80 chars recommended)

**Examples**:
```rust
#[noun("user", "User account management")]
#[noun("api_key", "API key management")]
#[noun("ssh_config", "SSH configuration")]
#[noun("data_export", "Data export utilities")]
```

## Auto-Discovery

The macro uses `linkme` distributed slice for compile-time registration:

```rust
// Generated internally by the macro:
#[linkme::distributed_slice(COMMANDS)]
static NOUN_NAME: CommandDefinition = CommandDefinition { ... };
```

This enables automatic command discovery without manual registration.

## Error Handling

Nouns don't return values directly. Errors come from verbs:

```rust
#[noun("user", "User management")]
#[verb("create")]
fn create_user(email: String) -> Result<UserId, UserError> {
    if !email.contains('@') {
        return Err(UserError::InvalidEmail);
    }
    Ok(UserId(1))
}
```

## Migration (v5.1 → v5.2)

**Before**:
```rust
#[noun("user", "User management")]
// Manual verb registration via builder API
```

**After**:
```rust
#[noun("user", "User management")]
// Verbs auto-discovered with #[verb] macro
```

No changes needed for existing noun declarations - fully backward compatible.

## See Also

- `#[verb]` - Child command macro
- `HandlerInput` - Argument access in verbs
- Result<T> - Return type requirements
- Auto-Discovery - How verbs are found
