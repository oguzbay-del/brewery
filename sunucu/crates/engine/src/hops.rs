//! Section 02: hop utilisation and IBU (`formuller/02-hop-ibu.md`).
//!
//! Units follow the specification of each function; see the section file. Internal
//! units are BeerSmith's: ounces, US gallons, degrees Fahrenheit, feet, minutes, alpha
//! acid in percent (5.0 = 5 %). Gravity is plain specific gravity (1.050).
//!
//! Every function mirrors `formuller/testler/02_hop_ibu.py` (the parity reference).

/// IBU conversion constant `K = 100 / 1.34` (spec §0): `IBU = U · alpha% · oz · K / gal`.
pub const K: f64 = 74.626_865_671_641_78;

/// Safety cap on the Garetz fixed-point iteration (it converges in well under 100 steps).
const GARETZ_MAX_ITER: usize = 10_000;

/// Garetz utilisation table in percent (spec §2.3), indexed by 5-minute step.
const GARETZ_TABLE: [f64; 15] = [
    0.0, 0.0, 2.0, 5.0, 8.0, 11.0, 14.0, 16.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 23.0,
];

/// Rager table upper bounds (minutes) and utilisation, spec §2.2 (`n = 0` is handled apart).
const RAGER_TABLE: [(i64, f64); 11] = [
    (5, 0.05),
    (10, 0.06),
    (15, 0.08),
    (20, 0.101),
    (25, 0.121),
    (30, 0.153),
    (35, 0.188),
    (40, 0.228),
    (45, 0.269),
    (50, 0.281),
    (55, 0.291),
];

/// Tinseth utilisation `U_T(SG, t) = 1.65 · 0.000125^(SG−1) · (1 − e^(−0.04 t)) / 4.15`
/// (spec §2.1). `sg` is specific gravity, `t` boil minutes. Result is a ratio 0..1.
#[must_use]
pub fn tinseth_utilization(sg: f64, t: f64) -> f64 {
    1.65 * 0.000_125_f64.powf(sg - 1.0) * (1.0 - (-0.04 * t).exp()) / 4.15
}

/// Temperature factor `Ftemp(T) = 2.39e11 · e^(−9773 / (T + 273.15))` for `T < 100`,
/// 1 otherwise (spec §3.1). `temp_c` in °C. Note the discontinuity just below 100 °C.
#[must_use]
pub fn temperature_factor(temp_c: f64) -> f64 {
    if temp_c >= 100.0 {
        1.0
    } else {
        2.39e11 * (-9773.0 / (temp_c + 273.15)).exp()
    }
}

/// Rager table `R(n)` for whole minutes `n` (spec §2.2): 0 for `n = 0`, 0.05 for
/// `n ≤ 5` (negatives included), up to 0.30 from 56 minutes.
#[must_use]
pub fn rager_table(n: i64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    for (limit, value) in RAGER_TABLE {
        if n <= limit {
            return value;
        }
    }
    0.30
}

/// Rager gravity adjustment `GA = (SG − 1.050) / 0.2` for `SG > 1.050`, else 0 (spec §2.2).
#[must_use]
pub fn rager_gravity_adjustment(sg: f64) -> f64 {
    if sg > 1.05 { (sg - 1.05) / 0.2 } else { 0.0 }
}

/// Rager utilisation `R(n) / (1 + GA)` with `n = int(t + 0.5)` truncating toward zero
/// (spec §2.2). `sg` specific gravity, `t` minutes. Ratio 0..1.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn rager_utilization(sg: f64, t: f64) -> f64 {
    rager_table((t + 0.5) as i64) / (1.0 + rager_gravity_adjustment(sg))
}

