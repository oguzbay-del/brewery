//! Conversions between the model and the version JSON of the API.

use serde_json::Value;

use crate::model::{Equipment, Fermentable, Hop, MashStep, Misc, Recipe, SourceValues, Yeast};

impl Recipe {
    /// Fills the values the API requires and a file may lack. Returns one note for each change.
    pub fn fill_defaults(&mut self) -> Vec<String> {
        let mut notes = Vec::new();
        let eq = self.equipment.clone();
        if self.batch_volume_l.is_none() {
            self.batch_volume_l = eq.as_ref().and_then(|e| e.batch_volume_l).or(Some(20.0));
            notes
                .push("batch_volume_l: not in the file, set from the equipment or 20 L".to_owned());
        }
        if self.efficiency_pct.is_none() {
            self.efficiency_pct = eq.as_ref().and_then(|e| e.efficiency_pct).or(Some(70.0));
            notes.push("efficiency_pct: not in the file, set from the equipment or 70".to_owned());
        }
        for f in &mut self.fermentables {
            if f.grain_type.is_none() {
                f.grain_type = Some("grain".to_owned());
            }
            if f.yield_pct.is_none() {
                let y = match f.grain_type.as_deref() {
                    Some("sugar") => 100.0,
                    Some("dry_extract") => 95.0,
                    Some("extract") => 80.0,
                    _ => 75.0,
                };
                f.yield_pct = Some(y);
                notes.push(format!("fermentable {}: yield_pct set to {y}", f.name));
            }
        }
        for h in &mut self.hops {
            if h.alpha_pct.is_none() {
                h.alpha_pct = Some(0.0);
                notes.push(format!(
                    "hop {}: alpha_pct not in the file, set to 0",
                    h.name
                ));
            }
            if !matches!(
                h.form.as_deref(),
                Some("pellet" | "plug" | "leaf" | "co2_extract" | "isomerized_extract")
            ) {
                if h.form.is_some() {
                    notes.push(format!("hop {}: form changed to pellet", h.name));
                }
                h.form = Some("pellet".to_owned());
            }
        }
        for y in &mut self.yeasts {
            if y.attenuation_pct.is_none() {
                y.attenuation_pct = Some(75.0);
                notes.push(format!(
                    "yeast {}: attenuation not in the file, set to 75",
                    y.name
                ));
            }
        }
        notes
    }

    /// Builds a recipe from a version JSON of the API (header, lines and `calc_summary`). The
    /// source values come from the stored calculation, else from the targets.
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn from_version_json(name: &str, notes: Option<&str>, v: &Value) -> Self {
        let f = |o: &Value, k: &str| o.get(k).and_then(Value::as_f64);
        let t = |o: &Value, k: &str| o.get(k).and_then(Value::as_str).map(str::to_owned);
        let arr = |k: &str| {
            v.get(k)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };
        let summary = v.get("calc_summary").filter(|s| s.is_object());
        let sv = |sk: &str, tk: &str| summary.and_then(|s| f(s, sk)).or_else(|| f(v, tk));
        let eq = v
            .get("equipment_snapshot")
            .filter(|e| e.get("name").is_some());
        Self {
            name: name.to_owned(),
            notes: notes.map(str::to_owned),
            kind: t(v, "kind").unwrap_or_else(|| "all_grain".to_owned()),
            style: None,
            brewer: None,
            style_details: None,
            ibu_method: v
                .get("options")
                .and_then(|o| o.get("ibu_equation"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            batch_volume_l: f(v, "batch_volume_l"),
            boil_volume_l: f(v, "boil_volume_l"),
            boil_time_min: f(v, "boil_time_min"),
            efficiency_pct: f(v, "efficiency_pct"),
            source: SourceValues {
                og_sg: sv("og_sg", "target_og_sg"),
                fg_sg: sv("fg_sg", "target_fg_sg"),
                ibu: sv("ibu", "target_ibu"),
                srm: sv("color_srm", "target_srm"),
                abv_pct: sv("abv_pct", "target_abv_pct"),
            },
            equipment: eq.map(|e| Equipment {
                name: t(e, "name").unwrap_or_default(),
                batch_volume_l: f(e, "batch_volume_l"),
                boil_volume_l: f(e, "boil_volume_l"),
                boil_time_min: f(e, "boil_time_min"),
                efficiency_pct: f(e, "efficiency_pct"),
                trub_loss_l: f(e, "trub_loss_l"),
                fermenter_loss_l: f(e, "fermenter_loss_l"),
                boil_off_l_per_h: f(e, "boil_off_l_per_h"),
            }),
            fermentables: arr("fermentables")
                .iter()
                .map(|l| Fermentable {
                    name: t(l, "name").unwrap_or_default(),
                    amount_kg: f(l, "amount_kg").unwrap_or(0.0),
                    usage: t(l, "usage"),
                    grain_type: t(l, "grain_type"),
                    yield_pct: f(l, "yield_pct"),
                    color_srm: f(l, "color_srm"),
                })
                .collect(),
            hops: arr("hops")
                .iter()
                .map(|l| Hop {
                    name: t(l, "name").unwrap_or_default(),
                    amount_kg: f(l, "amount_kg").unwrap_or(0.0),
                    alpha_pct: f(l, "alpha_pct"),
                    form: t(l, "form"),
                    usage: t(l, "usage"),
                    time_min: f(l, "time_min"),
                    whirlpool_temp_c: f(l, "whirlpool_temp_c"),
                })
                .collect(),
            yeasts: arr("yeasts")
                .iter()
                .map(|l| Yeast {
                    name: t(l, "name").unwrap_or_default(),
                    form: t(l, "form"),
                    attenuation_pct: match (
                        f(l, "min_attenuation_pct"),
                        f(l, "max_attenuation_pct"),
                    ) {
                        (Some(a), Some(b)) => Some(f64::midpoint(a, b)),
                        (a, b) => a.or(b),
                    },
                    amount: None,
                    amount_is_weight: None,
                    packs: f(l, "packs"),
                })
                .collect(),
            miscs: arr("miscs")
                .iter()
                .map(|l| Misc {
                    name: t(l, "name").unwrap_or_default(),
                    amount: f(l, "amount").unwrap_or(0.0),
                    is_liquid: l.get("is_liquid").and_then(Value::as_bool),
                    usage: t(l, "usage"),
                    time_min: f(l, "time_min"),
                })
                .collect(),
            mash_steps: v
                .get("mash_snapshot")
                .and_then(|m| m.get("steps"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|s| MashStep {
                    name: t(s, "name"),
                    kind: t(s, "kind").unwrap_or_else(|| "infusion".to_owned()),
                    temp_c: f(s, "temp_c").unwrap_or(67.0),
                    time_min: f(s, "time_min").unwrap_or(60.0),
                    infusion_l: f(s, "infusion_l"),
                })
                .collect(),
            warnings: Vec::new(),
        }
    }
}
