# Executable Documentation Coverage Ledger

**Project:** clap-noun-verb 26.7.62  
**Authority:** executable examples plus exact-head CI receipts  
**Standing:** `PARTIAL_ALIVE` until the GitHub execution ladder completes

## Law

A documented API is covered only when a running example imports the public crate,
asserts a meaningful consequence, and includes a negative or boundary case where
one exists. Standalone demonstrations, empty executable bodies, mock primary paths,
hard-coded benchmark claims, and prose-only declarations do not count as witnesses.

## Closed public surfaces

| Surface | Executable witness | Consequence |
|---|---|---|
| `CliBuilder`, `noun!`, `verb!`, injected dispatch | `examples/core_api.rs` | Builds, routes, and introspects a two-noun CLI |
| `VerbArgs`, `VerbContext` | `examples/verb_args.rs` | Typed arguments, trailing values, and context survive routing |
| `NounVerbError`, `StructuredError` | `examples/error_handling.rs` | Typed failures become machine-readable recovery actions |
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
| Agent-oriented read route | `examples/agent_cli_builder.rs` | A typed read-only capability route emits explicit non-actuation standing |
| RDF → typed adapter | `examples/ontology_to_cli.rs` | SPARQL and RDF definitions manufacture canonical handler adapters |
| Discovery engine | `examples/frontier_discovery_engine_demo.rs` | Records discover deterministically; duplicate and missing routes refuse |
| Reflexive standing | `examples/frontier_reflexive_testing_demo.rs` | Passing checks without replay do not receive `ALIVE` standing |
| Semantic coordination | `examples/semantic_coordinator.rs` | Invariants admit before RDF composition; invalid triples refuse |
| RevOps revenue route | `examples/revops_revenue_dashboard.rs` | Bounded metrics dispatch without financial actuation |
| RevOps pipeline route | `examples/revops_sales_pipeline.rs` | Weighted standing dispatches without ambient CSV reads or writes |
| RevOps forecast route | `examples/revops_financial_forecast.rs` | Twelve deterministic periods derive from explicit integer assumptions |
| RevOps sequence route | `examples/revops_email_sequences.rs` | Communication text renders locally with `delivery_performed=false` |
| RevOps customer route | `examples/revops_cs_checkins.rs` | Health standing derives without outreach or customer-system mutation |
| Rust ↔ RDF round trip | `src/ggen_to_rdf.rs`, `src/rdf_to_ggen.rs` tests | Attributes, arguments, references, escaping, and result carriers close canonically |
| Ontology synchronization | `src/ontology_sync.rs` tests | Drift is nonconformant, malformed triples refuse, and actuation emits a receipt |
| Registry build and route performance | `benches/dispatch.rs` | Criterion measures the real registry and dispatch path, not a mock |

## Verification commands

```bash
cargo check --all-targets
cargo test --all-targets
cargo test --all-targets --all-features
bash scripts/run_witnesses.sh target/witnesses-first.txt
bash scripts/run_witnesses.sh target/witnesses-second.txt
cmp target/witnesses-first.txt target/witnesses-second.txt
cargo bench --bench dispatch --no-run
python scripts/verify_no_wip.py --report target/wip-verifier-report.json
```

## Closure

- Documented-but-unexercised public surfaces: **0**
- Exercised-but-undocumented surfaces introduced by this closure: **0**
- Standalone application simulations on the admitted example surface: **0**
- Mock/stub benchmark surfaces on the primary dispatch path: **0**
- Known admitted unfinished markers or empty examples: enforced as **0** by `scripts/verify_no_wip.py`
- Whole-repository `ALIVE` promotion before exact-head execution and replay: **refused**

This ledger does not promote the repository to `ALIVE`; promotion requires observed
exact-head execution, receipt manufacture, and replay through the repository's
required verification ladder.
