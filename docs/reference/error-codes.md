# Reference: Error Codes

**Source**: `src/error.rs`
**Version**: 26.8.22

`clap-noun-verb` has two complementary error representations:

1. **`NounVerbError`** — the human-facing `Result<T>` error type returned throughout the
   framework (`pub type Result<T> = std::result::Result<T, NounVerbError>`).
2. **`StructuredError`** — a machine-readable, uniform JSON format emitted when a CLI is
   run with `--structured-errors`, designed for autonomic MAPE-K loops and agent consumers.

---

## NounVerbError

The core error enum. Each variant has a stable `Display` format (via `thiserror`).

| Variant | Fields | `Display` format |
|---------|--------|------------------|
| `CommandNotFound` | `noun`, `suggestion` | `Command '{noun}' not found{suggestion}` |
| `VerbNotFound` | `noun`, `verb`, `suggestion` | `Verb '{verb}' not found for noun '{noun}'{suggestion}` |
| `InvalidStructure` | `message` | `Invalid command structure: {message}` |
| `ExecutionError` | `message` | `Command execution failed: {message}` |
| `ArgumentError` | `message` | `Argument parsing failed: {message}` |
| `PluginError` | `String` | `Plugin error: {0}` |
| `ValidationFailed` | `String` | `Validation failed: {0}` |
| `MiddlewareError` | `String` | `Middleware error: {0}` |
| `TelemetryError` | `String` | `Telemetry error: {0}` |
| `Generic` | `String` | `Error: {0}` |

`std::io::Error` converts into `NounVerbError::ExecutionError` via `From`.

### Constructors

Prefer the constructor helpers over building variants directly:

```rust
use clap_noun_verb::NounVerbError;

NounVerbError::command_not_found("usr");
NounVerbError::verb_not_found("user", "lst");
NounVerbError::invalid_structure("missing subcommand");
NounVerbError::execution_error("downstream call failed");
NounVerbError::argument_error("--port expects an integer");
NounVerbError::missing_argument("name");        // → ArgumentError "Required argument 'name' is missing"
```

### "Did you mean?" suggestions

`CommandNotFound` and `VerbNotFound` can carry a suggestion built from a Levenshtein
search over known candidates (distance ≤ 3, shorter than the input):

```rust
let candidates = ["user", "session", "config"];
let err = NounVerbError::command_not_found_with_candidates("usr", &candidates);
// Display: Command 'usr' not found. Did you mean: user?
```

The same exists for verbs via `verb_not_found_with_candidates(noun, verb, &candidates)`.

### Validation helpers

```rust
NounVerbError::validation_error("port", "abc", Some("Must be a number"));
NounVerbError::validation_range_error("port", "70000", Some("1"), Some("65535"));
NounVerbError::validation_length_error("name", "", Some(1), Some(64));
```

All three produce an `ArgumentError` with a formatted constraint message.

---

## StructuredError (`--structured-errors`)

When the global `--structured-errors` flag is set, errors are emitted as a uniform JSON
object instead of a plain message. Shape (`src/error.rs`):

```jsonc
{
  "kind": "InvalidInput",          // ErrorKind
  "severity": "Error",             // Severity
  "message": "Argument parsing failed: --port expects an integer",
  "details": { "message": "..." }, // map<string, json>
  "action_templates": [ /* recovery suggestions */ ]
}
```

### `ErrorKind`

`InvalidInput`, `PermissionDenied`, `InvariantBreach`, `DeadlineExceeded`,
`GuardExceeded`, `CommandNotFound`, `VerbNotFound`, `ExecutionError`, `InternalError`.

### `Severity`

`Warning`, `Error`, `Critical`.

### `ActionTemplate`

Machine-actionable recovery hints (serialized untagged):

- `TimeoutAdjustment { suggested_timeout_ms, reason }`
- `CommandFix { suggested_command, reason }`

### Mapping: `NounVerbError` → `StructuredError`

`StructuredError::from_error(&err)` performs this mapping:

| `NounVerbError` | `kind` | `severity` | Notes |
|-----------------|--------|------------|-------|
| `CommandNotFound` | `CommandNotFound` | Error | Adds `CommandFix` template from the suggestion, if any |
| `VerbNotFound` | `VerbNotFound` | Error | Adds `CommandFix` (`"{noun} {verb}"`) from the suggestion |
| `InvalidStructure` | `InvalidInput` | Error | |
| `ArgumentError` | `InvalidInput` | Error | |
| `ExecutionError` (msg contains "deadline"/"timeout"/"budget exceeded") | `DeadlineExceeded` | **Critical** | Adds `TimeoutAdjustment` template |
| `ExecutionError` (other) | `ExecutionError` | Error | |
| `ValidationFailed` | `InvariantBreach` | Error | |
| `PluginError` | `InternalError` | Error | |
| `MiddlewareError` | `InternalError` | Error | |
| `TelemetryError` | `InternalError` | Error | |
| `Generic` | `InternalError` | Error | |

### Deadline-specific constructor

```rust
use clap_noun_verb::StructuredError;

let err = StructuredError::deadline_exceeded(500, 640);
// kind: DeadlineExceeded, severity: Critical,
// action_templates: [TimeoutAdjustment { suggested_timeout_ms: 740, .. }]
```

---

## See Also

- [Errors API](api/errors.md) — type-level reference for `NounVerbError` / `StructuredError`
- [Schema Validation](schema-validation.md) — `--introspect` tool schema
- [Advanced Features](api/advanced-features.md) — `--structured-errors` usage in context
