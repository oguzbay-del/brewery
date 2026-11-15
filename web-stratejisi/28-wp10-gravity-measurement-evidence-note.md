# WP-10 Evidence Note — Gravity Measurements and Instrument Context

**Status:** Research outline only; not an approved product method, formula, calculator or published instruction.  
**Reviewed:** 15 November 2026

## Purpose

Scope a future educational guide explaining what a gravity reading represents, why instrument and sample context matter, and why measurement provenance should be retained. This note does not prescribe a correction formula or claim that Demleme currently performs corrections.

## Evidence reviewed

| Source | What it supports | Limitation |
|---|---|---|
| Wright & Bean, NIST SP 250-78r1, *Calibration Services for Hydrometers* — https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.250-78r1.pdf | Hydrometer calibration is a metrology problem; reference temperature, liquid/glass thermal expansion and surface-tension corrections matter. | Not a universal homebrew wort correction table or algorithm for every instrument. |
| NIST publication record — https://www.nist.gov/publications/nist-calibration-services-hydrometers | Describes calibration by immersion in a liquid of known density and reports calibration uncertainty. | Not a brewing-specific SG/Plato conversion method. |
| Essery (1954), *Journal of the Institute of Brewing* 60(1), 21–28 — https://doi.org/10.1002/j.2050-0416.1954.tb02743.x | Wort and beer are mixed-solute systems; simple sucrose relationships are approximate and solution factors can vary. | Does not establish a modern universal wort correction factor or post-fermentation model. |
| Košin, Šavel & Brož (2016), *Kvasný Průmysl* 62(3), 90–93 — https://doi.org/10.18832/kp2016014 | Refractometry during fermentation is an analytical topic; sample preparation and instrument design matter. | Not applicable to every handheld refractometer or a generic consumer calculation. |
| American Homebrewers Association, *How to Take an Accurate Hydrometer Reading* — https://homebrewersassociation.org/how-to-brew/how-to-take-an-accurate-hydrometer-reading/ | Consumer guidance stresses the instrument's stated calibration temperature and verification. | Not a primary metrology standard. |

## Technical distinctions the guide must preserve

1. **Instrument reading vs normalized value:** hydrometer scale and reference temperature must be known; sample temperature and instrument specification matter.
2. **Density vs specific gravity:** SG is a ratio against a reference-water convention; avoid ambiguous reference conditions in precision contexts.
3. **Hydrometer vs refractometer:** they measure different physical properties. A Brix refractometer scale is conventionally based on sucrose solutions; wort is not pure sucrose solution.
4. **Unfermented wort vs fermenting beer:** ethanol changes refractive behaviour. Raw post-fermentation Brix must not be treated as an unfermented wort reading.
5. **Correction vs calibration:** applying a generic factor does not calibrate an instrument. Calibration, verification, sample handling and correction are distinct.
6. **Precision vs accuracy:** extra displayed decimal places do not establish measurement accuracy or uncertainty.

## Proposed page structure (not approved for publication)

- What a gravity reading tells you—and what it does not.
- Hydrometers: scale, reference temperature, sample temperature and reading technique.
- Refractometers: Brix scale, sucrose basis and why wort differs.
- Why fermentation changes refractometer interpretation.
- Measurement provenance to record: instrument type/model, scale, reference temperature, sample temperature, sample state, timestamp and correction method/version.
- Common interpretation errors, glossary, sources and limitations.

## Publication and implementation gates

- Obtain the full current ASBC method(s) for the exact analytical claim; the general methods catalogue is insufficient.
- Obtain brewing-science SME review of terminology, sample-state distinctions and worked examples.
- If a calculator is proposed, product owner must name the exact method/version and supported instruments, scales, temperature range and sample states.
- Independently generate golden vectors from an authoritative method; test boundaries, units, rounding, locale and invalid inputs.
- Compare against the actual product implementation. Do not infer formulae or default factors from commercial calculators or secondary tutorials.
- Confirm licensing and attribution rights before reproducing tables or method text.
- Obtain explicit product-stage and public-publication approval.

## Decision

**WP-10:** defensible research outline for a future measurement-context article; not publishable yet.  
**WP-09:** no calculation method, correction factor, default or calculator is approved by this note.

## References

- Wright, J. D. & Bean, V. (2010), *NIST Calibration Services for Hydrometers*, NIST SP 250-78r1. https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.250-78r1.pdf
- Essery, R. E. (1954), *Journal of the Institute of Brewing*, 60(1), 21–28. https://doi.org/10.1002/j.2050-0416.1954.tb02743.x
- Košin, P., Šavel, J. & Brož, A. (2016), *Kvasný Průmysl*, 62(3), 90–93. https://doi.org/10.18832/kp2016014
- American Homebrewers Association, *How to Take an Accurate Hydrometer Reading*. https://homebrewersassociation.org/how-to-brew/how-to-take-an-accurate-hydrometer-reading/
