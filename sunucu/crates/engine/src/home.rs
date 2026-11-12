//! Section 07: home MVP extra calculations (`formuller/07-ev-mvp-ek-hesaplar.md`).
//!
//! Units follow the specification: °C, L, kg, g, minutes, hours; gravity as SG; water
//! ions in mg/L. Water density is taken as 1 kg/L. The Python reference
//! (`formuller/testler/07_ev_mvp_ek_tv.py`) is the source of truth.
//!
//! The fermentation model ([`hourly_median`], [`fit_decay`], [`forecast`]) is public
//! because sections 10 (`manual`) and 11 (`trigger`) reuse it.

// Short names mirror the reference and the spec formulas; midpoint kept as `(a + b) / 2` for bit parity.
#![allow(
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::manual_midpoint
)]

use std::collections::BTreeMap;

use crate::error::{Error, Result, ensure_finite, ensure_positive, finite_result};
use crate::units::{gravity_units_checked, py_sum};

// ------------------------------------------------------------------ §1 fermentation forecast

/// Hourly median of `(t_hours, SG)` readings (§1.1).
///
/// Buckets by `trunc(t)`; each output point is `(hour + 0.5, median SG)`, sorted by hour.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn hourly_median(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut buckets: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    for &(t, g) in points {
        buckets.entry(t.trunc() as i64).or_default().push(g);
    }
    buckets
        .into_iter()
        .map(|(h, mut v)| {
            v.sort_by(f64::total_cmp);
            let n = v.len();
            let med = if n % 2 == 1 {
                v[n / 2]
            } else {
                (v[n / 2 - 1] + v[n / 2]) / 2.0
            };
            #[allow(clippy::cast_precision_loss)]
            let hour = h as f64;
            (hour + 0.5, med)
        })
        .collect()
}

/// Least-squares line `y = a + b·x`; returns `(a, b)`.
fn linfit(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    #[allow(clippy::cast_precision_loss)]
    let n = xs.len() as f64;
    let mx = py_sum(xs.iter().copied()) / n;
    let my = py_sum(ys.iter().copied()) / n;
    let sxx = py_sum(xs.iter().map(|x| (x - mx).powi(2)));
    let sxy = py_sum(xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)));
    let b = sxy / sxx;
    (my - b * mx, b)
}

/// Best fit of `SG(t) = fg_inf + a·e^(−k·t)` (§1.2–1.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecayFit {
    /// Estimated final gravity `FG∞` (SG).
    pub fg_inf: f64,
    /// Scale `A` (SG).
    pub a: f64,
    /// Rate `k` (1/hour).
    pub k: f64,
    /// Residual sum of squares of the fit (SG²).
    pub rss: f64,
}

/// Grid-search fit of `SG(t) = FG∞ + A·e^(−k·t)` (§1.3).
///
/// `pts` are `(t_hours, SG)`. `FG∞` is scanned over `steps + 1` grid points in
/// `[fg_lo, fg_hi]`; for each, `ln(SG − FG∞)` is regressed on `t`. Grid points where any
/// `SG − FG∞ ≤ 0` or `k ≤ 0` are skipped. Returns the best fit and the table
/// `(FG∞, RSS)` of all valid grid points, or `None` when no grid point is valid.
#[must_use]
pub fn fit_decay(
    pts: &[(f64, f64)],
    fg_lo: f64,
    fg_hi: f64,
    steps: usize,
) -> Option<(DecayFit, Vec<(f64, f64)>)> {
    let ts: Vec<f64> = pts.iter().map(|p| p.0).collect();
    let mut best: Option<DecayFit> = None;
    let mut table = Vec::new();
    for i in 0..=steps {
        #[allow(clippy::cast_precision_loss)]
        let f = fg_lo + (fg_hi - fg_lo) * i as f64 / steps as f64;
        if pts.iter().any(|&(_, g)| g - f <= 0.0) {
            continue;
        }
        let ys: Vec<f64> = pts.iter().map(|&(_, g)| (g - f).ln()).collect();
        let (a, b) = linfit(&ts, &ys);
        let k = -b;
        // T1.12 C5: non-finite or non-positive `k` never yields a grid point.
        if k.is_nan() || k <= 0.0 {
            continue;
        }
        let amp = a.exp();
        let rss = py_sum(
            pts.iter()
                .map(|&(t, g)| (g - (f + amp * (-k * t).exp())).powi(2)),
        );
        if !rss.is_finite() {
            continue;
        }
        table.push((f, rss));
        if best.is_none_or(|b| rss < b.rss) {
            best = Some(DecayFit {
                fg_inf: f,
                a: amp,
                k,
                rss,
            });
        }
    }
    best.map(|b| (b, table))
}

