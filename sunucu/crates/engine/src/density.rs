//! Section 01: gravity, alcohol, colour (`formuller/01-yogunluk-alkol-renk.md`).
//!
//! Units follow the specification of each function; see the section file. Internal units
//! are BeerSmith's: SG, °Plato, °Brix, °F, SRM, US fl oz, US gallons, percent (0-100).
//! Shared conversions (`to_plato`, `to_sg`) live in [`crate::units`].

use crate::units::{c_to_f, f_to_c, to_plato, to_sg};

/// Brix to SG (§1.4): `b / (258.6 - b/258.2 * 227.1) + 1`.
#[must_use]
pub fn brix_to_sg(brix: f64) -> f64 {
    brix / (258.6 - (brix / 258.2) * 227.1) + 1.0
}

/// SG to °Brix (§1.4), cubic.
#[must_use]
pub fn sg_to_brix(sg: f64) -> f64 {
    ((182.460_1 * sg - 775.682_1) * sg + 1_262.779_4) * sg - 669.562_2
}

/// °Brix to refractive index (§1.4).
#[must_use]
pub fn brix_to_ri(brix: f64) -> f64 {
    1.333_02 + 0.001_427_193 * brix + 5.791_157e-6 * brix * brix
}

/// Refractive index to °Brix (§1.4), inverse of [`brix_to_ri`].
#[must_use]
pub fn ri_to_brix(ri: f64) -> f64 {
    ((2.036_879_859_249e-06 - (1.333_02 - ri) * 2.316_462_8e-05).sqrt() - 0.001_427_193)
        / 1.158_231_4e-05
}

/// Original extract clamped to be at least the apparent extract (°P), §6.
fn clamped_extracts(og: f64, fg: f64) -> (f64, f64) {
    let f = to_plato(fg);
    (to_plato(og).max(f), f)
}

/// Real extract (°P) from OG and FG (SG), §6.1. `OE = max(OE, AE)`.
#[must_use]
pub fn real_extract(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    0.1808 * o + 0.8192 * f
}

/// Alcohol by weight (%) from OG and FG (SG), §6.2.
#[must_use]
pub fn abw(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    let re = 0.1808 * o + 0.8192 * f;
    (o - re) / (2.0665 - 0.010_665 * o)
}

/// Alcohol by volume (%) from OG and FG (SG), §6.3: `FG / 0.79661 * ABW`.
#[must_use]
pub fn abv(og: f64, fg: f64) -> f64 {
    fg / 0.79661 * abw(og, fg)
}

/// Apparent attenuation (%), §6.4. OG = 1.000 divides by zero (no guard, as in the reference).
#[must_use]
pub fn apparent_attenuation(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    (o - f) / o * 100.0
}

/// Real attenuation (%), §6.5.
#[must_use]
pub fn real_attenuation(og: f64, fg: f64) -> f64 {
    let (o, f) = clamped_extracts(og, fg);
    (1.0 - (0.1808 * o + 0.8192 * f) / o) * 100.0
}

/// Calories from alcohol per 12 US fl oz serving (kcal), §7.1. Takes OG and FG in SG.
#[must_use]
pub fn alcohol_calories(og: f64, fg: f64) -> f64 {
    1881.22 * fg * (og - fg) / (1.775 - og)
}

/// Calories from extract (carbohydrate) per 12 US fl oz serving (kcal), §7.2. OG and FG in SG.
#[must_use]
pub fn extract_calories(og: f64, fg: f64) -> f64 {
    3550.0 * fg * ((0.1808 * og + 0.8192 * fg) - 1.0004)
}

/// UK tax ABV (%) from measured OG and FG (SG), §6.7. Table step applies when `d > threshold`,
/// with `d = (OG - FG) * 1000`.
#[must_use]
pub fn uk_tax_abv(og: f64, fg: f64) -> f64 {
    const STEPS: [(f64, f64); 10] = [
        (6.95, 0.126),
        (10.45, 0.127),
        (17.25, 0.128),
        (26.15, 0.129),
        (36.05, 0.130),
        (46.55, 0.131),
        (57.15, 0.132),
        (67.95, 0.133),
        (78.85, 0.134),
        (89.75, 0.135),
    ];
    let d = (og - fg) * 1000.0;
    let mut k = 0.125;
    for (limit, factor) in STEPS {
        if d > limit {
            k = factor;
        }
    }
    k * d
}

/// Morey colour (SRM), §8.1: `1.4922 * (mcu_sum / volume_gal)^0.6859`, where `mcu_sum` is
/// the sum of (pounds x grain SRM) and `volume_gal` is in US gallons.
#[must_use]
pub fn morey_srm(mcu_sum: f64, volume_gal: f64) -> f64 {
    1.4922 * (mcu_sum / volume_gal).powf(0.6859)
}

/// Hydrometer temperature correction (SG), §9.1. `temp_f` is the sample temperature and
/// `calib_f` the hydrometer calibration temperature, both in °F.
#[must_use]
pub fn hydrometer_correction(sg: f64, temp_f: f64, calib_f: f64) -> f64 {
    let t = (temp_f - 32.0) * 5.0 / 9.0;
    let tc = (calib_f - 32.0) * 5.0 / 9.0;
    sg + sg
        * ((tc.powi(3) - t.powi(3)) * 1.532_436_35e-08 + (tc - t) * 1.628_605_87e-05
            - (tc.powi(2) - t.powi(2)) * 5.849_948_55e-06)
}

/// Refractometer, unfermented wort (SG), §9.2 type 0: `ToSG((brix - zero) / wcf)`.
/// Defaults in the reference: `zero = 0.0`, `wcf = 1.02`.
#[must_use]
pub fn refractometer_unfermented(brix: f64, zero: f64, wcf: f64) -> f64 {
    to_sg((brix - zero) / wcf)
}

/// Refractometer, fermenting wort with known OG (returns FG in SG), §9.2 type 1
/// (Terrill cubic as implemented by BeerSmith: no WCF, OB in °Plato).
#[must_use]
pub fn refractometer_fermenting(og: f64, brix: f64, zero: f64) -> f64 {
    let ob = to_plato(og);
    let fb = brix - zero;
    1.001_843 - 0.002_318_474 * ob - 7.775e-6 * ob * ob - 3.4e-8 * ob.powi(3)
        + 0.005_74 * fb
        + 3.344e-5 * fb * fb
        + 8.6e-8 * fb.powi(3)
}

