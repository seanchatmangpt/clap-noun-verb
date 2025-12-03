# Tutorial 06: Autonomic Features - Machine-Grade CLIs

**Learning Path:** Traditional CLIs → AI Agent-Ready Applications
**Time:** 30 minutes
**Prerequisites:** [Tutorial 05: Output Formats](05-output-formats.md)

---

## What You'll Learn

How to build machine-grade CLIs for AI agents:
- Introspection API (`--capabilities`, `--introspect`)
- Effect metadata and safety declarations
- Execution receipts for MAPE-K loops
- MCP protocol compatibility

---

## Why Autonomic CLIs?

**Traditional CLI:** Designed for humans
**Autonomic CLI:** Designed for autonomous agents

```bash
# Human CLI
$ deploy-app production --region us-west-2
Deploying to production...
✓ Deployment complete!

# Autonomic CLI
$ deploy-app production --region us-west-2
{
  "deployment_id": "dep-abc123",
  "status": "success",
  "effects": ["writes_state", "network_call"],
  "sensitivity": "high",
  "receipt": "0x3a7f...",
  "timestamp": "2025-12-03T18:00:00Z"
}
```

**Key differences:**
- ✅ Machine-parseable JSON
- ✅ Declared side effects
- ✅ Cryptographic receipts
- ✅ Introspectable capabilities

---

## Introspection API

### `--capabilities` Flag

Show what the CLI can do:

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeploymentResult {
    deployment_id: String,
    status: String,
    effects: Vec<String>,
}

#[verb(
    help = "Deploy application to environment",
    effects = ["writes_state", "network_call"],
    sensitivity = "high"
)]
pub fn deploy(
    #[arg(help = "Target environment")] environment: String,
    #[arg(help = "AWS region", default = "us-west-2")] region: String,
) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
    let deployment_id = crate::domain::deployments::deploy(&environment, &region)?;

    Ok(DeploymentResult {
        deployment_id,
        status: "success".to_string(),
        effects: vec!["writes_state".to_string(), "network_call".to_string()],
    })
}
```

**Usage:**
```bash
$ myapp --capabilities
{
  "capabilities": [
    {
      "noun": "deployments",
      "verb": "deploy",
      "effects": ["writes_state", "network_call"],
      "sensitivity": "high",
      "description": "Deploy application to environment",
      "arguments": [
        {
          "name": "environment",
          "type": "String",
          "required": true,
          "description": "Target environment"
        },
        {
          "name": "region",
          "type": "String",
          "required": false,
          "default": "us-west-2",
          "description": "AWS region"
        }
      ]
    }
  ]
}
```

**AI agents use this to:**
1. Discover available commands
2. Understand required arguments
3. Assess risk (effects, sensitivity)
4. Plan action sequences

---

### `--introspect` Flag

Deep inspection of a specific command:

```bash
$ myapp deployments deploy --introspect
{
  "command": "deployments deploy",
  "noun": "deployments",
  "verb": "deploy",
  "description": "Deploy application to environment",
  "effects": ["writes_state", "network_call"],
  "sensitivity": "high",
  "arguments": {
    "environment": {
      "type": "String",
      "required": true,
      "description": "Target environment",
      "valid_values": ["dev", "staging", "production"]
    },
    "region": {
      "type": "String",
      "required": false,
      "default": "us-west-2",
      "description": "AWS region",
      "valid_values": ["us-west-2", "us-east-1", "eu-west-1"]
    }
  },
  "examples": [
    {
      "command": "myapp deployments deploy --environment production --region us-west-2",
      "description": "Deploy to production in US West"
    }
  ],
  "guards": {
    "requires_approval": true,
    "budget_limit": 1000
  }
}
```

---

## Effect Metadata

Declare command side effects for safety:

### Effect Types

```rust
#[verb(
    help = "Read service configuration",
    effects = ["reads_state"] // ← Safe: read-only
)]
pub fn get_config(
    #[arg(help = "Service name")] service: String,
) -> Result<ConfigData, Box<dyn std::error::Error>> {
    // ...
}

#[verb(
    help = "Update service configuration",
    effects = ["writes_state"] // ← Caution: modifies state
)]
pub fn set_config(
    #[arg(help = "Service name")] service: String,
    #[arg(help = "Configuration key")] key: String,
    #[arg(help = "Configuration value")] value: String,
) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    // ...
}

