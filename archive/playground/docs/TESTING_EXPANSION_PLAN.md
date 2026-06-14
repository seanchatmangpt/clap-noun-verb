# Testing Expansion Implementation Plan

**Phase**: Phase 3 (after critical fixes and code quality)
**Duration**: 3-4 days
**Target**: 85% coverage with 170 tests (currently 107 tests, 55-60% coverage)
**New Tests Needed**: 63 tests

---

## Test Pyramid Target

```
                    /\
                   /E2E\         5 tests (3%)
                  /------\       End-to-end user workflows
                 /        \
                /Integration\    20 tests (12%)
               /------------\   Component interactions
              /              \
            /   Unit Tests    \  125 tests (85%)
           /------------------\ Isolated behavior
          /____________________\

Current Distribution:  38 unit + 30 integration + 0 E2E = 68 tests
Target Distribution:   125 unit + 20 integration + 5 E2E = 150 tests
Additional Needed:     87 new tests across all categories
```

---

## Test Coverage by Category

### Category 1: Unit Tests - Happy Paths (Add 20 tests)

**Current**: 50 happy path tests
**Target**: 70 happy path tests

#### Papers Module (Add 8 tests)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_creation_imrad_family() {
        let paper = Paper::new(
            "Test Paper",
            "John Doe",
            "A test paper",
            PaperFamily::Imrad,
        );
        assert_eq!(paper.title, "Test Paper");
        assert_eq!(paper.family, PaperFamily::Imrad);
    }

    #[test]
    fn test_paper_creation_all_families() {
        // Test creation for all 7 families
        let families = vec![
            PaperFamily::Imrad,
            PaperFamily::Argument,
            PaperFamily::Contribution,
            PaperFamily::Monograph,
            PaperFamily::Dsr,
            PaperFamily::Narrative,
            PaperFamily::Papers,
        ];

        for family in families {
            let paper = Paper::new("Test", "Author", "Abstract", family);
            assert_eq!(paper.family, family);
        }
    }

    #[test]
    fn test_paper_with_empty_abstract_allowed() {
        let paper = Paper::new("Test", "Author", "", PaperFamily::Imrad);
        assert_eq!(paper.abstract_text, "");
    }

    #[test]
    fn test_paper_with_special_characters_in_title() {
        let special_title = "Test: Theory & Practice (2024)";
        let paper = Paper::new(special_title, "Author", "Abstract", PaperFamily::Imrad);
        assert_eq!(paper.title, special_title);
    }

    #[test]
    fn test_paper_with_very_long_title() {
        let long_title = "A".repeat(500);
        let paper = Paper::new(&long_title, "Author", "Abstract", PaperFamily::Imrad);
        assert_eq!(paper.title.len(), 500);
    }

    #[test]
    fn test_paper_with_unicode_content() {
        let paper = Paper::new(
            "论文标题 (Chinese)",
            "作者 (Author)",
            "Résumé français",
            PaperFamily::Imrad,
        );
        assert!(paper.title.contains("论文"));
    }

    #[test]
    fn test_multiple_papers_independence() {
        let paper1 = Paper::new("Paper 1", "Author 1", "Abstract 1", PaperFamily::Imrad);
        let paper2 = Paper::new("Paper 2", "Author 2", "Abstract 2", PaperFamily::Argument);

        assert_ne!(paper1.title, paper2.title);
        assert_ne!(paper1.family, paper2.family);
    }

    #[test]
    fn test_paper_family_display() {
        let paper = Paper::new("Test", "Author", "Abstract", PaperFamily::Imrad);
        assert_eq!(format!("{:?}", paper.family), "Imrad");
    }
}
```

#### Thesis Module (Add 6 tests)
```rust
#[test]
fn test_thesis_family_variants() {
    for i in 0..26 {
        let family = ThesisFamily::from_index(i);
        assert!(family.is_some());
    }
}

#[test]
fn test_thesis_structure_creation() {
    let structure = ThesisStructure::new("Intro", "Method", "Results", "Discussion");
    assert_eq!(structure.introduction, "Intro");
}

