//! Section 07 parity: home MVP extras against `vektorler/07.json`.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::panic
)]

mod ortak;

use engine::home::{
    self, AbsorptionHopForm, BalanceClass, Forecast, IngredientKind, Ions, ScaleMode,
};
use serde_json::{Map, Value, json};

fn num(v: &Value) -> f64 {
    v.as_f64().expect("number")
}

fn pairs(v: &Value) -> Vec<(f64, f64)> {
    v.as_array()
        .expect("array of pairs")
        .iter()
        .map(|p| (num(&p[0]), num(&p[1])))
        .collect()
}

fn floats(v: &Value) -> Vec<f64> {
    v.as_array().expect("array").iter().map(num).collect()
}

fn pair_json(p: &[(f64, f64)]) -> Value {
    Value::Array(p.iter().map(|&(a, b)| json!([a, b])).collect())
}

fn ion(w: &Value, key: &str) -> f64 {
    w.get(key).map_or(0.0, num)
}

fn kind(s: &str) -> IngredientKind {
    match s {
        "mash" => IngredientKind::Mash,
        "seker" => IngredientKind::Sugar,
        "hop_ibu" => IngredientKind::HopBittering,
        "hop_aroma" => IngredientKind::HopAroma,
        "tuz" => IngredientKind::Salt,
        "misc" => IngredientKind::Misc,
        "maya_paket" => IngredientKind::YeastPack,
        other => panic!("unknown ingredient type {other}"),
    }
}

fn hop_losses(v: &Value) -> Vec<(AbsorptionHopForm, f64)> {
    v.as_object()
        .expect("object")
        .iter()
        .map(|(k, g)| {
            let form = match k.as_str() {
                "pellet" => AbsorptionHopForm::Pellet,
                "yaprak" => AbsorptionHopForm::Leaf,
                "cryo" => AbsorptionHopForm::Cryo,
                other => panic!("unknown hop form {other}"),
            };
            (form, num(g))
        })
        .collect()
}

