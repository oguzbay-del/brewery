//! Section 11: gravity triggers, keg priming, cask (`formuller/11-tetikleyici-fici-cask.md`).
//!
//! Units follow the specification of each function; see the section file. Metric:
//! °C, L, g, hours; gravity as SG; pressure in bar (gauge) unless a name ends in `_pa`.
//! Gravity thresholds are compared in integer units (`units::gravity_units`,
//! 1 unit = 0.00001 SG) per `formuller/00-OKU-BENI.md`. The CO2 tables of section 04
//! (`yeast::residual_co2`, `yeast::co2_psi`) take °F; this module converts with
//! `units::c_to_f`.

use crate::Error;
use crate::units::{BALLING_CO2, CO2_G_PER_VOL, TOL, c_to_f, gravity_units, round_py, to_plato};
use crate::yeast;

/// Lowest temperature, °C, for which the 04 §3.5 polynomial is reliable (spec §5, T1.12 D6).
pub const POLY_T_MIN_C: f64 = 0.0;
/// Highest CO2 volumes for which the 04 §3.5 polynomial is reliable (spec §5, T1.12 D6).
pub const POLY_VOL_MAX: f64 = 3.5;
/// Upper end of the [`equilibrium_vol`] search interval, volumes.
pub const EQ_VOL_MAX: f64 = 10.0;
/// Largest magnitude accepted for inputs (T1.12 D2).
const INPUT_LIMIT: f64 = 1e6;

fn finite(xs: &[f64]) -> bool {
    xs.iter().all(|x| x.is_finite() && x.abs() <= INPUT_LIMIT)
}

/// Where a range warning applies (T1.12 D6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyPoint {
    /// Conditioning temperature.
    Conditioning,
    /// Serving temperature.
    Serving,
    /// Target volumes.
    Target,
    /// Peak volumes at conditioning temperature.
    Peak,
}

impl PolyPoint {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Conditioning => "kondisyon",
            Self::Serving => "servis",
            Self::Target => "hedef",
            Self::Peak => "tepe",
        }
    }
}

/// Priming warning as a code with numeric fields; the client builds the text and units
/// (T1.12 D7, ADR-0004). Values are not rounded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimingWarning {
    /// Sugar dry matter unknown: 1.0 assumed (spec §2.3); `kuru_madde_bilinmiyor`.
    DryMatterUnknown,
    /// Temperature below the polynomial range (`polinom_sicaklik_alti`).
    PolynomialBelowTemperature {
        /// Which temperature.
        point: PolyPoint,
        /// Temperature, °C.
        t_c: f64,
    },
    /// Volumes above the polynomial range (`polinom_vol_ustu`).
    PolynomialAboveVolume {
        /// Which volumes.
        point: PolyPoint,
        /// Volumes.
        co2_vol: f64,
    },
    /// Total CO2 exceeds what 10 volumes hold: peak volumes saturate (`tepe_vol_doymus`).
    PeakVolumeSaturated {
        /// Search limit, volumes.
        limit_co2_vol: f64,
    },
    /// Peak gauge pressure above the keg limit (`tepe_basinci_siniri_asildi`).
    PeakAboveLimit {
        /// Peak gauge pressure including air, bar.
        peak_bar: f64,
        /// Keg limit, bar.
        limit_bar: f64,
        /// Conditioning temperature, °C.
        t_cond_c: f64,
    },
}

impl PrimingWarning {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::DryMatterUnknown => "kuru_madde_bilinmiyor",
            Self::PolynomialBelowTemperature { .. } => "polinom_sicaklik_alti",
            Self::PolynomialAboveVolume { .. } => "polinom_vol_ustu",
            Self::PeakVolumeSaturated { .. } => "tepe_vol_doymus",
            Self::PeakAboveLimit { .. } => "tepe_basinci_siniri_asildi",
        }
    }
}

// ================================================================ 1. Gravity trigger

