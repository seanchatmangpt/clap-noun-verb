# Example: wrapping a plain shell script

The simplest possible `cnv-any` target: no runtime, no interpreter startup
convention to reason about, just an executable file the OS can run directly.

Files:
- `greet.sh` — a trivial, real, executable script: `greet.sh <path...> <name>`
  prints `Hello, <name>!` and exits 0 (it reads the *last* argument so it
  doesn't care how many noun/verb segments ggen puts in front of it).
- `ontology.ttl` — this CLI's real command surface as RDF: one `cnv:Cli`
  under noun `cli`, command `greet`, with one required positional string
  argument `name`.
- `ggen.toml` — composes `clap-noun-verb-schema-pack` (vocabulary + every
  admission gate) and `cnv-any-manifest-pack` (the projection that emits
  `cnv-any.json`).
- `cnv-any.json` — **generated**, not hand-typed. Regenerate with:
  ```sh
  ggen sync run
  ```

`cnv-any.json` is the exact `clap_noun_verb_deploy::CliSchema` JSON shape,
but it is manufactured from the admitted RDF graph above the same way
`clap-noun-verb-crate-pack` manufactures a real Rust CLI's `Cargo.toml`/
`src/main.rs` from the same vocabulary — never hand-authored.

## Run it

```rust
let wrapped = clap_noun_verb_any::wrap(
    "examples/shell_script/greet.sh",
    Path::new("examples/shell_script/cnv-any.json"),
)?;
// wrapped.deploy() and wrapped.executor() now work with every existing
// clap-noun-verb-deploy surface (Gateway, McpServer, HttpServer, Kubernetes
// projection, container projection) exactly as they would for a native
// clap-noun-verb binary.
```
