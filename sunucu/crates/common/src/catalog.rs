//! Field, unit and text catalog (T2.17, ADR-0008 item 7): the single source of field limits.
//!
//! The data lives in `sozlesme/katalog/katalog.json`; `sozlesme/katalog/uret.py` validates it
//! against the JSON Schema and copies it to `catalog/katalog.json` next to this file (the Docker
//! build context is `sunucu/` only). Server validation, the `OpenAPI` limits and
//! `GET /v1/meta/fields` all read this one table. All values are in the SI-like metric units of
//! ADR-0007 (kg, L, °C, min, SG, SRM, ppm, bar, %); the unit of a field is its `unit`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// The embedded catalog document.
pub const RAW: &str = include_str!("catalog/katalog.json");

/// Brewing mode that may narrow a field's limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Home brewing.
    Home,
    /// Brewery.
    Brewery,
}

/// Optional narrower limits of one mode.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ModeLimits {
    /// Lower bound, when it differs from the base one.
    pub min: Option<f64>,
    /// Upper bound, when it differs from the base one.
    pub max: Option<f64>,
}

/// Per-mode overrides of a field.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Modes {
    /// Home mode limits.
    pub home: Option<ModeLimits>,
    /// Brewery mode limits.
    pub brewery: Option<ModeLimits>,
}

/// One field definition.
#[derive(Debug, Clone, Deserialize)]
pub struct FieldDef {
    /// Quantity id (see [`Catalog::quantities`]).
    pub quantity: String,
    /// SI unit id of the field; API values use it.
    pub unit: String,
    /// Widest accepted lower bound (any mode).
    pub min: f64,
    /// Widest accepted upper bound (any mode).
    pub max: f64,
    /// Input step for forms (display only; not enforced).
    pub step: f64,
    /// Display precision.
    pub decimals: u32,
    /// Required when the parent object is sent.
    pub required: bool,
    /// The limit is a product decision, not verified against a source ("doğrulanacak").
    pub verify: bool,
    /// Narrower per-mode limits.
    #[serde(default)]
    pub modes: Modes,
}

/// A quantity and its SI unit.
#[derive(Debug, Clone, Deserialize)]
pub struct Quantity {
    /// SI unit id.
    pub si: String,
}

/// A unit: `si = (value + pre) * mul / div + post`.
#[derive(Debug, Clone, Deserialize)]
pub struct Unit {
    /// Quantity id.
    pub quantity: String,
    /// Multiplier numerator.
    pub mul: f64,
    /// Multiplier denominator.
    pub div: f64,
    /// Offset added before scaling.
    pub pre: f64,
    /// Offset added after scaling.
    pub post: f64,
    /// Name of a non-linear formula (`plato`, `brix`); those have no linear conversion here.
    pub nonlinear: Option<String>,
}

impl Unit {
    /// Converts a value in this unit to the SI unit; `None` for non-linear units.
    #[must_use]
    pub fn to_si(&self, value: f64) -> Option<f64> {
        self.nonlinear
            .is_none()
            .then(|| (value + self.pre) * self.mul / self.div + self.post)
    }

    /// Converts an SI value to this unit; `None` for non-linear units.
    #[must_use]
    pub fn from_si(&self, si: f64) -> Option<f64> {
        self.nonlinear
            .is_none()
            .then(|| (si - self.post) * self.div / self.mul - self.pre)
    }
}

/// Name and symbol of a unit in one language.
#[derive(Debug, Clone, Deserialize)]
pub struct UnitText {
    /// Full name.
    pub name: String,
    /// Short symbol.
    pub symbol: String,
}

/// Label of a field in one language.
#[derive(Debug, Clone, Deserialize)]
pub struct FieldText {
    /// Short label.
    pub label: String,
    /// Optional help text.
    pub hint: Option<String>,
}

/// All texts of one language.
#[derive(Debug, Clone, Deserialize)]
pub struct LocaleTexts {
    /// Quantity names.
    pub quantities: BTreeMap<String, String>,
    /// Unit names and symbols.
    pub units: BTreeMap<String, UnitText>,
    /// Field labels.
    pub fields: BTreeMap<String, FieldText>,
    /// Validation message templates by error code (`{label}`, `{min}`, `{max}`, `{unit}`).
    pub errors: BTreeMap<String, String>,
}

