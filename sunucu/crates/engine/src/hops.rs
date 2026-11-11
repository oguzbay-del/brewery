//! Section 02: hop utilisation and IBU (`formuller/02-hop-ibu.md`).
//!
//! Units follow the specification of each function; see the section file. Internal
//! units are BeerSmith's: ounces, US gallons, degrees Fahrenheit, feet, minutes, alpha
//! acid in percent (5.0 = 5 %). Gravity is plain specific gravity (1.050).
//!
//! The recipe-level engine (spec §4–§9) takes the plain structs defined below; their
//! fields use the spec's internal units (fl oz for volumes, oz for weights, °F for the
//! whirlpool temperature, ft for altitude, minutes for times).
//!
//! Every function mirrors `formuller/testler/02_hop_ibu.py` (the parity reference).

use crate::units::{c_to_f, f_to_c};

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

// ---------------------------------------------------------------------------------------
// T1.14a: single-hop IBU, recipe-level engine, tools (spec §2–§10, previously unported).
// ---------------------------------------------------------------------------------------

/// Hop addition (spec §0, §1). `amount` in oz (fl oz for forms 3 and 4), `alpha` in %,
/// `form` 0 pellet / 1 plug / 2 leaf / 3 CO2 extract / 4 isomerised extract, `usage`
/// 0 boil / 1 dry hop / 2 mash / 3 first wort / 4 whirlpool, `time` in minutes,
/// `whirlpool_temp_f` in °F (used for `usage = 4`).
#[derive(Debug, Clone, PartialEq)]
pub struct Hop {
    pub amount: f64,
    pub alpha: f64,
    pub form: i64,
    pub usage: i64,
    pub time: f64,
    pub whirlpool_temp_f: f64,
}

/// Hop correction options (spec §4); defaults are BeerSmith's (`Default`).
#[derive(Debug, Clone, PartialEq)]
pub struct HopOptions {
    /// `O_HOP_FIRST_WORT_ADJ` in % (default +10).
    pub first_wort_adj: f64,
    /// `O_HOP_TYPE_ADJ`: non-zero applies the per-form corrections (default 1).
    pub type_adj: i64,
    /// `O_HOP_LEAF_ADJ` in % (default 0).
    pub leaf_adj: f64,
    /// `O_HOP_PLUG_ADJ` in % (default +2).
    pub plug_adj: f64,
    /// `O_HOP_PELLET_ADJ` in % (default +10).
    pub pellet_adj: f64,
    /// `O_HOP_MASH_ADJ` in % (default −80).
    pub mash_adj: f64,
}

impl Default for HopOptions {
    fn default() -> Self {
        Self {
            first_wort_adj: 10.0,
            type_adj: 1,
            leaf_adj: 0.0,
            plug_adj: 2.0,
            pellet_adj: 10.0,
            mash_adj: -80.0,
        }
    }
}

/// Recipe and equipment fields used by the hop engine (spec §5). Volumes in fl oz,
/// `altitude` in ft, times in minutes, `hop_util` in % (`F_E_HOP_UTIL`),
/// `recipe_type` 0 = Extract, `whirlpool_carry` = `F_E_WHIRLPOOL_CARRY` ≠ 0,
/// `whirlpool_time` = `F_E_WHIRLPOOL_TIME`.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Recipe {
    pub volume: f64,
    pub trub_loss: f64,
    pub boil_time: f64,
    pub boil_vol: f64,
    pub altitude: f64,
    pub recipe_type: i64,
    pub hop_util: f64,
    pub whirlpool_carry: bool,
    pub whirlpool_time: f64,
}

/// Fermentable as seen by the hop engine (spec §5.1, §5.4). `usage` is `F_G_USE`,
/// `kind` is `F_G_TYPE`, `late_extract` the late-extract boil minutes, `points` the
/// `GetOGPoints` result (supplied by the fermentable module), `amount_oz` in oz,
/// `yield_pct` in %, `ibu_gal_per_lb` the hopped-extract rating.
#[derive(Debug, Clone, PartialEq)]
pub struct Fermentable {
    pub name: String,
    pub usage: i64,
    pub kind: i64,
    pub amount_oz: f64,
    pub yield_pct: f64,
    pub late_extract: f64,
    pub points: f64,
    pub ibu_gal_per_lb: f64,
}

