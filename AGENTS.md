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

- RDF/Turtle is semantic authority for nouns, verbs, arguments, constraints, and
  lifecycle facts.
- SPARQL is selection and inference law.
- Tera is the terminal projection calculus.
- `ggen.toml` binds admitted semantic inputs to bounded filesystem consequences.
- `package.toml` is transport metadata only. It MUST NOT redefine semantic law.
- Consumer handlers own domain behavior. Generated wrappers own the CLI interface.
- Generated files MUST carry a generated banner and MUST NOT be hand-edited.
- `ggen.lock` and `.ggen-v2/receipt*.json*` are engine-produced evidence surfaces.
  They MUST NOT be fabricated or manually repaired.

## Required execution ladder

For changes affecting the CLI compiler, macros, ontology, queries, templates, or
pack transport, execute the narrowest applicable rung and then expand:

1. static contract verifier
2. ggen dry run / graph validation
3. first `ggen sync run`
4. `ggen receipt verify`
5. generated consumer `cargo check`
6. generated consumer `cargo test`
7. CLI `--help` and successful route execution
8. required-argument refusal
9. collision / invalid-identifier falsifier
10. second sync with byte-identical outputs and verified receipt
11. workspace library tests, then wider integration tests

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

## ggen compatibility boundary

The canonical ggen verifier for this repository is pinned by
`.github/workflows/ggen-authority.yml`. A ggen upgrade is an admitted contract
change: update the pin, execute the full ladder, retain negative falsifiers, and
record the exact head in the pull-request receipt.

The public vocabulary remains `http://clap-noun-verb.io/ontology#` until a
separate, explicit ontology migration proves equivalence and compatibility.
Repository identity may be bound to `chatmangpt.com`; identity binding does not
silently rewrite public RDF terms.
