//! Section 04: yeast, mead, carbonation (`formuller/04-maya-mead-karbonasyon.md`).
//!
//! Units follow the BeerSmith internal units of the specification: gallons, fl oz,
//! degrees Fahrenheit, litres where stated. Every function documents its units.
//! Ported one to one from `formuller/testler/04_maya_karb_tv.py` (ADR-0003).

use crate::density::brix_to_sg;
use crate::error::{Error, Result, ensure_finite, ensure_positive, finite_result};
use crate::recipe::{Yeast as RecipeYeast, YeastForm, code_enum};
use crate::units::{OZ_G, to_plato, to_sg};

/// Stir plate growth bonus, percent (spec §1.2).
const STIR: f64 = 45.0;

/// Cell viability, percent, after `days` (spec §1.4).
///
/// `v0_pct` percent, `rate_pct_per_month` monthly loss percent, floored at `floor_pct` (default 5). Zero or
/// negative `days` return `v0_pct` unchanged.
#[must_use]
pub fn viability(v0_pct: f64, rate_pct_per_month: f64, days: f64, floor_pct: f64) -> f64 {
    if days <= 0.0 {
        return v0_pct;
    }
    let v = v0_pct * ((100.0 - rate_pct_per_month) / 100.0).powf(days / 30.0);
    v.max(floor_pct)
}

/// Inoculation rate, million cells per mL, to growth factor `g` (spec §1.9).
/// Errors when `g` is not a finite positive number (the reference's `log` raises).
pub fn innoc_rate(g: f64) -> Result<f64> {
    ensure_positive("g", g)?;
    finite_result("innoc_rate", (g.ln() * -2.179_103_8 + 5.512_221).exp())
}

/// Growth factor for inoculation rate `inoc_rate_m_ml` (million cells/mL), clamped to 1..6 (spec §1.9).
///
/// Uses the exponent -2.1781038 (differs from [`innoc_rate`] in the reference).
/// Errors when `inoc_rate_m_ml` is not a finite positive number (the reference's `log` raises).
pub fn growth_rate(inoc_rate_m_ml: f64, stir: bool) -> Result<f64> {
    ensure_positive("inoc_rate_m_ml", inoc_rate_m_ml)?;
    let mut g = ((inoc_rate_m_ml.ln() - 5.512_221) / -2.178_103_8).exp();
    if stir {
        g *= 1.0 + STIR / 100.0;
    }
    Ok(g.clamp(1.0, 6.0))
}

/// Cells needed, billions: `rate (M/mL/°P) × volume (L) × °Plato(sg)` (spec §1.8).
#[must_use]
pub fn cells_needed(rate_m_ml_p: f64, volume_l: f64, sg: f64) -> f64 {
    rate_m_ml_p * volume_l * to_plato(sg)
}

/// Single-stage starter volume, litres, for `need_billion` and `avail_billion` billion cells (spec §1.10).
/// Errors on non-finite input and when cells are needed but `avail <= 0` (zero division).
pub fn new_starter_l(need_billion: f64, avail_billion: f64, stir: bool) -> Result<f64> {
    ensure_finite("need_billion", need_billion)?;
    ensure_finite("avail_billion", avail_billion)?;
    if need_billion <= avail_billion {
        return Ok(0.0);
    }
    ensure_positive("avail_billion", avail_billion)?;
    let mut r = need_billion / avail_billion;
    if stir {
        r /= 1.0 + STIR / 100.0;
    }
    if r <= 1.0 {
        return Ok(0.0);
    }
    Ok(avail_billion / innoc_rate(r)?)
}

/// Two-stage starter volumes of [`two_stage_tool`] and [`two_stage_recipe`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarterStages {
    /// First stage, L.
    pub stage1_l: f64,
    /// Second stage, L.
    pub stage2_l: f64,
}

impl StarterStages {
    const NONE: Self = Self {
        stage1_l: 0.0,
        stage2_l: 0.0,
    };
}

/// Two-stage starter volumes in litres (spec §1.11).
/// Errors on non-finite input.
pub fn two_stage_tool(need_billion: f64, avail_billion: f64, stir: bool) -> Result<StarterStages> {
    ensure_finite("need_billion", need_billion)?;
    ensure_finite("avail_billion", avail_billion)?;
    let avail_billion = avail_billion.max(0.1);
    if avail_billion > need_billion {
        return Ok(StarterStages::NONE);
    }
    let mut r = need_billion / avail_billion;
    let mut s = 1.0;
    if stir {
        s = 1.0 + STIR / 100.0;
        r /= s * s;
    }
    let g = r.sqrt();
    let v1 = avail_billion / innoc_rate(g)?;
    let v2 = avail_billion * g * s / innoc_rate(g)?;
    Ok(StarterStages {
        stage1_l: v1,
        stage2_l: v2,
    })
}

/// DME ounces for a starter of `vol_floz` fl oz at gravity `og` (spec §1.14).
#[must_use]
pub fn dme_oz(vol_floz: f64, og: f64) -> f64 {
    (og - 1.0) * 1000.0 * vol_floz / 128.0 / 45.0 * 16.0
}

/// Priming sugar, grams, for `co2_vol` volumes CO2 at `temp_f` °F in `volume_gal` gallons (spec §3.3).
///
/// Corn sugar by default; `dme` multiplies by 1.538.
#[must_use]
pub fn sugar_g(co2_vol: f64, temp_f: f64, volume_gal: f64, dme: bool) -> f64 {
    let t = temp_f.max(32.0);
    let g = ((t * 0.050_062 + (co2_vol - 3.0378)) - t * 0.000_265_55 * t) * volume_gal * 15.195;
    if dme { g * 1.538 } else { g }
}

/// Residual dissolved CO2, volumes, at `temp_f` °F (spec §3.2). Used by section 11.
#[must_use]
pub fn residual_co2(temp_f: f64) -> f64 {
    let t = temp_f.max(32.0);
    3.0378 - 0.050_062 * t + 0.000_265_55 * t * t
}

