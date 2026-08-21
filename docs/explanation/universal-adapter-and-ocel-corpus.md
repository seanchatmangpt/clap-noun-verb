# The Universal Adapter and the OCEL Corpus

---

## Context

`clap-noun-verb-deploy` already projects a native `clap-noun-verb` CLI to
MCP, HTTP, a Kubernetes projection, and an OCI container — but only for a
binary built on this crate's own macros. Two questions this explanation
answers: why wrapping *any* executable turned out to need almost no new
code, and why every such manifest is required to be ggen-generated rather
than hand-authored.

## Design Decision: reuse `CliSchema`, don't invent a new format

`clap-noun-verb-deploy`'s `ProcessExecutor` spawns an arbitrary `OsString`
executable and never inspects what produced it. `Deploy`, `Gateway`, and
every protocol surface (`mcp.rs`/`http.rs`/`kubernetes.rs`/`container.rs`)
are built on `CliSchema` + the `Executor` trait — not on Rust or Clap
specifically. `CliSchema`/`CommandSchema`/`ArgumentSchema` are themselves
plain `Serialize`/`Deserialize` data. The only missing piece for wrapping a
non-Rust binary was schema *acquisition*: `CliSchema::from_command`
requires an in-process `clap::Command`, which a foreign binary obviously
doesn't have.

`clap-noun-verb-any` closes that one gap: a wrapped target is declared by a
manifest that deserializes directly into the existing `CliSchema` shape.
No new format, no new deploy-surface code — `mcp.rs`/`http.rs`/
`kubernetes.rs`/`container.rs` were not modified to support this.

## Rationale: manifests must be ggen-generated, not hand-authored

This project's own generation model is: RDF is admitted, ggen manufactures,
hand-written source is the rare declared exception. A hand-typed
`cnv-any.json` would be exactly the kind of undeclared exception that
model exists to prevent — a second, informal source of truth for a CLI's
command surface, diverging silently from whatever ontology (if any) someone
eventually writes for it.

`cnv-any-manifest-pack` closes that gap by reusing
`clap-noun-verb-schema-pack`'s vocabulary and all 16 admission gates
unchanged — the exact same `cnv:Cli`/`cnv:Noun`/`cnv:Command`/`cnv:Argument`
classes and constraints `clap-noun-verb-crate-pack` already uses to compile
a real Rust CLI's `Cargo.toml`/`src/main.rs`. The only new artifact is one
additional SPARQL+Tera projection of the same admitted graph, into
`cnv-any.json` instead of Rust source. Every command in that projection
must carry `cnv:CustomBehavior` — the honest declaration that the real
logic lives in the wrapped foreign process, not in generated Rust, so the
schema-pack's usual generative guarantees don't (and can't) apply to the
command's actual behavior, only to its declared shape.

`CliSchema::from_manifest_path` itself has no way to enforce this — it's a
general JSON loader, and nothing stops a determined caller from hand-typing
a manifest. The discipline is a project convention, not a type-level
guarantee: every manifest this crate ships or documents in `examples/` is
ggen-generated, and `tests/fixtures/cnv-any.json` is the one deliberate
exception — three lines of hand-typed unit-test scaffolding, explicitly
distinct from the `examples/` policy (see
[cnv-any README](../../clap-noun-verb-any/README.md#testing)).

## Rationale: why the wrapped-binary OCEL event matters

Every native `clap-noun-verb` CLI already gets a zero-configuration OCEL
2.0 event log. A wrapped foreign binary can't self-emit that event — it
isn't built on this crate's dispatch path. `OcelExecutor` closes that gap
by calling `clap_noun_verb::ocel::record_invocation` on the wrapped
binary's behalf after every execution, deriving noun/verb attribution from
the manifest's matching command path. The resulting event is *structurally
identical* to one a native binary would have emitted itself: same object
types, same event type, same relationship shape.

This is the leverage point: combined with ggen's combinatorial-generation
strategy (many packs, many noun×verb×behavior combinations), the payoff
isn't "wrapping any language" as an isolated trick — it's that every
wrapped target, Rust or not, produces the *identical* OCEL schema as every
combinatorially-generated native CLI. A fleet built from both sources
becomes one comparable, aggregable observability corpus instead of N
bespoke logs with N bespoke parsers. `clap_noun_verb::ocel::merge_documents`
is the concrete seam for folding that fleet's evidence into one document.

## Trade-offs

- **What this buys**: zero marginal observability cost per wrapped target,
  a comparable corpus across an arbitrarily heterogeneous fleet, and — per
  [OCEL Feedback Loop](../reference/ocel-feedback-loop.md) — that corpus
  now feeding back into a real generation decision: `ggen-marketplace`'s
  `ocel-feedback-pack` refuses to regenerate a command real fleet-wide
  evidence says has never once been invoked, closing the loop this design
  made structurally possible (`merge_documents`, `to_rdf`, `drift_report`,
  `prune_candidates`, and now `compute_signals`/`write_signal_pack`).
- **What the convention costs**: an extra step (write an ontology, run
  `ggen sync run`) versus just hand-typing three lines of JSON for a
  trivial wrapper. For `examples/` and any real deployment, that cost buys
  the same generative discipline every other CLI this ecosystem produces
  already has; `tests/fixtures/` is the one place that trade isn't worth
  making, since it's throwaway unit-test scaffolding, not a worked example.

## Alternatives Considered

- **A bespoke `cnv-any`-specific manifest format**: rejected — `CliSchema`
  already exists, is already the shape every deploy surface consumes, and
  inventing a second shape would mean either dual-maintaining two schemas
  or writing a translation layer neither the schema-pack nor the deploy
  surfaces need.
- **Trusting parsed `--help` output as a live schema**: rejected as a
  schema *source* — no ecosystem's `--help` format is reliable enough to
  parse unreviewed. `scaffold::draft_manifest_from_help` exists only as a
  best-effort reading aid while hand-authoring a *new* ontology, explicitly
  never a substitute for one.
- **Letting `clap-noun-verb-any` enforce provenance at the type level**
  (e.g. requiring a signed manifest or a build-time ggen invocation):
  rejected for this pass — `CliSchema::from_manifest_path` stays a general
  JSON loader usable by any consumer of `clap-noun-verb-deploy`, and the
  ggen-only discipline is enforced by project convention (this document,
  the crate's own README, and every example) rather than by the loader
  itself.

## Further Reading

- [Reference: cnv-any](../reference/cnv-any.md)
- [Reference: OCEL v2](../reference/ocel-v2.md)
- [Reference: OCEL Fuller Capabilities](../reference/ocel-fuller-capabilities.md)
- [How-To: Generate a cnv-any manifest with ggen](../howto/generate-cnv-any-manifest.md)
- [Tutorial 07: Wrapping Any Executable](../tutorial/07-wrapping-any-executable.md)