/// Forecast values (§1.5), unrounded (T1.12 C9): rounding is a presentation concern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForecastResult {
    /// Final gravity `FG∞` (SG).
    pub fg: f64,
    /// Approximate 95 % profile interval `(low, high)` of `FG∞` (SG).
    pub range: (f64, f64),
    /// Rate `k` (1/hour).
    pub k_per_hour: f64,
    /// Finish time since batch start (hours): when `SG − FG∞ ≤ eps`.
    pub finish_hour: f64,
    /// Remaining time from the last point (hours).
    pub remaining_hours: f64,
    /// `|u(FG∞) − u(FG_recipe)| > 1000` ([`fg_warning`]): finishing differently than expected.
    pub warning: bool,
}

/// Outcome of [`forecast`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Forecast {
    /// "Veri yetersiz": fewer than `min_n` hourly points (or `min_n < 3`), drop below
    /// `min_drop`, or no valid fit grid point (§1.4, T1.12 C5).
    InsufficientData,
    /// A forecast could be made.
    Ready(ForecastResult),
}

/// §1.4 drop rule in integer gravity units (T1.12 C3): enough of the expected drop is done
/// unless `Δu(OG − SG_now) < min_drop · Δu(OG − FG_recipe) − 1e-9`. An exact 30 % drop passes.
///
/// Errors on non-finite input (T1.16, was silently 0 units).
#[allow(clippy::cast_precision_loss)]
pub fn drop_enough(og: f64, sg_now: f64, fg_recipe: f64, min_drop: f64) -> Result<bool> {
    ensure_finite("min_drop", min_drop)?;
    let done = (gravity_units_checked(og)? - gravity_units_checked(sg_now)?) as f64;
    let expected = (gravity_units_checked(og)? - gravity_units_checked(fg_recipe)?) as f64;
    Ok(done >= min_drop * expected - 1e-9)
}

/// §1.5 warning in integer gravity units (T1.12 C3): `|u(FG∞) − u(FG_recipe)| > 1000`
/// (0.010 SG); exactly 0.010 does not warn.
/// Errors on non-finite input.
pub fn fg_warning(fg_inf: f64, fg_recipe: f64) -> Result<bool> {
    Ok((gravity_units_checked(fg_inf)? - gravity_units_checked(fg_recipe)?).abs() > 1000)
}

/// Fermentation forecast from raw `(t_hours, SG)` readings (§1.4–1.5).
///
/// `og`, `fg_recipe` in SG; `eps` is the SG distance to `FG∞` that counts as finished
/// (spec 0.0005); `min_n` minimum hourly points (spec 12, must be ≥ 3); `min_drop` minimum
/// share of the expected drop already done (spec 0.30). Errors on non-finite input.
pub fn forecast(
    points: &[(f64, f64)],
    og: f64,
    fg_recipe: f64,
    eps: f64,
    min_n: usize,
    min_drop: f64,
) -> Result<Forecast> {
    ensure_finite("og", og)?;
    ensure_finite("fg_recipe", fg_recipe)?;
    ensure_positive("eps", eps)?;
    ensure_finite("min_drop", min_drop)?;
    for &(t, g) in points {
        ensure_finite("t", t)?;
        ensure_finite("sg", g)?;
    }
    Ok(forecast_checked(
        points, og, fg_recipe, eps, min_n, min_drop,
    ))
}

fn forecast_checked(
    points: &[(f64, f64)],
    og: f64,
    fg_recipe: f64,
    eps: f64,
    min_n: usize,
    min_drop: f64,
) -> Forecast {
    if min_n < 3 {
        return Forecast::InsufficientData;
    }
    let pts = hourly_median(points);
    let Some(&(t_last, sg_now)) = pts.last() else {
        return Forecast::InsufficientData;
    };
    if pts.len() < min_n || !drop_enough(og, sg_now, fg_recipe, min_drop).unwrap_or(false) {
        return Forecast::InsufficientData;
    }
    let (fg_lo, fg_hi) = (fg_recipe - 0.015, sg_now - 0.0002);
    let Some((fit, table)) = fit_decay(&pts, fg_lo, fg_hi, 400) else {
        return Forecast::InsufficientData;
    };
    #[allow(clippy::cast_precision_loss)]
    let n = pts.len() as f64;
    let lim = fit.rss * (1.0 + 4.0 / (n - 2.0));
    let ok = table.iter().filter(|&&(_, s)| s <= lim).map(|&(x, _)| x);
    let (lo, hi) = ok.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
        (lo.min(x), hi.max(x))
    });
    let t_end = (fit.a / eps).ln() / fit.k;
    Forecast::Ready(ForecastResult {
        fg: fit.fg_inf,
        range: (lo, hi),
        k_per_hour: fit.k,
        finish_hour: t_end,
        remaining_hours: t_end - t_last,
        warning: fg_warning(fit.fg_inf, fg_recipe).unwrap_or(false),
    })
}

