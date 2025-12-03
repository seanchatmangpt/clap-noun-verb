# ggen Noun-Verb Analysis: Complete Command Mapping

## Executive Summary

This document provides a complete analysis of all ggen commands from a noun-verb perspective, mapping the current enum-based structure to the clap-noun-verb v3.0.0 attribute macro API.

**Total Commands**: **12 top-level nouns** with **60+ verbs** (much larger than initially estimated)

---

## Command Structure Overview

### Current Structure (enum-based)

```
ggen
├── ai (noun)              # 10 verbs
├── market (noun)          # 14 verbs  
├── project (noun)         # 10 verbs
├── template (noun)        # 6 verbs
├── hook (noun)            # 5 verbs
├── graph (noun)           # 7 verbs
├── audit (noun)           # 3 verbs
├── ci (noun)              # 4 verbs
├── lifecycle (noun)       # 9 verbs
├── shell (noun)           # 1 verb (completion commented out)
├── doctor (standalone)    # 1 command (no verb)
└── help-me (standalone)   # 1 command (no verb)
```

---

## Complete Noun-Verb Mapping

### 1. `ai` Noun - AI-Powered Generation

**Current**: `ggen ai <verb>`

**Verbs** (10 total):
- `project` - Generate complete template projects
- `generate` - Generate templates from descriptions
- `graph` - Generate RDF graphs using AI
- `sparql` - Generate SPARQL queries using AI
- `demo` - Run the AI template demo (no args)
- `frontmatter` - Generate frontmatter using AI
- `models` - List available AI models
- `validate` - Validate templates
- `from-source` - Generate template from existing source file
- `config` - Configure AI settings (likely)

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen ai project      → ai.project
ggen ai generate     → ai.generate
ggen ai graph        → ai.graph
ggen ai sparql       → ai.sparql
ggen ai demo         → ai.demo
ggen ai frontmatter  → ai.frontmatter
ggen ai models       → ai.models
ggen ai validate     → ai.validate
ggen ai from-source  → ai.from_source
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/ai.rs
//! AI-powered template generation and analysis

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Business Logic Layer
fn create_project(name: String, description: Option<String>, rust: bool) -> ProjectOutput {
    // ...
}

// CLI Layer
#[verb("project", "ai")]
fn ai_project(name: String, description: Option<String>, rust: bool) -> Result<ProjectOutput> {
    Ok(create_project(name, description, rust))
}

#[verb("generate", "ai")]
fn ai_generate(description: String, output: Option<String>) -> Result<GenerateOutput> {
    // ...
}

#[verb("graph", "ai")]
fn ai_graph(description: String, output: Option<String>) -> Result<GraphOutput> {
    // ...
}

// ... 7 more verbs
```

---

### 2. `marketplace` Noun - Marketplace Operations

**Current**: `ggen market <verb>`

**Verbs** (14 total):
- `search` - Search for gpacks in the marketplace
- `add` - Add a gpack from marketplace to project
- `remove` - Remove a gpack from project
- `list` - List all installed gpacks
- `update` - Update gpacks to latest versions
- `info` - Show detailed information about a gpack
- `recommend` - Get personalized package recommendations
- `offline` - Browse marketplace offline using cached data
- `cache` - Manage marketplace cache (clear, stats, validate)
- `sync` - Synchronize with remote marketplace
- `categories` - Show popular categories and keywords
- `publish` - Publish a gpack to the marketplace
- `unpublish` - Unpublish a gpack from marketplace
- `natural` - Natural language search using AI

**Noun-Verb Mapping**:
```rust
// ⚠️ Rename: market → marketplace (breaking change in v2.0.0)
ggen market search      → marketplace.search
ggen market add         → marketplace.add
ggen market remove      → marketplace.remove
ggen market list        → marketplace.list
ggen market update      → marketplace.update
ggen market info        → marketplace.info
ggen market recommend   → marketplace.recommend
ggen market offline     → marketplace.offline
ggen market cache       → marketplace.cache
ggen market sync        → marketplace.sync
ggen market categories  → marketplace.categories
ggen market publish     → marketplace.publish
ggen market unpublish   → marketplace.unpublish
ggen market natural     → marketplace.natural
```

**Migration Strategy**:
- v1.4.0: Add alias `market` → `marketplace` (backward compatible)
- v2.0.0: Remove alias, require `marketplace`

**Migration to clap-noun-verb**:
```rust
// cli/commands/marketplace.rs
//! Marketplace operations for gpacks

