# Reference: the Autonomic Layer

**Source**: `src/autonomic.rs` -- **generated**, not hand-authored, by
`~/ggen-marketplace/packs/clap-noun-verb-autonomic-pack` from this repo's
own `ontology.ttl` (composed via the repo-root `ggen.toml`)
**Version**: 26.8.22 (unreleased)

Closes a real gap between claim and reality: `docs/reference/README.md`'s
API Stability Guarantees table already claimed
`✅ **Autonomic Layer**: Introspection, effects, guards, receipts` as a
stable, available API, but only Introspection (`--introspect`) existed in
`src/`. This module is the other three, made real.

---

## Effects

`Effect` is a formal declaration of what kind of consequence a verb's
execution has: `ReadOnly` / `Mutating` / `Idempotent` / `Unknown`.
`Unknown` is the honest default recorded automatically for every verb
today -- no macro-level attribute yet lets a verb author declare its real
effect, so nothing is ever silently guessed as `ReadOnly`/`Idempotent`
without a real declaration backing it.

## Guards

```rust
use clap_noun_verb::autonomic::{Guard, GuardContext, GuardDenial, GuardSet};

struct RequiresName;
impl Guard for RequiresName {
    fn name(&self) -> &'static str { "requires_name" }
    fn check(&self, ctx: &GuardContext<'_>) -> Result<(), GuardDenial> {
        if ctx.args.get("name").is_some() { Ok(()) }
        else { Err(GuardDenial::new("MISSING_NAME", "\"name\" is required")) }
    }
}

let mut guards = GuardSet::new();
guards.add(Box::new(RequiresName));
```

`GuardSet::evaluate` runs every registered guard and collects every
denial -- it does not short-circuit on the first failure, so a caller sees
every reason an invocation would be refused at once.

## Receipts

Every verb dispatched through `CommandRegistry::execute_verb`/
`execute_root_verb` gets one append-only, hash-chained `Receipt` recorded
automatically -- the same "always-on, never feature-gated" guarantee
`src/ocel.rs`'s OCEL event log already has (ADL-003), at the identical
dispatch site.

```rust
use clap_noun_verb::autonomic::read_and_verify_ledger;
use std::path::Path;

let receipts = read_and_verify_ledger(Path::new(".clap-noun-verb/receipts.jsonl"))?;
```

`read_and_verify_ledger` reads the whole ledger back and recomputes every
receipt's digest and chain link in one call, rejecting a tampered or
reordered ledger with a typed `ChainVerificationError`.

**Not cryptographic.** The chain digest is a plain FNV-1a 64-bit hash --
it detects accidental corruption or reordering, not deliberate tampering
by an adversary with write access to the ledger file. Stated explicitly
so "receipt" is never overclaimed as a security-grade guarantee.

## Generation: `src/autonomic.rs` is ggen-generated

Consistent with this project's own no-hand-coding-clap-noun-verb-code
policy, `src/autonomic.rs` is compiled by
`clap-noun-verb-autonomic-pack` from one admitted `cnv-autonomic:Config`
individual in this repo's own `ontology.ttl`:

```turtle
cnv-autonomic:ClapNounVerbAutonomicConfig
    a cnv-autonomic:Config ;
    cnv-autonomic:envPathVar "CLAP_NOUN_VERB_RECEIPT_PATH" ;
    cnv-autonomic:defaultRelativePath ".clap-noun-verb/receipts.jsonl" ;
    cnv-autonomic:fallbackFileName "clap-noun-verb-receipts.jsonl" .
```

```sh
ggen sync run   # regenerates src/autonomic.rs from ontology.ttl
```

The env var name, default ledger path, and fallback file name are the
real parametrized knobs; the Effect/Guard/Receipt algorithmic logic itself
is fixed framework behavior the template renders verbatim -- the same
relationship `clap-noun-verb-crate-pack` has to a generated consumer
`main.rs`: the template is the compiler, not hand-typed output. A
fail-closed gate (`gates/010_required.rq`) refuses generation if the
`Config` individual is missing any required field; another
(`gates/020_exactly_one_config.rq`) refuses unless exactly one `Config` is
admitted.

The two `crate::autonomic::record_receipt(...)` call sites inside
`src/cli/registry.rs`'s `execute_verb`/`execute_root_verb` remain a
declared, hand-edited exception: ggen emits whole files, not patches into
pre-existing hand-owned framework dispatch code.

## What's not yet built

- No macro-level `#[verb(effect = "read_only")]`-style attribute exists
  yet to let a verb author declare a real `Effect` -- every receipt today
  records `Effect::Unknown`.
- Delegation (agent-to-agent authorization chains), formal capability
  Contracts, and Governance -- named in this project's own archived
  playground design (`archive/playground/PLAYGROUND_OVERVIEW.md`) as
  further Autonomic Layer components -- are not part of this pass.

## See Also

- `docs/reference/ocel-v2.md` -- the OCEL 2.0 event log this module's
  "always-on" discipline mirrors
- `docs/reference/ocel-feedback-loop.md` -- another real closure of an
  observed-evidence corpus back into a generation decision, the same
  pattern this module's own ggen generation follows
- `src/autonomic.rs` -- full implementation and inline doc comments
