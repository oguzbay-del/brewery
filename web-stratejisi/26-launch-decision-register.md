# Demleme — Launch Decision Register

**Status:** Open decisions; no commercial or production assumptions are approved by this document.  
**Scope:** Marketing-site launch readiness only.  
**Audience/market:** Homebrewers and Professional Breweries; US and UK first; English-first; self-service-first.

## Decision register

| ID | Decision | Required owner/evidence | Safe interim behavior | Blocks |
|---|---|---|---|---|
| LCH-01 | Customer-facing app URL, sign-in and registration route | Product owner supplies tested public URLs and confirms onboarding is open | Keep discovery CTAs; no sign-in/signup promise | Conversion CTA |
| LCH-02 | Production domain and canonical origin | Domain owner confirms DNS, TLS, redirects, ownership and preferred host | Keep GitHub Pages/Vercel preview explicitly non-production | Canonical/sitemap/public release |
| LCH-03 | Hosting target and deployment plan | Owner confirms Vercel quota/plan or alternative and rollback | Keep current preview; batch deployments; do not change Vercel settings | Production deployment |
| LCH-04 | Pricing, free tier, trial, packages and limits | Commercial/product owner approves exact terms, billing cadence, limits, tax and cancellation | No pricing page, free-trial or “start free” claim | Commercial conversion |
| LCH-05 | Analytics and conversion events | Privacy owner approves provider, purposes, retention, region, consent and event taxonomy | No analytics/tracking scripts | Measurement |
| LCH-06 | Forms, contact/demo or waitlist | Owner confirms business process, inbox/CRM destination, privacy notice and response SLA | No dead forms or invented demo/waitlist | Lead capture |
| LCH-07 | Photography and product screenshots | User approves assets, rights, crop and product accuracy | Retain original SVG illustrations; no stock or fake UI | Final visual polish |
| LCH-08 | Public product maturity wording | Product owner validates each Available/In development/Planned/Under consideration/Requires validation state against deployed behavior | Use conservative current labels; do not imply roadmap is live | Claim approval |

## Conversion journeys to validate after decisions

### Homebrewers
Audience fit → concrete brewing workflow → technical method/evidence → product maturity and limitations → real app/onboarding destination (only when LCH-01 is approved).

### Professional Breweries
Business fit and operating context → recipe/production/batch record workflow → role and traceability information needs → explicit ERP exclusions → product maturity and limitations → real app/contact destination (only when LCH-01/LCH-06 are approved).

## Measurement plan — only after privacy approval

Candidate events, not implemented:
- `audience_path_selected` (homebrewers / breweries)
- `feature_detail_viewed` (feature slug and approved stage)
- `workflow_section_engaged` (audience and section)
- `technical_resource_opened` (resource ID)
- `app_entry_clicked` (destination class; never collect form field contents)

Do not collect sensitive personal data, recipe contents, batch details or free-text fields for marketing analytics. Define consent, retention and data residency before enabling any provider.

## Acceptance
- Every decision has a named approver and source of truth.
- URLs are manually tested before being used in CTAs.
- Pricing and access claims match actual production behavior.
- Analytics remains disabled until privacy/consent review is complete.
- Production domain, canonical URLs, sitemap and redirects are verified together.
- Final production release receives explicit user approval.
