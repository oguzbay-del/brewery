//! `BeerXML` 1.0 reader and writer.
//!
//! Decisions (recorded as warnings when they change a value):
//! - Fermentable `COLOR` is read as SRM (the usual practice of exporters). The spec says Lovebond.
//! - A fermentable that is not a grain gets the usage `boil`; a grain that is not mashed gets
//!   `steep`.
//! - `Aroma` hops become `whirlpool`.

use std::fmt::Write as _;

use crate::{
    Document, Equipment, Error, Recipe, Result,
    model::{Fermentable, Hop, MashStep, Misc, SourceValues, Yeast, kind_from, kind_to_xml},
    xml::{self, Element, Limits, escape},
};

fn grain_type(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "sugar" => "sugar",
        "extract" => "extract",
        "dry extract" => "dry_extract",
        "adjunct" => "adjunct",
        _ => "grain",
    }
}

fn grain_type_xml(t: &str) -> &'static str {
    match t {
        "sugar" | "honey" => "Sugar",
        "extract" => "Extract",
        "dry_extract" => "Dry Extract",
        "grain" => "Grain",
        _ => "Adjunct",
    }
}

fn hop_use(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "dry hop" => "dry_hop",
        "mash" => "mash",
        "first wort" => "first_wort",
        "aroma" => "whirlpool",
        _ => "boil",
    }
}

fn hop_use_xml(t: &str) -> &'static str {
    match t {
        "dry_hop" => "Dry Hop",
        "mash" => "Mash",
        "first_wort" => "First Wort",
        "whirlpool" => "Aroma",
        _ => "Boil",
    }
}

fn misc_use(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "mash" => "mash",
        "primary" => "primary",
        "secondary" => "secondary",
        "bottling" => "bottling",
        "sparge" => "sparge",
        _ => "boil",
    }
}

fn misc_use_xml(t: &str) -> &'static str {
    match t {
        "mash" => "Mash",
        "primary" => "Primary",
        "secondary" => "Secondary",
        "bottling" => "Bottling",
        "sparge" => "Sparge",
        _ => "Boil",
    }
}

fn yeast_form(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "dry" => "dry",
        "slant" => "slant",
        "culture" => "culture",
        _ => "liquid",
    }
}

fn mash_kind(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "decoction" => "decoction",
        "temperature" => "temperature",
        _ => "infusion",
    }
}

fn parse_equipment(e: &Element) -> Equipment {
    Equipment {
        name: e.text_of("NAME").unwrap_or("Imported equipment").to_owned(),
        batch_volume_l: e.number_of("BATCH_SIZE"),
        boil_volume_l: e.number_of("BOIL_SIZE"),
        boil_time_min: e.number_of("BOIL_TIME"),
        efficiency_pct: e.number_of("EFFICIENCY"),
        trub_loss_l: e.number_of("TRUB_CHILLER_LOSS"),
        fermenter_loss_l: None,
        boil_off_l_per_h: e.number_of("EVAP_RATE").zip(e.number_of("BOIL_SIZE")).map(
            // BeerXML gives the evaporation as percent of the boil volume per hour.
            |(pct, boil)| pct / 100.0 * boil,
        ),
    }
}

