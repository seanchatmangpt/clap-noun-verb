# Diataxis: V5 Semantic CLI How-To Guides

**Framework**: Diataxis Goal-Oriented Documentation
**Audience**: Practitioners, AI agents, developers with specific problems to solve
**Purpose**: Step-by-step solutions for common tasks
**Format**: Problem → Solution → Implementation

---

## How To: Query Available Commands via Introspection

### Problem
You need to discover what operations are available in a v5 application without reading documentation.

### Solution
Use the introspection API to get machine-readable capability metadata.

### Implementation

**Option 1: Command Line**
```bash
# Get all capabilities as JSON
./myapp --introspect

# Parse specific capability
./myapp --introspect | jq '.capabilities[] | select(.id == "pack:list")'
```

**Option 2: Programmatic (Rust)**
```rust
use serde_json::json;

async fn get_all_capabilities() -> Result<Vec<Capability>> {
    let output = std::process::Command::new("./myapp")
        .arg("--introspect")
        .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let capabilities = json["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| Capability {
            id: c["id"].as_str().unwrap().to_string(),
            name: c["name"].as_str().unwrap().to_string(),
            description: c["description"].as_str().unwrap().to_string(),
        })
        .collect();

    Ok(capabilities)
}

async fn find_capability_by_name(name: &str) -> Result<Option<serde_json::Value>> {
    let output = std::process::Command::new("./myapp")
        .arg("--introspect")
        .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    Ok(json["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"].as_str() == Some(name))
        .cloned())
}
```

**Option 3: Python Agent**
```python
import subprocess
import json

def get_capabilities():
    """Get all v5 capabilities as structured data."""
    result = subprocess.run(
        ["./myapp", "--introspect"],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

def find_capability(capability_id):
    """Find a specific capability by ID."""
    caps = get_capabilities()
    for cap in caps["capabilities"]:
        if cap["id"] == capability_id:
            return cap
    return None

# Usage
all_caps = get_capabilities()
print(f"Available capabilities: {len(all_caps['capabilities'])}")

pack_list = find_capability("pack:list")
print(f"pack:list inputs: {pack_list['input_schema']}")
```

### Result
You have programmatic access to all capabilities without hardcoding command names.

---

## How To: Validate Inputs Against Schema Before Calling

### Problem
You want to validate that your arguments match the expected schema before calling a v5 command.

### Solution
Extract the input schema from introspection and validate against it.

### Implementation

**Rust Validation Helper**
```rust
use jsonschema::JSONSchema;

async fn validate_inputs(
    capability_id: &str,
    inputs: &serde_json::Value,
) -> Result<bool> {
    // Get capability metadata
    let caps = get_all_capabilities().await?;
    let capability = caps
        .iter()
        .find(|c| c.id == capability_id)
        .ok_or("Capability not found")?;

    // Extract input schema
    let schema_value = capability.input_schema.clone();
    let schema = JSONSchema::compile(&schema_value)
        .map_err(|e| format!("Invalid schema: {}", e))?;

    // Validate inputs
    match schema.validate(inputs) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Validation error: {}", e)),
    }
}

// Usage
let inputs = json!({
    "name": "web-api",
    "force": false,
    "timeout_ms": 5000
});

let valid = validate_inputs("pack:install", &inputs).await?;
assert!(valid);
```

**Python Validation Helper**
```python
import jsonschema
import subprocess
import json

def validate_before_calling(capability_id, params):
    """Validate params match capability schema."""
    # Get capability
    result = subprocess.run(
        ["./myapp", "--introspect"],
        capture_output=True,
        text=True
    )
    caps = json.loads(result.stdout)

    capability = next(
        (c for c in caps["capabilities"] if c["id"] == capability_id),
        None
    )

    if not capability:
        raise ValueError(f"Capability {capability_id} not found")

    # Validate
    schema = capability["input_schema"]
    try:
        jsonschema.validate(instance=params, schema=schema)
        return True
    except jsonschema.ValidationError as e:
        print(f"Validation failed: {e.message}")
        return False

# Usage
params = {
    "name": "web-api",
    "force": False
}

if validate_before_calling("pack:install", params):
    print("✓ Parameters are valid")
else:
    print("✗ Parameters are invalid")
```

### Result
You catch invalid inputs before calling the command, improving error messages and debugging.

---

## How To: Call a Command and Process Machine-Readable Response

