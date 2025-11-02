# clap-noun-verb v3.3.0 Roadmap

## Release Goals

**Target Release Date**: Q2 2025  
**Focus**: Advanced clap features and "Typer-style" enhancements for developer experience

---

## 🎯 Core Features for v3.3.0

### 1. Custom Value Parsers (High Priority)

**Status**: Metadata exists but not fully implemented  
**Rationale**: Enable custom validation and type conversion like Typer's type callbacks

**Implementation**:
- Support `#[arg(value_parser = ...)]` with proper expression parsing
- Auto-infer parsers for common types (PathBuf, Url, etc.)
- Support clap's built-in parsers: `clap::value_parser!(u16).range(1..=65535)`
- Chain multiple parsers for validation

**Example**:
```rust
#[verb("deploy")]
fn deploy_service(
    #[arg(value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
    #[arg(value_parser = clap::value_parser!(PathBuf))]
    config_file: PathBuf,
) -> Result<DeployOutput> {
    Ok(deploy(port, config_file))
}
```

**Typer Equivalent**: `typer.Option(callback=validate_port)` where callback does validation

---

### 2. Enhanced Help System (High Priority)

**Status**: Basic help exists, needs enhancement  
**Rationale**: Better help formatting like Typer's rich help system

**Features**:
- **Long help**: Separate `long_help` from `help` (detailed explanation)
- **Next line help**: Help text on next line (better formatting)
- **Help override**: `#[arg(help = "...")]` to override docstring
- **Help extraction**: Better parsing of multi-line docstrings

**Example**:
```rust
/// Deploy a service
///
/// Short description appears in --help
///
/// Long description appears in --help with detailed
/// explanations and examples.
#[verb("deploy")]
fn deploy_service(
    /// Port number (short help)
    /// Detailed explanation of port configuration
    /// appears on the next line in help output.
    #[arg(help = "Override docstring with custom help")]
    port: u16,
) -> Result<DeployOutput> {
    Ok(deploy(port))
}
```

**Typer Equivalent**: Rich help with multiple lines and better formatting

---

### 3. Display Order Control (Medium Priority)

**Status**: Not implemented  
**Rationale**: Control argument order in help output (Typer allows custom ordering)

**Implementation**:
- Support `#[arg(display_order = N)]` attribute
- Apply `arg.display_order()` in `build_command()`
- Lower numbers appear first in help

**Example**:
```rust
#[verb("config")]
fn set_config(
    #[arg(display_order = 1)]  // Appears first
    host: String,
    #[arg(display_order = 2)]  // Appears second
    port: u16,
    #[arg(display_order = 99)] // Appears last
    debug: bool,
) -> Result<Config> {
    Ok(get_config(host, port, debug))
}
```

---

### 4. Exclusive vs Multiple Groups (Medium Priority)

**Status**: Groups exist but API unclear  
**Rationale**: Better API for exclusive vs multiple groups (like Typer)

**Implementation**:
- Support `#[arg(exclusive = true)]` for exclusive groups
- Support `#[arg(group = "name", multiple = true)]` for multiple groups
- Better group metadata storage

**Example**:
```rust
#[verb("filter")]
fn filter_items(
    #[arg(group = "format", exclusive = true)]  // Only one of these
    json: bool,
    #[arg(group = "format", exclusive = true)]
    yaml: bool,
    #[arg(group = "format", exclusive = true)]
    xml: bool,
) -> Result<FilteredItems> {
    Ok(filter(json, yaml, xml))
}
```

---

### 5. Trailing Varargs (Medium Priority)

**Status**: Not implemented  
**Rationale**: Support trailing positional arguments (like `*args` in Typer)

**Implementation**:
- Support `#[arg(trailing_vararg = true)]` for last positional
- Accept multiple values for trailing positional
- Common pattern: `git add <file>...` accepts multiple files

**Example**:
```rust
#[verb("add")]
fn add_files(
    #[arg(index = 0, trailing_vararg = true)]
    files: Vec<String>,  // Accepts: git add file1 file2 file3
) -> Result<AddResult> {
    Ok(add(files))
}
```

**Typer Equivalent**: `*args: tuple[str, ...]` for trailing arguments

---

### 6. Allow Negative Numbers (Low Priority)

**Status**: Not implemented  
**Rationale**: Some CLIs need negative numbers (e.g., `--offset -5`)

**Implementation**:
- Support `#[arg(allow_negative_numbers = true)]` for numeric types
- Apply `arg.allow_negative_numbers()` when specified

**Example**:
```rust
#[verb("move")]
fn move_cursor(
    #[arg(allow_negative_numbers = true)]
    offset: i32,  // Accepts: --offset -5 or --offset 5
) -> Result<Cursor> {
    Ok(move_cursor(offset))
}
```

