# System Flow Diagrams: v4 vs v5 Architecture
## Visual Analysis of Request Flows and Architectural Gaps

**Generated:** 2025-11-20
**Purpose:** Visual documentation to support GEMBA_WALK_FINDINGS.md

---

## Diagram 1: V4 CLI FLOW (Current Working Architecture)

```
┌───────────────────────────────────────────────────────────────────────┐
│                         V4 CLI ARCHITECTURE                          │
│                         (CURRENTLY WORKING)                          │
└───────────────────────────────────────────────────────────────────────┘

User Types CLI Command
        │
        │ argv: ["app", "config", "set", "key", "value"]
        ▼
┌───────────────────────┐
│  clap Parser          │ ◄─── Uses clap derives and attributes
│  (Argument Parsing)   │
└───────┬───────────────┘
        │ Parsed arguments
        ▼
┌───────────────────────┐
│  src/cli/mod.rs       │
│  run() or             │
│  CommandRegistry      │
└───────┬───────────────┘
        │
        ├─────────────────────────────────┐
        │                                 │
        ▼                                 ▼
┌───────────────────┐           ┌────────────────────┐
│  CommandRouter    │           │ TelemetryCollector │
│  Noun-Verb lookup │           │ (Optional)         │
└───────┬───────────┘           │ src/telemetry/     │
        │                       └────────────────────┘
        │ Route to handler
        ▼
┌───────────────────────┐
│  Handler Function     │
│  #[noun] #[verb]      │
│  Business Logic       │
└───────┬───────────────┘
        │ Result<Output, Error>
        ▼
┌───────────────────────┐
│  Output Formatter     │
│  JSON / Text          │
└───────┬───────────────┘
        │
        ▼
stdout/stderr (Human or Machine)
```

**Key Characteristics:**
- ✅ Entry point: `src/cli/run()`
- ✅ Uses: v4 CLI telemetry (`src/telemetry/`)
- ✅ Routing: Dynamic via CommandRegistry
- ✅ Output: Structured JSON or text
- ✅ Works today for CLI callers

---

## Diagram 2: V5 MACHINE FLOW (Expected but MISSING)

```
┌───────────────────────────────────────────────────────────────────────┐
│                      V5 MACHINE ARCHITECTURE                         │
│                    (SHOULD EXIST, DOES NOT)                          │
└───────────────────────────────────────────────────────────────────────┘

Machine Caller (SDK/Agent/MCP)
        │
        │ API Call or Protocol Message
        ▼
┌───────────────────────┐
│  ??? MISSING ???      │ ◄─── src/machine/ DOES NOT EXIST
│  Machine API Entry    │
│  src/machine/api.rs   │
└───────┬───────────────┘
        │ Structured Request
        ▼
┌───────────────────────┐
│  ??? MISSING ???      │ ◄─── No programmatic interface
│  Request Validator    │
│  Protocol Handler     │
└───────┬───────────────┘
        │
        ├─────────────────────────────────┐
        │                                 │
        ▼                                 ▼
┌───────────────────┐           ┌────────────────────────┐
│  AutonomicCli     │           │ AutonomicTelemetry     │
│  (EXISTS!)        │           │ (EXISTS!)              │
│ src/autonomic/    │           │ src/autonomic/         │
│ - Introspection   │           │ - TraceSpan            │
│ - Capability IDs  │           │ - TelemetryCollector   │
│ - Contracts       │           │ - PerformanceProfiler  │
└───────┬───────────┘           └────────────────────────┘
        │                                 ▲
        │                                 │
        │ Capability Execution            │ Observability
        ▼                                 │
┌───────────────────────┐                 │
│  Capability Handler   │─────────────────┘
│  (Autonomic layer)    │
└───────┬───────────────┘
        │ ExecutionReceipt
        ▼
┌───────────────────────┐
│  Structured Response  │
│  JSON/Protobuf        │
└───────┬───────────────┘
        │
        ▼
Machine Caller receives Response

❌ CRITICAL GAP: Entry point and protocol handler DO NOT EXIST
✅ Backend capabilities (autonomic layer) EXIST but are UNREACHABLE
```

