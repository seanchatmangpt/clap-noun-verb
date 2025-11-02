# ggen v2.0 Refactoring Plan: File-by-File Implementation Guide

## Overview

Literal, file-by-file refactoring plan for migrating ggen to v2.0 architecture using clap-noun-verb v3.0.0. This plan is designed for coding agents to execute.

**Target**: ggen v2.0.0  
**Based on**: clap-noun-verb v3.0.0  
**Architecture**: Pure RDF-driven with business logic separation

**Total LOC**: ~19,597 lines in CLI commands (77 files) + ~22,190 lines in ggen-core (62 files)

---

## Repository Structure

```
~/ggen/
├── Cargo.toml                          # Root workspace
├── cli/
│   ├── Cargo.toml                       # CLI crate
│   └── src/
│       ├── lib.rs                       # CLI entry point
│       └── cmds/                        # Command modules
│           ├── mod.rs                   # Commands enum (TO DELETE)
│           ├── ai/                      # AI commands
│           ├── market/                  # Marketplace commands (RENAME to marketplace)
│           ├── project/                 # Project commands
│           ├── template/                # Template commands
│           ├── hook/                    # Hook commands
│           ├── doctor.rs                # Move to utils/
│           └── help_progressive.rs      # Move to utils/
├── ggen-core/
│   ├── Cargo.toml                       # Core crate
│   └── src/
│       ├── lib.rs
│       ├── template.rs                  # Template processing
│       ├── templates/                   # Template engine
│       ├── rdf/                         # RDF processor
│       └── generator.rs                  # Code generator
└── utils/
    └── src/
        └── lib.rs                       # Utilities
```

---

## Phase 1: Dependencies & Foundation

### 1.1 Update Root Cargo.toml

**File**: `~/ggen/Cargo.toml`

**Changes**:
```toml
# Add clap-noun-verb v3.0.0
[workspace.dependencies]
clap-noun-verb = "3.0.0"
clap-noun-verb-macros = "3.0.0"

# Update version
[package]
version = "2.0.0"
```

**Actions**:
- [ ] Add `clap-noun-verb = "3.0.0"` to `[workspace.dependencies]`
- [ ] Add `clap-noun-verb-macros = "3.0.0"` to `[workspace.dependencies]`
- [ ] Update version from `1.2.0` to `2.0.0`

---

### 1.2 Update CLI Cargo.toml

**File**: `~/ggen/cli/Cargo.toml`

**Changes**:
```toml
[dependencies]
clap-noun-verb = { workspace = true }
clap-noun-verb-macros = { workspace = true }

# Remove or update clap
# clap = { version = "4.5", features = ["derive"] }  # Remove - use clap-noun-verb instead
```

**Actions**:
- [ ] Add `clap-noun-verb = { workspace = true }` to dependencies
- [ ] Add `clap-noun-verb-macros = { workspace = true }` to dependencies
- [ ] Remove `clap` dependency (replaced by clap-noun-verb)
- [ ] Update version to `2.0.0`

---

### 1.3 Create New Command Structure

**File**: `~/ggen/cli/src/commands/mod.rs` (NEW FILE)

**Create this file**:
```rust
//! Command modules for ggen v2.0
//! 
//! All commands use clap-noun-verb v3.0.0 auto-discovery

pub mod utils;
pub mod ai;
pub mod marketplace;
pub mod project;
pub mod template;
pub mod hook;
pub mod graph;
pub mod lifecycle;
pub mod shell;
pub mod ci;
pub mod audit;

// Auto-discovery happens at compile time via #[verb] attributes
// No manual registration needed
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/commands/`
- [ ] Create file: `~/ggen/cli/src/commands/mod.rs`

---

### 1.4 Create Utils Module

**File**: `~/ggen/cli/src/commands/utils/mod.rs` (NEW FILE)

**Create this file**:
```rust
//! Utility commands

// Auto-discovered via #[verb] attributes
// Business logic in src/domain/utils/
```

**File**: `~/ggen/cli/src/commands/utils/doctor.rs` (NEW FILE)

