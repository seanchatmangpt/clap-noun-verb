# Reference: cnv-any -- Wrapping Any Executable

**Source**: `clap-noun-verb-any/src/lib.rs`, `clap-noun-verb-any/src/scaffold.rs`, `clap-noun-verb-deploy/src/schema.rs`
**Version**: 26.9.1

`clap-noun-verb-any` wraps any executable -- Rust or not -- as a
`clap-noun-verb-deploy` deployable CLI: same `Deploy`/`Gateway`/protocol-surface
plumbing a native `clap-noun-verb` binary gets, plus OCEL parity, without
requiring the wrapped target to depend on this ecosystem at all.

---

## What it is

`clap-noun-verb-deploy`'s `ProcessExecutor` spawns an arbitrary `OsString`
executable and never inspects what produced it; `Deploy`, `Gateway`, and every
protocol surface (`mcp.rs`/`http.rs`/`kubernetes.rs`/`container.rs`) are built
on top of `CliSchema` + the `Executor` trait, not on Rust or Clap specifically.
The only missing piece for wrapping a non-`clap-noun-verb` binary was schema
*acquisition* -- `CliSchema::from_command` requires an in-process
`clap::Command`. `clap-noun-verb-any::wrap` closes that gap.

## The manifest-is-CliSchema design

A wrapped target is declared by a manifest file that deserializes directly
into the existing `CliSchema` shape -- `name`, `about`, and a list of
`CommandSchema` (each with a `path`, `about`, `arguments`, and `callable`
flag). No new format is introduced. Two constructors were added directly to
`clap-noun-verb-deploy::CliSchema` so any consumer of that crate can load a
manifest, not just this one:

- `CliSchema::from_manifest_reader<R: Read>(reader: R)`
- `CliSchema::from_manifest_path(path: &Path)`

`clap-noun-verb-any::wrap(executable, manifest_path)` loads the manifest,
builds a `ProcessExecutor` pinned to `executable`, and returns a `Wrapped`
holding the **same `Deploy` type** `Deploy::from_registry` produces for a
native binary (via the small additive `Deploy::from_schema` constructor). Every
existing protocol surface works against it unchanged -- `mcp.rs`, `http.rs`,
`kubernetes.rs`, and `container.rs` were not modified to support this.

## OCEL parity

Every clap-noun-verb CLI already gets a zero-configuration OCEL 2.0 event log
(see `docs/reference/ocel-v2.md`). A wrapped foreign binary cannot self-emit
that event -- it isn't built on this crate's dispatch path. `OcelExecutor`
wraps any `Executor` and, after each execution, calls
`clap_noun_verb::ocel::record_invocation` on the wrapped binary's behalf,
deriving the noun/verb attribution from the manifest's matching command path.
The resulting event is structurally identical to one a native
`clap-noun-verb` binary would have emitted itself -- same object types, same
event type, same relationship shape.

`clap_noun_verb::ocel::merge_documents(paths)` folds N real OCEL documents from
disk into one, unioning object/event types by name and deduping objects by id
-- the structural seam for aggregating a fleet's worth of invocation evidence
(native and wrapped alike) into one comparable corpus.

## Manifests are ggen-generated, not hand-authored

Consistent with this project's own generation model (RDF is admitted, ggen
manufactures, hand-written source is the rare declared exception): every
`cnv-any.json` this crate ships or documents is compiled by the real ggen
pack `cnv-any-manifest-pack` (in `~/ggen-marketplace/packs/`) from an
`ontology.ttl` describing the wrapped program's real command surface -- the
same `cnv:Cli`/`cnv:Noun`/`cnv:Command`/`cnv:Argument` vocabulary and
admission gates `clap-noun-verb-schema-pack` already uses to compile real
Rust CLIs, just projected into a `CliSchema` manifest instead of Rust source.
See `clap-noun-verb-any/examples/*/README.md` for three worked examples (a
shell script, a Python argparse CLI, and autofde-lab's 46+ planners), each
with its own `ontology.ttl` + `ggen.toml` + a real `ggen sync run`.

`CliSchema::from_manifest_path` itself is a general JSON loader with no way
to enforce provenance, but nothing in this crate's own examples or docs
treats a hand-typed manifest as the intended path. For a quick starting
point while authoring a NEW ontology (not a substitute for one),
`clap-noun-verb-any::scaffold::draft_manifest_from_help` runs
`<executable> --help` and applies a few simple, clearly-labeled heuristics
(subcommand-listing lines, `--long-flag` tokens) to produce a **draft**
`CliSchema` as a reading aid -- never a trusted schema source, since no
ecosystem's `--help` format is reliable enough to parse unreviewed. The
`cnv-any init <path-to-binary> [--out draft-schema.json]` binary writes that
draft to disk as pretty JSON.

## Why this matters at scale

Combined with ggen's combinatorial-generation strategy (many packs, many
noun x verb x behavior combinations), the leverage here isn't "wrapping any
language" as an isolated trick -- it's that every wrapped target, Rust or not,
produces the identical OCEL schema as every combinatorially-generated native
CLI. A fleet built from both sources becomes one comparable, aggregable
observability corpus instead of N bespoke logs with N bespoke parsers. That
corpus feeding back into pack-selection is no longer hypothetical: see
`docs/reference/ocel-feedback-loop.md` -- `compute_signals`/`write_signal_pack`
plus `ggen-marketplace`'s `ocel-feedback-pack` gate refuse to regenerate a
command real fleet-wide evidence says has never once been invoked, a real
(tested, subprocess-verified) closure of observation back into a
generation decision.

## `cnv-any doctor` -- preflight a manifest + executable pairing

`clap-noun-verb-any::doctor::diagnose(executable, manifest_path) -> DoctorReport`
checks a real pairing before deployment: does the executable exist and is
it actually executable (Unix mode bits), does the manifest parse into a
real `CliSchema`, and is every admitted command's argument shape internally
consistent (duplicate ids/long/short flags, a positional argument that also
declares a flag, duplicate command paths). Findings are `Error` (would
break `wrap()`/deployment) or `Warning` (works today but is likely
unintended). Run it directly:

```bash
cnv-any doctor <path-to-binary> <path-to-manifest>
```

`schema_shape_errors(&CliSchema) -> Vec<String>` exposes just the
error-only subset, reusable in-process. `wrap()` itself calls this and
hard-refuses (`WrapError::InvalidShape(errors)`) before ever constructing a
`ProcessExecutor` -- a shape-invalid manifest can never reach a spawned
process. This is the same kind of structural shape validation SHACL
performs, but it is **not** literal SHACL/RDF Shapes -- no such dependency
is shipped, it's hand-rolled Rust checks over the same `CliSchema`.

## See Also

- `docs/reference/ocel-v2.md` -- the OCEL 2.0 event log this crate achieves
  parity with
- `clap-noun-verb-any/README.md` -- crate-level usage, including the
  documented (not built) Gall integration point
- `clap-noun-verb-deploy/README.md` -- the deployment surfaces this crate
  reuses unchanged
