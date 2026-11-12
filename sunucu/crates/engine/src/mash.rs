//! Section 05: mash and volumes (`formuller/05-mash-hacimler.md`).
//!
//! Ported from the Python reference `formuller/testler/05_mash_tv.py`, which is the source of
//! truth. Units are metric here: volumes in litres, masses in kilograms, temperatures in °C,
//! specific heats in cal/g°C, gravities as SG. Conversion from BeerSmith's internal units is
//! the API layer's job (ADR-0003).

use crate::density::{GrainType, GrainUse, RecipeType};
use crate::error::{Error, Result, ensure_finite};
use crate::units::round_py;

/// One US fluid ounce in litres (`O_GRAIN_VOL`-style internal conversion, spec §0.1).
pub const FLOZ_L: f64 = 29.573_529_56 / 1000.0;
/// One pound in kilograms.
pub const LB_KG: f64 = 0.453_592_4;
/// One ounce in kilograms.
pub const OZ_KG: f64 = LB_KG / 16.0;
/// Volume occupied by grain, `O_GRAIN_VOL` (L/kg, spec §1.1).
pub const GRAIN_VOL: f64 = 0.652;
/// Malt specific heat, `O_MALT_SPECIFIC_HEAT` (cal/g°C, spec §2).
pub const C_MALT: f64 = 0.380;
/// Standard grain absorption, `O_ADV_GRAIN_ABSORB` (L/kg, spec §1.2).
pub const ABS_STD: f64 = 0.96 * FLOZ_L / OZ_KG;
/// BIAB grain absorption, `O_ADV_BIAB_ABSORB` (L/kg, spec §1.2).
pub const ABS_BIAB: f64 = 0.586 * FLOZ_L / OZ_KG;

/// Mash tun thermal data. [`Tun::NONE`] ignores the tun (spec §2, `F_MH_EQUIP_ADJUST = 0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tun {
    /// Tun mass `M_t` (kg).
    pub mass_kg: f64,
    /// Tun specific heat `c_t` (cal/g°C).
    pub specific_heat: f64,
    /// Tun volume `V_t` (L).
    pub volume_l: f64,
}

impl Tun {
    /// No tun heat capacity.
    pub const NONE: Tun = Tun {
        mass_kg: 0.0,
        specific_heat: 0.0,
        volume_l: 0.0,
    };
}

/// Effective tun mass with the partial-fill correction (spec §2.0).
///
/// `m_tun` kg, `v_mash` L, `v_tun` L. Returns kg: `m_tun · v_mash / (0.8 · v_tun)` when the
/// mash fills less than 80 % of the tun, otherwise `m_tun`.
#[must_use]
pub fn tun_eff(m_tun: f64, v_mash: f64, v_tun: f64) -> f64 {
    let cap = 0.8 * v_tun;
    if cap > 0.0 && v_mash < cap {
        m_tun * v_mash / cap
    } else {
        m_tun
    }
}

/// Strike water temperature (spec §2.1, `MashInTemp`).
///
/// `vw` water (L), `t_step` target step temperature (°C), `w` grain (kg), `t_grain` grain
/// temperature (°C), `t_tun` tun pre-heat temperature (°C; `None` = grain temperature).
/// Returns °C, or `None` when `vw == 0`.
#[must_use]
pub fn strike_temp(
    vw: f64,
    t_step: f64,
    w: f64,
    t_grain: f64,
    tun: Tun,
    t_tun: Option<f64>,
) -> Option<f64> {
    #[allow(clippy::float_cmp)]
    if vw == 0.0 {
        return None;
    }
    let t_tun = t_tun.unwrap_or(t_grain);
    let me = tun_eff(tun.mass_kg, GRAIN_VOL * w + vw, tun.volume_l);
    Some(
        t_step + ((t_step - t_tun) * tun.specific_heat * me + C_MALT * w * (t_step - t_grain)) / vw,
    )
}

/// Infusion water temperature needed to raise the mash from `t1` to `t2` (spec §2.2,
/// `InfuseTemp`).
///
/// `va` added water (L), `t2` target (°C), `vw` water already in the mash (L), `t1` current
/// mash temperature (°C), `w` grain (kg). Returns °C, or `None` when `va <= 0`.
#[must_use]
pub fn infusion_temp(va: f64, t2: f64, vw: f64, t1: f64, w: f64, tun: Tun) -> Option<f64> {
    if va <= 0.0 {
        return None;
    }
    let me = tun_eff(tun.mass_kg, GRAIN_VOL * w + va + vw, tun.volume_l);
    Some(t2 + (t2 - t1) * (tun.specific_heat * me + C_MALT * w + vw) / va)
}

/// Amount of water at `t_inf` to add to move the mash from `t1` to `t2` (spec §2.3,
/// `InfuseAmount`).
///
/// `t_inf`, `t2`, `t1` in °C, `vw` L, `w` kg. Returns litres; 0 unless `t2` lies strictly
/// between `t1` and `t_inf`.
#[must_use]
pub fn infusion_amount(t_inf: f64, t2: f64, vw: f64, t1: f64, w: f64, tun: Tun) -> f64 {
    if !(t1.min(t_inf) < t2 && t2 < t1.max(t_inf)) {
        return 0.0;
    }
    let me = tun_eff(tun.mass_kg, GRAIN_VOL * w + vw, tun.volume_l);
    (t2 - t1) * (tun.specific_heat * me + C_MALT * w + vw) / (t_inf - t2)
}

/// Mash volume to pull and boil for a decoction step (spec §2.4, `DecoctVol`).
///
/// `t2` target, `t1` current, `tb` boiling temperature (°C; reference default 100), `vw`
/// water (L), `w` grain (kg). Returns litres of mash (grain + water), at most the whole mash;
/// 0 when `t1 >= tb` or when there is neither grain nor water.
#[must_use]
pub fn decoct_vol(t2: f64, vw: f64, t1: f64, w: f64, tun: Tun, tb: f64) -> f64 {
    #[allow(clippy::float_cmp)]
    if t1 >= tb || (w == 0.0 && vw == 0.0) {
        return 0.0;
    }
    let vm = GRAIN_VOL * w + vw;
    let me = tun_eff(tun.mass_kg, vm, tun.volume_l);
    let f = (t2 - t1) / (tb - t1) * (tun.specific_heat * me + C_MALT * w + vw) / (C_MALT * w + vw);
    vm * f.min(1.0)
}

