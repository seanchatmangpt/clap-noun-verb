# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**clap-noun-verb** is a Rust CLI framework built on top of `clap`, providing noun-verb command patterns (e.g., `myapp services status`). It uses proc-macros (`#[noun]`, `#[verb]`) for declarative command registration with `linkme` distributed slices for compile-time auto-discovery. Version 5.6.0.

## Build Commands

**Always use `cargo make`, never direct `cargo` commands.**

| Task | Command |
|------|---------|
| Format check | `cargo make format-check` |
| Format | `cargo make format` |
| Clippy | `cargo make clippy` |
| Lint (all) | `cargo make lint` |
| Test (quick) | `cargo make test` |
| Test (single-threaded) | `cargo make test-lib-deterministic` |
| Test (all features) | `cargo make test-all` |
| Test (frontier features) | `cargo make test-frontier` |
| Check | `cargo make check` |
| Check (all features) | `cargo make check-all` |
| Build | `cargo make build` |
| Build (release) | `cargo make build-release` |
| Build examples | `cargo make build-examples` |
| Doc | `cargo make doc` |
| CI (full) | `cargo make ci` |
| Benchmarks | `cargo make bench` |

**Single test**: `cargo test test_name --quiet` (only use direct `cargo test` for single test runs).

## Crate Structure

Two crates in a workspace:

- **`clap-noun-verb`** (`src/`) — Main library crate. All core modules, optional feature-gated modules, examples, tests.
- **`clap-noun-verb-macros`** (`clap-noun-verb-macros/src/`) — Proc-macro crate providing `#[noun]`, `#[verb]`, `#[arg]`, `#[meta_aware]`, `#[federated]`, `#[spec]`, `#[semantic_composable]`, `#[competency]`, `#[assessment]`, `#[auto_test]`, and more. Published first before the main crate.

## Architecture

### Core Flow
1. `#[verb]` macro on a function generates a `linkme::distributed_slice` entry
2. At startup, `CommandRegistry` collects all registered verbs via the distributed slice
3. `CliBuilder` constructs the clap `Command` tree from the registry
4. `CommandRouter` dispatches parsed args to the registered handler

### Key Modules (`src/`)
- **`cli/`** — Entry point (`run()`), `CommandRegistry` (noun/verb registration), `ArgMetadata`
- **`builder.rs`** — `CliBuilder` API for constructing CLIs
- **`router.rs`** — `CommandRouter` for dispatching commands
- **`logic/`** — `HandlerInput`/`HandlerOutput` types bridging CLI to domain
- **`error.rs`** — `NounVerbError` and `Result<T>` type
- **`format.rs`** — Output formatting (JSON by default, agent-ready)
- **`noun.rs`** / **`verb.rs`** — Trait definitions (`NounCommand`, `VerbCommand`)
- **`registry.rs`** — `CommandRegistry` for noun/verb registration
- **`tree.rs`** — `CommandTree` for hierarchical command structure

### Feature-Gated Modules
- `async` → `async_verb.rs`, `io` → `io/`, `crypto` → hashing, `autonomic` → `autonomic/`, `kernel` → `kernel/`, `rdf` → `rdf/`, `semantic/`, `ggen_integration/`, `agent2028` → `agent2028/`, `agents/`, `wizard/`, `full` → `plugin/`, `middleware/`, `integration/`, `plugins/`

### Macro Crate (`clap-noun-verb-macros/src/`)
- **`lib.rs`** — `#[noun]` (deprecated no-op), `#[verb]` (main macro), `#[arg]` (parameter attributes)
- **`validation.rs`** — Compile-time validation (return type Serialize, duplicate detection, complexity checks)
- **`io_detection.rs`** — Auto-detection of `clio::Input`/`clio::Output` types
- **`macros/`** — Frontier feature macros (fractal patterns, federated network, semantic composition, executable specs, learning trajectories, reflexive testing)

### Feature System
- **Default**: No features (10 core dependencies only)
- **`full`**: All optional modules
- **Frontier features** (v5.4+): `meta-framework`, `rdf-composition`, `executable-specs`, `fractal-patterns`, `discovery-engine`, `federated-network`, `learning-trajectories`, `reflexive-testing`, `economic-sim`, `quantum-ready`
- **Meta-features**: `frontier-semantic`, `frontier-intelligence`, `frontier-quality`, `frontier-all`
- **`wizard`**: AI integration with rust-genai for multi-provider LLM support

## Critical Rules

### Error Handling
- **NEVER** use `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production code
- Clippy denies these via `lints.clippy`: `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`, `exit`
- Always use `Result<T>` with `?` operator or `map_err()`

### Trait Design
- Keep traits `dyn` compatible — no async methods in traits
- Use `&'static str` for trait method returns
- Use sync methods; async is handled via the `async` feature module

### Logging
- Library code: use `log!` macros (`log::error!`, `log::warn!`, `log::info!`, `log::debug!`)
- **NEVER** use `print!`/`println!` in library code (only in `src/bin/`, `build.rs`, and test code)

### Testing
- Follow AAA pattern (Arrange, Act, Assert)
- Test **behaviors** (observable outputs/state changes), not implementation details
- No tests that only check `assert!(result.is_ok())` — verify actual behavior
- Use descriptive test names: `test_verb_command_executes_successfully_with_required_args`
- Entire test suite must complete in <1 second with parallel execution

### Git
- **NEVER rebase** — only merge
- **NEVER** use `git reset --hard` — fix forward only
- **NEVER** use `--no-verify` — hooks are mandatory quality gates
- Branch prefixes: `claude/*`, `feat/*`, `fix/*`, `refactor/*`

## Formatting

- `rustfmt.toml`: `max_width = 100`, `tab_spaces = 4`, `use_small_heuristics = "Max"`
- `deny.toml`: Permissive licenses only (MIT, Apache-2.0, BSD, ISC). No copyleft (AGPL, GPL, LGPL denied)

## Publishing

Macros crate must be published before main crate:
```
cargo make publish-macros
cargo make publish
```

## SLOs

- Incremental compilation: <=2s (currently 0.66s)
- Binary size: <=10MB (currently 2.2MB)
