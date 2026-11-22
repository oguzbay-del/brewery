# ChatGPT work rules — GRISTORA (Claude and ChatGPT on one GitHub record)

**Owner decision (Oğuz, 22 November 2026).** This replaces the rule of 15 November 2026 ("marketing website only"). ChatGPT is an independent contributor. It works on the marketing website and on product tasks that have the label `assigned-chatgpt`.

Do not restore the agent bridge (Cloudflare Worker `gristora-agent-bridge`), agent runners, automatic reviews or automatic merge. The owner cancelled them on 22 November 2026. GitHub Issues, branches and pull requests are the only coordination layer.

## Task board (GitHub labels)

The board is the set of open issues and pull requests with these labels. A closed issue is done.

| Label | Meaning |
|---|---|
| `assigned-claude` · `assigned-chatgpt` | The owner of the task |
| `status:ready` | The task can start; its dependencies are done |
| `status:in-progress` | Work started; a branch exists |
| `status:review` | A pull request is open |
| `status:blocked` | The task cannot continue; the issue gives the reason |
| `area:product` · `area:marketing` | The work stream |

Before you start a task:

1. Read the open issues and pull requests.
2. Read the recent commits on `main`.
3. Make sure that no open task of Claude changes the same files.
4. Set `status:in-progress` on the issue.

## Marketing website (independent)

- Work in `marketing-site/` and `web-stratejisi/`. You can also change the marketing section of `OTURUM.md`.
- Take marketing tasks without an assignment. Open an issue for each task with `assigned-chatgpt` and `area:marketing`.
- Merge your own marketing pull request when the marketing CI is green. Do not use auto-merge.
- Keep public claims aligned with document 41 (`web-stratejisi/41-*`). Do not invent availability, features, integrations, customers, metrics, prices or certifications.

## Product (only assigned issues)

- Work only on issues with `assigned-chatgpt` and `area:product`. Do not take other product tasks.
- Change only the files that the issue names.
- Open a pull request to `main` from your branch. Do not merge a product pull request. Claude reviews it, runs the local tests and merges it.
- Do not change these files: `gelistirme/ISLER.md`, `gelistirme/DURUM.md`, the root `CLAUDE.md`, `sunucu/CLAUDE.md`, `web/CLAUDE.md`, `ios/CLAUDE.md`, ADRs in `adr/`. Claude keeps the product queue. If you think that a task or a rule must change, write a comment on the issue.
- Do not add a database migration while an open issue of Claude adds one. Two new migrations at the same time cause a number conflict.
- Read the `CLAUDE.md` file of the area that you change (`sunucu/`, `web/`, `ios/`). Write `TODO` notes, issue text and handoff notes in ASD-STE100 (root `CLAUDE.md` › Yazım standardı).

## Git and quality

- Use one branch for each task: `chatgpt/<task>-<short-name>`. Do not push to `main` directly.
- Start the commit message with the task number (`G2: ...`). Put `Closes #<issue>` in the pull request description.
- Run the checks that the issue names. Report the actual result. Do not write that a test passed if you did not run it, or if CI did not run it.
- When the pull request is open, set `status:review`. When you cannot continue, set `status:blocked` and write the reason.

## Approvals

- `main`: Claude merges product pull requests when the tests are green (owner rule of 11 November 2026). ChatGPT merges its own marketing pull requests when the marketing CI is green.
- Production: do not deploy to production, change production configuration or publish a release without the approval of the owner.

## Report for each completed task

Write it in the pull request description:

1. What changed
2. The files
3. The branch and the commit
4. The pull request
5. The tests that ran
6. The actual results
7. The risks that remain
8. The next recommended task
