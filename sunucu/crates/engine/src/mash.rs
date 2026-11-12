//! Section 05: mash and volumes (`formuller/05-mash-hacimler.md`).
//!
//! Ported from the Python reference `formuller/testler/05_mash_tv.py`, which is the source of
//! truth. Units are metric here: volumes in litres, masses in kilograms, temperatures in °C,
//! specific heats in cal/g°C, gravities as SG. Conversion from BeerSmith's internal units is
//! the API layer's job (ADR-0003).

use crate::error::{Error, Result, ensure_finite};
use crate::recipe::{GrainType, GrainUse, RecipeType, StepKind};
use crate::units::{FLOZ_L, LB_KG};

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
    pub specific_heat_cal: f64,
    /// Tun volume `V_t` (L).
    pub volume_l: f64,
}

impl Tun {
    /// No tun heat capacity.
    pub const NONE: Tun = Tun {
        mass_kg: 0.0,
        specific_heat_cal: 0.0,
        volume_l: 0.0,
    };
}

/// Effective tun mass with the partial-fill correction (spec §2.0).
///
/// `m_tun_kg` kg, `v_mash_l` L, `v_tun_l` L. Returns kg: `m_tun · v_mash / (0.8 · v_tun)` when the
/// mash fills less than 80 % of the tun, otherwise `m_tun_kg`.
#[must_use]
pub fn tun_eff(m_tun_kg: f64, v_mash_l: f64, v_tun_l: f64) -> f64 {
    let cap = 0.8 * v_tun_l;
    if cap > 0.0 && v_mash_l < cap {
        m_tun_kg * v_mash_l / cap
    } else {
        m_tun_kg
    }
}

/// Inputs of [`strike_temp`].
#[derive(Debug, Clone, Copy)]
pub struct StrikeTempInput {
    /// Water volume, L.
    pub water_l: f64,
    /// Target step temperature, °C.
    pub step_c: f64,
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Grain temperature, °C.
    pub grain_c: f64,
    /// Mash tun.
    pub tun: Tun,
    /// Tun pre-heat temperature, °C; `None` means the grain temperature.
    pub tun_c: Option<f64>,
}

/// Strike water temperature (spec §2.1, `MashInTemp`).
///
/// `vw` water (L), `t_step` target step temperature (°C), `w` grain (kg), `t_grain` grain
/// temperature (°C), `t_tun` tun pre-heat temperature (°C; `None` = grain temperature).
/// Returns °C, or `None` when `vw == 0`.
#[must_use]
pub fn strike_temp(p: &StrikeTempInput) -> Option<f64> {
    let StrikeTempInput {
        water_l: vw,
        step_c: t_step,
        grain_kg: w,
        grain_c: t_grain,
        tun,
        tun_c: t_tun,
    } = *p;
    #[allow(clippy::float_cmp)]
    if vw == 0.0 {
        return None;
    }
    let t_tun = t_tun.unwrap_or(t_grain);
    let me = tun_eff(tun.mass_kg, GRAIN_VOL * w + vw, tun.volume_l);
    Some(
        t_step
            + ((t_step - t_tun) * tun.specific_heat_cal * me + C_MALT * w * (t_step - t_grain))
                / vw,
    )
}

/// Inputs of [`infusion_temp`] and [`infusion_temp_checked`].
#[derive(Debug, Clone, Copy)]
pub struct InfusionTempInput {
    /// Added water, L.
    pub added_l: f64,
    /// Target mash temperature, °C.
    pub target_c: f64,
    /// Water already in the mash, L.
    pub mash_water_l: f64,
    /// Current mash temperature, °C.
    pub mash_c: f64,
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Mash tun.
    pub tun: Tun,
}

/// Infusion water temperature needed to raise the mash from `t1` to `t2` (spec §2.2,
/// `InfuseTemp`).
///
/// `va` added water (L), `t2` target (°C), `vw` water already in the mash (L), `t1` current
/// mash temperature (°C), `w` grain (kg). Returns °C, or `None` when `va <= 0`.
#[must_use]
pub fn infusion_temp(p: &InfusionTempInput) -> Option<f64> {
    let InfusionTempInput {
        added_l: va,
        target_c: t2,
        mash_water_l: vw,
        mash_c: t1,
        grain_kg: w,
        tun,
    } = *p;
    if va <= 0.0 {
        return None;
    }
    let me = tun_eff(tun.mass_kg, GRAIN_VOL * w + va + vw, tun.volume_l);
    Some(t2 + (t2 - t1) * (tun.specific_heat_cal * me + C_MALT * w + vw) / va)
}

/// Inputs of [`infusion_amount`].
#[derive(Debug, Clone, Copy)]
pub struct InfusionAmountInput {
    /// Infusion water temperature, °C.
    pub inf_c: f64,
    /// Target mash temperature, °C.
    pub target_c: f64,
    /// Water already in the mash, L.
    pub mash_water_l: f64,
    /// Current mash temperature, °C.
    pub mash_c: f64,
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Mash tun.
    pub tun: Tun,
}

/// Amount of water at `t_inf` to add to move the mash from `t1` to `t2` (spec §2.3,
/// `InfuseAmount`).
///
/// `t_inf`, `t2`, `t1` in °C, `vw` L, `w` kg. Returns litres; 0 unless `t2` lies strictly
/// between `t1` and `t_inf`.
#[must_use]
pub fn infusion_amount(p: &InfusionAmountInput) -> f64 {
    let InfusionAmountInput {
        inf_c: t_inf,
        target_c: t2,
        mash_water_l: vw,
        mash_c: t1,
        grain_kg: w,
        tun,
    } = *p;
    if !(t1.min(t_inf) < t2 && t2 < t1.max(t_inf)) {
        return 0.0;
    }
    let me = tun_eff(tun.mass_kg, GRAIN_VOL * w + vw, tun.volume_l);
    (t2 - t1) * (tun.specific_heat_cal * me + C_MALT * w + vw) / (t_inf - t2)
}

/// Inputs of [`decoct_vol`].
#[derive(Debug, Clone, Copy)]
pub struct DecoctVolInput {
    /// Target mash temperature, °C.
    pub target_c: f64,
    /// Water already in the mash, L.
    pub mash_water_l: f64,
    /// Current mash temperature, °C.
    pub mash_c: f64,
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Mash tun.
    pub tun: Tun,
    /// Boiling temperature, °C.
    pub boil_c: f64,
}

