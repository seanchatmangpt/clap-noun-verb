# CLI Testing & Validation Skill - Complete Index

## Overview

A comprehensive skill guide and reference for testing noun-verb commands in **clap-noun-verb v26.6.1**, including manual testing patterns, integration test design, feature-gated command testing, error validation, help text testing, test fixtures, and regression testing.

## 📄 Documentation Files

### 1. CLI_TESTING_README.md
**Purpose:** Overview and navigation guide  
**Length:** ~270 lines  
**Key Sections:**
- Quick start guides (by use case)
- File guide and reading order
- Architecture overview
- Common pitfalls
- Verification workflow

**When to use:** First, to understand what's available and where to look

---

### 2. CLI_TESTING_GUIDE.md (Main Reference)
**Purpose:** Comprehensive skill guide with patterns and practices  
**Length:** ~1,200 lines  
**Key Sections:**

1. **Manual CLI Testing Patterns** (lines 20-100)
   - Running single commands
   - Testing via examples
   - Common testing scenarios

2. **Integration Test Design** (lines 102-250)
   - AAA pattern structure
   - Testing command registration
   - Testing command dispatch
   - Testing argument validation
   - Test helpers and utilities

3. **Testing Feature-Gated Commands** (lines 252-400)
   - Feature configuration
   - Feature-gated tests with `#[cfg]`
   - Build commands for feature testing
   - Conditional compilation patterns
   - Testing feature interactions

4. **Error Message Validation** (lines 402-550)
   - Testing error creation
   - Testing error parsing and display
   - Structured errors with details
   - Levenshtein distance suggestions
   - Best practices for error testing

5. **Help Text Testing** (lines 552-750)
   - Testing help content
   - Testing subcommand help
   - Testing help flags (-h vs --help)
   - Testing argument help
   - Best practices for help testing

6. **Test Fixtures & Scenarios** (lines 752-1000)
   - Common test fixtures
   - Scenario builders
   - Test data generators
   - Fixture organization

7. **Regression Testing** (lines 1002-1150)
   - Snapshot testing for help text
   - Command structure snapshots
   - API contract tests
   - Mutation testing approach
   - Regression test checklist

8. **Performance & Stress Testing** (lines 1152-1300)
   - Argument parsing performance
   - Command registry performance
   - High-volume stress tests
   - Memory usage testing
   - Performance guidelines

9. **Build Verification** (lines 1302-1400)
   - Standard build verification
   - Feature-based verification
   - Test verification
   - Example building
   - Documentation generation
   - SLO verification

10. **Testing Workflow Examples** (lines 1400+)
    - Complete integration test
    - Feature-gated integration test
    - Quick reference assertions
    - Troubleshooting common issues

**When to use:** Learning patterns, designing test suites, understanding implementation

---

### 3. CLI_TESTING_EXAMPLES.rs (Code Examples)
**Purpose:** 12 complete, runnable code examples  
**Length:** ~710 lines  
**Examples:**

1. **Basic Command Structure** (lines 20-50)
   - Test that nouns and verbs exist
   - Verify CLI hierarchy

2. **Argument Parsing** (lines 52-120)
   - Required argument validation
   - Optional flag parsing

3. **Help Text** (lines 122-200)
   - Root help content
   - Subcommand help
   - Argument help descriptions

4. **Error Handling** (lines 202-270)
   - Invalid noun rejection
   - Invalid verb rejection
   - Unexpected arguments

5. **Value Parsing** (lines 272-330)
   - Numeric argument parsing
   - String choices validation

6. **Complex Commands** (lines 332-420)
   - Three-level command hierarchy
   - Multiple independent subcommands

7. **Global Arguments** (lines 422-470)
   - Global arguments with subcommands

8. **Multiple Values** (lines 472-550)
   - Append arguments
   - Value delimiters (comma-separated)

9. **Feature-Gated Testing** (lines 552-580)
   - `#[cfg(feature = "...")]` patterns

10. **Snapshot Tests** (lines 582-640)
    - Command structure snapshots
    - Verb structure verification

11. **Help Variants** (lines 642-680)
    - Testing -h and --help

12. **Complete Integration** (lines 682-710)
    - Full workflow combining multiple patterns

**When to use:** Copy-paste templates, implementation reference, concrete patterns

---

### 4. CLI_TESTING_CHECKLIST.md (Verification)
**Purpose:** Organized checklist for comprehensive testing  
**Length:** ~330 lines  
**Sections:**

