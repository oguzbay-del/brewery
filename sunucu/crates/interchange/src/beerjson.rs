//! `BeerJSON` 1.0 reader and writer (the subset the app keeps).
//!
//! Quantities are `{value, unit}` objects. The reader converts mass, volume, temperature and
//! time units to the internal units and rejects an unknown unit with a warning (the value is
//! then dropped). The writer uses `kg`, `l`, `C`, `min`, `sg`, `SRM` and `%`.
//!
//! Lossy mappings:
//! - `BeerJSON` has no first wort or whirlpool hop use. The writer maps a first wort hop to
//!   `add_to_boil` with the boil time as duration, and a whirlpool hop to `add_to_boil` with a
//!   duration and a `temperature`. The reader reads a hop with a temperature below 100 degrees
//!   C as a whirlpool hop. A first wort hop returns as a boil hop.

use serde_json::{Map, Value, json};

use crate::{
    Document, Equipment, Error, Recipe, Result,
    model::{Fermentable, Hop, MashStep, Misc, SourceValues, Yeast, kind_from},
};

fn mass_kg(v: &Value, warn: &mut Vec<String>) -> Option<f64> {
    let value = v.get("value")?.as_f64()?;
    let unit = v.get("unit")?.as_str()?.to_ascii_lowercase();
    match unit.as_str() {
        "kg" => Some(value),
        "g" => Some(value / 1000.0),
        "mg" => Some(value / 1_000_000.0),
        "lb" => Some(value * 0.453_592_37),
        "oz" => Some(value * 0.028_349_523_125),
        _ => {
            warn.push(format!("mass unit {unit} is not known; value dropped"));
            None
        }
    }
}

fn volume_l(v: &Value, warn: &mut Vec<String>) -> Option<f64> {
    let value = v.get("value")?.as_f64()?;
    let unit = v.get("unit")?.as_str()?.to_ascii_lowercase();
    match unit.as_str() {
        "l" => Some(value),
        "ml" => Some(value / 1000.0),
        "gal" => Some(value * 3.785_411_784),
        "qt" => Some(value * 0.946_352_946),
        "floz" => Some(value * 0.029_573_529_562_5),
        _ => {
            warn.push(format!("volume unit {unit} is not known; value dropped"));
            None
        }
    }
}

fn time_min(v: &Value) -> Option<f64> {
    let value = v.get("value")?.as_f64()?;
    match v.get("unit")?.as_str()?.to_ascii_lowercase().as_str() {
        "min" => Some(value),
        "sec" => Some(value / 60.0),
        "hr" => Some(value * 60.0),
        "day" => Some(value * 1440.0),
        "week" => Some(value * 10080.0),
        _ => None,
    }
}

fn temp_c(v: &Value) -> Option<f64> {
    let value = v.get("value")?.as_f64()?;
    match v.get("unit")?.as_str()?.to_ascii_lowercase().as_str() {
        "c" => Some(value),
        "f" => Some((value - 32.0) / 1.8),
        _ => None,
    }
}

fn plain(v: Option<&Value>) -> Option<f64> {
    v?.get("value")?.as_f64()
}

fn color_srm(v: Option<&Value>) -> Option<f64> {
    let v = v?;
    let value = v.get("value")?.as_f64()?;
    match v.get("unit")?.as_str()?.to_ascii_lowercase().as_str() {
        "srm" | "l" => Some(value),
        "ebc" => Some(value / 1.97),
        _ => None,
    }
}

fn str_of<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key)?.as_str()
}

