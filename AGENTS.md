# AGENTS.md — clap-noun-verb Verification Constitution

This file is the normative agent contract for this repository.

## Standing law

A repository state is one of `PARTIAL_ALIVE`, `ALIVE`, `BLOCKED`, `BUILD_BROKEN`,
`UNKNOWN`, or `UNSUPPORTED`. `ALIVE` requires observed execution evidence. A log,
a passing assertion, or a generated file alone is not a receipt.

The project follows the lawful pipeline:

`parse → route → admit/refuse → diagnose/repair → actuate → receipt → replay`

The hard invariant is **zero unreceipted actuation**.

## Authority map

- RDF/Turtle is semantic authority for nouns, verbs, arguments, constraints,
  capabilities, default verbs, and lifecycle facts.
- SPARQL is selection, inference, and gate law. Automatic sync gates live only
  in `gates/*.rq` and are referenced through `[validation].gates`.
- Tera is the terminal projection calculus.
- `ggen.toml` binds admitted semantic inputs to bounded filesystem consequences.
- `package.toml` is transport metadata only. It MUST NOT redefine semantic law.
- Consumer handlers own domain behavior. Generated wrappers own the CLI interface.
- Generated files MUST carry a generated banner and MUST NOT be hand-edited.
- `ggen.lock` and `.ggen-v2/receipt*.json*` are engine-produced evidence surfaces.
  They MUST NOT be fabricated or manually repaired.
- Authoritative project execution receipts are OCEL 2.0 and MUST be verified by
  the pinned wasm4pm boundary before standing is promoted.

## CI authority boundary

`.github/workflows/verify.yml` is the sole GitHub Actions verification surface.
It is read-only with respect to repository and external system state: CI may
observe, compile, test, manufacture bounded local evidence, verify receipts, and
upload verifier artifacts. CI MUST NOT push repairs, publish crates, deploy,
create releases, or otherwise perform consequential release actuation.

Release DO must re-enter through the admitted ecosystem release-governance / BRCE
boundary and emit its own consequence-bound receipt. A green CI run is admission
evidence for release; it is not authority to actuate release.

## Capability crown

The admitted feature crown contains 15 capabilities and 45 proof surfaces:
unit, integration, and replay for each capability. The ontology-first source is
`packs/clap-noun-verb-capability-pack/ontology.ttl`; generated consumer files are
consequences and are never manually authored.

Every Cargo feature named in the crown MUST be a real compilation surface. A
check-cfg placeholder, disabled PR workflow, unconditional `not implemented`
error, or documentation-only route is `UNSUPPORTED` or `BUILD_BROKEN`, never
`ALIVE`.

Capability standing is evidence-derived:

- `UNKNOWN`: no observed proof surface.
- `PARTIAL_ALIVE`: at least one but not all required surfaces is observed and replayed.
- `ALIVE`: every declared surface is observed and replayed with a non-empty receipt.
- `BLOCKED`, `BUILD_BROKEN`, and `UNSUPPORTED`: typed boundary states; none collapse
  into success.

## Required execution ladder

For changes affecting the CLI compiler, macros, ontology, queries, templates,
capabilities, or pack transport, execute the narrowest applicable rung and expand:

1. static contract verifier
2. ggen graph validation and external SPARQL gates
3. first `ggen sync run`
4. `ggen receipt verify`
5. generated consumer `cargo check`
6. generated consumer `cargo test`
7. baseline and individual Cargo feature compilation
8. frontier/federation integration tests
9. complete `--all-features` execution
10. clippy, rustdoc, and benchmark-build proofs
11. negative gate and identifier falsifiers
12. second sync with byte-identical outputs and verified ggen receipt
13. OCEL 2.0 receipt manufacture from observed results
14. pinned wasm4pm OCEL verification and canonicalization
15. machine-readable verifier report bound to the exact head SHA

`cargo test --lib` is load-bearing because the `linkme` distributed registry is
part of the command-discovery proof.

## Refusals and exclusions

- No mocks or stubs on the primary evidence path.
- No fabricated receipts, hashes, telemetry, or generated outputs.
- No `TODO`, `FIXME`, `todo!()`, or `unimplemented!()` in admitted production work.
- No direct network, deployment, Git, or filesystem actuation from ontology,
  query, template, or read-only observer code.
- No output path may escape the repository root.
- No consumer may depend on private pack internals.
- No direct edit of generated wrappers, generated proof tests, lockfiles, or
  receipt ledgers.
- No status promotion from documentation or route logs.
- No Python or shell-generated digest is receipt authority. Static scripts may
  diagnose contracts; wasm4pm-verified OCEL carries execution receipt standing.

## ggen compatibility boundary

The canonical ggen verifier for this repository is pinned by
`.github/workflows/verify.yml`. A ggen upgrade is an admitted contract change:
update the pin, execute the full ladder, retain negative falsifiers, and record
the exact head in the pull-request receipt.

The root public vocabulary remains `http://clap-noun-verb.io/ontology#` until a
separate ontology migration proves equivalence and compatibility. The capability
crown uses the admitted ChatmanGPT namespace. Repository identity binding does not
silently rewrite legacy public RDF terms.