#[test]
fn test_thesis_schedule_default_values() {
    let schedule = ThesisSchedule::default();
    assert!(schedule.start_date.is_some());
}

#[test]
fn test_thesis_schedule_duration_calculation() {
    let schedule = ThesisSchedule {
        start_date: Some(Date::new(2024, 1, 1)),
        end_date: Some(Date::new(2024, 12, 31)),
    };
    assert_eq!(schedule.duration_days(), 365);
}

#[test]
fn test_thesis_with_all_family_types() {
    let families = ThesisFamily::all();
    assert_eq!(families.len(), 26);
}

#[test]
fn test_thesis_schedule_with_past_dates() {
    let schedule = ThesisSchedule {
        start_date: Some(Date::new(2020, 1, 1)),
        end_date: Some(Date::new(2020, 12, 31)),
    };
    assert!(schedule.is_completed());
}
```

#### Format Output (Add 6 tests)
```rust
#[test]
fn test_format_json_single_paper() {
    let paper = Paper::test_fixture();
    let json = format_json(&paper).unwrap();
    assert!(json.contains("\"title\""));
    assert!(!json.contains("\n"));  // Compact by default
}

#[test]
fn test_format_json_multiple_papers() {
    let papers = vec![Paper::test_fixture(), Paper::test_fixture_different()];
    let json = format_json(&papers).unwrap();
    assert!(json.contains("title"));
}

#[test]
fn test_format_yaml() {
    let paper = Paper::test_fixture();
    let yaml = format_yaml(&paper).unwrap();
    assert!(yaml.contains("title:"));
}

#[test]
fn test_format_table() {
    let papers = vec![Paper::test_fixture(), Paper::test_fixture_different()];
    let table = format_table(&papers).unwrap();
    assert!(table.contains("│"));  // Table borders
}

#[test]
fn test_format_plain_text() {
    let paper = Paper::test_fixture();
    let plain = format_plain(&paper).unwrap();
    assert!(!plain.contains("{"));  // No JSON
}

#[test]
fn test_format_json_pretty() {
    let paper = Paper::test_fixture();
    let json = format_json_pretty(&paper).unwrap();
    assert!(json.contains("\n"));  // Pretty includes newlines
}
```

---

### Category 2: Unit Tests - Error Paths (Add 20 tests) - CRITICAL

**Current**: 2 error path tests
**Target**: 22 error path tests

#### Invalid Inputs (Add 10 tests)
```rust
#[test]
fn test_paper_family_validation_rejects_invalid() {
    let result = PaperFamily::from_str("InvalidFamily");
    assert!(result.is_err());
}

#[test]
fn test_paper_creation_rejects_empty_title() {
    let result = Paper::validate_title("");
    assert!(result.is_err());
}

#[test]
fn test_paper_creation_rejects_null_author() {
    let result = Paper::new("Title", "", "Abstract", PaperFamily::Imrad);
    assert!(result.is_err());
}

#[test]
fn test_papers_add_duplicate_prevention() {
    // Create paper, try to create same paper again
    let registry = InMemoryRegistry::new();
    registry.add_paper(paper1.clone()).unwrap();

    let result = registry.add_paper(paper1.clone());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("duplicate"));
}

#[test]
fn test_papers_export_missing_file_error() {
    let result = export_paper_from_missing_file("nonexistent.md");
    assert!(result.is_err());
}

#[test]
fn test_papers_export_invalid_format_error() {
    let result = export_with_format("paper", "invalid_format");
    assert!(result.is_err());
}

#[test]
fn test_sparql_query_syntax_error() {
    let store = create_test_store();
    let result = store.query("SELECT * INVALID SYNTAX");
    assert!(result.is_err());
}

#[test]
fn test_template_with_missing_variables() {
    let mut context = tera::Context::new();
    // Don't add required "title" variable
    let result = render_template("paper.tex", &context);
    assert!(result.is_err());
}

#[test]
fn test_json_format_invalid_utf8() {
    // Simulate invalid UTF-8 in data
    let result = format_json_strict(&invalid_utf8_data);
    assert!(result.is_err());
}