#[verb("search", "marketplace")]
fn marketplace_search(query: String, category: Option<String>) -> Result<SearchOutput> {
    // ...
}

#[verb("add", "marketplace")]
fn marketplace_add(package: String, version: Option<String>) -> Result<AddOutput> {
    // ...
}

// ... 12 more verbs
```

---

### 3. `project` Noun - Project Scaffolding

**Current**: `ggen project <verb>`

**Verbs** (10 total):
- `new` - Create a new project from scratch (bootstrap)
- `gen` - Generate artifacts from a template directly
- `plan` - Create a machine-readable plan without applying (dry-run)
- `apply` - Apply a previously generated plan to filesystem
- `diff` - Show unified diff of what a generation would change
- `test` - Run golden file snapshot tests for templates
- `freeze` - Add freeze blocks to generated files for immutability
- `inject` - Inject code idempotently into existing files
- `validate` - Validate plan files or generated output
- `watch` - Watch templates and continuously regenerate on changes

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen project new       → project.new
ggen project gen       → project.gen
ggen project plan      → project.plan
ggen project apply     → project.apply
ggen project diff      → project.diff
ggen project test      → project.test
ggen project freeze    → project.freeze
ggen project inject    → project.inject
ggen project validate  → project.validate
ggen project watch     → project.watch
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/project.rs
//! Project scaffolding and generation

#[verb("new", "project")]
fn project_new(name: String, project_type: String, framework: Option<String>) -> Result<ProjectOutput> {
    // ...
}

#[verb("gen", "project")]
fn project_gen(template: String, vars: Vec<String>) -> Result<GenOutput> {
    // ...
}

// ... 8 more verbs
```

---

### 4. `template` Noun - Template Management

**Current**: `ggen template <verb>`

**Verbs** (6 total):
- `new` - Create a new template
- `list` - List available templates
- `show` - Show template details
- `lint` - Lint a template
- `regenerate` - Regenerate code using delta-driven projection
- `generate-tree` - Generate file tree from template

**Note**: There's also a legacy `ggen gen` command that should map to `template generate` (or `project gen`? - needs clarification)

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern!
ggen template new           → template.new
ggen template list          → template.list
ggen template show          → template.show
ggen template lint          → template.lint
ggen template regenerate    → template.regenerate
ggen template generate-tree → template.generate_tree

// ⚠️ Legacy command: ggen gen → template.generate (or project.gen?)
ggen gen <template>         → template.generate (or remove in v2.0.0?)
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/template.rs
//! Template management

#[verb("new", "template")]
fn template_new(name: String, from: Option<String>) -> Result<TemplateOutput> {
    // ...
}

#[verb("list", "template")]
fn template_list(format: Option<String>) -> Result<ListOutput> {
    // ...
}

// ... 4 more verbs
```

---

### 5. `hook` Noun - Knowledge Hooks

**Current**: `ggen hook <verb>`

**Verbs** (5 total):
- `create` - Create a new knowledge hook for automatic graph regeneration
- `list` - List all knowledge hooks (active, disabled, or all)
- `run` - Manually run a knowledge hook (for testing)
- `remove` - Remove a knowledge hook and uninstall it
- `validate` - Validate a hook's configuration without running it

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen hook create   → hook.create
ggen hook list     → hook.list
ggen hook run      → hook.run
ggen hook remove   → hook.remove
ggen hook validate → hook.validate
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/hook.rs
//! Knowledge hooks for autonomic graph regeneration

#[verb("create", "hook")]
fn hook_create(name: String, trigger: String, template: String) -> Result<HookOutput> {
    // ...
}

#[verb("list", "hook")]
fn hook_list(active: Option<bool>) -> Result<ListOutput> {
    // ...
}

// ... 3 more verbs
```

