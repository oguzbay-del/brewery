//! Reader of the Brewfather JSON backup (recipes and equipment).
//!
//! The reader accepts three shapes: one recipe object, an array of recipes, or an object with a
//! `recipes` array and an optional `equipment` array.
//!
//! Assumptions about the format (to be verified with a real export file; a user file has not
//! been available): hop amounts are grams; fermentable and misc amounts are kg (misc may have a
//! `unit` of `g`, `kg`, `ml`, `l`, `pkg`, `tsp`, `tbsp`); fermentable `potential` is a specific
//! gravity (1.037 is 80 percent yield); `color` is SRM; recipe `og` and `fg` are SG.

use serde_json::Value;

use crate::{
    Document, Equipment, Error, Recipe, Result,
    model::{Fermentable, Hop, MashStep, Misc, SourceValues, Yeast, kind_from},
};

/// Extract yield of pure sucrose, points per pound per gallon.
const SUCROSE_POINTS: f64 = 46.21;

fn n(v: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|k| v.get(*k))
        .and_then(|x| {
            x.as_f64()
                .or_else(|| x.as_str().and_then(|s| s.trim().parse().ok()))
        })
        .filter(|x| x.is_finite())
}

fn s<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key)?.as_str().filter(|t| !t.is_empty())
}

fn parse_equipment(e: &Value) -> Equipment {
    let boil = n(e, &["boilSize"]);
    Equipment {
        name: s(e, "name").unwrap_or("Imported equipment").to_owned(),
        batch_volume_l: n(e, &["batchSize"]),
        boil_volume_l: boil,
        boil_time_min: n(e, &["boilTime"]),
        efficiency_pct: n(e, &["efficiency"]),
        trub_loss_l: n(e, &["trubChillerLoss"]),
        fermenter_loss_l: n(e, &["fermenterLoss"]),
        // The evaporation rate is a percent of the boil volume per hour.
        boil_off_l_per_h: n(e, &["evaporationRate"])
            .zip(boil)
            .map(|(p, b)| p / 100.0 * b),
    }
}

fn grain_type(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "sugar" => "sugar",
        "extract" => "extract",
        "dry extract" => "dry_extract",
        "adjunct" => "adjunct",
        "fruit" => "fruit",
        "juice" => "juice",
        "honey" => "honey",
        _ => "grain",
    }
}

fn hop_use(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "dry hop" => "dry_hop",
        "mash" => "mash",
        "first wort" => "first_wort",
        "aroma" | "whirlpool" | "flame out" => "whirlpool",
        _ => "boil",
    }
}

fn misc_use(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "mash" => "mash",
        "primary" => "primary",
        "secondary" => "secondary",
        "bottling" | "keg" => "bottling",
        "sparge" => "sparge",
        _ => "boil",
    }
}

/// Amount of a misc in kg or L and whether it is a liquid.
fn misc_amount(m: &Value, warn: &mut Vec<String>) -> (f64, bool) {
    let amount = n(m, &["amount"]).unwrap_or(0.0);
    match s(m, "unit").unwrap_or("g").to_ascii_lowercase().as_str() {
        "kg" => (amount, false),
        "ml" => (amount / 1000.0, true),
        "l" => (amount, true),
        "g" => (amount / 1000.0, false),
        other => {
            warn.push(format!(
                "misc unit {other} cannot be converted; amount read as grams"
            ));
            (amount / 1000.0, false)
        }
    }
}

