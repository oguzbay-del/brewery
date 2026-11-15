# Demleme — Feature Catalogue Product-Claim Audit

**Work package:** WP-03  
**Status:** Source mapping complete; marketing copy uses conservative “Under consideration” labels for unapproved analytical models. Product-plan contradictions still require product-owner resolution before those models can be committed.  
**Date:** 14 November 2026  
**Sources of truth:** `plan/demleme-plani.md`, `marketing-site/src/data/features.ts`, `marketing-site/src/pages/features/[slug].astro`.  
**Scope:** Marketing claims only. No product implementation or product roadmap changes.

## 1. Executive result

The 16 feature areas in the marketing catalogue broadly map to named product-plan domains and use explicit roadmap-stage labels. The catalogue is structurally useful and should be retained as a starting point.

However, the product plan contains a critical distinction that must remain visible: **designed, planned, in development, available, and under consideration are not interchangeable states**. The current public site must not imply that a designed screen, a written product specification, or code in the calculation engine is already a usable customer-facing feature.

Three areas need particular care:

1. **Analytical predictions:** the product plan explicitly marks its 18 analytical models as ideas that are not approved or committed. The fermentation page currently mentions estimated FG and finish-date predictions with confidence ranges. These must not be marketed as committed capabilities unless separately approved.
2. **Calibration and deviation alerts:** these appear in the additional-features roadmap but are also linked to analytical-model ideas whose approval is pending. The product plan needs one unambiguous status before stronger claims are used.
3. **Regulatory/compliance and security:** compliance rates, filings, label obligations and Turkish regulatory details require official-source/specialist validation. Security controls in a design plan are not evidence of deployed controls, certifications or audited compliance.

No feature should be described as currently available merely because it has a detailed design, code in an internal engine, or a target release label.

## 2. Feature-to-product-plan mapping

| # | Marketing feature page | Product-plan source area | Audience | Marketing stage labels | Claim / copy control |
|---:|---|---|---|---|---|
| 1 | Recipe design & management | Recipe wizard; recipe editor; style guides; recipe versions | Home + Business | Homebrew MVP; Business core | Import formats, substitutions, style mapping and approval workflow must retain their specified limits and roadmap status |
| 2 | Brewing calculations | Calculation engine; formula decisions; test vectors | Home + Business | Homebrew MVP; Business core; Business second release | Shared server-side engine is planned; formulas/defaults require reference and golden-recipe validation before release |
| 3 | Water, mash & yeast | Water profiles; mash schedule/pH; yeast preparation | Home + Business | Homebrew MVP | Do not imply validated mash-pH prediction or universal accuracy; explain model, inputs and assumptions |
| 4 | Brew day & production records | Brew-day workflow; paper/PDF; scanning; plan vs actual | Home + Business | Homebrew MVP; Business core | Distinguish planned iOS/system integrations and scanning from currently available workflows |
| 5 | Fermentation & batch tracking | Fermentation; concurrent batches; tank workflow; measurements | Home + Business | Homebrew MVP; Business core | Manual/device readings and batch records are roadmap items; prediction/forecast claims require separate approval |
| 6 | Equipment & calibration | Equipment profiles; vessel readiness; calibration | Home + Business | Homebrew MVP; Business core | Calibration status conflicts with unapproved analytical-model list; resolve before calling it committed |
| 7 | Planning & scheduling | Backward planning; calendar; tank scheduling; operator Today view | Home + Business | Homebrew MVP; Business core; Business second release | Tank capacity optimisation and advanced suggestions must remain in the stated later stage |
| 8 | Inventory, ingredients & costing | Ingredient library; stock; lot tracking; recipe/batch costing | Home + Business | Homebrew MVP; Business core; Business second release | Separate Home stock list from Business lot/supplier/FIFO workflows; costing is not accounting/ERP |
| 9 | Brewery production management | Production board; tank movements; batch traceability; operator records | Business | Business core; Business second release | Explicitly exclude sales, orders, distribution, keg tracking, CRM and accounting |
| 10 | Quality, laboratory & release | QC, specifications, lab, sensory, release controls | Business | Business core; Business second release | Do not imply validated laboratory hardware integrations or regulatory certification without evidence |
| 11 | Packaging, CIP & compliance | Packaging formats; CIP; tax/compliance records | Business | Business core; Business second release | Regulatory details need official and specialist review; no automatic filing or legal-compliance claim |
| 12 | Teams, roles & notifications | Users, roles, permissions, notifications and alerts | Home + Business | Homebrew MVP; Business core | Multi-user roles and approval controls belong to Business; alert intelligence may depend on unapproved models |
| 13 | Connected devices & integrations | Tilt/iSpindel/RAPT; professional sensors; gateways | Home + Business | Homebrew MVP; Business second release; Later | Device support must name exact integration mode/stage; control commands are later and approval-gated |
| 14 | Import, export & brewing tools | Recipe import matching; PDF/CSV; specialist tools | Home + Business | Homebrew MVP; Homebrew second release | Distinguish supported import formats from later export/tools; preserve mapping and user-confirmation steps |
| 15 | Platform, security & data ownership | Platform architecture; authentication; security; portability | Home + Business | Homebrew MVP; Business core | Describe design intent, not deployed security posture, certification, SLA or audit result |
| 16 | Reports, insights & sustainability | Reports; analytics models; sustainability ideas | Home + Business | Homebrew MVP; Business core; Business second release; Under consideration | Keep every unapproved model and speculative sustainability metric explicitly under consideration |