/// Mash volume to pull and boil for a decoction step (spec §2.4, `DecoctVol`).
///
/// `t2` target, `t1` current, `tb` boiling temperature (°C; reference default 100), `vw`
/// water (L), `w` grain (kg). Returns litres of mash (grain + water), at most the whole mash;
/// 0 when `t1 >= tb` or when there is neither grain nor water.
#[must_use]
pub fn decoct_vol(p: &DecoctVolInput) -> f64 {
    let DecoctVolInput {
        target_c: t2,
        mash_water_l: vw,
        mash_c: t1,
        grain_kg: w,
        tun,
        boil_c: tb,
    } = *p;
    #[allow(clippy::float_cmp)]
    if t1 >= tb || (w == 0.0 && vw == 0.0) {
        return 0.0;
    }
    let vm = GRAIN_VOL * w + vw;
    let me = tun_eff(tun.mass_kg, vm, tun.volume_l);
    let f =
        (t2 - t1) / (tb - t1) * (tun.specific_heat_cal * me + C_MALT * w + vw) / (C_MALT * w + vw);
    vm * f.min(1.0)
}

/// Inputs of [`equip_boil_vol`].
#[derive(Debug, Clone, Copy)]
pub struct EquipBoilVolInput {
    /// Batch volume into the fermenter, L.
    pub batch_l: f64,
    /// Fermenter top-up water, L.
    pub topup_l: f64,
    /// Trub loss, L.
    pub trub_l: f64,
    /// Cooling shrinkage, % (capped at 99).
    pub cool_pct: f64,
    /// Total boil-off, L.
    pub boil_off_total_l: f64,
}

/// Pre-boil volume from the equipment chain (spec §5.2, `F_E_BOIL_VOL`).
///
/// `batch` into fermenter, `topup` fermenter top-up, `trub` trub loss, `boil_off_total`
/// total boil-off (all L), `cool_pct` cooling shrinkage in percent (capped at 99).
/// Returns litres.
#[must_use]
pub fn equip_boil_vol(p: &EquipBoilVolInput) -> f64 {
    let EquipBoilVolInput {
        batch_l: batch,
        topup_l: topup,
        trub_l: trub,
        cool_pct,
        boil_off_total_l: boil_off_total,
    } = *p;
    let c = cool_pct.min(99.0);
    boil_off_total + (batch - topup + trub) / (1.0 - c / 100.0)
}

/// Batch sparge inputs shared by [`batch_sparge`] and [`sparge_plan`] (spec §4.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpargeParams {
    /// Sparge water `S` (L); negative values are clipped to 0.
    pub sparge_l: f64,
    /// Total grain absorption `A_t` (L).
    pub absorb_l: f64,
    /// Grain weight `W` (kg).
    pub grain_kg: f64,
    /// Mash tun volume `V_tun` (L).
    pub tun_l: f64,
    /// Batch fill percentage `P` (percent of the tun).
    pub fill_pct: f64,
    /// Total mash water (L).
    pub mash_water_l: f64,
    /// Equal-size batches (`F_MH_BATCH_EVEN`).
    pub even: bool,
    /// Drain first (`F_MH_BATCH_DRAIN`).
    pub drain: bool,
}

/// One batch sparge size of [`batch_sparge`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatchSpargeSize {
    /// Volume, L.
    pub volume_l: f64,
    /// True for the "drain mash tun" step.
    pub drain: bool,
}

/// One batch sparge step: the size and whether it is the "drain mash tun" step.
///
/// `GetBatchSpargeSize(i)` (spec §4.2). `i` is the step index (0 = before the first runoff).
#[must_use]
#[allow(clippy::many_single_char_names, clippy::manual_midpoint)] // names follow spec §4.2; midpoint kept literal for parity
pub fn batch_sparge(i: u32, p: &SpargeParams) -> BatchSpargeSize {
    let mut s = p.sparge_l.max(0.0);
    let vg = GRAIN_VOL * p.grain_kg;
    let mut cap = p.fill_pct * p.tun_l / 100.0 - vg - p.absorb_l;
    if cap < 0.0 {
        cap = p.fill_pct * p.tun_l / 100.0;
    }
    let m = p.mash_water_l - p.absorb_l;
    let r = cap - m;
    if !(i != 0 || (r > 0.0 && !p.drain)) {
        return BatchSpargeSize {
            volume_l: m,
            drain: true,
        };
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
                BatchSpargeSize {
                    volume_l: size - m,
                    drain: false,
                }
            } else {
                BatchSpargeSize {
                    volume_l: m,
                    drain: true,
                }
            };
        }
        // Python: int(n + 0.3) <= i, n is a small positive float.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let batches = (n + 0.3) as u32;
        return BatchSpargeSize {
            volume_l: if batches <= i { 0.0 } else { size },
            drain: false,
        };
    }
    let r = r.max(0.0);
    if i == 0 {
        return BatchSpargeSize {
            volume_l: s.min(r),
            drain: false,
        };
    }
    if !p.drain && !p.even {
        s -= r;
    }
    if !p.even {
        let k = f64::from(i - 1) * cap;
        if k <= s {
            return BatchSpargeSize {
                volume_l: cap.min(s - k),
                drain: false,
            };
        }
        return BatchSpargeSize {
            volume_l: 0.0,
            drain: false,
        };
    }
    if cap <= 0.0 {
        cap = 1.0;
    }
    // Python: int(S / cap + 0.99) truncates toward zero; S >= 0 here.
    #[allow(clippy::cast_possible_truncation)]
    let n = (s / cap + 0.99) as i64;
    if n < i64::from(i) {
        BatchSpargeSize {
            volume_l: 0.0,
            drain: false,
        }
    } else {
        #[allow(clippy::cast_precision_loss)]
        let nf = n as f64;
        BatchSpargeSize {
            volume_l: s / nf,
            drain: false,
        }
    }
}

/// What a sparge plan step asks the brewer to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpargeAction {
    /// Drain the mash tun (first runoff).
    Drain,
    /// Add the water volume.
    Add,
}

/// One step of a batch sparge plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpargeStep {
    /// Step index, from 0.
    pub index: u32,
    /// Volume in litres, unrounded (rounding is display, ADR-0004 madde 7).
    pub volume_l: f64,
    /// Action for this step.
    pub action: SpargeAction,
}