/// Slope limit for "FG stable": 0.0005 SG per day (spec §1.4).
pub const STABLE_PER_DAY: f64 = 0.0005;

/// First trigger time (spec §1.2): hourly medians `(t_hours, SG)`; the condition
/// `u(SG) ≤ u(fg_ref) + u(delta)` must hold at `consecutive` consecutive points
/// (at least 1). Returns the time of the first point of that run, `None` if never.
///
/// # Errors
/// [`Error::InvalidInput`] for `consecutive = 0` or non-finite or out-of-range input
/// (T1.12 D9, D2).
pub fn trigger_index(
    points: &[(f64, f64)],
    fg_ref: f64,
    delta: f64,
    consecutive: usize,
) -> Result<Option<f64>, Error> {
    if consecutive == 0
        || !finite(&[fg_ref, delta])
        || !points.iter().all(|&(t, g)| finite(&[t, g]))
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let limit = gravity_units(fg_ref) + gravity_units(delta);
    let mut run = 0_usize;
    for (i, &(_, g)) in points.iter().enumerate() {
        run = if gravity_units(g) <= limit {
            run + 1
        } else {
            0
        };
        if run >= consecutive {
            return Ok(Some(points[i + 1 - consecutive].0));
        }
    }
    Ok(None)
}

/// Predicted trigger from the 07 §1 decay model (spec §1.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerForecast {
    /// `("tetik", t)`: trigger after `t` hours.
    At(f64),
    /// `("gecmis", 0.0)`: threshold already satisfied at t = 0.
    AlreadyPast,
    /// `("erisilemez", None)`: the asymptote is at or above the threshold.
    Unreachable,
}

/// Inputs of [`predicted_trigger`].
#[derive(Debug, Clone, Copy)]
pub struct PredictedTriggerInput {
    /// Asymptotic final gravity, SG.
    pub fg_inf: f64,
    /// Amplitude `A`, SG.
    pub a_sg: f64,
    /// Rate `k`, 1/hour.
    pub k_per_h: f64,
    /// Reference final gravity, SG.
    pub fg_ref: f64,
    /// Offset above the reference, SG.
    pub delta: f64,
}

/// Trigger time for `SG(t) = fg_inf + a·e^(−k·t)` reaching `fg_ref + delta` (spec §1.3).
/// SG inputs, `a` in SG, `k` in 1/hour; result in hours.
pub fn predicted_trigger(p: &PredictedTriggerInput) -> Result<TriggerForecast, Error> {
    let PredictedTriggerInput {
        fg_inf,
        a_sg: a,
        k_per_h: k,
        fg_ref,
        delta,
    } = *p;
    if !finite(&[fg_inf, a, k, fg_ref, delta]) {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let target_u = gravity_units(fg_ref) + gravity_units(delta) - gravity_units(fg_inf);
    if target_u <= 0 {
        return Ok(TriggerForecast::Unreachable);
    }
    if gravity_units(a) <= target_u {
        return Ok(TriggerForecast::AlreadyPast);
    }
    if k < 0.0 {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    if k == 0.0 {
        return Err(Error::DivisionByZero { field: "divisor" });
    }
    #[allow(clippy::cast_precision_loss)]
    let target = target_u as f64 / 100_000.0;
    Ok(TriggerForecast::At((a / target).ln() / k))
}

/// Source of the reference FG (spec §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FgSource {
    /// `"tarif"`: the recipe FG.
    Recipe,
    /// `"tahmin"`: the forecast asymptote.
    Forecast,
}

/// Reference FG and its source, see [`fg_reference`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FgReference {
    /// Reference FG (SG).
    pub fg: f64,
    /// Where it comes from.
    pub source: FgSource,
}