---

### 7. Auto-Inferred Type Parsers (Medium Priority)

**Status**: Basic type inference exists  
**Rationale**: Automatically use best parser for common types (Typer does this)

**Auto-inferred Parsers**:
- `PathBuf` → `clap::value_parser!(PathBuf)`
- `Url` → `clap::value_parser!(Url)` (if `url` crate available)
- `IpAddr` → `clap::value_parser!(IpAddr)`
- `Duration` → Custom parser for duration strings
- `u8/u16/u32/u64` → Range validation based on type limits
- `i8/i16/i32/i64` → Range validation with negative support

**Example**:
```rust
#[verb("serve")]
fn serve_files(
    // Auto-inferred: uses PathBuf parser
    directory: PathBuf,
    // Auto-inferred: uses IpAddr parser
    bind: IpAddr,
    // Auto-inferred: uses u16 parser with port range validation
    port: u16,
) -> Result<Server> {
    Ok(serve(directory, bind, port))
}
```

---

### 8. Better Error Messages (Medium Priority)

**Status**: Basic error messages exist  
**Rationale**: Typer provides helpful error messages with suggestions

**Enhancements**:
- Suggestions for typos: "Did you mean `--port`?" when `--por` is used
- Better validation error messages
- Context in error messages (which argument failed)
- Usage examples in errors

**Example**:
```bash
$ myapp deploy --por 8080
error: unknown argument '--por'

  Did you mean '--port'?

  For more information, try '--help'
```

---

### 9. Long Help Extraction (Low Priority)

**Status**: Partially implemented  
**Rationale**: Extract detailed help from docstrings

**Implementation**:
- Parse multi-paragraph docstrings into `help` and `long_help`
- First paragraph = short help
- Subsequent paragraphs = long help
- Support explicit `#[arg(long_help = "...")]`

**Example**:
```rust
/// Deploy service
///
/// Short description (used in --help)
///
/// Long description with detailed explanation,
/// examples, and usage patterns (used in --help with details).
#[verb("deploy")]
fn deploy_service() -> Result<()> {
    Ok(())
}
```

---

### 10. Next Line Help (Low Priority)

**Status**: Not implemented  
**Rationale**: Better help formatting (Typer uses this)

**Implementation**:
- Support `#[arg(next_line_help = true)]`
- Help text appears on next line instead of same line
- Better for long descriptions

**Example**:
```rust
#[verb("build")]
fn build_project(
    #[arg(next_line_help = true)]
    /// This is a very long help description that
    /// would be better formatted on the next line
    /// in the help output for better readability.
    verbose: bool,
) -> Result<BuildResult> {
    Ok(build(verbose))
}
```

---

## 🔧 Technical Improvements

### Value Parser Implementation

Currently `value_parser` is stored as `Option<String>` but needs proper implementation:

1. **Expression Parsing**: Parse `clap::value_parser!(u16).range(1..=65535)` expressions
2. **Type Inference**: Auto-infer parsers from Rust types
3. **Parser Chaining**: Support multiple validators

**Challenge**: Parsing complex expressions in proc macros requires careful handling.

### Help System Enhancements

1. **Docstring Parsing**: Better extraction of help vs long_help
2. **Formatting**: Multi-line help text formatting
3. **Override System**: Priority: `#[arg(help)]` > docstring > default

---

## 📋 Implementation Priority

### High Priority
1. **Custom Value Parsers** - Core functionality needed
2. **Enhanced Help System** - Better DX
3. **Auto-Inferred Type Parsers** - Common use case

### Medium Priority
4. **Display Order Control** - Useful for complex CLIs
5. **Exclusive vs Multiple Groups** - Better API
6. **Trailing Varargs** - Common pattern
7. **Better Error Messages** - User experience

### Low Priority
8. **Allow Negative Numbers** - Niche use case
9. **Long Help Extraction** - Nice to have
10. **Next Line Help** - Formatting improvement

---

## 🚫 Out of Scope for v3.3.0

- **Async support**: Not planned (architectural decision)
- **Shell completions**: Defer to v3.4.0 (requires separate crate)
- **Custom output formats**: JSON is sufficient for now
- **Plugin system**: Defer to v4.0.0
- **Sub-noun nesting**: Defer to v4.0.0

---

## 🎯 Success Criteria

1. **All high-priority features** implemented with tests
2. **Test suite passes** in <1s
3. **Zero lint warnings** from clippy
4. **No breaking changes** from v3.2.0
5. **All examples compile and run**
6. **Documentation complete**

---

## 📝 Notes

- Focus on "Typer-like" developer experience improvements
- Maintain zero-cost abstraction principle
- Keep framework philosophy (composable, extensible)
- Prioritize features that complete partially implemented functionality

