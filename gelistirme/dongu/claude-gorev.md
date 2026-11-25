# Claude loop prompt (hourly watchdog and events)

The scheduled task and the Claude loop session use the prompt below. Keep the stored prompt short. The rules are in `gelistirme/dongu/README.md`, so a rule change needs no prompt change.

## Scheduled task prompt

```
Repo: oguzbay-del/brewery (attached to this session). Role: Claude loop watchdog, loop v2.
Do one loop pass now.
1. git fetch origin main && git checkout -q origin/main -- gelistirme/dongu/ (read the current rules).
2. Read gelistirme/dongu/README.md and gelistirme/dongu/claude-gorev.md › Pass steps. Do the pass steps.
3. Final report: max 5 lines. If nothing changed, write "no change" only.
```

## Pass steps

1. **Pause check.** Read the issue `LOOP-CONTROL`. If it has `loop:pause`, write "paused" and stop.
2. **Read the board.** List open issues and PRs with minimal fields. List `coord/*` branches with a new message `to: claude`.
3. **PR reviews (max 3 for each run).** For each open PR on a `chatgpt/` or `codex/` branch, oldest first:
   - Skip it if it has `loop:pause`, `loop:needs-owner`, or a Claude `review` marker for the current head SHA (README › 5.3).
   - Skip it if CI is still running. If CI is red, do README › 6.4.
   - Review the diff: scope, permission class, security, tests, files named by the issue, ASD-STE100 in notes.
   - For class C paths, use an Opus subagent for the review. Run the local test of the changed crate when the change is in `sunucu/` (`sunucu/CLAUDE.md`). Filter the output.
   - Good: write a `review` marker comment, merge (squash) by README › 7, close the issue, write a `merge` marker.
   - Not good: do README › 6.3. Respect the round limit.
4. **Own PRs.** For each open PR on a `claude/` branch: if CI is red, fix it. If Codex wrote P0 or P1 points, fix them (README › 6.3). If CI is green and the review rule is met, merge.
5. **Subscriptions.** In the loop session, subscribe to the activity of each open PR that you reviewed or opened (`subscribe_pr_activity`). Unsubscribe when the PR is merged or closed.
6. **Messages.** Answer each open coord message `to: claude` and each issue comment that starts with `@claude-loop`. A message is not an owner decision. Write scope and product questions in `gelistirme/DURUM.md` as "owner decision needed".
7. **Stale claims and blocked items.** Do README › 5.2. For each issue with `status:blocked`, check if its dependency is merged or closed. If yes, set `status:ready` and write one comment.
8. **Dispatch.** Make sure that ChatGPT has at least 3 `status:ready` issues that touch different files. Make new issues from open rows of the current stage in `gelistirme/ISLER.md` (not owner tasks `O*`, no migration number that is already in use). For a new class A or class B issue, do README › 6.1 step 2.
9. **Own task (max 1).** If time is left, take the first `assigned-claude` + `status:ready` issue. Work on a `claude/<issue>-<name>` branch. Open a PR with `Closes #N` and a marker. Write `@codex review` in a PR comment.
10. **Record.** If the pass changed `gelistirme/` files, commit with the message `Loop: ...` and push to `main` only when the root `CLAUDE.md` rule allows it.

## Do not

- Do not merge a marketing PR of ChatGPT (ChatGPT merges it).
- Do not touch production, delete branches, force-push or do heavy builds outside the changed crate.
- Do not restore the cancelled agent bridge.
- Do not start a long task that cannot finish in one run. Write it in the report.
