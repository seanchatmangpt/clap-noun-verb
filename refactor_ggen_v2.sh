#!/bin/bash

# ggen v2.0 Refactoring Script using claude-flow
# 
# This script uses claude-flow to automate the migration of ggen from v1.x to v2.0
# using clap-noun-verb v3.0.0. The script follows the comprehensive refactoring
# plan documented in docs/book/GGEN_V2_REFACTORING_PLAN.md
#
# Prerequisites:
# - Node.js 18+
# - npm 9+
# - claude-flow installed globally: npm install -g claude-flow@alpha
# - Claude Code activated: claude --dangerously-skip-permissions
#
# Usage:
#   ./refactor_ggen_v2.sh [ggen-project-path]
#
# Example:
#   ./refactor_ggen_v2.sh ~/ggen

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if ggen project path is provided
GGEN_PROJECT_PATH="${1:-$HOME/ggen}"

if [ ! -d "$GGEN_PROJECT_PATH" ]; then
    print_error "ggen project directory not found: $GGEN_PROJECT_PATH"
    print_info "Please provide the path to your ggen project:"
    print_info "  ./refactor_ggen_v2.sh /path/to/ggen"
    exit 1
fi

print_success "Using ggen project: $GGEN_PROJECT_PATH"

# Check for claude-flow installation
if ! command -v claude-flow &> /dev/null; then
    print_error "claude-flow is not installed"
    print_info "Install it with: npm install -g claude-flow@alpha"
    exit 1
fi

print_success "claude-flow found: $(claude-flow --version)"

# Navigate to project directory
cd "$GGEN_PROJECT_PATH" || {
    print_error "Failed to navigate to project directory"
    exit 1
}

print_info "Working directory: $(pwd)"

# Initialize claude-flow if not already initialized
if [ ! -f ".claude-flow/config.json" ]; then
    print_info "Initializing claude-flow..."
    claude-flow init --force || {
        print_error "Failed to initialize claude-flow"
        exit 1
    }
    print_success "claude-flow initialized"
else
    print_info "claude-flow already initialized"
fi

# Create refactoring context file with comprehensive plan
print_info "Creating refactoring context..."

cat > .claude-flow/refactor-v2-context.md << 'EOF'
# ggen v2.0 Refactoring Context

## Goal
Refactor ggen from v1.x to v2.0.0 using clap-noun-verb v3.0.0, with complete architectural changes including:
- CLI migration from clap enums to clap-noun-verb attribute macros
- Business logic separation (CLI layer vs domain layer)
- Async/sync compatibility patterns
- Pure RDF-driven template generation
- Command renames (market → marketplace, doctor → utils doctor, etc.)
- Removal of --vars flags, addition of --rdf flags

## Critical Constraints

### 1. Async/Sync Compatibility ⚠️ CRITICAL
- clap-noun-verb v3.0.0 uses sync-only functions (dyn compatibility requirement)
- ggen has 94 async functions in CLI commands
- Solution: Use sync CLI wrappers that spawn async runtimes:
  ```rust
  #[verb("command", "noun")]
  fn noun_command(...) -> Result<Output> {
      let rt = tokio::runtime::Runtime::new()
          .map_err(|e| clap_noun_verb::NounVerbError::execution_error(...))?;
      rt.block_on(async {
          crate::domain::noun::command(...).await
              .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))
      })
  }
  ```

### 2. Error Handling
- NEVER use unwrap() or expect() in production code
- Always use Result<T> types with proper error propagation
- Use ? operator for error propagation

### 3. File Structure
- CLI layer: `cli/src/commands/` (sync wrappers)
- Business logic: `cli/src/domain/` (async functions)
- Old structure: `cli/src/cmds/` (to be deleted after migration)

## Phases

### Phase 1: Dependencies & Foundation
1. Update Cargo.toml files:
   - Add clap-noun-verb = "3.0.0" to workspace dependencies
   - Add clap-noun-verb-macros = "3.0.0" to workspace dependencies
   - Update version from 1.2.0 to 2.0.0
   - Remove clap dependency from CLI crate (replaced by clap-noun-verb)

2. Create new directory structure:
   - `cli/src/commands/` (new CLI layer)
   - `cli/src/domain/` (new business logic layer)

### Phase 2: Proof of Concept - utils/doctor
Migrate one command fully to validate the pattern:
1. Extract business logic from `cli/src/cmds/doctor.rs` to `cli/src/domain/utils.rs`
2. Create sync wrapper in `cli/src/commands/utils.rs`
3. Test auto-discovery works
4. Verify business logic separation works

### Phase 3: Core Migration
Migrate commands in this order:
1. Template commands (gen → template generate, --vars → --rdf)
2. Project commands
3. AI commands
4. Marketplace commands (market → marketplace rename)
5. Hook commands
6. Utils commands (move doctor and help-me)

### Phase 4: Core Engine Updates
1. Remove frontmatter RDF/vars parsing from template.rs
2. Add RDF loading via CLI parameter
3. Add frozen section support
4. Add filesystem routing support

