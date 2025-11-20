# Diataxis: V5 Semantic CLI Reference

**Framework**: Diataxis Information-Oriented Documentation
**Audience**: Developers building with v5, looking for precise API details
**Purpose**: Complete API reference, command structure, data formats
**Format**: Structured reference for lookup

---

## Command Invocation Format

### Basic Syntax

```
myapp [GLOBAL_OPTIONS] <NOUN> [NOUN_OPTIONS] <VERB> [VERB_OPTIONS]
```

### Global Options

| Option | Format | Description | v5 Specific |
|--------|--------|-------------|-------------|
| `--help` | Flag | Show help text | N/A (v4 only) |
| `--version` | Flag | Show version | N/A |
| `--machine` | Flag | Use v5 machine mode | ✅ v5 only |
| `--introspect` | Flag | Get capability metadata | ✅ v5 only |
| `--json` | Flag | Output as JSON (requires input as JSON) | ✅ v5 |
| `--stream` | Flag | Stream results (long operations) | ✅ v5 only |
| `--mcp-mode` | Flag | Use MCP protocol format | ✅ v5 only |
| `--as-agent <ID>` | String | Execute as delegated agent | ✅ v5 only |
| `--delegation-cert <JSON>` | JSON | Delegation certificate | ✅ v5 only |
| `--audit` | Flag | Include full audit trail | ✅ v5 |
| `--format <fmt>` | String | Output format (json, openapi, sparql) | ✅ v5 |

### Examples

```bash
# v4 mode (human-friendly)
myapp --help
myapp pack list

# v5 mode (machine-friendly)
myapp --introspect
myapp --machine pack list --json '{"category":"templates"}'
myapp --machine --stream pack install --json '{"name":"web-api"}'
```

---

## Introspection API Response Format

### Root Structure

```json
{
  "version": "5.0.0",
  "capabilities": [ /* capability objects */ ],
  "metadata": {
    "timestamp": "2025-11-20T10:30:00Z",
    "schema_version": "1.0",
    "supported_formats": ["json", "openapi", "sparql"]
  }
}
```

### Capability Object

```json
{
  "id": "pack:install",
  "name": "pack install",
  "category": "Pack Management",
  "description": "Install a code generation pack",

  "input_schema": {
    "type": "object",
    "required": ["name"],
    "properties": {
      "name": {
        "type": "string",
        "description": "Name of the pack to install"
      },
      "force": {
        "type": "boolean",
        "default": false,
        "description": "Force installation even if exists"
      },
      "version": {
        "type": "string",
        "optional": true,
        "description": "Specific version to install"
      }
    }
  },

  "output_schema": {
    "type": "object",
    "properties": {
      "name": { "type": "string" },
      "version": { "type": "string" },
      "installed_at": { "type": "string" }
    }
  },

  "effects": {
    "read_only": false,
    "mutating": true,
    "isolation": "independent",
    "timeout_ms": 30000,
    "side_effects": [
      "filesystem_write",
      "network_call"
    ]
  },

  "guards": {
    "preconditions": [
      {
        "name": "pack_exists",
        "description": "Pack must exist in registry",
        "condition": "registry_contains(name)"
      }
    ],
    "postconditions": [
      {
        "name": "pack_installed",
        "description": "Pack must be installed",
        "condition": "pack_dir_exists(name)"
      }
    ]
  },

  "delegation": {
    "delegable": true,
    "delegable_to": ["admin", "installer"],
    "max_depth": 3,
    "require_signature": true
  },

  "audit": {
    "required": true,
    "log_all_calls": true,
    "retention_days": 90
  }
}
```

### Guard Object

```json
{
  "name": "pack_exists",
  "description": "Pack must exist in registry",
  "condition": "registry_contains(name)",
  "severity": "error",
  "recovery": "Check 'pack list' to see available packs"
}
```

---

## Command Response Format

### Success Response