fn parse_fermentable(f: &Value, warn: &mut Vec<String>) -> Fermentable {
    let kind = match str_of(f, "type").unwrap_or("grain") {
        "sugar" => "sugar",
        "extract" => "extract",
        "dry extract" => "dry_extract",
        "fruit" => "fruit",
        "juice" => "juice",
        "honey" => "honey",
        "other" => "adjunct",
        _ => "grain",
    };
    let usage = match f
        .get("timing")
        .and_then(|t| str_of(t, "use"))
        .unwrap_or("add_to_mash")
    {
        "add_to_boil" => "boil",
        "add_to_fermentation" => "primary",
        "add_to_package" => "bottling",
        _ if kind == "grain" => "mash",
        _ => "mash",
    };
    let yield_pct = f
        .get("yield")
        .and_then(|y| plain(y.get("fine_grind")).or_else(|| plain(y.get("coarse_grind"))));
    Fermentable {
        name: str_of(f, "name").unwrap_or("Fermentable").to_owned(),
        amount_kg: f
            .get("amount")
            .and_then(|a| mass_kg(a, warn))
            .unwrap_or(0.0),
        usage: Some(usage.to_owned()),
        grain_type: Some(kind.to_owned()),
        yield_pct,
        color_srm: color_srm(f.get("color")),
    }
}

fn parse_hop(h: &Value, boil_time: Option<f64>, warn: &mut Vec<String>) -> Hop {
    let timing = h.get("timing");
    let use_ = timing
        .and_then(|t| str_of(t, "use"))
        .unwrap_or("add_to_boil");
    let duration = timing.and_then(|t| t.get("duration")).and_then(time_min);
    let temp = timing.and_then(|t| t.get("temperature")).and_then(temp_c);
    let (usage, time) = match use_ {
        "add_to_mash" => ("mash", duration),
        "add_to_fermentation" | "add_to_package" => ("dry_hop", duration),
        _ if temp.is_some_and(|t| t < 100.0) => ("whirlpool", duration),
        _ => ("boil", duration.or(boil_time)),
    };
    Hop {
        name: str_of(h, "name").unwrap_or("Hop").to_owned(),
        amount_kg: h
            .get("amount")
            .and_then(|a| mass_kg(a, warn))
            .unwrap_or(0.0),
        alpha_pct: plain(h.get("alpha_acid")),
        form: str_of(h, "form").map(str::to_owned),
        usage: Some(usage.to_owned()),
        time_min: time,
        whirlpool_temp_c: if usage == "whirlpool" { temp } else { None },
    }
}

fn parse_equipment(e: &Value, warn: &mut Vec<String>) -> Equipment {
    let vol = |key: &str, w: &mut Vec<String>| e.get(key).and_then(|v| volume_l(v, w));
    Equipment {
        name: str_of(e, "name").unwrap_or("Imported equipment").to_owned(),
        batch_volume_l: None,
        boil_volume_l: None,
        boil_time_min: None,
        efficiency_pct: plain(e.get("brewhouse_efficiency")),
        trub_loss_l: vol("trub_chiller_loss", warn),
        fermenter_loss_l: vol("fermenter_loss", warn),
        boil_off_l_per_h: vol("boil_rate_per_hour", warn),
    }
}

