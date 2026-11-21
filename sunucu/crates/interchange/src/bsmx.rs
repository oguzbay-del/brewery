//! Reader of `BeerSmith` 2 files (`.bsmx`).
//!
//! The file is XML with a `Selections` root. Recipes are in `Data/Recipe`; equipment profiles
//! can be in `Data/Equipment`. The reader uses [`xml::Dialect::Html`] because the files have HTML
//! entities and characters that XML does not allow.
//!
//! Units of the file (checked against a sample file and the `brewscribe` format notes):
//! - Weights (grain, hops) are ounces. Volumes (batch, boil, losses, infusion) are fluid ounces
//!   (US). Temperatures are degrees F. Yield, efficiency and attenuation are percent. Times are
//!   minutes; the dry hop time is days (to be verified).
//! - Index fields: recipe type (extract, partial mash, all grain), grain type, hop form and use,
//!   yeast form, mash step type. The tables are in the functions below.
//! - Colors (`F_G_COLOR`, `F_R_DESIRED_COLOR`) are read as SRM, the same decision as for
//!   `BeerXML`.
//!
//! Decisions (recorded as warnings when they apply):
//! - OG, IBU and color of the recipe come from the `DESIRED` fields. The file does not store the
//!   calculated values. A value of zero means not set.
//! - Grain type 1 is "extract or sugar" in the format notes. It becomes `extract`.
//! - The fields of misc lines (spices, fining) are not verified. Misc lines are not imported.
//! - To be verified with files of other `BeerSmith` versions: the index tables.

use crate::{
    Document, Equipment, Error, Recipe, Result,
    model::{Fermentable, Hop, MashStep, SourceValues, Yeast},
    xml::{self, Dialect, Element, Limits},
};

const KG_PER_OZ: f64 = 0.028_349_523_125;
const L_PER_FL_OZ: f64 = 0.029_573_529_562_5;

fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn flag(e: &Element, name: &str) -> Option<bool> {
    e.text_of(name).map(|t| t == "1")
}