```json
{
  "status": "success",
  "data": {
    /* operation-specific result */
  },
  "receipt": {
    "id": "exec-d4c4f6a2-8e2c-4c5a-b7a1-f3c6e8d9b2a1",
    "timestamp": "2025-11-20T10:30:00Z",
    "duration_ms": 1250,
    "capabilities_used": ["pack:install"],
    "agent_id": "agent-installer-001",
    "signature": "sig_abc123def456...",
    "schema_hash": "sha256:abc123...",
    "audit_log_id": "audit-20251120-001"
  }
}
```

### Error Response

```json
{
  "status": "error",
  "code": "VALIDATION_ERROR",
  "message": "Input validation failed",
  "details": {
    "field": "name",
    "error": "Expected string, got number",
    "expected_type": "string",
    "received_value": 123,
    "schema_path": "$.properties.name.type"
  },
  "recovery": {
    "suggestion": "Pass name as string",
    "example": {
      "name": "web-api"
    },
    "documentation_url": "https://docs.example.com/pack-install"
  },
  "timestamp": "2025-11-20T10:30:01Z",
  "request_id": "req-abc123..."
}
```

### Error Codes

| Code | Severity | Meaning | Recovery |
|------|----------|---------|----------|
| `VALIDATION_ERROR` | User | Input schema mismatch | Fix inputs, see `recovery.suggestion` |
| `PRECONDITION_FAILED` | User | Guard check failed | Check preconditions, see guard details |
| `AUTHORIZATION_ERROR` | User | Insufficient permissions | Request elevation or delegation |
| `RESOURCE_NOT_FOUND` | User | Required resource missing | Create resource or use different ID |
| `TIMEOUT_ERROR` | Temporal | Operation exceeded timeout | Increase timeout or check system load |
| `INTERNAL_ERROR` | System | Server-side error | Retry with exponential backoff |
| `SERVICE_UNAVAILABLE` | Temporal | Service overloaded | Retry with exponential backoff |

---

## Input Schema Reference

### Standard Type Definitions

```json
{
  "string": {
    "type": "string",
    "description": "UTF-8 text",
    "pattern": "optional regex",
    "min_length": 0,
    "max_length": 65535
  },

  "integer": {
    "type": "integer",
    "description": "Signed 64-bit integer",
    "minimum": -9223372036854775808,
    "maximum": 9223372036854775807
  },

  "number": {
    "type": "number",
    "description": "IEEE 754 double-precision float"
  },

  "boolean": {
    "type": "boolean",
    "description": "True or false"
  },

  "array": {
    "type": "array",
    "items": { /* item schema */ },
    "min_items": 0,
    "max_items": 1000,
    "unique_items": false
  },

  "object": {
    "type": "object",
    "properties": { /* property schemas */ },
    "required": ["field1", "field2"],
    "additional_properties": false
  },

  "null": {
    "type": "null",
    "description": "Null value"
  }
}
```

### Special Formats

```json
{
  "date": {
    "type": "string",
    "format": "date",
    "pattern": "YYYY-MM-DD",
    "example": "2025-11-20"
  },

  "time": {
    "type": "string",
    "format": "time",
    "pattern": "HH:MM:SS",
    "example": "10:30:00"
  },

  "datetime": {
    "type": "string",
    "format": "date-time",
    "pattern": "RFC3339",
    "example": "2025-11-20T10:30:00Z"
  },

  "uuid": {
    "type": "string",
    "format": "uuid",
    "pattern": "UUID v4",
    "example": "550e8400-e29b-41d4-a716-446655440000"
  },

  "uri": {
    "type": "string",
    "format": "uri",
    "pattern": "RFC3986",
    "example": "https://example.com/api/v1/resource"
  },

  "path": {
    "type": "string",
    "format": "path",
    "description": "File system path",
    "example": "/home/user/file.txt"
  }
}
```

---

## Receipt Structure

### Receipt Object