#[test]
fn parite_07() {
    ortak::run("07", |c| {
        match c.fonk.as_str() {
            "hourly_median" => {
                let out = home::hourly_median(&pairs(c.v("points")));
                c.check(&pair_json(&out));
            }
            "fit_decay" => {
                let steps = usize::try_from(c.i("steps")).unwrap();
                let home::DecayFitResult { fit, table } =
                    home::fit_decay(&pairs(c.v("pts")), c.f("fg_lo"), c.f("fg_hi"), steps)
                        .expect("fit");
                c.check(&json!({
                    "f": fit.fg_inf, "a": fit.a, "k": fit.k_per_h, "rss": fit.rss,
                    "n_table": table.len(),
                    "table_ilk": [table[0].0, table[0].1],
                    "table_son": [table[table.len() - 1].0, table[table.len() - 1].1],
                }));
            }
            "forecast" => {
                let g = &c.girdi;
                let eps = g.get("eps").map_or(0.0005, num);
                let min_n = g
                    .get("min_n")
                    .map_or(12, |v| usize::try_from(v.as_i64().unwrap()).unwrap());
                let min_drop = g.get("min_drop").map_or(0.30, num);
                let out = home::forecast(&home::ForecastInput {
                    points: &pairs(c.v("points")),
                    og: c.f("og"),
                    fg_recipe: c.f("fg_recipe"),
                    eps_sg: eps,
                    min_points: min_n,
                    min_drop,
                });
                let v = match out.expect("finite input") {
                    Forecast::InsufficientData => json!({"durum": "veri yetersiz"}),
                    Forecast::Ready(r) => json!({
                        "FG": r.fg, "aralik": [r.range_sg.0, r.range_sg.1], "k_saat": r.k_per_h,
                        "bitis_saat": r.finish_h, "kalan_saat": r.remaining_h,
                        "bitti": r.finished,
                        "uyari": r.warning,
                    }),
                };
                c.check(&v);
            }
            "drop_enough" => c.check_res(
                home::drop_enough(c.f("og"), c.f("sg_now"), c.f("fg_recipe"), c.f("min_drop"))
                    .map(|b| json!(b)),
            ),
            "fg_warning" => {
                c.check_res(home::fg_warning(c.f("fg_inf"), c.f("fg_recipe")).map(|b| json!(b)));
            }
            "ispindel_eval" => {
                let home::IspindelReading { sg, warnings: w } = home::ispindel_eval(
                    &floats(c.v("co")),
                    c.f("a_min"),
                    c.f("a_max"),
                    c.f("angle"),
                );
                let codes: Vec<&str> = w.iter().map(|x| x.as_str()).collect();
                c.check(&json!({"sg": sg, "uyari": codes}));
            }
            "polyfit" => {
                let deg = usize::try_from(c.i("deg")).unwrap();
                let co = home::polyfit(&floats(c.v("xs")), &floats(c.v("ys")), deg);
                c.check(&json!(co));
            }
            "peval" => c.check_f(home::peval(&floats(c.v("co")), c.f("x"))),
            "ispindel_cal" => {
                let deg = usize::try_from(c.i("deg")).unwrap();
                let v = match home::ispindel_cal(&pairs(c.v("points")), deg) {
                    Ok(r) => {
                        let codes: Vec<&str> = r.warnings.iter().map(|x| x.as_str()).collect();
                        json!({
                            "co": r.coefficients, "r2": r.r2, "max_res": r.max_residual_sg,
                            "uyari": codes,
                        })
                    }
                    Err(_) => json!({"hata": "ValueError"}),
                };
                c.check(&v);
            }
            "tilt_cal" => {
                c.check_res(
                    home::tilt_cal(&pairs(c.v("points"))).map(|t| json!([t.slope, t.offset_sg])),
                );
            }
            "tilt_apply" => c.check_res(
                home::tilt_apply(
                    &home::TiltCorrection {
                        slope: c.f("slope"),
                        offset_sg: c.f("offset"),
                    },
                    c.f("tilt"),
                )
                .map(|x| json!(x)),
            ),
            "heat_time_min" => c.check_r(home::heat_time_min(&home::HeatTimeInput {
                water_l: c.f("water_l"),
                grain_kg: c.f("grain_kg"),
                kettle_kg: c.f("kettle_kg"),
                t0_c: c.f("t0"),
                t1_c: c.f("t1"),
                power_kw: c.f("power_kw"),
                eff: c.f("eff"),
            })),
            "boiloff_l_per_h" => {
                c.check_f(home::boiloff_l_per_h(c.f("power_kw"), c.f("eff_boil")));
            }
            "tau_from_batch" => c.check_r(home::tau_from_batch(
                c.f("t0"),
                c.f("tc"),
                c.f("t"),
                c.f("minutes"),
            )),
            "cool_time_immersion" => {
                c.check_r(home::cool_time_immersion(&home::CoolImmersionInput {
                    t0_c: c.f("t0"),
                    tap_c: c.f("tc"),
                    target_c: c.f("t"),
                    tau_min: c.f("tau"),
                    vol_l: c.f("vol_l"),
                    vol_ref_l: c.f("vol_ref_l"),
                }));
            }
            "cool_time_flow" => c.check_r(home::cool_time_flow(c.f("vol_l"), c.f("flow_l_min"))),
            "scale_recipe" => {
                let obj = c.v("ing").as_object().expect("ing object");
                let names: Vec<&String> = obj.keys().collect();
                let items: Vec<(f64, IngredientKind)> = obj
                    .values()
                    .map(|x| (num(&x[0]), kind(x[1].as_str().unwrap())))
                    .collect();
                let mode = if c.v("mode").as_str() == Some("oran") {
                    ScaleMode::Ratio
                } else {
                    ScaleMode::Target
                };
                // The list form is the single-ingredient factor applied to each amount.
                let factor = |kind: IngredientKind| {
                    home::scale_factor(&home::ScaleFactorInput {
                        kind,
                        v0_l: c.f("v0"),
                        v1_l: c.f("v1"),
                        eff0: c.f("eff0"),
                        eff1: c.f("eff1"),
                        mode,
                        per_batch: false,
                        water_ratio: None,
                    })
                };
                let out: engine::Result<Vec<f64>> = if items.is_empty() {
                    factor(IngredientKind::Misc).map(|_| Vec::new())
                } else {
                    items
                        .iter()
                        .map(|&(q, k)| factor(k).map(|f| q * f))
                        .collect()
                };
                c.check_res(out.map(|out| {
                    let map: Map<String, Value> = names
                        .into_iter()
                        .zip(out)
                        .map(|(n, q)| (n.clone(), json!(q)))
                        .collect();
                    Value::Object(map)
                }));
            }
            "scale_factor" => {
                let mode = if c.v("mode").as_str() == Some("oran") {
                    ScaleMode::Ratio
                } else {
                    ScaleMode::Target
                };
                c.check_r(home::scale_factor(&home::ScaleFactorInput {
                    kind: kind(c.v("tip").as_str().unwrap()),
                    v0_l: c.f("v0"),
                    v1_l: c.f("v1"),
                    eff0: c.f("eff0"),
                    eff1: c.f("eff1"),
                    mode,
                    per_batch: c.b("per_batch"),
                    water_ratio: c.v("water_ratio").as_f64(),
                }));
            }
            "yeast_packs" => c.check_res(
                home::yeast_packs(c.f("cells_needed_billion"), c.f("cells_per_pack_billion"))
                    .map(|n| json!(n)),
            ),
            "scaling_record" => {
                let scope = match c.v("scope").as_str().unwrap() {
                    "tarif" => Some(home::ScaleScope::Recipe),
                    "parti" => Some(home::ScaleScope::Batch),
                    _ => None,
                };
                let mode = match c.v("mode").as_str().unwrap() {
                    "hedef" => Some(ScaleMode::Target),
                    "oran" => Some(ScaleMode::Ratio),
                    _ => None,
                };
                let (Some(scope), Some(mode)) = (scope, mode) else {
                    c.check(&json!({"hata": "ValueError"}));
                    return true;
                };
                let Ok(r) = home::scaling_record(&home::ScalingRecordInput {
                    scope,
                    mode,
                    volumes_l: (c.f("v0"), c.f("v1")),
                    efficiencies: (c.f("eff0"), c.f("eff1")),
                    recipe_version: u32::try_from(c.i("recipe_version")).unwrap(),
                }) else {
                    c.check_hata();
                    return true;
                };
                c.check(&json!({
                    "kapsam": c.v("scope"), "mod": c.v("mode"), "v0": r.v0_l, "v1": r.v1_l,
                    "eff0": r.eff0, "eff1": r.eff1, "oran": r.ratio,
                    "tarif_surumu": r.recipe_version, "tarif_degisti": r.recipe_changed,
                }));
            }
            "round_to" => c.check_r(home::round_to(c.f("q"), c.f("step"))),
            "ion_balance" => {
                let w = c.v("p");
                let r = home::ion_balance(&Ions {
                    ca_ppm: ion(w, "Ca"),
                    mg_ppm: ion(w, "Mg"),
                    na_ppm: ion(w, "Na"),
                    k_ppm: ion(w, "K"),
                    so4_ppm: ion(w, "SO4"),
                    cl_ppm: ion(w, "Cl"),
                    hco3_ppm: ion(w, "HCO3"),
                    co3_ppm: ion(w, "CO3"),
                    no3_ppm: ion(w, "NO3"),
                });
                let class: BalanceClass = r.class;
                c.check(&json!({
                    "cation": r.cations_meq_l, "anion": r.anions_meq_l,
                    "balance_pct": r.balance_pct, "class": class.as_str(),
                }));
            }
            "hardness_caco3" => c.check_f(home::hardness_caco3(c.f("ca"), c.f("mg"))),
            "alk_caco3_from_hco3" => c.check_f(home::alk_caco3_from_hco3(c.f("hco3"))),
            "caco3_to" => {
                let u = home::caco3_to(c.f("x"));
                c.check(&json!({"dH": u.dh, "fH": u.fh, "mEq/L": u.meq_per_l}));
            }
            "kettle_hop_loss" => c.check_f(home::kettle_hop_loss(&hop_losses(c.v("g_by_form")))),
            "dryhop_loss" => c.check_f(home::dryhop_loss(&hop_losses(c.v("g_by_form")))),
            "fruit" => {
                c.check_res(
                    home::fruit(c.f("kg"), c.f("density"), c.f("sediment"))
                        .map(|f| json!([f.added_l, f.sediment_l])),
                );
            }
            _ => return false,
        }
        true
    });
}
