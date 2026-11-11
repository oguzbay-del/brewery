//! Section 04: yeast, mead, carbonation (`formuller/04-maya-mead-karbonasyon.md`).
//!
//! Units follow the BeerSmith internal units of the specification: gallons, fl oz,
//! degrees Fahrenheit, litres where stated. Every function documents its units.
//! Ported one to one from `formuller/testler/04_maya_karb_tv.py` (ADR-0003).

use crate::units::{GAL_L, to_plato, to_sg};

/// Stir plate growth bonus, percent (spec §1.2).
const STIR: f64 = 45.0;
/// Millilitres per US fluid ounce.
const FLOZ_ML: f64 = 29.573_529_56;
/// Grams per ounce (`0.4535924 / 16 × 1000`).
pub const OZ_G: f64 = 0.453_592_4 / 16.0 * 1000.0;

/// Litres to US fluid ounces.
#[must_use]
pub fn l_to_floz(litres: f64) -> f64 {
    litres * 1000.0 / FLOZ_ML
}

/// US fluid ounces to litres.
#[must_use]
pub fn floz_to_l(floz: f64) -> f64 {
    floz * FLOZ_ML / 1000.0
}

/// Litres to US gallons.
#[must_use]
pub fn gal_from_l(litres: f64) -> f64 {
    litres * 1000.0 / FLOZ_ML / 128.0
}

/// Brix to SG, keeping the BeerSmith quirk of 258.2 in the correction term (spec §2.8).
#[must_use]
pub fn brix_to_sg(brix: f64) -> f64 {
    brix / (258.6 - (brix / 258.2) * 227.1) + 1.0
}

/// Cell viability, percent, after `days` (spec §1.4).
///
/// `v0` percent, `rate` monthly loss percent, floored at `floor` (default 5). Zero or
/// negative `days` return `v0` unchanged.
#[must_use]
pub fn viability(v0: f64, rate: f64, days: f64, floor: f64) -> f64 {
    if days <= 0.0 {
        return v0;
    }
    let v = v0 * ((100.0 - rate) / 100.0).powf(days / 30.0);
    v.max(floor)
}

/// Inoculation rate, million cells per mL, to growth factor `g` (spec §1.9).
#[must_use]
pub fn innoc_rate(g: f64) -> f64 {
    (g.ln() * -2.179_103_8 + 5.512_221).exp()
}

/// Growth factor for inoculation rate `i` (million cells/mL), clamped to 1..6 (spec §1.9).
///
/// Uses the exponent -2.1781038 (differs from [`innoc_rate`] in the reference).
#[must_use]
pub fn growth_rate(i: f64, stir: bool) -> f64 {
    let mut g = ((i.ln() - 5.512_221) / -2.178_103_8).exp();
    if stir {
        g *= 1.0 + STIR / 100.0;
    }
    g.clamp(1.0, 6.0)
}

/// Cells needed, billions: `rate (M/mL/°P) × volume (L) × °Plato(sg)` (spec §1.8).
#[must_use]
pub fn cells_needed(rate: f64, volume_l: f64, sg: f64) -> f64 {
    rate * volume_l * to_plato(sg)
}

/// Single-stage starter volume, litres, for `need` and `avail` billion cells (spec §1.10).
#[must_use]
pub fn new_starter_l(need: f64, avail: f64, stir: bool) -> f64 {
    if need <= avail {
        return 0.0;
    }
    let mut r = need / avail;
    if stir {
        r /= 1.0 + STIR / 100.0;
    }
    if r <= 1.0 {
        return 0.0;
    }
    avail / innoc_rate(r)
}

/// Two-stage starter volumes `(stage1_l, stage2_l)`, litres (spec §1.11).
#[must_use]
pub fn two_stage_tool(need: f64, avail: f64, stir: bool) -> (f64, f64) {
    let avail = avail.max(0.1);
    if avail > need {
        return (0.0, 0.0);
    }
    let mut r = need / avail;
    let mut s = 1.0;
    if stir {
        s = 1.0 + STIR / 100.0;
        r /= s * s;
    }
    let g = r.sqrt();
    let v1 = avail / innoc_rate(g);
    let v2 = avail * g * s / innoc_rate(g);
    (v1, v2)
}

/// DME ounces for a starter of `vol_floz` fl oz at gravity `og` (spec §1.14).
#[must_use]
pub fn dme_oz(vol_floz: f64, og: f64) -> f64 {
    (og - 1.0) * 1000.0 * vol_floz / 128.0 / 45.0 * 16.0
}

