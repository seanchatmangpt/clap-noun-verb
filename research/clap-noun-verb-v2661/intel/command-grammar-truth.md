# clap-noun-verb v26.6.1: Command Grammar Truth Report

**Inspection Date:** 2026-06-01  
**Source Version:** 26.6.1  
**Inspector:** Claude Code (Haiku 4.5)  
**Method:** Source code analysis (lib.rs, verb.rs, noun.rs, registry.rs, macros)  
**Confidence Level:** HIGH

---

## Executive Summary

clap-noun-verb v26.6.1 does **NOT** implement "noun" and "verb" as language-level abstractions or reserved concepts. Instead:

1. **Nouns and verbs are implemented as Rust trait patterns** (`NounCommand`, `VerbCommand`)
2. **Developers choose to organize commands using these patterns** (not mandated)
3. **The dispatch mechanism is trait object polymorphism + recursive routing**
4. **Registration is handled by linkme distributed slices** (compile-time auto-discovery)
5. **All output is JSON-serialized by default** (designed for agents/LLMs, not humans)

This is a **framework for composable CLI patterns**, not a language or enforced naming convention.

---

## REALITY vs. ASSUMPTIONS

### Assumption: "noun" and "verb" are keywords or language features
**REALITY:** They are Rust traits defined as public interfaces:
- `pub trait NounCommand: Send + Sync { ... }` (src/noun.rs:77)
- `pub trait VerbCommand: Send + Sync { ... }` (src/verb.rs:469)

Developers **choose** to organize commands using these traits. The system doesn't force this naming or structure.

### Assumption: Commands are registered using "noun/verb" macros
**REALITY:** 
- `#[noun]` macro is **DEPRECATED** (v5.6.0+) and is now a no-op with a deprecation warning
- `#[verb]` macro is **the primary registration mechanism** but it's optional
  - Developers can also manually implement traits and call `CommandRegistry::register_noun()` and `CommandRegistry::register_verb()`
  - Both approaches work; macro is just convenience

