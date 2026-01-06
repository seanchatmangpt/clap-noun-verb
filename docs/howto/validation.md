# How-to: Validate Ontologies

**Problem**: You need to ensure your RDF ontology is correct before code generation

**Solution**: Use comprehensive validation patterns to catch ontology errors early

## Validation Strategy

Ontology validation happens at multiple levels:

1. **Syntax Validation** - Is the Turtle file valid?
2. **Semantic Validation** - Are the RDF semantics correct?
3. **Domain Validation** - Do nouns and verbs follow CLI conventions?
4. **Consistency Validation** - Are references valid and unambiguous?

## Step 1: Syntax Validation

```rust
use clap_noun_verb::rdf::turtle_parser::TurtleParser;
use std::fs;

pub fn validate_syntax(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();

    match parser.parse(&content) {
        Ok(_) => {
            println!("✅ Syntax valid");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Syntax error: {}", e);
            Err(e.into())
        }
    }
}
```

**Test it**:
```bash
cargo test validate_syntax --lib
```

## Step 2: Semantic Validation

```rust
pub fn validate_semantics(turtle_file: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;

    let mut warnings = Vec::new();

    // Validate ontology
    if let Err(e) = ontology.validate_ontology() {
        warnings.push(format!("Semantic error: {}", e));
    }

    if warnings.is_empty() {
        println!("✅ Semantics valid");
    } else {
        for warning in &warnings {
            eprintln!("⚠️  {}", warning);
        }
    }

    Ok(warnings)
}
```

## Step 3: Domain Validation

Ensure CLI-specific requirements:

```rust
use clap_noun_verb::rdf::sparql_executor::SparqlExecutor;

pub fn validate_cli_domain(turtle_file: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;

    let executor = SparqlExecutor::new(&ontology)?;
    let mut errors = Vec::new();

    // Check 1: All verbs must have nouns
    let orphaned_verbs = executor.execute_query(
        "SELECT ?verb WHERE { ?verb a cnv:Verb . FILTER NOT EXISTS { ?verb cnv:hasNoun ?n } }"
    )?;

    if !orphaned_verbs.is_empty() {
        errors.push(format!("❌ Found {} verbs without nouns", orphaned_verbs.len()));
    }

    // Check 2: All noun references must exist
    let invalid_refs = executor.execute_query(
        "SELECT ?verb ?noun WHERE { ?verb cnv:hasNoun ?noun . FILTER NOT EXISTS { ?noun a cnv:Noun } }"
    )?;

    if !invalid_refs.is_empty() {
        errors.push(format!("❌ Found {} verbs with invalid noun references", invalid_refs.len()));
    }

    // Check 3: Names must be valid Rust identifiers
    let invalid_names = executor.execute_query(
        "SELECT ?resource ?name WHERE { ?resource cnv:name ?name . FILTER (CONTAINS(?name, \" \")) }"
    )?;

    if !invalid_names.is_empty() {
        errors.push(format!("❌ Found {} names with spaces (invalid Rust identifiers)", invalid_names.len()));
    }

    // Check 4: Nouns must have at least one verb
    let orphaned_nouns = executor.execute_query(
        "SELECT ?noun ?name WHERE { ?noun a cnv:Noun ; cnv:name ?name . FILTER NOT EXISTS { ?v cnv:hasNoun ?noun } }"
    )?;

    if !orphaned_nouns.is_empty() {
        eprintln!("⚠️  Found {} nouns without verbs", orphaned_nouns.len());
    }

    if errors.is_empty() && orphaned_nouns.is_empty() {
        println!("✅ CLI domain valid");
    }

    for error in &errors {
        eprintln!("{}", error);
    }

    Ok(errors)
}
```

## Step 4: Consistency Validation

```rust
pub fn validate_consistency(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;
    let executor = SparqlExecutor::new(&ontology)?;

    // Check for duplicate verb names within a noun
    let duplicates = executor.execute_query(
        r#"SELECT ?noun ?name (COUNT(?verb) as ?count) WHERE {
            ?noun a cnv:Noun .
            ?verb1 cnv:hasNoun ?noun ; cnv:name ?name .
            ?verb2 cnv:hasNoun ?noun ; cnv:name ?name .
            FILTER (?verb1 != ?verb2)
        } GROUP BY ?noun ?name HAVING (?count > 1)"#
    )?;

    if !duplicates.is_empty() {
        eprintln!("❌ Found duplicate verb names within nouns");
        return Err("Consistency validation failed".into());
    }

    println!("✅ Consistency valid");
    Ok(())
}
```

## Step 5: Performance Validation

Ensure ontology generation performance:

