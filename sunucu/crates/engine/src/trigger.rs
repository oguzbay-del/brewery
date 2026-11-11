//! Section 11: gravity triggers, keg priming, cask (`formuller/11-tetikleyici-fici-cask.md`).
//!
//! Units follow the specification of each function; see the section file. Metric:
//! °C, L, g, hours; gravity as SG; pressure in bar (gauge) unless a name ends in `_pa`.
//! Gravity thresholds are compared in integer units (`units::gravity_units`,
//! 1 unit = 0.00001 SG) per `formuller/00-OKU-BENI.md`. The CO2 tables of section 04
//! (`yeast::residual_co2`, `yeast::co2_psi`) take °F; this module converts with
//! `units::c_to_f`.

use crate::units::{TOL, c_to_f, gravity_units, round_py, to_plato};
use crate::yeast;

/// Warning text when a sugar product's dry-matter fraction is unknown (spec §2.3).
pub const WARN_DRY_UNKNOWN: &str =
    "kuru madde bilinmiyor: 1.0 varsayıldı (şurup ve bloklarda şeker miktarı eksik çıkabilir)";

/// Errors mirroring the exceptions of the Python reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerError {
    /// Python `IndexError`: an empty series where at least one point is required.
    EmptySeries,
    /// Python `ZeroDivisionError`: a divisor (rate, rate constant, dry matter) is zero.
    ZeroDivision,
}

// ================================================================ 1. Gravity trigger

/// Slope limit for "FG stable": 0.0005 SG per day (spec §1.4).
pub const STABLE_PER_DAY: f64 = 0.0005;