/// Result of [`refractometer_finished`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FinishedBeer {
    /// Original gravity (SG).
    pub og: f64,
    /// Alcohol by weight (%).
    pub abw_percent: f64,
    /// Real extract (°P).
    pub re: f64,
    /// Original extract (°P).
    pub oe: f64,
}

/// Refractometer on finished beer with hydrometer FG (SG): original gravity, §9.2 type 2.
#[must_use]
pub fn refractometer_finished(fg: f64, brix: f64, zero: f64) -> FinishedBeer {
    let ri = brix_to_ri(brix - zero);
    let abw = (1017.5596 - 277.4 * fg + ri * (937.8135 * ri - 1805.1288)) / 100.0;
    let re = 194.5935 + 129.8 * fg + ri * (410.8815 * ri - 790.8732);
    let oe = (re + 2.0665 * abw * 100.0) / (1.0 + 0.010_665 * abw * 100.0);
    FinishedBeer {
        og: to_sg(oe),
        abw_percent: abw * 100.0,
        re,
        oe,
    }
}

/// FG (SG) limited by yeast alcohol tolerance, §5.2: bisection between `fg` and 1.18 until
/// `|ABV - tol| < 0.04` (at most 50 iterations, then the last midpoint). `tol` is in %ABV.
#[must_use]
#[allow(clippy::manual_midpoint)] // keep `(hi + lo) / 2` for bit parity with the reference
pub fn fg_from_tolerance(og: f64, fg: f64, tol: f64) -> f64 {
    let mut hi = 1.18;
    let mut lo = fg;
    let mut mid = (hi + lo) / 2.0;
    for _ in 0..50 {
        mid = (hi + lo) / 2.0;
        let a = abv(og, mid);
        if (a - tol).abs() < 0.04 {
            return mid;
        }
        if a > tol {
            lo = mid;
        } else if a < tol {
            hi = mid;
        }
    }
    mid
}

/// Extract to add to raise gravity (oz), §9.3. `current` and `target` in SG, `volume_floz`
/// in US fl oz (minimum 0.01). `kind`: 0 = DME (44 PPG), anything else = LME (36 PPG).
/// Returns 0 when `target <= current`.
#[must_use]
pub fn gravity_adjust_extract_oz(current: f64, target: f64, volume_floz: f64, kind: i64) -> f64 {
    let points = (target - current) * 1000.0;
    if points <= 0.0 {
        return 0.0;
    }
    let points = points * volume_floz.max(0.01) / 128.0;
    points * 16.0 / if kind == 0 { 44.0 } else { 36.0 }
}

/// Water to add to lower gravity (US fl oz), §9.4; negative results clamp to 0. `target = 1.000`
/// divides by zero (no guard, as in the reference).
#[must_use]
pub fn gravity_adjust_water_floz(current: f64, target: f64, volume_floz: f64) -> f64 {
    (volume_floz * (current - 1.0) * 1000.0 / ((target - 1.0) * 1000.0) - volume_floz).max(0.0)
}

// ---------------------------------------------------------------------------------------
// T1.14e: enums, unit converters and recipe-level calculations (spec §0.2, §1.3-1.7, §2-§5.1,
// §6.6, §6.8, §7.3, §8). Reference: `formuller/testler/01_yogunluk_tv.py` (appended part).
// ---------------------------------------------------------------------------------------

/// Fermentable type `F_G_TYPE` (§0.2). Codes 0-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainType {
    /// 0, mashable.
    Grain,
    /// 1, liquid malt extract, fixed yield.
    Extract,
    /// 2, sugar, fixed yield.
    Sugar,
    /// 3, adjunct, mashable.
    Adjunct,
    /// 4, dry malt extract, fixed yield.
    DryExtract,
    /// 5, fruit, fixed yield.
    Fruit,
    /// 6, juice, fixed yield.
    Juice,
    /// 7, honey, fixed yield.
    Honey,
    /// 8, acid, treated like grain (mashable).
    Acid,
}

impl GrainType {
    /// Type from its `F_G_TYPE` code; `None` outside 0-8.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        Some(match code {
            0 => Self::Grain,
            1 => Self::Extract,
            2 => Self::Sugar,
            3 => Self::Adjunct,
            4 => Self::DryExtract,
            5 => Self::Fruit,
            6 => Self::Juice,
            7 => Self::Honey,
            8 => Self::Acid,
            _ => return None,
        })
    }

    /// The `F_G_TYPE` code (0-8).
    #[must_use]
    pub const fn code(self) -> i64 {
        self as i64
    }

    /// Fixed-yield types `{1, 2, 4, 5, 6, 7}`: the brewhouse efficiency is not applied (§0.2).
    #[must_use]
    pub const fn is_fixed_yield(self) -> bool {
        matches!(
            self,
            Self::Extract
                | Self::Sugar
                | Self::DryExtract
                | Self::Fruit
                | Self::Juice
                | Self::Honey
        )
    }
}

/// Fermentable use phase `F_G_USE` (§0.2). Codes 0-8; the phase filters compare the codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainUse {
    /// 0
    Mash,
    /// 1
    Steep,
    /// 2
    Boil,
    /// 3, after the boil.
    Whirlpool,
    /// 4
    Primary,
    /// 5
    Secondary,
    /// 6
    Tertiary,
    /// 7
    Bottling,
    /// 8
    Sparge,
}

impl GrainUse {
    /// Use from its `F_G_USE` code; `None` outside 0-8.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        Some(match code {
            0 => Self::Mash,
            1 => Self::Steep,
            2 => Self::Boil,
            3 => Self::Whirlpool,
            4 => Self::Primary,
            5 => Self::Secondary,
            6 => Self::Tertiary,
            7 => Self::Bottling,
            8 => Self::Sparge,
            _ => return None,
        })
    }

    /// The `F_G_USE` code (0-8).
    #[must_use]
    pub const fn code(self) -> i64 {
        self as i64
    }

    /// Mash or Sparge: the brewhouse efficiency applies to mashable types (§2.1).
    #[must_use]
    pub const fn is_mash_or_sparge(self) -> bool {
        matches!(self, Self::Mash | Self::Sparge)
    }
}

