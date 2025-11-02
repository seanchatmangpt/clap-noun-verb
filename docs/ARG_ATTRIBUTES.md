# Argument Attributes (`#[arg]`)

## Why `#[arg]` Isn't a Real Attribute Macro

**Rust Limitation**: `#[proc_macro_attribute]` can only be used on items (functions, structs, etc.), not on function parameters.

Therefore, `#[arg]` attributes on function parameters are:
- **Parsed by the `#[verb]` macro** during expansion
- **Not recognized by the compiler** as a real attribute
- **Work correctly** at runtime because the `#[verb]` macro processes them

## Current Implementation

The `#[verb]` macro parses `#[arg(...)]` attributes from function parameters during macro expansion. The attributes are:

- Parsed from `pat_type.attrs` in the macro
- Used to generate `ArgMetadata` with `short`, `default_value`, `multiple`, `value_name`, `aliases`, etc.
- Applied to `clap::Arg` during command building

## Testing Strategy

Since `#[arg]` can't be used directly in test code (compiler rejects unknown attributes), we test:

1. **Parsing Logic**: The `parse_arg_attributes()` function correctly parses attribute syntax
2. **Registry Behavior**: Arguments with metadata are correctly registered and built
3. **Integration**: End-to-end behavior works when `#[arg]` attributes are parsed

## Future Options

1. Use `#[allow(unknown_attributes)]` to silence compiler warnings
2. Test parsing logic with `proc-macro2` in unit tests
3. Document that `#[arg]` is a "synthetic" attribute parsed by `#[verb]`

## Usage

In real code, users would write:

```rust
#[verb("set")]
fn set_config(
    #[arg(short = 'p', default_value = "8080")]
    port: u16,
) -> Result<()> {}
```

The compiler may warn about unknown attributes, but the macro processes them correctly.

## Supported Attributes (v3.2.0)

### Basic Attributes

- **`short = 'v'`** - Short flag character (e.g., `-v`)
- **`default_value = "value"`** - Default value as string
- **`env = "VAR_NAME"`** - Environment variable name (requires clap `env` feature)
- **`value_name = "NAME"`** - Custom value name for help text (e.g., "FILE", "PORT")
- **`multiple`** - Accept multiple values (also auto-detected from `Vec<T>`)
- **`alias = "name"`** - Single alias
- **`aliases = ["name1", "name2"]`** - Multiple aliases

### Positional Arguments

- **`index = 0`** - Positional argument index (0, 1, 2, ...)

**Example:**
```rust
#[verb("clone")]
fn clone_repo(
    #[arg(index = 0)]
    url: String,
    #[arg(index = 1)]
    destination: Option<String>,
) -> Result<Repo> {
    Ok(clone(url, destination))
}
```

### ArgAction

- **`action = "count"`** - Count occurrences (e.g., `-vvv` → 3, auto-inferred for `usize`)
- **`action = "set"`** - Set value (default for non-flags)
- **`action = "set_false"`** - Inverse flag (e.g., `--no-cache`)
- **`action = "set_true"`** - Set to true when present (auto-inferred for `bool`)
- **`action = "append"`** - Append multiple values (auto-inferred for `Vec<T>`)

**Example:**
```rust
#[verb("build")]
fn build_project(
    verbose: usize, // Auto-inferred as Count action
    #[arg(action = "set_false")]
    cache: bool,    // SetFalse action
) -> Result<BuildResult> {
    Ok(build(verbose, cache))
}
```

### Argument Groups

- **`group = "group_name"`** - Argument group name (for exclusive/multiple groups)

**Example:**
```rust
#[verb("filter")]
fn filter_items(
    #[arg(group = "filter")]
    by_name: Option<String>,
    #[arg(group = "filter")]
    by_id: Option<u64>,
) -> Result<FilteredItems> {
    Ok(filter(by_name, by_id))
}
```

### Argument Relationships

- **`requires = "arg_name"`** - Require another argument (single)
- **`requires = ["arg1", "arg2"]`** - Require multiple arguments
- **`conflicts_with = "arg_name"`** - Conflict with another argument (single)
- **`conflicts_with = ["arg1", "arg2"]`** - Conflict with multiple arguments

**Example:**
```rust
#[verb("export")]
fn export_data(
    #[arg(requires = "format")]
    filename: Option<String>,
    #[arg(short = 'f')]
    format: Option<String>,
    #[arg(conflicts_with = "format")]
    raw: bool,
) -> Result<ExportResult> {
    Ok(export(filename, format, raw))
}
```

### Help Customization

- **`hide = true`** - Hide argument from help text
- **`help_heading = "Heading"`** - Group argument under heading in help

**Example:**
```rust
#[verb("command")]
fn my_command(
    /// Visible argument
    visible: String,
    /// Hidden argument (not shown in help)
    #[arg(hide = true)]
    hidden: String,
    /// Argument under "Advanced" heading
    #[arg(help_heading = "Advanced")]
    advanced: bool,
) -> Result<Output> {
    Ok(create_output(visible, hidden, advanced))
}
```

## Complete Example

```rust
#[verb("deploy")]
fn deploy_service(
    // Short flag with default
    #[arg(short = 'p', default_value = "8080", value_name = "PORT")]
    port: u16,
    
    // Environment variable
    #[arg(env = "DEPLOY_HOST", default_value = "localhost")]
    host: String,
    
    // Positional argument
    #[arg(index = 0)]
    service: String,
    
    // Count action (auto-inferred for usize)
    verbose: usize,
    
    // Multiple values (auto-inferred from Vec<T>)
    tags: Vec<String>,
    
    // SetFalse action
    #[arg(action = "set_false")]
    cache: bool,
    
    // Argument group
    #[arg(group = "format")]
    json: bool,
    #[arg(group = "format")]
    yaml: bool,
    
    // Requires another argument
    #[arg(requires = "output", value_name = "FILE")]
    output: Option<String>,
    
    // Conflicts with another argument
    #[arg(conflicts_with = "output")]
    dry_run: bool,
    
    // Hidden argument
    #[arg(hide = true)]
    secret_key: String,
) -> Result<DeployResult> {
    Ok(deploy(port, host, service, verbose, tags, cache, json, yaml, output, dry_run, secret_key))
}
```