/// Reference FG (spec §1.1): the forecast `fg_inf` replaces the recipe FG when `shown`
/// and it deviates by more than `delta − 1 point` in either direction (integer units).
#[must_use]
pub fn fg_reference(fg_recipe: f64, fg_inf: Option<f64>, shown: bool, delta: f64) -> FgReference {
    if let (true, Some(inf)) = (shown, fg_inf) {
        let margin = gravity_units(delta) - 100;
        let recipe_u = gravity_units(fg_recipe);
        let inf_u = gravity_units(inf);
        if inf_u > recipe_u + margin || inf_u < recipe_u - margin {
            return FgReference {
                fg: inf,
                source: FgSource::Forecast,
            };
        }
    }
    FgReference {
        fg: fg_recipe,
        source: FgSource::Recipe,
    }
}

/// Result of the "FG stable" check (spec §1.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stability {
    /// Too few points or all at the same time: "veri yetersiz".
    Insufficient,
    /// Slope measured; `slope_per_day` in SG/day.
    Measured { stable: bool, slope_per_day: f64 },
}

/// FG stable when the least-squares slope over the last `hours` hours satisfies
/// `|slope × 24| ≤ 0.0005 + 1e-9` (spec §1.4). Points are `(t_hours, SG)`; at least
/// `min_n` points in the window are needed.
pub fn is_stable(points: &[(f64, f64)], hours: f64, min_n: usize) -> Result<Stability, Error> {
    let t_end = points
        .last()
        .ok_or(Error::EmptyInput { field: "points" })?
        .0;
    if !finite(&[hours]) || !points.iter().all(|&(t, g)| finite(&[t, g])) {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let p: Vec<(f64, f64)> = points
        .iter()
        .copied()
        .filter(|&(t, _)| t >= t_end - hours)
        .collect();
    if p.len() < min_n {
        return Ok(Stability::Insufficient);
    }
    #[allow(clippy::cast_precision_loss)]
    let n = p.len() as f64;
    let mx = p.iter().map(|&(t, _)| t).sum::<f64>() / n;
    let my = p.iter().map(|&(_, g)| g).sum::<f64>() / n;
    let sxx: f64 = p.iter().map(|&(t, _)| (t - mx).powi(2)).sum();
    if sxx <= 0.0 {
        return Ok(Stability::Insufficient);
    }
    let b = p.iter().map(|&(t, g)| (t - mx) * (g - my)).sum::<f64>() / sxx;
    Ok(Stability::Measured {
        stable: (b * 24.0).abs() <= STABLE_PER_DAY + TOL,
        slope_per_day: b * 24.0,
    })
}

/// Temperature ramp duration, hours: `|to − from| / rate × 24` (spec §1.5).
/// Temperatures in °C, `rate_c_per_day` in °C/day.
pub fn ramp_hours(t_from_c: f64, t_to_c: f64, rate_c_per_day: f64) -> Result<f64, Error> {
    if !finite(&[t_from_c, t_to_c, rate_c_per_day]) {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    if rate_c_per_day == 0.0 {
        return Err(Error::DivisionByZero { field: "divisor" });
    }
    Ok((t_to_c - t_from_c).abs() / rate_c_per_day * 24.0)
}

/// Lagering weeks (spec §1.5): OG converted to integer points (0.01-point units,
/// `round((OG − 1) × 100000)`), divided by `pts_per_week` points and rounded up,
/// clamped to `[lo, hi]` weeks. Defaults in the spec: 8, 2, 12.
///
/// # Errors
/// [`Error::InvalidInput`] for non-finite `og` or `pts_per_week` outside `0..=10^6`;
/// [`Error::DivisionByZero`] for `pts_per_week = 0` (T1.12 D9, D2).
pub fn lager_weeks(og: f64, pts_per_week: i64, lo_weeks: i64, hi_weeks: i64) -> Result<i64, Error> {
    if !finite(&[og]) || !(0..=1_000_000).contains(&pts_per_week) {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    #[allow(clippy::cast_possible_truncation)]
    let pts = ((og - 1.0) * 100_000.0).round_ties_even() as i64;
    let d = pts_per_week * 100;
    if d == 0 {
        return Err(Error::DivisionByZero { field: "divisor" });
    }
    let w = -(-pts).div_euclid(d); // ceil(pts / d) for d > 0
    Ok(hi_weeks.min(lo_weeks.max(w)))
}

// ================================================================ 2. Keg priming

/// Gas constant, J/(mol·K).
pub const R: f64 = 8.314_462_618;
/// One standard atmosphere, Pa.
pub const P_ATM: f64 = 101_325.0;
/// Molar volume of an ideal gas at 0 °C, 1 atm, L/mol.
pub const VM: f64 = 22.414;

/// Priming sugar (spec §2.3): molar mass and mol CO2 per mol sugar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sugar {
    /// Dextrose monohydrate, 198.17 g/mol, 2 mol CO2.
    Dextrose,
    /// Sucrose, 342.30 g/mol, 4 mol CO2.
    Sucrose,
    /// Invert sugar, 360.31 g/mol, 4 mol CO2.
    Invert,
}

impl Sugar {
    /// Molar mass, g/mol.
    #[must_use]
    pub fn molar_mass(self) -> f64 {
        match self {
            Sugar::Dextrose => 198.17,
            Sugar::Sucrose => 342.30,
            Sugar::Invert => 360.31,
        }
    }

    /// Mol CO2 per mol sugar.
    #[must_use]
    pub fn co2_per_mol(self) -> f64 {
        match self {
            Sugar::Dextrose => 2.0,
            Sugar::Sucrose | Sugar::Invert => 4.0,
        }
    }
}

/// Residual CO2 in beer, volumes, at `t_c` °C (spec §2.1; section 04 §3.2).
#[must_use]
pub fn residual_vol(t_c: f64) -> f64 {
    yeast::residual_co2(c_to_f(t_c))
}

/// Keg pressure, psi, to hold `co2_vol` volumes at `t_c` °C (section 04 §3.5).
#[must_use]
pub fn co2_psi_c(co2_vol: f64, t_c: f64) -> f64 {
    yeast::co2_psi(co2_vol, c_to_f(t_c))
}

/// Sugar mass, g, for `mol` mol CO2 (spec §2.3). `dry` is the dry-matter fraction
/// (0–1); `None` means unknown and 1.0 is used (callers add `WARN_DRY_UNKNOWN`).
pub fn sugar_from_mol(mol: f64, sugar: Sugar, dry: Option<f64>) -> Result<f64, Error> {
    let d = dry.unwrap_or(1.0);
    if d == 0.0 {
        return Err(Error::DivisionByZero { field: "divisor" });
    }
    Ok(mol / sugar.co2_per_mol() * sugar.molar_mass() / d)
}

/// CO2 needed in the beer itself, mol: `max(0, target − residual(t_res)) × v_beer / 22.414`.
#[must_use]
pub fn bottle_mol(co2_vol: f64, t_res_c: f64, v_beer_l: f64) -> f64 {
    (co2_vol - residual_vol(t_res_c)).max(0.0) * v_beer_l / VM
}

/// Absolute CO2 partial pressure, Pa, for `co2_vol` volumes at `t_c` °C (gauge table + 1 atm).
#[must_use]
pub fn p_abs_pa(co2_vol: f64, t_c: f64) -> f64 {
    co2_psi_c(co2_vol, t_c) * 6894.757 + P_ATM
}

/// CO2 in a `v_head_l` litre headspace in equilibrium with beer holding `co2_vol` volumes
/// at `t_c` °C, mol (spec §2.2).
#[must_use]
pub fn head_gas_mol(co2_vol: f64, t_c: f64, v_head_l: f64) -> f64 {
    p_abs_pa(co2_vol, t_c) * v_head_l / 1000.0 / (R * (t_c + 273.15))
}

/// Headspace CO2 amount and pressure of [`head_mol`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadspaceCo2 {
    /// CO2 amount, mol.
    pub mol: f64,
    /// Absolute pressure, Pa.
    pub p_abs_pa: f64,
}

/// Headspace CO2 at conditioning temperature (older definition): amount in mol and `P_abs` in Pa.
/// A purged keg starts with 1 atm CO2, which is subtracted.
#[must_use]
pub fn head_mol(co2_vol: f64, t_cond_c: f64, v_head_l: f64, purged: bool) -> HeadspaceCo2 {
    let n = head_gas_mol(co2_vol, t_cond_c, v_head_l);
    let n0 = if purged {
        P_ATM * v_head_l / 1000.0 / (R * (t_cond_c + 273.15))
    } else {
        0.0
    };
    HeadspaceCo2 {
        mol: (n - n0).max(0.0),
        p_abs_pa: p_abs_pa(co2_vol, t_cond_c),
    }
}

/// Beer volumes after the total CO2 `n_total_mol` (mol, beer plus headspace) equilibrates
/// at `t_c` °C; bisection on `[0, 10]` with 100 steps (spec §2.2).
#[must_use]
pub fn equilibrium_vol(n_total_mol: f64, t_c: f64, v_beer_l: f64, v_head_l: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, EQ_VOL_MAX);
    for _ in 0..100 {
        let mid = f64::midpoint(lo, hi);
        if total_co2(mid, t_c, v_beer_l, v_head_l) < n_total_mol {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::midpoint(lo, hi)
}

/// Total CO2, mol, in beer at `vol` volumes plus the headspace.
fn total_co2(vol: f64, t_c: f64, v_beer_l: f64, v_head_l: f64) -> f64 {
    vol * v_beer_l / VM + head_gas_mol(vol, t_c, v_head_l)
}

/// True when `n_total_mol` exceeds what 10 volumes hold, so [`equilibrium_vol`] saturates at 10
/// (T1.12 D6).
#[must_use]
pub fn equilibrium_saturated(n_total_mol: f64, t_c: f64, v_beer_l: f64, v_head_l: f64) -> bool {
    total_co2(EQ_VOL_MAX, t_c, v_beer_l, v_head_l) < n_total_mol
}

/// Keg priming mode (spec §2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KegMode {
    /// `"tam"`: full natural carbonation.
    Full,
    /// `"kismi"`: partial priming (fraction `part`) topped up with CO2.
    Partial,
}

/// Inputs of [`keg_priming`]; temperatures °C, volumes L, `limit_bar` bar (gauge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KegParams {
    /// Target CO2, volumes.
    pub co2_vol: f64,
    /// Fermentation (max) temperature for residual CO2.
    pub t_res_c: f64,
    /// Conditioning temperature.
    pub t_cond_c: f64,
    /// Beer volume.
    pub beer_l: f64,
    /// Headspace volume.
    pub head_l: f64,
    /// Priming sugar.
    pub sugar: Sugar,
    /// Keg flushed with CO2 (otherwise air).
    pub purged: bool,
    /// Full or partial priming.
    pub mode: KegMode,
    /// Fraction of the full amount in partial mode (spec: 1/3).
    pub part: f64,
    /// Serving temperature.
    pub t_serv_c: f64,
    /// Dry-matter fraction of the sugar product; `None` unknown.
    pub dry: Option<f64>,
    /// Keg working/safety pressure; `None` no check.
    pub limit_bar: Option<f64>,
}

