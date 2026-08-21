# Demo Fleet: 20 Combinatorially Distinct Generated CLIs

Item #17 of the 25-prompt closure pass: a real end-to-end demonstration of
`clap-noun-verb-gen`'s `gen from-yaml` code generator, run against 20
combinatorially distinct `CliSpec` YAMLs.

## The combinatorial space

`generate_specs.py` produces 20 distinct specs by varying, in a real
Cartesian product plus 2 extra distinct points:

- **Noun count**: 1, 2, or 3 (`fleet`, `storage`, `network`, ...)
- **Verb count per noun**: 1, 2, or 3 (`status`, `sync`, `report`)
- **Argument style**: none, one required `String` positional, or one
  required `String` plus one boolean flag

Regenerate the specs:

```sh
python3 generate_specs.py
```

## Running the real generation + verification pipeline

```sh
sh generate_and_verify.sh /tmp/demo-fleet-out
```

This:

1. Builds `clap-noun-verb-gen` if needed.
2. Runs `clap-noun-verb-gen gen from-yaml <spec>.yaml -o <dir>` for all 20
   specs, real subprocess calls, tallying real pass/fail.
3. Wraps one representative variant (`demo-fleet-00`) in a real,
   hand-completed `Cargo.toml` and runs a real `cargo check` against it.

Real run in this session:

```
Generated: 20 ok, 0 failed (of 20 combinatorially distinct specs)
demo-fleet-00 compiles for real.
```

## Real findings from this exercise

- **`gen from-yaml` never emits a `Cargo.toml`** (unlike `gen scaffold
  --with-cargo`), so verifying real compilation requires hand-completing
  one. The real dependency set a generated `#[verb]`-using crate needs:
  `clap-noun-verb` (path dep), `clap-noun-verb-macros` (path dep, for the
  `#[verb]` attribute macro itself), `serde` (`features = ["derive"]`),
  `serde_json`, and **`linkme`** -- the `#[verb]` macro's expansion
  references `linkme::distributed_slice` unqualified, so any consuming
  crate must depend on `linkme` directly too, not just transitively
  through `clap-noun-verb`. None of this is documented anywhere in
  `clap-noun-verb-gen --help` or its module doc comment; a first-time user
  following `gen from-yaml`'s own guidance would hit `cannot find crate
  linkme` with no hint why.
- **A real module-name collision when 2+ nouns share a verb name**: the
  20-spec sweep includes several variants where multiple nouns each have a
  `sync`/`report`/`status` verb (e.g. `demo-fleet-15`: 2 nouns x 3 verbs).
  `gen from-yaml`'s output writes one flat `src/commands/<verb_name>.rs`
  per verb name, with no noun-scoping in the file/module path -- so two
  nouns sharing a verb name produce two conflicting `pub mod sync;`
  declarations in `src/commands/mod.rs`, a real
  `error[E0428]: the name 'sync' is defined multiple times`. This is a
  real, reproducible generator bug (not a demo-fleet authoring mistake);
  fixing it (noun-scoped submodules, e.g. `src/commands/<noun>/<verb>.rs`)
  is real, separate follow-up work on `clap-noun-verb-gen` itself, out of
  scope for this demonstration. `demo-fleet-00` (1 noun, 1 verb) was
  chosen as the compile-verified sample specifically because it has no
  verb-name collision to hit this bug.

## Files

- `generate_specs.py` -- generates the 20 `specs/*.yaml` CliSpecs
- `specs/*.yaml` -- the 20 committed, combinatorially distinct specs
- `generate_and_verify.sh` -- the real generation + compile-verification pipeline
