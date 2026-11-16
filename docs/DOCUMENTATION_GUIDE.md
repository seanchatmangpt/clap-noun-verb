# Documentation Guide for clap-noun-verb

**Master guide for maintaining high-quality, accurate documentation across the clap-noun-verb project.**

> **New:** See [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md) for task-oriented documentation workflows organized by skill (Rustdoc, comments, examples, ADRs, README).

---

## Overview

This guide helps developers:
1. Generate and publish rustdoc
2. Write effective documentation comments with examples
3. Create guides for new macros and features
4. Update and organize examples
5. Write Architecture Decision Records (ADRs)
6. Maintain the examples/ directory
7. Update README and reference docs

All documentation follows the **[Diataxis Framework](https://diataxis.fr/)** (Tutorial, How-To, Reference, Explanation).

---

## Table of Contents

1. [Rustdoc Generation & Publishing](#rustdoc-generation--publishing)
2. [Documentation Comment Patterns](#documentation-comment-patterns)
3. [Creating Guides for New Features](#creating-guides-for-new-features)
4. [Managing Examples](#managing-examples)
5. [Writing Architecture Decision Records](#writing-architecture-decision-records)
6. [README Updates](#readme-updates)
7. [Quality Checklist](#quality-checklist)

---

## Rustdoc Generation & Publishing

### Generate Documentation Locally

```bash
# Generate docs with all dependencies resolved
cargo make doc

# Open in browser (macOS/Linux)
open target/doc/clap_noun_verb/index.html

# Or with generic command
xdg-open target/doc/clap_noun_verb/index.html  # Linux
start target/doc/clap_noun_verb/index.html    # Windows
```

### Rustdoc Configuration (Cargo.toml)

The project uses standard rustdoc settings:

```toml
[package.metadata.docs.rs]
all-features = false                    # Generate with default features only
default-target = "x86_64-unknown-linux-gnu"
targets = ["x86_64-unknown-linux-gnu"]
```

### Publishing Docs

Docs are automatically published to [docs.rs](https://docs.rs/clap-noun-verb/) on each crate release:

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md with changes
# 3. Commit and tag

git tag v26.9.1
git push origin main --tags

# 4. Publish (via CI/CD or manually)
cargo make publish-macros  # Macros crate first
cargo make publish         # Main crate second
```

### Viewing Published Docs

- **Latest release**: https://docs.rs/clap-noun-verb/
- **Specific version**: https://docs.rs/clap-noun-verb/26.9.1/

### CI/CD Documentation Check

Documentation is validated in CI to ensure:
- ✅ All doc tests compile (`cargo test --doc`)
- ✅ No broken links in doc comments
- ✅ Examples are realistic (not stubs)

See `.github/workflows/` for CI configuration.

---

## Documentation Comment Patterns

### Module-Level Documentation

Place at the **top of each module file**, before any code:

```rust
//! # Module Name
//!
//! Brief one-liner description of what this module does.
//!
//! ## Overview
//!
//! More detailed explanation of the module's purpose and typical usage patterns.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use clap_noun_verb::cli;
//!
//! # fn main() -> clap_noun_verb::Result<()> {
//! let registry = cli::CommandRegistry::new();
//! # Ok(())
//! # }
//! ```
//!
//! ### Advanced Configuration
//!
//! ```rust
//! # use clap_noun_verb::cli::CommandRegistry;
//! # fn main() -> clap_noun_verb::Result<()> {
//! let mut registry = CommandRegistry::new();
//! // Configure registry...
//! # Ok(())
//! # }
//! ```
//!
//! ## See Also
//!
//! - [`Module::function`] - For specific operation
//! - `other_module` - Related functionality
```

### Type Documentation

```rust
/// Brief one-liner summary of the type.
///
/// Detailed explanation of what this type represents and when to use it.
///
/// # Type Parameters
///
/// * `T` - The inner value type
///
/// # Examples
///
/// ```rust
/// # use clap_noun_verb::Result;
/// let value: Result<i32> = Ok(42);
/// ```
///
/// # See Also
///
/// - [`OtherType`] - Related type
pub struct MyType<T> {
    /// Field documentation
    pub field: T,
}
```

### Function Documentation

```rust
/// Brief summary of what the function does.
///
/// More detailed explanation covering:
/// - Purpose and use cases
/// - Preconditions/requirements
/// - Typical return values
///
/// # Arguments
///
/// * `name` - What this argument does
/// * `options` - Configuration options
///
/// # Returns
///
/// Describes what the function returns and what each variant means.
///
/// # Errors
///
/// Returns `Err` when:
/// - Invalid input is provided
/// - Operation fails (with example)
///
/// # Examples
///
/// ## Success Case
///
/// ```rust
/// # use clap_noun_verb::builder::CliBuilder;
/// let builder = CliBuilder::new();
/// let result = builder.build();
/// assert!(result.is_ok());
/// ```
///
/// ## Error Case
///
/// ```rust
/// # use clap_noun_verb::builder::CliBuilder;
/// let builder = CliBuilder::new();
/// // Configure with invalid state...
/// let result = builder.build();
/// assert!(result.is_err());
/// ```
///
/// # Performance
///
/// - Time complexity: O(n) where n is number of commands
/// - Space complexity: O(m) where m is total argument count
///
/// # See Also
///
/// - [`related_function`] - For related operation
pub fn my_function(name: &str, options: Config) -> Result<Output> {
    // Implementation
    Ok(Output::default())
}
```

### Trait Documentation

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
/// ```rust,ignore
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

### Macro Documentation

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
/// - `help = "..."` (optional) - Help text shown by `--help`
///
/// # Return Type Requirements
///
/// The return type MUST implement `Serialize + Send + 'static`:
/// - ✅ Structs with `#[derive(Serialize)]`
/// - ✅ `Result<T>` where `T: Serialize`
/// - ❌ Non-serializable types
/// - ❌ Async functions (use `async_verb` feature instead)
///
/// # Argument Type Support
///
/// - Primitives: `bool`, `i32`, `u64`, `f64`, `String`
/// - Collections: `Vec<T>`, `Option<T>`
/// - Custom: Any type with `FromStr`
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
/// ## With Flags
///
/// ```ignore
/// #[verb("deploy")]
/// fn cmd_deploy(
///     #[arg(long)]
///     environment: String,
///     
///     #[arg(long)]
///     dry_run: bool,
/// ) -> Result<DeployResult> {
///     // Implementation
/// }
/// ```
///
/// ## Nested Under Noun
///
/// ```ignore
/// #[verb("status", noun = "services")]
/// fn services_status() -> Result<Status> {
///     // Implementation
/// }
/// ```
///
/// # Compile-Time Validation
///
/// The macro validates at compile time:
/// - ✅ Return type is `Serialize`
/// - ✅ All arguments are supported types
/// - ✅ No duplicate command names
/// - ✅ Argument count within limits
///
/// # Errors
///
/// If validation fails, you'll see compiler errors like:
/// - `"return type must implement Serialize"`
/// - `"duplicate command name: 'greet'"`
/// - `"unsupported argument type"`
///
/// # See Also
///
/// - [`#[noun]`] - For noun subcommands
/// - [`#[arg]`] - For argument attributes
pub use clap_noun_verb_macros::verb;
```

### Enum Documentation

```rust
/// Possible outcomes of a CLI operation.
///
/// # Variants
///
/// * `Success(T)` - Operation succeeded with value T
/// * `PartialFailure { .. }` - Operation partially succeeded
/// * `Failure(Error)` - Operation completely failed
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

**Write realistic examples that compile and run:**

```rust
/// # Good: Complete, realistic example
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

/// # Bad: Incomplete, stub example
/// ```rust
/// # This won't compile as-is!
/// let builder = CliBuilder::new();
/// let command = builder.build();
/// ```

/// # Bad: Tautological assertion
/// ```rust
/// # This teaches nothing:
/// assert!(builder.build().is_ok());
/// ```
```

**Using ignore, text, and should_panic:**

```rust
/// For unsupported behavior:
/// ```text
/// This example demonstrates async handlers which require the `async` feature.
/// See examples/advanced/async.rs for full async support.
/// ```

/// For examples that deliberately panic:
/// ```rust,should_panic
/// #[verb("dangerous")]
/// fn dangerous_cmd() -> Result<()> {
///     panic!("This is expected to fail!");
/// }
/// ```

/// For pseudo-code or incomplete examples:
/// ```ignore
/// #[verb("pseudo_code")]
/// fn pseudo(data: impl Iterator<Item = String>) -> Result<Summary> {
///     // Implementation omitted for brevity
/// }
/// ```
```

---

## Creating Guides for New Macros and Features

### Step 1: Write the Macro Documentation

In the macro crate (`clap-noun-verb-macros/src/lib.rs`):

```rust
/// Register a new command with special metadata handling.
///
/// [See full macro documentation above]
pub use clap_noun_verb_macros::my_new_macro;
```

### Step 2: Create a How-To Guide

File: `docs/howto/my-feature.md`

```markdown
# How to Use [Feature Name]

**For:** Developers wanting to [specific goal]

## Quick Start

[5-10 lines showing the minimal working example]

## Detailed Walkthrough

### Step 1: [Requirement]

[Explanation and code snippet]

### Step 2: [Configuration]

[Explanation and code snippet]

### Step 3: [Validation]

[Explanation and code snippet]

## Common Patterns

### Pattern A: [Use case]

[Code example]

### Pattern B: [Use case]

[Code example]

## Troubleshooting

### Problem: [Common issue]

**Symptom:** [What the user sees]

**Solution:** [How to fix it]

**Why:** [Brief explanation]

## See Also

- [Related feature]
- [Related example]
- [Related documentation]
```

### Step 3: Create Reference Documentation

File: `docs/reference/api/my-macro.md`

```markdown
# #[my_macro] Reference

Complete reference for the `#[my_macro]` attribute macro.

## Syntax

```
#[my_macro("name", key = value)]
```

## Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| name | string | Yes | The command/feature name |
| key | string | No | Configuration key |

## Examples

[Minimal examples showing usage]

## See Also

- [Related macro](other-macro.md)
```

### Step 4: Create Example Implementation

File: `examples/howto/my-feature.rs`

```rust
//! How-to example: [Feature Name]
//!
//! This example demonstrates how to use the #[my_macro] attribute
//! for [specific purpose].
//!
//! Run with:
//! ```sh
//! cargo run --example howto_my_feature -- [ARGS]
//! ```

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct MyOutput {
    message: String,
}

#[verb("my-command")]
fn cmd_example(input: String) -> Result<MyOutput> {
    Ok(MyOutput {
        message: format!("Processed: {}", input),
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

### Step 5: Update Examples README

In `examples/README.md`, add to the relevant section:

```markdown
## 🔧 How-To Examples (Task-Oriented)

| Example | Command | Purpose |
|---------|---------|---------|
| ... | ... | ... |
| **my-feature** | `cargo run --example howto_my_feature` | How to use the new feature |
```

### Step 6: Update Main Reference

In `docs/reference/README.md`, add to the reference table:

```markdown
| **#[my_macro] macro** | [api/my-macro.md](api/my-macro.md) |
```

---

## Managing Examples

### Directory Structure

```
examples/
├── README.md                          # Navigation guide (Diataxis-organized)
├── tutorial/                          # Learning-oriented examples
│   ├── basic.rs                       # 5-minute "hello world"
│   ├── arguments.rs                   # Adding typed arguments
│   ├── positional.rs                  # Positional vs named
│   └── services.rs                    # Multi-noun structure
├── howto/                             # Task-oriented examples
│   ├── arg_groups.rs                  # Mutually exclusive args
│   ├── validation.rs                  # Custom validation
│   ├── env_vars.rs                    # Environment integration
│   └── deprecation.rs                 # Marking deprecated commands
├── reference/                         # Information-oriented examples
│   ├── attribute_macro.rs             # Complete #[verb] syntax
│   ├── framework.rs                   # Full integration
│   ├── nested.rs                      # Multi-level nesting
│   ├── context.rs                     # AppContext usage
│   └── collector.rs                   # Command collection
└── greet-demo/                        # Standalone project
    ├── Cargo.toml
    ├── src/
    ├── ontology.ttl
    └── ggen.toml
```

### Creating a New Example

#### 1. Choose Category

- **Tutorial** - If it's learning-focused (for beginners)
- **How-To** - If it solves a specific problem
- **Reference** - If it demonstrates exhaustive API usage

#### 2. Write the Example

```rust
//! Tutorial: [Topic]
//!
//! This example teaches [learning objective].
//!
//! ## Learning Goals
//!
//! By the end, you'll understand:
//! - Goal 1
//! - Goal 2
//!
//! Run with:
//! ```sh
//! cargo run --example tutorial_topic -- [ARGS]
//! ```

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use serde::Serialize;

// Step 1: Define output type
#[derive(Serialize)]
pub struct Output {
    // Fields
}

// Step 2: Define handler
#[verb("command")]
fn cmd_handler() -> Result<Output> {
    // Implementation
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

#### 3. Test the Example

```bash
# Build
cargo build --example tutorial_topic

# Run
cargo run --example tutorial_topic -- arg1 arg2

# Test with help
cargo run --example tutorial_topic -- --help
```

#### 4. Add to examples/README.md

```markdown
| tutorial/topic.rs | 5-minute intro to [topic] | Learning [concept] |
```

### Example Quality Checklist

- ✅ Has doc comment with learning goals
- ✅ Runnable without modification
- ✅ Accepts realistic input
- ✅ Produces meaningful output
- ✅ Shows common patterns, not edge cases
- ✅ Includes `--help` output in comment
- ✅ Compiles with `cargo build --example <name>`
- ✅ Works with `cargo run --example <name> --`

### Testing Examples

```bash
# Ensure all examples compile
cargo make build-examples

# Run a specific example
cargo run --example tutorial_basic -- help

# Test example with different features
cargo run --example tutorial_basic --features repl -- help
```

---

## Writing Architecture Decision Records

### What is an ADR?

An **Architecture Decision Record (ADR)** documents:
- ❓ **Context** - What problem were we facing?
- 🤔 **Decision** - What did we choose?
- ✅ **Consequences** - What are the trade-offs?
- 📚 **Alternatives** - What else did we consider?

### ADR File Location and Naming

File: `docs/adr/NNNN-short-title.md`

Examples:
- `docs/adr/0001-noun-verb-pattern.md`
- `docs/adr/0002-linkme-distributed-slices.md`
- `docs/adr/0003-zero-default-features.md`

### ADR Template

```markdown
# ADR NNNN: [Decision Title]

**Status:** Accepted | Superseded | Proposed

**Date:** 2026-06-14

**Deciders:** [Names of decision makers]

## Context

Describe the issue or challenge that prompted this decision.

- What problem were we trying to solve?
- What constraints did we have?
- What was the current state before this decision?

Example:
> Users found flat command structures (e.g., `login`, `logout`, `verify`)
> confusing when managing multiple resource types. They wanted a way to
> organize commands hierarchically around nouns (entities) and verbs (actions).

## Decision

State the decision clearly and concisely.

Example:
> We will adopt a noun-verb command pattern where:
> - Nouns represent resource types or entities (e.g., session, user, config)
> - Verbs represent actions on those nouns (e.g., create, list, delete)
> - Commands are registered via `#[verb]` macros with compile-time auto-discovery

## Rationale

Explain why this decision makes sense given the context.

- How does it solve the problem?
- What are the benefits?
- Why is this better than alternatives?

Example:
> This pattern:
> - **Mirrors domain models** - Naturally maps to business entities and operations
> - **Improves discoverability** - `services --help` shows all service commands
> - **Enables auto-discovery** - The `#[verb]` macro + `linkme` eliminates manual routing
> - **Scales well** - Works equally for 5 or 500 commands

## Consequences

Describe the benefits and drawbacks of this decision.

### Positive

- **Consequence 1** - Brief explanation
- **Consequence 2** - Brief explanation

Example:
- **Better UX** - Commands are naturally grouped and documented
- **Type safety** - Arguments are validated at compile time
- **Agent-friendly** - JSON output and structured args enable automation

### Negative

- **Consequence 1** - Brief explanation
- **Consequence 2** - Brief explanation

Example:
- **Macro overhead** - Requires proc-macro crate (0.5s compile time)
- **Learning curve** - Developers new to macros need onboarding
- **Error messages** - Proc-macro errors can be cryptic

### Neutral

- **Consequence** - Neither benefit nor drawback

Example:
- Requires Rust 1.70+ (minimum supported version anyway)

## Alternatives Considered

### Alternative A: [Name]

**Approach:** [Brief description]

**Trade-offs:**
- ✅ Benefit
- ❌ Drawback

Example:
### Alternative A: Trait-based registration

**Approach:** Users manually implement `VerbCommand` trait for each command

**Trade-offs:**
- ✅ No compile-time macro overhead
- ❌ Boilerplate for each command (vs. one `#[verb]` annotation)
- ❌ No auto-discovery (manual registration required)

### Alternative B: [Name]

[Similar format]

## Related Decisions

- [ADR NNNN: Previous decision](NNNN-title.md)
- [ADR NNNN: Related decision](NNNN-title.md)

Example:
- [ADR 0002: Linkme distributed slices](0002-linkme-distributed-slices.md) - Enables auto-discovery
- [ADR 0003: Zero default features](0003-zero-default-features.md) - Keeps core slim

## Changelog

- **2026-06-14** - Decision accepted
- **2026-06-15** - ADR documented

## References

- [Clap documentation](https://docs.rs/clap/)
- [Diataxis framework](https://diataxis.fr/)

---

**Status:** This decision is **[ACCEPTED | PROPOSED]** as of [date].
```

### Example ADR

```markdown
# ADR 0003: Minimalist Zero-Default-Features Architecture

**Status:** Accepted

**Date:** 2026-06-13

**Deciders:** Core team

## Context

clap-noun-verb was shipping with ~50 dependencies by default, many unused
by typical users. This bloated compile times and the dependency tree.

Users wanted a choice: compile with just the essentials, or opt into
advanced features like async, RDF, or agent integration.

## Decision

Redesign to **zero default features**:
- Core (always available): 10 dependencies for basic CLI
- Optional: `async`, `federated-network`, `repl`, etc.
- Users only pay for what they use

## Rationale

- **Fast iteration** - New developers compile faster
- **Minimal footprint** - CLI tools ship with <3MB binary
- **Flexibility** - Advanced users get their features; simple users don't wait
- **Clear separation** - Dependencies are obviously optional or required

## Consequences

✅ **Positive:**
- Incremental compile: 0.66s (was 2.4s)
- Binary size: 2.2MB (was 8.1MB)
- Only 10 core dependencies

❌ **Negative:**
- Users must enable features explicitly
- Slightly more complex Cargo.toml
- Some examples only work with specific features

## Alternatives Considered

### Alternative A: Keep all features by default

**Trade-offs:**
- ✅ Works out-of-box
- ❌ Slow compilation
- ❌ Bloated binaries

### Alternative B: Single "full" feature with all optional modules

**Trade-offs:**
- ✅ Still separable from core
- ❌ Still doesn't help incremental compilation
- ❌ Users must know about it

## Related Decisions

- [ADR 0001: Noun-verb pattern](0001-noun-verb-pattern.md)
- [ADR 0002: Linkme distributed slices](0002-linkme-distributed-slices.md)

---

**Status:** ACCEPTED as of 2026-06-13
```

---

## README Updates

### Main README (README.md)

Update these sections when appropriate:

#### 1. **"What's New" Section**

Keep this updated with each release:

```markdown
## What's New in 26.9.1

- **Feature 1** - Brief one-liner
- **Feature 2** - Brief one-liner
- See [CHANGELOG](CHANGELOG.md#26613) for full details
```

#### 2. **Installation Section**

Keep Cargo.toml examples current:

```markdown
## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "26.9.1"
clap-noun-verb-macros = "26.9.1"
```
```

#### 3. **Quick Start Section**

Keep examples working. Test with:

```bash
cargo run --example tutorial_basic
```

#### 4. **Feature Table**

Keep synchronized with Cargo.toml:

```markdown
| Feature | Purpose | Default |
|---------|---------|---------|
| `async` | Async command handlers | No |
| `repl` | Interactive REPL | No |
| `federated-network` | Node federation | No |
```

### Updating README for New Features

When adding a new feature:

1. **Update "What's New"** with a brief description
2. **Add a feature table row** with purpose and default
3. **Add a quick start example** if the feature is primary
4. **Link to full documentation** in `docs/howto/`

Example:

```markdown
## What's New in 26.9.1

- **New `#[query]` macro** - Simplified database query integration
- **Improved error messages** - Better feedback for argument validation
- See [CHANGELOG](CHANGELOG.md#2670) for full details

## Features

| Feature | Purpose | Default |
|---------|---------|---------|
| ... | ... | ... |
| `query-builder` | Database query DSL | No |

## Using the Query Builder

See [How-To: Database Queries](docs/howto/database-queries.md) for examples.
```

### Updating Feature Documentation

When features change, also update:

1. **`docs/reference/README.md`** - Update API table
2. **`Cargo.toml`** - Update feature list
3. **`CHANGELOG.md`** - Document the change
4. **`CLAUDE.md`** - Update if it affects development

---

## Quality Checklist

Use this checklist before publishing documentation:

### ✅ All Documentation

- [ ] Follows Diataxis framework (Tutorial/How-To/Reference/Explanation)
- [ ] No grammatical errors (spell-check with `cargo make format-check`)
- [ ] Links are accurate and not broken
- [ ] Uses consistent terminology across docs
- [ ] Includes code examples
- [ ] Examples compile and run successfully
- [ ] Clear headings and sections
- [ ] Appropriate tone for audience (learning/problem-solving/reference)

### ✅ Rustdoc (Doc Comments)

- [ ] Module has `//!` documentation at top
- [ ] All public items have `///` documentation
- [ ] Includes `# Examples` with realistic, compilable code
- [ ] All examples compile without modification
- [ ] Links to related items use `[`Item`]` syntax
- [ ] Uses `# Errors` for functions that return `Result`
- [ ] Uses `# Panics` if function can panic
- [ ] Uses `# See Also` for related functionality
- [ ] No `unwrap()`, `expect()`, or `panic!()` in doc test examples

### ✅ How-To Guides (docs/howto/)

- [ ] Focused on solving ONE specific problem
- [ ] Step-by-step walkthrough (not comprehensive reference)
- [ ] Real-world code examples that run
- [ ] Includes common mistakes/troubleshooting
- [ ] Approximately 800-2000 words
- [ ] Links to reference docs for deep dives
- [ ] Clear prerequisites and requirements

### ✅ Reference Documentation (docs/reference/)

- [ ] Accurate API signatures (cross-checked with source)
- [ ] Complete parameter lists
- [ ] Clear return type descriptions
- [ ] Minimal, focused examples
- [ ] No explanatory prose (use explanation/ for that)
- [ ] Links to related APIs
- [ ] Updated when API changes
- [ ] Approximately 500-1500 words

### ✅ Tutorial Documentation (docs/tutorial/)

- [ ] Assumes no prior knowledge of the framework
- [ ] Teaches concepts progressively
- [ ] Each chapter is ~1500-3000 words
- [ ] Includes "Learning Goals" section
- [ ] Practical, hands-on exercises
- [ ] Solutions provided
- [ ] Builds toward a complete working example

### ✅ Explanation Documentation (docs/explanation/)

- [ ] Provides context and background
- [ ] Explains design decisions
- [ ] Discusses trade-offs
- [ ] No step-by-step instructions (that's How-To)
- [ ] Philosophical or conceptual tone
- [ ] Connects multiple concepts together
- [ ] References related documentation

### ✅ Examples (examples/)

- [ ] Organized by Diataxis category
- [ ] Has module doc comment with learning goals
- [ ] Runs without modification: `cargo run --example <name>`
- [ ] Produces meaningful output
- [ ] Shows common patterns, not edge cases
- [ ] Compiles with `cargo build --example <name>`
- [ ] Listed in `examples/README.md`
- [ ] Works with feature flags as documented
- [ ] Output matches expected behavior

### ✅ Architecture Decision Records (docs/adr/)

- [ ] Clearly states the decision
- [ ] Explains context and problem being solved
- [ ] Documents positive and negative consequences
- [ ] Lists alternatives considered
- [ ] Links to related decisions
- [ ] Has status (Proposed/Accepted/Superseded)
- [ ] Dated and attributed to deciders
- [ ] Approximately 500-1000 words

### ✅ README Updates

- [ ] "What's New" mentions all major features
- [ ] Version numbers are current
- [ ] Example code is copy-paste ready
- [ ] All links work
- [ ] Feature table matches Cargo.toml
- [ ] Matches actual project structure
- [ ] Installation instructions are accurate
- [ ] License information is correct

---

## Documentation Workflow

### When Making a Code Change

```
1. Update code (in src/)
   ↓
2. Update rustdoc comments (in same file)
   ↓
3. Test: cargo test --doc
   ↓
4. If it's a new API/feature:
   ├→ Create How-To guide (docs/howto/)
   ├→ Create reference doc (docs/reference/api/)
   └→ Create example (examples/)
   ↓
5. If it's an architectural change:
   └→ Create ADR (docs/adr/)
   ↓
6. Update README.md if user-facing
   ↓
7. Run cargo make doc to verify
   ↓
8. Commit with clear message
```

### Documentation Commit Message Template

```
docs: [Type] [Subject]

[Body explaining the documentation changes]

- Updated [section] to cover [topic]
- Added example in [location]
- Fixed broken links in [file]

See also: [Related PRs/Issues]
```

Examples:

```
docs: howto - add guide for custom validation

Added comprehensive how-to guide for implementing custom argument
validators with the #[arg(validate = ...)] attribute.

- Step-by-step walkthrough with 3 examples
- Common validation patterns
- Troubleshooting section
- Added example: examples/howto/custom_validation.rs

Fixes #456

---

docs: reference - update verb macro to 26.9.1

Updated #[verb] macro reference to reflect latest attributes
and error handling behavior.

- Added `[help = ...]` attribute
- Clarified error variant descriptions
- Added example of nested commands under noun
```

### CI/CD Validation

Documentation is automatically validated in CI:

```bash
# Run locally before committing
cargo test --doc                 # Doc tests must compile and pass
cargo make format-check          # Documentation must be formatted
cargo make lint                  # Linting includes doc comments
```

---

## Tooling and Commands

### Essential Commands

```bash
# Generate documentation
cargo make doc

# Generate and view
cargo make doc && open target/doc/clap_noun_verb/index.html

# Test all doc examples
cargo test --doc

# Test doc examples in specific module
cargo test --doc module::

# Check doc tests compile but don't run
cargo test --doc --no-run

# Run specific example
cargo run --example tutorial_basic --

# Build all examples
cargo make build-examples

# Format code (including doc comments)
cargo make format

# Check formatting
cargo make format-check

# Lint documentation
cargo make lint
```

### Rust-Specific Tooling

```bash
# Generate docs with specific features
cargo doc --features async,repl --no-deps

# Open docs for a dependency
cargo doc --open  # Then search in browser

# Check doc coverage
cargo +nightly doc --no-deps 2>&1 | grep "missing docs"
```

---

## Documentation Templates

### Macro Documentation Template

```rust
/// Brief one-liner about what the macro does.
///
/// [Detailed explanation and use cases]
///
/// # Syntax
///
/// ```
/// #[macro_name(required_arg, optional = value)]
/// ```
///
/// # Parameters
///
/// - `required_arg` - [What it does]
/// - `optional` - [What it does, default: value]
///
/// # Return Type
///
/// [What the macro transforms the item into]
///
/// # Examples
///
/// [Examples showing typical usage]
///
/// # Compile-Time Validation
///
/// [What the macro validates]
///
/// # See Also
///
/// - [Related macro](link)
/// - [Related trait](link)
pub use clap_noun_verb_macros::macro_name;
```

### Error Type Documentation Template

```rust
/// Error type for [operation/module].
///
/// Represents failures that can occur when [operation description].
///
/// # Variants
///
/// - `InvalidInput` - When input doesn't match expected format
/// - `NotFound` - When requested resource doesn't exist
/// - `PermissionDenied` - When user lacks necessary permissions
///
/// # Example
///
/// ```rust
/// # fn operation() -> clap_noun_verb::Result<()> {
/// # Err(clap_noun_verb::NounVerbError::Custom("example".to_string()))?
/// # Ok(())
/// # }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

---

## Common Mistakes to Avoid

❌ **Don't:**
- Write examples that don't compile
- Use `assert!(result.is_ok())` as the only assertion
- Include `unwrap()`, `expect()` in doc examples (use `# Errors`)
- Write documentation for private APIs
- Create examples that require specific features without mentioning them
- Copy-paste documentation without understanding the concept
- Leave broken links in documentation
- Mix Diataxis categories (don't put how-to info in reference)
- Use inconsistent terminology across docs

✅ **Do:**
- Test all examples with `cargo test --doc`
- Write realistic, meaningful examples
- Document error cases and edge conditions
- Keep reference docs concise and accurate
- Update docs when APIs change
- Link between related documentation sections
- Use consistent terminology glossary (define terms once)
- Review for clarity and accuracy before committing
- Include "See Also" sections linking to related docs

---

## Getting Help

### When Documenting...

- **A macro** - See [Macro Documentation Template](#macro-documentation-template)
- **An error** - See [Error Type Template](#error-type-documentation-template)
- **A new feature** - See [Creating Guides for New Features](#creating-guides-for-new-features)
- **An architectural change** - See [Writing ADRs](#writing-architecture-decision-records)
- **A how-to guide** - Study [`docs/howto/arg_groups.md`](../howto/arg_groups.md)
- **A reference page** - Study [`docs/reference/api/verb-macro.md`](../reference/api/verb-macro.md)

### Quick Reference

| Need | Location |
|------|----------|
| API reference | `docs/reference/api/*.md` |
| How-to guides | `docs/howto/*.md` |
| Tutorials | `docs/tutorial/*.md` |
| Architecture | `docs/explanation/*.md` |
| ADRs | `docs/adr/*.md` |
| Code examples | `examples/` |
| Proc-macros | `clap-noun-verb-macros/src/` |
| Main lib | `src/lib.rs` |

---

## Version Compatibility

This documentation guide applies to **clap-noun-verb 26.9.1+**.

For older versions, documentation patterns may differ. Always consult the CLAUDE.md
for version-specific guidance.

---

**Last Updated:** 2026-08-20

See also: [CONTRIBUTING.md](../../CONTRIBUTING.md) for code contribution guidelines.