// ------------------------------------------------------------------ §2 device calibration

/// Least-squares polynomial `c0 + c1·x + c2·x² + …` of degree `deg` via normal equations
/// with partial-pivot Gauss elimination (§2.1). Returns coefficients, lowest order first.
#[must_use]
pub fn polyfit(xs: &[f64], ys: &[f64], deg: usize) -> Vec<f64> {
    let m = deg + 1;
    #[allow(clippy::cast_precision_loss)]
    let pw = |x: f64, e: usize| x.powf(e as f64);
    let mut a: Vec<Vec<f64>> = (0..m)
        .map(|i| {
            (0..m)
                .map(|j| py_sum(xs.iter().map(|&x| pw(x, i + j))))
                .collect()
        })
        .collect();
    let mut b: Vec<f64> = (0..m)
        .map(|i| py_sum(xs.iter().zip(ys).map(|(&x, &y)| y * pw(x, i))))
        .collect();
    for c in 0..m {
        let mut p = c;
        for r in c + 1..m {
            if a[r][c].abs() > a[p][c].abs() {
                p = r;
            }
        }
        a.swap(c, p);
        b.swap(c, p);
        for r in c + 1..m {
            let f = a[r][c] / a[c][c];
            for j in c..m {
                let v = a[c][j];
                a[r][j] -= f * v;
            }
            let bc = b[c];
            b[r] -= f * bc;
        }
    }
    let mut co = vec![0.0; m];
    for i in (0..m).rev() {
        let s = py_sum((i + 1..m).map(|j| a[i][j] * co[j]));
        co[i] = (b[i] - s) / a[i][i];
    }
    co
}

/// Evaluates `Σ c_i·x^i` (§2.1), summed like the reference.
#[must_use]
pub fn peval(co: &[f64], x: f64) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    py_sum(co.iter().enumerate().map(|(i, &c)| c * x.powf(i as f64)))
}

/// Calibration warning code (T1.12 C4, design, to be verified).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationWarning {
    /// Calibration angle span is below 10° (`span < 10 − 1e-9`).
    NarrowAngleRange,
    /// The evaluated angle lies outside the calibration angles (extrapolation).
    AngleOutsideRange,
}

impl CalibrationWarning {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NarrowAngleRange => "aci_araligi_dar",
            Self::AngleOutsideRange => "aci_aralik_disi",
        }
    }
}

/// Minimum calibration angle span in degrees before [`CalibrationWarning::NarrowAngleRange`].
pub const MIN_ANGLE_SPAN: f64 = 10.0;

/// iSpindel polynomial calibration result (§2.1).
#[derive(Debug, Clone, PartialEq)]
pub struct IspindelCal {
    /// Coefficients `c0, c1, …` of `SG = Σ c_i·α^i`.
    pub coefficients: Vec<f64>,
    /// Coefficient of determination R².
    pub r2: f64,
    /// Largest absolute residual (SG); above 0.002 the UI warns "check the points".
    pub max_residual: f64,
    /// Smallest calibration angle, degrees.
    pub angle_min: f64,
    /// Largest calibration angle, degrees.
    pub angle_max: f64,
    /// Warnings found at calibration time.
    pub warnings: Vec<CalibrationWarning>,
}

