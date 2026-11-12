//! Section 10 parity: manual entry, extract tools, no-chill against `vektorler/10.json`.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::panic
)]

mod ortak;

use engine::hops;
use engine::manual::{
    self, ExtractKind, FewForecast, Insufficient, ManualError, NochillMode, ReadingSource,
};
use engine::units::{gravity_units, round_py};
use serde_json::{Value, json};

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

fn triples(v: &Value) -> Vec<(f64, f64, f64)> {
    v.as_array()
        .expect("array of triples")
        .iter()
        .map(|p| (num(&p[0]), num(&p[1]), num(&p[2])))
        .collect()
}

fn kind(v: &Value) -> ExtractKind {
    match v.as_str().expect("kind string") {
        "LME" => ExtractKind::Lme,
        "DME" => ExtractKind::Dme,
        "bal" => ExtractKind::Honey,
        other => panic!("unknown extract kind {other}"),
    }
}

fn opt(c: &ortak::Case, name: &str, default: f64) -> f64 {
    c.girdi.get(name).map_or(default, num)
}

fn error_name(e: ManualError) -> &'static str {
    match e {
        ManualError::InvalidVolume | ManualError::InvalidInput => "ValueError",
        ManualError::NoReadings => "IndexError",
        ManualError::Unbounded => "OverflowError",
    }
}

