# ggen v2.0 Definition of Done

## Overview

Complete checklist for ggen v2.0 release. All items must be checked before release.

**Built on**: clap-noun-verb v3.0.0  
**Architecture**: Pure RDF-driven with business logic separation  
**Testing**: London TDD (outside-in)

---

## Architecture Requirements

### ✅ Core Architecture

- [ ] **Pure RDF-Driven Templates**
  - [ ] Templates contain only rendering logic (SPARQL queries + Tera syntax)
  - [ ] Zero hardcoded RDF references in templates
  - [ ] Zero hardcoded variables (`vars:` sections removed)
  - [ ] Zero hardcoded data in templates
  - [ ] All data comes from RDF via SPARQL queries

- [ ] **RDF as Single Source of Truth**
  - [ ] RDF files provided via CLI (`--rdf` or `--graph`)
  - [ ] Multiple RDF files can be merged
  - [ ] RDF files loaded into graph store at runtime
  - [ ] SPARQL SELECT queries extract data for templates
  - [ ] SPARQL CONSTRUCT queries transform RDF for templates

- [ ] **Business Logic Separation**
  - [ ] Templates generate CLI layer (thin wrapper)
  - [ ] CLI layer delegates to business logic files
  - [ ] Business logic files editable by agents
  - [ ] Business logic files never regenerated
  - [ ] Clear separation: CLI (regenerated) vs Business Logic (editable)

- [ ] **Frozen Sections**
  - [ ] `{% frozen %}` / `{% endfrozen %}` syntax in templates
  - [ ] `🔒 FROZEN START` / `🔒 FROZEN END` markers in generated code
  - [ ] Automatic detection of frozen sections during regeneration
  - [ ] Frozen content preserved during regeneration
  - [ ] Frozen sections merge with template changes

- [ ] **Filesystem Routing**
  - [ ] Automatic template discovery from `templates/` directory
  - [ ] Automatic RDF discovery from `domain/` directory
  - [ ] Automatic query discovery from `queries/` directory (optional)
  - [ ] Convention-based output path inference
  - [ ] Minimal `ggen.toml` configuration required

---

## CLI Integration with clap-noun-verb v3.0.0

### ✅ Command Migration

- [ ] **Command Syntax**
  - [ ] All commands use `ggen template generate` format
  - [ ] Auto-discovery via `#[verb]` attributes
  - [ ] Type inference for CLI arguments
  - [ ] JSON output by default
  - [ ] Proper error handling with Result types

- [ ] **Command Implementation**
  - [ ] `template generate` - Generate from template + RDF
  - [ ] `template list` - List available templates
  - [ ] `template show` - Show template content
  - [ ] `template validate` - Validate template syntax
  - [ ] `project init` - Initialize project with `ggen.toml`
  - [ ] `project generate` - Generate from project config

- [ ] **Argument Handling**
  - [ ] `--template` - Template path (required)
  - [ ] `--rdf` or `--graph` - RDF input (required)
  - [ ] `--output` - Output directory (optional, inferred from RDF)
  - [ ] `--regenerate` - Regenerate with frozen section preservation
  - [ ] `--watch` - Watch mode (regenerate on changes)
  - [ ] `--delta` - Delta generation (only changed files)

---

## Breaking Changes

### ✅ v1.x → v2.0 Migration

- [ ] **Removed Features**
  - [ ] `rdf:` in template frontmatter removed
  - [ ] `vars:` in template frontmatter removed
  - [ ] `--var` CLI flags removed
  - [ ] `ggen gen` command removed (use `ggen template generate`)

- [ ] **New Requirements**
  - [ ] `--rdf` or `--graph` flag required for RDF input
  - [ ] RDF files must be provided via CLI
  - [ ] Templates must be pure (no hardcoded data)

- [ ] **Migration Tools**
  - [ ] Template migration tool (v1.x → v2.0)
  - [ ] RDF extraction tool (extract hardcoded data to RDF)
  - [ ] Migration guide documentation
  - [ ] Breaking changes documentation

---

## Testing Requirements (London TDD)

### ✅ Integration Tests (20% of tests)

- [ ] **End-to-End Tests**
  - [ ] Full generation flow: CLI → Template Engine → RDF Processor → File System
  - [ ] CLI command execution with real components
  - [ ] File generation verification
  - [ ] Business logic separation verification
  - [ ] Frozen section preservation verification
  - [ ] Error handling and recovery

- [ ] **Test Scenarios**
  - [ ] Generate from template + RDF
  - [ ] Regenerate with frozen sections preserved
  - [ ] Multiple RDF files merged correctly
  - [ ] Filesystem routing discovers templates
  - [ ] Project config generation

### ✅ Component Tests (60% of tests)

- [ ] **Component Interaction Tests**
  - [ ] Template Engine + RDF Processor (with mocked RDF)
  - [ ] Template Engine + Frozen Merger (with mocked file system)
  - [ ] Template Engine + File Writer (with mocked file system)
  - [ ] Business Logic Separator (with mocked template engine)
  - [ ] Filesystem Router (with mocked file system)

