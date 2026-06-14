# Doc Coverage Log — Combinatorial Maximalist Doc Loop

**Project**: clap-noun-verb v26.6.14  
**Example runner**: `cargo run --example <name>`  
**Doc surface scope**: rustdoc (`///`/`//!` on pub items in `src/`), README, `docs/` Diataxis set  

---

## Iteration 1 — 2026-06-14

**Commit at start**: `950a8ba` (docs: archive research artifacts)  
**Tree state**: clean except 2 untracked files (`benches/dispatch.rs`, `src/otel.rs`)  
**Toolchain**: rustc 1.97.0-nightly, cargo 1.97.0-nightly

### Coverage Map (start of iteration 1)

#### Exercised-but-not-witnessing (examples that don't import `clap_noun_verb`)

| Example | clap_noun_verb imports | Verdict |
|---------|----------------------|---------|
| `revops_revenue_dashboard` | 0 | Standalone Rust — does NOT witness the framework |
| `revops_sales_pipeline` | 0 | Standalone Rust — does NOT witness the framework |
| `revops_financial_forecast` | 0 | Standalone Rust — does NOT witness the framework |
| `revops_email_sequences` | 0 | Standalone Rust — does NOT witness the framework |
| `revops_cs_checkins` | 0 | Standalone Rust — does NOT witness the framework |
| `semantic_coordinator` | 0 | Standalone Rust — does NOT witness the framework |
| `frontier_discovery_engine_demo` | 0 | Standalone Rust — does NOT witness the framework |
| `frontier_reflexive_testing_demo` | 0 | Standalone Rust — does NOT witness the framework |
| `agent_cli_builder` | 0 | Empty `fn main() {}` — witnesses nothing |
| `ontology_to_cli` | 2 (import string only) | Builds own types; does NOT call framework public API |

**Finding**: 10 of 10 pre-existing examples provide zero coverage of the documented public API surface.

#### Documented-but-unexercised (public API with no running example)

Every public re-export had no running example at iteration start:

| Symbol | Source | Gap |
|--------|--------|-----|
| `CliBuilder` | `src/builder.rs` | No example constructed and ran it |
| `run_cli_with_args` / `run_cli` | `src/builder.rs` | No example called it |
| `build_cli` | `src/builder.rs` | No example called it |
| `noun!` / `verb!` | `src/macros/mod.rs` | No example used them |
| `NounVerbError` (all constructors) | `src/error.rs` | No example constructed any variant |
| `StructuredError` | `src/error.rs` | No example serialized one |
| `VerbArgs::get_one_str` / `get_one` / `get_one_str_opt` | `src/verb.rs` | No example exercised argument access |
| `VerbArgs::trailing()` | `src/verb.rs` | No example used trailing positionals |
| `VerbContext` | `src/verb.rs` | No example built or read context |
| `CommandTree` / `CommandTreeBuilder` | `src/tree.rs` | No example |
| `AppContext` | `src/context.rs` | No example |
| `Deprecation` | `src/deprecation.rs` | No example |
| `OutputFormat` / `format_output` | `src/format.rs` | No example |
| `Graph` / `Triple` / `GraphLoadedOutput` | `src/graph/` | No example |
| `CapabilityRegistry` / `CapabilityPackage` | `src/capability/` | No example |
| `DoctorOutput` / `HealthIssue` | `src/diagnostics/` | No example |
| `Repl` | `src/repl.rs` | No example |

### Triples Closed This Iteration (max 3 per iteration)

#### Triple 1: Core API — `CliBuilder` + `noun!`/`verb!` + `run_cli_with_args` + `build_cli`

**Example**: `examples/core_api.rs`  
**Run**: `cargo run --example core_api`  
**Exit code**: 0  
**Captured output**:
```
running=true uptime=3600
[services status] running=true uptime=3600
key="debug" value="false"
[config get] key="debug" value="false"
command_not_found: Command 'usr' not found. Did you mean: user?
structured_deadline: kind=DeadlineExceeded severity=Critical
structure: services -> ["status", "restart"]
```
**What it witnesses**: Two-noun CLI built with `CliBuilder` + `noun!`/`verb!`, dispatched via `run_cli_with_args` with args injected inline (no stdin). `build_cli` introspects structure. `NounVerbError::command_not_found_with_candidates` and `StructuredError::deadline_exceeded` asserted against real output.  
**Would fail if**: dispatch broke, noun/verb registration failed, or suggestion algorithm returned wrong candidates.

