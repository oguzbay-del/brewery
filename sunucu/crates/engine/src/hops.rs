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

use crate::density::{LateExtractPoint, late_extract_profile};
use crate::error::{Error, Result, ensure_finite, finite_result};
use crate::recipe::{
    GrainType, Hop, HopForm, HopOptions, HopUse, Misc, MiscUse, RecipeInput, RecipeType,
};
use crate::units::{FLOZ_L, OZ_G, boil_temp_f, f_to_c};

/// IBU equation (spec §1). Codes: 0 Tinseth, 1 Rager, 2 Garetz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopEquation {
    /// Code 0.
    Tinseth,
    /// Code 1.
    Rager,
    /// Code 2.
    Garetz,
}

impl HopEquation {
    /// From the BeerSmith code (0-2); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::Tinseth),
            1 => Ok(Self::Rager),
            2 => Ok(Self::Garetz),
            _ => Err(Error::invalid("equation", "unknown code")),
        }
    }
}

/// Hop storage type (spec §10). Codes: 0 poly bag (factor 1.0), 1 oxygen barrier (0.75),
/// 2 vacuum (0.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// Code 0.
    Poly,
    /// Code 1.
    OxygenBarrier,
    /// Code 2.
    Vacuum,
}

impl StorageType {
    /// From the BeerSmith code (0-2); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::Poly),
            1 => Ok(Self::OxygenBarrier),
            2 => Ok(Self::Vacuum),
            _ => Err(Error::invalid("storage_type", "unknown code")),
        }
    }

    /// Storage factor of the ageing formula.
    #[must_use]
    pub const fn factor(self) -> f64 {
        match self {
            Self::Poly => 1.0,
            Self::OxygenBarrier => 0.75,
            Self::Vacuum => 0.5,
        }
    }
}

/// IBU conversion constant `K = 100 / 1.34` (spec §0): `IBU = U · alpha% · oz · K / gal`.
pub const K: f64 = 74.626_865_671_641_78;

/// Cap on the Garetz fixed-point iteration (spec §2.3, T1.15 B9). Normal inputs converge in
/// well under 100 steps; when the cap is hit the engine returns `Error::NoConvergence`
/// (T1.16), like the reference's `ValueError`.
const GARETZ_MAX_ITER: usize = 10_000;