**Create this file**:
```rust
//! Run diagnostics to check system status

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Run diagnostics to check system status
///
/// This command performs various health checks and reports the status.
#[verb("doctor", "utils")]
pub fn utils_doctor() -> Result<DoctorOutput> {
    Ok(crate::domain::utils::doctor::run_diagnostics())
}

#[derive(Debug, Serialize)]
pub struct DoctorOutput {
    pub rust_ok: bool,
    pub git_ok: bool,
    pub ollama_ok: bool,
    pub docker_ok: bool,
    pub message: Option<String>,
}
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/commands/utils/`
- [ ] Create file: `~/ggen/cli/src/commands/utils/mod.rs`
- [ ] Create file: `~/ggen/cli/src/commands/utils/doctor.rs`
- [ ] Move business logic to `~/ggen/cli/src/domain/utils/doctor.rs`

---

## Phase 2: Migrate Commands to clap-noun-verb v3.0.0

### 2.1 Migrate Template Commands

**File**: `~/ggen/cli/src/commands/template/mod.rs` (NEW FILE)

**Create this file**:
```rust
//! Template management commands

pub mod generate;
pub mod list;
pub mod show;
pub mod lint;
pub mod regenerate;
pub mod generate_tree;
```

**File**: `~/ggen/cli/src/commands/template/generate.rs` (NEW FILE)

**Create this file**:
```rust
//! Generate code from template

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Generate code from a template and RDF file
///
/// # Arguments
/// * `template` - Template file path (required)
/// * `rdf` - RDF file path (required)
/// * `output` - Output directory (optional)
/// * `regenerate` - Regenerate with frozen section preservation (flag)
#[verb("generate", "template")]
pub fn template_generate(
    template: String,
    rdf: String,
    output: Option<String>,
    regenerate: bool,
) -> Result<GenerateOutput> {
    Ok(crate::domain::template::generate::generate_from_template(
        template,
        rdf,
        output,
        regenerate,
    ))
}

#[derive(Debug, Serialize)]
pub struct GenerateOutput {
    pub template: String,
    pub rdf: String,
    pub output: Option<String>,
    pub files_generated: Vec<String>,
    pub success: bool,
}
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/commands/template/`
- [ ] Create file: `~/ggen/cli/src/commands/template/mod.rs`
- [ ] Create file: `~/ggen/cli/src/commands/template/generate.rs`
- [ ] Migrate from: `~/ggen/cli/src/cmds/template/mod.rs`
- [ ] Extract business logic to: `~/ggen/cli/src/domain/template/generate.rs`

---

### 2.2 Migrate Project Commands

**File**: `~/ggen/cli/src/commands/project/mod.rs` (NEW FILE)

**Create this file**:
```rust
//! Project scaffolding and generation commands

pub mod new;
pub mod gen;
pub mod plan;
pub mod apply;
pub mod diff;
pub mod test;
pub mod freeze;
pub mod inject;
pub mod validate;
pub mod watch;
```

**File**: `~/ggen/cli/src/commands/project/gen.rs` (NEW FILE)

**Create this file**:
```rust
//! Generate artifacts from a template directly

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Generate artifacts from a template directly
///
/// # Arguments
/// * `template` - Template file path (required)
/// * `rdf` - RDF file path (required)
/// * `output` - Output directory (optional)
#[verb("gen", "project")]
pub fn project_gen(
    template: String,
    rdf: String,
    output: Option<String>,
) -> Result<GenOutput> {
    Ok(crate::domain::project::gen::generate_from_template(
        template,
        rdf,
        output,
    ))
}

#[derive(Debug, Serialize)]
pub struct GenOutput {
    pub template: String,
    pub rdf: String,
    pub output: Option<String>,
    pub files_generated: Vec<String>,
    pub success: bool,
}
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/commands/project/`
- [ ] Create file: `~/ggen/cli/src/commands/project/mod.rs`
- [ ] Create file: `~/ggen/cli/src/commands/project/gen.rs`
- [ ] Migrate from: `~/ggen/cli/src/cmds/project/gen.rs`
- [ ] Extract business logic to: `~/ggen/cli/src/domain/project/gen.rs`

---

### 2.3 Migrate AI Commands

**File**: `~/ggen/cli/src/commands/ai/mod.rs` (NEW FILE)

**Create this file**:
```rust
//! AI-powered template generation and analysis

pub mod project;
pub mod generate;
pub mod graph;
pub mod sparql;
```

**File**: `~/ggen/cli/src/commands/ai/generate.rs` (NEW FILE)

