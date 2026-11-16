# clap-noun-verb: land or delete remote branch `agent/chatmangpt-namespace-26.7.29`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `agent/chatmangpt-namespace-26.7.29` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/clap-noun-verb --head agent/chatmangpt-namespace-26.7.29`) or delete (`git push origin --delete agent/chatmangpt-namespace-26.7.29`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `agent/chatmangpt-namespace-26.7.29`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | agent/chatmangpt-namespace-26.7.29 | decision pending