#[test]
fn parite_10() {
    ortak::run("10", |c| {
        match c.fonk.as_str() {
            "u" => c.check(&json!(gravity_units(c.f("sg")))),
            "r" => {
                let n = usize::try_from(c.i("n")).unwrap();
                c.check_f(round_py(c.f("x"), n));
            }
            "weight" => c.check_f(manual::weight(c.f("sg"), c.f("fg"))),
            "k_fit" => {
                let weighted = c.girdi.get("weighted").is_none_or(|w| w.as_bool().unwrap());
                let lag = opt(c, "lag", manual::LAG_H);
                let out =
                    manual::k_fit(&pairs(c.v("readings")), c.f("og"), c.f("fg"), lag, weighted);
                c.check(&match out {
                    None => json!({"yok": true}),
                    Some((k, each)) => json!({"yok": false, "k": k, "k_each": each}),
                });
            }
            "t_end" => c.check_f(manual::t_end(c.f("og"), c.f("fg"), c.f("k"), c.f("lag"))),
            "few_forecast" => {
                let out = manual::few_forecast(
                    &pairs(c.v("readings")),
                    c.f("og"),
                    c.f("fg_prior"),
                    opt(c, "lag", manual::LAG_H),
                    opt(c, "sigma", manual::SIGMA_FG),
                );
                let v = match out {
                    Err(e) => json!({"hata": error_name(e)}),
                    Ok(FewForecast::LooksFinished { fg_measured }) => {
                        json!({"durum": "bitmiş görünüyor", "FG_olculen": fg_measured})
                    }
                    Ok(FewForecast::PossiblyStuck { sg, fg_expected }) => {
                        json!({"durum": "takılmış olabilir", "SG": sg, "FG_beklenen": fg_expected})
                    }
                    Ok(FewForecast::Insufficient(why)) => {
                        let neden = match why {
                            Insufficient::NeedTwoReadings => {
                                "gecikme sonrası en az 2 okuma gerekir"
                            }
                            Insufficient::DropBelow30Percent => {
                                "beklenen düşüşün %30'u tamamlanmadı"
                            }
                            Insufficient::NonPositiveK => "k ≤ 0",
                        };
                        json!({"durum": "veri yetersiz", "neden": neden})
                    }
                    Ok(FewForecast::TargetReached) => {
                        json!({"durum": "hedefe ulaştı; durgunluk için 48 saat sonra tekrar ölç"})
                    }
                    Ok(FewForecast::Estimate {
                        k_per_hour,
                        finish_hour,
                        range_days,
                        remaining_days,
                        inconsistent,
                    }) => json!({
                        "k_saat": k_per_hour, "bitis_saat": finish_hour,
                        "aralik_gun": [range_days.0, range_days.1],
                        "kalan_gun": [remaining_days.0, remaining_days.1],
                        "tutarsiz": inconsistent, "etiket": "yaklaşık",
                    }),
                };
                c.check(&v);
            }
            "ppg" => c.check_f(manual::ppg(c.f("yield"))),
            "points_kg_l" => c.check_f(manual::points_kg_l(c.f("yield"))),
            "g_to_ml" => c.check_f(manual::g_to_ml(kind(c.v("kind")), c.f("g"))),
            "ml_to_g" => c.check_f(manual::ml_to_g(kind(c.v("kind")), c.f("ml"))),
            "swap" => c.check_f(manual::swap(
                kind(c.v("kind_from")),
                kind(c.v("kind_to")),
                c.f("g"),
            )),
            "extract_for_og" => c.check_f(manual::extract_for_og(
                kind(c.v("kind")),
                c.f("og"),
                c.f("v_final_l"),
            )),
            "partial_boil" => {
                let v = match manual::partial_boil(
                    c.f("og_final"),
                    c.f("v_final"),
                    c.f("v_kettle_post"),
                    c.f("loss"),
                ) {
                    Ok(p) => json!({"SG_kazan": p.kettle_sg, "ust_su_L": p.topup_l}),
                    Err(e) => json!({"hata": error_name(e)}),
                };
                c.check(&v);
            }
            "partial_boil_ibu" => c.check_f(manual::partial_boil_ibu(
                &triples(c.v("hops")),
                c.f("sg_kettle"),
                c.f("v_final"),
            )),
            "topoff_temp" => c.check_f(manual::topoff_temp(
                c.f("v_wort"),
                c.f("t_wort"),
                c.f("v_water"),
                c.f("t_target"),
                c.f("c_ratio"),
            )),
            "tinseth_util" => c.check_f(hops::tinseth_utilization(c.f("sg"), c.f("t"))),
            "ftemp" => c.check_f(hops::temperature_factor(c.f("tc"))),
            "tau_h" => c.check_f(manual::tau_h(c.f("v_l"), c.f("tau20"))),
            "t_eq_min" => match manual::t_eq_min(
                c.f("t0"),
                c.f("t_amb"),
                c.f("tau_hours"),
                c.f("dt"),
                c.f("t_stop"),
            ) {
                Ok(x) => c.check_f(x),
                Err(e) => c.check(&json!({ "hata": error_name(e) })),
            },
            "nochill_ibu" => {
                let mode = if c.v("mode").as_str() == Some("kural") {
                    NochillMode::Rule
                } else {
                    NochillMode::Model {
                        t0: opt(c, "T0", 95.0),
                        t_amb: opt(c, "T_amb", 20.0),
                        v_cube: c.girdi.get("v_cube").map(num),
                    }
                };
                match manual::nochill_ibu(&triples(c.v("hops")), c.f("sg"), c.f("v"), mode) {
                    Ok((base, nc, extra)) => c.check(&json!([base, nc, extra])),
                    Err(e) => c.check(&json!({ "hata": error_name(e) })),
                }
            }
            "stall_check" => c.check(&match manual::stall_check(
                &pairs(c.v("readings")),
                c.f("fg_prior"),
            ) {
                None => Value::Null,
                Some(s) => json!(s.as_str()),
            }),
            "few_status" => c.check(&json!(
                manual::few_status(
                    &pairs(c.v("readings")),
                    c.f("og"),
                    c.f("fg_prior"),
                    opt(c, "lag", manual::LAG_H),
                )
                .as_str()
            )),
            "forecast_method" => c.check(&json!(
                manual::forecast_method(
                    c.b("device_sufficient"),
                    usize::try_from(c.i("manual_count")).unwrap(),
                )
                .as_str()
            )),
            "reading_role" => {
                let src = match c.v("source").as_str().unwrap() {
                    "cihaz" => ReadingSource::Device,
                    "elle" => ReadingSource::Manual,
                    "kagit" => ReadingSource::Paper,
                    other => panic!("unknown source {other}"),
                };
                c.check(&json!(
                    manual::reading_role(src, c.b("batch_has_device")).as_str()
                ));
            }
            "nochill_rule_time" => c.check_f(manual::nochill_rule_time(c.f("t_min"))),
            _ => return false,
        }
        true
    });
}
