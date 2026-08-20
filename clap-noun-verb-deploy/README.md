# clap-noun-verb-deploy

Serve one `clap-noun-verb` command graph through MCP, HTTP, Kubernetes, and OCI/container deployment surfaces without rewriting consumer verb handlers.

The crate preserves a single authority path:

```text
CommandRegistry / Clap graph
  -> CliSchema
  -> validated Invocation
  -> AdmissionPolicy
  -> Gateway
  -> Executor
  -> ExecutionRecord
  -> ReplayVerification
```

Kubernetes and container modules are CONSTRUCT-only projections: they render validated manifests/build artifacts and never contact a cluster, registry, or container runtime.

## Features

- `mcp`: MCP 2026-07-28 JSON-RPC over stdio (`server/discover`, `ping`, `tools/list`, `tools/call`).
- `http`: health, readiness, schema, tool-list, and admitted invocation serving.
- `kubernetes`: deterministic hardened Deployment + Service YAML.
- `container`: deterministic multi-stage Dockerfile projection.

All features are enabled by default and can be selected independently.

## MCP 2026-07-28

The crate targets the modern stateless MCP era. It does not expose the legacy `initialize`/`initialized` session handshake. Every request must identify protocol revision `2026-07-28` in the reserved request `_meta` envelope and provide client capabilities. `server/discover` is available for explicit capability discovery. Successful responses carry `resultType: "complete"` and server identity metadata; list responses carry deterministic cache hints and tools are returned in stable name order.

```rust
use clap_noun_verb_deploy::{Deploy, ProcessExecutor};
use clap_noun_verb_deploy::mcp::McpServer;

# fn example(registry: &clap_noun_verb::CommandRegistry) -> Result<(), Box<dyn std::error::Error>> {
let deploy = Deploy::from_registry(registry);
let server = McpServer::new(
    "my-cli",
    env!("CARGO_PKG_VERSION"),
    deploy.into_schema(),
    ProcessExecutor::new("my-cli"),
);
server.serve_stdio(std::io::stdin().lock(), std::io::stdout())?;
# Ok(())
# }
```

For autonomous execution, use `McpServer::with_policy` with a bounded `AdmissionPolicy`. Protocol input cannot choose an arbitrary host executable: `ProcessExecutor` is pinned when constructed. `CliSchema` refuses unknown tools/arguments and type mismatches before the gateway is reachable; the policy can further restrict callable command paths.

## Autonomous / post-AGI operation

The post-AGI boundary is not a claim about intelligence level. It is an execution model designed for heterogeneous autonomous planners that may discover and compose tools without ambient authority.

```rust
use clap_noun_verb_deploy::{CommandAllowList, Gateway, Invocation, ProcessExecutor};

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let policy = CommandAllowList::default().allow(["cluster", "inspect"]);
let gateway = Gateway::new("ops-cli", ProcessExecutor::new("ops-cli"), policy);
let record = gateway.execute(Invocation::new(["cluster", "inspect", "prod"]))?;
assert!(record.verify_integrity());
# Ok(())
# }
```

The intended composition law is:

```text
observation -> tool/schema selection -> invocation construction
            -> admission/refusal -> explicit executor
            -> execution record -> deterministic replay verification
```

No model output, MCP request, HTTP request, Kubernetes manifest, or generated artifact has ambient execution authority. A transport can propose an invocation; policy admits it; an executor actuates it; the gateway manufactures an execution record.

Request-scoped environment mutation is refused by the default admission policy and independently refused by `ProcessExecutor`. An embedding application must explicitly allow a name at both boundaries before an invocation can supply it. This prevents a pinned executable from being undermined by loader/runtime environment variables.

The built-in fingerprint is deliberately only a deterministic corruption/replay guard. It is not a cryptographic signature and does not replace an external receipt authority. This keeps future cryptographic receipt systems, policy engines, or proof systems composable without falsely promoting a local hash to execution standing.

The core boundary is transport-neutral so future A2A, queue, workflow-engine, serverless, WASI, and agent-runtime adapters can reuse the same `CliSchema`, `Invocation`, `AdmissionPolicy`, `Gateway`, `ExecutionRecord`, and replay types.

## Kubernetes

```rust
use clap_noun_verb_deploy::kubernetes::KubernetesConfig;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut config = KubernetesConfig::new("my-cli", "ghcr.io/acme/my-cli:sha-123");
config.args = vec!["serve".into(), "http".into()];
print!("{}", config.render()?);
# Ok(())
# }
```

The projection validates Kubernetes identity/image/environment grammar before rendering and quotes command, argument, and environment values. Defaults include non-root execution, read-only root filesystem, dropped Linux capabilities, `RuntimeDefault` seccomp, disabled service-account-token automount, HTTP liveness/readiness probes, and a ClusterIP Service.

## Container projection

```rust
use clap_noun_verb_deploy::container::ContainerConfig;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut config = ContainerConfig::new("my-cli", "my-cli");
config.args = vec!["serve".into(), "http".into()];
print!("{}", config.render_dockerfile()?);
# Ok(())
# }
```

Dockerfile grammar-bearing fields are validated before rendering, while entrypoint arguments are JSON-escaped. Invalid fields produce typed construction errors rather than manufacturing a different build program.

## Receipts and replay

Every successful `Gateway::execute` manufactures an `ExecutionRecord` binding subject, admitted invocation, and observed execution. `ExecutionRecord::replay` re-executes the exact invocation and compares the observed result to the stored result. Replay mismatch remains an explicit failed verification rather than being collapsed into success.
