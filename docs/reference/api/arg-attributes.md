# Reference: Argument Attributes (Phase 2 - v5.2.0)

**File**: `clap-noun-verb-macros/src/lib.rs`

## Overview

Arguments are configured using Typer-like doc comment tags. These tags declare argument properties, relationships, and behavior inline with the documentation.

## Relationship Tags (Phase 2 - NEW)

### `[group: name]`

**Purpose**: Make argument part of exclusive group (only one in group allowed)

**Syntax**:
```rust
/// # Arguments
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
```

**Behavior**:
- Only ONE argument in the group can be provided
- Error if multiple group members present
- Works with boolean and option flags

**Example**:
```bash
$ myapp export --json         # OK
$ myapp export --yaml         # OK
$ myapp export --json --yaml  # ERROR: conflicting group
```

---

### `[requires: arg]`

**Purpose**: Declare that this argument requires another argument

**Syntax**:
```rust
/// # Arguments
/// * `format` - Output format
/// * `filename` - Output file [requires: format]
```

**Behavior**:
- Argument can only be used if required arg is also present
- Error if provided without required arg
- Can specify multiple requires via repeated tags

**Example**:
```bash
$ myapp save --format json              # ERROR: needs filename
$ myapp save --filename test.json       # ERROR: needs format
$ myapp save --format json --filename test.json  # OK
```

---

### `[conflicts: arg]`

**Purpose**: Declare that this argument conflicts with another

**Syntax**:
```rust
/// # Arguments
/// * `format` - Output format [conflicts: raw]
/// * `raw` - Raw output mode
```

**Behavior**:
- Arguments cannot be used together
- Error if both provided
- Can specify multiple conflicts

**Example**:
```bash
$ myapp output --format json  # OK
$ myapp output --raw          # OK
$ myapp output --format json --raw  # ERROR: conflicting args
```

---

## Value Configuration Tags

### `[env: VAR]`

**Purpose**: Read argument value from environment variable

**Syntax**:
```rust
/// # Arguments
/// * `log_level` - Log level [env: LOG_LEVEL]
```

**Behavior**:
- CLI argument overrides environment variable
- Environment variable used as default
- Works with Option and non-Option types

**Example**:
```bash
$ LOG_LEVEL=debug myapp run          # Uses env var
$ myapp run --log-level info         # CLI overrides env
$ LOG_LEVEL=debug myapp run --log-level info  # CLI wins
```

---

### `[default: value]`

**Purpose**: Provide default value if argument not supplied

**Syntax**:
```rust
/// # Arguments
/// * `retries` - Retry attempts [default: 3]
/// * `timeout` - Timeout in seconds [default: 30]
```

**Behavior**:
- Used if argument not provided on CLI
- Must be valid for the argument type
- Works with Option and non-Option types

**Example**:
```bash
$ myapp process                  # Uses default: timeout=30
$ myapp process --timeout 60     # CLI overrides default
```

---

### `[value_hint: type]`

**Purpose**: Provide shell completion hints

**Syntax**:
```rust
/// # Arguments
/// * `input` - Input file [value_hint: FilePath]
/// * `url` - API endpoint [value_hint: Url]
```

**Supported Types**:
- `FilePath` - File path completion
- `DirPath` - Directory path completion
- `CommandName` - Executable name completion
- `CommandString` - Full command completion
- `Username` - System username completion
- `Hostname` - Network hostname completion
- `Url` - URL completion
- `EmailAddress` - Email address completion
- `Unknown` - No completion hint

**Example**:
```bash
$ myapp process --input <TAB>  # Suggests files
$ myapp fetch --url <TAB>      # Suggests URLs
```

---

## Display Tags

### `[hide]`

**Purpose**: Hide argument from help output

**Syntax**:
```rust
/// # Arguments
/// * `debug_mode` - Debug output [hide]
/// * `internal_flag` - Internal use only [hide]
```

**Behavior**:
- Argument still works, just not shown in help
- Useful for deprecated or internal arguments
- Can still be discovered via shell completion

