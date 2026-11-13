//! Section 03: water chemistry and mash pH (`formuller/03-su-ph.md`).
//!
//! Units follow the specification (BeerSmith internal units): ion concentrations in ppm
//! (mg/L), alkalinity and RA in ppm as `CaCO₃` (50 ppm = 1 mEq/L), mash thickness in L/kg,
//! grain weights in kg, grain colour in SRM, acid strength in percent by weight, volumes in
//! litres. Conversion from other units is the API layer's job (ADR-0003).
//!
//! Acid codes (`F_R_ACID`, spec §0.2 and §6.1): 0 none, 1 lactic, 2 phosphoric, 3 acid malt,
//! 4 citric, 5 acetic, 6 `HCl`, 7 sulphuric, 8 tartaric. Model codes: 0 MPH 3 (default),
//! 1 BW. The `ours` flag selects the project's own model (spec §6.8) instead of the
//! BeerSmith-compatible one.
//!
//! The reference is `formuller/testler/03_su_ph_tv.py`; where the spec text and the reference
//! differ, the reference is followed (e.g. phosphoric density coefficient 0.375 in
//! [`acid_per_liter`], as in the reference).
//!
//! Defined mash acid codes are 0, 1, 2 and 3 only ([`Acid::is_mash_defined`]); codes 4 to 8
//! have a pK table entry (spec §6.1) but no molar mass or density, so mash amounts for them
//! are an error (T1.16).

#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::needless_range_loop
)]

use crate::error::{Error, Result, ensure_finite, ensure_positive, finite_result};
use crate::recipe::{GrainType, RecipeType};
use crate::units::GAL_L;

/// Acid code `F_R_ACID` (spec §0.2 and §6.1): 0 none, 1 lactic, 2 phosphoric, 3 acid malt,
/// 4 citric, 5 acetic, 6 `HCl`, 7 sulphuric, 8 tartaric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acid {
    /// Code 0.
    None,
    /// Code 1.
    Lactic,
    /// Code 2.
    Phosphoric,
    /// Code 3.
    AcidMalt,
    /// Code 4.
    Citric,
    /// Code 5.
    Acetic,
    /// Code 6.
    Hcl,
    /// Code 7.
    Sulphuric,
    /// Code 8.
    Tartaric,
}

impl Acid {
    /// From the BeerSmith code (0-8); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        Ok(match code {
            0 => Self::None,
            1 => Self::Lactic,
            2 => Self::Phosphoric,
            3 => Self::AcidMalt,
            4 => Self::Citric,
            5 => Self::Acetic,
            6 => Self::Hcl,
            7 => Self::Sulphuric,
            8 => Self::Tartaric,
            _ => return Err(Error::invalid("acid", "unknown code")),
        })
    }

    /// True for the codes whose molar mass and density the spec defines for mash amounts
    /// (§6.2): none, lactic, phosphoric, acid malt.
    #[must_use]
    pub const fn is_mash_defined(self) -> bool {
        matches!(
            self,
            Self::None | Self::Lactic | Self::Phosphoric | Self::AcidMalt
        )
    }
}

/// Mash pH model: 0 = MPH 3 (default), 1 = BW.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MashModel {
    /// Code 0.
    Mph3,
    /// Code 1.
    Bw,
}

impl MashModel {
    /// From the model code (0, 1); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::Mph3),
            1 => Ok(Self::Bw),
            _ => Err(Error::invalid("model", "unknown code")),
        }
    }
}

/// Misc amount unit (spec §0.1 `ToGrams`): 0 mg, 1 g, 2 oz, 3 lb, 4 kg, 5 mL (taken as grams,
/// spec "belirsiz"), 6 tsp, 7 tbsp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiscUnit {
    /// Code 0.
    Mg,
    /// Code 1.
    G,
    /// Code 2.
    Oz,
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
}

impl MiscUnit {
    /// From the unit code (0-7); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        Ok(match code {
            0 => Self::Mg,
            1 => Self::G,
            2 => Self::Oz,
            3 => Self::Lb,
            4 => Self::Kg,
            5 => Self::Ml,
            6 => Self::Tsp,
            7 => Self::Tbsp,
            _ => return Err(Error::invalid("misc_unit", "unknown code")),
        })
    }
}

/// Number of tracked ions: Ca, Mg, Na, SO₄, Cl, HCO₃ (in this order).
pub const ION_COUNT: usize = 6;

/// Water-treatment salts in the column order of the contribution matrix (spec §1.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Salt {
    /// Gypsum, `CaSO₄·2H₂O`.
    Gypsum,
    /// Table salt, `NaCl`.
    Nacl,
    /// Epsom salt, `MgSO₄·7H₂O`.
    Epsom,
    /// Calcium chloride, anhydrous `CaCl₂`.
    Cacl2,
    /// Baking soda, `NaHCO₃`.
    Soda,
    /// Chalk, `CaCO₃`.
    Chalk,
    /// Magnesium chloride, `MgCl₂·6H₂O`.
    Mgcl2,
    /// Pickling lime, `Ca(OH)₂`.
    Lime,
}

impl Salt {
    /// All salts in matrix column order.
    pub const ALL: [Salt; 8] = [
        Salt::Gypsum,
        Salt::Nacl,
        Salt::Epsom,
        Salt::Cacl2,
        Salt::Soda,
        Salt::Chalk,
        Salt::Mgcl2,
        Salt::Lime,
    ];

    /// Ion contribution in ppm per 1 g of salt in 1 US gallon: Ca, Mg, Na, SO₄, Cl, HCO₃ (spec §1.1).
    #[must_use]
    pub fn contribution(self) -> [f64; ION_COUNT] {
        match self {
            Salt::Gypsum => [61.478, 0.0, 0.0, 147.359, 0.0, 0.0],
            Salt::Nacl => [0.0, 0.0, 103.891, 0.0, 160.212, 0.0],
            Salt::Epsom => [0.0, 26.043, 0.0, 102.933, 0.0, 0.0],
            Salt::Cacl2 => [95.371, 0.0, 0.0, 0.0, 168.731, 0.0],
            Salt::Soda => [0.0, 0.0, 72.275, 0.0, 0.0, 191.827],
            Salt::Chalk => [105.754, 0.0, 0.0, 0.0, 0.0, 322.016],
            Salt::Mgcl2 => [0.0, 31.573, 0.0, 0.0, 92.111, 0.0],
            Salt::Lime => [142.856, 0.0, 0.0, 0.0, 0.0, 434.991],
        }
    }

    /// Lower-case key used in the parity vectors.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Salt::Gypsum => "gypsum",
            Salt::Nacl => "nacl",
            Salt::Epsom => "epsom",
            Salt::Cacl2 => "cacl2",
            Salt::Soda => "soda",
            Salt::Chalk => "chalk",
            Salt::Mgcl2 => "mgcl2",
            Salt::Lime => "lime",
        }
    }

    /// Parses [`Salt::key`].
    #[must_use]
    pub fn from_key(key: &str) -> Option<Salt> {
        Salt::ALL.into_iter().find(|s| s.key() == key)
    }
}

