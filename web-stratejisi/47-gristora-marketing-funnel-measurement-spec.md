# GRISTORA marketing funnel measurement specification

**Status:** Proposed — documentation only; not approved for implementation  
**Date:** 15 November 2026  
**Scope:** Public marketing website in `marketing-site/`  
**Decision rule:** No analytics SDK, cookies, pixels, session replay or network beacon is introduced by this specification.

## 1. Purpose

Define a small, privacy-conscious measurement model for understanding whether visitors can find the right audience path, understand the plan differences and complete the non-live plan configurator. This is a future implementation brief, not authorization to collect data.

## 2. Proposed funnel

| Stage | Event | Trigger | Allowed properties |
|---|---|---|---|
| Discovery | `marketing_page_viewed` | A public marketing route is viewed | route key, page family, referrer category (internal/external/direct/unknown) |
| Audience | `audience_path_selected` | Visitor chooses Homebrewer or Professional Brewery | audience enum: `home` / `brewery` |
| Pricing | `pricing_viewed` | Pricing page becomes visible | source route key |
| Comparison | `comparison_opened` | Comparison page is opened | source route key |
| Configurator | `plan_configurator_started` | First wizard interaction | audience enum |
| Configurator | `wizard_step_completed` | A step is completed | step enum, audience enum |
| Configurator | `billing_term_selected` | Billing term is selected | term enum: monthly / annual / two-year / three-year |
| Configurator | `plan_summary_viewed` | Summary is shown | audience enum, seat band: 1 / 5 / 10 / 15 / 20 / custom |
| Custom quote | `custom_quote_guidance_viewed` | 20+ seat guidance is displayed | audience enum, seat band: custom |
| Product education | `feature_detail_opened` | Feature detail route is opened | feature slug from the public route |

Events should be emitted once per meaningful user action, not on every render or component hydration. Route views must avoid duplicate events from client-side navigation and initial page load.

## 3. Funnel definitions

- **Audience-path selection rate:** unique consented sessions selecting an audience ÷ unique consented sessions viewing the homepage.
- **Pricing reach:** unique consented sessions viewing pricing ÷ unique consented sessions viewing any marketing page.
- **Configurator start rate:** unique consented sessions starting the configurator ÷ unique consented sessions viewing pricing.
- **Configurator completion rate:** unique consented sessions viewing a plan summary ÷ unique consented sessions starting the configurator.
- **Custom-quote interest:** sessions viewing 20+ seat guidance; this is an interest signal, not a submitted lead.
- **Content progression:** feature detail opens by feature slug, aggregated without user-level profiling.

Use aggregate counts and denominators. Do not report a “conversion”, “lead”, “trial” or “signup” for a non-live preview action.

## 4. Data minimisation and prohibited collection

Do not collect or transmit:
- Names, email addresses, phone numbers, company names, free-text input or form contents.
- Exact seat counts beyond the defined seat bands.
- Account IDs, persistent cross-site identifiers, advertising IDs, fingerprints or precise location.
- Full IP addresses, URL query strings, campaign parameters that contain personal data, or raw referrer URLs.
- Keystrokes, click heatmaps, session recordings, form-field contents or device sensor data.

Do not create user-level profiles, cross-device identity graphs, retargeting audiences or third-party advertising audiences.

## 5. Consent, jurisdiction and governance gates

Before any implementation:
1. Obtain owner approval for the analytics purpose, provider, data region, controller/processor roles and lawful basis.
2. Obtain privacy/legal review for Türkiye, UK, EEA and any other intended visitor jurisdictions.
3. Determine whether consent is required before storage or transmission; default to no collection until the applicable consent state is known.
4. If consent is required, load the measurement code only after valid opt-in. Refusal must not impair the website or configurator.
5. Publish an accurate privacy/cookie notice and provide a practical withdrawal mechanism.
6. Set a documented retention period and deletion process. Proposed default for review: raw event data no longer than 90 days, aggregate non-identifying totals up to 13 months.
7. Restrict access to named roles; document vendor/subprocessor and international transfer review.

The retention periods above are proposals for legal/owner review, not approved policy.

## 6. Technical safeguards if later approved

- Prefer a first-party, cookieless, aggregate solution only if counsel confirms it is appropriate.
- Use a strict allow-list for event names, enums and route keys; reject arbitrary properties.
- Avoid sending query strings, fragments, raw URLs or arbitrary DOM text.
- No third-party scripts on the public site without security, performance and privacy review.
- Provide a global kill switch and verify that no event is sent before consent where consent is required.
- Keep analytics out of the product application and do not couple this work to product identity or billing.

## 7. Validation and acceptance

A future implementation may proceed only after written approval of provider, lawful basis, consent behavior, notice, retention and event schema. Tests must verify:
- No network request or storage before required consent.
- Rejecting consent leaves all public pages and the plan configurator usable.
- Only allow-listed event names and properties are sent.
- No personal data or raw URL/query values are transmitted.
- No event implies a live signup, payment, trial activation or submitted sales lead.
- Keyboard and screen-reader operation is unchanged.

## 8. Explicitly out of scope

No analytics package, tag manager, cookie banner, consent platform, pixel, SDK, backend collector or tracking code is added in this work package. No third-party provider is selected. This document alone does not authorize collection.