/// Recipe type `F_R_TYPE` (§0.2). Codes 0-6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeType {
    /// 0
    Extract,
    /// 1
    PartialMash,
    /// 2
    AllGrain,
    /// 3
    Cider,
    /// 4
    Mead,
    /// 5
    Wine,
    /// 6
    Seltzer,
}

impl RecipeType {
    /// Type from its `F_R_TYPE` code; `None` outside 0-6.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        Some(match code {
            0 => Self::Extract,
            1 => Self::PartialMash,
            2 => Self::AllGrain,
            3 => Self::Cider,
            4 => Self::Mead,
            5 => Self::Wine,
            6 => Self::Seltzer,
            _ => return None,
        })
    }

    /// The `F_R_TYPE` code (0-6).
    #[must_use]
    pub const fn code(self) -> i64 {
        self as i64
    }
}

/// Fermentable (minimal input, §0.2 and §2).
#[derive(Debug, Clone, PartialEq)]
pub struct Fermentable {
    /// Name, used only to order late extracts with equal times (§3.6).
    pub name: String,
    /// Amount (oz).
    pub amount_oz: f64,
    /// Yield (%, 0-100).
    pub yield_pct: f64,
    /// Colour (SRM).
    pub color_srm: f64,
    /// Type `F_G_TYPE`.
    pub kind: GrainType,
    /// Use phase `F_G_USE`.
    pub usage: GrainUse,
    /// `F_G_NOT_FERMENTABLE` (e.g. lactose): the whole point value goes to FG (§2.2).
    pub not_fermentable: bool,
    /// `F_G_LATE_EXTRACT` (minutes before the end of the boil); 0 = not a late extract (§3.6).
    pub late_extract_min: f64,
    /// `F_G_MAX_IN_BATCH` (% of the grain bill), used by the colour adjustment (§8.3).
    pub max_in_batch_pct: f64,
}

/// Yeast (minimal input, §5.1 and §6.8).
#[derive(Debug, Clone, PartialEq)]
pub struct Yeast {
    /// `F_Y_MIN_ATTENUATION` (%).
    pub min_attenuation: f64,
    /// `F_Y_MAX_ATTENUATION` (%).
    pub max_attenuation: f64,
    /// `F_Y_TOLERANCE`, alcohol tolerance (%ABV).
    pub tolerance_abv: f64,
    /// `F_Y_ADD_TO_SECONDARY`: such yeasts are not used for the recipe FG.
    pub add_to_secondary: bool,
}

/// One mash step (§3.8, §5.1).
#[derive(Debug, Clone, PartialEq)]
pub struct MashStep {
    /// `F_MS_STEP_TEMP` (°F).
    pub temp_f: f64,
    /// `F_MS_INFUSION` (infusion volume, same unit as the caller uses; only its ratio matters).
    pub infusion: f64,
}

/// Mash profile (§3.8, §5.1).
#[derive(Debug, Clone, PartialEq)]
pub struct Mash {
    /// `F_MH_GRAIN_WEIGHT` (oz).
    pub grain_weight: f64,
    /// `F_MH_BIAB`.
    pub biab: bool,
    /// Steps in order.
    pub steps: Vec<MashStep>,
}

/// Minimal recipe input for the section 01 calculations. Volumes in US fl oz, temperatures in
/// °F, efficiencies in %. Options (§10) travel with the recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    /// `F_R_TYPE`.
    pub kind: RecipeType,
    /// `F_R_VOLUME`: batch volume into the fermenter (fl oz).
    pub volume_floz: f64,
    /// `F_R_TRUB_LOSS` / `F_E_TRUB_LOSS` (fl oz).
    pub trub_loss_floz: f64,
    /// `F_E_PRE_TRUB_LOSS`: volume before trub loss, used by Extract recipes (fl oz).
    pub pre_trub_loss_floz: f64,
    /// `F_R_BOIL_VOL`: boil volume (fl oz).
    pub boil_vol_floz: f64,
    /// `F_R_BOIL_TIME` (minutes).
    pub boil_time_min: f64,
    /// `F_R_EFFICIENCY`: brewhouse efficiency (%).
    pub efficiency: f64,
    /// `O_STEEP_EFFICIENCY` (%, default 15).
    pub steep_efficiency: f64,
    /// `O_MASH_FG_ADJUST`.
    pub mash_fg_adjust: bool,
    /// `O_MASH_FG_ADJ_TEMP` (°F, default 153.5).
    pub mash_fg_adj_temp: f64,
    /// `O_MASH_FG_ADJ_SLOPE` (% attenuation per °F, default -1.25).
    pub mash_fg_adj_slope: f64,
    /// Fermentables.
    pub fermentables: Vec<Fermentable>,
    /// Yeasts.
    pub yeasts: Vec<Yeast>,
    /// Mash profile.
    pub mash: Mash,
}

/// Fermentable field summed by [`total_grain_use_field`] (§3.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainField {
    /// Amount (oz).
    AmountOz,
    /// Yield (%).
    YieldPct,
    /// Colour (SRM).
    ColorSrm,
}

// --- §1.3-§1.7 unit converters --------------------------------------------------------

/// Gravity unit codes of `BConvertHydrometer` (§0.1, §1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HydrometerUnit {
    /// Code 3, specific gravity.
    Sg,
    /// Code 4, °Plato.
    Plato,
    /// Code 31, °Brix.
    Brix,
}

impl HydrometerUnit {
    /// Unit from its code (3, 4, 31); `None` otherwise.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            3 => Some(Self::Sg),
            4 => Some(Self::Plato),
            31 => Some(Self::Brix),
            _ => None,
        }
    }
}

