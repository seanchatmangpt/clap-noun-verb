# Pattern: JSON by Default

## Context
Traditionally, command-line interfaces are designed for human eyes, printing tabulate columns, colorized text, and informal logging strings directly to the standard output. However, in modern environments, CLIs are increasingly executed by automated agents, CI/CD pipelines, scripts, and LLM-driven orchestrators.

## Problem
Ad-hoc, human-centric text output makes programmatic consumption brittle, requiring developers to write complex regular expressions or parser scripts that break whenever the layout changes.

## Forces
* **Human readability vs. Machine parsing:** Humans prefer clean, interactive, visual CLI layouts, whereas automation scripts and LLM agents need rigid, structured, and predictable formats (like JSON).
* **Maintainability:** Adjusting the wording of a CLI's human output should not break downstream automated systems.
* **Failure diagnostics:** When a command fails, standard error strings often lack the structured metadata (like error codes or execution states) needed for automatic recovery or automated retry logic.

## Solution
Make JSON the default output format for all command executions. Each CLI verb must return a structured object that implements `serde::Serialize`. The CLI runner executes the command, captures this returned object, and prints its serialized JSON form directly to standard output. 

If human-oriented formatting (e.g., tables or colorized output) is desired, configure it as an optional presentation filter (or a global flag like `--format text`), keeping the command's primary logic entirely focused on returning structured data.

### Example

```rust
use serde::Serialize;
use clap_noun_verb::Result;

#[derive(Serialize)]
pub struct DeployResult {
    pub service_name: String,
    pub version: String,
    pub status: String,
    pub endpoints: Vec<String>,
}

#[clap_noun_verb_macros::verb("deploy")]
fn cmd_deploy(service: String) -> Result<DeployResult> {
    // Perform deployment...
    Ok(DeployResult {
        service_name: service,
        version: "1.0.4".to_string(),
        status: "success".to_string(),
        endpoints: vec!["https://app.service.local".to_string()],
    })
}
```

Running this command produces a clean JSON object:
```json
{
  "service_name": "auth-service",
  "version": "1.0.4",
  "status": "success",
  "endpoints": ["https://app.service.local"]
}
```

## Resulting Context / Connections
* **Agent readiness:** Downstream LLM tools can invoke the command and parse the resulting JSON directly, avoiding text extraction errors.
* **Error consistency:** Standardized error wrappers serialize failures into a consistent JSON shape (e.g., `{"error": "InvalidPort", "message": "Port 99999 out of range"}`), enabling robust error-handling logic.
* **Connections:** Relies on **Domain Separation** to ensure that logic returns serializable structs. Links with **Reflexive Introspection**, as the JSON inputs and outputs map directly to the schemas exposed during introspection.