```json
{
  "id": "exec-d4c4f6a2-8e2c-4c5a-b7a1-f3c6e8d9b2a1",
  "timestamp": "2025-11-20T10:30:00Z",
  "duration_ms": 1250,

  "execution": {
    "capabilities_used": ["pack:install"],
    "agent_id": "agent-001",
    "user_id": "user-123",
    "tenant_id": "tenant-abc"
  },

  "verification": {
    "signature": "sig_abc123def456...",
    "signature_algorithm": "ECDSA-SHA256",
    "public_key": "pk_...",
    "schema_hash": "sha256:abc123def456..."
  },

  "audit": {
    "audit_log_id": "audit-20251120-001",
    "audit_entries": [
      {
        "timestamp": "2025-11-20T10:30:00Z",
        "event": "precondition_check",
        "result": "passed"
      },
      {
        "timestamp": "2025-11-20T10:30:00.100Z",
        "event": "execution_start",
        "capability": "pack:install"
      },
      {
        "timestamp": "2025-11-20T10:30:01.350Z",
        "event": "execution_complete",
        "result": "success"
      }
    ]
  },

  "metadata": {
    "request_id": "req-xyz789",
    "version": "5.0.0"
  }
}
```

---

## Delegation Certificate Structure

### Certificate Object

```json
{
  "type": "delegation",
  "delegating_agent": "agent-000",
  "delegated_agent": "agent-001",
  "operation": "pack:install",

  "authorization": {
    "scope": "pack:install",
    "actions": ["execute"],
    "parameters": {
      "name": "web-api",
      "force": false
    }
  },

  "validity": {
    "issued_at": "2025-11-20T10:00:00Z",
    "expires_at": "2025-11-20T11:00:00Z",
    "not_before": "2025-11-20T10:00:00Z"
  },

  "signature": {
    "algorithm": "ECDSA-SHA256",
    "signature": "sig_abc123def456...",
    "public_key": "pk_..."
  },

  "audit": {
    "reason": "Agent B needs to install packs",
    "delegating_agent_signature": "sig_...",
    "chain_depth": 1
  }
}
```

---

## Streaming Response Format

### Stream Event Types

```json
{
  "type": "progress",
  "completed": 50,
  "total": 100,
  "percentage": 50.0,
  "message": "Installing files (50/100)"
}
```

```json
{
  "type": "log",
  "level": "info",
  "timestamp": "2025-11-20T10:30:00.500Z",
  "message": "Downloading pack from registry",
  "context": {
    "pack": "web-api",
    "source": "https://registry.example.com/packs/web-api"
  }
}
```

```json
{
  "type": "receipt",
  "receipt": {
    "id": "exec-partial-001",
    "timestamp": "2025-11-20T10:30:00.750Z",
    "phase": "downloading",
    "progress_percent": 50
  }
}
```

```json
{
  "type": "error",
  "code": "DOWNLOAD_FAILED",
  "message": "Failed to download pack",
  "recovery": "Check network connection and retry",
  "timestamp": "2025-11-20T10:30:01.000Z"
}
```

```json
{
  "type": "complete",
  "status": "success",
  "receipt": {
    "id": "exec-final-001",
    "timestamp": "2025-11-20T10:30:02.000Z",
    "duration_ms": 2000,
    "signature": "sig_..."
  }
}
```

---

## Effect Model Reference

### Effect Declaration

```json
{
  "effects": {
    "read_only": false,
    "mutating": true,

    "isolation": "independent",
    "concurrency_model": "exclusive",

    "timeout_ms": 30000,
    "max_retries": 3,
    "retry_backoff_ms": 1000,

    "side_effects": [
      "filesystem_write",
      "network_call",
      "environment_change"
    ],

    "resource_requirements": {
      "cpu_percent": 50,
      "memory_mb": 256,
      "disk_mb": 1000
    },

    "dependencies": [
      "network_available",
      "disk_space_available"
    ]
  }
}
```

### Isolation Levels

| Level | Meaning | Concurrency |
|-------|---------|-------------|
| `independent` | No shared state | Full parallelization |
| `shared_read` | Reads same state | Concurrent reads |
| `exclusive` | Exclusive access | Sequential only |
| `serializable` | ACID guarantee | Sequential with isolation |