/// Misc item for the whirlpool-time rule (spec §9): `usage` 6 = whirlpool, `time` minutes.
#[derive(Debug, Clone, PartialEq)]
pub struct MiscItem {
    pub usage: i64,
    pub time: f64,
}

/// Outcome of `scale_bitterness` (spec §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleStatus {
    /// Amounts were rescaled.
    Ok,
    /// Target equals the current IBU; nothing to do.
    Unchanged,
    /// Current or target IBU is about zero.
    Failed,
}

/// Result of `scale_bitterness`: status and the new amount per hop (oz, input order).
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleResult {
    pub status: ScaleStatus,
    pub amounts: Vec<f64>,
}

/// Result of the Hop Bitterness tool (spec §7): total IBU, total AAU, IBU per hop.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolBitterness {
    pub ibu: f64,
    pub aau: f64,
    pub contribs: Vec<f64>,
}

/// Tinseth IBU of a single hop (spec §2.1): `U_T · A · W · K / V`. `w` oz, `a` %, `t` min,
/// `sg`, `v` gallons. Returns 0 for `t ≤ 0`, `w ≤ 0`, `a < 0` or `v = 0`.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn tinseth_ibu(w: f64, a: f64, t: f64, sg: f64, v: f64) -> f64 {
    if t <= 0.0 || w <= 0.0 || a < 0.0 || v == 0.0 {
        return 0.0;
    }
    tinseth_utilization(sg, t) * a * w * K / v
}

/// Tinseth inverse (spec §2.1): ounces needed for `ibu` in `v` gallons,
/// `IBU · V / (U_T · A · K)`. Returns 0 when the divisor is 0 (choice, spec silent).
#[must_use]
#[allow(clippy::float_cmp)]
pub fn tinseth_weight(ibu: f64, v: f64, sg: f64, t: f64, a: f64) -> f64 {
    let d = tinseth_utilization(sg, t) * a * K;
    if d == 0.0 { 0.0 } else { ibu * v / d }
}

/// Rager IBU of a single hop (spec §2.2): `R(n) · A · W · K / ((1 + GA) · V)`, `w` oz,
/// `a` %, `t` min, `v` gallons. Same preconditions as [`tinseth_ibu`].
#[must_use]
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
pub fn rager_ibu(w: f64, a: f64, t: f64, sg: f64, v: f64) -> f64 {
    if t <= 0.0 || w <= 0.0 || a < 0.0 || v == 0.0 {
        return 0.0;
    }
    rager_table((t + 0.5) as i64) * a * w * K / ((1.0 + rager_gravity_adjustment(sg)) * v)
}

/// Rager inverse (spec §2.2): `(1 + GA) · V · IBU / (R · A · K)` in oz; 0 when the divisor
/// is 0 (choice, spec silent).
#[must_use]
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
pub fn rager_weight(ibu: f64, v: f64, sg: f64, t: f64, a: f64) -> f64 {
    let d = rager_table((t + 0.5) as i64) * a * K;
    if d == 0.0 {
        0.0
    } else {
        (1.0 + rager_gravity_adjustment(sg)) * v * ibu / d
    }
}

/// Garetz inverse by bisection (spec §2.3): ounces for target `ibu` (`a` %, `t` min, `sg`,
/// `v` gallons, boil volume = `v`). Range [0, 1e6], start 500, stop at `|IBU − target| ≤ 0.05`
/// (capped at 200 steps, choice).
#[must_use]
#[allow(clippy::manual_midpoint)]
pub fn garetz_weight(ibu: f64, a: f64, t: f64, sg: f64, v: f64) -> f64 {
    let (mut lo, mut hi, mut w) = (0.0_f64, 1e6_f64, 500.0_f64);
    for _ in 0..200 {
        let cur = garetz_ibu(w, a, t, sg, v, v);
        if (cur - ibu).abs() <= 0.05 {
            break;
        }
        if cur < ibu {
            lo = w;
        } else {
            hi = w;
        }
        w = (lo + hi) / 2.0;
    }
    w
}

/// Whirlpool utilisation `U_T(SG, t) · Ftemp(T)` (spec §3.4); `t` steep minutes, `temp_c` °C.
#[must_use]
pub fn whirlpool_utilization(sg: f64, t: f64, temp_c: f64) -> f64 {
    tinseth_utilization(sg, t) * temperature_factor(temp_c)
}