/// Result of [`keg_priming`]; pressures bar (gauge), already rounded like the reference.
#[derive(Debug, Clone, PartialEq)]
pub struct KegPriming {
    /// Sugar to add, g (1 decimal).
    pub sugar_g: f64,
    /// Sugar for the same beer without headspace, g (1 decimal).
    pub bottle_equivalent_g: f64,
    /// Headspace surcharge, %; `None` when the bottle equivalent is 0.
    pub head_share_pct: Option<f64>,
    /// Pressure to hold the target at serving temperature, bar (2 decimals).
    pub serving_bar: f64,
    /// Beer volumes at conditioning temperature (2 decimals).
    pub peak_co2_vol: f64,
    /// CO2 (partial) gauge pressure at conditioning temperature, bar (2 decimals).
    pub peak_bar: f64,
    /// Peak gauge pressure including air in an unpurged keg, bar (2 decimals).
    pub peak_total_gauge_bar: f64,
    /// Warnings as codes with numeric fields.
    pub warnings: Vec<PrimingWarning>,
    /// CO2 top-up pressure at serving temperature, bar; only in partial mode.
    pub co2_topup_bar: Option<f64>,
}

/// Keg priming (spec §2.2–2.4): sugar so that the SERVING-temperature equilibrium gives
/// the target volumes (total CO2 conserved), plus the peak pressure at conditioning
/// temperature from the same total CO2 and a warning when it exceeds `limit_bar`.
///
/// # Errors
/// [`Error::InvalidInput`] for non-finite or out-of-range input (negative volumes,
/// `part` outside `(0, 1]`, dry matter outside `[0, 1]`); [`Error::DivisionByZero`]
/// for dry matter 0.
pub fn keg_priming(p: &KegParams) -> Result<KegPriming, Error> {
    if !finite(&[
        p.co2_vol, p.t_res_c, p.t_cond_c, p.beer_l, p.head_l, p.part, p.t_serv_c,
    ]) || p.co2_vol < 0.0
        || p.beer_l < 0.0
        || p.head_l < 0.0
        || !(p.part > 0.0 && p.part <= 1.0)
        || p.limit_bar.is_some_and(|l| !finite(&[l]))
        || p.dry
            .is_some_and(|d| !finite(&[d]) || !(0.0..=1.0).contains(&d))
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let nb = bottle_mol(p.co2_vol, p.t_res_c, p.beer_l);
    let n0 = if p.purged {
        P_ATM * p.head_l / 1000.0 / (R * (p.t_cond_c + 273.15))
    } else {
        0.0
    };
    let nh = (head_gas_mol(p.co2_vol, p.t_serv_c, p.head_l) - n0).max(0.0);
    let factor = if p.mode == KegMode::Full { 1.0 } else { p.part };
    let n_sugar_co2 = (nb + nh) * factor;
    let full = sugar_from_mol(nb + nh, p.sugar, p.dry)?;
    let bottle_same = sugar_from_mol(nb, p.sugar, p.dry)?;

    let n_total = residual_vol(p.t_res_c) * p.beer_l / VM + n_sugar_co2 + n0;
    let v_peak = equilibrium_vol(n_total, p.t_cond_c, p.beer_l, p.head_l);
    let p_peak = (p_abs_pa(v_peak, p.t_cond_c) - P_ATM) / 1e5;
    let air = if p.purged || p.head_l <= 0.0 {
        0.0
    } else {
        P_ATM / 1e5
    };
    let p_peak_total = p_peak + air;
    let p_serv = (p_abs_pa(p.co2_vol, p.t_serv_c) - P_ATM) / 1e5;

    let mut warnings = Vec::new();
    if p.dry.is_none() {
        warnings.push(PrimingWarning::DryMatterUnknown);
    }
    for (point, t_c) in [
        (PolyPoint::Conditioning, p.t_cond_c),
        (PolyPoint::Serving, p.t_serv_c),
    ] {
        if t_c < POLY_T_MIN_C {
            warnings.push(PrimingWarning::PolynomialBelowTemperature { point, t_c });
        }
    }
    for (point, vol) in [(PolyPoint::Target, p.co2_vol), (PolyPoint::Peak, v_peak)] {
        if vol > POLY_VOL_MAX + TOL {
            warnings.push(PrimingWarning::PolynomialAboveVolume {
                point,
                co2_vol: vol,
            });
        }
    }
    if equilibrium_saturated(n_total, p.t_cond_c, p.beer_l, p.head_l) {
        warnings.push(PrimingWarning::PeakVolumeSaturated {
            limit_co2_vol: EQ_VOL_MAX,
        });
    }
    if let Some(limit) = p.limit_bar
        && p_peak_total > limit + TOL
    {
        warnings.push(PrimingWarning::PeakAboveLimit {
            peak_bar: p_peak_total,
            limit_bar: limit,
            t_cond_c: p.t_cond_c,
        });
    }
    Ok(KegPriming {
        sugar_g: round_py(
            if p.mode == KegMode::Full {
                full
            } else {
                full * p.part
            },
            1,
        ),
        bottle_equivalent_g: round_py(bottle_same, 1),
        head_share_pct: (bottle_same > 0.0)
            .then(|| round_py(100.0 * (full / bottle_same - 1.0), 1)),
        serving_bar: round_py(p_serv, 2),
        peak_co2_vol: round_py(v_peak, 2),
        peak_bar: round_py(p_peak, 2),
        peak_total_gauge_bar: round_py(p_peak_total, 2),
        warnings,
        co2_topup_bar: (p.mode == KegMode::Partial)
            .then(|| round_py(co2_psi_c(p.co2_vol, p.t_serv_c) * 0.068_947_6, 2)),
    })
}