---

## MCP Protocol Integration

### Tool Registration Format

```json
{
  "name": "pack_install",
  "description": "Install a code generation pack",
  "inputSchema": {
    "type": "object",
    "required": ["name"],
    "properties": {
      "name": {
        "type": "string",
        "description": "Pack name"
      },
      "force": {
        "type": "boolean",
        "default": false
      }
    }
  }
}
```

### MCP Request Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "pack_install",
    "arguments": {
      "name": "web-api",
      "force": false
    }
  }
}
```

### MCP Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"status\": \"success\", \"data\": {...}}"
      }
    ]
  }
}
```

---

## OpenAPI Export Format

### Generated OpenAPI Schema

```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "clap-noun-verb v5 API",
    "version": "5.0.0"
  },
  "paths": {
    "/pack/install": {
      "post": {
        "operationId": "pack_install",
        "description": "Install a code generation pack",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": ["name"],
                "properties": {
                  "name": { "type": "string" }
                }
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "status": { "type": "string", "enum": ["success"] },
                    "data": { },
                    "receipt": { }
                  }
                }
              }
            }
          },
          "400": {
            "description": "Validation Error"
          }
        }
      }
    }
  }
}
```

---

## SPARQL Ontology Format

### Triple Store Integration

```sparql
PREFIX clap: <https://clap-noun-verb.org/v5/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

# Capability definition
clap:pack_install a clap:Capability ;
  clap:id "pack:install"^^xsd:string ;
  clap:name "Install Pack"^^xsd:string ;
  clap:description "Install a code generation pack"^^xsd:string ;
  clap:readOnly false ;
  clap:requiresPrecondition clap:pack_exists ;
  clap:produces clap:PackInstalledOutput .

# Precondition
clap:pack_exists a clap:Guard ;
  clap:description "Pack must exist in registry"^^xsd:string ;
  clap:condition "registry_contains(name)"^^xsd:string .

# Effect
clap:PackInstallEffect a clap:Effect ;
  clap:capability clap:pack_install ;
  clap:mutates true ;
  clap:sideEffect clap:filesystem_write ;
  clap:timeout 30000 .
```

---

## Query Examples

### Get All Capabilities

```bash
./myapp --introspect | jq '.capabilities[]'
```

### Get Specific Capability

```bash
./myapp --introspect | jq '.capabilities[] | select(.id == "pack:install")'
```

### Get All Guards

```bash
./myapp --introspect | jq '.capabilities[] | .guards.preconditions[]'
```

### Filter by Effect Type

```bash
./myapp --introspect | jq '.capabilities[] | select(.effects.read_only == true)'
```

### Export to OpenAPI

```bash
./myapp --format openapi --introspect > openapi.json
```

### Export to SPARQL

```bash
./myapp --format sparql --introspect > ontology.ttl
```

---

## Compatibility Matrix

| Feature | v4 | v5 | Notes |
|---------|----|----|-------|
| `--help` | ✅ | ❌ | Use `--introspect` in v5 |
| `--json` | Partial | ✅ | Full support in v5 |
| `--machine` | ❌ | ✅ | v5 only |
| `--introspect` | ❌ | ✅ | v5 only |
| `--stream` | ❌ | ✅ | v5 only |
| Interactive mode | ✅ | ❌ | Machines don't need prompts |
| Error messages | Prose | Structured | Different formats |
| Receipts | ❌ | ✅ | Audit trails in v5 |
| Delegation | ❌ | ✅ | Agent-to-agent in v5 |
| Introspection | ❌ | ✅ | Machine discovery in v5 |

---

## Version History

### v5.0.0 (2025-11-20)
- Initial v5 release (machine-centric)
- Introspection API
- Effect models and guards
- Delegation certificates
- Execution receipts
- Streaming support

### v4.x (Current)
- Human-centric CLI
- Interactive help
- Progressive learning
- Rich error messages

---

