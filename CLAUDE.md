# Claude Code Configuration - clap-noun-verb Rust Project

## 📋 Project Identity

**clap-noun-verb** is a Rust CLI tool that parses Clap arguments and generates noun-verb command structures.

- **Language**: Rust (stable toolchain)
- **Architecture**: Binary crate (main) with supporting modules
- **Methodology**: SPARC + Chicago TDD + DfLSS (Design for Lean Six Sigma)
- **Core Principles**: Type-first thinking, zero-cost abstractions, memory safety, deterministic outputs

## 🚨 CRITICAL: CONCURRENT EXECUTION & FILE MANAGEMENT

**ABSOLUTE RULES**:
1. ALL operations MUST be concurrent/parallel in a single message
2. **NEVER save working files, text/mds and tests to the root folder**
3. ALWAYS organize files in appropriate subdirectories
4. **USE CLAUDE CODE'S TASK TOOL** for spawning agents concurrently, not just MCP
5. **NEVER USE DIRECT CARGO COMMANDS - ALWAYS USE `cargo make`**

### ⚡ GOLDEN RULE: "1 MESSAGE = ALL RELATED OPERATIONS"

**MANDATORY PATTERNS:**
- **TodoWrite**: ALWAYS batch ALL todos in ONE call (10+ todos minimum per .cursorrules)
- **Task tool (Claude Code)**: ALWAYS spawn ALL agents in ONE message with full instructions
- **File operations**: ALWAYS batch ALL reads/writes/edits in ONE message
- **Bash commands**: ALWAYS batch ALL terminal operations in ONE message
- **Memory operations**: ALWAYS batch ALL memory store/retrieve in ONE message

### 🎯 CRITICAL: Claude Code Task Tool for Agent Execution

**Claude Code's Task tool is the PRIMARY way to spawn agents:**
```rust
// ✅ CORRECT: Use Claude Code's Task tool for parallel agent execution
[Single Message]:
  Task("System Architect", "Design type-first API with zero-cost abstractions...", "system-architect")
  Task("Rust Coder", "Implement core features with Result<T,E>...", "coder")
  Task("Test Engineer", "Create Chicago TDD tests with behavior verification...", "tester")
  Task("Code Reviewer", "Review for type safety and Andon signals...", "reviewer")
  Task("Performance Benchmarker", "Benchmark against SLOs...", "performance-benchmarker")
```

**MCP tools are ONLY for coordination setup:**
- `mcp__claude-flow__swarm_init` - Initialize coordination topology
- `mcp__claude-flow__agent_spawn` - Define agent types for coordination
- `mcp__claude-flow__task_orchestrate` - Orchestrate high-level workflows

### 📁 File Organization Rules

**NEVER save to root folder. Use these directories:**
- `/src` - Source code files
- `/tests` - Integration tests
- `/docs` - Documentation and markdown files
- `/scripts` - Utility scripts (bash scripts with timeout wrappers)
- `/benches` - Benchmark suites
- `/examples` - Example code and demos

## Project Overview

This project uses SPARC (Specification, Pseudocode, Architecture, Refinement, Completion) methodology with Chicago TDD for Test-Driven Development in Rust.

## 🚨 CRITICAL: Andon Signals (Stop the Line)

**Andon signals are visual problem indicators - treat compiler errors, test failures, and warnings as stop signals.**

### Signal Types:
- **CRITICAL (Red) - Must stop immediately**:
  - Compiler errors (`error[E...]`)
  - Test failures (`test ... FAILED`)
- **HIGH (Yellow) - Should stop**:
  - Compiler warnings (`warning:`)
  - Linting errors (clippy warnings/errors)

### Andon Signal Workflow:
1. **Monitor**: Run `cargo make check`, `cargo make test`, `cargo make lint` to check for signals
2. **Stop**: When signal appears, immediately stop current work - do not proceed
3. **Investigate**: Use root cause analysis (5 Whys) to understand why signal appeared
4. **Fix**: Address root cause, not just symptom
5. **Verify**: Re-run checks to confirm signal cleared - signal must be cleared before work continues

**Andon Principle**: "Stop the line" - Any problem visible through signals must be fixed immediately. Never proceed with signals present.