### Problem
You need to call a v5 command and reliably parse the structured response.

### Solution
Use the `--machine --json` flags to get guaranteed JSON output with receipts.

### Implementation

**Basic Call**
```rust
async fn call_v5_command(
    noun: &str,
    verb: &str,
    params: &serde_json::Value,
) -> Result<CommandResponse> {
    let output = std::process::Command::new("./myapp")
        .args(&["--machine", noun, verb])
        .arg("--json")
        .arg(params.to_string())
        .output()?;

    if !output.status.success() {
        // Parse error response
        let error: ErrorResponse = serde_json::from_slice(&output.stderr)?;
        return Err(format!("Command failed: {:?}", error));
    }

    // Parse success response
    let response: CommandResponse = serde_json::from_slice(&output.stdout)?;
    Ok(response)
}

#[derive(serde::Deserialize)]
struct CommandResponse {
    status: String,
    data: serde_json::Value,
    receipt: Receipt,
}

#[derive(serde::Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
    recovery: Option<String>,
}

// Usage
let result = call_v5_command(
    "pack",
    "install",
    &json!({ "name": "web-api" })
).await?;

println!("Result: {}", result.data);
println!("Execution ID: {}", result.receipt.id);
```

**With Error Recovery**
```rust
async fn call_with_recovery(
    noun: &str,
    verb: &str,
    params: &serde_json::Value,
    max_retries: u32,
) -> Result<CommandResponse> {
    for attempt in 0..max_retries {
        match call_v5_command(noun, verb, params).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                println!("Attempt {} failed: {}", attempt + 1, e);

                if attempt < max_retries - 1 {
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                } else {
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}
```

### Result
Reliable, structured communication with guaranteed response format and error handling.

---

## How To: Build an Agent That Respects Preconditions

### Problem
Your agent needs to check if an operation can succeed before attempting it.

### Solution
Query the capability metadata for guards/preconditions and evaluate them locally.

### Implementation

```rust
#[derive(serde::Deserialize)]
struct Guard {
    name: String,
    description: String,
    condition: String,  // Machine-readable condition
}

async fn can_execute_safely(
    capability_id: &str,
    context: &AgentContext,
) -> Result<bool> {
    // Get capability metadata
    let caps = get_all_capabilities().await?;
    let capability = caps
        .iter()
        .find(|c| c.id == capability_id)
        .ok_or("Capability not found")?;

    // Check each guard
    println!("Checking preconditions for {}:", capability_id);
    for guard in &capability.guards {
        println!("  ✓ {}: {}", guard.name, guard.description);

        // Evaluate guard condition in agent context
        if !evaluate_condition(&guard.condition, context)? {
            println!("    ✗ FAILED");
            return Ok(false);
        }
    }

    println!("  ✓ All preconditions met");
    Ok(true)
}

fn evaluate_condition(condition: &str, ctx: &AgentContext) -> Result<bool> {
    // Example condition: "file_exists /home/user/pack.tar.gz"
    let parts: Vec<&str> = condition.split_whitespace().collect();

    match parts[0] {
        "file_exists" => Ok(std::path::Path::new(parts[1]).exists()),
        "dir_is_writable" => {
            let dir = std::path::Path::new(parts[1]);
            Ok(dir.exists() && !std::fs::metadata(dir)?.permissions().readonly())
        }
        "resource_available" => {
            let resource = parts[1];
            Ok(ctx.resources().contains(resource))
        }
        "time_within" => {
            let (start, end) = (parts[1], parts[2]);
            let now = chrono::Local::now();
            // Parse time and check
            Ok(true)
        }
        _ => Err(format!("Unknown condition: {}", parts[0]))?,
    }
}

// Usage
let context = AgentContext::new()
    .add_resource("network")
    .add_resource("disk");

if can_execute_safely("pack:install", &context).await? {
    call_v5_command("pack", "install", &json!({"name": "web-api"})).await?;
} else {
    println!("Cannot execute - preconditions not met");
}
```

### Result
Your agent verifies conditions before executing, preventing failed operations and improving reliability.

---

## How To: Parse and Verify Execution Receipts

### Problem
You need to prove that an operation executed correctly and audit what was done.

### Solution
Extract and verify the receipt from the response.

### Implementation