/// Garetz IBU, iterative (spec §2.3). `w` ounces, `a` alpha %, `t` minutes, `sg` boil
/// gravity, `v` final volume (gal), `bv` boil volume (gal). Returns 0 for `t ≤ 0`,
/// `w ≤ 0` or `a < 0`. No lower bound on the gravity factor (SG < 1.050 raises IBU).
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn garetz_ibu(w: f64, a: f64, t: f64, sg: f64, v: f64, bv: f64) -> f64 {
    if t <= 0.0 || w <= 0.0 || a < 0.0 {
        return 0.0;
    }
    // C integer division truncates toward zero, so 0 < t < 1 gives index 0.
    let idx = (((t as i64) - 1) / 5).min(14) as usize;
    let r = v / bv;
    let gf = ((sg - 1.0) * r + 1.0 - 1.05) * 5.0 + 1.0;
    let mut prev = 1.0_f64;
    let mut cur = 50.0_f64;
    for _ in 0..GARETZ_MAX_ITER {
        if (prev - cur).abs() <= 0.05 {
            break;
        }
        prev = cur;
        cur = GARETZ_TABLE[idx] * a * w * 0.749 / (v * gf * (r * cur / 260.0 + 1.0) * 1.02);
    }
    cur
}

/// Garetz utilisation in recipe form: `garetz_ibu(1, 1, t, SG, 1, 1) / 74.9` (spec §2.3).
/// `sg` specific gravity, `t` minutes. Ratio 0..1.
#[must_use]
pub fn garetz_utilization(sg: f64, t: f64) -> f64 {
    garetz_ibu(1.0, 1.0, t, sg, 1.0, 1.0) / 74.9
}

/// Extra utilisation of a boil hop during the whirlpool, `PostBoilUtil` (spec §3.5).
/// `sg` boil-end gravity, `t0` hop boil time (min), `dur` whirlpool minutes. Left
/// Riemann sum with step 0.1 min, accumulated exactly like the reference (the `tau < 5`
/// exception uses absolute hop time).
#[must_use]
pub fn post_boil_utilization(sg: f64, t0: f64, dur: f64) -> f64 {
    let gravity = 1.65 * 0.000_125_f64.powf(sg - 1.0);
    let mut sum = 0.0;
    let mut tau = t0;
    while tau < t0 + dur {
        let temp_f = (tau - t0) * -1.344 + 210.64 + 459.67;
        let mut f = 2.39e11 * (-9773.0 / (temp_f * (5.0 / 9.0))).exp();
        if tau < 5.0 {
            f = 1.0;
        }
        sum += gravity * 0.04 * (-0.04 * tau).exp() / 4.15 * f * 0.1;
        tau += 0.1;
    }
    sum
}

/// Boiling point in °C at an altitude: `100 − alt · 0.3048 · 0.00324` (spec §3.2).
/// `alt_ft` in feet.
#[must_use]
pub fn boil_temp_c(alt_ft: f64) -> f64 {
    100.0 - alt_ft * 0.3048 * 0.00324
}

/// Probabilistic combination of segment utilisations
/// `Σ s_k · Π_{j<k} (1 − s_j) = 1 − Π (1 − s_k)` (spec §5.2). Ratios 0..1.
#[must_use]
pub fn combine_utilizations(segs: &[f64]) -> f64 {
    let mut total = 0.0;
    let mut remaining = 1.0;
    for s in segs {
        total += s * remaining;
        remaining *= 1.0 - s;
    }
    total
}

/// Alpha acid after storage (spec §10): `A0 · exp(−k · TF · days · SF)` with
/// `k = −ln(1 − HSI/100)/180`, `TF = 0.5^((68 − T)/27)`. `alpha` %, `hsi` % lost in six
/// months at 68 °F (capped at 99), `temp_f` °F, `days` of storage, `storage` factor
/// (1.0 poly, 0.75 oxygen barrier, 0.5 vacuum). Returns alpha %.
#[must_use]
pub fn aged_alpha(alpha: f64, hsi: f64, temp_f: f64, days: f64, storage: f64) -> f64 {
    let hsi = hsi.min(99.0);
    let k = -(1.0 - hsi / 100.0).ln() / 180.0;
    alpha * (-k * 0.5_f64.powf((68.0 - temp_f) / 27.0) * days * storage).exp()
}
