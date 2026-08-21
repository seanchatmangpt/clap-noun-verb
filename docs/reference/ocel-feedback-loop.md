# Reference: the OCEL Feedback Loop

**Source**: `src/ocel.rs` (`compute_signals`, `signals_to_rdf`, `write_signal_pack`),
`~/ggen-marketplace/packs/ocel-feedback-pack`
**Version**: 26.8.22 (unreleased)

Closes the loop [`docs/reference/ocel-fuller-capabilities.md`](ocel-fuller-capabilities.md)
left open: a real merged OCEL corpus now feeds back into a real ggen
generation *decision*, not just an aggregated-but-inert audit log.

---

## What it does

Given a real merged OCEL corpus (`ocel::merge_documents`) and the set of
commands admitted in an ontology, `compute_signals` derives one
recommendation per command:

| Recommendation | Meaning |
|---|---|
| `prune` | Admitted, but never invoked once across the whole corpus. |
| `review` | Invoked at least once, but not within the recency window checked. |
| `harden` | Invoked recently, but below the configured success-rate threshold. |
| `keep` | Invoked recently and healthy. |

```rust
use clap_noun_verb::ocel::{compute_signals, merge_documents};
use std::time::Duration;

let observed = merge_documents(&fleet_ocel_paths)?;
let admitted = [("fleet", "dead"), ("fleet", "alive")];
let signals = compute_signals(&admitted, &observed, Duration::from_secs(60 * 60 * 24 * 30), chrono::Utc::now(), 0.5);
```

`signals_to_rdf` projects those signals as `cnv-ocel:Signal` Turtle
individuals; `write_signal_pack(dir, &signals)` writes a **complete,
ggen-composable pack directory** (`pack.toml` + `ontology.ttl` +
one status template) from them in a single call -- the turnkey mechanism
that turns a merged corpus into something `ggen sync run` can consume
directly, with no hand-authored boilerplate each cycle.

## The gate that closes the loop

`~/ggen-marketplace/packs/ocel-feedback-pack` composes alongside a
project's own `cnv:Cli` ontology and the regenerated signals pack. Its one
gate, `gates/005_ocel_pruning_advisory.rq`, is fail-closed (consistent
with every other gate in this ecosystem -- any returned row is a
refusal): it matches any `cnv:Command` whose `noun:verb` id corresponds to
a composed `cnv-ocel:Signal` marked `recommendation "prune"`, and refuses
the sync.

Concretely: a command real fleet-wide usage evidence says has **never
once been invoked** blocks the next `ggen sync run` for the project that
declares it -- forcing a human to explicitly remove it from the ontology
(or produce fresh evidence) rather than silently regenerating dead code
forever.

## Real, verified, automated proof

`tests/ocel_feedback_loop.rs`
(`real_ggen_refuses_a_command_a_real_ocel_corpus_says_is_dead_and_accepts_it_removed`,
`#[ignore]`d -- requires the real `ggen` binary and a real
`~/ggen-marketplace` checkout with `clap-noun-verb-schema-pack` and
`ocel-feedback-pack`) does the whole thing for real, twice, against the
SAME signals evidence:

1. Builds a real `OcelDocument` where `fleet:alive` was invoked (healthy)
   and `fleet:dead` was admitted but never invoked.
2. `compute_signals` + `write_signal_pack` produce a real, on-disk,
   ggen-composable signals pack.
3. A real `ggen sync run` subprocess against an ontology admitting BOTH
   commands is asserted to **fail**, with `stderr` naming
   `ocel-feedback-pack` and `fleet:dead` explicitly.
4. The same signals, same gate, but `fleet:dead` removed from the
   ontology: a real `ggen sync run` subprocess is asserted to **succeed**,
   and its generated `docs/OCEL_FEEDBACK_STATUS.md` is read back and
   asserted to contain `fleet:alive`/`keep`.

## Operational workflow

```sh
# 1. Fleet operates; OCEL logs accumulate on disk per deployment (native
#    clap-noun-verb binaries and clap-noun-verb-any-wrapped targets alike).

# 2. Merge + compute + write the signals pack (one Rust call):
let observed = clap_noun_verb::ocel::merge_documents(&fleet_ocel_paths)?;
let signals = clap_noun_verb::ocel::compute_signals(&admitted, &observed, min_age, chrono::Utc::now(), 0.5);
clap_noun_verb::ocel::write_signal_pack(Path::new("/var/ggen/ocel-signals"), &signals)?;

# 3. Compose it into the project generating that fleet's CLIs:
#    [packs]
#    clap-noun-verb-schema-pack = { path = "~/ggen-marketplace/packs/clap-noun-verb-schema-pack" }
#    ocel-feedback-pack         = { path = "~/ggen-marketplace/packs/ocel-feedback-pack" }
#    ocel-signals               = { path = "/var/ggen/ocel-signals" }

# 4. ggen sync run now refuses to regenerate anything real evidence says is dead.
ggen sync run
```

## What this doesn't do

The gate advises against a specific class of regression (regenerating a
command with zero real invocations) -- it is not a general-purpose
"promote what's used, demote what isn't" ranking, and `harden`/`review`
recommendations are surfaced (in the generated status report) but not
yet enforced by any gate. Extending `ocel-feedback-pack` with additional
gates for those recommendations is a natural next step, not built here.

## See Also

- [OCEL v2](ocel-v2.md) -- the zero-configuration event log this loop consumes
- [OCEL Fuller Capabilities](ocel-fuller-capabilities.md) -- `drift_report`/`prune_candidates`/`to_rdf`/`merge_documents`, the seam this loop closes
- [cnv-any](cnv-any.md) -- how wrapped foreign targets contribute to the same corpus
- [The Universal Adapter and the OCEL Corpus](../explanation/universal-adapter-and-ocel-corpus.md)
