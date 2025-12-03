# ggen v2.0.0 - Breaking Changes & Major Improvements

## Executive Summary

**Goal**: Direct migration to v2.0.0-style architecture with breaking changes. No backward compatibility - clean break.

**Timeline**: Direct implementation (4-6 weeks)
**Risk Level**: Medium (breaking changes require migration guides, but no aliases to maintain)
**Version Strategy**: Direct v2.0.0 migration - no backward compatibility layer

**Key Principles**:
- **Clean APIs**: Remove legacy patterns and deprecated features
- **Better Architecture**: Complete separation of concerns and module boundaries
- **Improved DX**: Auto-discovery, type inference, JSON output by default
- **Direct Implementation**: No migration needed - project has no users

---

## Current Version Status

- **Current Version**: 1.2.0
- **Planned v1.3.0**: Minor improvements, bug fixes
- **Planned v1.4.0**: CLI refactoring with backward compatibility (aliases)
- **Planned v2.0.0**: Breaking changes, cleanup, major improvements

---

## v2.0.0 Breaking Changes

### 1. CLI Command Structure (Breaking)

#### 1.1 Direct Command Renames (No Aliases)

**v1.x Behavior**:
```bash
ggen market search          # Old form
ggen doctor                 # Root-level command
ggen help-me                # Root-level command
ggen gen                    # Legacy command
```

**v2.0.0 Behavior** (direct rename, no aliases):
```bash
ggen market search          # ❌ ERROR: Command not found
ggen marketplace search     # ✅ Only this form works
ggen doctor                 # ❌ ERROR: Command not found
ggen utils doctor           # ✅ Only this form works
ggen help-me                # ❌ ERROR: Command not found
ggen utils help-me          # ✅ Only this form works
ggen gen                    # ❌ ERROR: Command not found (removed)
ggen template generate      # ✅ Use this instead
```

**Implementation**: Direct rename - no migration needed.

#### 1.2 Removed Root-Level Commands

**v1.x Behavior**:
```bash
ggen doctor                # Root-level command
ggen help-me               # Root-level command
```

**v2.0.0 Behavior** (grouped under `utils`):
```bash
ggen doctor                # ❌ ERROR: Command not found
ggen utils doctor          # ✅ New form required
ggen help-me               # ❌ ERROR: Command not found
ggen utils help-me         # ✅ New form required
```

**Migration**:
- Update scripts: `doctor` → `utils doctor`
- Update scripts: `help-me` → `utils help-me`
- Automated migration tool available

### 2. API Changes (Breaking)

#### 2.1 CLI Library API

**v1.x API**:
```rust
// Old: Manual command registration
use ggen_cli_lib::Commands;

let commands = Commands::Ai(ai::AiArgs {
    command: ai::AiCommand::Project(project::ProjectArgs {
        name: "my-app".to_string(),
        // ...
    }),
});

commands.run().await?;
```

**v2.0.0 API** (clap-noun-verb):
```rust
// New: Auto-discovery with attribute macros
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("project", "ai")]
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, rust))
}

fn main() -> Result<()> {
    clap_noun_verb::run()  // Auto-discovers all #[verb] functions
}
```

**Migration**:
- Migrate custom CLI implementations to use `#[verb]` attributes
- Update command handlers to return `Result<Serializable>` instead of `Result<()>`
- Use JSON output instead of println! for structured output

#### 2.2 Business Logic API

**v1.x API** (mixed CLI and business logic):
```rust
// Old: Business logic mixed with CLI
pub fn handle_ai_project(args: &ArgMatches) -> Result<()> {
    let name = args.get_one::<String>("name").unwrap();
    // Business logic here...
    println!("Project created: {}", name);
    Ok(())
}
```

**v2.0.0 API** (separated):
```rust
// New: Pure business logic (reusable)
pub fn create_project(name: String, rust: bool) -> ProjectOutput {
    // Pure function - no CLI dependencies
    ProjectOutput {
        name,
        rust,
        success: true,
        // ...
    }
}

// CLI layer delegates to business logic
#[verb("project", "ai")]
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, rust))
}
```

**Migration**:
- Extract business logic from CLI handlers
- Create pure functions with typed inputs/outputs
- CLI handlers become thin delegation layers

### 3. Output Format Changes (Breaking)

#### 3.1 JSON by Default

