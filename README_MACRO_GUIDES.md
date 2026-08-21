# Macro Development Guides for clap-noun-verb-macros

**Complete reference documentation for developers working on procedural macros**

## Overview

Three comprehensive documents have been created to guide macro development in the clap-noun-verb project. These guides provide everything you need to understand, debug, test, and optimize procedural macros.

## Documents

### 1. **MACRO_DEVELOPMENT_GUIDE.md** (Primary Reference)
**27 KB | 990 lines | Complete skill guide**

Start here for in-depth learning and detailed implementation guidance.

**Sections:**
- Macro Architecture Overview
- Core Proc-Macro Patterns (5 patterns with code)
- Compile-Time Validation (Poka-Yoke: 4 gaps + 2 guards)
- Common Macro Debugging Techniques (5 techniques)
- Validation Pattern Library (reusable patterns)
- Testing Strategies (6 approaches, AAA pattern)
- Performance Optimization (5 patterns, SLOs)
- Real-World Examples (2 detailed scenarios)
- Troubleshooting (10 problems + solutions)

**Use this when:**
- Learning macro architecture in depth
- Implementing new macro features
- Debugging complex macro issues
- Writing comprehensive tests
- Optimizing macro performance

### 2. **MACRO_DEVELOPER_INDEX.md** (Quick Reference)
**7.5 KB | 230 lines | Navigation and lookup guide**

Quick navigation for developers who know what they're looking for.

**Contents:**
- Quick navigation by role (6 paths for different developers)
- Essential patterns table (9 core patterns with line refs)
- Command reference (build, test, debug commands)
- Core concepts (Poka-Yoke, FM-1.1, FM-1.2, Linkme)
- Error message quality guidelines
- Common tasks (3 step-by-step processes)
- Testing guidelines (AAA pattern)
- Performance targets (SLO: 0.66s)
- Troubleshooting flowchart
- File map of macro crate

**Use this when:**
- You need a quick lookup
- Finding a specific pattern
- Need command reference
- Troubleshooting an issue
- Looking for line numbers

### 3. **MACRO_GUIDE_SUMMARY.txt** (Overview)
**9.8 KB | 350 lines | Orientation and statistics**

Executive summary and orientation document.

**Contents:**
- Content highlights overview
- Key statistics (coverage, metrics)
- Quick start paths by developer role
- Key file references with line numbers
- Practical commands reference
- Usage recommendations
- Maintenance notes

**Use this when:**
- Getting oriented to the guides
- Understanding what's covered
- Looking for quick start path
- Viewing project statistics

## Quick Start Paths

### New to the macro crate?
```
1. Read: MACRO_DEVELOPMENT_GUIDE.md
   → Macro Architecture Overview section
   
2. Study: Core Proc-Macro Patterns (5 patterns)
   
3. Explore: Real-World Examples section
   
4. Reference: MACRO_DEVELOPER_INDEX.md for lookups
```

### Debugging a macro issue?
```
1. See: Common Macro Debugging Techniques
   (MACRO_DEVELOPMENT_GUIDE.md)
   
2. Run: cargo expand to see generated code
   
3. Check: Validation Pattern Library
   
4. Search: Troubleshooting section
```

### Adding validation?
```
1. Read: Compile-Time Validation (Poka-Yoke)
   (MACRO_DEVELOPMENT_GUIDE.md)
   
2. Study: Validation Pattern Library
   
3. Write: Unit test first (test-driven)
   
4. Implement: Validation function
   
5. Integrate: Add to macro pipeline (lib.rs:346-348)
```

### Writing tests?
```
1. Reference: Testing Strategies
   (MACRO_DEVELOPMENT_GUIDE.md)
   
2. Follow: AAA pattern (Arrange-Act-Assert)
   
3. Look at: tests/macros/federated_network_test.rs
   
4. Ensure: Error message validation included
```

### Optimizing performance?
```
1. Check: Performance Optimization section
   (MACRO_DEVELOPMENT_GUIDE.md)
   
2. Profile: cargo build --message-format=short
   
3. Apply: Relevant optimization pattern
   
4. Verify: SLO met (target: 0.66s, max: 2s)
```

## What's Covered

