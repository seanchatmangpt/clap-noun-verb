# Original User Request

## Initial Request — 2026-07-10T04:22:34Z

Audit all of the existing documentation (including `docs/` and top-level files), archive it, and rewrite it from scratch. The new docs will be aimed primarily at end-users using the tools/APIs, and will include standard documentation files as well as an mdBook structured as a Christopher Alexander style pattern language.

Working directory: /Users/sac/clap-noun-verb
Integrity mode: development

## Requirements

### R1. Audit and Archive Existing Documentation
Identify all documentation files, including the `docs/` directory and top-level markdown files (e.g., `README.md`, `CONTRIBUTING.md`). Move all of these existing files into a designated archive directory to preserve them.

### R2. Rewrite Standard Documentation
Write a new set of standard top-level documentation files targeting end-users of the tools and APIs. This must include at least a comprehensive `README.md` that explains how to use the project.

### R3. Create an mdBook for Pattern Language
Create a new mdBook documenting the project using a Christopher Alexander style pattern language. This should define the core concepts and workflows of the project as interconnected patterns.

## Acceptance Criteria

### Archive Verification
- [ ] A script or programmatic check confirms that the original documentation files have been moved to an `archive/` directory.
- [ ] No old documentation files remain in their original locations (except for newly generated replacements).

### Standard Documentation Verification
- [ ] A new `README.md` exists in the root directory.
- [ ] The `README.md` contains sections relevant to end-users (e.g., "Usage", "Getting Started").

### mdBook Verification
- [ ] An mdBook project is initialized in a designated directory (e.g., `pattern-book/` or `docs/`).
- [ ] Running `mdbook build` inside the mdBook directory completes successfully with no errors.
- [ ] The mdBook's `SUMMARY.md` contains at least 3 distinct pattern entries.
