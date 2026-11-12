//! Parity vector loader shared by `tests/parite_NN.rs` (ADR-0003).
//!
//! Each vector file is produced by `formuller/testler/parite/NN_*.py` from the Python
//! reference. A test maps every case's `fonk` to an engine call and compares results.
#![allow(
    dead_code,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used
)]

use std::path::PathBuf;

use serde_json::Value;

/// Floating-point tolerance: `|a − b| ≤ ABS_TOL + REL_TOL · |b|` (ADR-0003 madde 3).
pub const REL_TOL: f64 = 1e-11;
/// Absolute floor, only meaningful around zero.
pub const ABS_TOL: f64 = 1e-12;

/// One case of a vector file.
#[derive(Debug)]
pub struct Case {
    pub index: usize,
    pub fonk: String,
    pub girdi: Value,
    pub beklenen: Value,
}

impl Case {
    /// Numeric input by name.
    pub fn f(&self, name: &str) -> f64 {
        self.girdi
            .get(name)
            .and_then(Value::as_f64)
            .unwrap_or_else(|| {
                panic!(
                    "case {} ({}): missing number input `{name}`",
                    self.index, self.fonk
                )
            })
    }

    /// Integer input by name.
    pub fn i(&self, name: &str) -> i64 {
        self.girdi
            .get(name)
            .and_then(Value::as_i64)
            .unwrap_or_else(|| {
                panic!(
                    "case {} ({}): missing integer input `{name}`",
                    self.index, self.fonk
                )
            })
    }

    /// Boolean input by name.
    pub fn b(&self, name: &str) -> bool {
        self.girdi
            .get(name)
            .and_then(Value::as_bool)
            .unwrap_or_else(|| {
                panic!(
                    "case {} ({}): missing bool input `{name}`",
                    self.index, self.fonk
                )
            })
    }

    /// Raw input value by name.
    pub fn v(&self, name: &str) -> &Value {
        self.girdi.get(name).unwrap_or_else(|| {
            panic!(
                "case {} ({}): missing input `{name}`",
                self.index, self.fonk
            )
        })
    }

    fn label(&self) -> String {
        format!("case {} `{}` girdi={}", self.index, self.fonk, self.girdi)
    }

    /// Compares a single number with `beklenen`.
    pub fn check_f(&self, actual: f64) {
        // NaN/inf would serialise to null and could match a Python None (T1.12 P4).
        assert!(
            actual.is_finite(),
            "{}: non-finite result {actual}",
            self.label()
        );
        check_value(&self.label(), &serde_json::json!(actual), &self.beklenen);
    }

    /// Compares an engine result: `Ok` against the number in `beklenen`, `Err` against a
    /// `{"hata": ...}` expectation (ADR-0004 madde 3).
    pub fn check_r(&self, r: engine::Result<f64>) {
        match r {
            Ok(x) => self.check_f(x),
            Err(_) => self.check_hata(),
        }
    }

    /// Like [`Case::check_r`] for any JSON-shaped result.
    pub fn check_res(&self, r: engine::Result<Value>) {
        match r {
            Ok(v) => self.check(&v),
            Err(_) => self.check_hata(),
        }
    }

    /// Asserts the case expects an error (`{"hata": ...}`) because the engine returned `Err`.
    pub fn check_hata(&self) {
        assert!(
            self.beklenen.get("hata").is_some(),
            "{}: engine returned Err but the reference returned {}",
            self.label(),
            self.beklenen
        );
    }

    /// Compares any JSON-shaped result (numbers within tolerance, everything else exact).
    pub fn check(&self, actual: &Value) {
        check_value(&self.label(), actual, &self.beklenen);
    }
}

