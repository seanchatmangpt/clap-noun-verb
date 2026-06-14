# Playground Improvement Roadmap

**Status**: Prioritized implementation plan from 12-agent analysis
**Total Effort**: 6-8 weeks of focused work
**Expected Outcome**: Production-grade playground with 8.5→9.2 code quality, 5.8→8.5 UX score

---

## Quick Impact Matrix

```
HIGHEST VALUE (do these first):
┌─────────────────────────────────────────────────────────┐
│ Impact: HIGH | Effort: LOW | Time: <1 week              │
├─────────────────────────────────────────────────────────┤
│ 1. Critical Fixes (5-8 hours)                           │
│    - Family validation, SPARQL timeout, caching         │
│    - Impact: Stability + 30-75% perf improvement        │
├─────────────────────────────────────────────────────────┤
│ 2. Main.rs Refactoring (8 hours)                        │
│    - Extract verb handlers: 640→200 lines               │
│    - Impact: Maintainability, readability               │
├─────────────────────────────────────────────────────────┤
│ 3. Testing Expansion Tier 1 (2 days)                    │
│    - Add 30 unit tests for error paths                  │
│    - Impact: Confidence, regression prevention          │
│─────────────────────────────────────────────────────────│
│ TOTAL: 4-5 days work, MASSIVE value delivered          │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 1: Critical Fixes (5-8 hours) - WEEK 1

**Status**: Must complete before other work
**Files Changed**: 3 files
**Breaking Changes**: None
**Expected Impact**: Stability + 30-75% performance

| # | Item | Effort | Impact | Priority |
|---|------|--------|--------|----------|
| 1 | Family validation | 30 min | CRITICAL - data integrity | P0 |
| 2 | SPARQL timeout | 1-2h | CRITICAL - CLI reliability | P0 |
| 3 | Template engine cache | 45 min | HIGH - 10-15ms perf | P1 |
| 4 | RDF store cache | 1-2h | HIGH - 20-50ms perf | P1 |
| 5 | UUID generation | 30 min | HIGH - reliability | P1 |

**Detailed Guide**: See `CRITICAL_FIXES.md`

**Definition of Done**:
```bash
✓ cargo test --all passes (107 tests)
✓ cargo clippy shows no warnings
✓ htf papers add "Test" InvalidFamily → Error
✓ Complex SPARQL query times out after 5s
✓ htf papers export runs in <20ms (was 150ms)
```

---

## Phase 2: Code Quality Refactoring (3-5 days) - WEEK 1-2

**Focus**: Reduce complexity, improve maintainability
**Files Changed**: 4-5 files
**Breaking Changes**: None
**Expected Impact**: 8.5→9.2 code quality score

### 2A: Extract CLI Handlers from main.rs (8 hours)

**Current State**: main.rs is 640 lines with inlined verb handlers
**Target State**: main.rs is 200 lines, handlers in separate module

**Changes**:
```
src/
├── main.rs                      (640 lines → 200 lines)
└── cmds/
    ├── mod.rs                   (new - exports all verbs)
    ├── papers.rs                (new - papers verb handlers)
    ├── thesis.rs                (new - thesis verb handlers)
    ├── config.rs                (new - config verb handlers)
    └── meta.rs                  (new - meta verb handlers)
```

**Example Extraction**:
```rust
// BEFORE: Inlined in main.rs
pub async fn papers_export(args: ExportArgs) -> Result<()> {
    // 50+ lines of code
}

// AFTER: In src/cmds/papers.rs
pub async fn export(args: ExportArgs) -> Result<()> {
    // Same code, but now in logical module
}

// In main.rs: just import and call
use cmds::papers;

// Route automatically via clap-noun-verb
```

**Time Breakdown**:
- Extract papers handlers: 2h
- Extract thesis handlers: 2h
- Extract config handlers: 1.5h
- Extract meta handlers: 1.5h
- Update tests and imports: 1h

**Success Criteria**:
```bash
✓ main.rs <250 lines
✓ All verb handlers in separate modules
✓ All tests still pass
✓ No clippy warnings
✓ Improved readability
```

---

### 2B: Consolidate Format Output Logic (2 hours)

**Current State**: Format output logic scattered across handlers
**Target State**: Centralized OutputFormatter trait

**Changes**:
```rust
// NEW: src/domain/formatter.rs
pub trait OutputFormatter {
    fn format_papers(&self, papers: Vec<Paper>) -> String;
    fn format_thesis(&self, thesis: Vec<ThesisFamily>) -> String;
}