#[verb(
    help = "Delete service",
    effects = ["writes_state", "destructive"], // ← Warning: destructive
    sensitivity = "critical"
)]
pub fn delete(
    #[arg(help = "Service name")] service: String,
    #[arg(help = "Confirm deletion")] confirm: bool,
) -> Result<DeleteResult, Box<dyn std::error::Error>> {
    if !confirm {
        return Err("Deletion requires --confirm flag".into());
    }
    // ...
}
```

**Standard effects:**
- `reads_state` - Reads system state
- `writes_state` - Modifies system state
- `network_call` - Makes network requests
- `destructive` - Irreversible changes
- `expensive` - High resource usage

---

### Sensitivity Levels

```rust
#[verb(
    effects = ["reads_state"],
    sensitivity = "low" // ← Safe for automated execution
)]
pub fn list_services() -> Result<Vec<ServiceInfo>, Box<dyn std::error::Error>> {
    // ...
}

#[verb(
    effects = ["writes_state"],
    sensitivity = "medium" // ← Requires review
)]
pub fn restart_service(
    #[arg] service: String,
) -> Result<RestartResult, Box<dyn std::error::Error>> {
    // ...
}

#[verb(
    effects = ["writes_state", "destructive"],
    sensitivity = "critical" // ← Requires human approval
)]
pub fn drop_database(
    #[arg] database: String,
) -> Result<DropResult, Box<dyn std::error::Error>> {
    // ...
}
```

**Sensitivity levels:**
- `low` - Safe for automation
- `medium` - Review recommended
- `high` - Approval required
- `critical` - Human-in-the-loop mandatory

---

## Execution Receipts

Generate cryptographic receipts for audit trails:

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeploymentReceipt {
    deployment_id: String,
    status: String,
    effects_applied: Vec<String>,
    receipt_hash: String, // ← Cryptographic proof
    timestamp: String,
    executed_by: String,
}

#[verb(
    help = "Deploy with execution receipt",
    effects = ["writes_state", "network_call"],
    sensitivity = "high",
    generate_receipt = true // ← Enable receipt generation
)]
pub fn deploy_with_receipt(
    #[arg(help = "Environment")] environment: String,
) -> Result<DeploymentReceipt, Box<dyn std::error::Error>> {
    let deployment = crate::domain::deployments::deploy(&environment, "us-west-2")?;

    // Generate cryptographic receipt
    let receipt_hash = generate_receipt_hash(
        &deployment.id,
        &["writes_state", "network_call"],
        "agent-123",
    );

    Ok(DeploymentReceipt {
        deployment_id: deployment.id,
        status: "success".to_string(),
        effects_applied: vec!["writes_state".to_string(), "network_call".to_string()],
        receipt_hash,
        timestamp: chrono::Utc::now().to_rfc3339(),
        executed_by: "agent-123".to_string(),
    })
}

fn generate_receipt_hash(
    deployment_id: &str,
    effects: &[&str],
    agent_id: &str,
) -> String {
    use blake3::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(deployment_id.as_bytes());
    hasher.update(effects.join(",").as_bytes());
    hasher.update(agent_id.as_bytes());

    format!("0x{}", hasher.finalize().to_hex())
}
```

**Receipt output:**
```json
{
  "deployment_id": "dep-abc123",
  "status": "success",
  "effects_applied": ["writes_state", "network_call"],
  "receipt_hash": "0x3a7f9e2b...",
  "timestamp": "2025-12-03T18:00:00Z",
  "executed_by": "agent-123"
}
```

**Use cases:**
- MAPE-K loop verification
- Audit trails for compliance
- Non-repudiation of actions
- Agent accountability

---

## Guards and Budgets

Enforce resource constraints:

```rust
#[verb(
    help = "Train ML model",
    effects = ["expensive", "writes_state"],
    sensitivity = "medium",
    guards = ["budget_check", "quota_check"]
)]
pub fn train_model(
    #[arg(help = "Model name")] model: String,
    #[arg(help = "Training epochs", default = "100")] epochs: u32,
) -> Result<TrainingResult, Box<dyn std::error::Error>> {
    // Check budget before execution
    let estimated_cost = estimate_training_cost(epochs);
    if !check_budget_available(estimated_cost) {
        return Err("Insufficient budget for training".into());
    }

    // Execute with quota tracking
    let result = crate::domain::ml::train(&model, epochs)?;

    Ok(TrainingResult {
        model,
        epochs,
        cost: result.actual_cost,
        budget_remaining: get_remaining_budget(),
    })
}
```

