//! Section 10: manual-entry fermentation, extract tools, no-chill (`formuller/10-elle-giris-ekstrakt-nochill.md`).
//!
//! Units follow the specification of each function; see the section file. Metric
//! throughout: °C, L, g, kg, minutes, hours; gravity is plain specific gravity (1.050).
//! Gravity thresholds are compared in integer units via [`gravity_units`].
//!
//! Every function mirrors `formuller/testler/10_elle_ekstrakt_nochill_tv.py` (the parity
//! reference). Tinseth utilisation and the temperature factor are the section 02
//! functions ([`hops::tinseth_utilization`], [`hops::temperature_factor`]).

use crate::Error;
use crate::hops;
use crate::units::{self, gravity_units, round_py};

/// Default lag-phase length in hours (spec §1.2) [to be verified].
pub const LAG_H: f64 = 12.0;
/// Uncertainty (±) of the prior FG estimate (spec §1.3) [to be verified].
pub const SIGMA_FG: f64 = 0.003;
/// End-of-fermentation margin above FG (same as 07 §1.5): 0.0005 SG.
pub const EPS: f64 = 0.0005;
/// [`EPS`] in gravity units.
pub const EPS_U: i64 = 50;
/// Stall rule: gravity difference of at most 0.001 (100 units), spec §1.5.
pub const STALL_U: i64 = 100;
/// "Looks finished": last SG at most FG + 0.004 (400 units), spec §1.5.
pub const NEAR_U: i64 = 400;
/// Late-reading weight: `SG − FG < 0.003` (300 units) is weighted `(SG − FG) / 0.003`.
pub const W_U: i64 = 300;

/// Pounds per kilogram.
pub const LB_PER_KG: f64 = 2.204_622_62;
/// US gallons per litre.
pub const GAL_PER_L: f64 = 0.264_172_052;
/// PPG to points per (kg/L): `LB_PER_KG / GAL_PER_L` ≈ 8.3454 (spec §2.1).
pub const PPG_TO_PKL: f64 = LB_PER_KG / GAL_PER_L;

/// Reference cube cooling time constant for 20 L, hours (spec §3.2) [to be verified].
pub const TAU_20L_H: f64 = 5.0;
/// Temperatures below this (°C) contribute nothing to no-chill isomerisation (spec §3.2).
pub const TSTOP_C: f64 = 40.0;

/// Upper bound on the cooling time integrated by [`t_eq_min`], minutes (30 days; T1.12 D3).
pub const T_EQ_MAX_MIN: f64 = 43_200.0;

/// Largest magnitude accepted for gravity, time and similar inputs (T1.12 D2).
const INPUT_LIMIT: f64 = 1e6;

fn finite(x: f64) -> bool {
    x.is_finite() && x.abs() <= INPUT_LIMIT
}

/// Extract kind (spec §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractKind {
    /// Liquid malt extract.
    Lme,
    /// Dry malt extract.
    Dme,
    /// Honey.
    Honey,
}

impl ExtractKind {
    /// Typical yield in percent (spec §2.1) [to be verified].
    #[must_use]
    pub fn yield_pct(self) -> f64 {
        match self {
            Self::Lme => 78.0,
            Self::Dme => 95.0,
            Self::Honey => 75.0,
        }
    }

    /// Mass density in g/mL (DME: loose bulk density), spec §2.2.
    #[must_use]
    pub fn density_g_per_ml(self) -> f64 {
        match self {
            Self::Lme | Self::Honey => 1.42,
            Self::Dme => 0.60,
        }
    }
}

/// Least-squares weight of a reading (spec §1.3): 1 when `SG − FG ≥ 0.003`, linearly
/// less below, computed in gravity units.
/// Errors on non-finite input (T1.16).
#[allow(clippy::cast_precision_loss)]
pub fn weight(sg: f64, fg: f64) -> crate::Result<f64> {
    Ok(
        ((units::gravity_units_checked(sg)? - units::gravity_units_checked(fg)?) as f64
            / W_U as f64)
            .min(1.0),
    )
}

#[allow(clippy::cast_precision_loss)]
fn weight_u(sg: f64, fg: f64) -> f64 {
    ((gravity_units(sg) - gravity_units(fg)) as f64 / W_U as f64).min(1.0)
}