pub struct JsonFormatter;
pub struct TableFormatter;
pub struct PlainFormatter;
pub struct YamlFormatter;

impl OutputFormatter for JsonFormatter {
    // JSON implementation
}

impl OutputFormatter for TableFormatter {
    // Table implementation
}

// Usage:
let formatter: Box<dyn OutputFormatter> = match args.format {
    Format::Json => Box::new(JsonFormatter),
    Format::Table => Box::new(TableFormatter),
    Format::Plain => Box::new(PlainFormatter),
    Format::Yaml => Box::new(YamlFormatter),
};

println!("{}", formatter.format_papers(papers));
```

**Benefits**:
- Eliminates 12+ format branching statements
- Single place to add new formats
- Type-safe formatting
- Testable independently

---

### 2C: Implement `FromStr` Trait Properly (1 hour)

**Current State**: Custom `from_str()` method on PaperFamily
**Target State**: Proper `FromStr` trait implementation

```rust
// CORRECT: Implement std::str::FromStr
use std::str::FromStr;

impl FromStr for PaperFamily {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "imrad" => Ok(PaperFamily::Imrad),
            // ...
            _ => Err(anyhow!("Invalid family: {}", s)),
        }
    }
}

// Now works with standard library functions
let family: PaperFamily = "imrad".parse()?;
```

---

### 2D: Replace Hand-Rolled Completions (2 hours)

**Current State**: Custom shell completion generation
**Target State**: Use clap_complete crate

**Before** (200+ lines in completions.rs):
```rust
pub fn generate_bash_completions() -> String {
    let mut out = String::new();
    // Manual completion building
    out.push_str("complete -c htf -n '__fish_seen_subcommand_from ...'")
    // ... 50+ more lines
}
```

**After** (with clap_complete):
```rust
use clap_complete::{generate, Shell};

fn generate_completions(shell: Shell) -> Result<String> {
    let mut cmd = build_cli();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, "htf", &mut buf);
    Ok(String::from_utf8(buf)?)
}
```

**Benefits**:
- 200 lines → 20 lines
- Better maintained (uses clap infrastructure)
- Correct by design
- Supports more shells (Bash, Zsh, Fish, PowerShell, Elvish)

---

### 2E: Clean Up Dead Code (1 hour)

**Current State**: 17 FUTURE comments and unreachable code
**Target State**: All dead code removed or documented

**Action Items**:
- Remove unused test helpers
- Remove commented-out code
- Convert FUTURE comments to proper GitHub issues
- Document truly deferred features

---

## Phase 3: Testing Expansion (3-4 days) - WEEK 2-3

**Focus**: Increase coverage from 55-60% to 85%
**Current Tests**: 107 (passing)
**Target Tests**: 170 (85% coverage)
**New Tests Needed**: 63 tests

### Test Pyramid Target

```
                    /\
                   /E2E\       (5%) 5 tests
                  /------\     Testing critical user workflows
                 /        \
                /Integration\  (20%) 20 tests
               /------------\  Component interactions, API flows
              /              \
            /   Unit Tests    \ (75%) 125 tests
           /------------------\ Isolated behavior, error paths
          /____________________\

Current:  38 unit, 30 integration, 0 E2E = 68 tests (55-60% coverage)
Target:   125 unit, 20 integration, 5 E2E = 150 tests (85% coverage)
```

### 3A: Unit Tests - Error Paths (20 tests)

**Current**: Only 2 error path tests
**Target**: 20 error path tests covering:

```rust
// Test invalid inputs
test_papers_add_invalid_family()
test_papers_add_empty_title()
test_papers_add_duplicate()
test_papers_export_missing_file()
test_papers_export_invalid_format()

// Test error recovery
test_papers_add_recovers_after_io_error()
test_thesis_schedule_handles_invalid_dates()

// Test boundary conditions
test_papers_with_very_long_title()
test_papers_with_special_characters()
test_config_max_entries_limits()

// Test thread safety
test_concurrent_paper_access()
test_concurrent_sparql_queries()

