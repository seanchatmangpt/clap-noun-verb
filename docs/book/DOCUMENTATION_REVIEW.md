# Documentation Review: Book vs Examples

This document reviews the `docs/book` documentation against the actual `examples` to identify inconsistencies, gaps, and areas for improvement.

## Executive Summary

**Critical Issues Found:**
1. ❌ **API Version Mismatch**: Documentation shows mixed use of old `noun!`/`verb!` macro API and new `#[verb]` attribute macro API
2. ❌ **Auto-inference Not Clearly Documented**: Powerful auto-inference features from examples are not clearly demonstrated
3. ❌ **Missing Core Patterns**: Several patterns shown in examples are not documented
4. ❌ **Inconsistent Function Names**: Examples use `clap_noun_verb::run()`, documentation shows `run_cli()`

**Key Findings:**
- Examples consistently use v3.0.0 attribute macro API (`#[verb]`)
- Examples demonstrate powerful auto-inference (verb names from function names, noun names from filenames)
- Examples show automatic JSON serialization with `Serialize`
- Examples demonstrate clear separation of business logic from CLI layer
- Documentation mixes old and new APIs inconsistently

## Detailed Analysis

### 1. API Version Consistency

#### Issue: Mixed API Versions in Documentation

**Examples Show:**
```rust
// All examples use v3.0.0 attribute macro API
#[verb] // Auto-inferred verb name, auto-inferred noun from filename
fn show_status() -> Result<Status> { ... }

fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovery
}
```

**Documentation Shows:**
- Mix of old `noun!`/`verb!` macro API and new `#[verb]` attribute macro API
- Inconsistent examples showing both approaches
- No clear guidance on which API to use

**Recommendation:**
- Document v3.0.0 attribute macro API as the primary/recommended approach
- Move old macro API to "Legacy API" section or remove
- Update all examples to use `#[verb]` attribute macro API consistently

### 2. Auto-Inference Documentation

#### Issue: Powerful Auto-Inference Features Not Clearly Documented

**Examples Demonstrate:**
1. **Verb Name Auto-Inference**: `show_status()` → verb name "status"
2. **Noun Name Auto-Inference**: File `services.rs` → noun name "services"
3. **Function Name Parsing**: `show_collector_status()` with noun="collector" → verb="status"
4. **Module Doc Extraction**: `//! Manage services` → noun about "Manage services"

**Examples Showing Auto-Inference:**

```rust
// collector.rs
//! Manage OpenTelemetry collector

#[verb("up")] // Custom verb name, noun "collector" auto-inferred from filename
fn start_collector() -> Result<CollectorStatus> { ... }

#[verb] // Verb name "status" auto-inferred, noun "collector" auto-inferred
fn show_collector_status() -> Result<CollectorStatus> { ... }
```

```rust
// services.rs
//! Manage application services

#[verb] // Verb "status" auto-inferred, noun "services" auto-inferred from filename
fn show_status() -> Result<ServiceStatus> { ... }
```

**Documentation Coverage:**
- ❌ Auto-inference mentioned but not clearly demonstrated
- ❌ File-based noun inference not explained
- ❌ Function name parsing rules not documented
- ❌ Module doc extraction not mentioned

**Recommendation:**
- Add dedicated section on auto-inference
- Show multiple examples of auto-inference patterns
- Document when to use explicit vs inferred names
- Show how module docs are extracted

### 3. Auto-Discovery Pattern

#### Issue: Function Name Mismatch

**Examples Use:**
```rust
fn main() -> Result<()> {
    clap_noun_verb::run() // Auto-discovers all #[verb] functions
}
```

**Documentation Shows:**
```rust
run_cli(|cli| {
    cli.name("ggen")
        .noun(/* ... */)
})
```

**Recommendation:**
- Update documentation to show `clap_noun_verb::run()` as primary approach
- Document `run_cli` as alternative for custom configuration
- Show both patterns clearly with use cases

### 4. JSON Output Feature

#### Issue: Automatic JSON Serialization Not Emphasized

**Examples Show:**
- All return types use `#[derive(Serialize)]`
- Output automatically serialized to JSON
- Perfect for agents/MCP servers

**Documentation Coverage:**
- ❌ JSON output mentioned but not emphasized
- ❌ `Serialize` requirement not clearly documented
- ❌ Use case (agents/MCP) not mentioned

**Recommendation:**
- Add prominent section on JSON output
- Show example of JSON output
- Explain `Serialize` requirement
- Mention use case for agents/MCP servers

### 5. Separation of Concerns Pattern

#### Issue: Business Logic Separation Not Clearly Demonstrated

**Examples Show:**
```rust
// Business Logic Layer (Pure Functions - Reusable)
fn get_service_status() -> ServiceStatus { ... }

// CLI Layer (Input Validation + Output Shaping Only)
#[verb]
fn show_status() -> Result<ServiceStatus> {
    Ok(get_service_status()) // Delegate to business logic
}
```

**Documentation Coverage:**
- Pattern mentioned but not clearly demonstrated
- No clear guidance on separation
- Examples in documentation don't show this pattern consistently

**Recommendation:**
- Add dedicated section on separation of concerns
- Show clear before/after examples
- Explain benefits of this pattern
- Show how business logic can be reused

### 6. Multiple Nouns in Same File

#### Issue: Pattern Not Documented

**Examples Show:**
```rust
// basic.rs - Multiple nouns with explicit nouns
#[verb("status", "services")] // Explicit noun since filename is "basic.rs"
fn show_status() -> Result<ServiceStatus> { ... }

#[verb("up", "collector")] // Explicit noun since filename is "basic.rs"
fn start_collector() -> Result<CollectorStatus> { ... }
```

