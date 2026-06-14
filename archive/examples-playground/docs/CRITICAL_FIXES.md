# Critical Fixes - Implementation Guide

**Status**: MUST IMPLEMENT BEFORE PRODUCTION RELEASE
**Total Time**: 5-8 hours
**Blocking**: Yes - All critical fixes block production readiness

---

## Fix #1: Family Value Validation 🔴 CRITICAL

**Severity**: CRITICAL (Data Integrity & UX)
**File**: `src/domain/papers.rs`
**Time**: 30 minutes
**Impact**: Prevents invalid family values from being accepted

### Problem

Currently, invalid family values are silently accepted:
```bash
$ htf add "Test Paper" InvalidFamily
✓ Paper added successfully
# Should have rejected "InvalidFamily"!
```

### Current Code
```rust
// File: src/domain/papers.rs
impl FromStr for PaperFamily {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_lowercase().as_str() {
            "imrad" => PaperFamily::Imrad,
            "argument" => PaperFamily::Argument,
            "contribution" => PaperFamily::Contribution,
            "monograph" => PaperFamily::Monograph,
            "dsr" => PaperFamily::Dsr,
            "narrative" => PaperFamily::Narrative,
            "papers" => PaperFamily::Papers,
            _ => PaperFamily::Unknown,  // ← BUG: Silent fallback!
        })
    }
}
```

### Solution

Use clap's `PossibleValuesParser` for compile-time validation:

```rust
// File: src/domain/papers.rs
use clap::builder::PossibleValuesParser;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PaperFamily {
    Imrad,
    Argument,
    Contribution,
    Monograph,
    Dsr,
    Narrative,
    Papers,
}

impl PaperFamily {
    pub fn valid_values() -> &'static [&'static str] {
        &["imrad", "argument", "contribution", "monograph", "dsr", "narrative", "papers"]
    }

    pub fn from_str_validated(s: &str) -> Result<Self> {
        let valid = Self::valid_values();
        if !valid.contains(&s.to_lowercase().as_str()) {
            return Err(Error::new(format!(
                "Invalid family '{}'. Must be one of: {}",
                s,
                valid.join(", ")
            )));
        }

        Ok(match s.to_lowercase().as_str() {
            "imrad" => PaperFamily::Imrad,
            "argument" => PaperFamily::Argument,
            "contribution" => PaperFamily::Contribution,
            "monograph" => PaperFamily::Monograph,
            "dsr" => PaperFamily::Dsr,
            "narrative" => PaperFamily::Narrative,
            "papers" => PaperFamily::Papers,
            _ => unreachable!(),  // Now safe with validation
        })
    }
}

impl FromStr for PaperFamily {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_str_validated(s)
    }
}
```

### Update CLI Command

```rust
// File: src/main.rs
#[derive(Parser)]
struct AddArgs {
    title: String,
    #[arg(value_parser = PossibleValuesParser::new(PaperFamily::valid_values()))]
    family: String,
}
```

### Test Case

```rust
#[test]
fn test_family_validation_rejects_invalid() {
    let result = PaperFamily::from_str("InvalidFamily");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid family"));
}

#[test]
fn test_family_validation_accepts_valid() {
    let result = PaperFamily::from_str("imrad");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PaperFamily::Imrad);
}
```

### Verification

```bash
# After fix - should error
$ htf add "Test" InvalidFamily
Error: invalid value 'InvalidFamily' for 'family': ...

# After fix - should succeed
$ htf add "Test" imrad
✓ Paper added successfully
```

---

## Fix #2: SPARQL Timeout Not Working 🔴 CRITICAL

**Severity**: CRITICAL (Reliability/Availability)
**File**: `src/integration/rdf.rs`
**Time**: 1-2 hours
**Impact**: Prevents CLI freeze on long-running queries

### Problem

Current timeout is checked AFTER query execution, which is too late:

```rust
pub fn execute_sparql(store: &Store, query: &str) -> Result<String> {
    let start = Instant::now();
    let result = query_engine.execute(query)?;  // ← Can freeze here indefinitely

    // Too late - timeout checked after blocking call
    if start.elapsed() > Duration::from_secs(5) {
        return Err(anyhow!("Query timeout"));
    }

    Ok(result)
}
```

### Solution: Thread-Based Timeout

```rust
// File: src/integration/rdf.rs
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn execute_sparql_with_timeout(
    store: &Store,
    query: &str,
    timeout_secs: u64,
) -> Result<String> {
    let store_clone = Arc::new(store.clone());
    let query = query.to_string();
    let timeout = Duration::from_secs(timeout_secs);

    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        match execute_sparql_internal(&store_clone, &query) {
            Ok(result) => {
                let _ = tx.send(Ok(result));
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Thread continues in background, but we return timeout error
            Err(anyhow!(
                "SPARQL query timed out after {} seconds",
                timeout_secs
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(anyhow!("SPARQL query thread disconnected"))
        }
    }
}

// Internal function with no timeout (called from thread)
fn execute_sparql_internal(store: &Arc<Store>, query: &str) -> Result<String> {
    let results = store.query(query)?;

    let mut json_results = Vec::new();
    for result in results {
        let bindings = result?;
        // Format bindings as JSON...
        json_results.push(bindings);
    }

    Ok(serde_json::to_string(&json_results)?)
}
```

