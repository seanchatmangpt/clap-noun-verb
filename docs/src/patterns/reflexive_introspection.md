# Pattern: Reflexive Introspection

## Context
Integrating CLI tools with automated platforms, remote execution servers, or Large Language Models (LLMs) requires a precise description of the tool's interface—specifically, the names of commands (nouns/verbs), expected arguments, data types, default values, and documentation strings.

## Problem
Maintaining separate interface schemas (such as OpenAPI specs, JSON Schema, or markdown documentation) leads to documentation drift, where the external specifications fall out of sync with the compiled binary's actual behavior.

## Forces
* **Documentation drift:** Manually updating external schema files whenever a developer changes a CLI argument name or type is highly error-prone.
* **LLM Integration:** Agentic systems require structured specifications (like JSON tool definitions) to bind commands to model capabilities. Parsing raw CLI help text (`--help`) is unreliable.
* **Runtime discovery:** Interactive CLI sessions, shell auto-completion systems, and remote federated command networks need to dynamically query what capabilities are available on a target system at runtime.

## Solution
Expose a built-in capability in the CLI runner to perform self-analysis and export its own command registry schema on demand. 

By utilizing compile-time command registration (e.g., using linkme or similar linker hooks) and procedural macros (`#[noun]` and `#[verb]`), the system captures command signatures, docstrings, and parameter types during build. A global `--introspect` flag (or an `introspect` system verb) is provided. When invoked, the binary formats this registry into standard JSON Schema or LLM tool definitions and prints it, terminating without running any side-effect-heavy domain code.

### Example Introspection Output

Running `myapp --introspect` or `myapp system introspect` outputs a structured description:

```json
{
  "commands": [
    {
      "noun": "calc",
      "verb": "add",
      "description": "Calculates the sum of two integers.",
      "arguments": [
        {
          "name": "x",
          "type": "integer",
          "required": true,
          "description": "First integer operand"
        },
        {
          "name": "y",
          "type": "integer",
          "required": true,
          "description": "Second integer operand"
        }
      ]
    }
  ]
}
```

## Resulting Context / Connections
* **Zero schema drift:** The introspected schemas are guaranteed to match the compiled binary because they are constructed directly from the code's AST/types.
* **Auto-agentification:** An orchestrating LLM can call the CLI with `--introspect` at startup, parse the schema, and automatically register the entire command line as tools it can call.
* **Connections:** Leverages **Domain Separation** (so that introspection metadata can be collected without executing domain logic or side effects) and integrates with **JSON by Default** (as the schema describes the exact structure of the JSON objects passed in and returned).
