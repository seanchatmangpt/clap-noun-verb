# CLI Parameter Preprocessor Guide

The `clap-noun-verb` framework includes a built-in preprocessor designed to enable dynamic workflows, command chaining, standard input binding, environment overrides, and variable expansion. This document details the inner workings of the preprocessor, its capabilities, and its safety features.

---

## 1. Delimiter Parameter Chaining (`++`)

For multi-step CLI operations, you can chain multiple commands together in a single invocation using the delimiter `++`. The preprocessor parses the arguments, splits them into independent commands, and executes them sequentially.

### How it Works

1. **Splitting Step Arguments**:
   When the program receives arguments, it scans the collection and splits on the exact sequence `"++"`. Each sub-slice is mapped to a distinct step.
   For example, the command:
   ```bash
   myapp noun1 verb1 arg1 ++ noun2 verb2 arg2
   ```
   is split into two steps:
   - Step 1: `["noun1", "verb1", "arg1"]`
   - Step 2: `["noun2", "verb2", "arg2"]`

2. **Sequential Execution**:
   Each step is run sequentially through the command router. The output from the handler of each step is captured as a JSON value in a sequential execution log:
   - Step 1 output is stored at index `0` (referenced as `1` in user templates).
   - Step 2 output is stored at index `1` (referenced as `2` in user templates).

3. **Global Stdin Check**:
   Before executing the first command, the preprocessor checks if *any* step requires standard input (`stdin`). If any step binds to stdin (e.g. using `@-` or `@-::path`), the preprocessor reads `stdin` once and caches it to resolve all bindings across all steps.

---

## 2. Dynamic Environment and Parameter Overrides

The framework integrates environment-variable-backed fallback values using standard Clap attributes mapped to `ArgMetadata`.

### Mapping Environment Variables
When defining parameters using `ArgMetadata`, you can specify an optional `env` field. For example:

```rust
let api_key_arg = ArgMetadata {
    name: "api_key".to_string(),
    required: true,
    env: Some("API_KEY".to_string()),
    // other fields...
};
```

### Precedence Rules
If an environment variable is mapped to an argument, the parameter resolution follows this order of precedence:
1. **Explicit CLI Argument**: Value passed directly via command-line arguments (e.g. `--api-key secret_123`).
2. **Environment Variable**: Value read from the environment if the CLI argument is missing (e.g. `API_KEY=secret_456`).
3. **Default Value**: Fallback value defined in `ArgMetadata::default_value`.

---

## 3. Variable Expansion (`@{var}`)

The preprocessor parses and expands variables dynamically before passing the final arguments to the parser.

### A. Step Reference Expansion (`@{step.path}`)

Subsequent commands in a chained execution can reference the output of preceding steps.

- **Syntax**: `@{step_index.json_path}` (where `step_index` is 1-indexed).
- **JSON Path Syntax**: Dot-notation is used to navigate objects and arrays (e.g. `@{1.user.roles.0}`).
- **String Interpolation**: Step references can be embedded anywhere within an argument string (e.g. `Bearer @{1.session.token}`).

#### Example Scenario
Suppose Step 1 (`myapp session login`) returns:
```json
{
  "token": "secret-123",
  "user": {
    "id": 42,
    "role": "admin"
  }
}
```

Step 2 can reference these fields:
```bash
myapp user get --id "@{1.user.id}" --auth "Bearer @{1.token}"
```
The preprocessor resolves this to:
```bash
myapp user get --id 42 --auth "Bearer secret-123"
```

### B. Stdin Bindings

- **Raw Stdin Binding (`@-`)**:
  Replaces the argument with the complete, raw content of `stdin`.
  ```bash
  echo "content" | myapp upload --data "@-"
  ```
- **JSON Stdin Path Binding (`@-::json_path`)**:
  Parses `stdin` as a JSON object and extracts a nested value via dot notation.
  ```bash
  echo '{"id": 99}' | myapp user update --id "@-::id"
  ```

---

## 4. Recursion Infinite-Loop Prevention Safety

A common risk when performing string expansion/interpolation is infinite recursion or loop cycles (e.g., expanding a variable that resolves to a string containing the same variable pattern). The preprocessor implements an elegant and robust mechanism to guarantee stack safety and prevent infinite loops.

### The Mitigation Mechanism

The preprocessor tracks a `search_idx` pointer as it scans each argument string for the `@{` delimiter:

```rust
let mut search_idx = 0;
while let Some(start_offset) = new_arg[search_idx..].find("@{") {
    let start_idx = search_idx + start_offset;
    if let Some(end_offset) = new_arg[start_idx..].find('}') {
        let end_idx = start_idx + end_offset;
        let ref_content = &new_arg[start_idx + 2..end_idx];
        
        let mut resolved = false;
        // ... resolution logic ...
        if let Some(resolved_val) = get_json_path(step_data, path) {
            new_arg.replace_range(start_idx..=end_idx, &resolved_val);
            search_idx = start_idx + resolved_val.len();
            resolved = true;
        }
        // ... fallback/non-resolved logic ...
    }
}
```

### Why This Prevents Infinite Loops

1. **Non-Recursive Scanning**: 
   When a reference is replaced by `resolved_val`, `search_idx` is advanced to `start_idx + resolved_val.len()`.
2. **Exclusion of Replacement Content**:
   Any new `@{` patterns present inside the replacement value are skipped. The scanner never goes backward to re-evaluate resolved content.
3. **Guaranteed Termination**:
   Because `search_idx` moves strictly forward, the expansion process terminates in a single pass ($O(N)$ complexity relative to the final string length), making it immune to self-referential expansion loops.

#### Example of Safety in Action
If a step result contains a self-referential loop:
```json
{
  "recursive": "looping @{1.recursive}"
}
```
And you execute:
```bash
myapp cmd --arg "@{1.recursive}"
```
The preprocessor resolves `@{1.recursive}` to `looping @{1.recursive}`. It places `search_idx` at the end of the resolved string. The scanner terminates, resulting in the literal string:
```bash
myapp cmd --arg "looping @{1.recursive}"
```
No infinite loop occurs, and the application remains completely stable.