/// Decay rate fit of [`k_fit`].
#[derive(Debug, Clone, PartialEq)]
pub struct KFit {
    /// Fitted rate, 1/hour.
    pub k_per_h: f64,
    /// Rate of each reading, 1/hour.
    pub k_each_per_h: Vec<f64>,
}

/// Inputs of [`k_fit`].
#[derive(Debug, Clone, Copy)]
pub struct KFitInput<'a> {
    /// Readings `(hours, SG)`.
    pub readings: &'a [(f64, f64)],
    /// Original gravity, SG.
    pub og: f64,
    /// Final gravity, SG.
    pub fg: f64,
    /// Lag phase, hours.
    pub lag_h: f64,
    /// Weight readings by their distance from FG.
    pub weighted: bool,
}

/// Decay rate fit with FG fixed and OG known at t = 0 (spec §1.3).
///
/// `y = ln((SG−FG)/(OG−FG)) = −k (t − lag)`, weighted least squares through the origin:
/// `k = −Σ w·x·y / Σ w·x²`. `readings` are `(hours, SG)`; points with `t ≤ lag` or
/// `SG − FG ≤ 0.0005` are skipped. Returns `(k per hour, per-reading k)`, or `None`
/// when no reading is usable.
#[must_use]
pub fn k_fit(p: &KFitInput<'_>) -> Option<KFit> {
    let KFitInput {
        readings,
        og,
        fg,
        lag_h: lag,
        weighted,
    } = *p;
    let (mut xs, mut ys, mut ws) = (Vec::new(), Vec::new(), Vec::new());
    for &(t, sg) in readings {
        let x = t - lag;
        if x <= 0.0 || gravity_units(sg) - gravity_units(fg) <= EPS_U {
            continue;
        }
        xs.push(x);
        ys.push(((sg - fg) / (og - fg)).ln());
        ws.push(if weighted { weight_u(sg, fg) } else { 1.0 });
    }
    if xs.is_empty() {
        return None;
    }
    let sxx: f64 = ws.iter().zip(&xs).map(|(w, x)| w * x * x).sum();
    let swxy: f64 = ws
        .iter()
        .zip(&xs)
        .zip(&ys)
        .map(|((w, x), y)| w * x * y)
        .sum();
    let k = -swxy / sxx;
    let k_each = xs.iter().zip(&ys).map(|(x, y)| -y / x).collect();
    Some(KFit {
        k_per_h: k,
        k_each_per_h: k_each,
    })
}

/// Hour at which the gravity is within [`EPS`] of FG: `lag + ln((OG−FG)/ε)/k` (spec §1.3).
#[must_use]
pub fn t_end(og: f64, fg: f64, k_per_h: f64, lag_h: f64) -> f64 {
    lag_h + ((og - fg) / EPS).ln() / k_per_h
}

/// Why [`few_forecast`] shows no estimate (spec §1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insufficient {
    /// Fewer than two readings after the lag phase.
    NeedTwoReadings,
    /// Less than 30 % of the expected drop is complete.
    DropBelow30Percent,
    /// The fitted decay rate is not positive.
    NonPositiveK,
}

/// Result of [`few_forecast`].
#[derive(Debug, Clone, PartialEq)]
pub enum FewForecast {
    /// Stall rule, close to FG: looks finished; carries the measured FG.
    LooksFinished {
        /// Last measured SG.
        fg_measured: f64,
    },
    /// Stall rule, far above FG: fermentation may be stuck.
    PossiblyStuck {
        /// Last measured SG.
        sg: f64,
        /// Expected FG (prior).
        fg_expected: f64,
    },
    /// No estimate possible yet.
    Insufficient(Insufficient),
    /// Within 0.0005 of the target; re-measure after 48 h to confirm a stall.
    TargetReached,
    /// Estimate (spec §1.6).
    Estimate {
        /// Decay rate per hour, rounded to 4 decimals.
        k_per_h: f64,
        /// Finish hour for the prior FG, rounded to 1 decimal.
        finish_h: f64,
        /// Finish range in whole days since pitching (floor of earliest, ceil of latest).
        range_days: (i64, i64),
        /// Days remaining from the last reading (floor, ceil), at least 0.
        remaining_days: (i64, i64),
        /// True when per-reading rates differ by more than a factor of 2.
        inconsistent: bool,
    },
}

