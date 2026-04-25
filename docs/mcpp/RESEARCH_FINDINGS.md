# MCPP (MCP Plus) Research Findings

This document synthesizes the research for the extraction of **MCPP (MCP Plus)** from the `clap-noun-verb` core into a standalone semantic control substrate.

## 1. Executive Summary
MCPP is the "Universal Port" for **UniverseOS**. It serves as the operational execution surface where ontologies ($O^*$), process models (POWL8), and tools (MCP) converge into a lawful, receipt-producing execution loop.

## 2. Core Components for Extraction

### A. The RDF Engine (O*)
- **Ontology Store**: Found in `src/rdf/ontology.rs`.
- **SPARQL Planner**: Found in `src/rdf/sparql_executor.rs`.
- **Capability Metadata**: Found in `src/semantic/capability.rs`.

### B. The Universal Port (MCP)
- **McpServer**: Found in `src/rdf/mcp_server.rs`.
- **McpAdapter**: Found in `src/semantic/protocol.rs`.

### C. Law & Acceptance (Governance)
- **SHACL Guards**: Found in `src/rdf/guard_validation.rs`.
- **Policy Governance**: Found in `src/kernel/policy_governance.rs`.

### D. Memory & Audit (Provenance)
- **Lockchain**: Found in `src/rdf/lockchain.rs`.
- **Execution Receipts**: Found in `src/rdf/receipt.rs`.

## 3. Key Findings

### POWL8 as the ISA
The research confirms that **POWL8** is the Instruction Set Architecture (ISA) for UniverseOS. It is not just a workflow formalism but an executable control alphabet that enables process progression at programming-language speed.

### Spec Kit as an RDF-Native Protocol
Spec Kit is being transformed from a Markdown-based human workflow into an RDF-native build protocol:
- **Constitution**: Mapped to ODRL/SHACL.
- **Specification**: Mapped to DCAT 3/ADMS.
- **Plan**: Mapped to PROV-O/P-Plan.
- **Tasks**: Mapped to ActivityStreams.

## 4. Extraction Strategy
- **MCPP (Machine Identity)**: The protocol, runtime, and control plane.
- **MCP Plus (Public Identity)**: The marketplace, ecosystem, and project face.
- **Universal Lowering**: MCPP acts as the µ operator in $A = \mu(O^*)$, lowering public ontologies into executable POWL8 control geometry.
