# Plan - clap-noun-verb shared utility package

We will construct a shared `utils` package in the `clap-noun-verb` workspace to provide unified helpers for various clap ecosystem crates (`clap_complete`, `clap-num`, `clap_mangen`, `clap-markdown`, `display_json`, `clap-adapters`, `clap-help`).

## Phase 1: Research and Analysis
- Spawn an Explorer subagent (`teamwork_preview_explorer`) to analyze the existing codebase structure, dependencies, and execution contexts.
- Specifically investigate how `unibit-cli` and `speckit-ralph` are integrated and whether they or the main library can benefit from these common utilities.
- Verify version compatibility and required dependencies for `clap_complete`, `clap_mangen`, `clap_num`, etc.

## Phase 2: Design and Implementation
- Add the `utils` package as a workspace member or implement as a module within the core. Since the requirement asks to "construct a shared `utils` package containing common tools", a new workspace member `utils` (or cargo package) is highly appropriate.
- Configure `Cargo.toml` for the new `utils` package, specifying the dependencies: `clap`, `clap_complete`, `clap-num`, `clap_mangen`, `clap-markdown`, `serde_json`, etc.
- Implement the utility helpers:
  - Completion generation wrapper using `clap_complete`.
  - Manpage generation wrapper using `clap_mangen`.
  - Markdown documentation generation wrapper using `clap-markdown`.
  - Number parsing helpers utilizing `clap-num`.
  - Display JSON helpers utilizing `serde_json` or specific display logic.
  - Help formatting utilities using `clap-help`.
  - Adapt/conversion utilities for adapters.
- Ensure no stub implementations, placeholders, or TODOs.
- Ensure strict error handling and clippy warnings compliance (e.g., no `unwrap`/`expect` where forbidden, follow codebase lint settings).

## Phase 3: Verification and Integration Testing
- Create comprehensive integration tests under a `tests/` directory in the `utils` package or in the main workspace.
- Run tests and lint checks (clippy, rustfmt) using a worker/reviewer.
- Run a Forensic Auditor to ensure no cheating, mock implementations, or integrity issues.