**Example**:
```bash
$ myapp cmd --help     # Doesn't show --debug-mode
$ myapp cmd --debug-mode true  # Still works
```

---

### `[help_heading: name]`

**Purpose**: Group argument under heading in help output

**Syntax**:
```rust
/// # Arguments
/// * `input` - Input file [help_heading: Input/Output]
/// * `output` - Output file [help_heading: Input/Output]
/// * `verbose` - Verbose mode [help_heading: Logging]
/// * `log_level` - Log level [help_heading: Logging]
```

**Behavior**:
- Organizes help into sections
- Multiple args with same heading grouped together
- Makes help more readable

**Example Help Output**:
```
Input/Output:
  -i, --input <FILE>      Input file
  -o, --output <FILE>     Output file

Logging:
  -v, --verbose           Verbose output
  --log-level <LEVEL>     Log level [default: info]
```

---

## Behavior Tags

### `[global]`

**Purpose**: Propagate argument to all subcommands

**Syntax**:
```rust
/// # Arguments
/// * `config` - Config file [global]
/// * `verbose` - Verbose mode [global]
```

**Behavior**:
- Argument available on root and all subcommands
- Useful for cross-cutting concerns (config, verbosity)
- Works with parent noun/verb hierarchies

**Example**:
```bash
$ myapp --config prod.toml user create alice  # Config available to user/create
```

---

### `[exclusive]`

**Purpose**: Argument cannot be combined with any other arguments

**Syntax**:
```rust
/// # Arguments
/// * `version_check` - Check for updates [exclusive]
/// * `help` - Show help [exclusive]
```

**Behavior**:
- When provided, no other arguments allowed
- Useful for standalone modes (--version, --help)
- Error if combined with any other arg

**Example**:
```bash
$ myapp --version              # OK
$ myapp --version --verbose    # ERROR: exclusive conflict
```

---

## Complete Example

```rust
/// Process data with advanced configuration
///
/// This command demonstrates all Phase 2 argument tags.
///
/// # Arguments
/// * `input` - Input file path [value_hint: FilePath] [help_heading: Input/Output]
/// * `output` - Output file path [value_hint: FilePath] [help_heading: Input/Output]
/// * `verbose` - Enable verbose output [help_heading: Logging]
/// * `quiet` - Suppress output [conflicts: verbose] [help_heading: Logging]
/// * `log_level` - Log level [env: LOG_LEVEL] [default: info] [help_heading: Logging]
/// * `config` - Config file [value_hint: FilePath] [env: CONFIG_PATH] [global]
/// * `debug_info` - Internal debugging [hide]
/// * `version_check` - Check version [exclusive]
#[verb("process")]
fn process(
    input: Option<String>,
    output: Option<String>,
    verbose: bool,
    quiet: bool,
    log_level: Option<String>,
    config: Option<String>,
    debug_info: bool,
    version_check: bool,
) -> Result<Output> {
    // Implementation
    Ok(Output::default())
}
```

## Tag Combinations

**Valid combinations**:
- `[env: VAR] [default: value]` - Env var with default fallback
- `[group: name] [help_heading: X]` - Group + organization
- `[requires: arg] [env: VAR]` - Dependency + env var
- `[value_hint: type] [env: VAR]` - Completion hint + env var

**Invalid combinations**:
- `[group: X]` + `[exclusive]` - Conflicting constraints
- `[conflicts: arg]` + `[requires: arg]` - Conflicting constraints

---

## Inline `#[arg]` Attribute Parameters Whitelist

When you need extra precision beyond doc-comment tags, you can use the inline `#[arg(...)]` attribute on your handler function parameters. The compiler enforces a strict whitelist of allowed parameters to avoid silent typos:

### Whitelisted Parameters
- `short` - Single-character flag (e.g. `short = 'v'`)
- `default_value` - Fallback string if not provided (e.g. `default_value = "8080"`)
- `env` - Environment variable source (e.g. `env = "PORT"`)
- `multiple` - Boolean indicating if the argument can be repeated
- `value_name` - Target label in help displays
- `aliases`, `alias` - Alternate command names
- `index` - Positional argument index (1-based)
- `positional` - Explicitly mark argument as positional
- `action` - Command action type (e.g. `action = "count"`)
- `group` - Group name constraint
- `requires` - Required argument dependency
- `conflicts_with` - Conflicting argument
- `value_parser` - Custom parser mapping
- `help`, `long_help` - Help message strings
- `display_order` - Numeric sorting order in help outputs
- `exclusive` - Mark argument as exclusive
- `trailing_vararg` - Capture trailing unparsed words
- `allow_negative_numbers` - Allow numbers starting with `-`
- `next_line_help` - Render help text on a new line

### Typo Suggestions (Levenshtein Distance)
If you specify a parameter not present in the whitelist, the macro's compile-time syntax check computes the Levenshtein distance against allowed parameters. If a close match is found (distance <= 3), it raises a compile error with a helpful correction:
```
error: Unknown argument parameter `shrt` in `#[arg]`. Did you mean `short`?
```

---

## Compile-Time Validation & Checking Rules

To guarantee runtime safety and preserve architectural boundaries, `clap-noun-verb` executes a series of strict compile-time checks (Poka-Yoke error-proofing rules) when parsing `#[verb]` macros:

### 1. Forgotten `#[verb]` Detection (unimplemented)
`clap-noun-verb-macros/src/validation.rs` defines `check_verb_registration!()`
as a literal no-op macro, and its generator is never invoked from anywhere in
the macro crate -- there is currently no compile-time check that a
`Result`-returning function is missing `#[verb]`. Treat this as an
unimplemented placeholder, not a real diagnostic, until it's wired up.

### 2. Duplicate Verb Detection
To prevent multiple functions from registering under the exact same noun-verb path, the macro generates a unique `const` identifier based on the noun name and verb name:
```rust
const __VERB_DUPLICATE_CHECK_[noun]_[verb]_[fn_name]: () = ();
```
If two handlers try to register the same noun and verb path, the compiler rejects it with a "duplicate definitions" error.

### 3. Return Type Serializability
All `#[verb]` functions must return a type that implements `serde::Serialize` (e.g., `Result<T>` or `Option<T>` where `T: Serialize`). The proc macro:
- Validates the return type structure recursively at parse-time.
- Emits a const serialization check: `const fn _assert_serialize<T: serde::Serialize>() {}` inside the wrapper to guarantee the type is serializable.

### 4. Syntax Verification
- `#[verb]` inputs must be double-quoted string literals. Common mistakes (such as passing unquoted identifiers like `#[verb(status)]` instead of `#[verb("status")]`) are intercepted with specific hints.

---

## Poka-Yoke Architectural Guards

### FM-1.1: Cyclomatic Complexity Boundary
To prevent business logic from leaking into the CLI layer, the proc-macro calculates the cyclomatic complexity of functions annotated with `#[verb]`.
- **Max Allowed Complexity**: 5
- **Branch Points Counted**: `if`, `match` arms (`=>`), `for` / `while` loops, logical operators (`&&`, `||`), and try operators (`?`).
- **Violation Result**: Compile error advising you to extract the business logic into a pure domain function.

### FM-1.2: CLI Type Contamination Guard
To keep the domain model isolated and avoid circular dependencies, `#[verb]` function parameter types are checked.
- **Forbidden Parameter Types**: `ArgMatches`, `Command`, `VerbContext`, `VerbArgs`, `HandlerInput` (or their qualified paths).
- **Violation Result**: Compile error forcing you to accept simple, CLI-agnostic types (like `String`, `u32`, `bool`, domain structs) instead of raw CLI context structures.

---

## See Also

- `#[verb]` - Parent macro documentation
- Doc Comment Syntax - How to write argument descriptions
- [Error Reference](./errors.md) - MAPE-K and standard error definitions