/// Keg pressure, psi, to hold `co2_vol` volumes CO2 at `temp_f` °F (spec §3.5). Used by section 11.
#[must_use]
pub fn co2_psi(co2_vol: f64, temp_f: f64) -> f64 {
    let t = temp_f.max(32.0);
    let p = co2_vol * 4.24267
        + (-16.6999 - t * 0.010_105_9)
        + t * 0.001_165_12 * t
        + t * 0.173_354 * co2_vol
        - co2_vol * 0.068_422_6 * co2_vol;
    p.max(0.0)
}

/// Recommended free SO2, ppm, for wine `ph` (spec §2.6).
#[must_use]
pub fn so2_rec(ph: f64) -> f64 {
    (ph * 2.21828 - 4.27809).exp()
}

/// Potassium metabisulfite, grams, for `v_gal` gallons to raise free SO2 from `free_ppm` to
/// `target_ppm` ppm (spec §2.6).
#[must_use]
pub fn kmeta_g(v_gal: f64, target_ppm: f64, free_ppm: f64) -> f64 {
    v_gal * 3.785 * (target_ppm - free_ppm) / 576.0
}

/// Potassium sorbate, grams, for `v_gal` gallons (spec §2.7).
#[must_use]
pub fn sorbate_g(v_gal: f64) -> f64 {
    v_gal * 1.25
}

/// Mead yeast pitch rate, g/L, from SG (spec §2.2): 0.25 up to 24 °P, 0.5 from 28 °P.
#[must_use]
pub fn mead_pitch(sg: f64) -> f64 {
    let p = to_plato(sg);
    if p <= 24.0 {
        0.25
    } else if p < 28.0 {
        (p - 24.0) * 0.0625 + 0.25
    } else {
        0.5
    }
}

/// Nutrient used for YAN (spec §2.5): code 0 Fermaid-O (40 ppm per g/L), 1 Fermaid-K (100),
/// 2 DAP (212.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YanMethod {
    /// Code 0.
    FermaidO,
    /// Code 1.
    FermaidK,
    /// Code 2.
    Dap,
}

impl YanMethod {
    /// From the code (0-2); other codes are an error (T1.16, was 100).
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::FermaidO),
            1 => Ok(Self::FermaidK),
            2 => Ok(Self::Dap),
            _ => Err(Error::invalid("yan_method", "unknown code")),
        }
    }

    /// ppm of YAN per g/L of the nutrient.
    #[must_use]
    pub const fn ppm_per_g_l(self) -> f64 {
        match self {
            Self::FermaidO => 40.0,
            Self::FermaidK => 100.0,
            Self::Dap => 212.3,
        }
    }
}

/// Yeast assimilable nitrogen, ppm, from `grams` of nutrient in `v_l` litres (spec §2.5).
/// Zero volume returns the bare factor.
#[must_use]
pub fn yan(method: YanMethod, grams: f64, v_l: f64) -> f64 {
    let r = method.ppm_per_g_l();
    if v_l == 0.0 { r } else { r * grams / v_l }
}

/// Backsweetening honey volume, litres (spec §2.8): `v_l` of `sg_now`, to `sg_target`,
/// honey at `sg_honey`.
#[must_use]
pub fn backsweeten_volume_l(v_l: f64, sg_now: f64, sg_target: f64, sg_honey: f64) -> f64 {
    // Spec §2.8: a target equal to the agent gravity gives 0 (no division by zero).
    #[allow(clippy::float_cmp)]
    let same = sg_target == sg_honey;
    if same {
        return 0.0;
    }
    v_l * (sg_now - sg_target) / (sg_target - sg_honey)
}

/// Honey gravity from its Plato value (spec §2.8).
#[must_use]
pub fn honey_sg(plato_p: f64) -> f64 {
    to_sg(plato_p)
}

// ---------------------------------------------------------------------------
// T1.14f: calculations that the specification defines and the Python reference
// gained in this task (enums, defaults, yeast counts and packs, mead, carbonation).
// ---------------------------------------------------------------------------

code_enum! {
    /// `F_YC_YEAST_FORM` of the starter tool (spec §1.1).
    StarterYeastForm { Liquid = 0, Dry = 1, Slurry = 2 }
}
code_enum! {
    /// `F_YC_TYPE` of the starter tool (spec §1.1, §1.7).
    StarterType { AleOrCider = 0, Lager = 1,
                  HybridWheat = 2, MeadOrWine = 3 }
}
code_enum! {
    /// `F_Y_TYPE` (spec §1.1).
    YeastType { Ale = 0, Lager = 1, Wine = 2,
                Champagne = 3, Wheat = 4 }
}
code_enum! {
    /// `F_R_ALE_TYPE` (spec §1.1).
    AleType { Ale = 0, Lager = 1, Hybrid = 2 }
}
code_enum! {
    /// `F_MT_METHOD` / `F_R_MEAD_METHOD` (spec §2.1). Code 2 (DAP) only appears in [`yan`].
    MeadMethod { Tosna = 0, Tiosna = 1 }
}
code_enum! {
    /// `F_MT_YEAST_FACTOR` / `F_R_YEAST_FACTOR` (spec §2.1).
    YeastNitrogen { Low = 0, Medium = 1, High = 2 }
}
code_enum! {
    /// `F_C_TYPE` / `F_R_CARB_TYPE` (spec §3.1).
    CarbType { Bottle = 0, Keg = 1, KegPriming = 2 }
}
code_enum! {
    /// Backsweetening agent (spec §2.8).
    BackAgent { Honey = 0, AppleJuiceConcentrate = 1,
                Juice = 2, DryMaltExtract = 3,
                TableSugar = 4, Other = 5 }
}

impl YeastNitrogen {
    /// Nitrogen factor `k` of the nutrient programme (spec §2.4).
    #[must_use]
    pub fn factor(self) -> f64 {
        match self {
            Self::Low => 0.75,
            Self::Medium => 0.9,
            Self::High => 1.25,
        }
    }
}

impl BackAgent {
    /// Default sugar potential, degrees Brix (spec §2.8).
    #[must_use]
    pub fn potential_brix(self) -> f64 {
        match self {
            Self::Honey => 82.5,
            Self::AppleJuiceConcentrate => 70.0,
            Self::Juice | Self::Other => 15.0,
            Self::DryMaltExtract => 75.0,
            Self::TableSugar => 98.0,
        }
    }
}