#[test]
fn test_config_invalid_toml_syntax() {
    let result = parse_config("invalid = = toml");
    assert!(result.is_err());
}
```

#### Resource Limits (Add 5 tests)
```rust
#[test]
fn test_papers_with_max_entries_limit() {
    let registry = InMemoryRegistry::with_max_size(100);

    for i in 0..100 {
        registry.add_paper(create_test_paper(i)).unwrap();
    }

    let result = registry.add_paper(create_test_paper(100));
    assert!(result.is_err());
}

#[test]
fn test_sparql_query_with_timeout() {
    let store = create_test_store();

    let result = execute_sparql_with_timeout(
        &store,
        "SELECT * WHERE { ?x ?y ?z }",
        1,  // 1 second timeout
    );

    assert!(result.is_err());
}

#[test]
fn test_memory_usage_with_large_ontology() {
    let store = load_large_ontology();
    let memory = get_memory_usage();
    assert!(memory < 200_000_000);  // Less than 200MB
}

#[test]
fn test_template_render_with_huge_paper() {
    let huge_paper = create_paper_with_sections(10000);
    let result = render_paper_latex(&huge_paper);
    // Should succeed but be slow
    assert!(result.is_ok());
}

#[test]
fn test_concurrent_operations_limit() {
    let store = Arc::new(create_test_store());
    let mut handles = vec![];

    for _ in 0..100 {
        let store_clone = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            execute_sparql(&store_clone, "SELECT COUNT(*) WHERE { ?x ?y ?z }")
        }));
    }

    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }
}
```

#### Error Recovery (Add 5 tests)
```rust
#[test]
fn test_papers_add_recovers_after_io_error() {
    let registry = Registry::new();

    // Simulate I/O error
    mock_io_error();
    let result = registry.add_paper(paper);
    assert!(result.is_err());

    // Clear error state
    clear_mock_io_error();
    let result = registry.add_paper(paper);
    assert!(result.is_ok());
}

#[test]
fn test_sparql_connection_retry() {
    let store = create_test_store();

    // First query fails
    disable_sparql();
    let result = store.query("SELECT * WHERE { ?x ?y ?z }");
    assert!(result.is_err());

    // Re-enable and retry
    enable_sparql();
    let result = store.query("SELECT * WHERE { ?x ?y ?z }");
    assert!(result.is_ok());
}

#[test]
fn test_template_render_fallback() {
    // Try to render with missing template
    let result = render_template("missing.tex", &context);
    assert!(result.is_err());

    // Use fallback template
    let result = render_template_with_fallback("missing.tex", "default.tex", &context);
    assert!(result.is_ok());
}

#[test]
fn test_config_load_with_corrupted_file() {
    let config = Config::load("corrupted.toml");
    assert!(config.is_err());

    // Load with defaults
    let config = Config::load_with_defaults("corrupted.toml");
    assert!(config.is_ok());
}

#[test]
fn test_cli_graceful_shutdown_on_signal() {
    let handle = run_cli_in_thread();

    thread::sleep(Duration::from_millis(100));
    send_signal(SIGTERM);

    let result = handle.join();
    assert!(result.is_ok());
}
```

---

### Category 3: Property-Based Tests (Add 5 tests)

```rust
use proptest::prelude::*;

#[test]
fn prop_paper_family_roundtrip_is_lossless() {
    proptest!(|(family_str in "[a-z]+") {
        if let Ok(family) = PaperFamily::from_str(&family_str) {
            let repr = format!("{:?}", family).to_lowercase();
            let recovered = PaperFamily::from_str(&repr);
            // Should recover family if valid
            if PaperFamily::is_valid_name(&family_str) {
                assert_eq!(family, recovered.unwrap());
            }
        }
    });
}

#[test]
fn prop_output_format_is_deterministic() {
    proptest!(|(paper in paper_strategy()) {
        let output1 = format_json(&paper).unwrap();
        let output2 = format_json(&paper).unwrap();
        assert_eq!(output1, output2);
    });
}