#[allow(clippy::cast_possible_truncation)]
fn floor_i(x: f64) -> i64 {
    x.floor() as i64
}

#[allow(clippy::cast_possible_truncation)]
fn ceil_i(x: f64) -> i64 {
    x.ceil() as i64
}

/// Inputs of [`few_forecast`].
#[derive(Debug, Clone, Copy)]
pub struct FewForecastInput<'a> {
    /// Readings `(hours, SG)`.
    pub readings: &'a [(f64, f64)],
    /// Original gravity, SG.
    pub og: f64,
    /// Prior final gravity, SG.
    pub fg_prior: f64,
    /// Lag phase, hours.
    pub lag_h: f64,
    /// Uncertainty of the prior FG, SG.
    pub sigma_sg: f64,
}

/// Fermentation forecast from few manual readings (spec §1.3–§1.6).
///
/// `readings` are `(hours since pitching, SG)`, any order; `og` and `fg_prior` are SG;
/// `lag` and `sigma` default to [`LAG_H`] and [`SIGMA_FG`].
///
/// # Errors
/// [`Error::EmptyInput`] for an empty list; [`Error::Unreachable`] when no
/// finite upper end time exists.
pub fn few_forecast(p: &FewForecastInput<'_>) -> Result<FewForecast, Error> {
    let FewForecastInput {
        readings,
        og,
        fg_prior,
        lag_h: lag,
        sigma_sg: sigma,
    } = *p;
    let mut readings = readings.to_vec();
    readings.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let &(t_now, sg_now) = readings
        .last()
        .ok_or(Error::EmptyInput { field: "readings" })?;
    if ![og, fg_prior, lag, sigma].into_iter().all(finite)
        || !readings.iter().all(|&(t, g)| finite(t) && finite(g))
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let post: Vec<(f64, f64)> = readings.iter().copied().filter(|&(t, _)| t > lag).collect();
    // Stall rule (§1.5): last two readings ≥ 48 h apart and ≤ 0.001 difference.
    if readings.len() >= 2 {
        let (t1, g1) = readings[readings.len() - 2];
        let (t2, g2) = readings[readings.len() - 1];
        if t2 - t1 >= 48.0 && (gravity_units(g1) - gravity_units(g2)).abs() <= STALL_U {
            return Ok(if gravity_units(g2) <= gravity_units(fg_prior) + NEAR_U {
                FewForecast::LooksFinished { fg_measured: g2 }
            } else {
                FewForecast::PossiblyStuck {
                    sg: g2,
                    fg_expected: fg_prior,
                }
            });
        }
    }
    if post.len() < 2 {
        return Ok(FewForecast::Insufficient(Insufficient::NeedTwoReadings));
    }
    if 10 * (gravity_units(og) - gravity_units(sg_now))
        < 3 * (gravity_units(og) - gravity_units(fg_prior))
    {
        return Ok(FewForecast::Insufficient(Insufficient::DropBelow30Percent));
    }
    if gravity_units(sg_now) <= gravity_units(fg_prior) + EPS_U {
        return Ok(FewForecast::TargetReached);
    }
    let Some(KFit {
        k_per_h: k,
        k_each_per_h: k_each,
    }) = k_fit(&KFitInput {
        readings: &post,
        og,
        fg: fg_prior,
        lag_h: lag,
        weighted: true,
    })
    .filter(|f| f.k_per_h > 0.0)
    else {
        return Ok(FewForecast::Insufficient(Insufficient::NonPositiveK));
    };
    let ends: Vec<f64> = [fg_prior - sigma, fg_prior, fg_prior + sigma]
        .iter()
        .map(|&fg| {
            if gravity_units(sg_now) - gravity_units(fg) <= EPS_U {
                return t_now; // this FG is already reached
            }
            match k_fit(&KFitInput {
                readings: &post,
                og,
                fg,
                lag_h: lag,
                weighted: true,
            }) {
                Some(KFit { k_per_h: kk, .. }) if kk > 0.0 => t_end(og, fg, kk, lag),
                _ => f64::INFINITY,
            }
        })
        .collect();
    let k_min = k_each.iter().copied().fold(f64::INFINITY, f64::min);
    let k_max = k_each.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let inconsistent = if k_min > 0.0 {
        k_each.len() >= 2 && k_max / k_min > 2.0 + units::TOL
    } else {
        true
    };
    let lo = ends.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ends.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !lo.is_finite() || !hi.is_finite() {
        return Err(Error::Unreachable {
            what: "forecast end time",
        });
    }
    Ok(FewForecast::Estimate {
        k_per_h: round_py(k, 4),
        finish_h: round_py(ends[1], 1),
        range_days: (floor_i(lo / 24.0), ceil_i(hi / 24.0)),
        remaining_days: (
            floor_i(((lo - t_now) / 24.0).max(0.0)),
            ceil_i((hi - t_now) / 24.0),
        ),
        inconsistent,
    })
}

