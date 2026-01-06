# How-to: Optimize Performance

**Problem**: Your ontologies are slow to parse, queries are taking too long, or code generation is inefficient

**Solution**: Apply targeted optimizations to meet performance SLOs

## Performance Targets (SLOs)

| Operation | Target | Typical | Margin |
|-----------|--------|---------|--------|
| Turtle parsing | < 50ms | 18.5ms | 63% |
| Validation | < 10ms | 2.3ms | 77% |
| SPARQL SELECT | < 10ms | 4.6ms | 54% |
| Code generation | < 500ms | 76.5ms | 85% |
| E2E workflow | < 500ms | 95.1ms | 81% |

## Step 1: Measure Current Performance

Create a benchmark:

```rust
use std::time::Instant;
use clap_noun_verb::rdf::turtle_parser::TurtleParser;

pub fn benchmark_operations(turtle_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(turtle_file)?;

    // Parse
    let start = Instant::now();
    let parser = TurtleParser::new();
    let ontology = parser.parse(&content)?;
    let parse_time = start.elapsed();

    // Validate
    let start = Instant::now();
    ontology.validate_ontology()?;
    let validate_time = start.elapsed();

    // Query
    let executor = SparqlExecutor::new(&ontology)?;
    let start = Instant::now();
    executor.execute_query("SELECT ?v WHERE { ?v a cnv:Verb }")?;
    let query_time = start.elapsed();

    // Generate
    let generator = CliCodeGenerator::new()?;
    let start = Instant::now();
    let generated = generator.generate_from_ontology(&ontology)?;
    let gen_time = start.elapsed();

    println!("Performance Report:");
    println!("  Parse:      {} ms (SLO: 50ms) {}",
        parse_time.as_millis(),
        if parse_time.as_millis() < 50 { "✅" } else { "❌" });
    println!("  Validate:   {} ms (SLO: 10ms) {}",
        validate_time.as_millis(),
        if validate_time.as_millis() < 10 { "✅" } else { "❌" });
    println!("  Query:      {} ms (SLO: 10ms) {}",
        query_time.as_millis(),
        if query_time.as_millis() < 10 { "✅" } else { "❌" });
    println!("  Generate:   {} ms (SLO: 500ms) {}",
        gen_time.as_millis(),
        if gen_time.as_millis() < 500 { "✅" } else { "❌" });
    println!("  Generated code: {} lines", generated.rust_code().lines().count());

    Ok(())
}
```

## Step 2: Parse Optimization

### Issue: Parsing Large Ontologies

**Solution 1: Lazy Loading**

```rust
pub struct LazyOntology {
    path: String,
    cached: Arc<RwLock<Option<ParsedTurtle>>>,
}

impl LazyOntology {
    pub fn new(path: String) -> Self {
        LazyOntology {
            path,
            cached: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get(&self) -> Result<Arc<ParsedTurtle>, Box<dyn std::error::Error>> {
        // Check cache
        {
            let cached = self.cached.read().await;
            if let Some(ref ontology) = *cached {
                return Ok(Arc::new(ontology.clone()));
            }
        }

        // Load and cache
        let content = std::fs::read_to_string(&self.path)?;
        let parser = TurtleParser::new();
        let ontology = parser.parse(&content)?;

        {
            let mut cached = self.cached.write().await;
            *cached = Some(ontology.clone());
        }

        Ok(Arc::new(ontology))
    }
}
```

**Solution 2: Incremental Parsing**

For very large ontologies, split into multiple files:

```turtle
# main.ttl - import pattern
@base <https://myapp.dev/> .

# Import sub-ontologies
@include <nouns/services.ttl>
@include <nouns/config.ttl>
@include <nouns/database.ttl>
```

## Step 3: Query Optimization

### Issue: Slow SPARQL Queries

**Solution 1: Query Caching**

```rust
use std::collections::HashMap;

pub struct CachedExecutor {
    executor: SparqlExecutor,
    cache: Arc<RwLock<HashMap<String, Vec<QueryResult>>>>,
    cache_ttl: std::time::Duration,
}

impl CachedExecutor {
    pub async fn execute_cached(
        &self,
        query: &str,
    ) -> Result<Vec<QueryResult>, Box<dyn std::error::Error>> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(results) = cache.get(query) {
                return Ok(results.clone());
            }
        }

        // Execute and cache
        let results = self.executor.execute_query(query)?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(query.to_string(), results.clone());
        }

        // Clear cache after TTL
        let cache = self.cache.clone();
        let query = query.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let mut c = cache.write().await;
            c.remove(&query);
        });

        Ok(results)
    }
}
```