/// Full batch sparge plan (spec §4.2): steps `0..50`, stopping at the first step with no
/// volume that is not a drain.
#[must_use]
pub fn sparge_plan(p: &SpargeParams) -> Vec<SpargeStep> {
    let mut out = Vec::new();
    for index in 0..50 {
        let BatchSpargeSize {
            volume_l: v,
            drain: flag,
        } = batch_sparge(index, p);
        if !flag && v <= 0.0 {
            break;
        }
        out.push(SpargeStep {
            index,
            volume_l: v,
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
    pub vol1_l: f64,
    /// Second-runnings volume (L).
    pub vol2_l: f64,
    /// First-runnings gravity (SG).
    pub og1: f64,
    /// Second-runnings gravity (SG).
    pub og2: f64,
}

/// Parti-gyle split (spec §7, `PartiFrac2`, `PartiVol1/2`, `PartiOG1/2`).
///
/// `og` unsplit gravity (SG), `vol_l` total volume (L), `f1_pct` first-runnings share in percent.
#[must_use]
pub fn parti(og: f64, vol_l: f64, f1_pct: f64) -> Parti {
    let k = 0.5_f64.powf(3.0 * f1_pct / 100.0);
    Parti {
        vol1_l: vol_l * f1_pct / 100.0,
        vol2_l: vol_l * (100.0 - f1_pct) / 100.0,
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

/// A mash step as entered by the user (spec §2.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepInput {
    /// Step type.
    pub kind: StepKind,
    /// `F_MS_INFUSION`: water added in this step (L).
    pub infusion_l: f64,
    /// Target step temperature (°C).
    pub step_c: f64,
}

/// Per-step output of [`recalc_mash`] and [`recalc_biab`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepResult {
    /// `F_MS_INFUSION` (L); for BIAB the first step's value is computed.
    pub infusion_l: f64,
    /// `F_MS_TUN_ADDITION` (L).
    pub tun_addition_l: f64,
    /// `F_MS_TOT_INFUSION` = water added in this step including the tun addition (L).
    pub tot_infusion_l: f64,
    /// `F_MS_START_VOL`: water already in the mash (L).
    pub start_vol_l: f64,
    /// `F_MS_START_TEMP` (°C); 0 for the first step.
    pub start_c: f64,
    /// Infusion water temperature (°C): 0 for decoction steps after the first, `None` when it
    /// is not calculated (temperature steps, zero water).
    pub infusion_c: Option<f64>,
    /// `F_MS_DECOCTION_AMT`: mash to pull (L).
    pub decoction_l: f64,
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

/// Volume the grain occupies in the mash (spec §1.1, `GrainVol`): `grain_kg` kg → litres.
#[must_use]
pub fn grain_vol(grain_kg: f64) -> f64 {
    GRAIN_VOL * grain_kg
}

/// Total mash volume (spec §1.1, `MashVol`): water `water_l` (L) plus grain `grain_kg` (kg) → litres.
#[must_use]
pub fn mash_vol(water_l: f64, grain_kg: f64) -> f64 {
    water_l + GRAIN_VOL * grain_kg
}

/// Water absorbed by the mash grain (spec §1.2, `RecipeGrainAbsorb`): `w_mash_kg` kg → litres,
/// BIAB uses the lower absorption.
#[must_use]
pub fn grain_absorb(w_mash_kg: f64, biab: bool) -> f64 {
    (if biab { ABS_BIAB } else { ABS_STD }) * w_mash_kg
}

/// Water absorbed by sparge grains (spec §1.3, `SpargeAbsorb`): `w_sparge_kg` kg → litres.
#[must_use]
pub fn sparge_absorb(w_sparge_kg: f64, biab: bool) -> f64 {
    (if biab { ABS_BIAB } else { ABS_STD }) * w_sparge_kg
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

/// Mash tun volume needed (spec §1.4, `MashTunVolFunc`): water (L) plus grain `w_mash_kg` (kg).
#[must_use]
pub fn tun_vol_needed(mash_tot_water_l: f64, w_mash_kg: f64) -> f64 {
    mash_tot_water_l + GRAIN_VOL * w_mash_kg
}

/// Recipe-level tun volume needed (spec §1.4): also counts sparge grains `w_sparge_kg` (kg).
/// Returns litres.
#[must_use]
pub fn tun_vol_needed_recipe(mash_tot_water_l: f64, w_mash_kg: f64, w_sparge_kg: f64) -> f64 {
    tun_vol_needed(mash_tot_water_l, w_mash_kg) + GRAIN_VOL * w_sparge_kg
}

/// Water of one step including the tun addition (spec §1.5, `F_MS_TOT_INFUSION`), litres.
#[must_use]
pub fn step_tot_infusion(infusion_l: f64, tun_addition_l: f64) -> f64 {
    infusion_l + tun_addition_l
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
pub fn mash_avail_water(tot_water_l: f64, absorb_l: f64, tun_deadspace_l: f64) -> f64 {
    tot_water_l - absorb_l - tun_deadspace_l
}

/// [`infusion_temp`] plus the above-boiling flag (spec §2.2, `MashStepError`).
///
/// Same inputs as [`infusion_temp`]; `None` when `va <= 0`, otherwise the temperature (°C)
/// and whether it exceeds 100 °C.
#[must_use]
pub fn infusion_temp_checked(p: &InfusionTempInput) -> Option<InfusionCheck> {
    let InfusionTempInput {
        added_l: va,
        target_c: t2,
        mash_water_l: vw,
        mash_c: t1,
        grain_kg: w,
        tun,
    } = *p;
    infusion_temp(&InfusionTempInput {
        added_l: va,
        target_c: t2,
        mash_water_l: vw,
        mash_c: t1,
        grain_kg: w,
        tun,
    })
    .map(|t| InfusionCheck {
        temp_c: t,
        above_boiling: t > ABOVE_BOILING_C,
    })
}

/// Inputs of [`update_infusion_temp`].
#[derive(Debug, Clone, Copy)]
pub struct UpdateInfusionTempInput {
    /// Step type.
    pub kind: StepKind,
    /// Water added in the step, L.
    pub infusion_l: f64,
    /// Tun addition, L.
    pub tun_addition_l: f64,
    /// Target step temperature, °C.
    pub step_c: f64,
    /// Mash temperature at the start of the step, °C; 0 marks the first step.
    pub start_c: f64,
    /// Water already in the mash, L.
    pub start_vol_l: f64,
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Grain temperature, °C.
    pub grain_c: f64,
    /// Mash tun.
    pub tun: Tun,
    /// Tun pre-heat temperature, °C; `None` means the grain temperature.
    pub tun_c: Option<f64>,
}

/// Single-step temperature update (spec §2.5, `UpdateInfusionTemp`).
///
/// Infusion step with `start_temp == 0` is the first step (strike on `infusion +
/// tun_addition`); other infusion steps use [`infusion_temp`]; every other type returns
/// `t_step`. Volumes L, temperatures °C, `w` kg.
#[must_use]
#[allow(clippy::float_cmp)] // 0 is a sentinel
pub fn update_infusion_temp(p: &UpdateInfusionTempInput) -> Option<f64> {
    let UpdateInfusionTempInput {
        kind,
        infusion_l: infusion,
        tun_addition_l: tun_addition,
        step_c: t_step,
        start_c: start_temp,
        start_vol_l: start_vol,
        grain_kg: w,
        grain_c: t_grain,
        tun,
        tun_c: t_tun,
    } = *p;
    if kind == StepKind::Infusion {
        if start_temp == 0.0 {
            return strike_temp(&StrikeTempInput {
                water_l: infusion + tun_addition,
                step_c: t_step,
                grain_kg: w,
                grain_c: t_grain,
                tun,
                tun_c: t_tun,
            });
        }
        return infusion_temp(&InfusionTempInput {
            added_l: infusion,
            target_c: t_step,
            mash_water_l: start_vol,
            mash_c: start_temp,
            grain_kg: w,
            tun,
        });
    }
    Some(t_step)
}

fn is_above_boiling(kind: StepKind, itemp: Option<f64>) -> bool {
    kind == StepKind::Infusion && itemp.is_some_and(|t| t > ABOVE_BOILING_C)
}

/// Inputs of [`recalc_mash`].
#[derive(Debug, Clone, Copy)]
pub struct RecalcMashInput<'a> {
    /// Mash steps.
    pub steps: &'a [StepInput],
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Grain temperature, °C.
    pub grain_c: f64,
    /// Mash tun.
    pub tun: Tun,
    /// Tun pre-heat temperature, °C; `None` means the grain temperature.
    pub tun_c: Option<f64>,
    /// Tun deadspace, L.
    pub deadspace_l: f64,
    /// Boiling temperature, °C.
    pub boil_c: f64,
}

/// Whole-mash recalculation, not BIAB (spec §2.5, `BSMash::RecalculateMash`).
///
/// Steps are processed in order with the running water `V_cum`. `deadspace` (L) is added to the
/// first step as its tun addition. `tb` is the decoction boiling temperature (°C). A first
/// decoction step is calculated like a strike and adds no water to `V_cum` (literal §2.5).
#[must_use]
#[allow(clippy::float_cmp)] // deadspace 0 is a sentinel
pub fn recalc_mash(p: &RecalcMashInput<'_>) -> Vec<StepResult> {
    let RecalcMashInput {
        steps,
        grain_kg: w,
        grain_c: t_grain,
        tun,
        tun_c: t_tun,
        deadspace_l: deadspace,
        boil_c: tb,
    } = *p;
    let mut out = Vec::with_capacity(steps.len());
    let mut v_cum = 0.0_f64;
    let mut t_prev = 0.0_f64;
    for (i, s) in steps.iter().enumerate() {
        let mut v_add = s.infusion_l;
        let mut tun_add = 0.0;
        if i == 0 && deadspace != 0.0 {
            v_add += deadspace;
            tun_add = deadspace;
        }
        let mut itemp = None;
        let mut dec = 0.0;
        if s.kind == StepKind::Infusion || (s.kind == StepKind::Decoction && i == 0) {
            itemp = if i == 0 {
                strike_temp(&StrikeTempInput {
                    water_l: v_add,
                    step_c: s.step_c,
                    grain_kg: w,
                    grain_c: t_grain,
                    tun,
                    tun_c: t_tun,
                })
            } else {
                infusion_temp(&InfusionTempInput {
                    added_l: v_add,
                    target_c: s.step_c,
                    mash_water_l: v_cum,
                    mash_c: t_prev,
                    grain_kg: w,
                    tun,
                })
            };
        } else if s.kind == StepKind::Decoction {
            dec = decoct_vol(&DecoctVolInput {
                target_c: s.step_c,
                mash_water_l: v_cum,
                mash_c: t_prev,
                grain_kg: w,
                tun,
                boil_c: tb,
            });
            itemp = Some(0.0);
        }
        out.push(StepResult {
            infusion_l: s.infusion_l,
            tun_addition_l: tun_add,
            tot_infusion_l: v_add,
            start_vol_l: v_cum,
            start_c: if i == 0 { 0.0 } else { t_prev },
            infusion_c: itemp,
            decoction_l: dec,
            above_boiling: is_above_boiling(s.kind, itemp),
            no_water: s.kind == StepKind::Decoction && v_cum <= 0.0,
        });
        if s.kind != StepKind::Decoction {
            v_cum += v_add;
        }
        t_prev = s.step_c;
    }
    out
}

/// Inputs of [`recalc_biab`].
#[derive(Debug, Clone, Copy)]
pub struct RecalcBiabInput<'a> {
    /// Mash steps.
    pub steps: &'a [StepInput],
    /// Grain mass, kg.
    pub grain_kg: f64,
    /// Grain temperature, °C.
    pub grain_c: f64,
    /// BIAB water volume, L.
    pub biab_vol_l: f64,
    /// Mash tun.
    pub tun: Tun,
    /// Tun pre-heat temperature, °C; `None` means the grain temperature.
    pub tun_c: Option<f64>,
    /// Boiling temperature, °C.
    pub boil_c: f64,
}

/// BIAB step distribution (spec §3.2, `BSMash::RecalculateBIAB`).
///
/// Steps are processed last to first starting from `biab_vol` (L); the first step receives all
/// the remaining water. Temperatures follow §2.5.
#[must_use]
pub fn recalc_biab(p: &RecalcBiabInput<'_>) -> Vec<StepResult> {
    let RecalcBiabInput {
        steps,
        grain_kg: w,
        grain_c: t_grain,
        biab_vol_l: biab_vol,
        tun,
        tun_c: t_tun,
        boil_c: tb,
    } = *p;
    let n = steps.len();
    let mut infusion: Vec<f64> = steps.iter().map(|s| s.infusion_l).collect();
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
        let t_prev = if k > 0 { steps[k - 1].step_c } else { 0.0 };
        let mut itemp = None;
        let mut dec = 0.0;
        if k == 0 {
            if s.kind == StepKind::Infusion || s.kind == StepKind::Decoction {
                itemp = strike_temp(&StrikeTempInput {
                    water_l: infusion[0],
                    step_c: s.step_c,
                    grain_kg: w,
                    grain_c: t_grain,
                    tun,
                    tun_c: t_tun,
                });
            }
        } else if s.kind == StepKind::Infusion {
            itemp = infusion_temp(&InfusionTempInput {
                added_l: infusion[k],
                target_c: s.step_c,
                mash_water_l: start[k],
                mash_c: t_prev,
                grain_kg: w,
                tun,
            });
        } else if s.kind == StepKind::Decoction {
            dec = decoct_vol(&DecoctVolInput {
                target_c: s.step_c,
                mash_water_l: start[k],
                mash_c: t_prev,
                grain_kg: w,
                tun,
                boil_c: tb,
            });
            itemp = Some(0.0);
        }
        out.push(StepResult {
            infusion_l: infusion[k],
            tun_addition_l: 0.0,
            tot_infusion_l: infusion[k],
            start_vol_l: start[k],
            start_c: if k == 0 { 0.0 } else { t_prev },
            infusion_c: itemp,
            decoction_l: dec,
            above_boiling: is_above_boiling(s.kind, itemp),
            no_water: s.kind == StepKind::Decoction && start[k] <= 0.0,
        });
    }
    out
}

/// Tun deadspace copied into the mash profile (spec §2.6, `F_MH_TUN_DEADSPACE`): tun addition
/// plus the deadspace when `adj_deadspace` is on. Litres.
#[must_use]
pub fn tun_deadspace_profile(
    tun_addition_l: f64,
    adj_deadspace: bool,
    tun_deadspace_l: f64,
) -> f64 {
    tun_addition_l + if adj_deadspace { tun_deadspace_l } else { 0.0 }
}

/// Rescales step infusions to a new grain weight (spec §2.6, `ScaleFromGrainWeight`),
/// keeping the water/grain ratio. Only non-BIAB with `w_old_kg`, `w_new > 0` and different;
/// only infusions above 0 are scaled. Returns litres per step.
#[must_use]
#[allow(clippy::float_cmp)] // spec: W_eski != W_yeni
pub fn scale_from_grain_weight(
    infusions: &[f64],
    w_old_kg: f64,
    w_new_kg: f64,
    biab: bool,
) -> Vec<f64> {
    if !biab && w_old_kg > 0.0 && w_new_kg > 0.0 && w_old_kg != w_new_kg {
        return infusions
            .iter()
            .map(|&i| if i > 0.0 { w_new_kg * i / w_old_kg } else { i })
            .collect();
    }
    infusions.to_vec()
}

/// Step water/grain ratio in L/kg (spec §2.7, `MashStepWaterGrainFunc`): decoction steps use
/// `start_vol / w`, others `(start_vol + infusion) / w`. `w == 0` gives 0 (the spec does not
/// say).
#[must_use]
#[allow(clippy::float_cmp)]
pub fn step_water_grain(start_vol_l: f64, infusion_l: f64, grain_kg: f64, decoction: bool) -> f64 {
    if grain_kg == 0.0 {
        return 0.0;
    }
    if decoction {
        start_vol_l / grain_kg
    } else {
        (start_vol_l + infusion_l) / grain_kg
    }
}

/// Infusion after the user enters a water/grain ratio `ratio_l_per_kg` L/kg (spec §2.7,
/// `ScaleWaterGrain`): `r·w − start_vol`, written only when it differs from `infusion_old_l` by
/// more than 1 fl oz; otherwise `infusion_old_l`. Litres.
#[must_use]
pub fn scale_water_grain(
    ratio_l_per_kg: f64,
    grain_kg: f64,
    start_vol_l: f64,
    infusion_old_l: f64,
) -> f64 {
    let new = ratio_l_per_kg * grain_kg - start_vol_l;
    if (new - infusion_old_l).abs() > FLOZ_L {
        new
    } else {
        infusion_old_l
    }
}

/// A step as seen by [`main_mash_thickness`] (L, kg, °C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThicknessStep {
    /// Step grain weight (kg); steps with 0 are skipped.
    pub grain_kg: f64,
    /// Step temperature (°C).
    pub step_c: f64,
    /// Water already in the mash (L).
    pub start_vol_l: f64,
    /// `F_MS_TOT_INFUSION` (L).
    pub tot_infusion_l: f64,
    /// Decoction step.
    pub decoction: bool,
}

/// Main mash thickness (spec §2.8, `GetMainMashThickness`), L/kg.
///
/// Picks the step closest to 153.5 °F (strictly within 40 °F, first wins ties) and uses its
/// water/grain ratio; without one it falls back to `tot_water / w_total` (0 when `w_total_kg` is
/// 0). The ratio is computed in fl oz/oz and multiplied by the code constant
/// [`THICKNESS_CONST`], so the result differs from a plain L/kg ratio in the 4th digit.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn main_mash_thickness(steps: &[ThicknessStep], tot_water_l: f64, w_total_kg: f64) -> f64 {
    let mut best: Option<&ThicknessStep> = None;
    let mut best_diff = 40.0_f64;
    for s in steps {
        if s.grain_kg <= 0.0 {
            continue;
        }
        let d = (s.step_c * 9.0 / 5.0 + 32.0 - MASH_FG_ADJ_TEMP_F).abs();
        if d < best_diff {
            best = Some(s);
            best_diff = d;
        }
    }
    let r = if let Some(s) = best {
        let v = if s.decoction {
            s.start_vol_l
        } else {
            s.start_vol_l + s.tot_infusion_l
        };
        (v / FLOZ_L) / (s.grain_kg / OZ_KG)
    } else if w_total_kg == 0.0 {
        0.0
    } else {
        (tot_water_l / FLOZ_L) / (w_total_kg / OZ_KG)
    };
    r * THICKNESS_CONST
}

/// BIAB total mash water (spec §3.1, `RecipeBIABMashVol`): pre-boil plus tun deadspace minus
/// kettle top-up (all L) plus BIAB absorption of `w_mash_kg` kg. Litres.
#[must_use]
pub fn biab_mash_vol(
    v_preboil_l: f64,
    tun_deadspace_l: f64,
    topup_kettle_l: f64,
    w_mash_kg: f64,
) -> f64 {
    v_preboil_l + tun_deadspace_l - topup_kettle_l + ABS_BIAB * w_mash_kg
}

/// Inputs of [`sparge_vol`].
#[derive(Debug, Clone, Copy)]
pub struct SpargeVolInput {
    /// Pre-boil volume, L.
    pub v_preboil_l: f64,
    /// Grain absorption, L.
    pub absorb_l: f64,
    /// Sparge grain absorption, L.
    pub sparge_absorb_l: f64,
    /// Tun deadspace, L.
    pub tun_deadspace_l: f64,
    /// Kettle top-up, L.
    pub topup_kettle_l: f64,
    /// Total mash water, L.
    pub mash_tot_water_l: f64,
}

/// Sparge water (spec §4.1, `RecipeSpargeVolFunc`): pre-boil + mash absorption + sparge
/// absorption + tun deadspace − kettle top-up − total mash water, all litres. May be negative.
#[must_use]
pub fn sparge_vol(p: &SpargeVolInput) -> f64 {
    let SpargeVolInput {
        v_preboil_l: v_preboil,
        absorb_l: absorb,
        sparge_absorb_l: sparge_absorb,
        tun_deadspace_l: tun_deadspace,
        topup_kettle_l: topup_kettle,
        mash_tot_water_l: mash_tot_water,
    } = *p;
    v_preboil + absorb + sparge_absorb + tun_deadspace - topup_kettle - mash_tot_water
}

/// Total boil-off (spec §5.1, `TotBoilOffFunc`): `boil_off_l` per hour times `boil_time_min / 60`
/// when `rate_flag`, else `boil_off_l` as a total volume. Litres.
#[must_use]
pub fn tot_boil_off(boil_off_l: f64, boil_time_min: f64, rate_flag: bool) -> f64 {
    if rate_flag {
        boil_off_l * (boil_time_min / 60.0)
    } else {
        boil_off_l
    }
}

/// Post-boil (hot) volume (spec §5.3, `PostBoilVolFunc`), litres.
#[must_use]
pub fn post_boil_vol(v_preboil_l: f64, boil_off_total_l: f64) -> f64 {
    v_preboil_l - boil_off_total_l
}

/// Cooling shrinkage (spec §5.4, `EquipCoolLossFunc`): `cool_pct` percent of the post-boil
/// volume (L), not capped here. Litres.
#[must_use]
pub fn cool_loss(cool_pct: f64, v_postboil_l: f64) -> f64 {
    (cool_pct / 100.0) * v_postboil_l
}

/// Pre-trub volume (spec §5.5, `PreTrubLossVolFunc`): batch plus trub loss, litres.
#[must_use]
pub fn pre_trub_vol(batch_l: f64, trub_l: f64) -> f64 {
    batch_l + trub_l
}

/// Evaporation rate in percent per hour (spec §5.6, `EquipEvapRate`).
///
/// `boil_off_total_l` L, `v_preboil_l` L (taken as 1 when not positive), `boil_time_min` minutes;
/// 0 when the time is not positive (the spec does not say).
#[must_use]
pub fn evap_rate(boil_off_total_l: f64, v_preboil_l: f64, boil_time_min: f64) -> f64 {
    if boil_time_min <= 0.0 {
        return 0.0;
    }
    let vp = if v_preboil_l > 0.0 { v_preboil_l } else { 1.0 };
    boil_off_total_l * 100.0 / (vp * boil_time_min / 60.0)
}

/// Equipment bottling volume (spec §5.7, `EquipBottleVolFunc`): batch minus fermenter loss, L.
#[must_use]
pub fn bottle_vol_equip(batch_l: f64, fermenter_loss_l: f64) -> f64 {
    batch_l - fermenter_loss_l
}

/// Inputs of [`bottle_vol_recipe`].
#[derive(Debug, Clone, Copy)]
pub struct BottleVolRecipeInput {
    /// Batch volume into the fermenter, L.
    pub batch_l: f64,
    /// Fermenter loss, L.
    pub fermenter_loss_l: f64,
    /// Add the starter to the batch.
    pub include_starter: bool,
    /// Starter volume, L.
    pub starter_l: f64,
    /// Add the second starter too.
    pub use_second: bool,
    /// Second starter volume, L.
    pub second_starter_l: f64,
}

/// Recipe bottling volume (spec §5.7, `FinalVolFunc`), litres.
///
/// With `include_starter` the starter (and the second one if `use_second`) is added to the
/// batch; the fermenter loss is subtracted either way.
#[must_use]
pub fn bottle_vol_recipe(p: &BottleVolRecipeInput) -> f64 {
    let BottleVolRecipeInput {
        batch_l: batch,
        fermenter_loss_l: fermenter_loss,
        include_starter,
        starter_l: starter,
        use_second,
        second_starter_l: second_starter,
    } = *p;
    if include_starter {
        batch + starter + if use_second { second_starter } else { 0.0 } - fermenter_loss
    } else {
        batch - fermenter_loss
    }
}

/// Inputs of [`total_water`].
#[derive(Debug, Clone, Copy)]
pub struct TotalWaterInput {
    /// Recipe type.
    pub recipe_type: RecipeType,
    /// Recipe has boil ingredients.
    pub has_boil_ing: bool,
    /// Total mash water, L.
    pub mash_tot_water_l: f64,
    /// Sparge water, L.
    pub sparge_l: f64,
    /// Kettle top-up, L.
    pub topup_kettle_l: f64,
    /// Fermenter top-up, L.
    pub topup_ferm_l: f64,
    /// Batch volume into the fermenter, L.
    pub batch_l: f64,
    /// Pre-boil volume, L.
    pub v_preboil_l: f64,
}

/// Total water (spec §5.8, `TotalWaterFunc`), litres.
///
/// `recipe_type` 1 or 2 (partial/all grain): mash + sparge + kettle top-up + fermenter top-up.
/// Any other type is treated as Extract: `batch` without boil ingredients, otherwise
/// `v_preboil + topup_ferm` (types above 2 are not specified).
#[must_use]
pub fn total_water(p: &TotalWaterInput) -> f64 {
    let TotalWaterInput {
        recipe_type,
        has_boil_ing,
        mash_tot_water_l: mash_tot_water,
        sparge_l: sparge,
        topup_kettle_l: topup_kettle,
        topup_ferm_l: topup_ferm,
        batch_l: batch,
        v_preboil_l: v_preboil,
    } = *p;
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
/// 4 qt/lb of the steeping grain `w_steep_kg` (kg). Litres.
#[must_use]
pub fn steep_vol(v_preboil_l: f64, v_boilgrains_l: f64, w_steep_kg: f64) -> f64 {
    let steep_max = STEEP_QT_PER_LB * (w_steep_kg / OZ_KG / 16.0) * 32.0 * FLOZ_L;
    (v_preboil_l - v_boilgrains_l).min(steep_max)
}

/// Water added to the kettle after steeping (spec §5.9, `BoilWaterAdd`), litres.
#[must_use]
pub fn boil_water_add(v_preboil_l: f64, v_steep_l: f64, v_boilgrains_l: f64) -> f64 {
    v_preboil_l - v_steep_l - v_boilgrains_l
}

/// Inputs of [`primary_water_add`].
#[derive(Debug, Clone, Copy)]
pub struct PrimaryWaterAddInput {
    /// Recipe type.
    pub recipe_type: RecipeType,
    /// Recipe has boil ingredients.
    pub has_boil_ing: bool,
    /// Batch volume into the fermenter, L.
    pub batch_l: f64,
    /// Post-boil volume, L.
    pub v_postboil_l: f64,
    /// Water absorbed by primary grains, L.
    pub v_primarygrains_l: f64,
    /// Fermenter top-up, L.
    pub topup_ferm_l: f64,
}

/// Water added in primary (spec §5.9, `PrimaryWaterAdd`), litres.
///
/// The post-boil term counts for recipe types up to 2 or when there are boil ingredients;
/// otherwise it is 0.
#[must_use]
pub fn primary_water_add(p: &PrimaryWaterAddInput) -> f64 {
    let PrimaryWaterAddInput {
        recipe_type,
        has_boil_ing,
        batch_l: batch,
        v_postboil_l: v_postboil,
        v_primarygrains_l: v_primarygrains,
        topup_ferm_l: topup_ferm,
    } = *p;
    let post = if recipe_type.code() <= 2 || has_boil_ing {
        v_postboil
    } else {
        0.0
    };
    batch - post - v_primarygrains - topup_ferm
}

/// Pre-boil gravity estimate (spec §5.10, `GetEstPostMashOG`): `og` SG, volumes L. `None`
/// when `v_preboil <= 0` (an error, T1.16).
pub fn preboil_sg(og: f64, batch_l: f64, trub_l: f64, v_preboil_l: f64) -> Result<f64> {
    ensure_finite("og", og)?;
    ensure_finite("batch_l", batch_l)?;
    ensure_finite("trub_l", trub_l)?;
    ensure_finite("v_preboil_l", v_preboil_l)?;
    if v_preboil_l <= 0.0 {
        return Err(Error::invalid("v_preboil_l", "must be > 0"));
    }
    Ok(1.0 + (og - 1.0) * (batch_l + trub_l) / v_preboil_l)
}

/// Equipment volume chain (spec §5.11): result of [`volume_chain`], litres except `evap_pct`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeChain {
    /// Total boil-off (§5.1).
    pub boil_off_l: f64,
    /// Pre-boil volume (§5.2).
    pub preboil_l: f64,
    /// Post-boil volume (§5.3).
    pub postboil_l: f64,
    /// Cooling shrinkage (§5.4).
    pub cool_loss_l: f64,
    /// Cold wort in the kettle: post-boil minus cooling shrinkage.
    pub cold_wort_l: f64,
    /// Pre-trub volume (§5.5).
    pub pretrub_l: f64,
    /// Evaporation rate in percent per hour (§5.6).
    pub evap_pct: f64,
    /// Bottling volume (§5.7, equipment).
    pub bottle_l: f64,
}

/// Inputs of [`volume_chain`].
#[derive(Debug, Clone, Copy)]
pub struct VolumeChainInput {
    /// Batch volume into the fermenter, L.
    pub batch_l: f64,
    /// Fermenter top-up, L.
    pub topup_ferm_l: f64,
    /// Trub loss, L.
    pub trub_l: f64,
    /// Cooling shrinkage, % (capped at 99).
    pub cool_pct: f64,
    /// Boil-off: litres per hour when `rate_flag`, else total litres.
    pub boil_off_l: f64,
    /// Boil time, minutes.
    pub boil_time_min: f64,
    /// Boil-off is a rate per hour.
    pub rate_flag: bool,
    /// Fermenter loss, L.
    pub fermenter_loss_l: f64,
}

/// Volume chain from boil-off to bottle (spec §5.11), litres.
///
/// `batch` into the fermenter, `topup_ferm`, `trub`, `cool_pct` (percent), `boil_off` (rate per
/// hour when `rate_flag`, else total), `boil_time_min`, `fermenter_loss`.
#[must_use]
pub fn volume_chain(p: &VolumeChainInput) -> VolumeChain {
    let VolumeChainInput {
        batch_l: batch,
        topup_ferm_l: topup_ferm,
        trub_l: trub,
        cool_pct,
        boil_off_l: boil_off,
        boil_time_min,
        rate_flag,
        fermenter_loss_l: fermenter_loss,
    } = *p;
    let bo = tot_boil_off(boil_off, boil_time_min, rate_flag);
    let pre = equip_boil_vol(&EquipBoilVolInput {
        batch_l: batch,
        topup_l: topup_ferm,
        trub_l: trub,
        cool_pct,
        boil_off_total_l: bo,
    });
    let post = post_boil_vol(pre, bo);
    let cool = cool_loss(cool_pct, post);
    VolumeChain {
        boil_off_l: bo,
        preboil_l: pre,
        postboil_l: post,
        cool_loss_l: cool,
        cold_wort_l: post - cool,
        pretrub_l: pre_trub_vol(batch, trub),
        evap_pct: evap_rate(bo, pre, boil_time_min),
        bottle_l: bottle_vol_equip(batch, fermenter_loss),
    }
}

/// "Water Needed" tool cooling loss (spec §5.12, `WNCoolLoss`): `(v_ferm − topup)·c/(1 − c)`
/// with `c = cool_pct / 100`, litres. Errors on non-finite input and `cool_pct = 100`.
#[allow(clippy::float_cmp)]
pub fn wn_cool_loss(v_ferm_l: f64, topup_l: f64, cool_pct: f64) -> Result<f64> {
    ensure_finite("v_ferm_l", v_ferm_l)?;
    ensure_finite("topup_l", topup_l)?;
    ensure_finite("cool_pct", cool_pct)?;
    if cool_pct == 100.0 {
        return Err(Error::invalid("cool_pct", "must differ from 100"));
    }
    Ok((v_ferm_l - topup_l) * (cool_pct / 100.0) / (1.0 - cool_pct / 100.0))
}

/// Inputs of [`wn_boil_vol`].
#[derive(Debug, Clone, Copy)]
pub struct WnBoilVolInput {
    /// Fermenter volume, L.
    pub v_ferm_l: f64,
    /// Fermenter top-up water, L.
    pub topup_l: f64,
    /// Trub loss, L.
    pub trub_l: f64,
    /// Cooled wort volume, L.
    pub v_cool_l: f64,
    /// Total boil-off, L.
    pub boil_off_l: f64,
}

/// "Water Needed" boil volume (spec §5.12, `WNBoilVol`), litres.
#[must_use]
pub fn wn_boil_vol(p: &WnBoilVolInput) -> f64 {
    let WnBoilVolInput {
        v_ferm_l: v_ferm,
        topup_l: topup,
        trub_l: trub,
        v_cool_l: v_cool,
        boil_off_l: boil_off,
    } = *p;
    (v_ferm - topup) + trub + v_cool + boil_off
}

/// "Water Needed" boil-off percentage (spec §5.12, `WNBoilPct`); 0 when `v_boil == 0`.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn wn_boil_pct(boil_off_l: f64, v_boil_l: f64) -> f64 {
    if v_boil_l == 0.0 {
        0.0
    } else {
        boil_off_l * 100.0 / v_boil_l
    }
}

