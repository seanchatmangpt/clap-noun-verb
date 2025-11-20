# Diataxis: V5 Semantic CLI Tutorials

**Framework**: Diataxis Learning-Oriented Documentation
**Audience**: Beginners, AI agents, developers new to v5
**Purpose**: Get hands-on experience with v5 machine API
**Prerequisites**: Basic Rust knowledge, familiarity with clap-noun-verb v4

---

## Tutorial 1: Your First v5 Machine API Call

### What You'll Learn
- How to query v5 for command capabilities
- Understanding introspection responses
- Parsing machine-readable command metadata

### Prerequisites
- `clap-noun-verb` v5 installed
- Basic HTTP client (curl or similar)
- Understanding of JSON structures

### Step 1: Start Your Application in Machine Mode

Create a simple Rust application with clap-noun-verb:

```rust
use clap_noun_verb::cli::run;

#[noun]
pub struct Pack {
    /// List available packs
    #[verb]
    pub async fn list(
        #[arg] category: Option<String>,
        #[arg(long)] verbose: bool,
    ) -> Result<Vec<String>> {
        // Business logic here
        Ok(vec!["pack1".to_string(), "pack2".to_string()])
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    run()
}
```

Build and run:
```bash
cargo build --release
./target/release/myapp --help  # v4 mode (human-friendly)
```

### Step 2: Query Machine Introspection API

Call with machine flag:
```bash
./target/release/myapp --introspect
```

**Expected Output** (JSON):
```json
{
  "version": "5.0.0",
  "capabilities": [
    {
      "id": "pack:list",
      "name": "pack list",
      "category": "Pack Management",
      "description": "List available packs",
      "input_schema": {
        "type": "object",
        "properties": {
          "category": {
            "type": "string",
            "optional": true
          },
          "verbose": {
            "type": "boolean",
            "default": false
          }
        }
      },
      "output_schema": {
        "type": "array",
        "items": { "type": "string" }
      },
      "effects": ["read-only"],
      "guards": {
        "preconditions": [],
        "timeout_ms": 5000
      }
    }
  ]
}
```

### Step 3: Call a Command with Machine Parameters

Instead of human-friendly arguments, use structured JSON:

```bash
./target/release/myapp --machine pack list --json '{"category":"templates","verbose":true}'
```

**Response** (machine-verifiable):
```json
{
  "status": "success",
  "data": ["template1", "template2"],
  "receipt": {
    "id": "exec-12345",
    "timestamp": "2025-11-20T10:30:00Z",
    "capabilities_used": ["pack:list"],
    "duration_ms": 42,
    "signature": "sig_abc123..."
  }
}
```

### Step 4: Handle Machine Errors

Call with invalid parameters:

```bash
./target/release/myapp --machine pack list --json '{"category":123}'
```

**Machine-Readable Error**:
```json
{
  "status": "error",
  "code": "VALIDATION_ERROR",
  "details": {
    "field": "category",
    "error": "Expected string, got number",
    "recovery": "Pass category as string: {\"category\": \"string\"}"
  },
  "capability": "pack:list",
  "timestamp": "2025-11-20T10:30:01Z"
}
```

### Key Takeaways

✅ V5 provides introspection without human help text
✅ All arguments are formally declared and validated
✅ Responses include execution receipts for audit trails
✅ Errors are structured codes, not prose messages
✅ Same binary handles both human and machine callers

---

## Tutorial 2: Building an Agent That Uses V5

### What You'll Learn
- How to make AI agents call v5 commands
- Parsing and handling structured responses
- Chain multiple operations together

### Prerequisites
- Completed Tutorial 1
- Understanding of agent patterns
- HTTP client library

### Step 1: Introspect Available Capabilities

```rust
use reqwest;
use serde_json::json;

async fn discover_capabilities() -> Result<()> {
    // Call the app with --introspect flag
    let output = std::process::Command::new("./myapp")
        .arg("--introspect")
        .output()?;

    let capabilities: serde_json::Value =
        serde_json::from_slice(&output.stdout)?;

    println!("Available capabilities:");
    for cap in capabilities["capabilities"].as_array().unwrap() {
        println!("  - {}: {}",
            cap["id"],
            cap["description"]
        );
    }

    Ok(())
}
```

### Step 2: Build Safe Agent Calls

