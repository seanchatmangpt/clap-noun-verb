# Developer Experience (DX) Guide for clap-noun-verb

A comprehensive guide to making clap-noun-verb development **fast, discoverable, and enjoyable**.

---

## Table of Contents

1. [First-Day Experience](#first-day-experience)
2. [Debugging Workflow](#debugging-workflow)
3. [Fast Iteration](#fast-iteration)
4. [IDE Integration](#ide-integration)
5. [Feedback Loops](#feedback-loops)
6. [Discoverability](#discoverability)
7. [Collaboration](#collaboration)
8. [Learning Resources](#learning-resources)
9. [Error Messages](#error-messages)
10. [Automation](#automation)

---

## First-Day Experience

### Prerequisites & Setup (5 minutes)

**What you need:**
- Rust 1.74+ (`rustc --version`)
- Git
- VS Code or JetBrains IDE (optional but recommended)

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version  # Verify: rustc 1.74+
```

### Clone & Build (3 minutes)

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/clap-noun-verb.git
cd clap-noun-verb

# Verify tooling installed
which cargo-make || cargo install cargo-make

# One-command full build & test
cargo make ci

# Expected output:
# ✓ Format check passes
# ✓ Clippy lint passes
# ✓ 94 tests pass in <1 second
# ✓ Docs build without warnings
```

### Run Your First Example (2 minutes)

```bash
# Basic example: noun-verb commands
cargo run --example tutorial_basic -- services status
# Output: {"services":["web-server","database","redis"],"all_running":true}

cargo run --example tutorial_basic -- services logs web-server
# Output: {"service":"web-server","entries":["[2024-01-01 10:00:00] INFO: ...

# Calculator example
cargo run --example core_api -- calc add 5 3
# Output: {"result":8}
```

### Create Your First Verb (5 minutes)

**File:** `src/my_first_command.rs`
```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct GreetOutput {
    message: String,
}

/// Greet a user by name
#[verb("greet")]
fn cmd_greet(
    /// Person to greet
    name: String,
) -> Result<GreetOutput> {
    Ok(GreetOutput {
        message: format!("Hello, {}!", name),
    })
}
```

**File:** `src/main.rs`
```rust
mod my_first_command;

fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
```

**Test it:**
```bash
cargo run -- greet Alice
# Output: {"message":"Hello, Alice!"}

cargo run -- --help
# See your new command listed
```

### Verification Checklist

- [ ] `cargo make test` passes (all 94 tests)
- [ ] `cargo make clippy` passes (zero warnings)
- [ ] `cargo make format-check` passes
- [ ] `cargo run --example tutorial_basic -- services status` works
- [ ] You created and ran your first verb successfully

**Time to productivity: ~20 minutes**

---

## Debugging Workflow

### Understanding the Flow

The core flow you'll debug:
```
#[verb] macro generation
         ↓
linkme distributed slice registration
         ↓
CommandRegistry collects all verbs
         ↓
CliBuilder constructs clap Command tree
         ↓
CommandRouter dispatches to handler
         ↓
Output formatting (JSON by default)
```

### Enable Logging

**For debugging, set the log level:**

```bash
# In development
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run your command
cargo run -- my-noun my-verb --verbose
```

**In code (lib or bin only, NEVER println! in library code):**

```rust
// Recommended: use log crate
use log::{debug, info, warn, error};

fn my_handler() -> Result<Output> {
    debug!("Processing request with args: {:?}", args);
    let result = expensive_operation()?;
    info!("Operation completed: {}", result.len());
    Ok(result)
}
```

**Expected output:**
```
[DEBUG] Processing request with args: Args { x: 5, y: 3 }
[INFO] Operation completed: 8
```

### Using Backtraces

**For panic locations (if they occur):**

```bash
# Get minimal backtrace (shows only your code)
RUST_BACKTRACE=1 cargo run -- my-noun my-verb

# Get full backtrace (includes stdlib and dependencies)
RUST_BACKTRACE=full cargo run -- my-noun my-verb

# Capture to file for analysis
RUST_BACKTRACE=full cargo run -- my-noun my-verb 2>&1 | tee backtrace.log
```

**Note:** The codebase denies `unwrap()`, `expect()`, and `panic!()` in production code via Clippy, so backtraces should be rare. If you see one, file a bug!

### IDE Breakpoints

#### VS Code + rust-analyzer

**File:** `.vscode/launch.json`
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Example",
      "cargo": {
        "args": [
          "build",
          "--example=tutorial_basic",
          "--release"
        ],
        "filter": {
          "name": "tutorial_basic",
          "kind": "bin"
        }
      },
      "args": ["services", "status"],
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    }
  ]
}
```

**Usage:**
1. Click left gutter on a line in `src/` or `examples/`
2. Press F5 to launch debugger
3. Step through code, inspect variables
4. Breakpoints persist across runs

#### JetBrains IDEs (IntelliJ IDEA, CLion, RustRover)

1. **Install plugin:** Search for Rust in IDE settings → Plugins → Install
2. **Set breakpoint:** Click left gutter
3. **Debug:** Right-click `main()` in example → **Debug 'tutorial_basic'**
4. **Variables:** Hover over names or use Variables panel

#### Vim + rust-gdb

**Script:** `scripts/debug.sh`
```bash
#!/bin/bash
set -e

# Compile with debug info
cargo build --example tutorial_basic

# Run under gdb
rust-gdb --ex run --args ./target/debug/examples/tutorial_basic services status
```

**Common gdb commands:**
```
b src/lib.rs:42          # Set breakpoint at line 42
c                         # Continue execution
n                         # Next line (step over)
s                         # Step into
p variable_name           # Print variable
bt                        # Backtrace
```

### Inspect Macro Expansion

**See what the `#[verb]` macro generates:**

```bash
# Install cargo-expand (one-time)
cargo install cargo-expand

# Expand a specific file
cargo expand --example tutorial_basic > expanded.rs

# Expand just one module
cargo expand my_module --example tutorial_basic
```

**Output will show:**
- The generated `linkme::distributed_slice` entry
- Wrapped handler function
- Serialization code

### Print-Style Debugging

**For CLI tools, use structured output:**

```rust
// ✗ Never in library code
println!("Value: {:?}", x);  // Breaks piping, agent parsing

// ✓ Use JSON to stderr for debugging
use serde_json::json;
eprintln!("{}", json!({ "debug": "value", "x": x }));

// ✓ Or use the log crate
log::debug!("Value: {:?}", x);
```

### Test in Isolation

**Run a single test for faster feedback:**

```bash
# Run one test (< 50ms)
cargo test test_verb_with_required_args --quiet

# With output
cargo test test_verb_with_required_args -- --nocapture

# With backtrace
RUST_BACKTRACE=1 cargo test test_verb_with_required_args --quiet
```

---

## Fast Iteration

### Incremental Compilation

**clap-noun-verb achieves 0.66s incremental builds.** To maintain this:

```bash
# Recommended: use `cargo watch` for file-change triggers
cargo install cargo-watch

# Watch for changes and run tests
cargo watch -x "make test"

# Watch and build examples
cargo watch -x "make build-examples"
```

**Setup:** Add to `.vscode/settings.json`
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "search.exclude": { "**/target": true }
}
```

### Hot Reload Patterns

**For CLI tools, use this workflow:**

```bash
# Terminal 1: Watch for changes
cargo watch -x "make test"

# Terminal 2: Keep testing your example
cargo watch -x "run --example tutorial_basic -- services status"
```

**For macro development:**
```bash
# Watch macros and rebuild main crate
cargo watch -w clap-noun-verb-macros/src -x "make test"
```

### Quick Test Cycles

**Tests are <1 second with parallelization:**

```bash
# Quick run (< 1 sec)
cargo make test

# Deterministic single-threaded (debugging flaky tests)
cargo make test-lib-deterministic

# All features (2 sec)
cargo make test-all

# One test + output
cargo test test_name -- --nocapture --exact
```

**Create a test template:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function_does_x_with_input_y() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = my_function(input).expect("should succeed");

        // Assert
        assert_eq!(result.value, expected_value);
    }
}
```

**Test patterns to avoid (slow feedback):**
- `assert!(result.is_ok())` — Test actual behavior, not success
- `panic!()` in tests — Use `?` operator instead
- Sleeps or timeouts — All tests should complete in <100ms

### IDE Test Runner Integration

#### VS Code (CodeLLDB + rust-analyzer)

Tests show as inline **▶ Run | Debug** links above test functions.

```rust
#[test]
fn test_example() {  // ← Click "Run" or "Debug"
    assert_eq!(2 + 2, 4);
}
```

#### JetBrains IDEs

Right-click test name → **Run 'test_name'** or **Debug 'test_name'**

Keyboard shortcuts:
- **Ctrl+Shift+F10** (Windows/Linux) — Run
- **Ctrl+Shift+D** (Windows/Linux) — Debug
- **Cmd+Shift+R** (macOS) — Run
- **Cmd+Shift+D** (macOS) — Debug

### Performance Profiling

**Check compilation time:**

```bash
# Timing each crate's compilation
cargo make check

# Measure incremental rebuild (target: <2 sec)
touch src/lib.rs && cargo make check

# Profile macro expansion time
cargo expand --timing
```

**Benchmark code performance:**

```bash
# Run all benchmarks
cargo make bench

# Run specific benchmark
cargo bench --bench my_benchmark -- --verbose
```

---

## IDE Integration

### VS Code Setup (Recommended)

**Extensions to install:**
```
rust-analyzer (rust-lang.rust-analyzer)
CodeLLDB (vadimcn.vscode-lldb)
Clippy (xvasilak.clippy)
Even Better TOML (tamasfe.even-better-toml)
Rust Test Explorer (swellaby.vscode-rust-test-explorer)
```

**File:** `.vscode/settings.json`
```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.hover.documentation.enable": true,
  "editor.codeActionsOnSave": {
    "source.organizeImports": true
  },
  "search.exclude": {
    "**/target": true,
    "**/.git": true
  }
}
```

**File:** `.vscode/launch.json`
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tutorial Basic",
      "cargo": {
        "args": ["build", "--example", "tutorial_basic"],
        "filter": { "name": "tutorial_basic", "kind": "bin" }
      },
      "args": ["services", "status"],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tests",
      "cargo": {
        "args": ["test", "--lib", "--no-run"],
        "filter": { "kind": "lib" }
      },
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

**File:** `.vscode/tasks.json`
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo make test",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "test"],
      "problemMatcher": ["$rustc"],
      "group": { "kind": "test", "isDefault": true }
    },
    {
      "label": "cargo make clippy",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "clippy"],
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo make format",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "format"]
    }
  ]
}
```

**Keyboard shortcuts:** `.vscode/keybindings.json`
```json
[
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo make test"
  },
  {
    "key": "ctrl+shift+l",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo make clippy"
  },
  {
    "key": "ctrl+shift+f",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo make format"
  }
]
```

**Expected behavior:**
- ✓ Hover over identifier → see type signature + docs
- ✓ Click identifier → jump to definition (Ctrl+Click or F12)
- ✓ Ctrl+Shift+T → run tests (see passing/failing inline)
- ✓ Type function name → auto-complete shows params + docs
- ✓ Ctrl+. → quick fixes (unused variables, add missing imports)

### JetBrains IDEs (IntelliJ IDEA, CLion, RustRover)

**Recommended plugins:**
- Rust (built-in)
- TOML (built-in)
- Makefile Language

**File:** `Settings → Languages & Frameworks → Rust`

```
Toolchain: Select from /home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu
Macros: Enable procedural macro expansion
```

**Run configurations:**

`Run → Edit Configurations → + (Add New)`

1. **Type:** Cargo Command
   - **Name:** Tutorial Basic
   - **Command:** run
   - **Arguments:** --example tutorial_basic -- services status
   - **Click:** Run (Shift+F10)

2. **Type:** Cargo Command
   - **Name:** Tests
   - **Command:** test
   - **Arguments:** --lib
   - **Click:** Debug (Shift+F9)

**Keyboard shortcuts:**
- **Shift+F10** — Run configuration
- **Shift+F9** — Debug configuration
- **Ctrl+Alt+L** — Reformat code
- **Ctrl+Shift+A** — Find action
- Hover identifier → inline docs

### Vim / Neovim Setup

**File:** `~/.config/nvim/init.lua` (or `.vim/vimrc`)

```lua
-- Using vim-plug or packer
Plug 'rust-lang/rust.vim'
Plug 'simrat39/rust-tools.nvim'
Plug 'nvim-lua/plenary.nvim'
Plug 'mfussenegger/nvim-dap'

-- rust-tools config
require('rust-tools').setup({
  tools = {
    inlay_hints = { auto = true }
  },
  dap = {
    adapter = 'lldb'
  }
})

-- Key mappings
vim.keymap.set('n', 'K', '<cmd>RustHoverActions<cr>')
vim.keymap.set('n', '<F12>', '<cmd>RustGoToDefinition<cr>')
vim.keymap.set('n', '<leader>t', '<cmd>RustTest<cr>')
vim.keymap.set('n', '<leader>d', '<cmd>RustDebugable<cr>')
```

**Usage:**
```vim
" Hover over identifier
K

" Jump to definition
<F12>

" Run tests
:RustTest

" Debug
:RustDebugable

" Format on save
:RustFmt
```

---

## Feedback Loops

### Immediate Compiler Feedback

**The compiler is your friend — it provides structured feedback:**

```bash
# Compile-time validation
cargo check

# Real error:
error[E0382]: value used after move
   --> src/my_module.rs:42:14
    |
 42 |   let x = vec![1, 2, 3];
 43 |   let y = x;  // <- x moved here
 44 |   println!("{:?}", x);  // <- error: x moved
    |                    ^ value used after move
```

**How to read compiler errors:**
1. **Location:** `src/my_module.rs:42:14` (file:line:column)
2. **Error code:** `E0382` (searchable on docs.rs)
3. **Context:** Code snippet with caret pointing to problem
4. **Suggestion:** Often includes `help:` with solution

**Fix suggestions are powerful:**
```
help: consider using a reference instead
   |
 43 |   let y = &x;
    |           +
```

### Clear Compiler Errors (Zero Compiler Bugs)

**clap-noun-verb has zero compiler bugs.** All errors come from your code, not the framework.

**Common mistakes:**

```rust
// ✗ Return type not Serialize
#[verb("example")]
fn cmd(x: i32) -> i32 {  // ← Error: not Serialize
    x * 2
}

// ✓ Wrap in Serialize struct
#[derive(Serialize)]
struct Result { value: i32 }

#[verb("example")]
fn cmd(x: i32) -> clap_noun_verb::Result<Result> {
    Ok(Result { value: x * 2 })
}

// ✗ Using unwrap in verb
#[verb("example")]
fn cmd(x: String) -> clap_noun_verb::Result<Output> {
    let parsed = x.parse::<i32>().unwrap();  // ← Clippy error
    Ok(Output { value: parsed })
}

// ✓ Use ? operator
#[verb("example")]
fn cmd(x: String) -> clap_noun_verb::Result<Output> {
    let parsed = x.parse::<i32>()?;  // ← Compiles ✓
    Ok(Output { value: parsed })
}
```

**Clippy catches logic errors:**
```bash
cargo clippy -- -D warnings

warning: this if-else-if chain can be rewritten with matches!
   --> src/my_module.rs:42:5
    |
 42 | /   if x == 1 {
 43 | |       y = 10;
 44 | | } else if x == 2 {
 45 | |       y = 20;
 46 | | }
    | |_^ help: try: `y = matches!(x, 1 => 10, 2 => 20, _ => ...)`
```

### Helpful CLI Output

**All error messages are structured + actionable:**

```bash
$ cargo run -- invalid-noun invalid-verb
error: No command found for: invalid-noun

Available nouns:
  services     Manage services
  users        Manage users

Try: myapp services --help
```

**JSON errors (with `--structured-errors`):**

```bash
$ cargo run -- calc add abc 5 -- --structured-errors
{
  "error": "invalid digit found in string",
  "code": "PARSE_ERROR",
  "context": {
    "arg": "x",
    "provided": "abc",
    "expected_type": "i32"
  },
  "suggestions": [
    "Did you mean: calc add 0 5?",
    "See: docs.rs/clap-noun-verb/latest/reference/errors"
  ]
}
```

### Example-Driven Feedback

**Examples serve as regression tests + documentation:**

```bash
# All examples must compile and run
cargo make build-examples

# Run specific example
cargo run --example tutorial_basic -- services status

# Check example output matches docs
cargo make doc
```

---

## Discoverability

### Finding Code

**Quick navigation strategies:**

```bash
# Find all verbs
grep -r "#\[verb" src/ examples/

# Find definitions of a type
grep -rn "struct MyType" src/

# Find where a trait is implemented
grep -rn "impl MyTrait" src/

# Search with rg (ripgrep) — faster
cargo install ripgrep
rg "CommandRouter" src/
rg "impl.*VerbCommand" --type rust
```

**Better: Use your IDE**

| IDE | Shortcut | Action |
|-----|----------|--------|
| **VS Code** | Ctrl+Shift+O | Go to symbol in file |
| **VS Code** | Ctrl+T | Go to symbol in workspace |
| **VS Code** | Ctrl+F | Find in file |
| **VS Code** | Ctrl+Shift+F | Find in workspace |
| **JetBrains** | Ctrl+F12 | File structure |
| **JetBrains** | Ctrl+N | Go to class/struct |
| **JetBrains** | Ctrl+Shift+F | Find in path |
| **Vim** | `:Telescope live_grep` | Find with fuzzy search |
| **Vim** | `:RustNavigate` | Jump to definition |

### Understanding Architecture

**Module map:**

```
src/
├── lib.rs                    ← Entry point, lists all modules
├── cli/
│   ├── mod.rs               ← run(), entry point, CommandRegistry, ArgMetadata
│   └── ...
├── builder.rs               ← CliBuilder API for constructing CLIs
├── router.rs                ← CommandRouter dispatches to handlers
├── registry.rs              ← CommandRegistry (noun/verb registration)
├── verb.rs                  ← VerbCommand trait
├── noun.rs                  ← NounCommand trait
├── logic/                   ← HandlerInput/HandlerOutput
├── error.rs                 ← NounVerbError, Result<T>
├── format.rs                ← Output formatting
├── graph/                   ← RDF operations
└── federation/              ← Feature-gated federation

clap-noun-verb-macros/src/
├── lib.rs                   ← #[verb], #[arg], #[noun] macros
├── validation.rs            ← Compile-time checks
└── macros/                  ← Frontier feature macros
```

**Flow diagram:**

```
User command: myapp services status
                    ↓
Macro expansion: #[verb("status")] fn cmd_status() {...}
                    ↓
Linkme registration: VERB_REGISTRY entry created
                    ↓
Startup: CommandRegistry collects all entries from VERB_REGISTRY
                    ↓
Build: CliBuilder creates clap::Command tree from registry
                    ↓
Parse: clap parses args
                    ↓
Route: CommandRouter looks up handler by noun+verb
                    ↓
Execute: Handler called with parsed args
                    ↓
Output: Serialize result to JSON
                    ↓
Print: JSON output to stdout
```

### Searching Symbols

**Find where something is defined:**

```bash
# Direct path to key modules
src/cli/mod.rs              # Entry point run()
src/registry.rs             # CommandRegistry definition
src/router.rs               # CommandRouter definition
src/builder.rs              # CliBuilder definition

# Trait implementations
rg "impl CommandRouter"     # See all CommandRouter implementations
rg "impl VerbCommand"       # See all verb command implementations

# Macro source
clap-noun-verb-macros/src/lib.rs  # #[verb], #[noun], #[arg] definitions
```

**VSCode shortcuts:**

1. **Ctrl+P** → Type filename to open
   - `cli/mod.rs` → Jump to registry module
   - `builder.rs` → Jump to CLI builder

2. **Ctrl+Shift+O** → Jump to symbol in current file
   - Type `CommandRegistry` → Jump to struct definition

3. **Ctrl+T** → Jump to symbol in workspace
   - Type `CommandRouter` → Find all definitions + usages

4. **F12** → Go to definition
   - Click on `CommandRegistry::new()` → Jump to impl

### Code Map for Common Tasks

**"I want to add a new verb"**
→ See examples in `examples/tutorial/*.rs`
→ Follow the pattern: `#[verb("name")] fn cmd_name(...) -> Result<T>`

**"I want to customize error messages"**
→ Edit `src/error.rs` (`NounVerbError` enum)
→ Edit `src/format.rs` (error formatting)

**"I want to understand how dispatch works"**
→ Read `src/router.rs` (`CommandRouter::dispatch()`)
→ Read `src/cli/mod.rs` (`run()` entry point)

**"I want to add a new feature"**
→ Check `Cargo.toml` features
→ Add feature flag: `[features] my_feature = []`
→ Conditionally compile: `#[cfg(feature = "my_feature")]`

---

## Collaboration

### Code Review Process

**Before submitting PR:**

```bash
# 1. Format code
cargo make format

# 2. Run all checks
cargo make lint

# 3. Run all tests
cargo make test-all

# 4. Build documentation
cargo make doc

# 5. Verify no clippy warnings
cargo clippy -- -D warnings
```

**PR checklist:**
- [ ] All tests pass (`cargo make test-all`)
- [ ] Format passes (`cargo make format-check`)
- [ ] Clippy passes (zero warnings)
- [ ] New tests added for new code
- [ ] Documentation updated
- [ ] Commit message is descriptive

### Git Workflow

**Branch naming:**
```
claude/*       # Work by Claude AI
feat/*         # New features
fix/*          # Bug fixes
refactor/*     # Refactoring
docs/*         # Documentation
```

**Example:**
```bash
git checkout -b feat/add-custom-formatters
# Make changes, commit, push

git push -u origin feat/add-custom-formatters
# Create PR on GitHub

# After review, merge (never rebase)
git merge feat/add-custom-formatters
```

**Commit message style:**

```
Subject line: Brief one-liner (50 chars max)

Body (if needed):
- Explain what changed
- Explain why it changed
- Link to issue: Closes #123

Note: NEVER use --no-verify or rebase
```

### Asking Questions

**Where to ask:**

| Question Type | Where |
|---------------|-------|
| **API usage** | Issues → Discussions tab |
| **Architecture** | GitHub Issues with `question` label |
| **Macro behavior** | GitHub Issues with `macro` label |
| **Bug report** | GitHub Issues with `bug` label |
| **Feature request** | GitHub Issues with `enhancement` label |

**Good question template:**

```markdown
# Question: How to customize JSON output?

## Context
I'm building a CLI with custom output format for my agents.

## What I tried
```rust
#[verb("my-cmd")]
fn cmd(x: i32) -> Result<MyOutput> {
    Ok(MyOutput { value: x })
}
```

## What I expected
Custom JSON structure: `{"custom": "format"}`

## What I got
Standard structure: `{"value": 5}`

## Relevant code
[Link to playground or gist]

## Related
- See also: #456 (similar feature request)
```

### Pair Programming Setup

**VS Code Live Share (easiest):**

1. **Host:** Ctrl+Shift+P → "Live Share: Start Collaboration Session"
2. **Guest:** Click shared link → joins session
3. **Both:** Can edit same files, see cursors, hear audio
4. **Debugging:** Host can share debug session

**Remote SSH (for full environment):**

```bash
# On host machine, install SSH server
sudo apt-get install openssh-server

# On remote, add to ~/.ssh/config
Host clap-dev
    HostName 192.168.1.100
    User alice
    IdentityFile ~/.ssh/id_rsa

# Connect from anywhere
ssh clap-dev
cd clap-noun-verb
cargo make test
```

**Screen sharing (Zoom, Google Meet):**
- Most lightweight
- Share VS Code window
- Co-pilot uses keyboard shortcuts to control

---

## Learning Resources

### Quick Start (30 minutes)

1. **README.md** (5 min) — Overview + noun-verb concept
2. **CLAUDE.md** (5 min) — Build commands + architecture
3. **examples/tutorial/basic.rs** (10 min) — First working code
4. **Create first verb** (10 min) — Follow "First-Day Experience" section

### Tutorial Series (2 hours)

**docs/tutorial/** (if present, or docs.rs):

1. **Domain Separation** — Learn design patterns
2. **Arguments & Validation** — Type-safe CLI args
3. **Error Handling** — Result<T> and custom errors
4. **Output Formatting** — JSON + custom serialization
5. **Testing** — AAA pattern
6. **Production** — Deployment, monitoring, configuration

**Work through progressively; skip if familiar.**

### Example-Based Learning

```bash
# Tutorial examples (progressive difficulty)
examples/tutorial/
├── basic.rs              # Simplest: noun-verb dispatch
├── services.rs           # Add arguments
├── arguments.rs          # Custom arg types
└── positional.rs         # Positional args

# How-to examples (specific use cases)
examples/howto/
├── validation.rs         # Input validation
├── env_vars.rs          # Environment variables
├── config_file.rs       # Config file parsing
├── completions_demo.rs  # Shell completions
└── deprecation.rs       # Deprecating commands

# Reference examples (API details)
examples/reference/
├── framework.rs         # Full framework API
├── format.rs           # Output formatting
├── context.rs          # Handler context
└── nested.rs           # Nested commands
```

**Learning by modification:**

```bash
# Start with an example
cp examples/tutorial/basic.rs my_experiment.rs

# Modify and test
# Change noun/verb names
# Add new arguments
# Change output format

cargo run --example my_experiment -- ...
```

### Documentation Map

| Topic | Location |
|-------|----------|
| **API Reference** | `docs.rs/clap-noun-verb` |
| **Architecture** | `docs/explanation/README.md` |
| **How-Tos** | `docs/howto/README.md` |
| **Macro API** | `docs/reference/api/verb-macro.md` |
| **Error Handling** | `docs/reference/api/errors.md` |
| **Changelog** | `CHANGELOG.md` |

### Books & External Resources

**Recommended reading order:**

1. **Rust Book** (if new to Rust) — https://doc.rust-lang.org/book/
   - Ch. 7 → Modules (organize code)
   - Ch. 15 → Smart Pointers (references)
   - Ch. 19 → Advanced Features (macros, traits)

2. **Clap Book** — https://docs.rs/clap/latest/clap/
   - Learn underlying CLI framework

3. **Procedural Macros** — https://doc.rust-lang.org/reference/procedural-macros.html
   - Optional: deep dive into how #[verb] works

4. **Serde Documentation** — https://serde.rs/
   - JSON serialization used throughout

---

## Error Messages

### Compiler Error Categories

**1. Type Errors (Most Common)**

```rust
#[verb("example")]
fn cmd(x: i32) -> i32 {  // ← Error: return type must be Serialize
    x * 2
}

// Fix:
use serde::Serialize;

#[derive(Serialize)]
struct Output {
    value: i32,
}

#[verb("example")]
fn cmd(x: i32) -> clap_noun_verb::Result<Output> {
    Ok(Output { value: x * 2 })
}
```

**2. Borrow Checker Errors**

```rust
#[verb("example")]
fn cmd(s: String) -> clap_noun_verb::Result<Output> {
    let len = s.len();
    drop(s);  // ← s moved here
    println!("{}", s);  // ← Error: s used after move
    Ok(Output { length: len })
}

// Fix:
#[verb("example")]
fn cmd(s: String) -> clap_noun_verb::Result<Output> {
    let len = s.len();
    Ok(Output { length: len, value: s })  // ← Use s before drop
}
```

**3. Trait Bound Errors**

```rust
#[verb("example")]
fn cmd(x: MyCustomType) -> clap_noun_verb::Result<Output> {
    Ok(Output { value: x })  // ← Error: x doesn't implement Clone/Debug
}

// Fix:
#[derive(Clone, Debug, Serialize)]
struct MyCustomType { ... }
```

**4. Macro Expansion Errors**

```rust
#[verb("example")]
async fn cmd(x: i32) -> clap_noun_verb::Result<Output> {  // ← Error: async not supported in #[verb]
    Ok(Output { value: x })
}

// Fix: Use async_verb feature
#[clap_noun_verb::async_verb("example")]
async fn cmd(x: i32) -> clap_noun_verb::Result<Output> {
    Ok(Output { value: x })
}
```

### Runtime Error Patterns

**Pattern: User passed invalid argument**

```bash
$ cargo run -- calc add abc 5
error: invalid digit found in string

Expected:
  calc add <INTEGER> <INTEGER>

Try:
  calc --help
```

**How to read:**
1. **Error description:** "invalid digit found in string"
2. **Expected format:** Show what was expected
3. **Next step:** Suggest --help

**Pattern: Parse failure with recovery**

```bash
$ cargo run -- users create --age 999  # Out of range for u8
error: value validation failed for 'age'
       expected: 0-255
       got: 999

Did you mean: users create --age 255?
```

### Actionable Error Messages

**All errors in clap-noun-verb follow this pattern:**

```
<Error Code>: <Human Description>

Context:
  - What failed
  - Why it failed
  - What was expected

Fix:
  - Exact change to make
  - Link to docs if needed
```

**Example:**

```
INVALID_ARG: argument 'email' failed validation

Context:
  Expected valid email address
  Got: "user@example"
  Missing: domain

Fix:
  Change to: users create --email user@example.com
  See: https://docs.rs/clap-noun-verb/latest/validators/email
```

---

## Automation

### Pre-Commit Hooks

**Install hooks (one-time):**

```bash
./scripts/setup-hooks.sh

# Verifies:
# - Format check
# - Clippy lint
# - Fast tests (< 1 sec)
```

**What it prevents:**

```bash
# Prevents committing:
git commit -m "My change"
✓ Checks formatting
✓ Runs clippy
✓ Runs tests
✗ COMMIT FAILED: formatter needed

# Fix and retry:
cargo make format
git add .
git commit -m "My change"
✓ All checks pass
✓ Commit accepted
```

### Makefile Tasks

**See all available tasks:**

```bash
cargo make --list-all-steps

# Output shows 50+ automation tasks
```

**Common workflows:**

```bash
# Daily development
cargo make test          # Run tests
cargo make clippy        # Lint
cargo make format        # Auto-format

# Before PR
cargo make lint          # All checks
cargo make test-all      # All features

# Before release
cargo make ci            # Full CI
cargo make doc           # Generate docs
```

### One-Command Workflows

**Create scripts for common tasks:**

**File:** `scripts/dev-loop.sh`
```bash
#!/bin/bash
# Watch for changes and run tests + clippy in loop

cargo watch \
  -x "make format-check" \
  -x "make clippy" \
  -x "make test"
```

**File:** `scripts/build-demo.sh`
```bash
#!/bin/bash
# Build and run the tutorial example

set -e
cargo make build-examples
cargo run --example tutorial_basic -- services status
cargo run --example tutorial_basic -- services logs web-server
cargo run --example tutorial_basic -- services restart database
```

**File:** `scripts/pr-ready.sh`
```bash
#!/bin/bash
# Full PR validation

set -e

echo "Running format check..."
cargo make format-check

echo "Running clippy..."
cargo make clippy

echo "Running tests..."
cargo make test-all

echo "Building docs..."
cargo make doc

echo "✓ PR ready to submit!"
```

**Usage:**

```bash
chmod +x scripts/*.sh

# Run anytime
./scripts/dev-loop.sh &        # Background
./scripts/pr-ready.sh          # Before PR
./scripts/build-demo.sh        # Demo mode
```

### GitHub Actions Workflow

**Already configured in `.github/workflows/ci.yml`**

Triggers on:
- Push to main
- Pull requests
- Scheduled nightly

Checks:
- ✓ Format
- ✓ Clippy
- ✓ Tests (all features)
- ✓ Documentation build
- ✓ Binary size

### Dependency Updates

**Check for outdated dependencies:**

```bash
cargo update
cargo tree --duplicates

# Run tests after updating
cargo make test-all
```

### Performance Monitoring

**Track compilation time:**

```bash
# Create baseline (first time)
cargo make build > baseline.txt

# After making changes
cargo make build > current.txt

# Compare
diff baseline.txt current.txt
```

**Monitor test duration:**

```bash
# Should be < 1 second with parallelization
time cargo make test

# Expected output:
# real 0m0.847s
# user 0m3.241s
# sys  0m0.324s
```

**SLO targets:**
- Incremental compilation: ≤ 2s
- Full test suite: ≤ 1s
- Binary size: ≤ 10MB
- Doc build: ≤ 30s

---

## Glossary

| Term | Meaning |
|------|---------|
| **Noun** | A resource or entity (user, service, config) |
| **Verb** | An action on that noun (create, list, delete, status) |
| **Handler** | Function decorated with `#[verb]` |
| **Distributed slice** | Compile-time auto-discovery via `linkme` crate |
| **Registry** | `CommandRegistry` collects all verbs at startup |
| **Router** | `CommandRouter` dispatches parsed args to handlers |
| **Macro expansion** | Generated code from `#[verb]` attribute |
| **Result<T>** | `std::result::Result<T, NounVerbError>` |
| **Serialize** | `serde::Serialize` trait for JSON output |
| **Linkme** | Linker-based distributed slices for auto-discovery |

---

## Troubleshooting

### Common Problems

**Problem: `cargo make` not found**

```bash
# Install cargo-make
cargo install cargo-make

# Verify
cargo make --version
```

**Problem: Tests hang or timeout**

```bash
# Run single-threaded for determinism
RUST_TEST_THREADS=1 cargo test --lib

# Check for infinite loops or deadlocks
cargo test -- --nocapture --test-threads=1
```

**Problem: Macro expansion not working**

```bash
# Ensure macro crate is published first
cargo build -p clap-noun-verb-macros
cargo build -p clap-noun-verb

# Or use path dependencies (for development)
# Cargo.toml already has: clap-noun-verb-macros = { path = "clap-noun-verb-macros" }
```

**Problem: "Value used after move" in verb handler**

```rust
// ✗ Wrong
#[verb("example")]
fn cmd(data: Vec<String>) -> Result<Output> {
    let len = data.len();
    println!("{:?}", data);  // ← Moved already
    Ok(Output { len })
}

// ✓ Right
#[verb("example")]
fn cmd(data: Vec<String>) -> Result<Output> {
    let len = data.len();
    Ok(Output { len, data })  // ← Return or explicitly use
}
```

---

## Quick Reference Card

### Essential Commands

```bash
# Format + Lint + Test (your daily workflow)
cargo make format && cargo make clippy && cargo make test

# Before submitting PR
cargo make ci

# Run specific example
cargo run --example tutorial_basic -- services status

# Debug with backtrace
RUST_BACKTRACE=1 cargo run -- my-command

# Expand macros to see generated code
cargo expand --example tutorial_basic > expanded.rs

# Run single test
cargo test test_name --quiet

# Watch for changes
cargo watch -x "make test"
```

### File Locations

```
src/cli/mod.rs              # Entry point run()
src/router.rs               # Dispatch logic
src/registry.rs             # Command registry
clap-noun-verb-macros/src/  # Macro definitions
examples/tutorial/          # Learning examples
examples/howto/             # Use case examples
tests/                      # Integration tests
Makefile.toml               # Build tasks
CLAUDE.md                   # Architecture guide
CHANGELOG.md                # Version history
```

### IDE Shortcuts

| Action | VS Code | JetBrains | Vim |
|--------|---------|-----------|-----|
| Go to definition | F12 | Ctrl+B | gd |
| Format code | Shift+Alt+F | Ctrl+Alt+L | :RustFmt |
| Run tests | Ctrl+Shift+T | Shift+F10 | :RustTest |
| Find in workspace | Ctrl+Shift+F | Ctrl+Shift+F | /pattern |
| Quick fix | Ctrl+. | Alt+Enter | :RustFix |

---

## Final Checklist

After reading this guide, you should be able to:

- [ ] Set up development environment (5 min)
- [ ] Run examples (`cargo run --example tutorial_basic`)
- [ ] Write your first verb (10 min)
- [ ] Run tests locally (`cargo make test`)
- [ ] Use debugger with breakpoints
- [ ] Search and navigate codebase in IDE
- [ ] Understand the flow: macro → registry → router → output
- [ ] Create effective error messages
- [ ] Collaborate via PRs with confidence
- [ ] Automate your workflow with Make tasks

**Time to productivity: ~1 hour**

---

## Support

- **Bug reports:** https://github.com/seanchatmangpt/clap-noun-verb/issues
- **Documentation:** https://docs.rs/clap-noun-verb
- **Examples:** `examples/` directory
- **Architecture:** `CLAUDE.md` and `docs/`

---

**Happy developing!**
