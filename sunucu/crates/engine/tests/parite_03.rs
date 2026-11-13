//! Section 03 parity: water chemistry, mash pH, NNLS against `vektorler/03.json`.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines // one match arm per engine function
)]

mod ortak;

use engine::recipe::GrainType;
use engine::water::{self, Acid, MashModel, MiscUnit, Salt};
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

fn typed_bill(c: &Case) -> Vec<(f64, f64, GrainType)> {
    c.v("bill")
        .as_array()
        .expect("bill array")
        .iter()
        .map(|p| {
            let p = p.as_array().expect("triple");
            (
                p[0].as_f64().expect("kg"),
                p[1].as_f64().expect("srm"),
                GrainType::from_code(p[2].as_i64().expect("type")).expect("grain type"),
            )
        })
        .collect()
}

/// Strict check for sparge acid outputs (T1.12 B8): relative 1e-12 of the expected value with
/// an absolute floor of 1e-18 (outputs are mol/L around 1e-6 to 1e-2 or mL), far below the
/// shared harness tolerance of 1e-12 absolute.
fn check_sparge(c: &Case, actual: engine::Result<f64>) {
    let Ok(actual) = actual else {
        c.check_hata();
        return;
    };
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

fn acid(c: &Case) -> Acid {
    Acid::from_code(c.i("acid")).expect("acid code")
}

fn model(c: &Case) -> MashModel {
    MashModel::from_code(c.i("model")).expect("model code")
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
                let r = water::color_range(c.f("ra"));
                let (lo, hi) = (r.min_srm, r.max_srm);
                c.check(&json!([lo, hi]));
            }
            "boil_scale" => c.check_r(water::boil_scale(&water::BoilScaleInput {
                vb_l: c.f("vb"),
                vp_l: c.f("vp"),
                cool_loss_l: c.f("cool_loss"),
                trub_loss_l: c.f("trub_loss"),
                top_up_l: c.f("top_up"),
                has_boil_ing: c.b("has_boil_ing"),
            })),
            "sulfate_chloride_label" => {
                c.check(&json!(balance_label(water::sulfate_chloride_balance(
                    c.f("ratio")
                ))));
            }
            "mash_ra_slope_checked" => c.check_r(water::mash_ra_slope_checked(
                model(c),
                c.f("t"),
                c.f("w_kg"),
            )),
            "misc_acid_ra" => c.check_r(water::misc_acid_ra(&water::MiscAcidRaInput {
                acid: acid(c),
                conc_pct: c.f("conc"),
                vol_l: c.f("vol_l"),
                f: c.f("f"),
                w_kg: c.f("w_kg"),
                t_l_per_kg: c.f("t"),
            })),
            "acid_malt_ra" => c.check_r(water::acid_malt_ra(&water::AcidMaltRaInput {
                acid_pct: c.f("pct"),
                malt_kg: c.f("malt_kg"),
                f: c.f("f"),
                w_kg: c.f("w_kg"),
                t_l_per_kg: c.f("t"),
            })),
            "sparge_acid_ml" => check_sparge(
                c,
                Ok(water::sparge_acid_ml(
                    c.f("mol_l"),
                    c.f("v_l"),
                    c.f("conc"),
                    acid(c),
                )),
            ),
            "sparge_acid_amount" => check_sparge(
                c,
                water::sparge_acid_amount(&water::SpargeAcidInput {
                    model: model(c),
                    ours: c.b("ours"),
                    hco3_ppm: c.f("hco3"),
                    ph_w: c.f("ph_w"),
                    ph_t: c.f("ph_t"),
                    acid: acid(c),
                    conc_pct: c.f("conc"),
                    mash_dph: c.f("mash_dph"),
                    v_sp_l: c.f("v_sp_l"),
                }),
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
            "agent_volume" => c.check_f(water::agent_volume(&water::AgentVolumeInput {
                rtype: engine::recipe::RecipeType::from_code(c.i("rtype")).expect("recipe type"),
                use_: u8::try_from(c.i("use")).expect("use code"),
                mash_tot_water_l: c.f("mash_tot"),
                sparge_vol_l: c.f("sparge"),
                boil_vol_l: c.f("boil"),
                volume_l: c.f("volume"),
                final_vol_l: c.f("final"),
            })),
            "total_water_ion" => {
                c.check(&json!(water::total_water_ion(&water::TotalWaterIonInput {
                    mash: &ions(c.v("mash")),
                    sparge: &ions(c.v("sparge")),
                    vm_l: c.f("vm"),
                    vs_l: c.f("vs"),
                    has_mash: c.b("has_mash")
                })));
            }
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
                let g = water::salt_grams(
                    &eight(c.v("x")),
                    c.f("v_mash_gal"),
                    c.f("v_sp_gal"),
                    c.b("combine"),
                );
                c.check(&json!([g.mash_g, g.sparge_g]));
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
                let g = water::grist_ph(&bill(c), c.b("ours"));
                let (ph, s) = (g.ph, g.s);
                c.check(&json!([ph, s]));
            }
            "mash_ra_slope" => c.check_f(water::mash_ra_slope(model(c), c.f("t"))),
            "est_mash_ph" => {
                c.check_f(water::est_mash_ph(c.f("grist_ph"), c.f("ra"), c.f("slope")));
            }
            "acid_pks" => {
                c.check_res(Acid::from_code(c.i("acid")).map(|a| json!(water::acid_pks(a))));
            }
            "acid_fraction" => c.check_f(water::acid_fraction(acid(c), c.f("ph"))),
            "dissociation_fraction" => c.check_f(water::dissociation_fraction(
                acid(c),
                model(c),
                c.f("ph_t"),
                c.b("ours"),
            )),
            "acid_per_liter" => c.check_r(water::acid_per_liter(&water::AcidPerLiterInput {
                meq_l: c.f("meq"),
                acid: acid(c),
                conc_pct: c.f("conc"),
                model: model(c),
                ph_t: c.f("ph_t"),
                ours: c.b("ours"),
            })),
            "mash_acid_amount" => c.check_r(water::mash_acid_amount(&water::MashAcidAmountInput {
                est_ph: c.f("est_ph"),
                target_ph: c.f("target_ph"),
                model: model(c),
                t_l_per_kg: c.f("t"),
                w_kg: c.f("w_kg"),
                acid: acid(c),
                conc_pct: c.f("conc"),
                ours: c.b("ours"),
            })),
            "lactic_acid_ra" => c.check_r(water::misc_acid_ra(&water::MiscAcidRaInput {
                acid: water::Acid::Lactic,
                conc_pct: c.f("conc"),
                vol_l: c.f("vol_l"),
                f: c.f("f"),
                w_kg: c.f("w_kg"),
                t_l_per_kg: c.f("t"),
            })),
            "adjusted_ph_multi_acid" => c.check_f(water::adjusted_ph_multi_acid(
                c.f("grist_ph"),
                c.f("ra"),
                &c.v("acid_ras")
                    .as_array()
                    .expect("acid_ras array")
                    .iter()
                    .map(|x| x.as_f64().expect("ra"))
                    .collect::<Vec<_>>(),
                c.f("slope"),
            )),
            "adjusted_ph" => {
                let r = water::adjusted_ph(&water::AdjustedPhInput {
                    bill: &typed_bill(c),
                    acid_malt_g: c.f("acid_malt_g"),
                    acid_pct: c.f("pct"),
                    model: model(c),
                    ra_ppm: c.f("ra"),
                    t_l_per_kg: c.f("t"),
                    target_ph: c.f("target_ph"),
                    ours: c.b("ours"),
                });
                c.check_res(r.map(|a| json!([a.ph, a.grain_kg])));
            }
            "acid_malt_amount" => {
                let r = water::acid_malt_amount(&water::AcidMaltInput {
                    bill: &typed_bill(c),
                    acid_pct: c.f("pct"),
                    model: model(c),
                    ra_ppm: c.f("ra"),
                    t_l_per_kg: c.f("t"),
                    target_ph: c.f("target_ph"),
                    ours: c.b("ours"),
                });
                c.check_res(r.map(|a| json!([a.total_g, a.ph])));
            }
            "sparge_acid_mph" => check_sparge(
                c,
                Ok(water::sparge_acid_mph(
                    c.f("hco3"),
                    c.f("ph_w"),
                    c.f("ph_t"),
                )),
            ),
            "sparge_acid_mph_ours" => check_sparge(
                c,
                Ok(water::sparge_acid_mph_ours(
                    c.f("hco3"),
                    c.f("ph_w"),
                    c.f("ph_t"),
                    acid(c),
                )),
            ),
            "sparge_acid_bw" => check_sparge(
                c,
                Ok(water::sparge_acid_bw(
                    c.f("hco3"),
                    c.f("ph_w"),
                    c.f("ph_t"),
                    acid(c),
                )),
            ),
            "lactic_ml" => check_sparge(
                c,
                Ok(water::sparge_acid_ml(
                    c.f("mol_l"),
                    c.f("v_l"),
                    c.f("conc"),
                    water::Acid::Lactic,
                )),
            ),
            "grist_ph_typed" => {
                let g = water::grist_ph_typed(&typed_bill(c), c.b("ours"));
                let (ph, sum) = (g.ph, g.s);
                c.check(&json!([ph, sum]));
            }
            "to_grams" => c.check_r((|| {
                water::to_grams(c.f("amount"), MiscUnit::from_code(c.i("units"))?)
            })()),
            "adjusted_ph_liquid" => {
                c.check_r(water::adjusted_ph_liquid(&water::AdjustedPhLiquidInput {
                    grist_ph: c.f("grist_ph"),
                    ra_ppm: c.f("ra"),
                    t_l_per_kg: c.f("t"),
                    model: model(c),
                    w_kg: c.f("w_kg"),
                    acid: acid(c),
                    conc_pct: c.f("conc"),
                    vol_l: c.f("vol_l"),
                    target_ph: c.f("target_ph"),
                    ours: c.b("ours"),
                }));
            }
            "acid_malt_amount_checked" => {
                let r = water::acid_malt_amount(&water::AcidMaltInput {
                    bill: &typed_bill(c),
                    acid_pct: c.f("pct"),
                    model: model(c),
                    ra_ppm: c.f("ra"),
                    t_l_per_kg: c.f("t"),
                    target_ph: c.f("target_ph"),
                    ours: c.b("ours"),
                });
                c.check_res(r.map(|r| json!([r.total_g, r.ph, r.converged])));
            }
            "sparge_volume_l" => {
                c.check_f(water::sparge_volume_l(
                    c.f("total_l"),
                    c.f("w_kg"),
                    c.f("t"),
                ));
            }
            "warnings" => {
                let w: Vec<&str> = water::warnings(model(c), &salts(c.v("salt_grams")))
                    .into_iter()
                    .map(warning_text)
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
                let n = water::nnls(&cols, &nums(c.v("b")));
                let (x, rnorm) = (n.x, n.residual_norm);
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
                let m = water::water_match(&base, &ions(c.v("target")), s, &excluded);
                let (x, rnorm) = (m.grams_per_gal, m.residual_norm);
                let profile = water::best_fit_profile(&base, s, &x);
                assert_kkt(c, &base, s, &excluded, &x);
                c.check(&json!({"profile": profile, "rnorm": rnorm}));
            }
            _ => return false,
        }
        true
    });
}