---

## Exercise: Build Autonomic Service Manager

**Goal:** Create a service manager with full autonomic features

**Arrange:** Define domain with safety metadata

```rust
// domain/services.rs
pub enum ServiceEffect {
    ReadsState,
    WritesState,
    NetworkCall,
    Destructive,
}

pub struct ServiceOperation {
    pub effects: Vec<ServiceEffect>,
    pub sensitivity: String,
}

pub fn start_service(name: &str) -> Result<ServiceInfo, DomainError> {
    // Implementation
}

pub fn stop_service(name: &str) -> Result<ServiceInfo, DomainError> {
    // Implementation
}

pub fn delete_service(name: &str) -> Result<(), DomainError> {
    // Implementation
}
```

**Act:** Create autonomic CLI commands

```rust
// commands/services.rs
#[verb(
    help = "Start a service",
    effects = ["writes_state", "network_call"],
    sensitivity = "medium"
)]
pub fn start(
    #[arg(help = "Service name")] name: String,
) -> Result<ServiceReceipt, Box<dyn std::error::Error>> {
    let info = crate::domain::services::start_service(&name)?;

    Ok(ServiceReceipt {
        service_name: name,
        action: "start".to_string(),
        status: "success".to_string(),
        effects: vec!["writes_state".to_string(), "network_call".to_string()],
        receipt_hash: generate_receipt("start", &name),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[verb(
    help = "Stop a service",
    effects = ["writes_state"],
    sensitivity = "medium"
)]
pub fn stop(
    #[arg(help = "Service name")] name: String,
) -> Result<ServiceReceipt, Box<dyn std::error::Error>> {
    let info = crate::domain::services::stop_service(&name)?;

    Ok(ServiceReceipt {
        service_name: name,
        action: "stop".to_string(),
        status: "success".to_string(),
        effects: vec!["writes_state".to_string()],
        receipt_hash: generate_receipt("stop", &name),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[verb(
    help = "Delete a service permanently",
    effects = ["writes_state", "destructive"],
    sensitivity = "critical"
)]
pub fn delete(
    #[arg(help = "Service name")] name: String,
    #[arg(help = "Confirm deletion")] confirm: bool,
) -> Result<ServiceReceipt, Box<dyn std::error::Error>> {
    if !confirm {
        return Err("Deletion requires --confirm flag".into());
    }

    crate::domain::services::delete_service(&name)?;

    Ok(ServiceReceipt {
        service_name: name,
        action: "delete".to_string(),
        status: "success".to_string(),
        effects: vec!["writes_state".to_string(), "destructive".to_string()],
        receipt_hash: generate_receipt("delete", &name),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
```

**Assert:** Test introspection

```bash
# Discover capabilities
$ myapp --capabilities | jq '.capabilities[] | select(.verb == "delete")'
{
  "noun": "services",
  "verb": "delete",
  "effects": ["writes_state", "destructive"],
  "sensitivity": "critical",
  "description": "Delete a service permanently"
}

# Execute with receipt
$ myapp services delete --name test-svc --confirm
{
  "service_name": "test-svc",
  "action": "delete",
  "status": "success",
  "effects": ["writes_state", "destructive"],
  "receipt_hash": "0x7b3e...",
  "timestamp": "2025-12-03T18:00:00Z"
}
```

---

## Key Takeaways

✅ **Introspection API** - `--capabilities` and `--introspect` for discovery
✅ **Effect metadata** - Declare side effects for safety
✅ **Sensitivity levels** - Risk assessment for automation
✅ **Execution receipts** - Cryptographic audit trails
✅ **Guards and budgets** - Resource constraint enforcement

---

## Next Steps

- **[Tutorial 07: Async Operations](07-async-operations.md)** - Async CLI commands
- **[AUTONOMIC.md](../../AUTONOMIC.md)** - Complete autonomic layer reference
- **[Explanation: Agent2028](../explanation/autonomic/agent2028.md)** - Trillion-agent vision

**Estimated time to next tutorial:** 25 minutes

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
