//! The neutral model between the file formats and the recipe API. Field names of the lines match
//! the API line objects, so [`Recipe::version_input`] can go to `NewRecipe` as it is.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Values a source file states about the recipe. They are not trusted: the server calculates its
/// own values and shows both.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SourceValues {
    /// Original gravity, SG.
    pub og_sg: Option<f64>,
    /// Final gravity, SG.
    pub fg_sg: Option<f64>,
    /// Bitterness, IBU.
    pub ibu: Option<f64>,
    /// Color, SRM.
    pub srm: Option<f64>,
    /// Alcohol, percent.
    pub abv_pct: Option<f64>,
}

/// A fermentable line.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Fermentable {
    /// Name.
    pub name: String,
    /// Amount, kg.
    pub amount_kg: f64,
    /// `mash`, `steep`, `boil`, ...
    pub usage: Option<String>,
    /// `grain`, `extract`, `sugar`, `adjunct`, `dry_extract`, `fruit`, `juice`, `honey`, `acid`.
    pub grain_type: Option<String>,
    /// Yield, percent.
    pub yield_pct: Option<f64>,
    /// Color, SRM.
    pub color_srm: Option<f64>,
}

/// A hop line.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Hop {
    /// Name.
    pub name: String,
    /// Amount, kg.
    pub amount_kg: f64,
    /// Alpha acid, percent.
    pub alpha_pct: Option<f64>,
    /// `pellet`, `plug`, `leaf`, ...
    pub form: Option<String>,
    /// `boil`, `dry_hop`, `mash`, `first_wort`, `whirlpool`.
    pub usage: Option<String>,
    /// Time, minutes.
    pub time_min: Option<f64>,
    /// Whirlpool temperature, degrees C.
    pub whirlpool_temp_c: Option<f64>,
}

/// A yeast line.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Yeast {
    /// Name.
    pub name: String,
    /// `liquid`, `dry`, `slant`, `culture`.
    pub form: Option<String>,
    /// Attenuation, percent.
    pub attenuation_pct: Option<f64>,
    /// Amount in the source unit (BeerXML: L or kg).
    #[serde(default)]
    pub amount: Option<f64>,
    /// Whether the source amount is a weight rather than a volume.
    #[serde(default)]
    pub amount_is_weight: Option<bool>,
    /// Packs.
    pub packs: Option<f64>,
}

/// A misc line.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Misc {
    /// Name.
    pub name: String,
    /// Amount (L for a liquid, else kg).
    pub amount: f64,
    /// The misc is a liquid.
    pub is_liquid: Option<bool>,
    /// `boil`, `mash`, `primary`, ...
    pub usage: Option<String>,
    /// Time, minutes.
    pub time_min: Option<f64>,
}

/// A mash step.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MashStep {
    /// Name.
    pub name: Option<String>,
    /// `infusion`, `decoction`, `temperature`.
    pub kind: String,
    /// Step temperature, degrees C.
    pub temp_c: f64,
    /// Time, minutes.
    pub time_min: f64,
    /// Infusion water, L.
    pub infusion_l: Option<f64>,
}

/// An equipment profile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    /// Name.
    pub name: String,
    /// Batch volume into the fermenter, L.
    pub batch_volume_l: Option<f64>,
    /// Pre-boil volume, L.
    pub boil_volume_l: Option<f64>,
    /// Boil time, minutes.
    pub boil_time_min: Option<f64>,
    /// Brewhouse efficiency, percent.
    pub efficiency_pct: Option<f64>,
    /// Trub and chiller loss, L.
    pub trub_loss_l: Option<f64>,
    /// Fermenter loss, L.
    pub fermenter_loss_l: Option<f64>,
    /// Boil off, L per hour.
    pub boil_off_l_per_h: Option<f64>,
}

/// Style metadata from a BeerXML STYLE record.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BeerStyle {
    pub name: String,
    pub category: String,
    pub category_number: String,
    pub style_letter: String,
    pub style_guide: String,
    pub kind: String,
    pub og_min: f64,
    pub og_max: f64,
    pub fg_min: f64,
    pub fg_max: f64,
    pub ibu_min: f64,
    pub ibu_max: f64,
    pub color_min: f64,
    pub color_max: f64,
    pub carb_min: Option<f64>,
    pub carb_max: Option<f64>,
    pub abv_min: Option<f64>,
    pub abv_max: Option<f64>,
    pub notes: Option<String>,
    pub profile: Option<String>,
    pub ingredients: Option<String>,
    pub examples: Option<String>,
}