/// The whole catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    /// Document version.
    pub version: u32,
    /// Quantities by id.
    pub quantities: BTreeMap<String, Quantity>,
    /// Units by id.
    pub units: BTreeMap<String, Unit>,
    /// Field definitions by key (`scope.field`).
    pub fields: BTreeMap<String, FieldDef>,
    /// Texts by language (`tr`, `en`).
    pub texts: BTreeMap<String, LocaleTexts>,
}

/// Why a value was rejected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldError {
    /// No field with that key.
    UnknownField,
    /// NaN or infinite.
    NotFinite,
    /// Outside `min..=max` (the limits applied are carried for the error `params`).
    OutOfRange {
        /// Lower bound applied.
        min: f64,
        /// Upper bound applied.
        max: f64,
    },
}

impl FieldError {
    /// The stable error code of ADR-0008 (`errors[].code`).
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnknownField => "unknown_field",
            Self::NotFinite => "invalid_number",
            Self::OutOfRange { .. } => "out_of_range",
        }
    }
}

impl Catalog {
    /// Limits of a field, narrowed by `mode` when given.
    #[must_use]
    pub fn limits(&self, key: &str, mode: Option<Mode>) -> Option<(f64, f64)> {
        let f = self.fields.get(key)?;
        let m = match mode {
            Some(Mode::Home) => f.modes.home,
            Some(Mode::Brewery) => f.modes.brewery,
            None => None,
        };
        Some((
            m.and_then(|m| m.min).unwrap_or(f.min),
            m.and_then(|m| m.max).unwrap_or(f.max),
        ))
    }

    /// Checks an SI value against the field definition.
    ///
    /// # Errors
    /// [`FieldError`] naming why the value is rejected.
    pub fn validate(&self, key: &str, value: f64, mode: Option<Mode>) -> Result<(), FieldError> {
        let (min, max) = self.limits(key, mode).ok_or(FieldError::UnknownField)?;
        if !value.is_finite() {
            return Err(FieldError::NotFinite);
        }
        if value < min || value > max {
            return Err(FieldError::OutOfRange { min, max });
        }
        Ok(())
    }
}

/// The embedded catalog, parsed once.
///
/// # Panics
/// Never for a released build: `tests::embedded_catalog_parses` fails first on bad data.
#[must_use]
#[allow(clippy::expect_used)] // embedded, test-verified document
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(RAW).expect("embedded catalog is valid"))
}

/// Checks an SI value against the embedded catalog (see [`Catalog::validate`]).
///
/// # Errors
/// [`FieldError`] naming why the value is rejected.
pub fn validate(key: &str, value: f64, mode: Option<Mode>) -> Result<(), FieldError> {
    catalog().validate(key, value, mode)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]
mod tests {
    use super::*;
    use engine::recipe::{
        Equipment, Fermentable, GrainType, GrainUse, Hop, HopForm, HopUse, Mash, MashStep, Misc,
        MiscUse, Options, RecipeInput, RecipeType, StepKind, Yeast, YeastForm,
    };
    use engine::units;

    #[test]
    fn embedded_catalog_parses() {
        let c = catalog();
        assert_eq!(c.version, 1);
        assert!(c.fields.len() > 30 && c.units.len() > 30);
    }

    #[test]
    fn fields_use_the_si_unit_of_their_quantity() {
        let c = catalog();
        for (key, f) in &c.fields {
            assert_eq!(f.unit, c.quantities[&f.quantity].si, "{key}");
            assert!(f.min < f.max, "{key}");
            assert!(c.texts["tr"].fields.contains_key(key), "{key} tr");
            assert!(c.texts["en"].fields.contains_key(key), "{key} en");
            for m in [Some(Mode::Home), Some(Mode::Brewery)] {
                let (lo, hi) = c.limits(key, m).unwrap();
                assert!(lo >= f.min && hi <= f.max && lo < hi, "{key} {m:?}");
            }
        }
    }

    #[test]
    fn validate_reports_the_applied_limits() {
        assert_eq!(validate("hop.alpha_pct", 12.0, None), Ok(()));
        assert_eq!(
            validate("hop.alpha_pct", 41.0, None),
            Err(FieldError::OutOfRange {
                min: 0.0,
                max: 40.0
            })
        );
        assert_eq!(
            validate("hop.alpha_pct", f64::NAN, None),
            Err(FieldError::NotFinite)
        );
        assert_eq!(validate("nope.x", 1.0, None), Err(FieldError::UnknownField));
        // Home mode is narrower than the base limit.
        assert_eq!(validate("hop.amount_kg", 6.0, Some(Mode::Brewery)), Ok(()));
        assert!(validate("hop.amount_kg", 6.0, Some(Mode::Home)).is_err());
        assert_eq!(FieldError::NotFinite.code(), "invalid_number");
    }