/// iSpindel angle → SG calibration from `(angle °, reference SG)` points (§2.1).
///
/// Needs at least `deg + 2` points with at least `deg + 2` distinct angles (spec default
/// degree 2) and reference SG values that are not all equal (T1.12 C4).
pub fn ispindel_cal(points: &[(f64, f64)], deg: usize) -> Result<IspindelCal> {
    if points.len() < deg + 2 {
        return Err(Error::invalid("points", "fewer than deg + 2 points"));
    }
    let mut angles: Vec<f64> = points.iter().map(|p| p.0).collect();
    angles.sort_by(f64::total_cmp);
    angles.dedup();
    if angles.len() < deg + 2 {
        return Err(Error::invalid(
            "points",
            "fewer than deg + 2 distinct angles",
        ));
    }
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    #[allow(clippy::cast_precision_loss)]
    let my = py_sum(ys.iter().copied()) / points.len() as f64;
    let den = py_sum(ys.iter().map(|g| (g - my).powi(2)));
    if den == 0.0 {
        return Err(Error::invalid(
            "points",
            "reference gravities are all equal",
        ));
    }
    let co = polyfit(&xs, &ys, deg);
    if co.iter().any(|c| !c.is_finite()) {
        return Err(Error::invalid("points", "calibration system is singular"));
    }
    let res: Vec<f64> = points.iter().map(|&(a, g)| g - peval(&co, a)).collect();
    let r2 = 1.0 - py_sum(res.iter().map(|x| x * x)) / den;
    let max_residual = res.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
    let (angle_min, angle_max) = (angles[0], angles[angles.len() - 1]);
    let mut warnings = Vec::new();
    if angle_max - angle_min < MIN_ANGLE_SPAN - 1e-9 {
        warnings.push(CalibrationWarning::NarrowAngleRange);
    }
    Ok(IspindelCal {
        coefficients: co,
        r2,
        max_residual,
        angle_min,
        angle_max,
        warnings,
    })
}

/// SG at `angle` from calibration coefficients, with an extrapolation warning when the
/// angle is outside `[angle_min − 1e-9, angle_max + 1e-9]` (T1.12 C4).
#[must_use]
pub fn ispindel_eval(
    co: &[f64],
    angle_min: f64,
    angle_max: f64,
    angle: f64,
) -> (f64, Vec<CalibrationWarning>) {
    let outside = angle < angle_min - 1e-9 || angle > angle_max + 1e-9;
    (
        peval(co, angle),
        if outside {
            vec![CalibrationWarning::AngleOutsideRange]
        } else {
            Vec::new()
        },
    )
}

/// Tilt correction `SG_flat = slope·SG_tilt + offset` from `(SG_tilt, SG_reference)`
/// points (§2.2). One point gives a constant offset (slope 1); two or more a linear fit.
/// Returns `(slope, offset)`. Errors (T1.16) on empty input, non-finite points, or two or more
/// points with the same tilt reading (the fit would divide by zero).
pub fn tilt_cal(points: &[(f64, f64)]) -> Result<(f64, f64)> {
    if points.is_empty() {
        return Err(Error::invalid("points", "at least one point is required"));
    }
    for &(x, y) in points {
        ensure_finite("tilt", x)?;
        ensure_finite("reference", y)?;
    }
    if points.len() == 1 {
        return Ok((1.0, points[0].1 - points[0].0));
    }
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let (a, b) = linfit(&xs, &ys);
    finite_result("tilt_cal slope", b)?;
    Ok((b, a))
}

// ------------------------------------------------------------------ §3 heating and cooling

/// Specific heat of water, kJ/(kg·K) (§3.1).
pub const C_WATER: f64 = 4.186;
/// Specific heat of grain, kJ/(kg·K) (§3.1).
pub const C_GRAIN: f64 = 1.59;
/// Specific heat of stainless steel, kJ/(kg·K) (§3.1).
pub const C_STEEL: f64 = 0.50;
/// Heat of vaporisation of water, kJ/kg (§3.2).
pub const H_VAP: f64 = 2257.0;

/// Heating time in minutes (§3.1): `C·(T1 − T0) / (P·η) / 60` with
/// `C = V·c_water + m_grain·c_grain + m_kettle·c_steel` (kJ/K).
///
/// `water_l` L, `grain_kg`/`kettle_kg` kg, `t0`/`t1` °C, `power_kw` kW, `eff` 0–1. Errors on
/// non-finite input and when `power_kw` or `eff` is not positive.
pub fn heat_time_min(
    water_l: f64,
    grain_kg: f64,
    kettle_kg: f64,
    t0: f64,
    t1: f64,
    power_kw: f64,
    eff: f64,
) -> Result<f64> {
    for (field, x) in [
        ("water_l", water_l),
        ("grain_kg", grain_kg),
        ("kettle_kg", kettle_kg),
        ("t0", t0),
        ("t1", t1),
    ] {
        ensure_finite(field, x)?;
    }
    ensure_positive("power_kw", power_kw)?;
    ensure_positive("eff", eff)?;
    let c = water_l * C_WATER + grain_kg * C_GRAIN + kettle_kg * C_STEEL;
    Ok(c * (t1 - t0) / (power_kw * eff) / 60.0)
}

/// Boil-off from heater power (§3.2): `P·η_boil·3600 / 2257`, litres per hour.
#[must_use]
pub fn boiloff_l_per_h(power_kw: f64, eff_boil: f64) -> f64 {
    power_kw * eff_boil * 3600.0 / H_VAP
}