## 3. Required roadmap vocabulary

Use one consistent visible status system across the feature directory and all detail pages:

| Status | Meaning | Allowed marketing language |
|---|---|---|
| Available | Implemented, deployed and verified for the stated audience | “Available” / present tense only after release verification |
| In development | Implementation actively underway and scope accepted | “In development”; no promise of availability date |
| Planned | Explicitly included in an approved product stage, not yet available | “Planned” / “designed to” / “the roadmap includes” |
| Under consideration | Idea or research item without approval | “Under consideration”; never included in committed feature lists |
| Requires validation | Formula, regulatory, integration or operational behaviour needs independent evidence | State the validation dependency and avoid accuracy/compliance claims |

A release-stage label such as “Homebrew MVP” is not itself a claim of current availability.

## 4. High-risk wording and remediation

| Area | Current risk | Required treatment |
|---|---|---|
| Fermentation forecast | Marketing copy now labels FG and finish-date estimates as under consideration; analytical model 1 remains unapproved | Keep the label until the product owner resolves the product-plan contradiction |
| Expected-curve deviation | Marketing copy now labels curve-deviation and stuck-fermentation warnings as under consideration; analytical model 2 remains unapproved | Keep the label until the product owner resolves the product-plan contradiction |
| Equipment calibration | Additional-features table includes calibration in Home MVP; analytical model 3 says not approved | Resolve contradiction in product plan; marketing must not decide the product status |
| Mash pH | Calculator/model is listed but model choice and validation remain open | Say “estimate” only with model/source/limitations; no accuracy claim |
| Calculations | Product plan specifies test vectors and golden recipes as release gates | Each public calculator requires independent reference examples, boundary tests, units, rounding and formula explanation |
| Device integrations | Device/gateway support is staged and varies by audience | Name supported device, data direction and stage; never imply equipment control if read-only |
| Regulatory content | Tax rates, filing and label rules are market-specific and need official/specialist validation | Publish educational context only; no compliance guarantee or automated filing promise |
| Security | Detailed architecture exists in product plan | No “certified”, “compliant”, “encrypted end-to-end”, “SLA” or deployed-control claim without implementation evidence |
| Costing | Recipe/batch cost is planned; accounting and ERP are excluded | Use “ingredient/production cost estimate” only; do not imply accounting, purchasing or financial ledger |
| Yeast viability and hop-alpha estimates | Analytical model 6 is not approved; copy now separates deterministic inputs/calculations from age-based estimates | Keep age-based estimates under consideration |
| Device fault diagnosis | Analytical model 9 is not approved; copy now limits committed wording to connection and reading status | Keep automated fault diagnosis under consideration |
| SPC and cross-batch trends | Analytical model 10 is not approved; copy now separates SPC from release and quality records | Keep SPC and trend analysis under consideration |
| Inventory and operational forecasts | Analytical models 12–14 are not approved; copy now labels demand, expiry and tank-bottleneck analysis as research ideas | Do not present forecasts or optimisation as committed capabilities |
| Detailed cost variance | Analytical model 15 is not approved; copy now retains basic costing but labels price/quantity/loss attribution as under consideration | Keep detailed variance attribution under consideration |
| AI / analytics / sustainability | The plan's 18 analytical models are unapproved. The site now separates yeast viability, device fault detection, SPC, sensory correlations, demand forecasts, expiry risk, tank bottleneck optimisation and detailed cost-variance attribution from committed capabilities | Keep these ideas under consideration until individually approved with scope and release stage |

