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

fn eight(v: &Value) -> [f64; 8] {
    nums(v).try_into().expect("8 salts")
}

fn waters(v: &Value) -> Vec<(f64, [f64; 6])> {
    v.as_array()
        .expect("waters")
        .iter()
        .map(|p| {
            let p = p.as_array().expect("pair");
            (p[0].as_f64().expect("amount"), ions(&p[1]))
        })
        .collect()
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

fn typed_bill(c: &Case) -> Vec<(f64, f64, u8)> {
    c.v("bill")
        .as_array()
        .expect("bill array")
        .iter()
        .map(|p| {
            let p = p.as_array().expect("triple");
            (
                p[0].as_f64().expect("kg"),
                p[1].as_f64().expect("srm"),
                u8::try_from(p[2].as_i64().expect("type")).expect("small type"),
            )
        })
        .collect()
}

/// Strict check for sparge acid outputs (T1.12 B8): relative 1e-12 of the expected value with
/// an absolute floor of 1e-18 (outputs are mol/L around 1e-6 to 1e-2 or mL), far below the
/// shared harness tolerance of 1e-12 absolute.
fn check_sparge(c: &Case, actual: f64) {
    let e = c.beklenen.as_f64().expect("number");
    assert!(
        (actual - e).abs() <= 1e-18 + 1e-12 * e.abs(),
        "case {} `{}` {}: got {actual:e}, expected {e:e}",
        c.index,
        c.fonk,
        c.girdi
    );
}

/// KKT conditions of the salt-matching NNLS: `x >= 0` (excluded salts exactly 0) and, with
/// `w = A^T (b - A x)`, `w_j ~ 0` where `x_j > 0` and `w_j <= 0` elsewhere (T1.12 B18).
#[allow(clippy::many_single_char_names)]
fn assert_kkt(c: &Case, base: &[f64; 6], s: f64, excluded: &[Salt], x: &[f64; 8]) {
    let target = ions(c.v("target"));
    let resid = water::best_fit_profile(base, s, x);
    let r: Vec<f64> = (0..6).map(|i| target[i] - resid[i]).collect();
    for (k, salt) in Salt::ALL.into_iter().enumerate() {
        assert!(x[k] >= 0.0, "case {}: negative grams", c.index);
        let w: f64 = (0..6).map(|i| salt.contribution()[i] * r[i]).sum();
        let scale = 1.0 + target.iter().fold(0.0_f64, |m, v| m.max(v.abs())) * 500.0;
        if excluded.contains(&salt) {
            assert!(x[k] == 0.0, "case {}: excluded salt used", c.index);
        } else if x[k] > 0.0 {
            assert!(w.abs() <= 1e-7 * scale, "case {}: KKT active {w}", c.index);
        } else {
            assert!(w <= 1e-7 * scale, "case {}: KKT inactive {w}", c.index);
        }
    }
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
                c.b("has_boil_ing"),
            )),
            "sulfate_chloride_label" => {
                c.check(&json!(water::sulfate_chloride_label(c.f("ratio"))));
            }
            "mash_ra_slope_checked" => c.check(&json!(water::mash_ra_slope_checked(
                small(c, "model"),
                c.f("t"),
                c.f("w_kg"),
            ))),
            "misc_acid_ra" => c.check_f(water::misc_acid_ra(
                small(c, "acid"),
                c.f("conc"),
                c.f("vol_l"),
                c.f("f"),
                c.f("w_kg"),
                c.f("t"),
            )),
            "acid_malt_ra" => c.check_f(water::acid_malt_ra(
                c.f("pct"),
                c.f("malt_kg"),
                c.f("f"),
                c.f("w_kg"),
                c.f("t"),
            )),
            "sparge_acid_ml" => check_sparge(
                c,
                water::sparge_acid_ml(c.f("mol_l"), c.f("v_l"), c.f("conc"), small(c, "acid")),
            ),
            "sparge_acid_amount" => check_sparge(
                c,
                water::sparge_acid_amount(
                    small(c, "model"),
                    c.b("ours"),
                    c.f("hco3"),
                    c.f("ph_w"),
                    c.f("ph_t"),
                    small(c, "acid"),
                    c.f("conc"),
                    c.f("mash_dph"),
                    c.f("v_sp_l"),
                ),
            ),
            "base_profile" => c.check(&json!(water::base_profile(&waters(c.v("waters"))))),
            "water_ph" => {
                let w: Vec<(f64, f64)> = c
                    .v("waters")
                    .as_array()
                    .expect("waters")
                    .iter()
                    .map(|p| {
                        let p = p.as_array().expect("pair");
                        (p[0].as_f64().expect("amount"), p[1].as_f64().expect("ph"))
                    })
                    .collect();
                c.check_f(water::water_ph(&w));
            }
            "agents_ppm" => c.check(&json!(water::agents_ppm(
                &salts(c.v("grams")),
                c.f("vol_l"),
                c.b("ignore"),
            ))),
            "agent_volume" => c.check_f(water::agent_volume(
                small(c, "rtype"),
                small(c, "use"),
                c.f("mash_tot"),
                c.f("sparge"),
                c.f("boil"),
                c.f("volume"),
                c.f("final"),
            )),
            "total_water_ion" => c.check(&json!(water::total_water_ion(
                &ions(c.v("mash")),
                &ions(c.v("sparge")),
                c.f("vm"),
                c.f("vs"),
                c.b("has_mash"),
            ))),
            "top_up_factor" => c.check_f(water::top_up_factor(
                c.f("v_avail"),
                c.f("vs"),
                c.f("v_topk"),
            )),
            "boil_ion" => c.check(&json!(water::boil_ion(
                &ions(c.v("total")),
                c.f("k_top"),
                &ions(c.v("adj0")),
            ))),
            "beer_ion" => c.check(&json!(water::beer_ion(
                &ions(c.v("boil")),
                c.f("s_boil"),
                &ions(c.v("adj2")),
            ))),
            "match_scale" => c.check_f(water::match_scale(
                c.b("mash_recipe"),
                c.b("has_boil_ing"),
                c.f("s_boil"),
            )),
            "salt_grams" => {
                let (m, sp) = water::salt_grams(
                    &eight(c.v("x")),
                    c.f("v_mash_gal"),
                    c.f("v_sp_gal"),
                    c.b("combine"),
                );
                c.check(&json!([m, sp]));
            }
            "profile_mix" => c.check(&json!(water::profile_mix(
                c.f("vb"),
                &ions(c.v("b")),
                c.f("vd"),
                &ions(c.v("d")),
            ))),
            "wp_grams" => c.check(&json!(water::wp_grams(&eight(c.v("x")), c.f("vb_gal")))),
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
            "sparge_acid_mph" => check_sparge(
                c,
                water::sparge_acid_mph(c.f("hco3"), c.f("ph_w"), c.f("ph_t")),
            ),
            "sparge_acid_mph_ours" => check_sparge(
                c,
                water::sparge_acid_mph_ours(
                    c.f("hco3"),
                    c.f("ph_w"),
                    c.f("ph_t"),
                    small(c, "acid"),
                ),
            ),
            "sparge_acid_bw" => check_sparge(
                c,
                water::sparge_acid_bw(c.f("hco3"), c.f("ph_w"), c.f("ph_t"), small(c, "acid")),
            ),
            "lactic_ml" => check_sparge(c, water::lactic_ml(c.f("mol_l"), c.f("v_l"), c.f("conc"))),
            "grist_ph_typed" => {
                let (ph, sum) = water::grist_ph_typed(&typed_bill(c), c.b("ours"));
                c.check(&json!([ph, sum]));
            }
            "to_grams" => c.check(&json!(water::to_grams(c.f("amount"), small(c, "units")))),
            "adjusted_ph_liquid" => c.check_f(water::adjusted_ph_liquid(
                c.f("grist_ph"),
                c.f("ra"),
                c.f("t"),
                small(c, "model"),
                c.f("w_kg"),
                small(c, "acid"),
                c.f("conc"),
                c.f("vol_l"),
                c.f("target_ph"),
                c.b("ours"),
            )),
            "acid_malt_amount_checked" => {
                let r = water::acid_malt_amount_checked(
                    &bill(c),
                    c.f("pct"),
                    small(c, "model"),
                    c.f("ra"),
                    c.f("t"),
                    c.f("target_ph"),
                    c.b("ours"),
                );
                c.check(&json!([r.total_g, r.ph, r.converged]));
            }
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
                assert_kkt(c, &base, s, &excluded, &x);
                c.check(&json!({"profile": profile, "rnorm": rnorm}));
            }
            _ => return false,
        }
        true
    });
}