## 🔧 Build Commands (ALWAYS USE `cargo make`)

**CRITICAL: NEVER USE DIRECT CARGO COMMANDS - THIS IS NON-NEGOTIABLE**

### Quick Feedback (Fast iteration):
- `cargo make check` - Quick compilation check (timeout 5s)
- `cargo make test-unit` - Unit tests only (timeout 10s)
- `cargo make test test_name` - Run single test
- `cargo make lint` - Clippy linting (NEVER use `cargo clippy` directly)

### Full Validation:
- `cargo make test` - All tests (timeout 10s unit + 30s integration)
- `cargo make pre-commit` - Format + lint + unit tests
- `cargo make ci` - Full CI pipeline
- `cargo make release-validate` - Comprehensive release checks

### Performance & Profiling:
- `cargo make slo-check` - Verify performance SLOs
- `cargo make bench` - Run benchmarks
- `cargo make profile` - Performance profiling

### Development Utilities:
- `cargo make timeout-check` - Verify timeout command exists before running tasks

## 🧪 Testing Strategy (Chicago TDD - MANDATORY)

**Chicago TDD**: State-based testing with real collaborators and behavior verification.

### Core Principles:
1. **State-based testing** - Verify outputs, not implementation
2. **Real collaborators** - Use real objects, minimize mocks
3. **Behavior verification** - Verify what code does (observable outputs/state changes)
4. **AAA pattern required** - Arrange-Act-Assert
5. **Tests verify**: Return values, state changes, side effects, execution order

### Test Categories:
- **Unit tests**: Colocated with source (`src/*_test.rs` or `#[cfg(test)] mod tests`)
- **Integration tests**: In `/tests`
- **Property tests**: Using `proptest` for command parsing
- **Snapshot tests**: Using `insta` for deterministic outputs

### Test Requirements:
- ✅ All public APIs must be tested
- ✅ Test error paths, edge cases, critical paths (80%+ coverage)
- ✅ **No meaningless tests** - Tests must verify observable outputs/state changes
- ✅ **Behavior verification** - Tests verify what code does, not just that functions exist
- ✅ **Never claim completion without running tests** - Tests must pass before work is done

## 🦀 Elite Rust Mindset & 80/20 Thinking

### Type-First Thinking:
- Types encode invariants; compiler as design tool
- Use types to make invalid states unrepresentable
- Const generics over runtime values
- Ask: **"What can I express in types?"** before "What values do I need?"

### Zero-Cost Awareness:
- Generics monomorphize (zero-cost)
- Const generics are zero-cost
- Macros expand efficiently
- References are zero-cost
- Trait objects have dynamic dispatch cost
- Ask: **"Is this abstraction zero-cost?"**

### Performance Intuition:
- References over owned values
- Stack over heap
- Minimize allocations
- Optimize hot paths (20% that matters)
- Ask: **"What's the performance characteristic?"**

### Memory Safety:
- Ownership is explicit
- Borrowing enables zero-cost
- Lifetimes prevent use-after-free
- Encapsulate unsafe in safe APIs
- Ask: **"What are the ownership semantics?"**

### API Design:
- Type-safe by default (errors impossible through types)
- Ergonomic interfaces (easy to use correctly, hard to misuse)
- Self-documenting types
- Explicit error handling (Result types, not panics)
- Ask: **"How can I make misuse impossible?"**

## 🚀 Hyper-Advanced Agents (PRIORITY for Rust)

**CRITICAL:** When task matches these agents' specializations, ALWAYS use them instead of basic agents.

- **`production-validator`** - Production readiness validation (dependencies, SLO compliance)
- **`code-analyzer`** - Advanced code quality analysis (type safety, architecture assessment)
- **`system-architect`** - System architecture design (API design, type-level solutions)
- **`performance-benchmarker`** - Performance measurement & optimization (SLO verification)
- **`backend-dev`** - Backend implementation (CLI tools, APIs, databases)
- **`task-orchestrator`** - Complex workflow orchestration (multi-phase development)

### Core Development (Use for simple tasks)
`coder`, `reviewer`, `tester`, `planner`, `researcher`

## 🎯 Claude Code vs MCP Tools

