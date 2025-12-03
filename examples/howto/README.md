# How-To Examples

**Task-oriented examples for solving specific problems**

These examples show how to accomplish common CLI development tasks.

## Quick Reference

| Problem | Example | v5.2.0 Tags Used |
|---------|---------|------------------|
| Mutually exclusive args | `arg_groups.rs` | `[group:]` |
| Argument dependencies | `arg_groups.rs` | `[requires:]` |
| Argument conflicts | `arg_groups.rs` | `[conflicts:]` |
| Validate input | `validation.rs` | - |
| Environment variables | `env_vars.rs` | `[env:]` |
| Shell completion | `completion.rs` | `[value_hint:]` |
| Deprecate commands | `deprecation.rs` | - |
| Count/append actions | `arg_actions.rs` | - |

## Examples

### arg_groups.rs - Argument Relationships
**The most important how-to for v5.2.0**

```bash
# Mutually exclusive (json OR yaml)
cargo run --example howto_arg_groups -- export data --json
cargo run --example howto_arg_groups -- export data --yaml
# Error: both flags
cargo run --example howto_arg_groups -- export data --json --yaml

# Required dependency
cargo run --example howto_arg_groups -- export data --format json --filename out.json
# Error: filename without format
cargo run --example howto_arg_groups -- export data --filename out.json
```

**Demonstrates:**
- `[group: name]` - Mutually exclusive argument groups
- `[requires: arg]` - Argument requires another to be present
- `[conflicts: arg]` - Arguments cannot be used together
- `[env: VAR]` - Read from environment variable
- `[default: value]` - Default when not provided
- `[value_hint: type]` - Shell completion hints
- `[hide]` - Hide from help
- `[help_heading: name]` - Organize help output
- `[exclusive]` - Cannot combine with any other args

### validation.rs - Input Validation
```bash
cargo run --example howto_validation
```

**Demonstrates:**
- Custom validators
- Range validation
- Format validation (email, URL, etc.)
- Error messages for invalid input

### env_vars.rs - Environment Variables
```bash
LOG_LEVEL=debug cargo run --example howto_env_vars
CONFIG_PATH=/etc/myapp.toml cargo run --example howto_env_vars
```

**Demonstrates:**
- `[env: VAR]` tag usage
- CLI overrides environment
- Fallback to environment when CLI not provided

### completion.rs - Shell Completion
```bash
# Generate completions
cargo run --example howto_completion -- --generate bash > completion.bash
cargo run --example howto_completion -- --generate zsh > completion.zsh
cargo run --example howto_completion -- --generate fish > completion.fish
```

**Demonstrates:**
- `[value_hint: FilePath]` for file completion
- `[value_hint: DirPath]` for directory completion
- `[value_hint: Url]` for URL completion
- Generating shell completion scripts

### deprecation.rs - Deprecated Commands
```bash
cargo run --example howto_deprecation -- old-command
# Warning: 'old-command' is deprecated, use 'new-command' instead
```

**Demonstrates:**
- Marking commands as deprecated
- Migration guidance in help text
- Warning messages on usage

### arg_actions.rs - Argument Actions
```bash
# Count flag occurrences
cargo run --example howto_arg_actions -- -v -v -v
# verbose_level = 3

# Append multiple values
cargo run --example howto_arg_actions -- --tag foo --tag bar --tag baz
# tags = ["foo", "bar", "baz"]
```

**Demonstrates:**
- Count action for verbosity flags
- Append action for collecting values
- Set action for overriding values

## v5.2.0 Phase 2 Syntax Reference

```rust
/// # Arguments
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
/// * `output` - Output file [requires: format] [value_hint: FilePath]
/// * `format` - Format string [conflicts: raw]
/// * `raw` - Raw output [conflicts: format]
/// * `log_level` - Log level [env: LOG_LEVEL] [default: info]
/// * `debug` - Debug mode [hide]
/// * `verbose` - Verbose output [help_heading: Logging]
/// * `version_check` - Check version [exclusive]
```

## Next Steps

1. See [docs/howto/production/](../../docs/howto/production/) for deployment guides
2. Explore [reference examples](../reference/) for complete API coverage
3. Try [advanced examples](../advanced/) for production patterns
