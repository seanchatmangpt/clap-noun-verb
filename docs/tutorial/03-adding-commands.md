# Tutorial 03: Adding Commands - Organization & Discovery

**Learning Path:** Basic CLI → Multi-Command Applications
**Time:** 20 minutes
**Prerequisites:** [Tutorial 02: Domain Separation](02-domain-separation.md)

---

## What You'll Learn

How to organize commands in clap-noun-verb:
- Auto-discovery of nouns and verbs
- Explicit noun/verb specification
- Command grouping patterns
- Argument attributes (Phase 2 tags)

---

## Auto-Discovery: The Magic

clap-noun-verb automatically discovers commands from:
1. **Module names** → Nouns
2. **Function names** → Verbs

### Example: File-Based Discovery

```
src/
└── commands/
    ├── services.rs    # Noun: "services"
    ├── database.rs    # Noun: "database"
    └── config.rs      # Noun: "config"
```

```rust
// commands/services.rs - Module name becomes noun
use clap_noun_verb_macros::verb;

#[verb] // Auto-discovered as "services status"
pub fn show_status() -> Result<ServiceStatus, Box<dyn std::error::Error>> {
    // ...
}

#[verb] // Auto-discovered as "services restart"
pub fn restart(
    #[arg(help = "Service name")] name: String,
) -> Result<RestartResult, Box<dyn std::error::Error>> {
    // ...
}
```

**CLI usage:**
```bash
myapp services status
myapp services restart --name api
```

---

## Explicit Specification

Sometimes you need explicit control over nouns and verbs:

### Explicit Verb Names

```rust
#[verb("logs")] // Explicit verb name (function name doesn't matter)
pub fn show_service_logs(
    #[arg(help = "Service name")] service: String,
) -> Result<LogsOutput, Box<dyn std::error::Error>> {
    // ...
}
```

Usage: `myapp services logs --service api`

---

### Explicit Noun Override

```rust
// In basic.rs (not a domain noun)
#[verb("status", "cluster")] // Explicit noun + verb
pub fn show_cluster_status() -> Result<ClusterStatus, Box<dyn std::error::Error>> {
    // ...
}
```

Usage: `myapp cluster status`

**When to use:**
- File name doesn't match noun (e.g., `basic.rs`, `utils.rs`)
- Multiple nouns in one file
- Legacy code migration

---

## Argument Attributes (Phase 2 Tags)

clap-noun-verb v5.2.0 supports 10 Phase 2 argument attributes:

### 1. `[help]` - Argument Description

```rust
#[verb(help = "Deploy application to environment")]
fn deploy(
    #[arg(help = "Target environment (dev, staging, prod)")] env: String,
    #[arg(help = "Docker image tag to deploy")] tag: String,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    // ...
}
```

---

### 2. `[default]` - Default Values

```rust
#[verb]
fn connect(
    #[arg(help = "Server hostname", default = "localhost")] host: String,
    #[arg(help = "Server port", default = "5432")] port: u16,
) -> Result<ConnectionStatus, Box<dyn std::error::Error>> {
    // Uses "localhost:5432" if not specified
}
```

---

### 3. `[env]` - Environment Variables

```rust
#[verb]
fn backup(
    #[arg(
        env = "DATABASE_URL",
        help = "Database connection string"
    )]
    database_url: String,
) -> Result<BackupResult, Box<dyn std::error::Error>> {
    // Reads from DATABASE_URL env var if not provided as argument
}
```

**CLI usage:**
```bash
# From environment
export DATABASE_URL="postgres://localhost/mydb"
myapp database backup

# Or explicit argument
myapp database backup --database-url "postgres://localhost/mydb"
```

---

### 4. `[value_hint]` - Shell Completion Hints

```rust
#[verb]
fn open_file(
    #[arg(
        help = "File to process",
        value_hint = "file_path"
    )]
    file_path: String,
) -> Result<FileInfo, Box<dyn std::error::Error>> {
    // Shell completion suggests file paths
}
```