**Solution 2: Query Specialization**

```rust
// Instead of generic query execution
// Use specialized methods for common queries

impl SparqlExecutor {
    pub fn list_verbs_cached(&mut self) -> Result<Vec<String>, SparqlError> {
        if let Some(cached) = self.verbs_cache.as_ref() {
            return Ok(cached.clone());
        }

        let verbs = self.list_verbs()?;
        self.verbs_cache = Some(verbs.clone());
        Ok(verbs)
    }
}
```

## Step 4: Code Generation Optimization

### Issue: Slow Code Generation for Large Ontologies

**Solution 1: Parallel Generation**

```rust
use rayon::prelude::*;

pub fn generate_parallel(
    ontology: &ParsedTurtle,
) -> Result<GeneratedCli, CodeGenError> {
    let executor = SparqlExecutor::new(ontology)?;

    // Get all verbs
    let verbs = executor.list_verbs()?;

    // Generate in parallel
    let generated_verbs: Vec<_> = verbs.par_iter()
        .map(|verb| generate_verb_code(verb))
        .collect();

    Ok(combine_generated_code(generated_verbs))
}
```

**Solution 2: Incremental Code Generation**

```rust
pub fn generate_incremental(
    old_ontology: &ParsedTurtle,
    new_ontology: &ParsedTurtle,
) -> Result<String, CodeGenError> {
    let old_executor = SparqlExecutor::new(old_ontology)?;
    let new_executor = SparqlExecutor::new(new_ontology)?;

    let old_verbs = old_executor.list_verbs()?;
    let new_verbs = new_executor.list_verbs()?;

    // Generate only new/changed verbs
    let new_only: Vec<_> = new_verbs.iter()
        .filter(|v| !old_verbs.contains(v))
        .collect();

    let mut code = String::new();
    for verb in new_only {
        code.push_str(&generate_verb_code(verb)?);
    }

    Ok(code)
}
```

## Step 5: Memory Optimization

### Issue: High Memory Usage

**Solution 1: Use Borrowing**

```rust
// Bad: Clones everywhere
pub fn process(ontology: ParsedTurtle) -> Result<()> {
    let copy1 = ontology.clone();
    let copy2 = ontology.clone();
    // ...
}

// Good: Borrow references
pub fn process(ontology: &ParsedTurtle) -> Result<()> {
    validate_ontology(ontology)?;
    generate_code(ontology)?;
    // ...
}
```

**Solution 2: Streaming**

```rust
// Bad: Load entire ontology into memory
let all_verbs = executor.list_verbs()?;
for verb in all_verbs {
    process_verb(&verb)?;
}

// Good: Stream results
let query = "SELECT ?v WHERE { ?v a cnv:Verb }";
for result in executor.execute_query_streaming(query)? {
    let verb = result?.get("v")?;
    process_verb(&verb)?;
}
```

## Step 6: Compiler Optimizations

Ensure your `Cargo.toml` has release optimizations:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
```

## Step 7: Profile Hot Paths

Identify where time is spent:

```bash
# Using perf (Linux)
cargo build --release
perf record -g ./target/release/my_cli
perf report

# Using flamegraph
cargo install flamegraph
cargo flamegraph -- your_command
```

## Step 8: Batch Operations

```rust
// Bad: Individual operations
for verb in verbs {
    validate_verb(&verb)?;
}
for verb in verbs {
    generate_code(&verb)?;
}

// Good: Batch operations
validate_all_verbs(&verbs)?;
generate_all_code(&verbs)?;
```

## Optimization Checklist

- ✅ Measured current performance against SLOs
- ✅ Identified bottlenecks with profiling
- ✅ Implemented caching for repeated queries
- ✅ Used lazy loading for large ontologies
- ✅ Parallelized independent operations
- ✅ Enabled compiler optimizations
- ✅ Re-measured and confirmed improvements
- ✅ Set performance alerts/monitoring

## Performance Validation

After optimization, verify improvements:

```bash
cargo make slo-check
```

Expected output:
```
✅ All SLOs met with margin > 50%
```

---

**Related**:
- [Explanation: Performance Optimization](../explanation/performance.md)
- [How-to: Validate Ontologies](validation.md)
- [Reference: Performance SLOs](../reference/performance-slos.md)