/// Cap on the `PostBoilUtil` Riemann steps (spec §3.5, T1.15 B1). The cooling model reaches
/// 0 K after 498.74 min (about 4990 steps), so the cap only matters for odd inputs such as
/// negative hop times.
const POST_BOIL_MAX_STEPS: usize = 20_000;

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
/// (spec §2.1). `sg` is specific gravity, `t_min` boil minutes. Result is a ratio 0..1.
#[must_use]
pub fn tinseth_utilization(sg: f64, t_min: f64) -> f64 {
    1.65 * 0.000_125_f64.powf(sg - 1.0) * (1.0 - (-0.04 * t_min).exp()) / 4.15
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

/// Rager table `R(n)` for whole minutes `n_min` (spec §2.2): 0 for `n = 0`, 0.05 for
/// `n ≤ 5` (negatives included), up to 0.30 from 56 minutes.
#[must_use]
pub fn rager_table(n_min: i64) -> f64 {
    if n_min == 0 {
        return 0.0;
    }
    for (limit, value) in RAGER_TABLE {
        if n_min <= limit {
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
/// (spec §2.2). `sg` specific gravity, `t_min` minutes. Ratio 0..1.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn rager_utilization(sg: f64, t_min: f64) -> f64 {
    rager_table((t_min + 0.5) as i64) / (1.0 + rager_gravity_adjustment(sg))
}

/// Inputs of [`garetz_ibu`].
#[derive(Debug, Clone, Copy)]
pub struct GaretzIbuInput {
    /// Hop weight, oz.
    pub w_oz: f64,
    /// Alpha acid, %.
    pub a_pct: f64,
    /// Time, minutes.
    pub t_min: f64,
    /// Boil gravity, SG.
    pub sg: f64,
    /// Final volume, gal.
    pub v_gal: f64,
    /// Boil volume, gal.
    pub bv_gal: f64,
}

/// Garetz IBU, iterative (spec §2.3). `w` ounces, `a` alpha %, `t` minutes, `sg` boil
/// gravity, `v` final volume (gal), `bv` boil volume (gal). Returns 0 for `t ≤ 0`,
/// `w ≤ 0` or `a < 0`. Errors (T1.16): non-finite input, `v ≤ 0`, `bv ≤ 0`, a gravity factor
/// `≤ 0`, or an iteration that does not converge within the cap.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn garetz_ibu(p: &GaretzIbuInput) -> Result<f64> {
    let GaretzIbuInput {
        w_oz: w,
        a_pct: a,
        t_min: t,
        sg,
        v_gal: v,
        bv_gal: bv,
    } = *p;
    for (field, x) in [
        ("w", w),
        ("a", a),
        ("t", t),
        ("sg", sg),
        ("v", v),
        ("bv", bv),
    ] {
        ensure_finite(field, x)?;
    }
    if t <= 0.0 || w <= 0.0 || a < 0.0 {
        return Ok(0.0);
    }
    if v <= 0.0 || bv <= 0.0 {
        return Err(Error::invalid("v", "volumes must be > 0"));
    }
    // C integer division truncates toward zero, so 0 < t < 1 gives index 0.
    let idx = (((t as i64) - 1) / 5).min(14) as usize;
    let r = v / bv;
    let gf = ((sg - 1.0) * r + 1.0 - 1.05) * 5.0 + 1.0;
    if gf <= 0.0 {
        return Err(Error::invalid("sg", "gravity factor must be > 0"));
    }
    let mut prev = 1.0_f64;
    let mut cur = 50.0_f64;
    for _ in 0..GARETZ_MAX_ITER {
        if (prev - cur).abs() <= 0.05 {
            return finite_result("garetz_ibu", cur);
        }
        prev = cur;
        cur = GARETZ_TABLE[idx] * a * w * 0.749 / (v * gf * (r * cur / 260.0 + 1.0) * 1.02);
    }
    if (prev - cur).abs() <= 0.05 {
        return finite_result("garetz_ibu", cur);
    }
    Err(Error::NoConvergence { what: "garetz_ibu" })
}

/// Garetz utilisation in recipe form: `garetz_ibu(1, 1, t, SG, 1, 1) / 74.9` (spec §2.3).
/// `sg` specific gravity, `t_min` minutes. Ratio 0..1.
pub fn garetz_utilization(sg: f64, t_min: f64) -> Result<f64> {
    Ok(garetz_ibu(&GaretzIbuInput {
        w_oz: 1.0,
        a_pct: 1.0,
        t_min,
        sg,
        v_gal: 1.0,
        bv_gal: 1.0,
    })? / 74.9)
}

/// Extra utilisation of a boil hop during the whirlpool, `PostBoilUtil` (spec §3.5).
/// `sg` boil-end gravity, `t0_min` hop boil time (min), `dur_min` whirlpool minutes. Left
/// Riemann sum with step 0.1 min, accumulated exactly like the reference (the `tau < 5`
/// exception uses absolute hop time). Decision T1.15 B1: once the linear cooling model
/// reaches 0 K (`e >= 498.74` min) the Arrhenius factor takes its limit 0 and the sum
/// stops, so the result is finite and the loop bounded for any `dur_min` (also infinite).
#[must_use]
pub fn post_boil_utilization(sg: f64, t0_min: f64, dur_min: f64) -> f64 {
    let gravity = 1.65 * 0.000_125_f64.powf(sg - 1.0);
    let mut sum = 0.0;
    let mut tau = t0_min;
    for _ in 0..POST_BOIL_MAX_STEPS {
        if tau >= t0_min + dur_min {
            break;
        }
        let kelvin = ((tau - t0_min) * -1.344 + 210.64 + 459.67) * (5.0 / 9.0);
        let f = if tau < 5.0 {
            1.0
        } else if kelvin <= 0.0 {
            break;
        } else {
            2.39e11 * (-9773.0 / kelvin).exp()
        };
        sum += gravity * 0.04 * (-0.04 * tau).exp() / 4.15 * f * 0.1;
        tau += 0.1;
    }
    sum
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

/// Inputs of [`aged_alpha`].
#[derive(Debug, Clone, Copy)]
pub struct AgedAlphaInput {
    /// Alpha acid at harvest, %.
    pub alpha_pct: f64,
    /// Hop storage index, % lost in six months at 68 °F.
    pub hsi_pct: f64,
    /// Storage temperature, °F.
    pub temp_f: f64,
    /// Days of storage.
    pub days: f64,
    /// Storage factor (1.0 poly, 0.75 oxygen barrier, 0.5 vacuum).
    pub storage: f64,
}

/// Alpha acid after storage (spec §10): `A0 · exp(−k · TF · days · SF)` with
/// `k = −ln(1 − HSI/100)/180`, `TF = 0.5^((68 − T)/27)`. `alpha` %, `hsi` % lost in six
/// months at 68 °F (capped at 99), `temp_f` °F, `days` of storage, `storage` factor
/// (1.0 poly, 0.75 oxygen barrier, 0.5 vacuum). Returns alpha %.
/// Errors on non-finite input and on `hsi < 0` (the reference's `log` would be complex).
pub fn aged_alpha(p: &AgedAlphaInput) -> Result<f64> {
    let AgedAlphaInput {
        alpha_pct: alpha,
        hsi_pct: hsi,
        temp_f,
        days,
        storage,
    } = *p;
    for (field, x) in [
        ("alpha", alpha),
        ("hsi", hsi),
        ("temp_f", temp_f),
        ("days", days),
        ("storage", storage),
    ] {
        ensure_finite(field, x)?;
    }
    if hsi < 0.0 {
        return Err(Error::invalid("hsi", "must be >= 0"));
    }
    let hsi = hsi.min(99.0);
    let k = -(1.0 - hsi / 100.0).ln() / 180.0;
    finite_result(
        "aged_alpha",
        alpha * (-k * 0.5_f64.powf((68.0 - temp_f) / 27.0) * days * storage).exp(),
    )
}

// ---------------------------------------------------------------------------------------
// T1.14a: single-hop IBU, recipe-level engine, tools (spec §2–§10, previously unported).
// ---------------------------------------------------------------------------------------

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
    pub amounts_oz: Vec<f64>,
}

/// Result of the Hop Bitterness tool (spec §7): total IBU, total AAU, IBU per hop.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolBitterness {
    pub ibu: f64,
    pub aau: f64,
    pub contribs_ibu: Vec<f64>,
}

/// Inputs of [`tinseth_ibu`] and [`rager_ibu`].
#[derive(Debug, Clone, Copy)]
pub struct HopIbuInput {
    /// Hop weight, oz.
    pub w_oz: f64,
    /// Alpha acid, %.
    pub a_pct: f64,
    /// Time, minutes.
    pub t_min: f64,
    /// Boil gravity, SG.
    pub sg: f64,
    /// Final volume, gal.
    pub v_gal: f64,
}

/// Tinseth IBU of a single hop (spec §2.1): `U_T · A · W · K / V`. `w` oz, `a` %, `t` min,
/// `sg`, `v` gallons. Returns 0 for `t ≤ 0`, `w ≤ 0`, `a < 0` or `v = 0`; errors on
/// non-finite input.
#[allow(clippy::float_cmp)]
pub fn tinseth_ibu(p: &HopIbuInput) -> Result<f64> {
    let HopIbuInput {
        w_oz: w,
        a_pct: a,
        t_min: t,
        sg,
        v_gal: v,
    } = *p;
    finite_all(&[("w", w), ("a", a), ("t", t), ("sg", sg), ("v", v)])?;
    if t <= 0.0 || w <= 0.0 || a < 0.0 || v == 0.0 {
        return Ok(0.0);
    }
    finite_result("tinseth_ibu", tinseth_utilization(sg, t) * a * w * K / v)
}

fn finite_all(xs: &[(&'static str, f64)]) -> Result<()> {
    for &(field, x) in xs {
        ensure_finite(field, x)?;
    }
    Ok(())
}

/// Inputs of [`tinseth_weight`], [`rager_weight`] and [`garetz_weight`].
#[derive(Debug, Clone, Copy)]
pub struct HopWeightInput {
    /// Target bitterness, IBU.
    pub ibu: f64,
    /// Final volume, gal.
    pub v_gal: f64,
    /// Boil gravity, SG.
    pub sg: f64,
    /// Time, minutes.
    pub t_min: f64,
    /// Alpha acid, %.
    pub a_pct: f64,
}

/// Tinseth inverse (spec §2.1): ounces needed for `ibu` in `v` gallons,
/// `IBU · V / (U_T · A · K)`. Errors (T1.16, was 0) when the divisor is 0 (`Unreachable`) or
/// an input is non-finite.
#[allow(clippy::float_cmp)]
pub fn tinseth_weight(p: &HopWeightInput) -> Result<f64> {
    let HopWeightInput {
        ibu,
        v_gal: v,
        sg,
        t_min: t,
        a_pct: a,
    } = *p;
    finite_all(&[("ibu", ibu), ("v", v), ("sg", sg), ("t", t), ("a", a)])?;
    let d = tinseth_utilization(sg, t) * a * K;
    if d == 0.0 {
        Err(Error::Unreachable {
            what: "tinseth_weight target",
        })
    } else {
        finite_result("tinseth_weight", ibu * v / d)
    }
}

/// Rager IBU of a single hop (spec §2.2): `R(n) · A · W · K / ((1 + GA) · V)`, `w` oz,
/// `a` %, `t` min, `v` gallons. Same preconditions as [`tinseth_ibu`].
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
pub fn rager_ibu(p: &HopIbuInput) -> Result<f64> {
    let HopIbuInput {
        w_oz: w,
        a_pct: a,
        t_min: t,
        sg,
        v_gal: v,
    } = *p;
    finite_all(&[("w", w), ("a", a), ("t", t), ("sg", sg), ("v", v)])?;
    if t <= 0.0 || w <= 0.0 || a < 0.0 || v == 0.0 {
        return Ok(0.0);
    }
    finite_result(
        "rager_ibu",
        rager_table((t + 0.5) as i64) * a * w * K / ((1.0 + rager_gravity_adjustment(sg)) * v),
    )
}

/// Rager inverse (spec §2.2): `(1 + GA) · V · IBU / (R · A · K)` in oz; 0 when the divisor
/// is 0 (`Unreachable`, T1.16; was 0) or an input is non-finite.
#[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
pub fn rager_weight(p: &HopWeightInput) -> Result<f64> {
    let HopWeightInput {
        ibu,
        v_gal: v,
        sg,
        t_min: t,
        a_pct: a,
    } = *p;
    finite_all(&[("ibu", ibu), ("v", v), ("sg", sg), ("t", t), ("a", a)])?;
    let d = rager_table((t + 0.5) as i64) * a * K;
    if d == 0.0 {
        Err(Error::Unreachable {
            what: "rager_weight target",
        })
    } else {
        finite_result(
            "rager_weight",
            (1.0 + rager_gravity_adjustment(sg)) * v * ibu / d,
        )
    }
}

/// Garetz inverse by bisection (spec §2.3): ounces for target `ibu` (`a` %, `t` min, `sg`,
/// `v` gallons, boil volume = `v`). Range [0, 1e6], start 500, stop at `|IBU − target| ≤ 0.05`
/// (capped at 200 steps, choice). Unreachable target (utilisation table 0, `t ≤ 10`, or the
/// bisection does not reach it): `Unreachable` (T1.16; was 0).
#[allow(clippy::manual_midpoint, clippy::many_single_char_names)]
pub fn garetz_weight(p: &HopWeightInput) -> Result<f64> {
    let HopWeightInput {
        ibu,
        a_pct: a,
        t_min: t,
        sg,
        v_gal: v,
    } = *p;
    finite_all(&[("ibu", ibu), ("a", a), ("t", t), ("sg", sg), ("v", v)])?;
    let (mut lo, mut hi, mut w) = (0.0_f64, 1e6_f64, 500.0_f64);
    for _ in 0..200 {
        let cur = match garetz_ibu(&GaretzIbuInput {
            w_oz: w,
            a_pct: a,
            t_min: t,
            sg,
            v_gal: v,
            bv_gal: v,
        }) {
            Ok(x) => x,
            Err(Error::NoConvergence { .. }) => f64::INFINITY,
            Err(e) => return Err(e),
        };
        if (cur - ibu).abs() <= 0.05 {
            return Ok(w);
        }
        if cur < ibu {
            lo = w;
        } else {
            hi = w;
        }
        w = (lo + hi) / 2.0;
    }
    Err(Error::Unreachable {
        what: "garetz_weight target",
    })
}

/// Whirlpool utilisation `U_T(SG, t) · Ftemp(T)` (spec §3.4); `t_min` steep minutes, `temp_c` °C.
#[must_use]
pub fn whirlpool_utilization(sg: f64, t_min: f64, temp_c: f64) -> f64 {
    tinseth_utilization(sg, t_min) * temperature_factor(temp_c)
}

/// Inputs of [`whirlpool_ibu`].
#[derive(Debug, Clone, Copy)]
pub struct WhirlpoolIbuInput {
    /// Hop weight, oz.
    pub w_oz: f64,
    /// Alpha acid, %.
    pub a_pct: f64,
    /// Time, minutes.
    pub t_min: f64,
    /// Boil gravity, SG.
    pub sg: f64,
    /// Final volume, gal.
    pub v_gal: f64,
    /// Steep temperature, °C.
    pub temp_c: f64,
}

/// Whirlpool hop IBU (spec §3.4): `U_W · A · W · K / V`, `w` oz, `a` %, `v` gallons,
/// `temp_c` °C. 0 when `v = 0`.
#[allow(clippy::float_cmp)]
pub fn whirlpool_ibu(p: &WhirlpoolIbuInput) -> Result<f64> {
    let WhirlpoolIbuInput {
        w_oz: w,
        a_pct: a,
        t_min: t,
        sg,
        v_gal: v,
        temp_c,
    } = *p;
    finite_all(&[
        ("w", w),
        ("a", a),
        ("t", t),
        ("sg", sg),
        ("v", v),
        ("temp_c", temp_c),
    ])?;
    if v == 0.0 {
        return Ok(0.0);
    }
    finite_result(
        "whirlpool_ibu",
        whirlpool_utilization(sg, t, temp_c) * a * w * K / v,
    )
}

/// Hop equation selection (spec §1): the per-recipe setting is a code where any value outside
/// 0..=2 is BeerSmith's "use the global setting" sentinel (spec-defined, not an error); the
/// global setting itself is a typed [`HopEquation`].
#[must_use]
pub fn resolve_equation(equation: i64, global_equation: HopEquation) -> HopEquation {
    HopEquation::from_code(equation).unwrap_or(global_equation)
}

/// Inputs of [`hop_utilization`].
#[derive(Debug, Clone, Copy)]
pub struct HopUtilizationInput {
    /// Boil gravity, SG.
    pub sg: f64,
    /// Segment time, minutes.
    pub t_min: f64,
    /// Bitterness equation.
    pub equation: HopEquation,
    /// Force the Tinseth utilisation (whirlpool).
    pub whirlpool_mu: bool,
    /// Wort temperature, °F (boil point for boil hops).
    pub temp_f: f64,
}

/// Segment utilisation `HopUtil` (spec §3.3). `equation` 0 Tinseth / 1 Rager / 2 Garetz;
/// `whirlpool_mu` forces Tinseth; `temp_f` is the wort temperature in °F (boil point for
/// boil hops). Result is a ratio 0..1.
pub fn hop_utilization(p: &HopUtilizationInput) -> Result<f64> {
    let HopUtilizationInput {
        sg,
        t_min: t,
        equation,
        whirlpool_mu,
        temp_f,
    } = *p;
    finite_all(&[("sg", sg), ("t", t), ("temp_f", temp_f)])?;
    let u = if whirlpool_mu {
        tinseth_utilization(sg, t)
    } else {
        match equation {
            HopEquation::Rager => rager_utilization(sg, t),
            HopEquation::Garetz => garetz_utilization(sg, t)?,
            HopEquation::Tinseth => tinseth_utilization(sg, t),
        }
    };
    Ok(u * temperature_factor(f_to_c(temp_f)))
}

/// Inputs of [`hop_adjustments`].
#[derive(Debug, Clone, Copy)]
pub struct HopAdjustmentsInput<'a> {
    /// Hop form.
    pub form: HopForm,
    /// Hop usage.
    pub usage: HopUse,
    /// Equipment hop utilisation, %.
    pub util_pct: f64,
    /// True when the recipe type is Extract.
    pub extract_recipe: bool,
    /// Hop correction options.
    pub opts: &'a HopOptions,
}

/// Hop correction multiplier `HopAdjustments` (spec §4). `util_pct` is the equipment hop
/// utilisation in %, `extract_recipe` is `recipe type == Extract`. Form 4 gives 1, dry hop 0.
#[must_use]
pub fn hop_adjustments(p: &HopAdjustmentsInput<'_>) -> f64 {
    let HopAdjustmentsInput {
        form,
        usage,
        util_pct,
        extract_recipe,
        opts,
    } = *p;
    if form == HopForm::IsomerizedExtract {
        return 1.0;
    }
    if usage == HopUse::DryHop {
        return 0.0;
    }
    let mut m = 1.0;
    if usage == HopUse::FirstWort && !extract_recipe {
        m *= (100.0 + opts.first_wort_adj_pct) / 100.0;
    }
    if opts.type_adj {
        match form {
            HopForm::Leaf => m *= (100.0 + opts.leaf_adj_pct) / 100.0,
            HopForm::Plug => m *= (100.0 + opts.plug_adj_pct) / 100.0,
            HopForm::Pellet => m *= (100.0 + opts.pellet_adj_pct) / 100.0,
            _ => {}
        }
    }
    if usage == HopUse::Mash && !extract_recipe {
        m *= (100.0 + opts.mash_adj_pct) / 100.0;
    }
    util_pct / 100.0 * m
}

/// Total utilisation of one hop `HopTotalUtil` (spec §5.2). `profile` comes from
/// [`late_extract_profile`] (spec 01 §3.6, the single implementation); ratio 0..1.
#[allow(clippy::float_cmp)]
pub fn hop_total_utilization(
    hop: &Hop,
    recipe: &RecipeInput,
    profile: &[LateExtractPoint],
    equation: HopEquation,
) -> Result<f64> {
    if hop.form == HopForm::IsomerizedExtract {
        return Ok(1.0);
    }
    if hop.amount_oz <= 0.0 || hop.usage == HopUse::DryHop {
        return Ok(0.0);
    }
    let (Some(first), Some(last)) = (profile.first(), profile.last()) else {
        return Ok(0.0);
    };
    let th = hop.time_min;
    if hop.usage == HopUse::Whirlpool {
        return Ok(
            tinseth_utilization(last.sg, th) * temperature_factor(f_to_c(hop.whirlpool_temp_f))
        );
    }
    let tbf = boil_temp_f(recipe.equipment.altitude_ft);
    let mut segs: Vec<f64> = Vec::new();
    if first.time_min < th {
        segs.push(
            hop_utilization(&HopUtilizationInput {
                sg: first.sg,
                t_min: th,
                equation,
                whirlpool_mu: false,
                temp_f: tbf,
            })? - hop_utilization(&HopUtilizationInput {
                sg: first.sg,
                t_min: first.time_min,
                equation,
                whirlpool_mu: false,
                temp_f: tbf,
            })?,
        );
    }
    let mut i = 0;
    while let Some(&LateExtractPoint { sg, time_min: t_i }) = profile.get(i) {
        if t_i == 0.0 {
            break;
        }
        let Some(&LateExtractPoint {
            time_min: t_next, ..
        }) = profile.get(i + 1)
        else {
            break;
        };
        if t_next <= th {
            let start = t_i.min(th);
            segs.push(
                hop_utilization(&HopUtilizationInput {
                    sg,
                    t_min: start,
                    equation,
                    whirlpool_mu: false,
                    temp_f: tbf,
                })? - hop_utilization(&HopUtilizationInput {
                    sg,
                    t_min: t_next,
                    equation,
                    whirlpool_mu: false,
                    temp_f: tbf,
                })?,
            );
        }
        i += 1;
    }
    if recipe.equipment.whirlpool_carry {
        segs.push(post_boil_utilization(
            last.sg,
            th,
            recipe.equipment.whirlpool_time_min,
        ));
    }
    Ok(combine_utilizations(&segs))
}

/// IBU of one hop in a recipe, `NewHopIBU` (spec §5.3):
/// `F_adj · W · U · A · 100 · 128 / (V_ibu · 1.34)` with `V_ibu = volume + trub` (fl oz,
/// 0.01 when ≤ 0). Form 4 amounts are fl oz (density 1). Hop options come from
/// `recipe.options.hop`.
pub fn new_hop_ibu(
    hop: &Hop,
    recipe: &RecipeInput,
    profile: &[LateExtractPoint],
    equation: HopEquation,
) -> Result<f64> {
    let mut vibu = recipe.volume_floz + recipe.equipment.trub_loss_floz;
    if vibu <= 0.0 {
        vibu = 0.01;
    }
    let u = hop_total_utilization(hop, recipe, profile, equation)?;
    let adj = hop_adjustments(&HopAdjustmentsInput {
        form: hop.form,
        usage: hop.usage,
        util_pct: recipe.equipment.hop_util_pct,
        extract_recipe: recipe.kind == RecipeType::Extract,
        opts: &recipe.options.hop,
    });
    finite_result(
        "new_hop_ibu",
        adj * hop.amount_oz * u * hop.alpha_pct * 100.0 * 128.0 / (vibu * 1.34),
    )
}

/// IBU of a hopped-extract fermentable (spec §5.4): `oz/16 · IBU·gal/lb / max((V+T)/128,
/// 0.0001)`. `volume_floz` and `trub_floz` in fl oz.
#[must_use]
pub fn hopped_extract_ibu(
    amount_oz: f64,
    ibu_gal_per_lb: f64,
    volume_floz: f64,
    trub_floz: f64,
) -> f64 {
    (amount_oz / 16.0 * ibu_gal_per_lb) / ((volume_floz + trub_floz) / 128.0).max(0.0001)
}

fn extract_total_ibu(recipe: &RecipeInput) -> f64 {
    let mut e = 0.0;
    for g in &recipe.fermentables {
        if g.kind == GrainType::Extract && g.ibu_gal_per_lb > 0.0 {
            e += hopped_extract_ibu(
                g.amount_oz,
                g.ibu_gal_per_lb,
                recipe.volume_floz,
                recipe.equipment.trub_loss_floz,
            );
        }
    }
    e
}

/// Recipe IBU `NewGetEstIBU` (spec §5.4): sum of hop IBUs plus hopped extracts of
/// `recipe.hops` and `recipe.fermentables`. `equation` 0 Tinseth / 1 Rager / 2 Garetz (the
/// `RecipeIBU*` variants), `v_boil_floz` fl oz.
pub fn recipe_ibu(recipe: &RecipeInput, equation: HopEquation, v_boil_floz: f64) -> Result<f64> {
    ensure_finite("v_boil_floz", v_boil_floz)?;
    let profile = late_extract_profile(recipe, v_boil_floz);
    let mut tot = 0.0;
    for h in &recipe.hops {
        tot += new_hop_ibu(h, recipe, &profile, equation)?;
    }
    finite_result("recipe_ibu", tot + extract_total_ibu(recipe))
}

/// Hop share `W · 100 / total` in % (spec §5.4); 0 when the total is ≤ 0 (choice).
#[must_use]
pub fn hop_percent(w_oz: f64, tot_hops_oz: f64) -> f64 {
    if tot_hops_oz <= 0.0 {
        0.0
    } else {
        w_oz * 100.0 / tot_hops_oz
    }
}

/// Inputs of [`scale_bitterness`].
#[derive(Debug, Clone, Copy)]
pub struct ScaleBitternessInput<'a> {
    /// Current recipe IBU.
    pub current_ibu: f64,
    /// Target IBU.
    pub target_ibu: f64,
    /// The recipe.
    pub recipe: &'a RecipeInput,
    /// Bitterness equation.
    pub equation: HopEquation,
    /// Boil volume, fl oz.
    pub v_boil_floz: f64,
}

/// Scale all non-dry hops to a target IBU, `ScaleBitterness` (spec §6.1). `current_ibu` is
/// `F_R_IBU`. Dry hops (any form) are neither counted nor scaled (the index-shift bug is
/// not reproduced). "About zero" means below 1e-9.
pub fn scale_bitterness(p: &ScaleBitternessInput<'_>) -> Result<ScaleResult> {
    let ScaleBitternessInput {
        current_ibu,
        target_ibu: target,
        recipe,
        equation,
        v_boil_floz: v_boil,
    } = *p;
    finite_all(&[
        ("current_ibu", current_ibu),
        ("target", target),
        ("v_boil", v_boil),
    ])?;
    let hops = &recipe.hops;
    let mut amounts: Vec<f64> = hops.iter().map(|h| h.amount_oz).collect();
    if (target - current_ibu).abs() < 1e-9 {
        return Ok(ScaleResult {
            status: ScaleStatus::Unchanged,
            amounts_oz: amounts,
        });
    }
    if current_ibu.abs() < 1e-9 || target.abs() < 1e-9 {
        return Ok(ScaleResult {
            status: ScaleStatus::Failed,
            amounts_oz: amounts,
        });
    }
    let mut rc = recipe.clone();
    if rc.volume_floz == 0.0 {
        rc.volume_floz = 0.0001;
    }
    let profile = late_extract_profile(&rc, v_boil);
    let mut ibus: Vec<f64> = Vec::with_capacity(hops.len());
    for h in hops {
        if h.usage == HopUse::DryHop && h.form != HopForm::IsomerizedExtract {
            ibus.push(0.0);
            continue;
        }
        let i = new_hop_ibu(h, &rc, &profile, equation)?;
        ibus.push(if i < 1e-9 { 0.0 } else { i });
    }
    let e = extract_total_ibu(&rc);
    let r2 = if e < current_ibu {
        current_ibu - e
    } else {
        current_ibu
    };
    let t2 = (target - e).max(0.0);
    for (k, h) in hops.iter().enumerate() {
        if (h.usage == HopUse::DryHop && h.form != HopForm::IsomerizedExtract) || ibus[k] == 0.0 {
            continue;
        }
        let u = hop_total_utilization(h, &rc, &profile, equation)?;
        if h.alpha_pct * u <= 0.0 {
            continue;
        }
        let adj = hop_adjustments(&HopAdjustmentsInput {
            form: h.form,
            usage: h.usage,
            util_pct: rc.equipment.hop_util_pct,
            extract_recipe: rc.kind == RecipeType::Extract,
            opts: &rc.options.hop,
        });
        amounts[k] =
            (ibus[k] / adj) * (t2 / r2) * (rc.volume_floz + rc.equipment.trub_loss_floz) * 1.34
                / (h.alpha_pct * u * 100.0 * 128.0);
    }
    if amounts.iter().any(|a| !a.is_finite()) {
        return Err(Error::invalid("amounts", "scaled amount is not finite"));
    }
    Ok(ScaleResult {
        status: ScaleStatus::Ok,
        amounts_oz: amounts,
    })
}

/// New hop amount for an edited IBU, `HopIBUAdjust` (spec §6.2): `W_old / IBU_old · IBU_new`;
/// error (T1.16; was 0) when `ibu_old ≤ 0` or an input is non-finite.
pub fn hop_ibu_adjust(w_old_oz: f64, ibu_old: f64, ibu_new: f64) -> Result<f64> {
    finite_all(&[
        ("w_old_oz", w_old_oz),
        ("ibu_old", ibu_old),
        ("ibu_new", ibu_new),
    ])?;
    if ibu_old <= 0.0 {
        Err(Error::invalid("ibu_old", "must be > 0"))
    } else {
        Ok(w_old_oz / ibu_old * ibu_new)
    }
}

/// Inputs of [`tool_hop_ibu`].
#[derive(Debug, Clone, Copy)]
pub struct ToolHopIbuInput<'a> {
    /// The hop.
    pub hop: &'a Hop,
    /// Boil gravity, SG.
    pub sg: f64,
    /// Batch volume, fl oz.
    pub v_floz: f64,
    /// Bitterness equation.
    pub equation: HopEquation,
    /// Equipment hop utilisation, %.
    pub util_pct: f64,
    /// Hop correction options.
    pub opts: &'a HopOptions,
}