## 5. Audience-specific claim boundaries

### Homebrewers

Emphasise:
- Recipe formulation, scaling and style comparison.
- Water, mash, yeast and equipment context.
- Brew-day plan versus actual records.
- Manual readings and supported home fermentation devices.
- Learning from completed batches.

Avoid:
- Professional multi-user approval/role features as Home defaults.
- Industrial tank automation or professional compliance language.
- Forecasts or personalised recommendations that depend on unapproved analytical models.

### Professional Breweries

Emphasise:
- Recipe review and approval.
- Production board, tank allocation and movement records.
- Operator records, deviations, QC and release workflows.
- Lot-to-batch traceability, stock and costing within the stated product scope.
- Roles, permissions, audit history and team notifications.

Avoid:
- ERP, sales, orders, shipping/distribution, keg tracking, CRM or accounting claims.
- Regulatory compliance guarantees.
- Automated equipment control or unvalidated device interoperability.

## 6. Page-level acceptance checks

Before publishing any feature page, verify:

- [ ] Feature exists in the product plan and has a named audience.
- [ ] Every capability maps to a product-plan section or approved decision.
- [ ] Release stage matches the product plan.
- [ ] Unapproved analytical ideas are marked “Under consideration” or omitted.
- [ ] “Available” is used only after a deployed, tested release.
- [ ] Formula claims link to method, inputs, units, assumptions, examples and limitations.
- [ ] Device claims specify model/protocol, direction (read/control) and stage.
- [ ] Compliance/security claims have evidence and specialist review.
- [ ] Home and Business wording reflects the correct workflow.
- [ ] ERP exclusions are not contradicted elsewhere on the site.
- [ ] CTA leads to a real action and does not imply a non-existent product flow.

## 7. Decisions required from product owner

These are product-plan consistency questions, not decisions the marketing site should invent:

1. Is batch calibration committed for Home MVP, or still an unapproved analytical model?
2. Are fermentation finish-date/FG predictions committed, or only a research idea?
3. Is expected-curve deviation alerting committed, or still pending approval?
4. Which exact calculators/formulas are approved for public independent web use, separate from the private server-side engine?
5. Which device integrations are included in the first release, by exact model/protocol and read-only/control capability?
6. Which regulatory jurisdictions and records are in scope for first Business release?

Until resolved, copy must use the more conservative status.

## 8. Result and next step

WP-03 source mapping is complete. The marketing catalogue now labels analytical models 1–3, 6, 9–15 as under consideration where they appear in feature copy. It does not present them as approved capabilities. The product plan still contains conflicts between the “additional features” table and the separate unapproved-model list; only the product owner can resolve those decisions. Continue to validate calculator, device, regulatory and security claims before content expansion.

**Next package:** WP-04 — separate Home and Business sitemap, user journeys and CTA map. WP-04 can proceed using conservative status labels; unresolved product decisions above remain explicit constraints and must not be silently assumed.