### Update Command Handler

```rust
#[verb]
pub async fn sparql(query: String) -> Result<()> {
    let store = create_store()?;

    // Use 5-second timeout
    match execute_sparql_with_timeout(&store, &query, 5) {
        Ok(results) => {
            println!("{}", results);
            Ok(())
        }
        Err(e) if e.to_string().contains("timeout") => {
            eprintln!("⏱️ Query timed out. Simplify query or reduce dataset.");
            Err(e)
        }
        Err(e) => Err(e),
    }
}
```

### Test Case

```rust
#[test]
fn test_sparql_timeout_returns_error() {
    let store = create_test_store();

    // Infinite loop query would hang without timeout
    let result = execute_sparql_with_timeout(
        &store,
        "SELECT * WHERE { ?x ?y ?z }", // Could match millions
        1, // 1 second timeout
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout"));
}

#[test]
fn test_sparql_completes_before_timeout() {
    let store = create_test_store();

    let result = execute_sparql_with_timeout(
        &store,
        "SELECT ?x WHERE { GRAPH ?x {} }",
        5, // 5 second timeout
    );

    assert!(result.is_ok());
}
```

### Verification

```bash
# After fix - complex query returns in <5s or errors
$ time htf sparql "SELECT * WHERE { ?x ?y ?z }" --limit 1000
⏱️ Query timed out. Simplify query or reduce dataset.

# Simple query returns immediately
$ time htf sparql "SELECT COUNT(*) WHERE { ?x rdf:type ?o }"
42
real    0m0.123s
```

---

## Fix #3: Template Engine Recreated Every Time ⚠️ HIGH

**Severity**: HIGH (Performance)
**File**: `src/integration/templates.rs`
**Time**: 45 minutes
**Impact**: 5-15ms overhead per template render removed

### Problem

```rust
pub fn render_paper_latex(paper: &Paper) -> Result<String> {
    // RECREATES ENGINE EVERY TIME - expensive!
    let mut tera = Tera::new("templates/**/*.tera")?;
    let context = create_context(paper);
    tera.render("paper.tex", &context)
}
```

This parses all templates on every call, wasting CPU cycles.

### Solution: Global Lazy-Loaded Cache

Add to `Cargo.toml`:
```toml
lazy_static = "1.4"
```

Update code:
```rust
// File: src/integration/templates.rs
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref TERA_ENGINE: Mutex<Tera> = {
        let mut tera = Tera::new("templates/**/*.tera")
            .expect("Failed to initialize Tera templates");
        Mutex::new(tera)
    };
}

pub fn render_paper_latex(paper: &Paper) -> Result<String> {
    let tera = TERA_ENGINE.lock().unwrap();
    let context = create_context(paper);
    tera.render("paper.tex", &context)
        .map_err(|e| anyhow!("Template render failed: {}", e))
}

pub fn render_template(template_name: &str, paper: &Paper) -> Result<String> {
    let tera = TERA_ENGINE.lock().unwrap();
    let context = create_context(paper);
    tera.render(template_name, &context)
        .map_err(|e| anyhow!("Template render failed: {}", e))
}

fn create_context(paper: &Paper) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("title", &paper.title);
    context.insert("author", &paper.author);
    context.insert("family", &format!("{:?}", paper.family));
    context.insert("abstract", &paper.abstract_text);
    context.insert("sections", &paper.sections);
    context
}
```

### Test Case

```rust
#[test]
fn test_template_cache_reuses_engine() {
    // First render - engine created and cached
    let paper1 = Paper::test_fixture();
    let result1 = render_paper_latex(&paper1).unwrap();
    assert!(!result1.is_empty());

    // Second render - reuses cached engine (no re-parse)
    let paper2 = Paper::test_fixture_different();
    let result2 = render_paper_latex(&paper2).unwrap();
    assert!(!result2.is_empty());
    assert_ne!(result1, result2);
}
```

### Performance Verification

```bash
# Before: ~10-15ms per render
$ time htf papers export test.pdf
real    0m0.150s

# After: ~1-2ms per render
$ time htf papers export test.pdf
real    0m0.020s
```

---

## Fix #4: RDF Store Recreated Per Query ⚠️ HIGH

**Severity**: HIGH (Performance)
**File**: `src/integration/rdf.rs`
**Time**: 1-2 hours
**Impact**: 20-50ms overhead per query removed

### Problem

```rust
pub fn execute_sparql(query: &str) -> Result<String> {
    // RECREATES STORE EVERY TIME
    let store = create_oxigraph_store()?;  // 30-50ms!
    store.query(query)
}
```

### Solution: Persistent Cached Store

