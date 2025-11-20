# Diataxis How-To Guides - Integration Test Scenarios

**Test Suite**: How-To Guide Validation (Chicago TDD)
**Purpose**: Validate task-oriented guides provide working solutions
**Focus**: Problem-solving documentation for machines
**Coverage**: FM-04, FM-07, FM-12, FM-16, FM-24

---

## Test Category 1: Helper Function Tests

### Test: FM-04 - Validation Helper Function Compilation

**Failure Mode**: How-To validation example uses undefined `get_all_capabilities()` (RPN: 640)
**Location**: `DIATAXIS_V5_HOW_TO_GUIDES.md`, lines 32-52

#### Arrange
- Extract `get_all_capabilities()` function from How-To
- Check if `Capability` type is defined anywhere
- Setup test project with serde_json dependency
- Create mock CLI for testing

#### Act
- Attempt to compile helper function
- Grep codebase for `Capability` struct definition
- Search for import path: `use clap_noun_verb::v5::Capability`
- Test function with actual CLI output

#### Assert
- **PASS**: `Capability` type is defined (struct with id, name, description)
- **PASS**: Function compiles without errors
- **PASS**: Function can parse actual CLI JSON output
- **PASS**: No dangerous unwraps (proper error handling)
- **FAIL (Current State)**: Type not defined, function incomplete

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler (easy - Detection score 2/10)

**Recovery Path**:
1. Define `Capability` struct at top of document:
```rust
#[derive(Debug, Deserialize)]
struct Capability {
    id: String,
    name: String,
    description: String,
}
```
2. Replace unwraps with proper error handling
3. Test against actual CLI output
4. Verify function is self-contained

---

### Test: FM-07 - Guard Evaluation Implementation

**Failure Mode**: How-To guard evaluation uses pseudocode instead of executable Rust (RPN: 504)
**Location**: `DIATAXIS_V5_HOW_TO_GUIDES.md`, lines 340-362

#### Arrange
- Extract guard evaluation code from How-To
- Identify pseudocode sections (e.g., `registry_contains(name)`)
- Prepare test cases for guard conditions
- Setup actual guard evaluation environment

#### Act
- Attempt to compile guard evaluation code
- Identify non-executable pseudocode patterns
- Replace pseudocode with actual Rust implementation
- Test guard evaluation with real conditions

#### Assert
- **PASS**: All guard conditions are executable Rust code
- **PASS**: No pseudocode syntax remains
- **PASS**: Guards evaluate correctly (true/false outputs)
- **PASS**: Code compiles and runs
- **FAIL (Current State)**: Contains pseudocode, not executable

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler + Static Analysis (moderate - Detection score 3/10)

**Recovery Path**:
1. Replace `registry_contains(name)` with actual function call:
```rust
// Before (pseudocode)
.check(|ctx| registry_contains(ctx.name))

// After (executable)
.check(|ctx| {
    let name = ctx.get_arg("name")?;
    TemplateRegistry::global().contains(name)
})
```
2. Implement all helper functions
3. Test guard evaluation end-to-end
4. Document guard condition syntax formally

---

### Test: FM-12 - Workflow Chain Type Definitions

**Failure Mode**: How-To workflow chain uses undefined AgentContext type (RPN: 441)
**Location**: `DIATAXIS_V5_HOW_TO_GUIDES.md`, lines 634-722

#### Arrange
- Extract workflow chain code from How-To
- Check if `AgentContext` type exists
- Verify workflow chaining API
- Setup multi-step workflow test

#### Act
- Grep codebase for `AgentContext` definition
- Attempt to compile workflow example
- Test workflow execution with multiple steps
- Verify context passing between steps

#### Assert
- **PASS**: `AgentContext` type is defined
- **PASS**: Context can be passed between workflow steps
- **PASS**: Code compiles and links correctly
- **PASS**: Workflow executes end-to-end
- **FAIL (Current State)**: Type never defined or imported

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Compiler (easy - Detection score 3/10)

**Recovery Path**:
1. Define `AgentContext` struct:
```rust
#[derive(Debug, Clone)]
struct AgentContext {
    agent_id: String,
    session_id: String,
    capabilities: Vec<String>,
    state: HashMap<String, Value>,
}
```
2. Show context creation and passing
3. Test workflow with actual context
4. Document context lifecycle

---

## Test Category 2: Error Handling Tests

### Test: FM-16 - Result<T, E> Unwrapping Safety

**Failure Mode**: How-To examples don't handle Result<T, E> unwrapping properly (RPN: 360)
**Location**: Multiple locations throughout How-To Guides

#### Arrange
- Extract all code examples from How-To Guides
- Identify uses of `?` operator
- Check for proper error handling
- Identify dangerous unwraps

#### Act
- Parse all code for `?` usage
- Verify error types are propagated correctly
- Check for `.unwrap()` calls without context
- Test error paths (simulate failures)

#### Assert
- **PASS**: All `?` operators propagate errors correctly
- **PASS**: Functions using `?` have proper Result return types
- **PASS**: No `.unwrap()` in production code
- **PASS**: Error messages are informative
- **FAIL (Current State)**: Missing error handling, dangerous unwraps

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Static Analysis + Testing (moderate - Detection score 3/10)

**Recovery Path**:
1. Replace unwraps with proper error handling:
```rust
// Before (dangerous)
let value = json["field"].as_str().unwrap();

// After (safe)
let value = json["field"]
    .as_str()
    .ok_or_else(|| anyhow!("Missing 'field' in JSON"))?;
```
2. Add error context with `anyhow` or `thiserror`
3. Show complete error handling examples
4. Test error paths explicitly

