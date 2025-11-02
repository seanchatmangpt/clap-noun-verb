# ggen CLI Refactoring Plan: Migration to clap-noun-verb v3.0.0

## Executive Summary

**Goal**: Migrate ggen's CLI from regular clap to clap-noun-verb v3.0.0 attribute macro API while maintaining backward compatibility and improving code organization.

**Timeline**: 4-6 weeks (phased approach)
**Risk Level**: Medium (requires careful backward compatibility testing)
**Expected Benefits**:
- **50% reduction** in CLI boilerplate code
- **Auto-discovery** eliminates manual command registration
- **Type inference** reduces argument definition boilerplate
- **JSON output** improves scriptability and automation
- **Better organization** with noun-verb pattern

**Based on**: [ggen GitHub Repository](https://github.com/seanchatmangpt/ggen)

---

## 1. Current State Analysis

### 1.1 Current CLI Structure

From the [ggen repository](https://github.com/seanchatmangpt/ggen), the current CLI has **13 subcommands**:

**AI Commands:**
- `ggen ai project` - Generate complete projects
- `ggen ai generate` - Generate templates from descriptions
- `ggen ai graph` - Generate RDF ontologies
- `ggen ai sparql` - Generate SPARQL queries

**Marketplace Commands:**
- `ggen market search` - Find packages
- `ggen market install` - Install package
- `ggen market list` - List installed packages
- `ggen market publish` - Publish package

**Template Commands:**
- `ggen gen` - Generate from template (with --vars)

**Utility Commands:**
- `ggen doctor` - Diagnostics
- `ggen help-me` - Personalized guidance
- `ggen project new` - Create new project
- `ggen hook create` - Create knowledge hooks

### 1.2 Current Architecture

```
ggen/
├── cli/              # Command-line interface (Clap-based)
├── ggen-core/        # Template engine, RDF/SPARQL processing
├── ggen-ai/          # AI providers (OpenAI, Anthropic, Ollama)
├── ggen-marketplace/ # Package management
├── node/             # Node.js NIF bindings
└── utils/            # Shared utilities
```

**Current Issues:**
- Verbose enum-based command structure
- Manual command registration required
- Scattered command definitions
- Harder to extend with new commands
- No automatic JSON output

---

## 2. Target State Design

### 2.1 Command Mapping to Noun-Verb Pattern

**Mapping Strategy:**

| Current Command | Target Noun-Verb | Notes |
|----------------|------------------|-------|
| `ggen ai project` | `ggen ai project` | ✅ Already matches |
| `ggen ai generate` | `ggen ai generate` | ✅ Already matches |
| `ggen ai graph` | `ggen ai graph` | ✅ Already matches |
| `ggen ai sparql` | `ggen ai sparql` | ✅ Already matches |
| `ggen market search` | `ggen marketplace search` | ⚠️ Rename `market` → `marketplace` |
| `ggen market install` | `ggen marketplace install` | ⚠️ Rename `market` → `marketplace` |
| `ggen market list` | `ggen marketplace list` | ⚠️ Rename `market` → `marketplace` |
| `ggen market publish` | `ggen marketplace publish` | ⚠️ Rename `market` → `marketplace` |
| `ggen gen` | `ggen template generate` | ⚠️ Group under `template` noun |
| `ggen doctor` | `ggen utils doctor` | ⚠️ Group under `utils` noun |
| `ggen help-me` | `ggen utils help-me` | ⚠️ Group under `utils` noun |
| `ggen project new` | `ggen project new` | ✅ Already matches |
| `ggen hook create` | `ggen hook create` | ✅ Already matches (or `utils hook create`) |

### 2.2 Target File Structure

```
ggen/
├── cli/
│   ├── mod.rs              # Main entry point with auto-discovery
│   ├── commands/
│   │   ├── mod.rs          # Re-export all command modules
│   │   ├── ai.rs           # AI commands (4 verbs)
│   │   ├── marketplace.rs  # Marketplace commands (4 verbs)
│   │   ├── template.rs     # Template commands (1 verb)
│   │   ├── project.rs      # Project commands (1 verb)
│   │   ├── hook.rs         # Hook commands (1 verb)
│   │   └── utils.rs        # Utility commands (2 verbs)
│   └── handlers/
│       ├── mod.rs          # Re-export all handlers
│       ├── ai_handlers.rs  # AI business logic
│       ├── marketplace_handlers.rs
│       └── template_handlers.rs
├── ggen-core/             # Unchanged
├── ggen-ai/               # Unchanged
├── ggen-marketplace/      # Unchanged
└── utils/                 # Unchanged
```

### 2.3 Target Command Structure

```
ggen (root)
├── ai (noun)
│   ├── project (verb)
│   ├── generate (verb)
│   ├── graph (verb)
│   └── sparql (verb)
├── marketplace (noun)
│   ├── search (verb)
│   ├── install (verb)
│   ├── list (verb)
│   └── publish (verb)
├── template (noun)
│   └── generate (verb)
├── project (noun)
│   └── new (verb)
├── hook (noun)
│   └── create (verb)
└── utils (noun)
    ├── doctor (verb)
    └── help-me (verb)
```

---

## 3. Migration Strategy

### 3.1 Phased Approach

**Phase 1: Foundation (Week 1)**
- Set up dependencies (`clap-noun-verb`, `clap-noun-verb-macros`, `serde`)
- Create new command file structure
- Implement one simple command as proof-of-concept (e.g., `utils doctor`)
- Verify auto-discovery works

**Phase 2: AI Commands (Week 2)**
- Migrate all AI commands (`ai project`, `ai generate`, `ai graph`, `ai sparql`)
- Maintain backward compatibility
- Add comprehensive tests
- Update documentation

**Phase 3: Marketplace Commands (Week 2-3)**
- Migrate marketplace commands
- Handle `market` → `marketplace` rename with aliases
- Test package management workflows
- Update integration tests

**Phase 4: Template & Project Commands (Week 3)**
- Migrate template generation
- Migrate project commands
- Test template workflows

**Phase 5: Utility Commands & Hooks (Week 4)**
- Migrate utility commands
- Migrate hook commands
- Complete command coverage

**Phase 6: Backward Compatibility & Testing (Week 5)**
- Implement command aliases for backward compatibility
- Comprehensive testing across all commands
- Performance validation
- Documentation updates

**Phase 7: Cleanup & Release (Week 6)**
- Remove old CLI code
- Final testing
- Release preparation

### 3.2 Backward Compatibility Strategy

**Critical**: Maintain 100% backward compatibility for user-facing commands.

**Aliases for Renamed Commands:**
```rust
// Old: ggen market search
// New: ggen marketplace search
// Alias: market → marketplace (at noun level)

// Old: ggen gen
// New: ggen template generate
// Alias: gen → template generate (at root level)
```

**Implementation Options:**

1. **Soft Deprecation** (Recommended):
   - New commands use new structure
   - Old commands still work via aliases
   - Show deprecation warnings
   - Remove in v2.0.0

2. **Hard Migration**:
   - Breaking change in major version
   - Not recommended for production tool

### 3.3 Risk Mitigation

**Risk 1: Breaking Existing Scripts**
- **Mitigation**: Comprehensive alias system
- **Testing**: Run all existing CI/CD scripts
- **Rollback**: Keep old code in feature branch

**Risk 2: Performance Regression**
- **Mitigation**: Benchmark before/after
- **Target**: <5% performance impact
- **Testing**: Run performance test suite

**Risk 3: Missing Features**
- **Mitigation**: Feature parity checklist
- **Testing**: Compare old vs new command output
- **Validation**: Run integration tests

**Risk 4: Test Failures**
- **Mitigation**: Update tests incrementally
- **Coverage**: Maintain 90%+ test coverage
- **Validation**: All 600+ tests must pass

---

## 4. Implementation Details

### 4.1 Dependencies

**Add to `Cargo.toml`:**

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap-noun-verb = "3.0.0"
clap-noun-verb-macros = "3.0.0"
serde = { version = "1.0", features = ["derive"] }
```

### 4.2 File Structure Example

**cli/commands/ai.rs:**

```rust
//! AI-powered generation

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
fn create_project(name: String, description: Option<String>, rust: bool) -> ProjectOutput {
    // Existing business logic from ggen-ai crate
    // ...
}

// CLI Layer (Input Validation + Output Shaping Only)
#[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
fn ai_project(
    name: String,
    description: Option<String>,
    rust: bool
) -> Result<ProjectOutput> {
    // Arguments automatically inferred from function signature
    Ok(create_project(name, description, rust))
}

#[derive(Serialize)]
struct ProjectOutput {
    name: String,
    description: Option<String>,
    rust: bool,
    success: bool,
    // ... other fields
}
```

**cli/mod.rs:**

```rust
mod commands;
mod handlers;

use clap_noun_verb::Result;

fn main() -> Result<()> {
    // Auto-discovers all #[verb] functions in commands/
    clap_noun_verb::run()
}
```

### 4.3 Command Aliases for Backward Compatibility

**Option 1: Macro-Level Aliases** (if supported):

```rust
// Marketplace commands with alias
#[verb("search", "marketplace")] // Explicit noun
#[verb("search", "market")]      // Alias noun
fn marketplace_search(query: String) -> Result<SearchResult> {
    // ...
}
```

**Option 2: Handler-Level Redirects** (manual):

```rust
// In old CLI entry point
match command {
    "market" => redirect_to("marketplace"),
    "gen" => redirect_to("template generate"),
    // ...
}
```

**Option 3: Separate Alias Layer** (recommended):

```rust
// cli/aliases.rs
pub fn register_aliases(registry: &mut CommandRegistry) {
    registry.alias("market", "marketplace");
    registry.alias("gen", "template generate");
    // ...
}
```

### 4.4 Business Logic Separation

**Before:**
```rust
// Old: Mixed CLI and business logic
fn handle_ai_project(args: &ArgMatches) {
    let name = args.get_one::<String>("name").unwrap();
    // Business logic here
    create_project_files(name);
}
```

**After:**
```rust
// Business Logic Layer (ggen-core or separate module)
pub fn create_project(name: String, rust: bool) -> ProjectOutput {
    // Pure function - can be tested independently
    // Can be used by CLI, API, Web UI, etc.
}

// CLI Layer (commands/ai.rs)
#[verb]
fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, rust))
}
```

---

## 5. Testing Strategy

### 5.1 Unit Tests

**Test Structure:**
```rust
// cli/commands/tests/ai_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ai_project_creates_output() -> Result<()> {
        // Test business logic directly
        let output = create_project("test-app".to_string(), true);
        assert!(output.success);
        Ok(())
    }
    
    #[test]
    fn test_ai_project_command_parsing() -> Result<()> {
        // Test CLI parsing with run_cli_with_args
        // ...
        Ok(())
    }
}
```

### 5.2 Integration Tests

**Test All Commands:**
```rust
// tests/cli_integration_tests.rs
#[test]
fn test_all_commands_backward_compatible() -> Result<()> {
    // Test old command syntax still works
    test_command("ggen market search rust", "ggen marketplace search rust")?;
    test_command("ggen gen template.tmpl", "ggen template generate template.tmpl")?;
    Ok(())
}

