# Diataxis Tutorials - Integration Test Scenarios

**Test Suite**: Tutorial Validation (Chicago TDD)
**Purpose**: Validate Tutorial code examples compile and execute correctly
**Focus**: Learning-oriented documentation for machines
**Coverage**: FM-01, FM-02, FM-03, FM-05, FM-08, FM-15

---

## Test Category 1: Code Compilation Tests

### Test: FM-01 - Tutorial 1 Code Example Compilation

**Failure Mode**: Tutorial 1 code example doesn't compile (RPN: 672)
**Location**: `DIATAXIS_V5_TUTORIALS.md`, lines 26-46

#### Arrange
- Extract Rust code block from Tutorial 1 (lines 26-46)
- Setup test Rust project with `clap_noun_verb` dependency
- Prepare compiler environment

#### Act
- Attempt to compile extracted code with `cargo make check`
- Capture compilation output and error messages
- Record compilation time and exit code

#### Assert
- **PASS**: Code compiles without errors (exit code 0)
- **PASS**: No compiler warnings emitted
- **PASS**: All imports resolve correctly
- **PASS**: `#[noun]` and `#[verb]` attributes exist in crate
- **PASS**: `Result` type is properly qualified
- **FAIL (Current State)**: Code does NOT compile - missing attributes, unqualified Result

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler (easy - Detection score 2/10)

**Recovery Path**:
1. Fix imports: Add `use clap_noun_verb::{noun, verb};`
2. Qualify Result: `Result<Vec<String>, Box<dyn std::error::Error>>`
3. Verify attributes exist in codebase
4. Re-test with `cargo make check`

---

### Test: FM-02 - Tutorial 2 Agent Integration Code Compilation

**Failure Mode**: Tutorial 2 code example doesn't compile (RPN: 672)
**Location**: `DIATAXIS_V5_TUTORIALS.md`, lines 168-191

#### Arrange
- Extract agent integration code from Tutorial 2
- Setup test project with required dependencies (serde_json, std::process)
- Create test harness

#### Act
- Compile extracted code with `cargo make check`
- Check for unused imports
- Verify Result type qualification
- Execute code against mock CLI

#### Assert
- **PASS**: Code compiles successfully
- **PASS**: No unused imports (specifically `reqwest`)
- **PASS**: `Result<()>` is qualified with error type
- **PASS**: All `use` statements are necessary
- **PASS**: `std::process::Command` is properly imported
- **FAIL (Current State)**: Unqualified Result, unused reqwest import

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler + Linter (easy - Detection score 2/10)

**Recovery Path**:
1. Remove `use reqwest;` (unused)
2. Change `Result<()>` to `Result<(), Box<dyn std::error::Error>>`
3. Add explicit `use std::process::Command;`
4. Re-test with `cargo make lint`

---

### Test: FM-03 - Tutorial 3 Guard API Compilation

**Failure Mode**: Tutorial 3 guard implementation uses non-existent API (RPN: 672)
**Location**: `DIATAXIS_V5_TUTORIALS.md`, lines 300-346

#### Arrange
- Extract guard implementation code from Tutorial 3
- Check if `Guard` type exists in codebase
- Check if `Guard::new()` constructor exists
- Check if `TemplateRegistry` exists

#### Act
- Attempt to compile guard example
- Grep codebase for `Guard::new` implementation
- Grep for `TemplateRegistry` type definition
- Test builder pattern methods (`.description()`, `.check()`)

#### Assert
- **PASS**: `Guard` type exists in codebase
- **PASS**: `Guard::new()` constructor is implemented
- **PASS**: Builder pattern (`.description()`, `.check()`) works
- **PASS**: `ctx.arg()` method exists
- **PASS**: `TemplateRegistry` type is defined
- **FAIL (Current State)**: API doesn't exist - aspirational code

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler (easy - Detection score 2/10)

**Recovery Path (Option A - If implemented)**:
1. Verify Guard API is implemented
2. Update documentation with correct import path
3. Test example compiles

**Recovery Path (Option B - If not implemented)**:
1. Mark code as "Future API (v5.2+)"
2. Add `rust,ignore` to code block
3. Provide current working guard implementation
4. Document actual guard API

---

### Test: FM-05 - Tutorial 4 Delegation Policy Compilation

**Failure Mode**: Tutorial 4 references non-existent DelegationPolicy type (RPN: 640)
**Location**: `DIATAXIS_V5_TUTORIALS.md`, lines 471-488

#### Arrange
- Extract delegation policy code from Tutorial 4
- Check if `DelegationPolicy` type exists
- Check if `AgentRole` enum exists
- Verify builder pattern methods

#### Act
- Grep codebase for `DelegationPolicy` type
- Grep for `AgentRole` enum
- Attempt compilation
- Search for delegation implementation

#### Assert
- **PASS**: `DelegationPolicy` type is defined
- **PASS**: `AgentRole` enum exists
- **PASS**: Builder methods (`.delegable_to()`, `.max_delegation_depth()`) work
- **PASS**: Code compiles and links
- **FAIL (Current State)**: Types don't exist in codebase

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler (easy - Detection score 2/10)

**Recovery Path**:
1. Check if delegation is implemented in v5
2. If YES: Document actual API and fix example
3. If NO: Mark as "Planned Feature" and show workaround
4. Provide working alternative for multi-agent coordination

---

### Test: FM-08 - Tutorial 5 MCP Integration Compilation