/// True for recipe types that use the wine pitch rate: Mead, Wine, Seltzer and the unlisted 7
/// (spec §1.7). Cider (3) is treated as beer.
#[must_use]
pub fn is_wine_like_recipe(r_type: i64) -> bool {
    matches!(r_type, 4..=7)
}

macro_rules! defaults {
    ($($(#[$m:meta])* $name:ident $key:literal = $val:expr;)+) => {
        $($(#[$m])* pub const $name: f64 = $val;)+

        /// Default value by lower-case option name (spec §1.2, §1.7, §1.13, §2.1, §3.1, §2.8).
        #[must_use]
        pub fn default_value(name: &str) -> Option<f64> {
            match name { $($key => Some($name),)+ _ => None }
        }
    };
}

defaults! {
    /// New dry yeast viability, percent.
    O_Y_DRY_VIABILITY "o_y_dry_viability" = 98.0;
    /// New liquid yeast viability, percent.
    O_Y_LIQUID_VIABILITY "o_y_liquid_viability" = 97.0;
    /// New slurry viability, percent.
    O_Y_SLURRY_VIABILITY "o_y_slurry_viability" = 96.0;
    /// Dry yeast loss, percent per month.
    O_Y_DRY_AGING "o_y_dry_aging" = 1.66;
    /// Liquid yeast loss, percent per month.
    O_Y_LIQUID_AGING "o_y_liquid_aging" = 10.0;
    /// Slurry loss, percent per month.
    O_Y_SLURRY_AGING "o_y_slurry_aging" = 25.0;
    /// Ale pitch target, million cells per mL per degree Plato.
    O_Y_ALE_PITCH "o_y_ale_pitch" = 0.75;
    /// Lager pitch target.
    O_Y_LAGER_PITCH "o_y_lager_pitch" = 1.5;
    /// Hybrid pitch target.
    O_Y_MIX_PITCH "o_y_mix_pitch" = 1.0;
    /// Wine and mead pitch target.
    O_Y_WINE_PITCH "o_y_wine_pitch" = 0.5;
    /// Largest growth factor allowed in one starter.
    O_Y_GROWTH_STARTER "o_y_growth_starter" = 4.0;
    /// Stir plate growth bonus, percent.
    O_Y_STIR_PCT "o_y_stir_pct" = 45.0;
    /// Dry yeast cell density used by package-mode grams, billion cells per gram.
    O_Y_DRY_CELLS_PER_GRAM "o_y_dry_cells_per_gram" = 18.0;
    /// Rehydration water, mL per gram.
    O_Y_DRY_HYDRATE "o_y_dry_hydrate" = 25.0;
    /// `GoFerm` per mL of rehydration water, grams.
    O_Y_GOFERM "o_y_goferm" = 0.05;
    /// Starter tool pitch rate default.
    YC_PITCH_RATE "yc_pitch_rate" = 0.75;
    /// Starter wort gravity default.
    YC_STARTER_OG "yc_starter_og" = 1.036;
    /// Recommended dry pitch, grams per hectolitre.
    REC_DRY_RATE "rec_dry_rate" = 75.0;
    /// Dry yeast pack, grams (11.5 g pack count).
    DRY_PACK_G "dry_pack_g" = 11.5;
    /// Small dry yeast pack, grams.
    DRY_PACK5_G "dry_pack5_g" = 5.0;
    /// Weight of one dry package in recipe totals, grams.
    DRY_PKG_G "dry_pkg_g" = 11.0;
    /// Effective dry cell density of the volume/weight mode, billion cells per gram.
    DRY_CELLS_PER_G_EFF "dry_cells_per_g_eff" = 10.0;
    /// Mead tool default OG.
    MEAD_OG "mead_og" = 1.090;
    /// Mead tool default volume, fl oz.
    MEAD_VOLUME_FLOZ "mead_volume_floz" = 640.0;
    /// Mead tool default yeast, grams.
    MEAD_YEAST_G "mead_yeast_g" = 5.0;
    /// Mead tool default honey share, percent.
    MEAD_HONEY_PCT "mead_honey_pct" = 100.0;
    /// Wine tool default pH.
    WINE_PH "wine_ph" = 3.4;
    /// Wine tool default volume, fl oz.
    WINE_VOLUME_FLOZ "wine_volume_floz" = 640.0;
    /// Wine tool default measured free SO2, ppm.
    WINE_FREE_SO2 "wine_free_so2" = 0.0;
    /// Wine tool default target SO2, ppm.
    WINE_TARGET_SO2 "wine_target_so2" = 30.0;
    /// Carbonation tool default volumes of CO2.
    CARB_VOLS "carb_vols" = 2.3;
    /// Carbonation tool default temperature, degrees Fahrenheit.
    CARB_TEMP_F "carb_temp_f" = 68.0;
    /// Carbonation tool default volume, fl oz.
    CARB_VOLUME_FLOZ "carb_volume_floz" = 640.0;
    /// Backsweeten tool default volume, fl oz.
    BACK_VOLUME_FLOZ "back_volume_floz" = 640.0;
    /// Backsweeten tool default beverage SG.
    BACK_SG "back_sg" = 1.015;
    /// Backsweeten tool default target SG.
    BACK_TARGET_SG "back_target_sg" = 1.025;
}

// --- §1.3 age ---

/// Whole days between the package date and the reference day (spec §1.3).
///
/// All dates are day numbers. The reference is `brew_day` when the yeast is in the recipe,
/// otherwise `today`.
#[must_use]
pub fn age_days(pkg_day: i64, in_recipe: bool, brew_day: i64, today: i64) -> i64 {
    let reference = if in_recipe { brew_day } else { today };
    reference - pkg_day
}

/// Starter tool yeast age, months (`days / 30`); 0 for a future date or under one day.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn yeast_age_tool(today: i64, date_day: i64) -> f64 {
    let d = today - date_day;
    if d < 1 {
        return 0.0;
    }
    d as f64 / 30.0
}

/// Recipe "Avg Yeast Age", months: the SUM of the day counts over 30, not an average (spec §1.3).
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn yeast_age_recipe(age_days: &[i64]) -> f64 {
    age_days.iter().sum::<i64>() as f64 / 30.0
}

// --- §1.4 viability helpers ---

/// New-yeast viability `V0`, percent, for an ingredient form (Dry 98, Culture 96, else 97).
#[must_use]
pub fn viability_v0_ingredient(form: YeastForm) -> f64 {
    match form {
        YeastForm::Dry => O_Y_DRY_VIABILITY,
        YeastForm::Culture => O_Y_SLURRY_VIABILITY,
        _ => O_Y_LIQUID_VIABILITY,
    }
}

/// New-yeast viability `V0`, percent, for a starter tool form (Dry 98, Slurry 96, else 97).
#[must_use]
pub fn viability_v0_tool(form: StarterYeastForm) -> f64 {
    match form {
        StarterYeastForm::Dry => O_Y_DRY_VIABILITY,
        StarterYeastForm::Slurry => O_Y_SLURRY_VIABILITY,
        StarterYeastForm::Liquid => O_Y_LIQUID_VIABILITY,
    }
}

/// Monthly loss rate filled in for legacy data, percent (`YeastImportHook`, spec §1.4).
#[must_use]
pub fn import_age_rate(form: YeastForm, white_labs: bool) -> f64 {
    match form {
        YeastForm::Dry => 1.66,
        YeastForm::Culture => 25.0,
        YeastForm::Liquid if white_labs => 5.0,
        _ => 10.0,
    }
}

/// Inputs of [`tool_viability`].
#[derive(Debug, Clone, Copy)]
pub struct ToolViabilityInput {
    /// Calculate the viability (otherwise `user_viability` is used).
    pub calc: bool,
    /// Typed viability, %.
    pub user_viability_pct: f64,
    /// Initial viability, %.
    pub v0_pct: f64,
    /// Viability loss rate, percent per month (`F_YC_AGE_RATE`, passed to [`viability`] as is).
    pub rate_pct_per_month: f64,
    /// Days of storage.
    pub days: f64,
}

/// Starter tool viability: the typed value when calculation is off (spec §1.4).
#[must_use]
pub fn tool_viability(p: &ToolViabilityInput) -> f64 {
    let ToolViabilityInput {
        calc,
        user_viability_pct: user_v,
        v0_pct: v0,
        rate_pct_per_month: rate,
        days,
    } = *p;
    if calc {
        viability(v0, rate, days, 5.0)
    } else {
        user_v
    }
}

/// Viable cells, billions: `total · v / 100`.
#[must_use]
pub fn viable_cells(total_billion: f64, viability_pct: f64) -> f64 {
    total_billion * viability_pct / 100.0
}

// --- §1.5 total cells ---

/// Inputs of [`yeast_total_cells`].
#[derive(Debug, Clone, Copy)]
pub struct YeastTotalCellsInput {
    /// Package mode.
    pub use_pkgs: bool,
    /// Cells per package, billions.
    pub cells_pkg_billion: f64,
    /// Number of packages.
    pub new_amount: f64,
    /// Yeast form.
    pub form: YeastForm,
    /// Amount in volume/weight mode: grams for dry yeast, mL for liquid, slant and culture.
    pub amount_g_or_ml: f64,
    /// Cells per mL, billions.
    pub cells_ml_billion: f64,
}

/// Ingredient total cells, billions (spec §1.5).
///
/// Package mode: `cells_pkg · new_amount`. Otherwise dry: `grams · 10`; liquid, slant,
/// culture: `ml · cells_ml`; other forms 0.
#[must_use]
pub fn yeast_total_cells(p: &YeastTotalCellsInput) -> f64 {
    let YeastTotalCellsInput {
        use_pkgs,
        cells_pkg_billion: cells_pkg,
        new_amount,
        form,
        amount_g_or_ml: amount,
        cells_ml_billion: cells_ml,
    } = *p;
    if use_pkgs {
        return cells_pkg * new_amount;
    }
    match form {
        YeastForm::Dry => amount * 10.0,
        YeastForm::Liquid | YeastForm::Slant | YeastForm::Culture => amount * cells_ml,
    }
}

/// Starter tool available cells, billions (`YeastCellsAvail`).
#[must_use]
pub fn yeast_cells_avail(dry: bool, dry_g: f64, wet_ml: f64, cells_ml: f64) -> f64 {
    if dry { dry_g * 10.0 } else { wet_ml * cells_ml }
}

// --- §1.6 recipe totals ---

fn primary(yeasts: &[RecipeYeast]) -> impl Iterator<Item = &RecipeYeast> {
    yeasts.iter().filter(|y| !y.add_to_secondary)
}

/// `YeastCellsNoStarter`, billions: total or viable cells of the primary yeasts.
#[must_use]
pub fn yeast_cells_no_starter(calc_viability: bool, yeasts: &[RecipeYeast]) -> f64 {
    primary(yeasts)
        .map(|y| {
            if calc_viability {
                y.viable_cells_billion
            } else {
                y.total_cells_billion
            }
        })
        .sum()
}

/// `YeastPackCount`: sum of `new_amount` over the primary yeasts.
#[must_use]
pub fn yeast_pack_count(yeasts: &[RecipeYeast]) -> f64 {
    primary(yeasts).map(|y| y.packs).sum()
}

/// `YeastAvgPackSize`, billions per pack: 0 for an invalid list, 100 when cells ≤ 0 or no packs.
#[must_use]
pub fn yeast_avg_pack_size(cells_billion: f64, amount_sum: f64, valid: bool) -> f64 {
    if !valid {
        return 0.0;
    }
    if cells_billion <= 0.0 || amount_sum == 0.0 {
        return 100.0;
    }
    cells_billion / amount_sum
}

/// `YeastPackViability`: mean `viability_pct` of the primary yeasts, 98 without any.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn yeast_pack_viability(yeasts: &[RecipeYeast]) -> f64 {
    let n = primary(yeasts).count();
    if n == 0 {
        return 98.0;
    }
    primary(yeasts).map(|y| y.viability_pct).sum::<f64>() / n as f64
}

/// `YeastTotGrams`: grams of dry yeast; a package counts 11 g, volume mode converts ounces.
#[must_use]
pub fn yeast_tot_grams(yeasts: &[RecipeYeast]) -> f64 {
    primary(yeasts)
        .filter(|y| y.form == YeastForm::Dry)
        .map(|y| {
            if y.use_packages {
                y.packs * DRY_PKG_G
            } else {
                y.amount_oz * OZ_G
            }
        })
        .sum()
}

/// Fields of the first primary yeast, see [`yeast_first_fields`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirstYeastFields {
    /// `YeastForm`.
    pub form: YeastForm,
    /// `YeastUsePkgs`.
    pub use_packages: bool,
    /// `YeastCellsMl`, billions per mL.
    pub cells_per_ml_billion: f64,
    /// `YeastDryPitchRate`, g/hL.
    pub dry_pitch_rate_g_per_hl: f64,
}

/// Fields of the first primary yeast; liquid, packages, 1.0 and 75.0 without one.
#[must_use]
pub fn yeast_first_fields(yeasts: &[RecipeYeast]) -> FirstYeastFields {
    primary(yeasts).next().map_or(
        FirstYeastFields {
            form: YeastForm::Liquid,
            use_packages: true,
            cells_per_ml_billion: 1.0,
            dry_pitch_rate_g_per_hl: 75.0,
        },
        |y| FirstYeastFields {
            form: y.form,
            use_packages: y.use_packages,
            cells_per_ml_billion: y.cells_per_ml_billion,
            dry_pitch_rate_g_per_hl: y.dry_pitch_rate_g_per_hl,
        },
    )
}

// --- §1.7 pitch targets ---

/// Pitch targets in effect, million cells per mL per degree Plato (spec §1.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchTargets {
    /// Ale.
    pub ale_m_ml_p: f64,
    /// Lager.
    pub lager_m_ml_p: f64,
    /// Hybrid.
    pub mix_m_ml_p: f64,
    /// Wine and mead.
    pub wine_m_ml_p: f64,
}