**Supported hints:**
- `"file_path"` - File paths
- `"dir_path"` - Directory paths
- `"number"` - Numeric values
- `"url"` - URLs
- `"email"` - Email addresses
- `"username"` - Usernames
- `"hostname"` - Hostnames

---

### 5. `[requires]` - Argument Dependencies

```rust
#[verb]
fn deploy(
    #[arg(help = "Enable verbose output")] verbose: bool,
    #[arg(
        help = "Log file path (required if verbose=true)",
        requires = "verbose"
    )]
    log_file: Option<String>,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    // --log-file can only be used with --verbose
}
```

**CLI behavior:**
```bash
# ✅ Valid
myapp deploy --verbose --log-file /tmp/deploy.log

# ❌ Error: --log-file requires --verbose
myapp deploy --log-file /tmp/deploy.log
```

---

### 6. `[conflicts]` - Mutually Exclusive Args

```rust
#[verb]
fn output(
    #[arg(help = "Output in JSON format", conflicts = "yaml")] json: bool,
    #[arg(help = "Output in YAML format", conflicts = "json")] yaml: bool,
) -> Result<OutputResult, Box<dyn std::error::Error>> {
    // Can't use --json and --yaml together
}
```

---

### 7. `[group]` - Argument Groups

```rust
#[verb]
fn authenticate(
    #[arg(
        help = "API token",
        group = "auth"
    )]
    token: Option<String>,

    #[arg(
        help = "Username",
        group = "auth"
    )]
    username: Option<String>,

    #[arg(
        help = "Password",
        group = "auth",
        requires = "username"
    )]
    password: Option<String>,
) -> Result<AuthResult, Box<dyn std::error::Error>> {
    // Must provide either --token OR (--username + --password)
}
```

---

### 8. `[hide]` - Hide from Help

```rust
#[verb]
fn debug_command(
    #[arg(help = "Enable debug mode", hide)] debug: bool,
) -> Result<DebugInfo, Box<dyn std::error::Error>> {
    // --debug hidden from help text (internal use)
}
```

---

### 9. `[help_heading]` - Group in Help Text

```rust
#[verb]
fn configure(
    #[arg(
        help = "Database host",
        help_heading = "Database Options"
    )]
    db_host: String,

    #[arg(
        help = "Database port",
        help_heading = "Database Options"
    )]
    db_port: u16,

    #[arg(
        help = "Log level",
        help_heading = "Logging Options"
    )]
    log_level: String,
) -> Result<ConfigResult, Box<dyn std::error::Error>> {
    // Groups related arguments in help output
}
```

**Help output:**
```
Database Options:
  --db-host <DB_HOST>    Database host
  --db-port <DB_PORT>    Database port

Logging Options:
  --log-level <LOG_LEVEL>  Log level
```

---

### 10. `[global]` - Global Arguments

```rust
#[verb]
fn status(
    #[arg(help = "Output format", global)] format: String,
) -> Result<StatusResult, Box<dyn std::error::Error>> {
    // --format available to all subcommands
}
```

---

## Exercise: Build a Multi-Command CLI

**Goal:** Create a service manager with multiple commands

**Arrange:** Set up project structure

```
service-manager/
└── src/
    ├── main.rs
    ├── domain/
    │   └── service.rs
    └── commands/
        ├── mod.rs
        └── services.rs
```

**Act:** Implement commands with Phase 2 tags

