# Wizard Package Type System and State Machine Diagram

## Type-Level State Machine Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     WizardSession<S: SessionState>                   │
│                                                                       │
│  Type Parameter S encodes state at compile time (zero-cost)          │
└─────────────────────────────────────────────────────────────────────┘

State Transitions (Type-Safe, Compile-Time Enforced):

    ┌─────────────┐
    │     New     │  Initial state after build()
    └──────┬──────┘
           │ start()
           ▼
    ┌─────────────┐
    │  Prompting  │◄─────────┐ Main loop state
    └──┬──────┬───┘          │
       │      │               │
       │      │ abort()       │
       │      ▼               │
       │   ┌─────────┐        │
       │   │ Aborted │        │ process()
       │   └─────────┘        │
       │                      │
       │ submit_response()    │
       ▼                      │
    ┌────────────┐            │
    │ Processing ├────────────┘
    └──────┬─────┘
           │ complete()
           ▼
    ┌──────────────┐
    │   Complete   │  Terminal state with receipt
    └──────────────┘
```

## Core Type Hierarchy

```
WizardBuilder (Construction API)
│
├── prompts: Vec<WizardPrompt>
│   └── WizardPrompt
│       ├── id: PromptId
│       ├── text: String
│       ├── response_type: ResponseType
│       │   ├── Text
│       │   ├── Integer { min, max }
│       │   ├── Choice { options }
│       │   ├── Boolean
│       │   ├── FilePath { must_exist }
│       │   └── Custom { validator: fn }
│       └── constraints: Vec<Constraint>
│
├── config: WizardConfig
│   ├── max_prompts: usize
│   ├── enable_ai: bool (feature-gated)
│   ├── ai_config: AIConfig (feature-gated)
│   ├── output_format: OutputFormat
│   └── timeout: Option<Duration>
│
└── context: AppContext
    └── (inherited from parent CLI verb)

            │ build()
            ▼

WizardSession<New>
│
├── state: PhantomData<New>  (zero-cost marker)
├── id: SessionId
├── conversation: Vec<Exchange>
│   └── Exchange
│       ├── prompt: WizardPrompt
│       └── response: WizardResponse
│           ├── prompt_id: PromptId
│           ├── raw_input: String
│           ├── parsed_value: serde_json::Value
│           ├── timestamp: Timestamp
│           └── ai_suggestions: Option<Vec<String>>
│
├── context: AppContext
└── config: WizardConfig

            │ start()
            ▼

WizardSession<Prompting> → WizardSession<Processing> → WizardSession<Complete>
                                                                │
                                                                │ receipt()
                                                                ▼
                                                        WizardReceipt
                                                        ├── session_id
                                                        ├── exchanges: Vec<Exchange>
                                                        ├── metadata: ReceiptMetadata
                                                        └── hash: ReceiptHash (crypto)
```

## Zero-Cost Abstractions

```
┌──────────────────────────────────────────────────────────────┐
│                    Zero-Cost Wrappers                         │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  #[repr(transparent)]                                         │
│  SessionId(uuid::Uuid)                                        │
│    → Same memory layout as Uuid                               │
│    → No indirection                                           │
│    → Zero runtime cost                                        │
│                                                               │
│  #[repr(transparent)]                                         │
│  PromptId(u64)                                                │
│    → Same memory layout as u64                                │
│    → Fits in CPU register                                     │
│    → Zero runtime cost                                        │
│                                                               │
│  PhantomData<S>                                               │
│    → Zero-sized type                                          │
│    → Optimized away at compile time                           │
│    → Only used for type checking                              │
│                                                               │
└──────────────────────────────────────────────────────────────┘

Memory Layout Verification:

assert_eq!(
    size_of::<WizardSession<New>>(),
    size_of::<WizardSession<Complete>>()
);  // True - PhantomData has no runtime cost

assert_eq!(
    size_of::<SessionId>(),
    size_of::<uuid::Uuid>()
);  // True - #[repr(transparent)] wrapper

assert_eq!(
    size_of::<PhantomData<SessionState>>(),
    0
);  // True - zero-sized type
```

## Error Handling Flow

```
User Input → Validation → WizardError?
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                 │
            ▼                 ▼                 ▼
    PromptValidation  ResponseValidation   AIService
            │                 │                 │
            └─────────────────┴─────────────────┘
                              │
                              ▼
                    RecoveryStrategy::recover()
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                 │
            ▼                 ▼                 ▼
         Retry             Skip            Abort
            │                 │                 │
            └─────────────────┴─────────────────┘
                              │
                              ▼
                      Continue or Terminate
```

## Integration with clap-noun-verb

```
┌────────────────────────────────────────────────────────────────┐
│                    User CLI Command                             │
│  #[verb(name = "init", noun = "project")]                       │
│  async fn init_project(ctx: AppContext) -> Result<...>          │
└────────────────────┬───────────────────────────────────────────┘
                     │
                     │ Pass context
                     ▼
┌────────────────────────────────────────────────────────────────┐
│                   WizardBuilder::new()                          │
│  .context(ctx)                                                  │
│  .prompt_text("Project name:")                                  │
│  .prompt_choice("Type:", &["lib", "bin"])                       │
│  .build()?                                                      │
└────────────────────┬───────────────────────────────────────────┘
                     │
                     │ Build session
                     ▼