/// Time constant τ (minutes) of an immersion chiller from a past batch (§3.3):
/// `minutes / ln((T0 − Tc)/(T − Tc))`. Temperatures in °C. Errors unless the cooling ratio is
/// finite, positive and not 1 (the target must lie between start and tap-water temperature).
pub fn tau_from_batch(t0: f64, tc: f64, t: f64, minutes: f64) -> Result<f64> {
    ensure_finite("minutes", minutes)?;
    let ln = cooling_ln(t0, tc, t)?;
    if ln == 0.0 {
        return Err(Error::invalid("t", "no temperature drop"));
    }
    finite_result("tau_from_batch", minutes / ln)
}

/// `ln((T0 − Tc)/(T − Tc))`; errors when the ratio is not finite and positive.
fn cooling_ln(t0: f64, tc: f64, t: f64) -> Result<f64> {
    ensure_finite("t0", t0)?;
    ensure_finite("tc", tc)?;
    ensure_finite("t", t)?;
    let ratio = (t0 - tc) / (t - tc);
    if !ratio.is_finite() || ratio <= 0.0 {
        return Err(Error::invalid(
            "t",
            "target must lie between start and tap-water temperature",
        ));
    }
    Ok(ratio.ln())
}

/// Immersion chiller cooling time in minutes (§3.3). τ scales linearly with volume:
/// `τ_V = τ·V/V_ref`; `t = τ_V·ln((T0 − Tc)/(T − Tc))`. Volumes in L, temperatures °C. Errors
/// (T1.16, was NaN) when the target is not between start and tap-water temperature, on
/// non-finite input and `vol_ref_l <= 0`.
pub fn cool_time_immersion(
    t0: f64,
    tc: f64,
    t: f64,
    tau: f64,
    vol_l: f64,
    vol_ref_l: f64,
) -> Result<f64> {
    ensure_finite("tau", tau)?;
    ensure_finite("vol_l", vol_l)?;
    ensure_positive("vol_ref_l", vol_ref_l)?;
    let ln = cooling_ln(t0, tc, t)?;
    let tau_v = tau * vol_l / vol_ref_l;
    finite_result("cool_time_immersion", tau_v * ln)
}

/// Plate or counterflow chiller time in minutes (§3.4): `V / flow`.
/// Errors on non-finite input and `flow_l_min <= 0`.
pub fn cool_time_flow(vol_l: f64, flow_l_min: f64) -> Result<f64> {
    ensure_finite("vol_l", vol_l)?;
    ensure_positive("flow_l_min", flow_l_min)?;
    Ok(vol_l / flow_l_min)
}

// ------------------------------------------------------------------ §4 scaling

/// Ingredient class for scaling (§4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngredientKind {
    /// Mashed grain (`mash`).
    Mash,
    /// Sugar or extract (`seker`).
    Sugar,
    /// Bittering hop (`hop_ibu`); first estimate only.
    HopBittering,
    /// Aroma hop (`hop_aroma`).
    HopAroma,
    /// Water salt (`tuz`).
    Salt,
    /// Other (`misc`).
    Misc,
    /// Yeast packs (`maya_paket`).
    YeastPack,
}

/// Scaling mode (§4.1–4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    /// "Hedef değerleri koru" (`hedef`): mash grain by `r·η0/η1`, the rest by `r`.
    Target,
    /// "Yalnız oranla çarp" (`oran`): everything by `r = V1/V0`.
    Ratio,
}

/// First-estimate scaling of ingredient amounts (§4.1–4.2); returns scaled amounts in
/// input order. `v0`/`v1` are fermenter volumes (L), `eff0`/`eff1` mash efficiencies. Errors on
/// non-finite input, `v0 <= 0`, and `eff1 <= 0` in target mode.
pub fn scale_recipe(
    ing: &[(f64, IngredientKind)],
    v0: f64,
    v1: f64,
    eff0: f64,
    eff1: f64,
    mode: ScaleMode,
) -> Result<Vec<f64>> {
    check_scale_inputs(v0, v1, eff0, eff1, mode)?;
    let rv = v1 / v0;
    ing.iter()
        .map(|&(q, kind)| {
            ensure_finite("amount", q)?;
            let f = match (mode, kind) {
                (ScaleMode::Target, IngredientKind::Mash) => rv * eff0 / eff1,
                _ => rv,
            };
            Ok(q * f)
        })
        .collect()
}

