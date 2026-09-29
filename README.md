# clap-noun-verb

[![Crates.io](https://img.shields.io/crates/v/clap-noun-verb)](https://crates.io/crates/clap-noun-verb)
[![Documentation](https://docs.rs/clap-noun-verb/badge.svg)](https://docs.rs/clap-noun-verb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

**Declarative noun-verb CLI framework for type-safe, agent-ready command registration.**

## What's New in 26.7.62

- **Ontology-first capability crown** — ggen manufactures a canonical catalog of
  15 Cargo capabilities and 45 required proof surfaces from RDF authority.
- **Completed frontier features** — semantic composition, discovery, learning,
  reflexive verification, economic simulation, fractal composition, executable
  specifications, PQC policy selection, and bounded federation are real compile
  and execution surfaces rather than check-cfg placeholders.
- **Evidence-backed standing** — capability packages remain `UNKNOWN`,
  `PARTIAL_ALIVE`, `BLOCKED`, `BUILD_BROKEN`, or `UNSUPPORTED` until observed and
  replayed proof surfaces justify `ALIVE`.
- **Exact-head verification** — a 21-cell Cargo feature matrix, ggen receipt/replay,
  negative SPARQL gates, and wasm4pm-verified OCEL 2.0 receipts bind evidence to
  the exact candidate SHA.
- **Optional OpenTelemetry** — the `otel` feature activates dispatch spans and a
  tracer-provider lifecycle without increasing the zero-feature core.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = "26.7.62"
clap-noun-verb-macros = "26.7.62" # For proc-macros
```

Or with `cargo add`:

```bash
cargo add clap-noun-verb clap-noun-verb-macros
```

## The Noun-Verb Pattern

A **noun-verb command** separates domain concepts from actions. Instead of flat command names like `login` or `logout`, organize commands hierarchically:

```
myapp session login          # noun: session, verb: login
myapp session verify         # noun: session, verb: verify
myapp user create --name Bob # noun: user, verb: create (with flags)
```

This pattern naturally maps to your domain model:

- **Noun** = a resource or entity (user, session, config)
- **Verb** = an action on that noun (create, list, delete, verify)

The `#[noun]` and `#[verb]` proc-macros auto-discover and register commands at compile time. No manual routing.

## Quick Start

Create a new Rust project:

```bash
cargo new myapp && cd myapp
cargo add clap-noun-verb clap-noun-verb-macros linkme serde
```

`linkme` powers `#[verb]`'s compile-time auto-registration (the macro expands to
`#[linkme::distributed_slice(...)]` directly in your crate, so `linkme` must be
a direct dependency of your binary, not just a transitive one of
`clap-noun-verb`).

Add to `src/main.rs`:

```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct CalcResult {
    result: i32,
}

fn add(x: i32, y: i32) -> i32 {
    x + y
}

#[verb("add", "calc")]
fn cmd_add(x: i32, y: i32) -> Result<CalcResult> {
    Ok(CalcResult { result: add(x, y) })
}

/// Multiply two numbers
///
/// # Arguments
/// * `profile_id` - Profile to use [default: default]
#[verb("multiply", "calc")]
fn cmd_multiply(x: i32, y: i32, profile_id: Option<String>) -> Result<CalcResult> {
    let _profile_id = profile_id;
    Ok(CalcResult { result: x * y })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
```

Run it:

```bash
cargo run -- calc add --x 2 --y 3
cargo run -- calc multiply --x 4 --y 5 --profile-id premium
cargo run -- --help
```

## Core Feature Matrix

| Capability | Availability | Example |
|---|---|---|
| Noun-verb auto-discovery | Core | `#[verb("add")]` registers `calc add` |
| Kebab-case normalization | Core | `--profile-id`, `--dry-run` |
| JSON output | Core | Agent-ready serialized consequences |
| Command chaining | Core | `session login ++ session verify @{1.token}` |
| Stdin extraction | Core | `@-`, `@-::json.path` |
| Dynamic completions | Core | `myapp completions zsh` |
| LLM introspection | Core | `myapp --introspect` |
| Structured errors | Core | Typed JSON errors and action templates |
| Graph and capability registry | Core | RDF graph operations and evidence standing |
| Interactive shell | `repl` | `Repl::new(registry).run()` |
| OpenTelemetry dispatch spans | `otel` | `otel::init_tracer("service")` |
| Bounded federation | `federated-network` | Advertise, resolve, and manufacture invocation envelopes |

## Cargo Features

The crate has **zero default features**. The core includes noun-verb dispatch,
chaining, stdin extraction, completions, introspection, structured errors,
validators, graph operations, capability standing, diagnostics, and RDF/ggen
synchronization.

| Feature | Family | Enables |
|---|---|---|
| `process-data` | Core extension | Process-data pipeline hooks |
| `autonomic` | Operations | Autonomic CI/CD policies; implies `process-data` |
| `contrib` | Extension | Contributor helpers; implies `process-data` |
| `repl` | Interface | Interactive rustyline REPL |
| `otel` | Observability | OpenTelemetry-backed tracing spans |
| `federated-network` | Integration | Bounded peer, capability, resolver, and envelope APIs |
| `meta-framework` | Semantic | Layer and invariant admission with standing |
| `rdf-composition` | Semantic | Deterministic duplicate-free semantic fragments |
| `fractal-patterns` | Semantic | Typed adjacent-level composition |
| `discovery-engine` | Intelligence | Deterministic capability indexing and search |
| `learning-trajectories` | Intelligence | Bounded replayable score trajectories |
| `economic-sim` | Intelligence | Vickrey auctions and deterministic allocation |
| `reflexive-testing` | Quality | Machine-readable replay-aware verifier reports |
| `quantum-ready` | Quality | Crypto-agility policy for ML-KEM, ML-DSA, and SLH-DSA |
| `executable-specs` | Quality | Executable Given/When/Then specifications |
| `frontier-semantic` | Aggregate | All semantic features |
| `frontier-intelligence` | Aggregate | All intelligence features |
| `frontier-quality` | Aggregate | All quality features |
| `frontier-all` | Aggregate | Semantic, intelligence, quality, and federation features |

Compile the entire crown:

```bash
cargo check --all-targets --all-features
cargo test --all-features
```

## Capability Standing

`CapabilityPackage` records an ontology-owned default verb, dependency closure,
and executable `ProofSurface` values. Standing is derived rather than asserted:

```rust
use clap_noun_verb::{CapabilityPackage, CapabilityStanding, ProofSurface};

let mut capability = CapabilityPackage::new(
    "receipt-verify",
    "Receipt Verification",
    "26.7.62",
    "Verifies one admitted execution receipt",
)
.with_default_verb("verify");

capability.record_proof(ProofSurface::new(
    "unit-contract",
    "unit",
    "receipt:unit:001",
    true,
    true,
))?;

assert_eq!(capability.standing, CapabilityStanding::Alive);
# Ok::<(), String>(())
```

`ALIVE` is refused when any declared proof surface is unobserved, unreplayed, or
missing its receipt identifier.

## ggen Authority and Replay

The root `ggen.toml` preserves the public noun-verb vocabulary. The dedicated
`packs/clap-noun-verb-capability-pack` owns the 15-capability crown:

```
RDF/Turtle → SPARQL selection/gates → Tera projection → bounded files
           → ggen receipt verify → byte-identical replay → OCEL receipt
```

The automatic law is external SPARQL under `gates/*.rq`. ASK `true` means a
violation. Generated consumers and `.ggen-v2` receipts are consequences and must
not be hand-edited.

## Additional Library Modules

- **Validators** (`validators`) — email, IPv4/6, URL, port, path, length, and regex checks.
- **Graph** (`graph`) — load N-Triples, query by subject/predicate, and validate RDF.
- **Capability** (`capability`) — deterministic registry, dependencies, standing, and proof surfaces.
- **Diagnostics** (`diagnostics`) — registry-backed health checks.
- **RDF ↔ ggen** (`ggen_to_rdf`, `rdf_to_ggen`, `ontology_sync`) — code/ontology synchronization.
- **Async verbs** (`async_verb`) — async command handlers.
- **Frontier** (`frontier`) — bounded semantic, intelligence, quality, federation, and simulation primitives.

## Learn More

### Tutorials
- [Domain Separation Architecture](docs/tutorial/01-domain-separation.md)
- [Tutorial Series](docs/tutorial/README.md)

### How-Tos
- [How-To Guides](docs/howto/README.md)
- [Production Guides](docs/howto/production/deployment.md)

### Reference
- [#[verb] Macro API](docs/reference/api/verb-macro.md)
- [API Reference](docs/reference/README.md)
- [Advanced Features](docs/reference/api/advanced-features.md)
- [API Catalog](docs/reference/api-catalog.md)

### Architecture
- [ggen Authority Contract](docs/GGEN_AUTHORITY.md)
- [Verification Constitution](AGENTS.md)
- [Changelog](CHANGELOG.md)

## Contributing

Issues and pull requests are welcome at the repository.

## License

Licensed under either Apache License 2.0 or MIT license at your option.
