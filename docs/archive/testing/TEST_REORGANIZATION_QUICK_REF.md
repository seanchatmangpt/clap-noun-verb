# Test Reorganization Quick Reference

**One-page guide for implementing Diataxis-aligned test reorganization**

## The Plan (7.5 hours, 85% alignment)

```
Current (Flat)          →    Target (Diataxis)
-------------------          ---------------------
tests/                       tests/
├── 41 .rs files             ├── tutorials/      📚
└── common/                  ├── howto/          🎯
                            ├── reference/      📖
                            ├── explanations/   🧠
                            └── common/
```

## Effort Breakdown

| Priority | Files | Hours | Alignment |
|----------|-------|-------|-----------|
| Setup | 0 | 0.5 | 15% |
| P1: Must Move | 8 | 2.0 | 60% |
| P2: Should Move | 12 | 3.5 | 75% |
| P3: Merge | 6→4 | 2.0 | 80% |
| P4: Create | 5 | 3.25 | 85% |
| Documentation | - | 0.75 | 85% |
| **Total** | **29** | **7.5** | **85%** |

## Critical Moves (Priority 1: 2 hours)

**Move these 8 files first (60% alignment)**:

```bash
# How-to quadrant
git mv tests/async_io_tests.rs tests/howto/async_operations.rs
git mv tests/env_vars.rs tests/howto/environment_vars.rs
git mv tests/concurrency_tests.rs tests/howto/concurrency.rs

# Explanations quadrant
git mv tests/edge_cases.rs tests/explanations/edge_cases.rs
git mv tests/hotpath_tests.rs tests/explanations/hotpath_optimization.rs

# Reference/advanced quadrant
git mv tests/autonomic_tests.rs tests/reference/advanced/autonomic.rs
git mv tests/contracts_tests.rs tests/reference/advanced/contracts.rs
git mv tests/governance_tests.rs tests/reference/advanced/governance.rs
```

**Verify**: `cargo test --all-features`

## Directory Structure

```
tests/
├── tutorials/                      # 📚 Learning-oriented
│   ├── mod.rs
│   ├── hello_world.rs             # CREATE (50 lines)
│   ├── basic_noun_verb.rs         # CREATE (80 lines)
│   └── adding_arguments.rs        # CREATE (100 lines)
│
├── howto/                          # 🎯 Problem-oriented
│   ├── mod.rs
│   ├── async_operations.rs        # MOVE from async_io_tests.rs
│   ├── environment_vars.rs        # MOVE from env_vars.rs
│   ├── concurrency.rs             # MOVE from concurrency_tests.rs
│   ├── io_integration.rs          # KEEP in place
│   ├── dx_improvements.rs         # KEEP in place
│   ├── positional_args.rs         # KEEP in place
│   └── arg_actions.rs             # KEEP in place
│
├── reference/                      # 📖 Information-oriented
│   ├── mod.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── unit.rs                # MOVE from unit.rs
│   │   ├── integration.rs         # MOVE from integration.rs
│   │   └── kernel.rs              # MOVE from kernel_tests.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── builder.rs             # MERGE cli_builder + cli_builder_new
│   │   ├── validator.rs           # MERGE cli_validator + cli_validator_new
│   │   └── router.rs              # MOVE from cli_router.rs
│   ├── runtime/
│   │   ├── mod.rs
│   │   ├── executor.rs            # MERGE runtime_executor + runtime_executor_new
│   │   └── interceptor.rs         # MOVE from runtime_interceptor.rs
│   ├── logic/
│   │   ├── mod.rs
│   │   ├── handler.rs             # MERGE logic_handler + logic_handler_new
│   │   └── core.rs                # MOVE from logic_core.rs
│   ├── macros/
│   │   ├── mod.rs
│   │   └── exact_output.rs        # MOVE from exact_macro_output.rs
│   ├── validation/
│   │   ├── mod.rs
│   │   └── acceptance.rs          # MOVE from validation_acceptance.rs
│   └── advanced/
│       ├── mod.rs
│       ├── autonomic.rs           # MOVE from autonomic_tests.rs
│       ├── contracts.rs           # MOVE from contracts_tests.rs
│       ├── governance.rs          # MOVE from governance_tests.rs
│       ├── delegation.rs          # MOVE from delegation_tests.rs
│       ├── certificates.rs        # MOVE from certificates_tests.rs
│       ├── graph.rs               # MOVE from graph_tests.rs
│       ├── cnv4_advanced.rs       # KEEP in place
│       ├── cnv4_integration.rs    # KEEP in place
│       └── advanced_property_tests.rs # KEEP in place
│
└── explanations/                   # 🧠 Understanding-oriented
    ├── mod.rs
    ├── edge_cases.rs              # MOVE from edge_cases.rs
    ├── hotpath_optimization.rs    # MOVE from hotpath_tests.rs
    ├── architecture.rs            # CREATE (150 lines)
    └── noun_verb_pattern.rs       # CREATE (120 lines)
```

## File Movement Checklist

### Phase 1: Setup (30 min)
- [ ] Create all directories
- [ ] Create all mod.rs files
- [ ] Verify: `cargo test` still passes

### Phase 2: Priority 1 Moves (2 hours)
- [ ] Move 3 how-to files
- [ ] Move 2 explanation files
- [ ] Move 3 advanced reference files
- [ ] Verify each: `cargo test --test <file>`
- [ ] Commit: "feat: Move Priority 1 files"

