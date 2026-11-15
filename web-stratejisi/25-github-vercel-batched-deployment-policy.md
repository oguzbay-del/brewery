# GitHub and Vercel Deployment Workflow

**Status:** Working agreement  
**Updated:** 15 November 2026

## Objective

Conserve the Vercel Hobby deployment/build quota while keeping GitHub as the source of truth. Do not deploy every small change or every feature branch.

## Mandatory workflow

1. **Work in GitHub first.** Make changes on feature branches. Never use a Vercel deployment as the normal development/test loop.
2. **Parallelize independent work.** Split work into small, non-overlapping tasks/branches where practical. Coordinate shared files to avoid conflicts.
3. **Accumulate related changes.** Group coherent improvements into a reviewable batch. Keep documentation and `OTURUM.md` in sync with the batch.
4. **Validate before deployment.** Run the available GitHub Actions checks: install, type/build, static validation, Playwright responsive tests, axe accessibility tests and relevant workflow checks. Review the full diff and generated artifacts.
5. **No automatic merge.** Do not enable GitHub Auto-merge. Merge manually only after required checks pass and the diff has been reviewed.
6. **Defer Vercel.** Do not trigger Vercel Preview/Production deployments for each PR or small commit. Keep Vercel deployment disabled/avoided during the accumulation phase where repository settings allow it without disrupting the production site.
7. **One grouped Vercel deployment.** Trigger or allow Vercel only when the agreed batch is complete, checks are green, the batch is merged to the intended deployment branch, and a deployment is actually useful. Prefer one deployment for the complete batch, not one per change.
8. **Protect production.** Never change production deployment settings, disable an existing production site, or alter domain/routing behavior without explicit user approval. If Vercel cannot be safely paused through repository configuration, leave it alone and simply avoid pushing deployment-triggering changes until the batch is ready.
9. **Report truthfully.** Distinguish GitHub Actions, GitHub Pages, Vercel Preview and Vercel Production. Never describe a deployment as successful unless its status is verified. Record quota/rate-limit blocks accurately.
10. **Keep handoff current.** Update `OTURUM.md` after each meaningful batch with branch/PR, changed scope, checks, merge/deployment status and remaining release gates.

## Parallel execution rules

- Parallelize research, copy audits, independent page reviews, test design and non-overlapping asset work.
- Do not concurrently edit the same file from multiple branches unless one owner coordinates integration.
- Keep feature branches short-lived and rebase/refresh against current `main` before opening or updating a PR.
- Batch documentation-only updates with the implementation they describe when practical.
- Do not parallelize risky production configuration changes.

## Release gates remain unchanged

- Do not invent product capabilities, customer proof, pricing, integrations, security certifications or launch dates.
- Do not publish calculators until WP-09 formula/test-vector validation and public-use approval are complete.
- Do not publish technical guides until WP-10 evidence, primary sources and subject-matter review are complete.
- Keep manual screen-reader, true 200% zoom, full keyboard/focus and SVG text-contrast checks explicitly open until actually performed.

## Definition of a deployment-ready batch

A batch is ready only when:
- Its scope and acceptance criteria are documented.
- GitHub Actions checks pass.
- The full diff and screenshots/artifacts are reviewed.
- No unsupported claims or unresolved high-severity accessibility defects remain.
- The PR is manually merged to the intended branch.
- Vercel is triggered once for the consolidated result, if the user wants a Vercel deployment and the quota permits it.
