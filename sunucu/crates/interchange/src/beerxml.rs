//! `BeerXML` 1.0 reader and writer.
//!
//! Decisions (recorded as warnings when they change a value):
//! - Fermentable `COLOR` is Lovibond, except liquid extract which is SRM; the neutral model stores
//!   SRM. The conversion is documented beside the reader.
//! - A fermentable that is not a grain gets the usage `boil`; a grain that is not mashed gets
//!   `steep`.
//! - `Aroma` hops become `whirlpool`.
//! - `OG` and `FG` are measured values in the specification. They are the source values when the
//!   file has no estimate. `EST_OG` and `EST_FG` (text with a unit, for example `1.052 sg`) win.
//! - `EST_COLOR` is text with a unit. A unit `EBC` is converted to SRM; any other unit is SRM.
//! - Required fields of BeerXML records are checked and errors name the record and field.
//! - Yeast `AMOUNT` and `AMOUNT_IS_WEIGHT` are preserved; they are not interpreted as pack counts.

use std::fmt::Write as _;

use crate::{
    Document, Equipment, Error, Recipe, Result,
    model::{
        BeerStyle, Fermentable, Hop, MashStep, Misc, SourceValues, Yeast, kind_from, kind_to_xml,
    },
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

fn parse_equipment(e: &Element) -> Result<Equipment> {
    let name = required_text(e, "EQUIPMENT", "NAME")?;
    required_text(e, "EQUIPMENT", "VERSION")?;
    let batch_volume_l = required_number(e, "EQUIPMENT", "BATCH_SIZE")?;
    let boil_volume_l = required_number(e, "EQUIPMENT", "BOIL_SIZE")?;
    Ok(Equipment {
        name,
        batch_volume_l: Some(batch_volume_l),
        boil_volume_l: Some(boil_volume_l),
        boil_time_min: e.number_of("BOIL_TIME"),
        efficiency_pct: e.number_of("EFFICIENCY"),
        trub_loss_l: e.number_of("TRUB_CHILLER_LOSS"),
        fermenter_loss_l: None,
        boil_off_l_per_h: e.number_of("EVAP_RATE").zip(e.number_of("BOIL_SIZE")).map(
            // BeerXML gives the evaporation as percent of the boil volume per hour.
            |(pct, boil)| pct / 100.0 * boil,
        ),
    })
}

/// The color of `EST_COLOR` in SRM. The text has a number and an optional unit.
fn est_color_srm(r: &Element) -> Option<f64> {
    let v = r.number_of("EST_COLOR")?.max(0.0);
    let ebc = r
        .text_of("EST_COLOR")
        .and_then(|t| t.split_whitespace().nth(1))
        .is_some_and(|u| u.eq_ignore_ascii_case("ebc"));
    Some(if ebc { v / 1.97 } else { v })
}

fn required_text(element: &Element, record: &str, field: &str) -> Result<String> {
    element
        .text_of(field)
        .map(str::to_owned)
        .ok_or_else(|| Error::Format(format!("{record} missing required field {field}")))
}

fn required_number(element: &Element, record: &str, field: &str) -> Result<f64> {
    element
        .number_of(field)
        .ok_or_else(|| Error::Format(format!("{record} missing required field {field}")))
}

fn required_child<'a>(element: &'a Element, record: &str, field: &str) -> Result<&'a Element> {
    element
        .child(field)
        .ok_or_else(|| Error::Format(format!("{record} missing required field {field}")))
}

