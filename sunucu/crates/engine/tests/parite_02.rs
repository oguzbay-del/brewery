//! Section 02 parity: hop utilisation and IBU against `vektorler/02.json`.
#![allow(clippy::expect_used, clippy::too_many_lines)]
mod ortak;

use engine::hops::{self, Fermentable, Hop, HopOptions, MiscItem, Recipe, ScaleStatus};
use serde_json::Value;

fn num(v: &Value, k: &str) -> f64 {
    v.get(k)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("missing number `{k}` in {v}"))
}

fn int(v: &Value, k: &str) -> i64 {
    v.get(k)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("missing integer `{k}` in {v}"))
}

fn hop(v: &Value) -> Hop {
    Hop {
        amount: num(v, "amount"),
        alpha: num(v, "alpha"),
        form: int(v, "form"),
        usage: int(v, "use"),
        time: num(v, "time"),
        whirlpool_temp_f: num(v, "whirlpool_temp_f"),
    }
}

fn hops(v: &Value) -> Vec<Hop> {
    v.as_array()
        .expect("hops is an array")
        .iter()
        .map(hop)
        .collect()
}

fn opts(v: &Value) -> HopOptions {
    HopOptions {
        first_wort_adj: num(v, "first_wort_adj"),
        type_adj: int(v, "type_adj"),
        leaf_adj: num(v, "leaf_adj"),
        plug_adj: num(v, "plug_adj"),
        pellet_adj: num(v, "pellet_adj"),
        mash_adj: num(v, "mash_adj"),
    }
}

fn recipe(v: &Value) -> Recipe {
    Recipe {
        volume: num(v, "volume"),
        trub_loss: num(v, "trub_loss"),
        boil_time: num(v, "boil_time"),
        boil_vol: num(v, "boil_vol"),
        altitude: num(v, "altitude"),
        recipe_type: int(v, "type"),
        hop_util: num(v, "hop_util"),
        whirlpool_carry: v["whirlpool_carry"].as_bool().expect("bool"),
        whirlpool_time: num(v, "whirlpool_time"),
    }
}

fn ferms(v: &Value) -> Vec<Fermentable> {
    v.as_array()
        .expect("fermentables is an array")
        .iter()
        .map(|g| Fermentable {
            name: g["name"].as_str().expect("name").to_owned(),
            usage: int(g, "use"),
            kind: int(g, "type"),
            amount_oz: num(g, "amount_oz"),
            yield_pct: num(g, "yield_pct"),
            late_extract: num(g, "late_extract"),
            points: num(g, "points"),
            ibu_gal_per_lb: num(g, "ibu_gal_per_lb"),
        })
        .collect()
}

fn profile(v: &Value) -> Vec<(f64, f64)> {
    v.as_array()
        .expect("profile is an array")
        .iter()
        .map(|p| (p[0].as_f64().expect("sg"), p[1].as_f64().expect("t")))
        .collect()
}

fn misc(v: &Value) -> Vec<MiscItem> {
    v.as_array()
        .expect("misc is an array")
        .iter()
        .map(|x| MiscItem {
            usage: int(x, "use"),
            time: num(x, "time"),
        })
        .collect()
}