impl Default for PitchTargets {
    fn default() -> Self {
        Self {
            ale_m_ml_p: O_Y_ALE_PITCH,
            lager_m_ml_p: O_Y_LAGER_PITCH,
            mix_m_ml_p: O_Y_MIX_PITCH,
            wine_m_ml_p: O_Y_WINE_PITCH,
        }
    }
}

/// Recipe `RecommendedLiquidPitch`: wine target for types 4..=7, else by ale type, else 0.75.
#[must_use]
pub fn recommended_liquid_pitch(r_type: i64, ale_type: i64, t: &PitchTargets) -> f64 {
    if is_wine_like_recipe(r_type) {
        return t.wine_m_ml_p;
    }
    match ale_type {
        0 => t.ale_m_ml_p,
        1 => t.lager_m_ml_p,
        2 => t.mix_m_ml_p,
        _ => 0.75,
    }
}

/// Pitch range of [`pitch_min_max_recipe`] and [`pitch_min_max_tool`], million cells per mL per
/// degree Plato.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchRange {
    /// Lower end.
    pub min_m_ml_p: f64,
    /// Upper end.
    pub max_m_ml_p: f64,
}

/// Recipe pitch range `(rec − 0.4, rec + 0.4)`.
#[must_use]
pub fn pitch_min_max_recipe(rec_m_ml_p: f64) -> PitchRange {
    PitchRange {
        min_m_ml_p: rec_m_ml_p - 0.4,
        max_m_ml_p: rec_m_ml_p + 0.4,
    }
}