/// Boiling point of water at altitude (spec §6.1, `BoilTemp`).
///
/// `alt_ft` feet above sea level; returns °C.
#[must_use]
pub fn boil_temp_c(alt_ft: f64) -> f64 {
    100.0 - alt_ft * 0.3048 * 0.00324
}

/// Pre-boil volume from the equipment chain (spec §5.2, `F_E_BOIL_VOL`).
///
/// `batch` into fermenter, `topup` fermenter top-up, `trub` trub loss, `boil_off_total`
/// total boil-off (all L), `cool_pct` cooling shrinkage in percent (capped at 99).
/// Returns litres.
#[must_use]
pub fn equip_boil_vol(
    batch: f64,
    topup: f64,
    trub: f64,
    cool_pct: f64,
    boil_off_total: f64,
) -> f64 {
    let c = cool_pct.min(99.0);
    boil_off_total + (batch - topup + trub) / (1.0 - c / 100.0)
}

/// Batch sparge inputs shared by [`batch_sparge`] and [`sparge_plan`] (spec §4.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpargeParams {
    /// Sparge water `S` (L); negative values are clipped to 0.
    pub sparge_vol: f64,
    /// Total grain absorption `A_t` (L).
    pub absorb: f64,
    /// Grain weight `W` (kg).
    pub grain_kg: f64,
    /// Mash tun volume `V_tun` (L).
    pub tun_vol: f64,
    /// Batch fill percentage `P` (percent of the tun).
    pub fill_pct: f64,
    /// Total mash water (L).
    pub mash_water: f64,
    /// Equal-size batches (`F_MH_BATCH_EVEN`).
    pub even: bool,
    /// Drain first (`F_MH_BATCH_DRAIN`).
    pub drain: bool,
}

/// One batch sparge step: the size and whether it is the "drain mash tun" step.
///
/// `GetBatchSpargeSize(i)` (spec §4.2). `i` is the step index (0 = before the first runoff).
/// Returns `(volume in L, drain_flag)`.
#[must_use]
#[allow(clippy::many_single_char_names, clippy::manual_midpoint)] // names follow spec §4.2; midpoint kept literal for parity
pub fn batch_sparge(i: u32, p: &SpargeParams) -> (f64, bool) {
    let mut s = p.sparge_vol.max(0.0);
    let vg = GRAIN_VOL * p.grain_kg;
    let mut cap = p.fill_pct * p.tun_vol / 100.0 - vg - p.absorb;
    if cap < 0.0 {
        cap = p.fill_pct * p.tun_vol / 100.0;
    }
    let m = p.mash_water - p.absorb;
    let r = cap - m;
    if !(i != 0 || (r > 0.0 && !p.drain)) {
        return (m, true);
    }
    if p.even && !p.drain {
        let mut n = 2.0_f64;
        let split_total = r >= 0.0 || i < 1;
        let mut size = if split_total { (s + m) / 2.0 } else { s };
        while cap < size && n < 10.0 {
            n += 1.0;
            size = if split_total {
                (s + m) / n
            } else {
                s / (n - 1.0)
            };
        }
        if i == 0 {
            return if size - m >= 0.0 {
                (size - m, false)
            } else {
                (m, true)
            };
        }
        // Python: int(n + 0.3) <= i, n is a small positive float.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let batches = (n + 0.3) as u32;
        return (if batches <= i { 0.0 } else { size }, false);
    }
    let r = r.max(0.0);
    if i == 0 {
        return (s.min(r), false);
    }
    if !p.drain && !p.even {
        s -= r;
    }
    if !p.even {
        let k = f64::from(i - 1) * cap;
        if k <= s {
            return (cap.min(s - k), false);
        }
        return (0.0, false);
    }
    if cap <= 0.0 {
        cap = 1.0;
    }
    // Python: int(S / cap + 0.99) truncates toward zero; S >= 0 here.
    #[allow(clippy::cast_possible_truncation)]
    let n = (s / cap + 0.99) as i64;
    if n < i64::from(i) {
        (0.0, false)
    } else {
        #[allow(clippy::cast_precision_loss)]
        let nf = n as f64;
        (s / nf, false)
    }
}

/// What a sparge plan step asks the brewer to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpargeAction {
    /// "DRAIN(ilk süzme)": drain the mash tun (first runoff).
    Drain,
    /// "EKLE": add the water volume.
    Add,
}

impl SpargeAction {
    /// Label used by the Python reference.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SpargeAction::Drain => "DRAIN(ilk süzme)",
            SpargeAction::Add => "EKLE",
        }
    }
}

/// One step of a batch sparge plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpargeStep {
    /// Step index, from 0.
    pub index: u32,
    /// Volume in litres, rounded to 3 decimals like the reference.
    pub volume: f64,
    /// Action for this step.
    pub action: SpargeAction,
}

/// Full batch sparge plan (spec §4.2): steps `0..50`, stopping at the first step with no
/// volume that is not a drain.
#[must_use]
pub fn sparge_plan(p: &SpargeParams) -> Vec<SpargeStep> {
    let mut out = Vec::new();
    for index in 0..50 {
        let (v, flag) = batch_sparge(index, p);
        if !flag && v <= 0.0 {
            break;
        }
        out.push(SpargeStep {
            index,
            volume: round_py(v, 3),
            action: if flag {
                SpargeAction::Drain
            } else {
                SpargeAction::Add
            },
        });
    }
    out
}

/// Parti-gyle split result (spec §7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parti {
    /// First-runnings volume (L).
    pub vol1: f64,
    /// Second-runnings volume (L).
    pub vol2: f64,
    /// First-runnings gravity (SG).
    pub og1: f64,
    /// Second-runnings gravity (SG).
    pub og2: f64,
}

