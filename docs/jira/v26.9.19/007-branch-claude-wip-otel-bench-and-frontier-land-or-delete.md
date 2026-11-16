# clap-noun-verb: land or delete remote branch `claude/wip-otel-bench-and-frontier`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `claude/wip-otel-bench-and-frontier` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/clap-noun-verb --head claude/wip-otel-bench-and-frontier`) or delete (`git push origin --delete claude/wip-otel-bench-and-frontier`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `claude/wip-otel-bench-and-frontier`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | claude/wip-otel-bench-and-frontier | decision pending
