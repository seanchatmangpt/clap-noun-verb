# Common Mistakes with clap-noun-verb

This guide documents the 10 most common mistakes users make when working with clap-noun-verb, along with solutions and examples. Each entry shows what users tried, why it failed, and the correct approach.

## 1. Missing Return Type

**What they tried:**
```rust
#[verb]
fn show_status() {
    println!("Status: OK");
}
```

**Why it failed:**
The `#[verb]` macro requires functions to return a value that implements `serde::Serialize` so it can be automatically converted to JSON output.

**Error message:**
```
Function 'show_status' must return a value that implements serde::Serialize
Hint: Add a return type like `Result<Status>` where Status derives Serialize
```

**Correct approach:**
```rust
#[derive(Serialize)]
struct Status {
    message: String,
}

#[verb]
fn show_status() -> Result<Status> {
    Ok(Status { message: "OK".to_string() })
}
```

**See also:** [README - How-to Guides](../README.md#how-to-guides)

---

## 2. Forgetting Double Quotes in Attributes

**What they tried:**
```rust
#[verb(status)]  // Missing quotes!
fn show_status() -> Result<String> {
    Ok("Running".to_string())
}
```

**Why it failed:**
Rust attribute macros require string literals (with quotes). Using an identifier without quotes is invalid syntax.

**Error message:**
```
Argument 1 in #[verb] must be a string literal
Found: status
Expected: "status"
Hint: Add double quotes around the identifier
```

**Correct approach:**
```rust
#[verb("status")]
fn show_status() -> Result<String> {
    Ok("Running".to_string())
}
```

---

## 3. Wrong Quote Type for `short` Argument

**What they tried:**
```rust
#[verb]
fn show_logs(
    #[arg(short = "v")]  // Wrong! Should be single quotes for char
    verbose: bool,
) -> Result<String> { /* ... */ }
```

**Why it failed:**
The `short` attribute expects a character literal (single quotes), not a string literal (double quotes).

**Error message:**
```
Invalid #[arg] attribute syntax
Expected: #[arg(short = 'v')]
Common mistakes:
- Wrong quotes: short = "v" should be short = 'v'
```

**Correct approach:**
```rust
#[verb]
fn show_logs(
    #[arg(short = 'v')]  // Single quotes for char
    verbose: bool,
) -> Result<String> { /* ... */ }
```

**See also:** [README - Argument Attributes](../README.md#argument-attributes)

---

## 4. Not Deriving `Serialize` on Return Types

**What they tried:**
```rust
struct Status {  // Missing #[derive(Serialize)]!
    running: bool,
}

#[verb]
fn show_status() -> Result<Status> {
    Ok(Status { running: true })
}
```

**Why it failed:**
All return types must implement `serde::Serialize` so clap-noun-verb can convert them to JSON.

**Error message:**
```
the trait bound `Status: Serialize` is not satisfied
help: consider annotating `Status` with `#[derive(Serialize)]`
```

**Correct approach:**
```rust
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    running: bool,
}

#[verb]
fn show_status() -> Result<Status> {
    Ok(Status { running: true })
}
```

---

## 5. Using Wrong Attribute Position

**What they tried:**
```rust
#[arg(short = 'p', default_value = "8080")]  // Wrong! Can't use on function
#[verb]
fn configure_server(port: u16) -> Result<String> { /* ... */ }
```

**Why it failed:**
The `#[arg(...)]` attribute must be placed on function **parameters**, not on the function itself.

**Correct approach:**
```rust
#[verb]
fn configure_server(
    #[arg(short = 'p', default_value = "8080")]
    port: u16,
) -> Result<String> { /* ... */ }
```

**See also:** [README - How to configure arguments](../README.md#how-to-configure-arguments)

---

## 6. Duplicate Verb Registration

**What they tried:**
```rust
// In services.rs
#[verb("status", "services")]
fn check_status() -> Result<String> { Ok("OK".to_string()) }

#[verb("status", "services")]  // Duplicate!
fn show_status() -> Result<String> { Ok("Running".to_string()) }
```

**Why it failed:**
Each noun-verb combination must be unique. The macro generates compile-time checks to prevent duplicates.

**Error message:**
```
duplicate definitions with name `__VERB_DUPLICATE_CHECK_services_status_...`
```

**Correct approach:**
Use unique verb names:
```rust
#[verb("check", "services")]
fn check_status() -> Result<String> { Ok("OK".to_string()) }

#[verb("status", "services")]
fn show_status() -> Result<String> { Ok("Running".to_string()) }
```

---

## 7. Confusing Auto-Inference with Explicit Names

**What they tried:**
```rust
// File: collector.rs
#[verb("collector_status")]  // Redundant! Will create "collector collector_status"
fn collector_status() -> Result<String> {
    Ok("Up".to_string())
}
```

**Why it failed:**
When the noun is auto-inferred from the filename (`collector.rs` → noun: `collector`), the verb name is automatically stripped if it contains the noun prefix. Explicitly including it creates redundancy.

**Expected command:** `myapp collector collector_status` (wrong)
**Intended command:** `myapp collector status` (correct)

**Correct approach:**

| Pattern | Verb Attribute | Function Name | Result |
|---------|---------------|---------------|---------|
| **Auto-infer both** | `#[verb]` | `fn show_status()` | `collector status` |
| **Explicit verb** | `#[verb("status")]` | `fn whatever_name()` | `collector status` |
| **Explicit both** | `#[verb("status", "collector")]` | `fn any_name()` | `collector status` |

**See also:** [README - Verb Registration](../README.md#verb-registration)

---

## 8. Missing `env` Variable Quotes

**What they tried:**
```rust
#[verb]
fn configure(
    #[arg(env = PORT)]  // Missing quotes!
    port: u16,
) -> Result<String> { /* ... */ }
```

**Why it failed:**
Environment variable names must be string literals (with quotes).

**Error message:**
```
Invalid #[arg] attribute syntax
Expected: env = "PORT"
Common mistakes:
- Missing quotes: env = PORT should be env = "PORT"
```

**Correct approach:**
```rust
#[verb]
fn configure(
    #[arg(env = "PORT", default_value = "8080")]
    port: u16,
) -> Result<String> { /* ... */ }
```

**See also:** [README - How to configure arguments](../README.md#how-to-configure-arguments), [Example: env_vars.rs](../examples/env_vars.rs)

---

## 9. Wrong Argument Count in `#[verb]`

**What they tried:**
```rust
#[verb("status", "services", "extra")]  // Too many arguments!
fn show_status() -> Result<String> {
    Ok("OK".to_string())
}
```

**Why it failed:**
The `#[verb]` attribute accepts 0, 1, or 2 arguments only:
- 0 args: Auto-infer verb name from function name
- 1 arg: Explicit verb name
- 2 args: Explicit verb name + noun name

**Error message:**
```
Too many arguments in #[verb] attribute
Expected: 0, 1, or 2 arguments
Found: 3 arguments
Hint: Remove extra arguments
```

**Correct approach:**

```rust
// Auto-infer (0 args)
#[verb]
fn show_status() -> Result<String> { /* ... */ }

// Explicit verb (1 arg)
#[verb("status")]
fn check_service() -> Result<String> { /* ... */ }

// Explicit verb + noun (2 args)
#[verb("status", "services")]
fn get_status() -> Result<String> { /* ... */ }
```

---

## 10. Mixing `index` with Named Arguments

**What they tried:**
```rust
#[verb]
fn upload_file(
    #[arg(index = 0, short = 'f')]  // Can't mix positional and named!
    file: String,
) -> Result<String> { /* ... */ }
```

**Why it failed:**
Positional arguments (using `index`) cannot have short or long flags. They are parsed by position, not by name.

**Correct approach:**

**For positional arguments:**
```rust
#[verb]
fn upload_file(
    #[arg(index = 0)]  // Only index, no short/long
    file: String,
) -> Result<String> { /* ... */ }
```

**For named arguments:**
```rust
#[verb]
fn upload_file(
    #[arg(short = 'f')]  // Only short/long, no index
    file: String,
) -> Result<String> { /* ... */ }
```

**See also:** [README - Argument Attributes](../README.md#argument-attributes)

---

## Summary Table

| Mistake | Quick Fix | Where to Learn More |
|---------|-----------|---------------------|
| Missing return type | Add `-> Result<T>` where `T: Serialize` | [README - Type Inference](../README.md#type-inference) |
| Missing quotes in `#[verb(...)]` | Use `#[verb("name")]` not `#[verb(name)]` | [README - Verb Registration](../README.md#verb-registration) |
| Wrong quotes for `short` | Use `short = 'v'` not `short = "v"` | [README - Argument Attributes](../README.md#argument-attributes) |
| Missing `Serialize` derive | Add `#[derive(Serialize)]` to struct | [README - Quick Start](../README.md#quick-start) |
| `#[arg]` on function | Put `#[arg(...)]` on **parameters** | [README - How to configure arguments](../README.md#how-to-configure-arguments) |
| Duplicate verbs | Use unique verb names per noun | [tests/compile_time_validation.rs](../tests/compile_time_validation.rs) |
| Redundant noun prefix | Use `#[verb("status")]` not `#[verb("noun_status")]` | [README - Verb Registration](../README.md#verb-registration) |
| Missing env quotes | Use `env = "PORT"` not `env = PORT` | [examples/env_vars.rs](../examples/env_vars.rs) |
| Too many verb args | Max 2 args: `#[verb("verb", "noun")]` | [README - Verb Registration](../README.md#verb-registration) |
| Mixing positional/named | Use `index` OR `short`/`long`, not both | [README - Argument Attributes](../README.md#argument-attributes) |

---

## Additional Resources

- **Main Documentation:** [README.md](../README.md)
- **Complete Examples:** [examples/](../examples/)
- **Validation Tests:** [tests/compile_time_validation.rs](../tests/compile_time_validation.rs)
- **Argument Examples:** [examples/arg_groups.rs](../examples/arg_groups.rs)
- **Environment Variables:** [examples/env_vars.rs](../examples/env_vars.rs)

## Getting Help

If you encounter an error not covered here:

1. Check the **compiler error message** - it includes hints
2. Review **[README.md](../README.md)** for patterns and examples
3. Look at **[examples/](../examples/)** for working code
4. Search **existing issues** on GitHub
5. Open a new issue with the error message and code sample

---

**Last Updated:** v4.0.1 (2025-11-18)
