# clap-noun-verb CLI Cookbook

Practical recipes and patterns for common CLI development tasks.

## Table of Contents

1. [Generate Code for a New Feature](#1-generate-code-for-a-new-feature)
2. [Customize Generated Output](#2-customize-generated-output)
3. [Use Multiple Templates Together](#3-use-multiple-templates-together)
4. [Search and Install Community Packs](#4-search-and-install-community-packs)
5. [Publish Your Own Template](#5-publish-your-own-template)
6. [Run Automated Workflows](#6-run-automated-workflows)
7. [Integrate with CI/CD](#7-integrate-with-cicd)
8. [Debug Template Issues](#8-debug-template-issues)
9. [Performance Tuning](#9-performance-tuning)
10. [Advanced Error Handling](#10-advanced-error-handling)

---

## 1. Generate Code for a New Feature

### Problem
You need to quickly scaffold a new feature with consistent structure.

### Solution

**Step 1: Create the module file**

```bash
# Create a new noun module
touch src/users.rs
```

**Step 2: Define your verbs**

```rust
// src/users.rs
//! User management commands

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct UserList {
    users: Vec<User>,
    total: usize,
}

/// List all users
#[verb]
fn list(
    /// Maximum number of users to return
    #[arg(short = 'n', default_value = "10")]
    limit: usize,

    /// Filter by email domain
    #[arg(short = 'd')]
    domain: Option<String>,
) -> Result<UserList> {
    // Implementation
    let users = get_users(limit, domain)?;
    Ok(UserList {
        total: users.len(),
        users,
    })
}

/// Create a new user
#[verb]
fn create(
    /// User's full name
    name: String,

    /// User's email address
    #[arg(validator = validate_email)]
    email: String,
) -> Result<User> {
    // Implementation
    let user = create_user(name, email)?;
    Ok(user)
}

/// Delete a user
#[verb]
fn delete(
    /// User ID to delete
    #[arg(index = 0)]
    user_id: u32,

    /// Skip confirmation prompt
    #[arg(short = 'f')]
    force: bool,
) -> Result<()> {
    if !force && !confirm_deletion(user_id)? {
        return Err("Deletion cancelled".into());
    }

    delete_user(user_id)?;
    Ok(())
}
```

**Step 3: Register in main**

```rust
// src/main.rs
mod users;

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
```

**Usage:**
```bash
cargo run -- users list -n 20
cargo run -- users create "Alice Smith" alice@example.com
cargo run -- users delete 42 -f
```

---

## 2. Customize Generated Output

### Problem
You need different output formats for different contexts (human-readable, machine-readable, etc.).

### Solution

**Step 1: Add format support**

```rust
use clap_noun_verb::OutputFormat;
use serde::Serialize;

#[derive(Serialize)]
struct ServiceStatus {
    name: String,
    status: String,
    uptime: u64,
}

impl ServiceStatus {
    fn to_human_readable(&self) -> String {
        format!(
            "Service: {}\nStatus: {}\nUptime: {} seconds",
            self.name, self.status, self.uptime
        )
    }
}

#[verb]
fn status(
    /// Output format: json, yaml, table, human
    #[arg(short = 'f', default_value = "json")]
    format: String,
) -> Result<ServiceStatus> {
    let status = ServiceStatus {
        name: "api".to_string(),
        status: "running".to_string(),
        uptime: 3600,
    };

    match format.as_str() {
        "human" => {
            println!("{}", status.to_human_readable());
            Ok(status)
        }
        _ => Ok(status)  // Framework handles json/yaml/table
    }
}
```

**Step 2: Color output for terminals**

```rust
use colored::Colorize;

#[derive(Serialize)]
struct HealthCheck {
    healthy: bool,
    message: String,
}

impl HealthCheck {
    fn display_colored(&self) {
        if self.healthy {
            println!("✓ {}", self.message.green());
        } else {
            println!("✗ {}", self.message.red());
        }
    }
}

#[verb]
fn health(
    /// Use colored output
    #[arg(long)]
    color: bool,
) -> Result<HealthCheck> {
    let health = HealthCheck {
        healthy: true,
        message: "All systems operational".to_string(),
    };

    if color {
        health.display_colored();
    }

    Ok(health)
}
```

**Usage:**
```bash
# JSON output
cargo run -- services status

# Human-readable
cargo run -- services status -f human

# Colored output
cargo run -- services health --color
```

---

## 3. Use Multiple Templates Together

### Problem
You need to compose complex functionality from multiple reusable components.

### Solution

**Step 1: Create shared traits**

```rust
// src/traits.rs
pub trait Listable {
    type Item: Serialize;
    fn list(&self, limit: usize) -> Result<Vec<Self::Item>>;
}

pub trait Creatable {
    type Input;
    type Output: Serialize;
    fn create(&self, input: Self::Input) -> Result<Self::Output>;
}

pub trait Deletable {
    fn delete(&self, id: u32) -> Result<()>;
}
```

**Step 2: Implement for different resources**

```rust
// src/users.rs
use crate::traits::{Listable, Creatable, Deletable};

struct UserManager;

impl Listable for UserManager {
    type Item = User;
    fn list(&self, limit: usize) -> Result<Vec<User>> {
        // Implementation
    }
}

impl Creatable for UserManager {
    type Input = CreateUserInput;
    type Output = User;
    fn create(&self, input: CreateUserInput) -> Result<User> {
        // Implementation
    }
}

// Generic CRUD verbs
#[verb]
fn list(limit: Option<usize>) -> Result<Vec<User>> {
    UserManager.list(limit.unwrap_or(10))
}

#[verb]
fn create(name: String, email: String) -> Result<User> {
    UserManager.create(CreateUserInput { name, email })
}
```

**Step 3: Reuse patterns**

```rust
// src/projects.rs
struct ProjectManager;

impl Listable for ProjectManager {
    type Item = Project;
    // Same pattern as UserManager
}

// Automatic CRUD for projects
#[verb]
fn list(limit: Option<usize>) -> Result<Vec<Project>> {
    ProjectManager.list(limit.unwrap_or(10))
}
```

---

## 4. Search and Install Community Packs

### Problem
You want to leverage existing CLI patterns and extensions.

### Solution

While clap-noun-verb doesn't have a package manager (yet), you can create reusable patterns:

**Step 1: Create a shared library**

```bash
# Create a new library crate
cargo new --lib cli-extensions
cd cli-extensions
```

**Step 2: Define reusable components**

```rust
// cli-extensions/src/lib.rs
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub build_date: String,
    pub git_hash: String,
}

pub fn get_version_info() -> Result<VersionInfo> {
    Ok(VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: env!("BUILD_DATE").to_string(),
        git_hash: env!("GIT_HASH").to_string(),
    })
}

// More reusable components...
```

**Step 3: Use in your CLI**

```rust
// Your main app
use cli_extensions::get_version_info;

#[verb]
fn version() -> Result<VersionInfo> {
    get_version_info()
}
```

---

## 5. Publish Your Own Template

### Problem
You've created useful CLI patterns and want to share them.

### Solution

**Step 1: Create a library crate**

```bash
cargo new --lib my-cli-templates
cd my-cli-templates
```

**Step 2: Define templates**

```rust
// src/database.rs
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Reusable database migration pattern
pub fn create_migration_verbs<T: DatabaseBackend>() {
    // Template for migration commands
}

#[derive(Serialize)]
pub struct MigrationStatus {
    pub applied: usize,
    pub pending: usize,
}

pub trait DatabaseBackend {
    fn migrate_up(&self) -> Result<MigrationStatus>;
    fn migrate_down(&self) -> Result<MigrationStatus>;
    fn migrate_status(&self) -> Result<MigrationStatus>;
}
```

**Step 3: Publish to crates.io**

```bash
# Add metadata to Cargo.toml
cat >> Cargo.toml << EOF
description = "Reusable CLI templates for clap-noun-verb"
keywords = ["cli", "templates", "clap"]
categories = ["command-line-utilities"]
EOF

# Publish
cargo publish
```

**Step 4: Document usage**

```markdown
# my-cli-templates

Reusable templates for clap-noun-verb CLIs.

## Installation

\`\`\`toml
[dependencies]
my-cli-templates = "0.1.0"
\`\`\`

## Usage

\`\`\`rust
use my_cli_templates::database::DatabaseBackend;

impl DatabaseBackend for MyDatabase {
    // Implement trait...
}
\`\`\`
```

---

## 6. Run Automated Workflows

### Problem
You need to execute complex multi-step workflows.

### Solution

**Step 1: Define workflow steps**

```rust
#[derive(Serialize)]
struct WorkflowResult {
    steps: Vec<StepResult>,
    success: bool,
}

#[derive(Serialize)]
struct StepResult {
    name: String,
    status: String,
    duration_ms: u64,
}

#[verb]
fn deploy(
    #[arg(index = 0)]
    environment: String,

    /// Skip tests
    #[arg(long)]
    skip_tests: bool,
) -> Result<WorkflowResult> {
    let mut steps = Vec::new();

    // Step 1: Build
    let start = std::time::Instant::now();
    build_project()?;
    steps.push(StepResult {
        name: "Build".to_string(),
        status: "success".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    });

    // Step 2: Test (optional)
    if !skip_tests {
        let start = std::time::Instant::now();
        run_tests()?;
        steps.push(StepResult {
            name: "Test".to_string(),
            status: "success".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Step 3: Deploy
    let start = std::time::Instant::now();
    deploy_to_environment(&environment)?;
    steps.push(StepResult {
        name: "Deploy".to_string(),
        status: "success".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    });

    Ok(WorkflowResult {
        steps,
        success: true,
    })
}
```

**Step 2: Add progress reporting**

```rust
use indicatif::{ProgressBar, ProgressStyle};

#[verb]
fn process(
    files: Vec<String>,
) -> Result<ProcessResult> {
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
            .unwrap()
    );

    for file in &files {
        pb.set_message(format!("Processing {}", file));
        process_file(file)?;
        pb.inc(1);
    }

    pb.finish_with_message("Done!");

    Ok(ProcessResult {
        processed: files.len(),
    })
}
```

---

## 7. Integrate with CI/CD

### Problem
You need to use your CLI in automated pipelines.

### Solution

**Step 1: Add CI-friendly output**

```rust
#[verb]
fn test(
    /// Output in CI format
    #[arg(long)]
    ci: bool,

    /// Exit code on failure
    #[arg(long, default_value = "1")]
    exit_code: i32,
) -> Result<TestResult> {
    let result = run_tests()?;

    if ci {
        // GitHub Actions format
        if !result.success {
            println!("::error::Tests failed");
        }
        for failure in &result.failures {
            println!("::error file={}::{}", failure.file, failure.message);
        }
    }

    if !result.success {
        std::process::exit(exit_code);
    }

    Ok(result)
}
```

**Step 2: GitHub Actions integration**

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build
        run: cargo build --release

      - name: Run tests
        run: ./target/release/myapp test --ci

      - name: Deploy (on main)
        if: github.ref == 'refs/heads/main'
        run: ./target/release/myapp deploy production
        env:
          DEPLOY_KEY: ${{ secrets.DEPLOY_KEY }}
```

**Step 3: Docker integration**

```dockerfile
# Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/myapp /usr/local/bin/
ENTRYPOINT ["myapp"]
```

```bash
# Run in Docker
docker run myapp services status
```

---

## 8. Debug Template Issues

### Problem
Your commands aren't working as expected.

### Solution

**Step 1: Enable debug output**

```rust
#[verb]
fn debug(
    /// Enable debug logging
    #[arg(short = 'v', action = "count")]
    verbose: usize,
) -> Result<DebugInfo> {
    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Warn,
            2 => log::LevelFilter::Info,
            3 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();

    log::debug!("Debug mode enabled");
    log::trace!("Trace information...");

    Ok(DebugInfo { verbose })
}
```

**Step 2: Inspect parsed arguments**

```rust
#[verb]
fn inspect(args: &VerbArgs) -> Result<InspectionResult> {
    let raw_args: Vec<String> = std::env::args().collect();

    Ok(InspectionResult {
        raw_args,
        parsed_args: format!("{:#?}", args),
        env_vars: std::env::vars().collect(),
    })
}
```

**Step 3: Validation errors**

```rust
#[verb]
fn validate(
    #[arg(validator = validate_port)]
    port: u16,
) -> Result<ValidateResult> {
    // Custom validation
    if port < 1024 {
        return Err(NounVerbError::ValidationError(
            format!("Port {} is privileged (< 1024)", port)
        ).into());
    }

    Ok(ValidateResult { port })
}

fn validate_port(s: &str) -> Result<(), String> {
    let port: u16 = s.parse()
        .map_err(|_| format!("Invalid port: {}", s))?;

    if port == 0 {
        return Err("Port cannot be 0".to_string());
    }

    Ok(())
}
```

---

## 9. Performance Tuning

### Problem
Your CLI is slow or uses too much memory.

### Solution

**Step 1: Use streaming for large data**

```rust
use std::io::{BufReader, BufRead};
use std::fs::File;

#[verb]
fn process_large_file(
    #[arg(index = 0, value_name = "FILE")]
    path: String,
) -> Result<ProcessResult> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        process_line(&line)?;
        count += 1;
    }

    Ok(ProcessResult { lines: count })
}
```

**Step 2: Lazy evaluation**

```rust
#[verb]
fn search(
    pattern: String,
    #[arg(index = 0, multiple)]
    files: Vec<String>,

    /// Stop after first match
    #[arg(short = 'm')]
    max_matches: Option<usize>,
) -> Result<SearchResult> {
    let max = max_matches.unwrap_or(usize::MAX);
    let mut matches = Vec::new();

    for file in files {
        if matches.len() >= max {
            break;  // Stop early
        }

        matches.extend(search_file(&file, &pattern)?);
    }

    matches.truncate(max);
    Ok(SearchResult { matches })
}
```

**Step 3: Parallel processing**

```rust
use rayon::prelude::*;

#[verb]
fn parallel_process(
    files: Vec<String>,

    /// Number of threads
    #[arg(short = 'j', default_value = "0")]
    jobs: usize,
) -> Result<ProcessResult> {
    if jobs > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .unwrap();
    }

    let results: Vec<_> = files
        .par_iter()
        .map(|file| process_file(file))
        .collect();

    Ok(ProcessResult {
        total: results.len(),
        success: results.iter().filter(|r| r.is_ok()).count(),
    })
}
```

---

## 10. Advanced Error Handling

### Problem
You need sophisticated error handling with context and recovery.

### Solution

**Step 1: Custom error types**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation failed: {field}: {message}")]
    Validation { field: String, message: String },
}

impl From<AppError> for NounVerbError {
    fn from(err: AppError) -> Self {
        NounVerbError::ExecutionError(err.to_string())
    }
}
```

**Step 2: Error context**

```rust
use anyhow::Context;

#[verb]
fn load_config(
    #[arg(value_name = "FILE")]
    path: String,
) -> Result<Config> {
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path))?;

    Ok(config)
}
```

**Step 3: Graceful degradation**

```rust
#[verb]
fn fetch_with_fallback(
    url: String,

    /// Fallback URL if primary fails
    #[arg(long)]
    fallback: Option<String>,
) -> Result<FetchResult> {
    match fetch_url(&url) {
        Ok(data) => Ok(FetchResult { data, source: "primary".to_string() }),
        Err(e) => {
            if let Some(fallback_url) = fallback {
                log::warn!("Primary URL failed: {}, trying fallback", e);
                let data = fetch_url(&fallback_url)?;
                Ok(FetchResult { data, source: "fallback".to_string() })
            } else {
                Err(e)
            }
        }
    }
}
```

**Step 4: Retry logic**

```rust
use std::time::Duration;
use std::thread::sleep;

#[verb]
fn retry_operation(
    /// Maximum retry attempts
    #[arg(long, default_value = "3")]
    max_retries: usize,

    /// Delay between retries (ms)
    #[arg(long, default_value = "1000")]
    retry_delay: u64,
) -> Result<OperationResult> {
    let mut attempts = 0;

    loop {
        match perform_operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                log::warn!("Attempt {} failed: {}, retrying...", attempts, e);
                sleep(Duration::from_millis(retry_delay));
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

---

## See Also

- [Quick Start Guide](./QUICKSTART.md) - Get started in 10 minutes
- [CLI Reference](./CLI_REFERENCE.md) - Complete API reference
- [Troubleshooting](./CLI_TROUBLESHOOTING.md) - Common issues and solutions
- [Examples](../examples/) - Working code examples

---

**Version:** 4.0.2
**Last Updated:** 2024-11-18
**License:** MIT OR Apache-2.0