// Test resource cleanup
test_temp_files_cleaned_up()
test_rdf_store_closed_properly()
```

**Effort**: 1 day

---

### 3B: Unit Tests - Happy Paths (15 tests)

**Current**: 80% coverage of happy paths
**Target**: 95% coverage (only stubs remain)

**Add tests for**:
- All format output modes (JSON, YAML, Table, Plain)
- All thesis families (7 frameworks)
- All paper family combinations
- Edge cases in template rendering

**Effort**: 1 day

---

### 3C: Property-Based Tests (5 tests)

**Current**: None
**Target**: 5 property tests for critical properties

```rust
// paperfamily roundtrip: string → enum → string is lossless
proptest!(|(family_str in "[a-z]+") {
    if let Ok(family) = PaperFamily::from_str(&family_str) {
        let repr = format!("{:?}", family);
        let recovered = PaperFamily::from_str(&repr);
        assert_eq!(family, recovered.unwrap());
    }
});

// output format stability: same input always produces same output
proptest!(|(paper in paper_strategy()) {
    let output1 = format_json(&paper);
    let output2 = format_json(&paper);
    assert_eq!(output1, output2);
});

// ontology triple count: matches expected cardinality
test_ontology_generates_expected_triples() {
    let triples = generate_ontology_triples();
    assert!(triples.len() > 100);
    assert!(triples.len() < 500);
}

// config immutability: changes don't leak between configs
proptest!(|(config1 in config_strategy(), config2 in config_strategy()) {
    let mut c1 = config1;
    let mut c2 = config2;
    c1.modify_something();
    assert_eq!(c2.get_original_value(), config2.get_original_value());
});

// shell completion non-empty: always generates content
test_shell_completions_non_empty() {
    for shell in &[Bash, Zsh, Fish, PowerShell, Elvish] {
        let completions = generate_completions(*shell);
        assert!(!completions.is_empty());
    }
}
```

**Effort**: 1 day

---

### 3D: Integration Tests (8 tests)

**Current**: 30 integration tests (consolidate)
**Target**: 20 focused integration tests

**Focus on critical workflows**:
```rust
test_workflow_create_paper_export_to_pdf()
test_workflow_schedule_thesis_validate_export()
test_workflow_configure_profiles_use_in_papers()
test_workflow_query_ontology_get_results()
test_workflow_introspect_matches_actual_commands()
```

**Effort**: 1 day

---

### 3E: Performance Tests (5 tests)

**Current**: None
**Target**: 5 performance tests with SLO validation

```rust
#[bench]
fn bench_cli_startup(b: &mut Bencher) {
    b.iter(|| run_command("htf --help"));
    // Assert: <100ms (SLO)
}

#[bench]
fn bench_template_render(b: &mut Bencher) {
    let paper = Paper::test_fixture();
    b.iter(|| render_paper_latex(&paper));
    // Assert: <20ms (SLO)
}

#[bench]
fn bench_sparql_query(b: &mut Bencher) {
    let store = create_test_store();
    let query = "SELECT * WHERE { ?x rdf:type ?y }";
    b.iter(|| store.query(query));
    // Assert: <50ms (SLO)
}

#[bench]
fn bench_paper_export_json(b: &mut Bencher) {
    let papers = create_test_papers(100);
    b.iter(|| format_json(&papers));
    // Assert: <100ms (SLO)
}

#[bench]
fn bench_memory_usage_with_large_ontology(b: &mut Bencher) {
    b.iter(|| {
        let store = load_large_ontology();
        black_box(store)
    });
    // Assert: <100MB (SLO)
}
```

**Effort**: 1 day

---

## Phase 4: Documentation (2-3 days) - WEEK 3

**Focus**: Increase Diataxis score from 65/100 to 90/100
**Critical Gap**: Tutorial quadrant (missing Getting Started)

### 4A: Getting Started Tutorial (4 hours) - CRITICAL

**What**: Complete beginner guide to playground
**Where**: `docs/GETTING_STARTED.md`
**Audience**: New users, learning Rust CLI patterns

**Outline**:
```markdown
# Getting Started with Playground

## Prerequisites
- Rust 1.70+ installed
- Understanding of CLI tools and RDF basics (5-min intro)

## Installation
- Clone and build
- Run: htf --help

## Your First Paper (5 minutes)
- htf papers add "My Research" imrad
- htf papers list
- Understand output

## Your First Thesis (10 minutes)
- htf thesis schedule MyThesis
- View schedule
- Modify with htf config