- [ ] **Mock Boundaries**
  - [ ] Mock SPARQL executor for RDF queries
  - [ ] Mock file system for file operations
  - [ ] Mock frozen preserver for frozen section handling
  - [ ] Behavior verification via mocks
  - [ ] Interaction verification via mocks

### ✅ Unit Tests (20% of tests)

- [ ] **Isolated Unit Tests**
  - [ ] SPARQL query parsing
  - [ ] Template syntax parsing
  - [ ] Frozen marker detection
  - [ ] Path resolution logic
  - [ ] RDF triple processing
  - [ ] CONSTRUCT query transformation

- [ ] **Edge Cases**
  - [ ] Invalid SPARQL queries
  - [ ] Missing frozen markers
  - [ ] Invalid file paths
  - [ ] Empty RDF files
  - [ ] Malformed templates
  - [ ] Error conditions

### ✅ Test Coverage

- [ ] **Coverage Metrics**
  - [ ] Integration tests: 80%+ coverage of critical paths
  - [ ] Component tests: 90%+ coverage of component interactions
  - [ ] Unit tests: 95%+ coverage of isolated units
  - [ ] Overall coverage: 85%+ code coverage

- [ ] **Test Quality**
  - [ ] All tests follow London TDD (outside-in)
  - [ ] All tests use appropriate test doubles (mocks/stubs/fakes)
  - [ ] All tests verify behavior, not implementation
  - [ ] All tests are fast (<100ms each)
  - [ ] Full test suite completes in <1 second

---

## Documentation Requirements

### ✅ Architecture Documentation

- [ ] **Core Documents**
  - [ ] [GGEN_V2_TEMPLATE_ARCHITECTURE.md](GGEN_V2_TEMPLATE_ARCHITECTURE.md) - Complete
  - [ ] [GGEN_V2_BUSINESS_LOGIC_SEPARATION.md](GGEN_V2_BUSINESS_LOGIC_SEPARATION.md) - Complete
  - [ ] [GGEN_V2_PROJECT_CONFIG.md](GGEN_V2_PROJECT_CONFIG.md) - Complete
  - [ ] [GGEN_V2_FILESYSTEM_ROUTING.md](GGEN_V2_FILESYSTEM_ROUTING.md) - Complete
  - [ ] [GGEN_V2_ARCHITECTURE_DIAGRAMS.puml](GGEN_V2_ARCHITECTURE_DIAGRAMS.puml) - Complete
  - [ ] [GGEN_V2_LONDON_TDD_DIAGRAMS.puml](GGEN_V2_LONDON_TDD_DIAGRAMS.puml) - Complete

- [ ] **Documentation Quality**
  - [ ] All documents cross-referenced
  - [ ] All examples are accurate and tested
  - [ ] All command syntax is consistent
  - [ ] All patterns are clearly explained
  - [ ] Migration guide included

### ✅ API Documentation

- [ ] **Public API Documentation**
  - [ ] All public types documented
  - [ ] All public functions documented
  - [ ] All public traits documented
  - [ ] All examples in documentation compile
  - [ ] All examples in documentation work

- [ ] **Documentation Standards**
  - [ ] Rust doc comments on all public items
  - [ ] Examples in doc comments
  - [ ] Error documentation
  - [ ] Usage examples
  - [ ] Best practices guide

---

## Code Quality Requirements

### ✅ Code Standards

- [ ] **Error Handling**
  - [ ] Zero `unwrap()` or `expect()` in production code
  - [ ] All functions return `Result<T>` types
  - [ ] All errors use structured error types
  - [ ] All errors have meaningful messages
  - [ ] Error propagation is correct

- [ ] **Code Organization**
  - [ ] Clear module boundaries
  - [ ] Proper separation of concerns
  - [ ] No circular dependencies
  - [ ] Clean interfaces between components
  - [ ] Proper use of traits for abstraction

- [ ] **Performance**
  - [ ] Thin wrapper over clap (zero-cost abstractions)
  - [ ] Efficient RDF query execution
  - [ ] Fast template rendering
  - [ ] Efficient file operations
  - [ ] No unnecessary allocations

### ✅ Linting and Formatting

- [ ] **Code Quality Checks**
  - [ ] `cargo clippy` passes with no warnings
  - [ ] `cargo fmt` formats all code consistently
  - [ ] All code follows Rust best practices
  - [ ] No dead code warnings
  - [ ] No unused imports

---

## Feature Completeness

### ✅ Core Features

- [ ] **Template Generation**
  - [ ] Pure RDF-driven template rendering
  - [ ] SPARQL SELECT query execution
  - [ ] SPARQL CONSTRUCT query execution
  - [ ] Template variable substitution
  - [ ] File tree generation

- [ ] **Business Logic Separation**
  - [ ] CLI layer generation
  - [ ] Business logic skeleton generation
  - [ ] Business logic file preservation
  - [ ] Automatic separation logic

- [ ] **Frozen Sections**
  - [ ] `{% frozen %}` tag parsing
  - [ ] Frozen marker generation
  - [ ] Frozen section detection
  - [ ] Frozen content preservation
  - [ ] Frozen section merging

