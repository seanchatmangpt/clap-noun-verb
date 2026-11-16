# Schema & Introspection in clap-noun-verb

**Version**: 26.9.1
**Source**: `src/registry.rs` (`collect_tools_from_cmd`, `ToolDefinition`)

`clap-noun-verb` exposes a CLI's command tree as a machine-readable tool schema so that
LLM agents and orchestrators can discover and call commands. This is driven by the global
`--introspect` flag; there is no separate schema-registration step — the schema is derived
from the registered verbs and their argument metadata.

> **Note:** Schema generation here is the JSON-Schema/tool-calling surface. It is not RDF
> SHACL validation — earlier drafts of this page described a SHACL/Turtle layer and an
> output-validation-hook API that are not part of the shipped framework.

---

## Output types must derive `Serialize`

Every verb returns `Result<T>` where `T: serde::Serialize`. This is enforced at
compile time by the `#[verb]` macro (see [Verb Macro](api/verb-macro.md)). The serialized
result is what the framework prints (JSON by default) and what downstream agents consume.

```rust
use serde::Serialize;
use clap_noun_verb::Result;

#[derive(Serialize)]
struct Status { running: bool, uptime: u64 }

#[clap_noun_verb_macros::verb("status")]
fn status() -> Result<Status> {
    Ok(Status { running: true, uptime: 3600 })
}
```

---

## JSON Schema Introspection (`--introspect`)

Passing the global `--introspect` flag instructs the CLI to output all registered
commands as a JSON array of tool definitions and exit. The output is produced by
`collect_tools_from_cmd` over the built `clap::Command` tree and serialized from the
`ToolDefinition` type.

### Query

```bash
myapp --introspect
```

### Schema shape

Each element is a `ToolDefinition`:

- `name` — the underscore-joined command path (e.g. `services_status`)
- `description` — the command's help/about text
- `parameters` — a JSON-Schema object:
  - `type`: always `"object"`
  - `properties`: map of argument name → `{ type, description?, default?, items? }`
  - `required`: list of required argument names

This is compatible with OpenAI, Anthropic, and Model Context Protocol tool-calling schemas.

### Example output

```json
[
  {
    "name": "services_status",
    "description": "Get the status of a registered service",
    "parameters": {
      "type": "object",
      "properties": {
        "service_name": {
          "type": "string",
          "description": "Name of the target service to query"
        },
        "verbose": {
          "type": "boolean",
          "description": "Enable verbose output logging",
          "default": "false"
        },
        "timeout": {
          "type": "string",
          "description": "Timeout duration in seconds",
          "default": "30"
        }
      },
      "required": [
        "service_name"
      ]
    }
  },
  {
    "name": "calculator_add",
    "description": "Perform basic integer addition",
    "parameters": {
      "type": "object",
      "properties": {
        "left":  { "type": "string", "description": "The left operand value" },
        "right": { "type": "string", "description": "The right operand value" }
      },
      "required": ["left", "right"]
    }
  }
]
```

Only leaf commands (verbs with no further subcommands) become tools; intermediate nouns
are traversed to build the `name` path.

---

## See Also

- [Verb Macro](api/verb-macro.md) — how arguments and descriptions feed the schema
- [Argument Attributes](api/arg-attributes.md) — `#[arg]` tags that shape `properties`
- [Advanced Features](api/advanced-features.md) — using `--introspect` with agents
- [Error Codes](error-codes.md) — `--structured-errors` machine-readable error format
