# Demleme — Technical Content & Calculator Validation Backlog

**Work package:** WP-09  
**Status:** Validation plan and backlog; no formula implementation or public calculator authorization.  
**Date:** 14 November 2026  
**Sources:** `plan/demleme-plani.md`, WP-03 feature-claim audit, WP-05 messaging architecture, WP-08 SEO/publishing standard.  
**Scope:** Marketing content and potential public web calculators. Product calculation engine remains separate.

---

## 1. Purpose and critical separation

The product plan specifies a shared Rust calculation engine on the server, with formula test vectors and a golden recipe set as release gates. That does **not** automatically authorize a separate public website calculator.

Two independent decisions are required:

1. **Product engine validation:** formula implementation and server-side behavior pass the product's tests.
2. **Public web calculator approval:** product owner explicitly approves which calculations may be exposed publicly, the inputs/outputs, limitations, references, and maintenance owner.

A marketing calculator must never be presented as connected to a customer account or product engine unless an actual integration exists and is verified. No public calculator should be published before both its formula evidence and the public-scope decision are complete.

## 2. Validation principles

Every calculation must have:
- A named method and authoritative reference/version.
- Explicit input definitions, units, accepted ranges and defaults.
- Expected output and unit.
- Independently sourced reference vectors (not copied from the implementation under test).
- Golden examples representative of real recipes or brewing scenarios.
- Boundary, invalid-input, precision and rounding tests.
- Unit conversion tests, including US gallon vs Imperial gallon distinction.
- Documented assumptions, model limitations and known non-goals.
- Technical reviewer and product-owner approval.
- Versioning so a later method change does not silently alter prior results.

Never invent a formula, constant, conversion factor, default or accuracy claim to fill a documentation gap.

## 3. Candidate validation backlog

Priority is a recommended order for evidence gathering, not a statement of product approval or release order.

| Priority | Calculation area | Audience | Required evidence / tests | Current publication gate |
|---:|---|---|---|---|
| P0 | Gravity and extract / OG | Home + Business | Reference method, extract potential assumptions, volumes, efficiency handling, SG/°P conversion, golden recipes, boundary tests | Requires formula/source validation and public-scope approval |
| P0 | ABV estimate | Home + Business | Explicit OG/FG method, attenuation assumptions, measured vs estimated distinction, reference examples, rounding and edge cases | No accuracy or prediction claim before validation |
| P1 | Recipe scaling and batch volume | Home + Business | Scaling invariants, target volume definitions, losses, rounding, unit conversions, ingredient precision | Validate against product plan and golden recipes |
| P1 | IBU / bitterness | Home + Business | Named calculation method, hop form/use, gravity/volume assumptions, utilization model and reference vectors | Model and supported inputs must be approved |
| P1 | Colour | Home + Business | Named method, malt colour inputs, unit system, volume/gravity assumptions and reference examples | No universal accuracy claim |
| P1 | Water profile / mineral additions | Primarily Home; Business as approved | Water report units, ion definitions, salt composition/purity assumptions, target ranges, additions and rounding | Model, supported salts and limits require review |
| P1 | Mash pH | Home + Business | Model selection, malt acidity assumptions, water alkalinity, temperature, measurement vs estimate, known uncertainty | Explicitly requires validation; no confident prediction claim |
| P2 | Yeast / pitch calculations | Home + Business | Viability assumptions, cell count units, starter assumptions, gravity/volume effects and references | Validate product scope and method |
| P2 | Boil-off / evaporation and volume losses | Home + Business | Equipment-specific vs generic inputs, time/volume behavior, losses and unit conversion | Validate assumptions; avoid one-size-fits-all defaults |
| P2 | Efficiency and yield | Home + Business | Brewhouse/mash/lauter definitions, denominator, losses, measured data and edge cases | Definitions must be consistent across UI and docs |
| P3 | Costing / tax / regulatory calculations | Business; limited Home costing | Currency, ingredient valuation, FIFO scope, jurisdiction, official sources and specialist review | Separate commercial/legal approval; no compliance guarantee |

### Explicitly excluded from public calculator commitments for now
- Fermentation finish-date / final-gravity predictions.
- Expected-curve deviation alerts.
- Automated recommendations based on unapproved analytical models.
- Calibration intelligence tied to the unresolved model-3 status.
- Any regulatory tax/filing result represented as legal advice or guaranteed compliance.

## 4. Formula validation record

Create one record per calculation using this schema:

| Field | Required content |
|---|---|
| Calculation ID | Stable identifier, e.g. `CALC-OG-001` |
| Name / audience | User-facing name and Home / Business / Both |
| Product-plan reference | Exact heading/table row in `plan/demleme-plani.md` |
| Method | Named formula/model and version |
| Reference source | Primary or authoritative source, edition/date/page/URL |
| Inputs | Field name, definition, unit, min/max, required/optional |
| Defaults | Rationale and source; no silent defaults |
| Output | Definition, unit, precision and display format |
| Assumptions | Equipment, ingredient, process and measurement assumptions |
| Golden vectors | Input set, expected output, tolerance and source |
| Boundary tests | Minimum, maximum, zero, negative, missing, extreme and invalid combinations |
| Unit tests | Conversion, rounding, precision and locale cases |
| Known limitations | What the model cannot represent |
| Implementation parity | Product engine version / public calculator implementation relationship |
| Review | Technical reviewer, product owner, review date and approval status |
| Publication | Draft / validation / approved / published / retired |

