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

fn check_result(c: &Case, r: &Result<Decimal, engine::Error>) {
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
    p.abv_pct = opt_dec(c, "abv");
    p.plato_p = opt_dec(c, "plato");
    p.rate = opt_dec(c, "rate");
    p.bands = &bands_v;
    p.tiers = &tiers_v;
    if let Some(y) = opt_dec(c, "ytd") {
        p.ytd_hl = y;
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
    if c.fonk == "tax_line_detail" {
        let opt = |x: Option<Decimal>| x.map_or(Value::Null, dec_json);
        match bt::tax_line_detail(&p) {
            Ok(d) => {
                c.check(&json!({"vergi": dec_json(d.tax), "lpa": opt(d.lpa), "oran": opt(d.rate)}));
            }
            Err(_) => c.check(&json!({ "hata": "ValueError" })),
        }
        return;
    }
    check_result(c, &bt::tax_line(&p));
}

fn bands_of(v: Option<&Value>) -> Vec<bt::Band> {
    v.map_or_else(Vec::new, |list| {
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

fn chain_step(v: &Value) -> Result<bt::ChainStep, engine::Error> {
    let get_dec = |k: &str| v.get(k).map(dec_str);
    let conditions = v.get("indirim_kosul").filter(|k| !k.is_null());
    Ok(bt::ChainStep {
        position: u32::try_from(v["konum"].as_i64().unwrap()).unwrap_or(0),
        rate_class: v["sinif"].as_str().unwrap().to_owned(),
        valid_from: v["baslangic"].as_str().unwrap().to_owned(),
        valid_to: v["bitis"].as_str().map(str::to_owned),
        method: bt::Method::parse(v["yontem"].as_str().unwrap())?,
        rate: get_dec("rate"),
        bands: bands_of(v.get("bands")),
        tiers: bands_of(v.get("tiers")),
        inclusive: v.get("inclusive").map(|b| b.as_bool().unwrap()),
        plato_trunc: v.get("plato_trunc").is_none_or(|b| b.as_bool().unwrap()),
        discount_per_lpa: get_dec("discount_per_lpa").unwrap_or(Decimal::ZERO),
        discount_conditions: bt::DiscountConditions {
            abv_below: conditions.and_then(|k| k.get("abv_alti")).map(dec_str),
            ytd_hl_below: conditions.and_then(|k| k.get("ytd_hl_alti")).map(dec_str),
        },
        input: match v.get("input_dp") {
            None => bt::InputRounding::Undefined,
            Some(Value::Null) => bt::InputRounding::NoRounding,
            Some(n) => bt::InputRounding::Places {
                dp: u32::try_from(n.as_i64().unwrap()).unwrap(),
                mode: bt::RoundMode::parse(v["input_mode"].as_str().unwrap()).unwrap(),
            },
        },
    })
}

fn run_chain(c: &Case) {
    let error = || c.check(&json!({ "hata": "ValueError" }));
    let Ok(steps) = c
        .v("steps")
        .as_array()
        .unwrap()
        .iter()
        .map(chain_step)
        .collect::<Result<Vec<_>, _>>()
    else {
        return error();
    };
    let class = c.v("sinif").as_str().unwrap();
    let (start, end) = (
        c.v("baslangic").as_str().unwrap(),
        c.v("bitis").as_str().unwrap(),
    );
    if c.fonk == "select_chain" {
        match bt::select_chain(&steps, class, start, end) {
            Ok(sel) => c.check(&json!(sel.iter().map(|a| a.position).collect::<Vec<_>>())),
            Err(_) => error(),
        }
        return;
    }
    let lines: Vec<bt::ChainLine> = c
        .v("satirlar")
        .as_array()
        .unwrap()
        .iter()
        .map(|l| bt::ChainLine {
            volume_l: dec_str(&l["volume_l"]),
            abv_pct: l.get("abv").map(dec_str),
            plato_p: l.get("plato").map(dec_str),
        })
        .collect();
    let opt = |x: Option<Decimal>| x.map_or(Value::Null, dec_json);
    match bt::tax_chain(&steps, class, start, end, &lines, dec(c, "ytd")) {
        Ok(r) => {
            let lines: Vec<Value> = r
                .lines
                .iter()
                .map(|l| {
                    let parts: Vec<Value> = l
                        .steps
                        .iter()
                        .map(|a| {
                            json!({
                                "konum": a.position,
                                "yontem": method_name(a.method),
                                "vergi": dec_json(a.detail.tax),
                                "lpa": opt(a.detail.lpa),
                                "oran": opt(a.detail.rate),
                                "indirim_uygulandi": a.discount_applied,
                            })
                        })
                        .collect();
                    json!({"ytd_basi": dec_json(l.ytd_start_hl), "vergi": dec_json(l.tax), "adimlar": parts})
                })
                .collect();
            c.check(&json!({"toplam": dec_json(r.total), "satirlar": lines}));
        }
        Err(_) => error(),
    }
}

fn method_name(m: bt::Method) -> &'static str {
    match m {
        bt::Method::Volume => "hacim",
        bt::Method::PureAlcohol => "saf_alkol",
        bt::Method::HlPlato => "hl_plato",
        bt::Method::PlatoBand => "plato_bandi",
        bt::Method::TieredVolume => "kademeli_hacim",
    }
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
                let a = bt::abv_from(c.f("og"), c.f("fg"));
                let (abv, w, re) = (a.abv_pct, a.abw_pct, a.real_extract_p);
                c.check(&json!([abv, w, re]));
            }
            "label_energy" => {
                let abv_measured = c.v("abv_measured").as_f64();
                let e = bt::label_energy(c.f("og"), c.f("fg"), c.f("protein_g100ml"), abv_measured);
                c.check_res(e.map(|e| {
                    json!({
                        "ABV": e.abv_pct,
                        "alkol_g": e.alcohol_g,
                        "gercek_ekstrakt_g": e.real_extract_g,
                        "karbonhidrat_g": e.carbohydrate_g,
                        "protein_g": e.protein_g,
                        "kJ": e.kj,
                        "kcal": e.kcal,
                    })
                }));
            }
            "tax_line" | "tax_line_detail" => run_tax_line(c),
            "select_chain" | "tax_chain" => run_chain(c),
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
            "row_status" => c.check(&json!(match bt::row_status(
                c.v("resmi_kaynak").as_str(),
                c.v("tarih").as_str()
            ) {
                bt::RowStatus::Verified => "doğrulandı",
                bt::RowStatus::Unverified => "doğrulanacak",
            })),
            "residual_co2" => c.check_f(engine::yeast::residual_co2(engine::units::c_to_f(
                c.f("t_c"),
            ))),
            "co2_pressure_psi" => c.check_f(engine::yeast::co2_psi(
                c.f("vol"),
                engine::units::c_to_f(c.f("t_c")),
            )),
            "spund" => {
                let Ok(s) = bt::spund(c.f("target_vol"), c.f("t_c"), c.f("fg_expected")) else {
                    c.check(&json!({ "hata": "ValueError" }));
                    return true;
                };
                c.check(&json!({
                    "kalinti_vol": s.residual_co2_vol,
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
                let w = bt::spund_warnings(&bt::SpundWarningsInput {
                    target_co2_vol: c.f("target_vol"),
                    t_c: c.f("t_c"),
                    tank_max_psi: c.v("tank_max_psi").as_f64(),
                    fg_range: range,
                    wide_u: c.i("wide_u"),
                });
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
            volume_l: dec_str(&e[2]),
        });
    }
    let deduct: Vec<bt::VolumeEventKind> = c
        .v("deduct")
        .as_array()
        .unwrap()
        .iter()
        .map(|k| bt::VolumeEventKind::parse(k.as_str().unwrap()).unwrap())
        .collect();
    match bt::taxable_volume(&bt::TaxableVolumeInput { events: &events, trigger, start: c.v("start").as_str().unwrap(), end: c.v("end").as_str().unwrap(), deduct: &deduct }) {
        Ok(r) => c.check(&json!({
            "vergilenecek": dec_json(r.taxable_l), "vergilenen": dec_json(r.taxed_l),
            "imha": dec_json(r.destroyed_l), "iade": dec_json(r.returned_l), "lab": dec_json(r.lab_l),
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
    let e = engine::Error::invalid("bands", "no band matched");
    assert!(matches!(e, engine::Error::InvalidInput { .. }));
}