/// Ion increase from salt additions (spec §2), ppm: Ca, Mg, Na, SO₄, Cl, HCO₃.
///
/// `grams` is a list of (salt, grams); `vol_l` the water volume in litres. Returns zeros when
/// the volume is not positive.
#[must_use]
pub fn salt_ppm(grams: &[(Salt, f64)], vol_l: f64) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    if vol_l <= 0.0 {
        return out;
    }
    let gal = vol_l / GAL_L;
    for &(salt, g) in grams {
        let a = salt.contribution();
        for (o, k) in out.iter_mut().zip(a) {
            *o += g / gal * k;
        }
    }
    out
}

/// Alkalinity in ppm `CaCO₃` from bicarbonate in ppm: `HCO₃ · 50 / 61` (spec §4.1).
#[must_use]
pub fn alkalinity(hco3_ppm: f64) -> f64 {
    hco3_ppm * 50.0 / 61.0
}

/// Effective (Kolbach) hardness in ppm `CaCO₃`: `Ca/1.4 + Mg/1.7` (spec §4.2).
#[must_use]
pub fn hardness(ca_ppm: f64, mg_ppm: f64) -> f64 {
    ca_ppm / 1.4 + mg_ppm / 1.7
}

/// Residual alkalinity in ppm `CaCO₃`: alkalinity minus effective hardness (spec §4.3).
#[must_use]
pub fn residual_alkalinity(ca_ppm: f64, mg_ppm: f64, hco3_ppm: f64) -> f64 {
    alkalinity(hco3_ppm) - hardness(ca_ppm, mg_ppm)
}

/// Sulphate to chloride ratio (spec §4.4); returns 0 when chloride is 0.
#[must_use]
pub fn sulfate_chloride_ratio(so4_ppm: f64, cl_ppm: f64) -> f64 {
    if cl_ppm == 0.0 { 0.0 } else { so4_ppm / cl_ppm }
}

/// Sulphate/chloride balance class (spec §4.4, `SulfateBalFunc` thresholds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SulfateChlorideBalance {
    /// Ratio outside `0..=18`.
    OutOfRange,
    /// Ratio below 0.4.
    ExtremelyMalty,
    /// 0.4 up to 0.6.
    VeryMalty,
    /// 0.6 up to 0.8.
    Malty,
    /// 0.8 up to 1.5.
    Balanced,
    /// 1.5 up to 2.01.
    SlightlyBitter,
    /// 2.01 up to 6.01.
    Bitter,
    /// 6.01 up to 8.01.
    VeryBitter,
    /// 8.01 and above.
    ExtremelyBitter,
}

/// Sulphate/chloride balance class of a ratio (spec §4.4, `SulfateBalFunc` thresholds).
#[must_use]
pub fn sulfate_chloride_balance(r: f64) -> SulfateChlorideBalance {
    use SulfateChlorideBalance as B;
    if !(0.0..=18.0).contains(&r) {
        B::OutOfRange
    } else if r < 0.4 {
        B::ExtremelyMalty
    } else if r < 0.6 {
        B::VeryMalty
    } else if r < 0.8 {
        B::Malty
    } else if r < 1.5 {
        B::Balanced
    } else if r < 2.01 {
        B::SlightlyBitter
    } else if r < 6.01 {
        B::Bitter
    } else if r < 8.01 {
        B::VeryBitter
    } else {
        B::ExtremelyBitter
    }
}

/// Beer colour range of [`color_range`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRange {
    /// Lowest suggested colour, SRM.
    pub min_srm: f64,
    /// Highest suggested colour, SRM.
    pub max_srm: f64,
}

/// Suggested beer colour range in SRM from RA (spec §4.5), floored at 0.
#[must_use]
pub fn color_range(ra_ppm: f64) -> ColorRange {
    ColorRange {
        min_srm: f64::max(0.0, 5.2 + 0.082 * ra_ppm),
        max_srm: f64::max(0.0, (ra_ppm + 122.4) / 12.2),
    }
}

/// Inputs of [`boil_scale`].
#[derive(Debug, Clone, Copy)]
pub struct BoilScaleInput {
    /// Boil volume, L.
    pub vb_l: f64,
    /// Post-boil volume, L.
    pub vp_l: f64,
    /// Cooling loss, L.
    pub cool_loss_l: f64,
    /// Trub loss, L.
    pub trub_loss_l: f64,
    /// Fermenter top-up water, L.
    pub top_up_l: f64,
    /// Recipe has boil ingredients.
    pub has_boil_ing: bool,
}

/// Boil concentration factor `S_boil` (spec §3.4). All volumes in litres: boil volume `vb`,
/// post-boil volume `vp`, cooling loss, trub loss and fermenter top-up water.
/// Returns 1 unless `vb > 0`, `vp > 0` and the recipe has boil ingredients (`F_R_HAS_BOIL_ING`).
/// Errors on non-finite input and when a division would be by zero (`vp = cool_loss`, or
/// `vp - cool_loss - trub_loss + top_up = 0`).
pub fn boil_scale(p: &BoilScaleInput) -> Result<f64> {
    let BoilScaleInput {
        vb_l: vb,
        vp_l: vp,
        cool_loss_l: cool_loss,
        trub_loss_l: trub_loss,
        top_up_l: top_up,
        has_boil_ing,
    } = *p;
    for (field, x) in [
        ("vb", vb),
        ("vp", vp),
        ("cool_loss", cool_loss),
        ("trub_loss", trub_loss),
        ("top_up", top_up),
    ] {
        ensure_finite(field, x)?;
    }
    if vb > 0.0 && vp > 0.0 && has_boil_ing {
        let a = vp - cool_loss;
        let b = a - trub_loss;
        if a == 0.0 || b + top_up == 0.0 {
            return Err(Error::invalid("vp", "boil scale divides by zero"));
        }
        finite_result("boil_scale", ((vb / a) * b + top_up) / (b + top_up))
    } else {
        Ok(1.0)
    }
}

/// Grist acidity function `g(c)` for malt colour `color_srm` in SRM (spec §5.1).
///
/// BeerSmith form is discontinuous at 14 SRM; with `ours` the smaller of the two lines is used
/// below 200 SRM (continuous, spec §5.1 "Bizim model"). 42 from 200 SRM on in both.
#[must_use]
pub fn grist_g(color_srm: f64, ours: bool) -> f64 {
    if color_srm >= 200.0 {
        42.0
    } else if ours {
        f64::min(color_srm - 1.9, 0.453 * color_srm + 5.0)
    } else if color_srm < 14.0 {
        color_srm - 1.9
    } else {
        0.453 * color_srm + 5.0
    }
}

/// Grist pH and the grist sum `S` of [`grist_ph`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GristPh {
    /// Grist pH.
    pub ph: f64,
    /// Grist sum `S`.
    pub s: f64,
}

impl GristPh {
    fn new(s: f64) -> Self {
        Self {
            ph: 5.72 - 0.0337 * s,
            s,
        }
    }
}

/// Grist pH (spec §5.1). `bill` is a list of (weight kg, colour SRM). Returns
/// the pH and `S`; an empty or zero-weight bill gives pH 7.0 and `S` 0.
#[must_use]
pub fn grist_ph(bill: &[(f64, f64)], ours: bool) -> GristPh {
    let total: f64 = bill.iter().map(|&(w, _)| w).sum();
    if total <= 0.0 {
        return GristPh { ph: 7.0, s: 0.0 };
    }
    let s: f64 = bill
        .iter()
        .map(|&(w, c)| w / total * grist_g(c, ours))
        .sum();
    GristPh::new(s)
}