Do not set tolerance values arbitrarily. The technical reviewer must define tolerance based on the method, source precision and intended use.

## 5. Minimum test matrix

For each formula, test all relevant categories:

| Test class | Example test intent | Expected result |
|---|---|---|
| Reference vector | Published example or independently calculated golden recipe | Match within approved tolerance |
| Typical case | Common recipe and equipment range | Stable, plausible result |
| Lower boundary | Minimum supported volume / gravity / input | Defined result or clear validation error |
| Upper boundary | Maximum supported input | Defined result or clear validation error |
| Zero / empty | Zero volume, no ingredient, blank optional input | Explicitly handled; no NaN/Infinity |
| Invalid | Negative quantity, impossible unit, malformed value | Actionable validation error |
| Unit equivalence | Same physical inputs in metric, US customary and Imperial where supported | Equivalent physical result within tolerance |
| Rounding | Values around precision boundaries | Deterministic, documented display |
| Locale | Decimal separator and number formatting | Correct parse and display without changing value |
| Regression | Previously approved golden vectors | No unexplained drift |
| Cross-platform | Rust engine vs any separately approved public implementation | Identical method and approved tolerance |
| Accessibility | Input labels, unit hints, error announcement, keyboard operation | Fully operable without pointer |

## 6. Public calculator UX and content requirements

Only after explicit public-scope approval:
1. State the task and intended user.
2. Explain required inputs and units before calculation.
3. Distinguish user-supplied measurements from estimates.
4. Show output with unit, precision and meaningful context.
5. Provide a “How this is calculated” explanation.
6. Include assumptions, supported range and limitations near the result.
7. Link to references and a technical guide.
8. Provide clear invalid-input and reset behavior.
9. Make no account sync, save, export or integration promise unless implemented.
10. Add a last-reviewed/method version record.

Do not add calculator pages simply to fill a keyword cluster. A calculator must perform a validated task and provide a better answer than a static article.

## 7. Editorial backlog around calculations

| Order | Content asset | Audience | Purpose | Gate |
|---:|---|---|---|---|
| 1 | Gravity / OG explainer | Both | Explain gravity, extract and measurement context | Source review |
| 2 | ABV: estimate vs measurement | Both | Explain method and uncertainty | Method validation |
| 3 | Recipe scaling and volume | Both | Explain target volume, losses and units | Golden recipe set |
| 4 | IBU method explainer | Both | Explain model differences and assumptions | Method selection |
| 5 | Colour method explainer | Both | Explain inputs and output scale | Reference vectors |
| 6 | Water report and units | Home + Business | Explain water ions and report interpretation | Technical review |
| 7 | Mash pH: measurement and estimates | Home + Business | Explain uncertainty and practical limits | Product model decision |
| 8 | Yeast pitch fundamentals | Both | Explain viability and pitch assumptions | Source review |
| 9 | Efficiency definitions | Both | Separate mash, brewhouse and yield measures | Product terminology decision |
| 10 | Costing concepts | Business / Home | Explain ingredient cost vs accounting | Product-scope review |

## 8. Product-owner decisions required

These decisions must be made by the product owner; the marketing work must not infer them:
1. Which formulas are approved for independent public web use?
2. Which formula method and version is authoritative for each calculation?
3. Which unit systems and input ranges are supported per audience?
4. Are public calculator outputs educational estimates or intended for operational decisions?
5. Should public calculators share the Rust engine through a public API, or be separately implemented? If separate, what parity and release controls apply?
6. Which calculation methods require licensed/attributed references?
7. Who owns formula maintenance and revalidation after a method or engine update?

Until answered, calculator pages remain planned content and no functional tool is published.

## 9. Acceptance gate

A calculation may be marked **Validated for public use** only when:
- [ ] Public use explicitly approved.
- [ ] Method and authoritative reference identified.
- [ ] Inputs, units, ranges and defaults documented.
- [ ] Independent golden vectors approved.
- [ ] Boundary, invalid, unit, rounding and locale tests pass.
- [ ] Limitations and uncertainty written.
- [ ] Product-engine parity decision recorded.
- [ ] Technical and product-owner review signed off.
- [ ] UX and accessibility checks pass.
- [ ] Maintenance owner and review trigger assigned.

## 10. Next work package

**WP-10 — Page content production backlog and evidence register.** Convert the strategy into a prioritized, page-level content plan with source requirements, product dependencies, review ownership and publish/no-publish status. Formula implementation and public calculator launch remain separate approval gates.
