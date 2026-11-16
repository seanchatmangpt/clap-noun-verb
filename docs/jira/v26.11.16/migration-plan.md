# clap-noun-verb migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/clap-noun-verb
- Remote: https://github.com/seanchatmangpt/clap-noun-verb.git
- Branch: fix/v26.8.8-feature-closure@ad16886 tracking origin/<same> ; last 2026-10-13 ;
  602 commits ; 730 MB
- Languages/build: Rust / cargo workspace (Cargo.toml, crates/, benches/), package.json, Justfile
- Nested repos: none (a stray empty `~/ggen` directory exists at repo root, 0 B)

## Observed state
- Dirty 2: CHANGELOG.md, templates/TEMPLATES.md
- Untracked 14: docs/jira/ (this catalogue dir and prior v26.9.19 notes)
- Ahead 0 / behind 0 on current branch; `main` is ahead 5 / behind 130 vs origin/main
- Stash: 210 entries (census) — large, un-triaged
- Unpushed local branches: 34 (mostly `worktree-wf_*` auto-snapshots, plus docs-80-20-rewrite,
  exp/cheat-t02-cargo-cicd, integration-attempt-20260903, wire-ggen-cheat-scanner-guard)
- No target/ dir present at root; 730 MB is largely git objects + evidence/report files

## What it is
Declarative noun-verb CLI framework for Rust (clap-based), with ontology-driven capability
catalog and evidence-backed verification surfaces. Published on crates.io.

## Classification
ACTIVE (census agrees; CLAUDE.md present, recent jira v26.9.19).

## Migration risk
HIGH. 210 stashes and 34 unpushed branches (`main` itself has 5 unpushed commits) would be
lost on a fresh clone. Working-tree edits are small (2 files + docs/jira).

## Recommended action
COMMIT-PUSH
1. `cd /Users/sac/clap-noun-verb && git add CHANGELOG.md templates/TEMPLATES.md docs/jira`
2. `git commit -m "docs: changelog/template notes + jira v26.11.16 catalogue"`
3. `git push origin fix/v26.8.8-feature-closure`
4. `git push origin main` (publishes the 5 local-only commits; DECISION: main is behind 130,
   expect a non-fast-forward — push to a side branch `main-local-20261116` instead if rejected)
5. `git push origin --all` then `git push origin --tags` (captures the 34 local branches;
   DECISION: delete the `worktree-wf_*` auto-snapshot branches instead of pushing them?)
6. Stashes: `git bundle create /Volumes/ARCHIVE/clap-noun-verb-full.bundle --all` plus copy
   `.git/refs/stash` and `.git/logs/refs/stash` so the 210 stashes survive as history.
7. Remove the stray `~` directory at repo root (`rmdir "/Users/sac/clap-noun-verb/~/ggen"`).
8. Cleanup before any copy: `rm -rf target` in every crate dir (none found at root now).

## First commands on the new machine
`cargo check --workspace --all-features && cargo test --workspace`

## See Also
- ../v26.9.19/
- /Users/sac/docs/jira/v26.11.16/README.md (master index)