/// Grain types counted in the grist sum (spec §5.1): `F_G_TYPE` 0 Grain and 3 Adjunct.
#[must_use]
pub fn grist_type_counted(grain_type: GrainType) -> bool {
    matches!(grain_type, GrainType::Grain | GrainType::Adjunct)
}

/// Grist pH with the spec §5.1 grain-type filter. `bill` is a list of (weight kg, colour SRM,
/// `F_G_TYPE`) for all mash grains: the denominator is the weight of every entry, only types 0
/// and 3 add to the sum (sugars and extracts contribute 0, like a base malt at pH 5.72).
/// Returns the pH and `S`; `W <= 0` gives pH 7.0 and `S` 0.
///
/// [`grist_ph`] is the same calculation for a bill in which every entry counts.
#[must_use]
pub fn grist_ph_typed(bill: &[(f64, f64, GrainType)], ours: bool) -> GristPh {
    let total: f64 = bill.iter().map(|&(w, _, _)| w).sum();
    if total <= 0.0 {
        return GristPh { ph: 7.0, s: 0.0 };
    }
    let s: f64 = bill
        .iter()
        .filter(|&&(_, _, ty)| grist_type_counted(ty))
        .map(|&(w, c, _)| w / total * grist_g(c, ours))
        .sum();
    GristPh::new(s)
}

/// RA to pH slope in pH per mEq/L (spec §5.2). `model` 0 = MPH 3, 1 = BW; `t_l_per_kg` is the mash
/// thickness in L/kg.
#[must_use]
pub fn mash_ra_slope(model: MashModel, t_l_per_kg: f64) -> f64 {
    if model == MashModel::Bw {
        t_l_per_kg * 0.479_306_13 * 0.17 / 1.5
    } else {
        0.037 + 0.014 * t_l_per_kg
    }
}

/// Adjusted mash pH with any number of acids (spec §5.5): `pH_grist + m · (RA/50 + Σ acid_ra)`,
/// computed as [`est_mash_ph`]`(grist, RA + 50 · Σ acid_ra, m)`. `acid_ras` are the RA equivalents
/// in mEq/L (negative) of liquid acids ([`misc_acid_ra`]) and acid malts ([`acid_malt_ra`]), summed
/// in the given order. An empty slice gives [`est_mash_ph`].
#[must_use]
pub fn adjusted_ph_multi_acid(grist_ph: f64, ra_ppm: f64, acid_ras: &[f64], slope: f64) -> f64 {
    let sum: f64 = acid_ras.iter().sum();
    est_mash_ph(grist_ph, 50.0 * sum + ra_ppm, slope)
}

/// Estimated mash pH without acid: `pH_grist + RA/50 · slope` (spec §5.3). `ra_ppm` in ppm `CaCO₃`.
#[must_use]
pub fn est_mash_ph(grist_ph: f64, ra_ppm: f64, slope: f64) -> f64 {
    grist_ph + ra_ppm / 50.0 * slope
}

/// Acid pK values `[pK1, pK2, pK3]` (spec §6.1); 20 means "no such proton". None, lactic and
/// acid malt use the lactic row (spec-defined).
#[must_use]
#[allow(clippy::approx_constant)] // 3.14 is citric acid pK1, not pi
pub fn acid_pks(acid: Acid) -> [f64; 3] {
    match acid {
        Acid::Phosphoric => [2.12, 7.2, 12.44],
        Acid::Citric => [3.14, 4.77, 6.39],
        Acid::Acetic => [4.75, 20.0, 20.0],
        Acid::Hcl => [-7.0, 20.0, 20.0],
        Acid::Sulphuric => [-1.0, 1.92, 20.0],
        Acid::Tartaric => [2.98, 4.34, 20.0],
        Acid::None | Acid::Lactic | Acid::AcidMalt => [3.86, 20.0, 20.0],
    }
}

/// Mean protons released per acid molecule at `ph` (mEq/mmol, spec §6.1).
#[must_use]
pub fn acid_fraction(acid: Acid, ph: f64) -> f64 {
    let [k1, k2, k3] = acid_pks(acid);
    let r1 = 10f64.powf(ph - k1);
    let r2 = 10f64.powf(ph - k2);
    let r3 = 10f64.powf(ph - k3);
    let f0 = 1.0 / (1.0 + r1 + r1 * r2 + r1 * r2 * r3);
    r1 * f0 + 2.0 * r1 * r2 * f0 + 3.0 * r1 * r2 * r3 * f0
}

/// Dissociation factor `f` used for mash acids (spec §5.4, §6.2, §6.8).
///
/// BeerSmith: only model 1, at fixed pH 5.4, else 1. Own model: at the target pH `ph_t` for
/// both models.
#[must_use]
pub fn dissociation_fraction(acid: Acid, model: MashModel, ph_t: f64, ours: bool) -> f64 {
    if ours {
        acid_fraction(acid, ph_t)
    } else if model == MashModel::Bw {
        acid_fraction(acid, 5.4)
    } else {
        1.0
    }
}

/// Inputs of [`acid_per_liter`].
#[derive(Debug, Clone, Copy)]
pub struct AcidPerLiterInput {
    /// Acid demand, mEq/L.
    pub meq_l: f64,
    /// Acid.
    pub acid: Acid,
    /// Acid strength, %.
    pub conc_pct: f64,
    /// Mash pH model.
    pub model: MashModel,
    /// Target pH for the dissociation fraction.
    pub ph_t: f64,
    /// Use the own model variant.
    pub ours: bool,
}

/// Acid needed per litre of mash water for `meq` mEq/L (spec §6.2): mL of solution for liquid
/// acids, grams of malt for acid malt (code 3). `conc` is percent strength. Code 0 gives 0.
/// Codes 4 to 8 have no molar mass or density in the spec: error (T1.16, was phosphoric
/// constants). Also errors on non-finite input and `conc <= 0` (zero division).
pub fn acid_per_liter(p: &AcidPerLiterInput) -> Result<f64> {
    let AcidPerLiterInput {
        meq_l: meq,
        acid,
        conc_pct: conc,
        model,
        ph_t,
        ours,
    } = *p;
    ensure_finite("meq", meq)?;
    ensure_finite("ph_t", ph_t)?;
    if !acid.is_mash_defined() {
        return Err(Error::invalid(
            "acid",
            "no mash amount defined for this acid",
        ));
    }
    if acid == Acid::None {
        return Ok(0.0);
    }
    ensure_positive("conc", conc)?;
    let (mw, rho) = if acid == Acid::Lactic || acid == Acid::AcidMalt {
        (90.09, 1.0 + conc * 0.237 / 100.0)
    } else {
        let p = conc / 100.0;
        (98.0, 1.0 + 0.49 * conc / 100.0 + 0.375 * p.powi(2))
    };
    let f = dissociation_fraction(acid, model, ph_t, ours);
    finite_result(
        "acid_per_liter",
        if acid == Acid::AcidMalt {
            meq * 100.0 * mw / (conc * f * 1000.0)
        } else {
            meq * 100.0 * mw / (conc * f * rho * 1000.0)
        },
    )
}

