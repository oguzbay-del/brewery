//! Section 04: yeast, mead, carbonation (`formuller/04-maya-mead-karbonasyon.md`).
//!
//! Units follow the BeerSmith internal units of the specification: gallons, fl oz,
//! degrees Fahrenheit, litres where stated. Every function documents its units.
//! Ported one to one from `formuller/testler/04_maya_karb_tv.py` (ADR-0003).

use crate::error::{Error, Result, ensure_finite, ensure_positive, finite_result};
use crate::units::{GAL_L, to_plato, to_sg};

/// Stir plate growth bonus, percent (spec §1.2).
const STIR: f64 = 45.0;
/// Millilitres per US fluid ounce.
const FLOZ_ML: f64 = 29.573_529_56;
/// Grams per ounce (`0.4535924 / 16 × 1000`).
pub const OZ_G: f64 = 0.453_592_4 / 16.0 * 1000.0;

/// Litres to US fluid ounces.
#[must_use]
pub fn l_to_floz(litres: f64) -> f64 {
    litres * 1000.0 / FLOZ_ML
}

/// US fluid ounces to litres.
#[must_use]
pub fn floz_to_l(floz: f64) -> f64 {
    floz * FLOZ_ML / 1000.0
}

/// Litres to US gallons.
#[must_use]
pub fn gal_from_l(litres: f64) -> f64 {
    litres * 1000.0 / FLOZ_ML / 128.0
}

/// Brix to SG, keeping the BeerSmith quirk of 258.2 in the correction term (spec §2.8).
#[must_use]
pub fn brix_to_sg(brix: f64) -> f64 {
    brix / (258.6 - (brix / 258.2) * 227.1) + 1.0
}

/// Cell viability, percent, after `days` (spec §1.4).
///
/// `v0` percent, `rate` monthly loss percent, floored at `floor` (default 5). Zero or
/// negative `days` return `v0` unchanged.
#[must_use]
pub fn viability(v0: f64, rate: f64, days: f64, floor: f64) -> f64 {
    if days <= 0.0 {
        return v0;
    }
    let v = v0 * ((100.0 - rate) / 100.0).powf(days / 30.0);
    v.max(floor)
}

/// Inoculation rate, million cells per mL, to growth factor `g` (spec §1.9).
/// Errors when `g` is not a finite positive number (the reference's `log` raises).
pub fn innoc_rate(g: f64) -> Result<f64> {
    ensure_positive("g", g)?;
    finite_result("innoc_rate", (g.ln() * -2.179_103_8 + 5.512_221).exp())
}

/// Growth factor for inoculation rate `i` (million cells/mL), clamped to 1..6 (spec §1.9).
///
/// Uses the exponent -2.1781038 (differs from [`innoc_rate`] in the reference).
/// Errors when `i` is not a finite positive number (the reference's `log` raises).
pub fn growth_rate(i: f64, stir: bool) -> Result<f64> {
    ensure_positive("i", i)?;
    let mut g = ((i.ln() - 5.512_221) / -2.178_103_8).exp();
    if stir {
        g *= 1.0 + STIR / 100.0;
    }
    Ok(g.clamp(1.0, 6.0))
}

/// Cells needed, billions: `rate (M/mL/°P) × volume (L) × °Plato(sg)` (spec §1.8).
#[must_use]
pub fn cells_needed(rate: f64, volume_l: f64, sg: f64) -> f64 {
    rate * volume_l * to_plato(sg)
}

/// Single-stage starter volume, litres, for `need` and `avail` billion cells (spec §1.10).
/// Errors on non-finite input and when cells are needed but `avail <= 0` (zero division).
pub fn new_starter_l(need: f64, avail: f64, stir: bool) -> Result<f64> {
    ensure_finite("need", need)?;
    ensure_finite("avail", avail)?;
    if need <= avail {
        return Ok(0.0);
    }
    ensure_positive("avail", avail)?;
    let mut r = need / avail;
    if stir {
        r /= 1.0 + STIR / 100.0;
    }
    if r <= 1.0 {
        return Ok(0.0);
    }
    Ok(avail / innoc_rate(r)?)
}

