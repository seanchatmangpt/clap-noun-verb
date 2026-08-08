# Reference: Error Types and MAPE-K Structured Errors

**File**: `src/error.rs`

## NounVerbError Enum

The primary error type for `clap-noun-verb` operations.

**Definition**:
```rust
#[derive(Error, Debug)]
pub enum NounVerbError {
    /// Command not found
    #[error("Command '{noun}' not found{suggestion}")]
    CommandNotFound { noun: String, suggestion: String },

    /// Verb not found for a given noun
    #[error("Verb '{verb}' not found for noun '{noun}'{suggestion}")]
    VerbNotFound { noun: String, verb: String, suggestion: String },

    /// Invalid command structure
    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    /// Command execution error
    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },

    /// Argument parsing error
    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },

    /// Plugin-related error
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Middleware error
    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    /// Telemetry error
    #[error("Telemetry error: {0}")]
    TelemetryError(String),

    /// Generic error wrapper
    #[error("Error: {0}")]
    Generic(String),
}
```

---

## MAPE-K Structured Errors

For machine-grade orchestration and self-healing systems (MAPE-K control loops), `clap-noun-verb` provides a uniform, machine-readable structured error representation (`StructuredError`).

### StructuredError Struct

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StructuredError {
    pub kind: ErrorKind,
    pub severity: Severity,
    pub message: String,
    pub details: std::collections::HashMap<String, serde_json::Value>,
    pub action_templates: Vec<ActionTemplate>,
}
```

### ErrorKind Enum

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidInput,
    PermissionDenied,
    InvariantBreach,
    DeadlineExceeded,
    GuardExceeded,
    CommandNotFound,
    VerbNotFound,
    ExecutionError,
    InternalError,
}
```

### Severity Enum

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
    Critical,
}
```

### ActionTemplate Enum

Autonomic loops rely on executable recovery templates proposed by the error layer to automatically resolve runtime issues:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ActionTemplate {
    TimeoutAdjustment {
        suggested_timeout_ms: u64,
        reason: String,
    },
    CommandFix {
        suggested_command: String,
        reason: String,
    },
}
```

---

## Mapping NounVerbError to StructuredError

When errors occur during command routing or execution, the framework automatically maps the `NounVerbError` to a `StructuredError` via `StructuredError::from_error(&err)`.

### Mapping Matrix

| NounVerbError Variant | Mapped ErrorKind | Severity | Dynamic Action Templates / Suggestions |
|---|---|---|---|
| `CommandNotFound` | `ErrorKind::CommandNotFound` | `Severity::Error` | Misspelling suggestion mapped to `ActionTemplate::CommandFix` (using Levenshtein distance). |
| `VerbNotFound` | `ErrorKind::VerbNotFound` | `Severity::Error` | Misspelling suggestion mapped to `ActionTemplate::CommandFix` with correct parent noun. |
| `InvalidStructure` | `ErrorKind::InvalidInput` | `Severity::Error` | None |
| `ExecutionError` | `ErrorKind::ExecutionError` or `DeadlineExceeded` | `Severity::Error` or `Critical` | Mapped to `ErrorKind::DeadlineExceeded` with `ActionTemplate::TimeoutAdjustment` if message contains "deadline", "timeout", or "budget exceeded". |
| `ArgumentError` | `ErrorKind::InvalidInput` | `Severity::Error` | None |
| `ValidationFailed` | `ErrorKind::InvariantBreach` | `Severity::Error` | None |
| `PluginError`, `MiddlewareError`, `TelemetryError`, `Generic` | `ErrorKind::InternalError` | `Severity::Error` | None |

### JSON Output Example (Command Spellcheck Recovery)

If a user tries to run a misspelled command:
```bash
$ myapp servise list
```
The resulting structured JSON error response will be:
```json
{
  "kind": "CommandNotFound",
  "severity": "Error",
  "message": "Command 'servise' not found. Did you mean: service?",
  "details": {
    "noun": "servise",
    "suggestion": ". Did you mean: \u001b[1m\u001b[33mservice\u001b[0m?"
  },
  "action_templates": [
    {
      "suggested_command": "service",
      "reason": "Suggested correction for misspelled command 'servise'"
    }
  ]
}
```

---

## Best Practices

1. **Leverage Recovery Suggestions**: Inspect the `action_templates` array in the JSON response of your autonomic control program to execute self-healing steps (e.g. automatically retrying with the suggested command or adjusted timeouts).
2. **Preserve Exit Codes**: Use structured error formatting while maintaining the correct exit status code on the shell to preserve interoperability.
3. **Chicago TDD Testing**: Validate expected error scenarios directly on your domain interfaces as well as the CLI boundaries to prevent parsing or validation regressions.