/// Parti-gyle split (spec §7, `PartiFrac2`, `PartiVol1/2`, `PartiOG1/2`).
///
/// `og` unsplit gravity (SG), `vol` total volume (L), `f1` first-runnings share in percent.
#[must_use]
pub fn parti(og: f64, vol: f64, f1: f64) -> Parti {
    let k = 0.5_f64.powf(3.0 * f1 / 100.0);
    Parti {
        vol1: vol * f1 / 100.0,
        vol2: vol * (100.0 - f1) / 100.0,
        og1: 1.0 + (og - 1.0) * (1.0 - 0.5 * k),
        og2: 1.0 + (og - 1.0) * 0.5 * k,
    }
}

// ---------------------------------------------------------------------------------------------
// T1.14b: calculations that the spec has but the reference lacked until `05_mash_tv.py` got
// them appended (same names, same order of operations). Metric units: L, kg, °C.
// ---------------------------------------------------------------------------------------------

/// Infusion water above this temperature (°C) is an error in BeerSmith (spec §2.2).
pub const ABOVE_BOILING_C: f64 = 100.0;
/// `GetMainMashThickness` constant, fl oz/oz → L/kg (spec §2.8; deliberately not the exact
/// unit ratio, see §11 item 5).
pub const THICKNESS_CONST: f64 = 1.043_192_807_606_429;
/// Reference temperature for the main mash step, `O_MASH_FG_ADJ_TEMP` (°F, spec §0.2).
pub const MASH_FG_ADJ_TEMP_F: f64 = 153.5;
/// Steep water per grain, `O_ADV_STEEP_WATER_GRAIN` (qt/lb, spec §0.2).
pub const STEEP_QT_PER_LB: f64 = 4.0;

/// Mash step type, `F_MS_TYPE` (spec §0.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// 0: add water.
    Infusion,
    /// 1: pull and boil part of the mash.
    Decoction,
    /// 2: direct heating (no temperature calculation).
    Temperature,
}

impl StepKind {
    /// Maps the spec's numeric code (0, 1, 2); anything else is an error.
    pub const fn from_code(code: i64) -> Result<StepKind> {
        match code {
            0 => Ok(StepKind::Infusion),
            1 => Ok(StepKind::Decoction),
            2 => Ok(StepKind::Temperature),
            _ => Err(Error::invalid("step_kind", "unknown code")),
        }
    }
}

/// A mash step as entered by the user (spec §2.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepInput {
    /// Step type.
    pub kind: StepKind,
    /// `F_MS_INFUSION`: water added in this step (L).
    pub infusion: f64,
    /// Target step temperature (°C).
    pub t_step: f64,
}

/// Per-step output of [`recalc_mash`] and [`recalc_biab`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepResult {
    /// `F_MS_INFUSION` (L); for BIAB the first step's value is computed.
    pub infusion: f64,
    /// `F_MS_TUN_ADDITION` (L).
    pub tun_addition: f64,
    /// `F_MS_TOT_INFUSION` = water added in this step including the tun addition (L).
    pub tot_infusion: f64,
    /// `F_MS_START_VOL`: water already in the mash (L).
    pub start_vol: f64,
    /// `F_MS_START_TEMP` (°C); 0 for the first step.
    pub start_temp: f64,
    /// Infusion water temperature (°C): 0 for decoction steps after the first, `None` when it
    /// is not calculated (temperature steps, zero water).
    pub infusion_temp: Option<f64>,
    /// `F_MS_DECOCTION_AMT`: mash to pull (L).
    pub decoction_amt: f64,
    /// Infusion step whose water temperature is above [`ABOVE_BOILING_C`] (spec §2.2).
    pub above_boiling: bool,
    /// Decoction step with no water in the mash (`start_vol <= 0`): BeerSmith's "Decoction
    /// Error: Must have some water in mash to Decoct it!" (spec §2.4).
    pub no_water: bool,
}

/// Result of [`infusion_temp_checked`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfusionCheck {
    /// Infusion water temperature (°C).
    pub temp_c: f64,
    /// True when `temp_c` is above [`ABOVE_BOILING_C`] ("Infusion temperature above boiling.
    /// Add more water!", spec §2.2).
    pub above_boiling: bool,
}

/// Volume the grain occupies in the mash (spec §1.1, `GrainVol`): `w` kg → litres.
#[must_use]
pub fn grain_vol(w: f64) -> f64 {
    GRAIN_VOL * w
}

/// Total mash volume (spec §1.1, `MashVol`): water `vsu` (L) plus grain `w` (kg) → litres.
#[must_use]
pub fn mash_vol(vsu: f64, w: f64) -> f64 {
    vsu + GRAIN_VOL * w
}

/// Water absorbed by the mash grain (spec §1.2, `RecipeGrainAbsorb`): `w_mash` kg → litres,
/// BIAB uses the lower absorption.
#[must_use]
pub fn grain_absorb(w_mash: f64, biab: bool) -> f64 {
    (if biab { ABS_BIAB } else { ABS_STD }) * w_mash
}

/// Water absorbed by sparge grains (spec §1.3, `SpargeAbsorb`): `w_sparge` kg → litres.
#[must_use]
pub fn sparge_absorb(w_sparge: f64, biab: bool) -> f64 {
    (if biab { ABS_BIAB } else { ABS_STD }) * w_sparge
}

/// Mash grain weight (spec §2.6): sum of `(use, type, amount_kg)` entries with `use < 1`
/// (mash) and `type` Grain (0) or Adjunct (3). Returns kg.
#[must_use]
pub fn mash_grain_weight(grains: &[(GrainUse, GrainType, f64)]) -> f64 {
    grains
        .iter()
        .filter(|(u, t, _)| {
            *u == GrainUse::Mash && matches!(t, GrainType::Grain | GrainType::Adjunct)
        })
        .map(|(_, _, a)| a)
        .sum()
}

/// Sparge grain weight (spec §1.3): entries with `use == 8` and `type` 0 or 3. Returns kg.
#[must_use]
pub fn sparge_grain_weight(grains: &[(GrainUse, GrainType, f64)]) -> f64 {
    grains
        .iter()
        .filter(|(u, t, _)| {
            *u == GrainUse::Sparge && matches!(t, GrainType::Grain | GrainType::Adjunct)
        })
        .map(|(_, _, a)| a)
        .sum()
}