```rust
// File: src/integration/rdf.rs
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref SPARQL_STORE: Mutex<Arc<oxigraph::store::Store>> = {
        Mutex::new(Arc::new(load_or_create_store()))
    };
}

fn load_or_create_store() -> oxigraph::store::Store {
    // Load from ontology file
    let store = oxigraph::store::Store::new().unwrap();

    if let Ok(ontology_data) = std::fs::read_to_string("thesis-ontology.ttl") {
        for triple in oxigraph::io::read(
            std::io::Cursor::new(ontology_data),
            oxigraph::io::GraphFormat::Turtle,
        ) {
            if let Ok(triple) = triple {
                store.insert(&triple).unwrap();
            }
        }
    }

    store
}

pub fn get_sparql_store() -> Arc<oxigraph::store::Store> {
    Arc::clone(&*SPARQL_STORE.lock().unwrap())
}

pub fn execute_sparql(query: &str) -> Result<String> {
    let store = get_sparql_store();
    let results = store.query(query)?;

    let mut json_results = Vec::new();
    for result in results {
        json_results.push(result?);
    }

    Ok(serde_json::to_string(&json_results)?)
}
```

### Test Case

```rust
#[test]
fn test_store_cache_persists() {
    let store1 = get_sparql_store();
    let store2 = get_sparql_store();

    // Should be same Arc
    assert_eq!(Arc::strong_count(&store1), Arc::strong_count(&store2));
}
```

---

## Fix #5: Predictable ID Generation ⚠️ HIGH

**Severity**: HIGH (Reliability)
**File**: `src/domain/telemetry.rs`
**Time**: 30 minutes
**Impact**: Unique, collision-free IDs

### Problem

```rust
pub fn new() -> Self {
    Self {
        id: format!("{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()),  // ← Predictable, collision risk!
    }
}
```

### Solution: UUID v4

Add to `Cargo.toml`:
```toml
uuid = { version = "1.0", features = ["v4", "serde"] }
```

Update code:
```rust
// File: src/domain/telemetry.rs
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSpan {
    pub id: String,
    pub name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
}

impl ExecutionSpan {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),  // ← UUID v4: globally unique
            name: name.into(),
            start_time: SystemTime::now(),
            end_time: None,
        }
    }

    pub fn finish(mut self) -> ExecutionReceipt {
        self.end_time = Some(SystemTime::now());
        ExecutionReceipt {
            span_id: self.id,
            duration: self.end_time.unwrap()
                .duration_since(self.start_time)
                .unwrap_or_default(),
            name: self.name,
        }
    }
}

#[test]
fn test_execution_span_ids_are_unique() {
    let span1 = ExecutionSpan::new("test");
    let span2 = ExecutionSpan::new("test");

    assert_ne!(span1.id, span2.id);

    // UUIDs are valid format
    assert!(Uuid::parse_str(&span1.id).is_ok());
    assert!(Uuid::parse_str(&span2.id).is_ok());
}
```

---

## Implementation Checklist

- [ ] **Fix #1: Family Validation** (30 min)
  - [ ] Update `PaperFamily::from_str_validated()`
  - [ ] Add clap integration with PossibleValuesParser
  - [ ] Add unit tests
  - [ ] Manual test: `htf add "Test" InvalidFamily` → Error

- [ ] **Fix #2: SPARQL Timeout** (1-2 hours)
  - [ ] Implement `execute_sparql_with_timeout()`
  - [ ] Update command handler
  - [ ] Add timeout test case
  - [ ] Manual test with complex query

- [ ] **Fix #3: Template Cache** (45 min)
  - [ ] Add lazy_static dependency
  - [ ] Create TERA_ENGINE lazy_static
  - [ ] Update render functions
  - [ ] Verify <2ms render time

- [ ] **Fix #4: RDF Store Cache** (1-2 hours)
  - [ ] Create SPARQL_STORE lazy_static
  - [ ] Implement load_or_create_store()
  - [ ] Update execute_sparql()
  - [ ] Verify <20ms query time

- [ ] **Fix #5: UUID Generation** (30 min)
  - [ ] Add uuid dependency
  - [ ] Update ExecutionSpan::new()
  - [ ] Add uniqueness test
  - [ ] Verify UUID format

- [ ] **Verification**
  - [ ] All 107 existing tests still pass
  - [ ] New unit tests pass
  - [ ] Manual testing confirms all fixes work
  - [ ] Performance improvements verified

---

## Priority Order

1. **Family Validation** - Start here (30 min, blocks data integrity)
2. **SPARQL Timeout** - Critical (1-2 hours, CLI reliability)
3. **Template Cache** - Quick win (45 min, 10-15ms improvement)
4. **RDF Store Cache** - Impact (1-2 hours, 20-50ms improvement)
5. **UUID Generation** - Polish (30 min, reliability)

**Total Time**: 5-8 hours of focused implementation

---

**Next Step**: After implementing all 5 fixes, proceed to IMPROVEMENT_ROADMAP.md for Phase 2 work.