```rust
// domain/service.rs
pub fn start_service(name: &str, port: u16) -> Result<ServiceInfo, DomainError> {
    // Pure domain logic
}

pub fn stop_service(name: &str) -> Result<ServiceInfo, DomainError> {
    // Pure domain logic
}

pub fn get_status(name: &str) -> Result<ServiceStatus, DomainError> {
    // Pure domain logic
}

// commands/services.rs
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceOutput {
    name: String,
    status: String,
    port: u16,
}

/// Start a service
#[verb(help = "Start a service on specified port")]
pub fn start(
    #[arg(help = "Service name to start")] name: String,
    #[arg(
        help = "Port to bind to",
        default = "8080",
        value_hint = "number"
    )]
    port: u16,
    #[arg(
        help = "Enable verbose logging",
        env = "VERBOSE"
    )]
    verbose: bool,
) -> Result<ServiceOutput, Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("Starting service {} on port {}", name, port);
    }

    let info = crate::domain::service::start_service(&name, port)?;
    Ok(ServiceOutput::from(info))
}

/// Stop a service
#[verb(help = "Stop a running service")]
pub fn stop(
    #[arg(help = "Service name to stop")] name: String,
    #[arg(help = "Force stop", conflicts = "graceful")] force: bool,
    #[arg(help = "Graceful shutdown", conflicts = "force")] graceful: bool,
) -> Result<ServiceOutput, Box<dyn std::error::Error>> {
    let info = crate::domain::service::stop_service(&name)?;
    Ok(ServiceOutput::from(info))
}

/// Show service status
#[verb(help = "Get service status")]
pub fn status(
    #[arg(help = "Service name")] name: String,
    #[arg(
        help = "Output format",
        default = "json",
        group = "output"
    )]
    format: String,
) -> Result<ServiceOutput, Box<dyn std::error::Error>> {
    let status = crate::domain::service::get_status(&name)?;
    Ok(ServiceOutput::from(status))
}
```

**Assert:** Test the CLI

```bash
# Start service
cargo run -- services start --name api --port 3000
{"name":"api","status":"running","port":3000}

# Stop with force
cargo run -- services stop --name api --force
{"name":"api","status":"stopped","port":0}

# Get status
cargo run -- services status --name api
{"name":"api","status":"stopped","port":0}
```

---

## Command Organization Patterns

### Pattern 1: Resource-Based (Recommended)

Group by domain resource (noun):

```
commands/
├── users.rs      # user create, user delete, user list
├── projects.rs   # project create, project deploy
└── services.rs   # service start, service stop, service status
```

---

### Pattern 2: Action-Based

Group by action type:

```
commands/
├── create.rs     # create_user, create_project
├── delete.rs     # delete_user, delete_project
└── list.rs       # list_users, list_projects
```

**Use when:** Actions are more important than resources

---

### Pattern 3: Hybrid

Mix both patterns:

```
commands/
├── users/
│   ├── crud.rs       # create, update, delete
│   └── advanced.rs   # migrate, import, export
└── services/
    ├── lifecycle.rs  # start, stop, restart
    └── monitoring.rs # status, logs, metrics
```

---

## Key Takeaways

✅ **Auto-discovery** - Module names → nouns, function names → verbs
✅ **Explicit control** - Override nouns/verbs when needed
✅ **Phase 2 tags** - 10 argument attributes for rich CLIs
✅ **Organized structure** - Group commands by resource or action
✅ **Type-safe** - Compile-time validation of all attributes

---

## Next Steps

- **[Tutorial 04: Testing Basics](04-testing-basics.md)** - Test your commands with Chicago TDD
- **[Tutorial 05: Output Formats](05-output-formats.md)** - JSON, YAML, custom serialization
- **[Reference: Verb Macro](../reference/api/verb-macro.md)** - Complete verb attribute reference
- **[Reference: Arg Attributes](../reference/api/arg-attributes.md)** - All Phase 2 tags documented

**Estimated time to next tutorial:** 20 minutes

---

## Quick Reference: Phase 2 Tags

| Tag | Purpose | Example |
|-----|---------|---------|
| `help` | Argument description | `#[arg(help = "Port number")]` |
| `default` | Default value | `#[arg(default = "8080")]` |
| `env` | Environment variable | `#[arg(env = "PORT")]` |
| `value_hint` | Shell completion | `#[arg(value_hint = "file_path")]` |
| `requires` | Dependency | `#[arg(requires = "verbose")]` |
| `conflicts` | Mutual exclusion | `#[arg(conflicts = "json")]` |
| `group` | Argument group | `#[arg(group = "auth")]` |
| `hide` | Hide from help | `#[arg(hide)]` |
| `help_heading` | Group in help | `#[arg(help_heading = "Database")]` |
| `global` | Global argument | `#[arg(global)]` |

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