```rust
#[derive(serde::Deserialize)]
struct Receipt {
    id: String,
    timestamp: String,
    capabilities_used: Vec<String>,
    duration_ms: u64,
    signature: String,
    agent_id: Option<String>,
}

async fn verify_receipt(receipt: &Receipt) -> Result<bool> {
    // 1. Check timestamp is recent
    let receipt_time = chrono::DateTime::parse_from_rfc3339(&receipt.timestamp)?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    let age = (now - receipt_time).num_minutes();

    if age > 60 {
        println!("⚠ Receipt is {} minutes old", age);
    }

    // 2. Verify signature
    let is_valid = verify_signature(&receipt.signature)?;
    println!("Signature valid: {}", is_valid);

    // 3. Check capabilities used
    println!("Capabilities used:");
    for cap in &receipt.capabilities_used {
        println!("  - {}", cap);
    }

    // 4. Store in audit log
    audit_log_store(receipt).await?;

    Ok(is_valid)
}

async fn audit_log_store(receipt: &Receipt) -> Result<()> {
    // Store receipt in immutable audit log
    let log_entry = json!({
        "execution_id": receipt.id,
        "timestamp": receipt.timestamp,
        "capabilities": receipt.capabilities_used,
        "duration_ms": receipt.duration_ms,
        "agent": receipt.agent_id,
    });

    // Write to audit log (file, database, blockchain, etc)
    let log_path = format!("./audit/{}.json", receipt.id);
    std::fs::write(log_path, log_entry.to_string())?;

    Ok(())
}

// Usage in command execution
let response = call_v5_command("pack", "install", &params).await?;

// Verify the receipt
if verify_receipt(&response.receipt).await? {
    println!("✓ Execution verified and logged");
} else {
    println!("✗ Receipt verification failed");
}
```

### Result
Cryptographic proof of execution for audit trails, compliance, and debugging.

---

## How To: Implement Delegation Between Agents

### Problem
Agent A needs to ask Agent B to perform an operation on its behalf.

### Solution
Use delegation certificates to grant temporary authority.

### Implementation

```rust
#[derive(serde::Serialize)]
struct DelegationCertificate {
    delegating_agent: String,
    delegated_agent: String,
    operation: String,
    parameters: serde_json::Value,
    issued_at: String,
    expires_at: String,
    signature: String,
}

async fn delegate_operation(
    from_agent: &AgentIdentity,
    to_agent: &AgentIdentity,
    operation: &str,
    params: &serde_json::Value,
) -> Result<CommandResponse> {
    // Create delegation certificate
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::hours(1);

    let cert = DelegationCertificate {
        delegating_agent: from_agent.id().to_string(),
        delegated_agent: to_agent.id().to_string(),
        operation: operation.to_string(),
        parameters: params.clone(),
        issued_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
        signature: from_agent.sign_delegation_certificate()?,
    };

    // Parse noun and verb from operation (e.g., "pack:install" → "pack", "install")
    let parts: Vec<&str> = operation.split(':').collect();
    let (noun, verb) = (parts[0], parts[1]);

    // Call command with delegation certificate
    let output = std::process::Command::new("./myapp")
        .args(&["--machine"])
        .args(&["--as-agent", &to_agent.id()])
        .arg("--delegation-cert")
        .arg(serde_json::to_string(&cert)?)
        .args(&[noun, verb])
        .arg("--json")
        .arg(params.to_string())
        .output()?;

    if !output.status.success() {
        let error: serde_json::Value = serde_json::from_slice(&output.stderr)?;
        return Err(format!("Delegation failed: {:?}", error));
    }

    let response: CommandResponse = serde_json::from_slice(&output.stdout)?;
    Ok(response)
}

// Usage
let agent_a = AgentIdentity::new("agent-a");
let agent_b = AgentIdentity::new("agent-b");

let response = delegate_operation(
    &agent_a,
    &agent_b,
    "pack:install",
    &json!({ "name": "web-api" })
).await?;

println!("Delegation executed: {}", response.receipt.id);
```

### Result
Secure, auditable agent-to-agent authorization with cryptographic proof.

---

## How To: Monitor Long-Running Operations via Streaming

### Problem
You need to execute a long operation and get real-time progress updates.

### Solution
Use streaming mode to get incremental receipts.

### Implementation

