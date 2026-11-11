//! Section 01: gravity, alcohol, colour (`formuller/01-yogunluk-alkol-renk.md`).
//!
//! Units follow the specification of each function; see the section file. Internal units
//! are BeerSmith's: SG, °Plato, °Brix, °F, SRM, US fl oz, US gallons, percent (0-100).
//! Shared conversions (`to_plato`, `to_sg`) live in [`crate::units`].

use crate::units::{to_plato, to_sg};

/// Brix to SG (§1.4): `b / (258.6 - b/258.2 * 227.1) + 1`.
#[must_use]
pub fn brix_to_sg(brix: f64) -> f64 {
    brix / (258.6 - (brix / 258.2) * 227.1) + 1.0
}

/// SG to °Brix (§1.4), cubic.
#[must_use]
pub fn sg_to_brix(sg: f64) -> f64 {
    ((182.460_1 * sg - 775.682_1) * sg + 1_262.779_4) * sg - 669.562_2
}

/// °Brix to refractive index (§1.4).
#[must_use]
pub fn brix_to_ri(brix: f64) -> f64 {
    1.333_02 + 0.001_427_193 * brix + 5.791_157e-6 * brix * brix
}

/// Refractive index to °Brix (§1.4), inverse of [`brix_to_ri`].
#[must_use]
pub fn ri_to_brix(ri: f64) -> f64 {
    ((2.036_879_859_249e-06 - (1.333_02 - ri) * 2.316_462_8e-05).sqrt() - 0.001_427_193)
        / 1.158_231_4e-05
}

/// Original extract clamped to be at least the apparent extract (°P), §6.
fn clamped_extracts(og: f64, fg: f64) -> (f64, f64) {
    let f = to_plato(fg);
    (to_plato(og).max(f), f)
}

/// Real extract (°P) from OG and FG (SG), §6.1. `OE = max(OE, AE)`.
#[must_use]
pub fn real_extract(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    0.1808 * o + 0.8192 * f
}

/// Alcohol by weight (%) from OG and FG (SG), §6.2.
#[must_use]
pub fn abw(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    let re = 0.1808 * o + 0.8192 * f;
    (o - re) / (2.0665 - 0.010_665 * o)
}

/// Alcohol by volume (%) from OG and FG (SG), §6.3: `FG / 0.79661 * ABW`.
#[must_use]
pub fn abv(og: f64, fg: f64) -> f64 {
    fg / 0.79661 * abw(og, fg)
}

/// Apparent attenuation (%), §6.4. OG = 1.000 divides by zero (no guard, as in the reference).
#[must_use]
pub fn apparent_attenuation(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    (o - f) / o * 100.0
}

/// Real attenuation (%), §6.5.
#[must_use]
pub fn real_attenuation(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    (1.0 - (0.1808 * o + 0.8192 * f) / o) * 100.0
}

/// Calories from alcohol per 12 US fl oz serving (kcal), §7.1. Takes OG and FG in SG.
#[must_use]
pub fn alcohol_calories(og: f64, fg: f64) -> f64 {
    1881.22 * fg * (og - fg) / (1.775 - og)
}

/// Calories from extract (carbohydrate) per 12 US fl oz serving (kcal), §7.2. OG and FG in SG.
#[must_use]
pub fn extract_calories(og: f64, fg: f64) -> f64 {
    3550.0 * fg * ((0.1808 * og + 0.8192 * fg) - 1.0004)
}

/// UK tax ABV (%) from measured OG and FG (SG), §6.7. Table step applies when `d > threshold`,
/// with `d = (OG - FG) * 1000`.
#[must_use]
pub fn uk_tax_abv(og: f64, fg: f64) -> f64 {
    const STEPS: [(f64, f64); 10] = [
        (6.95, 0.126),
        (10.45, 0.127),
        (17.25, 0.128),
        (26.15, 0.129),
        (36.05, 0.130),
        (46.55, 0.131),
        (57.15, 0.132),
        (67.95, 0.133),
        (78.85, 0.134),
        (89.75, 0.135),
    ];
    let d = (og - fg) * 1000.0;
    let mut k = 0.125;
    for (limit, factor) in STEPS {
        if d > limit {
            k = factor;
        }
    }
    k * d
}

/// Morey colour (SRM), §8.1: `1.4922 * (mcu_sum / volume_gal)^0.6859`, where `mcu_sum` is
/// the sum of (pounds x grain SRM) and `volume_gal` is in US gallons.
#[must_use]
pub fn morey_srm(mcu_sum: f64, volume_gal: f64) -> f64 {
    1.4922 * (mcu_sum / volume_gal).powf(0.6859)
}

