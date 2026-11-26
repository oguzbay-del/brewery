# W26 — Public claims audit (EN/TR)

**Date:** 25 November 2026  
**Scope:** Read-only review of the English and Turkish public pages named in issue #320, the feature catalogue and the claims guardrails in document 41.  
**Result:** The English copy is generally aligned with the roadmap-only baseline. Several Turkish audience and FAQ statements still describe roadmap capabilities as if they are available now.

## Baseline

Document 41 says that the product is in development. Roadmap stages do not prove production availability. Public API, webhooks and device integrations are not currently available. Named compatibility, validated calculations, production security, trials and commercial terms need separate evidence and approval.

## Findings

| Priority | Route / source | Finding | Required correction |
|---|---|---|---|
| High | `/tr/faq/`, `marketing-site/src/pages/tr/faq.astro` (around line 34) | The answer says Business mode includes an open API and production-data webhooks, and Home mode supports named devices and a webhook endpoint. This conflicts with the English FAQ, which says these are roadmap items and no public API, webhook or device integration is currently available. | State that these are planned roadmap capabilities, not available integrations. Remove named compatibility claims until each connector is implemented and tested. |
| High | `/tr/breweries/`, `marketing-site/src/pages/tr/breweries.astro` (around lines 31–32, 42) | The page says Business mode includes fermentation/tank workflows, quality/lot relationships and an open API/webhooks. The API statement is explicitly unqualified; the workflow statements also read as current functionality. | Use the English page's intended/planned framing. Keep API and webhook availability explicitly negative until released. |
| High | `/tr/homebrewers/`, `marketing-site/src/pages/tr/homebrewers.astro` (around lines 32, 37) | The page says batch timelines keep connected readings and that GRISTORA supports Tilt, iSpindel, RAPT, TiltBridge, Pitch and compatible webhook bridges. The English page says device connectivity is a roadmap area and named compatibility will be confirmed only after implementation and testing. | Say that read-only device data is a planned direction. Do not name supported devices or bridges as current compatibility. |
| Medium | `/tr/how-it-works/`, `marketing-site/src/pages/tr/how-it-works.astro` (around lines 30, 35–36) | Some workflow descriptions use direct imperatives for connected readings and production records without the repeated planned/intended qualification used in the English page. | Add clear planned-workflow framing to the relevant descriptions and keep target-versus-actual wording. |
| Low | `/tr/technical-principles/` and `/technical-principles/` | The reviewed statements use intended/planned language and call for evidence before technical claims. No material maturity or availability correction identified in the sampled copy. | No change recommended from this audit. |

## Pages and catalogue reviewed

- `marketing-site/src/pages/how-it-works.astro` and `marketing-site/src/pages/tr/how-it-works.astro`
- `marketing-site/src/pages/technical-principles.astro` and `marketing-site/src/pages/tr/technical-principles.astro`
- `marketing-site/src/pages/faq.astro` and `marketing-site/src/pages/tr/faq.astro`
- `marketing-site/src/pages/breweries.astro` and `marketing-site/src/pages/tr/breweries.astro`
- `marketing-site/src/pages/homebrewers.astro` and `marketing-site/src/pages/tr/homebrewers.astro`
- `marketing-site/src/pages/features/index.astro`, `marketing-site/src/pages/features/[slug].astro`, and their Turkish equivalents
- `marketing-site/src/data/features.ts`, `marketing-site/src/data/features.tr.ts`
- `marketing-site/TR-GLOSSARY.md`
- `web-stratejisi/41-gristora-public-marketing-claims-guardrails.md`

The feature directory and detail templates visibly label content as planned product scope. The catalogue itself contains detailed future capabilities; keep that framing around every generated feature route.

## Change boundary and follow-up

Open PR #272 already changes overlapping Turkish audience/FAQ files. This audit therefore makes no edits to those pages. The findings are recorded here so they can be applied after the overlapping PR is reviewed or merged. No other public-page copy was changed.

No trademark/domain research, DNS, production deployment, product code or roadmap changes were made.

## Validation

This is a source-level claims audit only. No source page was changed, so no new build or browser test is claimed for this report. A follow-up correction PR must run Marketing CI, browser QA, Lighthouse and the protected-path guard on its exact head.