/// A number that is greater than zero. The file writes "not set" as zero.
fn set(e: &Element, name: &str) -> Option<f64> {
    e.number_of(name).filter(|v| *v > 0.0)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn index(e: &Element, name: &str) -> Option<usize> {
    e.number_of(name)
        .filter(|v| (0.0..1000.0).contains(v))
        .map(|v| v.trunc() as usize)
}

fn oz_to_kg(e: &Element, name: &str) -> Option<f64> {
    e.number_of(name).map(|v| v * KG_PER_OZ)
}

fn fl_oz_to_l(e: &Element, name: &str) -> Option<f64> {
    e.number_of(name).map(|v| v * L_PER_FL_OZ)
}

fn parse_equipment(e: &Element) -> Equipment {
    Equipment {
        name: e
            .text_of("F_E_NAME")
            .unwrap_or("Imported equipment")
            .to_owned(),
        batch_volume_l: fl_oz_to_l(e, "F_E_BATCH_VOL").filter(|v| *v > 0.0),
        boil_volume_l: fl_oz_to_l(e, "F_E_BOIL_VOL").filter(|v| *v > 0.0),
        boil_time_min: set(e, "F_E_BOIL_TIME"),
        efficiency_pct: set(e, "F_E_EFFICIENCY"),
        trub_loss_l: fl_oz_to_l(e, "F_E_TRUB_LOSS"),
        fermenter_loss_l: fl_oz_to_l(e, "F_E_FERMENTER_LOSS"),
        boil_off_l_per_h: fl_oz_to_l(e, "F_E_BOIL_OFF"),
    }
}

fn data_children<'a>(e: &'a Element, container: &str, item: &'a str) -> Vec<&'a Element> {
    e.child(container)
        .and_then(|c| c.child("Data"))
        .map(|d| d.children_named(item).collect())
        .unwrap_or_default()
}

fn parse_fermentable(g: &Element, warnings: &mut Vec<String>) -> Fermentable {
    let gt = match index(g, "F_G_TYPE").unwrap_or(0) {
        0 => "grain",
        1 => {
            let msg = "grain type 1 (extract or sugar) was mapped to extract".to_owned();
            if !warnings.contains(&msg) {
                warnings.push(msg);
            }
            "extract"
        }
        3 => "dry_extract",
        _ => "adjunct",
    };
    let usage = if gt != "grain" {
        "boil"
    } else if flag(g, "F_G_RECOMMEND_MASH") == Some(false) {
        "steep"
    } else {
        "mash"
    };
    Fermentable {
        name: g.text_of("F_G_NAME").unwrap_or("Fermentable").to_owned(),
        amount_kg: oz_to_kg(g, "F_G_AMOUNT").unwrap_or(0.0),
        usage: Some(usage.to_owned()),
        grain_type: Some(gt.to_owned()),
        yield_pct: g.number_of("F_G_YIELD"),
        color_srm: g.number_of("F_G_COLOR"),
    }
}

fn parse_hop(h: &Element) -> Hop {
    let hop_use = index(h, "F_H_USE").unwrap_or(0);
    let usage = match hop_use {
        1 => "dry_hop",
        2 => "mash",
        3 => "first_wort",
        4 => "whirlpool",
        _ => "boil",
    };
    let time_min = if hop_use == 1 {
        // The dry hop time is days.
        h.number_of("F_H_DRY_HOP_TIME").map(|d| d * 1440.0)
    } else {
        h.number_of("F_H_BOIL_TIME")
    };
    let form = match index(h, "F_H_FORM").unwrap_or(0) {
        1 => "plug",
        2 => "leaf",
        _ => "pellet",
    };
    Hop {
        name: h.text_of("F_H_NAME").unwrap_or("Hop").to_owned(),
        amount_kg: oz_to_kg(h, "F_H_AMOUNT").unwrap_or(0.0),
        alpha_pct: h.number_of("F_H_ALPHA"),
        form: Some(form.to_owned()),
        usage: Some(usage.to_owned()),
        time_min,
        whirlpool_temp_c: None,
    }
}

fn parse_yeast(y: &Element) -> Yeast {
    let lo = set(y, "F_Y_MIN_ATTENUATION");
    let hi = set(y, "F_Y_MAX_ATTENUATION");
    let attenuation = match (lo, hi) {
        (Some(a), Some(b)) => Some(f64::midpoint(a, b)),
        (a, b) => a.or(b),
    };
    let form = match index(y, "F_Y_FORM").unwrap_or(0) {
        1 => "dry",
        2 => "slant",
        3 => "culture",
        _ => "liquid",
    };
    Yeast {
        name: y.text_of("F_Y_NAME").unwrap_or("Yeast").to_owned(),
        form: Some(form.to_owned()),
        attenuation_pct: attenuation,
        packs: y.number_of("F_Y_AMOUNT").or(Some(1.0)),
    }
}

fn parse_mash_step(s: &Element) -> MashStep {
    let kind = match index(s, "F_MS_TYPE").unwrap_or(0) {
        1 => "decoction",
        2 => "temperature",
        _ => "infusion",
    };
    MashStep {
        name: s.text_of("F_MS_NAME").map(str::to_owned),
        kind: kind.to_owned(),
        temp_c: s.number_of("F_MS_STEP_TEMP").map_or(67.0, f_to_c),
        time_min: s.number_of("F_MS_STEP_TIME").unwrap_or(60.0),
        infusion_l: fl_oz_to_l(s, "F_MS_INFUSION").filter(|v| *v > 0.0),
    }
}

fn parse_recipe(r: &Element) -> Recipe {
    let mut warnings = Vec::new();
    let equipment = r.child("F_R_EQUIPMENT").map(parse_equipment);
    let fermentables: Vec<Fermentable> = data_children(r, "Ingredients", "Grain")
        .into_iter()
        .map(|g| parse_fermentable(g, &mut warnings))
        .collect();
    let hops = data_children(r, "Ingredients", "Hops")
        .into_iter()
        .map(parse_hop)
        .collect();
    let yeasts = data_children(r, "Ingredients", "Yeast")
        .into_iter()
        .map(parse_yeast)
        .collect();
    let misc_count = data_children(r, "Ingredients", "Misc").len();
    if misc_count > 0 {
        warnings.push(format!("{misc_count} misc lines were not imported"));
    }
    let mash_steps = r
        .child("F_R_MASH")
        .map(|m| data_children(m, "steps", "MashStep"))
        .unwrap_or_default()
        .into_iter()
        .map(parse_mash_step)
        .collect();
    let kind = match index(r, "F_R_TYPE").unwrap_or(2) {
        0 => "extract",
        1 => "partial_mash",
        _ => "all_grain",
    };
    let og = set(r, "F_R_DESIRED_OG");
    let valid_og = og.filter(|v| (0.9..=1.3).contains(v));
    if og.is_some() && valid_og.is_none() {
        warnings.push("OG in the file is outside 0.9 to 1.3 and was ignored".to_owned());
    }
    let batch = equipment
        .as_ref()
        .and_then(|e| e.batch_volume_l)
        .or_else(|| fl_oz_to_l(r, "F_R_FINAL_VOL_MEASURED").filter(|v| *v > 0.0));
    Recipe {
        name: r
            .text_of("F_R_NAME")
            .unwrap_or("Imported recipe")
            .to_owned(),
        notes: r.text_of("F_R_NOTES").map(str::to_owned),
        kind: kind.to_owned(),
        style: r
            .child("F_R_STYLE")
            .and_then(|s| s.text_of("F_S_NAME"))
            .map(str::to_owned),
        ibu_method: None,
        batch_volume_l: batch,
        boil_volume_l: equipment.as_ref().and_then(|e| e.boil_volume_l),
        boil_time_min: equipment.as_ref().and_then(|e| e.boil_time_min),
        efficiency_pct: equipment.as_ref().and_then(|e| e.efficiency_pct),
        source: SourceValues {
            og_sg: valid_og,
            fg_sg: None,
            ibu: set(r, "F_R_DESIRED_IBU"),
            srm: set(r, "F_R_DESIRED_COLOR"),
            abv_pct: None,
        },
        equipment,
        fermentables,
        hops,
        yeasts,
        miscs: Vec::new(),
        mash_steps,
        warnings,
    }
}

/// Parses a `.bsmx` file (recipes and equipment records).
pub fn parse(input: &[u8]) -> Result<Document> {
    let root = xml::parse_with(input, Limits::default(), Dialect::Html)?;
    if root.name != "Selections" {
        return Err(Error::Format(format!(
            "unexpected root element {}",
            root.name
        )));
    }
    let mut doc = Document::default();
    if let Some(data) = root.child("Data") {
        doc.recipes = data.children_named("Recipe").map(parse_recipe).collect();
        doc.equipment = data
            .children_named("Equipment")
            .map(parse_equipment)
            .collect();
    }
    if doc.recipes.is_empty() && doc.equipment.is_empty() {
        return Err(Error::Format("no recipe or equipment found".into()));
    }
    Ok(doc)
}