- Pre-Development Checklist (8 items)
- Build & Compilation (6 items)
- Manual Testing (7 items)
- Unit Test Coverage (18 items)
- Integration Test Coverage (5 items)
- Feature-Gated Testing (8 items)
- Help Text Verification (17 items)
- Error Message Verification (6 items)
- Output Format Testing (6 items)
- Performance Testing (5 items)
- Example Testing (5 items)
- Regression Testing (6 items)
- Documentation (5 items)
- CI/CD (5 items)
- Pre-Commit (5 items)
- Pre-Merge (9 items)
- Testing by Feature Type (22 items)
- Debugging Tips (4 sections)
- Quick Commands (10 commands)
- Exit Codes Convention (5 codes)
- Testing Priority Order (4 levels)
- Sign-Off Checklist (9 items)

**When to use:** Before committing, in CI/CD pipelines, verification and sign-off

---

### 5. CLI_TESTING_FIXTURES.md (Test Infrastructure)
**Purpose:** Patterns for building reusable test infrastructure  
**Length:** ~690 lines  
**Sections:**

1. **File Structure** (lines 1-30)
   - Directory organization
   - Module layout

2. **CLI Fixture Builders** (lines 32-150)
   - Basic CLI builder
   - CLI with globals
   - CLI with complex arguments
   - Feature-gated CLI
   - Builder pattern implementation
   - Usage examples

3. **Test Data Generators** (lines 152-280)
   - Value generators (nouns, verbs, edge cases)
   - Numeric generators
   - Port numbers
   - JSON validation
   - Scenario builders
   - Scenario executor
   - Test data usage example

4. **Assertion Helpers** (lines 282-350)
   - Help text assertions
   - Noun/verb extraction
   - Command existence checks
   - Counting helpers
   - Test templates

5. **Snapshot Storage** (lines 352-400)
   - Help text snapshot example
   - Structure snapshot (JSON)

6. **Comprehensive Test Template** (lines 402-450)
   - Complete integration test example

7. **Existing Integration** (lines 452-480)
   - Module re-exports
   - Usage patterns

8. **Performance Fixtures** (lines 482-530)
   - Load test fixtures
   - Stress test setup
   - Smoke test patterns

9. **Maintenance Tips** (lines 532-545)
   - Best practices
   - Snapshot updates
   - Fixture versioning

**When to use:** Setting up test infrastructure, building helpers, creating reusable patterns

---

## 🎯 Navigation Guide

### By Task

**"I need to test my first noun-verb command"**
1. Start: CLI_TESTING_README.md (Quick Start)
2. Reference: CLI_TESTING_GUIDE.md → Integration Test Design
3. Copy from: CLI_TESTING_EXAMPLES.rs → Examples 1-3
4. Verify: CLI_TESTING_CHECKLIST.md → Unit Test Coverage

**"How do I test feature-gated commands?"**
1. Read: CLI_TESTING_GUIDE.md → Testing Feature-Gated Commands
2. Example: CLI_TESTING_EXAMPLES.rs → Example 9
3. Check: CLI_TESTING_CHECKLIST.md → Feature-Gated Testing

**"What should I verify before merging?"**
1. Use: CLI_TESTING_CHECKLIST.md → Before Merging/Publishing
2. Reference: CLI_TESTING_GUIDE.md → Build Verification
3. Complete: Regression Testing section

**"How do I set up test fixtures and helpers?"**
1. Read: CLI_TESTING_FIXTURES.md → File Structure & Builders
2. Reference: CLI_TESTING_GUIDE.md → Test Fixtures & Scenarios
3. Copy: CLI_TESTING_EXAMPLES.rs → Complete Integration Test

**"How do I test help text and error messages?"**
1. Read: CLI_TESTING_GUIDE.md → Help Text Testing & Error Message Validation
2. Examples: CLI_TESTING_EXAMPLES.rs → Examples 3, 4, 11
3. Check: CLI_TESTING_CHECKLIST.md → Help Text & Error Message Verification

**"Something is broken, how do I debug?"**
1. Check: CLI_TESTING_CHECKLIST.md → Debugging Tips
2. Reference: CLI_TESTING_GUIDE.md → Troubleshooting section
3. Run: Suggested commands from checklist

### By Role

**Developer (Writing Tests)**
1. CLI_TESTING_EXAMPLES.rs - Copy code patterns
2. CLI_TESTING_GUIDE.md - Understand patterns
3. CLI_TESTING_FIXTURES.md - Set up infrastructure

**QA Engineer (Verifying)**
1. CLI_TESTING_CHECKLIST.md - Run verification steps
2. CLI_TESTING_GUIDE.md - Reference details
3. CLI_TESTING_README.md - Understand architecture

**Architect (Designing Test Infrastructure)**
1. CLI_TESTING_FIXTURES.md - Complete design patterns
2. CLI_TESTING_GUIDE.md - Best practices
3. CLI_TESTING_CHECKLIST.md - Verification requirements