    #[test]
    fn unit_conversions_use_the_defined_values() {
        let c = catalog();
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-9 * b.abs().max(1.0);
        assert!(near(c.units["gal_us"].to_si(1.0).unwrap(), 3.785_411_784));
        assert!(near(c.units["gal_uk"].to_si(1.0).unwrap(), 4.54609));
        assert!(near(c.units["f"].to_si(212.0).unwrap(), 100.0));
        assert!(near(c.units["f"].from_si(-40.0).unwrap(), -40.0));
        assert!(near(c.units["lb"].to_si(1.0).unwrap(), 0.453_592_37));
        assert!(c.units["plato"].to_si(12.0).is_none());
    }

    /// The catalog uses exact definitions; the engine keeps BeerSmith's rounded constants.
    #[test]
    fn unit_constants_agree_with_the_engine() {
        let c = catalog();
        let rel = |a: f64, b: f64| ((a - b) / b).abs();
        assert!(rel(c.units["fl_oz_us"].mul * 1000.0, units::FLOZ_ML) < 1e-8);
        assert!(rel(c.units["gal_us"].mul, units::GAL_L) < 1e-8);
        assert!(rel(c.units["lb"].mul, units::LB_KG) < 1e-6);
        assert!(rel(c.units["oz"].mul * 1000.0, units::OZ_G) < 1e-6);
        assert!(rel(c.units["psi"].mul, units::PSI_BAR) < 1e-12);
        // 1.9635 g/L per vol (spec 09) against the ideal-gas constant 44.01 / 22.414.
        assert!(rel(c.units["g_l_co2"].div, units::CO2_G_PER_VOL) < 1e-4);
        for celsius in [-40.0, 0.0, 20.0, 100.0] {
            let f = units::c_to_f(celsius);
            assert!((c.units["f"].to_si(f).unwrap() - celsius).abs() < 1e-9);
        }
    }

    fn recipe() -> RecipeInput {
        RecipeInput {
            kind: RecipeType::from_code(0).unwrap(),
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
            fermentables: vec![Fermentable {
                name: String::new(),
                amount_oz: 1.0,
                yield_pct: 80.0,
                color_srm: 2.0,
                kind: GrainType::from_code(0).unwrap(),
                usage: GrainUse::from_code(0).unwrap(),
                not_fermentable: false,
                late_extract_min: 0.0,
                max_in_batch_pct: 0.0,
                ibu_gal_per_lb: 0.0,
            }],
            hops: vec![Hop {
                amount_oz: 1.0,
                alpha_pct: 5.0,
                form: HopForm::from_code(0).unwrap(),
                usage: HopUse::from_code(0).unwrap(),
                time_min: 60.0,
                whirlpool_temp_f: 180.0,
            }],
            yeasts: vec![Yeast {
                min_attenuation_pct: 70.0,
                max_attenuation_pct: 80.0,
                tolerance_abv_pct: 10.0,
                add_to_secondary: false,
                form: YeastForm::from_code(0).unwrap(),
                total_cells_billion: 0.0,
                viable_cells_billion: 0.0,
                packs: 1.0,
                viability_pct: 100.0,
                use_packages: true,
                amount_oz: 0.0,
                cells_per_ml_billion: 0.0,
                dry_pitch_rate_g_per_hl: 0.0,
            }],
            miscs: vec![Misc {
                usage: MiscUse::from_code(0).unwrap(),
                time_min: 0.0,
            }],
            mash: Mash {
                grain_weight_oz: 0.0,
                biab: false,
                steps: vec![MashStep {
                    kind: StepKind::from_code(0).unwrap(),
                    temp_f: 150.0,
                    infusion_floz: 0.0,
                }],
            },
            options: Options::default(),
        }
    }

    type Setter = fn(&mut RecipeInput, f64);

    fn oz_from_kg(kg: f64) -> f64 {
        kg / units::LB_KG * 16.0
    }