---

### 6. `graph` Noun - RDF Graph Operations

**Current**: `ggen graph <verb>`

**Verbs** (7 total):
- `query` - Execute SPARQL query against RDF graph
- `load` - Load RDF data into graph
- `export` - Export RDF graph
- `validate` - Validate graph against SHACL shapes
- `stats` - Show graph statistics
- `diff` - Compare two RDF graphs and show differences
- `snapshot` - Manage graph snapshots for delta-driven projection

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen graph query     → graph.query
ggen graph load      → graph.load
ggen graph export    → graph.export
ggen graph validate  → graph.validate
ggen graph stats     → graph.stats
ggen graph diff      → graph.diff
ggen graph snapshot  → graph.snapshot
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/graph.rs
//! RDF graph operations

#[verb("query", "graph")]
fn graph_query(query: String, graph: Option<String>) -> Result<QueryOutput> {
    // ...
}

#[verb("load", "graph")]
fn graph_load(file: String, format: Option<String>) -> Result<LoadOutput> {
    // ...
}

// ... 5 more verbs
```

---

### 7. `audit` Noun - Security & Performance Auditing

**Current**: `ggen audit <verb>`

**Verbs** (3 total):
- `hazard` - Check for hazardous patterns and anti-patterns
- `security` - Perform security vulnerability scans
- `performance` - Analyze performance characteristics

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen audit hazard      → audit.hazard
ggen audit security    → audit.security
ggen audit performance → audit.performance
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/audit.rs
//! Security and performance auditing

#[verb("hazard", "audit")]
fn audit_hazard(path: String, recursive: bool) -> Result<HazardOutput> {
    // ...
}

#[verb("security", "audit")]
fn audit_security(path: String, level: Option<String>) -> Result<SecurityOutput> {
    // ...
}

#[verb("performance", "audit")]
fn audit_performance(path: String, benchmarks: bool) -> Result<PerformanceOutput> {
    // ...
}
```

---

### 8. `ci` Noun - CI/CD Operations

**Current**: `ggen ci <verb>`

**Verbs** (4 total):
- `pages` - Manage GitHub Pages deployment
- `release` - Run release workflows locally with act
- `workflow` - Manage GitHub Actions workflows
- `trigger` - Trigger CI/CD workflows manually

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen ci pages     → ci.pages
ggen ci release   → ci.release
ggen ci workflow  → ci.workflow
ggen ci trigger   → ci.trigger
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/ci.rs
//! CI/CD operations and GitHub integration

#[verb("pages", "ci")]
fn ci_pages(action: String, branch: Option<String>) -> Result<PagesOutput> {
    // ...
}

#[verb("release", "ci")]
fn ci_release(version: String, dry_run: bool) -> Result<ReleaseOutput> {
    // ...
}

#[verb("workflow", "ci")]
fn ci_workflow(action: String, workflow: String) -> Result<WorkflowOutput> {
    // ...
}

#[verb("trigger", "ci")]
fn ci_trigger(workflow: String, ref: Option<String>) -> Result<TriggerOutput> {
    // ...
}
```

---

### 9. `lifecycle` Noun - Universal Lifecycle Management

**Current**: `ggen lifecycle <verb>`

**Verbs** (9 total):
- `list` - List all available lifecycle phases
- `show` - Show details of a specific phase
- `run` - Run a single lifecycle phase
- `pipeline` - Run multiple phases in sequence
- `readiness` - Check production readiness status
- `readiness-update` - Update production readiness status
- `placeholders` - Show placeholders that need implementation
- `validate` - Validate production readiness for deployment

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern perfectly!
ggen lifecycle list             → lifecycle.list
ggen lifecycle show             → lifecycle.show
ggen lifecycle run              → lifecycle.run
ggen lifecycle pipeline         → lifecycle.pipeline
ggen lifecycle readiness        → lifecycle.readiness
ggen lifecycle readiness-update → lifecycle.readiness_update
ggen lifecycle placeholders     → lifecycle.placeholders
ggen lifecycle validate         → lifecycle.validate
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/lifecycle.rs
//! Universal lifecycle management

#[verb("list", "lifecycle")]
fn lifecycle_list(root: Option<String>) -> Result<ListOutput> {
    // ...
}

#[verb("show", "lifecycle")]
fn lifecycle_show(phase: String, root: Option<String>) -> Result<ShowOutput> {
    // ...
}

#[verb("run", "lifecycle")]
fn lifecycle_run(phase: String, root: Option<String>, env: Option<String>) -> Result<RunOutput> {
    // ...
}

// ... 5 more verbs
```