/// Two-stage starter volumes `(stage1_l, stage2_l)`, litres (spec §1.11).
/// Errors on non-finite input.
pub fn two_stage_tool(need: f64, avail: f64, stir: bool) -> Result<(f64, f64)> {
    ensure_finite("need", need)?;
    ensure_finite("avail", avail)?;
    let avail = avail.max(0.1);
    if avail > need {
        return Ok((0.0, 0.0));
    }
    let mut r = need / avail;
    let mut s = 1.0;
    if stir {
        s = 1.0 + STIR / 100.0;
        r /= s * s;
    }
    let g = r.sqrt();
    let v1 = avail / innoc_rate(g)?;
    let v2 = avail * g * s / innoc_rate(g)?;
    Ok((v1, v2))
}

/// DME ounces for a starter of `vol_floz` fl oz at gravity `og` (spec §1.14).
#[must_use]
pub fn dme_oz(vol_floz: f64, og: f64) -> f64 {
    (og - 1.0) * 1000.0 * vol_floz / 128.0 / 45.0 * 16.0
}

/// Priming sugar, grams, for `vols` volumes CO2 at `temp_f` °F in `gal` gallons (spec §3.3).
///
/// Corn sugar by default; `dme` multiplies by 1.538.
#[must_use]
pub fn sugar_g(vols: f64, temp_f: f64, gal: f64, dme: bool) -> f64 {
    let t = temp_f.max(32.0);
    let g = ((t * 0.050_062 + (vols - 3.0378)) - t * 0.000_265_55 * t) * gal * 15.195;
    if dme { g * 1.538 } else { g }
}

/// Residual dissolved CO2, volumes, at `temp_f` °F (spec §3.2). Used by section 11.
#[must_use]
pub fn residual_co2(temp_f: f64) -> f64 {
    let t = temp_f.max(32.0);
    3.0378 - 0.050_062 * t + 0.000_265_55 * t * t
}

/// Keg pressure, psi, to hold `vols` volumes CO2 at `temp_f` °F (spec §3.5). Used by section 11.
#[must_use]
pub fn co2_psi(vols: f64, temp_f: f64) -> f64 {
    let t = temp_f.max(32.0);
    let p =
        vols * 4.24267 + (-16.6999 - t * 0.010_105_9) + t * 0.001_165_12 * t + t * 0.173_354 * vols
            - vols * 0.068_422_6 * vols;
    p.max(0.0)
}

/// Recommended free SO2, ppm, for wine `ph` (spec §2.6).
#[must_use]
pub fn so2_rec(ph: f64) -> f64 {
    (ph * 2.21828 - 4.27809).exp()
}