/// Inputs of [`mash_acid_amount`].
#[derive(Debug, Clone, Copy)]
pub struct MashAcidAmountInput {
    /// Estimated mash pH without acid.
    pub est_ph: f64,
    /// Target mash pH.
    pub target_ph: f64,
    /// Mash pH model.
    pub model: MashModel,
    /// Mash thickness, L/kg.
    pub t_l_per_kg: f64,
    /// Mash grain, kg.
    pub w_kg: f64,
    /// Acid.
    pub acid: Acid,
    /// Acid strength, %.
    pub conc_pct: f64,
    /// Use the own model variant.
    pub ours: bool,
}

/// Total mash acid amount (spec §6.3): mL for liquid acids, grams for acid malt, for the
/// whole mash of `w_kg` kg grain at `t` L/kg. Zero when no acid is chosen or the estimate is
/// already at or below target. Errors (T1.16) on non-finite input, `t <= 0` or `w_kg <= 0`
/// (spec §5.2: mash thickness or grain weight missing), `conc <= 0` and undefined acids.
pub fn mash_acid_amount(p: &MashAcidAmountInput) -> Result<f64> {
    let MashAcidAmountInput {
        est_ph,
        target_ph,
        model,
        t_l_per_kg: t,
        w_kg,
        acid,
        conc_pct: conc,
        ours,
    } = *p;
    ensure_finite("est_ph", est_ph)?;
    ensure_finite("target_ph", target_ph)?;
    let slope = mash_ra_slope_checked(model, t, w_kg)?;
    if !acid.is_mash_defined() {
        return Err(Error::invalid(
            "acid",
            "no mash amount defined for this acid",
        ));
    }
    let dph = est_ph - target_ph;
    if acid == Acid::None || dph < 0.0 {
        return Ok(0.0);
    }
    Ok(acid_per_liter(&AcidPerLiterInput {
        meq_l: dph / slope,
        acid,
        conc_pct: conc,
        model,
        ph_t: target_ph,
        ours,
    })? * w_kg
        * t)
}

/// Adjusted mash pH and total grain of [`adjusted_ph`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdjustedPh {
    /// Adjusted mash pH.
    pub ph: f64,
    /// Total grain, kg.
    pub grain_kg: f64,
}

/// Inputs of [`adjusted_ph`].
#[derive(Debug, Clone, Copy)]
pub struct AdjustedPhInput<'a> {
    /// Mash grain bill `(weight kg, colour SRM, grain type)`; the type filter of spec §5.1 applies.
    pub bill: &'a [(f64, f64, GrainType)],
    /// Acid malt added, g.
    pub acid_malt_g: f64,
    /// Acid malt acid content, %.
    pub acid_pct: f64,
    /// Mash pH model.
    pub model: MashModel,
    /// Residual alkalinity, ppm as `CaCO₃`.
    pub ra_ppm: f64,
    /// Mash thickness, L/kg.
    pub t_l_per_kg: f64,
    /// Target mash pH.
    pub target_ph: f64,
    /// Use the own model variant.
    pub ours: bool,
}

/// Adjusted mash pH with `acid_malt_g` grams of acid malt (3 SRM, `pct` percent acid) added to
/// the bill (spec §5.5), grist pH with the §5.1 grain-type filter (T1.17; the added malt is type Grain). Returns the pH and the total grain in kg. `ra` in ppm `CaCO₃`. Errors
/// (T1.16) on non-finite input and when the total grain weight or `t` is not positive (the
/// reference divides by `W * t`).
pub fn adjusted_ph(p: &AdjustedPhInput<'_>) -> Result<AdjustedPh> {
    let AdjustedPhInput {
        bill,
        acid_malt_g,
        acid_pct: pct,
        model,
        ra_ppm: ra,
        t_l_per_kg: t,
        target_ph,
        ours,
    } = *p;
    for (field, x) in [
        ("acid_malt_g", acid_malt_g),
        ("pct", pct),
        ("ra", ra),
        ("t", t),
        ("target_ph", target_ph),
    ] {
        ensure_finite(field, x)?;
    }
    for &(w, c, _) in bill {
        ensure_finite("bill weight", w)?;
        ensure_finite("bill colour", c)?;
    }
    let mut b = bill.to_vec();
    if acid_malt_g > 0.0 {
        b.push((acid_malt_g / 1000.0, 3.0, GrainType::Grain));
    }
    let w: f64 = b.iter().map(|&(wj, _, _)| wj).sum();
    if w <= 0.0 || t <= 0.0 {
        return Err(Error::invalid(
            "bill",
            "grain weight and thickness must be > 0",
        ));
    }
    let gp = grist_ph_typed(&b, ours).ph;
    let slope = mash_ra_slope(model, t);
    let f = dissociation_fraction(Acid::AcidMalt, model, target_ph, ours);
    let acid_ra = -((pct / 100.0) * 1000.0 * (acid_malt_g / 1000.0) * f) / (w * t * 0.09009);
    Ok(AdjustedPh {
        ph: gp + slope * (ra / 50.0 + acid_ra),
        grain_kg: w,
    })
}

/// Outcome of the iterative acid malt calculation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AcidMaltResult {
    /// Total acid malt in grams.
    pub total_g: f64,
    /// Adjusted mash pH after the last round.
    pub ph: f64,
    /// True when the loop stopped because `pH - target < 0.005`; false when all 8 rounds were
    /// used without reaching the target.
    pub converged: bool,
}

/// Inputs of [`acid_malt_amount`].
#[derive(Debug, Clone, Copy)]
pub struct AcidMaltInput<'a> {
    /// Mash grain bill `(weight kg, colour SRM, grain type)`; the type filter of spec §5.1 applies.
    pub bill: &'a [(f64, f64, GrainType)],
    /// Acid malt acid content, %.
    pub acid_pct: f64,
    /// Mash pH model.
    pub model: MashModel,
    /// Residual alkalinity, ppm as `CaCO₃`.
    pub ra_ppm: f64,
    /// Mash thickness, L/kg.
    pub t_l_per_kg: f64,
    /// Target mash pH.
    pub target_ph: f64,
    /// Use the own model variant.
    pub ours: bool,
}

/// Iterative acid malt amount (spec §6.4), at most 8 rounds. `pct` is the malt's acid percent;
/// the added malt is 3 SRM. `converged` is false when the 8 rounds ran out and the pH is still
/// not within 0.005 of the target. Same errors as [`adjusted_ph`] plus `pct <= 0`.
pub fn acid_malt_amount(p: &AcidMaltInput<'_>) -> Result<AcidMaltResult> {
    let AcidMaltInput {
        bill,
        acid_pct: pct,
        model,
        ra_ppm: ra,
        t_l_per_kg: t,
        target_ph,
        ours,
    } = *p;
    ensure_positive("pct", pct)?;
    let mut total = 0.0;
    let mut ph2 = 0.0;
    let mut converged = false;
    for _ in 0..8 {
        let AdjustedPh { ph, grain_kg: w } = adjusted_ph(&AdjustedPhInput {
            bill,
            acid_malt_g: total,
            acid_pct: pct,
            model,
            ra_ppm: ra,
            t_l_per_kg: t,
            target_ph,
            ours,
        })?;
        let slope = mash_ra_slope(model, t);
        let amt = if ph - target_ph >= 0.0 {
            acid_per_liter(&AcidPerLiterInput {
                meq_l: (ph - target_ph) / slope,
                acid: Acid::AcidMalt,
                conc_pct: pct,
                model,
                ph_t: target_ph,
                ours,
            })? * w
                * t
        } else {
            0.0
        };
        total += amt;
        ph2 = adjusted_ph(&AdjustedPhInput {
            bill,
            acid_malt_g: total,
            acid_pct: pct,
            model,
            ra_ppm: ra,
            t_l_per_kg: t,
            target_ph,
            ours,
        })?
        .ph;
        if ph2 - target_ph < 0.005 {
            converged = true;
            break;
        }
    }
    Ok(AcidMaltResult {
        total_g: total,
        ph: ph2,
        converged,
    })
}

