# Explanation: Design Patterns for CLIs

**Purpose**: Understand patterns and best practices for designing semantic CLIs

## Pattern 1: Command Hierarchies

### Flat vs. Hierarchical

**Flat design** (hard to navigate):
```
mycli status
mycli start
mycli stop
mycli create-user
mycli delete-user
mycli update-config
```

**Hierarchical design** (semantic clarity):
```
mycli services status
mycli services start
mycli services stop
mycli users create
mycli users delete
mycli config update
```

**In RDF**:
```turtle
cnv:Services a cnv:Noun ; cnv:name "services" .
cnv:ServiceStatus a cnv:Verb ; cnv:hasNoun cnv:Services ; cnv:name "status" .

cnv:Users a cnv:Noun ; cnv:name "users" .
cnv:UserCreate a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "create" .
```

### When to Use Hierarchies

**Good fits for hierarchies**:
- Systems with 3+ logical domains
- Natural grouping exists (Users, Services, Config)
- Each domain has 2+ related commands

**Keep flat for**:
- Very simple tools (<5 commands total)
- No natural grouping

## Pattern 2: Semantic Naming

### Noun Choices

**Resources** (typically nouns):
- `users` - User accounts
- `services` - System services
- `databases` - Database instances
- `config` - Configuration

**Abstract concepts**:
- `system` - System-wide operations
- `admin` - Administrative tasks
- `audit` - Audit and logging

### Verb Choices

**CRUD operations**:
- `create` - Create new resource
- `list` - List all resources
- `get` - Get specific resource
- `delete` - Delete resource
- `update` / `set` - Modify resource

**Status operations**:
- `status` - Current state
- `health` - Health check
- `metrics` - Metrics/stats

**Action operations**:
- `start` - Begin operation
- `stop` - End operation
- `restart` - Restart
- `reset` - Reset to initial state

**Configuration operations**:
- `show` - Display configuration
- `set` - Change setting
- `validate` - Verify configuration
- `apply` - Apply configuration

### Naming Pattern Examples

```turtle
# CRUD pattern
cnv:Users a cnv:Noun .
cnv:UserCreate a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "create" .
cnv:UserList a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "list" .
cnv:UserDelete a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "delete" .

# Status pattern
cnv:Services a cnv:Noun .
cnv:ServiceStatus a cnv:Verb ; cnv:hasNoun cnv:Services ; cnv:name "status" .
cnv:ServiceStart a cnv:Verb ; cnv:hasNoun cnv:Services ; cnv:name "start" .

# Config pattern
cnv:Config a cnv:Noun .
cnv:ConfigShow a cnv:Verb ; cnv:hasNoun cnv:Config ; cnv:name "show" .
cnv:ConfigSet a cnv:Verb ; cnv:hasNoun cnv:Config ; cnv:name "set" .
```

## Pattern 3: Information Architecture

### Single Responsibility per Verb

Each command should do one thing:

```turtle
# ❌ Too much responsibility
cnv:UserManage a cnv:Verb ;
    cnv:name "manage" ;
    cnv:description "Create, list, or delete users" .

# ✅ Single responsibility
cnv:UserCreate a cnv:Verb ; cnv:name "create" .
cnv:UserList a cnv:Verb ; cnv:name "list" .
cnv:UserDelete a cnv:Verb ; cnv:name "delete" .
```

### Consistent Naming Across Nouns

Use same verb names for same actions:

```turtle
# ✅ Consistent
cnv:Users a cnv:Noun .
cnv:UserCreate a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "create" .
cnv:UserDelete a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "delete" .

cnv:Services a cnv:Noun .
cnv:ServiceCreate a cnv:Verb ; cnv:hasNoun cnv:Services ; cnv:name "create" .
cnv:ServiceDelete a cnv:Verb ; cnv:hasNoun cnv:Services ; cnv:name "delete" .

# Users learn: "create" works on everything
# Services learn: "create" works on everything
```

## Pattern 4: Progressive Disclosure

Show simple commands first, advanced options second:

```rust
// Simple usage
mycli services status

// With options
mycli services status --detailed
mycli services status --json

// Advanced
mycli services status --detailed --json --refresh
```

In ontology:
```turtle
cnv:ServiceStatus a cnv:Verb ;
    cnv:name "status" ;
    cnv:description "Show service status" ;
    cnv:options [
        cnv:detailed "Show detailed information" ;
        cnv:json "Output in JSON format" ;
    ] .
```

## Pattern 5: Backward Compatibility

Plan for evolution of CLI:

```turtle
# Version 1.0
cnv:UserCreate a cnv:Verb ; cnv:name "create" .

# Version 2.0 - add new option, keep old one
cnv:UserCreate a cnv:Verb ;
    cnv:name "create" ;
    cnv:deprecated false ;
    cnv:version "2.0" .

# Version 3.0 - deprecate old command
cnv:UserCreate_v1 a cnv:Verb ;
    cnv:name "create" ;
    cnv:deprecated true ;
    cnv:deprecationMessage "Use 'user create' instead" .
```

