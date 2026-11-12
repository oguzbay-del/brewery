//! Section 09 parity: label energy, tax engine (Decimal), spunding against `vektorler/09.json`.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod ortak;

use engine::brewery_tax as bt;
use engine::units;
use ortak::Case;
use rust_decimal::Decimal;
use serde_json::{Value, json};

fn dec_str(v: &Value) -> Decimal {
    // Inputs are plain strings; expected results are `{"ondalik": "..."}`.
    v.as_str()
        .or_else(|| v["ondalik"].as_str())
        .unwrap_or_else(|| panic!("not a decimal: {v}"))
        .parse()
        .unwrap()
}

fn dec(c: &Case, name: &str) -> Decimal {
    dec_str(c.v(name))
}

fn opt_dec(c: &Case, name: &str) -> Option<Decimal> {
    c.girdi.get(name).map(dec_str)
}

fn mode(c: &Case, name: &str) -> bt::RoundMode {
    bt::RoundMode::parse(c.v(name).as_str().unwrap()).unwrap()
}

fn bands(c: &Case, name: &str) -> Vec<bt::Band> {
    c.girdi.get(name).map_or_else(Vec::new, |list| {
        list.as_array()
            .unwrap()
            .iter()
            .map(|b| bt::Band {
                upper: if b[0].is_null() {
                    None
                } else {
                    Some(dec_str(&b[0]))
                },
                rate: dec_str(&b[1]),
            })
            .collect()
    })
}

fn dp(c: &Case) -> u32 {
    u32::try_from(c.i("dp")).unwrap()
}

fn dec_json(x: Decimal) -> Value {
    json!({ "ondalik": x.to_string() })
}

fn check_result(c: &Case, r: &Result<Decimal, bt::TaxError>) {
    match *r {
        Ok(x) => c.check(&dec_json(x)),
        Err(_) => c.check(&json!({ "hata": "ValueError" })),
    }
}

fn run_tax_line(c: &Case) {
    let method = match bt::Method::parse(c.v("method").as_str().unwrap()) {
        Ok(m) => m,
        Err(e) => return check_result(c, &Err(e)),
    };
    let bands_v = bands(c, "bands");
    let tiers_v = bands(c, "tiers");
    let mut p = bt::TaxLine::new(method, dec(c, "volume_l"));
    p.abv = opt_dec(c, "abv");
    p.plato = opt_dec(c, "plato");
    p.rate = opt_dec(c, "rate");
    p.bands = &bands_v;
    p.tiers = &tiers_v;
    if let Some(y) = opt_dec(c, "ytd") {
        p.ytd = y;
    }
    if let Some(t) = c.girdi.get("plato_trunc") {
        p.plato_trunc = t.as_bool().unwrap();
    }
    if let Some(x) = opt_dec(c, "discount_per_lpa") {
        p.discount_per_lpa = x;
    }
    p.inclusive = c.girdi.get("inclusive").map(|b| b.as_bool().unwrap());
    p.input = match c.girdi.get("input_dp") {
        None => bt::InputRounding::Undefined,
        Some(Value::Null) => bt::InputRounding::NoRounding,
        Some(n) => bt::InputRounding::Places {
            dp: u32::try_from(n.as_i64().unwrap()).unwrap(),
            mode: mode(c, "input_mode"),
        },
    };
    check_result(c, &bt::tax_line(&p));
}