/// T1.16: non-finite input and unknown codes are errors.
#[test]
fn invalid_inputs_are_errors() {
    assert!(Acid::from_code(9).is_err());
    assert!(MashModel::from_code(2).is_err());
    assert!(MiscUnit::from_code(8).is_err());
    assert!(water::to_grams(f64::NAN, MiscUnit::G).is_err());
    assert!(
        water::boil_scale(&water::BoilScaleInput {
            vb_l: f64::INFINITY,
            vp_l: 24.0,
            cool_loss_l: 1.0,
            trub_loss_l: 1.0,
            top_up_l: 0.0,
            has_boil_ing: true
        })
        .is_err()
    );
    assert!(
        water::mash_acid_amount(&water::MashAcidAmountInput {
            est_ph: 5.8,
            target_ph: 5.4,
            model: MashModel::Bw,
            t_l_per_kg: 0.0,
            w_kg: 5.0,
            acid: Acid::Lactic,
            conc_pct: 88.0,
            ours: false
        })
        .is_err()
    );
}

/// Display text of the reference (not part of the engine, ADR-0004 madde 7).
fn warning_text(w: water::Warning) -> &'static str {
    match w {
        water::Warning::BwModelUnverified => {
            "BW modeli doğrulanacak: MPH'ye göre yaklaşık yarı asit önerir (ölçülmüş mash pH ile kalibre edilene kadar)"
        }
        water::Warning::ChalkSolubility => {
            "çözünürlük düşük, asitli mash olmadan etkisi abartılı olabilir (tebeşir)"
        }
    }
}

/// Display text of the reference (not part of the engine, ADR-0004 madde 7).
fn balance_label(b: water::SulfateChlorideBalance) -> &'static str {
    use water::SulfateChlorideBalance as B;
    match b {
        B::OutOfRange => "Out of Range",
        B::ExtremelyMalty => "Extremely Malty",
        B::VeryMalty => "Very Malty",
        B::Malty => "Malty",
        B::Balanced => "Balanced",
        B::SlightlyBitter => "Slightly Bitter",
        B::Bitter => "Bitter",
        B::VeryBitter => "Very Bitter",
        B::ExtremelyBitter => "Extremely Bitter",
    }
}