    /// Catalog field, SI to engine unit, setter, and whether the engine rejects negatives.
    #[allow(clippy::type_complexity, clippy::too_many_lines)]
    fn engine_fields() -> Vec<(&'static str, fn(f64) -> f64, Setter, bool)> {
        let id: fn(f64) -> f64 = |x| x;
        let floz: fn(f64) -> f64 = units::l_to_floz;
        let oz: fn(f64) -> f64 = oz_from_kg;
        let f: fn(f64) -> f64 = units::c_to_f;
        let ft: fn(f64) -> f64 = |m| m / 0.3048;
        vec![
            (
                "recipe.batch_volume_l",
                floz,
                |r, x| r.volume_floz = x,
                true,
            ),
            (
                "recipe.boil_volume_l",
                floz,
                |r, x| r.boil_vol_floz = x,
                true,
            ),
            ("recipe.boil_time_min", id, |r, x| r.boil_time_min = x, true),
            (
                "recipe.efficiency_pct",
                id,
                |r, x| r.efficiency_pct = x,
                false,
            ),
            (
                "equipment.trub_loss_l",
                floz,
                |r, x| r.equipment.trub_loss_floz = x,
                true,
            ),
            (
                "equipment.pre_trub_loss_l",
                floz,
                |r, x| r.equipment.pre_trub_loss_floz = x,
                true,
            ),
            (
                "equipment.altitude_m",
                ft,
                |r, x| r.equipment.altitude_ft = x,
                false,
            ),
            (
                "equipment.hop_util_pct",
                id,
                |r, x| r.equipment.hop_util_pct = x,
                true,
            ),
            (
                "equipment.whirlpool_time_min",
                id,
                |r, x| r.equipment.whirlpool_time_min = x,
                true,
            ),
            (
                "fermentable.amount_kg",
                oz,
                |r, x| r.fermentables[0].amount_oz = x,
                true,
            ),
            (
                "fermentable.yield_pct",
                id,
                |r, x| r.fermentables[0].yield_pct = x,
                true,
            ),
            (
                "fermentable.color_srm",
                id,
                |r, x| r.fermentables[0].color_srm = x,
                true,
            ),
            (
                "fermentable.late_extract_min",
                id,
                |r, x| r.fermentables[0].late_extract_min = x,
                true,
            ),
            ("hop.amount_kg", oz, |r, x| r.hops[0].amount_oz = x, true),
            ("hop.alpha_pct", id, |r, x| r.hops[0].alpha_pct = x, true),
            ("hop.time_min", id, |r, x| r.hops[0].time_min = x, true),
            (
                "hop.whirlpool_temp_c",
                f,
                |r, x| r.hops[0].whirlpool_temp_f = x,
                false,
            ),
            (
                "yeast.min_attenuation_pct",
                id,
                |r, x| r.yeasts[0].min_attenuation_pct = x,
                true,
            ),
            (
                "yeast.max_attenuation_pct",
                id,
                |r, x| r.yeasts[0].max_attenuation_pct = x,
                true,
            ),
            (
                "yeast.tolerance_abv_pct",
                id,
                |r, x| r.yeasts[0].tolerance_abv_pct = x,
                true,
            ),
            (
                "yeast.viability_pct",
                id,
                |r, x| r.yeasts[0].viability_pct = x,
                true,
            ),
            (
                "mash.grain_weight_kg",
                oz,
                |r, x| r.mash.grain_weight_oz = x,
                true,
            ),
            (
                "mash.step_temp_c",
                f,
                |r, x| r.mash.steps[0].temp_f = x,
                false,
            ),
        ]
    }

    /// The catalog range must lie inside what the engine gate accepts; where the engine's own
    /// bound is 0 (non-negative), the catalog minimum must be exactly that bound.
    #[test]
    fn limits_are_consistent_with_engine_validation() {
        let c = catalog();
        assert!(recipe().validate().is_ok());
        for (key, to_engine, set, non_negative) in engine_fields() {
            let def = &c.fields[key];
            for si in [def.min, def.max, f64::midpoint(def.min, def.max)] {
                let mut r = recipe();
                set(&mut r, to_engine(si));
                assert!(
                    r.validate().is_ok(),
                    "{key}: engine rejects catalog value {si}"
                );
            }
            if non_negative {
                assert!(
                    def.min >= 0.0,
                    "{key}: catalog allows what the engine rejects"
                );
                if def.min == 0.0 {
                    let mut r = recipe();
                    set(&mut r, to_engine(-1e-6));
                    assert!(
                        r.validate().is_err(),
                        "{key}: engine accepts below catalog min"
                    );
                }
            }
        }
    }

    #[test]
    fn engine_scope_fields_are_listed() {
        let c = catalog();
        for (key, ..) in engine_fields() {
            assert!(c.fields.contains_key(key), "{key}");
        }
    }
}