/// Starter tool pitch range by `F_YC_TYPE`.
#[must_use]
pub fn pitch_min_max_tool(tool_type: StarterType, t: &PitchTargets) -> PitchRange {
    let (center, delta) = match tool_type {
        StarterType::AleOrCider => (t.ale_m_ml_p, 0.30),
        StarterType::Lager => (t.lager_m_ml_p, 0.25),
        StarterType::HybridWheat => (t.mix_m_ml_p, 0.40),
        StarterType::MeadOrWine => (t.wine_m_ml_p, 0.25),
    };
    PitchRange {
        min_m_ml_p: center - delta,
        max_m_ml_p: center + delta,
    }
}

/// Minimum dry pitch, g/hL: `max(5, floor(rec_dry / 4))`.
#[must_use]
pub fn dry_min_pitch_rate(rec_dry_g_per_hl: f64) -> f64 {
    (rec_dry_g_per_hl / 4.0).floor().max(5.0)
}

/// Maximum dry pitch, g/hL: `2 · rec_dry`.
#[must_use]
pub fn dry_max_pitch_rate(rec_dry_g_per_hl: f64) -> f64 {
    2.0 * rec_dry_g_per_hl
}

/// Whether the dry yeast is pitched by weight: recipe `form == 1 && use_pkgs != 1`, tool `form == 1`.
#[must_use]
pub fn is_dry_by_weight(form: YeastForm, use_pkgs: bool, tool: bool) -> bool {
    if tool {
        form == YeastForm::Dry
    } else {
        form == YeastForm::Dry && !use_pkgs
    }
}

// --- §1.8 cells needed ---

/// Inputs of [`tot_yeast_cells_needed`].
#[derive(Debug, Clone, Copy)]
pub struct CellsNeededInput {
    /// Dry yeast is pitched by weight.
    pub dry_by_weight: bool,
    /// Pitch rate, million cells per mL per °P.
    pub pitch_m_ml_p: f64,
    /// V l.
    pub v_l: f64,
    /// Gravity, SG.
    pub sg: f64,
    /// Dry pitch rate, g/hL.
    pub dry_rate_g_per_hl: f64,
    /// Recommended dry pitch rate, g/hL.
    pub rec_dry_rate_g_per_hl: f64,
}

/// `TotYeastCellsNeeded`, billions. Dry by weight: `V · 7.5 · (dry_rate / rec_dry_rate)` with
/// both rates floored at 0.01; otherwise [`cells_needed`].
#[must_use]
pub fn tot_yeast_cells_needed(p: &CellsNeededInput) -> f64 {
    let CellsNeededInput {
        dry_by_weight,
        pitch_m_ml_p: pitch,
        v_l,
        sg,
        dry_rate_g_per_hl: dry_rate,
        rec_dry_rate_g_per_hl: rec_dry_rate,
    } = *p;
    if dry_by_weight {
        v_l * 7.5 * (dry_rate.max(0.01) / rec_dry_rate.max(0.01))
    } else {
        cells_needed(pitch, v_l, sg)
    }
}