/// Whirlpool hop IBU (spec §3.4): `U_W · A · W · K / V`, `w` oz, `a` %, `v` gallons,
/// `temp_c` °C. 0 when `v = 0`.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn whirlpool_ibu(w: f64, a: f64, t: f64, sg: f64, v: f64, temp_c: f64) -> f64 {
    if v == 0.0 {
        return 0.0;
    }
    whirlpool_utilization(sg, t, temp_c) * a * w * K / v
}

/// Boiling point in °F at an altitude (spec §3.2 `BoilTemp`); `alt_ft` in feet.
#[must_use]
pub fn boil_temp_f(alt_ft: f64) -> f64 {
    c_to_f(boil_temp_c(alt_ft))
}

/// Hop equation selection (spec §1): `equation` outside 0..=2 falls back to the global
/// setting.
#[must_use]
pub fn resolve_equation(equation: i64, global_equation: i64) -> i64 {
    if (0..=2).contains(&equation) {
        equation
    } else {
        global_equation
    }
}

/// Segment utilisation `HopUtil` (spec §3.3). `equation` 0 Tinseth / 1 Rager / 2 Garetz;
/// `whirlpool_mu` forces Tinseth; `temp_f` is the wort temperature in °F (boil point for
/// boil hops). Result is a ratio 0..1.
#[must_use]
pub fn hop_utilization(sg: f64, t: f64, equation: i64, whirlpool_mu: bool, temp_f: f64) -> f64 {
    let u = if whirlpool_mu {
        tinseth_utilization(sg, t)
    } else if equation == 1 {
        rager_utilization(sg, t)
    } else if equation == 2 {
        garetz_utilization(sg, t)
    } else {
        tinseth_utilization(sg, t)
    };
    u * temperature_factor(f_to_c(temp_f))
}

/// Hop correction multiplier `HopAdjustments` (spec §4). `util_pct` is the equipment hop
/// utilisation in %, `extract_recipe` is `recipe type == Extract`. Form 4 gives 1, dry hop 0.
#[must_use]
pub fn hop_adjustments(
    form: i64,
    usage: i64,
    util_pct: f64,
    extract_recipe: bool,
    opts: &HopOptions,
) -> f64 {
    if form == 4 {
        return 1.0;
    }
    if usage == 1 {
        return 0.0;
    }
    let mut m = 1.0;
    if usage == 3 && !extract_recipe {
        m *= (100.0 + opts.first_wort_adj) / 100.0;
    }
    if opts.type_adj != 0 {
        match form {
            2 => m *= (100.0 + opts.leaf_adj) / 100.0,
            1 => m *= (100.0 + opts.plug_adj) / 100.0,
            0 => m *= (100.0 + opts.pellet_adj) / 100.0,
            _ => {}
        }
    }
    if usage == 2 && !extract_recipe {
        m *= (100.0 + opts.mash_adj) / 100.0;
    }
    util_pct / 100.0 * m
}

fn is_late_extract(g: &Fermentable) -> bool {
    matches!(g.kind, 1 | 2 | 4 | 5 | 6 | 7) && g.late_extract > 0.0
}

fn sg_from_points(p: f64, v: f64, r: f64) -> f64 {
    if v <= 0.0 {
        1.0
    } else {
        1.0 + p * r / (v / 128.0) / 1000.0
    }
}

/// Boil gravity profile `GetLateExtractSGs` (spec §5.1): list of `(SG, minutes)`, minutes
/// descending, last element at 0. `v_boil` in fl oz (≤ 0 reads `recipe.boil_vol`; at least
/// 0.1). Late-extract points are `oz/16 · yield · 46/100`.
#[must_use]
pub fn late_extract_sgs(recipe: &Recipe, ferms: &[Fermentable], v_boil: f64) -> Vec<(f64, f64)> {
    let vb = (if v_boil > 0.0 {
        v_boil
    } else {
        recipe.boil_vol
    })
    .max(0.1);
    let v = recipe.volume;
    let r = (v + recipe.trub_loss) / vb;
    let mut p: f64 = ferms
        .iter()
        .filter(|g| matches!(g.usage, 0 | 1 | 2 | 8) && !is_late_extract(g))
        .map(|g| g.points)
        .sum();
    let mut prof = vec![(sg_from_points(p, v, r), recipe.boil_time)];
    let mut late: Vec<&Fermentable> = ferms.iter().filter(|g| is_late_extract(g)).collect();
    late.sort_by(|a, b| {
        b.late_extract
            .total_cmp(&a.late_extract)
            .then_with(|| a.name.cmp(&b.name))
    });
    for g in late {
        p += g.amount_oz / 16.0 * (g.yield_pct * 46.0 / 100.0);
        let t = recipe.boil_time.min(g.late_extract);
        let sg = sg_from_points(p, v, r);
        match prof.last_mut() {
            Some(last) if (last.1 - t).abs() < 0.1 => last.0 = sg,
            _ => prof.push((sg, t)),
        }
    }
    p += ferms
        .iter()
        .filter(|g| g.usage == 3 && !is_late_extract(g))
        .map(|g| g.points)
        .sum::<f64>();
    prof.push((
        if p <= 0.0 {
            1.0
        } else {
            sg_from_points(p, v, r)
        },
        0.0,
    ));
    prof
}

