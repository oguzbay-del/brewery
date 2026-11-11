//! Section 07 parity: home MVP extras against `vektorler/07.json`.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::panic
)]

mod ortak;

use engine::home::{self, BalanceClass, Forecast, HopForm, IngredientKind, Ions, ScaleMode};
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

fn hop_losses(v: &Value) -> Vec<(HopForm, f64)> {
    v.as_object()
        .expect("object")
        .iter()
        .map(|(k, g)| {
            let form = match k.as_str() {
                "pellet" => HopForm::Pellet,
                "yaprak" => HopForm::Leaf,
                "cryo" => HopForm::Cryo,
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
                let (fit, table) =
                    home::fit_decay(&pairs(c.v("pts")), c.f("fg_lo"), c.f("fg_hi"), steps)
                        .expect("fit");
                c.check(&json!({
                    "f": fit.fg_inf, "a": fit.a, "k": fit.k, "rss": fit.rss,
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
                let out = home::forecast(
                    &pairs(c.v("points")),
                    c.f("og"),
                    c.f("fg_recipe"),
                    eps,
                    min_n,
                    min_drop,
                );
                let v = match out {
                    Forecast::InsufficientData => json!({"durum": "veri yetersiz"}),
                    Forecast::Ready(r) => json!({
                        "FG": r.fg, "aralik": [r.range.0, r.range.1], "k_saat": r.k_per_hour,
                        "bitis_saat": r.finish_hour, "kalan_saat": r.remaining_hours,
                        "uyari": r.warning,
                    }),
                };
                c.check(&v);
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
                    Ok(r) => json!({"co": r.coefficients, "r2": r.r2, "max_res": r.max_residual}),
                    Err(home::CalibrationError::TooFewPoints) => json!({"hata": "ValueError"}),
                };
                c.check(&v);
            }
            "tilt_cal" => {
                let (slope, offset) = home::tilt_cal(&pairs(c.v("points")));
                c.check(&json!([slope, offset]));
            }
            "heat_time_min" => c.check_f(home::heat_time_min(
                c.f("water_l"),
                c.f("grain_kg"),
                c.f("kettle_kg"),
                c.f("t0"),
                c.f("t1"),
                c.f("power_kw"),
                c.f("eff"),
            )),
            "boiloff_l_per_h" => {
                c.check_f(home::boiloff_l_per_h(c.f("power_kw"), c.f("eff_boil")));
            }
            "tau_from_batch" => c.check_f(home::tau_from_batch(
                c.f("t0"),
                c.f("tc"),
                c.f("t"),
                c.f("minutes"),
            )),
            "cool_time_immersion" => c.check_f(home::cool_time_immersion(
                c.f("t0"),
                c.f("tc"),
                c.f("t"),
                c.f("tau"),
                c.f("vol_l"),
                c.f("vol_ref_l"),
            )),
            "cool_time_flow" => c.check_f(home::cool_time_flow(c.f("vol_l"), c.f("flow_l_min"))),
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
                let out = home::scale_recipe(
                    &items,
                    c.f("v0"),
                    c.f("v1"),
                    c.f("eff0"),
                    c.f("eff1"),
                    mode,
                );
                let map: Map<String, Value> = names
                    .into_iter()
                    .zip(out)
                    .map(|(n, q)| (n.clone(), json!(q)))
                    .collect();
                c.check(&Value::Object(map));
            }
            "round_to" => c.check_f(home::round_to(c.f("q"), c.f("step"))),
            "ion_balance" => {
                let w = c.v("p");
                let r = home::ion_balance(&Ions {
                    ca: ion(w, "Ca"),
                    mg: ion(w, "Mg"),
                    na: ion(w, "Na"),
                    k: ion(w, "K"),
                    so4: ion(w, "SO4"),
                    cl: ion(w, "Cl"),
                    hco3: ion(w, "HCO3"),
                    co3: ion(w, "CO3"),
                    no3: ion(w, "NO3"),
                });
                let class: BalanceClass = r.class;
                c.check(&json!({
                    "cation": r.cations, "anion": r.anions,
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
                let (v, s) = home::fruit(c.f("kg"), c.f("density"), c.f("sediment"));
                c.check(&json!([v, s]));
            }
            _ => return false,
        }
        true
    });
}