/// Hop Bitterness tool, `CalcIBU` (spec §7). `sg` boil gravity, `v_floz` batch volume in fl
/// oz, `util_pct` the tool's hop utilisation in %. Form 4 gives `W · A · 100 · 128 / (V ·
/// 1.34)` (0 when `v_floz ≤ 0`, choice). Recipe type is never Extract in the tool.
pub fn tool_hop_ibu(p: &ToolHopIbuInput<'_>) -> Result<f64> {
    let ToolHopIbuInput {
        hop,
        sg,
        v_floz,
        equation,
        util_pct,
        opts,
    } = *p;
    finite_all(&[("sg", sg), ("v_floz", v_floz), ("util_pct", util_pct)])?;
    let (w, a, t) = (hop.amount_oz, hop.alpha_pct, hop.time_min);
    if hop.form == HopForm::IsomerizedExtract {
        return Ok(if v_floz <= 0.0 {
            0.0
        } else {
            w * a * 100.0 * 128.0 / (v_floz * 1.34)
        });
    }
    if w <= 0.0 || hop.usage == HopUse::DryHop || v_floz <= 0.0 {
        return Ok(0.0);
    }
    let vg = v_floz / 128.0;
    let ibu = if hop.usage == HopUse::Whirlpool {
        whirlpool_ibu(&WhirlpoolIbuInput {
            w_oz: w,
            a_pct: a,
            t_min: t,
            sg,
            v_gal: vg,
            temp_c: f_to_c(hop.whirlpool_temp_f),
        })?
    } else {
        match equation {
            HopEquation::Rager => rager_ibu(&HopIbuInput {
                w_oz: w,
                a_pct: a,
                t_min: t,
                sg,
                v_gal: vg,
            })?,
            HopEquation::Garetz => garetz_ibu(&GaretzIbuInput {
                w_oz: w,
                a_pct: a,
                t_min: t,
                sg,
                v_gal: vg,
                bv_gal: vg,
            })?,
            HopEquation::Tinseth => tinseth_ibu(&HopIbuInput {
                w_oz: w,
                a_pct: a,
                t_min: t,
                sg,
                v_gal: vg,
            })?,
        }
    };
    Ok(ibu
        * hop_adjustments(&HopAdjustmentsInput {
            form: hop.form,
            usage: hop.usage,
            util_pct,
            extract_recipe: false,
            opts,
        }))
}

