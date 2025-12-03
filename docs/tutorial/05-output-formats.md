# Tutorial 05: Output Formats - JSON, YAML, and Custom

**Learning Path:** Basic CLI → Production-Ready Output
**Time:** 20 minutes
**Prerequisites:** [Tutorial 04: Testing Basics](04-testing-basics.md)

---

## What You'll Learn

How to handle output formats in clap-noun-verb:
- Default JSON serialization for AI agents
- Multiple format support (JSON, YAML, custom)
- Structured output for machine consumption
- Human-readable alternatives

---

## Default: JSON for AI Agents

clap-noun-verb defaults to JSON output for machine readability:

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceStatus {
    name: String,
    status: String,
    uptime_seconds: u64,
}

#[verb(help = "Get service status")]
pub fn status(
    #[arg(help = "Service name")] name: String,
) -> Result<ServiceStatus, Box<dyn std::error::Error>> {
    let status = crate::domain::services::get_status(&name)?;

    Ok(ServiceStatus {
        name: status.name,
        status: status.state.to_string(),
        uptime_seconds: status.uptime.as_secs(),
    })
}
```

**CLI output:**
```bash
$ myapp services status --name api
{"name":"api","status":"running","uptime_seconds":3600}
```

**Why JSON by default?**
- ✅ Machine-parseable for AI agents
- ✅ MCP protocol compatibility
- ✅ Structured data for automation
- ✅ Language-agnostic format

---

## Multiple Format Support

Add format selection with `--format` argument:

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceStatus {
    name: String,
    status: String,
    uptime_seconds: u64,
}

#[verb(help = "Get service status with format selection")]
pub fn status(
    #[arg(help = "Service name")] name: String,
    #[arg(
        help = "Output format",
        default = "json",
        value_hint = "string"
    )]
    format: String,
) -> Result<FormattedOutput, Box<dyn std::error::Error>> {
    let status = crate::domain::services::get_status(&name)?;

    let data = ServiceStatus {
        name: status.name,
        status: status.state.to_string(),
        uptime_seconds: status.uptime.as_secs(),
    };

    Ok(match format.as_str() {
        "json" => FormattedOutput::json(data)?,
        "yaml" => FormattedOutput::yaml(data)?,
        "text" => FormattedOutput::text(format_status_text(&data)),
        _ => return Err(format!("Unknown format: {}", format).into()),
    })
}

fn format_status_text(status: &ServiceStatus) -> String {
    format!(
        "Service: {}\nStatus: {}\nUptime: {} seconds",
        status.name, status.status, status.uptime_seconds
    )
}
```

**Usage:**
```bash
# JSON (default)
$ myapp services status --name api
{"name":"api","status":"running","uptime_seconds":3600}

# YAML
$ myapp services status --name api --format yaml
name: api
status: running
uptime_seconds: 3600

# Human-readable text
$ myapp services status --name api --format text
Service: api
Status: running
Uptime: 3600 seconds
```

---

## Custom Output Types

### Pattern 1: Simple Wrapper

```rust
use serde::Serialize;

pub enum FormattedOutput {
    Json(String),
    Yaml(String),
    Text(String),
}

impl FormattedOutput {
    pub fn json<T: Serialize>(data: T) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::Json(serde_json::to_string_pretty(&data)?))
    }

    pub fn yaml<T: Serialize>(data: T) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::Yaml(serde_yaml::to_string(&data)?))
    }

    pub fn text(content: String) -> Self {
        Self::Text(content)
    }
}

// Implement Display for automatic printing
impl std::fmt::Display for FormattedOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(s) | Self::Yaml(s) | Self::Text(s) => write!(f, "{}", s),
        }
    }
}

// Implement Serialize for clap-noun-verb
impl Serialize for FormattedOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Json(s) => serializer.serialize_str(s),
            Self::Yaml(s) => serializer.serialize_str(s),
            Self::Text(s) => serializer.serialize_str(s),
        }
    }
}
```

---

### Pattern 2: Table Output

```rust
use tabled::{Table, Tabled};
use serde::Serialize;

#[derive(Serialize, Tabled)]
pub struct ServiceInfo {
    #[tabled(rename = "Name")]
    pub name: String,

    #[tabled(rename = "Status")]
    pub status: String,

    #[tabled(rename = "Uptime")]
    pub uptime: String,
}

#[verb(help = "List all services")]
pub fn list(
    #[arg(help = "Output format", default = "table")]
    format: String,
) -> Result<FormattedOutput, Box<dyn std::error::Error>> {
    let services = crate::domain::services::list_all()?;

    let data: Vec<ServiceInfo> = services
        .into_iter()
        .map(|s| ServiceInfo {
            name: s.name,
            status: s.state.to_string(),
            uptime: format_duration(s.uptime),
        })
        .collect();

    Ok(match format.as_str() {
        "json" => FormattedOutput::json(data)?,
        "yaml" => FormattedOutput::yaml(data)?,
        "table" => {
            let table = Table::new(&data).to_string();
            FormattedOutput::text(table)
        }
        _ => return Err(format!("Unknown format: {}", format).into()),
    })
}
```

**Dependencies:**
```toml
[dependencies]
tabled = "0.15"
serde_yaml = "0.9"
```

**Table output:**
```bash
$ myapp services list --format table
+------+---------+---------+
| Name | Status  | Uptime  |
+------+---------+---------+
| api  | running | 1h 30m  |
| db   | running | 2h 15m  |
| cache| stopped | 0s      |
+------+---------+---------+
```

---

## Nested Structures

Handle complex nested data:

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct DeploymentStatus {
    pub deployment_id: String,
    pub environment: String,
    pub services: Vec<ServiceInfo>,
    pub metadata: DeploymentMetadata,
}