/// Gravity converter `BConvertHydrometer` (§1.3). `from == to` returns `x` as is. For SG input
/// a value entered as points is normalised first: `> 880` → `/1000`, `> 88` → `/100`,
/// `> 8.8` → `/10` (1050 → 1.050). Plato ↔ Brix go through SG.
#[must_use]
pub fn convert_hydrometer(x: f64, from: HydrometerUnit, to: HydrometerUnit) -> f64 {
    if from == to {
        return x;
    }
    let x = if from == HydrometerUnit::Sg {
        if x > 880.0 {
            x / 1000.0
        } else if x > 88.0 {
            x / 100.0
        } else if x > 8.8 {
            x / 10.0
        } else {
            x
        }
    } else {
        x
    };
    let sg = match from {
        HydrometerUnit::Sg => x,
        HydrometerUnit::Plato => to_sg(x),
        HydrometerUnit::Brix => brix_to_sg(x),
    };
    match to {
        HydrometerUnit::Sg => sg,
        HydrometerUnit::Plato => to_plato(sg),
        HydrometerUnit::Brix => sg_to_brix(sg),
    }
}

/// Refractometer unit codes of `BConvertRefractometer` (§1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefractometerUnit {
    /// Code 3, specific gravity.
    Sg,
    /// Code 31, °Brix.
    Brix,
    /// Code 32, refractive index (nD).
    Ri,
}

impl RefractometerUnit {
    /// Unit from its code (3, 31, 32); `None` otherwise.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            3 => Some(Self::Sg),
            31 => Some(Self::Brix),
            32 => Some(Self::Ri),
            _ => None,
        }
    }
}

/// Refractometer converter `BConvertRefractometer` (§1.4). Everything goes through °Brix; SG ↔
/// Brix uses [`convert_hydrometer`] (including its points normalisation).
#[must_use]
pub fn convert_refractometer(x: f64, from: RefractometerUnit, to: RefractometerUnit) -> f64 {
    if from == to {
        return x;
    }
    let brix = match from {
        RefractometerUnit::Sg => convert_hydrometer(x, HydrometerUnit::Sg, HydrometerUnit::Brix),
        RefractometerUnit::Brix => x,
        RefractometerUnit::Ri => ri_to_brix(x),
    };
    match to {
        RefractometerUnit::Sg => convert_hydrometer(brix, HydrometerUnit::Brix, HydrometerUnit::Sg),
        RefractometerUnit::Brix => brix,
        RefractometerUnit::Ri => brix_to_ri(brix),
    }
}

/// Colour unit codes of `BConvertColor` (§1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorUnit {
    /// Code 29, SRM.
    Srm,
    /// Code 30, EBC.
    Ebc,
    /// Code 47, °Lovibond.
    Lovibond,
}

impl ColorUnit {
    /// Unit from its code (29, 30, 47); `None` otherwise.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            29 => Some(Self::Srm),
            30 => Some(Self::Ebc),
            47 => Some(Self::Lovibond),
            _ => None,
        }
    }
}

/// Colour converter `BConvertColor` (§1.5): `EBC = SRM x 1.97`, `SRM = 1.3564 x °L - 0.76`
/// (the program's 1.3564, not the literature's 1.3546).
#[must_use]
pub fn convert_color(x: f64, from: ColorUnit, to: ColorUnit) -> f64 {
    if from == to {
        return x;
    }
    let srm = match from {
        ColorUnit::Srm => x,
        ColorUnit::Ebc => x / 1.97,
        ColorUnit::Lovibond => 1.3564 * x - 0.76,
    };
    match to {
        ColorUnit::Srm => srm,
        ColorUnit::Ebc => srm * 1.97,
        ColorUnit::Lovibond => (srm + 0.76) / 1.3564,
    }
}

/// Temperature unit codes of `BConvertTemperature` (§1.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnit {
    /// Code 1, °C.
    Celsius,
    /// Code 2, °F.
    Fahrenheit,
    /// Code 40, kelvin.
    Kelvin,
}

impl TemperatureUnit {
    /// Unit from its code (1, 2, 40); `None` otherwise.
    #[must_use]
    pub const fn from_code(code: i64) -> Option<Self> {
        match code {
            1 => Some(Self::Celsius),
            2 => Some(Self::Fahrenheit),
            40 => Some(Self::Kelvin),
            _ => None,
        }
    }
}

/// Temperature converter `BConvertTemperature` (§1.7); kelvin is °C ± 273.15.
#[must_use]
pub fn convert_temperature(x: f64, from: TemperatureUnit, to: TemperatureUnit) -> f64 {
    if from == to {
        return x;
    }
    let celsius = match from {
        TemperatureUnit::Celsius => x,
        TemperatureUnit::Fahrenheit => f_to_c(x),
        TemperatureUnit::Kelvin => x - 273.15,
    };
    match to {
        TemperatureUnit::Celsius => celsius,
        TemperatureUnit::Fahrenheit => c_to_f(celsius),
        TemperatureUnit::Kelvin => celsius + 273.15,
    }
}

/// Potential gravity (SG) from yield (%), §1.6: `1 + yield x 0.046 / 100` (80 % → 1.0368).
#[must_use]
pub fn potential_from_yield(yield_pct: f64) -> f64 {
    1.0 + yield_pct * 0.046 / 100.0
}

/// Yield (%) from potential gravity (SG), §1.6, clamped to `[0, 100]`.
#[must_use]
pub fn yield_from_potential(sg: f64) -> f64 {
    (((sg - 1.0) * 1000.0 / 46.0) * 100.0).clamp(0.0, 100.0)
}

/// `BConvertGrainYield` (§1.6): `from_sg` (source code 3) → [`yield_from_potential`], otherwise
/// [`potential_from_yield`]. The spec's target code is not used by either branch.
#[must_use]
pub fn grain_yield_convert(x: f64, from_sg: bool) -> f64 {
    if from_sg {
        yield_from_potential(x)
    } else {
        potential_from_yield(x)
    }
}

// --- §2 fermentable points -------------------------------------------------------------

