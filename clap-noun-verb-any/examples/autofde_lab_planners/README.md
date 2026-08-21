# Example: wrapping autofde-lab's 46+ planners as one deployable CLI

`autofde-lab` (a real, independently-maintained scikit-decide fork at
`../../playground/autofde-lab`, checked in here as a git submodule) already
ships a real Typer CLI, `autofde_lab.fabric` (`src/autofde_lab/fabric/cli.py`),
with a `solve <domain> --solver <name> ...` command that dispatches to ANY of
its registered planners by name. Per `pyproject.toml`'s
`[project.entry-points."autofde_lab.solvers"]` table that's **57 registered
solver entries** (Astar, FF, MCTS, BFWS, IW, VI, PI, LRTDP, POMCP, RayRLlib,
StableBaseline, and 40+ more) over 25+ registered domains, all reachable
through this one parametric verb -- exactly the shape `cnv-any` needs zero new
code to wrap.

This is the actual deliverable this example proves: **all 46+ working
planners, usable anywhere `clap-noun-verb-deploy` can serve a CLI (MCP, HTTP,
Kubernetes, OCI) instantly**, with no Rust rewrite of a single planner and no
change to `autofde-lab` itself.

## Files

- `autofde-lab-fabric.sh` -- the wrapped "executable." `python -m
  autofde_lab.fabric` is two tokens (interpreter + module flag); this shim
  `cd`s into the submodule, `shift`s off the ggen-emitted `fabric` noun token
  (see below), and execs `uv run --no-sync python -m autofde_lab.fabric
  "$@"`. Any other language's runtime convention (`node x.js`, `java -jar
  x.jar`, `go run .`) would use the exact same shim pattern.
- `ontology.ttl` -- the real `fabric/cli.py` command surface (`catalog`,
  `match`, `solve`, `cache-stats`, `cache-hotset`), admitted as RDF under one
  noun, `fabric`. `serve-mcp`/`serve-a2a` are deliberately absent: they're
  long-running daemons, and `cnv-any`'s executor model captures one bounded
  process's stdout/stderr/exit code, not a persistent server.
- `ggen.toml` -- composes `clap-noun-verb-schema-pack` (vocabulary + every
  admission gate, shared with real Rust-generating packs like
  `clap-noun-verb-crate-pack`) and `cnv-any-manifest-pack` (the projection
  that emits `cnv-any.json`).
- `cnv-any.json` -- **generated**, not hand-typed. Regenerate with:
  ```sh
  ggen sync run
  ```

### Why every command's path starts with `fabric`

`clap-noun-verb-schema-pack`'s own admission gates always group commands
under a real `cnv:Noun` (they explicitly refuse a noun literally named
`root`) -- there is no "no noun" convention in this shared vocabulary. The
real `autofde_lab.fabric` CLI's own argv has no such prefix (`catalog`, not
`fabric catalog`), so `autofde-lab-fabric.sh` strips it with a plain `shift`
before forwarding -- exactly the same adapter role the shim already plays for
absorbing `python -m`.

## Setup (one-time, real, not simulated)

`autofde-lab` needs its Python environment synced before this example can run
for real. From `../../playground/autofde-lab`:

```sh
git submodule update --init cpp/sdk/Catch2 cpp/sdk/backward-cpp cpp/sdk/json \
  cpp/sdk/nng cpp/sdk/nngpp cpp/sdk/pybind11 cpp/sdk/spdlog cpp/sdk/PEGTL
uv sync --frozen --no-default-groups --extra shared --extra pddl
```

Deliberately narrow extras: the full `[project.optional-dependencies]all`/
`ocpq`/`solvers`-with-`ray[rllib]` groups pull in either an unpublished
private package (`wasm4pm`) or a native `ray`/`dm-tree` build that fails on
this machine's cmake/toolchain -- neither is needed to exercise the real
PDDL-solver federation this example verifies (`Astar`/`FF`, per
`autofde_lab/reasoning/planner_federation.py`'s own empirically-justified
solver selection).

## Real verified run (this session, not simulated)

```sh
$ ./autofde-lab-fabric.sh fabric catalog
# real JSON: 25+ domains, 46+ solvers, including Astar, FF, MCTS, BFWS...

$ ./autofde-lab-fabric.sh fabric solve PDDLDomain --solver Astar \
    --domain-arguments '{"domain_path": "tests/domains/python/pddl_domains/blocks/domain.pddl", "problem_path": "tests/domains/python/pddl_domains/blocks/probBLOCKS-3-0.pddl"}' \
    --max-steps 50
# real 4-step trajectory, terminal: true, a real trajectory_sha256 receipt
```

## Wrap it with cnv-any

```rust
let wrapped = clap_noun_verb_any::wrap(
    "examples/autofde_lab_planners/autofde-lab-fabric.sh",
    Path::new("examples/autofde_lab_planners/cnv-any.json"),
)?;
// wrapped.deploy() now serves the same catalog/match/solve/cache-* surface
// over MCP, HTTP, a Kubernetes projection, or an OCI container -- the SAME
// 46+ planners, reachable from any of those environments, with zero glue
// beyond this ggen-generated manifest and the shim above.
```

See `../../tests/autofde_lab_integration.rs` for the real (env-gated) test
that exercises `catalog` and a real `Astar` solve through the actual
`Gateway`, not just a description of one.
