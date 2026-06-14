# CLI Testing & Validation Documentation

Complete skill guide for testing noun-verb commands in **clap-noun-verb v26.6.1**.

## 📚 Documentation Files

### 1. **CLI_TESTING_GUIDE.md** (Primary Reference)
The comprehensive testing skill guide covering:
- Manual CLI testing patterns
- Integration test design (AAA pattern)
- Testing feature-gated commands
- Error message validation
- Help text testing
- Test fixtures and scenarios
- Regression testing
- Performance and stress testing
- Build verification with `cargo make`

**Use this for:** Learning how to test CLIs, designing test suites, understanding patterns.

### 2. **CLI_TESTING_EXAMPLES.rs** (Practical Code Examples)
12 complete, runnable examples demonstrating:
1. Basic command structure tests
2. Argument parsing validation
3. Help text assertion
4. Error handling
5. Value parsing and type conversion
6. Complex multi-level commands
7. Global arguments
8. Multiple values and collections
9. Feature-gated command testing
10. Snapshot-style structure tests
11. Help flag variants (-h vs --help)
12. Complete integration test workflow

**Use this for:** Copy-paste patterns, implementation templates, concrete examples.

### 3. **CLI_TESTING_CHECKLIST.md** (Quick Reference)
Organized checklist for:
- Pre-development planning
- Build & compilation verification
- Manual testing steps
- Unit & integration test coverage
- Feature-gated testing
- Help text verification
- Error message validation
- Output format testing
- Performance testing
- Regression testing
- Debugging tips
- Exit codes convention
- Sign-off requirements

**Use this for:** Before committing, CI/CD integration, sign-off verification.

### 4. **CLI_TESTING_FIXTURES.md** (Reusable Test Infrastructure)
Patterns for building robust test infrastructure:
- CLI fixture builders (basic, advanced, builder pattern)
- Test data generators (values, scenarios, edge cases)
- Assertion helpers (subcommand verification, noun/verb extraction)
- Snapshot storage for help text and structure
- Complete test templates
- Performance fixtures for load testing

**Use this for:** Setting up test infrastructure, building helpers, creating reusable fixtures.

## 🚀 Quick Start

### Testing a New Command

1. **Read the structure guide:**
   - CLI_TESTING_GUIDE.md → Integration Test Design section

2. **Copy an example:**
   - CLI_TESTING_EXAMPLES.rs → Example matching your command type

3. **Verify with checklist:**
   - CLI_TESTING_CHECKLIST.md → Relevant section

### Testing an Existing Command

1. **Run the checklist:**
   - CLI_TESTING_CHECKLIST.md → All relevant sections

2. **Reference fixtures:**
   - CLI_TESTING_FIXTURES.md → Create test infrastructure

3. **Debug issues:**
   - CLI_TESTING_GUIDE.md → Troubleshooting section

## 📋 File Guide

| File | Purpose | Audience | Size |
|------|---------|----------|------|
| CLI_TESTING_GUIDE.md | Comprehensive reference | All | Large |
| CLI_TESTING_EXAMPLES.rs | Code examples | Developers | Medium |
| CLI_TESTING_CHECKLIST.md | Quick verification | QA/CI | Small |
| CLI_TESTING_FIXTURES.md | Test infrastructure | Architects | Medium |

## 🎯 Key Topics by Use Case

### "How do I test my first noun-verb command?"
→ CLI_TESTING_GUIDE.md (Integration Test Design) + CLI_TESTING_EXAMPLES.rs (Example 1-3)

### "What should I verify before merging?"
→ CLI_TESTING_CHECKLIST.md (Before Merging section)

### "How do I set up test fixtures?"
→ CLI_TESTING_FIXTURES.md (entire document)

### "How do I test feature-gated commands?"
→ CLI_TESTING_GUIDE.md (Testing Feature-Gated Commands) + CLI_TESTING_EXAMPLES.rs (Example 9)

### "What makes a good error message?"
→ CLI_TESTING_GUIDE.md (Error Message Validation)

### "How do I test help text?"
→ CLI_TESTING_GUIDE.md (Help Text Testing) + CLI_TESTING_EXAMPLES.rs (Example 3, 11)

### "How do I set up CI/CD testing?"
→ CLI_TESTING_GUIDE.md (Build Verification) + CLI_TESTING_CHECKLIST.md (Continuous Integration)

### "How do I create reusable test utilities?"
→ CLI_TESTING_FIXTURES.md (all sections)

## 🏗️ Architecture Overview

Tests follow the **Arrange-Act-Assert (AAA)** pattern:

```rust
#[test]
fn test_command_executes() {
    // ARRANGE: Set up test data
    let cmd = create_test_cli();

    // ACT: Execute the behavior being tested
    let result = cmd.try_get_matches_from(vec!["app", "noun", "verb"]);

    // ASSERT: Verify observable outcomes
    assert!(result.is_ok());
}
```

## 🔍 Test Levels

1. **Unit Tests** - Individual argument parsing, validation
2. **Integration Tests** - Complete command flow from CLI args to output
3. **Feature Tests** - Commands with feature gates enabled/disabled
4. **Regression Tests** - Comparing against known-good snapshots
5. **Performance Tests** - Throughput, latency, memory

## 🛠️ Build Commands

All testing uses `cargo make`:

```bash
# Quick tests
cargo make test

# All features
cargo make test-all

# Frontier features
cargo make test-frontier

# Full CI suite
cargo make ci

# Format check
cargo make format-check

# Linting
cargo make lint
```

See CLAUDE.md for complete build reference.

## 📊 Test Coverage Goals

- **Structure:** 100% - All nouns and verbs must be discoverable
- **Arguments:** 100% - Required/optional validation, type checking
- **Help Text:** 100% - All commands documented
- **Error Handling:** 100% - Invalid inputs rejected with helpful messages
- **Happy Path:** 100% - Valid commands execute without error
- **Edge Cases:** 80%+ - Unusual but valid inputs
- **Performance:** Baseline established - Incremental compilation ≤2s

## ⚠️ Common Pitfalls to Avoid

1. **Testing implementation, not behavior**
   - ❌ Assert internal state changed
   - ✓ Assert observable output/result changed

2. **Skipping error path tests**
   - ❌ Only testing `is_ok()`
   - ✓ Verify error message content

3. **Not testing help text**
   - ❌ Assuming it works
   - ✓ Verify all commands in help

4. **Feature gate misses**
   - ❌ Forgetting `#[cfg(feature = "...")]`
   - ✓ Explicitly test with/without feature

5. **Flaky tests**
   - ❌ Tests depending on execution order
   - ✓ Each test independent

6. **Ignoring error messages**
   - ❌ Just checking `is_err()`
   - ✓ Verify error message is helpful

## 🔗 Related Documentation

- **CLAUDE.md** - Project overview, build system, feature flags
- **src/cli/mod.rs** - CLI architecture
- **src/builder.rs** - CliBuilder API
- **src/router.rs** - CommandRouter
- **tests/common/mod.rs** - Built-in assertion helpers

## 💡 Tips for Success

1. **Start with manual testing** - Run the CLI manually first
2. **Use fixtures** - Build reusable CLI factories
3. **Test scenarios** - Create scenario builders for complex flows
4. **Snapshot help text** - Store and compare help output
5. **Performance baseline** - Establish baseline metrics early
6. **CI integration** - Run full checklist in CI/CD
7. **Keep fixtures simple** - One responsibility per fixture
8. **Document patterns** - Reuse successful patterns

## 📖 Reading Order

For a complete understanding, read in this order:

1. CLI_TESTING_GUIDE.md - Understand the patterns
2. CLI_TESTING_EXAMPLES.rs - See concrete implementations
3. CLI_TESTING_FIXTURES.md - Build test infrastructure
4. CLI_TESTING_CHECKLIST.md - Use as verification

For quick reference:
- Reference: CLI_TESTING_GUIDE.md (use Ctrl+F)
- Patterns: CLI_TESTING_EXAMPLES.rs
- Verification: CLI_TESTING_CHECKLIST.md
- Infrastructure: CLI_TESTING_FIXTURES.md

## ✅ Verification Workflow

Before committing:
```
1. cargo make format-check    # Format OK?
2. cargo make clippy           # No warnings?
3. cargo make lint             # All checks pass?
4. cargo make test             # Tests pass?
5. Use CLI_TESTING_CHECKLIST   # Manual verification
6. cargo make ci               # Full CI suite
```

## 📞 Getting Help

- **How do I...?** → Look in CLI_TESTING_GUIDE.md TOC
- **Show me an example** → See CLI_TESTING_EXAMPLES.rs
- **Did I test everything?** → Use CLI_TESTING_CHECKLIST.md
- **How do I set this up?** → Read CLI_TESTING_FIXTURES.md
- **Something failed** → Check Troubleshooting in CLI_TESTING_GUIDE.md

---

**Version:** Aligned with clap-noun-verb v26.6.1
**Last Updated:** 2024
**Maintained By:** clap-noun-verb maintainers
