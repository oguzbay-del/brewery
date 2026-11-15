# WP-10 Evidence Addendum — Fermentation Sample Handling

**Status:** Research note only; not a product specification, operating procedure or approved calculation method.  
**Reviewed:** 15 November 2026

## Why this topic matters

A density or extract reading taken during fermentation can be affected by the state of the sample itself. This is a measurement-quality issue, not merely a formula issue. A future guide should distinguish measurement procedure from mathematical interpretation.

## Evidence

### Laboratory protocol: beer and wort analysis

The peer-reviewed open-access protocol *A Hands-On Guide to Brewing and Analyzing Beer in the Laboratory* (Journal of Visualized Experiments; PMC9286407) describes hydrometer-based extract measurement and sample handling. Its protocol calls for a homogeneous sample, temperature conditioning, avoiding foam, preventing bubbles from adhering to the hydrometer, and reading the meniscus according to the instrument's calibration convention. It also distinguishes original extract/gravity before fermentation from residual extract/final gravity after fermentation.

Source: https://pmc.ncbi.nlm.nih.gov/articles/PMC9286407/

### Fermentation-specific interference

The paper *Prediction of Ethanol Content and Total Extract Using Densimetry and Refractometry* (Beverages, 2023) discusses CO₂ saturation, bubbles, foam and suspended yeast/material as potential sources of interference in density measurements during fermentation. It also describes refractometry as an alternative for small samples while noting analytical limitations.

Source: https://www.mdpi.com/2306-5710/9/2/31

### Metrology context

NIST's hydrometer calibration service describes calibration at specified scale markings in a liquid of known density and reports corrections and uncertainty. NIST handbook material also makes clear that reference temperature and the specific hydrometer scale matter. These laboratory calibration concepts should not be confused with a generic homebrew temperature correction.

Sources:
- https://www.nist.gov/publications/nist-calibration-services-hydrometers
- https://nvlpubs.nist.gov/nistpubs/hb/2023/NIST.HB.143-2023.pdf

## Editorial requirements for any future guide

- Explain why a sample with foam, CO₂ bubbles or suspended solids may not produce a representative reading.
- Distinguish a clean, homogeneous sample from an in-vessel observation.
- Explain that instrument-specific meniscus conventions must be checked in the instrument manual.
- Distinguish pre-fermentation original gravity/extract from post-fermentation final gravity/residual extract.
- Do not imply that every hydrometer or density meter uses the same calibration temperature, scale or reading convention.
- Do not convert these laboratory steps into a mandatory product workflow without product-owner and brewing-SME review.

## Product implications (questions, not requirements)

A future measurement record may need to preserve:
- measurement method and instrument identity;
- sample context (wort / active fermentation / finished beer);
- measured value and original unit/scale;
- sample temperature and instrument reference temperature, where applicable;
- whether sample preparation or degassing was performed;
- timestamp, operator/source and correction-method version, if any.

These are candidate provenance fields only. The existing product schema must be reviewed before any implementation or public capability claim.

## Decision

This evidence strengthens the WP-10 editorial outline. It does not validate a formula, authorize a calculator, or establish that Demleme currently captures these fields.