/// A recipe.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    /// Name.
    pub name: String,
    /// Notes.
    pub notes: Option<String>,
    /// `extract`, `partial_mash`, `all_grain`, ...
    pub kind: String,
    /// Style name in the source.
    pub style: Option<String>,
    /// Brewer name when the source format provides it.
    #[serde(default)]
    pub brewer: Option<String>,
    /// Full BeerXML style metadata when available.
    #[serde(default)]
    pub style_details: Option<BeerStyle>,
    /// Bitterness formula the source names (`tinseth`, `rager`, ...).
    pub ibu_method: Option<String>,
    /// Batch volume, L.
    pub batch_volume_l: Option<f64>,
    /// Pre-boil volume, L.
    pub boil_volume_l: Option<f64>,
    /// Boil time, minutes.
    pub boil_time_min: Option<f64>,
    /// Brewhouse efficiency, percent.
    pub efficiency_pct: Option<f64>,
    /// What the file states.
    pub source: SourceValues,
    /// The equipment of the recipe.
    pub equipment: Option<Equipment>,
    /// Fermentables.
    pub fermentables: Vec<Fermentable>,
    /// Hops.
    pub hops: Vec<Hop>,
    /// Yeasts.
    pub yeasts: Vec<Yeast>,
    /// Miscs.
    pub miscs: Vec<Misc>,
    /// Mash steps.
    pub mash_steps: Vec<MashStep>,
    /// What the parser changed or could not map.
    pub warnings: Vec<String>,
}

/// A parsed file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Recipes.
    pub recipes: Vec<Recipe>,
    /// Equipment profiles that are not tied to a recipe.
    pub equipment: Vec<Equipment>,
    /// Warnings of the whole file.
    pub warnings: Vec<String>,
}

fn put(m: &mut Map<String, Value>, key: &str, v: Option<f64>) {
    if let Some(v) = v {
        m.insert(key.to_owned(), json!(v));
    }
}

fn lines<T: Serialize>(items: &[T]) -> Value {
    Value::Array(
        items
            .iter()
            .filter_map(|i| serde_json::to_value(i).ok())
            .map(|mut v| {
                if let Some(o) = v.as_object_mut() {
                    o.retain(|_, x| !x.is_null());
                }
                v
            })
            .collect(),
    )
}

impl Recipe {
    /// The `VersionInput` JSON of the API. The source values become the targets.
    #[must_use]
    pub fn version_input(&self, equipment_id: Option<&str>) -> Value {
        let mut m = Map::new();
        m.insert("kind".into(), json!(self.kind));
        if let Some(id) = equipment_id {
            m.insert("equipment_id".into(), json!(id));
        }
        put(&mut m, "batch_volume_l", self.batch_volume_l);
        put(&mut m, "boil_volume_l", self.boil_volume_l);
        put(&mut m, "boil_time_min", self.boil_time_min);
        put(&mut m, "efficiency_pct", self.efficiency_pct);
        put(&mut m, "target_og_sg", self.source.og_sg);
        put(&mut m, "target_fg_sg", self.source.fg_sg);
        put(&mut m, "target_ibu", self.source.ibu);
        put(&mut m, "target_srm", self.source.srm);
        put(&mut m, "target_abv_pct", self.source.abv_pct);
        if !self.mash_steps.is_empty() {
            let steps: Vec<Value> = self
                .mash_steps
                .iter()
                .map(|s| {
                    let mut o = Map::new();
                    o.insert("kind".into(), json!(s.kind));
                    o.insert("temp_c".into(), json!(s.temp_c));
                    o.insert("time_min".into(), json!(s.time_min));
                    put(&mut o, "infusion_l", s.infusion_l);
                    if let Some(n) = &s.name {
                        o.insert("name".into(), json!(n));
                    }
                    Value::Object(o)
                })
                .collect();
            m.insert(
                "mash_snapshot".into(),
                json!({"biab": false, "steps": steps}),
            );
        }
        m.insert("fermentables".into(), lines(&self.fermentables));
        m.insert("hops".into(), lines(&self.hops));
        m.insert("yeasts".into(), lines(&self.yeasts_for_api()));
        m.insert("miscs".into(), lines(&self.miscs));
        Value::Object(m)
    }

    fn yeasts_for_api(&self) -> Vec<Value> {
        self.yeasts
            .iter()
            .map(|y| {
                let mut o = Map::new();
                o.insert("name".into(), json!(y.name));
                if let Some(f) = &y.form {
                    o.insert("form".into(), json!(f));
                }
                put(&mut o, "min_attenuation_pct", y.attenuation_pct);
                put(&mut o, "max_attenuation_pct", y.attenuation_pct);
                put(&mut o, "packs", y.packs);
                Value::Object(o)
            })
            .collect()
    }
}

/// Converts a SRM value to EBC.
#[must_use]
pub fn srm_to_ebc(srm: f64) -> f64 {
    srm * 1.97
}

/// Normalizes a recipe kind name from a file to the API values.
#[must_use]
pub fn kind_from(text: &str) -> String {
    let t = text.to_ascii_lowercase().replace(['_', '-'], " ");
    match t.trim() {
        "extract" => "extract",
        "partial mash" => "partial_mash",
        "cider" => "cider",
        "mead" => "mead",
        "wine" => "wine",
        "seltzer" => "seltzer",
        _ => "all_grain",
    }
    .to_owned()
}

/// The `BeerXML` name of an API recipe kind.
#[must_use]
pub fn kind_to_xml(kind: &str) -> &'static str {
    match kind {
        "extract" => "Extract",
        "partial_mash" => "Partial Mash",
        _ => "All Grain",
    }
}
