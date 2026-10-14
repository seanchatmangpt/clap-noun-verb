# 07. Wrapping Any Executable with cnv-any

**Time**: 15-20 minutes
**Prerequisites**: Tutorials 01-04, a `clap-noun-verb-deploy` checkout

---

## What You'll Build

A deployable CLI wrapper around a plain shell script — no Rust rewrite, no
Clap parsing, using the real `clap-noun-verb-any` crate end to end. By the
end you'll have wrapped `greet.sh` and run it through the real `Gateway`,
getting the same MCP/HTTP/Kubernetes/OCI deploy surface a native
`clap-noun-verb` binary gets.

---

## Step 1: The target program

`clap-noun-verb-any/examples/shell_script/greet.sh` is the trivial fixture
this tutorial uses:

```sh
#!/usr/bin/env bash
echo "Hello, ${!#}!"
```

It reads its *last* argument — deliberately agnostic to how many noun/verb
segments precede it, since that's ggen's job to add, not the script's.

## Step 2: Describe its command surface as RDF

`ontology.ttl` admits one `cnv:Cli` under noun `cli`, command `greet`, with
one required positional string argument `name`:

```turtle
@prefix cnv: <https://clap-noun-verb.dev/ontology#> .

<#cli> a cnv:Cli ;
    cnv:binaryName "greet" ;
    cnv:crateName "greet" ;
    cnv:about "Greets someone by name" .

<#noun-cli> a cnv:Noun ;
    cnv:name "cli" ;
    cnv:hasCommand <#cmd-greet> .

<#cmd-greet> a cnv:Command ;
    cnv:name "greet" ;
    cnv:about "Print a greeting" ;
    cnv:belongsToNoun <#noun-cli> ;
    cnv:hasBehavior <#behavior-greet> ;
    cnv:hasArgument <#arg-name> .

<#behavior-greet> a cnv:CustomBehavior .

<#arg-name> a cnv:Argument ;
    cnv:fieldName "name" ;
    cnv:valueKind "string" ;
    cnv:required true ;
    cnv:position 1 .
```

`cnv:CustomBehavior` marks this command's real behavior as living in the
wrapped process, not generated Rust — the honest declaration for anything
`cnv-any` wraps.

## Step 3: Generate the manifest — never hand-type it

```sh
cd clap-noun-verb-any/examples/shell_script
ggen sync run
```

This composes `clap-noun-verb-schema-pack` (the same vocabulary and 16
admission gates that compile real Rust CLIs) with `cnv-any-manifest-pack`
(one additional projection that emits `cnv-any.json` instead of Rust
source). The output is the exact `CliSchema` JSON shape
`clap_noun_verb_deploy::CliSchema::from_manifest_path` expects — see
[How-To: Generate a cnv-any manifest with ggen](../howto/generate-cnv-any-manifest.md)
if a gate rejects your own ontology.

## Step 4: Wrap it

```rust
use clap_noun_verb_any::wrap;
use std::path::Path;

let wrapped = wrap(
    "examples/shell_script/greet.sh",
    Path::new("examples/shell_script/cnv-any.json"),
)?;
```

`wrapped.deploy()` is the *same* `Deploy` type `Deploy::from_registry`
produces for a native binary. Every existing protocol surface —
`mcp.rs`/`http.rs`/`kubernetes.rs`/`container.rs` — works against it
unchanged.

## Step 5: Run it through the real Gateway and check OCEL parity

Invoking `wrapped` through `Gateway::execute` does two things: it runs the
real subprocess, and it calls `clap_noun_verb::ocel::record_invocation` on
the script's behalf using the manifest's noun/verb — producing an OCEL
event structurally identical to one a native `clap-noun-verb` binary would
emit itself. See
[Explanation: The universal adapter and the OCEL corpus](../explanation/universal-adapter-and-ocel-corpus.md)
for why that identical shape is the point, not an incidental detail.

## What's next

- Wrap a heavier real target: `clap-noun-verb-any/examples/autofde_lab_planners/`
  wraps a real Python CLI exposing 57 registered planners through this same
  pattern.
- [Reference: cnv-any](../reference/cnv-any.md) for the full API surface.
- [How-To: Generate a cnv-any manifest with ggen](../howto/generate-cnv-any-manifest.md)
  for writing your own ontology against a new target.
