//! Section 11 parity: gravity triggers, keg priming, cask against `vektorler/11.json`.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::panic
)]

mod ortak;

use engine::trigger::{
    self, CaskPriming, FgSource, KegMode, KegParams, KegPriming, Stability, Sugar, TriggerError,
    TriggerForecast,
};
use engine::{units, yeast};
use serde_json::{Value, json};

fn num(v: &Value) -> f64 {
    v.as_f64().expect("number")
}

fn opt(c: &ortak::Case, name: &str) -> Option<f64> {
    c.girdi.get(name).and_then(Value::as_f64)
}

fn pairs(v: &Value) -> Vec<(f64, f64)> {
    v.as_array()
        .expect("array of pairs")
        .iter()
        .map(|p| (num(&p[0]), num(&p[1])))
        .collect()
}

fn sugar(c: &ortak::Case) -> Sugar {
    match c.girdi["kind"].as_str().expect("kind") {
        "dekstroz" => Sugar::Dextrose,
        "sakaroz" => Sugar::Sucrose,
        "invert" => Sugar::Invert,
        other => panic!("unknown sugar {other}"),
    }
}

fn err_json(e: TriggerError) -> Value {
    match e {
        TriggerError::EmptySeries => json!({"hata": "IndexError"}),
        TriggerError::ZeroDivision => json!({"hata": "ZeroDivisionError"}),
    }
}

fn result_json<T>(r: Result<T, TriggerError>, ok: impl FnOnce(T) -> Value) -> Value {
    match r {
        Ok(v) => ok(v),
        Err(e) => err_json(e),
    }
}

fn keg_json(k: &KegPriming) -> Value {
    let mut v = json!({
        "seker_g": k.sugar_g,
        "sise_esdegeri_g": k.bottle_equivalent_g,
        "kafa_payi_%": k.head_share_pct,
        "servis_basinci_bar": k.serving_bar,
        "tepe_vol": k.peak_vol,
        "tepe_basinci_bar": k.peak_bar,
        "tepe_toplam_gosterge_bar": k.peak_total_gauge_bar,
        "uyarilar": k.warnings,
    });
    if let Some(b) = k.co2_topup_bar {
        v["co2_tamamlama_bar"] = json!(b);
    }
    v
}

fn cask_json(k: &CaskPriming) -> Value {
    json!({
        "kalinti_vol": k.residual_vol,
        "ekstrakttan_vol": k.from_extract_vol,
        "eksik_vol": k.missing_vol,
        "seker_g": k.sugar_g,
        "g_L": k.g_per_l,
        "fazla": k.excess,
        "uyarilar": k.warnings,
    })
}

