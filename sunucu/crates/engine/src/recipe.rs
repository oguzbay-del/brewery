//! Shared recipe input model (ADR-0004 items 1 and 4): data types and validation only, no
//! formulas.
//!
//! Units are BeerSmith's internal units (oz, fl oz, °F, minutes, SG, SRM, percent); every
//! field name carries its unit suffix. `density`, `hops` and `yeast` take these types, so a
//! recipe is described once for every section. Metric modules (`mash`, `home`) keep their
//! own metric parameters; conversion happens in `units` or in `engine-service`.

use crate::error::{Error, Result, ensure_finite, ensure_non_negative};

/// Maximum number of items in any recipe list (ADR-0004 item 3).
pub const MAX_LIST_LEN: usize = 128;

/// Defines a code enum with `ALL`, `from_code`, `code` and `name` (English spec name).
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
pub(crate) use code_enum;

code_enum! {
    /// `F_R_TYPE` (spec 01 §0.2, 04 §1.1). Code 7 is accepted as a wine type by the 04
    /// reference code but is not listed; see `yeast::is_wine_like_recipe`.
    RecipeType { Extract = 0 => "Extract", PartialMash = 1 => "Partial Mash",
                 AllGrain = 2 => "All Grain", Cider = 3 => "Cider", Mead = 4 => "Mead",
                 Wine = 5 => "Wine", Seltzer = 6 => "Seltzer" }
}
code_enum! {
    /// `F_Y_FORM` of an ingredient (spec 04 §1.1). Culture behaves like slurry for viability.
    YeastForm { Liquid = 0 => "Liquid", Dry = 1 => "Dry", Slant = 2 => "Slant",
                Culture = 3 => "Culture" }
}
code_enum! {
    /// Misc usage `F_M_USE` (spec 03 §0 codes 0-5; code 6 whirlpool, spec 02 §9).
    MiscUse { Boil = 0 => "Boil", Mash = 1 => "Mash", Primary = 2 => "Primary",
              Secondary = 3 => "Secondary", Bottling = 4 => "Bottling", Sparge = 5 => "Sparge",
              Whirlpool = 6 => "Whirlpool" }
}

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
    /// Type from its `F_G_TYPE` code; other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        Ok(match code {
            0 => Self::Grain,
            1 => Self::Extract,
            2 => Self::Sugar,
            3 => Self::Adjunct,
            4 => Self::DryExtract,
            5 => Self::Fruit,
            6 => Self::Juice,
            7 => Self::Honey,
            8 => Self::Acid,
            _ => return Err(Error::invalid("grain_type", "unknown code")),
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
    /// Use from its `F_G_USE` code; other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        Ok(match code {
            0 => Self::Mash,
            1 => Self::Steep,
            2 => Self::Boil,
            3 => Self::Whirlpool,
            4 => Self::Primary,
            5 => Self::Secondary,
            6 => Self::Tertiary,
            7 => Self::Bottling,
            8 => Self::Sparge,
            _ => return Err(Error::invalid("grain_use", "unknown code")),
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

/// Hop form `F_H_FORM` (spec §0). Codes: 0 pellet, 1 plug, 2 leaf, 3 CO2 extract,
/// 4 isomerised extract (amounts of forms 3 and 4 are fl oz).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopForm {
    /// Code 0.
    Pellet,
    /// Code 1.
    Plug,
    /// Code 2.
    Leaf,
    /// Code 3.
    Co2Extract,
    /// Code 4.
    IsomerizedExtract,
}

impl HopForm {
    /// From the BeerSmith code (0-4); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::Pellet),
            1 => Ok(Self::Plug),
            2 => Ok(Self::Leaf),
            3 => Ok(Self::Co2Extract),
            4 => Ok(Self::IsomerizedExtract),
            _ => Err(Error::invalid("hop_form", "unknown code")),
        }
    }
}