fn parse_recipe(r: &Value) -> Recipe {
    let mut warnings = Vec::new();
    let list = |key: &str| -> Vec<Value> {
        r.get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let fermentables = list("fermentables")
        .iter()
        .map(|f| {
            let gt = grain_type(s(f, "type").unwrap_or("Grain"));
            Fermentable {
                name: s(f, "name").unwrap_or("Fermentable").to_owned(),
                amount_kg: n(f, &["amount"]).unwrap_or(0.0),
                usage: Some(if gt == "grain" { "mash" } else { "boil" }.to_owned()),
                grain_type: Some(gt.to_owned()),
                yield_pct: n(f, &["potential"])
                    .filter(|p| *p > 1.0)
                    .map(|p| ((p - 1.0) * 1000.0 / SUCROSE_POINTS * 100.0).clamp(0.0, 100.0))
                    .or_else(|| n(f, &["yield"])),
                color_srm: n(f, &["color"]),
            }
        })
        .collect();
    let hops = list("hops")
        .iter()
        .map(|h| {
            let usage = hop_use(s(h, "use").unwrap_or("Boil"));
            Hop {
                name: s(h, "name").unwrap_or("Hop").to_owned(),
                amount_kg: n(h, &["amount"]).unwrap_or(0.0) / 1000.0,
                alpha_pct: n(h, &["alpha"]),
                form: s(h, "type").map(str::to_ascii_lowercase),
                usage: Some(usage.to_owned()),
                // Brewfather gives the dry hop time in minutes, as for a boil hop.
                time_min: n(h, &["time"]),
                whirlpool_temp_c: if usage == "whirlpool" {
                    n(h, &["temp"])
                } else {
                    None
                },
            }
        })
        .collect();
    let yeasts = list("yeasts")
        .iter()
        .map(|y| Yeast {
            name: s(y, "name").unwrap_or("Yeast").to_owned(),
            form: Some(
                match s(y, "form").unwrap_or("").to_ascii_lowercase().as_str() {
                    "dry" => "dry",
                    "slant" => "slant",
                    "culture" => "culture",
                    _ => "liquid",
                }
                .to_owned(),
            ),
            attenuation_pct: n(y, &["attenuation"]),
            amount: None,
            amount_is_weight: None,
            packs: Some(n(y, &["amount"]).filter(|a| *a > 0.0).unwrap_or(1.0)),
        })
        .collect();
    let miscs = list("miscs")
        .iter()
        .map(|m| {
            let (amount, liquid) = misc_amount(m, &mut warnings);
            Misc {
                name: s(m, "name").unwrap_or("Misc").to_owned(),
                amount,
                is_liquid: Some(liquid),
                usage: Some(misc_use(s(m, "use").unwrap_or("Boil")).to_owned()),
                time_min: n(m, &["time"]),
            }
        })
        .collect();
    let mash_steps = r
        .get("mash")
        .and_then(|m| m.get("steps"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|st| MashStep {
            name: s(st, "name").map(str::to_owned),
            kind: match s(st, "type").unwrap_or("").to_ascii_lowercase().as_str() {
                "decoction" => "decoction",
                "temperature" => "temperature",
                _ => "infusion",
            }
            .to_owned(),
            temp_c: n(st, &["stepTemp"]).unwrap_or(67.0),
            time_min: n(st, &["stepTime"]).unwrap_or(60.0),
            infusion_l: n(st, &["infuseAmount"]),
        })
        .collect();
    let equipment = r
        .get("equipment")
        .filter(|e| e.is_object())
        .map(parse_equipment);
    Recipe {
        name: s(r, "name").unwrap_or("Imported recipe").to_owned(),
        notes: s(r, "notes").map(str::to_owned),
        kind: kind_from(s(r, "type").unwrap_or("All Grain")),
        style: r.get("style").and_then(|x| s(x, "name")).map(str::to_owned),
        brewer: None,
        style_details: None,
        ibu_method: s(r, "ibuFormula").map(str::to_ascii_lowercase),
        batch_volume_l: n(r, &["batchSize"]),
        boil_volume_l: n(r, &["boilSize"]),
        boil_time_min: n(r, &["boilTime"]),
        efficiency_pct: n(r, &["efficiency"]),
        source: SourceValues {
            og_sg: n(r, &["og"]),
            fg_sg: n(r, &["fg"]),
            ibu: n(r, &["ibu"]),
            srm: n(r, &["color"]),
            abv_pct: n(r, &["abv"]),
        },
        equipment,
        fermentables,
        hops,
        yeasts,
        miscs,
        mash_steps,
        warnings,
    }
}

/// Parses a Brewfather JSON file.
pub fn parse(input: &[u8]) -> Result<Document> {
    let root = crate::parse_json(input)?;
    let mut doc = Document::default();
    match &root {
        Value::Array(items) => doc.recipes = items.iter().map(parse_recipe).collect(),
        Value::Object(o) if o.contains_key("recipes") || o.contains_key("equipment") => {
            if let Some(list) = o.get("recipes").and_then(Value::as_array) {
                doc.recipes = list.iter().map(parse_recipe).collect();
            }
            match o.get("equipment") {
                Some(Value::Array(list)) => {
                    doc.equipment = list.iter().map(parse_equipment).collect();
                }
                Some(e @ Value::Object(_)) => doc.equipment = vec![parse_equipment(e)],
                _ => {}
            }
        }
        Value::Object(o) if o.contains_key("fermentables") || o.contains_key("batchSize") => {
            doc.recipes.push(parse_recipe(&root));
        }
        _ => return Err(Error::Format("not a Brewfather recipe file".into())),
    }
    if doc.recipes.is_empty() && doc.equipment.is_empty() {
        return Err(Error::Format("no recipe or equipment found".into()));
    }
    Ok(doc)
}
