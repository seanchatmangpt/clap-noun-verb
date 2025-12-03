# Reference: CLI Runner

**File**: `src/cli/mod.rs`

## run() Function

Execute the CLI application with auto-discovered commands.

**Signature**:
```rust
pub fn run() -> Result<()>
```

**Description**:
Starts the CLI with all commands registered via `#[noun]` and `#[verb]` macros. This is the typical entry point for `main()`.

**Usage**:
```rust
fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

**What It Does**:
1. Discovers all commands (linkme distributed slice)
2. Parses CLI arguments via clap
3. Routes to appropriate handler
4. Serializes output to JSON
5. Returns result or error

**Example Application**:
```rust
// src/main.rs
use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};

#[noun("user", "User management")]
#[verb("list")]
fn list_users() -> Result<Vec<String>> {
    Ok(vec!["alice".to_string(), "bob".to_string()])
}

#[noun("user", "User management")]
#[verb("create")]
fn create_user(name: String) -> Result<String> {
    Ok(format!("Created user: {}", name))
}

fn main() -> Result<()> {
    clap_noun_verb::run()  // Auto-discover and run
}
```

**CLI Usage**:
```bash
$ myapp user list
["alice", "bob"]

$ myapp user create charlie
"Created user: charlie"

$ myapp --help
COMMANDS:
  user      User management
  help      Print this message
```

---

## build() Function

Build a clap Command without executing it.

**Signature**:
```rust
pub fn build() -> clap::Command
```

**Description**:
Returns the clap Command builder for advanced customization.

**Usage**:
```rust
use clap_noun_verb::build;

fn main() {
    let mut app = build();
    // Customize before execution
    let matches = app.get_matches();
}
```

**Advanced Example** (Custom error handling):
```rust
fn main() -> Result<()> {
    let app = clap_noun_verb::build();
    let matches = app.try_get_matches();

    match matches {
        Ok(m) => {
            // Handle matches
            clap_noun_verb::dispatch(&m)
        }
        Err(e) => {
            // Custom error handling
            eprintln!("Error: {}", e);
            Err(CliError::Custom(e.to_string()).into())
        }
    }
}
```

---

## Command Registry

Central registry of all available commands.

**Signature**:
```rust
pub struct CommandRegistry {
    commands: HashMap<String, CommandDefinition>,
}

impl CommandRegistry {
    pub fn get(name: &str) -> Option<&'static CommandRegistry>;
    pub fn list_all() -> Vec<&'static CommandDefinition>;
    pub fn find_by_verb(verb: &str) -> Vec<&'static CommandDefinition>;
    pub fn verb_count() -> usize;
}
```

**Usage**:
```rust
// Get all commands
let all_commands = CommandRegistry::list_all();
println!("Available commands: {}", all_commands.len());

// Find specific command
let user_cmds = CommandRegistry::find_by_verb("create");
for cmd in user_cmds {
    println!("{} {}", cmd.noun, cmd.verb);
}
```

**Auto-Population**:
The registry is populated at compile time via the `#[noun]` and `#[verb]` macros using linkme distributed slices.

---

## CommandDefinition

Individual command specification.

**Signature**:
```rust
pub struct CommandDefinition {
    pub noun: String,
    pub verb: String,
    pub about: String,
    pub args: Vec<ArgMetadata>,
    pub handler: fn(HandlerInput) -> Result<HandlerOutput>,
}

impl CommandDefinition {
    pub fn full_name(&self) -> String;
    pub fn get_arg(&self, name: &str) -> Option<&ArgMetadata>;
}
```

**Fields**:
- `noun` - Parent command name (e.g., "user")
- `verb` - Subcommand name (e.g., "create")
- `about` - Help description
- `args` - List of arguments
- `handler` - Function to execute

**Usage**:
```rust
for cmd in CommandRegistry::list_all() {
    println!("$ myapp {} {}", cmd.noun, cmd.verb);
    println!("  {}", cmd.about);
    for arg in &cmd.args {
        println!("  - {}: {}", arg.name, arg.help.as_deref().unwrap_or(""));
    }
}
```

---

## Argument Metadata

Details about individual arguments.

**Signature**:
```rust
pub struct ArgMetadata {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub value_name: Option<String>,
    pub help: Option<String>,
    pub group: Option<String>,
    pub requires: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub is_flag: bool,
    pub is_global: bool,
}

impl ArgMetadata {
    pub fn full_name(&self) -> String;
    pub fn is_required(&self) -> bool;
}
```

**Usage**:
```rust
let cmd = CommandRegistry::list_all()[0];
for arg in &cmd.args {
    let req = if arg.is_required() { "required" } else { "optional" };
    println!("{} ({})", arg.name, req);

    if !arg.requires.is_empty() {
        println!("  Requires: {:?}", arg.requires);
    }
    if !arg.conflicts_with.is_empty() {
        println!("  Conflicts: {:?}", arg.conflicts_with);
    }
}
```

---

## Execution Flow

**Step-by-step execution**:

```
┌─────────────────────────────┐
│  run() called from main()   │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│ Build clap Command from     │
│ CommandRegistry             │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│ Parse CLI arguments         │
│ (clap::ArgMatches)          │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│ Dispatch to handler         │
│ (HandlerInput → handler)    │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│ Serialize output to JSON    │
│ (Serialize trait)           │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│ Return Result               │
│ (success or error)          │
└─────────────────────────────┘
```

---

## Custom Integration

**Extending the CLI**:

```rust
use clap_noun_verb::{build, dispatch, CommandRegistry};

fn main() -> Result<()> {
    let mut app = build();

    // Add custom global flags
    app = app
        .arg(clap::Arg::new("config")
            .global(true)
            .help("Custom config file"));

    // Parse with custom error handling
    let matches = app.try_get_matches()?;

    // Access custom args
    if let Some(config) = matches.get_one::<String>("config") {
        println!("Using config: {}", config);
    }

    // Dispatch to registered commands
    dispatch(&matches)
}
```

---

## Help Output Structure

Generated help automatically follows this structure:

```
myapp 5.2.0
A high-level, ergonomic API for building noun-verb CLI patterns

USAGE:
    myapp [OPTIONS] <COMMAND>

COMMANDS:
    database    Database operations
    user        User management
    config      Configuration
    help        Print this message or the help of the given subcommand(s)

OPTIONS:
    -h, --help                 Print help information
    -V, --version              Print version
```

Subcommand help:

```
myapp-user
User management commands

USAGE:
    myapp user [COMMAND]

COMMANDS:
    create    Create new user
    delete    Delete user
    list      List all users
    update    Update user
    help      Print this message
```

---

## Version Management

Print version info:

```bash
$ myapp --version
myapp 5.2.0

$ myapp user create --help | head -1
myapp-user (v5.2.0)
```

---

## Exit Codes

Standard exit code mapping:

- `0` - Success
- `1` - General error
- `2` - Usage error (invalid arguments)
- Custom codes via `HandlerOutput::status_code`

---

## See Also

- run() - Primary entry point
- CommandRegistry - Command discovery
- build() - Advanced customization
- HandlerInput - Argument access
- dispatch() - Route to handler
