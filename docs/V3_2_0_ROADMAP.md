# clap-noun-verb v3.2.0 Roadmap

## Release Goals

**Target Release Date**: Q1 2025  
**Focus**: Completing partially implemented features and adding essential clap capabilities

---

## 🎯 Core Features for v3.2.0

### 1. Environment Variable Support ✅ (High Priority)

**Status**: Metadata already stored, needs clap integration  
**Current State**: `ArgMetadata.env` exists but isn't applied to clap `Arg`

**Implementation**:
- Enable clap `env` feature flag in `Cargo.toml`
- Apply `arg.env()` when `arg_meta.env` is present
- Support `#[arg(env = "VAR_NAME")]` syntax in macros
- Add tests for env var fallback behavior

**Example**:
```rust
#[verb("config")]
fn set_config(
    #[arg(env = "SERVER_PORT", default_value = "8080")]
    port: u16,
) -> Result<Config> {
    Ok(get_config(port))
}
```

---

### 2. Positional Arguments ✅ (High Priority)

**Status**: Not implemented  
**Rationale**: Many CLIs need positional args (e.g., `git clone <url>`)

**Implementation**:
- Add `positional: Option<usize>` to `ArgMetadata`
- Support `#[arg(index = 0)]` syntax in macros
- Apply `arg.index()` when positional is specified
- Update type inference to handle positional args
- Add tests for positional argument parsing

**Example**:
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

**Usage**: `git clone https://example.com/repo.git [destination]`

---

### 3. Enhanced ArgAction Support ✅ (Medium Priority)

**Status**: Only `SetTrue` and `Append` are used  
**Rationale**: Support all clap `ArgAction` types for maximum flexibility

**Actions to Add**:
- `ArgAction::Count` - Count occurrences (e.g., `-vvv` → 3)
- `ArgAction::Set` - Set value (default for non-flags)
- `ArgAction::SetFalse` - Inverse flag (e.g., `--no-cache`)
- `ArgAction::CollectMultiple` - Collect multiple values differently

**Implementation**:
- Add `action: Option<ArgAction>` to `ArgMetadata`
- Support `#[arg(action = "count")]` syntax
- Auto-infer `Count` for `usize` flags, `SetFalse` for inverted flags
- Update macro to parse action attribute

**Example**:
```rust
#[verb("build")]
fn build_project(
    #[arg(short = 'v', action = "count")]
    verbosity: usize, // -v = 1, -vv = 2, -vvv = 3
    #[arg(action = "set_false")]
    no_cache: bool, // --no-cache sets to false
) -> Result<BuildOutput> {
    Ok(build(verbosity, no_cache))
}
```

---

### 4. Argument Groups and Conflicts ✅ (Medium Priority)

**Status**: Not implemented  
**Rationale**: Common CLI pattern (e.g., `--all` OR `--name`, but not both)

**Implementation**:
- Add `ArgGroup` support to `CommandRegistry`
- Support `#[group(...)]` attribute on verbs
- Parse `requires`, `conflicts_with`, `group` from `#[arg(...)]`
- Apply groups when building clap `Command`

**Example**:
```rust
#[verb("list")]
fn list_items(
    #[arg(group = "filter")]
    all: bool,
    #[arg(group = "filter")]
    name: Option<String>,
    #[arg(requires = "name")]
    exact: bool,
) -> Result<ListOutput> {
    Ok(list(all, name, exact))
}
```

---

### 5. Enhanced Validation ✅ (Medium Priority)

**Status**: Basic validation exists, needs expansion  
**Rationale**: Better UX with clearer error messages

**Enhancements**:
- Support `#[arg(value_parser = ...)]` for custom parsers
- Better error messages with suggestions
- Validation chain (multiple validators per arg)
- Custom error messages via `#[arg(conflicts_with = "...", help = "...")]`

**Example**:
```rust
#[verb("deploy")]
fn deploy_service(
    #[arg(value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
) -> Result<DeployOutput> {
    Ok(deploy(port))
}
```

---

### 6. Improved Help Generation ✅ (Low Priority)

**Status**: Basic help exists from docstrings  
**Rationale**: Better developer experience

**Enhancements**:
- Support `long_about` from docstrings
- Better formatting of multi-line help text
- Support `#[arg(help = "...")]` override
- Support `#[arg(hide = true)]` for hidden args
- Support `#[arg(next_help_heading = "...")]` for grouping

