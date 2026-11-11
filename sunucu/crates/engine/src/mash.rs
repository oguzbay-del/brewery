//! Section 05: mash and volumes (`formuller/05-mash-hacimler.md`).
//!
//! Ported from the Python reference `formuller/testler/05_mash_tv.py`, which is the source of
//! truth. Units are metric here: volumes in litres, masses in kilograms, temperatures in °C,
//! specific heats in cal/g°C, gravities as SG. Conversion from BeerSmith's internal units is
//! the API layer's job (ADR-0003).

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