/// OG points of one fermentable (point·gallon), §2.1. `eff` = brewhouse efficiency (%),
/// `steep_eff` = `O_STEEP_EFFICIENCY` (%). Fixed-yield types ignore the efficiency; mashable
/// types in Mash or Sparge use `eff`, in any other phase `steep_eff`.
#[must_use]
pub fn grain_og_points(g: &Fermentable, eff: f64, steep_eff: f64) -> f64 {
    let lb_ppg = (g.amount_oz / 16.0) * (g.yield_pct * 46.0 / 100.0);
    if g.kind.is_fixed_yield() {
        lb_ppg
    } else if g.usage.is_mash_or_sparge() {
        lb_ppg * eff / 100.0
    } else {
        lb_ppg * steep_eff / 100.0
    }
}

/// FG points of one fermentable (point·gallon left after fermentation), §2.2. `atten` is the
/// apparent attenuation (%). Sugar leaves -0.231 of its points, fruit, juice and honey -0.05,
/// the rest `1 - atten/100`; a not-fermentable fermentable (lactose) keeps all its points.
#[must_use]
pub fn grain_fg_points(g: &Fermentable, eff: f64, atten: f64, steep_eff: f64) -> f64 {
    let points = grain_og_points(g, eff, steep_eff);
    if g.not_fermentable {
        return points;
    }
    let factor = match g.kind {
        GrainType::Sugar => -0.231,
        GrainType::Fruit | GrainType::Juice | GrainType::Honey => -0.05,
        _ => 1.0 - atten / 100.0,
    };
    points * factor
}

// --- §3 OG and derivatives -------------------------------------------------------------

/// Sum of OG points for fermentables with `use <= phase` or Sparge (§3.1). `phase` is an
/// `F_G_USE` code.
#[must_use]
pub fn og_points_phase(r: &Recipe, phase: i64) -> f64 {
    let mut total = 0.0;
    for g in &r.fermentables {
        if g.usage.code() <= phase || g.usage == GrainUse::Sparge {
            total += grain_og_points(g, r.efficiency, r.steep_efficiency);
        }
    }
    total
}

/// OG estimate (SG) for `volume_floz` (fl oz) up to `phase`, §3.2; 1.0 when the volume is not
/// positive.
#[must_use]
pub fn estimate_og(r: &Recipe, volume_floz: f64, phase: i64) -> f64 {
    if volume_floz <= 0.0 {
        return 1.0;
    }
    1.0 + og_points_phase(r, phase) / (volume_floz / 128.0) / 1000.0
}

/// Volume used for OG and FG: the pre-trub volume for Extract recipes, else the batch volume.
fn og_volume(r: &Recipe) -> f64 {
    if r.kind == RecipeType::Extract {
        r.pre_trub_loss_floz
    } else {
        r.volume_floz
    }
}

/// Recipe OG (SG), §3.3: phases up to Primary plus Sparge.
#[must_use]
pub fn recipe_og(r: &Recipe) -> f64 {
    estimate_og(r, og_volume(r), 4)
}

/// Virtual OG (SG), §3.4: like [`recipe_og`] but Secondary and Tertiary count too.
#[must_use]
pub fn recipe_virtual_og(r: &Recipe) -> f64 {
    estimate_og(r, og_volume(r), 6)
}

/// Estimated post-mash gravity (SG), §3.5: Mash + Sparge points diluted from batch volume to
/// the boil volume.
#[must_use]
pub fn est_post_mash_og(r: &Recipe) -> f64 {
    let mut p0 = (estimate_og(r, r.volume_floz, 0) - 1.0) * 1000.0;
    if r.boil_vol_floz > 0.0 {
        p0 = p0 * (r.trub_loss_floz + r.volume_floz) / r.boil_vol_floz;
    }
    1.0 + p0 / 1000.0
}

/// One entry of the late-extract SG profile (§3.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LateExtractPoint {
    /// Gravity at that time (SG).
    pub sg: f64,
    /// Minutes before the end of the boil.
    pub time_min: f64,
}

fn push_profile_point(profile: &mut Vec<LateExtractPoint>, sg: f64, time_min: f64) {
    if let Some(last) = profile.last_mut()
        && (time_min - last.time_min).abs() < 0.1
    {
        last.sg = sg;
        return;
    }
    profile.push(LateExtractPoint { sg, time_min });
}