## Export to PDF (5 minutes)
- Create paper
- Export: htf papers export MyPaper.pdf
- View result

## Common Tasks
- Switching families: htf papers change MyPaper --family argument
- Using SPARQL: htf sparql "SELECT ?x WHERE ..."
- Shell completions: eval "$(htf --completions bash)"

## Next Steps
- See How-To Guides for specific tasks
- See Reference for complete command list
- See Explanation for design concepts
```

**Effort**: 4 hours
**Quality Target**: 5-star beginner-friendly

---

### 4B: Design Philosophy (2 hours)

**What**: Why playground exists and how it works
**Where**: `docs/DESIGN_PHILOSOPHY.md`
**Audience**: Developers, architects

**Topics**:
- Why 26 family types?
- How RDF models thesis structure
- Determinism and reproducibility
- Zero-copy template design
- Command structure rationale

---

### 4C: Troubleshooting Guide (3 hours)

**What**: Common issues and solutions
**Where**: `docs/TROUBLESHOOTING.md`
**Audience**: Users experiencing issues

**Sections**:
- "htf command not found" → Installation steps
- "Invalid family error" → Valid families list
- "SPARQL query times out" → Simplify query
- "Template rendering fails" → Check template variables
- "PDF export is blank" → Verify LaTeX installation
- "Memory usage is high" → Reduce ontology size

---

### 4D: Architectural Diagrams (4 hours)

**What**: Visual representations of system
**Where**: `docs/diagrams/`
**Diagrams**:
- Component architecture (C4 diagram)
- Data flow (Paper → Template → PDF)
- SPARQL query flow
- RDF ontology structure
- CLI command routing

---

### 4E: Examples Expansion (2 hours)

**What**: Real-world usage examples
**Where**: `examples/`
**Examples**:
- Simple paper creation
- Complete thesis workflow
- Custom SPARQL queries
- Advanced templating
- Batch operations

---

## Phase 5: Performance Optimization (2-3 days) - WEEK 4

**Focus**: Implement 5 optimizations from Agent 8
**Current Performance**:
- CLI startup: 50-100ms
- Template render: 5-15ms
- SPARQL query: 20-50ms

**Target Performance**:
- CLI startup: 30-50ms (40-50% faster)
- Template render: 1-2ms (70-80% faster)
- SPARQL query: 5-15ms (60-75% faster)

### 5A: Lazy Async Runtime (Medium)

**What**: Don't initialize tokio unless needed
**Impact**: 30-50ms startup reduction
**Effort**: 4 hours

```rust
// Only create runtime for async operations
lazy_static::lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new().unwrap();
}

// Only used if command needs async
if command.needs_async() {
    RUNTIME.block_on(execute_async_command())
}
```

---

### 5B: Persistent RDF Store (Medium)

**What**: Keep store in memory between queries
**Impact**: 20-40ms/query reduction (already done in Phase 1)
**Status**: Included in Critical Fixes

---

### 5C: Global Template Cache (Low)

**What**: Cache parsed templates (already done in Phase 1)
**Impact**: 10-15ms/render reduction
**Status**: Included in Critical Fixes

---

### 5D: Arc<str> for Strings (High)

**What**: Use Arc<str> instead of String in hot paths
**Impact**: 1-3ms/operation reduction
**Effort**: 8 hours

**Before**:
```rust
pub struct Paper {
    title: String,      // Owned
    author: String,
    abstract_text: String,
}
```

**After**:
```rust
pub struct Paper {
    title: Arc<str>,    // Shared ownership, cheap clones
    author: Arc<str>,
    abstract_text: Arc<str>,
}
```

---

### 5E: Compact JSON Default (Low)

**What**: Use compact JSON by default (no whitespace)
**Impact**: 2-5ms/serialize reduction
**Effort**: 1 hour

```rust
// Instead of serde_json::to_string_pretty
let compact = serde_json::to_string(&value)?;  // 2-5ms faster

// Or with configuration
let json = serde_json::to_string_pretty_or_compact(&value, args.pretty)?;
```

---

## Phase 6: Release Automation (1-2 days) - WEEK 4-5

**Focus**: Automated releases via GitHub Actions
**Outcome**: One-command releases with proper versioning

### 6A: GitHub Actions Workflow (4 hours)

**File**: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test --all

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: target/release/htf
          generate_release_notes: true
```

---

### 6B: Release Script (2 hours)