#[test]
fn prop_ontology_triple_count_is_stable() {
    proptest!(|(_seed in 0u64..1000u64)| {
        let triples = generate_ontology_triples();
        assert!(triples.len() > 100);
        assert!(triples.len() < 500);
    });
}

#[test]
fn prop_shell_completions_always_non_empty() {
    proptest!(|(shell in "[a-z]+") {
        if let Ok(shell_type) = ShellType::from_str(&shell) {
            let completions = generate_completions(shell_type);
            assert!(!completions.is_empty());
        }
    });
}

#[test]
fn prop_config_immutability_between_instances() {
    proptest!(|(config1 in config_strategy(), config2 in config_strategy()) {
        let mut c1 = config1.clone();
        let mut c2 = config2.clone();

        c1.modify_something();

        // c2 should not be affected
        assert_eq!(c2.get_original_value(), config2.get_original_value());
    });
}
```

---

### Category 4: Integration Tests (Consolidate to 20 focused tests)

**Current**: 30 integration tests (some redundant)
**Target**: 20 focused tests

#### Critical User Workflows (Add 5 new tests)
```rust
#[test]
fn test_workflow_create_paper_export_to_pdf() {
    // Create CLI instance
    let cli = CliRunner::new();

    // 1. Create paper
    cli.run("papers add My-Paper imrad").assert_success();

    // 2. List papers
    cli.run("papers list").assert_success()
        .assert_output_contains("My-Paper");

    // 3. Export to PDF
    cli.run("papers export My-Paper.pdf").assert_success();

    // 4. Verify file exists
    assert!(Path::new("My-Paper.pdf").exists());
}

#[test]
fn test_workflow_schedule_thesis_validate_export() {
    let cli = CliRunner::new();

    // 1. Schedule thesis
    cli.run("thesis schedule MyThesis").assert_success();

    // 2. Check schedule
    cli.run("thesis list").assert_success()
        .assert_output_contains("MyThesis");

    // 3. Validate
    cli.run("thesis check MyThesis").assert_success();

    // 4. Export schedule
    cli.run("thesis export MyThesis.csv").assert_success();
}

#[test]
fn test_workflow_configure_profiles_use_in_papers() {
    let cli = CliRunner::new();

    // 1. Create config profile
    cli.run("config set profile:research family=imrad").assert_success();

    // 2. Create paper with profile
    cli.run("papers add MyPaper --profile research").assert_success();

    // 3. Verify paper used profile settings
    cli.run("papers info MyPaper").assert_success()
        .assert_output_contains("imrad");
}

#[test]
fn test_workflow_query_ontology_get_results() {
    let cli = CliRunner::new();

    // 1. Run SPARQL query
    let results = cli.run("sparql SELECT ?x WHERE { ?x rdf:type Paper }")
        .assert_success()
        .get_json_output();

    // 2. Verify results structure
    assert!(results.is_array());
    assert!(!results.as_array().unwrap().is_empty());
}

#[test]
fn test_workflow_introspect_matches_actual_commands() {
    let cli = CliRunner::new();

    // 1. Get introspection
    let introspect = cli.run("meta introspect")
        .assert_success()
        .get_json_output();

    // 2. Get help
    let help = cli.run("--help")
        .assert_success()
        .get_output();

    // 3. Verify introspection matches help
    for command in introspect["commands"].as_array().unwrap() {
        let cmd_name = command["name"].as_str().unwrap();
        assert!(help.contains(cmd_name));
    }
}
```

---

### Category 5: Performance Tests (Add 5 new benchmark tests)

```rust
#[bench]
fn bench_cli_startup_cold(b: &mut Bencher) {
    b.iter(|| {
        std::process::Command::new("htf")
            .arg("--help")
            .output()
    });
    // Assert: <100ms (SLO)
}

#[bench]
fn bench_template_render_single_paper(b: &mut Bencher) {
    let paper = Paper::test_fixture();
    b.iter(|| render_paper_latex(&paper));
    // Assert: <20ms (SLO)
}