#[test]
fn parite_02() {
    ortak::run("02", |c| {
        match c.fonk.as_str() {
            "tinseth_utilization" => c.check_f(hops::tinseth_utilization(c.f("sg"), c.f("t"))),
            "temperature_factor" => c.check_f(hops::temperature_factor(c.f("temp_c"))),
            "rager_table" => c.check_f(hops::rager_table(c.i("n"))),
            "rager_gravity_adjustment" => c.check_f(hops::rager_gravity_adjustment(c.f("sg"))),
            "rager_utilization" => c.check_f(hops::rager_utilization(c.f("sg"), c.f("t"))),
            "garetz_ibu" => c.check_f(hops::garetz_ibu(
                c.f("w"),
                c.f("a"),
                c.f("t"),
                c.f("sg"),
                c.f("v"),
                c.f("bv"),
            )),
            "garetz_utilization" => c.check_f(hops::garetz_utilization(c.f("sg"), c.f("t"))),
            "post_boil_utilization" => c.check_f(hops::post_boil_utilization(
                c.f("sg"),
                c.f("t0"),
                c.f("dur"),
            )),
            "boil_temp_c" => c.check_f(hops::boil_temp_c(c.f("alt_ft"))),
            "combine_utilizations" => {
                let segs: Vec<f64> = c
                    .v("segs")
                    .as_array()
                    .expect("segs is an array")
                    .iter()
                    .map(|x| x.as_f64().expect("seg is a number"))
                    .collect();
                c.check_f(hops::combine_utilizations(&segs));
            }
            "aged_alpha" => c.check_f(hops::aged_alpha(
                c.f("alpha"),
                c.f("hsi"),
                c.f("temp_f"),
                c.f("days"),
                c.f("storage"),
            )),
            "tinseth_ibu" => c.check_f(hops::tinseth_ibu(
                c.f("w"),
                c.f("a"),
                c.f("t"),
                c.f("sg"),
                c.f("v"),
            )),
            "rager_ibu" => c.check_f(hops::rager_ibu(
                c.f("w"),
                c.f("a"),
                c.f("t"),
                c.f("sg"),
                c.f("v"),
            )),
            "tinseth_weight" => c.check_f(hops::tinseth_weight(
                c.f("ibu"),
                c.f("v"),
                c.f("sg"),
                c.f("t"),
                c.f("a"),
            )),
            "rager_weight" => c.check_f(hops::rager_weight(
                c.f("ibu"),
                c.f("v"),
                c.f("sg"),
                c.f("t"),
                c.f("a"),
            )),
            "garetz_weight" => c.check_f(hops::garetz_weight(
                c.f("ibu"),
                c.f("a"),
                c.f("t"),
                c.f("sg"),
                c.f("v"),
            )),
            "whirlpool_ibu" => c.check_f(hops::whirlpool_ibu(
                c.f("w"),
                c.f("a"),
                c.f("t"),
                c.f("sg"),
                c.f("v"),
                c.f("temp_c"),
            )),
            "whirlpool_utilization" => c.check_f(hops::whirlpool_utilization(
                c.f("sg"),
                c.f("t"),
                c.f("temp_c"),
            )),
            "boil_temp_f" => c.check_f(hops::boil_temp_f(c.f("alt_ft"))),
            "resolve_equation" => {
                c.check(&serde_json::json!(hops::resolve_equation(
                    c.i("equation"),
                    c.i("global_equation")
                )));
            }
            "hop_util" => c.check_f(hops::hop_utilization(
                c.f("sg"),
                c.f("t"),
                c.i("equation"),
                c.b("whirlpool_mu"),
                c.f("temp_f"),
            )),
            "hop_adjustments" => c.check_f(hops::hop_adjustments(
                c.i("form"),
                c.i("use"),
                c.f("util_pct"),
                c.b("extract_recipe"),
                &opts(c.v("options")),
            )),
            "late_extract_sgs" => {
                let prof = hops::late_extract_sgs(
                    &recipe(c.v("recipe")),
                    &ferms(c.v("fermentables")),
                    c.f("v_boil"),
                );
                let json: Vec<Value> = prof
                    .iter()
                    .map(|&(sg, t)| serde_json::json!([sg, t]))
                    .collect();
                c.check(&Value::Array(json));
            }
            "hop_total_utilization" => c.check_f(hops::hop_total_utilization(
                &hop(c.v("hop")),
                &recipe(c.v("recipe")),
                &profile(c.v("profile")),
                c.i("equation"),
            )),
            "new_hop_ibu" => c.check_f(hops::new_hop_ibu(
                &hop(c.v("hop")),
                &recipe(c.v("recipe")),
                &profile(c.v("profile")),
                c.i("equation"),
                &opts(c.v("options")),
            )),
            "hopped_extract_ibu" => c.check_f(hops::hopped_extract_ibu(
                c.f("amount_oz"),
                c.f("ibu_gal_per_lb"),
                c.f("volume"),
                c.f("trub"),
            )),
            "hop_percent" => c.check_f(hops::hop_percent(c.f("w"), c.f("tot_hops"))),
            "recipe_ibu" => c.check_f(hops::recipe_ibu(
                &hops(c.v("hops")),
                &recipe(c.v("recipe")),
                &ferms(c.v("fermentables")),
                c.i("equation"),
                &opts(c.v("options")),
                c.f("v_boil"),
            )),
            "scale_bitterness" => {
                let r = hops::scale_bitterness(
                    c.f("current_ibu"),
                    c.f("target"),
                    &hops(c.v("hops")),
                    &recipe(c.v("recipe")),
                    &ferms(c.v("fermentables")),
                    c.i("equation"),
                    &opts(c.v("options")),
                    c.f("v_boil"),
                );
                let status = match r.status {
                    ScaleStatus::Ok => "ok",
                    ScaleStatus::Unchanged => "unchanged",
                    ScaleStatus::Failed => "failed",
                };
                c.check(&serde_json::json!({"status": status, "amounts": r.amounts}));
            }
            "hop_ibu_adjust" => c.check_f(hops::hop_ibu_adjust(
                c.f("w_old"),
                c.f("ibu_old"),
                c.f("ibu_new"),
            )),
            "tool_hop_ibu" => c.check_f(hops::tool_hop_ibu(
                &hop(c.v("hop")),
                c.f("sg"),
                c.f("v_floz"),
                c.i("equation"),
                c.f("util_pct"),
                &opts(c.v("options")),
            )),
            "tool_bitterness" => {
                let r = hops::tool_bitterness(
                    &hops(c.v("hops")),
                    c.f("sg"),
                    c.f("v_floz"),
                    c.i("equation"),
                    c.f("util_pct"),
                    &opts(c.v("options")),
                );
                c.check(&serde_json::json!({"ibu": r.ibu, "aau": r.aau, "contribs": r.contribs}));
            }
            "aau" => c.check_f(hops::hop_aau(c.f("w"), c.f("a"), c.i("use"), c.f("t"))),
            "total_hops_phase" => {
                c.check_f(hops::total_hops_phase(&hops(c.v("hops")), c.i("phase")));
            }
            "hop_rate" => c.check_f(hops::hop_rate(
                &hops(c.v("hops")),
                c.f("volume_floz"),
                c.i("phase"),
            )),
            "total_whirlpool_time" => c.check(&serde_json::json!(hops::total_whirlpool_time(
                c.i("recipe_time"),
                &hops(c.v("hops")),
                &misc(c.v("misc")),
            ))),
            "aged_alpha_months" => c.check_f(hops::aged_alpha_months(
                c.f("alpha"),
                c.f("hsi"),
                c.f("temp_f"),
                c.f("months"),
                c.i("storage_type"),
            )),
            _ => return false,
        }
        true
    });
}