/// Total utilisation of one hop `HopTotalUtil` (spec §5.2). `profile` comes from
/// [`late_extract_sgs`]; `equation` 0..=2. Ratio 0..1.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn hop_total_utilization(
    hop: &Hop,
    recipe: &Recipe,
    profile: &[(f64, f64)],
    equation: i64,
) -> f64 {
    if hop.form == 4 {
        return 1.0;
    }
    if hop.amount <= 0.0 || hop.usage == 1 {
        return 0.0;
    }
    let (Some(first), Some(last)) = (profile.first(), profile.last()) else {
        return 0.0;
    };
    let th = hop.time;
    if hop.usage == 4 {
        return tinseth_utilization(last.0, th) * temperature_factor(f_to_c(hop.whirlpool_temp_f));
    }
    let tbf = boil_temp_f(recipe.altitude);
    let mut segs: Vec<f64> = Vec::new();
    if first.1 < th {
        segs.push(
            hop_utilization(first.0, th, equation, false, tbf)
                - hop_utilization(first.0, first.1, equation, false, tbf),
        );
    }
    let mut i = 0;
    while let Some(&(sg, t_i)) = profile.get(i) {
        if t_i == 0.0 {
            break;
        }
        let Some(&(_, t_next)) = profile.get(i + 1) else {
            break;
        };
        if t_next <= th {
            let start = t_i.min(th);
            segs.push(
                hop_utilization(sg, start, equation, false, tbf)
                    - hop_utilization(sg, t_next, equation, false, tbf),
            );
        }
        i += 1;
    }
    if recipe.whirlpool_carry {
        segs.push(post_boil_utilization(last.0, th, recipe.whirlpool_time));
    }
    combine_utilizations(&segs)
}

/// IBU of one hop in a recipe, `NewHopIBU` (spec §5.3):
/// `F_adj · W · U · A · 100 · 128 / (V_ibu · 1.34)` with `V_ibu = volume + trub` (fl oz,
/// 0.01 when ≤ 0). Form 4 amounts are fl oz (density 1).
#[must_use]
pub fn new_hop_ibu(
    hop: &Hop,
    recipe: &Recipe,
    profile: &[(f64, f64)],
    equation: i64,
    opts: &HopOptions,
) -> f64 {
    let mut vibu = recipe.volume + recipe.trub_loss;
    if vibu <= 0.0 {
        vibu = 0.01;
    }
    let u = hop_total_utilization(hop, recipe, profile, equation);
    let adj = hop_adjustments(
        hop.form,
        hop.usage,
        recipe.hop_util,
        recipe.recipe_type == 0,
        opts,
    );
    adj * hop.amount * u * hop.alpha * 100.0 * 128.0 / (vibu * 1.34)
}

/// IBU of a hopped-extract fermentable (spec §5.4): `oz/16 · IBU·gal/lb / max((V+T)/128,
/// 0.0001)`. `volume` and `trub` in fl oz.
#[must_use]
pub fn hopped_extract_ibu(amount_oz: f64, ibu_gal_per_lb: f64, volume: f64, trub: f64) -> f64 {
    (amount_oz / 16.0 * ibu_gal_per_lb) / ((volume + trub) / 128.0).max(0.0001)
}

fn extract_total_ibu(ferms: &[Fermentable], recipe: &Recipe) -> f64 {
    let mut e = 0.0;
    for g in ferms {
        if g.kind == 1 && g.ibu_gal_per_lb > 0.0 {
            e += hopped_extract_ibu(
                g.amount_oz,
                g.ibu_gal_per_lb,
                recipe.volume,
                recipe.trub_loss,
            );
        }
    }
    e
}