/// Late extract SG profile (§3.6). `boil_vol_floz`: boil volume; when `<= 0` the recipe's
/// `boil_vol_floz` is used, and `0.1` if that is `<= 0.1` too. The first entry is the boil
/// start gravity at `boil_time_min`; each late extract (fixed-yield type, `late_extract_min > 0`,
/// use Mash, Steep, Boil or Sparge) follows in decreasing time (ties by name), then the
/// Whirlpool fermentables at time 0. An entry less than 0.1 min from the previous one updates
/// the previous entry's SG instead of adding a new one.
#[must_use]
pub fn late_extract_profile(r: &Recipe, boil_vol_floz: f64) -> Vec<LateExtractPoint> {
    let mut boil_vol = if boil_vol_floz > 0.0 {
        boil_vol_floz
    } else {
        r.boil_vol_floz
    };
    if boil_vol <= 0.1 {
        boil_vol = 0.1;
    }
    let ratio = (r.volume_floz + r.trub_loss_floz) / boil_vol;
    let sg_of = |points: f64| -> f64 {
        if r.volume_floz <= 0.0 {
            1.0
        } else {
            1.0 + points * ratio / (r.volume_floz / 128.0) / 1000.0
        }
    };
    let is_late = |g: &Fermentable| g.kind.is_fixed_yield() && g.late_extract_min > 0.0;
    let in_boil_phase = |g: &Fermentable| {
        matches!(
            g.usage,
            GrainUse::Mash | GrainUse::Steep | GrainUse::Boil | GrainUse::Sparge
        )
    };

    let mut points = 0.0;
    for g in &r.fermentables {
        if in_boil_phase(g) && !is_late(g) {
            points += grain_og_points(g, r.efficiency, r.steep_efficiency);
        }
    }
    let mut profile = vec![LateExtractPoint {
        sg: sg_of(points),
        time_min: r.boil_time_min,
    }];
    let mut lates: Vec<&Fermentable> = r
        .fermentables
        .iter()
        .filter(|g| in_boil_phase(g) && is_late(g))
        .collect();
    lates.sort_by(|a, b| {
        b.late_extract_min
            .partial_cmp(&a.late_extract_min)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    for g in lates {
        points += grain_og_points(g, r.efficiency, r.steep_efficiency);
        push_profile_point(
            &mut profile,
            sg_of(points),
            g.late_extract_min.min(r.boil_time_min),
        );
    }
    for g in &r.fermentables {
        if g.usage == GrainUse::Whirlpool {
            points += grain_og_points(g, r.efficiency, r.steep_efficiency);
        }
    }
    push_profile_point(&mut profile, sg_of(points), 0.0);
    profile
}

/// Pre-boil (boil start) gravity (SG), §3.6: first entry of [`late_extract_profile`].
#[must_use]
pub fn recipe_og_boil(r: &Recipe, boil_vol_floz: f64) -> f64 {
    late_extract_profile(r, boil_vol_floz)[0].sg
}

/// Error of [`scale_gravity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GravityScaleError {
    /// The target gravity has no positive points (`(OG - 1) x 1000 <= 0`).
    TargetNotPositive,
}

/// Scales every fermentable by the same ratio so the recipe reaches `og_target` (SG), §3.7.
/// The IBU rescaling of the spec belongs to section 02 and is not done here. Nothing changes
/// when the current OG has no points; a target without points is an error (checked first).
///
/// # Errors
/// [`GravityScaleError::TargetNotPositive`] when `og_target <= 1.000`.
pub fn scale_gravity(r: &Recipe, og_target: f64) -> Result<Recipe, GravityScaleError> {
    let p_old = (recipe_og(r) - 1.0) * 1000.0;
    let p_new = (og_target - 1.0) * 1000.0;
    let mut out = r.clone();
    if p_old > 0.0 && p_new > 0.0 {
        for g in &mut out.fermentables {
            g.amount_oz = g.amount_oz * p_new / p_old;
        }
        return Ok(out);
    }
    if p_new <= 0.0 {
        return Err(GravityScaleError::TargetNotPositive);
    }
    Ok(out)
}

/// Scales a mash profile to a new grain weight, §3.8. Only the weight changes when either
/// weight is not positive or they are equal, or for BIAB; otherwise every positive infusion keeps
/// its water/grain ratio.
#[must_use]
#[allow(clippy::float_cmp)] // spec: equality of the two weights is a plain comparison
pub fn scale_mash_from_grain_weight(mash: &Mash, grain_weight_new: f64) -> Mash {
    let mut out = mash.clone();
    let old = mash.grain_weight;
    out.grain_weight = grain_weight_new;
    if grain_weight_new <= 0.0 || old <= 0.0 || grain_weight_new == old || out.biab {
        return out;
    }
    for step in &mut out.steps {
        if step.infusion > 0.0 {
            step.infusion = grain_weight_new * (step.infusion / old);
        }
    }
    out
}

/// Sum of `field` over the fermentables with the given use (§3.9).
#[must_use]
pub fn total_grain_use_field(r: &Recipe, usage: GrainUse, field: GrainField) -> f64 {
    let mut total = 0.0;
    for g in &r.fermentables {
        if g.usage == usage {
            total += match field {
                GrainField::AmountOz => g.amount_oz,
                GrainField::YieldPct => g.yield_pct,
                GrainField::ColorSrm => g.color_srm,
            };
        }
    }
    total
}

// --- §4 efficiency ---------------------------------------------------------------------

/// Efficiency (%), §4.1 `GetEfficiency`. `og` in SG (`<= 0` → the recipe OG), `volume_floz` in
/// fl oz (`<= 0` → 0.01). Only Partial Mash and All Grain recipes; others give 0. `mash_only`
/// counts Mash, Steep and Sparge fermentables and does not subtract the fixed-yield points.
#[must_use]
pub fn efficiency(r: &Recipe, og: f64, volume_floz: f64, mash_only: bool) -> f64 {
    let volume = if volume_floz <= 0.0 {
        0.01
    } else {
        volume_floz
    };
    let og = if og <= 0.0 { recipe_og(r) } else { og };
    if !matches!(r.kind, RecipeType::PartialMash | RecipeType::AllGrain) {
        return 0.0;
    }
    let mut fixed = 0.0;
    let mut mashable = 0.0;
    for g in &r.fermentables {
        let counted = if mash_only {
            matches!(g.usage, GrainUse::Mash | GrainUse::Steep | GrainUse::Sparge)
        } else {
            !matches!(
                g.usage,
                GrainUse::Secondary | GrainUse::Tertiary | GrainUse::Bottling
            )
        };
        if counted {
            let points = (g.amount_oz / 16.0) * (g.yield_pct * 46.0 / 100.0);
            if g.kind.is_fixed_yield() {
                fixed += points;
            } else {
                mashable += points;
            }
        }
    }
    let mashable_per_gal = mashable / (volume / 128.0);
    let mut measured = (og - 1.0) * 1000.0;
    if !mash_only {
        measured -= fixed / (volume / 128.0);
    }
    if measured < 0.0 || mashable_per_gal <= 0.0 {
        return 0.0;
    }
    measured / mashable_per_gal * 100.0
}

// --- §5.1 recipe FG, §6.6, §6.8 --------------------------------------------------------

/// Average yeast attenuation (%), §6.8: `(min + max) / 2`.
#[must_use]
#[allow(clippy::manual_midpoint)] // keep `(min + max) / 2` for bit parity with the reference
pub fn avg_attenuation(min: f64, max: f64) -> f64 {
    (min + max) / 2.0
}

/// Attenuation (%) and alcohol tolerance (%ABV) used by the recipe FG, §5.1 step 1: the first
/// yeast not added to secondary, else 73 % and 10 %.
#[must_use]
pub fn yeast_attenuation(r: &Recipe) -> (f64, f64) {
    for y in &r.yeasts {
        if !y.add_to_secondary {
            return (
                avg_attenuation(y.min_attenuation, y.max_attenuation),
                y.tolerance_abv,
            );
        }
    }
    (73.0, 10.0)
}

/// Attenuation (%) after the mash temperature adjustment, §5.1 step 2 (All Grain only).
/// The step nearest to the reference temperature (within 40 °F) counts as the saccharification
/// temperature; a warmer step after a qualifying one below the reference (>= 139 °F) is skipped.
/// With the option on, the chosen temperature is clamped to 144-160 °F and
/// `atten += slope x (T - Tref)`; no steps means `T = Tref` (no change).
#[must_use]
pub fn mash_adjusted_attenuation(r: &Recipe, atten: f64) -> f64 {
    if r.kind != RecipeType::AllGrain {
        return atten;
    }
    let tref = r.mash_fg_adj_temp;
    let mut best = tref;
    let mut best_diff = 40.0;
    let mut prev = 0.0;
    for step in &r.mash.steps {
        let t = step.temp_f;
        let diff = (t - tref).abs();
        if diff < best_diff {
            if prev < tref && prev >= 139.0 && t > tref {
                continue;
            }
            best = t;
            best_diff = diff;
            prev = t;
        }
    }
    if r.mash_fg_adjust {
        best = best.clamp(144.0, 160.0);
        return atten + r.mash_fg_adj_slope * (best - tref);
    }
    atten
}

/// Recipe FG (SG), §5.1: FG points of all fermentables (no phase filter), then limited by the
/// yeast alcohol tolerance via [`fg_from_tolerance`] when the ABV (from [`recipe_og`]) exceeds it.
#[must_use]
pub fn recipe_fg(r: &Recipe) -> f64 {
    let (atten, tolerance) = yeast_attenuation(r);
    let atten = mash_adjusted_attenuation(r, atten);
    let mut points = 0.0;
    for g in &r.fermentables {
        points += grain_fg_points(g, r.efficiency, atten, r.steep_efficiency);
    }
    let volume = og_volume(r);
    let mut fg = if volume <= 0.0 {
        1.0
    } else {
        1.0 + points / (volume / 128.0) / 1000.0
    };
    let og = recipe_og(r);
    if abv(og, fg) > tolerance && tolerance > 0.0 {
        fg = fg_from_tolerance(og, fg, tolerance);
    }
    fg
}

/// Original extract (°P) of a tool's start gravity (SG), §6.6.
#[must_use]
pub fn original_extract(start_sg: f64) -> f64 {
    to_plato(start_sg)
}

/// Apparent extract (°P) of a tool's final gravity (SG), §6.6.
#[must_use]
pub fn apparent_extract(final_sg: f64) -> f64 {
    to_plato(final_sg)
}

// --- §7.3 calorie uses (OG = max(OG, FG) first) ----------------------------------------

/// Alcohol calories (kcal per 12 US fl oz) with `OG = max(OG, FG)`, §7.3.
#[must_use]
pub fn calories_alcohol(og: f64, fg: f64) -> f64 {
    alcohol_calories(og.max(fg), fg)
}

/// Carbohydrate (extract) calories (kcal per 12 US fl oz) with `OG = max(OG, FG)`, §7.3.
#[must_use]
pub fn calories_carbs(og: f64, fg: f64) -> f64 {
    extract_calories(og.max(fg), fg)
}

/// Total calories (kcal per 12 US fl oz): alcohol + carbohydrate, §7.3.
#[must_use]
pub fn calories_total(og: f64, fg: f64) -> f64 {
    calories_alcohol(og, fg) + calories_carbs(og, fg)
}

/// Carbohydrates (g) from carbohydrate calories (kcal), 4 kcal/g, §7.3.
#[must_use]
pub fn carbohydrates_grams(carb_kcal: f64) -> f64 {
    carb_kcal / 4.0
}

/// Carbohydrate calories (kcal) as total minus alcohol calories, §7.3 `CarbCalories`.
#[must_use]
pub fn carb_calories(total_kcal: f64, alc_kcal: f64) -> f64 {
    total_kcal - alc_kcal
}

// --- §8 colour -------------------------------------------------------------------------

/// Recipe colour (SRM), §8.1 Morey: all fermentables (no phase filter), volume
/// `(volume + trub loss) / 128` gallons; 0 when that volume is not positive.
#[must_use]
pub fn recipe_color(r: &Recipe) -> f64 {
    let mut mcu_total = 0.0;
    for g in &r.fermentables {
        mcu_total += (g.amount_oz / 16.0) * g.color_srm;
    }
    let volume_gal = (r.volume_floz + r.trub_loss_floz) / 128.0;
    if volume_gal <= 0.0 {
        return 0.0;
    }
    morey_srm(mcu_total, volume_gal)
}

/// Inverse Morey (MCU) from SRM, §8.2: `(0.67015 x SRM)^1.45794`.
#[must_use]
pub fn mcu_from_srm(srm: f64) -> f64 {
    (0.67015 * srm).powf(1.45794)
}

/// Total fermentable amount (oz), the assumed `F_R_TOT_GRAINS` (§11 item 5).
fn total_grains_oz(r: &Recipe) -> f64 {
    let mut total = 0.0;
    for g in &r.fermentables {
        total += g.amount_oz;
    }
    total
}

/// Reachable colour range of the colour adjustment (§8.3 `GetColorFlexibility`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorFlexibility {
    /// Lowest reachable colour (SRM).
    pub lo: f64,
    /// Highest reachable colour (SRM).
    pub hi: f64,
    /// False when no adjustment is possible.
    pub ok: bool,
}

