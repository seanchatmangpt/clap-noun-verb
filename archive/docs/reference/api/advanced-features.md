# Reference: Advanced and Autonomic Features

This page documents advanced and autonomic capabilities newly introduced in the `clap-noun-verb` framework.

---

## 1. Dynamic Shell Completions Subcommands

The framework can dynamically add a standard `completions` subcommand to your CLI, which generates shell autocompletion scripts at runtime.

### Registration

To register the completions subcommand, use the fluent builder option on your `CommandRegistry` or `CliBuilder`:

```rust
use clap_noun_verb::build_cli;

let mut cli = build_cli()
    .with_completions_subcommand(); // Dynamically registers the `completions` noun
```

### CLI Usage

Users of the CLI can generate shell completion scripts by invoking:

```bash
# Explicitly choose the shell (bash, zsh, fish, or powershell)
$ myapp completions bash
$ myapp completions zsh
$ myapp completions fish
$ myapp completions powershell

# Or let the tool auto-detect the active shell environment
$ myapp completions
```

You can also pass the `--shell` flag:
```bash
$ myapp completions --shell fish
```

Under the hood, the generator maps verbs and options defined in the registry to corresponding completion scripts, ensuring the autocomplete schema remains synchronized with the binary.

---

## 2. CLI Parameter Chaining (Step References)

For complex execution workflows (such as multi-step orchestration chains), arguments in a step can dynamically reference output values produced by previous command steps using the `@{step_index.json_path}` notation.

### Syntax

- `@{1.token}`: Extracts the `"token"` field from the JSON output of Step 1.
- `@{2.user.id}`: Extracts the nested path `"id"` under the `"user"` object from the JSON output of Step 2.
- `Bearer @{1.token}`: Embeds the extracted step value inside larger string arguments.

### Example Scenario

If Step 1 (e.g. `myapp session login --username admin`) returns:
```json
{
  "token": "secret-abc-123",
  "user": {
    "id": 42,
    "role": "admin"
  }
}
```

A subsequent step in a chained call can reference it:
```bash
$ myapp user get --id "@{1.user.id}" --auth "Bearer @{1.token}"
```

The preprocessor resolves the paths and rewrites the arguments prior to executing the handler:
```bash
$ myapp user get --id 42 --auth "Bearer secret-abc-123"
```

---

## 3. Stdin Stream Extraction (Stdin Bindings)

Command arguments can bind directly to the standard input (`stdin`) stream, which is useful when piping output between processes or feeding dynamic JSON payloads.

### Stdin Bindings

- **`@-`**: Replaces the argument with the complete raw string read from `stdin`.
- **`@-::json_path`**: Parses `stdin` as a JSON object and extracts a specific nested value using dot notation.

### Example Usage

#### Raw Stdin binding:
```bash
$ echo "hello world" | myapp message send --body "@-"
```
This maps the `--body` argument to `"hello world"`.

#### JSON Stdin path binding:
```bash
$ echo '{"session": {"id": "session-xyz", "expired": false}}' | myapp connection verify --session-id "@-::session.id"
```
This extracts `"session-xyz"` and maps it to `--session-id`.

---

## 4. Introspect Schema Exports for LLMs

To integrate CLI binaries with LLM agents or tool-calling frameworks (e.g. Model Context Protocol, OpenAI tools, or Anthropic tool specifications), the CLI supports a global `--introspect` flag.

### CLI Usage

```bash
$ myapp --introspect
```

### Behavior

When this flag is passed, the CLI halts execution and dumps a list of all registered capabilities formatted as a JSON Schema array representing LLM tools:

```json
[
  {
    "name": "session_login",
    "description": "Login to session",
    "parameters": {
      "type": "object",
      "properties": {
        "username": {
          "type": "string",
          "description": "Username"
        }
      },
      "required": ["username"]
    }
  }
]
```

This output can be piped or ingested directly by AI agents to configure tool-calling definitions dynamically.

---

## 5. Interactive REPL Configurations

For interactive terminal sessions, `clap-noun-verb` offers a built-in interactive REPL shell execution loop, complete with shell-word parsing, command autocompletion, and command history persistence.

### Feature Gate

To use the REPL, ensure the `repl` feature is enabled in your `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = { version = "...", features = ["repl"] }
```

### API Configuration

You can instantiate and configure a REPL helper using the `Repl` struct:

```rust
use clap_noun_verb::{CommandRegistry, Repl};
use std::path::PathBuf;

fn main() -> clap_noun_verb::Result<()> {
    let registry = CommandRegistry::new();
    // Register your nouns and verbs here...

    let repl = Repl::new(registry)
        .with_history_file(PathBuf::from("/Users/user/.myapp_history"));

    // Run the interactive REPL shell execution loop
    repl.run()?;
    Ok(())
}
```

### REPL Commands and Controls

Within the REPL environment:
- **Tab Autocomplete**: Press `<TAB>` to dynamically suggest nouns and verbs registered in the CLI.
- **Help**: Type `help` to list all available commands.
- **History**: Use `<Up>` and `<Down>` arrow keys to navigate past commands.
- **Exit**: Type `exit` or `quit`, or press `CTRL-C` / `CTRL-D` to gracefully exit the REPL shell session.

---

## See Also

- [Error Reference & MAPE-K Loops](./errors.md)
- [API Catalog](../api-catalog.md)