/// PPG of an extract from its yield: `yield% · 46 / 100` (spec §2.1).
#[must_use]
pub fn ppg(yield_pct: f64) -> f64 {
    yield_pct * 46.0 / 100.0
}

/// Points given by 1 kg in 1 L, from yield percent (spec §2.1): PPG × 8.3454.
#[must_use]
pub fn points_kg_l(yield_pct: f64) -> f64 {
    ppg(yield_pct) * PPG_TO_PKL
}

/// Grams to millilitres for an extract kind (spec §2.2).
#[must_use]
pub fn g_to_ml(kind: ExtractKind, g: f64) -> f64 {
    g / kind.density_g_per_ml()
}

/// Millilitres to grams for an extract kind (spec §2.2).
#[must_use]
pub fn ml_to_g(kind: ExtractKind, ml: f64) -> f64 {
    ml * kind.density_g_per_ml()
}

/// Amount of `kind_to` giving the same points as `g` grams of `kind_from`:
/// `m2 = m1 · yield1 / yield2` (spec §2.3). Grams in, grams out.
#[must_use]
pub fn swap(kind_from: ExtractKind, kind_to: ExtractKind, g: f64) -> f64 {
    g * kind_from.yield_pct() / kind_to.yield_pct()
}

/// Extract mass in grams for a target OG in `v_final_l` litres, extract efficiency taken
/// as 100 % (spec §2.4).
#[must_use]
pub fn extract_for_og(kind: ExtractKind, og: f64, v_final_l: f64) -> f64 {
    let pts = (og - 1.0) * 1000.0;
    pts * v_final_l / points_kg_l(kind.yield_pct()) * 1000.0
}

/// Partial boil result (spec §2.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartialBoil {
    /// Kettle gravity (SG), rounded to 4 decimals.
    pub kettle_sg: f64,
    /// Top-up water in litres, rounded to 2 decimals.
    pub topup_l: f64,
}

/// Partial boil: kettle gravity and top-up water (spec §2.5).
///
/// `og_final` SG after topping up, `v_final_l` litres in the fermenter, `v_kettle_post_l`
/// litres in the kettle at end of boil, `loss_l` litres left in the kettle (trub, hop
/// absorption). Volume sent to the fermenter is `v_kettle_post − loss`.
///
/// # Errors
/// [`Error::InvalidInput`] unless `0 < v_out ≤ v_final`.
pub fn partial_boil(
    og_final: f64,
    v_final_l: f64,
    v_kettle_post_l: f64,
    loss_l: f64,
) -> Result<PartialBoil, Error> {
    let v_out = v_kettle_post_l - loss_l;
    if v_out <= 0.0 || v_final_l < v_out {
        return Err(Error::invalid("volume", "invalid volumes"));
    }
    let pts_k = (og_final - 1.0) * 1000.0 * v_final_l / v_out;
    Ok(PartialBoil {
        kettle_sg: round_py(1.0 + pts_k / 1000.0, 4),
        topup_l: round_py(v_final_l - v_out, 2),
    })
}

/// IBU of a partial boil (spec §2.5): Tinseth utilisation at the kettle gravity, mg/L in
/// the final volume. `hop_list` entries are `(grams, alpha-acid fraction, minutes)`;
/// `v_final_l` litres.
#[must_use]
pub fn partial_boil_ibu(hop_list: &[(f64, f64, f64)], sg_kettle: f64, v_final_l: f64) -> f64 {
    hop_list
        .iter()
        .map(|&(g, aa, t)| hops::tinseth_utilization(sg_kettle, t) * aa * g * 1000.0 / v_final_l)
        .sum()
}