/// Hop usage `F_H_USE` (spec §0). Codes: 0 boil, 1 dry hop, 2 mash, 3 first wort, 4 whirlpool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopUse {
    /// Code 0.
    Boil,
    /// Code 1.
    DryHop,
    /// Code 2.
    Mash,
    /// Code 3.
    FirstWort,
    /// Code 4.
    Whirlpool,
}

impl HopUse {
    /// From the BeerSmith code (0-4); other codes are an error.
    pub const fn from_code(code: i64) -> Result<Self> {
        match code {
            0 => Ok(Self::Boil),
            1 => Ok(Self::DryHop),
            2 => Ok(Self::Mash),
            3 => Ok(Self::FirstWort),
            4 => Ok(Self::Whirlpool),
            _ => Err(Error::invalid("hop_use", "unknown code")),
        }
    }
}
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

impl StepKind {
    /// The `F_MS_TYPE` code (0-2).
    #[must_use]
    pub const fn code(self) -> i64 {
        self as i64
    }
}

impl HopForm {
    /// The `F_H_FORM` code (0-4).
    #[must_use]
    pub const fn code(self) -> i64 {
        self as i64
    }
}

impl HopUse {
    /// The `F_H_USE` code (0-4).
    #[must_use]
    pub const fn code(self) -> i64 {
        self as i64
    }
}

/// Hop addition (spec 02 §0, §1). `amount_oz` in oz (fl oz for forms 3 and 4), `alpha_pct`
/// in %, `time_min` in minutes, `whirlpool_temp_f` in °F (used for `HopUse::Whirlpool`).
#[derive(Debug, Clone, PartialEq)]
pub struct Hop {
    pub amount_oz: f64,
    pub alpha_pct: f64,
    pub form: HopForm,
    pub usage: HopUse,
    pub time_min: f64,
    pub whirlpool_temp_f: f64,
}

/// Hop correction options (spec 02 §4); defaults are BeerSmith's (`Default`).
#[derive(Debug, Clone, PartialEq)]
pub struct HopOptions {
    /// `O_HOP_FIRST_WORT_ADJ` in % (default +10).
    pub first_wort_adj_pct: f64,
    /// `O_HOP_TYPE_ADJ`: true applies the per-form corrections (default true).
    pub type_adj: bool,
    /// `O_HOP_LEAF_ADJ` in % (default 0).
    pub leaf_adj_pct: f64,
    /// `O_HOP_PLUG_ADJ` in % (default +2).
    pub plug_adj_pct: f64,
    /// `O_HOP_PELLET_ADJ` in % (default +10).
    pub pellet_adj_pct: f64,
    /// `O_HOP_MASH_ADJ` in % (default -80).
    pub mash_adj_pct: f64,
}

impl Default for HopOptions {
    fn default() -> Self {
        Self {
            first_wort_adj_pct: 10.0,
            type_adj: true,
            leaf_adj_pct: 0.0,
            plug_adj_pct: 2.0,
            pellet_adj_pct: 10.0,
            mash_adj_pct: -80.0,
        }
    }
}

/// Fermentable (spec 01 §0.2, §2; spec 02 §5.4 for `ibu_gal_per_lb`).
#[derive(Debug, Clone, PartialEq)]
pub struct Fermentable {
    /// Name, used only to order late extracts with equal times (spec 01 §3.6).
    pub name: String,
    pub amount_oz: f64,
    /// Yield (%, 0-100).
    pub yield_pct: f64,
    pub color_srm: f64,
    /// `F_G_TYPE`.
    pub kind: GrainType,
    /// `F_G_USE`.
    pub usage: GrainUse,
    /// `F_G_NOT_FERMENTABLE` (e.g. lactose): the whole point value goes to FG.
    pub not_fermentable: bool,
    /// `F_G_LATE_EXTRACT` (minutes before the end of the boil); 0 = not a late extract.
    pub late_extract_min: f64,
    /// `F_G_MAX_IN_BATCH` (% of the grain bill), used by the colour adjustment.
    pub max_in_batch_pct: f64,
    /// Hopped-extract rating (IBU·gal/lb), 0 = not hopped (spec 02 §5.4).
    pub ibu_gal_per_lb: f64,
}