**Example**:
```rust
/// Deploy a service to production
///
/// This command deploys the current service to the production environment.
/// Make sure you've tested everything in staging first.
#[verb("deploy")]
fn deploy_service(
    /// Port number (1-65535)
    #[arg(help = "The port to listen on")]
    port: u16,
    #[arg(hide = true)] // Hidden from help
    debug: bool,
) -> Result<DeployOutput> {
    Ok(deploy(port, debug))
}
```

---

## 🔧 Technical Improvements

### Dependency Updates
- **clap**: Ensure compatibility with latest 4.x features
- **syn/quote**: Update if needed for macro improvements
- **serde**: Maintain compatibility

### Performance
- No expected regressions
- Maintain zero-cost abstraction principle
- Keep test suite <1s runtime

### Testing
- Add tests for all new features
- Integration tests for complex scenarios
- Performance benchmarks

---

## 📋 Implementation Checklist

### Environment Variables
- [ ] Enable clap `env` feature
- [ ] Implement `arg.env()` in `CommandRegistry::build_command()`
- [ ] Parse `#[arg(env = "...")]` in macro
- [ ] Add tests for env var behavior
- [ ] Update examples
- [ ] Document in README

### Positional Arguments
- [ ] Add `positional: Option<usize>` to `ArgMetadata`
- [ ] Parse `#[arg(index = N)]` in macro
- [ ] Apply `arg.index()` in `build_command()`
- [ ] Update type inference logic
- [ ] Add tests for positional args
- [ ] Update examples

### Enhanced ArgAction
- [ ] Add `action: Option<ArgAction>` to `ArgMetadata`
- [ ] Parse `#[arg(action = "...")]` in macro
- [ ] Auto-infer actions for common types
- [ ] Update `build_command()` to apply actions
- [ ] Add tests for all action types
- [ ] Update examples

### Argument Groups
- [ ] Design API for `#[group(...)]`
- [ ] Implement `ArgGroup` support in registry
- [ ] Parse `requires`, `conflicts_with`, `group` in macro
- [ ] Apply groups in `build_command()`
- [ ] Add tests for groups and conflicts
- [ ] Update examples

### Enhanced Validation
- [ ] Support `value_parser` attribute
- [ ] Improve error messages
- [ ] Add validation chain support
- [ ] Add tests
- [ ] Update examples

### Help Generation
- [ ] Support `long_about` extraction
- [ ] Support `#[arg(help = "...")]` override
- [ ] Support `#[arg(hide = true)]`
- [ ] Support `next_help_heading`
- [ ] Add tests
- [ ] Update examples

---

## 🚫 Out of Scope for v3.2.0

- **Async support**: Not planned (architectural decision)
- **Sub-noun nesting**: Defer to v3.3.0 or v4.0.0
- **Shell completions**: Defer to v3.3.0 (requires separate crate)
- **Custom output formats**: JSON is sufficient for now
- **Plugin system**: Defer to v4.0.0

---

## 📚 Documentation Updates

- Update README with new features
- Add migration guide from v3.1.0
- Create examples for each new feature
- Update API documentation
- Add cookbook-style guides

---

## 🎯 Success Criteria

1. **All features implemented** with tests
2. **Test suite passes** in <1s
3. **Zero lint warnings** from clippy
4. **No breaking changes** from v3.1.0
5. **All examples compile and run**
6. **Documentation complete**

---

## 📅 Timeline Estimate

- **Environment Variables**: 2-3 days
- **Positional Arguments**: 3-4 days
- **Enhanced ArgAction**: 2-3 days
- **Argument Groups**: 4-5 days
- **Enhanced Validation**: 2-3 days
- **Help Generation**: 2-3 days
- **Testing & Documentation**: 3-4 days

**Total**: ~3-4 weeks

---

## 🔄 Versioning Strategy

Following semantic versioning:
- **v3.2.0**: Minor version (new features, backward compatible)
- All features are additive (no breaking changes)
- Deprecations will be announced in v3.2.0 for removal in v4.0.0

---

## 📝 Notes

- Prioritize features that complete partially implemented functionality
- Focus on developer experience improvements
- Maintain zero-cost abstraction principle
- Keep framework philosophy (composable, extensible)
- Ensure all features are well-tested before release