/// Inputs of [`topoff_temp`].
#[derive(Debug, Clone, Copy)]
pub struct TopoffInput {
    /// Wort volume, L.
    pub v_wort_l: f64,
    /// Wort temperature, °C.
    pub t_wort_c: f64,
    /// Water volume to add, L.
    pub v_water_l: f64,
    /// Target mix temperature, °C.
    pub t_target_c: f64,
    /// Wort-to-water heat capacity ratio.
    pub c_ratio: f64,
}

/// Temperature (°C) of the water to add so the mix reaches `t_target` (spec §2.6).
/// Volumes in litres, temperatures in °C; `c_ratio` is the wort-to-water heat capacity
/// ratio (≈ 1).
#[must_use]
pub fn topoff_temp(p: &TopoffInput) -> f64 {
    let TopoffInput {
        v_wort_l: v_wort,
        t_wort_c: t_wort,
        v_water_l: v_water,
        t_target_c: t_target,
        c_ratio,
    } = *p;
    (t_target * (v_wort * c_ratio + v_water) - v_wort * c_ratio * t_wort) / v_water
}

/// Cooling time constant (hours) of a cube of `v_l` litres, scaled from the 20 L value
/// `tau20_h` by the cube root of volume (spec §3.2).
#[must_use]
pub fn tau_h(v_l: f64, tau20_h: f64) -> f64 {
    tau20_h * (v_l / 20.0).powf(1.0 / 3.0)
}

/// Inputs of [`t_eq_min`].
#[derive(Debug, Clone, Copy)]
pub struct TEqInput {
    /// Start temperature, °C.
    pub t0_c: f64,
    /// Ambient temperature, °C.
    pub t_amb_c: f64,
    /// Cooling time constant, hours.
    pub tau_h: f64,
    /// Integration step, minutes.
    pub dt_min: f64,
    /// Stop temperature, °C.
    pub t_stop_c: f64,
}

/// Equivalent boil minutes of a no-chill cube (spec §3.2).
///
/// Cooling `T(t) = T_amb + (T0 − T_amb) e^(−t/τ)`; result is `∫ Ftemp(T) dt` as a left
/// Riemann sum with step `dt` minutes, stopping once `T < t_stop` (°C). `tau_hours`
/// is the time constant in hours.
///
/// # Errors
/// [`Error::InvalidInput`] when an input is not finite, `t_amb ≥ t_stop`, `dt ≤ 0`,
/// `tau_hours ≤ 0`, or the cube is still above `t_stop` after 30 days ([`T_EQ_MAX_MIN`]).
pub fn t_eq_min(p: &TEqInput) -> Result<f64, Error> {
    let TEqInput {
        t0_c: t0,
        t_amb_c: t_amb,
        tau_h: tau_hours,
        dt_min: dt,
        t_stop_c: t_stop,
    } = *p;
    if ![t0, t_amb, tau_hours, dt, t_stop].into_iter().all(finite) {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    if t_amb >= t_stop || dt <= 0.0 || tau_hours <= 0.0 {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let tau = tau_hours * 60.0;
    let (mut t, mut s) = (0.0_f64, 0.0_f64);
    loop {
        let temp = t_amb + (t0 - t_amb) * (-t / tau).exp();
        if temp < t_stop {
            return Ok(s);
        }
        if t >= T_EQ_MAX_MIN {
            return Err(Error::invalid("input", "not finite or out of range"));
        }
        s += hops::temperature_factor(temp) * dt;
        t += dt;
    }
}

/// No-chill extra-time mode (spec §3.1–§3.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NochillMode {
    /// Rule of thumb: every addition gets +20 minutes.
    Rule,
    /// Model: plus the equivalent minutes from [`t_eq_min`] (dt 0.5 min, stop 40 °C).
    Model {
        /// Wort temperature at the start of the cube stand, °C.
        t0_c: f64,
        /// Ambient temperature, °C.
        t_amb_c: f64,
        /// Cube volume in litres; `None` or 0 uses the batch volume.
        v_cube_l: Option<f64>,
    },
}

/// Result of [`nochill_ibu`] (rounded to 1 decimal like the reference).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NochillIbu {
    /// IBU without the extra standing time.
    pub base_ibu: f64,
    /// IBU with the no-chill extra time.
    pub nochill_ibu: f64,
    /// Extra standing time, minutes.
    pub extra_min: f64,
}