---

## Diagram 3: UNIFIED ARCHITECTURE (Target State with Dispatcher)

```
┌───────────────────────────────────────────────────────────────────────┐
│                    UNIFIED v4/v5 ARCHITECTURE                        │
│                      (TARGET STATE - NOT IMPLEMENTED)                │
└───────────────────────────────────────────────────────────────────────┘

                        Request Arrives
                              │
                              ▼
                    ┌─────────────────────┐
                    │  Caller Detection   │ ◄─── NEW COMPONENT NEEDED
                    │  (RequestDispatcher)│      src/dispatcher.rs
                    │                     │
                    │  Detect caller via: │
                    │  - argv presence    │
                    │  - API entry point  │
                    │  - Protocol headers │
                    └──────────┬──────────┘
                               │
                ┌──────────────┴───────────────┐
                │                              │
         CLI Caller?                    Machine Caller?
                │                              │
                ▼                              ▼
    ┌───────────────────────┐      ┌───────────────────────┐
    │   V4 CLI PATH         │      │   V5 MACHINE PATH     │
    │                       │      │                       │
    │  src/cli/             │      │  src/machine/         │
    │  - CommandRegistry    │      │  - Machine API        │
    │  - Router             │      │  - Protocol Handler   │
    │  - Validator          │      │  - AutonomicCli       │
    │                       │      │                       │
    │  Telemetry:           │      │  Telemetry:           │
    │  src/telemetry/       │      │  src/autonomic/       │
    │  - CLI metrics        │      │  - TraceSpan          │
    │  - Command tracking   │      │  - Trillion-scale     │
    └───────────┬───────────┘      └───────────┬───────────┘
                │                              │
                │                              │
                └──────────────┬───────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │  Unified Telemetry  │ ◄─── NEW FACADE NEEDED
                    │  Facade             │      src/telemetry/unified.rs
                    │  (TelemetryManager) │
                    │                     │
                    │  Delegates to:      │
                    │  - CLI telemetry    │
                    │  - Kernel tracing   │
                    │  - Autonomic        │
                    └─────────────────────┘
                               │
                               ▼
                         Output Layer
                    (JSON/Text/Protocol)
```

**Critical Components MISSING:**
1. ❌ RequestDispatcher (src/dispatcher.rs)
2. ❌ Machine API subsystem (src/machine/)
3. ❌ Unified Telemetry Facade (src/telemetry/unified.rs)

**Components that EXIST:**
1. ✅ V4 CLI layer (src/cli/)
2. ✅ V5 Autonomic layer (src/autonomic/)
3. ✅ Kernel tracing (src/kernel/distributed_tracing.rs)

---

## Diagram 4: TELEMETRY ARCHITECTURE (Current Fragmentation)