/// `GetColorFlexibility` (§8.3, approximate in the spec). Darkening (`target > current`) skips
/// fermentables at least as dark as the target whose share already reaches
/// `max_in_batch + 5` %; lightening skips fermentables with zero amount darker than the target.
#[must_use]
#[allow(clippy::float_cmp)] // spec: `lo == hi` means no range
pub fn color_flexibility(r: &Recipe, target: f64) -> ColorFlexibility {
    let current = recipe_color(r);
    let total = total_grains_oz(r);
    if total <= 0.0 || r.volume_floz <= 0.0 {
        return ColorFlexibility {
            lo: current,
            hi: current,
            ok: false,
        };
    }
    let mut lo = 1000.0_f64;
    let mut hi = current;
    for g in &r.fermentables {
        let share = g.amount_oz / total;
        if target > current {
            if g.color_srm >= target && share * 100.0 >= g.max_in_batch_pct + 5.0 {
                continue;
            }
        } else if g.amount_oz == 0.0 && g.color_srm > target {
            continue;
        }
        let s = (2.0 * share).min(1.0);
        let mcu = ((1.0 - s) * current * total + g.color_srm * s * total)
            / 16.0
            / (r.volume_floz / 128.0);
        let srm = 1.4922 * mcu.powf(0.6859);
        lo = lo.min(srm);
        hi = hi.max(srm);
    }
    if lo > hi {
        return ColorFlexibility {
            lo: 0.0,
            hi: 0.0,
            ok: false,
        };
    }
    ColorFlexibility {
        lo,
        hi,
        ok: lo != hi,
    }
}

