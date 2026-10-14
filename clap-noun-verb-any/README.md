# clap-noun-verb-any

Wrap **any** executable -- Rust or not -- as a `clap-noun-verb-deploy` deployable
CLI, with OCEL parity.

## Design: the manifest reuses `CliSchema` directly

`clap-noun-verb-deploy`'s `ProcessExecutor` spawns an arbitrary `OsString`
executable and never inspects what produced it. `Deploy`, `Gateway`, and every
protocol surface (`mcp.rs`/`http.rs`/`kubernetes.rs`/`container.rs`) are built on
top of `CliSchema` + the `Executor` trait, not on Rust or Clap specifically. The
entire gap between "a native `clap-noun-verb` binary" and "any executable" is
schema *acquisition*.

`clap-noun-verb-any` closes that gap without inventing a new format: a
**manifest is exactly `CliSchema`'s existing JSON shape**. A wrapped target is a
directory containing the executable plus a `cnv-any.json` that deserializes
straight into `clap_noun_verb_deploy::CliSchema` via the new
`CliSchema::from_manifest_path` constructor (added directly to
`clap-noun-verb-deploy`, so any consumer of that crate can use it, not just this
one).

```rust
use clap_noun_verb_any::wrap;
use std::path::Path;

let wrapped = wrap("./my-tool", Path::new("cnv-any.json"))?;
// wrapped.deploy() is the *same* Deploy type a native clap-noun-verb binary
// produces via Deploy::from_registry -- hand its schema + wrapped.executor()
// to McpServer::new / HttpServer::new / the kubernetes/container renderers
// exactly as you would for a native CLI. Zero changes needed in
// mcp.rs/http.rs/kubernetes.rs/container.rs.
```

## OCEL parity: zero marginal observability cost

Every clap-noun-verb CLI already gets a zero-configuration OCEL 2.0 event log
(`src/ocel.rs`). A wrapped foreign binary can't self-emit that event -- it
isn't built on this crate. `clap-noun-verb-any::OcelExecutor` closes that gap: it
wraps any `Executor` (typically a `ProcessExecutor`) and, after every
execution, calls `clap_noun_verb::ocel::record_invocation` on the wrapped
target's behalf, using the manifest's command path as the noun/verb. The
resulting OCEL event is **structurally identical** to one a native
`clap-noun-verb` binary would have emitted itself.

This is the concrete mechanism that makes a fleet of CLIs -- native,
combinatorially generated, and any-language-wrapped alike -- into one
comparable, aggregable observability corpus rather than N bespoke logs. See
`clap_noun_verb::ocel::merge_documents` for folding N such documents into one.

## `cnv-any.json` is generated, not hand-authored

Consistent with this whole ecosystem's principle (RDF is admitted, `ggen`
manufactures, hand-written source is the rare declared exception): every
`cnv-any.json` in `examples/` is compiled by a real ggen pack,
[`cnv-any-manifest-pack`](https://github.com/seanchatmangpt/ggen-marketplace/tree/main/packs/cnv-any-manifest-pack),
from an `ontology.ttl` describing the wrapped program's real command surface
-- the SAME `cnv:Cli`/`cnv:Noun`/`cnv:Command`/`cnv:Argument` vocabulary and
admission gates (`clap-noun-verb-schema-pack`) that `clap-noun-verb-crate-pack`
already compiles into a real Rust CLI's `Cargo.toml`/`src/main.rs`. This pack
adds no new vocabulary or gates -- one new projection of the same admitted
graph, into a manifest instead of Rust source.

```sh
cd examples/<name>
ggen sync run   # writes cnv-any.json from ontology.ttl
```

`CliSchema::from_manifest_path` (in `clap-noun-verb-deploy`) itself is a
general JSON loader and has no way to know or enforce where a `cnv-any.json`
came from -- but every manifest this crate ships or documents is
ggen-generated, never hand-typed, and every example's `README.md` shows the
real `ggen.toml` composition that produces it.

For a quick draft to seed a NEW ontology (not a substitute for one):

```sh
cnv-any init ./my-tool --out draft-schema.json
```

This runs `./my-tool --help` and applies a few simple heuristics
(subcommand-listing lines, `--long-flag` tokens) to produce a draft
`CliSchema` as a reading aid while hand-authoring the real `ontology.ttl` --
**not** a trusted schema source and **not** what `cnv-any.json` should be
generated from directly; no ecosystem's `--help` output format is reliable
enough to parse and trust unreviewed. See `src/scaffold.rs` for the exact
heuristics and their documented limits.

## Gall integration point (documented only, not built)

chatman-ecosystem's `gall` crate does not currently invoke external programs --
it is a deliberately closed, dependency-free (`unsafe_code = "forbid"`, zero
deps) proof capsule with two hardcoded capabilities. Its `Broker::actuate`
(`crates/gall/src/lib.rs:568-583`) is a
`match action.capability.as_str() { "echo" => ..., "wasm.actuate" => ..., _ => Err(..) }`.
A third arm, `"cnv-any.invoke"`, could dispatch `action.arguments` through
`clap-noun-verb-any`'s `Gateway::execute`, folding the resulting
`ExecutionRecord` into Gall's existing `Receipt`/replay chain unchanged. This is
documented as the exact seam, not implemented here: adding subprocess
execution to a crate whose entire design point is `unsafe_code = "forbid"` plus
zero dependencies is a decision for a dedicated follow-up, not a rider on this
crate. Nothing under `chatman-ecosystem` is touched by this crate.

## Testing

`tests/wrap_integration.rs` wraps a real trivial fixture script
(`tests/fixtures/greet.sh`) with a manifest (`tests/fixtures/cnv-any.json`),
runs a real admitted invocation through the real `Gateway`/`ProcessExecutor`
path, and asserts the real captured stdout/exit-code plus a real OCEL event
recorded on the wrapped script's behalf -- Chicago style, no mocks. That
fixture's manifest is intentionally minimal, hand-typed unit-test scaffolding
(three lines of JSON, not a worked example) -- the ggen-generation policy
above applies to `examples/`, the surface meant to demonstrate real usage;
`tests/autofde_lab_integration.rs` exercises the real ggen-generated
`examples/autofde_lab_planners/cnv-any.json` end to end instead.