fn parse_style(s: &Element) -> Result<BeerStyle> {
    Ok(BeerStyle {
        name: required_text(s, "STYLE", "NAME")?,
        category: required_text(s, "STYLE", "CATEGORY")?,
        category_number: required_text(s, "STYLE", "CATEGORY_NUMBER")?,
        style_letter: required_text(s, "STYLE", "STYLE_LETTER")?,
        style_guide: required_text(s, "STYLE", "STYLE_GUIDE")?,
        kind: required_text(s, "STYLE", "TYPE")?,
        og_min: required_number(s, "STYLE", "OG_MIN")?,
        og_max: required_number(s, "STYLE", "OG_MAX")?,
        fg_min: required_number(s, "STYLE", "FG_MIN")?,
        fg_max: required_number(s, "STYLE", "FG_MAX")?,
        ibu_min: required_number(s, "STYLE", "IBU_MIN")?,
        ibu_max: required_number(s, "STYLE", "IBU_MAX")?,
        color_min: required_number(s, "STYLE", "COLOR_MIN")?,
        color_max: required_number(s, "STYLE", "COLOR_MAX")?,
        carb_min: s.number_of("CARB_MIN"),
        carb_max: s.number_of("CARB_MAX"),
        abv_min: s.number_of("ABV_MIN"),
        abv_max: s.number_of("ABV_MAX"),
        notes: s.text_of("NOTES").map(str::to_owned),
        profile: s.text_of("PROFILE").map(str::to_owned),
        ingredients: s.text_of("INGREDIENTS").map(str::to_owned),
        examples: s.text_of("EXAMPLES").map(str::to_owned),
    })
}

