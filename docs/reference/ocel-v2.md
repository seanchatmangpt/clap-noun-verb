# Reference: OCEL 2.0 Event Logging

**Source**: `src/ocel.rs`, `clap-noun-verb-deploy/src/http.rs`, `clap-noun-verb-deploy/src/mcp.rs`, `clap-noun-verb-deploy/src/kubernetes.rs`
**Version**: 26.8.22

OCEL 2.0 (Object-Centric Event Log) is a standard JSON format for recording events
together with the objects they concern; `clap-noun-verb` uses it to record every
CLI invocation as a real, spec-shaped event log, with zero configuration required.

---

## Zero-configuration, always on

OCEL logging is not feature-gated and requires no opt-in. Every command dispatched
through the standard entry points — `CommandRegistry::execute_verb` (noun+verb
commands) and `execute_root_verb` (root-level verbs) — records one `cli_invocation`
event via `crate::ocel::record_invocation`, mirroring this crate's existing
"telemetry is always compiled, never feature-gated" precedent (`src/telemetry.rs`,
ADL-003).

Logging is a best-effort convenience layer: it can never fail or panic the
invocation it is describing. Any I/O failure downgrades to a `log::warn!` and the
command proceeds and returns normally.

---

## File path and env var override

| | |
|---|---|
| Default path | `.clap-noun-verb/ocel.json` (relative to the current working directory) |
| Env var override | `CLAP_NOUN_VERB_OCEL_PATH` |
| Fallback path | `$TMPDIR/clap-noun-verb-ocel.json` (or `/tmp/clap-noun-verb-ocel.json` if `TMPDIR` is unset) |

`primary_path()` resolves `CLAP_NOUN_VERB_OCEL_PATH` when set, else the default
relative path. If a write to the primary path fails for any reason (missing
permissions, a path component that isn't a directory, read-only filesystem), the
event is retried against `fallback_path()` instead. If both writes fail, the
failure is logged via `log::warn!` and the invocation proceeds normally — OCEL
logging never blocks or fails a command.

---

## Document and schema shape

The document is the standard four-array OCEL 2.0 top level: `objectTypes`,
`eventTypes`, `objects`, `events` (camelCase on the wire, produced by a manual
`Serialize`/`Deserialize` impl on `OcelDocument` so the Rust struct fields stay
idiomatic snake_case internally).

### Object types

| Object type | Meaning | Object id shape |
|---|---|---|
| `command` | The noun:verb pair being invoked | `"{noun}:{verb}"`, e.g. `"things:list"`, or `"_root:{verb}"` for root-level verbs |
| `process` | The OS process performing the invocation | `"proc-{pid}-{start_nanos_hex}"`, stable for the process's lifetime; every event recorded within one process shares the same `process` object |

### Event types

| Event type | Attributes | Relationships |
|---|---|---|
| `cli_invocation` | `noun` (string), `verb` (string), `success` (boolean), `duration_ms` (integer) | `regards` → the `command` object; `performed_by` → the `process` object |

`command` and `process` objects, and the `cli_invocation` event type, are
registered idempotently — running the same noun:verb pair twice does not create a
duplicate object, it appends a new event against the existing objects.

Example document after two invocations of `things list` and one of `things create`
in the same process:

```json
{
  "objectTypes": [
    { "name": "command", "attributes": [] },
    { "name": "process", "attributes": [] }
  ],
  "eventTypes": [
    { "name": "cli_invocation", "attributes": [] }
  ],
  "objects": [
    { "id": "things:list", "type": "command", "attributes": [] },
    { "id": "proc-6958-...", "type": "process", "attributes": [] },
    { "id": "things:create", "type": "command", "attributes": [] }
  ],
  "events": [
    {
      "id": "evt-...",
      "type": "cli_invocation",
      "time": "2026-08-20T00:00:00Z",
      "attributes": [
        { "name": "noun", "value": "things" },
        { "name": "verb", "value": "list" },
        { "name": "success", "value": true },
        { "name": "duration_ms", "value": 5 }
      ],
      "relationships": [
        { "objectId": "things:list", "qualifier": "regards" },
        { "objectId": "proc-6958-...", "qualifier": "performed_by" }
      ]
    }
  ]
}
```

---

## Reading it back programmatically

`src/ocel.rs` exposes the reading half of the API publicly (re-exported from the
crate root as `clap_noun_verb::{OcelDocument, OcelEvent, OcelObject}`):

```rust
use clap_noun_verb::ocel;

// Resolve the same paths the recorder uses.
let primary = ocel::primary_path();
let fallback = ocel::fallback_path();

// Load whichever one has data (an empty document if neither exists yet).
let doc = ocel::read_document(&primary)
    .or_else(|_| ocel::read_document(&fallback))
    .unwrap_or_default();

for event in &doc.events {
    println!("{} events of type {}", doc.events.len(), event.event_type);
}
```

`read_document` returns `Ok(OcelDocument::empty())` if the file does not exist yet,
and `Err` if the file exists but isn't valid OCEL 2.0 JSON.

---

## `/ocel` HTTP route (`clap-noun-verb-deploy`)

The `http` server surface (`clap-noun-verb-deploy/src/http.rs`) exposes a `GET
/ocel` route that returns the current OCEL document as JSON. It checks the primary
path first, then the fallback path, returning `OcelDocument::empty()` (a
spec-shaped document with empty arrays) if neither file has been written yet — a
freshly-started server with no admitted invocations still returns valid,
spec-shaped OCEL JSON rather than a 404 or an error.

```
GET /ocel
200 OK
{"objectTypes": [...], "eventTypes": [...], "objects": [...], "events": [...]}
```

---

## MCP equivalent (`clap-noun-verb-deploy`)

The MCP stdio adapter (`clap-noun-verb-deploy/src/mcp.rs`) exposes the same
document as an MCP resource at the URI `clap-noun-verb://ocel`. It is listed for
discovery and readable via the standard MCP resource-read flow, resolving the
document the same way as the HTTP route (primary path, then fallback, then an
empty spec-shaped document).

---

## Kubernetes: automatic `emptyDir` at `/tmp`

`clap-noun-verb-deploy/src/kubernetes.rs` defaults
`KubernetesConfig::read_only_root_filesystem` to `true`. A read-only root
filesystem makes `/tmp` unwritable by default, which would otherwise break the
OCEL fallback path (`$TMPDIR/clap-noun-verb-ocel.json`) whenever the primary path
is also unwritable inside the container.

When `read_only_root_filesystem` is `true`, the generated Kubernetes manifest
automatically adds a `volumeMounts` entry mounting `/tmp` and a matching `volumes`
entry declaring it as `emptyDir: {}` — giving the container a writable `/tmp` for
the OCEL fallback path (and any other library that needs scratch space) without
weakening the read-only root filesystem hardening itself. No configuration is
required to get this; it is derived automatically from
`read_only_root_filesystem`.

---

## See Also

- `docs/reference/api/telemetry.md` — the existing always-on telemetry layer this
  feature's "always compiled, never feature-gated" precedent follows
- `docs/reference/error-codes.md` — `NounVerbError` variants used by
  `ocel::read_document`
- `src/ocel.rs` — full implementation and inline doc comments