/// Inputs of [`wn_sparge_water`].
#[derive(Debug, Clone, Copy)]
pub struct WnSpargeWaterInput {
    /// Boil volume, L.
    pub v_boil_l: f64,
    /// Grain absorption, L.
    pub absorb_l: f64,
    /// Kettle top-up, L.
    pub topup_kettle_l: f64,
    /// Lauter deadspace, L.
    pub lauter_deadspace_l: f64,
    /// Mash water, L.
    pub v_mash_l: f64,
}

/// "Water Needed" sparge water (spec §5.12, `WNSpargeWater`), litres.
#[must_use]
pub fn wn_sparge_water(p: &WnSpargeWaterInput) -> f64 {
    let WnSpargeWaterInput {
        v_boil_l: v_boil,
        absorb_l: absorb,
        topup_kettle_l: topup_kettle,
        lauter_deadspace_l: lauter_deadspace,
        v_mash_l: v_mash,
    } = *p;
    v_boil + absorb - topup_kettle + lauter_deadspace - v_mash
}

/// Inputs of [`wn_total_water`].
#[derive(Debug, Clone, Copy)]
pub struct WnTotalWaterInput {
    /// Recipe type.
    pub brew_type: RecipeType,
    /// Boil volume, L.
    pub v_boil_l: f64,
    /// Fermenter top-up water, L.
    pub topup_l: f64,
    /// Sparge water, L.
    pub v_sparge_l: f64,
    /// Mash water, L.
    pub v_mash_l: f64,
    /// Kettle top-up, L.
    pub topup_kettle_l: f64,
}