### Claude Code Handles ALL EXECUTION:
- **Task tool**: Spawn and run agents concurrently for actual work
- File operations (Read, Write, Edit, Glob, Grep)
- Rust code generation and programming
- Bash commands with timeout wrappers
- Implementation work
- Project navigation and analysis
- TodoWrite and task management (10+ todos per batch)
- Git operations
- Cargo make commands (NEVER direct cargo)
- Testing and debugging

### MCP Tools ONLY COORDINATE:
- Swarm initialization (topology setup)
- Agent type definitions (coordination patterns)
- Task orchestration (high-level planning)
- Memory management
- Performance tracking

**KEY**: MCP coordinates the strategy, Claude Code's Task tool executes with real agents.

## 🚀 Agent Execution Flow with Claude Code (Rust-Specific)

### The Correct Pattern:

1. **Optional**: Use MCP tools to set up coordination topology
2. **REQUIRED**: Use Claude Code's Task tool to spawn agents that do actual work
3. **REQUIRED**: Each agent uses `cargo make` commands (NEVER direct cargo)
4. **REQUIRED**: Each agent runs hooks for coordination
5. **REQUIRED**: Batch all operations in single messages

### Example: Rust CLI Feature Development with Andon Signals

```rust
// ✅ CORRECT: Rust-specific agent swarm with Andon signal workflow
[Single Message - Parallel Agent Execution]:

  Task("System Architect", "Design type-first API for argument parsing. Store architecture in memory.", "system-architect")
  Task("Rust Coder", "Implement CLI with clap and Result<T,E>. Use cargo make lint.", "coder")
  Task("Code Analyzer", "Review for type safety, zero-cost abstractions, memory safety.", "code-analyzer")
  Task("Test Engineer", "Create Chicago TDD tests: state-based, real collaborators, behavior verification.", "tester")
  Task("Performance Benchmarker", "Benchmark CLI against performance targets. Use cargo make bench.", "performance-benchmarker")
  Task("Production Validator", "Validate production readiness: dependencies, security, SLO compliance.", "production-validator")

  TodoWrite { todos: [
    {content: "Design type-first CLI API architecture", status: "in_progress", activeForm: "Designing CLI API"},
    {content: "Implement argument parsing with clap derives", status: "pending", activeForm: "Implementing argument parsing"},
    {content: "Run cargo make check to verify no compiler errors (Andon signal)", status: "pending", activeForm: "Running cargo make check"},
    {content: "Fix any compiler errors immediately (Stop the Line)", status: "pending", activeForm: "Fixing compiler errors"},
    {content: "Create Chicago TDD unit tests with AAA pattern and state verification", status: "pending", activeForm: "Creating Chicago TDD unit tests"},
    {content: "Run cargo make test to verify all tests pass (Andon signal)", status: "pending", activeForm: "Running cargo make test"},
    {content: "Fix failing tests immediately (Stop the Line)", status: "pending", activeForm: "Fixing failing tests"},
    {content: "Run cargo make lint to check for clippy warnings (Andon signal)", status: "pending", activeForm: "Running cargo make lint"},
    {content: "Fix clippy warnings immediately (Stop the Line)", status: "pending", activeForm: "Fixing clippy warnings"},
    {content: "Run cargo make slo-check to verify performance SLOs", status: "pending", activeForm: "Running cargo make slo-check"},
    {content: "Verify all Andon signals cleared before marking complete", status: "pending", activeForm: "Verifying Andon signals"}
  ]}

  Write "src/main.rs"
  Write "src/cli.rs"
  Write "tests/integration_test.rs"
```

## 🚨 Definition of Done (Andon Signals Enforced)

**BEFORE MARKING ANY TASK AS COMPLETE - MANDATORY VALIDATION CHECKS:**

### 1. Verify Timeout Command
```bash
cargo make timeout-check
```

### 2. Check for Compiler Errors (CRITICAL SIGNAL)
```bash
cargo make check
```
- **IF ERRORS FOUND**: STOP THE LINE - Fix immediately
- **VERIFY**: No `error[E...]` patterns in output

### 3. Check for Compiler Warnings (HIGH SIGNAL)
- **IF WARNINGS FOUND**: STOP THE LINE - Fix warnings
- **VERIFY**: No `warning:` patterns in output