## Pattern 6: Help System

Leverage RDF for automatic help:

```turtle
cnv:Users a cnv:Noun ;
    cnv:name "users" ;
    rdfs:comment "User account management system" ;
    cnv:examples [
        cnv:example1 "mycli users create --name alice --role admin" ;
        cnv:example2 "mycli users list --role admin" ;
    ] .

cnv:UserCreate a cnv:Verb ;
    cnv:name "create" ;
    rdfs:comment "Create a new user account" ;
    cnv:options [
        cnv:name "User name (required)" ;
        cnv:role "User role (admin|user)" ;
        cnv:email "Email address (optional)" ;
    ] .
```

**Generated help**:
```
$ mycli users --help
User account management system

  create   Create a new user account
  list     List all users
  delete   Delete a user

$ mycli users create --help
Create a new user account

Example:
  mycli users create --name alice --role admin

Options:
  --name <NAME>    User name (required)
  --role <ROLE>    User role (admin|user)
  --email <EMAIL>  Email address (optional)
```

## Pattern 7: Error Handling

Use consistent error patterns:

```rust
// Generated from ontology
pub async fn create_user(args: &CreateUserArgs) -> Result<CreateUserResponse> {
    // Validation errors
    if args.name.is_empty() {
        return Err(UserError::InvalidName("Name cannot be empty".into()).into());
    }

    // Domain errors
    if user_exists(&args.name).await? {
        return Err(UserError::AlreadyExists(args.name.clone()).into());
    }

    // System errors
    match db.create_user(args).await {
        Ok(user) => Ok(CreateUserResponse { user_id: user.id }),
        Err(e) => Err(UserError::DatabaseError(e.to_string()).into()),
    }
}
```

In ontology:
```turtle
cnv:UserCreate a cnv:Verb ;
    cnv:name "create" ;
    cnv:errors [
        cnv:InvalidName "User name is invalid" ;
        cnv:AlreadyExists "User already exists" ;
        cnv:DatabaseError "Database operation failed" ;
    ] .
```

## Pattern 8: Subcommand Grouping

Organize related commands:

```
mycli services
  ├── status
  ├── start
  ├── stop
  └── restart

mycli config
  ├── show
  ├── set
  └── validate
```

When to use subcommands vs. separate tools:
- **Subcommands**: Tightly related (services: start/stop/status)
- **Separate tools**: Independent purpose (mycli vs. myctl)

## Pattern 9: Extensibility

Design for future growth:

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix myapp: <https://myapp.dev/ontology#> .

# Core verbs
cnv:Users a cnv:Noun .
cnv:UserCreate a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "create" .

# App-specific extensions (future)
myapp:UserValidate a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "validate" .
myapp:UserSync a cnv:Verb ; cnv:hasNoun cnv:Users ; cnv:name "sync" .
```

Agents can discover and use extensions without modification.

## Pattern 10: Composability

Commands that work together:

```
# Get user ID
mycli users list | grep alice | awk '{print $1}'

# Use in another command
mycli users get <user-id>

# Output JSON for piping
mycli users list --json | jq '.[] | select(.role=="admin")'

# Compose operations
mycli users create --name bob && mycli users get bob
```

## Design Checklist

- ✅ Clear noun/verb hierarchy
- ✅ Consistent verb naming across nouns
- ✅ Single responsibility per command
- ✅ Helpful error messages
- ✅ Progressive disclosure of options
- ✅ Good help system
- ✅ Backward compatible evolution
- ✅ Composable commands
- ✅ Clear exit codes and output formats
- ✅ Documented with examples

## Anti-Patterns to Avoid

**❌ Cryptic command names**:
```turtle
cnv:Cmd1 a cnv:Verb ; cnv:name "c1" .  # What does this do?
```

**❌ Inconsistent verbs**:
```turtle
cnv:UserCreate a cnv:Verb ; cnv:name "create" .
cnv:ServiceAdd a cnv:Verb ; cnv:name "add" .     # Different verb for same action
```

**❌ Verbs that do too much**:
```turtle
cnv:DoStuff a cnv:Verb ; cnv:name "dostuff" .    # Too vague
```

**❌ Nouns that aren't nouns**:
```turtle
cnv:Quickly a cnv:Noun ; cnv:name "quickly" .    # Noun should be resource/thing
```

---

**Related**:
- [Tutorial 2: Create Your First RDF Ontology](../tutorials/tutorial-2-first-rdf.md)
- [How-to: Build Multi-Level CLIs](../howto/multi-level-cli.md)
- [Explanation: Semantic Web Fundamentals](semantic-web.md)