---

### Test: FM-24 - Streaming Parser Correctness

**Failure Mode**: How-To streaming example uses BufRead::lines which blocks on incomplete JSON (RPN: 225)
**Location**: `DIATAXIS_V5_HOW_TO_GUIDES.md`, streaming section

#### Arrange
- Extract streaming parsing code
- Create test with incomplete JSON lines
- Setup streaming data source
- Prepare test cases for partial data

#### Act
- Test streaming parser with complete JSON
- Test with incomplete JSON lines (mid-stream)
- Test with buffered data
- Measure blocking behavior

#### Assert
- **PASS**: Parser doesn't block on incomplete lines
- **PASS**: Handles streaming JSON correctly
- **PASS**: Buffers partial data appropriately
- **PASS**: No data loss or corruption
- **FAIL (Current State)**: BufRead::lines blocks incorrectly

**Test Status**: ❌ FAIL (as of 2025-11-20)

**Detection**: Runtime Testing (moderate - Detection score 3/10)

**Recovery Path**:
1. Replace `BufRead::lines()` with proper streaming parser:
```rust
// Before (blocks)
for line in BufReader::new(stream).lines() {
    let json: Value = serde_json::from_str(&line?)?;
}

// After (streams)
let mut buffer = Vec::new();
let mut reader = BufReader::new(stream);
loop {
    buffer.clear();
    let n = reader.read_until(b'\n', &mut buffer)?;
    if n == 0 { break; }
    if let Ok(json) = serde_json::from_slice::<Value>(&buffer) {
        // Process complete JSON
    }
}
```
2. Test with real streaming data
3. Document streaming best practices

---

## Test Category 3: Task-Oriented User Journeys

### Test: UJ-HT-01 - Query and Validate Commands

**Scenario**: Machine needs to query CLI and validate capabilities

#### Arrange
- Start with How-To: "Query Commands"
- Follow step-by-step instructions
- Track success/failure at each step

#### Act - Step 1: Run introspection
```bash
./myapp --introspect > capabilities.json
```

**Assert Step 1**: ✅ PASS (introspection works)

#### Act - Step 2: Parse capabilities
- Copy `get_all_capabilities()` helper
- Attempt to compile
- Run against actual JSON

**Assert Step 2**:
- ❌ FAIL: `Capability` type undefined (FM-04)
- **BLOCKED**: Cannot parse capabilities

**Test Result**: ❌ FAIL at step 2 (50% completion)

---

### Test: UJ-HT-02 - Validate Command Guards

**Scenario**: Machine needs to implement guard validation

#### Arrange
- Navigate to "Validate Guards" How-To
- Extract guard validation code
- Prepare test guards

#### Act - Step 1: Copy guard evaluation code
- Extract guard evaluation logic
- Attempt to compile

**Assert Step 1**:
- ❌ FAIL: Pseudocode not executable (FM-07)
- **BLOCKED**: Cannot evaluate guards

**Test Result**: ❌ FAIL immediately (0% completion)

---

### Test: UJ-HT-03 - Chain Multiple Commands

**Scenario**: Machine needs to orchestrate multi-step workflow

#### Arrange
- Navigate to "Workflow Chains" How-To
- Extract chaining example
- Prepare multi-step test

#### Act - Step 1: Setup workflow context
- Copy `AgentContext` usage
- Attempt to compile

**Assert Step 1**:
- ❌ FAIL: `AgentContext` type undefined (FM-12)
- **BLOCKED**: Cannot setup context

**Test Result**: ❌ FAIL immediately (0% completion)

---

## Test Coverage Summary

### Critical Failure Modes Tested (RPN > 600)
- ✅ FM-04: Validation helper undefined (Test: FM-04)

### Coverage: 1 / 5 critical failures (20%)

### High-Priority Failure Modes Tested (RPN > 400)
- ✅ FM-07: Guard evaluation pseudocode (Test: FM-07)
- ✅ FM-12: Workflow context undefined (Test: FM-12)

### Coverage: 3 / 8 critical + high failures (37.5%)

### Medium-Priority Failure Modes Tested (RPN > 300)
- ✅ FM-16: Result unwrapping (Test: FM-16)

---

## Test Execution Results

### Current State (2025-11-20)
- **Total Tests**: 8
- **Passed**: 0 ❌
- **Failed**: 8 ❌
- **Pass Rate**: 0%

### User Journey Tests
- **UJ-HT-01**: ❌ FAIL (50% completion - blocked at step 2)
- **UJ-HT-02**: ❌ FAIL (0% completion - immediate blockage)
- **UJ-HT-03**: ❌ FAIL (0% completion - immediate blockage)
- **Overall Success Rate**: 0%

---

## Recommendations

### Immediate Actions
1. **Define Missing Types**: Add `Capability`, `AgentContext` struct definitions
2. **Replace Pseudocode**: Convert all pseudocode to executable Rust
3. **Fix Error Handling**: Replace unwraps with proper error propagation
4. **Test Streaming**: Fix BufRead blocking issues

### Validation Steps
1. Compile each How-To example in isolation
2. Test against actual CLI output
3. Verify error paths execute correctly
4. Test streaming with real data

### Success Criteria
- All helper functions compile ✅
- All types are defined ✅
- No pseudocode remains ✅
- Error handling is complete ✅
- Streaming works correctly ✅

---

**Test Suite Version**: 1.0.0
**Last Updated**: 2025-11-20
**Tester**: QA Agent (Hive Mind Swarm)