fn parse_recipe(r: &Element) -> Recipe {
    let mut warnings = Vec::new();
    let mut fermentables = Vec::new();
    for f in r
        .child("FERMENTABLES")
        .into_iter()
        .flat_map(|c| c.children_named("FERMENTABLE"))
    {
        let gt = grain_type(f.text_of("TYPE").unwrap_or("Grain"));
        let usage = if gt != "grain" {
            "boil"
        } else if f.bool_of("RECOMMEND_MASH") == Some(false) {
            "steep"
        } else {
            "mash"
        };
        fermentables.push(Fermentable {
            name: f.text_of("NAME").unwrap_or("Fermentable").to_owned(),
            amount_kg: f.number_of("AMOUNT").unwrap_or(0.0),
            usage: Some(usage.to_owned()),
            grain_type: Some(gt.to_owned()),
            yield_pct: f.number_of("YIELD"),
            color_srm: f.number_of("COLOR"),
        });
    }
    let hops = r
        .child("HOPS")
        .into_iter()
        .flat_map(|c| c.children_named("HOP"))
        .map(|h| Hop {
            name: h.text_of("NAME").unwrap_or("Hop").to_owned(),
            amount_kg: h.number_of("AMOUNT").unwrap_or(0.0),
            alpha_pct: h.number_of("ALPHA"),
            form: h.text_of("FORM").map(str::to_ascii_lowercase),
            usage: Some(hop_use(h.text_of("USE").unwrap_or("Boil")).to_owned()),
            time_min: h.number_of("TIME"),
            whirlpool_temp_c: None,
        })
        .collect();
    let yeasts = r
        .child("YEASTS")
        .into_iter()
        .flat_map(|c| c.children_named("YEAST"))
        .map(|y| Yeast {
            name: y.text_of("NAME").unwrap_or("Yeast").to_owned(),
            form: Some(yeast_form(y.text_of("FORM").unwrap_or("Liquid")).to_owned()),
            attenuation_pct: y.number_of("ATTENUATION"),
            packs: Some(1.0),
        })
        .collect();
    let miscs = r
        .child("MISCS")
        .into_iter()
        .flat_map(|c| c.children_named("MISC"))
        .map(|m| Misc {
            name: m.text_of("NAME").unwrap_or("Misc").to_owned(),
            amount: m.number_of("AMOUNT").unwrap_or(0.0),
            is_liquid: Some(m.bool_of("AMOUNT_IS_WEIGHT") == Some(false)),
            usage: Some(misc_use(m.text_of("USE").unwrap_or("Boil")).to_owned()),
            time_min: m.number_of("TIME"),
        })
        .collect();
    let mash_steps = r
        .child("MASH")
        .and_then(|m| m.child("MASH_STEPS"))
        .into_iter()
        .flat_map(|s| s.children_named("MASH_STEP"))
        .map(|s| MashStep {
            name: s.text_of("NAME").map(str::to_owned),
            kind: mash_kind(s.text_of("TYPE").unwrap_or("Infusion")).to_owned(),
            temp_c: s.number_of("STEP_TEMP").unwrap_or(67.0),
            time_min: s.number_of("STEP_TIME").unwrap_or(60.0),
            infusion_l: s.number_of("INFUSE_AMOUNT"),
        })
        .collect();
    let og = r.number_of("OG");
    if og.is_some_and(|v| !(0.9..=1.3).contains(&v)) {
        warnings.push("OG in the file is outside 0.9 to 1.3 and was ignored".to_owned());
    }
    let valid_og = og.filter(|v| (0.9..=1.3).contains(v));
    let kind = kind_from(r.text_of("TYPE").unwrap_or("All Grain"));
    let equipment = r.child("EQUIPMENT").map(parse_equipment);
    Recipe {
        name: r.text_of("NAME").unwrap_or("Imported recipe").to_owned(),
        notes: r.text_of("NOTES").map(str::to_owned),
        kind,
        style: r
            .child("STYLE")
            .and_then(|s| s.text_of("NAME"))
            .map(str::to_owned),
        ibu_method: r.text_of("IBU_METHOD").map(str::to_ascii_lowercase),
        batch_volume_l: r.number_of("BATCH_SIZE"),
        boil_volume_l: r.number_of("BOIL_SIZE"),
        boil_time_min: r.number_of("BOIL_TIME"),
        efficiency_pct: r.number_of("EFFICIENCY"),
        source: SourceValues {
            og_sg: valid_og,
            fg_sg: r.number_of("FG"),
            ibu: r.number_of("IBU").or_else(|| r.number_of("EST_IBU")),
            srm: r
                .number_of("EST_COLOR")
                .map(|c| c.max(0.0))
                .or_else(|| r.number_of("COLOR")),
            abv_pct: r.number_of("ABV").or_else(|| r.number_of("EST_ABV")),
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

/// Parses a `BeerXML` file (recipes and equipment records).
pub fn parse(input: &[u8]) -> Result<Document> {
    let root = xml::parse(input, Limits::default())?;
    let mut doc = Document::default();
    match root.name.as_str() {
        "RECIPES" => doc.recipes = root.children_named("RECIPE").map(parse_recipe).collect(),
        "RECIPE" => doc.recipes.push(parse_recipe(&root)),
        "EQUIPMENTS" => {
            doc.equipment = root
                .children_named("EQUIPMENT")
                .map(parse_equipment)
                .collect();
        }
        other => return Err(Error::Format(format!("unexpected root element {other}"))),
    }
    if doc.recipes.is_empty() && doc.equipment.is_empty() {
        return Err(Error::Format("no recipe or equipment found".into()));
    }
    Ok(doc)
}

fn tag(out: &mut String, indent: usize, name: &str, text: &str) {
    let _ = writeln!(
        out,
        "{:indent$}<{name}>{}</{name}>",
        "",
        escape(text),
        indent = indent
    );
}

fn num(out: &mut String, indent: usize, name: &str, v: Option<f64>) {
    if let Some(v) = v {
        tag(out, indent, name, &format!("{v}"));
    }
}

fn write_equipment(out: &mut String, indent: usize, e: &Equipment) {
    let pad = " ".repeat(indent);
    let _ = writeln!(out, "{pad}<EQUIPMENT>");
    let i = indent + 2;
    tag(out, i, "VERSION", "1");
    tag(out, i, "NAME", &e.name);
    num(out, i, "BOIL_SIZE", e.boil_volume_l);
    num(out, i, "BATCH_SIZE", e.batch_volume_l);
    num(out, i, "BOIL_TIME", e.boil_time_min);
    num(out, i, "EFFICIENCY", e.efficiency_pct);
    num(out, i, "TRUB_CHILLER_LOSS", e.trub_loss_l);
    if let (Some(off), Some(boil)) = (e.boil_off_l_per_h, e.boil_volume_l)
        && boil > 0.0
    {
        num(out, i, "EVAP_RATE", Some(off / boil * 100.0));
    }
    let _ = writeln!(out, "{pad}</EQUIPMENT>");
}

/// Writes a `BeerXML` 1.0 file.
#[must_use]
pub fn write(recipes: &[Recipe]) -> String {
    let mut o = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<RECIPES>\n");
    for r in recipes {
        o.push_str("  <RECIPE>\n");
        tag(&mut o, 4, "VERSION", "1");
        tag(&mut o, 4, "NAME", &r.name);
        tag(&mut o, 4, "TYPE", kind_to_xml(&r.kind));
        num(&mut o, 4, "BATCH_SIZE", r.batch_volume_l);
        num(&mut o, 4, "BOIL_SIZE", r.boil_volume_l);
        num(&mut o, 4, "BOIL_TIME", r.boil_time_min);
        num(&mut o, 4, "EFFICIENCY", r.efficiency_pct);
        if let Some(m) = &r.ibu_method {
            tag(&mut o, 4, "IBU_METHOD", m);
        }
        if let Some(n) = &r.notes {
            tag(&mut o, 4, "NOTES", n);
        }
        num(&mut o, 4, "OG", r.source.og_sg);
        num(&mut o, 4, "FG", r.source.fg_sg);
        num(&mut o, 4, "IBU", r.source.ibu);
        num(&mut o, 4, "EST_COLOR", r.source.srm);
        num(&mut o, 4, "ABV", r.source.abv_pct);
        if let Some(s) = &r.style {
            o.push_str("    <STYLE>\n");
            tag(&mut o, 6, "VERSION", "1");
            tag(&mut o, 6, "NAME", s);
            o.push_str("    </STYLE>\n");
        }
        if let Some(e) = &r.equipment {
            write_equipment(&mut o, 4, e);
        }
        o.push_str("    <HOPS>\n");
        for h in &r.hops {
            o.push_str("      <HOP>\n");
            tag(&mut o, 8, "VERSION", "1");
            tag(&mut o, 8, "NAME", &h.name);
            num(&mut o, 8, "ALPHA", h.alpha_pct);
            tag(&mut o, 8, "AMOUNT", &format!("{}", h.amount_kg));
            tag(
                &mut o,
                8,
                "USE",
                hop_use_xml(h.usage.as_deref().unwrap_or("boil")),
            );
            num(&mut o, 8, "TIME", h.time_min);
            if let Some(f) = &h.form {
                let mut f = f.clone();
                if let Some(c) = f.get_mut(0..1) {
                    c.make_ascii_uppercase();
                }
                tag(&mut o, 8, "FORM", &f);
            }
            o.push_str("      </HOP>\n");
        }
        o.push_str("    </HOPS>\n    <FERMENTABLES>\n");
        for f in &r.fermentables {
            o.push_str("      <FERMENTABLE>\n");
            tag(&mut o, 8, "VERSION", "1");
            tag(&mut o, 8, "NAME", &f.name);
            tag(
                &mut o,
                8,
                "TYPE",
                grain_type_xml(f.grain_type.as_deref().unwrap_or("grain")),
            );
            tag(&mut o, 8, "AMOUNT", &format!("{}", f.amount_kg));
            num(&mut o, 8, "YIELD", f.yield_pct);
            num(&mut o, 8, "COLOR", f.color_srm);
            if f.usage.as_deref() == Some("steep") {
                tag(&mut o, 8, "RECOMMEND_MASH", "FALSE");
            }
            o.push_str("      </FERMENTABLE>\n");
        }
        o.push_str("    </FERMENTABLES>\n    <MISCS>\n");
        for m in &r.miscs {
            o.push_str("      <MISC>\n");
            tag(&mut o, 8, "VERSION", "1");
            tag(&mut o, 8, "NAME", &m.name);
            tag(&mut o, 8, "TYPE", "Other");
            tag(
                &mut o,
                8,
                "USE",
                misc_use_xml(m.usage.as_deref().unwrap_or("boil")),
            );
            num(&mut o, 8, "TIME", m.time_min);
            tag(&mut o, 8, "AMOUNT", &format!("{}", m.amount));
            tag(
                &mut o,
                8,
                "AMOUNT_IS_WEIGHT",
                if m.is_liquid == Some(true) {
                    "FALSE"
                } else {
                    "TRUE"
                },
            );
            o.push_str("      </MISC>\n");
        }
        o.push_str("    </MISCS>\n    <YEASTS>\n");
        for y in &r.yeasts {
            o.push_str("      <YEAST>\n");
            tag(&mut o, 8, "VERSION", "1");
            tag(&mut o, 8, "NAME", &y.name);
            tag(&mut o, 8, "TYPE", "Ale");
            let form = match y.form.as_deref() {
                Some("dry") => "Dry",
                Some("slant") => "Slant",
                Some("culture") => "Culture",
                _ => "Liquid",
            };
            tag(&mut o, 8, "FORM", form);
            num(&mut o, 8, "ATTENUATION", y.attenuation_pct);
            o.push_str("      </YEAST>\n");
        }
        o.push_str("    </YEASTS>\n");
        if !r.mash_steps.is_empty() {
            o.push_str("    <MASH>\n      <VERSION>1</VERSION>\n      <NAME>Mash</NAME>\n      <MASH_STEPS>\n");
            for s in &r.mash_steps {
                o.push_str("        <MASH_STEP>\n");
                tag(&mut o, 10, "NAME", s.name.as_deref().unwrap_or("Step"));
                let kind = match s.kind.as_str() {
                    "decoction" => "Decoction",
                    "temperature" => "Temperature",
                    _ => "Infusion",
                };
                tag(&mut o, 10, "TYPE", kind);
                num(&mut o, 10, "INFUSE_AMOUNT", s.infusion_l);
                tag(&mut o, 10, "STEP_TEMP", &format!("{}", s.temp_c));
                tag(&mut o, 10, "STEP_TIME", &format!("{}", s.time_min));
                o.push_str("        </MASH_STEP>\n");
            }
            o.push_str("      </MASH_STEPS>\n    </MASH>\n");
        }
        o.push_str("  </RECIPE>\n");
    }
    o.push_str("</RECIPES>\n");
    o
}

/// Writes equipment records as a `BeerXML` file.
#[must_use]
pub fn write_equipment_file(items: &[Equipment]) -> String {
    let mut o = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<EQUIPMENTS>\n");
    for e in items {
        write_equipment(&mut o, 2, e);
    }
    o.push_str("</EQUIPMENTS>\n");
    o
}