### Core Content
- **23 patterns** from basic to advanced
- **6 validation approaches** with examples
- **5 debugging techniques** with step-by-step instructions
- **6 testing strategies** with code examples
- **5 performance optimizations** with SLOs
- **50+ code examples** from actual codebase
- **200+ line number references** for easy lookup

### Key Topics
- Proc-macro fundamentals (syn 2.0, quote!, token streams)
- Compile-time validation (Poka-Yoke error-proofing)
- Code generation with quote! macro
- Type inspection and validation
- Error handling with meaningful messages
- Testing patterns (unit, integration, performance)
- Debugging techniques (cargo expand, spans, token inspection)
- Performance optimization and profiling

### Real Codebase References
- **lib.rs**: 2,800+ lines (all macro implementations)
- **validation.rs**: 908 lines (Poka-Yoke checks)
- **io_detection.rs**: 215 lines (type detection)
- **tests/macros/**: Integration test examples
- **CLAUDE.md**: Project guidelines and SLOs

## Essential Commands

### Development
```bash
cargo make build              # Build macro crate
cargo make test               # Run all tests
cargo make clippy             # Lint with clippy
cargo make format-check       # Check formatting
```

### Debugging
```bash
cargo expand --test integration_test | head -100
cargo check 2>&1 | grep -A 5 "error"
RUST_LOG=debug cargo build 2>&1
cargo build --message-format=short 2>&1 | grep macro
```

## File Locations

All documents are in the project root:
```
/home/user/clap-noun-verb/

├── MACRO_DEVELOPMENT_GUIDE.md     (27 KB) - Comprehensive guide
├── MACRO_DEVELOPER_INDEX.md       (7.5 KB) - Quick reference
├── MACRO_GUIDE_SUMMARY.txt        (9.8 KB) - Overview
└── README_MACRO_GUIDES.md         (this file)
```

## Getting Started

1. **First time?** Start with section 1 of MACRO_DEVELOPMENT_GUIDE.md (Macro Architecture Overview)

2. **Need quick answer?** Jump to MACRO_DEVELOPER_INDEX.md

3. **Getting oriented?** Read MACRO_GUIDE_SUMMARY.txt first

4. **Stuck on problem?** Check Troubleshooting section in main guide

## Key Concepts

### Poka-Yoke (Mistake-Proofing)
Compile-time error detection prevents mistakes before runtime:
- Gap 1: Forgotten `#[verb]` detection
- Gap 2: Duplicate verb registration
- Gap 3: Return type validation
- Gap 4: Attribute syntax validation

### FM-1.1: CLI Layer Guard
Prevents business logic from leaking into `#[verb]` functions (max complexity: 5)

### FM-1.2: CLI Type Guard
Prevents domain functions from depending on CLI types (ArgMatches, Command, etc.)

### Linkme Integration
Distributed slice registration for compile-time verb discovery with zero-runtime overhead

## Performance Targets (SLOs)

From CLAUDE.md:
- Incremental compilation: ≤ 2 seconds
- Current performance: 0.66 seconds
- Macro parsing: < 10 milliseconds per invocation

## Version Info

- clap-noun-verb: 26.9.1
- clap-noun-verb-macros: 26.9.1
- Rust edition: 2021
- syn: 2.0
- quote: 1.0
- proc-macro2: 1.0

## Document Statistics

| Metric | Count |
|--------|-------|
| Total documentation | 44.3 KB |
| Total lines | 1,570+ |
| Code examples | 50+ |
| Patterns documented | 23 |
| Line references | 200+ |
| Test strategies | 6 |
| Debugging techniques | 5 |
| Optimization patterns | 5 |
| Troubleshooting scenarios | 10 |

## Next Steps

1. Choose a document above based on your needs
2. Follow the quick start path for your situation
3. Reference the specific section you need
4. Use the line numbers to jump to actual code
5. Apply the patterns in your implementation

## Questions?

Refer to the specific document sections:
- **Architecture questions?** → Macro Architecture Overview
- **Implementation questions?** → Core Proc-Macro Patterns
- **Validation questions?** → Compile-Time Validation section
- **Debugging questions?** → Common Macro Debugging Techniques
- **Testing questions?** → Testing Strategies
- **Performance questions?** → Performance Optimization
- **Stuck?** → Troubleshooting section

---

**These guides are comprehensive, current (2026-06-14), and ready for immediate use.**
