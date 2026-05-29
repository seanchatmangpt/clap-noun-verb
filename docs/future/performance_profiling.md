# Performance Profiling and Optimization Guide

This guide details the methodologies, tools, and best practices for profiling and optimizing CLI applications built with the `clap-noun-verb` framework. 

CLI applications demand a different performance profile than long-running services: startup latency (time-to-first-output/help) must be sub-10 milliseconds, memory footprint (resident set size, RSS) must remain small for invocation in tight loop scripts, and allocations must be kept to a minimum.

---

## 1. The `clap-noun-verb` Architecture Profile

Understanding how `clap-noun-verb` is constructed is key to optimizing it.

### Compile-Time Registration (`linkme`)
* **How it works:** The framework uses the `linkme` crate to construct distributed slices (`__NOUN_REGISTRY` and `__VERB_REGISTRY`) of command metadata at compile time.
* **Performance Impact:**
  * **Startup cost:** Near-zero. There is no runtime scanning of directories or dynamic loading of libraries.
  * **Memory cost:** Minor read-only text segment size overhead (`.rodata`).
  * **Tree construction:** The final `clap::Command` tree is constructed dynamically on startup by reading this static slice. If you have hundreds of commands, constructing the clap tree can take several milliseconds.

### Metadata Leakage (`Box::leak`)
* **How it works:** To satisfy `clap`'s requirement for `&'static str` names, help text, and argument metadata, `clap-noun-verb` converts dynamic strings into static references at initialization via `Box::leak()`.
* **Performance Impact:**
  * **Memory footprint:** Typical applications leak less than 50KB. However, in extremely constraint-heavy environments, avoid generating dynamic help text or command names at runtime.
  * **Recommendation:** Use static string constants in macro attributes (`#[verb(about = "Static description")]`) rather than interpolating or generating descriptions dynamically.

---

## 2. Low-Overhead Startup Time Optimization

The golden rule of CLI applications is to **defer expensive operations until after command parsing**. If a user runs `cli --help` or inputs invalid arguments, they should not wait for database connections, configuration file parsing, or thread pool creation.

```
[CLI Executed] ──> [Min. Env Init] ──> [Clap Parse Arguments] ──> [Command Validated]
                                                                        │
                                             ┌──────────────────────────┘
                                             ▼
                                     [Initialize Tokio]
                                             │
                                             ▼
                                     [Parse Config File]
                                             │
                                             ▼
                                     [Execute Handlers]
```

### Techniques for Sub-10ms Startup

#### A. Defer Async Runtime Initialization (Tokio)
Do not place `#[tokio::main]` on your `fn main()`. A multi-threaded Tokio runtime spawns multiple background threads and coordinates synchronization, which takes **1 to 5 milliseconds** depending on system load.

* **Incorrect (High Overhead):**
  ```rust
  #[tokio::main]
  async fn main() {
      // Argument parsing happens inside the heavy multi-threaded runtime
      clap_noun_verb::run().await;
  }
  ```

* **Correct (Low Overhead):**
  Parse arguments using a synchronous bootstrap wrapper, and only run the async executor if the command requires it:
  ```rust
  fn main() -> Result<(), Box<dyn std::error::Error>> {
      // 1. Initialize synchronous telemetry/logging quickly
      setup_simple_logger();

      // 2. Parse arguments synchronously (fast exit on --help / invalid arguments)
      let matches = clap_noun_verb::parse_args()?;

      // 3. Spawns/enters runtime only when entering runtime-dependent verbs
      if matches.subcommand_name() == Some("daemon") {
          let rt = tokio::runtime::Builder::new_multi_thread()
              .enable_all()
              .build()?;
          rt.block_on(async { run_daemon().await })?;
      } else {
          // For simple command handlers, use a single-threaded current-thread runtime
          let rt = tokio::runtime::Builder::new_current_thread()
              .enable_all()
              .build()?;
          rt.block_on(async { run_sync_command(matches).await })?;
      }
      Ok(())
  }
  ```

#### B. Lazy Configuration Parsing
Do not read and parse configuration files (`config.toml`, `.env`, YAML files) globally during startup. Instead, load them inside the handler execution phase.

```rust
// In a verb handler:
#[verb(noun = "config", name = "get")]
fn config_get(args: HandlerInput) -> Result<HandlerOutput> {
    // Parser is initialized only when the specific verb is executing
    let config = LazyConfig::load()?;
    let value = config.get(&args.get_arg("key")?)?;
    Ok(HandlerOutput::from(value))
}
```