**Create this file**:
```rust
//! Generate a template from a description using AI

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Generate a template from a description using AI
///
/// # Arguments
/// * `description` - Template description (required)
/// * `output` - Output file path (optional)
#[verb("generate", "ai")]
pub fn ai_generate(
    description: String,
    output: Option<String>,
) -> Result<AiGenerateOutput> {
    Ok(crate::domain::ai::generate::generate_template(
        description,
        output,
    ))
}

#[derive(Debug, Serialize)]
pub struct AiGenerateOutput {
    pub description: String,
    pub output: String,
    pub success: bool,
    pub template: String,
}
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/commands/ai/`
- [ ] Create file: `~/ggen/cli/src/commands/ai/mod.rs`
- [ ] Create file: `~/ggen/cli/src/commands/ai/generate.rs`
- [ ] Migrate from: `~/ggen/cli/src/cmds/ai/generate.rs`
- [ ] Extract business logic to: `~/ggen/cli/src/domain/ai/generate.rs`

---

### 2.4 Rename Market to Marketplace

**File**: `~/ggen/cli/src/commands/marketplace/mod.rs` (NEW FILE)

**Create this file**:
```rust
//! Marketplace operations for gpacks

pub mod search;
pub mod add;
pub mod remove;
pub mod list;
pub mod update;
pub mod info;
pub mod recommend;
pub mod offline;
pub mod cache;
pub mod sync;
pub mod categories;
pub mod publish;
pub mod unpublish;
pub mod natural;
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/commands/marketplace/`
- [ ] Migrate all files from: `~/ggen/cli/src/cmds/market/` → `~/ggen/cli/src/commands/marketplace/`
- [ ] Update all command functions to use `#[verb]` attributes
- [ ] Extract business logic to: `~/ggen/cli/src/domain/marketplace/`

---

## Phase 3: Update Core Template Engine

### 3.1 Update Template Processing for Pure RDF

**File**: `~/ggen/ggen-core/src/template.rs`

**Changes**:
- Remove support for `rdf:` in frontmatter
- Remove support for `vars:` in frontmatter
- Require RDF to be provided via API (not frontmatter)
- Support `{% frozen %}` tag parsing

**Actions**:
- [ ] Remove `rdf:` field from frontmatter parsing
- [ ] Remove `vars:` field from frontmatter parsing
- [ ] Update `Template::load()` to not parse RDF from frontmatter
- [ ] Add `Template::render_with_rdf()` method that takes RDF as parameter
- [ ] Add `{% frozen %}` tag parsing support

---

### 3.2 Add Frozen Section Support

**File**: `~/ggen/ggen-core/src/templates/generator.rs`

**Changes**:
- Add `FrozenSection` struct
- Add `FrozenParser` component
- Add `FrozenMerger` component
- Update file writing to preserve frozen sections

**Actions**:
- [ ] Add `frozen` module to `templates/`
- [ ] Create `~/ggen/ggen-core/src/templates/frozen.rs` (NEW FILE)
- [ ] Implement `FrozenParser` to detect `{% frozen %}` tags
- [ ] Implement `FrozenMerger` to preserve frozen sections during regeneration
- [ ] Update `FileWriter` to use `FrozenMerger` when regenerating

---

### 3.3 Add Business Logic Separation

**File**: `~/ggen/ggen-core/src/templates/generator.rs`

**Changes**:
- Add `BusinessLogicSeparator` component
- Update template rendering to generate CLI layer and business logic files separately
- Add logic to never regenerate business logic files