```rust
use std::time::Instant;

pub fn validate_performance(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(turtle_file)?;
    let parser = TurtleParser::new();

    // Measure parse time
    let start = Instant::now();
    let ontology = parser.parse(&content)?;
    let parse_time = start.elapsed();

    if parse_time.as_millis() > 50 {
        eprintln!("⚠️  Parse time {} ms exceeds SLO of 50ms", parse_time.as_millis());
    }

    // Measure validation time
    let start = Instant::now();
    ontology.validate_ontology()?;
    let validation_time = start.elapsed();

    if validation_time.as_millis() > 10 {
        eprintln!("⚠️  Validation time {} ms exceeds SLO of 10ms", validation_time.as_millis());
    }

    // Measure SPARQL execution time
    let executor = SparqlExecutor::new(&ontology)?;
    let start = Instant::now();
    executor.execute_query("SELECT ?verb WHERE { ?verb a cnv:Verb }")?;
    let query_time = start.elapsed();

    if query_time.as_millis() > 5 {
        eprintln!("⚠️  Query time {} ms exceeds SLO of 5ms", query_time.as_millis());
    }

    println!("✅ Performance acceptable");
    println!("  Parse:      {} ms (SLO: 50ms)", parse_time.as_millis());
    println!("  Validate:   {} ms (SLO: 10ms)", validation_time.as_millis());
    println!("  Query:      {} ms (SLO: 5ms)", query_time.as_millis());

    Ok(())
}
```

## Step 6: Comprehensive Validation

Combine all checks:

```rust
pub async fn validate_ontology_complete(turtle_file: &str) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    println!("📋 Validating ontology: {}\n", turtle_file);

    let mut report = ValidationReport {
        file: turtle_file.to_string(),
        passed: Vec::new(),
        failed: Vec::new(),
        warnings: Vec::new(),
    };

    // Syntax validation
    match validate_syntax(turtle_file) {
        Ok(_) => report.passed.push("Syntax validation".to_string()),
        Err(e) => report.failed.push(format!("Syntax: {}", e)),
    }

    // Semantic validation
    match validate_semantics(turtle_file) {
        Ok(warns) => {
            report.passed.push("Semantic validation".to_string());
            report.warnings.extend(warns);
        }
        Err(e) => report.failed.push(format!("Semantics: {}", e)),
    }

    // Domain validation
    match validate_cli_domain(turtle_file) {
        Ok(errs) if errs.is_empty() => report.passed.push("Domain validation".to_string()),
        Ok(errs) => report.failed.extend(errs),
        Err(e) => report.failed.push(format!("Domain: {}", e)),
    }

    // Consistency validation
    match validate_consistency(turtle_file) {
        Ok(_) => report.passed.push("Consistency validation".to_string()),
        Err(e) => report.failed.push(format!("Consistency: {}", e)),
    }

    // Performance validation
    match validate_performance(turtle_file) {
        Ok(_) => report.passed.push("Performance validation".to_string()),
        Err(e) => report.failed.push(format!("Performance: {}", e)),
    }

    // Print report
    println!("\n✅ Passed ({}):", report.passed.len());
    for check in &report.passed {
        println!("  ✓ {}", check);
    }

    if !report.warnings.is_empty() {
        println!("\n⚠️  Warnings ({}):", report.warnings.len());
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }

    if !report.failed.is_empty() {
        println!("\n❌ Failed ({}):", report.failed.len());
        for failure in &report.failed {
            println!("  ✗ {}", failure);
        }
        return Err("Validation failed".into());
    }

    println!("\n🎉 All validation checks passed!");
    Ok(report)
}

pub struct ValidationReport {
    pub file: String,
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub warnings: Vec<String>,
}
```

## Usage Example

```bash
# In your build script or CI pipeline
cargo test --lib -- validate_ontology_complete
```

## Validation Checklist

Before deploying any ontology:

- ✅ Turtle syntax is valid
- ✅ RDF semantics are correct
- ✅ All verbs have nouns (no orphaned verbs)
- ✅ All noun references exist (no dangling references)
- ✅ Names are valid Rust identifiers (no spaces)
- ✅ No duplicate verb names within a noun
- ✅ Nouns have at least one verb (no orphaned nouns)
- ✅ Parse time < 50ms
- ✅ Validation time < 10ms
- ✅ Query time < 5ms

---

**Related**:
- [Tutorial 2: Create Your First RDF Ontology](../tutorials/tutorial-2-first-rdf.md)
- [How-to: Query with SPARQL](sparql-queries.md)
- [Reference: Error Codes](../reference/error-codes.md)