/// Gravity equivalent of `amount` oz of a fermentable (§8.3 `IterateColor`): fixed-yield types
/// `y x A / 16`, Extract recipes use a hard-coded 15 % (`y x A x 15 / 1600`), the rest the
/// brewhouse efficiency.
fn gravity_equivalent(kind: RecipeType, eff: f64, g: &Fermentable, amount: f64) -> f64 {
    let yield_fraction = g.yield_pct / 100.0;
    if g.kind.is_fixed_yield() {
        yield_fraction * amount / 16.0
    } else if kind == RecipeType::Extract {
        yield_fraction * amount * 15.0 / 1600.0
    } else {
        yield_fraction * amount * eff / 1600.0
    }
}

/// `min(max(x, lo), hi)` as the reference writes it (no panic when `lo > hi`).
fn clamp_range(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}

/// One colour iteration towards `target` SRM (§8.3 `IterateColor`): colour-giving fermentables
/// are rescaled by the needed MCU change, base fermentables compensate the gravity change, then
/// the recipe is scaled back to its former OG. Returns the new recipe and its colour (SRM).
///
/// Choices where the spec is silent: the colour-giver branch is skipped when the current colour
/// MCU is zero (division); base fermentables with no gravity contribution are left alone; the
/// compensation applies only to fermentables outside the colour-giver group.
#[must_use]
#[allow(clippy::float_cmp)] // spec: `ΔP != 0`
pub fn iterate_color(r: &Recipe, target: f64) -> (Recipe, f64) {
    let mut out = r.clone();
    let og0 = recipe_og(&out);
    let current = recipe_color(&out);
    let flex = color_flexibility(&out, target);
    let volume_gal = (out.volume_floz + out.trub_loss_floz) / 128.0;
    let mcu_now = mcu_from_srm(current) * volume_gal;
    let delta = mcu_from_srm(target) * volume_gal - mcu_now;
    let weight = total_grains_oz(&out);
    let (kind, eff) = (out.kind, out.efficiency);

    let mut delta_points = 0.0;
    let mut base_weight = 0.0;
    let mut base: Vec<usize> = Vec::new();
    for (i, g) in out.fermentables.iter_mut().enumerate() {
        let (colour, amount) = (g.color_srm, g.amount_oz);
        let mcu = amount * colour / 16.0;
        if (target <= colour && flex.lo < colour) || flex.hi <= colour {
            if colour > 0.0 && mcu_now > 0.0 {
                let amax = (g.max_in_batch_pct + 15.0) / 100.0 * weight;
                let new = (mcu + (mcu / mcu_now) * delta) * 16.0 / colour;
                let new = clamp_range(new, 0.0, amax);
                delta_points += gravity_equivalent(kind, eff, g, new - amount);
                g.amount_oz = new;
            }
        } else {
            base_weight += amount;
            base.push(i);
        }
    }
    if base_weight > 0.0 && delta_points != 0.0 {
        for i in base {
            let g = &mut out.fermentables[i];
            let (colour, amount) = (g.color_srm, g.amount_oz);
            if colour < flex.hi && (colour <= target || colour <= flex.lo) && g.yield_pct != 0.0 {
                let equivalent = gravity_equivalent(kind, eff, g, amount);
                if equivalent <= 0.0 {
                    continue;
                }
                let new = if equivalent <= delta_points {
                    0.0
                } else {
                    amount * (1.0 - amount * (delta_points / equivalent) / base_weight)
                };
                let mut amax = (g.max_in_batch_pct + 15.0) / 100.0 * weight;
                if kind == RecipeType::PartialMash && g.kind == GrainType::Grain {
                    amax = amax.max(0.25 * weight);
                }
                g.amount_oz = clamp_range(new, 0.0, amax);
            }
        }
    }
    if let Ok(scaled) = scale_gravity(&out, og0) {
        out = scaled;
    }
    let colour = recipe_color(&out);
    (out, colour)
}

/// Colour adjustment towards `target` SRM (§8.3 `AdjustColor`, without the IBU preservation of
/// section 02): at most 40 iterations of [`iterate_color`], stopping when the flexibility check
/// fails, the colour moves less than 0.005 or comes within 0.049 of the target. Returns the
/// adjusted recipe and whether `|colour - target| < 0.049`.
#[must_use]
pub fn adjust_color(r: &Recipe, target: f64) -> (Recipe, bool) {
    let mut current = r.clone();
    for _ in 0..40 {
        if !color_flexibility(&current, target).ok {
            break;
        }
        let previous = recipe_color(&current);
        let (next, colour) = iterate_color(&current, target);
        current = next;
        if (colour - previous).abs() < 0.005 {
            break;
        }
        if (colour - target).abs() < 0.049 {
            break;
        }
    }
    let success = (recipe_color(&current) - target).abs() < 0.049;
    (current, success)
}

/// Reachable colour range (SRM), §8.3 `GetColorAdjRange`: the colours after adjusting to 1.0
/// and to 150.0 SRM, as `(min, max)`; adjustment is possible when they differ.
#[must_use]
pub fn color_adj_range(r: &Recipe) -> (f64, f64) {
    let low = recipe_color(&adjust_color(r, 1.0).0);
    let high = recipe_color(&adjust_color(r, 150.0).0);
    (low.min(high), low.max(high))
}
