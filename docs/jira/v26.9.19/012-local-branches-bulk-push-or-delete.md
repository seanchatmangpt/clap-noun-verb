# clap-noun-verb: triage 33 local-only branches

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: local branches with commits not on `origin/main` and no upstream
- Evidence: `git rev-list --count origin/main..<branch>` > 0 for each: exp/cheat-t02-cargo-cicd:158 integration-attempt-20260903:160 wire-ggen-cheat-scanner-guard:156 worktree-wf_8eb95093-5ee-1:1 worktree-wf_8eb95093-5ee-10:1 worktree-wf_8eb95093-5ee-2:1 worktree-wf_8eb95093-5ee-3:1 worktree-wf_8eb95093-5ee-4:1 worktree-wf_8eb95093-5ee-5:1 worktree-wf_8eb95093-5ee-6:1 worktree-wf_8eb95093-5ee-7:1 worktree-wf_8eb95093-5ee-8:1 worktree-wf_8eb95093-5ee-9:1 worktree-wf_9a2fbe6c-e8a-1:1 worktree-wf_9a2fbe6c-e8a-10:1 worktree-wf_9a2fbe6c-e8a-11:1 worktree-wf_9a2fbe6c-e8a-2:1 worktree-wf_9a2fbe6c-e8a-3:1 worktree-wf_9a2fbe6c-e8a-4:1 worktree-wf_9a2fbe6c-e8a-5:1 worktree-wf_9a2fbe6c-e8a-6:1 worktree-wf_9a2fbe6c-e8a-7:1 worktree-wf_9a2fbe6c-e8a-8:1 worktree-wf_9a2fbe6c-e8a-9:1 worktree-wf_e3c8ef84-1d2-11:1 worktree-wf_e3c8ef84-1d2-12:1 worktree-wf_e3c8ef84-1d2-13:1 worktree-wf_e3c8ef84-1d2-14:1 worktree-wf_e3c8ef84-1d2-15:1 worktree-wf_e3c8ef84-1d2-16:1 worktree-wf_e3c8ef84-1d2-17:1 worktree-wf_e3c8ef84-1d2-18:1 worktree-wf_e3c8ef84-1d2-19:1

## Work to complete
- Triage each branch: push (`git push -u origin <branch>`) or delete after confirming the commits are obsolete/recoverable. Batch the work; record decisions in History.

## Acceptance
- Every listed branch is pushed or deleted; no local-only branch with unique commits remains.

## History
- 2026-09-19 | OPEN | survey found 33 local-only branches | full list above | triage pending
