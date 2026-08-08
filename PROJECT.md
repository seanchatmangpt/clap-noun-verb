# Project: clap-noun-verb Documentation Overhaul

## Architecture
- This project is a Rust command-line tool and library (`clap-noun-verb`) utilizing a noun-verb structure for declarative CLIs.
- Core components:
  - `clap-noun-verb` library (Rust crate).
  - `clap-noun-verb-macros` procedural macros.
  - Built-in capabilities (telemetry, REPL, JSON serialization).
  - Command registry built with compilation-time hooks (`linkme`).

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Archive Documentation | Move 24 files/dirs to archive/ | None | DONE |
| 2 | Rewrite Standard Docs | Create new README.md at root | M1 | DONE |
| 3 | Initialize mdBook | Initialize mdBook structure in docs/ | M1 | DONE |
| 4 | Write Pattern Language | Document 3 patterns in mdBook | M3 | DONE |
| 5 | Verify & Audit | Run tests, build mdBook, run auditor | M2, M4 | DONE |

## Interface Contracts
- None (documentation-only project changes)