```rust
async fn execute_with_streaming(
    noun: &str,
    verb: &str,
    params: &serde_json::Value,
) -> Result<()> {
    let mut child = std::process::Command::new("./myapp")
        .args(&["--machine", "--stream"])
        .args(&[noun, verb])
        .arg("--json")
        .arg(params.to_string())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = std::io::BufReader::new(stdout);

    // Read streaming responses
    for line in std::io::BufRead::lines(reader) {
        let line = line?;

        // Each line is a JSON event
        let event: serde_json::Value = serde_json::from_str(&line)?;

        match event["type"].as_str() {
            Some("progress") => {
                println!("Progress: {}/{}",
                    event["completed"],
                    event["total"]
                );
            }
            Some("log") => {
                println!("Log: {}", event["message"]);
            }
            Some("receipt") => {
                println!("Incremental receipt: {}", event["receipt_id"]);
            }
            Some("complete") => {
                println!("Operation completed");
                println!("Final receipt: {}", event["receipt"]);
                break;
            }
            Some("error") => {
                eprintln!("Error: {}", event["message"]);
                return Err(event["message"].as_str().unwrap().into());
            }
            _ => {}
        }
    }

    Ok(())
}

// Usage
execute_with_streaming(
    "pack",
    "install",
    &json!({ "name": "large-pack" })
).await?;
```

### Result
Real-time feedback for long operations without blocking.

---

## How To: Build an Agentic Workflow Chain

### Problem
Multiple operations need to execute in sequence, with each step dependent on previous results.

### Solution
Chain operations using guards and effect models to ensure safety.

### Implementation

```rust
struct WorkflowStep {
    operation: String,
    params_fn: Box<dyn Fn(&AgentContext) -> serde_json::Value>,
    guards: Vec<String>,
    on_failure: FailureStrategy,
}

enum FailureStrategy {
    Abort,
    Retry(u32),
    Continue,
}

async fn execute_workflow(
    steps: Vec<WorkflowStep>,
    context: &mut AgentContext,
) -> Result<()> {
    for (i, step) in steps.into_iter().enumerate() {
        println!("\n=== Step {} ===", i + 1);
        println!("Operation: {}", step.operation);

        // Check guards
        if !can_execute_safely(&step.operation, context).await? {
            match step.on_failure {
                FailureStrategy::Abort => {
                    return Err("Precondition failed, aborting".into());
                }
                FailureStrategy::Retry(times) => {
                    for attempt in 0..times {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        if can_execute_safely(&step.operation, context).await? {
                            break;
                        }
                    }
                }
                FailureStrategy::Continue => {
                    println!("Skipping due to failed preconditions");
                    continue;
                }
            }
        }

        // Execute operation
        let params = (step.params_fn)(context);
        let (noun, verb) = parse_operation(&step.operation);

        let response = call_v5_command(noun, verb, &params).await?;

        // Update context with results
        context.set_last_result(&response.data);
        context.add_executed_operation(&step.operation, &response.receipt);

        println!("✓ Completed");
    }

    Ok(())
}

// Usage
let mut context = AgentContext::new();

let steps = vec![
    WorkflowStep {
        operation: "pack:list".to_string(),
        params_fn: Box::new(|_| json!({})),
        guards: vec!["network_available".to_string()],
        on_failure: FailureStrategy::Abort,
    },
    WorkflowStep {
        operation: "pack:install".to_string(),
        params_fn: Box::new(|ctx| {
            let packs = ctx.last_result()["data"].as_array().unwrap();
            json!({ "name": packs[0]["name"] })
        }),
        guards: vec!["disk_space_available".to_string()],
        on_failure: FailureStrategy::Retry(3),
    },
    WorkflowStep {
        operation: "pack:verify".to_string(),
        params_fn: Box::new(|ctx| {
            json!({ "name": ctx.last_operation_param("name") })
        }),
        guards: vec![],
        on_failure: FailureStrategy::Continue,
    },
];

execute_workflow(steps, &mut context).await?;
```

### Result
Safe, composable multi-step agent workflows with error recovery.

---

## Summary: Problem → Solution Mapping

| Problem | Solution | Documentation |
|---------|----------|-----------------|
| Discover available operations | Use `--introspect` | How To #1 |
| Validate inputs before calling | Extract schema, use jsonschema | How To #2 |
| Call command reliably | Use `--machine --json` | How To #3 |
| Check preconditions | Query guards from metadata | How To #4 |
| Verify execution | Parse and verify receipt | How To #5 |
| Multi-agent workflows | Use delegation certificates | How To #6 |
| Long operations | Use streaming mode | How To #7 |
| Sequential operations | Chain with guards/effects | How To #8 |

---

