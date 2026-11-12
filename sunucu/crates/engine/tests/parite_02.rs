//! Section 02 parity: hop utilisation and IBU against `vektorler/02.json`.
#![allow(clippy::expect_used, clippy::too_many_lines)]
mod ortak;

use engine::density::LateExtractPoint;
use engine::hops::{self, HopEquation, ScaleStatus, StorageType};
use engine::recipe::{
    Equipment, Hop, HopForm, HopOptions, HopUse, Mash, Misc, Options, RecipeInput, RecipeType,
};
use engine::units;
use ortak::{hop, hop_options as opts, hops, recipe_input as recipe};
use serde_json::Value;

fn eq(c: &ortak::Case) -> engine::Result<HopEquation> {
    HopEquation::from_code(c.i("equation"))
}

fn profile(v: &Value) -> Vec<LateExtractPoint> {
    v.as_array()
        .expect("profile is an array")
        .iter()
        .map(|p| LateExtractPoint {
            sg: p[0].as_f64().expect("sg"),
            time_min: p[1].as_f64().expect("t"),
        })
        .collect()
}

fn misc(v: &Value) -> Vec<Misc> {
    v.as_array()
        .expect("misc is an array")
        .iter()
        .map(ortak::misc)
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
            "garetz_ibu" => c.check_r(hops::garetz_ibu(&hops::GaretzIbuInput {
                w_oz: c.f("w"),
                a_pct: c.f("a"),
                t_min: c.f("t"),
                sg: c.f("sg"),
                v_gal: c.f("v"),
                bv_gal: c.f("bv"),
            })),
            "garetz_utilization" => c.check_r(hops::garetz_utilization(c.f("sg"), c.f("t"))),
            "post_boil_utilization" => c.check_f(hops::post_boil_utilization(
                c.f("sg"),
                c.f("t0"),
                c.f("dur"),
            )),
            "boil_temp_c" => c.check_f(units::boil_temp_c(c.f("alt_ft"))),
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
            "aged_alpha" => c.check_r(hops::aged_alpha(&hops::AgedAlphaInput {
                alpha_pct: c.f("alpha"),
                hsi_pct: c.f("hsi"),
                temp_f: c.f("temp_f"),
                days: c.f("days"),
                storage: c.f("storage"),
            })),
            "tinseth_ibu" => c.check_r(hops::tinseth_ibu(&hops::HopIbuInput {
                w_oz: c.f("w"),
                a_pct: c.f("a"),
                t_min: c.f("t"),
                sg: c.f("sg"),
                v_gal: c.f("v"),
            })),
            "rager_ibu" => c.check_r(hops::rager_ibu(&hops::HopIbuInput {
                w_oz: c.f("w"),
                a_pct: c.f("a"),
                t_min: c.f("t"),
                sg: c.f("sg"),
                v_gal: c.f("v"),
            })),
            "tinseth_weight" => c.check_r(hops::tinseth_weight(&hops::HopWeightInput {
                ibu: c.f("ibu"),
                v_gal: c.f("v"),
                sg: c.f("sg"),
                t_min: c.f("t"),
                a_pct: c.f("a"),
            })),
            "rager_weight" => c.check_r(hops::rager_weight(&hops::HopWeightInput {
                ibu: c.f("ibu"),
                v_gal: c.f("v"),
                sg: c.f("sg"),
                t_min: c.f("t"),
                a_pct: c.f("a"),
            })),
            "garetz_weight" => c.check_r(hops::garetz_weight(&hops::HopWeightInput {
                ibu: c.f("ibu"),
                a_pct: c.f("a"),
                t_min: c.f("t"),
                sg: c.f("sg"),
                v_gal: c.f("v"),
            })),
            "whirlpool_ibu" => c.check_r(hops::whirlpool_ibu(&hops::WhirlpoolIbuInput {
                w_oz: c.f("w"),
                a_pct: c.f("a"),
                t_min: c.f("t"),
                sg: c.f("sg"),
                v_gal: c.f("v"),
                temp_c: c.f("temp_c"),
            })),
            "whirlpool_utilization" => c.check_f(hops::whirlpool_utilization(
                c.f("sg"),
                c.f("t"),
                c.f("temp_c"),
            )),
            "boil_temp_f" => c.check_f(units::boil_temp_f(c.f("alt_ft"))),
            "resolve_equation" => {
                let global = HopEquation::from_code(c.i("global_equation")).expect("equation");
                let r = hops::resolve_equation(c.i("equation"), global);
                c.check(&serde_json::json!(r as i64));
            }
            "hop_util" => c.check_r((|| {
                hops::hop_utilization(&hops::HopUtilizationInput {
                    sg: c.f("sg"),
                    t_min: c.f("t"),
                    equation: eq(c)?,
                    whirlpool_mu: c.b("whirlpool_mu"),
                    temp_f: c.f("temp_f"),
                })
            })()),
            "hop_adjustments" => c.check_r((|| {
                Ok(hops::hop_adjustments(&hops::HopAdjustmentsInput {
                    form: HopForm::from_code(c.i("form"))?,
                    usage: HopUse::from_code(c.i("use"))?,
                    util_pct: c.f("util_pct"),
                    extract_recipe: c.b("extract_recipe"),
                    opts: &opts(c.v("options")),
                }))
            })()),
            "hop_total_utilization" => c.check_r(hops::hop_total_utilization(
                &hop(c.v("hop")),
                &recipe(c.v("recipe")),
                &profile(c.v("profile")),
                eq(c).expect("equation"),
            )),
            "new_hop_ibu" => c.check_r(hops::new_hop_ibu(
                &hop(c.v("hop")),
                &recipe(c.v("recipe")),
                &profile(c.v("profile")),
                eq(c).expect("equation"),
            )),
            "hopped_extract_ibu" => c.check_f(hops::hopped_extract_ibu(
                c.f("amount_oz"),
                c.f("ibu_gal_per_lb"),
                c.f("volume"),
                c.f("trub"),
            )),
            "hop_percent" => c.check_f(hops::hop_percent(c.f("w"), c.f("tot_hops"))),
            "recipe_ibu" => c.check_r(hops::recipe_ibu(
                &recipe(c.v("recipe")),
                eq(c).expect("equation"),
                c.f("v_boil"),
            )),
            "scale_bitterness" => {
                let Ok(r) = hops::scale_bitterness(&hops::ScaleBitternessInput {
                    current_ibu: c.f("current_ibu"),
                    target_ibu: c.f("target"),
                    recipe: &recipe(c.v("recipe")),
                    equation: eq(c).expect("equation"),
                    v_boil_floz: c.f("v_boil"),
                }) else {
                    c.check_hata();
                    return true;
                };
                let status = match r.status {
                    ScaleStatus::Ok => "ok",
                    ScaleStatus::Unchanged => "unchanged",
                    ScaleStatus::Failed => "failed",
                };
                c.check(&serde_json::json!({"status": status, "amounts": r.amounts_oz}));
            }
            "hop_ibu_adjust" => c.check_r(hops::hop_ibu_adjust(
                c.f("w_old"),
                c.f("ibu_old"),
                c.f("ibu_new"),
            )),
            "tool_hop_ibu" => c.check_r(hops::tool_hop_ibu(&hops::ToolHopIbuInput {
                hop: &hop(c.v("hop")),
                sg: c.f("sg"),
                v_floz: c.f("v_floz"),
                equation: eq(c).expect("equation"),
                util_pct: c.f("util_pct"),
                opts: &opts(c.v("options")),
            })),
            "tool_bitterness" => {
                let Ok(r) = hops::tool_bitterness(&hops::ToolBitternessInput {
                    hops: &hops(c.v("hops")),
                    sg: c.f("sg"),
                    v_floz: c.f("v_floz"),
                    equation: eq(c).expect("equation"),
                    util_pct: c.f("util_pct"),
                    opts: &opts(c.v("options")),
                }) else {
                    c.check_hata();
                    return true;
                };
                c.check(
                    &serde_json::json!({"ibu": r.ibu, "aau": r.aau, "contribs": r.contribs_ibu}),
                );
            }
            "aau" => c.check_f(hops::hop_aau(
                c.f("w"),
                c.f("a"),
                HopUse::from_code(c.i("use")).expect("hop use"),
                c.f("t"),
            )),
            "total_hops_phase" => {
                c.check_f(hops::total_hops_phase(
                    &hops(c.v("hops")),
                    HopUse::from_code(c.i("phase")).expect("hop use"),
                ));
            }
            "hop_rate" => c.check_f(hops::hop_rate(
                &hops(c.v("hops")),
                c.f("volume_floz"),
                HopUse::from_code(c.i("phase")).expect("hop use"),
            )),
            "total_whirlpool_time" => c.check(&serde_json::json!(hops::total_whirlpool_time(
                c.i("recipe_time"),
                &hops(c.v("hops")),
                &misc(c.v("misc")),
            ))),
            "aged_alpha_months" => c.check_r((|| {
                hops::aged_alpha_months(&hops::AgedAlphaMonthsInput {
                    alpha_pct: c.f("alpha"),
                    hsi_pct: c.f("hsi"),
                    temp_f: c.f("temp_f"),
                    months: c.f("months"),
                    storage_type: StorageType::from_code(c.i("storage_type"))?,
                })
            })()),
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
    let h = |amount_oz: f64, alpha_pct: f64, form: HopForm, usage: HopUse, time_min: f64| Hop {
        amount_oz,
        alpha_pct,
        form,
        usage,
        time_min,
        whirlpool_temp_f: 176.0,
    };
    let rc = RecipeInput {
        kind: RecipeType::AllGrain,
        volume_floz: 640.0,
        boil_vol_floz: 768.0,
        boil_time_min: 60.0,
        efficiency_pct: 72.0,
        equipment: Equipment {
            trub_loss_floz: 0.0,
            pre_trub_loss_floz: 0.0,
            altitude_ft: 0.0,
            hop_util_pct: 100.0,
            whirlpool_carry: false,
            whirlpool_time_min: 0.0,
        },
        fermentables: vec![],
        hops: vec![
            h(1.0, 10.0, HopForm::Pellet, HopUse::Boil, 60.0),
            h(0.05, 50.0, HopForm::IsomerizedExtract, HopUse::DryHop, 0.0),
        ],
        yeasts: vec![],
        miscs: vec![],
        mash: Mash {
            grain_weight_oz: 0.0,
            biab: false,
            steps: vec![],
        },
        options: Options {
            hop: HopOptions::default(),
            ..Options::default()
        },
    };
    let t = HopEquation::Tinseth;
    let cur = hops::recipe_ibu(&rc, t, 0.0).expect("ibu");
    let r = hops::scale_bitterness(&hops::ScaleBitternessInput {
        current_ibu: cur,
        target_ibu: 60.0,
        recipe: &rc,
        equation: t,
        v_boil_floz: 0.0,
    })
    .expect("scale");
    assert!(matches!(r.status, ScaleStatus::Ok));
    let mut scaled = rc.clone();
    for (x, &amount_oz) in scaled.hops.iter_mut().zip(&r.amounts_oz) {
        x.amount_oz = amount_oz;
    }
    let got = hops::recipe_ibu(&scaled, t, 0.0).expect("ibu");
    assert!((got - 60.0).abs() < 1e-9, "got {got}");
}