- [ ] **Filesystem Routing**
  - [ ] Template auto-discovery
  - [ ] RDF file auto-discovery
  - [ ] Query auto-discovery
  - [ ] Output path inference
  - [ ] Convention-based routing

- [ ] **Project Configuration**
  - [ ] `ggen.toml` parsing
  - [ ] Project-level queries
  - [ ] Template configuration
  - [ ] RDF file configuration
  - [ ] Output directory configuration

---

## Integration Requirements

### ✅ clap-noun-verb v3.0.0 Integration

- [ ] **Command Registration**
  - [ ] All commands use `#[verb]` attributes
  - [ ] Auto-discovery works correctly
  - [ ] Type inference works correctly
  - [ ] JSON output works correctly
  - [ ] Error handling works correctly

- [ ] **Argument Handling**
  - [ ] Required arguments work correctly
  - [ ] Optional arguments work correctly
  - [ ] Flag arguments work correctly
  - [ ] Multiple RDF files work correctly
  - [ ] Output directory works correctly

### ✅ External Dependencies

- [ ] **Dependencies Verified**
  - [ ] clap-noun-verb v3.0.0 dependency
  - [ ] Tera template engine dependency
  - [ ] SPARQL/RDF library dependencies
  - [ ] All dependencies up to date
  - [ ] All dependencies have compatible licenses

---

## Performance Requirements

### ✅ Performance Targets

- [ ] **Test Performance**
  - [ ] Full test suite completes in <1 second
  - [ ] Individual tests complete in <100ms
  - [ ] Parallel test execution works
  - [ ] Test caching works

- [ ] **Runtime Performance**
  - [ ] Template rendering: <100ms per file
  - [ ] RDF query execution: <50ms per query
  - [ ] File generation: <200ms per file
  - [ ] Full generation: <1 second for typical project
  - [ ] Memory usage: <100MB for typical project

---

## Security Requirements

### ✅ Security Checks

- [ ] **Path Security**
  - [ ] No directory traversal vulnerabilities
  - [ ] Path validation implemented
  - [ ] Safe file operations
  - [ ] Proper permissions handling

- [ ] **Input Validation**
  - [ ] RDF file validation
  - [ ] Template file validation
  - [ ] SPARQL query validation
  - [ ] Command argument validation

---

## Release Requirements

### ✅ Pre-Release Checklist

- [ ] **Version Management**
  - [ ] Version bumped to 2.0.0
  - [ ] CHANGELOG.md updated
  - [ ] Release notes prepared
  - [ ] Breaking changes documented

- [ ] **Release Artifacts**
  - [ ] Release binaries built
  - [ ] Release binaries tested
  - [ ] Release binaries signed (if applicable)
  - [ ] Documentation built and verified

- [ ] **Quality Assurance**
  - [ ] All tests pass
  - [ ] All linters pass
  - [ ] All documentation complete
  - [ ] Migration guide complete
  - [ ] Examples work correctly

---

## Success Metrics

### ✅ Metrics

- [ ] **Code Metrics**
  - [ ] 85%+ code coverage
  - [ ] 0 `unwrap()` or `expect()` in production code
  - [ ] 0 linting warnings
  - [ ] 0 compilation warnings

- [ ] **Feature Metrics**
  - [ ] 100% of core features implemented
  - [ ] 100% of breaking changes documented
  - [ ] 100% of migration paths provided
  - [ ] 100% of documentation complete

- [ ] **Performance Metrics**
  - [ ] Full test suite: <1 second
  - [ ] Template rendering: <100ms per file
  - [ ] Full generation: <1 second for typical project

---

## Validation Checklist

Before marking ggen v2.0 as complete, verify:

- [ ] ✅ All architecture requirements met
- [ ] ✅ All CLI integration with clap-noun-verb v3.0.0 complete
- [ ] ✅ All breaking changes implemented
- [ ] ✅ All tests pass (London TDD)
- [ ] ✅ All documentation complete
- [ ] ✅ All code quality requirements met
- [ ] ✅ All features implemented
- [ ] ✅ All integration requirements met
- [ ] ✅ All performance targets met
- [ ] ✅ All security requirements met
- [ ] ✅ All release requirements met
- [ ] ✅ All success metrics achieved

---

## Definition of Done Summary

**ggen v2.0 is considered complete when:**

1. ✅ **Architecture**: Pure RDF-driven templates, business logic separation, frozen sections, filesystem routing
2. ✅ **Integration**: Built on clap-noun-verb v3.0.0 with auto-discovery and type inference
3. ✅ **Testing**: London TDD approach with 85%+ coverage
4. ✅ **Documentation**: Complete architecture documentation with examples
5. ✅ **Quality**: Zero unwrap/expect, proper error handling, clean code
6. ✅ **Performance**: Test suite <1s, generation <1s for typical project
7. ✅ **Breaking Changes**: All v1.x patterns removed, migration guide provided
8. ✅ **Release**: Version 2.0.0 with all artifacts ready

---

**Last Updated**: Definition of Done for ggen v2.0

