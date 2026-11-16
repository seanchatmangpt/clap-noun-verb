# clap-noun-verb: land or delete remote branch `fanout/r83-tcps-ready-set`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `fanout/r83-tcps-ready-set` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/clap-noun-verb --head fanout/r83-tcps-ready-set`) or delete (`git push origin --delete fanout/r83-tcps-ready-set`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `fanout/r83-tcps-ready-set`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | fanout/r83-tcps-ready-set | decision pending