/// Inputs of [`adjusted_ph_liquid`].
#[derive(Debug, Clone, Copy)]
pub struct AdjustedPhLiquidInput {
    /// Grist pH.
    pub grist_ph: f64,
    /// Residual alkalinity, ppm as `CaCO₃`.
    pub ra_ppm: f64,
    /// Mash thickness, L/kg.
    pub t_l_per_kg: f64,
    /// Mash pH model.
    pub model: MashModel,
    /// Mash grain, kg.
    pub w_kg: f64,
    /// Acid.
    pub acid: Acid,
    /// Acid strength, %.
    pub conc_pct: f64,
    /// Acid volume, L.
    pub vol_l: f64,
    /// Target mash pH.
    pub target_ph: f64,
    /// Use the own model variant.
    pub ours: bool,
}

/// Adjusted mash pH with a liquid acid (spec §5.5): `pH_grist + m · (RA/50 + MiscAcidRA)`.
/// lactic or phosphoric (other acids add no acid), `conc` percent, `vol_l` acid volume in
/// litres, `w_kg` mash grain, `t` L/kg, `target_ph` as in [`dissociation_fraction`]. Errors on
/// non-finite input and `w_kg * t = 0`.
pub fn adjusted_ph_liquid(p: &AdjustedPhLiquidInput) -> Result<f64> {
    let AdjustedPhLiquidInput {
        grist_ph,
        ra_ppm: ra,
        t_l_per_kg: t,
        model,
        w_kg,
        acid,
        conc_pct: conc,
        vol_l,
        target_ph,
        ours,
    } = *p;
    ensure_finite("grist_ph", grist_ph)?;
    ensure_finite("ra", ra)?;
    ensure_finite("target_ph", target_ph)?;
    let f = if acid == Acid::Lactic || acid == Acid::Phosphoric {
        dissociation_fraction(acid, model, target_ph, ours)
    } else {
        1.0
    };
    Ok(grist_ph
        + mash_ra_slope(model, t)
            * (ra / 50.0
                + misc_acid_ra(&MiscAcidRaInput {
                    acid,
                    conc_pct: conc,
                    vol_l,
                    f,
                    w_kg,
                    t_l_per_kg: t,
                })?))
}

/// Misc unit to grams (spec §0.1 `ToGrams`): 0 mg, 1 g, 2 oz, 3 lb, 4 kg, 5 mL (taken as grams,
/// spec "belirsiz"), 6 tsp and 7 tbsp (0.8 g/mL). Errors on a non-finite amount.
pub fn to_grams(amount: f64, units: MiscUnit) -> Result<f64> {
    ensure_finite("amount", amount)?;
    let lb_g = crate::units::LB_KG * 1000.0;
    Ok(match units {
        MiscUnit::Mg => amount / 1000.0,
        MiscUnit::G | MiscUnit::Ml => amount,
        MiscUnit::Oz => amount * lb_g / 16.0,
        MiscUnit::Lb => amount * lb_g,
        MiscUnit::Kg => amount * 1000.0,
        MiscUnit::Tsp => amount * 0.8 * 4.93,
        MiscUnit::Tbsp => amount * 0.8 * 14.78,
    })
}

/// Sparge water volume in litres: total water minus the main mash water `w_kg · t` (spec §6.5).
#[must_use]
pub fn sparge_volume_l(total_l: f64, w_kg: f64, t_l_per_kg: f64) -> f64 {
    total_l - w_kg * t_l_per_kg
}

/// Sparge acid, model 0 (closed carbonate system charge balance, spec §6.5), mol/L.
/// `hco3_ppm` in ppm, water pH `ph_w`, target pH `ph_t`.
#[must_use]
pub fn sparge_acid_mph(hco3_ppm: f64, ph_w: f64, ph_t: f64) -> f64 {
    let c = hco3_ppm / 61_016.0;
    let k1 = 4.4435e-07;
    let k1k2 = 2.073_781_45e-17;
    let k2 = 4.667e-11;
    let xw = 10f64.powf(ph_w);
    let xt = 10f64.powf(ph_t);
    let co3 = xw * c * k2;
    let ct = (k1k2 * xw * xw + k1 * xw + 1.0) * c / (k1 * xw);
    (10f64.powf(-ph_t) + (10f64.powf(ph_w - 14.0) + 2.0 * co3 + c) - 10f64.powf(-ph_w))
        - 10f64.powf(ph_t - 14.0)
        - (2.0 * k1k2 * xt * xt + k1 * xt) * ct / (k1k2 * xt * xt + k1 * xt + 1.0)
}

/// Sparge acid, own model 0: [`sparge_acid_mph`] divided by the acid's dissociation fraction at
/// the sparge target pH (spec §6.8), mol/L.
#[must_use]
pub fn sparge_acid_mph_ours(hco3_ppm: f64, ph_w: f64, ph_t: f64, acid: Acid) -> f64 {
    sparge_acid_mph(hco3_ppm, ph_w, ph_t) / acid_fraction(acid, ph_t)
}

/// Sparge acid, model 1 (Bru'n Water style alkalinity titration, spec §6.5), mol/L.
/// Not defined for `ph_w <= 4.3` (water already acidic): returns 0.
#[must_use]
pub fn sparge_acid_bw(hco3_ppm: f64, ph_w: f64, ph_t: f64, acid: Acid) -> f64 {
    if ph_w <= 4.3 {
        return 0.0;
    }
    let alpha = |ph: f64| {
        let r1 = 10f64.powf(ph - 6.38);
        let r2 = 10f64.powf(ph - 10.373);
        let d = 1.0 + r1 + r1 * r2;
        (1.0 / d, r1 * r2 / d)
    };
    let (a0w, a2w) = alpha(ph_w);
    let (a0e, a2e) = alpha(4.3);
    let (a0t, a2t) = alpha(ph_t);
    let ct = (hco3_ppm * 50.0 / 61.0 / 50.0) / ((a0e - a0w) + (a2w - a2e));
    ct * ((a2w - a2t) + (a0t - a0w)) / acid_fraction(acid, ph_t) / 1000.0
}

/// Calculation warnings (spec §6.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warning {
    /// BW model is not yet validated against measured mash pH.
    BwModelUnverified,
    /// Chalk dissolves poorly; effect is overstated without acidic mash.
    ChalkSolubility,
}