/// No-chill IBU: `(base IBU, no-chill IBU, extra minutes)`, each rounded to 1 decimal
/// (spec §3).
///
/// `hop_list` entries are `(grams, alpha-acid fraction, minutes)`, `sg` the boil
/// gravity, `v_l` litres.
///
/// # Errors
/// [`Error::InvalidInput`] for non-finite `sg`, `v ≤ 0`, or a cooling model that cannot
/// end (see [`t_eq_min`]).
pub fn nochill_ibu(
    hop_list: &[(f64, f64, f64)],
    sg: f64,
    v_l: f64,
    mode: NochillMode,
) -> Result<NochillIbu, Error> {
    if !finite(sg) || !finite(v_l) || v_l <= 0.0 {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let extra = match mode {
        NochillMode::Rule => 20.0,
        NochillMode::Model {
            t0_c: t0,
            t_amb_c: t_amb,
            v_cube_l: v_cube,
        } => {
            let vc = v_cube.filter(|x| *x != 0.0).unwrap_or(v_l);
            if !finite(vc) || vc <= 0.0 {
                return Err(Error::invalid("input", "not finite or out of range"));
            }
            t_eq_min(&TEqInput {
                t0_c: t0,
                t_amb_c: t_amb,
                tau_h: tau_h(vc, TAU_20L_H),
                dt_min: 0.5,
                t_stop_c: TSTOP_C,
            })?
        }
    };
    let ibu = |add: f64| -> f64 {
        hop_list
            .iter()
            .map(|&(g, aa, t)| hops::tinseth_utilization(sg, t + add) * aa * g * 1000.0 / v_l)
            .sum()
    };
    Ok(NochillIbu {
        base_ibu: round_py(ibu(0.0), 1),
        nochill_ibu: round_py(ibu(extra), 1),
        extra_min: round_py(extra, 1),
    })
}

// ------------------------------------------------------------------ method choice, display and stall rules

/// Stall-rule outcome (spec §1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stall {
    /// Last SG at most FG + 0.004: "looks finished".
    LooksFinished,
    /// Last SG above FG + 0.004: "may be stuck".
    PossiblyStuck,
}

impl Stall {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LooksFinished => "bitmis_gorunuyor",
            Self::PossiblyStuck => "takilmis",
        }
    }
}

fn sorted_readings(readings: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut rd = readings.to_vec();
    rd.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    rd
}

/// Stall rule (spec §1.5): the last two readings (sorted by time) are at least 48 h apart
/// and differ by at most 100 gravity units. `None` when the rule does not apply (also for
/// fewer than two readings).
#[must_use]
pub fn stall_check(readings: &[(f64, f64)], fg_prior: f64) -> Option<Stall> {
    let rd = sorted_readings(readings);
    let [.., (t1, g1), (t2, g2)] = rd.as_slice() else {
        return None;
    };
    if t2 - t1 >= 48.0 && (gravity_units(*g1) - gravity_units(*g2)).abs() <= STALL_U {
        Some(if gravity_units(*g2) <= gravity_units(fg_prior) + NEAR_U {
            Stall::LooksFinished
        } else {
            Stall::PossiblyStuck
        })
    } else {
        None
    }
}

/// Which forecast method applies (spec §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastMethod {
    /// Device data is sufficient (07 §1.4): use the 07 §1 model.
    Device,
    /// Manual readings only: use the section 10 model.
    Manual,
    /// Neither: no forecast.
    None,
}

impl ForecastMethod {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Device => "cihaz",
            Self::Manual => "elle",
            Self::None => "yok",
        }
    }
}

/// Method choice (spec §1.1): device data when sufficient, else manual readings if any.
#[must_use]
pub fn forecast_method(device_sufficient: bool, manual_count: usize) -> ForecastMethod {
    if device_sufficient {
        ForecastMethod::Device
    } else if manual_count > 0 {
        ForecastMethod::Manual
    } else {
        ForecastMethod::None
    }
}

