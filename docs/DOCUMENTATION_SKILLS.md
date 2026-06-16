# Documentation Skills & Workflow Guide

**A practical, task-oriented guide for maintaining high-quality documentation across clap-noun-verb.**

Version: 1.0  
Last Updated: 2026-06-14  
Status: Active

---

## Table of Contents

1. [Overview](#overview)
2. [Core Documentation Skills](#core-documentation-skills)
3. [Skill: Generating & Publishing Rustdoc](#skill-generating--publishing-rustdoc)
4. [Skill: Documentation Comment Patterns](#skill-documentation-comment-patterns)
5. [Skill: Creating Feature Documentation](#skill-creating-feature-documentation)
6. [Skill: Managing Examples](#skill-managing-examples)
7. [Skill: Writing ADRs](#skill-writing-adrs)
8. [Skill: Updating README](#skill-updating-readme)
9. [Quality Assurance Workflows](#quality-assurance-workflows)
10. [Integration with Development](#integration-with-development)

---

## Overview

This guide organizes documentation work into discrete **skills** — focused competencies developers can learn and apply independently. Each skill includes:

- **Quick reference** - Essential commands at a glance
- **Detailed walkthrough** - Step-by-step instructions
- **Common patterns** - Reusable templates
- **Validation checklist** - Quality gates
- **Troubleshooting** - How to fix common issues

### When to Use This Guide

Use this guide when:
- Adding a new feature and documenting it
- Creating examples for the project
- Publishing a new version
- Writing ADRs for architectural decisions
- Maintaining examples/ directory
- Updating README or reference docs

### Related Documentation

- **Full Guide:** [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md) — Comprehensive reference
- **Quick Reference:** [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) — One-pager
- **Examples:** [examples/README.md](../examples/README.md) — Example navigation
- **Contributing:** [../CONTRIBUTING.md](../CONTRIBUTING.md) — Code contribution guidelines

---

## Core Documentation Skills

The project uses **Diataxis Framework** for documentation organization:

| Skill | Purpose | Examples | Location |
|-------|---------|----------|----------|
| **Tutorial** | Learn from scratch | Step-by-step guides | `docs/tutorial/` |
| **How-To** | Solve a specific problem | Task-oriented guides | `docs/howto/` |
| **Reference** | Look up exact API | API signatures, complete specs | `docs/reference/` |
| **Explanation** | Understand design decisions | Architecture, philosophy, rationale | `docs/explanation/` |

Each skill builds on the others. Use in sequence when learning a topic:

```
Tutorial (learn basics)
   ↓
How-To (solve specific problems)
   ↓
Reference (detailed API)
   ↓
Explanation (understand why)
```

---

## Skill: Generating & Publishing Rustdoc

**What this covers:** Creating and publishing documentation from Rust doc comments.

### Quick Reference

```bash
# Generate docs locally
cargo make doc

# View in browser (macOS/Linux)
open target/doc/clap_noun_verb/index.html

# Test all doc examples compile
cargo test --doc

# Test doc examples in specific module
cargo test --doc module::

# Generate with specific features
cargo doc --features async,repl --no-deps

# Publish to crates.io (maintainers only)
cargo make publish-macros  # Macros first
cargo make publish         # Then main crate
```

### Detailed Walkthrough

#### Step 1: Write or Update Doc Comments

In any `.rs` file, add or update documentation above public items:

```rust
//! # Module Name
//!
//! Brief description of module's purpose.

/// Brief one-liner description.
///
/// Longer explanation with examples and use cases.
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::MyType;
/// let value = MyType::new();
/// ```
pub fn my_function() {
    // Implementation
}
```

#### Step 2: Test Doc Examples Locally

Before committing, ensure all examples compile and run:

```bash
# Test all doc examples
cargo test --doc

# Test specific module's doc examples
cargo test --doc cli::

# Test and show output
cargo test --doc -- --nocapture
```

**Expected output:**
```
test doc-tests::cli::builder ... ok
test doc-tests::cli::router ... ok
...
test result: ok. X passed; 0 failed; 0 ignored
```

#### Step 3: Generate Documentation

Generate the complete HTML documentation:

```bash
cargo make doc
```

This creates documentation in `target/doc/clap_noun_verb/index.html`.

#### Step 4: Review Generated Docs

Open in a browser and review:

```bash
# macOS
open target/doc/clap_noun_verb/index.html

# Linux
xdg-open target/doc/clap_noun_verb/index.html

# Windows
start target/doc/clap_noun_verb/index.html
```

Check for:
- ✅ All public items documented
- ✅ Examples are visible and correct
- ✅ Links are working
- ✅ Code blocks render properly
- ✅ Navigation is clear

#### Step 5: Publish (Maintainers)

When releasing a new version:

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
# Commit and tag

git tag v26.6.13
git push origin main --tags

# Publish (must publish macros first!)
cargo make publish-macros
cargo make publish
```

Documentation automatically appears on [docs.rs](https://docs.rs/clap-noun-verb/).

### Common Patterns

#### Pattern: Module Documentation

```rust
//! # Module Name
//!
//! Brief one-sentence description.
//!
//! ## Overview
//!
//! Longer explanation of what this module provides and typical use cases.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! # use clap_noun_verb::module;
//! // Example code
//! ```
//!
//! ### Advanced Configuration
//!
//! ```rust
//! # use clap_noun_verb::module;
//! // More complex example
//! ```
//!
//! ## See Also
//!
//! - [`related_function`] - For related operation
//! - `other_module` - Related functionality
```

#### Pattern: Function with Errors

```rust
/// Does something and returns a result.
///
/// Longer explanation of what the function does and when to use it.
///
/// # Arguments
///
/// * `input` - What this parameter does
/// * `options` - Configuration options
///
/// # Returns
///
/// Describes what the function returns in success and error cases.
///
/// # Errors
///
/// Returns `Err` when:
/// - Condition 1 (e.g., "input is empty")
/// - Condition 2 (e.g., "resource not found")
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::my_function;
/// # fn main() -> clap_noun_verb::Result<()> {
/// let result = my_function("input")?;
/// assert_eq!(result.status, "success");
/// # Ok(())
/// # }
/// ```
pub fn my_function(input: &str, options: Config) -> Result<Output> {
    // Implementation
}
```

#### Pattern: Type with Fields

```rust
/// Brief description of what this type represents.
///
/// Detailed explanation of when and why to use this type.
///
/// # Fields
///
/// * `field1` - What this field represents
/// * `field2` - What this field represents
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::MyType;
/// let instance = MyType {
///     field1: "value",
///     field2: 42,
/// };
/// ```
pub struct MyType {
    /// Field documentation
    pub field1: String,
    /// Field documentation
    pub field2: i32,
}
```

### Validation Checklist

Before committing doc changes:

- [ ] All public items have `///` or `//!` documentation
- [ ] All doc tests compile: `cargo test --doc`
- [ ] Examples are realistic and complete
- [ ] No `unwrap()`, `expect()`, or `panic!()` in doc examples
- [ ] Links use backtick syntax: ``[`Type`]``
- [ ] Error cases documented with `# Errors`
- [ ] Related items linked with "See Also"
- [ ] Generated docs look good: `cargo make doc`
- [ ] No broken links in documentation

### Troubleshooting

**Problem: Doc test fails to compile**

```
error: could not compile `clap-noun-verb`
...
error[E0433]: cannot find function `my_function` in this scope
```

**Solution:** Ensure you're importing the right items in your doc test:

```rust
/// ```rust
/// use clap_noun_verb::my_function;  // Add the import!
/// # fn main() -> clap_noun_verb::Result<()> {
/// let result = my_function("input")?;
/// # Ok(())
/// # }
/// ```
```

**Problem: Doc examples show unwrap() warnings**

```
warning: use of `unwrap` in doc tests
```

**Solution:** Use `#` prefix to hide setup code:

```rust
/// ```rust
/// # use clap_noun_verb::my_function;
/// # fn main() -> clap_noun_verb::Result<()> {
/// let result = my_function("input")?;
/// # Ok(())
/// # }
/// ```
```

---

## Skill: Documentation Comment Patterns

**What this covers:** Writing clear, maintainable documentation comments that follow project conventions.

### Quick Reference

```bash
# Check doc formatting
cargo make format-check

# Fix doc formatting
cargo make format

# Test all doc comments compile
cargo test --doc

# Check for missing docs (nightly)
cargo +nightly doc --no-deps 2>&1 | grep "missing docs"
```

### Common Patterns by Item Type

#### Macro Documentation

```rust
/// Registers a function as a CLI verb (command).
///
/// This macro is the primary entry point for command registration.
/// Commands are automatically discovered at compile time and registered
/// with the CLI framework.
///
/// # Syntax
///
/// ```ignore
/// #[verb("command-name")]
/// fn handler_name(arg1: Type1, arg2: Type2) -> Result<Output>
/// ```
///
/// # Attributes
///
/// - `"name"` (required) - The command name visible to users
/// - `noun = "parent"` (optional) - Parent noun for hierarchical commands
///
/// # Return Type Requirements
///
/// The return type MUST implement `Serialize + Send + 'static`.
///
/// # Examples
///
/// ## Basic Command
///
/// ```ignore
/// #[verb("greet")]
/// fn cmd_greet(name: String) -> Result<Greeting> {
///     Ok(Greeting {
///         message: format!("Hello, {}!", name),
///     })
/// }
/// ```
///
/// # Compile-Time Validation
///
/// The macro validates:
/// - Return type is `Serialize`
/// - All arguments are supported types
/// - No duplicate command names
///
/// # See Also
///
/// - [`#[noun]`] - For noun subcommands
/// - [`#[arg]`] - For argument attributes
pub use clap_noun_verb_macros::verb;
```

#### Trait Documentation

```rust
/// Trait for types that can be converted to CLI commands.
///
/// Implement this trait to register commands that are automatically
/// discovered at compile time via the `#[verb]` macro.
///
/// # Examples
///
/// Typically, you don't implement this manually. Use the `#[verb]`
/// macro instead:
///
/// ```ignore
/// use clap_noun_verb_macros::verb;
/// use clap_noun_verb::Result;
///
/// #[verb("create")]
/// fn create_resource(name: String) -> Result<()> {
///     println!("Created: {}", name);
///     Ok(())
/// }
/// ```
///
/// # See Also
///
/// - [`NounCommand`] - For noun-level trait
pub trait VerbCommand {
    /// Execute the command with given arguments
    fn execute(&self, input: HandlerInput) -> Result<HandlerOutput>;
}
```

#### Error Type Documentation

```rust
/// Error type for [operation/module].
///
/// Represents failures that can occur when [operation description].
///
/// # Variants
///
/// - `InvalidInput` - Input doesn't match expected format
/// - `NotFound` - Requested resource doesn't exist
/// - `PermissionDenied` - User lacks necessary permissions
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::MyError;
/// # fn operation() -> Result<(), MyError> {
/// # Err(MyError::NotFound("resource".to_string()))?
/// # Ok(())
/// # }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
```

#### Enum Documentation

```rust
/// Possible outcomes of a CLI operation.
///
/// # Variants
///
/// * `Success(T)` - Operation succeeded with value T
/// * `PartialFailure { .. }` - Operation partially succeeded
/// * `Failure(Error)` - Operation completely failed
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::Outcome;
/// let result: Outcome<String> = Outcome::Success("data".to_string());
/// match result {
///     Outcome::Success(data) => println!("Got: {}", data),
///     Outcome::PartialFailure { value, warnings } => {
///         println!("Got: {} with {} warnings", value, warnings.len());
///     },
///     Outcome::Failure(e) => println!("Error: {}", e),
/// }
/// ```
#[derive(Debug, Serialize)]
pub enum Outcome<T> {
    Success(T),
    PartialFailure {
        value: T,
        warnings: Vec<String>,
    },
    Failure(Error),
}
```

### Doc Test Guidelines

#### ✅ Good Example

```rust
/// ```rust
/// use clap_noun_verb::builder::CliBuilder;
///
/// # fn main() -> clap_noun_verb::Result<()> {
/// let builder = CliBuilder::new();
/// let command = builder.build()?;
/// println!("Built command with {} subcommands", command.get_subcommands().count());
/// # Ok(())
/// # }
/// ```
```

Features:
- Complete, compilable example
- Shows realistic usage
- Handles errors properly with `?`
- Uses `# fn main()` to hide setup
- Produces meaningful output

#### ❌ Bad Examples

```rust
/// ```rust
/// // Missing imports - won't compile
/// let builder = CliBuilder::new();
/// builder.build()
/// ```

/// ```rust
/// // Stub - teaches nothing
/// assert!(builder.build().is_ok());
/// ```

/// ```rust
/// // Uses unwrap - violates guidelines
/// let result = builder.build().unwrap();
/// ```
```

### Special Doc Test Syntax

```rust
/// Compile and run normally:
/// ```rust
/// let x = 2 + 2;
/// assert_eq!(x, 4);
/// ```

/// Pseudo-code (compile only, don't run):
/// ```rust,ignore
/// #[verb("pseudo")]
/// fn pseudocode(data: impl Iterator<Item = String>) -> Result<Summary> {
///     // Implementation details omitted
/// }
/// ```

/// Text documentation (don't compile):
/// ```text
/// This is documentation text, not Rust code.
/// It won't be compiled or run.
/// ```

/// Expected to panic:
/// ```rust,should_panic
/// panic!("This example demonstrates panic handling");
/// ```

/// Long-running operation (compile only):
/// ```rust,no_run
/// async fn long_operation() -> Result<()> {
///     // This won't be run in tests
///     Ok(())
/// }
/// ```
```

### Validation Checklist

- [ ] All public items documented
- [ ] All doc examples compile: `cargo test --doc`
- [ ] No unwrap/expect/panic in examples
- [ ] Examples are realistic, not stubs
- [ ] Error cases shown where applicable
- [ ] Related items linked with `[`Type`]`
- [ ] "See Also" section present for cross-references
- [ ] Headings follow standard format
- [ ] Code formatting is consistent
- [ ] Terminology is consistent across docs

---

## Skill: Creating Feature Documentation

**What this covers:** Documenting new macros, features, and APIs comprehensively.

### Quick Reference

When adding a new feature, create four artifacts:

```
1. Macro/API doc comments (src/ or clap-noun-verb-macros/src/)
2. How-To guide (docs/howto/feature-name.md)
3. Reference documentation (docs/reference/api/feature-name.md)
4. Working example (examples/howto/feature_name.rs)
```

### Step-by-Step Walkthrough

#### Step 1: Document the Macro/API

In the source file, add comprehensive doc comments:

```rust
// clap-noun-verb-macros/src/lib.rs

/// Validates arguments with custom constraints.
///
/// This macro allows you to register custom validation rules on arguments,
/// providing better error messages than simple type checking.
///
/// # Syntax
///
/// ```ignore
/// #[arg(validate = MyValidator)]
/// field: String,
/// ```
///
/// # Requirements
///
/// The validator must implement `Fn(&str) -> Result<()>`.
///
/// # Examples
///
/// ```ignore
/// fn is_positive(s: &str) -> Result<()> {
///     s.parse::<i32>()
///         .and_then(|n| if n > 0 { Ok(()) } else {
///             Err(/* error */)
///         })
/// }
///
/// #[verb("add")]
/// fn cmd_add(
///     #[arg(validate = is_positive)]
///     number: i32,
/// ) -> Result<Sum> {
///     Ok(Sum { value: number })
/// }
/// ```
///
/// # See Also
///
/// - [`#[arg]`] - Base argument attribute
/// - [`validators`] module - Built-in validators
pub use clap_noun_verb_macros::validate;
```

Test the doc comments:

```bash
cargo test --doc
```

#### Step 2: Write a How-To Guide

File: `docs/howto/custom-validation.md`

```markdown
# How to Use Custom Validation

**For:** Developers who need validation beyond simple type checking.

## Quick Start

```rust
fn email_validator(s: &str) -> clap_noun_verb::Result<()> {
    if s.contains('@') && s.contains('.') {
        Ok(())
    } else {
        Err(clap_noun_verb::Error::InvalidInput("Invalid email".into()))
    }
}

#[verb("signup")]
fn cmd_signup(
    #[arg(validate = email_validator)]
    email: String,
) -> Result<SignupResult> {
    Ok(SignupResult {
        email,
        status: "created",
    })
}
```

## Detailed Walkthrough

### Step 1: Define Your Validator

[Detailed explanation...]

### Step 2: Apply to Arguments

[Detailed explanation...]

### Step 3: Test Validation

[Detailed explanation...]

## Common Patterns

### Pattern A: Email Validation

[Code example...]

### Pattern B: Range Validation

[Code example...]

## Troubleshooting

### Problem: Validation runs but doesn't display error

[Solution...]

## See Also

- [Reference: validate attribute](../reference/api/validate-macro.md)
- [Example: howto_validation.rs](../../examples/howto/validation.rs)
```

#### Step 3: Create Reference Documentation

File: `docs/reference/api/validate-macro.md`

```markdown
# #[arg(validate = ...)] Reference

Complete reference for the validate attribute on arguments.

## Syntax

```
#[arg(validate = validator_function)]
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| validator | function | Yes | Function taking `&str` and returning `Result<()>` |

## Return Type

The validator function must have signature:
```rust
fn(input: &str) -> clap_noun_verb::Result<()>
```

## Execution

- Called after type parsing
- Error messages displayed to user
- Can access full input string

## Examples

[Minimal examples...]

## See Also

- [How-To: Custom Validation](../../howto/custom-validation.md)
- [`validators` module](validators-module.md)
```

#### Step 4: Create a Working Example

File: `examples/howto/validation.rs`

```rust
//! How-To: Custom Input Validation
//!
//! This example demonstrates how to create and use custom validators
//! for CLI arguments beyond simple type checking.
//!
//! # Learning Goals
//!
//! By the end, you'll know:
//! - How to write a custom validator function
//! - How to apply validators to arguments
//! - How validation integrates with the CLI framework
//!
//! # Run With
//!
//! ```sh
//! cargo run --example howto_validation -- --help
//! cargo run --example howto_validation -- create user@example.com
//! cargo run --example howto_validation -- create invalid-email
//! ```

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Custom validator function
fn email_validator(input: &str) -> Result<()> {
    if input.contains('@') && input.contains('.') {
        Ok(())
    } else {
        Err("Must be valid email (contains @ and .)".into())
    }
}

#[derive(Serialize)]
pub struct CreateResult {
    email: String,
    status: String,
}

#[verb("create")]
fn cmd_create(
    /// User email address
    #[arg(long, validate = email_validator)]
    email: String,
) -> Result<CreateResult> {
    Ok(CreateResult {
        email,
        status: "created".to_string(),
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

Test the example:

```bash
cargo run --example howto_validation -- --help
cargo run --example howto_validation -- create user@example.com
```

#### Step 5: Update examples/README.md

Add to the appropriate section:

```markdown
## 🔧 How-To Examples (Task-Oriented)

| Example | Command | Purpose |
|---------|---------|---------|
| ... | ... | ... |
| **validation** | `cargo run --example howto_validation` | Custom argument validators |
```

#### Step 6: Update docs/reference/README.md

Add to the API reference table:

```markdown
| [`#[arg(validate = ...)]` attribute](api/validate-macro.md) | Custom validators for arguments |
```

### Validation Checklist

- [ ] Macro documentation complete with examples
- [ ] Doc tests compile: `cargo test --doc`
- [ ] How-To guide written (800-2000 words)
- [ ] Reference documentation complete
- [ ] Working example created and tested
- [ ] examples/README.md updated
- [ ] docs/reference/README.md updated
- [ ] All links verified
- [ ] Cross-references in place

---

## Skill: Managing Examples

**What this covers:** Creating, organizing, and maintaining working code examples.

### Quick Reference

```bash
# Build all examples
cargo make build-examples

# Run a specific example
cargo run --example tutorial_basic -- --help

# List all examples
cargo build --examples 2>&1 | grep "Compiling example"

# Test example with features
cargo run --example advanced_async --features async -- --help
```

### Directory Structure

```
examples/
├── README.md                          # Navigation guide
├── tutorial/                          # Learning-oriented
│   ├── basic.rs                       # 5-minute intro
│   ├── arguments.rs                   # Adding arguments
│   ├── positional.rs                  # Positional args
│   └── services.rs                    # Multi-noun CLI
├── howto/                             # Task-oriented
│   ├── arg_groups.rs                  # Mutually exclusive args
│   ├── validation.rs                  # Custom validation
│   ├── env_vars.rs                    # Environment variables
│   ├── arg_actions.rs                 # Count, append, etc.
│   └── deprecation.rs                 # Deprecated commands
├── reference/                         # API demonstrations
│   ├── attribute_macro.rs             # Complete #[verb] usage
│   ├── framework.rs                   # Full integration
│   ├── nested.rs                      # Multi-level nesting
│   ├── context.rs                     # AppContext usage
│   ├── collector.rs                   # Command collection
│   ├── format.rs                      # Output formatting
│   └── root_verb.rs                   # Root-level commands
└── greet-demo/                        # Standalone project
    ├── Cargo.toml
    ├── src/
    └── ontology.ttl
```

### Creating a New Example

#### Step 1: Choose Category

| Category | When | Example |
|----------|------|---------|
| **Tutorial** | Learning-focused, for beginners | Basic concepts step-by-step |
| **How-To** | Solves specific problem | Task-oriented, practical |
| **Reference** | Exhaustive API usage | Shows all options and patterns |

#### Step 2: Write the Example

Template for `tutorial`:

```rust
//! Tutorial: [Topic]
//!
//! This example teaches how to [learning objective].
//!
//! ## Learning Goals
//!
//! By the end, you'll understand:
//! - [Goal 1]
//! - [Goal 2]
//!
//! ## Run With
//!
//! ```sh
//! cargo run --example tutorial_topic -- [ARGS]
//! ```

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Step 1: Define the output type
#[derive(Serialize)]
pub struct Output {
    // Fields
}

// Step 2: Define the handler
#[verb("command")]
fn cmd_handler() -> Result<Output> {
    // Implementation
    Ok(Output {
        // Fields
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

#### Step 3: Test the Example

```bash
# Build
cargo build --example tutorial_topic

# Run
cargo run --example tutorial_topic -- --help

# Run with arguments
cargo run --example tutorial_topic -- arg1 arg2

# Check output
cargo run --example tutorial_topic -- command
```

#### Step 4: Update examples/README.md

Add to the appropriate section with:

```markdown
| tutorial/topic.rs | 5-minute introduction to [topic] | Learning [concept] |
```

### Example Quality Standards

Every example must:

- ✅ Have a module doc comment with learning goals
- ✅ Run without modification: `cargo run --example <name> -- [args]`
- ✅ Accept realistic input and produce meaningful output
- ✅ Compile cleanly: `cargo build --example <name>`
- ✅ Show common patterns, not edge cases
- ✅ Be listed in `examples/README.md` with category
- ✅ Be ≤200 lines of code (keep focused)
- ✅ Work with documented feature flags
- ✅ Have no `unwrap()` or `expect()` in main code

### Common Example Patterns

#### Pattern: Basic CLI

```rust
//! Tutorial: Basic CLI
//!
//! Demonstrates the simplest possible working CLI.

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct Output {
    message: String,
}

#[verb("hello")]
fn cmd_hello() -> Result<Output> {
    Ok(Output {
        message: "Hello, world!".to_string(),
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

#### Pattern: With Arguments

```rust
//! Tutorial: Arguments
//!
//! Shows how to add typed arguments to commands.

#[derive(Serialize)]
pub struct Output {
    greeting: String,
}

#[verb("greet")]
fn cmd_greet(
    /// Name to greet
    name: String,
    /// Number of times to greet
    #[arg(long, default_value = "1")]
    count: u32,
) -> Result<Output> {
    let greeting = format!(
        "{}\n",
        (0..count)
            .map(|_| format!("Hello, {}!", name))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(Output { greeting })
}
```

### Validation Checklist

- [ ] Module doc comment present
- [ ] Compiles: `cargo build --example <name>`
- [ ] Runs: `cargo run --example <name> -- --help`
- [ ] Output is meaningful and formatted well
- [ ] ≤200 lines of code
- [ ] Shows common patterns
- [ ] Listed in examples/README.md
- [ ] Feature requirements documented
- [ ] No unwrap() or panic!() in main code

---

## Skill: Writing ADRs

**What this covers:** Documenting architectural decisions for future reference.

### Quick Reference

```bash
# Create new ADR
touch docs/adr/NNNN-short-title.md

# List existing ADRs
ls docs/adr/

# ADR format: NNNN-short-title.md
# Example: 0003-minimalist-zero-default-features.md
```

### ADR Template

```markdown
# ADR NNNN: [Decision Title]

**Status:** Proposed | Accepted | Superseded

**Date:** YYYY-MM-DD

**Deciders:** Names of decision makers

## Context

Describe the problem or question that prompted this decision.

### Situation

What was happening that led to this decision?

### Challenge

What constraints or problems did we face?

### Example

> Users found flat command structures (e.g., `login`, `logout`) confusing.
> They wanted to organize commands hierarchically by resource type.

## Decision

State the decision clearly and concisely.

Example:
> We adopted a noun-verb command pattern where nouns represent resources
> (e.g., session, user) and verbs represent actions (e.g., create, delete).
> Commands are registered via `#[verb]` macros with compile-time auto-discovery.

## Rationale

Explain why this decision makes sense.

### Benefits

- **Benefit 1** - Brief explanation of why this helps
- **Benefit 2** - Brief explanation of why this helps

Example:
- **Domain alignment** - Maps naturally to business entity models
- **Discoverability** - `services --help` shows all service commands
- **Auto-discovery** - `#[verb]` + `linkme` eliminates manual routing
- **Scalability** - Works equally for 5 or 500 commands

### Trade-Offs

What are the costs or downsides?

Example:
- **Macro complexity** - Adds 0.5s to compile time
- **Learning curve** - Developers new to macros need onboarding
- **Error messages** - Proc-macro errors can be cryptic

## Consequences

### Positive

- **Consequence 1** - What good this enabled
- **Consequence 2** - What good this enabled

### Negative

- **Consequence 1** - What this prevented or complicated
- **Consequence 2** - What this prevented or complicated

### Neutral

- **Consequence** - Neither benefit nor drawback

Example for minimalist architecture:
- Requires Rust 1.74+ (already minimum version anyway)

## Alternatives Considered

### Alternative A: [Name]

**Description:** Brief description of approach

**Trade-offs:**
- ✅ Benefit
- ❌ Drawback

Example:
### Alternative A: Trait-based registration

**Description:** Users manually implement `VerbCommand` trait

**Trade-offs:**
- ✅ No macro overhead
- ❌ More boilerplate per command
- ❌ No compile-time auto-discovery

### Alternative B: [Name]

[Similar format]

## Related Decisions

- [ADR 0001: Previous decision](0001-noun-verb-pattern.md)
- [ADR 0002: Related decision](0002-linkme-distributed-slices.md)

## Changelog

- **2026-06-14** - Proposed
- **2026-06-15** - Accepted

## References

- [Clap documentation](https://docs.rs/clap/)
- [ADR Guide](https://adr.github.io/)

---

**Status:** [ACCEPTED | PROPOSED | SUPERSEDED] as of YYYY-MM-DD
```

### Step-by-Step: Writing an ADR

#### Step 1: Identify the Decision

What architectural choice needs documenting?

Examples:
- Why use `linkme` for auto-discovery?
- Why zero default features?
- Why noun-verb pattern?
- Why JSON output by default?

#### Step 2: Write Context

Describe the problem without proposing a solution:

```markdown
## Context

Users were struggling with large flat command structures. When managing
10+ resource types (users, sessions, configs, roles), commands like:

- `list_users`, `create_user`, `delete_user`
- `list_sessions`, `create_session`, `revoke_session`
- `list_configs`, `update_config`, `validate_config`

...became hard to discover and organize. Users wanted a hierarchical
structure that mirrored their domain models.
```

#### Step 3: State the Decision

Be clear and specific:

```markdown
## Decision

We adopted a noun-verb command pattern where:
- **Nouns** represent resource types or entities (user, session, config)
- **Verbs** represent actions on those nouns (create, list, delete)
- Commands are registered via `#[verb("action")]` macros
- Auto-discovery uses `linkme` distributed slices
```

#### Step 4: Document Consequences

Be honest about trade-offs:

```markdown
## Consequences

### Positive

- **Better organization** - Related commands group naturally
- **Discoverability** - `services --help` lists all service commands
- **Type safety** - Arguments validated at compile time
- **Agent-friendly** - JSON output enables automation

### Negative

- **Learning curve** - New to macros? Need onboarding
- **Compilation** - Proc-macros add ~0.5s overhead
- **Error messages** - Macro errors can be cryptic initially
- **Constraints** - Return types must be Serialize
```

#### Step 5: List Alternatives

Show what else was considered:

```markdown
## Alternatives Considered

### A: Trait-based registration

Users implement `VerbCommand` manually for each command.

**Trade-offs:**
- ✅ No macro overhead
- ✅ Familiar pattern
- ❌ ~15 lines boilerplate per command
- ❌ Manual registration error-prone

### B: Procedural macro without auto-discovery

Use macro but require explicit registration.

**Trade-offs:**
- ✅ Simpler macro implementation
- ❌ Boilerplate for registration
- ❌ Easy to forget a command
```

### ADR Quality Checklist

- [ ] Clear problem statement
- [ ] Decision is concrete and actionable
- [ ] Consequences are specific, not vague
- [ ] Trade-offs honestly documented
- [ ] Alternatives considered with reasoning
- [ ] Links to related ADRs included
- [ ] Appropriate status (Proposed/Accepted)
- [ ] Dated and attributed to deciders
- [ ] 500-1000 words, not too long

---

## Skill: Updating README

**What this covers:** Keeping README.md current with new features and changes.

### Quick Reference

```bash
# Check links in README
cargo make lint

# Verify examples in README still work
cargo build --example tutorial_basic
cargo run --example tutorial_basic -- --help
```

### Sections to Update

#### 1. "What's New" Section

Update with each release:

```markdown
## What's New in 26.7.0

- **New `#[query]` macro** - Simplified database integration
- **Improved error messages** - Better feedback for CLI users
- **Async verb improvements** - Support for `async fn` handlers
- See [CHANGELOG](CHANGELOG.md#2670) for full details
```

**Quality gates:**
- ✅ Mentions all major features
- ✅ Brief one-liner per feature
- ✅ Links to CHANGELOG for details
- ✅ Version number is accurate

#### 2. Installation Section

Keep version numbers and import paths current:

```markdown
## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "26.7.0"
clap-noun-verb-macros = "26.7.0"
```

Or with `cargo add`:

```bash
cargo add clap-noun-verb clap-noun-verb-macros
```
```

**Quality gates:**
- ✅ Version numbers match current release
- ✅ Instructions are copy-paste ready
- ✅ Both macros and main crate included

#### 3. Quick Start Section

Test examples before committing:

```bash
cargo run --example tutorial_basic -- --help
```

Make sure:
- ✅ Example code compiles
- ✅ Example produces meaningful output
- ✅ All commands shown actually work
- ✅ Output matches what's documented

#### 4. Feature Table

Keep synchronized with Cargo.toml:

```markdown
| Feature | Purpose | Default |
|---------|---------|---------|
| `async` | Async command handlers | No |
| `repl` | Interactive REPL mode | No |
| `federated-network` | Node federation | No |
```

**Quality gates:**
- ✅ Lists all cargo features
- ✅ Default column matches Cargo.toml
- ✅ Purpose is clear and concise
- ✅ Alphabetically ordered

#### 5. Contributing Section

Point to detailed guidelines:

```markdown
## Contributing

Issues and PRs welcome at [github.com/seanchatmangpt/clap-noun-verb](...)

For detailed guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).
```

### Workflow: Adding a New Feature to README

```
1. Code change completed + tests pass
2. Update version in Cargo.toml
3. Update CHANGELOG.md
4. Update "What's New" in README
5. Update feature table if applicable
6. Test examples: cargo run --example <name> -- --help
7. Verify all links work
8. Commit with clear message
```

### Common README Updates

#### Adding a Feature

```markdown
# Before
## Features

The core ships with **zero default features** — everything
in the feature matrix above is available with just `clap-noun-verb`.

# After
## Features

The core ships with **zero default features** — everything
in the feature matrix above is available with just `clap-noun-verb`.
With optional features you can enable:
- **Async handlers** (`async` feature)
- **Interactive REPL** (`repl` feature)
- **Query builder** (`query-builder` feature) - NEW!
```

#### Updating Installation Instructions

```markdown
# Before
cargo add clap-noun-verb clap-noun-verb-macros

# After
# For basic CLI
cargo add clap-noun-verb clap-noun-verb-macros

# For async support
cargo add clap-noun-verb clap-noun-verb-macros --features async
```

### Validation Checklist

- [ ] "What's New" updated with all major changes
- [ ] Version numbers are current and consistent
- [ ] Example code compiles and runs
- [ ] All links are functional
- [ ] Feature table matches Cargo.toml
- [ ] Installation instructions work as-is
- [ ] No broken cross-references
- [ ] Quick start example produces expected output
- [ ] Feature descriptions are accurate
- [ ] Contributing section points to guidelines

---

## Quality Assurance Workflows

### Full Documentation Validation

Before committing documentation changes:

```bash
# 1. Test doc comments
cargo test --doc

# 2. Test examples
cargo build --example <name>

# 3. Format check
cargo make format-check

# 4. Lint check
cargo make lint

# 5. Generate and view
cargo make doc
open target/doc/clap_noun_verb/index.html
```

### Pre-Commit Checklist

Documentation commit checklist:

- [ ] Doc tests compile and pass: `cargo test --doc`
- [ ] All examples build: `cargo build --examples`
- [ ] All links are valid (grep for `[` to check)
- [ ] Spelling is correct (review before commit)
- [ ] Examples are realistic (not stubs)
- [ ] Terminology is consistent
- [ ] No unwrap/expect/panic in examples
- [ ] Formatting passes: `cargo make format-check`
- [ ] Linting passes: `cargo make lint`
- [ ] Generated docs look good

### Common Validation Issues

**Issue: Doc test fails to compile**

```bash
error: could not compile
...
error[E0433]: cannot find function `X`
```

Fix: Add missing imports to doc comment.

**Issue: Example has no main function**

```bash
error[E0601]: `main` function not found
```

Fix: Wrap in `# fn main() -> Result<()> { ... # Ok(()) # }`.

**Issue: Unwrap warnings**

```
warning: use of `unwrap` in doc tests
```

Fix: Use `#` prefix to hide error handling setup.

---

## Integration with Development

### When Making a Code Change

```
1. Update src/file.rs (your feature/fix)
   ↓
2. Update doc comments (same file)
   ↓
3. cargo test --doc (verify examples compile)
   ↓
4. If new API/feature:
   ├→ Create docs/howto/feature.md (task-oriented)
   ├→ Create examples/howto/feature.rs (working example)
   ├→ Create docs/reference/api/feature.md (exhaustive)
   └→ Update examples/README.md
   ↓
5. If architectural change:
   └→ Create docs/adr/NNNN-title.md
   ↓
6. Update README.md if user-facing
   ↓
7. cargo make doc (verify all documentation generates)
   ↓
8. Commit with clear message: "docs: [type] [subject]"
```

### Documentation Commit Message Template

```
docs: [type] [subject]

[Body with 1-2 sentences explaining what changed]

Changes:
- Updated [section] to cover [topic]
- Added example in [location]
- Fixed [broken reference]

Relates to: #[issue number]
```

Examples:

```
docs: howto - guide for custom validation

Added comprehensive how-to guide showing how to implement
custom validators with #[arg(validate = ...)].

Changes:
- Step-by-step walkthrough with 3 examples
- Common validation patterns section
- Troubleshooting section
- New example: examples/howto/validation.rs

Closes #456

---

docs: reference - update verb macro to 26.7.0

Updated #[verb] macro reference with v26.7.0 features
and clarifications on error handling.

Changes:
- Added [help = "..."] attribute documentation
- Clarified error variant descriptions
- Added nested command examples
```

### Continuous Integration

Documentation is validated in CI:

```bash
# Run locally before pushing
cargo test --doc           # Doc tests must pass
cargo make format-check    # Formatting
cargo make lint            # All lints pass
cargo make build-examples  # Examples must compile
```

If CI fails:

```bash
# Fix formatting
cargo make format

# Run tests
cargo test --doc

# Rebuild examples
cargo make build-examples

# Re-check
cargo make lint
```

---

## Reference: Command Summary

### Essential Commands

```bash
# Documentation generation
cargo make doc                 # Generate all docs
cargo test --doc              # Test doc examples
cargo test --doc module::     # Test specific module
cargo make format             # Format code + docs
cargo make format-check       # Check formatting

# Examples
cargo make build-examples     # Build all examples
cargo run --example <name>    # Run specific example
cargo build --example <name>  # Build specific example

# Publishing (maintainers)
cargo make publish-macros     # Publish macro crate first
cargo make publish            # Publish main crate

# Validation
cargo make lint               # All quality checks
cargo make verify             # Format, clippy, tests
cargo make ci                 # Full CI suite
```

### Feature-Specific

```bash
# With async feature
cargo test --doc --features async
cargo run --example async_example --features async

# With repl feature
cargo test --doc --features repl
cargo run --example repl_example --features repl

# All features
cargo test --doc --all-features
cargo doc --all-features --no-deps
```

---

## Additional Resources

### Documentation References

- **Full Guide:** [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md)
- **Quick Reference:** [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md)
- **Example Navigation:** [examples/README.md](../examples/README.md)

### Framework References

- **Diataxis Framework:** https://diataxis.fr/
- **Rust Documentation:** https://doc.rust-lang.org/rustdoc/
- **ADR Guide:** https://adr.github.io/
- **Contributing:** [../CONTRIBUTING.md](../CONTRIBUTING.md)

### Project Documentation

- **Main README:** [../README.md](../README.md)
- **CHANGELOG:** [../CHANGELOG.md](../CHANGELOG.md)
- **CLAUDE.md:** [../CLAUDE.md](../CLAUDE.md) — Development guidelines

---

## Glossary

**Diataxis** - Documentation framework with four categories: Tutorial (learning), How-To (problem-solving), Reference (lookup), Explanation (understanding).

**Doc comment** - Rust documentation comment (`///` for items, `//!` for modules).

**Doc test** - Executable code example in a doc comment that's compiled and run as a test.

**ADR** - Architecture Decision Record; documents architectural choices and their trade-offs.

**Rustdoc** - Rust's built-in documentation tool; generates HTML from doc comments.

**Example** - Runnable code file demonstrating a feature or pattern.

**Module doc** - Documentation at the top of a `.rs` file describing what the module does.

---

**Last Updated:** 2026-06-14

**Version:** 1.0

**Status:** Active

See also: [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md) for comprehensive reference.