/// "Water Needed" total water (spec §5.12, `WNTotalWaterVol`): `brew_type` 0 is Extract
/// (`v_boil + topup`), anything else is mash based. Litres.
#[must_use]
pub fn wn_total_water(p: &WnTotalWaterInput) -> f64 {
    let WnTotalWaterInput {
        brew_type,
        v_boil_l: v_boil,
        topup_l: topup,
        v_sparge_l: v_sparge,
        v_mash_l: v_mash,
        topup_kettle_l: topup_kettle,
    } = *p;
    if brew_type == RecipeType::Extract {
        v_boil + topup
    } else {
        v_sparge + v_mash + topup_kettle + topup
    }
}

/// "Water Needed" bottling volume (spec §5.12, `WNBottleVol`), litres.
#[must_use]
pub fn wn_bottle_vol(v_ferm_l: f64, v_loss_ferm_l: f64) -> f64 {
    v_ferm_l - v_loss_ferm_l
}

/// Result of [`boil_off_tool`] (spec §6.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoilOffResult {
    /// Hourly evaporation (L/h).
    pub rate_l_per_h: f64,
    /// Total boil-off (L).
    pub boil_off_l: f64,
    /// Cooling shrinkage (L).
    pub cool_loss_l: f64,
    /// End volume (L).
    pub v_end_l: f64,
    /// End gravity (SG).
    pub sg_end: f64,
}