**v1.x Behavior**:
```bash
ggen ai project my-app --rust
# Output: Plain text
Project created: my-app
Files: src/main.rs, Cargo.toml
```

**v2.0.0 Behavior** (JSON by default):
```bash
ggen ai project my-app --rust
# Output: JSON
{
  "name": "my-app",
  "rust": true,
  "success": true,
  "files_created": ["src/main.rs", "Cargo.toml"]
}

# Plain text still available with --format text
ggen ai project my-app --rust --format text
```

**Migration**:
- Update scripts that parse CLI output to handle JSON
- Use `--format text` for human-readable output
- Use `jq` or similar tools for JSON parsing in scripts

#### 3.2 Structured Error Output

**v1.x Behavior**:
```bash
# Errors: Plain text
Error: Invalid argument 'name'
```

**v2.0.0 Behavior** (structured JSON errors):
```bash
# Errors: JSON structure
{
  "error": {
    "code": "INVALID_ARGUMENT",
    "message": "Invalid argument 'name'",
    "details": {
      "argument": "name",
      "value": null,
      "required": true
    }
  }
}
```

### 4. Dependency Changes (Breaking)

#### 4.1 New Required Dependencies

**v2.0.0 Requires**:
```toml
[dependencies]
clap-noun-verb = "3.0.0"
clap-noun-verb-macros = "3.0.0"
serde = { version = "1.0", features = ["derive"] }
linkme = "0.3"  # Already included in clap-noun-verb-macros
```

**Migration**:
- Add new dependencies to `Cargo.toml`
- Remove old manual command registration code
- Update imports to use clap-noun-verb

#### 4.2 Removed Dependencies

**v2.0.0 Removes**:
- Old CLI registration helpers (now in clap-noun-verb)
- Manual command enum builders (replaced by auto-discovery)

### 5. Configuration Changes (Breaking)

#### 5.1 Config File Format

**v1.x Config**:
```toml
[cli]
command_prefix = "ggen"
verbose = true
```

**v2.0.0 Config** (simplified):
```toml
[cli]
# Command prefix removed (handled by clap-noun-verb)
# Verbose flag now global --verbose
```

**Migration**:
- Update config files to new format
- Remove deprecated config options
- Use global flags instead of config options

---

## v2.0.0 New Features

### 1. Auto-Discovery

**New**: Commands automatically discovered from `#[verb]` attributes

```rust
// No manual registration needed!
#[verb("project", "ai")]
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, rust))
}

// Automatically available as: ggen ai project --name <name> --rust
```

### 2. Type Inference

**New**: Arguments automatically inferred from function signatures

```rust
#[verb("project", "ai")]
fn ai_project(
    name: String,              // Required argument: --name <value>
    description: Option<String>, // Optional argument: --description <value>
    rust: bool,                // Flag: --rust
    verbose: Option<usize>     // Optional count: --verbose (can use -vvv)
) -> Result<ProjectOutput> {
    // Arguments automatically parsed and validated
}
```

### 3. JSON Output

**New**: All commands return structured JSON by default

```bash
$ ggen ai project my-app --rust
{
  "name": "my-app",
  "description": null,
  "rust": true,
  "success": true,
  "files_created": ["src/main.rs", "Cargo.toml", "README.md"]
}
```

### 4. Better Error Messages

**New**: Structured error messages with context

```json
{
  "error": {
    "code": "MISSING_ARGUMENT",
    "message": "Required argument 'name' is missing",
    "context": {
      "command": "ai project",
      "help": "Use --name <value> to specify project name"
    }
  }
}
```

### 5. Improved Help System

**New**: Auto-generated help from docstrings

```rust
/// Generate a complete project using AI
///
/// # Arguments
/// * `name` - Project name (required)
/// * `description` - Optional project description
/// * `rust` - Generate Rust project structure (flag)
#[verb("project", "ai")]
fn ai_project(name: String, description: Option<String>, rust: bool) -> Result<ProjectOutput> {
    // Help automatically generated from docstring!
}
```

---

## Migration Strategy

### Phase 1: Preparation (Week 1)

1. **Document Changes**
   - Document breaking changes
   - Command rename mapping
   - Architecture improvements

2. **Update Documentation**
   - Breaking changes list
   - Command rename mapping table
   - New API documentation