/// Mash tun volume needed (spec §1.4, `MashTunVolFunc`): water (L) plus grain `w_mash` (kg).
#[must_use]
pub fn tun_vol_needed(mash_tot_water: f64, w_mash: f64) -> f64 {
    mash_tot_water + GRAIN_VOL * w_mash
}

/// Recipe-level tun volume needed (spec §1.4): also counts sparge grains `w_sparge` (kg).
/// Returns litres.
#[must_use]
pub fn tun_vol_needed_recipe(mash_tot_water: f64, w_mash: f64, w_sparge: f64) -> f64 {
    tun_vol_needed(mash_tot_water, w_mash) + GRAIN_VOL * w_sparge
}

/// Water of one step including the tun addition (spec §1.5, `F_MS_TOT_INFUSION`), litres.
#[must_use]
pub fn step_tot_infusion(infusion: f64, tun_addition: f64) -> f64 {
    infusion + tun_addition
}

/// Total mash water (spec §1.5, `MashTotWaterFunc`): sum of `infusion + tun_addition` over
/// `(kind, infusion, tun_addition)` steps, decoction steps excluded. Returns litres.
#[must_use]
pub fn mash_tot_water(steps: &[(StepKind, f64, f64)]) -> f64 {
    steps
        .iter()
        .filter(|(k, _, _)| *k != StepKind::Decoction)
        .map(|(_, i, a)| step_tot_infusion(*i, *a))
        .sum()
}

/// Wort available from the mash (spec §1.6, `MashAvailWaterFunc`): total water minus grain
/// absorption minus tun deadspace, all litres.
#[must_use]
pub fn mash_avail_water(tot_water: f64, absorb: f64, tun_deadspace: f64) -> f64 {
    tot_water - absorb - tun_deadspace
}

/// [`infusion_temp`] plus the above-boiling flag (spec §2.2, `MashStepError`).
///
/// Same inputs as [`infusion_temp`]; `None` when `va <= 0`, otherwise the temperature (°C)
/// and whether it exceeds 100 °C.
#[must_use]
pub fn infusion_temp_checked(
    va: f64,
    t2: f64,
    vw: f64,
    t1: f64,
    w: f64,
    tun: Tun,
) -> Option<InfusionCheck> {
    infusion_temp(va, t2, vw, t1, w, tun).map(|t| InfusionCheck {
        temp_c: t,
        above_boiling: t > ABOVE_BOILING_C,
    })
}

/// Single-step temperature update (spec §2.5, `UpdateInfusionTemp`).
///
/// Infusion step with `start_temp == 0` is the first step (strike on `infusion +
/// tun_addition`); other infusion steps use [`infusion_temp`]; every other type returns
/// `t_step`. Volumes L, temperatures °C, `w` kg.
#[must_use]
#[allow(clippy::too_many_arguments, clippy::float_cmp)] // mirrors the spec signature; 0 is a sentinel
pub fn update_infusion_temp(
    kind: StepKind,
    infusion: f64,
    tun_addition: f64,
    t_step: f64,
    start_temp: f64,
    start_vol: f64,
    w: f64,
    t_grain: f64,
    tun: Tun,
    t_tun: Option<f64>,
) -> Option<f64> {
    if kind == StepKind::Infusion {
        if start_temp == 0.0 {
            return strike_temp(infusion + tun_addition, t_step, w, t_grain, tun, t_tun);
        }
        return infusion_temp(infusion, t_step, start_vol, start_temp, w, tun);
    }
    Some(t_step)
}

fn is_above_boiling(kind: StepKind, itemp: Option<f64>) -> bool {
    kind == StepKind::Infusion && itemp.is_some_and(|t| t > ABOVE_BOILING_C)
}

/// Whole-mash recalculation, not BIAB (spec §2.5, `BSMash::RecalculateMash`).
///
/// Steps are processed in order with the running water `V_cum`. `deadspace` (L) is added to the
/// first step as its tun addition. `tb` is the decoction boiling temperature (°C). A first
/// decoction step is calculated like a strike and adds no water to `V_cum` (literal §2.5).
#[must_use]
#[allow(clippy::too_many_arguments, clippy::float_cmp)] // mirrors the spec; deadspace 0 is a sentinel
pub fn recalc_mash(
    steps: &[StepInput],
    w: f64,
    t_grain: f64,
    tun: Tun,
    t_tun: Option<f64>,
    deadspace: f64,
    tb: f64,
) -> Vec<StepResult> {
    let mut out = Vec::with_capacity(steps.len());
    let mut v_cum = 0.0_f64;
    let mut t_prev = 0.0_f64;
    for (i, s) in steps.iter().enumerate() {
        let mut v_add = s.infusion;
        let mut tun_add = 0.0;
        if i == 0 && deadspace != 0.0 {
            v_add += deadspace;
            tun_add = deadspace;
        }
        let mut itemp = None;
        let mut dec = 0.0;
        if s.kind == StepKind::Infusion || (s.kind == StepKind::Decoction && i == 0) {
            itemp = if i == 0 {
                strike_temp(v_add, s.t_step, w, t_grain, tun, t_tun)
            } else {
                infusion_temp(v_add, s.t_step, v_cum, t_prev, w, tun)
            };
        } else if s.kind == StepKind::Decoction {
            dec = decoct_vol(s.t_step, v_cum, t_prev, w, tun, tb);
            itemp = Some(0.0);
        }
        out.push(StepResult {
            infusion: s.infusion,
            tun_addition: tun_add,
            tot_infusion: v_add,
            start_vol: v_cum,
            start_temp: if i == 0 { 0.0 } else { t_prev },
            infusion_temp: itemp,
            decoction_amt: dec,
            above_boiling: is_above_boiling(s.kind, itemp),
            no_water: s.kind == StepKind::Decoction && v_cum <= 0.0,
        });
        if s.kind != StepKind::Decoction {
            v_cum += v_add;
        }
        t_prev = s.t_step;
    }
    out
}