/// First trigger time (spec §1.2): hourly medians `(t_hours, SG)`; the condition
/// `u(SG) ≤ u(fg_ref) + u(delta)` must hold at `consecutive` consecutive points
/// (at least 1). Returns the time of the first point of that run, `None` if never.
#[must_use]
pub fn trigger_index(
    points: &[(f64, f64)],
    fg_ref: f64,
    delta: f64,
    consecutive: usize,
) -> Option<f64> {
    let consecutive = consecutive.max(1);
    let limit = gravity_units(fg_ref) + gravity_units(delta);
    let mut run = 0_usize;
    for (i, &(_, g)) in points.iter().enumerate() {
        run = if gravity_units(g) <= limit {
            run + 1
        } else {
            0
        };
        if run >= consecutive {
            return Some(points[i + 1 - consecutive].0);
        }
    }
    None
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

/// Trigger time for `SG(t) = fg_inf + a·e^(−k·t)` reaching `fg_ref + delta` (spec §1.3).
/// SG inputs, `a` in SG, `k` in 1/hour; result in hours.
pub fn predicted_trigger(
    fg_inf: f64,
    a: f64,
    k: f64,
    fg_ref: f64,
    delta: f64,
) -> Result<TriggerForecast, TriggerError> {
    let target_u = gravity_units(fg_ref) + gravity_units(delta) - gravity_units(fg_inf);
    if target_u <= 0 {
        return Ok(TriggerForecast::Unreachable);
    }
    if gravity_units(a) <= target_u {
        return Ok(TriggerForecast::AlreadyPast);
    }
    if k == 0.0 {
        return Err(TriggerError::ZeroDivision);
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

/// Reference FG (spec §1.1): the forecast `fg_inf` replaces the recipe FG when `shown`
/// and it deviates by more than `delta − 1 point` in either direction (integer units).
#[must_use]
pub fn fg_reference(
    fg_recipe: f64,
    fg_inf: Option<f64>,
    shown: bool,
    delta: f64,
) -> (f64, FgSource) {
    if let (true, Some(inf)) = (shown, fg_inf) {
        let margin = gravity_units(delta) - 100;
        let recipe_u = gravity_units(fg_recipe);
        let inf_u = gravity_units(inf);
        if inf_u > recipe_u + margin || inf_u < recipe_u - margin {
            return (inf, FgSource::Forecast);
        }
    }
    (fg_recipe, FgSource::Recipe)
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
pub fn is_stable(
    points: &[(f64, f64)],
    hours: f64,
    min_n: usize,
) -> Result<Stability, TriggerError> {
    let t_end = points.last().ok_or(TriggerError::EmptySeries)?.0;
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
pub fn ramp_hours(t_from: f64, t_to: f64, rate_c_per_day: f64) -> Result<f64, TriggerError> {
    if rate_c_per_day == 0.0 {
        return Err(TriggerError::ZeroDivision);
    }
    Ok((t_to - t_from).abs() / rate_c_per_day * 24.0)
}

/// Lagering weeks (spec §1.5): OG converted to integer points (0.01-point units,
/// `round((OG − 1) × 100000)`), divided by `pts_per_week` points and rounded up,
/// clamped to `[lo, hi]` weeks. Defaults in the spec: 8, 2, 12.
pub fn lager_weeks(og: f64, pts_per_week: i64, lo: i64, hi: i64) -> Result<i64, TriggerError> {
    #[allow(clippy::cast_possible_truncation)]
    let pts = ((og - 1.0) * 100_000.0).round_ties_even() as i64;
    let d = pts_per_week * 100;
    if d == 0 {
        return Err(TriggerError::ZeroDivision);
    }
    let w = -(-pts).div_euclid(d); // ceil(pts / d) for d > 0
    Ok(hi.min(lo.max(w)))
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

/// Keg pressure, psi, to hold `vols` volumes at `t_c` °C (section 04 §3.5).
#[must_use]
pub fn co2_psi_c(vols: f64, t_c: f64) -> f64 {
    yeast::co2_psi(vols, c_to_f(t_c))
}

/// Sugar mass, g, for `mol` mol CO2 (spec §2.3). `dry` is the dry-matter fraction
/// (0–1); `None` means unknown and 1.0 is used (callers add [`WARN_DRY_UNKNOWN`]).
pub fn sugar_from_mol(mol: f64, sugar: Sugar, dry: Option<f64>) -> Result<f64, TriggerError> {
    let d = dry.unwrap_or(1.0);
    if d == 0.0 {
        return Err(TriggerError::ZeroDivision);
    }
    Ok(mol / sugar.co2_per_mol() * sugar.molar_mass() / d)
}

/// CO2 needed in the beer itself, mol: `max(0, target − residual(t_res)) × v_beer / 22.414`.
#[must_use]
pub fn bottle_mol(vol_t: f64, t_res_c: f64, v_beer_l: f64) -> f64 {
    (vol_t - residual_vol(t_res_c)).max(0.0) * v_beer_l / VM
}

/// Absolute CO2 partial pressure, Pa, for `vol` volumes at `t_c` °C (gauge table + 1 atm).
#[must_use]
pub fn p_abs_pa(vol: f64, t_c: f64) -> f64 {
    co2_psi_c(vol, t_c) * 6894.757 + P_ATM
}

/// CO2 in a `v_head_l` litre headspace in equilibrium with beer holding `vol` volumes
/// at `t_c` °C, mol (spec §2.2).
#[must_use]
pub fn head_gas_mol(vol: f64, t_c: f64, v_head_l: f64) -> f64 {
    p_abs_pa(vol, t_c) * v_head_l / 1000.0 / (R * (t_c + 273.15))
}

/// Headspace CO2 at conditioning temperature (older definition), `(mol, P_abs Pa)`.
/// A purged keg starts with 1 atm CO2, which is subtracted.
#[must_use]
pub fn head_mol(vol_t: f64, t_cond_c: f64, v_head_l: f64, purged: bool) -> (f64, f64) {
    let n = head_gas_mol(vol_t, t_cond_c, v_head_l);
    let n0 = if purged {
        P_ATM * v_head_l / 1000.0 / (R * (t_cond_c + 273.15))
    } else {
        0.0
    };
    ((n - n0).max(0.0), p_abs_pa(vol_t, t_cond_c))
}

/// Beer volumes after the total CO2 `n_total` (mol, beer plus headspace) equilibrates
/// at `t_c` °C; bisection on `[0, 10]` with 100 steps (spec §2.2).
#[must_use]
pub fn equilibrium_vol(n_total: f64, t_c: f64, v_beer_l: f64, v_head_l: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 10.0_f64);
    for _ in 0..100 {
        let mid = f64::midpoint(lo, hi);
        let tot = mid * v_beer_l / VM + head_gas_mol(mid, t_c, v_head_l);
        if tot < n_total {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::midpoint(lo, hi)
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
    pub vol_t: f64,
    /// Fermentation (max) temperature for residual CO2.
    pub t_res: f64,
    /// Conditioning temperature.
    pub t_cond: f64,
    /// Beer volume.
    pub v_beer: f64,
    /// Headspace volume.
    pub v_head: f64,
    /// Priming sugar.
    pub sugar: Sugar,
    /// Keg flushed with CO2 (otherwise air).
    pub purged: bool,
    /// Full or partial priming.
    pub mode: KegMode,
    /// Fraction of the full amount in partial mode (spec: 1/3).
    pub part: f64,
    /// Serving temperature.
    pub t_serv: f64,
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
    pub peak_vol: f64,
    /// CO2 (partial) gauge pressure at conditioning temperature, bar (2 decimals).
    pub peak_bar: f64,
    /// Peak gauge pressure including air in an unpurged keg, bar (2 decimals).
    pub peak_total_gauge_bar: f64,
    /// Warnings.
    pub warnings: Vec<String>,
    /// CO2 top-up pressure at serving temperature, bar; only in partial mode.
    pub co2_topup_bar: Option<f64>,
}

/// Keg priming (spec §2.2–2.4): sugar so that the SERVING-temperature equilibrium gives
/// the target volumes (total CO2 conserved), plus the peak pressure at conditioning
/// temperature from the same total CO2 and a warning when it exceeds `limit_bar`.
pub fn keg_priming(p: &KegParams) -> Result<KegPriming, TriggerError> {
    let nb = bottle_mol(p.vol_t, p.t_res, p.v_beer);
    let n0 = if p.purged {
        P_ATM * p.v_head / 1000.0 / (R * (p.t_cond + 273.15))
    } else {
        0.0
    };
    let nh = (head_gas_mol(p.vol_t, p.t_serv, p.v_head) - n0).max(0.0);
    let factor = if p.mode == KegMode::Full { 1.0 } else { p.part };
    let n_sugar_co2 = (nb + nh) * factor;
    let full = sugar_from_mol(nb + nh, p.sugar, p.dry)?;
    let bottle_same = sugar_from_mol(nb, p.sugar, p.dry)?;

    let n_total = residual_vol(p.t_res) * p.v_beer / VM + n_sugar_co2 + n0;
    let v_peak = equilibrium_vol(n_total, p.t_cond, p.v_beer, p.v_head);
    let p_peak = (p_abs_pa(v_peak, p.t_cond) - P_ATM) / 1e5;
    let air = if p.purged || p.v_head <= 0.0 {
        0.0
    } else {
        P_ATM / 1e5
    };
    let p_peak_total = p_peak + air;
    let p_serv = (p_abs_pa(p.vol_t, p.t_serv) - P_ATM) / 1e5;

    let mut warnings = Vec::new();
    if p.dry.is_none() {
        warnings.push(WARN_DRY_UNKNOWN.to_owned());
    }
    if let Some(limit) = p.limit_bar
        && p_peak_total > limit
    {
        warnings.push(format!(
            "tepe basıncı fıçı sınırını aşıyor: {p_peak_total:.2} bar > {limit:.2} bar (kondisyon {:.0} °C)",
            p.t_cond
        ));
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
        peak_vol: round_py(v_peak, 2),
        peak_bar: round_py(p_peak, 2),
        peak_total_gauge_bar: round_py(p_peak_total, 2),
        warnings,
        co2_topup_bar: (p.mode == KegMode::Partial)
            .then(|| round_py(co2_psi_c(p.vol_t, p.t_serv) * 0.068_947_6, 2)),
    })
}

// ================================================================ 3. Cask

/// Balling: 0.9565 g CO2 per 2.0665 g extract (spec §3.2).
pub const CO2_PER_EXTRACT: f64 = 0.9565 / 2.0665;
/// Grams of CO2 per litre per volume: 44.01 / 22.414 = 1.9635.
pub const G_PER_VOL: f64 = 44.01 / VM;

/// CO2 from remaining extract when racking at `sg_rack` with reference `fg_ref`,
/// volumes (spec §3.2); 0 when `u(sg_rack) ≤ u(fg_ref)`.
#[must_use]
pub fn co2_from_remaining(sg_rack: f64, fg_ref: f64) -> f64 {
    if gravity_units(sg_rack) <= gravity_units(fg_ref) {
        return 0.0;
    }
    let d_ae = to_plato(sg_rack) - to_plato(fg_ref);
    let d_re_g_l = 0.8192 * d_ae * 10.0 * sg_rack;
    d_re_g_l * CO2_PER_EXTRACT / G_PER_VOL
}

/// Result of [`cask_priming`] (rounded like the reference).
#[derive(Debug, Clone, PartialEq)]
pub struct CaskPriming {
    /// Residual CO2 at fermentation temperature, volumes (2 decimals).
    pub residual_vol: f64,
    /// CO2 from remaining extract, volumes (2 decimals).
    pub from_extract_vol: f64,
    /// Missing CO2, volumes (2 decimals); negative means excess.
    pub missing_vol: f64,
    /// Priming sugar, g (1 decimal).
    pub sugar_g: f64,
    /// Priming sugar, g/L (2 decimals).
    pub g_per_l: f64,
    /// True when there is excess CO2 (no sugar advised).
    pub excess: bool,
    /// Warnings.
    pub warnings: Vec<String>,
}

/// Cask priming (spec §3.3). `v_l` litres, `vol_t` target volumes, `t_res_c` max
/// fermentation temperature °C, gravities as SG. Spec default sugar is invert.
/// `v_l = 0` is a `ZeroDivision` error, as in the reference (`g / V`).
pub fn cask_priming(
    v_l: f64,
    vol_t: f64,
    t_res_c: f64,
    sg_rack: f64,
    fg_ref: f64,
    sugar: Sugar,
    dry: Option<f64>,
) -> Result<CaskPriming, TriggerError> {
    let res = residual_vol(t_res_c);
    let from_x = co2_from_remaining(sg_rack, fg_ref);
    let need = vol_t - res - from_x;
    let g = sugar_from_mol(need.max(0.0) * v_l / VM, sugar, dry)?;
    if v_l == 0.0 {
        return Err(TriggerError::ZeroDivision); // g / V in the reference
    }
    Ok(CaskPriming {
        residual_vol: round_py(res, 2),
        from_extract_vol: round_py(from_x, 2),
        missing_vol: round_py(need, 2),
        sugar_g: round_py(g, 1),
        g_per_l: round_py(g / v_l, 2),
        excess: need < 0.0,
        warnings: if dry.is_none() {
            vec![WARN_DRY_UNKNOWN.to_owned()]
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