**CI/CD Engineer (Automating Tests)**
1. CLI_TESTING_CHECKLIST.md - Test matrix
2. CLI_TESTING_GUIDE.md → Build Verification
3. CLI_TESTING_README.md - Verification workflow

## 📊 Content Coverage

| Topic | Guide | Examples | Checklist | Fixtures |
|-------|-------|----------|-----------|----------|
| Manual Testing | ✓ | | | |
| Integration Tests | ✓ | ✓✓✓ | ✓ | ✓ |
| Argument Validation | ✓ | ✓✓ | ✓ | ✓ |
| Help Text Testing | ✓ | ✓ | ✓ | |
| Error Handling | ✓ | ✓ | ✓ | |
| Feature Gates | ✓ | ✓ | ✓ | ✓ |
| Fixtures & Data | ✓ | ✓ | | ✓✓✓ |
| Regression Testing | ✓ | ✓ | ✓ | ✓ |
| Performance Testing | ✓ | | ✓ | ✓ |
| Build Verification | ✓ | | ✓ | |
| Best Practices | ✓ | | ✓ | ✓ |

## 🔍 Quick Reference Index

### Common Patterns

- **AAA Pattern (Arrange-Act-Assert):** CLI_TESTING_GUIDE.md line ~140
- **Command Structure Test:** CLI_TESTING_EXAMPLES.rs Example 1
- **Argument Validation Test:** CLI_TESTING_EXAMPLES.rs Example 2
- **Help Text Assertion:** CLI_TESTING_EXAMPLES.rs Example 3
- **Error Testing:** CLI_TESTING_EXAMPLES.rs Example 4
- **Feature-Gated Test:** CLI_TESTING_EXAMPLES.rs Example 9
- **CLI Builder Pattern:** CLI_TESTING_FIXTURES.md line ~100

### Key Commands

- **Format check:** `cargo make format-check`
- **Run tests:** `cargo make test`
- **Test all features:** `cargo make test-all`
- **Full CI:** `cargo make ci`
- **Build release:** `cargo make build-release`

### Key Concepts

- **Test levels:** CLI_TESTING_README.md
- **Coverage goals:** CLI_TESTING_README.md
- **Common pitfalls:** CLI_TESTING_README.md
- **Exit codes:** CLI_TESTING_CHECKLIST.md
- **Performance SLOs:** CLAUDE.md (referenced in guides)

## 📝 Document Statistics

| Document | Lines | Purpose | Audience |
|----------|-------|---------|----------|
| CLI_TESTING_README.md | 272 | Navigation & overview | All |
| CLI_TESTING_GUIDE.md | 1,197 | Comprehensive patterns | All |
| CLI_TESTING_EXAMPLES.rs | 711 | Code examples | Developers |
| CLI_TESTING_CHECKLIST.md | 334 | Verification | QA/CI |
| CLI_TESTING_FIXTURES.md | 691 | Infrastructure | Architects |
| **Total** | **3,205** | | |

## 🚀 Getting Started

1. **Read:** CLI_TESTING_README.md (5 minutes)
2. **Learn:** CLI_TESTING_GUIDE.md sections relevant to your task (15-30 minutes)
3. **Implement:** CLI_TESTING_EXAMPLES.rs patterns (copy & adapt)
4. **Verify:** CLI_TESTING_CHECKLIST.md (before committing)
5. **Build:** Use CLI_TESTING_FIXTURES.md for infrastructure (as needed)

## 🔗 Related Documentation

- **CLAUDE.md** - Project overview, build system, feature system, critical rules
- **src/cli/mod.rs** - CLI architecture (validation + routing)
- **src/builder.rs** - CliBuilder API documentation
- **src/router.rs** - CommandRouter for dispatching
- **src/error.rs** - Error types and helpers
- **tests/common/mod.rs** - Built-in test utilities

## ✅ Verification Checklist

Before considering this skill complete, verify:

- [ ] Can run all examples from CLI_TESTING_EXAMPLES.rs
- [ ] Can create a new test file using fixtures from CLI_TESTING_FIXTURES.md
- [ ] Can complete a checklist from CLI_TESTING_CHECKLIST.md
- [ ] Understand the AAA pattern from CLI_TESTING_GUIDE.md
- [ ] Can navigate to relevant sections using this index

## 📞 Tips for Using This Documentation

1. **Use the table of contents** in each document to find sections
2. **Search by keyword** (Ctrl+F) within documents
3. **Follow cross-references** between documents
4. **Start with examples** then read the reference
5. **Use checklist** as a final verification step

---

**Skill Completeness:** ✓ 5 comprehensive documents  
**Code Examples:** ✓ 12 complete, runnable patterns  
**Test Coverage:** ✓ All major CLI testing scenarios  
**Feature Support:** ✓ Feature-gated testing included  
**Production Ready:** ✓ Aligned with clap-noun-verb v26.6.1