// ================================================================ 3. Cask

/// CO2 from remaining extract when racking at `sg_rack` with reference `fg_ref`,
/// volumes (spec §3.2); 0 when `u(sg_rack) ≤ u(fg_ref)`.
#[must_use]
pub fn co2_from_remaining(sg_rack: f64, fg_ref: f64) -> f64 {
    if gravity_units(sg_rack) <= gravity_units(fg_ref) {
        return 0.0;
    }
    let d_ae = to_plato(sg_rack) - to_plato(fg_ref);
    let d_re_g_l = 0.8192 * d_ae * 10.0 * sg_rack;
    d_re_g_l * BALLING_CO2 / CO2_G_PER_VOL
}

/// Result of [`cask_priming`] (rounded like the reference).
#[derive(Debug, Clone, PartialEq)]
pub struct CaskPriming {
    /// Residual CO2 at fermentation temperature, volumes (2 decimals).
    pub residual_co2_vol: f64,
    /// CO2 from remaining extract, volumes (2 decimals).
    pub from_extract_co2_vol: f64,
    /// Missing CO2, volumes (2 decimals); negative means excess.
    pub missing_co2_vol: f64,
    /// Priming sugar, g (1 decimal).
    pub sugar_g: f64,
    /// Priming sugar, g/L (2 decimals).
    pub g_per_l: f64,
    /// True when there is excess CO2 (no sugar advised).
    pub excess: bool,
    /// Warnings as codes.
    pub warnings: Vec<PrimingWarning>,
}