/// Source label of a reading (spec §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingSource {
    /// Device (sensor).
    Device,
    /// Manual hydrometer or refractometer.
    Manual,
    /// Paper log.
    Paper,
}

/// What a reading is used for (spec §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingRole {
    /// A point of the fermentation model.
    Model,
    /// Device bias correction (07 §2.2); not a model point.
    BiasCorrection,
}

impl ReadingRole {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::BiasCorrection => "sapma_duzeltmesi",
        }
    }
}

/// Role of a reading by source label (spec §1.1): manual and paper readings of a batch
/// with device data are bias-correction points, otherwise model points.
#[must_use]
pub fn reading_role(source: ReadingSource, batch_has_device: bool) -> ReadingRole {
    if source != ReadingSource::Device && batch_has_device {
        ReadingRole::BiasCorrection
    } else {
        ReadingRole::Model
    }
}

/// Display status of the few-readings estimate (spec §1.4, evaluation order as listed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FewStatus {
    /// All conditions hold: show the estimate.
    Show,
    /// Stall rule: looks finished.
    LooksFinished,
    /// Stall rule: may be stuck.
    PossiblyStuck,
    /// Fewer than two readings after the lag phase (also: no readings).
    NeedTwoReadings,
    /// Less than 30 % of the expected drop is complete.
    DropBelow30Percent,
    /// Within 0.0005 of the target.
    TargetReached,
    /// The fitted decay rate is not positive.
    NonPositiveK,
}

impl FewStatus {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Show => "goster",
            Self::LooksFinished => "bitmis_gorunuyor",
            Self::PossiblyStuck => "takilmis",
            Self::NeedTwoReadings => "az_okuma",
            Self::DropBelow30Percent => "dusus_yetersiz",
            Self::TargetReached => "hedefe_ulasti",
            Self::NonPositiveK => "k_pozitif_degil",
        }
    }
}

/// Whether the few-readings estimate is shown (spec §1.4): stall rule, two readings after
/// lag, 30 % drop, target reached, positive `k`, in that order.
///
/// # Errors
/// [`Error::InvalidInput`] for non-finite or out-of-range input (T1.16).
pub fn few_status(
    readings: &[(f64, f64)],
    og: f64,
    fg_prior: f64,
    lag_h: f64,
) -> Result<FewStatus, Error> {
    if ![og, fg_prior, lag_h].into_iter().all(finite)
        || !readings.iter().all(|&(t, g)| finite(t) && finite(g))
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    Ok(few_status_checked(readings, og, fg_prior, lag_h))
}

fn few_status_checked(readings: &[(f64, f64)], og: f64, fg_prior: f64, lag: f64) -> FewStatus {
    let rd = sorted_readings(readings);
    let Some(&(_, sg_now)) = rd.last() else {
        return FewStatus::NeedTwoReadings;
    };
    if let Some(s) = stall_check(&rd, fg_prior) {
        return match s {
            Stall::LooksFinished => FewStatus::LooksFinished,
            Stall::PossiblyStuck => FewStatus::PossiblyStuck,
        };
    }
    let post: Vec<(f64, f64)> = rd.iter().copied().filter(|&(t, _)| t > lag).collect();
    if post.len() < 2 {
        return FewStatus::NeedTwoReadings;
    }
    if 10 * (gravity_units(og) - gravity_units(sg_now))
        < 3 * (gravity_units(og) - gravity_units(fg_prior))
    {
        return FewStatus::DropBelow30Percent;
    }
    if gravity_units(sg_now) <= gravity_units(fg_prior) + EPS_U {
        return FewStatus::TargetReached;
    }
    match k_fit(&KFitInput {
        readings: &post,
        og,
        fg: fg_prior,
        lag_h: lag,
        weighted: true,
    }) {
        Some(KFit { k_per_h: k, .. }) if k > 0.0 => FewStatus::Show,
        _ => FewStatus::NonPositiveK,
    }
}

/// Simple no-chill rule: extra minutes added to every hop addition (spec §3.1).
pub const NOCHILL_RULE_MIN: f64 = 20.0;

/// No-chill rule time (spec §3.1): addition time plus 20 minutes; a flame-out (0 min)
/// addition counts as 20 minutes.
#[must_use]
pub fn nochill_rule_time(t_min: f64) -> f64 {
    t_min + NOCHILL_RULE_MIN
}
