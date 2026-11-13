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
    pub k_per_h: f64,
    /// Residual sum of squares of the fit (SG²).
    pub rss: f64,
}

/// Result of [`fit_decay`].
#[derive(Debug, Clone, PartialEq)]
pub struct DecayFitResult {
    /// The best fit.
    pub fit: DecayFit,
    /// Table `(FG∞, RSS)` of all valid grid points.
    pub table: Vec<(f64, f64)>,
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
) -> Option<DecayFitResult> {
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
                k_per_h: k,
                rss,
            });
        }
    }
    best.map(|fit| DecayFitResult { fit, table })
}

/// Forecast values (§1.5), unrounded (T1.12 C9): rounding is a presentation concern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForecastResult {
    /// Final gravity `FG∞` (SG).
    pub fg: f64,
    /// Fitted amplitude `A` of `SG(t) = FG∞ + A·e^(−k·t)` (SG), for the trigger time and the
    /// chart curve.
    pub a_sg: f64,
    /// Approximate 95 % profile interval `(low, high)` of `FG∞` (SG).
    pub range_sg: (f64, f64),
    /// Rate `k` (1/hour).
    pub k_per_h: f64,
    /// Finish time since batch start (hours): when `SG − FG∞ ≤ eps`.
    pub finish_h: f64,
    /// Remaining time from the last point (hours), never negative: `0.0` when [`Self::finished`]
    /// (T1.17 C10).
    pub remaining_h: f64,
    /// "Bitti" (T1.17 C10, spec §1.5): `finish_h − t_last ≤ 0`, i.e. SG is already within `eps`
    /// of `FG∞`. The raw `finish_h` may then lie in the past.
    pub finished: bool,
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

/// Default SG distance to `FG∞` that counts as finished, spec 07 §1.5.
pub const FINISH_EPS_SG: f64 = 0.0005;
/// Default minimum number of hourly points, spec 07 §1.4.
pub const MIN_POINTS: usize = 12;
/// Default minimum share (fraction) of the expected drop already done, spec 07 §1.4.
pub const MIN_DROP: f64 = 0.30;

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

/// Inputs of [`forecast`].
#[derive(Debug, Clone, Copy)]
pub struct ForecastInput<'a> {
    /// Readings `(t_hours, SG)`.
    pub points: &'a [(f64, f64)],
    /// Original gravity, SG.
    pub og: f64,
    /// Recipe final gravity, SG.
    pub fg_recipe: f64,
    /// SG distance to the asymptote that counts as finished.
    pub eps_sg: f64,
    /// Minimum number of hourly points.
    pub min_points: usize,
    /// Minimum share of the expected drop already done.
    pub min_drop: f64,
}

/// Fermentation forecast from raw `(t_hours, SG)` readings (§1.4–1.5).
///
/// `og`, `fg_recipe` in SG; `eps` is the SG distance to `FG∞` that counts as finished
/// (spec 0.0005); `min_n` minimum hourly points (spec 12, must be ≥ 3); `min_drop` minimum
/// share of the expected drop already done (spec 0.30). Errors on non-finite input.
pub fn forecast(p: &ForecastInput<'_>) -> Result<Forecast> {
    let ForecastInput {
        points,
        og,
        fg_recipe,
        eps_sg: eps,
        min_points: min_n,
        min_drop,
    } = *p;
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
    let Some(DecayFitResult { fit, table }) = fit_decay(&pts, fg_lo, fg_hi, 400) else {
        return Forecast::InsufficientData;
    };
    #[allow(clippy::cast_precision_loss)]
    let n = pts.len() as f64;
    let lim = fit.rss * (1.0 + 4.0 / (n - 2.0));
    let ok = table.iter().filter(|&&(_, s)| s <= lim).map(|&(x, _)| x);
    let (lo, hi) = ok.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
        (lo.min(x), hi.max(x))
    });
    let t_end = (fit.a / eps).ln() / fit.k_per_h;
    let remaining = t_end - t_last;
    Forecast::Ready(ForecastResult {
        fg: fit.fg_inf,
        a_sg: fit.a,
        range_sg: (lo, hi),
        k_per_h: fit.k_per_h,
        finish_h: t_end,
        remaining_h: remaining.max(0.0),
        finished: remaining <= 0.0,
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
    pub max_residual_sg: f64,
    /// Smallest calibration angle, degrees.
    pub angle_min_deg: f64,
    /// Largest calibration angle, degrees.
    pub angle_max_deg: f64,
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
        max_residual_sg: max_residual,
        angle_min_deg: angle_min,
        angle_max_deg: angle_max,
        warnings,
    })
}

/// SG and warnings of [`ispindel_eval`].
#[derive(Debug, Clone, PartialEq)]
pub struct IspindelReading {
    /// SG at the angle.
    pub sg: f64,
    /// Extrapolation warnings.
    pub warnings: Vec<CalibrationWarning>,
}