### Phase 5: Cleanup
1. Delete old command structure (`cli/src/cmds/`)
2. Remove v1.x patterns
3. Update tests
4. Update documentation

## Command Renames (v2.0 Breaking Changes)

- `market` → `marketplace` (all 14 commands)
- `doctor` → `utils doctor`
- `help-me` → `utils help-me`
- `ggen gen` → `ggen template generate`
- `--vars` flags → `--rdf` flag

## Key Files Reference

See docs/book/GGEN_V2_REFACTORING_PLAN.md for detailed file-by-file instructions.

EOF

print_success "Refactoring context created"

# Create phase-by-phase task file
print_info "Creating phase-by-phase tasks..."

cat > .claude-flow/refactor-v2-tasks.md << 'EOF'
# ggen v2.0 Refactoring Tasks

## Phase 1: Dependencies & Foundation

### Task 1.1: Update Root Cargo.toml
- [ ] Add `clap-noun-verb = "3.0.0"` to `[workspace.dependencies]`
- [ ] Add `clap-noun-verb-macros = "3.0.0"` to `[workspace.dependencies]`
- [ ] Update `version = "2.0.0"` in `[package]`

### Task 1.2: Update CLI Cargo.toml
- [ ] Add `clap-noun-verb = { workspace = true }` to dependencies
- [ ] Add `clap-noun-verb-macros = { workspace = true }` to dependencies
- [ ] Remove `clap` dependency (replaced by clap-noun-verb)
- [ ] Update version to `2.0.0`

### Task 1.3: Create New Directory Structure
- [ ] Create `cli/src/commands/` directory
- [ ] Create `cli/src/domain/` directory
- [ ] Create `cli/src/commands/mod.rs` (empty for now)
- [ ] Create `cli/src/domain/mod.rs` (empty for now)

## Phase 2: Proof of Concept - utils/doctor

### Task 2.1: Extract Business Logic
- [ ] Read `cli/src/cmds/doctor.rs` (156 LOC)
- [ ] Extract business logic to `cli/src/domain/utils.rs`
- [ ] Keep business logic as async functions
- [ ] Ensure proper error handling (no unwrap/expect)

### Task 2.2: Create Sync CLI Wrapper
- [ ] Create `cli/src/commands/utils.rs`
- [ ] Implement `#[verb("doctor", "utils")]` sync wrapper
- [ ] Use runtime spawning pattern for async business logic
- [ ] Ensure proper error handling

### Task 2.3: Update CLI Entry Point
- [ ] Update `cli/src/lib.rs` to use `clap_noun_verb::run()`
- [ ] Remove old command enum matching
- [ ] Test auto-discovery works

### Task 2.4: Test
- [ ] Verify `ggen utils doctor` works
- [ ] Verify JSON output is correct
- [ ] Verify error handling works

## Phase 3: Core Migration

### Task 3.1: Template Commands
Migrate these files from `cli/src/cmds/template/`:
- [ ] `generate_tree.rs` (254 LOC) → `commands/template/generate.rs` + `domain/template/generate.rs`
- [ ] `list.rs` (311 LOC) → `commands/template/list.rs` + `domain/template/list.rs`
- [ ] `show.rs` (355 LOC) → `commands/template/show.rs` + `domain/template/show.rs`
- [ ] `lint.rs` (433 LOC) → `commands/template/lint.rs` + `domain/template/lint.rs`
- [ ] `new.rs` (439 LOC) → `commands/template/new.rs` + `domain/template/new.rs`

**Key Changes**:
- Change `gen` → `template generate`
- Remove `--vars` flags
- Add `--rdf` flag
- Separate CLI layer from business logic

### Task 3.2: Project Commands
- [ ] `gen.rs` (376 LOC) → `commands/project/gen.rs` + `domain/project/gen.rs`
- [ ] `new.rs` (148 LOC) → `commands/project/new.rs` + `domain/project/new.rs`
- [ ] Remove `freeze.rs` (functionality moved to template engine)

### Task 3.3: AI Commands
- [ ] `generate.rs` (198 LOC) → `commands/ai/generate.rs` + `domain/ai/generate.rs`
- [ ] `graph.rs` → `commands/ai/graph.rs` + `domain/ai/graph.rs`
- [ ] `project.rs` → `commands/ai/project.rs` + `domain/ai/project.rs`
- [ ] `sparql.rs` → `commands/ai/sparql.rs` + `domain/ai/sparql.rs`

### Task 3.4: Marketplace Commands (Rename market → marketplace)
- [ ] Rename `cli/src/cmds/market/` → `cli/src/cmds/marketplace/` (temporary)
- [ ] Migrate all 14 marketplace commands
- [ ] Update all command references from `market` to `marketplace`

### Task 3.5: Hook Commands
- [ ] `create.rs` → `commands/hook/create.rs` + `domain/hook/create.rs`
- [ ] Other hook commands as needed