#### C. Stripping and Link-Time Optimizations
Reduce disk I/O and instruction cache misses by keeping the final binary small. Put this configuration in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = "fat"             # Performs link-time optimizations across crates
codegen-units = 1       # Reduces parallel code generation to maximize optimization
panic = "abort"         # Removes unwinding stack tables, shrinking size significantly
strip = true            # Strips symbols and debuginfo from the binary automatically
```

---

## 3. Controlling Memory Limits and Footprint

Memory footprint (RSS) is critical when CLI tools are executed concurrently (e.g., in a CI/CD pipeline or daemonized runners).

### Memory Allocators
By default, Rust uses the system allocator (`malloc` / `free`). While suitable on macOS/Windows, on Linux the default glibc allocator can fragment memory under rapid thread creation.

Consider swapping in high-performance allocators if your CLI processes heavy datasets:
* **`mimalloc`:** Excellent for multi-threaded CLI workloads and has a very low footprint.
* **`jemalloc`:** Best for large heaps and heavy multi-threading.

```rust
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

### Static String Reuse to Avoid `Box::leak`
Avoid dynamic allocation of strings when building command argument schemas:
* Use `'static` string slices directly instead of calling `.to_string()`.
* When validating input, check length and bounds before transforming parameters into owned data structures.

---

## 4. Target Profiling Tools

To debug startup time and memory footprint, use these target tools in development.

### A. Startup Benchmarking: `hyperfine`
`hyperfine` is the standard tool for measuring CLI execution time.

```bash
# Measure execution time of a quick command
hyperfine --warmup 3 'my-cli noun verb --help'

# Compare the startup speed against a baseline version
hyperfine --warmup 3 'my-cli-old --version' 'my-cli-new --version'
```

### B. Execution/CPU Profiling: `samply`
`samply` is a low-overhead, sampling CPU profiler for macOS and Linux. It outputs profile data that can be analyzed in the Firefox Profiler.

```bash
# Install samply
cargo install samply

# Run CPU profiling
samply record target/release/my-cli noun verb --input large_file.json
```
Once run, it automatically hosts a local web server displaying a timeline, call tree, and flame graph of the execution. Look for expensive functions like `clap::Command::build` or Tokio thread spawns.

### C. Allocation & Memory Profiling: `dhat`
`dhat` (DHAT from Valgrind, integrated directly via Rust macros) measures heap allocations, peak memory usage, and memory lifetimes.

1. Add the feature to your binary:
   ```toml
   [dependencies]
   dhat = { version = "0.3", features = ["ad-hoc"] }
   ```
2. Enable it in your entrypoint:
   ```rust
   fn main() {
       #[cfg(feature = "dhat-heap")]
       let _profiler = dhat::Profiler::new_heap();
       
       // Your application code...
   }
   ```
3. Run the binary:
   ```bash
   cargo run --features dhat-heap --release
   ```
4. Examine the generated `dhat-heap.json` in `viewer.html` (provided by `dhat` or uploaded to [https://valgrind.org/dhat/dh_view.html](https://valgrind.org/dhat/dh_view.html)) to find short-lived dynamic allocations during startup.

### D. Detailed Trace Analysis: `tracing` and `tracing-chrome`
If you need microsecond-accurate trace spans for custom logic, use `tracing` combined with `tracing-chrome` to output files compatible with Chrome’s tracing tool (`about:tracing`).

```rust
use tracing_subscriber::prelude::*;

fn main() {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();

    // The application logic now records span entries
    let _span = tracing::info_span!("clap_parsing").entered();
    // ... clap parsing logic
}
```
Open `chrome://tracing` in Google Chrome or Microsoft Edge and load the generated JSON file to inspect the timeline.

---

## 5. Performance Checklist for New Verbs

When writing new `#[verb]` command handlers, run through this verification checklist:

1. [ ] **No static initializer block blocking:** Are you sure the handler function has no heavy initialization in global or thread-local storage?
2. [ ] **Lazy resources:** Are configuration files, template caches, and database pools initialized *within* the function body rather than in `main`?
3. [ ] **Synchronous fallback:** Can this command be run on a single thread (`tokio::runtime::Builder::new_current_thread()`) instead of launching a multi-threaded pool?
4. [ ] **Zero unneeded clones:** Are we taking parameters by value or reference where possible instead of cloning strings?
5. [ ] **Static help metadata:** Are you avoiding dynamically generated about/help strings that require dynamic `Box::leak` overhead?