---

### 10. `shell` Noun - Shell Integration

**Current**: `ggen shell <verb>`

**Verbs** (1 total - completion commented out):
- `init` - Initialize shell integration
- ~~`completion`~~ - Generate shell completion scripts (commented out)

**Noun-Verb Mapping**:
```rust
// ✅ Already matches noun-verb pattern!
ggen shell init → shell.init
// ggen shell completion → shell.completion (disabled)
```

**Migration to clap-noun-verb**:
```rust
// cli/commands/shell.rs
//! Shell integration and completion

#[verb("init", "shell")]
fn shell_init(shell: String) -> Result<InitOutput> {
    // ...
}
```

---

### 11. `doctor` - Standalone Command

**Current**: `ggen doctor` (root-level, no verb)

**Options**:
- `--verbose` - Show verbose output with fix instructions

**Migration Options**:

**Option 1: Group under `utils` noun** (recommended):
```rust
// ⚠️ Breaking change: doctor → utils.doctor
ggen doctor → utils.doctor
```

**Option 2: Keep as standalone** (not recommended for consistency):
```rust
// Keep as root-level command (doesn't fit noun-verb pattern)
ggen doctor → doctor (no verb, just a command)
```

**Recommendation**: Group under `utils` noun for consistency with noun-verb pattern.

**Migration to clap-noun-verb**:
```rust
// cli/commands/utils.rs
//! Utility commands

#[verb("doctor", "utils")]
fn utils_doctor(verbose: bool) -> Result<DoctorOutput> {
    // ...
}
```

---

### 12. `help-me` - Standalone Command

**Current**: `ggen help-me` (root-level, no verb)

**Options**:
- `COMMAND` - Command to get help for
- `--tips` - Show tips based on usage patterns

**Migration Options**:

**Option 1: Group under `utils` noun** (recommended):
```rust
// ⚠️ Breaking change: help-me → utils.help_me
ggen help-me → utils.help_me
```

**Option 2: Keep as standalone** (not recommended):
```rust
// Keep as root-level (doesn't fit noun-verb pattern)
ggen help-me → help_me (no verb, just a command)
```

**Recommendation**: Group under `utils` noun for consistency.

**Migration to clap-noun-verb**:
```rust
// cli/commands/utils.rs
//! Utility commands

#[verb("help-me", "utils")]
fn utils_help_me(command: Option<String>, tips: bool) -> Result<HelpOutput> {
    // ...
}
```

---

## Summary Statistics

### Command Count by Noun

| Noun | Verbs | Status |
|------|-------|--------|
| `ai` | 10 | ✅ Perfect noun-verb match |
| `marketplace` | 14 | ⚠️ Rename `market` → `marketplace` |
| `project` | 10 | ✅ Perfect noun-verb match |
| `template` | 6 | ✅ Perfect noun-verb match |
| `hook` | 5 | ✅ Perfect noun-verb match |
| `graph` | 7 | ✅ Perfect noun-verb match |
| `audit` | 3 | ✅ Perfect noun-verb match |
| `ci` | 4 | ✅ Perfect noun-verb match |
| `lifecycle` | 9 | ✅ Perfect noun-verb match |
| `shell` | 1 | ✅ Perfect noun-verb match |
| `doctor` | 0 | ⚠️ Standalone → `utils.doctor` |
| `help-me` | 0 | ⚠️ Standalone → `utils.help_me` |

**Total**: 12 nouns, **69 verbs** (10 + 14 + 10 + 6 + 5 + 7 + 3 + 4 + 9 + 1 + 0 + 0)

### Breaking Changes Required

