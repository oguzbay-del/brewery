# Demleme — WP-09 / WP-10 Primary Source Register

**Status:** Research candidates only — not formula approval, product validation or publication authorization.  
**Reviewed:** 15 November 2026  
**Scope:** Brewing calculations, measurement terminology and unit conversion for US/UK-first English content.

## 1. Research rule

This register identifies sources worth using during technical review. A source being listed does not validate a formula, authorize a public calculator, or prove Demleme's implementation is correct. Each calculation still needs a named method/version, independently prepared golden vectors, boundary and unit tests, technical review, product-owner approval and an explicit public-use decision under WP-09.

## 2. Candidate source register

| ID | Source | Best use | Important limitation | WP mapping |
|---|---|---|---|---|
| SRC-001 | ASBC, *Methods of Analysis* — https://www.asbcnet.org/methods | Primary brewing-analytical method catalogue; identify current Beer method IDs and method revisions for specific laboratory measurements | Full method access may require membership/subscription. A method title or catalogue entry is not enough to reproduce a calculation or claim equivalence to a homebrew estimate | OG/SG, extract, alcohol, bitterness, colour, pH |
| SRC-002 | ASBC, *About ASBC Methods of Analysis* — https://www.asbcnet.org/methods/about | Establishes that methods are developed/reviewed by technical committees and that the online 14th edition is the supported, continually updated edition | Does not itself provide the technical procedure or numeric formula for an individual method | All technical articles; method/version governance |
| SRC-003 | ASBC Laboratory Proficiency Program — https://www.asbcnet.org/lab/lpp | Maps analytical topics to recommended ASBC methods; useful for finding relevant method IDs (e.g. specific gravity, alcohol, bitterness, colour, pH) | Proficiency/lab method mapping is not a consumer calculator specification; confirm current method details and applicability | OG/SG, ABV, IBU, colour, pH |
| SRC-004 | NIST, *Guide to the SI*, Appendix B.9 — https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9 | Exact unit conversion factors, including US and Imperial volume units | Conversion factors do not define brewing process assumptions, significant digits, instrument calibration or formula accuracy | All unit handling; US/Imperial/metric |
| SRC-005 | NIST Handbook 44, Appendix C (2026) — https://nvlpubs.nist.gov/nistpubs/hb/2026/NIST.HB.44-2026.pdf | Cross-checks customary and metric unit relationships, including distinct US liquid and British gallon values | Legal metrology handbook, not a brewing-method reference; use only for the relevant measurement-unit definitions | Volume/unit glossary and QA vectors |
| SRC-006 | NIST, U.S. Metrication FAQs — https://www.nist.gov/pml/owm/faqs/us-metrication-faqs | Explains legal measurement-unit context and the distinction between US customary and Imperial units | General metrology guidance; not brewing-specific | Unit terminology and editorial guidance |
| SRC-007 | UK HMRC, *Alcoholic products technical guide*, Section 11 — https://www.gov.uk/guidance/alcoholic-products-technical-guide/section-11-alcoholic-strength | UK jurisdictional context for alcoholic-strength measurement and excise calculations; useful to flag that regulatory ABV calculations have defined measurement conditions | Tax/excise guidance is not a universal homebrew ABV formula. Do not reuse its factors as a general consumer estimate or imply legal/tax advice | UK regulatory caveat only; not general ABV calculator authority |

## 3. Topic-to-source routing

| Topic | First review path | Required additional evidence before publication |
|---|---|---|
| Specific gravity / extract / OG | SRC-001 + SRC-003; unit conventions from SRC-004/005 | Exact current method, reference temperature, instrument/sample conditions, independent OG/°P vectors and stated range |
| ABV | SRC-001 + SRC-003 for analytical methods; SRC-007 only for UK excise context | Explicitly selected estimate or analytical method, OG/FG measurement conditions, carbonation/priming treatment where relevant, uncertainty and independent vectors |
| IBU / bitterness | SRC-001 + SRC-003 | Selected analytical vs recipe-estimation method, hop/process inputs, matrix applicability and independently sourced vectors |
| Colour | SRC-001 + SRC-003 | Current method, sample/measurement conditions, scale and independent reference values |
| pH | SRC-001 + SRC-003 | Calibration and sample-temperature assumptions, measurement vs prediction distinction, uncertainty and supported range |
| Volume / mass / temperature conversion | SRC-004 + SRC-005; terminology SRC-006 | Automated conversion tests, exact vs rounded factor policy, locale and display precision |
| Water chemistry / salts / mash pH | No sufficient source approved by this register | Product model decision, current specialist primary references, salt composition/purity assumptions, water report definitions and SME review |
| Recipe scaling / efficiency / evaporation | No single source approved by this register | Product-specific definitions, equipment/loss assumptions, golden recipe set and product-owner terminology approval |

## 4. Source quality and use controls

- Prefer current primary methods and standards over calculators, blogs, forum posts or copied formula tables.
- Treat secondary teaching material as explanatory background only; it cannot be the sole authority for a production or public calculator formula.
- Record the exact edition, method identifier, revision/date, page/section and access rights when a method is selected.
- Confirm whether the source permits reproduction, attribution or implementation; do not copy licensed tables or methods into public content without permission.
- Distinguish analytical laboratory methods from recipe estimates. They may answer related questions but are not interchangeable.
- Keep US customary gallons and Imperial gallons distinct. Never label one simply “gallon” in a context where the distinction changes the result.
- Do not claim regulatory compliance, laboratory equivalence, accuracy or precision without appropriate evidence and reviewer approval.

## 5. Immediate WP-09 / WP-10 actions

1. Product/formula owner selects the authoritative method and version for each calculation candidate.
2. Technical reviewer records the method's exact scope, inputs, reference conditions, range, precision and limitations.
3. A separate reviewer creates golden vectors from an independent source or independently checked derivation; do not copy expected outputs from the implementation.
4. Product owner decides whether any calculation may be exposed independently on the marketing site and how parity with the Rust engine is maintained.
5. Content owner drafts only explanatory material that can be supported now; formula-dependent guides remain in Technical review or Hold.
6. Recheck source editions and links before publication and whenever a method changes.

## 6. Current decision

**No calculator or technical guide is promoted to Approved by this register.** SRC-001–SRC-007 are source-discovery references. WP-09 acceptance criteria and WP-10 page-level review remain mandatory.
