//! Section 03 parity: water chemistry, mash pH, NNLS against `vektorler/03.json`.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines // one match arm per engine function
)]

mod ortak;

use engine::water::{self, Salt};
use ortak::Case;
use serde_json::{Value, json};

fn nums(v: &Value) -> Vec<f64> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|x| x.as_f64().expect("number"))
        .collect()
}

fn ions(v: &Value) -> [f64; 6] {
    nums(v).try_into().expect("6 ions")
}

fn bill(c: &Case) -> Vec<(f64, f64)> {
    c.v("bill")
        .as_array()
        .expect("bill array")
        .iter()
        .map(|p| {
            let p = nums(p);
            (p[0], p[1])
        })
        .collect()
}

fn salts(v: &Value) -> Vec<(Salt, f64)> {
    v.as_object()
        .expect("salt object")
        .iter()
        .map(|(k, g)| {
            (
                Salt::from_key(k).expect("known salt"),
                g.as_f64().expect("grams"),
            )
        })
        .collect()
}

fn small(c: &Case, name: &str) -> u8 {
    u8::try_from(c.i(name)).expect("small code")
}

#[test]
fn parite_03() {
    ortak::run("03", |c| {
        match c.fonk.as_str() {
            "salt_ppm" => c.check(&json!(water::salt_ppm(&salts(c.v("grams")), c.f("vol_l")))),
            "alkalinity" => c.check_f(water::alkalinity(c.f("hco3"))),
            "hardness" => c.check_f(water::hardness(c.f("ca"), c.f("mg"))),
            "residual_alkalinity" => {
                c.check_f(water::residual_alkalinity(
                    c.f("ca"),
                    c.f("mg"),
                    c.f("hco3"),
                ));
            }
            "sulfate_chloride_ratio" => {
                c.check_f(water::sulfate_chloride_ratio(c.f("so4"), c.f("cl")));
            }
            "color_range" => {
                let (lo, hi) = water::color_range(c.f("ra"));
                c.check(&json!([lo, hi]));
            }
            "boil_scale" => c.check_f(water::boil_scale(
                c.f("vb"),
                c.f("vp"),
                c.f("cool_loss"),
                c.f("trub_loss"),
                c.f("top_up"),
            )),
            "grist_g" => c.check_f(water::grist_g(c.f("c"), c.b("ours"))),
            "grist_ph" => {
                let (ph, s) = water::grist_ph(&bill(c), c.b("ours"));
                c.check(&json!([ph, s]));
            }
            "mash_ra_slope" => c.check_f(water::mash_ra_slope(small(c, "model"), c.f("t"))),
            "est_mash_ph" => {
                c.check_f(water::est_mash_ph(c.f("grist_ph"), c.f("ra"), c.f("slope")));
            }
            "acid_pks" => c.check(&json!(water::acid_pks(small(c, "acid")))),
            "acid_fraction" => c.check_f(water::acid_fraction(small(c, "acid"), c.f("ph"))),
            "dissociation_fraction" => c.check_f(water::dissociation_fraction(
                small(c, "acid"),
                small(c, "model"),
                c.f("ph_t"),
                c.b("ours"),
            )),
            "acid_per_liter" => c.check_f(water::acid_per_liter(
                c.f("meq"),
                small(c, "acid"),
                c.f("conc"),
                small(c, "model"),
                c.f("ph_t"),
                c.b("ours"),
            )),
            "mash_acid_amount" => c.check_f(water::mash_acid_amount(
                c.f("est_ph"),
                c.f("target_ph"),
                small(c, "model"),
                c.f("t"),
                c.f("w_kg"),
                small(c, "acid"),
                c.f("conc"),
                c.b("ours"),
            )),
            "lactic_acid_ra" => c.check_f(water::lactic_acid_ra(
                c.f("conc"),
                c.f("vol_l"),
                c.f("f"),
                c.f("w_kg"),
                c.f("t"),
            )),
            "adjusted_ph" => {
                let (ph, w) = water::adjusted_ph(
                    &bill(c),
                    c.f("acid_malt_g"),
                    c.f("pct"),
                    small(c, "model"),
                    c.f("ra"),
                    c.f("t"),
                    c.f("target_ph"),
                    c.b("ours"),
                );
                c.check(&json!([ph, w]));
            }
            "acid_malt_amount" => {
                let (g, ph) = water::acid_malt_amount(
                    &bill(c),
                    c.f("pct"),
                    small(c, "model"),
                    c.f("ra"),
                    c.f("t"),
                    c.f("target_ph"),
                    c.b("ours"),
                );
                c.check(&json!([g, ph]));
            }
            "sparge_acid_mph" => {
                c.check_f(water::sparge_acid_mph(
                    c.f("hco3"),
                    c.f("ph_w"),
                    c.f("ph_t"),
                ));
            }
            "sparge_acid_mph_ours" => c.check_f(water::sparge_acid_mph_ours(
                c.f("hco3"),
                c.f("ph_w"),
                c.f("ph_t"),
                small(c, "acid"),
            )),
            "sparge_acid_bw" => c.check_f(water::sparge_acid_bw(
                c.f("hco3"),
                c.f("ph_w"),
                c.f("ph_t"),
                small(c, "acid"),
            )),
            "lactic_ml" => c.check_f(water::lactic_ml(c.f("mol_l"), c.f("v_l"), c.f("conc"))),
            "sparge_volume_l" => {
                c.check_f(water::sparge_volume_l(
                    c.f("total_l"),
                    c.f("w_kg"),
                    c.f("t"),
                ));
            }
            "warnings" => {
                let w: Vec<&str> = water::warnings(small(c, "model"), &salts(c.v("salt_grams")))
                    .into_iter()
                    .map(water::Warning::message)
                    .collect();
                c.check(&json!(w));
            }
            "nnls" => {
                let cols: Vec<Vec<f64>> = c
                    .v("a_cols")
                    .as_array()
                    .expect("columns")
                    .iter()
                    .map(nums)
                    .collect();
                let (x, rnorm) = water::nnls(&cols, &nums(c.v("b")));
                c.check(&json!({"x": x, "rnorm": rnorm}));
            }
            "water_match" => {
                let base = ions(c.v("base"));
                let s = c.f("s");
                let excluded: Vec<Salt> = c
                    .v("excluded")
                    .as_array()
                    .expect("excluded")
                    .iter()
                    .map(|k| Salt::from_key(k.as_str().expect("key")).expect("known salt"))
                    .collect();
                let (x, rnorm) = water::water_match(&base, &ions(c.v("target")), s, &excluded);
                let profile = water::best_fit_profile(&base, s, &x);
                c.check(&json!({"profile": profile, "rnorm": rnorm}));
            }
            _ => return false,
        }
        true
    });
}
