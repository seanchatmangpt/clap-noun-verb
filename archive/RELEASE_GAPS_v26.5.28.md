# Release Gap Analysis & Capability Matrix: v26.5.28

This report evaluates the release readiness of the `clap-noun-verb` framework for version `v26.5.28`. It details all outstanding CLI, macro, validation, formatting, and safety capabilities that must be implemented, remediated, or verified before release.

---

## 1. Capability Matrix

| Capability Area | Feature / Check | Status | Required Action for Release `v26.5.28` | Associated Files |
|---|---|---|---|---|
| **CLI Core & Routing** | Subcommand routing and delimited execution | **Production Ready** | None. DELIMITER (`"++"`) is fully functional. | `src/registry.rs`, `src/cli/router.rs` |
| **CLI Core & Routing** | CLI Option Parsing | **Production Ready** | None. Standard parsing via `clap` handles options. | `src/cli/registry.rs`, `src/lib.rs` |
| **CLI Core & Routing** | Interactive Help Interface | **Broken (Coupled)** | Decouple hardcoded `ggen` command lists, templates, and AI prompts. Make it generic. | `src/cli/interactive.rs` |
| **CLI Core & Routing** | Command Category Directory | **Broken (Coupled)** | Decouple hardcoded `ggen` categories (`Pack`, `AI`, `Marketplace`). Allow user-defined categories. | `src/cli/help.rs` |
| **CLI Core & Routing** | CLI Command Examples Registry | **Broken (Coupled)** | Decouple hardcoded workflow examples referring to `ggen` commands. | `src/cli/examples.rs` |
| **CLI Core & Routing** | Configuration Loading & Merging | **Orphaned (Disabled)** | Compile the config module by adding `pub mod config;` to `src/lib.rs`. | `src/config.rs`, `src/lib.rs` |
| **CLI Core & Routing** | Route Matching & Middleware | **Orphaned (Disabled)** | Resolve broken dependencies on non-existent `crate::middleware` and declare the module. | `src/router.rs`, `src/lib.rs` |
| **CLI Core & Routing** | V2 Noun/Verb Command Layouts | **Orphaned (Disabled)** | Omitted from compilation. Register inside `src/verb.rs` if needed or clean up. | `src/verb/command.rs`, `src/verb/v2.rs` |
| **CLI Core & Routing** | CLI Preprocessor | **Safety Issue** | Fix infinite loop risk in step-reference replacement loop when resolved value contains `"@{"`. | `src/cli/preprocessor.rs` |
| **Macros** | Production Core (`#[verb]`, `#[arg]`) | **Production Ready** | None. Verified compile-time validation of complexity/types. | `clap-noun-verb-macros/src/lib.rs` |
| **Macros** | Frontier Layer (`#[meta_aware]`, etc.) | **Pre-production (Gaps)**| Keep disabled/uncompiled for release, or fix compilation and export missing types (`OptimizationHint`, `Capability`). | `clap-noun-verb-macros/src/meta_framework.rs` |
| **Macros** | Reflexive Testing (`#[auto_test]`) | **Stub** | Replace `assert!(true)` mock code generator with actual test assertion checks. | `clap-noun-verb-macros/src/macros/reflexive_testing_macro.rs` |
| **Validation** | SemVer Comparison | **Broken** | Replace lexicographical byte string comparison with proper semantic version parsing. | `src/deprecation.rs` |
| **Formatting** | Shared Utilities (`completions`, `display_json`, etc.) | **Production Ready** | None. Playgrounds and examples correctly utilize the shared library. | `utils/src/` |
| **Formatting** | ANSI Terminal Styling (Help) | **Production Ready** | None. CJK/emoji layout calculation handles width 2 successfully. | `utils/src/help.rs` |
| **Safety & Quality** | Strict Clippy Compliance | **Broken** | Remove three `.unwrap()` calls in registry JSON generation that violate workspace deny-unwrap rules. | `src/cli/registry.rs` |
| **Safety & Quality** | Telemetry Versioning | **Inconsistent** | Synchronize hardcoded `"1.0.0"` and `"3.8.0"` CLI version headers with actual crate versions. | `src/telemetry.rs` |
| **Verification** | Automation & Playgrounds | **Gap** | Include `playground` and `examples/playground` in cargo workspace check or CI script commands. | `Cargo.toml` |

---

## 2. Detailed Release Gaps & Blockers

