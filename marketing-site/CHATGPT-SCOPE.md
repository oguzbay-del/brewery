# ChatGPT work boundary — GRISTORA marketing website only

**Owner instruction (15 November 2026): ChatGPT must work only on the public marketing website.**

## Allowed scope

- Work only in `marketing-site/` and the website-specific documentation in `web-stratejisi/`.
- Improve the public marketing website: page structure, copy, responsive design, accessibility, SEO, metadata, visual assets, performance, static build, browser tests and the website's own CI/deployment configuration.
- Implement owner-authorized public pricing, plan comparison and guided membership-intent pages. Keep actual account provisioning, payment processing and subscription services out of scope unless separately authorized.
- Update marketing-site documentation and the website handoff when needed.
- Use the existing product plan and feature catalogue as read-only sources to keep public statements accurate.

## Forbidden scope

- Do not change the main product or interfere with Claude's product-development work.
- Do not edit product application code, Rust services or calculation engine, APIs, database or migrations, data model, web application, iOS application, shared design system, product specifications, product roadmap, `gelistirme/ISLER.md`, `gelistirme/DURUM.md`, product ADRs, or product-development task queue.
- Do not assign, reprioritise, close, or create product-development tasks.
- Do not commit product changes, rebase or merge Claude's product branches, or modify Claude's instructions in the root `CLAUDE.md`.
- Do not perform external trademark, domain, competitor, market, or other brand-clearance research unless the owner explicitly requests it. Such research is not the active workstream.

## Working rules

1. Treat the product repository and Claude's product work as read-only context. Read only what is needed to describe the product accurately on the public site.
2. Keep all implementation changes inside `marketing-site/`. Website-specific strategy and handoff documents may be updated in `web-stratejisi/`, `marketing-site/`, and the marketing section of `OTURUM.md`.
3. Keep public claims aligned with the current product maturity. Do not invent availability, features, integrations, customers, metrics, prices, certifications, or conversion flows.
4. Run the isolated marketing checks, static build, generated-site validation, responsive browser tests and accessibility checks relevant to each change. Review the diff and CI before merging.
5. Do not use auto-merge. Merge website pull requests manually only after the website CI is green. Do not deploy publicly without the required hosting, domain and owner launch approvals.
6. Do not wait for unrelated product work. Do not switch to the product task queue when the marketing website is blocked; record the website blocker and continue with another independent website task.

## Definition of done — stop condition

The assignment ends when the public marketing website is complete and verified. Completion requires:

- All agreed public pages and audience journeys are implemented and internally consistent.
- GRISTORA brand, approved logo assets, visual system, copy and product-maturity disclosures are correctly applied across every public route.
- Responsive layouts, accessibility, navigation, links, metadata, structured data, sitemap, robots rules, social preview, favicon and 404 behaviour pass the agreed checks.
- Static build and generated-site validation pass; responsive browser and accessibility tests pass.
- Website documentation and handoff are current; no known critical or high-severity website defects remain.
- The production domain/hosting and launch-only approvals are recorded as either completed or explicitly owner-dependent. Do not claim a public launch if it has not happened.

**When these conditions are met, stop. The work is finished. Do not continue into the main product, Claude's queue, or unrelated research.**