### Assumption: "noun" and "verb" are the **only** ways to organize commands
**REALITY:** The framework supports arbitrary nesting:
- `NounCommand::sub_nouns()` allows nested command groups
- No limit on depth (limited only by clap's architecture)
- Developers can build any tree structure they want

---

## ACTUAL DISPATCH MECHANISM

### Layer 1: Trait-Based Polymorphism

All commands are registered as **trait objects**:

```rust
// From src/registry.rs:30-34
pub struct CommandRegistry {
    nouns: HashMap<String, Box<dyn NounCommand>>,
    verbs: HashMap<String, HashMap<String, VerbMetadata>>,
    root_verbs: HashMap<String, VerbMetadata>,
}
```

- Nouns stored as `Box<dyn NounCommand>` (trait objects)
- Verbs stored as `VerbMetadata` containing `Box<dyn Fn(HandlerInput) -> Result<HandlerOutput>>`
- No type information at runtime; all dispatch via trait method calls

### Layer 2: Linkme Distributed Slices (Compile-Time Discovery)

Commands are **auto-discovered at link time** using linkme:

```rust
// From src/cli/registry.rs:163-168
#[distributed_slice]
pub static __NOUN_REGISTRY: [fn()] = [..];

#[distributed_slice]
pub static __VERB_REGISTRY: [fn()] = [..];
```

**How it works:**

1. **Compile-time:** `#[verb]` macro (in clap-noun-verb-macros) generates a registration function
2. **Link-time:** linkme collects all registration function pointers into distributed slices
3. **Runtime:** `CommandRegistry::init()` (src/cli/registry.rs:261-278) iterates slices and calls each function:
   ```rust
   for init_fn in __NOUN_REGISTRY { init_fn(); }
   for init_fn in __VERB_REGISTRY { init_fn(); }
   ```
4. **Registration functions** call `CommandRegistry::register_verb_with_args()` to populate the HashMap

**Key insight:** The distributed slices are **empty at source compile-time**. Linkme **adds entries at link time** from all compiled object files. This is a linker-level mechanism, not a runtime registry.

### Layer 3: Recursive Routing

Once parsed by clap, dispatch is handled by `CommandRegistry::route_recursive()` (src/registry.rs:327-363):

```rust
fn route_recursive(
    &self,
    noun: &dyn NounCommand,
    noun_name: &str,
    matches: &ArgMatches,
    root_matches: &ArgMatches,
) -> Result<()> {
    if let Some((sub_name, sub_matches)) = matches.subcommand() {
        // Check if it's a verb
        if let Some(verb) = noun.verbs().iter().find(|v| v.name() == sub_name) {
            verb.run(&args)
        }
        // Check if it's a sub-noun
        else if let Some(sub_noun) = noun.sub_nouns().iter().find(|n| n.name() == sub_name) {
            self.route_recursive(sub_noun.as_ref(), sub_name, sub_matches, root_matches)
        }
    } else {
        noun.handle_direct(&args)
    }
}
```

**Algorithm:**
1. Extract the subcommand name from clap matches
2. Check if it's a verb (call `verb.run()`)
3. If not, check if it's a sub-noun (recurse)
4. If neither, try direct noun execution

This is a **tree traversal** that handles arbitrary nesting.

### Layer 4: Argument Extraction

Once a verb is selected, arguments are extracted via `VerbArgs` (src/verb.rs:89-436):

```rust
pub struct VerbArgs {
    pub matches: ArgMatches,
    pub parent_matches: Option<ArgMatches>,
    pub context: VerbContext,
}
```

Methods like `get_one<T>()`, `get_many<T>()`, `is_flag_set()` delegate directly to clap's `ArgMatches` API.

This is a **thin wrapper** that:
- Provides ergonomic access methods
- Handles type inference errors gracefully
- Carries context (noun name, verb name, extensions)

---

## COMMAND GROUPING: How Subcommands Are Organized

### By Design

The framework distinguishes between:
1. **Nouns** - command groups (e.g., `services`, `collector`, `config`)
2. **Verbs** - actions within a noun (e.g., `status`, `logs`, `restart`)
3. **Sub-nouns** - nested command groups (e.g., `services` → `remote` → `start`)

### But Terminology Is Flexible

The trait names are just conventions. A developer implementing `NounCommand` could name their struct anything. The trait name doesn't constrain the semantic meaning.

### Nesting Support

`NounCommand::sub_nouns()` returns a `Vec<Box<dyn NounCommand>>`, enabling arbitrary nesting:

```rust
// From src/noun.rs:87-90
fn sub_nouns(&self) -> Vec<Box<dyn NounCommand>> {
    Vec::new()
}
```

This is a **default implementation** (returns empty). Developers can override to support nested structures like:

```
myapp
  ├── services (noun)
  │   ├── status (verb)
  │   ├── logs (verb)
  │   └── remote (sub-noun)
  │       ├── start (verb)
  │       └── stop (verb)
  └── config (noun)
      ├── get (verb)
      └── set (verb)
```

---

## ACTION SELECTION: How Verbs Are Dispatched

### Execution Entry Point

When the user runs `myapp services status --verbose`, clap parses this into:

```
ArgMatches {
  subcommand: ("services", submatches_noun)
    subcommand: ("status", submatches_verb)
      arg: ("verbose", true)
}
```

### Dispatch Path

1. **CommandRegistry::route()** (src/registry.rs:304-322):
   - Extracts top-level subcommand name (`services`)
   - Finds noun in HashMap
   - Calls `route_recursive(noun, "services", submatches_noun, root_matches)`

2. **route_recursive()** (src/registry.rs:327-363):
   - Extracts next subcommand (`status`)
   - Checks `noun.verbs().find(|v| v.name() == "status")`
   - Calls `verb.run(&VerbArgs)`

3. **VerbCommand::run()** (src/verb.rs:481):
   - User-provided implementation
   - Receives `VerbArgs` (clap matches + context)
   - Returns `Result<()>` or `Result<T: Serialize>`

### No Explicit Dispatch Table

There's **no central dispatch table** mapping (noun, verb) pairs to handlers. Instead:

- Each `NounCommand` holds references to its verbs (via `verbs()` method)
- Dispatch is **late-bound** via trait method calls
- Handlers are **trait object closures** stored in `VerbMetadata`

This is classic **dynamic dispatch** using Rust's trait system.

---

## ARGUMENT MODELING: How CLI Arguments Are Represented

### Compile-Time Extraction

The `#[verb]` macro inspects the function signature and extracts argument metadata:

```rust
#[verb("status")]
fn show_status(
    #[arg(env = "FILTER")]
    filter: Option<String>,
    #[arg(default_value = "json")]
    format: String,
) -> Result<Status> { ... }
```

The macro:
1. Parses function parameters and `#[arg]` attributes
2. Infers types (String, u16, bool, PathBuf, etc.)
3. Generates `Vec<ArgMetadata>` with extracted properties
4. Embeds this in the registration function passed to linkme

### Runtime Representation

Arguments are stored as `ArgMetadata` (src/cli/registry.rs:192-246):

```rust
pub struct ArgMetadata {
    pub name: String,
    pub required: bool,
    pub is_flag: bool,
    pub help: Option<String>,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub value_parser: Option<String>,
    // ... 20+ more fields
}
```

This is a **flat structure** (not recursive) that captures all clap Arg properties.

### Handler Access

Inside a verb handler, arguments are accessed via `VerbArgs`:

```rust
impl VerbArgs {
    pub fn get_one<T: 'static>(&self, name: &str) -> Result<T> { ... }
    pub fn get_many<T: 'static>(&self, name: &str) -> Result<Vec<T>> { ... }
    pub fn is_flag_set(&self, name: &str) -> bool { ... }
}
```

These delegate to clap's `ArgMatches::get_one()`, `get_many()`, `get_flag()` methods.

**Type Safety:** The handler function signature defines types directly:

```rust
#[verb("config")]
fn set_config(
    key: String,      // Required string
    value: String,    // Required string
    #[arg(env = "TTL")]
    ttl: Option<u32>, // Optional u32
) -> Result<()> { ... }
```

The macro **generates** argument metadata from these types. There's **no separate schema file**; types are inferred from Rust signatures.

---

## REGISTRATION MECHANISM

### Primary: Linkme Distributed Slices

**Flow:**

1. Developer writes:
   ```rust
   #[verb("status")]
   fn show_status() -> Result<Status> { ... }
   ```

2. Macro generates (pseudo-code):
   ```rust
   #[linkme::distributed_slice(crate::cli::registry::__VERB_REGISTRY)]
   fn __register_verb_show_status() {
       CommandRegistry::register_verb_with_args(
           "services",
           "status",
           "Show service status",
           vec![/* args */],
           |input| {
               let args = VerbArgs::from_input(input);
               show_status()
                   .map(|result| serde_json::to_value(result).unwrap())
           }
       );
   }
   ```

3. Linkme adds function pointer to distributed slice at link time

4. At runtime, `CommandRegistry::init()` calls all pointers in slice

5. Nouns and verbs populated in `REGISTRY: OnceLock<Mutex<CommandRegistry>>`

### Fallback: Runtime Registration

For dynamic CLIs, developers can also call:

```rust
registry.register_verb_with_args(
    "services",
    "status",
    "Show status",
    vec![],
    |input| { /* handler */ }
);
```

This bypasses linkme entirely and inserts directly into the HashMap.

### Initialization

The `REGISTRY` is a singleton initialized on first access (src/cli/registry.rs:281-283):

```rust
pub fn get() -> &'static Mutex<CommandRegistry> {
    Self::init()
}
```

`OnceLock::get_or_init()` ensures only one initialization, even with concurrent access.

---

## OUTPUT SERIALIZATION

### Default: JSON

All handler return types must implement `serde::Serialize`:

```rust
#[verb("status")]
fn show_status() -> Result<Status> { ... }  // ✓ Status must impl Serialize

#[verb("invalid")]
fn invalid() -> Result<impl Display> { ... }  // ✗ COMPILE ERROR (not Serialize)
```

This is enforced at compile-time by the `#[verb]` macro (src/clap-noun-verb-macros/src/validation.rs).

### Serialization Flow

1. Handler returns `Result<T>`
2. Macro wraps in `HandlerOutput` enum
3. Framework serializes to JSON via `serde_json::to_string(output)`
4. JSON printed to stdout

### Customization

Global flags control format:

- `--format json|yaml|toml|csv|tsv|html`
- `--select <jsonpath|jmespath>` - filter output via JSONPath or JMESPath
- `--structured-errors` - output errors as JSON instead of plaintext

**Hook System:** Developers can register custom output validators:

```rust
register_output_validation_hook(|output: &str| {
    // Custom validation/transformation
    Ok(output.to_string())
});
```

---

## ERROR & REFUSAL HANDLING

### Error Type: NounVerbError

All failures return `NounVerbError` variants (src/error.rs):

- `ArgumentError` - invalid argument
- `InvalidStructure` - bad command tree (e.g., noun with no verbs)
- `CommandNotFound { noun, candidates }` - noun not registered
- `VerbNotFound { noun, verb, candidates }` - verb not in noun
- `ExecutionError` - handler failed

### Suggestion Mechanism

When a command is not found, the framework suggests candidates:

```rust
// From src/registry.rs:317-319
let candidates: Vec<&str> = self.nouns.keys().map(|s| s.as_str()).collect();
NounVerbError::command_not_found_with_candidates(noun_name, &candidates)
```

This generates error messages like:

```
Error: Command 'servces' not found

Did you mean one of these?
  - services
  - config
  - auth
```

### Validation

Two layers:

1. **Compile-time (macro validation):**
   - Return type must impl Serialize
   - No duplicate verbs
   - Function complexity limits (prevents business logic in verb layer)

2. **Runtime (CommandRegistry validation):**
   - `validate()` method checks for:
     - Duplicate noun names
     - Empty nouns (no verbs/sub-nouns)
     - Duplicate verb names within a noun
     - Verb/sub-noun name conflicts

---

## HELP GENERATION

### Sources

Help text comes from multiple sources:

1. **Trait methods:**
   ```rust
   impl NounCommand for Services {
       fn about(&self) -> &'static str { "Manage services" }
   }
   ```

2. **Docstrings (extracted by macro):**
   ```rust
   /// Show the status of all services
   /// 
   /// # Arguments
   /// 
   /// filter: Service name filter (optional)
   #[verb("status")]
   fn show_status(filter: Option<String>) -> Result<Status> { ... }
   ```

3. **Argument attributes:**
   ```rust
   #[arg(help = "Service name to filter by")]
   filter: Option<String>
   ```

### Clap Integration

The help text is converted to clap's `Command` and `Arg` objects:

```rust
// From src/registry.rs:479-482
let about: &'static str = Box::leak(verb_meta.about.clone().into_boxed_str());
let mut verb_cmd = clap::Command::new(verb_name_static).about(about);
```

**Memory note:** Strings are leaked to `&'static str` because clap requires static lifetimes. This is acceptable for CLI apps (one-time allocation during startup).

### Introspection Flag

The `--introspect` flag emits JSON Schema for all commands:

```bash
$ myapp --introspect
[
  {
    "name": "services_status",
    "description": "Show service status",
    "parameters": {
      "type": "object",
      "properties": {
        "filter": { "type": "string", "description": "..." }
      },
      "required": []
    }
  }
]
```

This is designed for **LLM tool-calling**, not human CLI help.

---

## COMMAND INTROSPECTION & LISTING

### Runtime Queries

The `CommandRegistry` exposes query methods (src/registry.rs:351-382):

```rust
pub fn get_nouns(&self) -> Vec<(&str, &str)>
pub fn get_verbs(&self, noun_name: &str) -> Vec<(&str, &str)>
pub fn get_all_noun_names(&self) -> Vec<&str>
pub fn command_structure(&self) -> HashMap<String, Vec<String>>
```

These allow programs to introspect the CLI at runtime.

### JSON Schema Introspection

The `--introspect` flag invokes `collect_tools_from_cmd()` (src/registry.rs:699-781):

1. Recursively traverses clap Command tree
2. For each leaf command (executable), generates `ToolDefinition`
3. Emits as JSON array

**Use case:** LLM agents can read this JSON and call any command via tool-calling without executing external scripts.

### Programmatic Discovery

Developers can iterate commands in code:

```rust
let registry = CommandRegistry::get();
for (noun_name, about) in registry.get_nouns() {
    for (verb_name, about) in registry.get_verbs(noun_name) {
        println!("{}::{}", noun_name, verb_name);
    }
}
```

---

## DESIGN PHILOSOPHY

### From Library Documentation (src/lib.rs:6-44)

The crate is explicitly described as:

> A framework for building **composable CLI patterns** on top of clap, similar to how Python's Typer provides a simpler interface over Click.

**Key principles:**

1. **Zero Boilerplate** - Just add `#[verb]` attributes to functions
2. **Auto-Discovery** - Commands automatically discovered at compile time (linkme)
3. **Type Inference** - Arguments inferred from function signatures
4. **JSON by Default** - Perfect for agents, MCP, and modern tooling
5. **Minimal Dependencies** - Core CLI needs only 9 crates

### Agent-First Design

The entire architecture is optimized for **machine consumption**, not human CLI UX:

- JSON output by default (not pretty-printed text)
- `--introspect` flag for LLM tool discovery
- `--structured-errors` for error JSON
- Serializable return types (not human-readable printing)

This is intentional. The framework is positioned for use in **agent/MCP ecosystems**, not traditional CLI tooling.

---

## CRITICAL LIMITATIONS

### 1. No Sync Methods in Traits

Traits are synchronous only:

```rust
pub trait VerbCommand: Send + Sync {
    fn run(&self, args: &VerbArgs) -> Result<()>;  // Sync only
}
```

Async is supported via a separate module (`src/async_verb.rs`), but not in the main trait. This limits support for async handlers in the core framework.

### 2. Memory Leaks via Box::leak()

Strings are converted to `&'static str` using `Box::leak()`:

```rust
let about: &'static str = Box::leak(verb_meta.about.clone().into_boxed_str());
```

This is acceptable for CLI apps (one-time allocation), but would be problematic for:
- Long-running services
- Dynamically loaded commands
- Memory-constrained environments

### 3. No Automatic Help Customization

Help text is static. Customizing help requires:
- Overriding `build_command()` on traits
- Manual clap `Command` manipulation

There's no declarative help customization DSL.

### 4. Distributed Slices Are Link-Time

Linkme is a linker-level mechanism. This means:
- Commands cannot be added after program start
- Dynamic plugin loading would require re-linking
- No REPL-style command injection (except at runtime via `register_verb()`)

---

## COMPARISON: What This Is NOT

### Not a Language
- "noun" and "verb" are not language features
- No special syntax (besides `#[noun]` and `#[verb]` attributes, which are optional)
- Fully composable via Rust's standard trait system

### Not a Binary Protocol
- No serialization format for command definitions
- No RPC or network layer
- Purely in-process trait dispatch

### Not a Pattern Library
- Not a collection of pre-built commands
- No "standard" set of nouns/verbs
- Developers define their own command semantics

### Not a Human-First CLI Framework
- No ANSI color output by default
- No interactive prompts or wizards (except in optional `wizard` feature)
- Designed for agent/LLM tool-calling, not terminal UX

---

## CLASSIFICATION SUMMARY

| Aspect | Classification | Rationale |
|--------|----------------|-----------|
| **Command Grouping (Nouns)** | TRAIT_BASED | NounCommand trait + HashMap registry |
| **Action Selection (Verbs)** | TRAIT_BASED | VerbCommand trait + recursive routing |
| **Argument Modeling** | MACRO_BASED | Extracted from function signatures by #[verb] |
| **Registration** | REGISTRY_MECHANISM | linkme distributed slices + OnceLock singleton |
| **Output Serialization** | GENERATED_TABLE | serde::Serialize on return types |
| **Error Handling** | EXPLICIT_NOUN_VERB_API | NounVerbError variants with suggestions |
| **Help Generation** | CLAP_NATIVE_WRAPPER | Clap's built-in + docstring extraction |
| **Introspection** | EXPLICIT_NOUN_VERB_API | JSON Schema via --introspect flag |

---

## CONCLUSION

**clap-noun-verb v26.6.1 is a thin trait-based framework for composable CLI patterns** built on top of clap. 

The terms "noun" and "verb" are **conventions**, not enforced concepts. The framework provides:

1. **Traits** for organizing commands (NounCommand, VerbCommand)
2. **Macros** for ergonomic registration (optional; traits can be used directly)
3. **Dispatch mechanism** via trait object recursion
4. **Auto-discovery** via linkme at compile time
5. **Output serialization** as JSON for agent consumption

The dispatch flow is straightforward: **clap parses → CommandRegistry::route → noun trait dispatch → verb trait dispatch → handler execution**.

There are no hidden mechanisms or special language features. All behavior is traceable through standard Rust trait polymorphism and the linker-level linkme mechanism.

---

**End of Report**