#[test]
fn parite_09() {
    ortak::run("09", |c| {
        match c.fonk.as_str() {
            "to_plato" => c.check_f(units::to_plato(c.f("sg"))),
            "to_sg" => c.check_f(units::to_sg(c.f("plato"))),
            "real_extract" => c.check_f(bt::real_extract(c.f("oe"), c.f("ae"))),
            "abw" => c.check_f(bt::abw(c.f("oe"), c.f("re"))),
            "abv_from" => {
                let (abv, w, re) = bt::abv_from(c.f("og"), c.f("fg"));
                c.check(&json!([abv, w, re]));
            }
            "label_energy" => {
                let abv_measured = c.v("abv_measured").as_f64();
                let e = bt::label_energy(c.f("og"), c.f("fg"), c.f("protein_g100ml"), abv_measured);
                c.check_res(
                    e.map(|e| {
                        json!({
                            "ABV": e.abv,
                            "alkol_g": e.alcohol_g,
                            "gercek_ekstrakt_g": e.real_extract_g,
                            "karbonhidrat_g": e.carbohydrate_g,
                            "protein_g": e.protein_g,
                            "kJ": e.kj,
                            "kcal": e.kcal,
                        })
                    })
                    .map_err(engine::Error::from),
                );
            }
            "tax_line" => run_tax_line(c),
            "round_in" => {
                let rounding = if c.v("dp").is_null() {
                    None
                } else {
                    Some((dp(c), mode(c, "mode")))
                };
                check_result(c, &bt::round_in(dec(c, "x"), rounding));
            }
            "band" => {
                let b = bands(c, "bands");
                let r = bt::band(dec(c, "value"), &b, c.b("inclusive"));
                check_result(c, &r);
            }
            "money" => check_result(c, &bt::money(dec(c, "x"), dp(c), mode(c, "mode"))),
            "row_status" => c.check(&json!(bt::row_status(
                c.v("resmi_kaynak").as_str(),
                c.v("tarih").as_str()
            ))),
            "residual_co2" => c.check_f(bt::residual_co2(c.f("t_c"))),
            "co2_pressure_psi" => c.check_f(bt::co2_pressure_psi(c.f("t_c"), c.f("vol"))),
            "spund" => {
                let Ok(s) = bt::spund(c.f("target_vol"), c.f("t_c"), c.f("fg_expected")) else {
                    c.check(&json!({ "hata": "ValueError" }));
                    return true;
                };
                c.check(&json!({
                    "kalinti_vol": s.residual_vol,
                    "gereken_g_L": s.needed_g_per_l,
                    "kapama_SG": s.closing_sg,
                    "puan_once": s.points_before,
                    "ayar_psi": s.setting_psi,
                    "ayar_bar": s.setting_bar,
                }));
            }
            "label_declaration" => c.check(&json!(bt::label_declaration(dec(c, "abv")).as_str())),
            "taxable_volume" => taxable_volume_case(c),
            "spund_warnings" => {
                let range = c
                    .v("fg_range")
                    .as_array()
                    .map(|r| (r[0].as_f64().unwrap(), r[1].as_f64().unwrap()));
                let w = bt::spund_warnings(
                    c.f("target_vol"),
                    c.f("t_c"),
                    c.v("tank_max_psi").as_f64(),
                    range,
                    c.i("wide_u"),
                );
                let codes: Vec<&str> = w.warnings.iter().map(|x| x.as_str()).collect();
                c.check(&json!({"uyari": codes, "engellendi": w.blocked}));
            }
            _ => return false,
        }
        true
    });
}

fn taxable_volume_case(c: &Case) {
    let error = || c.check(&json!({ "hata": "ValueError" }));
    let trigger = match c.v("trigger").as_str().unwrap() {
        "ambalaj" => bt::TaxTrigger::Packaging,
        "cikis" => bt::TaxTrigger::Release,
        "satis" => bt::TaxTrigger::Sale,
        _ => return error(),
    };
    let mut events = Vec::new();
    for e in c.v("events").as_array().unwrap() {
        let Some(kind) = bt::VolumeEventKind::parse(e[1].as_str().unwrap()) else {
            return error();
        };
        events.push(bt::VolumeEvent {
            date: e[0].as_str().unwrap().to_owned(),
            kind,
            litres: dec_str(&e[2]),
        });
    }
    let deduct: Vec<bt::VolumeEventKind> = c
        .v("deduct")
        .as_array()
        .unwrap()
        .iter()
        .map(|k| bt::VolumeEventKind::parse(k.as_str().unwrap()).unwrap())
        .collect();
    match bt::taxable_volume(
        &events,
        trigger,
        c.v("start").as_str().unwrap(),
        c.v("end").as_str().unwrap(),
        &deduct,
    ) {
        Ok(r) => c.check(&json!({
            "vergilenecek": dec_json(r.taxable), "vergilenen": dec_json(r.taxed),
            "imha": dec_json(r.destroyed), "iade": dec_json(r.returned), "lab": dec_json(r.lab),
        })),
        Err(_) => error(),
    }
}

/// T1.16: non-finite energy input, over-long rounding rules and the error conversion.
#[test]
fn invalid_inputs_are_errors() {
    assert!(bt::label_energy(f64::NAN, 1.010, 0.4, None).is_err());
    assert!(bt::label_energy(1.050, 1.010, 0.4, Some(f64::INFINITY)).is_err());
    let one = Decimal::from(1);
    assert!(bt::money(one, 13, bt::RoundMode::HalfUp).is_err());
    assert!(bt::round_in(one, Some((30, bt::RoundMode::HalfUp))).is_err());
    let e: engine::Error = bt::TaxError::NoBand.into();
    assert!(matches!(e, engine::Error::InvalidInput { .. }));
}