```rust
async fn call_pack_list(category: Option<&str>) -> Result<serde_json::Value> {
    let params = match category {
        Some(cat) => json!({ "category": cat, "verbose": false }),
        None => json!({ "verbose": true }),
    };

    let output = std::process::Command::new("./myapp")
        .args(&["--machine", "pack", "list"])
        .arg("--json")
        .arg(params.to_string())
        .output()?;

    if !output.status.success() {
        let error: serde_json::Value = serde_json::from_slice(&output.stderr)?;
        return Err(format!("Command failed: {:?}", error).into());
    }

    let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(response)
}
```

### Step 3: Chain Multiple Operations

```rust
async fn install_and_verify_pack(pack_name: &str) -> Result<bool> {
    // Step 1: Check if pack exists
    println!("Discovering packs...");
    let packs = call_pack_list(None).await?;

    if !packs["data"].as_array().unwrap().iter()
        .any(|p| p.as_str() == Some(pack_name)) {
        return Err(format!("Pack {} not found", pack_name).into());
    }

    // Step 2: Install the pack
    println!("Installing {}...", pack_name);
    let install = std::process::Command::new("./myapp")
        .args(&["--machine", "pack", "install"])
        .arg("--json")
        .arg(json!({ "name": pack_name }).to_string())
        .output()?;

    if !install.status.success() {
        return Err("Installation failed".into());
    }

    // Step 3: Verify installation
    println!("Verifying...");
    let verified = std::process::Command::new("./myapp")
        .args(&["--machine", "pack", "verify"])
        .arg("--json")
        .arg(json!({ "name": pack_name }).to_string())
        .output()?;

    Ok(verified.status.success())
}
```

### Step 4: Handle Execution Receipts

```rust
async fn audit_operation(pack_name: &str) -> Result<()> {
    let response = call_pack_list(Some(pack_name)).await?;

    // Extract the receipt (proof of execution)
    let receipt = &response["receipt"];

    println!("Operation Audit Trail:");
    println!("  Execution ID: {}", receipt["id"]);
    println!("  Timestamp: {}", receipt["timestamp"]);
    println!("  Duration: {}ms", receipt["duration_ms"]);
    println!("  Signature: {}", receipt["signature"]);

    // Verify the signature (in real implementation)
    verify_receipt_signature(&receipt)?;

    Ok(())
}
```

### Key Takeaways

✅ Agents discover what they can do via introspection
✅ All calls are strongly typed with JSON schemas
✅ Receipts provide proof of execution for audit
✅ Agents can compose operations safely
✅ Error handling is structured and machine-readable

---

## Tutorial 3: Implementing Formal Preconditions (Guards)

### What You'll Learn
- Declaring preconditions for operations
- Understanding guards and effect models
- Building safe, verifiable commands

### Prerequisites
- Completed Tutorials 1-2
- Understanding of formal verification concepts

### Step 1: Define a Command with Guards

```rust
use clap_noun_verb::autonomic::*;

#[noun]
pub struct Template {
    /// Render a template with validation
    #[verb]
    pub async fn render(
        #[arg] name: String,
        #[arg] variables: Option<String>,
    ) -> Result<String> {
        // Guards are applied automatically
        render_internal(name, variables).await
    }
}

impl Template {
    /// Define what must be true before render() executes
    fn preconditions() -> Vec<Guard> {
        vec![
            Guard::new("template_exists")
                .description("Template must be registered")
                .check(|ctx| {
                    let name = ctx.arg("name").unwrap();
                    TemplateRegistry::has(name)
                }),

            Guard::new("json_valid")
                .description("Variables must be valid JSON")
                .check(|ctx| {
                    if let Some(vars) = ctx.arg("variables") {
                        serde_json::from_str::<serde_json::Value>(vars).is_ok()
                    } else {
                        true
                    }
                }),
        ]
    }

    /// Declare what this operation does
    fn effects() -> EffectModel {
        EffectModel::new()
            .read_only(true)  // Doesn't modify state
            .description("Generates code from template")
            .output_type(OutputType::Text)
    }
}
```

### Step 2: Agent Checks Preconditions Before Calling