/// Inputs of [`boil_off_tool`].
#[derive(Debug, Clone, Copy)]
pub struct BoilOffToolInput {
    /// Start volume, L.
    pub v0_l: f64,
    /// Evaporation, % per hour.
    pub evap_pct: f64,
    /// Boil time, minutes.
    pub boil_time_min: f64,
    /// Cooling shrinkage, % (capped at 99).
    pub cool_pct: f64,
    /// Start gravity, SG.
    pub sg0: f64,
}

/// Boil-off tool (spec §6.2): start volume `v0` L, evaporation `evap_pct` %/h, `boil_time_min`
/// minutes, `cool_pct` percent, start gravity `sg0`. End gravity is 1.0 when the end volume is 0.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn boil_off_tool(p: &BoilOffToolInput) -> BoilOffResult {
    let BoilOffToolInput {
        v0_l: v0,
        evap_pct,
        boil_time_min,
        cool_pct,
        sg0,
    } = *p;
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
        rate_l_per_h: rate,
        boil_off_l: bo,
        cool_loss_l: cool,
        v_end_l: v_end,
        sg_end,
    }
}

/// Result of [`dilution`] (spec §8.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dilution {
    /// Total volume (L).
    pub v_tot_l: f64,
    /// Final gravity (SG).
    pub sg: f64,
}

/// Inputs of [`dilution`].
#[derive(Debug, Clone, Copy)]
pub struct DilutionInput {
    /// Start gravity, SG.
    pub sg0: f64,
    /// Start volume, L.
    pub v0_l: f64,
    /// Added volume, L.
    pub v_add_l: f64,
    /// The added liquid is wort rather than water.
    pub wort: bool,
    /// Gravity of the added wort, SG.
    pub sg_add: f64,
}

/// Dilution (spec §8.1): `v0` L at `sg0` plus `v_add` L of water, or of wort at `sg_add` when
/// `wort`. A negative `v_add` leaves the gravity unchanged.
///
/// Errors on non-finite input and a total volume of 0 when the gravity would be computed
/// (division by zero).
pub fn dilution(p: &DilutionInput) -> Result<Dilution> {
    let DilutionInput {
        sg0,
        v0_l: v0,
        v_add_l: v_add,
        wort,
        sg_add,
    } = *p;
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
    Ok(Dilution { v_tot_l: v_tot, sg })
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