```
┌───────────────────────────────────────────────────────────────────────┐
│                  TELEMETRY LAYER FRAGMENTATION                       │
│                   (THREE INDEPENDENT IMPLEMENTATIONS)                │
└───────────────────────────────────────────────────────────────────────┘

                    Tests Expect This API:
                    ┌─────────────────────┐
                    │ TelemetryManager    │ ◄─── DOES NOT EXIST
                    │                     │
                    │ .new(app_name)      │
                    │   → Result<Self,E>  │
                    │                     │
                    │ Span::new(name)     │
                    │   → Result<Span,E>  │
                    │                     │
                    │ TraceContext::new() │
                    │   → Result<Ctx,E>   │
                    └─────────────────────┘
                               ▲
                               │
                               │ Tests import from:
                               │ clap_noun_verb::telemetry::*
                               │
                               ❌ MISMATCH!
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼

┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│  Layer A:      │  │  Layer B:      │  │  Layer C:      │
│  CLI Telemetry │  │  Kernel        │  │  Autonomic     │
│                │  │  Tracing       │  │  Telemetry     │
├────────────────┤  ├────────────────┤  ├────────────────┤
│ Location:      │  │ Location:      │  │ Location:      │
│ src/telemetry/ │  │ src/kernel/    │  │ src/autonomic/ │
│                │  │ distributed_   │  │ telemetry.rs   │
│                │  │ tracing.rs     │  │                │
├────────────────┤  ├────────────────┤  ├────────────────┤
│ Types:         │  │ Types:         │  │ Types:         │
│ - Telemetry    │  │ - TraceContext │  │ - Telemetry    │
│   Collector    │  │ - Span         │  │   Collector    │
│ - Span         │  │ - TraceFlags   │  │ - TraceSpan    │
│ - Metrics      │  │ - SpanExporter │  │ - Histogram    │
│   Collector    │  │ - Tracing      │  │ - Performance  │
│ - Tracing      │  │   Provider     │  │   Profiler     │
│   Collector    │  │                │  │                │
├────────────────┤  ├────────────────┤  ├────────────────┤
│ API:           │  │ API:           │  │ API:           │
│ .new()         │  │ .new()         │  │ .new()         │
│  → Self        │  │  → Self        │  │  → Self        │
│                │  │                │  │                │
│ Span::new(     │  │ Span::new(     │  │ TraceSpan::    │
│   name,        │  │   name,        │  │   new_root(    │
│   trace_id)    │  │   &context)    │  │   operation)   │
│  → Self        │  │  → Self        │  │  → Self        │
├────────────────┤  ├────────────────┤  ├────────────────┤
│ Purpose:       │  │ Purpose:       │  │ Purpose:       │
│ CLI command    │  │ Distributed    │  │ Trillion-scale │
│ metrics        │  │ tracing W3C    │  │ autonomic      │
│                │  │ compatible     │  │ observability  │
├────────────────┤  ├────────────────┤  ├────────────────┤
│ Scope:         │  │ Scope:         │  │ Scope:         │
│ Single process │  │ Multi-process  │  │ Trillion       │
│ CLI commands   │  │ Kernel ops     │  │ invocations    │
└────────────────┘  └────────────────┘  └────────────────┘

             ┌──────────────────────────────┐
             │  NONE OF THESE MATCH TESTS   │
             │  No Result<T,E> wrappers     │
             │  Different constructor args  │
             │  Different type names        │
             └──────────────────────────────┘
```

**Root Cause:** Tests were written for a unified API that was never implemented. Three separate implementations exist for different purposes but no facade unifies them.

---

## Diagram 5: CALLER DETECTION FLOW (MISSING Component)

```
┌───────────────────────────────────────────────────────────────────────┐
│                    CALLER DETECTION MECHANISM                        │
│                    (SHOULD EXIST, DOES NOT)                          │
└───────────────────────────────────────────────────────────────────────┘

                         Request Entry
                              │
                              ▼
                    ┌─────────────────────┐
                    │  Caller Detection   │
                    │  Algorithm          │
                    └──────────┬──────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼

┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│  Check 1:      │  │  Check 2:      │  │  Check 3:      │
│  argv Present? │  │  API Entry?    │  │  Context Meta? │
└────┬───────────┘  └────┬───────────┘  └────┬───────────┘
     │                   │                   │
     │ If argv[0-N]      │ If API call       │ If protocol
     │ exist             │ via library       │ headers
     ▼                   ▼                   ▼
┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│  CLI Caller    │  │  SDK Caller    │  │  Agent/MCP     │
│                │  │                │  │  Caller        │
│  Route to:     │  │  Route to:     │  │  Route to:     │
│  v4 CLI path   │  │  v5 Machine    │  │  v5 Machine    │
│                │  │  path          │  │  path          │
│  Telemetry:    │  │                │  │                │
│  CLI layer     │  │  Telemetry:    │  │  Telemetry:    │
│                │  │  Autonomic     │  │  Autonomic     │
└────────────────┘  └────────────────┘  └────────────────┘

                    ┌─────────────────────┐
                    │  Environment Vars   │
                    │  Check:             │
                    │  CNV_CALLER_TYPE=   │
                    │  - cli              │
                    │  - sdk              │
                    │  - agent            │
                    │  - test             │
                    └─────────────────────┘

❌ NONE OF THIS EXISTS TODAY
```