/// Priming sugar, grams, for `vols` volumes CO2 at `temp_f` °F in `gal` gallons (spec §3.3).
///
/// Corn sugar by default; `dme` multiplies by 1.538.
#[must_use]
pub fn sugar_g(vols: f64, temp_f: f64, gal: f64, dme: bool) -> f64 {
    let t = temp_f.max(32.0);
    let g = ((t * 0.050_062 + (vols - 3.0378)) - t * 0.000_265_55 * t) * gal * 15.195;
    if dme { g * 1.538 } else { g }
}

/// Residual dissolved CO2, volumes, at `temp_f` °F (spec §3.2). Used by section 11.
#[must_use]
pub fn residual_co2(temp_f: f64) -> f64 {
    let t = temp_f.max(32.0);
    3.0378 - 0.050_062 * t + 0.000_265_55 * t * t
}

/// Keg pressure, psi, to hold `vols` volumes CO2 at `temp_f` °F (spec §3.5). Used by section 11.
#[must_use]
pub fn co2_psi(vols: f64, temp_f: f64) -> f64 {
    let t = temp_f.max(32.0);
    let p =
        vols * 4.24267 + (-16.6999 - t * 0.010_105_9) + t * 0.001_165_12 * t + t * 0.173_354 * vols
            - vols * 0.068_422_6 * vols;
    p.max(0.0)
}

/// Recommended free SO2, ppm, for wine `ph` (spec §2.6).
#[must_use]
pub fn so2_rec(ph: f64) -> f64 {
    (ph * 2.21828 - 4.27809).exp()
}

/// Potassium metabisulfite, grams, for `v_gal` gallons to raise free SO2 from `free` to
/// `target` ppm (spec §2.6).
#[must_use]
pub fn kmeta_g(v_gal: f64, target: f64, free: f64) -> f64 {
    v_gal * 3.785 * (target - free) / 576.0
}

/// Potassium sorbate, grams, for `v_gal` gallons (spec §2.7).
#[must_use]
pub fn sorbate_g(v_gal: f64) -> f64 {
    v_gal * 1.25
}

/// Mead yeast pitch rate, g/L, from SG (spec §2.2): 0.25 up to 24 °P, 0.5 from 28 °P.
#[must_use]
pub fn mead_pitch(sg: f64) -> f64 {
    let p = to_plato(sg);
    if p <= 24.0 {
        0.25
    } else if p < 28.0 {
        (p - 24.0) * 0.0625 + 0.25
    } else {
        0.5
    }
}

/// Total TOSNA nutrient, grams (spec §2.4).
///
/// `fac` is the yeast nitrogen need (0 low, 1 medium, 2 high; other values are treated as
/// high), `method` 0 Fermaid-O (/50) or otherwise Fermaid-K (/100), `pct` honey percent.
#[must_use]
pub fn tosna(sg: f64, v_gal: f64, pct: f64, fac: i64, method: i64) -> f64 {
    let k = match fac {
        0 => 0.75,
        1 => 0.9,
        _ => 1.25,
    };
    let base = to_plato(sg) * 10.0 * (pct / 100.0) * k * v_gal;
    if method == 0 {
        base / 50.0
    } else {
        base / 100.0
    }
}

/// Yeast assimilable nitrogen, ppm, from `grams` of nutrient in `v_l` litres (spec §2.5).
///
/// `method` 0 = 40, 2 = 212.3, anything else = 100 ppm per g/L. Zero volume returns the
/// bare factor.
#[must_use]
pub fn yan(method: i64, grams: f64, v_l: f64) -> f64 {
    let r = match method {
        0 => 40.0,
        2 => 212.3,
        _ => 100.0,
    };
    if v_l == 0.0 { r } else { r * grams / v_l }
}

/// Re-exported for callers converting gallons: litres per US gallon.
pub const LITRES_PER_GAL: f64 = GAL_L;

/// Backsweetening honey volume, litres (spec §2.8): `v_l` of `sg_now`, to `sg_target`,
/// honey at `sg_honey`.
#[must_use]
pub fn backsweeten_volume_l(v_l: f64, sg_now: f64, sg_target: f64, sg_honey: f64) -> f64 {
    v_l * (sg_now - sg_target) / (sg_target - sg_honey)
}

/// Honey gravity from its Plato value (spec §2.8).
#[must_use]
pub fn honey_sg(plato: f64) -> f64 {
    to_sg(plato)
}
