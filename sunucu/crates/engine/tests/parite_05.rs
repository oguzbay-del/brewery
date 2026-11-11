//! Section 05 parity (mash, volumes) against `vektorler/05.json`.
#![allow(clippy::unwrap_used)] // vector shapes are fixed by the generator; a panic is the right failure
mod ortak;

use engine::mash::{self, SpargeParams, StepInput, StepKind, StepResult, ThicknessStep, Tun};
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
        specific_heat: or0(c, "c_tun"),
        volume_l: or0(c, "v_tun"),
    }
}

fn sparge_params(c: &Case) -> SpargeParams {
    SpargeParams {
        sparge_vol: c.f("S"),
        absorb: c.f("absorb"),
        grain_kg: c.f("W"),
        tun_vol: c.f("tun_vol"),
        fill_pct: c.f("pct"),
        mash_water: c.f("mash_water"),
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
        specific_heat: or0(c, "c_tun"),
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
            infusion: s["infusion"].as_f64().unwrap(),
            t_step: s["t_step"].as_f64().unwrap(),
        })
        .collect()
}

fn result_json(r: &StepResult) -> Value {
    json!({
        "infusion": r.infusion,
        "tun_addition": r.tun_addition,
        "tot_infusion": r.tot_infusion,
        "start_vol": r.start_vol,
        "start_temp": r.start_temp,
        "infusion_temp": opt_json(r.infusion_temp),
        "decoction_amt": r.decoction_amt,
        "above_boiling": r.above_boiling,
    })
}