/// Loads `formuller/testler/vektorler/<bolum>.json`.
pub fn load(bolum: &str) -> Vec<Case> {
    // Self-check: decimal text must parse to the nearest f64 (`float_roundtrip`), as Python does.
    let probe: f64 = serde_json::from_str("9.395656388205701").expect("number");
    assert_eq!(
        probe.to_bits(),
        9.395_656_388_205_701_f64.to_bits(),
        "serde_json float_roundtrip is off"
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../formuller/testler/vektorler")
        .join(format!("{bolum}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let root: Value = serde_json::from_str(&text).expect("vector file is valid JSON");
    assert_eq!(
        root["bolum"],
        bolum,
        "section id mismatch in {}",
        path.display()
    );
    let cases: Vec<Case> = root["vakalar"]
        .as_array()
        .expect("`vakalar` is an array")
        .iter()
        .enumerate()
        .map(|(index, c)| Case {
            index,
            fonk: c["fonk"].as_str().expect("`fonk` is a string").to_owned(),
            girdi: c["girdi"].clone(),
            beklenen: c["beklenen"].clone(),
        })
        .collect();
    assert!(!cases.is_empty(), "no cases in {}", path.display());
    cases
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= ABS_TOL + REL_TOL * b.abs()
}

fn check_value(label: &str, actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(e)) => {
            // Integers (Python int) compare exactly (ADR-0003 madde 3, T1.12 P3).
            if let (Some(ai), Some(ei), false, false) =
                (a.as_i64(), e.as_i64(), a.is_f64(), e.is_f64())
            {
                assert_eq!(ai, ei, "{label}: got {ai}, expected {ei}");
                return;
            }
            let (a, e) = (a.as_f64().unwrap(), e.as_f64().unwrap());
            assert!(close(a, e), "{label}: got {a}, expected {e}");
        }
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(
                a.len(),
                e.len(),
                "{label}: length {} vs {}",
                a.len(),
                e.len()
            );
            for (x, y) in a.iter().zip(e) {
                check_value(label, x, y);
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            let mut ak: Vec<_> = a.keys().collect();
            let mut ek: Vec<_> = e.keys().collect();
            ak.sort();
            ek.sort();
            assert_eq!(ak, ek, "{label}: keys differ");
            for (k, y) in e {
                check_value(label, &a[k], y);
            }
        }
        _ => assert_eq!(actual, expected, "{label}"),
    }
}

/// Runs `f` on every case and fails if any `fonk` is unhandled (`f` returns `false`).
pub fn run(bolum: &str, mut f: impl FnMut(&Case) -> bool) {
    let cases = load(bolum);
    for c in &cases {
        assert!(
            f(c),
            "{}: unhandled fonk `{}` in section {bolum}",
            c.label(),
            c.fonk
        );
    }
}

// ---------------------------------------------------------------------------------------
// Shared recipe fixtures (ADR-0004 items 1 and 4): sections 01 and 02 read the same JSON
// recipe whose field names are those of `engine::recipe::RecipeInput`.
// ---------------------------------------------------------------------------------------

use engine::recipe::{
    Equipment, Fermentable, GrainType, GrainUse, Hop, HopForm, HopOptions, HopUse, Mash, MashStep,
    Misc, MiscUse, Options, RecipeInput, RecipeType, StepKind, Yeast, YeastForm,
};

fn jnum(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing number `{key}` in {v}"))
}

fn jflag(v: &Value, key: &str) -> bool {
    v[key]
        .as_bool()
        .unwrap_or_else(|| panic!("missing bool `{key}` in {v}"))
}

fn jcode(v: &Value, key: &str) -> i64 {
    v[key]
        .as_i64()
        .unwrap_or_else(|| panic!("missing integer `{key}` in {v}"))
}

fn jarr<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    v[key]
        .as_array()
        .unwrap_or_else(|| panic!("missing array `{key}` in {v}"))
}

pub fn fermentable(v: &Value) -> Fermentable {
    Fermentable {
        name: v["name"].as_str().expect("name").to_owned(),
        amount_oz: jnum(v, "amount_oz"),
        yield_pct: jnum(v, "yield_pct"),
        color_srm: jnum(v, "color_srm"),
        kind: GrainType::from_code(jcode(v, "kind")).expect("grain type"),
        usage: GrainUse::from_code(jcode(v, "usage")).expect("grain use"),
        not_fermentable: jflag(v, "not_fermentable"),
        late_extract_min: jnum(v, "late_extract_min"),
        max_in_batch_pct: jnum(v, "max_in_batch_pct"),
        ibu_gal_per_lb: jnum(v, "ibu_gal_per_lb"),
    }
}

pub fn hop(v: &Value) -> Hop {
    Hop {
        amount_oz: jnum(v, "amount_oz"),
        alpha_pct: jnum(v, "alpha_pct"),
        form: HopForm::from_code(jcode(v, "form")).expect("hop form"),
        usage: HopUse::from_code(jcode(v, "usage")).expect("hop use"),
        time_min: jnum(v, "time_min"),
        whirlpool_temp_f: jnum(v, "whirlpool_temp_f"),
    }
}