/// BIAB step distribution (spec §3.2, `BSMash::RecalculateBIAB`).
///
/// Steps are processed last to first starting from `biab_vol` (L); the first step receives all
/// the remaining water. Temperatures follow §2.5.
#[must_use]
pub fn recalc_biab(
    steps: &[StepInput],
    w: f64,
    t_grain: f64,
    biab_vol: f64,
    tun: Tun,
    t_tun: Option<f64>,
    tb: f64,
) -> Vec<StepResult> {
    let n = steps.len();
    let mut infusion: Vec<f64> = steps.iter().map(|s| s.infusion).collect();
    let mut start = vec![0.0_f64; n];
    let mut v_rem = biab_vol;
    for k in (0..n).rev() {
        if k > 0 && steps[k].kind != StepKind::Decoction {
            v_rem -= infusion[k];
        }
        if k == 0 {
            infusion[0] = v_rem;
            start[0] = 0.0;
        } else {
            start[k] = v_rem;
        }
    }
    let mut out = Vec::with_capacity(n);
    for (k, s) in steps.iter().enumerate() {
        let t_prev = if k > 0 { steps[k - 1].t_step } else { 0.0 };
        let mut itemp = None;
        let mut dec = 0.0;
        if k == 0 {
            if s.kind == StepKind::Infusion || s.kind == StepKind::Decoction {
                itemp = strike_temp(infusion[0], s.t_step, w, t_grain, tun, t_tun);
            }
        } else if s.kind == StepKind::Infusion {
            itemp = infusion_temp(infusion[k], s.t_step, start[k], t_prev, w, tun);
        } else if s.kind == StepKind::Decoction {
            dec = decoct_vol(s.t_step, start[k], t_prev, w, tun, tb);
            itemp = Some(0.0);
        }
        out.push(StepResult {
            infusion: infusion[k],
            tun_addition: 0.0,
            tot_infusion: infusion[k],
            start_vol: start[k],
            start_temp: if k == 0 { 0.0 } else { t_prev },
            infusion_temp: itemp,
            decoction_amt: dec,
            above_boiling: is_above_boiling(s.kind, itemp),
            no_water: s.kind == StepKind::Decoction && start[k] <= 0.0,
        });
    }
    out
}

/// Tun deadspace copied into the mash profile (spec §2.6, `F_MH_TUN_DEADSPACE`): tun addition
/// plus the deadspace when `adj_deadspace` is on. Litres.
#[must_use]
pub fn tun_deadspace_profile(tun_addition: f64, adj_deadspace: bool, tun_deadspace: f64) -> f64 {
    tun_addition + if adj_deadspace { tun_deadspace } else { 0.0 }
}

/// Rescales step infusions to a new grain weight (spec §2.6, `ScaleFromGrainWeight`),
/// keeping the water/grain ratio. Only non-BIAB with `w_old`, `w_new > 0` and different;
/// only infusions above 0 are scaled. Returns litres per step.
#[must_use]
#[allow(clippy::float_cmp)] // spec: W_eski != W_yeni
pub fn scale_from_grain_weight(infusions: &[f64], w_old: f64, w_new: f64, biab: bool) -> Vec<f64> {
    if !biab && w_old > 0.0 && w_new > 0.0 && w_old != w_new {
        return infusions
            .iter()
            .map(|&i| if i > 0.0 { w_new * i / w_old } else { i })
            .collect();
    }
    infusions.to_vec()
}

/// Step water/grain ratio in L/kg (spec §2.7, `MashStepWaterGrainFunc`): decoction steps use
/// `start_vol / w`, others `(start_vol + infusion) / w`. `w == 0` gives 0 (the spec does not
/// say).
#[must_use]
#[allow(clippy::float_cmp)]
pub fn step_water_grain(start_vol: f64, infusion: f64, w: f64, decoction: bool) -> f64 {
    if w == 0.0 {
        return 0.0;
    }
    if decoction {
        start_vol / w
    } else {
        (start_vol + infusion) / w
    }
}

/// Infusion after the user enters a water/grain ratio `r` L/kg (spec §2.7,
/// `ScaleWaterGrain`): `r·w − start_vol`, written only when it differs from `infusion_old` by
/// more than 1 fl oz; otherwise `infusion_old`. Litres.
#[must_use]
pub fn scale_water_grain(r: f64, w: f64, start_vol: f64, infusion_old: f64) -> f64 {
    let new = r * w - start_vol;
    if (new - infusion_old).abs() > FLOZ_L {
        new
    } else {
        infusion_old
    }
}

/// A step as seen by [`main_mash_thickness`] (L, kg, °C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThicknessStep {
    /// Step grain weight (kg); steps with 0 are skipped.
    pub w: f64,
    /// Step temperature (°C).
    pub t_step: f64,
    /// Water already in the mash (L).
    pub start_vol: f64,
    /// `F_MS_TOT_INFUSION` (L).
    pub tot_infusion: f64,
    /// Decoction step.
    pub decoction: bool,
}

/// Main mash thickness (spec §2.8, `GetMainMashThickness`), L/kg.
///
/// Picks the step closest to 153.5 °F (strictly within 40 °F, first wins ties) and uses its
/// water/grain ratio; without one it falls back to `tot_water / w_total` (0 when `w_total` is
/// 0). The ratio is computed in fl oz/oz and multiplied by the code constant
/// [`THICKNESS_CONST`], so the result differs from a plain L/kg ratio in the 4th digit.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn main_mash_thickness(steps: &[ThicknessStep], tot_water: f64, w_total: f64) -> f64 {
    let mut best: Option<&ThicknessStep> = None;
    let mut best_diff = 40.0_f64;
    for s in steps {
        if s.w <= 0.0 {
            continue;
        }
        let d = (s.t_step * 9.0 / 5.0 + 32.0 - MASH_FG_ADJ_TEMP_F).abs();
        if d < best_diff {
            best = Some(s);
            best_diff = d;
        }
    }
    let r = if let Some(s) = best {
        let v = if s.decoction {
            s.start_vol
        } else {
            s.start_vol + s.tot_infusion
        };
        (v / FLOZ_L) / (s.w / OZ_KG)
    } else if w_total == 0.0 {
        0.0
    } else {
        (tot_water / FLOZ_L) / (w_total / OZ_KG)
    };
    r * THICKNESS_CONST
}

