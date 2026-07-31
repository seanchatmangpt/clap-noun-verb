# ggen authority and replay contract

## Preserved system

`clap-noun-verb` already separates the generated CLI interface from
consumer-owned behavior. The ontology declares nouns, verbs, arguments, Rust
field identities, return types, and handler names. SPARQL selects admitted rows.
Tera renders thin `#[verb]` wrappers. Consumer handlers implement domain logic.
That seam remains the Chesterton fence.

## Calculus

`O → O* → μ₁ → μ₃ → A → R`

- `O`: authored ontology and pack inputs.
- `O*`: inputs admitted by syntax, bounded-path, identifier, and collision gates.
- `μ₁`: idempotent SPARQL `CONSTRUCT` normalization.
- `μ₃`: SELECT + Tera projection into bounded Rust source files.
- `A`: generated wrappers and module aggregation.
- `R`: ggen receipt binding graph hash, input closure, decisions, pack hashes,
  and output BLAKE3 hashes.

The receipt is replayable evidence. It is not replaced by `cargo test passed`.

## Ownership

| Surface | Authority | Edit policy |
|---|---|---|
| `ontology/**/*.ttl` | semantic authority | authored |
| `queries/**/*.rq` | selection / validation law | authored |
| `templates/**/*.tera` | terminal projection law | authored |
| `ggen.toml` | admitted mapping and bounded writes | authored |
| `package.toml` | pack transport | authored, no semantic duplication |
| `examples/greet-demo/src/verbs/**` | generated consequence | ggen only |
| `.ggen-v2/receipt.json` | latest execution receipt | ggen only |
| `.ggen-v2/receipt-log.jsonl` | append-only execution history | ggen only |
| consumer `handlers.rs` | domain behavior | authored |

## Gate semantics

Current ggen gate law is exact:

- ASK `true` means a violation exists and generation is refused.
- ASK `false` means no violation was found.
- SELECT with one or more rows means violations exist.

The canonical field-name collision ASK therefore directly matches colliding
pairs. It MUST NOT wrap the collision pattern in `FILTER NOT EXISTS`, which
would invert the verdict.

## Verification

```bash
python3 scripts/verify_ggen_contract.py
python3 -m unittest scripts/test_verify_ggen_contract.py -v

ggen sync run
ggen receipt verify

cd examples/greet-demo
ggen sync run
ggen receipt verify
cargo check
cargo test
cargo run -- tool greet --name Sean
```

Run sync twice and compare every declared output byte-for-byte. Then execute the
collision falsifier and require a non-zero exit. Only the complete observed
ladder may promote the integration to `ALIVE`.