---

## Diagram 6: DATA TRANSFORMATION FLOW (Layer Analysis)

```
┌───────────────────────────────────────────────────────────────────────┐
│              DATA TRANSFORMATION THROUGH LAYERS                      │
│              (SHOWING MISSING LAYERS)                                │
└───────────────────────────────────────────────────────────────────────┘

Layer 0: INPUT (Exists)
━━━━━━━━━━━━━━━━━━━━━━━━━━
CLI: argv[] or stdin
SDK: API call(args)
                │
                ▼
┌───────────────────────────────────────┐
│ Layer 1: UNIFIED ENTRY POINT          │ ◄─── ❌ MISSING
│ - Input validation                    │
│ - Caller detection                    │
│ - Telemetry context init              │
│                                       │
│ Output: ValidatedRequest              │
└───────────┬───────────────────────────┘
            │
            ▼
┌───────────────────────────────────────┐
│ Layer 2: DISPATCHER (Routing)         │ ◄─── ❌ MISSING
│ - Route to v4 or v5                   │
│ - Select telemetry layer              │
│ - Load appropriate handler            │
│                                       │
│ Output: RouteDecision                 │
└───────────┬───────────────────────────┘
            │
    ┌───────┴────────┐
    │                │
    ▼                ▼
┌─────────┐    ┌─────────┐
│ V4 Path │    │ V5 Path │ ◄─── ✅ Both EXIST
└────┬────┘    └────┬────┘
     │              │
     └──────┬───────┘
            ▼
┌───────────────────────────────────────┐
│ Layer 3: EXECUTION (Business Logic)   │ ◄─── ✅ EXISTS
│ - Execute command/capability          │
│ - Record telemetry                    │
│ - Generate output                     │
│                                       │
│ Output: ExecutionResult               │
└───────────┬───────────────────────────┘
            │
            ▼
┌───────────────────────────────────────┐
│ Layer 4: OUTPUT FORMATTING            │ ◄─── ✅ EXISTS
│ - Serialize to JSON/Text/Protocol     │
│ - Apply format preferences            │
│                                       │
│ Output: FormattedResponse             │
└───────────┬───────────────────────────┘
            │
            ▼
Layer 5: DELIVERY (Exists)
━━━━━━━━━━━━━━━━━━━━━━━━━━
stdout/stderr or API response
```

**Summary:**
- ✅ Layer 0, 3, 4, 5: EXIST
- ❌ Layer 1, 2: MISSING (root cause of v5 gaps)

---

## Diagram 7: DEPENDENCY MAP (Module Relationships)

```
┌───────────────────────────────────────────────────────────────────────┐
│                       MODULE DEPENDENCY MAP                          │
└───────────────────────────────────────────────────────────────────────┘

                        src/lib.rs (Root)
                              │
                              │ pub mod telemetry;
                              │ pub mod autonomic;
                              │ pub mod kernel;
                              │ pub mod cli;
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│ src/telemetry │   │ src/autonomic │   │ src/kernel    │
│               │   │               │   │               │
│ Exports:      │   │ Exports:      │   │ Exports:      │
│ - Telemetry   │   │ - Autonomic   │   │ - Trace       │
│   Collector   │   │   Cli         │   │   Context     │
│ - Span        │   │ - TraceSpan   │   │ - Span        │
│ - Metrics     │   │ - Telemetry   │   │ - Tracing     │
│   Collector   │   │   Collector   │   │   Provider    │
└───────────────┘   └───────────────┘   └───────────────┘
        │                     │                     │
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                    Tests expect imports:
                    clap_noun_verb::telemetry::*

                    ❌ But types are split across:
                    - clap_noun_verb::telemetry::
                    - clap_noun_verb::autonomic::
                    - clap_noun_verb::kernel::

┌───────────────────────────────────────────────────────────────────────┐
│  MISSING: Re-export facade in src/telemetry/mod.rs:                 │
│                                                                       │
│  pub use crate::kernel::distributed_tracing::TraceContext;           │
│  pub use crate::autonomic::telemetry::TraceSpan;                     │
│                                                                       │
│  pub struct TelemetryManager { /* facade */ }                       │
└───────────────────────────────────────────────────────────────────────┘
```