/// Yeast ingredient: attenuation fields (spec 01 §5.1, §6.8) and the fields of the yeast
/// totals (spec 04 §1.6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Yeast {
    /// `F_Y_MIN_ATTENUATION` (%).
    pub min_attenuation_pct: f64,
    /// `F_Y_MAX_ATTENUATION` (%).
    pub max_attenuation_pct: f64,
    /// `F_Y_TOLERANCE`, alcohol tolerance (%ABV).
    pub tolerance_abv_pct: f64,
    /// `F_Y_ADD_TO_SECONDARY`: skipped by the recipe FG and by every yeast total.
    pub add_to_secondary: bool,
    /// `F_Y_FORM`.
    pub form: YeastForm,
    /// `F_Y_TOT_CELLS` (billions).
    pub total_cells_billion: f64,
    /// `F_Y_VIABLE_CELLS` (billions).
    pub viable_cells_billion: f64,
    /// `F_Y_NEW_AMOUNT` (packs).
    pub packs: f64,
    /// `F_Y_VIABILITY_PCT`.
    pub viability_pct: f64,
    /// `F_Y_USE_PKGS`: package mode (true) or volume/weight mode.
    pub use_packages: bool,
    /// Amount in volume/weight mode (oz).
    pub amount_oz: f64,
    /// `F_Y_CELLS_ML` (billions per mL).
    pub cells_per_ml_billion: f64,
    /// `F_Y_DRY_PITCH_RATE` (g/hL).
    pub dry_pitch_rate_g_per_hl: f64,
}

/// Misc item; only usage and time are read (whirlpool-time rule, spec 02 §9).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Misc {
    pub usage: MiscUse,
    pub time_min: f64,
}

/// One mash step (spec 01 §3.8, §5.1; spec 05).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MashStep {
    pub kind: StepKind,
    /// `F_MS_STEP_TEMP` (°F).
    pub temp_f: f64,
    /// `F_MS_INFUSION` (fl oz).
    pub infusion_floz: f64,
}

/// Mash profile (spec 01 §3.8, §5.1).
#[derive(Debug, Clone, PartialEq)]
pub struct Mash {
    /// `F_MH_GRAIN_WEIGHT` (oz).
    pub grain_weight_oz: f64,
    /// `F_MH_BIAB`.
    pub biab: bool,
    pub steps: Vec<MashStep>,
}

/// Equipment fields read by the recipe-level calculations.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Equipment {
    /// `F_R_TRUB_LOSS` / `F_E_TRUB_LOSS` (fl oz).
    pub trub_loss_floz: f64,
    /// `F_E_PRE_TRUB_LOSS`: volume before trub loss, used by Extract recipes (fl oz).
    pub pre_trub_loss_floz: f64,
    /// `F_R_ALTITUDE` (ft).
    pub altitude_ft: f64,
    /// `F_E_HOP_UTIL` (%).
    pub hop_util_pct: f64,
    /// `F_E_WHIRLPOOL_CARRY`.
    pub whirlpool_carry: bool,
    /// `F_E_WHIRLPOOL_TIME` (minutes).
    pub whirlpool_time_min: f64,
}

/// Options that travel with the recipe (spec 01 §10, spec 02 §4).
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// `O_STEEP_EFFICIENCY` (%, default 15).
    pub steep_efficiency_pct: f64,
    /// `O_MASH_FG_ADJUST` (default on).
    pub mash_fg_adjust: bool,
    /// `O_MASH_FG_ADJ_TEMP` (°F, default 153.5).
    pub mash_fg_adj_temp_f: f64,
    /// `O_MASH_FG_ADJ_SLOPE` (% attenuation per °F, default -1.25).
    pub mash_fg_adj_slope: f64,
    pub hop: HopOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            steep_efficiency_pct: 15.0,
            mash_fg_adjust: true,
            mash_fg_adj_temp_f: 153.5,
            mash_fg_adj_slope: -1.25,
            hop: HopOptions::default(),
        }
    }
}

