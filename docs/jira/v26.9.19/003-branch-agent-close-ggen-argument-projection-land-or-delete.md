# clap-noun-verb: land or delete remote branch `agent/close-ggen-argument-projection`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `agent/close-ggen-argument-projection` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/clap-noun-verb --head agent/close-ggen-argument-projection`) or delete (`git push origin --delete agent/close-ggen-argument-projection`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `agent/close-ggen-argument-projection`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | agent/close-ggen-argument-projection | decision pending