/// `EstCellsML`, million cells per mL: `n · 1000 / max(v_ml, 0.1)`.
#[must_use]
pub fn est_cells_ml(n_billion: f64, v_ml: f64) -> f64 {
    n_billion * 1000.0 / v_ml.max(0.1)
}

// --- §1.9 cells after a starter ---

/// `YeastCellsStarter`: `viable · GrowthRate(viable / starter_l)`; no starter means no growth.
/// Errors on non-finite input.
pub fn cells_starter(viable_billion: f64, starter_l: f64, stir: bool) -> Result<f64> {
    ensure_finite("viable_billion", viable_billion)?;
    ensure_finite("starter_l", starter_l)?;
    let i = if starter_l > 0.0 {
        viable_billion / starter_l
    } else {
        0.0
    };
    if i <= 0.0 {
        return Ok(viable_billion);
    }
    Ok(viable_billion * growth_rate(i, stir)?)
}

/// `YeastCellsSecStarter`: the second stage starts from the first stage's cells.
pub fn cells_sec_starter(cells1_billion: f64, sec_l: f64, stir: bool) -> Result<f64> {
    cells_starter(cells1_billion, sec_l, stir)
}

// --- §1.11 recipe two-stage ---

/// Recipe two-stage starter volumes (litres): needs `avail < need`, growth per
/// stage capped at `growth_cap` (spec §1.11).
/// Errors on non-finite input.
pub fn two_stage_recipe(
    need_billion: f64,
    avail_billion: f64,
    stir: bool,
    growth_cap: f64,
) -> Result<StarterStages> {
    ensure_finite("need_billion", need_billion)?;
    ensure_finite("avail_billion", avail_billion)?;
    ensure_finite("growth_cap", growth_cap)?;
    let avail_billion = avail_billion.max(0.1);
    if avail_billion >= need_billion {
        return Ok(StarterStages::NONE);
    }
    let mut r = need_billion / avail_billion;
    let mut s = 1.0;
    if stir {
        s = 1.0 + STIR / 100.0;
        r /= s * s;
    }
    let g = r.sqrt().min(growth_cap);
    let v1 = avail_billion / innoc_rate(g)?;
    let v2 = avail_billion * g * s / innoc_rate(g)?;
    Ok(StarterStages {
        stage1_l: v1,
        stage2_l: v2,
    })
}

// --- §1.12 packs ---

/// `YeastPacksNoStarter`: `1` below one pack, else `int(n / pkt + 0.99)`; `pkt ≤ 0` counts as 1.
/// Errors on non-finite input.
#[allow(clippy::cast_possible_truncation)]
pub fn packs_no_starter(n_billion: f64, pkt_billion: f64) -> Result<i64> {
    ensure_finite("n_billion", n_billion)?;
    ensure_finite("pkt_billion", pkt_billion)?;
    let pkt_billion = if pkt_billion <= 0.0 { 1.0 } else { pkt_billion };
    Ok(if n_billion < pkt_billion {
        1
    } else {
        (n_billion / pkt_billion + 0.99).trunc() as i64
    })
}

/// `YeastPacksStarter` / `CalcYeastPacksNeeded` with a starter: `int(packs) + 0.1`.
///
/// `growth ≤ 0` counts as 1.2; a stir plate divides it by 1.45 (needs more packs).
/// Errors on non-finite input.
pub fn packs_starter(n_billion: f64, pkt_billion: f64, growth: f64, stir: bool) -> Result<f64> {
    ensure_finite("n_billion", n_billion)?;
    ensure_finite("pkt_billion", pkt_billion)?;
    ensure_finite("growth", growth)?;
    let pkt_billion = if pkt_billion <= 0.0 { 1.0 } else { pkt_billion };
    let mut g = if growth > 0.0 { growth } else { 1.2 };
    if stir {
        g /= 1.0 + STIR / 100.0;
    }
    let p = if n_billion < pkt_billion {
        1.0
    } else {
        (n_billion / (g * pkt_billion) + 0.99).trunc()
    };
    Ok(p + 0.1)
}

/// Legacy `CalcYeastStarterSize`, litres: `pkt · pkt_size / InnocRate(r)` for `r > 1`.
/// Errors on non-finite input.
pub fn calc_yeast_starter_size(r: f64, pkt_size_billion: f64, pkt_count: f64) -> Result<f64> {
    ensure_finite("r", r)?;
    ensure_finite("pkt_size_billion", pkt_size_billion)?;
    ensure_finite("pkt_count", pkt_count)?;
    if r > 1.0 {
        Ok(pkt_count * pkt_size_billion / innoc_rate(r)?)
    } else {
        Ok(0.0)
    }
}

// --- §1.13 dry yeast ---

/// Dry yeast grams from a rate in g/hL: `rate · V(L) / 100`.
#[must_use]
pub fn dry_grams_vol(dry_rate_g_per_hl: f64, v_l: f64) -> f64 {
    dry_rate_g_per_hl * v_l / 100.0
}

/// Dry yeast grams in package mode: `n / cells_per_g` (≤ 0 counts as 0.1).
#[must_use]
pub fn dry_grams_pkg(n_billion: f64, cells_per_g: f64) -> f64 {
    n_billion / if cells_per_g > 0.0 { cells_per_g } else { 0.1 }
}

/// Number of 11.5 g packs (`YeastPacksDry`, no rounding).
#[must_use]
pub fn yeast_packs_dry(g: f64) -> f64 {
    g / DRY_PACK_G
}

/// Number of 5 g packs (`YeastDry5`).
#[must_use]
pub fn yeast_dry5(g: f64) -> f64 {
    g / DRY_PACK5_G
}

/// Rehydration water, mL (`YeastDryHydrate`): `grams · per_g`.
#[must_use]
pub fn yeast_dry_hydrate(tot_g: f64, per_g: f64) -> f64 {
    tot_g * per_g
}

/// `GoFerm` grams (`YeastGoferm`): `grams · per_g · goferm` (1.25 g per g of yeast by default).
#[must_use]
pub fn yeast_goferm(tot_g: f64, per_g: f64, goferm: f64) -> f64 {
    tot_g * per_g * goferm
}

// --- §1.15 other ---