/// AAU / HBU `CalcAAU` (spec §7.1): `W · A` for boil or first-wort hops of at least
/// 15 minutes, else 0. `w_oz` oz, `a_pct` %, `t_min` minutes.
#[must_use]
pub fn hop_aau(w_oz: f64, a_pct: f64, usage: HopUse, t_min: f64) -> f64 {
    if matches!(usage, HopUse::Boil | HopUse::FirstWort) && t_min >= 15.0 {
        w_oz * a_pct
    } else {
        0.0
    }
}

/// Inputs of [`tool_bitterness`].
#[derive(Debug, Clone, Copy)]
pub struct ToolBitternessInput<'a> {
    /// The hops.
    pub hops: &'a [Hop],
    /// Boil gravity, SG.
    pub sg: f64,
    /// Batch volume, fl oz.
    pub v_floz: f64,
    /// Bitterness equation.
    pub equation: HopEquation,
    /// Equipment hop utilisation, %.
    pub util_pct: f64,
    /// Hop correction options.
    pub opts: &'a HopOptions,
}

/// Hop Bitterness tool totals (spec §7): summed IBU, summed AAU and the per-hop IBUs.
pub fn tool_bitterness(p: &ToolBitternessInput<'_>) -> Result<ToolBitterness> {
    let ToolBitternessInput {
        hops,
        sg,
        v_floz,
        equation,
        util_pct,
        opts,
    } = *p;
    let contribs: Vec<f64> = hops
        .iter()
        .map(|h| {
            tool_hop_ibu(&ToolHopIbuInput {
                hop: h,
                sg,
                v_floz,
                equation,
                util_pct,
                opts,
            })
        })
        .collect::<Result<_>>()?;
    Ok(ToolBitterness {
        ibu: contribs.iter().sum(),
        aau: hops
            .iter()
            .map(|h| hop_aau(h.amount_oz, h.alpha_pct, h.usage, h.time_min))
            .sum(),
        contribs_ibu: contribs,
    })
}