### 4. Run Tests (CRITICAL SIGNAL)
```bash
cargo make test
```
- **IF TESTS FAIL**: STOP THE LINE - Extract failing tests and fix with rich todos
- **VERIFY**: No `test ... FAILED` patterns

### 5. Check for Linting Errors (HIGH SIGNAL)
```bash
cargo make lint
```
- **IF LINTING ERRORS FOUND**: STOP THE LINE - Fix before proceeding
- **VERIFY**: No clippy warnings/errors

### 6. Verify Performance SLOs
```bash
cargo make slo-check
```
- **VERIFY**: All SLOs met

### 7. Final Verification - All Signals Cleared
- ✅ `cargo make check` - No compiler errors or warnings
- ✅ `cargo make test` - All tests pass
- ✅ `cargo make lint` - No linting errors
- ✅ All failing tests fixed and removed from todos

**ONLY mark complete when ALL signals are cleared and ALL validation checks pass**

## 🚫 Prohibited Patterns (Production-Ready Standards)

- **NEVER USE DIRECT CARGO COMMANDS - ALWAYS USE `cargo make`**
- **NEVER USE `cargo fmt`, `cargo clippy`, `cargo test` DIRECTLY**
- **NEVER RUN COMMANDS WITHOUT TIMEOUT WRAPPERS**
- **NEVER IGNORE ANDON SIGNALS** - Stop the line when signals appear
- **NEVER PROCEED WITH SIGNALS PRESENT** - Do not continue with errors/warnings
- **NEVER SUPPRESS OR HIDE SIGNALS** - Do not use `#[allow(...)]` without fixing root cause
- **NEVER MARK COMPLETE WITHOUT VERIFYING SIGNALS CLEARED**
- No placeholders - No "In production, this would..." comments
- No TODOs - No TODO comments except `FUTURE:` prefix for documented future enhancements
- No `unimplemented!()` - Complete implementations required
- No `unwrap()`/`expect()` in production code - Use `Result<T, E>` error handling
- No stubs - No functions that always succeed without implementation
- No claims without verification - Never claim code works without test validation
- No `print!` or `println!` in library code - Use `log!` macros or alert macros
- No Chicago TDD violations - Must use state-based testing, real collaborators, AAA pattern

## 📊 Performance SLOs

Performance targets for clap-noun-verb:
- Compilation: Incremental ≤ 2s
- Tests: Unit ≤ 10s, Integration ≤ 30s
- CLI execution: ≤ 100ms end-to-end
- Memory usage: ≤ 10MB

## 💡 Integration Tips

1. Always use `cargo make` for all development workflows
2. Run Andon signal checks continuously (check, test, lint)
3. Stop the line immediately when Andon signals appear
4. Batch todos (10+ minimum) in single TodoWrite calls
5. Use Chicago TDD for all tests (AAA pattern, behavior verification)
6. Use type-first thinking to encode invariants at compile time
7. Prefer zero-cost abstractions (generics, const generics, macros)
8. Never proceed with compiler errors, test failures, or clippy warnings

## 📚 Support & Documentation

- **clap-noun-verb Repository**: https://github.com/sac/clap-noun-verb
- **Claude Flow Documentation**: https://github.com/ruvnet/claude-flow
- **Rust Book**: https://doc.rust-lang.org/book/

---

## 🎯 Key Associations & Mental Models

- **Types = invariants = compile-time guarantees**
- **Zero-cost = generics/macros/const generics**
- **Performance = references/stack/minimize allocations**
- **Ownership = explicit = memory safety**
- **APIs = type-safe = ergonomic = composable**
- **Tests = observable outputs = behavior verification**
- **Andon Signals = stop = fix = verify**
- **DfLSS = prevent defects AND waste from start**

## 📝 Remember

**Always use `cargo make` - NEVER direct cargo commands!**

**Stop the line when Andon signals appear - fix root cause before proceeding!**

**TodoWrite always has 10+ todos in a single batch!**

**Test results are truth - code doesn't work if tests don't pass!**

---

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
Never save working files, text/mds and tests to the root folder.
TODO LISTS ARE ALWAYS 10 ITEMS OR MORE. THEY ARE ALWAYS FULLY COMPLETED BEFORE PROGRESSING TO THE NEXT TASK.