pub fn hops(v: &Value) -> Vec<Hop> {
    v.as_array().expect("hops").iter().map(hop).collect()
}

pub fn misc(v: &Value) -> Misc {
    Misc {
        usage: MiscUse::from_code(jcode(v, "usage")).expect("misc use"),
        time_min: jnum(v, "time_min"),
    }
}

pub fn hop_options(v: &Value) -> HopOptions {
    HopOptions {
        first_wort_adj_pct: jnum(v, "first_wort_adj_pct"),
        type_adj: jflag(v, "type_adj"),
        leaf_adj_pct: jnum(v, "leaf_adj_pct"),
        plug_adj_pct: jnum(v, "plug_adj_pct"),
        pellet_adj_pct: jnum(v, "pellet_adj_pct"),
        mash_adj_pct: jnum(v, "mash_adj_pct"),
    }
}

pub fn yeast(v: &Value) -> Yeast {
    Yeast {
        min_attenuation_pct: jnum(v, "min_attenuation_pct"),
        max_attenuation_pct: jnum(v, "max_attenuation_pct"),
        tolerance_abv_pct: jnum(v, "tolerance_abv_pct"),
        add_to_secondary: jflag(v, "add_to_secondary"),
        form: YeastForm::from_code(jcode(v, "form")).expect("yeast form"),
        total_cells_billion: jnum(v, "total_cells_billion"),
        viable_cells_billion: jnum(v, "viable_cells_billion"),
        packs: jnum(v, "packs"),
        viability_pct: jnum(v, "viability_pct"),
        use_packages: jflag(v, "use_packages"),
        amount_oz: jnum(v, "amount_oz"),
        cells_per_ml_billion: jnum(v, "cells_per_ml_billion"),
        dry_pitch_rate_g_per_hl: jnum(v, "dry_pitch_rate_g_per_hl"),
    }
}

pub fn mash(v: &Value) -> Mash {
    Mash {
        grain_weight_oz: jnum(v, "grain_weight_oz"),
        biab: jflag(v, "biab"),
        steps: jarr(v, "steps")
            .iter()
            .map(|s| MashStep {
                kind: StepKind::from_code(jcode(s, "kind")).expect("step kind"),
                temp_f: jnum(s, "temp_f"),
                infusion_floz: jnum(s, "infusion_floz"),
            })
            .collect(),
    }
}

pub fn mash_json(m: &Mash) -> Value {
    serde_json::json!({
        "grain_weight_oz": m.grain_weight_oz,
        "biab": m.biab,
        "steps": m.steps.iter()
            .map(|s| serde_json::json!({
                "kind": s.kind.code(), "temp_f": s.temp_f, "infusion_floz": s.infusion_floz,
            }))
            .collect::<Vec<_>>(),
    })
}

pub fn recipe_input(v: &Value) -> RecipeInput {
    let e = &v["equipment"];
    let o = &v["options"];
    RecipeInput {
        kind: RecipeType::from_code(jcode(v, "kind")).expect("recipe type"),
        volume_floz: jnum(v, "volume_floz"),
        boil_vol_floz: jnum(v, "boil_vol_floz"),
        boil_time_min: jnum(v, "boil_time_min"),
        efficiency_pct: jnum(v, "efficiency_pct"),
        equipment: Equipment {
            trub_loss_floz: jnum(e, "trub_loss_floz"),
            pre_trub_loss_floz: jnum(e, "pre_trub_loss_floz"),
            altitude_ft: jnum(e, "altitude_ft"),
            hop_util_pct: jnum(e, "hop_util_pct"),
            whirlpool_carry: jflag(e, "whirlpool_carry"),
            whirlpool_time_min: jnum(e, "whirlpool_time_min"),
        },
        fermentables: jarr(v, "fermentables").iter().map(fermentable).collect(),
        hops: jarr(v, "hops").iter().map(hop).collect(),
        yeasts: jarr(v, "yeasts").iter().map(yeast).collect(),
        miscs: jarr(v, "miscs").iter().map(misc).collect(),
        mash: mash(&v["mash"]),
        options: Options {
            steep_efficiency_pct: jnum(o, "steep_efficiency_pct"),
            mash_fg_adjust: jflag(o, "mash_fg_adjust"),
            mash_fg_adj_temp_f: jnum(o, "mash_fg_adj_temp_f"),
            mash_fg_adj_slope_pct_per_f: jnum(o, "mash_fg_adj_slope"),
            hop: hop_options(&o["hop"]),
        },
    }
}