┌────────────────────────────────────────────────────────────────┐
│                WizardSession<New>::start()                      │
│  run_wizard_session(session).await?                             │
└────────────────────┬───────────────────────────────────────────┘
                     │
                     │ Interactive loop
                     ▼
┌────────────────────────────────────────────────────────────────┐
│              WizardSession<Complete>                            │
│  .responses() → serde_json::Value                               │
│  .receipt() → WizardReceipt                                     │
│  .format_output(OutputFormat::Json)?                            │
└────────────────────┬───────────────────────────────────────────┘
                     │
                     │ Return to verb handler
                     ▼
┌────────────────────────────────────────────────────────────────┐
│              Use responses to execute action                    │
│  initialize_project_from_responses(&responses)?                 │
└────────────────────────────────────────────────────────────────┘
```

## Feature Flag Conditional Compilation

```
╔═══════════════════════════════════════════════════════════════╗
║                   Without "wizard" feature                     ║
╠═══════════════════════════════════════════════════════════════╣
║  • wizard module NOT compiled                                  ║
║  • Zero overhead in binary size                                ║
║  • No rust-genai dependency                                    ║
║  • Existing CLI functionality unaffected                       ║
╚═══════════════════════════════════════════════════════════════╝

╔═══════════════════════════════════════════════════════════════╗
║                    With "wizard" feature                       ║
╠═══════════════════════════════════════════════════════════════╣
║  #[cfg(feature = "wizard")]                                    ║
║  pub mod wizard;                                               ║
║                                                                ║
║  Full wizard functionality:                                    ║
║  ✓ WizardBuilder API                                           ║
║  ✓ Type-safe state machine                                     ║
║  ✓ AI-powered suggestions (rust-genai)                         ║
║  ✓ Async wizard execution                                      ║
║  ✓ Deterministic receipts                                      ║
╚═══════════════════════════════════════════════════════════════╝

AI-Specific Types (Nested Feature Gating):

  #[cfg(feature = "wizard")]
  pub struct WizardConfig {
      // Always available when wizard enabled
      pub max_prompts: usize,
      pub output_format: OutputFormat,

      // AI-specific fields (feature-gated within wizard)
      #[cfg(feature = "wizard")]
      pub enable_ai: bool,

      #[cfg(feature = "wizard")]
      pub ai_config: AIConfig,
  }
```

## Type Safety Example (Compile-Time Errors)

```rust
// ✅ CORRECT: Type-safe state transitions
let session = WizardBuilder::new()
    .prompt_text("Name:")
    .build()?;                          // → WizardSession<New>

let prompting = session.start();       // → WizardSession<Prompting>
let response = get_user_input();
let processing = prompting.submit_response(response)?;  // → WizardSession<Processing>
let complete = processing.complete();  // → WizardSession<Complete>
let receipt = complete.receipt();      // → WizardReceipt

// ❌ COMPILE ERROR: Can't call complete() on New state
let session = WizardBuilder::new().build()?;
let receipt = session.complete();
// ERROR: no method named `complete` found for struct `WizardSession<New>`

// ❌ COMPILE ERROR: Can't call submit_response() on Complete state
let completed = ...; // WizardSession<Complete>
completed.submit_response(response);
// ERROR: no method named `submit_response` found for struct `WizardSession<Complete>`

// ❌ COMPILE ERROR: Can't call start() twice
let session = WizardBuilder::new().build()?;  // WizardSession<New>
let prompting1 = session.start();  // Consumes session, returns Prompting
let prompting2 = session.start();  // ERROR: value used after move
```

## Performance Characteristics

```
┌─────────────────────────────────────────────────────────────┐
│            Operation              │  Target SLO              │
├───────────────────────────────────┼──────────────────────────┤
│  Session creation                 │  ≤ 1ms                   │
│  Prompt validation                │  ≤ 100μs per prompt      │
│  Response validation              │  ≤ 500μs per response    │
│  Receipt generation               │  ≤ 10ms                  │
│  AI suggestion (optional)         │  ≤ 2s (network-bound)    │
│                                   │                          │
│  Memory overhead per session      │  ~1KB + conversation     │
│  State machine overhead           │  0 bytes (PhantomData)   │
│  Wrapper type overhead            │  0 bytes (transparent)   │
└─────────────────────────────────────────────────────────────┘
```

## Security Model

```
┌────────────────────────────────────────────────────────────────┐
│                      Input Validation                           │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Input → Type Constraints → Validators → Parsed Value     │
│                      │                │                         │
│                      ▼                ▼                         │
│              ResponseType        Custom fn                      │
│               - Integer          - FilePath check               │
│               - Choice           - Regex match                  │
│               - Boolean          - Custom logic                 │
│                                                                 │
│  ✓ Type-safe constraints prevent injection                     │
│  ✓ File path validation prevents directory traversal           │
│  ✓ Choice validation prevents arbitrary input                  │
│                                                                 │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│                      Receipt Integrity                          │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  #[cfg(feature = "crypto")]                                     │
│                                                                 │
│  WizardReceipt → SHA3-256 hash → ReceiptHash                    │
│                                                                 │
│  verify() method:                                               │
│    1. Recompute hash from exchanges                             │
│    2. Compare with stored hash                                  │
│    3. Return true if match                                      │
│                                                                 │
│  ✓ Tamper detection                                             │
│  ✓ Audit trail verification                                     │
│  ✓ Reproducible builds                                          │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```