/// Recipe IBU `NewGetEstIBU` (spec §5.4): sum of hop IBUs plus hopped extracts.
/// `equation` 0 Tinseth / 1 Rager / 2 Garetz (the `RecipeIBU*` variants), `v_boil` fl oz.
#[must_use]
pub fn recipe_ibu(
    hops: &[Hop],
    recipe: &Recipe,
    ferms: &[Fermentable],
    equation: i64,
    opts: &HopOptions,
    v_boil: f64,
) -> f64 {
    let profile = late_extract_sgs(recipe, ferms, v_boil);
    let mut tot = 0.0;
    for h in hops {
        tot += new_hop_ibu(h, recipe, &profile, equation, opts);
    }
    tot + extract_total_ibu(ferms, recipe)
}

/// Hop share `W · 100 / total` in % (spec §5.4); 0 when the total is ≤ 0 (choice).
#[must_use]
pub fn hop_percent(w: f64, tot_hops: f64) -> f64 {
    if tot_hops <= 0.0 {
        0.0
    } else {
        w * 100.0 / tot_hops
    }
}

/// Scale all non-dry hops to a target IBU, `ScaleBitterness` (spec §6.1). `current_ibu` is
/// `F_R_IBU`. Dry hops (any form) are neither counted nor scaled (the index-shift bug is
/// not reproduced). "About zero" means below 1e-9.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn scale_bitterness(
    current_ibu: f64,
    target: f64,
    hops: &[Hop],
    recipe: &Recipe,
    ferms: &[Fermentable],
    equation: i64,
    opts: &HopOptions,
    v_boil: f64,
) -> ScaleResult {
    let mut amounts: Vec<f64> = hops.iter().map(|h| h.amount).collect();
    if (target - current_ibu).abs() < 1e-9 {
        return ScaleResult {
            status: ScaleStatus::Unchanged,
            amounts,
        };
    }
    if current_ibu.abs() < 1e-9 || target.abs() < 1e-9 {
        return ScaleResult {
            status: ScaleStatus::Failed,
            amounts,
        };
    }
    let mut rc = recipe.clone();
    if rc.volume == 0.0 {
        rc.volume = 0.0001;
    }
    let profile = late_extract_sgs(&rc, ferms, v_boil);
    let mut ibus: Vec<f64> = Vec::with_capacity(hops.len());
    for h in hops {
        if h.usage == 1 {
            ibus.push(0.0);
            continue;
        }
        let i = new_hop_ibu(h, &rc, &profile, equation, opts);
        ibus.push(if i < 1e-9 { 0.0 } else { i });
    }
    let e = extract_total_ibu(ferms, &rc);
    let r2 = if e < current_ibu {
        current_ibu - e
    } else {
        current_ibu
    };
    let t2 = (target - e).max(0.0);
    for (k, h) in hops.iter().enumerate() {
        if h.usage == 1 || ibus[k] == 0.0 {
            continue;
        }
        let u = hop_total_utilization(h, &rc, &profile, equation);
        if h.alpha * u <= 0.0 {
            continue;
        }
        let adj = hop_adjustments(h.form, h.usage, rc.hop_util, rc.recipe_type == 0, opts);
        amounts[k] = (ibus[k] / adj) * (t2 / r2) * (rc.volume + rc.trub_loss) * 1.34
            / (h.alpha * u * 100.0 * 128.0);
    }
    ScaleResult {
        status: ScaleStatus::Ok,
        amounts,
    }
}

/// New hop amount for an edited IBU, `HopIBUAdjust` (spec §6.2): `W_old / IBU_old · IBU_new`;
/// 0 when `ibu_old ≤ 0` (choice).
#[must_use]
pub fn hop_ibu_adjust(w_old: f64, ibu_old: f64, ibu_new: f64) -> f64 {
    if ibu_old <= 0.0 {
        0.0
    } else {
        w_old / ibu_old * ibu_new
    }
}