/// Warnings for the chosen model and salt additions in grams (spec §6.8).
#[must_use]
pub fn warnings(model: MashModel, salt_grams: &[(Salt, f64)]) -> Vec<Warning> {
    let mut out = Vec::new();
    if model == MashModel::Bw {
        out.push(Warning::BwModelUnverified);
    }
    let chalk: f64 = salt_grams
        .iter()
        .filter(|(s, _)| *s == Salt::Chalk)
        .map(|&(_, g)| g)
        .sum();
    if chalk > 0.0 {
        out.push(Warning::ChalkSolubility);
    }
    out
}

/// Least squares for the given columns via Householder QR; `cols` are columns of equal length
/// `b.len()` and at most that many.
fn lstsq(cols: &[&[f64]], b: &[f64]) -> Vec<f64> {
    let m = b.len();
    let k = cols.len();
    let mut a: Vec<Vec<f64>> = cols.iter().map(|c| c.to_vec()).collect();
    let mut qtb = b.to_vec();
    for j in 0..k {
        let norm = (j..m).map(|i| a[j][i] * a[j][i]).sum::<f64>().sqrt();
        if norm == 0.0 {
            continue;
        }
        let alpha = if a[j][j] > 0.0 { -norm } else { norm };
        let mut v: Vec<f64> = a[j][j..].to_vec();
        v[0] -= alpha;
        let vn2: f64 = v.iter().map(|x| x * x).sum();
        if vn2 == 0.0 {
            continue;
        }
        for c in j..k {
            let dot: f64 = v.iter().enumerate().map(|(i, vi)| vi * a[c][j + i]).sum();
            let f = 2.0 * dot / vn2;
            for (i, vi) in v.iter().enumerate() {
                a[c][j + i] -= f * vi;
            }
        }
        let dot: f64 = v.iter().enumerate().map(|(i, vi)| vi * qtb[j + i]).sum();
        let f = 2.0 * dot / vn2;
        for (i, vi) in v.iter().enumerate() {
            qtb[j + i] -= f * vi;
        }
    }
    let mut x = vec![0.0; k];
    for i in (0..k).rev() {
        let mut s = qtb[i];
        for c in (i + 1)..k {
            s -= a[c][i] * x[c];
        }
        let d = a[i][i];
        x[i] = if d.abs() > 1e-300 { s / d } else { 0.0 };
    }
    x
}

/// Solution of [`nnls`].
#[derive(Debug, Clone, PartialEq)]
pub struct Nnls {
    /// The non-negative solution.
    pub x: Vec<f64>,
    /// Residual norm `‖A x − b‖₂`.
    pub residual_norm: f64,
}

/// Non-negative least squares, Lawson–Hanson active set method (spec §7.1): minimise
/// `‖A x − b‖₂` subject to `x ≥ 0`. `a` holds the columns of `A` (each of length `b.len()`).
/// At most `3n` iterations. Returns `x` and `‖A x − b‖₂`.
///
/// The solution `x` is not unique when columns are linearly dependent; `A x` and the residual
/// norm always are.
#[must_use]
pub fn nnls(a: &[Vec<f64>], b: &[f64]) -> Nnls {
    let n = a.len();
    let m = b.len();
    let mut x = vec![0.0; n];
    let mut passive = vec![false; n];
    let anorm = a
        .iter()
        .map(|c| c.iter().map(|v| v.abs()).sum::<f64>())
        .fold(0.0, f64::max);
    let tol = 10.0 * (m.max(n) as f64) * f64::EPSILON * anorm;
    let max_iter = 3 * n;
    let residual = |x: &[f64]| -> Vec<f64> {
        (0..m)
            .map(|i| b[i] - (0..n).map(|j| a[j][i] * x[j]).sum::<f64>())
            .collect()
    };
    let mut it = 0;
    loop {
        let r = residual(&x);
        let w: Vec<f64> = (0..n)
            .map(|j| (0..m).map(|i| a[j][i] * r[i]).sum())
            .collect();
        let mut best: Option<usize> = None;
        for j in 0..n {
            if !passive[j] && w[j] > tol && best.is_none_or(|bj| w[j] > w[bj]) {
                best = Some(j);
            }
        }
        let Some(best) = best else { break };
        passive[best] = true;
        loop {
            let idx: Vec<usize> = (0..n).filter(|&j| passive[j]).collect();
            let cols: Vec<&[f64]> = idx.iter().map(|&j| a[j].as_slice()).collect();
            let s = lstsq(&cols, b);
            if s.iter().all(|&v| v > 0.0) {
                x.fill(0.0);
                for (&j, &v) in idx.iter().zip(&s) {
                    x[j] = v;
                }
                break;
            }
            let alpha = idx
                .iter()
                .zip(&s)
                .filter(|&(_, &sv)| sv <= 0.0)
                .map(|(&j, &sv)| x[j] / (x[j] - sv))
                .fold(f64::INFINITY, f64::min);
            for (&j, &sv) in idx.iter().zip(&s) {
                x[j] += alpha * (sv - x[j]);
            }
            for &j in &idx {
                if x[j] <= tol {
                    x[j] = 0.0;
                    passive[j] = false;
                }
            }
            it += 1;
            if it > max_iter {
                break;
            }
        }
        it += 1;
        if it > max_iter {
            break;
        }
    }
    let rn = residual(&x).iter().map(|v| v * v).sum::<f64>().sqrt();
    Nnls {
        x,
        residual_norm: rn,
    }
}

/// Solution of [`water_match`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SaltMatch {
    /// Grams per US gallon for every salt in [`Salt::ALL`] order.
    pub grams_per_gal: [f64; 8],
    /// Residual norm, ppm.
    pub residual_norm: f64,
}

/// Best salt additions to reach a target profile (spec §7.1). `base` and `target` are ion
/// profiles in ppm (Ca, Mg, Na, SO₄, Cl, HCO₃), `s` the base scaling (1 for mash recipes,
/// `S_boil` for extract recipes), `excluded` salts are fixed at 0.
///
/// Returns grams per US gallon for every salt in [`Salt::ALL`] order, and the residual norm.
#[must_use]
pub fn water_match(
    base: &[f64; ION_COUNT],
    target: &[f64; ION_COUNT],
    s: f64,
    excluded: &[Salt],
) -> SaltMatch {
    let active: Vec<Salt> = Salt::ALL
        .into_iter()
        .filter(|salt| !excluded.contains(salt))
        .collect();
    let cols: Vec<Vec<f64>> = active
        .iter()
        .map(|salt| salt.contribution().to_vec())
        .collect();
    let b: Vec<f64> = (0..ION_COUNT).map(|i| target[i] - s * base[i]).collect();
    let Nnls {
        x,
        residual_norm: rn,
    } = nnls(&cols, &b);
    let mut out = [0.0; 8];
    for (salt, v) in active.iter().zip(x) {
        let pos = Salt::ALL.iter().position(|q| q == salt).unwrap_or(0);
        out[pos] = v;
    }
    SaltMatch {
        grams_per_gal: out,
        residual_norm: rn,
    }
}

/// Resulting profile of a [`water_match`] solution (spec §7.3): `s·base + Σ x_k · A_k` in ppm.
#[must_use]
pub fn best_fit_profile(base: &[f64; ION_COUNT], s: f64, x: &[f64; 8]) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    for (i, o) in out.iter_mut().enumerate() {
        *o = s * base[i]
            + Salt::ALL
                .iter()
                .zip(x)
                .map(|(salt, xk)| xk * salt.contribution()[i])
                .sum::<f64>();
    }
    out
}