#[test]
fn test_json_output_format() -> Result<()> {
    // Verify all commands return valid JSON
    // ...
}
```

### 5.3 Compatibility Tests

**Test Suite:**
1. ✅ All 13 existing commands work
2. ✅ All command aliases work
3. ✅ Argument parsing matches exactly
4. ✅ Help output matches (or improves)
5. ✅ Error messages match (or improve)
6. ✅ JSON output format is consistent
7. ✅ Performance within 5% of baseline

### 5.4 Regression Tests

**Validate Against Existing Tests:**
- Run all 600+ existing tests
- Ensure 90%+ test coverage maintained
- Performance tests pass (<3s generation time)
- Stress tests pass

---

## 6. Rollout Plan

### 6.1 Pre-Migration Checklist

- [ ] Create feature branch: `refactor/cli-noun-verb`
- [ ] Set up CI/CD for new branch
- [ ] Document current command structure
- [ ] Create test matrix for all commands
- [ ] Set performance baselines
- [ ] Create rollback plan

### 6.2 Migration Execution

**Week-by-Week Breakdown:**

**Week 1: Foundation**
- [ ] Add dependencies
- [ ] Create file structure
- [ ] Implement `utils doctor` (proof-of-concept)
- [ ] Verify auto-discovery
- [ ] Write tests

**Week 2: AI Commands**
- [ ] Migrate `ai project`
- [ ] Migrate `ai generate`
- [ ] Migrate `ai graph`
- [ ] Migrate `ai sparql`
- [ ] Integration tests
- [ ] Performance benchmarks

**Week 2-3: Marketplace Commands**
- [ ] Migrate `marketplace search`
- [ ] Migrate `marketplace install`
- [ ] Migrate `marketplace list`
- [ ] Migrate `marketplace publish`
- [ ] Implement `market` → `marketplace` alias
- [ ] Test package workflows

**Week 3: Template & Project**
- [ ] Migrate `template generate`
- [ ] Migrate `project new`
- [ ] Implement `gen` → `template generate` alias
- [ ] Test template workflows

**Week 4: Utilities & Hooks**
- [ ] Migrate `utils doctor`
- [ ] Migrate `utils help-me`
- [ ] Migrate `hook create`
- [ ] Complete command coverage

**Week 5: Compatibility & Testing**
- [ ] Implement all aliases
- [ ] Comprehensive testing
- [ ] Performance validation
- [ ] Documentation updates
- [ ] Update CI/CD scripts

**Week 6: Cleanup & Release**
- [ ] Remove old CLI code
- [ ] Final testing
- [ ] Release notes
- [ ] Documentation updates
- [ ] Release candidate

### 6.3 Post-Migration

**Immediate:**
- Monitor error logs
- Collect user feedback
- Fix critical issues

**Short-term (1-2 weeks):**
- Address user-reported issues
- Performance optimizations
- Documentation improvements

**Long-term:**
- Deprecate old command syntax
- Remove aliases in v2.0.0
- Add new commands using noun-verb pattern

---

## 7. Success Metrics

### 7.1 Code Metrics

**Target Improvements:**
- **CLI code reduction**: 50%+ reduction in boilerplate
- **Command registration**: Eliminated (auto-discovery)
- **Type safety**: Maintained or improved
- **Test coverage**: Maintain 90%+

**Measurable:**
- Lines of code in `cli/` directory: Before vs After
- Number of enum variants: Before vs After
- Number of match statements: Before vs After

### 7.2 Performance Metrics

**Targets:**
- Command parsing: <5% slower
- Command execution: <5% slower
- Memory usage: <10% increase
- Binary size: <5% increase

### 7.3 User Experience Metrics

**Targets:**
- **Backward compatibility**: 100% of existing commands work
- **JSON output**: All commands return valid JSON
- **Help quality**: Improved or maintained
- **Error messages**: Improved or maintained

---

## 8. Risk Assessment

### 8.1 High Risk Areas

**1. Marketplace Rename (`market` → `marketplace`)**
- **Risk**: Breaking existing scripts
- **Mitigation**: Alias support + deprecation warnings
- **Testing**: Run all marketplace-related scripts

**2. Template Command Rename (`gen` → `template generate`)**
- **Risk**: Breaking existing workflows
- **Mitigation**: Alias support + clear migration guide
- **Testing**: Test all template generation workflows

**3. JSON Output Format Changes**
- **Risk**: Breaking scripts that parse output
- **Mitigation**: Consistent JSON structure, document format
- **Testing**: Test all scripts that parse CLI output

### 8.2 Medium Risk Areas

**1. Performance Regression**
- **Risk**: Slower command execution
- **Mitigation**: Benchmarking, optimization
- **Testing**: Performance test suite

**2. Missing Command Features**
- **Risk**: Incomplete migration
- **Mitigation**: Feature parity checklist
- **Testing**: Compare old vs new behavior

### 8.3 Low Risk Areas

**1. Internal Code Organization**
- **Risk**: Development friction
- **Mitigation**: Clear file structure
- **Testing**: Developer feedback

**2. Documentation Updates**
- **Risk**: Outdated docs
- **Mitigation**: Update docs incrementally
- **Testing**: Documentation review

---

## 9. Rollback Plan

### 9.1 If Migration Fails

**Immediate Actions:**
1. Revert to feature branch
2. Analyze failure points
3. Document issues
4. Plan fixes

**Rollback Triggers:**
- Critical bugs in production
- Performance regression >10%
- >5% of tests failing
- Breaking backward compatibility

### 9.2 Partial Rollback

**Option: Keep Hybrid Approach**
- New commands use clap-noun-verb
- Old commands stay as-is
- Migrate incrementally

---

## 10. Documentation Updates

### 10.1 User Documentation

**Update:**
- CLI reference guide
- Command examples
- Migration guide for renamed commands
- JSON output format specification

### 10.2 Developer Documentation

**Update:**
- Architecture documentation
- Adding new commands guide
- Testing guide
- Code organization guide

### 10.3 API Documentation

**Update:**
- Public API reference
- Command handler interfaces
- Error handling guide

---

## 11. Timeline Summary

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1: Foundation | Week 1 | Dependencies, structure, proof-of-concept |
| Phase 2: AI Commands | Week 2 | All AI commands migrated |
| Phase 3: Marketplace | Week 2-3 | All marketplace commands + aliases |
| Phase 4: Template & Project | Week 3 | Template and project commands |
| Phase 5: Utilities | Week 4 | Utility commands complete |
| Phase 6: Testing | Week 5 | All tests pass, compatibility verified |
| Phase 7: Release | Week 6 | Cleanup, documentation, release |

**Total Duration**: 4-6 weeks

---

## 12. Next Steps

### Immediate Actions

1. **Review this plan** with the team
2. **Create feature branch**: `refactor/cli-noun-verb`
3. **Set up dependencies** and basic structure
4. **Implement proof-of-concept** (`utils doctor`)
5. **Validate approach** before full migration

### Decision Points

1. **Alias strategy**: Macro-level vs handler-level vs separate layer
2. **Rename strategy**: Soft deprecation vs hard migration
3. **Timeline**: Accelerate vs extend based on complexity
4. **Scope**: Full migration vs hybrid approach

---

## 13. Conclusion

This refactoring plan provides a comprehensive roadmap for migrating ggen's CLI from regular clap to clap-noun-verb v3.0.0. The phased approach minimizes risk while delivering significant improvements in code organization, maintainability, and developer experience.

**Key Success Factors:**
- ✅ Maintain 100% backward compatibility
- ✅ Comprehensive testing at each phase
- ✅ Clear communication with users
- ✅ Incremental migration reduces risk
- ✅ Performance validation throughout

**Expected Outcomes:**
- Cleaner, more maintainable CLI code
- Easier to add new commands
- Better developer experience
- JSON output for automation
- Foundation for future CLI enhancements

---

**References:**
- [ggen GitHub Repository](https://github.com/seanchatmangpt/ggen)
- [clap-noun-verb Documentation](https://github.com/seanchatmangpt/clap-noun-verb)
- Porting Guide: `docs/book/src/porting-commands.md`

