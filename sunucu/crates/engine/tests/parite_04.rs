//! Section 04 parity: yeast, mead, carbonation against `vektorler/04.json`.
#![allow(clippy::unwrap_used, clippy::too_many_lines)]
mod ortak;

use engine::density;
use engine::recipe::{Yeast as RecipeYeast, YeastForm};
use engine::units;
use engine::yeast::{
    self, BackAgent, CarbAmount, CarbType, MeadFerm, MeadMethod, PitchTargets, PrimingAgent,
    StarterType, StarterYeastForm, YanMethod, YeastNitrogen, YeastType, YeastUnit,
};
use serde_json::{Value, json};

fn yeasts(c: &ortak::Case) -> Vec<RecipeYeast> {
    c.v("yeasts")
        .as_array()
        .unwrap()
        .iter()
        .map(|y| {
            let f = |k: &str| y[k].as_f64().unwrap();
            let i = |k: &str| y[k].as_i64().unwrap();
            RecipeYeast {
                min_attenuation_pct: 0.0,
                max_attenuation_pct: 0.0,
                tolerance_abv_pct: 0.0,
                add_to_secondary: y["secondary"].as_bool().unwrap(),
                total_cells_billion: f("tot_cells"),
                viable_cells_billion: f("viable_cells"),
                packs: f("new_amount"),
                viability_pct: f("viability_pct"),
                form: YeastForm::from_code(i("form")).unwrap(),
                use_packages: i("use_pkgs") == 1,
                amount_oz: f("amount_oz"),
                cells_per_ml_billion: f("cells_ml"),
                dry_pitch_rate_g_per_hl: f("dry_pitch_rate"),
            }
        })
        .collect()
}

fn yform(c: &ortak::Case) -> engine::Result<YeastForm> {
    YeastForm::from_code(c.i("form"))
}

