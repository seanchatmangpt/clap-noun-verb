# Example: wrapping a plain Python script (no shim needed)

Unlike `autofde_lab.fabric` (invoked as `python -m autofde_lab.fabric`, two
tokens needing a shim -- see `../autofde_lab_planners/`), `calc.py` has a
`#!/usr/bin/env python3` shebang and is `chmod +x`, so it's directly
executable as a single token, exactly like a compiled binary or a shell
script. This is the simplest real case of "any language": as long as the OS
can exec it as one program, `cnv-any` doesn't care what runtime is behind it.

`ontology.ttl` admits a real `cnv:Cli` with noun `calc` and two commands,
`add`/`multiply`, each with two required positional integer arguments;
`cnv-any.json` is **generated** from it by `cnv-any-manifest-pack` (see
`ggen.toml`), never hand-typed:

```sh
ggen sync run
```

```sh
$ ./calc.py calc add 2 3
5
$ ./calc.py calc multiply 4 5
20
```

```rust
let wrapped = clap_noun_verb_any::wrap(
    "examples/python_argparse_cli/calc.py",
    Path::new("examples/python_argparse_cli/cnv-any.json"),
)?;
```

`calc.py` strips the ggen-emitted `calc` noun token itself before handing the
rest to `argparse` (`sys.argv = [sys.argv[0]] + sys.argv[2:]`) -- the same
adapter role `autofde-lab-fabric.sh`'s shim plays for the heavier example,
just inline since this script is small enough to own the translation
directly.