#[derive(Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub replicas: u32,
}

#[derive(Serialize)]
pub struct DeploymentMetadata {
    pub deployed_by: String,
    pub deployed_at: String,
    pub commit_sha: String,
}

#[verb(help = "Get deployment status")]
pub fn deployment_status(
    #[arg(help = "Deployment ID")] id: String,
) -> Result<DeploymentStatus, Box<dyn std::error::Error>> {
    let deployment = crate::domain::deployments::get(&id)?;

    Ok(DeploymentStatus {
        deployment_id: deployment.id,
        environment: deployment.environment,
        services: deployment.services.into_iter().map(|s| ServiceInfo {
            name: s.name,
            version: s.version,
            replicas: s.replicas,
        }).collect(),
        metadata: DeploymentMetadata {
            deployed_by: deployment.metadata.user,
            deployed_at: deployment.metadata.timestamp.to_rfc3339(),
            commit_sha: deployment.metadata.commit,
        },
    })
}
```

**JSON output:**
```json
{
  "deployment_id": "dep-123",
  "environment": "production",
  "services": [
    {
      "name": "api",
      "version": "v2.1.0",
      "replicas": 3
    },
    {
      "name": "worker",
      "version": "v2.1.0",
      "replicas": 2
    }
  ],
  "metadata": {
    "deployed_by": "alice",
    "deployed_at": "2025-12-03T18:00:00Z",
    "commit_sha": "abc123def"
  }
}
```

---

## Exercise: Multi-Format User Report

**Goal:** Create a user report command with JSON, YAML, and text formats

**Arrange:** Define domain logic

```rust
// domain/users.rs
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

pub fn get_user_report(user_id: u64) -> Result<User, DomainError> {
    // Load user from database/storage
}
```

**Act:** Create CLI command with format support

```rust
// commands/users.rs
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserReport {
    id: u64,
    username: String,
    email: String,
    created_at: String,
    status: String,
}

#[verb(help = "Generate user report")]
pub fn report(
    #[arg(help = "User ID", value_hint = "number")] user_id: u64,
    #[arg(help = "Output format", default = "json")] format: String,
) -> Result<FormattedOutput, Box<dyn std::error::Error>> {
    let user = crate::domain::users::get_user_report(user_id)?;

    let report = UserReport {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at.to_rfc3339(),
        status: if user.is_active { "active" } else { "inactive" }.to_string(),
    };

    match format.as_str() {
        "json" => FormattedOutput::json(report),
        "yaml" => FormattedOutput::yaml(report),
        "text" => Ok(FormattedOutput::text(format_user_text(&report))),
        _ => Err(format!("Unknown format: {}", format).into()),
    }
}

fn format_user_text(report: &UserReport) -> String {
    format!(
        "User Report\n\
         ===========\n\
         ID: {}\n\
         Username: {}\n\
         Email: {}\n\
         Created: {}\n\
         Status: {}",
        report.id, report.username, report.email, report.created_at, report.status
    )
}
```

**Assert:** Test all formats

```bash
# JSON
$ myapp users report --user-id 42
{"id":42,"username":"alice","email":"alice@example.com","created_at":"2025-01-15T12:00:00Z","status":"active"}

# YAML
$ myapp users report --user-id 42 --format yaml
id: 42
username: alice
email: alice@example.com
created_at: '2025-01-15T12:00:00Z'
status: active

# Text
$ myapp users report --user-id 42 --format text
User Report
===========
ID: 42
Username: alice
Email: alice@example.com
Created: 2025-01-15T12:00:00Z
Status: active
```

---

## Streaming Output

For large datasets, use streaming:

```rust
#[verb(help = "Export all users")]
pub fn export(
    #[arg(help = "Output format", default = "jsonl")] format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let users = crate::domain::users::stream_all()?;

    match format.as_str() {
        "jsonl" => {
            for user in users {
                let json = serde_json::to_string(&user)?;
                println!("{}", json);
            }
        }
        "csv" => {
            println!("id,username,email,status");
            for user in users {
                println!("{},{},{},{}", user.id, user.username, user.email, user.status);
            }
        }
        _ => return Err(format!("Unknown format: {}", format).into()),
    }

    Ok(())
}
```

**JSONL (JSON Lines) output:**
```bash
$ myapp users export --format jsonl
{"id":1,"username":"alice","email":"alice@example.com","status":"active"}
{"id":2,"username":"bob","email":"bob@example.com","status":"inactive"}
{"id":3,"username":"carol","email":"carol@example.com","status":"active"}
```

---

## Key Takeaways

✅ **JSON default** - Machine-readable for AI agents and MCP
✅ **Multiple formats** - JSON, YAML, text, tables, CSV
✅ **Structured output** - Nested data with serde serialization
✅ **Format selection** - `--format` argument for flexibility
✅ **Streaming support** - JSONL for large datasets

---

## Next Steps

- **[Tutorial 06: Autonomic Features](06-autonomic-features.md)** - Machine-grade introspection
- **[Tutorial 07: Async Operations](07-async-operations.md)** - Async CLI commands
- **[How-To: Custom Serialization](../howto/advanced/custom-serialization.md)** - Advanced output patterns

**Estimated time to next tutorial:** 20 minutes

---

## Quick Reference: Format Options

| Format | Use Case | Library |
|--------|----------|---------|
| **JSON** | Default, machine-readable | `serde_json` |
| **YAML** | Human-friendly structured | `serde_yaml` |
| **Text** | Simple human output | Built-in `format!` |
| **Table** | Tabular data | `tabled` |
| **CSV** | Spreadsheet export | `csv` |
| **JSONL** | Streaming large data | `serde_json` (per line) |

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
