# Clap Noun-Verb Pattern Language

Welcome to the pattern language documentation for `clap-noun-verb`.

This pattern language provides a blueprint for designing command-line interfaces (CLIs) that are not only type-safe and pleasant for human operators, but also native, robust, and highly discoverable for automated agents (such as Large Language Models and script orchestrators).

## Core Philosophy

Traditional command-line design assumes a human operator typing commands and reading unstructured text in a terminal. While effective historically, this approach introduces significant challenges for programmatic automation. 

By applying a structured **Pattern Language**, we can systematically resolve the forces of CLI design to create systems that are easy to test, decouple presentation from execution, and support dynamic runtime discovery.

## The Patterns

This documentation covers three fundamental architectural patterns:

1. **[Domain Separation](patterns/domain_separation.md)**: Strict division of concern between pure application logic and command-line parsing code.
2. **[JSON by Default](patterns/json_by_default.md)**: Structuring CLI outputs as machine-readable JSON serialized objects rather than arbitrary human-centric text.
3. **[Reflexive Introspection](patterns/reflexive_introspection.md)**: Enabling the binary to describe its own API schema programmatically to eliminate documentation drift and support auto-discovery by AI agents.

## How they Connect

Together, these patterns form a cohesive lifecycle:
* **Domain Separation** allows business logic to be developed, tested, and exported cleanly.
* **JSON by Default** ensures that once executed, the output returned from the domain logic is wrapped in a structured data model suitable for programmatic consumption.
* **Reflexive Introspection** uses the structure defined in both layers to generate standard API schemas dynamically at runtime, allowing agents to understand, interact with, and verify the CLI with zero manual configuration.