/// Hop Bitterness tool, `CalcIBU` (spec §7). `sg` boil gravity, `v_floz` batch volume in fl
/// oz, `util_pct` the tool's hop utilisation in %. Form 4 gives `W · A · 100 · 128 / (V ·
/// 1.34)` (0 when `v_floz ≤ 0`, choice). Recipe type is never Extract in the tool.
#[must_use]
pub fn tool_hop_ibu(
    hop: &Hop,
    sg: f64,
    v_floz: f64,
    equation: i64,
    util_pct: f64,
    opts: &HopOptions,
) -> f64 {
    let (w, a, t) = (hop.amount, hop.alpha, hop.time);
    if hop.form == 4 {
        return if v_floz <= 0.0 {
            0.0
        } else {
            w * a * 100.0 * 128.0 / (v_floz * 1.34)
        };
    }
    if w <= 0.0 || hop.usage == 1 || v_floz <= 0.0 {
        return 0.0;
    }
    let vg = v_floz / 128.0;
    let ibu = if hop.usage == 4 {
        whirlpool_ibu(w, a, t, sg, vg, f_to_c(hop.whirlpool_temp_f))
    } else if equation == 1 {
        rager_ibu(w, a, t, sg, vg)
    } else if equation == 2 {
        garetz_ibu(w, a, t, sg, vg, vg)
    } else {
        tinseth_ibu(w, a, t, sg, vg)
    };
    ibu * hop_adjustments(hop.form, hop.usage, util_pct, false, opts)
}

/// AAU / HBU `CalcAAU` (spec §7.1): `W · A` for boil or first-wort hops of at least
/// 15 minutes, else 0. `w` oz, `a` %, `t` minutes.
#[must_use]
pub fn hop_aau(w: f64, a: f64, usage: i64, t: f64) -> f64 {
    if matches!(usage, 0 | 3) && t >= 15.0 {
        w * a
    } else {
        0.0
    }
}

/// Hop Bitterness tool totals (spec §7): summed IBU, summed AAU and the per-hop IBUs.
#[must_use]
pub fn tool_bitterness(
    hops: &[Hop],
    sg: f64,
    v_floz: f64,
    equation: i64,
    util_pct: f64,
    opts: &HopOptions,
) -> ToolBitterness {
    let contribs: Vec<f64> = hops
        .iter()
        .map(|h| tool_hop_ibu(h, sg, v_floz, equation, util_pct, opts))
        .collect();
    ToolBitterness {
        ibu: contribs.iter().sum(),
        aau: hops
            .iter()
            .map(|h| hop_aau(h.amount, h.alpha, h.usage, h.time))
            .sum(),
        contribs,
    }
}

/// Total hop amount of a phase, `RecipeTotHopsPhase` (spec §8): hops with `usage == phase`,
/// plus first-wort hops (3) when `phase == 0`. Oz as stored.
#[must_use]
pub fn total_hops_phase(hops: &[Hop], phase: i64) -> f64 {
    hops.iter()
        .filter(|h| h.usage == phase || (phase == 0 && h.usage == 3))
        .map(|h| h.amount)
        .sum()
}

/// Hop rate in g/L (spec §8): phase total (oz → g) over the batch volume (`volume_floz` in
/// fl oz, no trub). 0 when the volume is 0.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn hop_rate(hops: &[Hop], volume_floz: f64, phase: i64) -> f64 {
    if volume_floz == 0.0 {
        return 0.0;
    }
    total_hops_phase(hops, phase) * 28.349_523_125 / (volume_floz * 29.573_529_56 / 1000.0)
}

/// Total whirlpool time in whole minutes, `TotWhirlpoolTime` (spec §9): the larger of
/// `recipe_time` and `int(t + 0.48)` over whirlpool hops (usage 4) and whirlpool misc
/// items (usage 6).
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn total_whirlpool_time(recipe_time: i64, hops: &[Hop], misc: &[MiscItem]) -> i64 {
    let mut m = recipe_time;
    for h in hops.iter().filter(|h| h.usage == 4) {
        m = m.max((h.time + 0.48) as i64);
    }
    for x in misc.iter().filter(|x| x.usage == 6) {
        m = m.max((x.time + 0.48) as i64);
    }
    m
}

/// Alpha after storage from months and storage type, `HopAlphaFunc` (spec §10): days =
/// `months · 30`; `storage_type` 0 poly 1.0, 1 oxygen barrier 0.75, 2 vacuum 0.5 (other
/// values 1.0, choice). Returns alpha %.
#[must_use]
pub fn aged_alpha_months(alpha: f64, hsi: f64, temp_f: f64, months: f64, storage_type: i64) -> f64 {
    let sf = match storage_type {
        1 => 0.75,
        2 => 0.5,
        _ => 1.0,
    };
    aged_alpha(alpha, hsi, temp_f, months * 30.0, sf)
}
