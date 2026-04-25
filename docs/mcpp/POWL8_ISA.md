# POWL8: The Process ISA

This document defines **POWL8** as the Instruction Set Architecture (ISA) for executable process geometry in MCPP and UniverseOS.

## 1. Definition
POWL8 is the **executable control alphabet** of UniverseOS. It is a minimal set of process instructions that allows POWL (Partial Order Workflow Language) to execute at programming-language speed.

## 2. Why an ISA?
Without an ISA, process models remain external notations that must be interpreted by slow orchestration engines. POWL8 brings process *inside* the execution model.

- **Micro-ops**: sequence, choice, loop, partial-order concurrency, and synchronization.
- **Native Concurrency**: Designed for AtomVM and concurrent runtime execution.
- **Zero-Cost Lowering**: MCPP lowers high-level semantic plans ($O^*$) directly into POWL8 micro-ops.

## 3. The Hierarchy
- **POWL**: The general formal process representation (Petri nets, BPMN, etc.).
- **POWL64**: The larger, durable process-state geometry stored in the graph.
- **POWL8**: The micro-architectural ISA for immediate execution.

## 4. Operational Invariant
Process itself belongs inside the execution model. Noun-verb commands are not just "invoked"; they are precipitated from the instruction geometry of a POWL8 process.