fn check_scale_inputs(v0: f64, v1: f64, eff0: f64, eff1: f64, mode: ScaleMode) -> Result<()> {
    ensure_positive("v0", v0)?;
    ensure_finite("v1", v1)?;
    ensure_finite("eff0", eff0)?;
    ensure_finite("eff1", eff1)?;
    if mode == ScaleMode::Target && eff1 <= 0.0 {
        return Err(Error::invalid("eff1", "must be > 0"));
    }
    Ok(())
}

/// Rounds `q` to a multiple of `step`, half up (§4.3, T1.12 C15):
/// `⌊q/step + 0.5 + 1e-9⌋ · step`; the 1e-9 share treats binary residues such as
/// `0.35/0.1 = 3.4999999999999996` as halves. Errors on non-finite input and `step <= 0`.
pub fn round_to(q: f64, step: f64) -> Result<f64> {
    ensure_finite("q", q)?;
    ensure_positive("step", step)?;
    Ok((q / step + 0.5 + 1e-9).floor() * step)
}

/// First-estimate factor of one ingredient (spec §4.1–4.2), covering the rules that
/// [`scale_recipe`] leaves out.
///
/// [`ScaleMode::Ratio`]: everything by `r = v1 / v0` (`per_batch` and `water_ratio` are
/// ignored). [`ScaleMode::Target`]: mash grain `r·η0/η1`; salt by the water-volume ratio
/// `water_ratio` (falls back to `r`); misc marked "per batch" stays 1; everything else `r`.
/// Acid, yeast packs ([`yeast_packs`]) and the mash profile are recomputed with 03/04/05, not
/// scaled by a factor. Errors as [`scale_recipe`].
#[allow(clippy::too_many_arguments)]
pub fn scale_factor(
    kind: IngredientKind,
    v0: f64,
    v1: f64,
    eff0: f64,
    eff1: f64,
    mode: ScaleMode,
    per_batch: bool,
    water_ratio: Option<f64>,
) -> Result<f64> {
    check_scale_inputs(v0, v1, eff0, eff1, mode)?;
    let rv = v1 / v0;
    if mode == ScaleMode::Ratio {
        return Ok(rv);
    }
    Ok(match (mode, kind) {
        (ScaleMode::Target, IngredientKind::Mash) => rv * eff0 / eff1,
        (ScaleMode::Target, IngredientKind::Salt) => match water_ratio {
            Some(w) => {
                ensure_finite("water_ratio", w)?;
                w
            }
            None => rv,
        },
        (ScaleMode::Target, IngredientKind::Misc) if per_batch => 1.0,
        _ => rv,
    })
}

/// Number of yeast packs from the recomputed cell need, rounded up (spec §4.1), at least 1.
/// The ratio is compared with a 1e-9 tolerance: a ratio of 2.0000000001 counts as 2 packs.
/// Errors on non-finite or negative need and `cells_per_pack_billion <= 0`.
#[allow(clippy::cast_possible_truncation)]
pub fn yeast_packs(cells_needed_billion: f64, cells_per_pack_billion: f64) -> Result<i64> {
    ensure_finite("cells_needed_billion", cells_needed_billion)?;
    ensure_positive("cells_per_pack_billion", cells_per_pack_billion)?;
    let packs = (cells_needed_billion / cells_per_pack_billion - 1e-9).ceil();
    if packs.abs() > 9.0e15 {
        return Err(Error::invalid("cells_needed_billion", "out of range"));
    }
    Ok((packs as i64).max(1))
}

/// Where a scaling is applied (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleScope {
    /// Scaling in the recipe: creates a new recipe version.
    Recipe,
    /// Scaling at batch start: applies to that batch only; the recipe is unchanged.
    Batch,
}

/// Scaling record (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalingRecord {
    /// Scope.
    pub scope: ScaleScope,
    /// Mode.
    pub mode: ScaleMode,
    /// Source fermenter volume, L.
    pub v0: f64,
    /// Target fermenter volume, L.
    pub v1: f64,
    /// Source mash efficiency.
    pub eff0: f64,
    /// Target mash efficiency.
    pub eff1: f64,
    /// Volume ratio `v1 / v0`.
    pub ratio: f64,
    /// Recipe version after the scaling (+1 for [`ScaleScope::Recipe`]).
    pub recipe_version: u32,
    /// True when the recipe itself changed.
    pub recipe_changed: bool,
}

