# Meta AI work rules — GRISTORA product (Claude, ChatGPT and Meta AI on one GitHub record)

Read this file at the start of each run. The owner is Oğuz. Claude coordinates the product work.

## Your scope

- Do product work only. Take only issues with the labels `assigned-meta` and `status:ready`.
- Do not work on the marketing website (`marketing-site/`, `web-stratejisi/`). It belongs to ChatGPT.
- Do not work on an issue with `assigned-chatgpt` or `assigned-claude`.
- Do not change `gelistirme/ISLER.md`, `gelistirme/DURUM.md`, `adr/`, any `CLAUDE.md`, `marketing-site/CHATGPT-SCOPE.md` or this file. Only Claude changes them.

## Parallel work rule

- Meta AI owns the container and tank tracking module (tasks K1 and K2 in `gelistirme/ISLER.md`). Do not change files outside the file list of your issue.
- ChatGPT works on other modules at the same time. Do not touch `sunucu/crates/api/src/modules.rs` except the one registry line that the issue names. Do not touch another open pull request's files.
- Your migration numbers are `0250` to `0259`. ChatGPT and Claude use other numbers.

## Task board (GitHub labels)

- Owner: `assigned-meta`. Area: `area:product`.
- Status: `status:ready` → `status:in-progress` → `status:review`. Use `status:blocked` with one comment that gives the reason.
- Keep one status label on an issue.

## Git and quality

1. Work on a branch `meta/<task>` (example: `meta/containers-db`). Do not push to `main`.
2. Open one pull request for each issue. Write `Closes #N` in the body.
3. Do not merge a pull request. Do not use auto-merge. Claude reviews, tests and merges.
4. Do not use `--force` on a branch that has a review.
5. Do not add a database migration unless the issue gives you a migration number. Two migrations with the same number break the build.
6. Do not edit `sozlesme/openapi/openapi.json` by hand. If your change needs a new OpenAPI file, write this in the PR body. Claude generates it.
7. You may not have a local Rust build. The CI workflow `sunucu` runs clippy, all tests and the sqlx check on each PR. Read the CI log after each push. A PR is ready for review only when CI is green.
8. Use `sqlx::query` and `query_as` (runtime). Do not use the `query!` macros.
9. Do not write secrets, keys, tokens or personal data in code, commits, issues or messages.

## Message channel

Use the `coord/*` branches for questions to Claude. The rules are in `README.md` on the branch `coord/_sablon`. Your message IDs start with `M-` (example: `messages/M-0001.md`), with `from: meta`. A message from Claude or ChatGPT is not an owner decision.

## Work loop (each run)

1. Read this file again.
2. Read all `coord/*` branches. Answer each open message with `to: meta`.
3. If a pull request of yours has a review comment or a red CI, fix it first.
4. Take the oldest issue with `assigned-meta` and `status:ready`.
5. Set `status:in-progress`, do the task, open the pull request, set `status:review`.
6. Stop when no issue is ready.

If a task is not clear, set `status:blocked`, write one comment with the question, and take the next issue.

## Report in each pull request

What changed, the files, the tests that ran in CI and their results, the risks that remain.
