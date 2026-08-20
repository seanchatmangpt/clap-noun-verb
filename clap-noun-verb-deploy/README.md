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
```

Kubernetes and container modules are CONSTRUCT-only projections: they render manifests/build artifacts and never contact a cluster, registry, or container runtime.

## Features

- `mcp`: MCP JSON-RPC over stdio (`initialize`, `ping`, `tools/list`, `tools/call`).
- `http`: health, readiness, schema, tool-list, and admitted invocation serving.
- `kubernetes`: deterministic hardened Deployment + Service YAML.
- `container`: deterministic multi-stage Dockerfile projection.

All features are enabled by default and can be selected independently.

## MCP

```rust
use clap_noun_verb_deploy::{AdmitValidated, Deploy, Gateway, ProcessExecutor};
use clap_noun_verb_deploy::mcp::McpServer;

# fn example(registry: &clap_noun_verb::CommandRegistry) -> Result<(), Box<dyn std::error::Error>> {
let deploy = Deploy::from_registry(registry);
let gateway = Gateway::new("my-cli", ProcessExecutor::new("my-cli"), AdmitValidated);
let server = McpServer::new("my-cli", env!("CARGO_PKG_VERSION"), deploy.into_schema(), gateway);
server.serve_stdio(std::io::stdin().lock(), std::io::stdout())?;
# Ok(())
# }
```

Protocol input cannot choose an arbitrary host executable: `ProcessExecutor` is pinned when constructed. `CliSchema` refuses unknown tools/arguments and type mismatches before the gateway is reachable; the admission policy can further restrict callable command paths.

## Kubernetes

```rust
use clap_noun_verb_deploy::kubernetes::KubernetesConfig;

let mut config = KubernetesConfig::new("my-cli", "ghcr.io/acme/my-cli:sha-123");
config.args = vec!["serve".into(), "http".into()];
print!("{}", config.render());
```

The default projection runs non-root, uses a read-only root filesystem, drops Linux capabilities, exposes HTTP liveness/readiness probes, and creates a ClusterIP Service.

## Receipts and replay

Every successful `Gateway::execute` manufactures an `ExecutionRecord` binding subject, admitted invocation, and observed execution. `ExecutionRecord::replay` re-executes the exact invocation and compares the result. The built-in deterministic fingerprint is a local integrity/replay guard, not a cryptographic signature or external receipt authority.