fn parse_recipe(r: &Element) -> Result<Recipe> {
    let name = required_text(r, "RECIPE", "NAME")?;
    required_text(r, "RECIPE", "VERSION")?;
    let recipe_type = required_text(r, "RECIPE", "TYPE")?;
    let brewer = required_text(r, "RECIPE", "BREWER")?;
    let batch_volume_l = required_number(r, "RECIPE", "BATCH_SIZE")?;
    let boil_volume_l = required_number(r, "RECIPE", "BOIL_SIZE")?;
    let boil_time_min = required_number(r, "RECIPE", "BOIL_TIME")?;
    let style_details = parse_style(required_child(r, "RECIPE", "STYLE")?)?;

    let mut warnings = Vec::new();
    let mut fermentables = Vec::new();
    for f in required_child(r, "RECIPE", "FERMENTABLES")?.children_named("FERMENTABLE") {
        let name = required_text(f, "FERMENTABLE", "NAME")?;
        required_text(f, "FERMENTABLE", "VERSION")?;
        let gt = grain_type(&required_text(f, "FERMENTABLE", "TYPE")?);
        let amount_kg = required_number(f, "FERMENTABLE", "AMOUNT")?;
        let yield_pct = required_number(f, "FERMENTABLE", "YIELD")?;
        let color = required_number(f, "FERMENTABLE", "COLOR")?;
        let usage = if gt != "grain" {
            "boil"
        } else if f.bool_of("RECOMMEND_MASH") == Some(false) {
            "steep"
        } else {
            "mash"
        };
        // BeerXML specifies Lovibond for fermentables, except liquid extracts (SRM).
        // Source: Handbook of Alcoholic Beverages, Eq. 4.4.8:
        // https://www.rexresearch1.com/BrewingLibrary/HandbookAlcoholicBeverages.pdf
        // SRM = 1.3546 × Lovibond − 0.76.
        let color_srm = if gt == "extract" {
            color
        } else {
            (1.3546 * color - 0.76).max(0.0)
        };
        fermentables.push(Fermentable {
            name,
            amount_kg,
            usage: Some(usage.to_owned()),
            grain_type: Some(gt.to_owned()),
            yield_pct: Some(yield_pct),
            color_srm: Some(color_srm),
        });
    }

    let mut hops = Vec::new();
    for h in required_child(r, "RECIPE", "HOPS")?.children_named("HOP") {
        hops.push(Hop {
            name: required_text(h, "HOP", "NAME")?,
            amount_kg: required_number(h, "HOP", "AMOUNT")?,
            alpha_pct: Some(required_number(h, "HOP", "ALPHA")?),
            form: h.text_of("FORM").map(str::to_ascii_lowercase),
            usage: Some(hop_use(&required_text(h, "HOP", "USE")?).to_owned()),
            time_min: Some(required_number(h, "HOP", "TIME")?),
            whirlpool_temp_c: None,
        });
        required_text(h, "HOP", "VERSION")?;
    }

    let mut yeasts = Vec::new();
    for y in required_child(r, "RECIPE", "YEASTS")?.children_named("YEAST") {
        let amount = required_number(y, "YEAST", "AMOUNT")?;
        required_text(y, "YEAST", "VERSION")?;
        required_text(y, "YEAST", "TYPE")?;
        yeasts.push(Yeast {
            name: required_text(y, "YEAST", "NAME")?,
            form: Some(yeast_form(&required_text(y, "YEAST", "FORM")?).to_owned()),
            attenuation_pct: y.number_of("ATTENUATION"),
            amount: Some(amount),
            amount_is_weight: Some(y.bool_of("AMOUNT_IS_WEIGHT").unwrap_or(false)),
            packs: None,
        });
    }

    let mut miscs = Vec::new();
    for m in required_child(r, "RECIPE", "MISCS")?.children_named("MISC") {
        required_text(m, "MISC", "VERSION")?;
        let weight = m.bool_of("AMOUNT_IS_WEIGHT").unwrap_or(false);
        miscs.push(Misc {
            name: required_text(m, "MISC", "NAME")?,
            amount: required_number(m, "MISC", "AMOUNT")?,
            is_liquid: Some(!weight),
            usage: Some(misc_use(&required_text(m, "MISC", "USE")?).to_owned()),
            time_min: Some(required_number(m, "MISC", "TIME")?),
        });
        required_text(m, "MISC", "TYPE")?;
    }
    let waters = required_child(r, "RECIPE", "WATERS")?;
    for water in waters.children_named("WATER") {
        required_text(water, "WATER", "NAME")?;
        required_text(water, "WATER", "VERSION")?;
        for field in [
            "AMOUNT",
            "CALCIUM",
            "BICARBONATE",
            "SULFATE",
            "CHLORIDE",
            "SODIUM",
            "MAGNESIUM",
        ] {
            required_number(water, "WATER", field)?;
        }
    }

    let mut mash_steps = Vec::new();
    if let Some(mash) = r.child("MASH") {
        required_text(mash, "MASH", "NAME")?;
        required_text(mash, "MASH", "VERSION")?;
        required_number(mash, "MASH", "GRAIN_TEMP")?;
        let steps = required_child(mash, "MASH", "MASH_STEPS")?;
        for step in steps.children_named("MASH_STEP") {
            required_text(step, "MASH_STEP", "VERSION")?;
            let kind = mash_kind(&required_text(step, "MASH_STEP", "TYPE")?).to_owned();
            let infusion_l = step.number_of("INFUSE_AMOUNT");
            if kind == "infusion" && infusion_l.is_none() {
                return Err(Error::Format(
                    "MASH_STEP missing required field INFUSE_AMOUNT".into(),
                ));
            }
            mash_steps.push(MashStep {
                name: Some(required_text(step, "MASH_STEP", "NAME")?),
                kind,
                temp_c: required_number(step, "MASH_STEP", "STEP_TEMP")?,
                time_min: required_number(step, "MASH_STEP", "STEP_TIME")?,
                infusion_l,
            });
        }
    }

    let og = r.number_of("EST_OG").or_else(|| r.number_of("OG"));
    if og.is_some_and(|v| !(0.9..=1.3).contains(&v)) {
        warnings.push("OG in the file is outside 0.9 to 1.3 and was ignored".to_owned());
    }
    let valid_og = og.filter(|v| (0.9..=1.3).contains(v));
    let kind = kind_from(&recipe_type);
    if kind != "extract" && r.number_of("EFFICIENCY").is_none() {
        return Err(Error::Format(
            "RECIPE missing required field EFFICIENCY".into(),
        ));
    }
    if kind != "extract" && r.child("MASH").is_none() {
        return Err(Error::Format(
            "RECIPE missing required field MASH".into(),
        ));
    }
    let equipment = r.child("EQUIPMENT").map(parse_equipment).transpose()?;
    Ok(Recipe {
        name,
        notes: r.text_of("NOTES").map(str::to_owned),
        kind,
        style: Some(style_details.name.clone()),
        brewer: Some(brewer),
        style_details: Some(style_details),
        ibu_method: r.text_of("IBU_METHOD").map(str::to_ascii_lowercase),
        batch_volume_l: Some(batch_volume_l),
        boil_volume_l: Some(boil_volume_l),
        boil_time_min: Some(boil_time_min),
        efficiency_pct: r.number_of("EFFICIENCY"),
        source: SourceValues {
            og_sg: valid_og,
            fg_sg: r.number_of("EST_FG").or_else(|| r.number_of("FG")),
            ibu: r.number_of("IBU").or_else(|| r.number_of("EST_IBU")),
            srm: est_color_srm(r).or_else(|| r.number_of("COLOR")),
            abv_pct: r.number_of("ABV").or_else(|| r.number_of("EST_ABV")),
        },
        equipment,
        fermentables,
        hops,
        yeasts,
        miscs,
        mash_steps,
        warnings,
    })
}

