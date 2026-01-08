# Test Engineer Brief - v6.0.0 Test Suite Design

**Agent ID**: test-engineer-v6
**Memory Key**: test_validation
**Dependencies**: Awaits v6_specification from Specification Agent
**Timeline**: Start at +10 min

## Mission
Design and implement comprehensive Chicago TDD test suite for v6.0.0, ensuring all breaking changes, new features, and edge cases are thoroughly tested with state-based behavior verification.

## Chicago TDD Principles (MANDATORY)

- **State-based testing**: Verify outputs and state changes, not implementation
- **Real collaborators**: Use real objects, minimize mocks
- **Behavior verification**: Test what code DOES (observable results)
- **AAA Pattern**: Arrange-Act-Assert structure for all tests
- **Observable**: Tests verify external behavior users can observe

## Work Steps

1. **Poll v6_specification** (2 min)
   - Retrieve breaking change specifications
   - Extract feature specifications
   - Get acceptance criteria

2. **Design Test Categories** (8 min)
   - Unit tests for individual components (80% coverage target)
   - Integration tests for feature interactions
   - Property tests for deterministic CLI behavior
   - Snapshot tests for output determinism
   - Edge case tests for error paths

3. **Create Breaking Change Tests** (10 min)
   - For each breaking change:
     - Test verifies new behavior is correct
     - Test verifies old behavior no longer works
     - Test demonstrates migration path
   - Example: Old API should not compile (verify type error)
   - Example: New API produces expected output

4. **Implement Feature Tests** (10 min)
   - For each new feature:
     - Test verifies feature works as specified
     - Test covers happy path AND error cases
     - Test state changes (if applicable)
   - Chicago TDD: Behavior verification, not implementation details

5. **Test Coverage Analysis** (5 min)
   - Run coverage analysis on new code
   - Target: 80%+ coverage
   - Identify gaps and add tests
   - Focus on critical paths

6. **Store Test Suite in Memory** (2 min)
   - Save test_validation findings
   - Include test code, structure, coverage report
   - Ready for CI/CD pipeline

## Test Structure

```rust
// Example: Chicago TDD test structure
#[test]
fn test_breaking_change_old_api_fails_to_compile() {
    // ARRANGE
    let builder = OldAPIBuilder::new();

    // ACT & ASSERT
    // Code should not compile - this is verified by type system
    // Don't actually include this code, just verify compiler rejects it
}

#[test]
fn test_breaking_change_new_api_works() {
    // ARRANGE
    let builder = NewAPIBuilder::new()
        .with_feature(Feature::Enhanced)
        .build();

    // ACT
    let result = builder.execute();

    // ASSERT (behavior verification)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().feature_count(), 1);
    // Verify observable state changed as expected
}

#[test]
fn test_new_feature_integration() {
    // ARRANGE - Set up realistic scenario
    let mut cli = ClapNounVerbBuilder::new();
    cli.add_command(/* ... */);

    // ACT - Execute the feature
    let output = cli.run(/* ... */);

    // ASSERT - Verify observable behavior
    assert!(output.is_success);
    assert_eq!(output.commands_executed, 2);
}
```

## Deliverables

### Test Suite Files
- `/tests/v6_breaking_changes_test.rs` - Tests for breaking changes
- `/tests/v6_features_test.rs` - Tests for new features
- `/tests/v6_integration_test.rs` - Integration tests
- `/tests/v6_edge_cases_test.rs` - Error paths and edge cases

### Test Coverage Report
```
Module                Coverage
src/main.rs          85%
src/builder.rs       82%
src/parser.rs        88%
...
TOTAL                84%
```

### Test Documentation
- Test structure explanation
- How tests verify breaking changes
- How tests demonstrate migration
- Coverage analysis

## Constraints (CRITICAL)
- **NO meaningless tests** - Tests must verify observable behavior
- **NO implementation details** - Test behavior, not how it's coded
- **AAA PATTERN MANDATORY** - Arrange, Act, Assert for every test
- **CHICAGO TDD STRICT** - State-based with real collaborators
- **Coverage target**: 80%+ for new code
- **All tests must pass** - This is a blocking validation

## Success Criteria
- ✅ All breaking changes have tests
- ✅ All new features have tests
- ✅ Edge cases and error paths tested
- ✅ Chicago TDD principles strictly followed
- ✅ AAA pattern in all tests
- ✅ 80%+ coverage on new code
- ✅ All tests pass
- ✅ Memory key test_validation populated
- ✅ Ready for CI/CD

## Critical Andon Signal
**IF ANY TEST FAILS**: This is a STOP-THE-LINE signal
- Investigation required immediately
- Root cause analysis (5 Whys)
- Fix must address root cause
- Tests must pass before release consideration

## Notes
- Tests are TRUTH - if tests pass, code works
- Never claim implementation complete without passing tests
- Use real data, real scenarios in tests
- Mock minimally - test with real collaborators
- Verify observable outputs, not internal state (unless observable)
