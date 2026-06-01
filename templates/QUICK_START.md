# Quick Start: Using clap-noun-verb Templates

This guide shows how to use the templates to scaffold a new CLI project from scratch.

## Create a New Project

```bash
# Create a new Rust project
cargo new --lib my-task-cli
cd my-task-cli

# Create template directory
mkdir -p templates

# Copy templates from clap-noun-verb
cp /path/to/clap-noun-verb/templates/*.jinja templates/
```

## Setup Cargo.toml

```toml
[package]
name = "my-task-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "my-task-cli"
path = "src/main.rs"

[dependencies]
clap-noun-verb = "5.6.0"
clap-noun-verb-macros = "5.6.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
log = "0.4"

[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
```

---

## Example: Task Management CLI

Let's build a simple task CLI with this structure:

```
my-task-cli/
├── src/
│   ├── main.rs              ← main.rs.jinja
│   ├── lib.rs               ← lib.rs.jinja
│   ├── commands/
│   │   ├── mod.rs
│   │   └── tasks.rs         ← noun.rs.jinja
│   ├── domain/
│   │   ├── mod.rs
│   │   └── task.rs
│   ├── integration/
│   │   ├── mod.rs
│   │   └── mod.rs
│   └── outputs/
│       ├── mod.rs
│       └── task.rs
└── templates/
    ├── noun.rs.jinja
    ├── verb.rs.jinja
    ├── lib.rs.jinja
    └── main.rs.jinja
```

---

## Step 1: Create Domain Layer

**File**: `src/domain/task.rs`

```rust
// Pure business logic - no CLI, no I/O
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "archived")]
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl Task {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

#[derive(Debug, Clone)]
pub struct TaskStore {
    tasks: HashMap<String, Task>,
    next_id: usize,
}

impl Default for TaskStore {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }
}

impl TaskStore {
    pub fn create(&mut self, title: String) -> String {
        let id = self.next_id.to_string();
        self.next_id += 1;
        let task = Task::new(id.clone(), title);
        self.tasks.insert(id.clone(), task);
        id
    }

    pub fn list(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    pub fn complete(&mut self, id: &str) -> Option<Task> {
        self.tasks.get_mut(id).map(|task| {
            task.complete();
            task.clone()
        })
    }
}
```

**File**: `src/domain/mod.rs`

```rust
pub mod task;
pub use task::{Task, TaskStatus, TaskStore};
```

---

## Step 2: Create Output Types

**File**: `src/outputs/task.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutput {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListOutput {
    pub tasks: Vec<TaskOutput>,
    pub count: usize,
}
```

**File**: `src/outputs/mod.rs`

```rust
pub mod task;
pub use task::{TaskOutput, TaskListOutput};
```

---

## Step 3: Create CLI Commands (Using verb.rs.jinja)

**File**: `src/commands/tasks.rs` (from `noun.rs.jinja`):

```rust
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! tasks noun - Manage task lists
//!
//! Encapsulates all task domain operations and state.

use crate::domain::TaskStore;
use clap_noun_verb::{NounCommand, Result};
use std::sync::Mutex;
use std::sync::OnceLock;

static TASK_STORE: OnceLock<Mutex<TaskStore>> = OnceLock::new();

pub fn task_store() -> &'static Mutex<TaskStore> {
    TASK_STORE.get_or_init(|| Mutex::new(TaskStore::default()))
}

#[derive(Debug, Clone)]
pub struct TasksCommand;

impl Default for TasksCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TasksCommand {
    pub fn new() -> Self {
        Self
    }
}

// Verb handlers follow below
```

---

## Step 4: Create Verb Handlers (Using verb.rs.jinja)

**File**: `src/commands/create.rs` (excerpt from `verb.rs.jinja`):