### Task 3.6: Utils Commands
- [ ] `help_progressive.rs` (~160 LOC) → `commands/utils/help_me.rs` + `domain/utils/help_me.rs`
- [ ] Already done: `doctor.rs` → `commands/utils/doctor.rs` + `domain/utils/doctor.rs`

## Phase 4: Core Engine Updates

### Task 4.1: Template Engine
- [ ] Remove `rdf: Vec<String>` from Frontmatter struct in `ggen-core/src/template.rs`
- [ ] Remove RDF loading from frontmatter
- [ ] Add `render_with_rdf(rdf_files: Vec<PathBuf>) -> Result<String>` method
- [ ] Update all template rendering to use CLI-provided RDF

### Task 4.2: Frozen Sections
- [ ] Add `{% frozen %}` / `{% endfrozen %}` tag support
- [ ] Add merge algorithm for preserving frozen content
- [ ] Integrate with template engine

### Task 4.3: Filesystem Routing
- [ ] Add discovery for RDF files in `domain/` directory
- [ ] Add discovery for templates in `templates/` directory
- [ ] Add discovery for SPARQL queries in `queries/` directory
- [ ] Support path inference from filesystem structure

## Phase 5: Cleanup

### Task 5.1: Remove Old Structure
- [ ] Delete `cli/src/cmds/mod.rs` (Commands enum)
- [ ] Delete `cli/src/cmds/` directory (after migration complete)
- [ ] Remove all v1.x patterns

### Task 5.2: Update Tests
- [ ] Update all tests for new command structure
- [ ] Add tests for async/sync compatibility
- [ ] Add tests for business logic separation

### Task 5.3: Update Documentation
- [ ] Update README.md
- [ ] Update command documentation
- [ ] Update examples

## Success Criteria

- [ ] All commands migrated to `#[verb]` attributes
- [ ] All commands use sync wrappers with async business logic
- [ ] No `unwrap()` or `expect()` in production code
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] All breaking changes documented
EOF

print_success "Phase-by-phase tasks created"

# Start refactoring process
print_info "Starting ggen v2.0 refactoring process..."
print_warning "This will make significant changes to the codebase"
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Refactoring cancelled"
    exit 0
fi

# Initialize claude-flow swarm with comprehensive context
print_info "Initializing claude-flow swarm..."

cat > .claude-flow/swarm-instructions.txt << 'EOF'
Refactor ggen from v1.x to v2.0.0 following the comprehensive plan in:
- .claude-flow/refactor-v2-context.md (overall strategy)
- .claude-flow/refactor-v2-tasks.md (phase-by-phase tasks)
- docs/book/GGEN_V2_REFACTORING_PLAN.md (detailed file-by-file instructions)

CRITICAL CONSTRAINTS:
1. Async/Sync: All CLI functions must be sync wrappers that spawn async runtimes for business logic
2. Error Handling: NEVER use unwrap() or expect() - always use Result<T> with proper error propagation
3. Business Logic Separation: CLI layer in commands/, business logic in domain/
4. Command Renames: market → marketplace, doctor → utils doctor, help-me → utils help-me, gen → template generate
5. RDF-Driven: Remove --vars flags, add --rdf flags, make templates pure RDF-driven

Start with Phase 1 (Dependencies & Foundation), then Phase 2 (Proof of Concept with utils/doctor),
then proceed through remaining phases systematically.

After each phase:
- Verify code compiles
- Run tests if available
- Check for unwrap/expect usage
- Verify auto-discovery works
EOF

# Start the swarm
print_info "Starting claude-flow swarm..."
claude-flow swarm "Refactor ggen to v2.0.0 following the plan in .claude-flow/refactor-v2-context.md and .claude-flow/refactor-v2-tasks.md" \
    --context ".claude-flow/refactor-v2-context.md" \
    --context ".claude-flow/refactor-v2-tasks.md" \
    --context "docs/book/GGEN_V2_REFACTORING_PLAN.md" \
    --context "docs/book/ASYNC_SYNC_COMPATIBILITY.md" \
    --instructions ".claude-flow/swarm-instructions.txt" || {
    print_error "Failed to start claude-flow swarm"
    exit 1
}

print_success "claude-flow swarm started"
print_info "Monitor progress with: claude-flow status"
print_info "View logs with: claude-flow logs"

# Wait for completion and check status
print_info "Waiting for refactoring to complete..."
sleep 5

# Check status
print_info "Current status:"
claude-flow status || print_warning "Status check failed - swarm may still be running"

print_info ""
print_success "Refactoring process initiated!"
print_info ""
print_info "Next steps:"
print_info "1. Monitor progress: claude-flow status"
print_info "2. View detailed logs: claude-flow logs"
print_info "3. After completion, verify changes:"
print_info "   - cargo check"
print_info "   - cargo test"
print_info "   - cargo clippy"
print_info "4. Review changes and commit when ready"
print_info ""
print_warning "Remember to review all changes before committing!"
print_info ""
print_info "For detailed refactoring plan, see: docs/book/GGEN_V2_REFACTORING_PLAN.md"