**Actions**:
- [ ] Add `business_logic` module to `templates/`
- [ ] Create `~/ggen/ggen-core/src/templates/business_logic.rs` (NEW FILE)
- [ ] Implement `BusinessLogicSeparator` to:
  - Generate CLI layer (thin wrapper)
  - Generate business logic skeleton (only if doesn't exist)
  - Never regenerate business logic files
- [ ] Update `FileWriter` to check if business logic file exists before writing

---

## Phase 4: Update Main Entry Point

### 4.1 Update CLI Entry Point

**File**: `~/ggen/cli/src/lib.rs`

**Changes**:
- Remove enum-based command registration
- Use clap-noun-verb auto-discovery
- Remove `Commands` enum
- Use `clap_noun_verb::run()` for auto-discovery

**Actions**:
- [ ] Remove `Commands` enum usage
- [ ] Remove manual command registration
- [ ] Add `use clap_noun_verb::run;`
- [ ] Replace `Cli::parse()` with `clap_noun_verb::run()`
- [ ] Remove `cmds::mod.rs` dependency

---

### 4.2 Delete Old Command Structure

**Files to Delete**:
- `~/ggen/cli/src/cmds/mod.rs`
- `~/ggen/cli/src/cmds/doctor.rs` (moved to `commands/utils/doctor.rs`)
- `~/ggen/cli/src/cmds/help_progressive.rs` (moved to `commands/utils/help_me.rs`)

**Actions**:
- [ ] Delete `~/ggen/cli/src/cmds/mod.rs`
- [ ] Delete `~/ggen/cli/src/cmds/doctor.rs`
- [ ] Delete `~/ggen/cli/src/cmds/help_progressive.rs`
- [ ] Keep `cmds/` directory structure temporarily for migration reference

---

## Phase 5: Business Logic Separation

### 5.1 Create Domain Module Structure

**Directory**: `~/ggen/cli/src/domain/` (NEW)

**Structure**:
```
domain/
├── mod.rs
├── utils/
│   ├── mod.rs
│   └── doctor.rs          # Business logic for doctor
├── template/
│   ├── mod.rs
│   └── generate.rs        # Business logic for template generate
├── project/
│   ├── mod.rs
│   └── gen.rs             # Business logic for project gen
├── ai/
│   ├── mod.rs
│   └── generate.rs        # Business logic for ai generate
└── marketplace/
    ├── mod.rs
    └── search.rs           # Business logic for marketplace search
```

**Actions**:
- [ ] Create directory: `~/ggen/cli/src/domain/`
- [ ] Create `domain/mod.rs` with module declarations
- [ ] Create subdirectories for each noun
- [ ] Move business logic from CLI handlers to domain modules

---

### 5.2 Extract Business Logic

**Example**: `~/ggen/cli/src/cmds/utils/doctor.rs` → Extract to `~/ggen/cli/src/domain/utils/doctor.rs`

**Actions**:
- [ ] For each command, extract business logic to domain module
- [ ] CLI layer functions delegate to domain functions
- [ ] Domain functions are pure (no CLI dependencies)

---

## Phase 6: Filesystem Routing

### 6.1 Add Filesystem Router

**File**: `~/ggen/ggen-core/src/templates/filesystem_router.rs` (NEW FILE)

**Create this file**:
```rust
//! Filesystem-based routing for templates, RDF, and queries

pub struct FilesystemRouter;

impl FilesystemRouter {
    /// Discover templates from filesystem
    pub fn discover_templates(project_root: &Path) -> Vec<PathBuf> {
        // Discover from templates/ directory
    }
    
    /// Discover RDF files from filesystem
    pub fn discover_rdf(project_root: &Path) -> Vec<PathBuf> {
        // Discover from domain/ directory
    }
    
    /// Discover SPARQL queries from filesystem
    pub fn discover_queries(project_root: &Path) -> Vec<PathBuf> {
        // Discover from queries/ directory (optional)
    }
    
    /// Infer output path from template and RDF
    pub fn infer_output_path(template: &Path, rdf: &Path) -> PathBuf {
        // Convention-based path inference
    }
}
```

**Actions**:
- [ ] Create file: `~/ggen/ggen-core/src/templates/filesystem_router.rs`
- [ ] Implement template discovery
- [ ] Implement RDF discovery
- [ ] Implement query discovery
- [ ] Implement output path inference

---

## Phase 7: Remove v1.x Patterns

### 7.1 Remove Frontmatter Support

**Files to Update**:
- `~/ggen/ggen-core/src/template.rs`
- `~/ggen/ggen-core/src/rdf/template_metadata.rs`

**Actions**:
- [ ] Remove `rdf:` field parsing from frontmatter
- [ ] Remove `vars:` field parsing from frontmatter
- [ ] Update template loading to not parse these fields
- [ ] Update documentation/comments to reflect removal

---

### 7.2 Remove --var CLI Flags

**Files to Update**:
- All command files in `~/ggen/cli/src/commands/`

**Actions**:
- [ ] Remove `--var` argument definitions
- [ ] Update command functions to not accept `vars` parameter
- [ ] Update help text to reflect removal

---

## Phase 8: Update Tests

### 8.1 Update Integration Tests

**File**: `~/ggen/cli/tests/integration.rs` (or similar)

**Changes**:
- Update tests to use `ggen template generate` syntax
- Remove tests for `--var` flags
- Add tests for RDF-driven generation
- Add tests for frozen sections
- Add tests for business logic separation

**Actions**:
- [ ] Update test command syntax
- [ ] Remove tests for v1.x patterns
- [ ] Add tests for v2.0 patterns
- [ ] Add tests for frozen section preservation
- [ ] Add tests for business logic separation

---

### 8.2 Update Unit Tests

**Files**: All test files in `~/ggen/ggen-core/tests/`

**Changes**:
- Update tests to not use frontmatter `rdf:` or `vars:`
- Add tests for frozen section parsing
- Add tests for business logic separation
- Add tests for filesystem routing

**Actions**:
- [ ] Update all tests to use RDF via API (not frontmatter)
- [ ] Add unit tests for frozen sections
- [ ] Add unit tests for business logic separation
- [ ] Add unit tests for filesystem routing

---

## Phase 9: Update Documentation

### 9.1 Update README

**File**: `~/ggen/README.md`

**Actions**:
- [ ] Update command examples to v2.0 syntax
- [ ] Remove references to v1.x patterns
- [ ] Add v2.0 features section

---

### 9.2 Update CHANGELOG

**File**: `~/ggen/CHANGELOG.md`

**Actions**:
- [ ] Add v2.0.0 section
- [ ] Document breaking changes
- [ ] Document new features
- [ ] Document migration path

---

## File-by-File Refactoring Details

### Command Files Analysis

#### 1. Template Commands (7 files, ~2,200 LOC)

**Source**: `~/ggen/cli/src/cmds/template/`  
**Target**: `~/ggen/cli/src/commands/template/`

##### 1.1 `template/generate_tree.rs` (254 LOC)
- **Keep**: Lines 40-222 (business logic: file tree generation, validation, preview)
- **Extract to**: `~/ggen/cli/src/domain/template/generate_tree.rs` (lines 40-222)
- **Replace**: Lines 1-39 (Args struct, CLI parsing) → `#[verb]` attribute with function parameters
- **Delete**: `parse_key_val` function (lines 33-38) - not needed with `#[verb]` auto-inference
- **Action**: Create `commands/template/generate_tree.rs` (thin wrapper, ~30 LOC)

##### 1.2 `template/list.rs` (311 LOC)
- **Keep**: Lines 48-196 (business logic: template listing, filtering, description extraction)
- **Extract to**: `~/ggen/cli/src/domain/template/list.rs` (lines 48-196)
- **Replace**: Lines 1-47 (Args struct, CLI parsing) → `#[verb]` attribute
- **Keep**: Lines 198-310 (tests) → move to domain module tests
- **Action**: Create `commands/template/list.rs` (thin wrapper, ~30 LOC)

##### 1.3 `template/show.rs` (355 LOC)
- **Keep**: Lines 64-287 (business logic: metadata parsing, template inspection)
- **Extract to**: `~/ggen/cli/src/domain/template/show.rs` (lines 64-287)
- **Replace**: Lines 1-63 (Args struct, CLI parsing) → `#[verb]` attribute
- **Keep**: Lines 289-354 (tests) → move to domain module tests
- **Action**: Create `commands/template/show.rs` (thin wrapper, ~30 LOC)

##### 1.4 `template/lint.rs` (433 LOC)
- **Keep**: Lines 59-284 (business logic: linting logic, validation, SPARQL checks)
- **Extract to**: `~/ggen/cli/src/domain/template/lint.rs` (lines 59-284)
- **Replace**: Lines 1-58 (Args struct, CLI parsing) → `#[verb]` attribute
- **Keep**: Lines 142-284 (lint implementation) - this is core business logic
- **Action**: Create `commands/template/lint.rs` (thin wrapper, ~30 LOC)

##### 1.5 `template/new.rs` (439 LOC)
- **Keep**: Lines 49-439 (business logic: template creation, validation, file writing)
- **Extract to**: `~/ggen/cli/src/domain/template/new.rs` (lines 49-439)
- **Replace**: Lines 1-48 (Args struct, CLI parsing) → `#[verb]` attribute
- **Action**: Create `commands/template/new.rs` (thin wrapper, ~30 LOC)

##### 1.6 `template/regenerate.rs` (~400 LOC estimated)
- **Keep**: All business logic (regeneration with delta tracking)
- **Extract to**: `~/ggen/cli/src/domain/template/regenerate.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/template/regenerate.rs` (thin wrapper, ~30 LOC)

##### 1.7 `template/mod.rs` (47 LOC)
- **DELETE**: Entire file - replaced by auto-discovery
- **Action**: No migration needed, use `#[verb]` attributes directly

---

#### 2. Project Commands (11 files, ~3,500 LOC)

**Source**: `~/ggen/cli/src/cmds/project/`  
**Target**: `~/ggen/cli/src/commands/project/`

##### 2.1 `project/gen.rs` (376 LOC)
- **Keep**: Lines 99-219 (business logic: template resolution, plan generation, application)
- **Keep**: Lines 64-98 (validation functions) - move to domain
- **Extract to**: `~/ggen/cli/src/domain/project/gen.rs` (lines 64-219)
- **Replace**: Lines 1-63 (Args struct, CLI parsing) → `#[verb]` attribute
- **Keep**: Lines 221-375 (tests) → move to domain module tests
- **Delete**: Lines 159-190 (cargo make invocation) - replace with direct API calls
- **Action**: Create `commands/project/gen.rs` (thin wrapper, ~30 LOC)

##### 2.2 `project/new.rs` (148 LOC)
- **Keep**: Lines 49-97 (business logic: project creation, validation)
- **Extract to**: `~/ggen/cli/src/domain/project/new.rs` (lines 49-97)
- **Replace**: Lines 1-48 (Args struct, CLI parsing) → `#[verb]` attribute
- **Keep**: Lines 99-147 (tests) → move to domain module tests
- **Action**: Create `commands/project/new.rs` (thin wrapper, ~30 LOC)

##### 2.3 `project/plan.rs` (~250 LOC estimated)
- **Keep**: All business logic (plan generation, dry-run logic)
- **Extract to**: `~/ggen/cli/src/domain/project/plan.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/plan.rs` (thin wrapper, ~30 LOC)

##### 2.4 `project/apply.rs` (~250 LOC estimated)
- **Keep**: All business logic (plan application, file operations)
- **Extract to**: `~/ggen/cli/src/domain/project/apply.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/apply.rs` (thin wrapper, ~30 LOC)

##### 2.5 `project/diff.rs` (~250 LOC estimated)
- **Keep**: All business logic (diff generation, file comparison)
- **Extract to**: `~/ggen/cli/src/domain/project/diff.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/diff.rs` (thin wrapper, ~30 LOC)

##### 2.6 `project/test.rs` (~250 LOC estimated)
- **Keep**: All business logic (golden file testing, snapshot management)
- **Extract to**: `~/ggen/cli/src/domain/project/test.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/test.rs` (thin wrapper, ~30 LOC)

##### 2.7 `project/freeze.rs` (~250 LOC estimated)
- **Keep**: All business logic (freeze block detection, preservation logic)
- **Extract to**: `~/ggen/cli/src/domain/project/freeze.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Note**: Freeze command removed per v2.0 design - freezing is template-driven
- **Action**: DELETE this file - functionality moved to template engine

##### 2.8 `project/inject.rs` (271 LOC)
- **Keep**: All business logic (code injection, anchor detection, file modification)
- **Extract to**: `~/ggen/cli/src/domain/project/inject.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/inject.rs` (thin wrapper, ~30 LOC)

##### 2.9 `project/validate.rs` (~250 LOC estimated)
- **Keep**: All business logic (plan validation, schema checking)
- **Extract to**: `~/ggen/cli/src/domain/project/validate.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/validate.rs` (thin wrapper, ~30 LOC)

##### 2.10 `project/watch.rs` (277 LOC)
- **Keep**: All business logic (file watching, change detection, regeneration)
- **Extract to**: `~/ggen/cli/src/domain/project/watch.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Action**: Create `commands/project/watch.rs` (thin wrapper, ~30 LOC)

##### 2.11 `project/mod.rs` (~100 LOC estimated)
- **DELETE**: Entire file - replaced by auto-discovery
- **Action**: No migration needed, use `#[verb]` attributes directly

---

#### 3. AI Commands (11 files, ~2,500 LOC)

**Source**: `~/ggen/cli/src/cmds/ai/`  
**Target**: `~/ggen/cli/src/commands/ai/`

##### 3.1 `ai/generate.rs` (198 LOC)
- **Keep**: Lines 50-197 (business logic: AI generation, template creation, validation)
- **Extract to**: `~/ggen/cli/src/domain/ai/generate.rs` (lines 50-197)
- **Replace**: Lines 1-49 (Args struct, CLI parsing) → `#[verb]` attribute
- **Action**: Create `commands/ai/generate.rs` (thin wrapper, ~30 LOC)

##### 3.2-3.11 Other AI commands (similar pattern)
- **Keep**: All business logic
- **Extract to**: `~/ggen/cli/src/domain/ai/`
- **Replace**: Args structs → `#[verb]` attributes
- **Action**: Create thin wrapper files (~30 LOC each)

---

#### 4. Marketplace Commands (14 files, ~4,500 LOC) → Rename to `marketplace/`

**Source**: `~/ggen/cli/src/cmds/market/`  
**Target**: `~/ggen/cli/src/commands/marketplace/`

##### 4.1 `marketplace/search.rs` (635 LOC - largest command file)
- **Keep**: Lines 102-636 (business logic: search logic, filtering, ranking)
- **Extract to**: `~/ggen/cli/src/domain/marketplace/search.rs` (lines 102-636)
- **Replace**: Lines 1-101 (Args struct, CLI parsing) → `#[verb]` attribute
- **Keep**: All test code (lines 400-636) → move to domain module tests
- **Action**: Create `commands/marketplace/search.rs` (thin wrapper, ~30 LOC)

##### 4.2-4.14 Other marketplace commands (similar pattern)
- **Keep**: All business logic
- **Extract to**: `~/ggen/cli/src/domain/marketplace/`
- **Replace**: Args structs → `#[verb]` attributes
- **Action**: Create thin wrapper files (~30 LOC each)
- **Rename**: All `market/` → `marketplace/`

---

#### 5. Utils Commands (2 files, ~300 LOC)

**Source**: `~/ggen/cli/src/cmds/` (root level)  
**Target**: `~/ggen/cli/src/commands/utils/`

##### 5.1 `utils/doctor.rs` (156 LOC)
- **Keep**: Lines 13-155 (business logic: environment checks, validation)
- **Extract to**: `~/ggen/cli/src/domain/utils/doctor.rs` (lines 13-155)
- **Replace**: Lines 1-12 (Args struct, CLI parsing) → `#[verb]` attribute
- **Action**: Create `commands/utils/doctor.rs` (thin wrapper, ~30 LOC)

##### 5.2 `utils/help_me.rs` (160 LOC estimated from help_progressive.rs)
- **Keep**: All business logic (progressive help, user level tracking)
- **Extract to**: `~/ggen/cli/src/domain/utils/help_me.rs`
- **Replace**: Args struct → `#[verb]` attribute
- **Rename**: `help_progressive.rs` → `help_me.rs`
- **Action**: Create `commands/utils/help_me.rs` (thin wrapper, ~30 LOC)

---

#### 6. Other Command Modules

Similar pattern for:
- `graph/` (8 files, ~2,500 LOC)
- `hook/` (6 files, ~1,500 LOC)
- `ci/` (5 files, ~2,500 LOC)
- `audit/` (4 files, ~1,500 LOC)
- `shell/` (3 files, ~900 LOC)
- `lifecycle/` (1 file, ~656 LOC)

**Pattern for all**:
- **Keep**: All business logic (extract to `domain/`)
- **Replace**: Args structs → `#[verb]` attributes
- **Action**: Create thin wrapper files (~30 LOC each)

---

### Core Template Engine Files

#### `ggen-core/src/template.rs` (882 LOC - largest core file)

**Current**: Lines 38-100 (Frontmatter struct with `rdf:`, `vars:`, etc.)
- **Delete**: Lines 73-75 (`rdf: Vec<String>`) - RDF comes from CLI, not frontmatter
- **Delete**: Lines 79-82 (`vars: BTreeMap<String, serde_yaml::Value>`) - vars come from RDF
- **Keep**: Lines 38-72 (Hygen-compatible frontmatter: `to`, `from`, `inject`, etc.)
- **Keep**: Lines 83-100 (freeze_policy, freeze_slots_dir) - needed for frozen sections

**Modifications**:
- **Add**: `render_with_rdf(rdf_files: Vec<PathBuf>) -> Result<String>` method
- **Remove**: RDF loading from frontmatter (lines ~200-300 estimated)
- **Remove**: Variable extraction from frontmatter (lines ~300-400 estimated)
- **Keep**: Template rendering logic (Tera integration, SPARQL query execution)

**Action**: Update to accept RDF via API parameter, not frontmatter

---

#### `ggen-core/src/templates/generator.rs` (150+ LOC)

**Modifications**:
- **Add**: `FrozenParser` to detect `{% frozen %}` tags in templates
- **Add**: `FrozenMerger` to preserve frozen sections during regeneration
- **Add**: `BusinessLogicSeparator` to generate CLI layer + business logic files separately
- **Keep**: All existing file generation logic

**Action**: Add frozen section and business logic separation support

---

### Entry Point Files

#### `cli/src/lib.rs` (275 LOC)

**Current**: Lines 1-275 (enum-based command routing)
- **Delete**: Lines 11-50 (`Cli` struct with `Commands` enum)
- **Delete**: Lines 56-99 (`cli_match` function with enum matching)
- **Delete**: Lines 111-134 (`run_with_config` enum matching)
- **Keep**: Lines 101-110 (`RunResult` struct) - still needed for Node.js integration
- **Keep**: Lines 111-274 (`run_for_node` function) - keep for programmatic API

**Modifications**:
- **Replace**: Lines 1-100 → Use `clap_noun_verb::run()` for auto-discovery
- **Keep**: Node.js integration (lines 101-274)

**Action**: Replace enum-based routing with clap-noun-verb auto-discovery

---

#### `cli/src/cmds/mod.rs` (136 LOC)

**DELETE**: Entire file - replaced by auto-discovery via `#[verb]` attributes

---

### Summary Statistics

**Total LOC to Keep (Business Logic)**: ~15,000 LOC (extracted to domain modules)  
**Total LOC to Delete (CLI Layer)**: ~4,500 LOC (Args structs, enum matching)  
**Total LOC to Create (Thin Wrappers)**: ~2,300 LOC (77 files × ~30 LOC each)  
**Net Change**: -2,200 LOC (simpler, cleaner codebase)

**Core Engine Changes**:
- **Remove**: ~500 LOC (frontmatter RDF/vars parsing)
- **Add**: ~300 LOC (frozen sections, business logic separation)
- **Net Change**: -200 LOC

---

### Files to Delete

1. **Old Command Structure**:
   - `~/ggen/cli/src/cmds/mod.rs` (136 LOC) - Replaced by auto-discovery
   - `~/ggen/cli/src/cmds/doctor.rs` (156 LOC) - Moved to `commands/utils/doctor.rs`
   - `~/ggen/cli/src/cmds/help_progressive.rs` (~160 LOC) - Moved to `commands/utils/help_me.rs`

2. **Project Freeze Command**:
   - `~/ggen/cli/src/cmds/project/freeze.rs` (~250 LOC) - Functionality moved to template engine

---

## Migration Order

### Step 1: Foundation
1. Update dependencies (Cargo.toml files)
2. Create new command structure directories
3. Create domain structure directories

### Step 2: Proof of Concept
4. Migrate one command (utils/doctor) fully
5. Test auto-discovery works
6. Verify business logic separation works

### Step 3: Core Migration
7. Migrate template commands
8. Migrate project commands
9. Migrate ai commands
10. Rename and migrate marketplace commands

### Step 4: Core Features
11. Add frozen section support
12. Add business logic separation
13. Add filesystem routing

### Step 5: Cleanup
14. Remove v1.x patterns
15. Delete old command structure
16. Update tests
17. Update documentation

---

## Testing Checklist

### After Each Phase

- [ ] Code compiles without errors
- [ ] All tests pass (where applicable)
- [ ] No `unwrap()` or `expect()` in production code
- [ ] Proper error handling with `Result<T>` types
- [ ] Auto-discovery works correctly
- [ ] JSON output works correctly
- [ ] Type inference works correctly

---

## Success Criteria

### Definition of Done

- [ ] All commands migrated to `#[verb]` attributes
- [ ] All business logic separated to domain modules
- [ ] Frozen sections implemented and tested
- [ ] Filesystem routing implemented
- [ ] All v1.x patterns removed
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] No `unwrap()` or `expect()` in production code
- [ ] Documentation updated

---

**Last Updated**: Literal refactoring plan for coding agents