### Phase 2: Implementation (Week 2-4)

1. **Implement Breaking Changes**
   - Rename `market` → `marketplace` (14 commands)
   - Group `doctor` → `utils.doctor`
   - Group `help-me` → `utils.help-me`
   - Remove legacy `ggen gen` command
   - Update all command handlers

2. **Implement New Features**
   - Auto-discovery with `#[verb]` attributes
   - JSON output by default
   - Type inference from function signatures
   - Business logic separation

3. **Testing**
   - Unit tests for all commands
   - Integration tests
   - Performance benchmarks

### Phase 3: Release (Week 5)

1. **Release v2.0.0**
   - Final testing
   - Documentation complete
   - Official release

---

## Command Mapping

| Old Command | New Command | Status |
|-------------|-------------|--------|
| `ggen market search` | `ggen marketplace search` | ✅ Direct rename |
| `ggen gen` | `ggen template generate` | ✅ Direct rename |
| `ggen doctor` | `ggen utils doctor` | ✅ Direct rename |
| `ggen help-me` | `ggen utils help-me` | ✅ Direct rename |
| Manual command registration | `#[verb]` attributes | ✅ Auto-discovery |
| Plain text output | JSON output (default) | ✅ New default |
| Type inference | Automatic | ✅ New feature |

---

## Rollout Plan

### Pre-Release (Week 1-2)

- [ ] Announce v2.0.0 plans
- [ ] Create migration tools
- [ ] Update documentation
- [ ] Add deprecation warnings to v1.4.0

### Beta Release (Week 3-4)

- [ ] Release v2.0.0-beta.1
- [ ] Collect feedback
- [ ] Fix critical issues
- [ ] Improve migration tools

### Release Candidate (Week 5)

- [ ] Release v2.0.0-rc.1
- [ ] Final testing
- [ ] Performance validation
- [ ] Documentation review

### Official Release (Week 6)

- [ ] Release v2.0.0
- [ ] Publish migration guides
- [ ] Community announcement
- [ ] Begin v1.x maintenance period

---

## Success Metrics

### Adoption Metrics

- **Migration Rate**: 80%+ of users migrate within 3 months
- **Migration Tool Usage**: 60%+ use automated migration tools
- **Issue Reports**: <5% of users report migration issues

### Quality Metrics

- **Breaking Changes**: All documented and testable
- **Migration Tools**: 95%+ success rate
- **Documentation**: Complete migration guides
- **Test Coverage**: 90%+ maintained

### Performance Metrics

- **CLI Performance**: <5% regression from v1.x
- **JSON Output**: <10ms overhead per command
- **Auto-Discovery**: No measurable startup impact

---

## Risk Mitigation

### Risk 1: Low Adoption Due to Breaking Changes

**Mitigation**:
- Comprehensive migration guides
- Automated migration tools
- 6-month v1.x support window
- Gradual deprecation warnings

### Risk 2: Migration Tool Failures

**Mitigation**:
- Extensive beta testing
- Multiple migration tool versions
- Manual migration guides as backup
- Community support for migration issues

### Risk 3: Missing Features

**Mitigation**:
- Feature parity checklist
- Beta testing period
- Community feedback collection
- Quick fixes in v2.0.1

### Risk 4: Performance Regression

**Mitigation**:
- Performance benchmarks
- Target: <5% regression
- Optimization before release
- Performance tests in CI/CD

---

## Timeline Summary

| Phase | Duration | Deliverables | Status |
|-------|----------|-------------|--------|
| Preparation | Week 1 | Documentation, planning | ⏳ Planned |
| Implementation | Week 2-4 | Breaking changes, new features | ⏳ Planned |
| Release | Week 5 | Final testing, release | ⏳ Planned |

**Total Duration**: 5 weeks (direct implementation, no migration needed)

---

## References

- **v1.x Refactoring Plan**: `docs/book/FULL_IMPLEMENTATION_PLAN.md`
- **80/20 Refactoring Plan**: `docs/book/80-20-refactor.plan.md`
- **Current State Analysis**: `docs/book/GGEN_CURRENT_STATE.md`
- **Migration Guide**: `docs/book/MIGRATION_GUIDE_V2.md` (to be created)

---

**Last Updated**: v2.0.0 plan created based on refactoring analysis. Ready for review and feedback.

