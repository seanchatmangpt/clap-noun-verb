# clap-noun-verb 26.9.1

First announced release of `clap-noun-verb` on GitHub. This release closes out the
Autonomic Layer and OCEL feedback-loop work started in the 26.8.x series, and adds
real deployment/multi-CLI-serving capabilities to `clap-noun-verb-deploy` and
`clap-noun-verb-any`.

Crate versions in this release: `clap-noun-verb` 26.9.1, `clap-noun-verb-macros`
26.9.1, `clap-noun-verb-deploy` 26.9.1, `clap-noun-verb-any` 26.9.1.
`clap-noun-verb-utils` stays at 26.6.1 (unchanged in this cycle).

## Highlights

**Autonomic Layer, extended.** Two new capabilities join the existing
Effects/Guards/Receipts machinery:
- `Delegation` — hash-chained (same FNV-1a/genesis-digest scheme as `Receipt`)
  agent-to-agent authorization chains, with a bounded transitive `is_authorized` walk.
- `GovernancePanel` — a rule-based admission layer that plugs directly into the
  existing `Guard`/`GuardSet` machinery; every registered rule must approve for an
  invocation to be admitted (fail-closed).

**`clap-noun-verb-any` grows a doctor and a multi-CLI server.** `doctor` preflights
a manifest + executable pairing (existence, executable bit, parse errors, duplicate
argument ids/flags, duplicate command paths) before deployment, and `wrap()` now
hard-refuses a shape-invalid manifest before ever spawning a process. A new
`MultiExecutor`/`merge_schemas` lets one `Executor`/`CliSchema` pair — and so one MCP
server — serve tools from several distinct wrapped foreign binaries at once. Five
more real fixture CLIs (word-count, calc, list-fruits, status-check, repeat) round
out the wrapped-executable test surface.

**`clap-noun-verb-deploy` grows a CronJob projection and a real OCI builder.**
`kubernetes::CronJobConfig` produces a deterministic, CONSTRUCT-only `CronJob` YAML
projection for scheduled/batch verbs, following the same hardening defaults as the
existing `Deployment` projection. `oci_builder` is a separate, explicitly-effectful
path from the pure Dockerfile projection: it shells out to a real, already-installed
`docker`/`podman`/custom OCI CLI to actually build an image.

**OCEL feedback loop gets an inverse and a ggen gate.** `ocel::from_rdf` is the real,
narrow inverse of `to_rdf`'s emission grammar (not a general Turtle parser — numeric
literals always round-trip as `f64`, no nested blank-node support). `drift_report`'s
coverage ratio is now closeable into a real, composable `ggen-marketplace` gate
(`ocel-drift-pack`), proven via a real `ggen sync run` subprocess test.

**Verbs can now declare their effect.** `#[verb(..., effect = "read_only" |
"mutating" | "idempotent")]` flows mechanically into `ExecutionContract`'s
`IsolationLevel`/`idempotent` fields and the recorded `Receipt`.

**Test coverage.** Criterion benchmarks for `compute_signals`/`drift_report`/`to_rdf`
at 10k/100k/1M OCEL event scale; property-based tests for `merge_documents`/
`compute_signals`; real concurrent-writer and permission-denied fault-injection tests
for the receipt/invocation ledger; a scaled concurrent-dispatch load test with
honest, measured evidence for why a literal 100k-invocation scale is not yet
feasible against the current O(n)-per-append ledger design.

## Changed

- `src/policies.rs` and `src/telemetry.rs` are now ggen-generated (previously
  hand-written), matching `src/autonomic.rs`'s existing generation discipline. A
  stale `cli_version` literal (`"3.8.0"`) was caught and fixed during the migration.
- `CommandRegistry::add_guard`/`GuardSet` are now actually wired into
  `execute_verb`/`execute_root_verb` dispatch (previously available but unused).
- Removed dead fields (`NounMetadata.name`, `VerbMetadata.noun_name`/`verb_name`,
  `SimpleNoun.verbs`) per ADL-005.

## Fixed

A live-code audit of `docs/reference/` found and corrected several
overclaiming/fabrication gaps: fabricated `HandlerInput`/`HandlerOutput`/
`AppContext`/`ArgMetadata` signatures, a fabricated "v5 Autonomic API" telemetry
section, a false claim that `#[verb]` supports `async fn` handlers directly, a false
claim that `check_verb_registration!()` performs a real compile-time check (it is a
documented no-op), and a README claim that `schema-validation.md` covers SHACL (it
explicitly does not ship one).

## Verification

- `cargo build --workspace`: clean, exit 0.
- `cargo test --workspace`: 0 failed across every suite (root unit tests 180 passed,
  macros 45 passed, utils doc-tests 24 passed, plus integration test binaries; 31
  macros doc-tests are `ignored`, not failed).
- `cargo clippy --workspace --all-targets`: 0 warnings, 0 errors.

## Full changelog

See [CHANGELOG.md](./CHANGELOG.md#2691---2026-08-21) for the complete, itemized
entry with source-file references.

## Install

```toml
[dependencies]
clap-noun-verb = "26.9.1"
```

---

*This file is a prepared draft. Publishing to GitHub Releases has not been done —
this would be the first-ever public release of this repository and requires
explicit user go-ahead.*