/// RA to pH slope with the spec §5.2 edge case: `None` (error, "mash thickness or grain weight
/// missing") when `t <= 0` or `w_kg <= 0` (also non-finite).
pub fn mash_ra_slope_checked(model: MashModel, t_l_per_kg: f64, w_kg: f64) -> Result<f64> {
    ensure_positive("t_l_per_kg", t_l_per_kg)?;
    ensure_positive("w_kg", w_kg)?;
    Ok(mash_ra_slope(model, t_l_per_kg))
}

/// Inputs of [`misc_acid_ra`].
#[derive(Debug, Clone, Copy)]
pub struct MiscAcidRaInput {
    /// Acid.
    pub acid: Acid,
    /// Acid strength, %.
    pub conc_pct: f64,
    /// Acid volume, L.
    pub vol_l: f64,
    /// Dissociation factor.
    pub f: f64,
    /// Mash grain, kg.
    pub w_kg: f64,
    /// Mash thickness, L/kg.
    pub t_l_per_kg: f64,
}

/// RA equivalent in mEq/L (negative) of a liquid mash acid (spec §5.4a): `acid` 1 lactic,
/// 2 phosphoric (density coefficient 0.365, unlike [`acid_per_liter`]'s 0.375). Other codes
/// are not scanned by `FilterMashAgents` and contribute 0. Errors on non-finite input and
/// `w_kg * t = 0`.
pub fn misc_acid_ra(p: &MiscAcidRaInput) -> Result<f64> {
    let MiscAcidRaInput {
        acid,
        conc_pct: conc,
        vol_l,
        f,
        w_kg,
        t_l_per_kg: t,
    } = *p;
    for (field, x) in [
        ("conc", conc),
        ("vol_l", vol_l),
        ("f", f),
        ("w_kg", w_kg),
        ("t", t),
    ] {
        ensure_finite(field, x)?;
    }
    let p = conc / 100.0;
    let (mw, rho) = match acid {
        Acid::Lactic => (0.09009, 1.0 + 0.237 * p),
        Acid::Phosphoric => (0.098, 1.0 + 0.49 * p + 0.365 * p * p),
        _ => return Ok(0.0),
    };
    if w_kg * t == 0.0 {
        return Err(Error::invalid(
            "w_kg",
            "grain weight and thickness must be non-zero",
        ));
    }
    Ok(-(p * 1000.0 * rho * vol_l * f) / (mw * w_kg * t))
}

/// Inputs of [`acid_malt_ra`].
#[derive(Debug, Clone, Copy)]
pub struct AcidMaltRaInput {
    /// Acid malt acid content, %.
    pub acid_pct: f64,
    /// Acid malt, kg.
    pub malt_kg: f64,
    /// Dissociation factor.
    pub f: f64,
    /// Mash grain, kg.
    pub w_kg: f64,
    /// Mash thickness, L/kg.
    pub t_l_per_kg: f64,
}

/// RA equivalent in mEq/L (negative) of acid malt (spec §5.4b); `w_kg` is total mash grain.
pub fn acid_malt_ra(p: &AcidMaltRaInput) -> Result<f64> {
    let AcidMaltRaInput {
        acid_pct: pct,
        malt_kg,
        f,
        w_kg,
        t_l_per_kg: t,
    } = *p;
    for (field, x) in [
        ("pct", pct),
        ("malt_kg", malt_kg),
        ("f", f),
        ("w_kg", w_kg),
        ("t", t),
    ] {
        ensure_finite(field, x)?;
    }
    if w_kg * t == 0.0 {
        return Err(Error::invalid(
            "w_kg",
            "grain weight and thickness must be non-zero",
        ));
    }
    Ok(-((pct / 100.0) * 1000.0 * malt_kg * f) / (w_kg * t * 0.09009))
}

/// Sparge acid volume in mL (spec §6.5 "Hacme çevirme"): lactic (1) or phosphoric (2, density
/// coefficient 0.365); every other code gives 0; a non-positive result is 0.
#[must_use]
pub fn sparge_acid_ml(mol_l: f64, v_l: f64, conc_pct: f64, acid: Acid) -> f64 {
    let c = f64::max(conc_pct, 1.0) / 100.0;
    let ml = match acid {
        Acid::Lactic => 0.09009 / c / (1.0 + 0.237 * c) * mol_l * v_l * 1000.0,
        Acid::Phosphoric => 0.098 / c / (1.0 + 0.49 * c + 0.365 * c * c) * mol_l * v_l * 1000.0,
        _ => return 0.0,
    };
    if ml > 0.0 { ml } else { 0.0 }
}

/// Inputs of [`sparge_acid_amount`].
#[derive(Debug, Clone, Copy)]
pub struct SpargeAcidInput {
    /// Mash pH model.
    pub model: MashModel,
    /// Use the own model variant.
    pub ours: bool,
    /// Bicarbonate, ppm.
    pub hco3_ppm: f64,
    /// Water pH.
    pub ph_w: f64,
    /// Target pH for the dissociation fraction.
    pub ph_t: f64,
    /// Acid.
    pub acid: Acid,
    /// Acid strength, %.
    pub conc_pct: f64,
    /// Mash pH difference.
    pub mash_dph: f64,
    /// Sparge water, L.
    pub v_sp_l: f64,
}

/// Whole sparge acidification (spec §6.5), mL. Zero for acid 0, 3 or other, or when the mash
/// pH difference `mash_dph` is negative. Model 1 uses [`sparge_acid_bw`] (guarded), model 0
/// [`sparge_acid_mph`] (own model: [`sparge_acid_mph_ours`]).
/// Errors on non-finite input or a non-finite result.
pub fn sparge_acid_amount(p: &SpargeAcidInput) -> Result<f64> {
    let SpargeAcidInput {
        model,
        ours,
        hco3_ppm: hco3,
        ph_w,
        ph_t,
        acid,
        conc_pct: conc,
        mash_dph,
        v_sp_l,
    } = *p;
    for (field, x) in [
        ("hco3", hco3),
        ("ph_w", ph_w),
        ("ph_t", ph_t),
        ("conc", conc),
        ("mash_dph", mash_dph),
        ("v_sp_l", v_sp_l),
    ] {
        ensure_finite(field, x)?;
    }
    if !(acid == Acid::Lactic || acid == Acid::Phosphoric) || mash_dph < 0.0 {
        return Ok(0.0);
    }
    let mol = if model == MashModel::Bw {
        sparge_acid_bw(hco3, ph_w, ph_t, acid)
    } else if ours {
        sparge_acid_mph_ours(hco3, ph_w, ph_t, acid)
    } else {
        sparge_acid_mph(hco3, ph_w, ph_t)
    };
    finite_result(
        "sparge_acid_amount",
        sparge_acid_ml(mol, v_sp_l, conc, acid),
    )
}

/// Amount-weighted base ion profile (spec §3.1); `waters` is (amount, ions). Zero profile when
/// the total amount is not positive.
#[must_use]
pub fn base_profile(waters: &[(f64, [f64; ION_COUNT])]) -> [f64; ION_COUNT] {
    let tot: f64 = waters.iter().map(|&(a, _)| a).sum();
    let mut out = [0.0; ION_COUNT];
    if tot <= 0.0 {
        return out;
    }
    for (i, o) in out.iter_mut().enumerate() {
        *o = waters.iter().map(|(a, ion)| ion[i] * a).sum::<f64>() / tot;
    }
    out
}