1. **`market` → `marketplace`** (14 commands affected)
   - v1.4.0: Add alias for backward compatibility
   - v2.0.0: Remove alias, require `marketplace`

2. **`doctor` → `utils.doctor`** (1 command affected)
   - v1.4.0: Add alias for backward compatibility
   - v2.0.0: Remove alias, require `utils.doctor`

3. **`help-me` → `utils.help-me`** (1 command affected)
   - v1.4.0: Add alias for backward compatibility
   - v2.0.0: Remove alias, require `utils.help-me`

4. **Legacy `ggen gen`** (1 command - unclear mapping)
   - Option A: Map to `template.generate`
   - Option B: Map to `project.gen` (if exists)
   - Option C: Remove in v2.0.0 (requires using full command)

**Total Breaking Changes**: 16 commands (14 marketplace + 2 utils commands)

---

## File Structure After Migration

```
cli/
├── mod.rs                 # Main entry point with clap_noun_verb::run()
├── commands/
│   ├── mod.rs             # Re-export all command modules
│   ├── ai.rs              # 10 verbs
│   ├── marketplace.rs     # 14 verbs (renamed from market)
│   ├── project.rs         # 10 verbs
│   ├── template.rs        # 6 verbs
│   ├── hook.rs            # 5 verbs
│   ├── graph.rs           # 7 verbs
│   ├── audit.rs           # 3 verbs
│   ├── ci.rs              # 4 verbs
│   ├── lifecycle.rs       # 9 verbs
│   ├── shell.rs           # 1 verb
│   └── utils.rs           # 2 verbs (doctor, help-me)
└── handlers/
    ├── mod.rs             # Re-export all handlers
    ├── ai_handlers.rs     # AI business logic
    ├── marketplace_handlers.rs
    ├── project_handlers.rs
    ├── template_handlers.rs
    └── ... (other handlers)
```

---

## Migration Priority

### Phase 1: Foundation (Week 1-2)
- ✅ `utils.doctor` (proof-of-concept)
- ✅ `utils.help-me`
- ✅ `project.new`
- ✅ `hook.create`

### Phase 2: Core Commands (Week 2-3)
- ✅ `ai.project` (most used)
- ✅ `ai.generate` (most used)
- ✅ `marketplace.search` (most used)
- ✅ `template.generate` (or `project.gen`)

### Phase 3: Remaining High-Usage (Week 3-4)
- ✅ `ai.graph`, `ai.sparql`
- ✅ `marketplace.add`, `marketplace.list`, `marketplace.publish`
- ✅ `project.gen`, `project.watch`

### Phase 4: Complete Coverage (Week 4-6)
- All remaining commands (50+ verbs)
- Backward compatibility aliases
- Comprehensive testing

---

## Benefits of Migration

### Current State Issues

1. **Large Enum-Based Structure**:
   - 12 top-level enum variants
   - 69+ nested enum variants
   - Massive match statements

2. **Manual Registration**:
   - Every command must be added to enum
   - Every command must be added to match statement
   - Harder to extend

3. **Scattered Definitions**:
   - Commands defined in 12+ separate modules
   - No centralized registration
   - Harder to discover

### After Migration Benefits

1. **Auto-Discovery**:
   - Commands automatically discovered from `#[verb]` attributes
   - No manual registration required
   - Easier to extend

2. **Type Inference**:
   - Arguments inferred from function signatures
   - Less boilerplate code
   - Compile-time type safety

3. **JSON Output**:
   - All commands return structured JSON
   - Better for scripting/automation
   - Better for agent/MCP integration

4. **Separation of Concerns**:
   - Business logic separated from CLI layer
   - Easier to test independently
   - More reusable code

---

## Next Steps

1. **Review this analysis** - Confirm all commands are correctly identified
2. **Clarify legacy commands** - Determine `ggen gen` mapping
3. **Create migration plan** - Detailed week-by-week plan
4. **Start Phase 1** - Begin with foundation commands
5. **Implement aliases** - Ensure backward compatibility

---

**Last Updated**: Complete noun-verb analysis of all 69+ ggen commands. Ready for migration planning.

