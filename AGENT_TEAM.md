# GRISTORA Claude + GPT development protocol

## Roles

- Claude Code is the primary implementation agent for product and repository tasks.
- GPT is the independent architecture, security, code-review, test-design, and documentation agent.
- Either agent can create a task for the other through the Cloudflare MCP bridge.
- The GitHub repository is the only source of truth for code, branches, pull requests, and review history.

## Parallel work

1. Create a task with a precise scope, acceptance criteria, affected paths, and test commands (`create_task`; the assignee defaults to the other agent).
2. Assign each task to one agent. Do not assign overlapping files to parallel agents.
3. Claim the task before work starts (`claim_task`). A runner resumes its own claimed task before it claims a new one.
4. Use a separate branch for every task: `claude/<topic>` for Claude, `gpt/<topic>` for GPT. Do not commit directly to `main`.
5. Open a pull request and include test evidence.
6. Ask the other agent for an independent review through `send_message`.
7. Resolve all blocking review comments and rerun the affected tests.
8. Update the task with the PR URL and final result (`update_task`).

## Merge and production policy

Automatic merge and production deployment run only through the **Agent auto-merge** workflow (`.github/workflows/agent-auto-merge.yml`, gate logic in `.github/scripts/agent-auto-merge.mjs`). It runs from the default branch only and never checks out pull request code, so a PR cannot change the gate that judges it.

A PR merges automatically only when **all** of these are true:

- The head branch is `claude/*` or `gpt/*` in this repository, and the PR targets `main` and is not a draft.
- Label `auto-merge` is set and `do-not-merge` is not set.
- The **Agent review** workflow posted an approving Claude record and an approving GPT record for the current head SHA. Records count only when `github-actions[bot]` wrote them, so a person cannot add one by hand. A record with a `blocking` or `major` finding, a truncated diff or an invalid model answer is never an approval. The `review:*-ok` labels are only a visible hint.
- Every check run and commit status on the head SHA is complete and successful (or neutral/skipped). A pending or failed check stops the merge. This includes Cloudflare Pages and Workers Builds.
- The workflow for each changed area has a successful run on the head SHA: `marketing-site/**` needs **Marketing site checks** (build, browser QA, Lighthouse), `agent-bridge/**` needs **Agent bridge checks**, workflow changes need **Actionlint**, product paths need their own workflows (`sunucu`, `web`, `ios`, `sozlesme`, `altyapi`, `belgeler`).
- GitHub reports the PR as mergeable with state `clean` (no conflict, not behind when branch protection requires it).
- A `gpt/*` branch changes only `marketing-site/` and `web-stratejisi/` (see `marketing-site/CHATGPT-SCOPE.md`).
- The PR changes nothing under `.github/`. Workflow and gate changes always need a manual merge by the owner.

Known limit: any workflow that runs with a write `GITHUB_TOKEN` can also post as `github-actions[bot]`. A branch that adds such a workflow could forge a review record for another PR. Keep **Settings › Actions › General › Workflow permissions** at *Read repository contents*, and review every new workflow file by hand.

The merge is a squash merge with the expected head SHA, so GitHub refuses it if the head moved after evaluation.

After the merge:

- Cloudflare Pages deploys the marketing site from `main` (existing Git integration, unchanged).
- The gate dispatches **Deploy GRISTORA Agent Bridge** and **Deploy marketing preview to GitHub Pages** when their paths changed, because a merge made with `GITHUB_TOKEN` starts no `push` workflow.
- The bridge deploy checks `/health` and the 401 on `/mcp`, and rolls back with `wrangler rollback` when the check fails.

Other rules:

- Never merge manually with a failing or pending required check, unresolved blocking findings, merge conflicts or a stale head SHA.
- Keep the existing Cloudflare Pages production configuration and domain unchanged. Do not add Vercel.
- If a workflow or secret is unavailable, report the blocker. Do not claim an automated action occurred.
- Run the gate without merging: **Actions › Agent auto-merge › Run workflow** with `dry_run` on.

## Product boundaries

Read the root `CLAUDE.md` and relevant scoped instructions before editing. The marketing website is separate from the product application. Do not infer permission to change product scope, APIs, the Rust engine, iOS app, data model, or product roadmap from a website task.

## Required task report

Every completed task must report:
- What changed.
- Branch and PR URL.
- Tests run and their results.
- Independent review result.
- Whether merge and production deployment completed.
- Any required follow-up.