/// Amount-weighted water pH (spec §3.1); 7.0 when there is no water. `waters` is (amount, pH).
#[must_use]
pub fn water_ph(waters: &[(f64, f64)]) -> f64 {
    let tot: f64 = waters.iter().map(|&(a, _)| a).sum();
    if tot <= 0.0 {
        return 7.0;
    }
    waters.iter().map(|&(a, ph)| ph * a).sum::<f64>() / tot
}

/// Salt additions' ion contribution honouring `F_R_IGNORE_WATER_AGENTS` (spec §2): zeros when
/// `ignore`.
#[must_use]
pub fn agents_ppm(grams: &[(Salt, f64)], vol_l: f64, ignore: bool) -> [f64; ION_COUNT] {
    if ignore {
        [0.0; ION_COUNT]
    } else {
        salt_ppm(grams, vol_l)
    }
}

/// Inputs of [`agent_volume`].
#[derive(Debug, Clone, Copy)]
pub struct AgentVolumeInput {
    /// Recipe type.
    pub rtype: RecipeType,
    /// Misc-use code.
    pub use_: u8,
    /// Mash tot water.
    pub mash_tot_water_l: f64,
    /// Sparge volume, L.
    pub sparge_vol_l: f64,
    /// Boil volume, L.
    pub boil_vol_l: f64,
    /// Batch volume, L.
    pub volume_l: f64,
    /// Final volume, L.
    pub final_vol_l: f64,
}

/// Reference volume `F_M_TEMP_VOL` of a water agent (spec §2). `use_` is the misc-use code:
/// 0 boil, 1 mash, 2 primary, 5 sparge, others fall to `final_vol` (spec-defined).
#[must_use]
pub fn agent_volume(p: &AgentVolumeInput) -> f64 {
    let AgentVolumeInput {
        rtype,
        use_,
        mash_tot_water_l: mash_tot_water,
        sparge_vol_l: sparge_vol,
        boil_vol_l: boil_vol,
        volume_l: volume,
        final_vol_l: final_vol,
    } = *p;
    if matches!(rtype, RecipeType::PartialMash | RecipeType::AllGrain) {
        match use_ {
            1 => mash_tot_water,
            5 => sparge_vol,
            0 => boil_vol,
            2 => volume,
            _ => final_vol,
        }
    } else {
        match use_ {
            0 | 1 => boil_vol,
            2 => volume,
            _ => final_vol,
        }
    }
}

/// Inputs of [`total_water_ion`].
#[derive(Debug, Clone, Copy)]
pub struct TotalWaterIonInput<'a> {
    /// Mash profile, ppm.
    pub mash: &'a [f64; ION_COUNT],
    /// Sparge profile, ppm.
    pub sparge: &'a [f64; ION_COUNT],
    /// Mash water volume, L.
    pub vm_l: f64,
    /// Sparge water volume, L.
    pub vs_l: f64,
    /// The recipe has a mash.
    pub has_mash: bool,
}

/// Mash + sparge blended profile (spec §3.3 `GetTotalWaterIon`).
#[must_use]
pub fn total_water_ion(p: &TotalWaterIonInput<'_>) -> [f64; ION_COUNT] {
    let TotalWaterIonInput {
        mash,
        sparge,
        vm_l: vm,
        vs_l: vs,
        has_mash,
    } = *p;
    if has_mash && vs > 0.0 {
        let mut out = [0.0; ION_COUNT];
        for (i, o) in out.iter_mut().enumerate() {
            *o = (vm * mash[i] + vs * sparge[i]) / (vm + vs);
        }
        out
    } else {
        *mash
    }
}

/// Kettle top-up dilution factor `k_top` (spec §3.3); 1 when `v_topk_l` is 0.
#[must_use]
pub fn top_up_factor(v_avail_l: f64, vs_l: f64, v_topk_l: f64) -> f64 {
    if v_topk_l == 0.0 {
        1.0
    } else {
        (v_avail_l + vs_l) / (v_avail_l + vs_l + v_topk_l)
    }
}

/// Boil input profile `Tot·k_top + Adj(boil agents)` (spec §3.3).
#[must_use]
pub fn boil_ion(total: &[f64; ION_COUNT], k_top: f64, adj0: &[f64; ION_COUNT]) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    for (i, o) in out.iter_mut().enumerate() {
        *o = total[i] * k_top + adj0[i];
    }
    out
}

/// Fermenter input profile `Boil·S_boil + Adj(primary agents)` (spec §3.3).
#[must_use]
pub fn beer_ion(boil: &[f64; ION_COUNT], s_boil: f64, adj2: &[f64; ION_COUNT]) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    for (i, o) in out.iter_mut().enumerate() {
        *o = boil[i] * s_boil + adj2[i];
    }
    out
}

/// Base scaling `s` of the salt matching objective (spec §7.1): `S_boil` only for extract
/// recipes with boil ingredients, else 1.
#[must_use]
pub fn match_scale(mash_recipe: bool, has_boil_ing: bool, s_boil: f64) -> f64 {
    if mash_recipe || !has_boil_ing {
        1.0
    } else {
        s_boil
    }
}

/// Salt grams of [`salt_grams`], in [`Salt::ALL`] order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SaltGrams {
    /// Mash water additions, g.
    pub mash_g: [f64; 8],
    /// Sparge water additions, g.
    pub sparge_g: [f64; 8],
}

/// Salt grams from per-gallon concentrations `x` (spec §7.2): mash and sparge grams.
/// With `combine` and sparge volume > 0 the volumes are added and no sparge addition is made.
/// The 0.01 g removal threshold of `SetWaterAddition` is the caller's.
#[must_use]
pub fn salt_grams(x: &[f64; 8], v_mash_gal: f64, v_sp_gal: f64, combine: bool) -> SaltGrams {
    let vm = if combine && v_sp_gal > 0.0 {
        v_mash_gal + v_sp_gal
    } else {
        v_mash_gal
    };
    let sp = if v_sp_gal > 0.0 && !combine {
        v_sp_gal
    } else {
        0.0
    };
    SaltGrams {
        mash_g: x.map(|k| k * vm),
        sparge_g: x.map(|k| k * sp),
    }
}

/// Water profile tool mix (spec §7.4): volume-weighted base and dilution water. The spec
/// leaves `Vb + Vd = 0` undefined; this returns the zero profile.
#[must_use]
#[allow(clippy::similar_names)] // base and dilution water (`vb_l`, `vd_l`)
pub fn profile_mix(
    vb_l: f64,
    b: &[f64; ION_COUNT],
    vd_l: f64,
    d: &[f64; ION_COUNT],
) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    if vb_l + vd_l <= 0.0 {
        return out;
    }
    for (i, o) in out.iter_mut().enumerate() {
        *o = (vb_l * b[i] + vd_l * d[i]) / (vb_l + vd_l);
    }
    out
}

/// Water profile tool grams (spec §7.4 quirk): `x · V_b` with the base volume only (gal).
#[must_use]
pub fn wp_grams(x: &[f64; 8], vb_gal: f64) -> [f64; 8] {
    x.map(|k| k * vb_gal)
}