/// Recipe input shared by sections 01, 02 and 04 (ADR-0004 item 1). Volumes in US fl oz,
/// temperatures in °F, efficiencies in %.
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeInput {
    /// `F_R_TYPE`.
    pub kind: RecipeType,
    /// `F_R_VOLUME`: batch volume into the fermenter (fl oz).
    pub volume_floz: f64,
    /// `F_R_BOIL_VOL`: boil volume (fl oz).
    pub boil_vol_floz: f64,
    /// `F_R_BOIL_TIME` (minutes).
    pub boil_time_min: f64,
    /// `F_R_EFFICIENCY`: brewhouse efficiency (%).
    pub efficiency_pct: f64,
    pub equipment: Equipment,
    pub fermentables: Vec<Fermentable>,
    pub hops: Vec<Hop>,
    pub yeasts: Vec<Yeast>,
    pub miscs: Vec<Misc>,
    pub mash: Mash,
    pub options: Options,
}

fn check_len(field: &'static str, len: usize) -> Result<()> {
    if len > MAX_LIST_LEN {
        Err(Error::invalid(field, "too many items (max 128)"))
    } else {
        Ok(())
    }
}

impl RecipeInput {
    /// Gate of the engine (ADR-0004 item 3): every number is finite, amounts, volumes, times,
    /// alpha, yield and colour are not negative, and every list has at most
    /// [`MAX_LIST_LEN`] items. Enum codes are already checked by `from_code`.
    ///
    /// # Errors
    /// [`Error::InvalidInput`] naming the first offending field.
    pub fn validate(&self) -> Result<()> {
        check_len("fermentables", self.fermentables.len())?;
        check_len("hops", self.hops.len())?;
        check_len("yeasts", self.yeasts.len())?;
        check_len("miscs", self.miscs.len())?;
        check_len("mash.steps", self.mash.steps.len())?;
        ensure_non_negative("volume_floz", self.volume_floz)?;
        ensure_non_negative("boil_vol_floz", self.boil_vol_floz)?;
        ensure_non_negative("boil_time_min", self.boil_time_min)?;
        ensure_finite("efficiency_pct", self.efficiency_pct)?;
        let e = &self.equipment;
        ensure_non_negative("equipment.trub_loss_floz", e.trub_loss_floz)?;
        ensure_non_negative("equipment.pre_trub_loss_floz", e.pre_trub_loss_floz)?;
        ensure_finite("equipment.altitude_ft", e.altitude_ft)?;
        ensure_non_negative("equipment.hop_util_pct", e.hop_util_pct)?;
        ensure_non_negative("equipment.whirlpool_time_min", e.whirlpool_time_min)?;
        for g in &self.fermentables {
            ensure_non_negative("fermentable.amount_oz", g.amount_oz)?;
            ensure_non_negative("fermentable.yield_pct", g.yield_pct)?;
            ensure_non_negative("fermentable.color_srm", g.color_srm)?;
            ensure_non_negative("fermentable.late_extract_min", g.late_extract_min)?;
            ensure_non_negative("fermentable.max_in_batch_pct", g.max_in_batch_pct)?;
            ensure_non_negative("fermentable.ibu_gal_per_lb", g.ibu_gal_per_lb)?;
        }
        for h in &self.hops {
            ensure_non_negative("hop.amount_oz", h.amount_oz)?;
            ensure_non_negative("hop.alpha_pct", h.alpha_pct)?;
            ensure_non_negative("hop.time_min", h.time_min)?;
            ensure_finite("hop.whirlpool_temp_f", h.whirlpool_temp_f)?;
        }
        for y in &self.yeasts {
            ensure_non_negative("yeast.min_attenuation_pct", y.min_attenuation_pct)?;
            ensure_non_negative("yeast.max_attenuation_pct", y.max_attenuation_pct)?;
            ensure_non_negative("yeast.tolerance_abv_pct", y.tolerance_abv_pct)?;
            ensure_non_negative("yeast.total_cells_billion", y.total_cells_billion)?;
            ensure_non_negative("yeast.viable_cells_billion", y.viable_cells_billion)?;
            ensure_non_negative("yeast.packs", y.packs)?;
            ensure_non_negative("yeast.viability_pct", y.viability_pct)?;
            ensure_non_negative("yeast.amount_oz", y.amount_oz)?;
            ensure_non_negative("yeast.cells_per_ml_billion", y.cells_per_ml_billion)?;
            ensure_non_negative("yeast.dry_pitch_rate_g_per_hl", y.dry_pitch_rate_g_per_hl)?;
        }
        for m in &self.miscs {
            ensure_non_negative("misc.time_min", m.time_min)?;
        }
        ensure_non_negative("mash.grain_weight_oz", self.mash.grain_weight_oz)?;
        for s in &self.mash.steps {
            ensure_finite("mash.step.temp_f", s.temp_f)?;
            ensure_finite("mash.step.infusion_floz", s.infusion_floz)?;
        }
        let o = &self.options;
        ensure_non_negative("options.steep_efficiency_pct", o.steep_efficiency_pct)?;
        ensure_finite("options.mash_fg_adj_temp_f", o.mash_fg_adj_temp_f)?;
        ensure_finite("options.mash_fg_adj_slope", o.mash_fg_adj_slope)?;
        let h = &o.hop;
        ensure_finite("options.hop.first_wort_adj_pct", h.first_wort_adj_pct)?;
        ensure_finite("options.hop.leaf_adj_pct", h.leaf_adj_pct)?;
        ensure_finite("options.hop.plug_adj_pct", h.plug_adj_pct)?;
        ensure_finite("options.hop.pellet_adj_pct", h.pellet_adj_pct)?;
        ensure_finite("options.hop.mash_adj_pct", h.mash_adj_pct)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RecipeInput {
        RecipeInput {
            kind: RecipeType::AllGrain,
            volume_floz: 640.0,
            boil_vol_floz: 768.0,
            boil_time_min: 60.0,
            efficiency_pct: 72.0,
            equipment: Equipment {
                trub_loss_floz: 0.0,
                pre_trub_loss_floz: 0.0,
                altitude_ft: 0.0,
                hop_util_pct: 100.0,
                whirlpool_carry: false,
                whirlpool_time_min: 0.0,
            },
            fermentables: vec![],
            hops: vec![],
            yeasts: vec![],
            miscs: vec![],
            mash: Mash {
                grain_weight_oz: 0.0,
                biab: false,
                steps: vec![],
            },
            options: Options::default(),
        }
    }

    #[test]
    fn validate_accepts_sample_and_rejects_bad_numbers() {
        assert!(sample().validate().is_ok());
        let mut r = sample();
        r.volume_floz = f64::NAN;
        assert!(r.validate().is_err());
        let mut r = sample();
        r.equipment.trub_loss_floz = -1.0;
        assert!(r.validate().is_err());
        let mut r = sample();
        r.options.mash_fg_adj_slope = f64::INFINITY;
        assert!(r.validate().is_err());
    }

    #[test]
    fn validate_bounds_lists() {
        let mut r = sample();
        r.miscs = vec![
            Misc {
                usage: MiscUse::Boil,
                time_min: 1.0
            };
            MAX_LIST_LEN + 1
        ];
        assert!(r.validate().is_err());
        r.miscs.truncate(MAX_LIST_LEN);
        assert!(r.validate().is_ok());
    }
}