**Failure Mode**: Tutorial 5 MCP integration uses hypothetical API (RPN: 504)
**Location**: `DIATAXIS_V5_TUTORIALS.md`, lines 586-610

#### Arrange
- Extract MCP integration code from Tutorial 5
- Check if `mcp_server` crate exists
- Verify MCP dependencies in `Cargo.toml`
- Check for MCP client implementation

#### Act
- Grep `Cargo.toml` for MCP dependencies
- Attempt to compile MCP example
- Verify MCP protocol implementation
- Test JSON-RPC 2.0 communication

#### Assert
- **PASS**: `mcp_server` crate is specified in dependencies
- **PASS**: MCP client implementation exists
- **PASS**: Code compiles with MCP integration
- **PASS**: JSON-RPC 2.0 protocol is implemented
- **FAIL (Current State)**: MCP API is hypothetical, not implemented

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler + Runtime (moderate - Detection score 3/10)

**Recovery Path**:
1. Clarify MCP implementation status
2. If implemented: Fix code and update imports
3. If planned: Mark as "Future Feature (v5.3+)"
4. Provide current integration approach

---

## Test Category 2: Type Safety Tests

### Test: FM-15 - Async/Await Pattern Verification

**Failure Mode**: Tutorial progression assumes synchronous execution but v5 is async (RPN: 378)

#### Arrange
- Extract all code examples from Tutorials
- Identify functions that should be async
- Check for missing `async` keywords
- Check for missing `await` keywords

#### Act
- Parse each code example for async patterns
- Verify `async fn` declarations
- Verify `.await` calls on futures
- Check for blocking operations in async context

#### Assert
- **PASS**: All async functions marked with `async`
- **PASS**: All futures are awaited
- **PASS**: No blocking operations in async context
- **PASS**: Tokio runtime is properly initialized
- **FAIL (Current State)**: Examples missing async/await

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler + Runtime (moderate - Detection score 3/10)

**Recovery Path**:
1. Add `async` to function signatures
2. Add `.await` to future calls
3. Show Tokio runtime setup
4. Document async best practices

---

## Test Category 3: User Journey Tests

### Test: UJ-01 - New User Learning Path (Tutorial 1 → Tutorial 5)

**Scenario**: Machine agent attempting to learn v5 from scratch

#### Arrange
- Start with blank knowledge (no prior v5 experience)
- Follow "New to v5" path from Index
- Track success/failure at each step

#### Act - Step 1: Read Explanations
- Navigate to Explanations document
- Read conceptual overview
- Extract key concepts

**Assert Step 1**:
- ✅ PASS: Explanations are readable (no executable code required)

#### Act - Step 2: Tutorial 1 (First Command)
- Navigate to Tutorial 1
- Copy first code example
- Attempt to compile

**Assert Step 2**:
- ❌ FAIL: Code doesn't compile (FM-01)
- **BLOCKED**: Cannot proceed to next tutorial

**Test Result**: ❌ FAIL at step 2 (50% completion)
**Blockage Rate**: 100% (all machines blocked at Tutorial 1)

---

### Test: UJ-02 - Experienced User Task-Oriented Path

**Scenario**: Machine with Rust experience trying to validate commands

#### Arrange
- Skip Tutorials, go directly to How-To Guides
- Follow "Query Commands" guide
- Attempt to implement validation

#### Act - Step 1: Copy validation helper
- Navigate to How-To Guides (line 32-52)
- Copy `get_all_capabilities()` function
- Attempt to compile

**Assert Step 1**:
- ❌ FAIL: `Capability` type undefined (FM-04)
- **BLOCKED**: Cannot implement validation

**Test Result**: ❌ FAIL immediately (0% completion)
**Blockage Rate**: 100% (blocked at first How-To example)

---

## Test Coverage Summary

### Critical Failure Modes Tested (RPN > 600)
- ✅ FM-01: Tutorial 1 compilation (Test: FM-01)
- ✅ FM-02: Tutorial 2 compilation (Test: FM-02)
- ✅ FM-03: Tutorial 3 Guard API (Test: FM-03)
- ✅ FM-05: Tutorial 4 DelegationPolicy (Test: FM-05)

### Coverage: 4 / 5 critical failures (80%)

### High-Priority Failure Modes Tested (RPN > 500)
- ✅ FM-08: Tutorial 5 MCP integration (Test: FM-08)

### Coverage: 5 / 8 critical + high failures (62.5%)

---

## Test Execution Results

### Current State (2025-11-20)
- **Total Tests**: 7
- **Passed**: 0 ❌
- **Failed**: 7 ❌
- **Pass Rate**: 0%

### Expected State (After Mitigation)
- **Total Tests**: 7
- **Passed**: 7 ✅
- **Failed**: 0
- **Pass Rate**: 100%

---

## Recommendations

### Immediate Actions (Week 1)
1. **Add CI Pipeline**: Extract and compile all Tutorial code blocks
2. **Fix FM-01 to FM-05**: Fix all compilation failures
3. **Tag Aspirational Code**: Mark unimplemented APIs clearly

### Validation Steps
1. Run `cargo make check` on every code example
2. Run `cargo make lint` to catch warnings
3. Run `cargo make test` to verify runtime behavior
4. Add automated test suite in CI

### Success Criteria
- All Tutorial code examples compile ✅
- Zero compiler warnings ✅
- Zero linting errors ✅
- 100% test pass rate ✅

---

**Test Suite Version**: 1.0.0
**Last Updated**: 2025-11-20
**Tester**: QA Agent (Hive Mind Swarm)