/// SG at `angle_deg` from calibration coefficients, with an extrapolation warning when the
/// angle is outside `[angle_min − 1e-9, angle_max + 1e-9]` (T1.12 C4).
#[must_use]
pub fn ispindel_eval(
    co: &[f64],
    angle_min_deg: f64,
    angle_max_deg: f64,
    angle_deg: f64,
) -> IspindelReading {
    let outside = angle_deg < angle_min_deg - 1e-9 || angle_deg > angle_max_deg + 1e-9;
    IspindelReading {
        sg: peval(co, angle_deg),
        warnings: if outside {
            vec![CalibrationWarning::AngleOutsideRange]
        } else {
            Vec::new()
        },
    }
}

/// Linear tilt correction of [`tilt_cal`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TiltCorrection {
    /// Slope of `SG_flat = slope·SG_tilt + offset`.
    pub slope: f64,
    /// Offset, SG.
    pub offset_sg: f64,
}

/// Tilt correction `SG_flat = slope·SG_tilt + offset` from `(SG_tilt, SG_reference)`
/// points (§2.2). One point gives a constant offset (slope 1); two or more a linear fit.
/// Returns `(slope, offset)`. Errors (T1.16) on empty input, non-finite points, or two or more
/// points with the same tilt reading (the fit would divide by zero).
pub fn tilt_cal(points: &[(f64, f64)]) -> Result<TiltCorrection> {
    if points.is_empty() {
        return Err(Error::invalid("points", "at least one point is required"));
    }
    for &(x, y) in points {
        ensure_finite("tilt", x)?;
        ensure_finite("reference", y)?;
    }
    if points.len() == 1 {
        return Ok(TiltCorrection {
            slope: 1.0,
            offset_sg: points[0].1 - points[0].0,
        });
    }
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let (a, b) = linfit(&xs, &ys);
    finite_result("tilt_cal slope", b)?;
    Ok(TiltCorrection {
        slope: b,
        offset_sg: a,
    })
}

