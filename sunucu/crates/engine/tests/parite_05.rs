//! Section 05 parity (mash, volumes) against `vektorler/05.json`.
#![allow(clippy::unwrap_used)] // vector shapes are fixed by the generator; a panic is the right failure
mod ortak;

use engine::density;
use engine::mash::{self, MiscFluidUnit, SpargeParams, StepInput, StepResult, ThicknessStep, Tun};
use engine::recipe::{GrainType, GrainUse, RecipeType, StepKind};
use engine::units;
use ortak::Case;
use serde_json::{Value, json};

fn opt(c: &Case, name: &str) -> Option<f64> {
    c.girdi.get(name).and_then(Value::as_f64)
}

fn or0(c: &Case, name: &str) -> f64 {
    opt(c, name).unwrap_or(0.0)
}

fn tun(c: &Case) -> Tun {
    Tun {
        mass_kg: or0(c, "m_tun"),
        specific_heat_cal: or0(c, "c_tun"),
        volume_l: or0(c, "v_tun"),
    }
}

fn sparge_params(c: &Case) -> SpargeParams {
    SpargeParams {
        sparge_l: c.f("S"),
        absorb_l: c.f("absorb"),
        grain_kg: c.f("W"),
        tun_l: c.f("tun_vol"),
        fill_pct: c.f("pct"),
        mash_water_l: c.f("mash_water"),
        even: c.b("even"),
        drain: c.b("drain"),
    }
}

fn opt_json(v: Option<f64>) -> Value {
    v.map_or(Value::Null, |x| json!(x))
}

fn kind(code: i64) -> StepKind {
    StepKind::from_code(code).unwrap()
}

fn tun_caps(c: &Case) -> Tun {
    Tun {
        mass_kg: or0(c, "m_tun"),
        specific_heat_cal: or0(c, "c_tun"),
        volume_l: or0(c, "V_tun"),
    }
}

fn steps_in(c: &Case) -> Vec<StepInput> {
    c.v("steps")
        .as_array()
        .unwrap()
        .iter()
        .map(|s| StepInput {
            kind: kind(s["type"].as_i64().unwrap()),
            infusion_l: s["infusion"].as_f64().unwrap(),
            step_c: s["t_step"].as_f64().unwrap(),
        })
        .collect()
}

fn result_json(r: &StepResult) -> Value {
    json!({
        "infusion": r.infusion_l,
        "tun_addition": r.tun_addition_l,
        "tot_infusion": r.tot_infusion_l,
        "start_vol": r.start_vol_l,
        "start_temp": r.start_c,
        "infusion_temp": opt_json(r.infusion_c),
        "decoction_amt": r.decoction_l,
        "above_boiling": r.above_boiling,
        "no_water": r.no_water,
    })
}

fn triples(c: &Case, name: &str) -> engine::Result<Vec<(GrainUse, GrainType, f64)>> {
    c.v(name)
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            Ok((
                GrainUse::from_code(e[0].as_i64().unwrap())?,
                GrainType::from_code(e[1].as_i64().unwrap())?,
                e[2].as_f64().unwrap(),
            ))
        })
        .collect()
}

fn floats(c: &Case, name: &str) -> Vec<f64> {
    c.v(name)
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_f64().unwrap())
        .collect()
}