/// Inputs of [`cask_priming`].
#[derive(Debug, Clone, Copy)]
pub struct CaskPrimingInput {
    /// Batch volume, L.
    pub volume_l: f64,
    /// Target carbonation, volumes of CO2.
    pub co2_vol: f64,
    /// Maximum fermentation temperature, °C.
    pub t_res_c: f64,
    /// Gravity at racking, SG.
    pub sg_rack: f64,
    /// Reference final gravity, SG.
    pub fg_ref: f64,
    /// Priming sugar.
    pub sugar: Sugar,
    /// Dry-matter fraction of the sugar product; `None` when unknown.
    pub dry: Option<f64>,
}

/// Cask priming (spec §3.3). `v_l` litres, `vol_t` target volumes, `t_res_c` max
/// fermentation temperature °C, gravities as SG. Spec default sugar is invert.
/// `v_l = 0` is a `ZeroDivision` error, as in the reference (`g / V`).
///
/// # Errors
/// [`Error::InvalidInput`] for non-finite or out-of-range input (negative volume,
/// dry matter outside `[0, 1]`).
pub fn cask_priming(p: &CaskPrimingInput) -> Result<CaskPriming, Error> {
    let CaskPrimingInput {
        volume_l: v_l,
        co2_vol: vol_t,
        t_res_c,
        sg_rack,
        fg_ref,
        sugar,
        dry,
    } = *p;
    if !finite(&[v_l, vol_t, t_res_c, sg_rack, fg_ref])
        || v_l < 0.0
        || dry.is_some_and(|d| !finite(&[d]) || !(0.0..=1.0).contains(&d))
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let res = residual_vol(t_res_c);
    let from_x = co2_from_remaining(sg_rack, fg_ref);
    let need = vol_t - res - from_x;
    let g = sugar_from_mol(need.max(0.0) * v_l / VM, sugar, dry)?;
    if v_l == 0.0 {
        return Err(Error::DivisionByZero { field: "divisor" }); // g / V in the reference
    }
    Ok(CaskPriming {
        residual_co2_vol: round_py(res, 2),
        from_extract_co2_vol: round_py(from_x, 2),
        missing_co2_vol: round_py(need, 2),
        sugar_g: round_py(g, 1),
        g_per_l: round_py(g / v_l, 2),
        excess: need < 0.0,
        warnings: if dry.is_none() {
            vec![PrimingWarning::DryMatterUnknown]
        } else {
            Vec::new()
        },
    })
}