/// Potassium metabisulfite, grams, for `v_gal` gallons to raise free SO2 from `free` to
/// `target` ppm (spec §2.6).
#[must_use]
pub fn kmeta_g(v_gal: f64, target: f64, free: f64) -> f64 {
    v_gal * 3.785 * (target - free) / 576.0
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

/// Re-exported for callers converting gallons: litres per US gallon.
pub const LITRES_PER_GAL: f64 = GAL_L;

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
pub fn honey_sg(plato: f64) -> f64 {
    to_sg(plato)
}

// ---------------------------------------------------------------------------
// T1.14f: calculations that the specification defines and the Python reference
// gained in this task (enums, defaults, yeast counts and packs, mead, carbonation).
// ---------------------------------------------------------------------------

macro_rules! code_enum {
    ($(#[$m:meta])* $name:ident { $($var:ident = $code:literal => $label:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($var),+ }

        impl $name {
            /// Every variant in code order.
            pub const ALL: &'static [Self] = &[$(Self::$var),+];

            /// Variant for a stored code; a code that is not listed is an error.
            pub fn from_code(code: i64) -> Result<Self> {
                match code {
                    $($code => Ok(Self::$var),)+
                    _ => Err(Error::invalid(stringify!($name), "unknown code")),
                }
            }

            /// Stored code.
            #[must_use]
            pub fn code(self) -> i64 {
                match self { $(Self::$var => $code),+ }
            }

            /// English display name of the specification.
            #[must_use]
            pub fn name(self) -> &'static str {
                match self { $(Self::$var => $label),+ }
            }
        }
    };
}

code_enum! {
    /// `F_Y_FORM` of an ingredient (spec §1.1). Culture behaves like slurry for viability.
    YeastForm { Liquid = 0 => "Liquid", Dry = 1 => "Dry", Slant = 2 => "Slant",
                Culture = 3 => "Culture" }
}
code_enum! {
    /// `F_YC_YEAST_FORM` of the starter tool (spec §1.1).
    StarterYeastForm { Liquid = 0 => "Liquid", Dry = 1 => "Dry", Slurry = 2 => "Slurry" }
}
code_enum! {
    /// `F_YC_TYPE` of the starter tool (spec §1.1, §1.7).
    StarterType { AleOrCider = 0 => "Ale or Cider", Lager = 1 => "Lager",
                  HybridWheat = 2 => "Hybrid/Wheat", MeadOrWine = 3 => "Mead or Wine" }
}
code_enum! {
    /// `F_Y_TYPE` (spec §1.1).
    YeastType { Ale = 0 => "Ale", Lager = 1 => "Lager", Wine = 2 => "Wine",
                Champagne = 3 => "Champagne", Wheat = 4 => "Wheat" }
}
code_enum! {
    /// `F_R_TYPE` (spec §1.1). Code 7 is accepted as a wine type by the reference code but
    /// is not listed; see [`is_wine_like_recipe`].
    RecipeType { Extract = 0 => "Extract", PartialMash = 1 => "Partial Mash",
                 AllGrain = 2 => "All Grain", Cider = 3 => "Cider", Mead = 4 => "Mead",
                 Wine = 5 => "Wine", Seltzer = 6 => "Seltzer" }
}
code_enum! {
    /// `F_R_ALE_TYPE` (spec §1.1).
    AleType { Ale = 0 => "Ale", Lager = 1 => "Lager", Hybrid = 2 => "Hybrid" }
}
code_enum! {
    /// `F_MT_METHOD` / `F_R_MEAD_METHOD` (spec §2.1). Code 2 (DAP) only appears in [`yan`].
    MeadMethod { Tosna = 0 => "TOSNA 2/3 (Fermaid-O)", Tiosna = 1 => "TiOSNA (Fermaid-K)" }
}
code_enum! {
    /// `F_MT_YEAST_FACTOR` / `F_R_YEAST_FACTOR` (spec §2.1).
    YeastNitrogen { Low = 0 => "Low N", Medium = 1 => "Medium N", High = 2 => "High N" }
}
code_enum! {
    /// `F_C_TYPE` / `F_R_CARB_TYPE` (spec §3.1).
    CarbType { Bottle = 0 => "Bottle", Keg = 1 => "Keg", KegPriming = 2 => "Keg with priming agent" }
}
code_enum! {
    /// Backsweetening agent (spec §2.8).
    BackAgent { Honey = 0 => "Honey", AppleJuiceConcentrate = 1 => "Apple Juice Concentrate",
                Juice = 2 => "Juice", DryMaltExtract = 3 => "Dry Malt Extract",
                TableSugar = 4 => "Table Sugar", Other = 5 => "Other" }
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

/// Display name of an enum code by kind (`y_form`, `yc_yeast_form`, `yc_type`, `y_type`,
/// `r_type`, `r_ale_type`, `mead_method`, `yeast_factor`, `carb_type`, `back_agent`).
#[must_use]
pub fn enum_name(kind: &str, code: i64) -> Option<&'static str> {
    match kind {
        "y_form" => YeastForm::from_code(code).ok().map(YeastForm::name),
        "yc_yeast_form" => StarterYeastForm::from_code(code)
            .ok()
            .map(StarterYeastForm::name),
        "yc_type" => StarterType::from_code(code).ok().map(StarterType::name),
        "y_type" => YeastType::from_code(code).ok().map(YeastType::name),
        "r_type" => RecipeType::from_code(code).ok().map(RecipeType::name),
        "r_ale_type" => AleType::from_code(code).ok().map(AleType::name),
        "mead_method" => MeadMethod::from_code(code).ok().map(MeadMethod::name),
        "yeast_factor" => YeastNitrogen::from_code(code).ok().map(YeastNitrogen::name),
        "carb_type" => CarbType::from_code(code).ok().map(CarbType::name),
        "back_agent" => BackAgent::from_code(code).ok().map(BackAgent::name),
        _ => None,
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

/// Starter tool viability: the typed value when calculation is off (spec §1.4).
#[must_use]
pub fn tool_viability(calc: bool, user_v: f64, v0: f64, rate: f64, days: f64) -> f64 {
    if calc {
        viability(v0, rate, days, 5.0)
    } else {
        user_v
    }
}

/// Viable cells, billions: `total · v / 100`.
#[must_use]
pub fn viable_cells(total: f64, v: f64) -> f64 {
    total * v / 100.0
}

// --- §1.5 total cells ---

/// Ingredient total cells, billions (spec §1.5).
///
/// Package mode: `cells_pkg · new_amount`. Otherwise dry: `grams · 10`; liquid, slant,
/// culture: `ml · cells_ml`; other forms 0.
#[must_use]
pub fn yeast_total_cells(
    use_pkgs: bool,
    cells_pkg: f64,
    new_amount: f64,
    form: YeastForm,
    amount: f64,
    cells_ml: f64,
) -> f64 {
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

/// One yeast ingredient of a recipe, the fields the recipe totals read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecipeYeast {
    /// Added to secondary fermentation (skipped by every total).
    pub secondary: bool,
    /// `F_Y_TOT_CELLS`, billions.
    pub tot_cells: f64,
    /// `F_Y_VIABLE_CELLS`, billions.
    pub viable_cells: f64,
    /// `F_Y_NEW_AMOUNT`, packs.
    pub new_amount: f64,
    /// `F_Y_VIABILITY_PCT`.
    pub viability_pct: f64,
    /// `F_Y_FORM`.
    pub form: YeastForm,
    /// `F_Y_USE_PKGS` (code 1 = package mode).
    pub use_pkgs: bool,
    /// Amount in volume/weight mode, ounces.
    pub amount_oz: f64,
    /// `F_Y_CELLS_ML`.
    pub cells_ml: f64,
    /// `F_Y_DRY_PITCH_RATE`, g/hL.
    pub dry_pitch_rate: f64,
}

fn primary(yeasts: &[RecipeYeast]) -> impl Iterator<Item = &RecipeYeast> {
    yeasts.iter().filter(|y| !y.secondary)
}

/// `YeastCellsNoStarter`, billions: total or viable cells of the primary yeasts.
#[must_use]
pub fn yeast_cells_no_starter(calc_viability: bool, yeasts: &[RecipeYeast]) -> f64 {
    primary(yeasts)
        .map(|y| {
            if calc_viability {
                y.viable_cells
            } else {
                y.tot_cells
            }
        })
        .sum()
}

/// `YeastPackCount`: sum of `new_amount` over the primary yeasts.
#[must_use]
pub fn yeast_pack_count(yeasts: &[RecipeYeast]) -> f64 {
    primary(yeasts).map(|y| y.new_amount).sum()
}

/// `YeastAvgPackSize`, billions per pack: 0 for an invalid list, 100 when cells ≤ 0 or no packs.
#[must_use]
pub fn yeast_avg_pack_size(cells: f64, amount_sum: f64, valid: bool) -> f64 {
    if !valid {
        return 0.0;
    }
    if cells <= 0.0 || amount_sum == 0.0 {
        return 100.0;
    }
    cells / amount_sum
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
            if y.use_pkgs {
                y.new_amount * DRY_PKG_G
            } else {
                y.amount_oz * OZ_G
            }
        })
        .sum()
}

/// `(YeastForm, YeastUsePkgs, YeastCellsMl, YeastDryPitchRate)` of the first primary yeast,
/// `(0, 1, 1.0, 75.0)` without one.
#[must_use]
pub fn yeast_first_fields(yeasts: &[RecipeYeast]) -> (i64, i64, f64, f64) {
    primary(yeasts).next().map_or((0, 1, 1.0, 75.0), |y| {
        (
            y.form.code(),
            i64::from(y.use_pkgs),
            y.cells_ml,
            y.dry_pitch_rate,
        )
    })
}

// --- §1.7 pitch targets ---

/// Pitch targets in effect, million cells per mL per degree Plato (spec §1.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchTargets {
    /// Ale.
    pub ale: f64,
    /// Lager.
    pub lager: f64,
    /// Hybrid.
    pub mix: f64,
    /// Wine and mead.
    pub wine: f64,
}