/// T1.16: non-finite input, bad volumes and unknown codes are errors, not NaN or defaults.
#[test]
fn invalid_inputs_are_errors() {
    use engine::Error;
    assert!(
        hops::garetz_ibu(&hops::GaretzIbuInput {
            w_oz: f64::NAN,
            a_pct: 5.0,
            t_min: 60.0,
            sg: 1.05,
            v_gal: 5.0,
            bv_gal: 5.0
        })
        .is_err()
    );
    assert!(
        hops::garetz_ibu(&hops::GaretzIbuInput {
            w_oz: 1.0,
            a_pct: 5.0,
            t_min: 60.0,
            sg: 1.05,
            v_gal: 0.0,
            bv_gal: 5.0
        })
        .is_err()
    );
    assert!(
        hops::tinseth_ibu(&hops::HopIbuInput {
            w_oz: 1.0,
            a_pct: f64::INFINITY,
            t_min: 60.0,
            sg: 1.05,
            v_gal: 5.0
        })
        .is_err()
    );
    assert!(
        hops::aged_alpha(&hops::AgedAlphaInput {
            alpha_pct: 5.0,
            hsi_pct: 30.0,
            temp_f: f64::NAN,
            days: 180.0,
            storage: 1.0
        })
        .is_err()
    );
    assert!(HopEquation::from_code(3).is_err());
    assert!(HopForm::from_code(5).is_err());
    assert!(HopUse::from_code(-1).is_err());
    assert!(StorageType::from_code(9).is_err());
    assert_eq!(
        hops::resolve_equation(7, HopEquation::Rager),
        HopEquation::Rager
    );
    // 1e9 oz never converges within the cap.
    assert!(matches!(
        hops::garetz_ibu(&hops::GaretzIbuInput {
            w_oz: 1e9,
            a_pct: 20.0,
            t_min: 60.0,
            sg: 1.05,
            v_gal: 5.0,
            bv_gal: 5.0
        }),
        Err(Error::NoConvergence { .. })
    ));
}