/// Finings dose, mL: `v_l × rate` (spec §3.4; default rate 2 mL/L isinglass).
#[must_use]
pub fn finings_ml(v_l: f64, rate_ml_per_l: f64) -> f64 {
    v_l * rate_ml_per_l
}

/// Cask conditioning days (spec §3.5): `base × 2^((t0 − t_cellar)/10)`; defaults 3 days at 12 °C.
#[must_use]
pub fn cask_days(t_cellar_c: f64, base_days: f64, t0_c: f64) -> f64 {
    base_days * 2.0_f64.powf((t0_c - t_cellar_c) / 10.0)
}

// ================================================================ 4. Defaults and progress (T1.14d)

/// Default Δ for a lager diacetyl rest, SG (spec §1.2; to be verified).
pub const DELTA_LAGER: f64 = 0.004;
/// Default Δ for an ale, SG (spec §1.2; to be verified).
pub const DELTA_ALE: f64 = 0.002;
/// Default lager cooling rate, °C/day (spec §1.5; to be verified).
pub const LAGER_COOL_RATE: f64 = 2.0;
/// Default fraction of the full sugar amount in partial keg mode (spec §2.4).
pub const PARTIAL_FRACTION: f64 = 1.0 / 3.0;
/// Default cask target CO2, volumes (spec §3.1).
pub const CASK_TARGET_VOL: f64 = 1.1;
/// Default finings dose, mL per litre (spec §3.4).
pub const FININGS_ML_PER_L: f64 = 2.0;

/// Fermentation type selecting the default trigger Δ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FermentationType {
    /// `"lager"`.
    Lager,
    /// `"ale"`.
    Ale,
}

/// Default Δ (SG) for a fermentation type (spec §1.2).
#[must_use]
pub fn default_delta(kind: FermentationType) -> f64 {
    match kind {
        FermentationType::Lager => DELTA_LAGER,
        FermentationType::Ale => DELTA_ALE,
    }
}

/// Progress ratio `(OG − SG) / (OG − FG_ref)` in integer gravity units (0–1 scale,
/// multiply by 100 for percent; spec §1.1–1.2). `None` when `u(OG) ≤ u(FG_ref)`
/// (non-positive denominator). Not clamped.
#[must_use]
pub fn progress(og: f64, sg: f64, fg_ref: f64) -> Option<f64> {
    let den = gravity_units(og) - gravity_units(fg_ref);
    if den <= 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    Some((gravity_units(og) - gravity_units(sg)) as f64 / den as f64)
}