---

## Diagram 8: ARCHITECTURAL GAP SUMMARY (Visual Checklist)

```
┌───────────────────────────────────────────────────────────────────────┐
│              V5 RELEASE ARCHITECTURE CHECKLIST                       │
└───────────────────────────────────────────────────────────────────────┘

V4 CLI Path (Working)
━━━━━━━━━━━━━━━━━━━━━
✅ Entry point: src/cli/run()
✅ Routing: CommandRegistry
✅ Telemetry: src/telemetry/
✅ Output: JSON/Text formatter
✅ Tests: Passing for v4

V5 Machine Path (Broken)
━━━━━━━━━━━━━━━━━━━━━━━━━
❌ Entry point: src/machine/ MISSING
❌ Protocol handler: MISSING
❌ SDK API: MISSING
❌ Machine API docs: MISSING
✅ Backend: src/autonomic/ EXISTS
✅ Capabilities: EXISTS
❌ Accessibility: UNREACHABLE

Integration (Critical Gap)
━━━━━━━━━━━━━━━━━━━━━━━━━━
❌ Dispatcher: src/dispatcher.rs MISSING
❌ Caller detection: MISSING
❌ v4↔v5 routing: MISSING
❌ Unified entry: MISSING

Telemetry (Fragmented)
━━━━━━━━━━━━━━━━━━━━━━
✅ CLI telemetry: src/telemetry/ EXISTS
✅ Kernel tracing: src/kernel/ EXISTS
✅ Autonomic: src/autonomic/ EXISTS
❌ Unified facade: src/telemetry/unified.rs MISSING
❌ TelemetryManager: DOES NOT EXIST
❌ Test compatibility: FAILING

Tests (API Mismatch)
━━━━━━━━━━━━━━━━━━━
❌ Import paths: WRONG
❌ API signatures: WRONG
❌ Type names: WRONG
❌ Result wrappers: NOT PRESENT
❌ Test passing rate: 0%

┌───────────────────────────────────────────────────────────────────────┐
│  CRITICAL PATH TO v5 RELEASE:                                        │
│                                                                       │
│  1. Create TelemetryManager facade (src/telemetry/unified.rs)       │
│  2. Implement RequestDispatcher (src/dispatcher.rs)                 │
│  3. Create Machine API subsystem (src/machine/)                     │
│  4. Refactor test suite to match actual APIs                        │
│  5. Add v4↔v5 integration tests                                     │
│                                                                       │
│  Estimated effort: 26 hours                                          │
└───────────────────────────────────────────────────────────────────────┘
```

---

## NEXT STEPS FOR IMPLEMENTATION

Based on these diagrams, the implementation team should:

1. **Start with TelemetryManager Facade** (Diagram 4)
   - Unifies three telemetry layers
   - Matches test expectations
   - Enables test suite to compile

2. **Implement RequestDispatcher** (Diagrams 3, 5)
   - Routes between v4 and v5 paths
   - Detects caller type
   - Selects appropriate telemetry layer

3. **Create Machine API Subsystem** (Diagram 2)
   - Provides programmatic entry point
   - Wraps autonomic layer capabilities
   - Implements protocol handlers

4. **Fill Layer Gaps** (Diagram 6)
   - Complete Layer 1: Unified Entry Point
   - Complete Layer 2: Dispatcher

5. **Unify Namespaces** (Diagram 7)
   - Add re-exports to src/telemetry/mod.rs
   - Create type aliases for compatibility
   - Update lib.rs exports

---

**Document Status:** Complete
**Next Document:** Implementation plan with detailed code specifications
**Related:** GEMBA_WALK_FINDINGS.md