impl Default for PitchTargets {
    fn default() -> Self {
        Self {
            ale: O_Y_ALE_PITCH,
            lager: O_Y_LAGER_PITCH,
            mix: O_Y_MIX_PITCH,
            wine: O_Y_WINE_PITCH,
        }
    }
}

/// Recipe `RecommendedLiquidPitch`: wine target for types 4..=7, else by ale type, else 0.75.
#[must_use]
pub fn recommended_liquid_pitch(r_type: i64, ale_type: i64, t: &PitchTargets) -> f64 {
    if is_wine_like_recipe(r_type) {
        return t.wine;
    }
    match ale_type {
        0 => t.ale,
        1 => t.lager,
        2 => t.mix,
        _ => 0.75,
    }
}

/// Recipe pitch range `(rec − 0.4, rec + 0.4)`.
#[must_use]
pub fn pitch_min_max_recipe(rec: f64) -> (f64, f64) {
    (rec - 0.4, rec + 0.4)
}

/// Starter tool pitch range by `F_YC_TYPE`.
#[must_use]
pub fn pitch_min_max_tool(tool_type: StarterType, t: &PitchTargets) -> (f64, f64) {
    let (center, delta) = match tool_type {
        StarterType::AleOrCider => (t.ale, 0.30),
        StarterType::Lager => (t.lager, 0.25),
        StarterType::HybridWheat => (t.mix, 0.40),
        StarterType::MeadOrWine => (t.wine, 0.25),
    };
    (center - delta, center + delta)
}