/// BIAB total mash water (spec §3.1, `RecipeBIABMashVol`): pre-boil plus tun deadspace minus
/// kettle top-up (all L) plus BIAB absorption of `w_mash` kg. Litres.
#[must_use]
pub fn biab_mash_vol(v_preboil: f64, tun_deadspace: f64, topup_kettle: f64, w_mash: f64) -> f64 {
    v_preboil + tun_deadspace - topup_kettle + ABS_BIAB * w_mash
}

/// Sparge water (spec §4.1, `RecipeSpargeVolFunc`): pre-boil + mash absorption + sparge
/// absorption + tun deadspace − kettle top-up − total mash water, all litres. May be negative.
#[must_use]
pub fn sparge_vol(
    v_preboil: f64,
    absorb: f64,
    sparge_absorb: f64,
    tun_deadspace: f64,
    topup_kettle: f64,
    mash_tot_water: f64,
) -> f64 {
    v_preboil + absorb + sparge_absorb + tun_deadspace - topup_kettle - mash_tot_water
}

/// Total boil-off (spec §5.1, `TotBoilOffFunc`): `boil_off` per hour times `boil_time_min / 60`
/// when `rate_flag`, else `boil_off` as a total volume. Litres.
#[must_use]
pub fn tot_boil_off(boil_off: f64, boil_time_min: f64, rate_flag: bool) -> f64 {
    if rate_flag {
        boil_off * (boil_time_min / 60.0)
    } else {
        boil_off
    }
}

/// Post-boil (hot) volume (spec §5.3, `PostBoilVolFunc`), litres.
#[must_use]
pub fn post_boil_vol(v_preboil: f64, boil_off_total: f64) -> f64 {
    v_preboil - boil_off_total
}

/// Cooling shrinkage (spec §5.4, `EquipCoolLossFunc`): `cool_pct` percent of the post-boil
/// volume (L), not capped here. Litres.
#[must_use]
pub fn cool_loss(cool_pct: f64, v_postboil: f64) -> f64 {
    (cool_pct / 100.0) * v_postboil
}

/// Pre-trub volume (spec §5.5, `PreTrubLossVolFunc`): batch plus trub loss, litres.
#[must_use]
pub fn pre_trub_vol(batch: f64, trub: f64) -> f64 {
    batch + trub
}

/// Evaporation rate in percent per hour (spec §5.6, `EquipEvapRate`).
///
/// `boil_off_total` L, `v_preboil` L (taken as 1 when not positive), `boil_time_min` minutes;
/// 0 when the time is not positive (the spec does not say).
#[must_use]
pub fn evap_rate(boil_off_total: f64, v_preboil: f64, boil_time_min: f64) -> f64 {
    if boil_time_min <= 0.0 {
        return 0.0;
    }
    let vp = if v_preboil > 0.0 { v_preboil } else { 1.0 };
    boil_off_total * 100.0 / (vp * boil_time_min / 60.0)
}

/// Equipment bottling volume (spec §5.7, `EquipBottleVolFunc`): batch minus fermenter loss, L.
#[must_use]
pub fn bottle_vol_equip(batch: f64, fermenter_loss: f64) -> f64 {
    batch - fermenter_loss
}

/// Recipe bottling volume (spec §5.7, `FinalVolFunc`), litres.
///
/// With `include_starter` the starter (and the second one if `use_second`) is added to the
/// batch; the fermenter loss is subtracted either way.
#[must_use]
pub fn bottle_vol_recipe(
    batch: f64,
    fermenter_loss: f64,
    include_starter: bool,
    starter: f64,
    use_second: bool,
    second_starter: f64,
) -> f64 {
    if include_starter {
        batch + starter + if use_second { second_starter } else { 0.0 } - fermenter_loss
    } else {
        batch - fermenter_loss
    }
}

/// Total water (spec §5.8, `TotalWaterFunc`), litres.
///
/// `recipe_type` 1 or 2 (partial/all grain): mash + sparge + kettle top-up + fermenter top-up.
/// Any other type is treated as Extract: `batch` without boil ingredients, otherwise
/// `v_preboil + topup_ferm` (types above 2 are not specified).
#[must_use]
#[allow(clippy::too_many_arguments)] // mirrors the spec signature
pub fn total_water(
    recipe_type: RecipeType,
    has_boil_ing: bool,
    mash_tot_water: f64,
    sparge: f64,
    topup_kettle: f64,
    topup_ferm: f64,
    batch: f64,
    v_preboil: f64,
) -> f64 {
    if matches!(recipe_type, RecipeType::PartialMash | RecipeType::AllGrain) {
        mash_tot_water + sparge + topup_kettle + topup_ferm
    } else if !has_boil_ing {
        batch
    } else {
        v_preboil + topup_ferm
    }
}

/// Sum of the amounts of `(use, amount)` entries whose use equals `use` (spec §5.9,
/// `TotalGrainUseField`). Weight or volume, whatever the entries carry.
#[must_use]
pub fn total_grain_use(items: &[(GrainUse, f64)], use_code: GrainUse) -> f64 {
    items
        .iter()
        .filter(|(u, _)| *u == use_code)
        .map(|(_, a)| a)
        .sum()
}

/// Steep water (spec §5.9, `SteepVolFunc`): the smaller of `v_preboil − v_boilgrains` and
/// 4 qt/lb of the steeping grain `w_steep` (kg). Litres.
#[must_use]
pub fn steep_vol(v_preboil: f64, v_boilgrains: f64, w_steep: f64) -> f64 {
    let steep_max = STEEP_QT_PER_LB * (w_steep / OZ_KG / 16.0) * 32.0 * FLOZ_L;
    (v_preboil - v_boilgrains).min(steep_max)
}

/// Water added to the kettle after steeping (spec §5.9, `BoilWaterAdd`), litres.
#[must_use]
pub fn boil_water_add(v_preboil: f64, v_steep: f64, v_boilgrains: f64) -> f64 {
    v_preboil - v_steep - v_boilgrains
}