```rust
async fn safe_template_render(
    introspect: &serde_json::Value,
    name: &str,
    variables: Option<&str>,
) -> Result<String> {
    // Find the capability metadata
    let render_cap = introspect["capabilities"]
        .as_array().unwrap()
        .iter()
        .find(|c| c["id"].as_str() == Some("template:render"))
        .ok_or("Capability not found")?;

    // Extract the guards (preconditions)
    let guards = &render_cap["guards"]["preconditions"];

    println!("Checking preconditions...");
    for guard in guards.as_array().unwrap() {
        let guard_name = guard["name"].as_str().unwrap();
        println!("  Checking: {}", guard["description"]);

        // In real implementation, evaluate guard conditions
        // For now, just log them
    }

    // Only proceed if preconditions would pass
    let params = json!({
        "name": name,
        "variables": variables
    });

    // Call the command
    let output = std::process::Command::new("./myapp")
        .args(&["--machine", "template", "render"])
        .arg("--json")
        .arg(params.to_string())
        .output()?;

    if output.status.success() {
        let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(response["data"].as_str().unwrap().to_string())
    } else {
        let error: serde_json::Value = serde_json::from_slice(&output.stderr)?;
        Err(format!("Precondition failed: {:?}", error).into())
    }
}
```

### Step 3: Understanding Effect Models

```rust
// Declare what this operation does to the system
impl Template {
    fn effects() -> EffectModel {
        EffectModel::new()
            // This is a read-only operation
            .read_only(true)

            // Can it be isolated? (run in parallel)
            .isolation(Isolation::Independent)

            // Expected latency
            .timeout(Duration::from_secs(5))

            // What does it produce?
            .output_schema(json!({
                "type": "object",
                "properties": {
                    "result": { "type": "string" },
                    "template_id": { "type": "string" }
                }
            }))

            // Who can call it?
            .required_permissions(vec![
                Permission::ReadTemplates,
            ])
    }
}
```

### Key Takeaways

✅ Guards formally verify preconditions before execution
✅ Effects declare what operations do (read-only, mutating, etc)
✅ Agents can verify conditions before calling
✅ Enables safe, composable agent workflows
✅ Preconditions prevent invalid states

---

## Tutorial 4: Agent Delegation with Proofs

### What You'll Learn
- How agents delegate to other agents
- Understanding delegation chains
- Verifying authorization with proofs

### Prerequisites
- Completed Tutorials 1-3
- Understanding of distributed systems

### Step 1: Define Delegatable Operations

```rust
#[noun]
pub struct Pack {
    /// Install a pack (can be delegated)
    #[verb]
    #[delegatable]
    pub async fn install(
        #[arg] name: String,
        #[arg(long)] force: bool,
    ) -> Result<()> {
        // Business logic
        pack_install(&name, force).await
    }
}

impl Pack {
    fn delegation_policy() -> DelegationPolicy {
        DelegationPolicy::new()
            // Who can delegate this operation
            .delegable_to(vec![
                AgentRole::Admin,
                AgentRole::Installer,
            ])

            // What depth of delegation is allowed
            .max_delegation_depth(3)

            // Signature requirements
            .require_signature(true)

            // Audit requirements
            .audit_all(true)
    }
}
```

### Step 2: Agent Delegates with Proof

```rust
async fn delegate_pack_installation(
    delegating_agent: &AgentIdentity,
    target_agent: &AgentIdentity,
    pack_name: &str,
) -> Result<DelegationProof> {
    // Create delegation certificate
    let delegation = Certificate::new()
        .delegating_agent(delegating_agent.id())
        .target_agent(target_agent.id())
        .operation("pack:install")
        .parameters(json!({ "name": pack_name }))
        .timestamp(Utc::now())
        .signature(delegating_agent.sign()?);

    // Execute operation as delegated agent
    let output = std::process::Command::new("./myapp")
        .args(&["--machine", "--as-agent", &target_agent.id()])
        .args(&["--delegation-cert", &delegation.to_string()])
        .args(&["pack", "install"])
        .arg("--json")
        .arg(json!({ "name": pack_name }).to_string())
        .output()?;

    if !output.status.success() {
        return Err("Delegation failed".into());
    }

    // Extract proof of execution
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let proof = DelegationProof::from_receipt(&response["receipt"])?;

    Ok(proof)
}
```

### Step 3: Verify Delegation Chain