/// Builds the scaling record (spec §4.4): a recipe scaling bumps the recipe version; a batch
/// scaling leaves recipe and version untouched. Errors on non-finite input, `v0 <= 0` and a
/// recipe version that would overflow.
pub fn scaling_record(
    scope: ScaleScope,
    mode: ScaleMode,
    volumes: (f64, f64),
    efficiencies: (f64, f64),
    recipe_version: u32,
) -> Result<ScalingRecord> {
    ensure_positive("v0", volumes.0)?;
    ensure_finite("v1", volumes.1)?;
    ensure_finite("eff0", efficiencies.0)?;
    ensure_finite("eff1", efficiencies.1)?;
    let recipe = scope == ScaleScope::Recipe;
    let version = recipe_version
        .checked_add(u32::from(recipe))
        .ok_or(Error::invalid("recipe_version", "overflow"))?;
    Ok(ScalingRecord {
        scope,
        mode,
        v0: volumes.0,
        v1: volumes.1,
        eff0: efficiencies.0,
        eff1: efficiencies.1,
        ratio: volumes.1 / volumes.0,
        recipe_version: version,
        recipe_changed: recipe,
    })
}

// ------------------------------------------------------------------ §5 ion balance

/// Equivalent weights, g/eq (§5.1).
pub const EQW_CA: f64 = 20.039;
/// Mg²⁺ equivalent weight, g/eq.
pub const EQW_MG: f64 = 12.153;
/// Na⁺ equivalent weight, g/eq.
pub const EQW_NA: f64 = 22.990;
/// K⁺ equivalent weight, g/eq.
pub const EQW_K: f64 = 39.098;
/// SO₄²⁻ equivalent weight, g/eq.
pub const EQW_SO4: f64 = 48.031;
/// Cl⁻ equivalent weight, g/eq.
pub const EQW_CL: f64 = 35.453;
/// HCO₃⁻ equivalent weight, g/eq.
pub const EQW_HCO3: f64 = 61.017;
/// CO₃²⁻ equivalent weight, g/eq.
pub const EQW_CO3: f64 = 30.004;
/// NO₃⁻ equivalent weight, g/eq.
pub const EQW_NO3: f64 = 62.004;
/// `CaCO₃` equivalent weight, g/eq (§5.3).
pub const EQW_CACO3: f64 = 50.04;

/// Water ion concentrations in mg/L; missing ions are zero.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Ions {
    /// Calcium.
    pub ca: f64,
    /// Magnesium.
    pub mg: f64,
    /// Sodium.
    pub na: f64,
    /// Potassium.
    pub k: f64,
    /// Sulfate.
    pub so4: f64,
    /// Chloride.
    pub cl: f64,
    /// Bicarbonate.
    pub hco3: f64,
    /// Carbonate.
    pub co3: f64,
    /// Nitrate.
    pub no3: f64,
}

/// Ion balance class (§5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceClass {
    /// `|balance| ≤ 5 %` ("iyi").
    Good,
    /// `5–10 %` ("kabul").
    Acceptable,
    /// `> 10 %` ("kontrol et"): a report may lack ions.
    Check,
}

impl BalanceClass {
    /// Reference label: `iyi`, `kabul`, `kontrol et`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Good => "iyi",
            Self::Acceptable => "kabul",
            Self::Check => "kontrol et",
        }
    }
}

/// Ion balance result (§5.2), unrounded (T1.12 C9); the class uses the raw percentage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IonBalance {
    /// Σ cations, mEq/L.
    pub cations: f64,
    /// Σ anions, mEq/L.
    pub anions: f64,
    /// `100·(Σcat − Σan)/(Σcat + Σan)`, percent.
    pub balance_pct: f64,
    /// Class from `|balance|`.
    pub class: BalanceClass,
}

/// Ion balance of a water report (§5.2). All-zero input yields NaN (the reference raises;
/// error type: T1.16).
#[must_use]
pub fn ion_balance(w: &Ions) -> IonBalance {
    let c = w.ca / EQW_CA + w.mg / EQW_MG + w.na / EQW_NA + w.k / EQW_K;
    let a = w.so4 / EQW_SO4 + w.cl / EQW_CL + w.hco3 / EQW_HCO3 + w.co3 / EQW_CO3 + w.no3 / EQW_NO3;
    let b = 100.0 * (c - a) / (c + a);
    let class = if b.abs() <= 5.0 + 1e-9 {
        BalanceClass::Good
    } else if b.abs() <= 10.0 + 1e-9 {
        BalanceClass::Acceptable
    } else {
        BalanceClass::Check
    };
    IonBalance {
        cations: c,
        anions: a,
        balance_pct: b,
        class,
    }
}