fn triples(c: &Case, name: &str) -> Vec<(i64, i64, f64)> {
    c.v(name)
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e[0].as_i64().unwrap(),
                e[1].as_i64().unwrap(),
                e[2].as_f64().unwrap(),
            )
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
        "mash_grain_weight" => c.check_f(mash::mash_grain_weight(&triples(c, "grains"))),
        "sparge_grain_weight" => c.check_f(mash::sparge_grain_weight(&triples(c, "grains"))),
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
            let r = mash::infusion_temp_checked(
                c.f("Va"),
                c.f("T2"),
                c.f("Vw"),
                c.f("T1"),
                c.f("W"),
                tun_caps(c),
            );
            c.check(&r.map_or(Value::Null, |x| json!([x.temp_c, x.above_boiling])));
        }
        "update_infusion_temp" => c.check(&opt_json(mash::update_infusion_temp(
            kind(c.i("kind")),
            c.f("infusion"),
            c.f("tun_addition"),
            c.f("t_step"),
            c.f("start_temp"),
            c.f("start_vol"),
            c.f("W"),
            c.f("T_grain"),
            tun_caps(c),
            opt(c, "T_tun"),
        ))),
        "recalc_mash" => {
            let out: Vec<Value> = mash::recalc_mash(
                &steps_in(c),
                c.f("W"),
                c.f("T_grain"),
                tun_caps(c),
                opt(c, "T_tun"),
                or0(c, "deadspace"),
                opt(c, "Tb").unwrap_or(100.0),
            )
            .iter()
            .map(result_json)
            .collect();
            c.check(&Value::Array(out));
        }
        "recalc_biab" => {
            let out: Vec<Value> = mash::recalc_biab(
                &steps_in(c),
                c.f("W"),
                c.f("T_grain"),
                c.f("biab_vol"),
                tun_caps(c),
                opt(c, "T_tun"),
                opt(c, "Tb").unwrap_or(100.0),
            )
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
                    w: s["w"].as_f64().unwrap(),
                    t_step: s["t_step"].as_f64().unwrap(),
                    start_vol: s["start_vol"].as_f64().unwrap(),
                    tot_infusion: s["tot_infusion"].as_f64().unwrap(),
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
        "sparge_vol" => c.check_f(mash::sparge_vol(
            c.f("v_preboil"),
            c.f("absorb"),
            c.f("sparge_absorb_"),
            c.f("tun_deadspace"),
            c.f("topup_kettle"),
            c.f("mash_tot_water_"),
        )),
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
        "bottle_vol_recipe" => c.check_f(mash::bottle_vol_recipe(
            c.f("batch"),
            c.f("fermenter_loss"),
            c.b("include_starter"),
            c.f("starter"),
            c.b("use_second"),
            c.f("second_starter"),
        )),
        "total_water" => c.check_f(mash::total_water(
            c.i("recipe_type"),
            c.b("has_boil_ing"),
            c.f("mash_tot_water_"),
            c.f("sparge"),
            c.f("topup_kettle"),
            c.f("topup_ferm"),
            c.f("batch"),
            c.f("v_preboil"),
        )),
        "total_grain_use" => {
            let items: Vec<(i64, f64)> = c
                .v("items")
                .as_array()
                .unwrap()
                .iter()
                .map(|e| (e[0].as_i64().unwrap(), e[1].as_f64().unwrap()))
                .collect();
            c.check_f(mash::total_grain_use(&items, c.i("use")));
        }
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
        "primary_water_add" => c.check_f(mash::primary_water_add(
            c.i("recipe_type"),
            c.b("has_boil_ing"),
            c.f("batch"),
            c.f("v_postboil"),
            c.f("v_primarygrains"),
            c.f("topup_ferm"),
        )),
        "preboil_sg" => c.check(&opt_json(mash::preboil_sg(
            c.f("og"),
            c.f("batch"),
            c.f("trub"),
            c.f("v_preboil"),
        ))),
        "volume_chain" => {
            let r = mash::volume_chain(
                c.f("batch"),
                c.f("topup_ferm"),
                c.f("trub"),
                c.f("cool_pct"),
                c.f("boil_off"),
                c.f("boil_time_min"),
                c.b("rate_flag"),
                c.f("fermenter_loss"),
            );
            c.check(&json!({
                "boil_off": r.boil_off, "preboil": r.preboil, "postboil": r.postboil,
                "cool_loss": r.cool_loss, "cold_wort": r.cold_wort, "pretrub": r.pretrub,
                "evap_pct": r.evap_pct, "bottle": r.bottle,
            }));
        }
        "wn_cool_loss" => c.check_f(mash::wn_cool_loss(
            c.f("v_ferm"),
            c.f("topup"),
            c.f("cool_pct"),
        )),
        "wn_boil_vol" => c.check_f(mash::wn_boil_vol(
            c.f("v_ferm"),
            c.f("topup"),
            c.f("trub"),
            c.f("v_cool"),
            c.f("boil_off"),
        )),
        "wn_boil_pct" => c.check_f(mash::wn_boil_pct(c.f("boil_off"), c.f("v_boil"))),
        "wn_grain_absorb" => c.check_f(mash::wn_grain_absorb(c.f("W"))),
        "wn_sparge_water" => c.check_f(mash::wn_sparge_water(
            c.f("v_boil"),
            c.f("absorb"),
            c.f("topup_kettle"),
            c.f("lauter_deadspace"),
            c.f("v_mash"),
        )),
        "wn_total_water" => c.check_f(mash::wn_total_water(
            c.i("brew_type"),
            c.f("v_boil"),
            c.f("topup"),
            c.f("v_sparge"),
            c.f("v_mash"),
            c.f("topup_kettle"),
        )),
        "wn_bottle_vol" => c.check_f(mash::wn_bottle_vol(c.f("v_ferm"), c.f("v_loss_ferm"))),
        "boil_off_tool" => {
            let r = mash::boil_off_tool(
                c.f("v0"),
                c.f("evap_pct"),
                c.f("boil_time_min"),
                c.f("cool_pct"),
                c.f("sg0"),
            );
            c.check(&json!({
                "rate": r.rate, "boil_off": r.boil_off, "cool_loss": r.cool_loss,
                "v_end": r.v_end, "sg_end": r.sg_end,
            }));
        }
        "dilution" => {
            let r = mash::dilution(
                c.f("sg0"),
                c.f("v0"),
                c.f("v_add"),
                c.b("wort"),
                opt(c, "sg_add").unwrap_or(1.0),
            );
            c.check(&json!({"v_tot": r.v_tot, "sg": r.sg}));
        }
        "gravity_adj_water_vol" => c.check_f(mash::gravity_adj_water_vol(
            c.f("v"),
            c.f("sg"),
            c.f("sg_target"),
        )),
        "misc_to_fluid_oz" => c.check(&opt_json(mash::misc_to_fluid_oz(
            c.f("amount"),
            c.i("units"),
        ))),
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
                    "FLOZ_L" => mash::FLOZ_L,
                    "LB_KG" => mash::LB_KG,
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
            "strike_temp" => c.check(&opt_json(mash::strike_temp(
                c.f("vw"),
                c.f("t_step"),
                c.f("w"),
                c.f("t_grain"),
                tun(c),
                opt(c, "t_tun"),
            ))),
            "infusion_temp" => c.check(&opt_json(mash::infusion_temp(
                c.f("va"),
                c.f("t2"),
                c.f("vw"),
                c.f("t1"),
                c.f("w"),
                tun(c),
            ))),
            "infusion_amount" => c.check_f(mash::infusion_amount(
                c.f("tinf"),
                c.f("t2"),
                c.f("vw"),
                c.f("t1"),
                c.f("w"),
                tun(c),
            )),
            "decoct_vol" => c.check_f(mash::decoct_vol(
                c.f("t2"),
                c.f("vw"),
                c.f("t1"),
                c.f("w"),
                tun(c),
                opt(c, "tb").unwrap_or(100.0),
            )),
            "boil_temp_c" => c.check_f(mash::boil_temp_c(c.f("alt_ft"))),
            "equip_boil_vol" => c.check_f(mash::equip_boil_vol(
                c.f("batch"),
                c.f("topup"),
                c.f("trub"),
                c.f("cool_pct"),
                c.f("boil_off_total"),
            )),
            "batch_sparge" => {
                let i = u32::try_from(c.i("i")).unwrap();
                let (v, flag) = mash::batch_sparge(i, &sparge_params(c));
                c.check(&json!([v, flag]));
            }
            "sparge_plan" => {
                let steps: Vec<Value> = mash::sparge_plan(&sparge_params(c))
                    .iter()
                    .map(|s| json!([s.index, s.volume, s.action.label()]))
                    .collect();
                c.check(&Value::Array(steps));
            }
            "parti" => {
                let p = mash::parti(c.f("og"), c.f("vol"), c.f("f1"));
                c.check(&json!({"vol1": p.vol1, "vol2": p.vol2, "og1": p.og1, "og2": p.og2}));
            }
            _ => return extra(c),
        }
        true
    });
}