**File**: `scripts/release.sh`

```bash
#!/bin/bash
# Usage: ./scripts/release.sh 0.2.0

VERSION=$1
BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Validate version format
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Invalid version format. Use: x.y.z"
    exit 1
fi

# Update version in Cargo.toml files
sed -i "" "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Commit and tag
git add Cargo.toml
git commit -m "chore: bump version to $VERSION"
git tag -a "v$VERSION" -m "Release $VERSION"
git push origin $BRANCH
git push origin "v$VERSION"

echo "✓ Released v$VERSION"
```

---

### 6C: Dependabot Configuration (1 hour)

**File**: `.github/dependabot.yml`

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
    allow:
      - dependency-type: direct
      - dependency-type: indirect
```

---

## Implementation Timeline

```
Week 1:
  Mon-Tue: Phase 1 (Critical Fixes) ..................... 5-8 hours
  Wed-Thu: Phase 2A (Extract handlers) .................. 8 hours
  Fri: Phase 2B-E (Consolidation & cleanup) ............. 4 hours

Week 2:
  Mon-Tue: Phase 3A-B (Unit tests) ....................... 16 hours
  Wed-Thu: Phase 3C-D (Property & integration tests) .... 16 hours
  Fri: Phase 3E (Performance tests) & fixes .............. 8 hours

Week 3:
  Mon-Tue: Phase 4A-B (Getting Started & philosophy) ... 8 hours
  Wed-Fri: Phase 4C-E (Troubleshooting, diagrams, examples) ... 12 hours

Week 4:
  Mon-Wed: Phase 5A-E (Performance optimizations) ....... 16 hours
  Thu-Fri: Phase 6A-C (Release automation) .............. 8 hours

Total: ~125 hours (3-4 weeks at 30-40 hours/week)
```

---

## Success Metrics

### Code Quality
- [ ] Code quality: 8.5 → 9.2/10
- [ ] Complexity: Reduced, main.rs <250 lines
- [ ] Duplication: Eliminated from format output
- [ ] Type safety: Improved with proper trait impls
- [ ] Maintainability: Handler extraction improves readability

### Testing
- [ ] Test count: 107 → 170 tests
- [ ] Coverage: 55-60% → 85%
- [ ] Error paths: 2 → 20 tests
- [ ] Performance tests: 0 → 5 benchmarks
- [ ] Property tests: 0 → 5 tests

### Performance
- [ ] CLI startup: 50-100ms → 30-50ms
- [ ] Template render: 5-15ms → 1-2ms
- [ ] SPARQL query: 20-50ms → 5-15ms
- [ ] Memory peak: 80MB → 35MB
- [ ] All SLOs met

### Documentation
- [ ] Diataxis: 65 → 90/100
- [ ] Tutorial: 45 → 95/100 (CRITICAL)
- [ ] How-To: 85 → 95/100
- [ ] Reference: 75 → 90/100
- [ ] Explanation: 55 → 85/100

### UX
- [ ] UX Score: 5.8 → 8.5/10
- [ ] Family validation: Implemented
- [ ] Multi-format output: Implemented
- [ ] Shell completions: Generated correctly
- [ ] Help text: Comprehensive

---

## Dependencies to Add

```toml
# Phase 1-2
lazy_static = "1.4"
uuid = { version = "1.0", features = ["v4", "serde"] }

# Phase 2
clap_complete = "4.5"

# Phase 3
proptest = "1.4"
insta = "1.35"
criterion = { version = "0.5", features = ["html_reports"] }

# Phase 4
# (Documentation uses no new deps)

# Phase 5
# (Performance uses existing)

# Phase 6
# (Release uses GitHub Actions, no new Rust deps)
```

---

## Related Documents

- **PLAYGROUND_SYNTHESIS.md** - Complete 12-agent analysis
- **CRITICAL_FIXES.md** - Detailed fix implementations
- **TESTING_EXPANSION_PLAN.md** - Test strategy details
- **PERFORMANCE_OPTIMIZATION_GUIDE.md** - SLO & optimization details
- **DOCUMENTATION_PLAN.md** - Writing guidelines & templates
- **RELEASE_AUTOMATION_GUIDE.md** - GitHub Actions details

---

**Status**: Ready to implement
**Next Step**: Start with Phase 1 (Critical Fixes)
**Expected Completion**: 3-4 weeks at 30-40 hours/week