**Documentation Coverage:**
- ❌ Pattern not documented
- ❌ No explanation of when to use explicit nouns

**Recommendation:**
- Document pattern for multiple nouns in same file
- Explain when explicit nouns are required
- Show example of single noun file vs multiple noun file

### 7. Type Inference Documentation

#### Issue: Not Comprehensive Enough

**Examples Show:**
- Automatic type inference from function signatures
- `String` → required argument
- `Option<T>` → optional argument
- `bool` → flag
- `Option<usize>` → optional argument with type validation

**Documentation Coverage:**
- Basic type inference shown
- ❌ Type validation (u8, u16 ranges) not documented
- ❌ Multiple examples of type inference missing

**Recommendation:**
- Add comprehensive type inference table
- Show all supported types
- Document automatic validation (u8 = 0-255, u16 = 0-65535)
- Show examples for each type

### 8. Argument Descriptions from Doc Comments

#### Issue: Pattern Not Documented

**Examples Show:**
```rust
/// Show logs for a service
///
/// # Arguments
/// * `service` - Service name (required)
/// * `lines` - Number of lines to show (default: 50)
#[verb]
fn show_logs(service: String, lines: Option<usize>) -> Result<Logs> { ... }
```

**Documentation Coverage:**
- Pattern mentioned but not clearly demonstrated
- No examples showing doc comment parsing

**Recommendation:**
- Add section on doc comment argument descriptions
- Show examples of doc comment format
- Explain how descriptions are extracted

## Missing Patterns from Examples

### 1. Auto-Noun Pattern
**Example:** `auto_noun.rs` shows file-based noun auto-inference
**Documentation:** ❌ Not documented

### 2. Explicit Noun in Multi-Noun Files
**Example:** `basic.rs` shows multiple nouns with explicit nouns
**Documentation:** ❌ Not documented

### 3. Custom Verb Names
**Example:** `collector.rs` shows `#[verb("up")]` and `#[verb("down")]`
**Documentation:** ⚠️ Mentioned but not clearly demonstrated

### 4. Module Doc Extraction
**Example:** All examples show `//! Manage services` at top
**Documentation:** ❌ Not documented

### 5. Business Logic Separation
**Example:** All examples show clear separation with comments
**Documentation:** ⚠️ Mentioned but not emphasized

## Specific Documentation Issues

### `getting-started.md`
- ❌ Shows `noun!` macro API alongside `#[verb]` without clear guidance
- ❌ Type inference section not comprehensive
- ⚠️ Basic concepts section uses old macro API

### `porting-commands.md`
- ❌ Most examples use old `noun!`/`verb!` macro API
- ❌ Only one example shows `#[verb]` attribute macro API
- ❌ Doesn't show auto-inference clearly

### `advanced-patterns.md`
- ❌ Uses old macro API throughout
- ❌ Doesn't show `#[verb]` attribute macro API
- ❌ Doesn't demonstrate auto-inference patterns

### `introduction.md`
- ⚠️ Shows both APIs but doesn't clearly recommend v3.0.0
- ❌ Doesn't emphasize auto-discovery

## Recommendations

### Priority 1: Critical Fixes
1. **Standardize on v3.0.0 API**: Update all examples to use `#[verb]` attribute macro API
2. **Document Auto-Inference**: Add comprehensive section on auto-inference
3. **Fix Function Names**: Update `run_cli` to `clap_noun_verb::run()`
4. **Emphasize JSON Output**: Add prominent section on automatic JSON serialization

### Priority 2: Important Improvements
1. **Separation of Concerns**: Add dedicated section with clear examples
2. **Multiple Nouns Pattern**: Document pattern for multiple nouns in same file
3. **Type Inference**: Add comprehensive type inference documentation
4. **Doc Comment Parsing**: Document argument descriptions from doc comments

### Priority 3: Nice to Have
1. **Module Doc Extraction**: Document how module docs are extracted
2. **Business Logic Examples**: Show more examples of separated business logic
3. **Migration Guide**: Add section showing old API → new API migration

## Example Corrections Needed

### `getting-started.md`
```diff
- ```rust,no_run
- noun!("ai", "AI-powered generation", [
-     verb!("project", ...),
- ])
- ```
+ ```rust,no_run
+ // ai.rs
+ //! AI-powered generation
+ 
+ use clap_noun_verb_macros::verb;
+ use clap_noun_verb::Result;
+ 
+ #[verb] // Verb "project" auto-inferred, noun "ai" auto-inferred from filename
+ fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
+     Ok(ProjectOutput { name, rust })
+ }
+ 
+ fn main() -> Result<()> {
+     clap_noun_verb::run() // Auto-discovers all commands!
+ }
+ ```
```

### `porting-commands.md`
- Replace all `noun!`/`verb!` examples with `#[verb]` attribute macro examples
- Show auto-inference clearly
- Add examples matching actual examples in codebase

### `advanced-patterns.md`
- Update all examples to use `#[verb]` attribute macro API
- Add section on auto-inference
- Show multiple nouns in same file pattern

## Conclusion

The documentation needs significant updates to match the actual implementation and examples. The main issues are:

1. **API Inconsistency**: Mix of old and new APIs without clear guidance
2. **Missing Features**: Auto-inference and auto-discovery not clearly documented
3. **Pattern Gaps**: Several patterns from examples not documented
4. **Incomplete Examples**: Examples don't match actual codebase examples

Priority should be given to standardizing on the v3.0.0 attribute macro API and clearly documenting the auto-inference and auto-discovery features that make the framework powerful.