/// Unit shown for a yeast amount: `Pkgs` in package mode, else the dry or wet unit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YeastUnit {
    /// Package count (`pkgs`, code 0x29).
    Pkgs,
    /// Unit code (default dry 10 = g, wet 15 = mL).
    Code(i64),
}

/// `YeastAmountUnitsFunc` / `YeastUnitsFunc`.
#[must_use]
pub fn yeast_units(use_pkgs: bool, form: YeastForm, dry_unit: i64, wet_unit: i64) -> YeastUnit {
    if use_pkgs {
        YeastUnit::Pkgs
    } else if form == YeastForm::Dry {
        YeastUnit::Code(dry_unit)
    } else {
        YeastUnit::Code(wet_unit)
    }
}

/// `YeastImportHook` tolerance fix: tolerance 10 becomes 15 for Wine and 18 for Champagne.
#[must_use]
pub fn import_hook_tolerance(tol_abv_pct: i64, y_type: YeastType) -> i64 {
    match (tol_abv_pct, y_type) {
        (10, YeastType::Wine) => 15,
        (10, YeastType::Champagne) => 18,
        _ => tol_abv_pct,
    }
}

// --- §2.2, §2.3 mead yeast ---

/// `MeadYeastGrams`: `V(L) · mead_pitch(sg)`.
#[must_use]
pub fn mead_yeast_grams(sg: f64, v_l: f64) -> f64 {
    v_l * mead_pitch(sg)
}

/// `MeadGoferm`, grams: `yeast_g · 25 · 0.05`.
#[must_use]
pub fn mead_goferm(yeast_g: f64) -> f64 {
    yeast_g * O_Y_DRY_HYDRATE * O_Y_GOFERM
}

/// `MeadHydrate`, mL of water: `yeast_g · 25`.
#[must_use]
pub fn mead_hydrate(yeast_g: f64) -> f64 {
    yeast_g * O_Y_DRY_HYDRATE
}

/// `MeadHydrate1`: same value as [`mead_hydrate`] (the specification gives one formula).
#[must_use]
pub fn mead_hydrate1(yeast_g: f64) -> f64 {
    mead_hydrate(yeast_g)
}

// --- §2.4 nutrient programme ---

/// Target YAN, ppm: `°P(OG) · 10 · k · (pct / 100)`. Errors on non-finite input.
pub fn mead_yan_target(sg: f64, yan_pct: f64, fac: YeastNitrogen) -> Result<f64> {
    ensure_finite("sg", sg)?;
    ensure_finite("yan_pct", yan_pct)?;
    Ok(to_plato(sg) * 10.0 * fac.factor() * (yan_pct / 100.0))
}

/// Inputs of [`mead_tot_ferm_o`], [`mead_tot_ferm_k`], [`mead_ferm_o`] and [`mead_ferm_k`].
#[derive(Debug, Clone, Copy)]
pub struct MeadFermInput {
    /// Gravity, SG.
    pub sg: f64,
    /// Final volume, gal.
    pub v_gal: f64,
    /// Acid malt acid content, %.
    pub yan_pct: f64,
    /// Yeast nitrogen factor.
    pub fac: YeastNitrogen,
    /// Nutrient method.
    pub method: MeadMethod,
}

/// `MeadTotFermO`, grams: Fermaid-O total, only for method 0 (else 0).
pub fn mead_tot_ferm_o(p: &MeadFermInput) -> Result<f64> {
    let MeadFermInput {
        sg,
        v_gal,
        yan_pct: pct,
        fac,
        method,
    } = *p;
    ensure_finite("v_gal", v_gal)?;
    if method == MeadMethod::Tosna {
        Ok(mead_yan_target(sg, pct, fac)? * v_gal / 50.0)
    } else {
        mead_yan_target(sg, pct, fac).map(|_| 0.0)
    }
}

/// `MeadTotFermK`, grams: Fermaid-K total, only for method 1 (else 0).
pub fn mead_tot_ferm_k(p: &MeadFermInput) -> Result<f64> {
    let MeadFermInput {
        sg,
        v_gal,
        yan_pct: pct,
        fac,
        method,
    } = *p;
    ensure_finite("v_gal", v_gal)?;
    if method == MeadMethod::Tiosna {
        Ok(mead_yan_target(sg, pct, fac)? * v_gal / 100.0)
    } else {
        mead_yan_target(sg, pct, fac).map(|_| 0.0)
    }
}

/// `MeadFermO`, grams: one of the four Fermaid-O additions.
pub fn mead_ferm_o(p: &MeadFermInput) -> Result<f64> {
    let MeadFermInput {
        sg,
        v_gal,
        yan_pct: pct,
        fac,
        method,
    } = *p;
    Ok(mead_tot_ferm_o(&MeadFermInput {
        sg,
        v_gal,
        yan_pct: pct,
        fac,
        method,
    })? / 4.0)
}

/// `MeadFermK`, grams: one of the four Fermaid-K additions.
pub fn mead_ferm_k(p: &MeadFermInput) -> Result<f64> {
    let MeadFermInput {
        sg,
        v_gal,
        yan_pct: pct,
        fac,
        method,
    } = *p;
    Ok(mead_tot_ferm_k(&MeadFermInput {
        sg,
        v_gal,
        yan_pct: pct,
        fac,
        method,
    })? / 4.0)
}

/// A fermentable for [`mead_pct_honey`]: `F_G_TYPE`, subtype and OG points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeadFerm {
    /// `F_G_TYPE` (7 = honey family).
    pub kind: i64,
    /// Subtype (honey when < 7).
    pub subtype: i64,
    /// OG points contributed.
    pub pts: f64,
}

/// `MeadPctHoney`: honey points over primary-phase OG points, percent; 0 when `primary_pts` is 0.
#[must_use]
pub fn mead_pct_honey(ferms: &[MeadFerm], primary_pts: f64) -> f64 {
    if primary_pts == 0.0 {
        return 0.0;
    }
    let honey: f64 = ferms
        .iter()
        .filter(|f| f.kind == 7 && f.subtype < 7)
        .map(|f| f.pts)
        .sum();
    honey / primary_pts * 100.0
}

// --- §2.5 YAN ---