/// Cases for the functions added in T1.14b; `false` when `fonk` is unknown.
#[allow(clippy::too_many_lines)]
fn extra(c: &Case) -> bool {
    match c.fonk.as_str() {
        "grain_vol" => c.check_f(mash::grain_vol(c.f("W"))),
        "mash_vol" => c.check_f(mash::mash_vol(c.f("Vsu"), c.f("W"))),
        "grain_absorb" => c.check_f(mash::grain_absorb(
            c.f("W_mash"),
            c.girdi
                .get("biab")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        )),
        "sparge_absorb" => c.check_f(mash::sparge_absorb(
            c.f("W_sparge"),
            c.girdi
                .get("biab")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        )),
        "mash_grain_weight" => c.check_r(triples(c, "grains").map(|g| mash::mash_grain_weight(&g))),
        "sparge_grain_weight" => {
            c.check_r(triples(c, "grains").map(|g| mash::sparge_grain_weight(&g)));
        }
        "tun_vol_needed" => c.check_f(mash::tun_vol_needed(c.f("mash_tot_water"), c.f("W_mash"))),
        "tun_vol_needed_recipe" => c.check_f(mash::tun_vol_needed_recipe(
            c.f("mash_tot_water"),
            c.f("W_mash"),
            c.f("W_sparge"),
        )),
        "step_tot_infusion" => c.check_f(mash::step_tot_infusion(
            c.f("infusion"),
            c.f("tun_addition"),
        )),
        "mash_tot_water" => {
            let steps: Vec<(StepKind, f64, f64)> = c
                .v("steps")
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    (
                        kind(e[0].as_i64().unwrap()),
                        e[1].as_f64().unwrap(),
                        e[2].as_f64().unwrap(),
                    )
                })
                .collect();
            c.check_f(mash::mash_tot_water(&steps));
        }
        "mash_avail_water" => c.check_f(mash::mash_avail_water(
            c.f("tot_water"),
            c.f("absorb"),
            c.f("tun_deadspace"),
        )),
        "infusion_temp_checked" => {
            let r = mash::infusion_temp_checked(&mash::InfusionTempInput {
                added_l: c.f("Va"),
                target_c: c.f("T2"),
                mash_water_l: c.f("Vw"),
                mash_c: c.f("T1"),
                grain_kg: c.f("W"),
                tun: tun_caps(c),
            });
            c.check(&r.map_or(Value::Null, |x| json!([x.temp_c, x.above_boiling])));
        }
        "update_infusion_temp" => c.check(&opt_json(mash::update_infusion_temp(
            &mash::UpdateInfusionTempInput {
                kind: kind(c.i("kind")),
                infusion_l: c.f("infusion"),
                tun_addition_l: c.f("tun_addition"),
                step_c: c.f("t_step"),
                start_c: c.f("start_temp"),
                start_vol_l: c.f("start_vol"),
                grain_kg: c.f("W"),
                grain_c: c.f("T_grain"),
                tun: tun_caps(c),
                tun_c: opt(c, "T_tun"),
            },
        ))),
        "recalc_mash" => {
            let out: Vec<Value> = mash::recalc_mash(&mash::RecalcMashInput {
                steps: &steps_in(c),
                grain_kg: c.f("W"),
                grain_c: c.f("T_grain"),
                tun: tun_caps(c),
                tun_c: opt(c, "T_tun"),
                deadspace_l: or0(c, "deadspace"),
                boil_c: opt(c, "Tb").unwrap_or(100.0),
            })
            .iter()
            .map(result_json)
            .collect();
            c.check(&Value::Array(out));
        }
        "recalc_biab" => {
            let out: Vec<Value> = mash::recalc_biab(&mash::RecalcBiabInput {
                steps: &steps_in(c),
                grain_kg: c.f("W"),
                grain_c: c.f("T_grain"),
                biab_vol_l: c.f("biab_vol"),
                tun: tun_caps(c),
                tun_c: opt(c, "T_tun"),
                boil_c: opt(c, "Tb").unwrap_or(100.0),
            })
            .iter()
            .map(result_json)
            .collect();
            c.check(&Value::Array(out));
        }
        "tun_deadspace_profile" => c.check_f(mash::tun_deadspace_profile(
            c.f("tun_addition"),
            c.b("adj_deadspace"),
            c.f("tun_deadspace"),
        )),
        "scale_from_grain_weight" => {
            let r = mash::scale_from_grain_weight(
                &floats(c, "infusions"),
                c.f("W_old"),
                c.f("W_new"),
                c.girdi
                    .get("biab")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            );
            c.check(&json!(r));
        }
        "step_water_grain" => c.check_f(mash::step_water_grain(
            c.f("start_vol"),
            c.f("infusion"),
            c.f("W"),
            c.b("decoction"),
        )),
        "scale_water_grain" => c.check_f(mash::scale_water_grain(
            c.f("r"),
            c.f("W"),
            c.f("start_vol"),
            c.f("infusion_old"),
        )),
        "main_mash_thickness" => {
            let steps: Vec<ThicknessStep> = c
                .v("steps")
                .as_array()
                .unwrap()
                .iter()
                .map(|s| ThicknessStep {
                    grain_kg: s["w"].as_f64().unwrap(),
                    step_c: s["t_step"].as_f64().unwrap(),
                    start_vol_l: s["start_vol"].as_f64().unwrap(),
                    tot_infusion_l: s["tot_infusion"].as_f64().unwrap(),
                    decoction: s["decoction"].as_bool().unwrap(),
                })
                .collect();
            c.check_f(mash::main_mash_thickness(
                &steps,
                c.f("tot_water"),
                c.f("W_total"),
            ));
        }
        "biab_mash_vol" => c.check_f(mash::biab_mash_vol(
            c.f("v_preboil"),
            c.f("tun_deadspace"),
            c.f("topup_kettle"),
            c.f("W_mash"),
        )),
        "sparge_vol" => c.check_f(mash::sparge_vol(&mash::SpargeVolInput {
            v_preboil_l: c.f("v_preboil"),
            absorb_l: c.f("absorb"),
            sparge_absorb_l: c.f("sparge_absorb_"),
            tun_deadspace_l: c.f("tun_deadspace"),
            topup_kettle_l: c.f("topup_kettle"),
            mash_tot_water_l: c.f("mash_tot_water_"),
        })),
        "tot_boil_off" => c.check_f(mash::tot_boil_off(
            c.f("boil_off"),
            c.f("boil_time_min"),
            c.b("rate_flag"),
        )),
        "post_boil_vol" => c.check_f(mash::post_boil_vol(c.f("v_preboil"), c.f("boil_off_total"))),
        "cool_loss" => c.check_f(mash::cool_loss(c.f("cool_pct"), c.f("v_postboil"))),
        "pre_trub_vol" => c.check_f(mash::pre_trub_vol(c.f("batch"), c.f("trub"))),
        "evap_rate" => c.check_f(mash::evap_rate(
            c.f("boil_off_total"),
            c.f("v_preboil"),
            c.f("boil_time_min"),
        )),
        "bottle_vol_equip" => {
            c.check_f(mash::bottle_vol_equip(c.f("batch"), c.f("fermenter_loss")));
        }
        "bottle_vol_recipe" => c.check_f(mash::bottle_vol_recipe(&mash::BottleVolRecipeInput {
            batch_l: c.f("batch"),
            fermenter_loss_l: c.f("fermenter_loss"),
            include_starter: c.b("include_starter"),
            starter_l: c.f("starter"),
            use_second: c.b("use_second"),
            second_starter_l: c.f("second_starter"),
        })),
        "total_water" => c.check_r((|| {
            Ok(mash::total_water(&mash::TotalWaterInput {
                recipe_type: RecipeType::from_code(c.i("recipe_type"))?,
                has_boil_ing: c.b("has_boil_ing"),
                mash_tot_water_l: c.f("mash_tot_water_"),
                sparge_l: c.f("sparge"),
                topup_kettle_l: c.f("topup_kettle"),
                topup_ferm_l: c.f("topup_ferm"),
                batch_l: c.f("batch"),
                v_preboil_l: c.f("v_preboil"),
            }))
        })()),
        "total_grain_use" => c.check_r((|| {
            let items: engine::Result<Vec<(GrainUse, f64)>> = c
                .v("items")
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    Ok((
                        GrainUse::from_code(e[0].as_i64().unwrap())?,
                        e[1].as_f64().unwrap(),
                    ))
                })
                .collect();
            Ok(mash::total_grain_use(
                &items?,
                GrainUse::from_code(c.i("use"))?,
            ))
        })()),
        "steep_vol" => c.check_f(mash::steep_vol(
            c.f("v_preboil"),
            c.f("v_boilgrains"),
            c.f("W_steep"),
        )),
        "boil_water_add" => c.check_f(mash::boil_water_add(
            c.f("v_preboil"),
            c.f("v_steep"),
            c.f("v_boilgrains"),
        )),
        "primary_water_add" => c.check_r((|| {
            Ok(mash::primary_water_add(&mash::PrimaryWaterAddInput {
                recipe_type: RecipeType::from_code(c.i("recipe_type"))?,
                has_boil_ing: c.b("has_boil_ing"),
                batch_l: c.f("batch"),
                v_postboil_l: c.f("v_postboil"),
                v_primarygrains_l: c.f("v_primarygrains"),
                topup_ferm_l: c.f("topup_ferm"),
            }))
        })()),
        "preboil_sg" => c.check_r(mash::preboil_sg(
            c.f("og"),
            c.f("batch"),
            c.f("trub"),
            c.f("v_preboil"),
        )),
        "volume_chain" => {
            let r = mash::volume_chain(&mash::VolumeChainInput {
                batch_l: c.f("batch"),
                topup_ferm_l: c.f("topup_ferm"),
                trub_l: c.f("trub"),
                cool_pct: c.f("cool_pct"),
                boil_off_l: c.f("boil_off"),
                boil_time_min: c.f("boil_time_min"),
                rate_flag: c.b("rate_flag"),
                fermenter_loss_l: c.f("fermenter_loss"),
            });
            c.check(&json!({
                "boil_off": r.boil_off_l, "preboil": r.preboil_l, "postboil": r.postboil_l,
                "cool_loss": r.cool_loss_l, "cold_wort": r.cold_wort_l, "pretrub": r.pretrub_l,
                "evap_pct": r.evap_pct, "bottle": r.bottle_l,
            }));
        }
        "wn_cool_loss" => c.check_r(mash::wn_cool_loss(
            c.f("v_ferm"),
            c.f("topup"),
            c.f("cool_pct"),
        )),
        "wn_boil_vol" => c.check_f(mash::wn_boil_vol(&mash::WnBoilVolInput {
            v_ferm_l: c.f("v_ferm"),
            topup_l: c.f("topup"),
            trub_l: c.f("trub"),
            v_cool_l: c.f("v_cool"),
            boil_off_l: c.f("boil_off"),
        })),
        "wn_boil_pct" => c.check_f(mash::wn_boil_pct(c.f("boil_off"), c.f("v_boil"))),
        "wn_grain_absorb" => c.check_f(mash::grain_absorb(c.f("W"), false)),
        "wn_sparge_water" => c.check_f(mash::wn_sparge_water(&mash::WnSpargeWaterInput {
            v_boil_l: c.f("v_boil"),
            absorb_l: c.f("absorb"),
            topup_kettle_l: c.f("topup_kettle"),
            lauter_deadspace_l: c.f("lauter_deadspace"),
            v_mash_l: c.f("v_mash"),
        })),
        "wn_total_water" => c.check_r((|| {
            Ok(mash::wn_total_water(&mash::WnTotalWaterInput {
                brew_type: RecipeType::from_code(c.i("brew_type"))?,
                v_boil_l: c.f("v_boil"),
                topup_l: c.f("topup"),
                v_sparge_l: c.f("v_sparge"),
                v_mash_l: c.f("v_mash"),
                topup_kettle_l: c.f("topup_kettle"),
            }))
        })()),
        "wn_bottle_vol" => c.check_f(mash::wn_bottle_vol(c.f("v_ferm"), c.f("v_loss_ferm"))),
        "boil_off_tool" => {
            let r = mash::boil_off_tool(&mash::BoilOffToolInput {
                v0_l: c.f("v0"),
                evap_pct: c.f("evap_pct"),
                boil_time_min: c.f("boil_time_min"),
                cool_pct: c.f("cool_pct"),
                sg0: c.f("sg0"),
            });
            c.check(&json!({
                "rate": r.rate_l_per_h, "boil_off": r.boil_off_l, "cool_loss": r.cool_loss_l,
                "v_end": r.v_end_l, "sg_end": r.sg_end,
            }));
        }
        "dilution" => {
            let r = mash::dilution(&mash::DilutionInput {
                sg0: c.f("sg0"),
                v0_l: c.f("v0"),
                v_add_l: c.f("v_add"),
                wort: c.b("wort"),
                sg_add: opt(c, "sg_add").unwrap_or(1.0),
            });
            c.check_res(r.map(|r| json!({"v_tot": r.v_tot_l, "sg": r.sg})));
        }
        "gravity_adj_water_vol" => c.check_r(density::gravity_adjust_water(
            c.f("sg"),
            c.f("sg_target"),
            c.f("v"),
        )),
        "misc_to_fluid_oz" => c.check_r((|| {
            mash::misc_to_fluid_oz(c.f("amount"), MiscFluidUnit::from_code(c.i("units"))?)
        })()),
        _ => return false,
    }
    true
}