### Phase 3: Priority 2 Moves (3.5 hours)
- [ ] Move 3 core reference files
- [ ] Move 1 cli reference file
- [ ] Move 1 runtime reference file
- [ ] Move 1 logic reference file
- [ ] Move 1 macro reference file
- [ ] Move 1 validation reference file
- [ ] Move 3 additional advanced files
- [ ] Verify: `cargo test --all-features`
- [ ] Commit: "feat: Move Priority 2 files"

### Phase 4: Merge Duplicates (2 hours)
- [ ] Merge cli_builder files → reference/cli/builder.rs
- [ ] Merge cli_validator files → reference/cli/validator.rs
- [ ] Merge runtime_executor files → reference/runtime/executor.rs
- [ ] Merge logic_handler files → reference/logic/handler.rs
- [ ] Remove old files
- [ ] Verify: `cargo test --test reference`
- [ ] Commit: "feat: Merge duplicate test files"

### Phase 5: Create New Tests (3.25 hours)
- [ ] Create tutorials/hello_world.rs
- [ ] Create tutorials/basic_noun_verb.rs
- [ ] Create tutorials/adding_arguments.rs
- [ ] Create explanations/architecture.rs
- [ ] Create explanations/noun_verb_pattern.rs
- [ ] Verify: `cargo test --test tutorials`
- [ ] Verify: `cargo test --test explanations`
- [ ] Commit: "feat: Add tutorial and explanation tests"

### Phase 6: Documentation (45 min)
- [ ] Write tests/README.md
- [ ] Update main README.md
- [ ] Verify: All docs accurate
- [ ] Commit: "docs: Add Diataxis test organization docs"

### Phase 7: Final Validation (30 min)
- [ ] Run full test suite: `cargo test --all-features --all-targets`
- [ ] Compare with baseline: `diff baseline_tests.log final_tests.log`
- [ ] Verify backward compat
- [ ] Create PR

## mod.rs Template

```rust
//! # <Quadrant Name> - <Purpose>
//!
//! <Description of what tests in this quadrant do>
//!
//! See: [README - <Section>](../../README.md#<anchor>)

#[cfg(test)]
mod file1;
#[cfg(test)]
mod file2;
```

**Example**:
```rust
//! # Tutorials - Learning-Oriented Tests
//!
//! Step-by-step guides for beginners to learn clap-noun-verb.
//!
//! See: [README - Quick Start](../../README.md#quick-start)

#[cfg(test)]
mod hello_world;
#[cfg(test)]
mod basic_noun_verb;
```

## Running Tests by Quadrant

```bash
# All tests
cargo test

# By quadrant
cargo test --test tutorials        # Learning-oriented
cargo test --test howto             # Problem-oriented
cargo test --test reference         # API lookup
cargo test --test explanations      # Understanding-oriented

# By subsystem (reference)
cargo test --test reference::core
cargo test --test reference::cli
cargo test --test reference::runtime

# Specific file
cargo test --test tutorials::hello_world

# With output
cargo test -- --nocapture
```

## Verification Commands

```bash
# Before any changes
cargo test --all-features > baseline_tests.log

# After each phase
cargo test --all-features > phase_N_tests.log
diff baseline_tests.log phase_N_tests.log

# Final validation
cargo test --all-features --all-targets
cargo test --doc
cargo clippy --all-targets
```

## Git Commands

```bash
# Create feature branch
git checkout -b feat/diataxis-test-reorganization

# Move file (preserves history)
git mv tests/old_name.rs tests/new/location.rs

# Stage all changes
git add tests/

# Commit with descriptive message
git commit -m "feat: <description>"

# Push and create PR
git push origin feat/diataxis-test-reorganization
gh pr create --title "feat: Reorganize tests using Diataxis framework"
```

## Success Criteria

After implementation, verify:

- [ ] **85%+ Diataxis alignment** achieved
- [ ] **All tests pass** (cargo test --all-features)
- [ ] **Zero breaking changes** (backward compat 100%)
- [ ] **Documentation complete** (tests/README.md + main README)
- [ ] **CI/CD green** (all workflows pass)

## Quick ROI Calculation

| Metric | Value |
|--------|-------|
| **Total Effort** | 7.5 hours |
| **Alignment Gained** | +70 percentage points (15% → 85%) |
| **ROI** | 11.3 per hour |
| **Files Affected** | 29 files |
| **Backward Compat** | 100% |
| **Risk** | Low |

## When to Stop and Ask

**Stop if**:
- Tests fail after move
- Imports break
- CI/CD breaks
- Unsure which quadrant

**Otherwise**: Keep going - you're on track!

## Resources

- **Full Guide**: [TEST_REORGANIZATION_IMPLEMENTATION.md](TEST_REORGANIZATION_IMPLEMENTATION.md)
- **Architecture**: [TEST_REORGANIZATION_ARCHITECTURE.md](TEST_REORGANIZATION_ARCHITECTURE.md)
- **Comparison**: [TEST_REORGANIZATION_COMPARISON.md](TEST_REORGANIZATION_COMPARISON.md)
- **Summary**: [TEST_REORGANIZATION_SUMMARY.md](TEST_REORGANIZATION_SUMMARY.md)

---

**Ready?** Start with Phase 1: Setup (30 minutes)

```bash
mkdir -p tests/{tutorials,howto,reference/{core,cli,runtime,logic,macros,validation,advanced},explanations}
```