/// `YANEquivalent`: organic nitrogen counts 4x for Fermaid-O (method 0).
#[must_use]
pub fn yan_equivalent(method: YanMethod, y_ppm: f64) -> f64 {
    if method == YanMethod::FermaidO {
        y_ppm * 4.0
    } else {
        y_ppm
    }
}

/// `MeadYANAdded`, ppm. `total_g` is the nutrient total. The recipe version reads one
/// addition (`total / 4`), a quirk of the reference (spec §2.5).
#[must_use]
pub fn mead_yan_added(method: YanMethod, total_g: f64, v_l: f64, recipe: bool) -> f64 {
    yan(method, if recipe { total_g / 4.0 } else { total_g }, v_l)
}

/// `MeadYANEquivalent`, ppm.
#[must_use]
pub fn mead_yan_equivalent(method: YanMethod, total_g: f64, v_l: f64, recipe: bool) -> f64 {
    yan_equivalent(method, mead_yan_added(method, total_g, v_l, recipe))
}

/// `NutrientNeeded`, grams: `ppm · V(L) / c`.
#[must_use]
pub fn nutrient_needed(ppm: f64, method: YanMethod, v_l: f64) -> f64 {
    ppm * v_l / method.ppm_per_g_l()
}

// --- §2.8 backsweetening ---

/// Default potential of a backsweetening agent, degrees Brix.
#[must_use]
pub fn back_agent_og(agent: BackAgent) -> f64 {
    agent.potential_brix()
}

/// `BackAgentVol`, litres of agent for `v_l` of beverage at `sg_bev` to reach `sg_target`.
#[must_use]
pub fn back_agent_vol(v_l: f64, sg_bev: f64, sg_target: f64, agent: BackAgent) -> f64 {
    backsweeten_volume_l(v_l, sg_bev, sg_target, to_sg(back_agent_og(agent)))
}

/// `BackAgentWeight`, kg: `BrixToSG(potential) · volume` (the 258.2 Brix variant).
#[must_use]
pub fn back_agent_weight(v_l: f64, sg_bev: f64, sg_target: f64, agent: BackAgent) -> f64 {
    brix_to_sg(back_agent_og(agent)) * back_agent_vol(v_l, sg_bev, sg_target, agent)
}

// --- §3.3, §3.4 carbonation amount ---

/// Priming sugar agent: code 0 corn sugar, 1 DME (x1.538), 2 table sugar (corn / 1.099).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimingAgent {
    /// Code 0.
    CornSugar,
    /// Code 1.
    Dme,
    /// Code 2.
    TableSugar,
}

impl PrimingAgent {
    /// From the code (0-2); other codes are an error (T1.16, was corn sugar).
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::CornSugar),
            1 => Ok(Self::Dme),
            2 => Ok(Self::TableSugar),
            _ => Err(Error::invalid("priming_agent", "unknown code")),
        }
    }
}

/// Priming sugar grams by agent (corn sugar baseline).
#[must_use]
pub fn carb_sugar_agent(agent: PrimingAgent, co2_vol: f64, temp_f: f64, volume_gal: f64) -> f64 {
    let g = sugar_g(co2_vol, temp_f, volume_gal, false);
    match agent {
        PrimingAgent::Dme => g * 1.538,
        PrimingAgent::TableSugar => g / 1.099,
        PrimingAgent::CornSugar => g,
    }
}

/// Numeric part of `CarbAmountText`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarbAmount {
    /// Keg: pressure, psi.
    PressurePsi(f64),
    /// Bottle or keg with priming agent: sugar, grams.
    SugarG(f64),
}

/// Inputs of [`carb_amount`].
#[derive(Debug, Clone, Copy)]
pub struct CarbAmountInput {
    /// Carbonation type.
    pub c_type: CarbType,
    /// Target carbonation, volumes of CO2.
    pub co2_vol: f64,
    /// Beer temperature, °F.
    pub temp_f: f64,
    /// Volume, US gal.
    pub volume_gal: f64,
    /// Carbonation agent rate, %.
    pub carb_rate_pct: f64,
    /// Priming agent.
    pub agent: PrimingAgent,
}

/// `CarbAmountText` number: keg pressure for type 1, else priming sugar divided by
/// `carb_rate / 100` when `carb_rate ≠ 0`. No keg factor (spec §3.3).
#[must_use]
pub fn carb_amount(p: &CarbAmountInput) -> CarbAmount {
    let CarbAmountInput {
        c_type,
        co2_vol: vols,
        temp_f,
        volume_gal: gal,
        carb_rate_pct: carb_rate,
        agent,
    } = *p;
    if c_type == CarbType::Keg {
        return CarbAmount::PressurePsi(co2_psi(vols, temp_f));
    }
    let mut g = carb_sugar_agent(agent, vols, temp_f, gal);
    if carb_rate != 0.0 {
        g /= carb_rate / 100.0;
    }
    CarbAmount::SugarG(g)
}

/// `CarbEquivFunc`: `100 / max(rate, 1)`.
#[must_use]
pub fn carb_equiv(rate_pct: f64) -> f64 {
    100.0 / rate_pct.max(1.0)
}

// --- §3.6, §3.7 ---

/// `CarbCalories`: total minus alcohol calories.
#[must_use]
pub fn carb_calories(total_kcal: f64, alc_kcal: f64) -> f64 {
    total_kcal - alc_kcal
}

/// `CarbGrams`: carbohydrate calories over 4 kcal/g.
#[must_use]
pub fn carb_grams(carb_kcal: f64) -> f64 {
    carb_kcal / 4.0
}

/// BeerSmith volumes to g/L factor (spec §3.7). The engine itself uses 1.9635.
pub const BS_VOL_TO_GL: f64 = 1.9642;
/// BeerSmith g/L to volumes divisor (spec §3.7), not the inverse of [`BS_VOL_TO_GL`].
pub const BS_GL_TO_VOL: f64 = 1.96042;

/// `BConvertCarbonation`: volumes to g/L when `to_gl`, else g/L to volumes (BeerSmith constants).
#[must_use]
pub fn convert_carbonation(v: f64, to_gl: bool) -> f64 {
    if to_gl {
        v * BS_VOL_TO_GL
    } else {
        v / BS_GL_TO_VOL
    }
}