#[bench]
fn bench_sparql_simple_query(b: &mut Bencher) {
    let store = create_test_store();
    let query = "SELECT COUNT(*) WHERE { ?x rdf:type ?y }";
    b.iter(|| store.query(query));
    // Assert: <50ms (SLO)
}

#[bench]
fn bench_paper_export_json_100_papers(b: &mut Bencher) {
    let papers = create_test_papers(100);
    b.iter(|| format_json(&papers));
    // Assert: <100ms (SLO)
}

#[bench]
fn bench_memory_usage_large_ontology(b: &mut Bencher) {
    b.iter(|| {
        let _store = load_large_ontology();
        black_box(())
    });
    // Assert: <200MB peak (SLO)
}
```

---

## Test Implementation Schedule

### Day 1: Unit Tests - Happy Paths & Error Paths (16 hours)
- Morning (4h): Papers module tests (8 tests)
- Midday (4h): Thesis module & format tests (12 tests)
- Afternoon (4h): Error paths tests (10 tests)
- Evening (4h): Resource limits & error recovery (10 tests)

### Day 2: Property-Based Tests & Integration (16 hours)
- Morning (4h): Property-based tests (5 tests)
- Midday (4h): Consolidate existing integration tests
- Afternoon (4h): Add critical workflow tests (5 tests)
- Evening (4h): Verify all tests pass, fix failures

### Day 3: Performance Tests & Coverage (8 hours)
- Morning (4h): Create benchmark suite (5 tests)
- Afternoon (4h): Run test coverage analysis, identify gaps
- Evening: Fix any failing tests

### Day 4: Final Validation & Documentation (8 hours)
- Morning (4h): Final test runs, coverage verification
- Afternoon (4h): Update test documentation, CI/CD configuration

---

## Coverage Target

```
Total tests: 107 → 170 (63 new tests)

Coverage breakdown:
├── Unit tests: 38 → 125 tests
│   ├── Happy paths: 50 → 70
│   ├── Error paths: 2 → 22
│   ├── Edge cases: 15 → 20
│   └── Thread safety: 8 → 13
├── Integration tests: 30 → 20 tests (consolidate)
├── E2E tests: 0 → 5 tests
├── Property tests: 0 → 5 tests
└── Performance tests: 0 → 5 tests

Coverage %: 55-60% → 85%
```

---

## Implementation Checklist

- [ ] **Unit Test Suite** (40 tests)
  - [ ] Papers module happy paths (8 tests)
  - [ ] Thesis module happy paths (6 tests)
  - [ ] Format output happy paths (6 tests)
  - [ ] Invalid inputs error paths (10 tests)
  - [ ] Resource limits (5 tests)
  - [ ] Error recovery (5 tests)

- [ ] **Property-Based Tests** (5 tests)
  - [ ] Paper family roundtrip (1 test)
  - [ ] Format determinism (1 test)
  - [ ] Ontology stability (1 test)
  - [ ] Shell completions (1 test)
  - [ ] Config immutability (1 test)

- [ ] **Integration Tests** (5 new)
  - [ ] Paper create → export workflow
  - [ ] Thesis schedule → export workflow
  - [ ] Config profile → usage workflow
  - [ ] Ontology query workflow
  - [ ] Introspect match workflow

- [ ] **Performance Tests** (5 tests)
  - [ ] CLI startup benchmark
  - [ ] Template render benchmark
  - [ ] SPARQL query benchmark
  - [ ] JSON format benchmark
  - [ ] Memory usage benchmark

- [ ] **Verification**
  - [ ] All 170+ tests pass
  - [ ] Coverage ≥85%
  - [ ] No flaky tests
  - [ ] Performance benchmarks documented

---

## Success Criteria

- [ ] 170 total tests (63 new tests added)
- [ ] 85% code coverage
- [ ] All tests pass consistently (no flaky tests)
- [ ] Performance benchmarks meet SLOs
- [ ] Error paths thoroughly tested
- [ ] Property-based tests validate critical properties
- [ ] Integration tests cover user workflows
- [ ] Test documentation complete

---

**Status**: Implementation guide complete
**Next**: Begin implementation on Day 1
**Expected Duration**: 4 days