/// Total hop amount of a phase, `RecipeTotHopsPhase` (spec §8): hops with `usage == phase`,
/// plus first-wort hops (3) when `phase == 0`. Oz as stored.
#[must_use]
pub fn total_hops_phase(hops: &[Hop], phase: HopUse) -> f64 {
    hops.iter()
        .filter(|h| h.usage == phase || (phase == HopUse::Boil && h.usage == HopUse::FirstWort))
        .map(|h| h.amount_oz)
        .sum()
}

/// Hop rate in g/L (spec §8): phase total (oz → g) over the batch volume (`volume_floz` in
/// fl oz, no trub). 0 when the volume is 0.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn hop_rate(hops: &[Hop], volume_floz: f64, phase: HopUse) -> f64 {
    if volume_floz == 0.0 {
        return 0.0;
    }
    total_hops_phase(hops, phase) * OZ_G / (volume_floz * FLOZ_L)
}

/// Total whirlpool time in whole minutes, `TotWhirlpoolTime` (spec §9): the larger of
/// `recipe_time_min` and `int(t + 0.48)` over whirlpool hops (usage 4) and whirlpool misc
/// items (usage 6).
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn total_whirlpool_time(recipe_time_min: i64, hops: &[Hop], misc: &[Misc]) -> i64 {
    let mut m = recipe_time_min;
    for h in hops.iter().filter(|h| h.usage == HopUse::Whirlpool) {
        m = m.max((h.time_min + 0.48) as i64);
    }
    for x in misc.iter().filter(|x| x.usage == MiscUse::Whirlpool) {
        m = m.max((x.time_min + 0.48) as i64);
    }
    m
}

/// Inputs of [`aged_alpha_months`].
#[derive(Debug, Clone, Copy)]
pub struct AgedAlphaMonthsInput {
    /// Alpha acid at harvest, %.
    pub alpha_pct: f64,
    /// Hop storage index, % lost in six months at 68 °F.
    pub hsi_pct: f64,
    /// Storage temperature, °F.
    pub temp_f: f64,
    /// Months of storage.
    pub months: f64,
    /// Storage type.
    pub storage_type: StorageType,
}

/// Alpha after storage from months and storage type, `HopAlphaFunc` (spec §10): days =
/// `months · 30`; [`StorageType`] gives the factor (poly 1.0, oxygen barrier 0.75, vacuum
/// 0.5). Returns alpha %.
pub fn aged_alpha_months(p: &AgedAlphaMonthsInput) -> Result<f64> {
    let AgedAlphaMonthsInput {
        alpha_pct: alpha,
        hsi_pct: hsi,
        temp_f,
        months,
        storage_type,
    } = *p;
    aged_alpha(&AgedAlphaInput {
        alpha_pct: alpha,
        hsi_pct: hsi,
        temp_f,
        days: months * 30.0,
        storage: storage_type.factor(),
    })
}