/// T1.15 B1: the whirlpool carry is finite and quick for any duration.
#[test]
fn post_boil_is_finite_for_long_durations() {
    for dur in [498.0, 499.0, 600.0, 1e7, 1e300, f64::INFINITY] {
        let u = hops::post_boil_utilization(1.050, 60.0, dur);
        assert!(u.is_finite() && (0.0..1.0).contains(&u), "dur {dur}: {u}");
    }
    let capped = hops::post_boil_utilization(1.050, 60.0, 499.0);
    assert!((hops::post_boil_utilization(1.050, 60.0, 1e7) - capped).abs() < 1e-15);
}

/// T1.15 B2: scaling with an isomerised-extract dry hop lands on the target (60 IBU).
#[test]
fn scale_bitterness_with_form4_dry_hop_hits_target() {
    let h = |amount: f64, alpha: f64, form: i64, usage: i64, time: f64| Hop {
        amount,
        alpha,
        form,
        usage,
        time,
        whirlpool_temp_f: 176.0,
    };
    let hs = vec![h(1.0, 10.0, 0, 0, 60.0), h(0.05, 50.0, 4, 1, 0.0)];
    let rc = Recipe {
        volume: 640.0,
        trub_loss: 0.0,
        boil_time: 60.0,
        boil_vol: 768.0,
        altitude: 0.0,
        recipe_type: 2,
        hop_util: 100.0,
        whirlpool_carry: false,
        whirlpool_time: 0.0,
    };
    let o = HopOptions::default();
    let cur = hops::recipe_ibu(&hs, &rc, &[], 0, &o, 0.0);
    let r = hops::scale_bitterness(cur, 60.0, &hs, &rc, &[], 0, &o, 0.0);
    assert!(matches!(r.status, ScaleStatus::Ok));
    let scaled: Vec<Hop> = hs
        .iter()
        .zip(&r.amounts)
        .map(|(x, &amount)| Hop {
            amount,
            ..x.clone()
        })
        .collect();
    let got = hops::recipe_ibu(&scaled, &rc, &[], 0, &o, 0.0);
    assert!((got - 60.0).abs() < 1e-9, "got {got}");
}