/// Applies a Tilt correction to a raw reading: `SG_flat = slope·SG_tilt + offset` (§2.2,
/// T1.17). One-point calibrations have slope 1. The result is unrounded and not range-checked.
/// Errors on non-finite input.
pub fn tilt_apply(c: &TiltCorrection, sg_tilt: f64) -> Result<f64> {
    ensure_finite("slope", c.slope)?;
    ensure_finite("offset", c.offset_sg)?;
    ensure_finite("tilt", sg_tilt)?;
    Ok(c.slope * sg_tilt + c.offset_sg)
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

/// Inputs of [`heat_time_min`].
#[derive(Debug, Clone, Copy)]
pub struct HeatTimeInput {
    /// Water volume, L.
    pub water_l: f64,
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Kettle mass, kg.
    pub kettle_kg: f64,
    /// Start temperature, °C.
    pub t0_c: f64,
    /// Target temperature, °C.
    pub t1_c: f64,
    /// Heating power, kW.
    pub power_kw: f64,
    /// Efficiency, ratio 0-1.
    pub eff: f64,
}

/// Heating time in minutes (§3.1): `C·(T1 − T0) / (P·η) / 60` with
/// `C = V·c_water + m_grain·c_grain + m_kettle·c_steel` (kJ/K).
///
/// `water_l` L, `grain_kg`/`kettle_kg` kg, `t0`/`t1` °C, `power_kw` kW, `eff` 0–1. Errors on
/// non-finite input and when `power_kw` or `eff` is not positive.
pub fn heat_time_min(p: &HeatTimeInput) -> Result<f64> {
    let HeatTimeInput {
        water_l,
        grain_kg,
        kettle_kg,
        t0_c: t0,
        t1_c: t1,
        power_kw,
        eff,
    } = *p;
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
pub fn tau_from_batch(t0_c: f64, tap_c: f64, target_c: f64, minutes: f64) -> Result<f64> {
    ensure_finite("minutes", minutes)?;
    let ln = cooling_ln(t0_c, tap_c, target_c)?;
    if ln == 0.0 {
        return Err(Error::invalid("target_c", "no temperature drop"));
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

/// Inputs of [`cool_time_immersion`].
#[derive(Debug, Clone, Copy)]
pub struct CoolImmersionInput {
    /// Start temperature, °C.
    pub t0_c: f64,
    /// Tap-water temperature, °C.
    pub tap_c: f64,
    /// Target temperature, °C.
    pub target_c: f64,
    /// Cooling time constant for the reference volume, minutes.
    pub tau_min: f64,
    /// Volume, L.
    pub vol_l: f64,
    /// Reference volume of the time constant, L.
    pub vol_ref_l: f64,
}

/// Immersion chiller cooling time in minutes (§3.3). τ scales linearly with volume:
/// `τ_V = τ·V/V_ref`; `t = τ_V·ln((T0 − Tc)/(T − Tc))`. Volumes in L, temperatures °C. Errors
/// (T1.16, was NaN) when the target is not between start and tap-water temperature, on
/// non-finite input and `vol_ref_l <= 0`.
pub fn cool_time_immersion(p: &CoolImmersionInput) -> Result<f64> {
    let CoolImmersionInput {
        t0_c: t0,
        tap_c: tc,
        target_c: t,
        tau_min: tau,
        vol_l,
        vol_ref_l,
    } = *p;
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

/// Inputs of [`scale_factor`].
#[derive(Debug, Clone, Copy)]
pub struct ScaleFactorInput {
    /// Ingredient kind.
    pub kind: IngredientKind,
    /// Source fermenter volume, L.
    pub v0_l: f64,
    /// Target fermenter volume, L.
    pub v1_l: f64,
    /// Source mash efficiency, ratio.
    pub eff0: f64,
    /// Target mash efficiency, ratio.
    pub eff1: f64,
    /// Scaling mode.
    pub mode: ScaleMode,
    /// Misc item marked "per batch".
    pub per_batch: bool,
    /// Water-volume ratio for salts; `None` falls back to the volume ratio.
    pub water_ratio: Option<f64>,
}

/// First-estimate factor of one ingredient (spec §4.1–4.2).
///
/// [`ScaleMode::Ratio`]: everything by `r = v1 / v0` (`per_batch` and `water_ratio` are
/// ignored). [`ScaleMode::Target`]: mash grain `r·η0/η1`; salt by the water-volume ratio
/// `water_ratio` (falls back to `r`); misc marked "per batch" stays 1; everything else `r`.
/// Acid, yeast packs ([`yeast_packs`]) and the mash profile are recomputed with 03/04/05, not
/// scaled by a factor. Errors on non-finite input, `v0 <= 0`, and `eff1 <= 0` in target mode.
pub fn scale_factor(p: &ScaleFactorInput) -> Result<f64> {
    let ScaleFactorInput {
        kind,
        v0_l: v0,
        v1_l: v1,
        eff0,
        eff1,
        mode,
        per_batch,
        water_ratio,
    } = *p;
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
    pub v0_l: f64,
    /// Target fermenter volume, L.
    pub v1_l: f64,
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

/// Inputs of [`scaling_record`].
#[derive(Debug, Clone, Copy)]
pub struct ScalingRecordInput {
    /// Where the scaling is applied.
    pub scope: ScaleScope,
    /// Scaling mode.
    pub mode: ScaleMode,
    /// Source and target fermenter volumes, L.
    pub volumes_l: (f64, f64),
    /// Source and target mash efficiencies, ratio.
    pub efficiencies: (f64, f64),
    /// Recipe version before the scaling.
    pub recipe_version: u32,
}

/// Builds the scaling record (spec §4.4): a recipe scaling bumps the recipe version; a batch
/// scaling leaves recipe and version untouched. Errors on non-finite input, `v0 <= 0` and a
/// recipe version that would overflow.
pub fn scaling_record(p: &ScalingRecordInput) -> Result<ScalingRecord> {
    let ScalingRecordInput {
        scope,
        mode,
        volumes_l: volumes,
        efficiencies,
        recipe_version,
    } = *p;
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
        v0_l: volumes.0,
        v1_l: volumes.1,
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
    pub ca_ppm: f64,
    /// Magnesium.
    pub mg_ppm: f64,
    /// Sodium.
    pub na_ppm: f64,
    /// Potassium.
    pub k_ppm: f64,
    /// Sulfate.
    pub so4_ppm: f64,
    /// Chloride.
    pub cl_ppm: f64,
    /// Bicarbonate.
    pub hco3_ppm: f64,
    /// Carbonate.
    pub co3_ppm: f64,
    /// Nitrate.
    pub no3_ppm: f64,
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
    pub cations_meq_l: f64,
    /// Σ anions, mEq/L.
    pub anions_meq_l: f64,
    /// `100·(Σcat − Σan)/(Σcat + Σan)`, percent.
    pub balance_pct: f64,
    /// Class from `|balance|`.
    pub class: BalanceClass,
}

/// Ion balance of a water report (§5.2). All-zero input yields NaN (the reference raises;
/// error type: T1.16).
#[must_use]
pub fn ion_balance(w: &Ions) -> IonBalance {
    let c = w.ca_ppm / EQW_CA + w.mg_ppm / EQW_MG + w.na_ppm / EQW_NA + w.k_ppm / EQW_K;
    let a = w.so4_ppm / EQW_SO4
        + w.cl_ppm / EQW_CL
        + w.hco3_ppm / EQW_HCO3
        + w.co3_ppm / EQW_CO3
        + w.no3_ppm / EQW_NO3;
    let b = 100.0 * (c - a) / (c + a);
    let class = if b.abs() <= 5.0 + 1e-9 {
        BalanceClass::Good
    } else if b.abs() <= 10.0 + 1e-9 {
        BalanceClass::Acceptable
    } else {
        BalanceClass::Check
    };
    IonBalance {
        cations_meq_l: c,
        anions_meq_l: a,
        balance_pct: b,
        class,
    }
}

/// Total hardness as mg/L `CaCO₃` (§5.3): `Ca·50.04/20.039 + Mg·50.04/12.153`.
#[must_use]
pub fn hardness_caco3(ca_ppm: f64, mg_ppm: f64) -> f64 {
    ca_ppm * EQW_CACO3 / EQW_CA + mg_ppm * EQW_CACO3 / EQW_MG
}

/// Alkalinity as mg/L `CaCO₃` from HCO₃ mg/L (§5.3, valid for pH < 8.3).
#[must_use]
pub fn alk_caco3_from_hco3(hco3_ppm: f64) -> f64 {
    hco3_ppm * EQW_CACO3 / EQW_HCO3
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
pub fn caco3_to(caco3_ppm: f64) -> HardnessUnits {
    HardnessUnits {
        dh: caco3_ppm / 17.848,
        fh: caco3_ppm / 10.009,
        meq_per_l: caco3_ppm / EQW_CACO3,
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

/// Fruit volumes of [`fruit`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FruitVolumes {
    /// Added volume, L.
    pub added_l: f64,
    /// Volume left in the sediment, L.
    pub sediment_l: f64,
}

/// Fruit volumes (§6.3): added volume and volume left in sediment, both in L.
/// Errors on non-finite input and `density <= 0`.
pub fn fruit(kg: f64, density_kg_per_l: f64, sediment: f64) -> Result<FruitVolumes> {
    ensure_finite("kg", kg)?;
    ensure_positive("density_kg_per_l", density_kg_per_l)?;
    ensure_finite("sediment", sediment)?;
    let v = kg / density_kg_per_l;
    Ok(FruitVolumes {
        added_l: v,
        sediment_l: v * sediment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forecast_reports_the_fitted_amplitude() {
        // Vector 07 case 16 (12 hourly points).
        let pts = [
            (0.2, 1.09),
            (1.2, 1.086_197),
            (2.2, 1.082_757),
            (3.2, 1.079_679),
            (4.2, 1.076_963),
            (5.2, 1.074_609),
            (6.2, 1.072_617),
            (7.2, 1.070_987),
            (8.2, 1.069_72),
            (9.2, 1.068_814),
            (10.2, 1.068_271),
            (11.2, 1.068_09),
        ];
        let Forecast::Ready(r) = forecast(&ForecastInput {
            points: &pts,
            og: 1.09,
            fg_recipe: 1.017,
            eps_sg: 0.0005,
            min_points: 12,
            min_drop: 0.30,
        })
        .unwrap() else {
            panic!("forecast must be ready");
        };
        let hm = hourly_median(&pts);
        let fit = fit_decay(&hm, 1.017 - 0.015, hm[hm.len() - 1].1 - 0.0002, 400)
            .unwrap()
            .fit;
        assert!((r.a_sg - fit.a).abs() < 1e-12);
        assert!((r.finish_h - (fit.a / 0.0005).ln() / fit.k_per_h).abs() < 1e-12);
    }

    #[test]
    fn heat_time_spec_vector() {
        let t = heat_time_min(&HeatTimeInput {
            water_l: 15.0,
            grain_kg: 0.0,
            kettle_kg: 8.0,
            t0_c: 15.0,
            t1_c: 72.0,
            power_kw: 2.5,
            eff: 0.90,
        })
        .unwrap();
        assert!((t - 28.2).abs() < 0.05);
    }

    #[test]
    fn degenerate_inputs_are_errors() {
        assert!(
            heat_time_min(&HeatTimeInput {
                water_l: 15.0,
                grain_kg: 0.0,
                kettle_kg: 8.0,
                t0_c: 15.0,
                t1_c: 72.0,
                power_kw: 0.0,
                eff: 0.90
            })
            .is_err()
        );
        assert!(
            heat_time_min(&HeatTimeInput {
                water_l: f64::NAN,
                grain_kg: 0.0,
                kettle_kg: 8.0,
                t0_c: 15.0,
                t1_c: 72.0,
                power_kw: 2.5,
                eff: 0.90
            })
            .is_err()
        );
        // target at or below tap-water temperature
        assert!(
            cool_time_immersion(&CoolImmersionInput {
                t0_c: 100.0,
                tap_c: 20.0,
                target_c: 15.0,
                tau_min: 25.0,
                vol_l: 20.0,
                vol_ref_l: 20.0
            })
            .is_err()
        );
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