/// Total hardness as mg/L `CaCO₃` (§5.3): `Ca·50.04/20.039 + Mg·50.04/12.153`.
#[must_use]
pub fn hardness_caco3(ca: f64, mg: f64) -> f64 {
    ca * EQW_CACO3 / EQW_CA + mg * EQW_CACO3 / EQW_MG
}

/// Alkalinity as mg/L `CaCO₃` from HCO₃ mg/L (§5.3, valid for pH < 8.3).
#[must_use]
pub fn alk_caco3_from_hco3(hco3: f64) -> f64 {
    hco3 * EQW_CACO3 / EQW_HCO3
}

/// A `CaCO₃` amount in other hardness units (§5.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardnessUnits {
    /// German degrees `°dH` = `CaCO₃` / 17.848.
    pub dh: f64,
    /// French degrees `°fH` = `CaCO₃` / 10.009.
    pub fh: f64,
    /// mEq/L = `CaCO₃` / 50.04.
    pub meq_per_l: f64,
}

/// Converts mg/L `CaCO₃` to °dH, °fH and mEq/L (§5.3).
#[must_use]
pub fn caco3_to(x: f64) -> HardnessUnits {
    HardnessUnits {
        dh: x / 17.848,
        fh: x / 10.009,
        meq_per_l: x / EQW_CACO3,
    }
}

// ------------------------------------------------------------------ §6 hop and fruit losses

/// Hop form for absorption (§6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbsorptionHopForm {
    /// Pellet (`pellet`).
    Pellet,
    /// Whole leaf (`yaprak`).
    Leaf,
    /// Cryo / lupulin (`cryo`).
    Cryo,
}

/// Default wort absorption in L/g by hop form (§6.1) [to be verified].
#[must_use]
pub fn hop_absorption(form: AbsorptionHopForm) -> f64 {
    match form {
        AbsorptionHopForm::Pellet => 0.0070,
        AbsorptionHopForm::Leaf => 0.0130,
        AbsorptionHopForm::Cryo => 0.0045,
    }
}

/// Kettle volume lost to hops in litres (§6.2): `Σ grams·absorption`.
#[must_use]
pub fn kettle_hop_loss(g_by_form: &[(AbsorptionHopForm, f64)]) -> f64 {
    py_sum(g_by_form.iter().map(|&(f, g)| hop_absorption(f) * g))
}

/// Fermenter volume lost to dry hops in litres (§6.2): `Σ grams·absorption`.
#[must_use]
pub fn dryhop_loss(g_by_form: &[(AbsorptionHopForm, f64)]) -> f64 {
    py_sum(g_by_form.iter().map(|&(f, g)| hop_absorption(f) * g))
}

/// Default fruit puree density, kg/L (§6.3) [to be verified].
pub const FRUIT_DENSITY: f64 = 1.05;
/// Default fruit sediment share of the added volume (§6.3) [to be verified].
pub const FRUIT_SEDIMENT: f64 = 0.20;

/// Fruit volumes (§6.3): `(added volume L, volume left in sediment L)`.
/// Errors on non-finite input and `density <= 0`.
pub fn fruit(kg: f64, density: f64, sediment: f64) -> Result<(f64, f64)> {
    ensure_finite("kg", kg)?;
    ensure_positive("density", density)?;
    ensure_finite("sediment", sediment)?;
    let v = kg / density;
    Ok((v, v * sediment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_time_spec_vector() {
        let t = heat_time_min(15.0, 0.0, 8.0, 15.0, 72.0, 2.5, 0.90).unwrap();
        assert!((t - 28.2).abs() < 0.05);
    }

    #[test]
    fn degenerate_inputs_are_errors() {
        assert!(heat_time_min(15.0, 0.0, 8.0, 15.0, 72.0, 0.0, 0.90).is_err());
        assert!(heat_time_min(f64::NAN, 0.0, 8.0, 15.0, 72.0, 2.5, 0.90).is_err());
        // target at or below tap-water temperature
        assert!(cool_time_immersion(100.0, 20.0, 15.0, 25.0, 20.0, 20.0).is_err());
        assert!(tau_from_batch(100.0, 15.0, 100.0, 25.0).is_err());
        assert!(round_to(1.0, 0.0).is_err());
        assert!(yeast_packs(100.0, 0.0).is_err());
        assert!(tilt_cal(&[]).is_err());
        assert!(tilt_cal(&[(1.0, 1.0), (1.0, 1.1)]).is_err());
        assert!(fruit(1.0, 0.0, 0.2).is_err());
    }

    #[test]
    fn py_sum_matches_plain_sum_for_exact_values() {
        assert!((py_sum([0.5, 0.25, 0.125]) - 0.875).abs() < f64::EPSILON);
    }
}
