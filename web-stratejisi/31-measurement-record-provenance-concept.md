# Measurement Record Provenance — Conceptual Data Contract

**Status:** Discovery artifact only. Not an approved database schema, API contract or shipped feature.  
**Audience:** Product, engineering and brewing-science reviewers.

## Objective

A brewing measurement should remain interpretable after the brew day. A numeric value alone is not enough to understand what was measured, with which instrument, under what conditions, and whether a later correction or conversion was applied.

## Candidate record envelope

| Field | Purpose | Gate / note |
|---|---|---|
| `measurement_type` | Gravity, temperature, pH, volume, mass, pressure, etc. | Controlled vocabulary needs product-owner approval. |
| `value` + `unit` | Original observed value and scale/unit. | Preserve original observation; do not silently normalize. |
| `normalized_value` + `normalized_unit` | Optional derived representation. | Only when an approved, versioned conversion exists. |
| `method_id` + `method_version` | How the value was obtained or transformed. | Required for reproducibility when method-dependent. |
| `instrument_id` / `instrument_model` | Device provenance. | Optional for manual readings; device identity rules TBD. |
| `calibration_reference` | Instrument's stated calibration/reference temperature or basis. | Do not assume one universal reference. |
| `sample_temperature` | Temperature at measurement time. | Distinct from instrument reference temperature. |
| `sample_state` | Wort, active fermentation, finished beer, etc. | Controlled values and definitions need SME review. |
| `sample_preparation` | e.g. mixed, degassed, filtered, cooled. | Capture only if meaningful to method. |
| `observed_at` | When the measurement occurred. | Store timezone-aware timestamp. |
| `recorded_at` | When the value was entered/imported. | Distinguish delayed entry from observation time. |
| `source` | Manual entry, sensor, import, lab report. | Provenance and trust semantics need design. |
| `correction` | Whether a correction was applied and its method/version. | No correction by default without approved method. |
| `uncertainty_or_precision` | Known uncertainty or instrument resolution. | Avoid false precision; fields require domain review. |
| `batch_id` / `vessel_id` | Process context. | Only where product relationships support it. |

## Invariants worth considering

1. Keep the raw observation immutable; store transformations separately.
2. Never overwrite the original unit or scale when normalizing.
3. A derived value must reference the method and version that produced it.
4. Missing context must remain explicitly unknown, not silently defaulted.
5. Distinguish observed time from entry time.
6. Device-reported, manually entered and laboratory values must retain their source.
7. Do not imply calibration simply because an instrument model is recorded.
8. Preserve the difference between planned targets and actual observations.

## Open decisions

- Which measurement types are in the first product release?
- Are methods centrally versioned, or attached to individual calculations?
- What is the canonical unit strategy, and which conversions are lossless?
- Which instruments/sensors are actually supported?
- How are imported lab certificates and uncertainty represented?
- Which fields are mandatory for each method and sample state?
- What retention, audit and correction history is required for professional users?

## Approval gates

Product owner, engineering, brewing-science SME and (where relevant) QA/security must approve the data contract before implementation. Any public description must match shipped fields and supported workflows. This document does not authorize schema changes.