fn parse_recipe(r: &Value) -> Recipe {
    let mut warnings = Vec::new();
    let w = &mut warnings;
    let ing = r.get("ingredients").cloned().unwrap_or(Value::Null);
    let arr = |key: &str| -> Vec<Value> {
        ing.get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let boil_time = r
        .get("boil")
        .and_then(|b| b.get("boil_time"))
        .and_then(time_min);
    let fermentables = arr("fermentable_additions")
        .iter()
        .map(|f| parse_fermentable(f, w))
        .collect();
    let hops = arr("hop_additions")
        .iter()
        .map(|h| parse_hop(h, boil_time, w))
        .collect();
    let yeasts = arr("culture_additions")
        .iter()
        .map(|y| Yeast {
            name: str_of(y, "name").unwrap_or("Yeast").to_owned(),
            form: Some(
                match str_of(y, "form").unwrap_or("liquid") {
                    "dry" => "dry",
                    "slant" => "slant",
                    "culture" => "culture",
                    _ => "liquid",
                }
                .to_owned(),
            ),
            attenuation_pct: y
                .get("attenuation")
                .and_then(|a| plain(a.get("minimum")).or_else(|| plain(Some(a)))),
            amount: None,
            amount_is_weight: None,
            packs: Some(1.0),
        })
        .collect();
    let miscs = arr("miscellaneous_additions")
        .iter()
        .map(|m| {
            let amount = m.get("amount");
            let liquid = amount
                .and_then(|a| str_of(a, "unit"))
                .is_some_and(|u| matches!(u, "l" | "ml" | "gal" | "qt"));
            let value = amount.and_then(|a| {
                if liquid {
                    volume_l(a, w)
                } else {
                    mass_kg(a, w)
                }
            });
            let timing = m.get("timing");
            let usage = match timing
                .and_then(|t| str_of(t, "use"))
                .unwrap_or("add_to_boil")
            {
                "add_to_mash" => "mash",
                "add_to_fermentation" => "primary",
                "add_to_package" => "bottling",
                _ => "boil",
            };
            Misc {
                name: str_of(m, "name").unwrap_or("Misc").to_owned(),
                amount: value.unwrap_or(0.0),
                is_liquid: Some(liquid),
                usage: Some(usage.to_owned()),
                time_min: timing.and_then(|t| t.get("duration")).and_then(time_min),
            }
        })
        .collect();
    let mash_steps = r
        .get("mash")
        .and_then(|m| m.get("mash_steps"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|s| MashStep {
            name: str_of(s, "name").map(str::to_owned),
            kind: match str_of(s, "type").unwrap_or("infusion") {
                "decoction" => "decoction",
                "temperature" => "temperature",
                _ => "infusion",
            }
            .to_owned(),
            temp_c: s.get("step_temperature").and_then(temp_c).unwrap_or(67.0),
            time_min: s.get("step_time").and_then(time_min).unwrap_or(60.0),
            infusion_l: s.get("amount").and_then(|a| volume_l(a, &mut Vec::new())),
        })
        .collect();
    let batch = r.get("batch_size").and_then(|v| volume_l(v, w));
    let boil_volume = r
        .get("boil")
        .and_then(|b| b.get("pre_boil_size"))
        .and_then(|v| volume_l(v, w));
    let ibu_method = r
        .get("ibu_estimate")
        .and_then(|i| str_of(i, "method"))
        .map(str::to_ascii_lowercase);
    Recipe {
        name: str_of(r, "name").unwrap_or("Imported recipe").to_owned(),
        notes: str_of(r, "notes").map(str::to_owned),
        kind: kind_from(str_of(r, "type").unwrap_or("all grain")),
        style: r
            .get("style")
            .and_then(|s| str_of(s, "name"))
            .map(str::to_owned),
        brewer: None,
        style_details: None,
        ibu_method,
        batch_volume_l: batch,
        boil_volume_l: boil_volume,
        boil_time_min: boil_time,
        efficiency_pct: r.get("efficiency").and_then(|e| plain(e.get("brewhouse"))),
        source: SourceValues {
            og_sg: plain(r.get("original_gravity")),
            fg_sg: plain(r.get("final_gravity")),
            ibu: r
                .get("ibu_estimate")
                .and_then(|i| i.get("value").and_then(Value::as_f64)),
            srm: color_srm(r.get("color_estimate")),
            abv_pct: plain(r.get("alcohol_by_volume")),
        },
        equipment: r
            .get("equipment")
            .map(|e| parse_equipment(e, &mut Vec::new())),
        fermentables,
        hops,
        yeasts,
        miscs,
        mash_steps,
        warnings,
    }
}

/// Parses a `BeerJSON` file.
pub fn parse(input: &[u8]) -> Result<Document> {
    let root = crate::parse_json(input)?;
    let body = root
        .get("beerjson")
        .ok_or_else(|| Error::Format("no beerjson object".into()))?;
    let mut doc = Document::default();
    if let Some(list) = body.get("recipes").and_then(Value::as_array) {
        doc.recipes = list.iter().map(parse_recipe).collect();
    }
    if let Some(list) = body.get("equipments").and_then(Value::as_array) {
        doc.equipment = list
            .iter()
            .map(|e| parse_equipment(e, &mut doc.warnings))
            .collect();
    }
    if doc.recipes.is_empty() && doc.equipment.is_empty() {
        return Err(Error::Format("no recipe or equipment found".into()));
    }
    Ok(doc)
}

fn q(value: f64, unit: &str) -> Value {
    json!({"value": value, "unit": unit})
}

fn set(m: &mut Map<String, Value>, key: &str, v: Option<f64>, unit: &str) {
    if let Some(v) = v {
        m.insert(key.to_owned(), q(v, unit));
    }
}

fn write_recipe(r: &Recipe) -> Value {
    let mut m = Map::new();
    m.insert("name".into(), json!(r.name));
    m.insert(
        "type".into(),
        json!(match r.kind.as_str() {
            "extract" => "extract",
            "partial_mash" => "partial mash",
            "cider" => "cider",
            "mead" => "mead",
            "wine" => "wine",
            _ => "all grain",
        }),
    );
    m.insert("author".into(), json!("Demleme"));
    if let Some(n) = &r.notes {
        m.insert("notes".into(), json!(n));
    }
    set(&mut m, "batch_size", r.batch_volume_l, "l");
    if let Some(e) = r.efficiency_pct {
        m.insert("efficiency".into(), json!({"brewhouse": q(e, "%")}));
    }
    let mut boil = Map::new();
    set(&mut boil, "pre_boil_size", r.boil_volume_l, "l");
    set(&mut boil, "boil_time", r.boil_time_min, "min");
    if !boil.is_empty() {
        m.insert("boil".into(), Value::Object(boil));
    }
    set(&mut m, "original_gravity", r.source.og_sg, "sg");
    set(&mut m, "final_gravity", r.source.fg_sg, "sg");
    set(&mut m, "color_estimate", r.source.srm, "SRM");
    set(&mut m, "alcohol_by_volume", r.source.abv_pct, "%");
    if let Some(ibu) = r.source.ibu {
        m.insert(
            "ibu_estimate".into(),
            json!({"method": r.ibu_method.clone().unwrap_or_else(|| "tinseth".into()), "value": ibu}),
        );
    }
    if let Some(s) = &r.style {
        m.insert(
            "style".into(),
            json!({"name": s, "category": "", "category_number": 0, "style_letter": "", "style_guide": "", "type": "beer"}),
        );
    }
    let ferms: Vec<Value> = r
        .fermentables
        .iter()
        .map(|f| {
            let mut o = Map::new();
            o.insert("name".into(), json!(f.name));
            o.insert(
                "type".into(),
                json!(match f.grain_type.as_deref().unwrap_or("grain") {
                    "sugar" => "sugar",
                    "extract" => "extract",
                    "dry_extract" => "dry extract",
                    "fruit" => "fruit",
                    "juice" => "juice",
                    "honey" => "honey",
                    "adjunct" => "other",
                    _ => "grain",
                }),
            );
            o.insert("amount".into(), q(f.amount_kg, "kg"));
            if let Some(y) = f.yield_pct {
                o.insert("yield".into(), json!({"fine_grind": q(y, "%")}));
            }
            set(&mut o, "color", f.color_srm, "SRM");
            let use_ = match f.usage.as_deref().unwrap_or("mash") {
                "boil" | "whirlpool" => "add_to_boil",
                "primary" | "secondary" | "tertiary" => "add_to_fermentation",
                "bottling" => "add_to_package",
                _ => "add_to_mash",
            };
            o.insert("timing".into(), json!({"use": use_}));
            Value::Object(o)
        })
        .collect();
    let hops: Vec<Value> = r
        .hops
        .iter()
        .map(|h| {
            let mut o = Map::new();
            o.insert("name".into(), json!(h.name));
            set(&mut o, "alpha_acid", h.alpha_pct, "%");
            o.insert("amount".into(), q(h.amount_kg, "kg"));
            if let Some(f) = &h.form {
                o.insert("form".into(), json!(f));
            }
            let mut t = Map::new();
            let time = h.time_min;
            match h.usage.as_deref().unwrap_or("boil") {
                "mash" => {
                    t.insert("use".into(), json!("add_to_mash"));
                }
                "dry_hop" => {
                    t.insert("use".into(), json!("add_to_fermentation"));
                }
                "whirlpool" => {
                    t.insert("use".into(), json!("add_to_boil"));
                    set(
                        &mut t,
                        "temperature",
                        h.whirlpool_temp_c.or(Some(80.0)),
                        "C",
                    );
                }
                _ => {
                    t.insert("use".into(), json!("add_to_boil"));
                }
            }
            set(&mut t, "duration", time, "min");
            o.insert("timing".into(), Value::Object(t));
            Value::Object(o)
        })
        .collect();
    let cultures: Vec<Value> = r
        .yeasts
        .iter()
        .map(|y| {
            let mut o = Map::new();
            o.insert("name".into(), json!(y.name));
            o.insert("type".into(), json!("ale"));
            o.insert(
                "form".into(),
                json!(y.form.clone().unwrap_or_else(|| "liquid".into())),
            );
            o.insert("amount".into(), q(1.0, "pkg"));
            if let Some(a) = y.attenuation_pct {
                o.insert("attenuation".into(), q(a, "%"));
            }
            Value::Object(o)
        })
        .collect();
    let miscs: Vec<Value> = r
        .miscs
        .iter()
        .map(|x| {
            let mut o = Map::new();
            o.insert("name".into(), json!(x.name));
            o.insert("type".into(), json!("other"));
            let unit = if x.is_liquid == Some(true) { "l" } else { "kg" };
            o.insert("amount".into(), q(x.amount, unit));
            let use_ = match x.usage.as_deref().unwrap_or("boil") {
                "mash" => "add_to_mash",
                "primary" | "secondary" => "add_to_fermentation",
                "bottling" => "add_to_package",
                _ => "add_to_boil",
            };
            let mut t = Map::new();
            t.insert("use".into(), json!(use_));
            set(&mut t, "duration", x.time_min, "min");
            o.insert("timing".into(), Value::Object(t));
            Value::Object(o)
        })
        .collect();
    m.insert(
        "ingredients".into(),
        json!({
            "fermentable_additions": ferms,
            "hop_additions": hops,
            "culture_additions": cultures,
            "miscellaneous_additions": miscs,
        }),
    );
    if !r.mash_steps.is_empty() {
        let steps: Vec<Value> = r
            .mash_steps
            .iter()
            .map(|s| {
                let mut o = Map::new();
                o.insert(
                    "name".into(),
                    json!(s.name.clone().unwrap_or_else(|| "Step".into())),
                );
                o.insert("type".into(), json!(s.kind));
                o.insert("step_temperature".into(), q(s.temp_c, "C"));
                o.insert("step_time".into(), q(s.time_min, "min"));
                set(&mut o, "amount", s.infusion_l, "l");
                Value::Object(o)
            })
            .collect();
        m.insert(
            "mash".into(),
            json!({"name": "Mash", "grain_temperature": q(20.0, "C"), "mash_steps": steps}),
        );
    }
    Value::Object(m)
}

/// Writes a `BeerJSON` 1.0 file.
#[must_use]
pub fn write(recipes: &[Recipe]) -> String {
    let doc = json!({
        "beerjson": {
            "version": 1.0,
            "recipes": recipes.iter().map(write_recipe).collect::<Vec<_>>(),
        }
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}