```rust
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! create verb - Create a new task

use clap_noun_verb_macros::verb;
use clap_noun_verb::{NounVerbError, Result};
use serde::{Deserialize, Serialize};
use crate::domain::Task;
use crate::outputs::TaskOutput;
use super::task_store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskDomain {
    pub title: String,
}

impl CreateTaskDomain {
    pub fn new(title: String) -> Self {
        Self { title }
    }

    pub fn execute(&self) -> Result<TaskOutput> {
        if self.title.is_empty() {
            return Err(NounVerbError::validation_error(
                "title".to_string(),
                self.title.clone(),
                Some("Title cannot be empty"),
            ));
        }

        let mut store = task_store().lock().unwrap();
        let id = store.create(self.title.clone());
        let task = store.list()
            .into_iter()
            .find(|t| t.id == id)
            .unwrap();

        Ok(TaskOutput {
            id: task.id,
            title: task.title,
            status: format!("{:?}", task.status),
            created_at: task.created_at,
            completed_at: task.completed_at,
        })
    }
}

/// Create a new task
///
/// # Arguments
///
/// * `title` - Task title
///
/// # Examples
///
/// ```bash
/// $ my-task-cli tasks create "Buy groceries"
/// ```
#[verb("tasks", "create")]
pub fn create_task(
    #[arg(index = 1)]
    title: Option<String>,
) -> Result<TaskOutput> {
    let title = title.ok_or_else(|| {
        NounVerbError::validation_error(
            "title".to_string(),
            "<missing>".to_string(),
            Some("Usage: my-task-cli tasks create <title>"),
        )
    })?;

    let domain = CreateTaskDomain::new(title);
    domain.execute()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_with_valid_title() {
        let domain = CreateTaskDomain::new("Buy milk".to_string());
        let result = domain.execute();
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.title, "Buy milk");
        assert_eq!(output.status, "Pending");
    }

    #[test]
    fn test_create_task_rejects_empty_title() {
        let domain = CreateTaskDomain::new(String::new());
        let result = domain.execute();
        assert!(result.is_err());
    }
}
```

**File**: `src/commands/list.rs` (excerpt from `verb.rs.jinja`):

```rust
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! list verb - List all tasks

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;
use crate::outputs::{TaskOutput, TaskListOutput};
use super::task_store;

/// List all tasks
///
/// # Examples
///
/// ```bash
/// $ my-task-cli tasks list
/// ```
#[verb("tasks", "list")]
pub fn list_tasks() -> Result<TaskListOutput> {
    let store = task_store().lock().unwrap();
    let tasks: Vec<_> = store.list()
        .into_iter()
        .map(|task| TaskOutput {
            id: task.id,
            title: task.title,
            status: format!("{:?}", task.status),
            created_at: task.created_at,
            completed_at: task.completed_at,
        })
        .collect();

    let count = tasks.len();
    Ok(TaskListOutput { tasks, count })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tasks_empty() {
        let result = list_tasks();
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.count, 0);
    }
}
```

---

## Step 5: Update lib.rs (Using lib.rs.jinja)

**File**: `src/lib.rs`

```rust
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! my-task-cli - A simple task management CLI
//!
//! Architecture: CLI validates, domain computes, outputs serialize

pub mod domain;
pub mod commands;
pub mod outputs;

use clap_noun_verb::Result;

pub fn run() -> Result<()> {
    clap_noun_verb::run()
}
```

---

## Step 6: Create main.rs (Using main.rs.jinja)

**File**: `src/main.rs`

```rust
// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! my-task-cli - Binary entry point

use my_task_cli::Result;
use std::process;

fn main() {
    match my_task_cli::run() {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
```

---

## Build and Test

```bash
# Format
cargo make format

# Lint
cargo make lint

# Test
cargo make test

# Build
cargo make build

# Run
cargo run -- tasks create "My first task"
cargo run -- tasks list
cargo run -- tasks complete "1"
```

---

## Output Examples

### Create Task

```bash
$ cargo run -- tasks create "Buy groceries"
```

**Output** (JSON):

```json
{
  "id": "1",
  "title": "Buy groceries",
  "status": "Pending",
  "created_at": "2024-01-15T10:30:00Z",
  "completed_at": null
}
```

### List Tasks

```bash
$ cargo run -- tasks list
```

**Output**:

```json
{
  "tasks": [
    {
      "id": "1",
      "title": "Buy groceries",
      "status": "Pending",
      "created_at": "2024-01-15T10:30:00Z",
      "completed_at": null
    }
  ],
  "count": 1
}
```

---

## Key Patterns Demonstrated

✓ **Three-layer architecture** - Domain, integration, CLI  
✓ **Type-safe command registration** - `#[verb]` macros  
✓ **JSON output** - Automatic serialization  
✓ **Error handling** - Structured error types  
✓ **AAA testing** - Arrange, Act, Assert pattern  
✓ **Zero boilerplate** - Auto-discovery of commands  

---

## Next Steps

1. **Add more verbs** - Copy `verb.rs.jinja`, customize for your domain
2. **Add authentication** - Implement in integration layer
3. **Add persistence** - Replace in-memory TaskStore with database
4. **Add middleware** - Logging, metrics, rate limiting
5. **Add completions** - Shell completions for bash/zsh

See `examples/playground/` for a complete reference implementation.