/// `PitchRange` text, `"min-max"` with two decimals.
#[must_use]
pub fn pitch_range(lo: f64, hi: f64) -> String {
    format!("{lo:.2}-{hi:.2}")
}

/// Minimum dry pitch, g/hL: `max(5, floor(rec_dry / 4))`.
#[must_use]
pub fn dry_min_pitch_rate(rec_dry: f64) -> f64 {
    (rec_dry / 4.0).floor().max(5.0)
}

/// Maximum dry pitch, g/hL: `2 · rec_dry`.
#[must_use]
pub fn dry_max_pitch_rate(rec_dry: f64) -> f64 {
    2.0 * rec_dry
}

/// `DryPitchRange` text, `"min-max"` with one decimal.
#[must_use]
pub fn dry_pitch_range(rec_dry: f64) -> String {
    format!(
        "{:.1}-{:.1}",
        dry_min_pitch_rate(rec_dry),
        dry_max_pitch_rate(rec_dry)
    )
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

/// `TotYeastCellsNeeded`, billions. Dry by weight: `V · 7.5 · (dry_rate / rec_dry_rate)` with
/// both rates floored at 0.01; otherwise [`cells_needed`].
#[must_use]
pub fn tot_yeast_cells_needed(
    dry_by_weight: bool,
    pitch: f64,
    v_l: f64,
    sg: f64,
    dry_rate: f64,
    rec_dry_rate: f64,
) -> f64 {
    if dry_by_weight {
        v_l * 7.5 * (dry_rate.max(0.01) / rec_dry_rate.max(0.01))
    } else {
        cells_needed(pitch, v_l, sg)
    }
}

/// `EstCellsML`, million cells per mL: `n · 1000 / max(v_ml, 0.1)`.
#[must_use]
pub fn est_cells_ml(n: f64, v_ml: f64) -> f64 {
    n * 1000.0 / v_ml.max(0.1)
}

// --- §1.9 cells after a starter ---

/// `YeastCellsStarter`: `viable · GrowthRate(viable / starter_l)`; no starter means no growth.
/// Errors on non-finite input.
pub fn cells_starter(viable: f64, starter_l: f64, stir: bool) -> Result<f64> {
    ensure_finite("viable", viable)?;
    ensure_finite("starter_l", starter_l)?;
    let i = if starter_l > 0.0 {
        viable / starter_l
    } else {
        0.0
    };
    if i <= 0.0 {
        return Ok(viable);
    }
    Ok(viable * growth_rate(i, stir)?)
}

/// `YeastCellsSecStarter`: the second stage starts from the first stage's cells.
pub fn cells_sec_starter(cells1: f64, sec_l: f64, stir: bool) -> Result<f64> {
    cells_starter(cells1, sec_l, stir)
}

// --- §1.11 recipe two-stage ---

/// Recipe two-stage starter volumes `(stage1_l, stage2_l)`: needs `avail < need`, growth per
/// stage capped at `growth_cap` (spec §1.11).
/// Errors on non-finite input.
pub fn two_stage_recipe(need: f64, avail: f64, stir: bool, growth_cap: f64) -> Result<(f64, f64)> {
    ensure_finite("need", need)?;
    ensure_finite("avail", avail)?;
    ensure_finite("growth_cap", growth_cap)?;
    let avail = avail.max(0.1);
    if avail >= need {
        return Ok((0.0, 0.0));
    }
    let mut r = need / avail;
    let mut s = 1.0;
    if stir {
        s = 1.0 + STIR / 100.0;
        r /= s * s;
    }
    let g = r.sqrt().min(growth_cap);
    let v1 = avail / innoc_rate(g)?;
    let v2 = avail * g * s / innoc_rate(g)?;
    Ok((v1, v2))
}

// --- §1.12 packs ---

/// `YeastPacksNoStarter`: `1` below one pack, else `int(n / pkt + 0.99)`; `pkt ≤ 0` counts as 1.
/// Errors on non-finite input.
#[allow(clippy::cast_possible_truncation)]
pub fn packs_no_starter(n: f64, pkt: f64) -> Result<i64> {
    ensure_finite("n", n)?;
    ensure_finite("pkt", pkt)?;
    let pkt = if pkt <= 0.0 { 1.0 } else { pkt };
    Ok(if n < pkt {
        1
    } else {
        (n / pkt + 0.99).trunc() as i64
    })
}

/// `YeastPacksStarter` / `CalcYeastPacksNeeded` with a starter: `int(packs) + 0.1`.
///
/// `growth ≤ 0` counts as 1.2; a stir plate divides it by 1.45 (needs more packs).
/// Errors on non-finite input.
pub fn packs_starter(n: f64, pkt: f64, growth: f64, stir: bool) -> Result<f64> {
    ensure_finite("n", n)?;
    ensure_finite("pkt", pkt)?;
    ensure_finite("growth", growth)?;
    let pkt = if pkt <= 0.0 { 1.0 } else { pkt };
    let mut g = if growth > 0.0 { growth } else { 1.2 };
    if stir {
        g /= 1.0 + STIR / 100.0;
    }
    let p = if n < pkt {
        1.0
    } else {
        (n / (g * pkt) + 0.99).trunc()
    };
    Ok(p + 0.1)
}

/// Legacy `CalcYeastStarterSize`, litres: `pkt · pkt_size / InnocRate(r)` for `r > 1`.
/// Errors on non-finite input.
pub fn calc_yeast_starter_size(r: f64, pkt_size: f64, pkt: f64) -> Result<f64> {
    ensure_finite("r", r)?;
    ensure_finite("pkt_size", pkt_size)?;
    ensure_finite("pkt", pkt)?;
    if r > 1.0 {
        Ok(pkt * pkt_size / innoc_rate(r)?)
    } else {
        Ok(0.0)
    }
}

// --- §1.13 dry yeast ---

/// Dry yeast grams from a rate in g/hL: `rate · V(L) / 100`.
#[must_use]
pub fn dry_grams_vol(dry_rate: f64, v_l: f64) -> f64 {
    dry_rate * v_l / 100.0
}

/// Dry yeast grams in package mode: `n / cells_per_g` (≤ 0 counts as 0.1).
#[must_use]
pub fn dry_grams_pkg(n: f64, cells_per_g: f64) -> f64 {
    n / if cells_per_g > 0.0 { cells_per_g } else { 0.1 }
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

/// `AvgAttenuation`: mean of the minimum and maximum attenuation.
#[must_use]
pub fn avg_attenuation(lo: f64, hi: f64) -> f64 {
    f64::midpoint(lo, hi)
}

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
pub fn import_hook_tolerance(tol: i64, y_type: YeastType) -> i64 {
    match (tol, y_type) {
        (10, YeastType::Wine) => 15,
        (10, YeastType::Champagne) => 18,
        _ => tol,
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
pub fn mead_yan_target(sg: f64, pct: f64, fac: YeastNitrogen) -> Result<f64> {
    ensure_finite("sg", sg)?;
    ensure_finite("pct", pct)?;
    Ok(to_plato(sg) * 10.0 * fac.factor() * (pct / 100.0))
}

/// `MeadTotFermO`, grams: Fermaid-O total, only for method 0 (else 0).
pub fn mead_tot_ferm_o(
    sg: f64,
    v_gal: f64,
    pct: f64,
    fac: YeastNitrogen,
    method: MeadMethod,
) -> Result<f64> {
    ensure_finite("v_gal", v_gal)?;
    if method == MeadMethod::Tosna {
        Ok(mead_yan_target(sg, pct, fac)? * v_gal / 50.0)
    } else {
        mead_yan_target(sg, pct, fac).map(|_| 0.0)
    }
}

/// `MeadTotFermK`, grams: Fermaid-K total, only for method 1 (else 0).
pub fn mead_tot_ferm_k(
    sg: f64,
    v_gal: f64,
    pct: f64,
    fac: YeastNitrogen,
    method: MeadMethod,
) -> Result<f64> {
    ensure_finite("v_gal", v_gal)?;
    if method == MeadMethod::Tiosna {
        Ok(mead_yan_target(sg, pct, fac)? * v_gal / 100.0)
    } else {
        mead_yan_target(sg, pct, fac).map(|_| 0.0)
    }
}

/// `MeadFermO`, grams: one of the four Fermaid-O additions.
pub fn mead_ferm_o(
    sg: f64,
    v_gal: f64,
    pct: f64,
    fac: YeastNitrogen,
    method: MeadMethod,
) -> Result<f64> {
    Ok(mead_tot_ferm_o(sg, v_gal, pct, fac, method)? / 4.0)
}

/// `MeadFermK`, grams: one of the four Fermaid-K additions.
pub fn mead_ferm_k(
    sg: f64,
    v_gal: f64,
    pct: f64,
    fac: YeastNitrogen,
    method: MeadMethod,
) -> Result<f64> {
    Ok(mead_tot_ferm_k(sg, v_gal, pct, fac, method)? / 4.0)
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
pub fn yan_equivalent(method: YanMethod, y: f64) -> f64 {
    if method == YanMethod::FermaidO {
        y * 4.0
    } else {
        y
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
pub fn carb_sugar_agent(agent: PrimingAgent, vols: f64, temp_f: f64, gal: f64) -> f64 {
    let g = sugar_g(vols, temp_f, gal, false);
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

/// `CarbAmountText` number: keg pressure for type 1, else priming sugar divided by
/// `carb_rate / 100` when `carb_rate ≠ 0`. No keg factor (spec §3.3).
#[must_use]
pub fn carb_amount(
    c_type: CarbType,
    vols: f64,
    temp_f: f64,
    gal: f64,
    carb_rate: f64,
    agent: PrimingAgent,
) -> CarbAmount {
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
pub fn carb_equiv(rate: f64) -> f64 {
    100.0 / rate.max(1.0)
}

// --- §3.6, §3.7 ---

/// `CarbCalories`: total minus alcohol calories.
#[must_use]
pub fn carb_calories(total: f64, alc: f64) -> f64 {
    total - alc
}

/// `CarbGrams`: carbohydrate calories over 4 kcal/g.
#[must_use]
pub fn carb_grams(carb_cal: f64) -> f64 {
    carb_cal / 4.0
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
