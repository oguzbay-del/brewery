# GRISTORA Claude + GPT development protocol

## Roles

- Claude Code is the primary implementation agent for product and repository tasks.
- GPT is the independent architecture, security, code-review, test-design, and documentation agent.
- Either agent can create a task for the other through the Cloudflare MCP bridge.
- The GitHub repository is the only source of truth for code, branches, pull requests, and review history.

## Parallel work

1. Create a task with a precise scope, acceptance criteria, affected paths, and test commands.
2. Assign each task to one agent. Do not assign overlapping files to parallel agents.
3. Claim the task before work starts.
4. Use a separate branch for every task. Do not commit directly to `main`.
5. Open a pull request and include test evidence.
6. Ask the other agent for an independent review through `send_message`.
7. Resolve all blocking review comments and rerun the affected tests.
8. Update the task with the PR URL and final result.

## Merge and production policy

The requested operating mode is automatic merge and production deployment after all gates pass.

- Never merge a PR with a failing or pending required check.
- Never merge a PR with unresolved blocking review findings.
- Never merge a PR with merge conflicts or a stale head SHA.
- Never deploy if the build, browser QA, security checks, or Lighthouse policy fails.
- Use expected-head-SHA checks to avoid merging a moving PR.
- Keep the existing Cloudflare Pages production configuration and domain unchanged.
- Keep GitHub Pages workflows only where they are still intentionally used; do not add Vercel.
- If a workflow or secret is unavailable, report the blocker. Do not claim an automated action occurred.

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