#[test]
fn parite_05() {
    ortak::run("05", |c| {
        match c.fonk.as_str() {
            "constant" => {
                let v = match c.v("name").as_str().unwrap() {
                    "FLOZ_L" => units::FLOZ_L,
                    "LB_KG" => units::LB_KG,
                    "OZ_KG" => mash::OZ_KG,
                    "GRAIN_VOL" => mash::GRAIN_VOL,
                    "C_MALT" => mash::C_MALT,
                    "ABS_STD" => mash::ABS_STD,
                    "ABS_BIAB" => mash::ABS_BIAB,
                    _ => return false,
                };
                c.check_f(v);
            }
            "tun_eff" => c.check_f(mash::tun_eff(c.f("m_tun"), c.f("v_mash"), c.f("v_tun"))),
            "strike_temp" => c.check(&opt_json(mash::strike_temp(&mash::StrikeTempInput {
                water_l: c.f("vw"),
                step_c: c.f("t_step"),
                grain_kg: c.f("w"),
                grain_c: c.f("t_grain"),
                tun: tun(c),
                tun_c: opt(c, "t_tun"),
            }))),
            "infusion_temp" => c.check(&opt_json(mash::infusion_temp(&mash::InfusionTempInput {
                added_l: c.f("va"),
                target_c: c.f("t2"),
                mash_water_l: c.f("vw"),
                mash_c: c.f("t1"),
                grain_kg: c.f("w"),
                tun: tun(c),
            }))),
            "infusion_amount" => c.check_f(mash::infusion_amount(&mash::InfusionAmountInput {
                inf_c: c.f("tinf"),
                target_c: c.f("t2"),
                mash_water_l: c.f("vw"),
                mash_c: c.f("t1"),
                grain_kg: c.f("w"),
                tun: tun(c),
            })),
            "decoct_vol" => c.check_f(mash::decoct_vol(&mash::DecoctVolInput {
                target_c: c.f("t2"),
                mash_water_l: c.f("vw"),
                mash_c: c.f("t1"),
                grain_kg: c.f("w"),
                tun: tun(c),
                boil_c: opt(c, "tb").unwrap_or(100.0),
            })),
            "boil_temp_c" => c.check_f(units::boil_temp_c(c.f("alt_ft"))),
            "equip_boil_vol" => c.check_f(mash::equip_boil_vol(&mash::EquipBoilVolInput {
                batch_l: c.f("batch"),
                topup_l: c.f("topup"),
                trub_l: c.f("trub"),
                cool_pct: c.f("cool_pct"),
                boil_off_total_l: c.f("boil_off_total"),
            })),
            "batch_sparge" => {
                let i = u32::try_from(c.i("i")).unwrap();
                let s = mash::batch_sparge(i, &sparge_params(c));
                c.check(&json!([s.volume_l, s.drain]));
            }
            "sparge_plan" => {
                let steps: Vec<Value> = mash::sparge_plan(&sparge_params(c))
                    .iter()
                    .map(|s| {
                        let action = match s.action {
                            mash::SpargeAction::Drain => "drain",
                            mash::SpargeAction::Add => "add",
                        };
                        json!([s.index, s.volume_l, action])
                    })
                    .collect();
                c.check(&Value::Array(steps));
            }
            "parti" => {
                let p = mash::parti(c.f("og"), c.f("vol"), c.f("f1"));
                c.check(&json!({"vol1": p.vol1_l, "vol2": p.vol2_l, "og1": p.og1, "og2": p.og2}));
            }
            _ => return extra(c),
        }
        true
    });
}

/// T1.16: degenerate input is an error, not NaN or infinity.
#[test]
fn invalid_inputs_are_errors() {
    assert!(StepKind::from_code(3).is_err());
    assert!(MiscFluidUnit::from_code(13).is_err());
    assert!(
        mash::dilution(&mash::DilutionInput {
            sg0: 1.050,
            v0_l: 0.0,
            v_add_l: 0.0,
            wort: false,
            sg_add: 1.0
        })
        .is_err()
    );
    assert!(density::gravity_adjust_water(1.050, 1.000, 20.0).is_err());
    assert!(mash::wn_cool_loss(20.0, 0.0, 100.0).is_err());
    assert!(mash::preboil_sg(1.050, 20.0, 1.0, 0.0).is_err());
    assert!(
        mash::dilution(&mash::DilutionInput {
            sg0: f64::NAN,
            v0_l: 10.0,
            v_add_l: 1.0,
            wort: false,
            sg_add: 1.0
        })
        .is_err()
    );
}