/// Water added in primary (spec §5.9, `PrimaryWaterAdd`), litres.
///
/// The post-boil term counts for recipe types up to 2 or when there are boil ingredients;
/// otherwise it is 0.
#[must_use]
pub fn primary_water_add(
    recipe_type: RecipeType,
    has_boil_ing: bool,
    batch: f64,
    v_postboil: f64,
    v_primarygrains: f64,
    topup_ferm: f64,
) -> f64 {
    let post = if recipe_type.code() <= 2 || has_boil_ing {
        v_postboil
    } else {
        0.0
    };
    batch - post - v_primarygrains - topup_ferm
}

/// Pre-boil gravity estimate (spec §5.10, `GetEstPostMashOG`): `og` SG, volumes L. `None`
/// when `v_preboil <= 0` (an error, T1.16).
pub fn preboil_sg(og: f64, batch: f64, trub: f64, v_preboil: f64) -> Result<f64> {
    ensure_finite("og", og)?;
    ensure_finite("batch", batch)?;
    ensure_finite("trub", trub)?;
    ensure_finite("v_preboil", v_preboil)?;
    if v_preboil <= 0.0 {
        return Err(Error::invalid("v_preboil", "must be > 0"));
    }
    Ok(1.0 + (og - 1.0) * (batch + trub) / v_preboil)
}

/// Equipment volume chain (spec §5.11): result of [`volume_chain`], litres except `evap_pct`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeChain {
    /// Total boil-off (§5.1).
    pub boil_off: f64,
    /// Pre-boil volume (§5.2).
    pub preboil: f64,
    /// Post-boil volume (§5.3).
    pub postboil: f64,
    /// Cooling shrinkage (§5.4).
    pub cool_loss: f64,
    /// Cold wort in the kettle: post-boil minus cooling shrinkage.
    pub cold_wort: f64,
    /// Pre-trub volume (§5.5).
    pub pretrub: f64,
    /// Evaporation rate in percent per hour (§5.6).
    pub evap_pct: f64,
    /// Bottling volume (§5.7, equipment).
    pub bottle: f64,
}

/// Volume chain from boil-off to bottle (spec §5.11), litres.
///
/// `batch` into the fermenter, `topup_ferm`, `trub`, `cool_pct` (percent), `boil_off` (rate per
/// hour when `rate_flag`, else total), `boil_time_min`, `fermenter_loss`.
#[must_use]
#[allow(clippy::too_many_arguments)] // mirrors the spec signature
pub fn volume_chain(
    batch: f64,
    topup_ferm: f64,
    trub: f64,
    cool_pct: f64,
    boil_off: f64,
    boil_time_min: f64,
    rate_flag: bool,
    fermenter_loss: f64,
) -> VolumeChain {
    let bo = tot_boil_off(boil_off, boil_time_min, rate_flag);
    let pre = equip_boil_vol(batch, topup_ferm, trub, cool_pct, bo);
    let post = post_boil_vol(pre, bo);
    let cool = cool_loss(cool_pct, post);
    VolumeChain {
        boil_off: bo,
        preboil: pre,
        postboil: post,
        cool_loss: cool,
        cold_wort: post - cool,
        pretrub: pre_trub_vol(batch, trub),
        evap_pct: evap_rate(bo, pre, boil_time_min),
        bottle: bottle_vol_equip(batch, fermenter_loss),
    }
}

/// "Water Needed" tool cooling loss (spec §5.12, `WNCoolLoss`): `(v_ferm − topup)·c/(1 − c)`
/// with `c = cool_pct / 100`, litres. Errors on non-finite input and `cool_pct = 100`.
#[allow(clippy::float_cmp)]
pub fn wn_cool_loss(v_ferm: f64, topup: f64, cool_pct: f64) -> Result<f64> {
    ensure_finite("v_ferm", v_ferm)?;
    ensure_finite("topup", topup)?;
    ensure_finite("cool_pct", cool_pct)?;
    if cool_pct == 100.0 {
        return Err(Error::invalid("cool_pct", "must differ from 100"));
    }
    Ok((v_ferm - topup) * (cool_pct / 100.0) / (1.0 - cool_pct / 100.0))
}

/// "Water Needed" boil volume (spec §5.12, `WNBoilVol`), litres.
#[must_use]
pub fn wn_boil_vol(v_ferm: f64, topup: f64, trub: f64, v_cool: f64, boil_off: f64) -> f64 {
    (v_ferm - topup) + trub + v_cool + boil_off
}

/// "Water Needed" boil-off percentage (spec §5.12, `WNBoilPct`); 0 when `v_boil == 0`.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn wn_boil_pct(boil_off: f64, v_boil: f64) -> f64 {
    if v_boil == 0.0 {
        0.0
    } else {
        boil_off * 100.0 / v_boil
    }
}

/// "Water Needed" grain absorption (spec §5.12, `WNGrainAbsorb`): `w` kg → litres, no BIAB.
#[must_use]
pub fn wn_grain_absorb(w: f64) -> f64 {
    ABS_STD * w
}

/// "Water Needed" sparge water (spec §5.12, `WNSpargeWater`), litres.
#[must_use]
pub fn wn_sparge_water(
    v_boil: f64,
    absorb: f64,
    topup_kettle: f64,
    lauter_deadspace: f64,
    v_mash: f64,
) -> f64 {
    v_boil + absorb - topup_kettle + lauter_deadspace - v_mash
}

/// "Water Needed" total water (spec §5.12, `WNTotalWaterVol`): `brew_type` 0 is Extract
/// (`v_boil + topup`), anything else is mash based. Litres.
#[must_use]
pub fn wn_total_water(
    brew_type: RecipeType,
    v_boil: f64,
    topup: f64,
    v_sparge: f64,
    v_mash: f64,
    topup_kettle: f64,
) -> f64 {
    if brew_type == RecipeType::Extract {
        v_boil + topup
    } else {
        v_sparge + v_mash + topup_kettle + topup
    }
}

/// "Water Needed" bottling volume (spec §5.12, `WNBottleVol`), litres.
#[must_use]
pub fn wn_bottle_vol(v_ferm: f64, v_loss_ferm: f64) -> f64 {
    v_ferm - v_loss_ferm
}