#### Triple 2: `VerbArgs` — argument access, typed gets, trailing positionals, context

**Example**: `examples/verb_args.rs`  
**Run**: `cargo run --example verb_args`  
**Exit code**: 0  
**Captured output**:
```
deploy --service api --verbose: service=api verbose=true
explain ISSUE-001 ISSUE-002: trailing=["ISSUE-001", "ISSUE-002"]
verb()=status noun()=Some("services")
```
**What it witnesses**: `get_one_str` (required string), `get_one::<bool>` (flag), `trailing()` (var-arg positionals), `VerbContext`/`with_context`/`verb()`/`noun()`. Assertions check the actual values, not just `is_ok()`.  
**Would fail if**: argument parsing broke, `trailing()` returned wrong order, or context was not preserved through `with_context`.

#### Triple 3: `NounVerbError` constructors + `StructuredError` + error propagation

**Example**: `examples/error_handling.rs`  
**Run**: `cargo run --example error_handling`  
**Exit code**: 0  
**Captured output**:
```
CommandNotFound: Command 'usr' not found. Did you mean: user?
VerbNotFound: Verb 'lst' not found for noun 'user'. Did you mean: list, get?
ValidationError (port, abc): Argument parsing failed: Invalid value 'abc' for argument 'port'. Must be a number
RangeError (port, 70000): Argument parsing failed: Invalid value '70000' for argument 'port'. Must be between 1 and 65535
LengthError (name, ): Argument parsing failed: Invalid value '' for argument 'name'. Length must be between 1 and 64 characters
deadline kind=DeadlineExceeded severity=Critical suggested_ms=740
verb error propagated: Command execution failed: db unreachable
```
**What it witnesses**: All 5 `NounVerbError` constructor families, `StructuredError::deadline_exceeded` (kind+severity+20%-padded suggestion), `StructuredError::from_error` mapping, and `Err` propagation through `run_cli_with_args` to the caller.  
**Would fail if**: any constructor changed its Display format, the Levenshtein distance threshold changed, or the StructuredError deadline padding formula changed.

### Key Insight Recorded

The `verb!` macro requires `Fn(&VerbArgs) -> Result<()>` — it is NOT the `Result<T: Serialize>` path used by the `#[verb]` proc-macro. Two distinct patterns exist: the builder/inline pattern (`verb!` macro + `CliBuilder`) and the distributed-slice auto-registration pattern (`#[verb]` proc-macro + `linkme`). The auto-registration path is the advertised zero-boilerplate path and has **no running example yet** — this is the highest-value remaining gap (OPEN-doc-substrate: described everywhere, witnessed nowhere by a running example).

### Queued for Next Iterations

| Priority | Gap | Notes |
|----------|-----|-------|
| HIGH | `#[verb]` proc-macro auto-registration (distributed slice) | The advertised primary path; `#[clap_noun_verb_macros::verb]` + `cli::run()` |
| HIGH | `OutputFormat` / `format_output` | Documented in tutorial 04; no running example |
| MEDIUM | `CommandTree` / `CommandTreeBuilder` | Public API, referenced in docs |
| MEDIUM | `AppContext` | Public API, no example |
| MEDIUM | `Graph` + `Triple` + `GraphLoadedOutput` | `src/graph/` exists, public |
| MEDIUM | `CapabilityRegistry` + `CapabilityPackage` | `src/capability/` exists, public |
| MEDIUM | `DoctorOutput` + `HealthIssue` | `src/diagnostics/` exists, public |
| LOW | `Deprecation` / `DeprecationType` | Public API |
| LOW | `Repl` (feature `repl`) | Feature-gated |
| LOW | Cross-product: `VerbArgs` + `OutputFormat` + error | Composition example |

### Hard Stops

None.

---

## Coverage Status After Iteration 1

**Documented-but-unexercised**: 14 → 11 (closed: `CliBuilder`, `VerbArgs`, `NounVerbError`/`StructuredError`)  
**Exercised-but-undocumented**: 0 (all new examples are documented inline and reference their docs)  
**Running examples that witness the framework**: 0 → 3