```rust
async fn audit_delegation_chain(
    proof: &DelegationProof,
) -> Result<()> {
    println!("Delegation Chain Verification:");
    println!("  Original Agent: {}", proof.delegating_agent);
    println!("  Delegated To: {}", proof.target_agent);
    println!("  Operation: {}", proof.operation);

    // Verify signatures at each step
    for (i, cert) in proof.certificates.iter().enumerate() {
        println!("\n  Signature {}: {}", i + 1, cert.signature_id());

        let is_valid = cert.verify_signature()?;
        println!("    Valid: {}", is_valid);

        let signer = cert.signer_identity();
        println!("    Signed by: {}", signer);
    }

    // Verify final execution
    println!("\n  Final Execution:");
    println!("    Executed at: {}", proof.execution_time);
    println!("    Duration: {}ms", proof.duration_ms);
    println!("    Result: {:?}", proof.result);

    Ok(())
}
```

### Key Takeaways

✅ Agents can safely delegate operations
✅ Delegation is verified with cryptographic proofs
✅ Complete audit trail of who did what
✅ Authorization is explicit and verifiable
✅ Enables secure multi-agent systems

---

## Tutorial 5: Building an MCP Server for V5

### What You'll Learn
- Exposing v5 commands via MCP protocol
- Integrating with Claude and other LLMs
- Real-time command execution and monitoring

### Prerequisites
- Completed Tutorials 1-4
- Understanding of MCP protocol
- Familiarity with async Rust

### Step 1: Create MCP Server Skeleton

```rust
use mcp_server::{Server, Tool, ToolInput};
use clap_noun_verb::autonomic::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = Server::new("clap-noun-verb-v5")
        .version("5.0.0")
        .description("Machine Control Protocol for v5 semantic CLI");

    // Get available capabilities from introspection
    let capabilities = discover_capabilities().await?;

    // Register each capability as an MCP tool
    for cap in capabilities {
        let tool = Tool::new(&cap.id)
            .description(&cap.description)
            .input_schema(cap.input_schema.clone());

        server.register_tool(tool);
    }

    // Start the server
    server.listen().await?;
    Ok(())
}
```

### Step 2: Implement Tool Handlers

```rust
async fn handle_tool_call(
    tool_id: &str,
    input: ToolInput,
) -> Result<String> {
    // Parse the input according to schema
    let params = json!(input.parameters);

    // Call the underlying v5 command
    let output = std::process::Command::new("./myapp")
        .args(&["--machine", "--mcp-mode"])
        .arg("--tool")
        .arg(tool_id)
        .arg("--json")
        .arg(params.to_string())
        .output()?;

    if !output.status.success() {
        let error: serde_json::Value = serde_json::from_slice(&output.stderr)?;
        return Err(error.to_string());
    }

    // Return structured response
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(response.to_string())
}
```

### Step 3: Connect with Claude

```rust
// Use with Claude via MCP
// In your Claude Code configuration:

let mcp_server = Server::new("clap-noun-verb")
    .stdio_transport()
    .expose_tools_to_claude();

// Now Claude can:
// 1. Query introspection: "What can you do?"
// 2. Check preconditions: "Can you install pack X?"
// 3. Execute operations: "Install the web-api pack"
// 4. Verify results: "Check the audit trail"
```

### Key Takeaways

✅ V5 commands become MCP tools for LLMs
✅ Full introspection available to Claude
✅ Structured I/O for reliable automation
✅ Real-time execution with receipts
✅ Audit trails for accountability

---

## Summary: Learning Path

### Progression
1. **Tutorial 1**: Query machine API (introspection basics)
2. **Tutorial 2**: Agent integrations (chaining operations)
3. **Tutorial 3**: Formal preconditions (safety)
4. **Tutorial 4**: Delegation chains (distributed)
5. **Tutorial 5**: MCP integration (LLM-ready)

### Key Skills Learned
- ✅ V5 command discovery and introspection
- ✅ Structured machine-readable I/O
- ✅ Agent-safe operation composition
- ✅ Formal verification and preconditions
- ✅ Delegation with cryptographic proof
- ✅ MCP protocol integration

### Next Steps
- Explore **How-to Guides** for specific scenarios
- Read **Reference Documentation** for detailed API
- Study **Explanations** for conceptual understanding

---