### G1. Strict Clippy Compiler Errors (Build Blocker)
- **Description**: The workspace denies `unwrap` lint violations, but the core library contains three `.unwrap()` calls inside error-handling JSON generators. This prevents compilation under `cargo clippy`.
- **Occurrences**:
  - `src/cli/registry.rs:801`
  - `src/cli/registry.rs:858`
  - `src/cli/registry.rs:862`
- **Remediation**: Use `serde_json::to_string(...)` and propagate formatting errors via `?` or map them to `NounVerbError::SerializationError`.

### G2. Incorrect SemVer Comparison in `deprecation.rs` (Logic Blocker)
- **Description**: In `deprecation.rs`, the `is_removable` comparison evaluates versions as byte slices: `current_version.as_bytes() >= removed.as_bytes()`.
- **Defect**: Lexicographical comparisons fail once the version contains multiple digits (e.g. `"10.0.0" < "4.0.0"`).
- **Remediation**: Parse versions using the `semver` crate (or a simple integer tuple conversion) to perform proper numeric comparison.

### G3. Hardcoded Domain Coupling (Architecture Blocker)
- **Description**: Several modules inside the core framework library are statically coupled to the `ggen` CLI tool application domain.
- **Impacted Files**:
  - `src/cli/interactive.rs`: References `Welcome to ggen Interactive Help`, Ollama setup, and templates.
  - `src/cli/help.rs`: Categories like `Pack`, `AI`, and `Marketplace` are hardcoded.
  - `src/cli/examples.rs`: Statically registers hardcoded `ggen` workflow examples.
- **Remediation**: Redesign these modules to accept configurations, categories, and examples at runtime during registry initialization.

### G4. Orphaned/Uncompiled Source Code (Completeness Blocker)
- **Description**: Multiple source files are present in the filesystem but completely missing from module declarations in `src/lib.rs` and `src/verb.rs`, meaning they are not built.
- **Omitted Files**:
  - `src/config.rs`: Fully functional config file loader and watcher.
  - `src/router.rs`: Features broken imports referencing a non-existent `crate::middleware` module.
  - `src/verb/command.rs` & `src/verb/v2.rs`: Legacy/V2 layouts.
- **Remediation**: Declare `pub mod config;` in `src/lib.rs`. Fix dependencies in `src/router.rs` and declare `pub mod router;` if it should be public.

### G5. Preprocessor Infinite Loop Risk (Safety Blocker)
- **Description**: Argument preprocessing uses a `while let Some(start_idx) = new_arg.find("@{")` loop to expand variables.
- **Defect**: If the resolved value of a variable contains the substring `"@{"`, the loop will match it indefinitely, leading to an infinite hang.
- **Remediation**: Track and increment the search index rather than performing a fresh find from index `0`, or block nesting cycles explicitly.

### G6. Uncompiled Playgrounds in CI (Verification Blocker)
- **Description**: `playground` and `examples/playground` are excluded from workspace members in the root `Cargo.toml`.
- **Defect**: Standard workspace validation checks (`cargo check --workspace` / `cargo test --workspace`) skip these packages. API breakages in the utility consumption of these crates are not caught by standard CI runs.
- **Remediation**: Explicitly cd into playground directories and run checks in CI, or resolve workspace structure to include them.

### G7. Miscellaneous Code Placeholders & Compile Warnings
- **Placeholders**:
  - `playground/src/domain/capability.rs:41`: `// TODO: Implement resolution logic`
  - `src/error.rs:101`: `/// FUTURE: v5.1 - Complete RDF recovery suggestions` (rdf-control features deferred to v5.1).
- **Warnings**:
  - Unused variables and imports exist inside tests (`tests/completions_e2e.rs`, `tests/cli_validator.rs`, `tests/cli/telemetry_cli_tests.rs`) and in `speckit-ralph/src/main.rs`.
- **Remediation**: Implement or clean up the single `TODO` in the playground. Address unused imports/variables.

---

## 3. Toolchain & Testing Status

- **Toolchain Constraint**: Workspace targets require the `nightly` Rust toolchain because `unibit-cli` relies on unstable generic features (`generic_const_exprs`) via `unibit-kernel`. Standard compilation under `stable` fails.
- **Test Suite Stats**:
  - Core library tests: **97 passed**
  - Macros tests: **120 passed**
  - Utilities tests: **48 passed** (24 integration, 24 doc-tests)
  - All tests compile and run successfully using the `nightly` compiler.