/// Parses a `BeerXML` file (recipes and equipment records).
pub fn parse(input: &[u8]) -> Result<Document> {
    let root = xml::parse(input, Limits::default())?;
    let mut doc = Document::default();
    match root.name.as_str() {
        "RECIPES" => {
            doc.recipes = root
                .children_named("RECIPE")
                .map(parse_recipe)
                .collect::<Result<Vec<_>>>()?;
        }
        "RECIPE" => doc.recipes.push(parse_recipe(&root)?),
        "EQUIPMENTS" => {
            doc.equipment = root
                .children_named("EQUIPMENT")
                .map(parse_equipment)
                .collect::<Result<Vec<_>>>()?;
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

fn write_style(
    out: &mut String,
    indent: usize,
    style: Option<&BeerStyle>,
    name: &str,
    source: &SourceValues,
) {
    let fallback = BeerStyle {
        name: name.to_owned(),
        category: "Other".to_owned(),
        category_number: "0".to_owned(),
        style_letter: "A".to_owned(),
        style_guide: "Unknown".to_owned(),
        kind: "Ale".to_owned(),
        og_min: source.og_sg.unwrap_or(0.0),
        og_max: source.og_sg.unwrap_or(0.0),
        fg_min: source.fg_sg.unwrap_or(0.0),
        fg_max: source.fg_sg.unwrap_or(0.0),
        ibu_min: source.ibu.unwrap_or(0.0),
        ibu_max: source.ibu.unwrap_or(0.0),
        color_min: source.srm.unwrap_or(0.0),
        color_max: source.srm.unwrap_or(0.0),
        ..BeerStyle::default()
    };
    let s = style.unwrap_or(&fallback);
    let pad = " ".repeat(indent);
    let _ = writeln!(out, "{pad}<STYLE>");
    let i = indent + 2;
    tag(out, i, "NAME", &s.name);
    tag(out, i, "CATEGORY", &s.category);
    tag(out, i, "CATEGORY_NUMBER", &s.category_number);
    tag(out, i, "STYLE_LETTER", &s.style_letter);
    tag(out, i, "STYLE_GUIDE", &s.style_guide);
    tag(out, i, "VERSION", "1");
    tag(out, i, "TYPE", &s.kind);
    for (field, value) in [
        ("OG_MIN", s.og_min),
        ("OG_MAX", s.og_max),
        ("FG_MIN", s.fg_min),
        ("FG_MAX", s.fg_max),
        ("IBU_MIN", s.ibu_min),
        ("IBU_MAX", s.ibu_max),
        ("COLOR_MIN", s.color_min),
        ("COLOR_MAX", s.color_max),
    ] {
        tag(out, i, field, &value.to_string());
    }
    num(out, i, "CARB_MIN", s.carb_min);
    num(out, i, "CARB_MAX", s.carb_max);
    num(out, i, "ABV_MIN", s.abv_min);
    num(out, i, "ABV_MAX", s.abv_max);
    for (field, value) in [
        ("NOTES", s.notes.as_deref()),
        ("PROFILE", s.profile.as_deref()),
        ("INGREDIENTS", s.ingredients.as_deref()),
        ("EXAMPLES", s.examples.as_deref()),
    ] {
        if let Some(value) = value {
            tag(out, i, field, value);
        }
    }
    let _ = writeln!(out, "{pad}</STYLE>");
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
        if r.brewer.is_none() {
            let _ = writeln!(
                o,
                "    <!-- Brewer was not present in the neutral model; placeholder required by BeerXML. -->"
            );
        }
        tag(
            &mut o,
            4,
            "BREWER",
            r.brewer.as_deref().unwrap_or("Unknown"),
        );
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
        write_style(
            &mut o,
            4,
            r.style_details.as_ref(),
            r.style.as_deref().unwrap_or("Other"),
            &r.source,
        );
        if let Some(e) = &r.equipment {
            write_equipment(&mut o, 4, e);
        }
        o.push_str("    <HOPS>\n");
        for h in &r.hops {
            o.push_str("      <HOP>\n");
            tag(&mut o, 8, "VERSION", "1");
            tag(&mut o, 8, "NAME", &h.name);
            if h.alpha_pct.is_none() {
                let _ = writeln!(
                    o,
                    "        <!-- Missing alpha acid; BeerXML requires ALPHA. -->"
                );
            }
            tag(&mut o, 8, "ALPHA", &h.alpha_pct.unwrap_or(0.0).to_string());
            tag(&mut o, 8, "AMOUNT", &format!("{}", h.amount_kg));
            tag(
                &mut o,
                8,
                "USE",
                hop_use_xml(h.usage.as_deref().unwrap_or("boil")),
            );
            tag(&mut o, 8, "TIME", &h.time_min.unwrap_or(0.0).to_string());
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
            if f.yield_pct.is_none() {
                let _ = writeln!(o, "        <!-- Missing yield; BeerXML requires YIELD. -->");
            }
            tag(&mut o, 8, "YIELD", &f.yield_pct.unwrap_or(0.0).to_string());
            if f.color_srm.is_none() {
                let _ = writeln!(o, "        <!-- Missing color; BeerXML requires COLOR. -->");
            }
            let beerxml_color = match f.color_srm {
                None => 0.0,
                Some(color) if f.grain_type.as_deref() == Some("extract") => color,
                Some(color) if color <= 0.0 => 0.0,
                Some(color) => ((color + 0.76) / 1.3546).max(0.0),
            };
            tag(&mut o, 8, "COLOR", &beerxml_color.to_string());
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
            if y.amount.is_none() {
                let _ = writeln!(
                    o,
                    "        <!-- Yeast amount was not present in the source; BeerXML requires AMOUNT. -->"
                );
            }
            tag(&mut o, 8, "AMOUNT", &y.amount.unwrap_or(0.0).to_string());
            tag(
                &mut o,
                8,
                "AMOUNT_IS_WEIGHT",
                if y.amount_is_weight.unwrap_or(false) {
                    "TRUE"
                } else {
                    "FALSE"
                },
            );
            num(&mut o, 8, "ATTENUATION", y.attenuation_pct);
            o.push_str("      </YEAST>\n");
        }
        o.push_str("    </YEASTS>\n    <WATERS>\n    </WATERS>\n");
        if !r.mash_steps.is_empty() {
            o.push_str("    <MASH>\n      <VERSION>1</VERSION>\n      <NAME>Mash</NAME>\n      <GRAIN_TEMP>20</GRAIN_TEMP>\n      <MASH_STEPS>\n");
            for s in &r.mash_steps {
                o.push_str("        <MASH_STEP>\n");
                tag(&mut o, 10, "VERSION", "1");
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
