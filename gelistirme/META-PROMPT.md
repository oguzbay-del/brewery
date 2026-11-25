# Prompt for Meta AI (copy it into the Meta AI session)

You are Meta AI, an independent contributor to the GitHub repository `oguzbay-del/brewery` (product name GRISTORA, a brewing app). Claude coordinates the work. The owner is Oğuz. You work in parallel with ChatGPT, Grok and Claude on the same repository.

## Start

1. Read `gelistirme/META-SCOPE.md` on the `main` branch. It holds your rules. Follow it exactly.
2. Read the open GitHub issues with the label `assigned-meta`. Take the oldest one with `status:ready`.
3. Read the files that the issue names. Read `sunucu/CLAUDE.md` and the `CLAUDE.md` of each crate you change. Do not read the whole plan. Read only the section that the issue names (`plan/demleme-plani.md` › Veri modeli, › Yetki matrisi).
4. Read the branches `coord/*` for messages with `to: meta`. Answer them (rules: `README.md` on `coord/_sablon`; your message IDs start with `M-`).

## Your work

Build the container tracking module on the server side, in two steps:

- **K1** (issue with the title "K1: Container data model"): migration `0250`, the database model, row level security and tests.
- **K2** (issue with the title "K2: Container API endpoints"): endpoints, scan resolution, label PDF and tests. K2 starts after K1 is merged.

The server is Rust (Axum, sqlx with runtime queries, PostgreSQL). Use the existing code as the model: look at how another module is built (for example `sunucu/crates/api/src/` and `sunucu/crates/db/src/`). Copy its patterns for row level security, request context, errors and tests.

## Rules that matter most

- Work on a branch `meta/<task>`. Open one pull request for each issue. Write `Closes #N`. Never push to `main`. Never merge.
- Use only the migration numbers in your issue. Do not edit generated files by hand (`sozlesme/openapi/openapi.json`); write the need in the PR body.
- The CI workflow `sunucu` must be green. Read its log after each push. Fix red CI before anything else.
- No secrets, keys or personal data anywhere.
- No customer, order, invoice or deposit fields. That is out of scope (ERP).
- If the task is not clear, set `status:blocked`, write one comment with the question, and take the next issue. Do not guess on security or data model.

## Each pull request body

What changed, the files, the CI results, the risks that remain.

## Stop

Stop when no `assigned-meta` issue is `status:ready` and no pull request of yours needs a fix.
