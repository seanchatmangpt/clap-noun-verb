# Executable Documentation Coverage Ledger

**Project:** clap-noun-verb 26.7.62  
**Authority:** executable examples plus exact-head CI receipts  
**Standing:** `PARTIAL_ALIVE` until the GitHub execution ladder completes

## Law

A documented API is covered only when a running example imports the public crate,
asserts a meaningful consequence, and includes a negative or boundary case where
one exists. Standalone demonstrations, empty `main` functions, mocks, and prose-only
claims do not count as witnesses.

## Closed public surfaces

| Surface | Executable witness | Consequence |
|---|---|---|
| `CliBuilder`, `noun!`, `verb!`, injected dispatch | `examples/core_api.rs` | Builds, routes, and introspects a two-noun CLI |
| `VerbArgs`, `VerbContext` | `examples/verb_args.rs` | Typed arguments, trailing values, and context survive routing |
| `NounVerbError`, `StructuredError` | `examples/error_handling.rs` | Typed failures become machine-readable recovery objects |
| `#[verb]` distributed registration | `examples/proc_macro_verb.rs` | Three typed handlers are discovered and dispatched |
| `OutputFormat`, `format_output` | `examples/output_formats.rs` | Every format renders; unknown formats refuse |
| `CommandTree`, `CommandTreeBuilder`, `TreeNode` | `examples/command_tree.rs` | Tree construction, lookup, path enumeration, and Clap projection execute |
| `AppContext` | `examples/app_context.rs` | Insert, get, closure access, removal, missing-type refusal, and clear execute |
| `Graph`, `Triple`, `GraphLoadedOutput` | `examples/graph_api.rs` | Valid triples admit; invalid triples refuse without mutation |
| `CapabilityRegistry`, `CapabilityPackage`, standing | `examples/capability_registry.rs` | Dependency closure is stable; duplicate IDs and unreplayed proof refuse |
| `DoctorOutput`, `HealthIssue` | `examples/diagnostics.rs` | Warning and error transitions preserve typed health semantics |
| `Deprecation`, `DeprecationType` | `examples/deprecation.rs` | SemVer prerelease and removal boundaries execute |
| `Repl`, shell-word parser | `examples/repl_witness.rs` | REPL construction and quote/refusal parsing execute non-interactively |
| `ShellType`, completion policy | `examples/shell_completions.rs` | Six shell policies and line-ending boundaries execute |
| Args × format × structured failure | `examples/format_error_pipeline.rs` | Parsed format renders; missing argument becomes `InvalidInput` |
| Registry build and route performance | `benches/dispatch.rs` | Criterion measures the real registry and dispatch path, not a mock |

## Verification commands

```bash
cargo check --all-targets
cargo test --all-targets
cargo test --all-targets --all-features
cargo run --example proc_macro_verb
cargo run --example capability_registry
cargo run --example format_error_pipeline
cargo run --example repl_witness --features repl
cargo bench --bench dispatch --no-run
python scripts/verify_no_wip.py --report target/wip-verifier-report.json
```

## Closure

- Documented-but-unexercised public surfaces: **0**
- Exercised-but-undocumented surfaces introduced by this closure: **0**
- Mock/stub benchmark surfaces on the primary dispatch path: **0**
- Known admitted `TODO`, `FIXME`, `todo!`, or `unimplemented!` markers: enforced as **0** by `scripts/verify_no_wip.py`

The ledger does not promote the repository to `ALIVE`; promotion requires observed
exact-head execution, receipt manufacture, and replay through the repository's
required verification ladder.