/// Result of [`boil_off_tool`] (spec §6.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoilOffResult {
    /// Hourly evaporation (L/h).
    pub rate: f64,
    /// Total boil-off (L).
    pub boil_off: f64,
    /// Cooling shrinkage (L).
    pub cool_loss: f64,
    /// End volume (L).
    pub v_end: f64,
    /// End gravity (SG).
    pub sg_end: f64,
}

/// Boil-off tool (spec §6.2): start volume `v0` L, evaporation `evap_pct` %/h, `boil_time_min`
/// minutes, `cool_pct` percent, start gravity `sg0`. End gravity is 1.0 when the end volume is 0.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn boil_off_tool(
    v0: f64,
    evap_pct: f64,
    boil_time_min: f64,
    cool_pct: f64,
    sg0: f64,
) -> BoilOffResult {
    let rate = evap_pct * v0 / 100.0;
    let bo = boil_time_min * evap_pct * v0 / 6000.0;
    let cool = (cool_pct / 100.0) * (v0 - bo);
    let v_end = v0 - bo - cool;
    let sg_end = if v_end == 0.0 {
        1.0
    } else {
        1.0 + (sg0 - 1.0) * v0 / v_end
    };
    BoilOffResult {
        rate,
        boil_off: bo,
        cool_loss: cool,
        v_end,
        sg_end,
    }
}

/// Result of [`dilution`] (spec §8.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dilution {
    /// Total volume (L).
    pub v_tot: f64,
    /// Final gravity (SG).
    pub sg: f64,
}

/// Dilution (spec §8.1): `v0` L at `sg0` plus `v_add` L of water, or of wort at `sg_add` when
/// `wort`. A negative `v_add` leaves the gravity unchanged.
///
/// Errors on non-finite input and a total volume of 0 when the gravity would be computed
/// (division by zero).
pub fn dilution(sg0: f64, v0: f64, v_add: f64, wort: bool, sg_add: f64) -> Result<Dilution> {
    ensure_finite("sg0", sg0)?;
    ensure_finite("v0", v0)?;
    ensure_finite("v_add", v_add)?;
    ensure_finite("sg_add", sg_add)?;
    let v_tot = v0 + v_add;
    if v_add >= 0.0 && v_tot == 0.0 {
        return Err(Error::invalid("v0", "total volume must not be 0"));
    }
    let sg = if v_add < 0.0 {
        sg0
    } else if wort {
        1.0 + ((sg0 - 1.0) * v0 + (sg_add - 1.0) * v_add) / v_tot
    } else {
        1.0 + (sg0 - 1.0) * v0 / v_tot
    };
    Ok(Dilution { v_tot, sg })
}

/// Water to dilute `v` L from `sg` to `sg_target` (spec §8.2, `GravityAdjWaterVol`); 0 if
/// negative. Litres. Errors on non-finite input and `sg_target = 1.000` (division by zero).
#[allow(clippy::float_cmp)]
pub fn gravity_adj_water_vol(v: f64, sg: f64, sg_target: f64) -> Result<f64> {
    ensure_finite("v", v)?;
    ensure_finite("sg", sg)?;
    ensure_finite("sg_target", sg_target)?;
    if sg_target == 1.0 {
        return Err(Error::invalid("sg_target", "must differ from 1.000"));
    }
    Ok((v * (sg - 1.0) / (sg_target - 1.0) - v).max(0.0))
}

/// Volume of a misc ingredient in US fluid ounces (spec §9, `BSMisc::ToFluidOz`).
///
/// `amount` in the unit given by BeerSmith's `F_M_UNITS` code (0 mg, 1 g, 2 fl oz, 3 lb,
/// 4 kg, 5 mL, 6 tsp, 7 tbsp, 8 cup, 9 pint, 10 quart, 11 L, 12 gal); weights count as
/// 1 g = 1 mL. Returns BeerSmith's internal unit, not litres. Errors on a non-finite amount.
pub fn misc_to_fluid_oz(amount: f64, units: MiscFluidUnit) -> Result<f64> {
    ensure_finite("amount", amount)?;
    let ml = match units {
        MiscFluidUnit::Mg => amount / 1000.0,
        MiscFluidUnit::G | MiscFluidUnit::Ml => amount,
        MiscFluidUnit::Lb => amount * LB_KG * 1000.0,
        MiscFluidUnit::Kg | MiscFluidUnit::L => amount * 1000.0,
        MiscFluidUnit::Tsp => amount * 4.93,
        MiscFluidUnit::Tbsp => amount * 14.78,
        MiscFluidUnit::FlOz => return Ok(amount),
        MiscFluidUnit::Cup => return Ok(amount * 8.0),
        MiscFluidUnit::Pint => return Ok(amount * 16.0),
        MiscFluidUnit::Quart => return Ok(amount * 32.0),
        MiscFluidUnit::Gal => return Ok(amount * 128.0),
    };
    Ok(ml / (FLOZ_L * 1000.0))
}

/// Misc amount unit `F_M_UNITS` (spec §9): 0 mg, 1 g, 2 fl oz, 3 lb, 4 kg, 5 mL, 6 tsp,
/// 7 tbsp, 8 cup, 9 pint, 10 quart, 11 L, 12 gal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscFluidUnit {
    /// Code 0.
    Mg,
    /// Code 1.
    G,
    /// Code 2.
    FlOz,
    /// Code 3.
    Lb,
    /// Code 4.
    Kg,
    /// Code 5.
    Ml,
    /// Code 6.
    Tsp,
    /// Code 7.
    Tbsp,
    /// Code 8.
    Cup,
    /// Code 9.
    Pint,
    /// Code 10.
    Quart,
    /// Code 11.
    L,
    /// Code 12.
    Gal,
}

impl MiscFluidUnit {
    /// From the unit code (0-12); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        Ok(match code {
            0 => Self::Mg,
            1 => Self::G,
            2 => Self::FlOz,
            3 => Self::Lb,
            4 => Self::Kg,
            5 => Self::Ml,
            6 => Self::Tsp,
            7 => Self::Tbsp,
            8 => Self::Cup,
            9 => Self::Pint,
            10 => Self::Quart,
            11 => Self::L,
            12 => Self::Gal,
            _ => return Err(Error::invalid("misc_unit", "unknown code")),
        })
    }
}
