# How-To: Generate a cnv-any Manifest with ggen

**Problem**: You have an existing executable (any language) and want to
wrap it with `clap-noun-verb-any`, but manifests must be ggen-generated,
never hand-typed — see
[Explanation: The universal adapter and the OCEL corpus](../explanation/universal-adapter-and-ocel-corpus.md)
for why.

---

## Solution Overview

1. Write an `ontology.ttl` describing the wrapped program's real command
   surface using the same `cnv:Cli`/`cnv:Noun`/`cnv:Command`/`cnv:Argument`
   vocabulary `clap-noun-verb-schema-pack` already uses to compile real Rust
   CLIs.
2. Compose `clap-noun-verb-schema-pack` + `cnv-any-manifest-pack` in a
   `ggen.toml`.
3. Run `ggen sync run` to produce `cnv-any.json` — the exact
   `clap_noun_verb_deploy::CliSchema` JSON shape.

---

## Step-by-Step Instructions

### 1. Declare the CLI and its noun

```turtle
@prefix cnv: <https://clap-noun-verb.dev/ontology#> .

<#cli> a cnv:Cli ;
    cnv:binaryName "my-tool" ;   # must be a valid identifier — no dots
    cnv:crateName "my-tool" ;
    cnv:about "What this tool does" .

<#noun-mynoun> a cnv:Noun ;
    cnv:name "mynoun" ;          # NEVER "root" — the gates refuse it
    cnv:hasCommand <#cmd-verb> .
```

`cnv:name` must match `^[a-z][a-z0-9-]*$` (hyphens, not underscores).

### 2. Declare each command

```turtle
<#cmd-verb> a cnv:Command ;
    cnv:name "verb" ;
    cnv:about "What this command does" ;
    cnv:belongsToNoun <#noun-mynoun> ;
    cnv:hasBehavior <#behavior-verb> ;
    cnv:hasArgument <#arg-input> .

<#behavior-verb> a cnv:CustomBehavior .
```

Always use `cnv:CustomBehavior` for a wrapped command — the real logic
lives in the wrapped process, not in generated Rust.

### 3. Declare each argument

```turtle
<#arg-input> a cnv:Argument ;
    cnv:fieldName "input_path" ;   # underscores fine here
    cnv:valueKind "string" ;       # or "bool"/"i64"/"u64"/"f64"
    cnv:required true ;
    cnv:position 0 ;               # 0 = option (--flag); >=1 = positional
    cnv:longFlag "input-path" ;    # hyphenated cnv:name-style token
    cnv:action "value" .           # or "set_true"/"set_false"/"count"/"append"
```

`cnv:fieldName` allows underscores; `cnv:longFlag`/`cnv:name`-style tokens
must be hyphenated (`max-steps`, not `max_steps`) to satisfy the same
literal-value gate a Rust-generating pack uses.

### 4. Compose the packs

```toml
# ggen.toml
[packs.schema]
path = "../../../../ggen-marketplace/packs/clap-noun-verb-schema-pack"

[packs.manifest]
path = "../../../../ggen-marketplace/packs/cnv-any-manifest-pack"
```

### 5. Generate

```sh
ggen sync run
```

This writes `cnv-any.json` — regenerate it any time `ontology.ttl` changes;
never hand-edit the output.

---

## Complete Example

See `clap-noun-verb-any/examples/autofde_lab_planners/ontology.ttl` for a
real, non-trivial worked example: 5 commands, one with a `"required": false`
optional argument, wrapping a real Python Typer CLI with 57 registered
planners.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Gate refuses `cnv:Noun ; cnv:name "root"` | `root` is reserved by the shared schema-pack convention | Pick any other noun name; strip it in your wrapper shim if the real program doesn't expect a prefix |
| "SELECT overrides an existing variable using an expression" | An `OPTIONAL` pattern variable and a `(... AS ?x)` alias share the name `?x` | Use a distinct internal variable, then alias it — see `cnv-any-manifest-pack`'s `templates/cnv-any.json.tmpl` for the pattern |
| `cnv:binaryName`/`cnv:crateName` rejected | Contains a dot (e.g. `"calc.py"`) | Use the identifier form (`"calc"`) and let your wrapper resolve the real filename |
| `cnv:name "domain_arguments"` rejected | `cnv:name` requires hyphens, not underscores | Use `"domain-arguments"`; keep `cnv:fieldName` as `"domain_arguments"` if the wrapped program's real flag is hyphenated but the field itself is conceptually snake_case |
| Generated argv order doesn't match what you expected | Arguments are ordered by `(position, field)` — options (`position: 0`) come first, in field-alphabetical order, positionals last | Adjust your wrapper shim or test assertions to the real generated order rather than assuming source order |

---

## Related Guides

- [Tutorial 07: Wrapping Any Executable](../tutorial/07-wrapping-any-executable.md)
- [Reference: cnv-any](../reference/cnv-any.md)
- [Explanation: The universal adapter and the OCEL corpus](../explanation/universal-adapter-and-ocel-corpus.md)