/// Hydrometer temperature correction (SG), §9.1. `temp_f` is the sample temperature and
/// `calib_f` the hydrometer calibration temperature, both in °F.
#[must_use]
pub fn hydrometer_correction(sg: f64, temp_f: f64, calib_f: f64) -> f64 {
    let t = (temp_f - 32.0) * 5.0 / 9.0;
    let tc = (calib_f - 32.0) * 5.0 / 9.0;
    sg + sg
        * ((tc.powi(3) - t.powi(3)) * 1.532_436_35e-08 + (tc - t) * 1.628_605_87e-05
            - (tc.powi(2) - t.powi(2)) * 5.849_948_55e-06)
}

/// Refractometer, unfermented wort (SG), §9.2 type 0: `ToSG((brix - zero) / wcf)`.
/// Defaults in the reference: `zero = 0.0`, `wcf = 1.02`.
#[must_use]
pub fn refractometer_unfermented(brix: f64, zero: f64, wcf: f64) -> f64 {
    to_sg((brix - zero) / wcf)
}

/// Refractometer, fermenting wort with known OG (returns FG in SG), §9.2 type 1
/// (Terrill cubic as implemented by BeerSmith: no WCF, OB in °Plato).
#[must_use]
pub fn refractometer_fermenting(og: f64, brix: f64, zero: f64) -> f64 {
    let ob = to_plato(og);
    let fb = brix - zero;
    1.001_843 - 0.002_318_474 * ob - 7.775e-6 * ob * ob - 3.4e-8 * ob.powi(3)
        + 0.005_74 * fb
        + 3.344e-5 * fb * fb
        + 8.6e-8 * fb.powi(3)
}

/// Result of [`refractometer_finished`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FinishedBeer {
    /// Original gravity (SG).
    pub og: f64,
    /// Alcohol by weight (%).
    pub abw_percent: f64,
    /// Real extract (°P).
    pub re: f64,
    /// Original extract (°P).
    pub oe: f64,
}

/// Refractometer on finished beer with hydrometer FG (SG): original gravity, §9.2 type 2.
#[must_use]
pub fn refractometer_finished(fg: f64, brix: f64, zero: f64) -> FinishedBeer {
    let ri = brix_to_ri(brix - zero);
    let abw = (1017.5596 - 277.4 * fg + ri * (937.8135 * ri - 1805.1288)) / 100.0;
    let re = 194.5935 + 129.8 * fg + ri * (410.8815 * ri - 790.8732);
    let oe = (re + 2.0665 * abw * 100.0) / (1.0 + 0.010_665 * abw * 100.0);
    FinishedBeer {
        og: to_sg(oe),
        abw_percent: abw * 100.0,
        re,
        oe,
    }
}

/// FG (SG) limited by yeast alcohol tolerance, §5.2: bisection between `fg` and 1.18 until
/// `|ABV - tol| < 0.04` (at most 50 iterations, then the last midpoint). `tol` is in %ABV.
#[must_use]
#[allow(clippy::manual_midpoint)] // keep `(hi + lo) / 2` for bit parity with the reference
pub fn fg_from_tolerance(og: f64, fg: f64, tol: f64) -> f64 {
    let mut hi = 1.18;
    let mut lo = fg;
    let mut mid = (hi + lo) / 2.0;
    for _ in 0..50 {
        mid = (hi + lo) / 2.0;
        let a = abv(og, mid);
        if (a - tol).abs() < 0.04 {
            return mid;
        }
        if a > tol {
            lo = mid;
        } else if a < tol {
            hi = mid;
        }
    }
    mid
}

/// Extract to add to raise gravity (oz), §9.3. `current` and `target` in SG, `volume_floz`
/// in US fl oz (minimum 0.01). `kind`: 0 = DME (44 PPG), anything else = LME (36 PPG).
/// Returns 0 when `target <= current`.
#[must_use]
pub fn gravity_adjust_extract_oz(current: f64, target: f64, volume_floz: f64, kind: i64) -> f64 {
    let points = (target - current) * 1000.0;
    if points <= 0.0 {
        return 0.0;
    }
    let points = points * volume_floz.max(0.01) / 128.0;
    points * 16.0 / if kind == 0 { 44.0 } else { 36.0 }
}

/// Water to add to lower gravity (US fl oz), §9.4; negative results clamp to 0. `target = 1.000`
/// divides by zero (no guard, as in the reference).
#[must_use]
pub fn gravity_adjust_water_floz(current: f64, target: f64, volume_floz: f64) -> f64 {
    (volume_floz * (current - 1.0) * 1000.0 / ((target - 1.0) * 1000.0) - volume_floz).max(0.0)
}