#[test]
fn parite_11() {
    ortak::run("11", |c| {
        match c.fonk.as_str() {
            "u" => c.check(&json!(units::gravity_units(c.f("sg")))),
            "f_from_c" => c.check_f(units::c_to_f(c.f("c"))),
            "to_plato" => c.check_f(units::to_plato(c.f("sg"))),
            "residual_vol" => c.check_f(yeast::residual_co2(units::c_to_f(c.f("t_c")))),
            "co2_psi" => c.check_f(yeast::co2_psi(c.f("vols"), units::c_to_f(c.f("t_c")))),
            "trigger_index" => {
                let n = usize::try_from(c.i("consecutive")).unwrap();
                let t =
                    trigger::trigger_index(&pairs(c.v("points")), c.f("fg_ref"), c.f("delta"), n);
                c.check(&json!(t));
            }
            "predicted_trigger" => {
                let out = result_json(
                    trigger::predicted_trigger(
                        c.f("fg_inf"),
                        c.f("a"),
                        c.f("k"),
                        c.f("fg_ref"),
                        c.f("delta"),
                    ),
                    |f| match f {
                        TriggerForecast::At(t) => json!(["tetik", t]),
                        TriggerForecast::AlreadyPast => json!(["gecmis", 0.0]),
                        TriggerForecast::Unreachable => json!(["erisilemez", null]),
                    },
                );
                c.check(&out);
            }
            "fg_reference" => {
                let (fg, src) = trigger::fg_reference(
                    c.f("fg_recipe"),
                    opt(c, "fg_inf"),
                    c.b("shown"),
                    c.f("delta"),
                );
                let s = match src {
                    FgSource::Recipe => "tarif",
                    FgSource::Forecast => "tahmin",
                };
                c.check(&json!([fg, s]));
            }
            "is_stable" => {
                let min_n = usize::try_from(c.i("min_n")).unwrap();
                let out = result_json(
                    trigger::is_stable(&pairs(c.v("points")), c.f("hours"), min_n),
                    |s| match s {
                        Stability::Insufficient => Value::Null,
                        Stability::Measured {
                            stable,
                            slope_per_day,
                        } => json!({"sabit": stable, "egim_gun": slope_per_day}),
                    },
                );
                c.check(&out);
            }
            "ramp_hours" => {
                let out = result_json(
                    trigger::ramp_hours(c.f("t_from"), c.f("t_to"), c.f("rate")),
                    |h| json!(h),
                );
                c.check(&out);
            }
            "lager_weeks" => {
                let out = result_json(
                    trigger::lager_weeks(c.f("og"), c.i("pts_per_week"), c.i("lo"), c.i("hi")),
                    |w| json!(w),
                );
                c.check(&out);
            }
            "sugar_from_mol" => {
                let out = result_json(
                    trigger::sugar_from_mol(c.f("mol"), sugar(c), opt(c, "dry")),
                    |g| json!(g),
                );
                c.check(&out);
            }
            "bottle_mol" => c.check_f(trigger::bottle_mol(
                c.f("vol_t"),
                c.f("t_res"),
                c.f("v_beer"),
            )),
            "p_abs_pa" => c.check_f(trigger::p_abs_pa(c.f("vol"), c.f("t_c"))),
            "head_gas_mol" => {
                c.check_f(trigger::head_gas_mol(c.f("vol"), c.f("t_c"), c.f("v_head")));
            }
            "head_mol" => {
                let (n, p) =
                    trigger::head_mol(c.f("vol_t"), c.f("t_cond"), c.f("v_head"), c.b("purged"));
                c.check(&json!([n, p]));
            }
            "equilibrium_vol" => c.check_f(trigger::equilibrium_vol(
                c.f("n_total"),
                c.f("t_c"),
                c.f("v_beer"),
                c.f("v_head"),
            )),
            "keg_priming" => {
                let mode = match c.girdi["mode"].as_str().expect("mode") {
                    "tam" => KegMode::Full,
                    "kismi" => KegMode::Partial,
                    other => panic!("unknown mode {other}"),
                };
                let p = KegParams {
                    vol_t: c.f("vol_t"),
                    t_res: c.f("T_res"),
                    t_cond: c.f("T_cond"),
                    v_beer: c.f("V_beer"),
                    v_head: c.f("V_head"),
                    sugar: sugar(c),
                    purged: c.b("purged"),
                    mode,
                    part: c.f("part"),
                    t_serv: c.f("T_serv"),
                    dry: opt(c, "dry"),
                    limit_bar: opt(c, "limit_bar"),
                };
                c.check(&result_json(trigger::keg_priming(&p), |k| keg_json(&k)));
            }
            "co2_from_remaining" => {
                c.check_f(trigger::co2_from_remaining(c.f("sg_rack"), c.f("fg_ref")));
            }
            "cask_priming" => {
                let out = result_json(
                    trigger::cask_priming(
                        c.f("v"),
                        c.f("vol_t"),
                        c.f("t_res"),
                        c.f("sg_rack"),
                        c.f("fg_ref"),
                        sugar(c),
                        opt(c, "dry"),
                    ),
                    |k| cask_json(&k),
                );
                c.check(&out);
            }
            "finings_ml" => c.check_f(trigger::finings_ml(c.f("v"), c.f("rate"))),
            "cask_days" => c.check_f(trigger::cask_days(c.f("t_cellar"), c.f("base"), c.f("t0"))),
            _ => return false,
        }
        true
    });
}