#[test]
fn parite_04() {
    ortak::run("04", |c| {
        match c.fonk.as_str() {
            "l_to_floz" => c.check_f(units::l_to_floz(c.f("litres"))),
            "gal_from_l" => c.check_f(units::gal_from_l(c.f("litres"))),
            "floz_to_l" => c.check_f(units::floz_to_l(c.f("floz"))),
            "brix_to_sg" => c.check_f(density::brix_to_sg(c.f("brix"))),
            "viability" => c.check_f(yeast::viability(
                c.f("v0"),
                c.f("rate"),
                c.f("days"),
                c.f("floor"),
            )),
            "innoc_rate" => c.check_r(yeast::innoc_rate(c.f("g"))),
            "growth_rate" => c.check_r(yeast::growth_rate(c.f("i"), c.b("stir"))),
            "cells_needed" => {
                c.check_f(yeast::cells_needed(c.f("rate"), c.f("volume_l"), c.f("sg")));
            }
            "new_starter_l" => {
                c.check_r(yeast::new_starter_l(c.f("need"), c.f("avail"), c.b("stir")));
            }
            "two_stage_tool" => {
                c.check_res(
                    yeast::two_stage_tool(c.f("need"), c.f("avail"), c.b("stir"))
                        .map(|s| json!([s.stage1_l, s.stage2_l])),
                );
            }
            "dme_oz" => c.check_f(yeast::dme_oz(c.f("vol_floz"), c.f("og"))),
            "sugar_g" => c.check_f(yeast::sugar_g(
                c.f("vols"),
                c.f("temp_f"),
                c.f("gal"),
                c.b("dme"),
            )),
            "residual_co2" => c.check_f(yeast::residual_co2(c.f("temp_f"))),
            "co2_psi" => c.check_f(yeast::co2_psi(c.f("vols"), c.f("temp_f"))),
            "so2_rec" => c.check_f(yeast::so2_rec(c.f("ph"))),
            "kmeta_g" => c.check_f(yeast::kmeta_g(c.f("v_gal"), c.f("target"), c.f("free"))),
            "sorbate_g" => c.check_f(yeast::sorbate_g(c.f("v_gal"))),
            "mead_pitch" => c.check_f(yeast::mead_pitch(c.f("sg"))),
            "is_wine_like_recipe" => c.check(&json!(yeast::is_wine_like_recipe(c.i("r_type")))),
            "yan" => c.check_r((|| {
                Ok(yeast::yan(
                    YanMethod::from_code(c.i("method"))?,
                    c.f("grams"),
                    c.f("v_l"),
                ))
            })()),
            // Display names are the client's; the engine only maps codes (valid or not).
            "enum_name" => {
                let code = c.i("code");
                let valid = match c.v("kind").as_str().unwrap() {
                    "y_form" => YeastForm::from_code(code).is_ok(),
                    "yc_yeast_form" => yeast::StarterYeastForm::from_code(code).is_ok(),
                    "yc_type" => yeast::StarterType::from_code(code).is_ok(),
                    "y_type" => yeast::YeastType::from_code(code).is_ok(),
                    "r_type" => engine::recipe::RecipeType::from_code(code).is_ok(),
                    "r_ale_type" => yeast::AleType::from_code(code).is_ok(),
                    "mead_method" => yeast::MeadMethod::from_code(code).is_ok(),
                    "yeast_factor" => yeast::YeastNitrogen::from_code(code).is_ok(),
                    "carb_type" => yeast::CarbType::from_code(code).is_ok(),
                    "back_agent" => yeast::BackAgent::from_code(code).is_ok(),
                    other => panic!("unknown enum kind {other}"),
                };
                assert_eq!(valid, !c.beklenen.is_null(), "enum_name case {}", c.index);
            }
            "default_value" => c.check(
                &yeast::default_value(c.v("name").as_str().unwrap())
                    .map_or(Value::Null, |n| json!(n)),
            ),
            "age_days" => c.check(&json!(yeast::age_days(
                c.i("pkg_day"),
                c.i("in_recipe") != 0,
                c.i("brew_day"),
                c.i("today")
            ))),
            "yeast_age_tool" => c.check_f(yeast::yeast_age_tool(c.i("today"), c.i("date_day"))),
            "yeast_age_recipe" => {
                let d: Vec<i64> = c
                    .v("days")
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_i64().unwrap())
                    .collect();
                c.check_f(yeast::yeast_age_recipe(&d));
            }
            "v0_ingredient" => c.check_r((|| Ok(yeast::viability_v0_ingredient(yform(c)?)))()),
            "v0_tool" => c.check_r((|| {
                Ok(yeast::viability_v0_tool(StarterYeastForm::from_code(
                    c.i("form"),
                )?))
            })()),
            "import_age_rate" => c.check_r((|| {
                Ok(yeast::import_age_rate(yform(c)?, c.b("white_labs")))
            })()),
            "tool_viability" => c.check_f(yeast::tool_viability(&yeast::ToolViabilityInput {
                calc: c.b("calc"),
                user_viability_pct: c.f("user_v"),
                v0_pct: c.f("v0"),
                rate_pct_per_month: c.f("rate"),
                days: c.f("days"),
            })),
            "viable_cells" => c.check_f(yeast::viable_cells(c.f("total"), c.f("v"))),
            "yeast_total_cells" => c.check_r((|| {
                Ok(yeast::yeast_total_cells(&yeast::YeastTotalCellsInput {
                    use_pkgs: c.b("use_pkgs"),
                    cells_pkg_billion: c.f("cells_pkg"),
                    new_amount: c.f("new_amount"),
                    form: yform(c)?,
                    amount_g_or_ml: c.f("amount"),
                    cells_ml_billion: c.f("cells_ml"),
                }))
            })()),
            "yeast_cells_avail" => c.check_f(yeast::yeast_cells_avail(
                c.b("dry"),
                c.f("dry_g"),
                c.f("wet_ml"),
                c.f("cells_ml"),
            )),
            "cells_no_starter" => c.check_f(yeast::yeast_cells_no_starter(
                c.b("calc_viability"),
                &yeasts(c),
            )),
            "pack_count" => c.check_f(yeast::yeast_pack_count(&yeasts(c))),
            "pack_viability" => c.check_f(yeast::yeast_pack_viability(&yeasts(c))),
            "tot_grams" => c.check_f(yeast::yeast_tot_grams(&yeasts(c))),
            "first_fields" => {
                let f = yeast::yeast_first_fields(&yeasts(c));
                c.check(&json!([
                    f.form.code(),
                    i64::from(f.use_packages),
                    f.cells_per_ml_billion,
                    f.dry_pitch_rate_g_per_hl
                ]));
            }
            "avg_pack_size" => c.check_f(yeast::yeast_avg_pack_size(
                c.f("cells"),
                c.f("amount_sum"),
                c.b("valid"),
            )),
            "recommended_liquid_pitch" => c.check_f(yeast::recommended_liquid_pitch(
                c.i("r_type"),
                c.i("ale_type"),
                &PitchTargets::default(),
            )),
            "pitch_min_max_recipe" => {
                let r = yeast::pitch_min_max_recipe(c.f("rec"));
                c.check(&json!([r.min_m_ml_p, r.max_m_ml_p]));
            }
            "pitch_min_max_tool" => c.check_res((|| {
                let r = yeast::pitch_min_max_tool(
                    StarterType::from_code(c.i("tool_type"))?,
                    &PitchTargets::default(),
                );
                Ok(json!([r.min_m_ml_p, r.max_m_ml_p]))
            })()),
            "pitch_range" => c.check(&json!(format!("{:.2}-{:.2}", c.f("lo"), c.f("hi")))),
            "dry_min_pitch" => c.check_f(yeast::dry_min_pitch_rate(c.f("rec_dry"))),
            "dry_max_pitch" => c.check_f(yeast::dry_max_pitch_rate(c.f("rec_dry"))),
            "dry_pitch_range" => c.check(&json!(format!(
                "{:.1}-{:.1}",
                yeast::dry_min_pitch_rate(c.f("rec_dry")),
                yeast::dry_max_pitch_rate(c.f("rec_dry"))
            ))),
            "is_dry_by_weight" => c.check_res((|| {
                Ok(json!(yeast::is_dry_by_weight(
                    yform(c)?,
                    c.i("use_pkgs") == 1,
                    c.b("tool")
                )))
            })()),
            "tot_yeast_cells_needed" => {
                c.check_f(yeast::tot_yeast_cells_needed(&yeast::CellsNeededInput {
                    dry_by_weight: c.b("dry_by_weight"),
                    pitch_m_ml_p: c.f("pitch"),
                    v_l: c.f("v_l"),
                    sg: c.f("sg"),
                    dry_rate_g_per_hl: c.f("dry_rate"),
                    rec_dry_rate_g_per_hl: c.f("rec_dry_rate"),
                }));
            }
            "est_cells_ml" => c.check_f(yeast::est_cells_ml(c.f("n"), c.f("v_ml"))),
            "cells_starter" => c.check_r(yeast::cells_starter(
                c.f("viable"),
                c.f("starter_l"),
                c.b("stir"),
            )),
            "cells_sec_starter" => c.check_r(yeast::cells_sec_starter(
                c.f("cells1"),
                c.f("sec_l"),
                c.b("stir"),
            )),
            "two_stage_recipe" => {
                c.check_res(
                    yeast::two_stage_recipe(
                        c.f("need"),
                        c.f("avail"),
                        c.b("stir"),
                        c.f("growth_cap"),
                    )
                    .map(|s| json!([s.stage1_l, s.stage2_l])),
                );
            }
            "packs_no_starter" => {
                c.check_res(yeast::packs_no_starter(c.f("n"), c.f("pkt")).map(|n| json!(n)));
            }
            "packs_starter" => c.check_r(yeast::packs_starter(
                c.f("n"),
                c.f("pkt"),
                c.f("growth"),
                c.b("stir"),
            )),
            "calc_starter_size" => c.check_r(yeast::calc_yeast_starter_size(
                c.f("r"),
                c.f("pkt_size"),
                c.f("pkt"),
            )),
            "dry_grams_vol" => c.check_f(yeast::dry_grams_vol(c.f("dry_rate"), c.f("v_l"))),
            "dry_grams_pkg" => c.check_f(yeast::dry_grams_pkg(c.f("n"), c.f("cells_per_g"))),
            "packs_dry" => c.check_f(yeast::yeast_packs_dry(c.f("g"))),
            "dry5" => c.check_f(yeast::yeast_dry5(c.f("g"))),
            "dry_hydrate" => c.check_f(yeast::yeast_dry_hydrate(c.f("tot_g"), c.f("per_g"))),
            "goferm" => c.check_f(yeast::yeast_goferm(
                c.f("tot_g"),
                c.f("per_g"),
                c.f("goferm"),
            )),
            "avg_attenuation" => c.check_f(density::avg_attenuation(c.f("lo"), c.f("hi"))),
            "units_code" => c.check_res((|| {
                Ok(
                    match yeast::yeast_units(
                        c.b("use_pkgs"),
                        yform(c)?,
                        c.i("dry_unit"),
                        c.i("wet_unit"),
                    ) {
                        YeastUnit::Pkgs => json!("pkgs"),
                        YeastUnit::Code(k) => json!(k),
                    },
                )
            })()),
            "import_tolerance" => c.check_res((|| {
                Ok(json!(yeast::import_hook_tolerance(
                    c.i("tol"),
                    YeastType::from_code(c.i("y_type"))?
                )))
            })()),
            "mead_yeast_grams" => c.check_f(yeast::mead_yeast_grams(c.f("sg"), c.f("v_l"))),
            "mead_goferm" => c.check_f(yeast::mead_goferm(c.f("g"))),
            "mead_hydrate" => c.check_f(yeast::mead_hydrate(c.f("g"))),
            "mead_hydrate1" => c.check_f(yeast::mead_hydrate1(c.f("g"))),
            "yan_target" => c.check_r((|| {
                yeast::mead_yan_target(c.f("sg"), c.f("pct"), YeastNitrogen::from_code(c.i("fac"))?)
            })()),
            "mead_tot_ferm_o" => c.check_r((|| {
                yeast::mead_tot_ferm_o(&yeast::MeadFermInput {
                    sg: c.f("sg"),
                    v_gal: c.f("v_gal"),
                    yan_pct: c.f("pct"),
                    fac: YeastNitrogen::from_code(c.i("fac"))?,
                    method: MeadMethod::from_code(c.i("method"))?,
                })
            })()),
            "mead_tot_ferm_k" => c.check_r((|| {
                yeast::mead_tot_ferm_k(&yeast::MeadFermInput {
                    sg: c.f("sg"),
                    v_gal: c.f("v_gal"),
                    yan_pct: c.f("pct"),
                    fac: YeastNitrogen::from_code(c.i("fac"))?,
                    method: MeadMethod::from_code(c.i("method"))?,
                })
            })()),
            "mead_ferm_o" => c.check_r((|| {
                yeast::mead_ferm_o(&yeast::MeadFermInput {
                    sg: c.f("sg"),
                    v_gal: c.f("v_gal"),
                    yan_pct: c.f("pct"),
                    fac: YeastNitrogen::from_code(c.i("fac"))?,
                    method: MeadMethod::from_code(c.i("method"))?,
                })
            })()),
            "mead_ferm_k" => c.check_r((|| {
                yeast::mead_ferm_k(&yeast::MeadFermInput {
                    sg: c.f("sg"),
                    v_gal: c.f("v_gal"),
                    yan_pct: c.f("pct"),
                    fac: YeastNitrogen::from_code(c.i("fac"))?,
                    method: MeadMethod::from_code(c.i("method"))?,
                })
            })()),
            "mead_pct_honey" => {
                let fs: Vec<MeadFerm> = c
                    .v("ferms")
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|f| MeadFerm {
                        kind: f["type"].as_i64().unwrap(),
                        subtype: f["subtype"].as_i64().unwrap(),
                        pts: f["pts"].as_f64().unwrap(),
                    })
                    .collect();
                c.check_f(yeast::mead_pct_honey(&fs, c.f("primary_pts")));
            }
            "yan_equivalent" => c.check_r((|| {
                Ok(yeast::yan_equivalent(
                    YanMethod::from_code(c.i("method"))?,
                    c.f("yan"),
                ))
            })()),
            "mead_yan_added" => c.check_r((|| {
                Ok(yeast::mead_yan_added(
                    YanMethod::from_code(c.i("method"))?,
                    c.f("total_g"),
                    c.f("v_l"),
                    c.b("recipe"),
                ))
            })()),
            "mead_yan_equivalent" => c.check_r((|| {
                Ok(yeast::mead_yan_equivalent(
                    YanMethod::from_code(c.i("method"))?,
                    c.f("total_g"),
                    c.f("v_l"),
                    c.b("recipe"),
                ))
            })()),
            "nutrient_needed" => c.check_r((|| {
                Ok(yeast::nutrient_needed(
                    c.f("ppm"),
                    YanMethod::from_code(c.i("method"))?,
                    c.f("v_l"),
                ))
            })()),
            "back_agent_og" => c.check_r((|| {
                Ok(yeast::back_agent_og(BackAgent::from_code(c.i("agent"))?))
            })()),
            "back_agent_vol" => c.check_r((|| {
                Ok(yeast::back_agent_vol(
                    c.f("v_l"),
                    c.f("sg_bev"),
                    c.f("sg_target"),
                    BackAgent::from_code(c.i("agent"))?,
                ))
            })()),
            "back_agent_weight" => c.check_r((|| {
                Ok(yeast::back_agent_weight(
                    c.f("v_l"),
                    c.f("sg_bev"),
                    c.f("sg_target"),
                    BackAgent::from_code(c.i("agent"))?,
                ))
            })()),
            "back_agent_vol_sg" => c.check_f(yeast::backsweeten_volume_l(
                c.f("v_l"),
                c.f("sg_bev"),
                c.f("sg_target"),
                c.f("sg_a"),
            )),
            "honey_sg" => c.check_f(yeast::honey_sg(c.f("plato"))),
            "carb_sugar_agent" => c.check_r((|| {
                Ok(yeast::carb_sugar_agent(
                    PrimingAgent::from_code(c.i("agent"))?,
                    c.f("vols"),
                    c.f("temp_f"),
                    c.f("gal"),
                ))
            })()),
            "carb_amount" => c.check_res((|| {
                Ok(
                    match yeast::carb_amount(&yeast::CarbAmountInput {
                        c_type: CarbType::from_code(c.i("c_type"))?,
                        co2_vol: c.f("vols"),
                        temp_f: c.f("temp_f"),
                        volume_gal: c.f("gal"),
                        carb_rate_pct: c.f("carb_rate"),
                        agent: PrimingAgent::from_code(c.i("agent"))?,
                    }) {
                        CarbAmount::PressurePsi(x) => json!(["psi", x]),
                        CarbAmount::SugarG(x) => json!(["g", x]),
                    },
                )
            })()),
            "carb_equiv" => c.check_f(yeast::carb_equiv(c.f("rate"))),
            "carb_calories" => c.check_f(yeast::carb_calories(c.f("total"), c.f("alc"))),
            "carb_grams" => c.check_f(yeast::carb_grams(c.f("carb_cal"))),
            "convert_carbonation" => c.check_f(yeast::convert_carbonation(c.f("v"), c.b("to_gl"))),
            _ => return false,
        }
        true
    });
}

/// T1.16: invalid codes and degenerate numeric input are errors, not defaults or NaN.
#[test]
fn invalid_inputs_are_errors() {
    assert!(YanMethod::from_code(3).is_err());
    assert!(YeastNitrogen::from_code(7).is_err());
    assert!(PrimingAgent::from_code(5).is_err());
    assert!(yeast::growth_rate(0.0, false).is_err());
    assert!(yeast::growth_rate(f64::NAN, false).is_err());
    assert!(yeast::new_starter_l(100.0, 0.0, false).is_err());
    assert!(yeast::packs_no_starter(f64::NAN, 1.0).is_err());
    assert!(yeast::innoc_rate(-1.0).is_err());
}
