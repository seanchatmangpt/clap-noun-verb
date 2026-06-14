# greet-demo golden — frozen first-render of the pack-rendered verb wrappers

**Provenance = first-render.** The three files in this directory
(`convert.rs`, `greet.rs`, `mod.rs`) are a byte-for-byte freeze of the FIRST
correct render produced by:

```
cd examples/greet-demo
/Users/sac/ggen/target/debug/ggen sync --force true   # source-build engine, NOT the PATH binary
```

from the example's `ontology.ttl` through the authoritative pack artifacts:

- query   `../../queries/verb-signatures.rq`  → template `../../templates/verb.rs.tera`   → `src/verbs/{verb}.rs`
- query   `../../queries/verbs-mod.rq`         → template `../../templates/verbs-mod.rs.tera` → `src/verbs/mod.rs`

## What this golden gates (R3 — Tera fidelity)

`ggen sync` succeeding proves the ontology is well-formed; it does NOT prove the
rendered Rust is correctly *structured* (whitespace-control divergence in a Tera
template can yield compiling-but-mis-structured source). This golden is the frozen
reference the live render must reproduce EXACTLY. To check fidelity:

```
diff src/verbs/convert.rs expected/convert.rs
diff src/verbs/greet.rs   expected/greet.rs
diff src/verbs/mod.rs     expected/mod.rs
```

All three must be empty. A non-empty diff is a template-fidelity defect, not a
cosmetic difference — investigate the `.tera` whitespace markers.

## Provenance note (R5/R6 — external witness)

This golden is committed as a SEPARATE artifact from the render-and-diff step so
that "render == render" cannot pass vacuously. The render that produced these bytes
was witnessed correct by inspection (the human-read trim-tab termination), then
frozen here. The diff above is run against THIS committed copy, not a re-render.

## Held dispatch convention (R6)

`greet-demo tool <verb>` emits `uppercase(<verb>)` as the leading token of its
deterministic output — derived from the committed ontology's verb *names*, not from
any agent spec:

| Command | Expected (proves it reached ITS handler) |
|---|---|
| `greet-demo tool greet [--name N]`            | `GREET hello <N or world>` |
| `greet-demo tool convert --type T [--dry-run B]` | `CONVERT to=<T> dry_run=<B or false>` |

The load-bearing case is the keyword arg `--type` (Rust ident `r#type`) and the
kebab flag `--dry-run` (Rust field `dry_run`): both must reach `convert` with the
CLI-facing flag names intact (`--type`, not `--r#type`).
