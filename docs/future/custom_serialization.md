# Custom Serialization and Deserialization Patterns

**Status:** Proposed for v6.0 (Phase 2)  
**Timeline:** 2026-06-16 – 2026-06-29

This document provides a comprehensive reference and guide to the custom serialization, deserialization, parsing, and formatting patterns utilized across the `clap-noun-verb` framework and the companion `clap-noun-verb-utils` utility library.

---

## 1. Complex CLI Argument Parsing & Validation

Parsing complex command-line arguments into strongly-typed Rust structures involves combining custom validator functions, `clap`'s `value_parser` mechanisms, and string parsers.

### 1.1 Range-Bounded Parsing (`clap-num` Wrappers)
To restrict integer inputs to custom bounds at the command-line parsing phase, `clap-noun-verb-utils` provides range-bounded helpers. Any closure of type `Fn(&str) -> Result<T, String>` seamlessly integrates with `clap::Arg::value_parser`.

```rust
use clap_noun_verb_utils::number_parsing::{decimal_range, maybe_hex_range};
use clap::{Arg, Command};

let cmd = Command::new("app")
    .arg(
        Arg::new("port")
            .long("port")
            .value_parser(decimal_range(1024u16, 65535u16))
    )
    .arg(
        Arg::new("mask")
            .long("mask")
            .value_parser(maybe_hex_range(0x00u8, 0xffu8))
    );
```

- **`decimal_range<T>(min: T, max: T)`**: Validates that the input represents a decimal integer within the closed interval `[min, max]`. It detects overflows, invalid characters, and empty inputs.
- **`maybe_hex_range<T>(min: T, max: T)`**: Validates decimals or hexadecimals (prefixed with `0x` or `0X`) within the closed interval `[min, max]`.

### 1.2 Custom String Format Parsers
For advanced domain-specific types, custom string parser functions map raw CLI inputs into semantic types:

| Custom Parser | Input Format | Output Type | Validation Bounds / Behavior |
|---|---|---|---|
| `parse_percentage` | `"50%"`, `"12.5%"` | `f64` (in range `[0.0, 1.0]`) | Must end with `%`; values must be within `[0.0, 100.0]`. |
| `parse_bytes` | `"10kb"`, `"5MB"`, `"2g"`, `"10"` | `u64` (representing bytes) | Evaluates units (`b`, `kb`, `mb`, `gb`, `tb`) case-insensitively. Protects against multiplication overflows. |
| `parse_duration` | `"30s"`, `"1h 15m"`, `"1d 2h"` | `std::time::Duration` | Evaluates tokens with units (`s`, `m`, `h`, `d`). Protects against overflow. |

#### Implementation Pattern Example
```rust
use clap_noun_verb_utils::number_parsing::parse_bytes;
use clap::{Arg, Command};

let matches = Command::new("storage")
    .arg(
        Arg::new("limit")
            .long("limit")
            .value_parser(clap::builder::ValueParser::new(parse_bytes))
    )
    .get_matches_from(vec!["storage", "--limit", "10gb"]);

let limit_bytes: &u64 = matches.get_one::<u64>("limit").unwrap();
assert_eq!(*limit_bytes, 10_737_418_240);
```

### 1.3 Flattening and Normalization (Dot Notation / Environment Variables)
For nested configurations, `clap-noun-verb` utilizes layered adapters that map command line arguments and environment variables to structured JSON objects before deserializing.
- **Environment Variables**: Variables utilizing double underscores (e.g., `APP__DATABASE__PORT`) are converted into nested structures (`{"app": {"database": {"port": ...}}}`) by replacing `__` with object level delimiters.
- **Dot Notation Arguments**: Command line overrides containing dots (e.g., `--set database.port=5432`) are parsed and normalized via map-merging logic into a cohesive JSON value before being deserialized into a configuration struct via `serde::Deserialize`.

---

## 2. JSON Outputs & Formatting Engine

Once a command executes successfully, the output must be serialized and presented in the user's requested style.

### 2.1 The `OutputFormat` Engine
Output rendering is managed by the `OutputFormat` enum:

```rust
pub enum OutputFormat {
    Json,        // Compact machine-readable JSON
    JsonPretty,  // Human-readable pretty-printed JSON (Default)
    Yaml,        // YAML-like presentation (built-in, dependency-free)
    Table,       // ASCII table representation for list/array outputs
    Plain,       // Plain text key-value format
    Tsv,         // Tab-Separated Values for tabular automation
    Quiet,       // Suppresses all stdout rendering (returns empty string)
}
```

The system automatically serializes any type implementing `serde::Serialize` into the chosen format.

### 2.2 Output Validation Hooks
For security compliance, schema enforcement, or limit checking, validation hooks can check the serialized output before it is printed:

```rust
use clap_noun_verb::format::{register_output_validation_hook, OutputFormat};

// Enforce that output JSON never contains sensitive API keys or exceeds maximum depth
register_output_validation_hook(|json_val| {
    if let Some(obj) = json_val.as_object() {
        if obj.contains_key("api_key") {
            return Err("Security Violation: Serialized output contains 'api_key'".into());
        }
    }
    Ok(())
});
```

### 2.3 Deep Serialization Bounds & Mitigation
When serializing recursive or deeply nested commands and subcommands:
- **Serde Recursion Limit**: Extremely deep hierarchies (exceeding 128 levels) can fail to deserialize using default `serde_json` parameters.
- **Mitigation Pattern**: Clients deserializing serialized command schemas should disable or increase the recursion limit:
  ```rust
  let mut deserializer = serde_json::Deserializer::from_str(&json_data);
  deserializer.disable_recursion_limit();
  let schema = CommandSchema::deserialize(&mut deserializer)?;
  ```

---

## 3. Custom Error Formats

`clap-noun-verb` separates user-facing console errors from machine-readable autonomic loop formats.

### 3.1 NounVerbError (Human-Facing)
The `NounVerbError` enum represents typical runtime CLI errors, formatted with colors and user suggestions (e.g. Levenshtein spelling suggestions):

```
Command 'servise' not found. Did you mean: service?
```

### 3.2 StructuredError (Machine-Facing / MAPE-K Control Loops)
For integration into autonomic self-healing systems (MAPE-K loops), `clap-noun-verb` converts standard errors into structured `StructuredError` JSON outputs.

```rust
pub struct StructuredError {
    pub kind: ErrorKind,
    pub severity: Severity,
    pub message: String,
    pub details: std::collections::HashMap<String, serde_json::Value>,
    pub action_templates: Vec<ActionTemplate>,
}
```

#### Error Mapping Matrix

| NounVerbError Variant | Mapped ErrorKind | Severity | Dynamic Action Templates / Recovery Suggestions |
|---|---|---|---|
| `CommandNotFound` | `ErrorKind::CommandNotFound` | `Severity::Error` | Dynamic `ActionTemplate::CommandFix` containing spelling suggestions. |
| `VerbNotFound` | `ErrorKind::VerbNotFound` | `Severity::Error` | Dynamic `ActionTemplate::CommandFix` containing correct verb candidate. |
| `ExecutionError` (Timeout) | `ErrorKind::DeadlineExceeded` | `Severity::Critical` | `ActionTemplate::TimeoutAdjustment` suggesting a new deadline budget. |
| `ValidationFailed` | `ErrorKind::InvariantBreach` | `Severity::Error` | None. |
| `ArgumentError` | `ErrorKind::InvalidInput` | `Severity::Error` | None. |
| Internal errors | `ErrorKind::InternalError` | `Severity::Error` | None. |

### 3.3 Example JSON Representation
When a command spellcheck correction is found, the CLI outputs a machine-readable structured error:

```json
{
  "kind": "CommandNotFound",
  "severity": "Error",
  "message": "Command 'servise' not found. Did you mean: service?",
  "details": {
    "noun": "servise",
    "suggestion": ". Did you mean: service?"
  },
  "action_templates": [
    {
      "suggested_command": "service",
      "reason": "Suggested correction for misspelled command 'servise'"
    }
  ]
}
```
