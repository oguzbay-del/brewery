//! Section 08 parity against `vektorler/08.json` (ADR-0003).
#![allow(clippy::expect_used, clippy::too_many_lines)]
mod ortak;

use engine::brewery as b;
use ortak::Case;
use rust_decimal::Decimal;
use serde_json::{Value, json};

/// Decimal input (plain string) or expected value (`{"ondalik": "..."}`).
fn dec_in(v: &Value) -> Decimal {
    v.as_str()
        .or_else(|| v["ondalik"].as_str())
        .unwrap_or_else(|| panic!("not a decimal: {v}"))
        .parse()
        .expect("decimal")
}

fn dec(c: &Case, name: &str) -> Decimal {
    dec_in(c.v(name))
}

/// Decimals compare by value (Python's printed scale does not matter).
fn same(c: &Case, got: Decimal, expected: &Value, what: &str) {
    assert_eq!(
        got,
        dec_in(expected),
        "case {} `{}` {what}: got {got}, expected {expected}",
        c.index,
        c.fonk
    );
}

fn num(v: &Value) -> f64 {
    v.as_f64().expect("number")
}

fn part(v: &Value) -> b::BlendPart {
    b::BlendPart {
        v: num(&v["V"]),
        abv: num(&v["ABV"]),
        ibu: num(&v["IBU"]),
        srm: num(&v["SRM"]),
        co2: num(&v["CO2"]),
        p: num(&v["P"]),
    }
}

#[test]
fn parite_08() {
    ortak::run("08", |c| {
        match c.fonk.as_str() {
            "plato_to_sg" => c.check_f(b::plato_to_sg(c.f("p"))),
            "fifo_consume" => {
                let lots: Vec<b::Lot> = c
                    .v("lots")
                    .as_array()
                    .expect("lots")
                    .iter()
                    .map(|l| b::Lot {
                        lot: l[0].as_str().expect("lot name").to_owned(),
                        left: dec_in(&l[1]),
                        price: dec_in(&l[2]),
                    })
                    .collect();
                let r = b::fifo_consume(&lots, dec(c, "qty"));
                let exp_rows = c.beklenen["rows"].as_array().expect("rows");
                assert_eq!(r.rows.len(), exp_rows.len(), "case {}: row count", c.index);
                for (x, e) in r.rows.iter().zip(exp_rows) {
                    assert_eq!(e[0].as_str(), Some(x.lot.as_str()), "case {}: lot", c.index);
                    same(c, x.take, &e[1], "take");
                    same(c, x.price, &e[2], "price");
                    same(c, x.cost, &e[3], "row cost");
                }
                same(c, r.cost, &c.beklenen["cost"], "cost");
                same(c, r.missing, &c.beklenen["missing"], "missing");
            }
            "landed_price" => {
                let r = b::landed_price(
                    dec(c, "invoice_price"),
                    dec(c, "qty"),
                    dec(c, "shipping_share"),
                );
                match r {
                    Some(x) => same(c, x, &c.beklenen, "price"),
                    None => c.check(&json!({"hata": "ValueError"})),
                }
            }
            "energy_cost_electric" => {
                same(
                    c,
                    b::energy_cost_electric(dec(c, "kwh"), dec(c, "price_kwh")),
                    &c.beklenen,
                    "cost",
                );
            }
            "kwh_from_heat" => c.check_f(b::kwh_from_heat(c.f("kj"), c.f("eff"))),
            "gas_m3_from_heat" => {
                c.check_f(b::gas_m3_from_heat(c.f("kj"), c.f("eff"), c.f("hu_kj_m3")));
            }
            "batch_cost" => {
                let r = b::batch_cost(
                    dec(c, "ingredient_cost"),
                    dec(c, "water_l"),
                    dec(c, "water_price_m3"),
                    dec(c, "energy_cost"),
                    dec(c, "pack_units"),
                    dec(c, "pack_unit_cost"),
                    dec(c, "packaged_l"),
                );
                let Some(r) = r else {
                    c.check(&json!({"hata": "ValueError"}));
                    return true;
                };
                let e = &c.beklenen;
                same(c, r.ingredients, &e["malzeme"], "ingredients");
                same(c, r.water, &e["su"], "water");
                same(c, r.energy, &e["enerji"], "energy");
                same(c, r.packaging, &e["ambalaj"], "packaging");
                same(c, r.total, &e["toplam"], "total");
                same(c, r.per_litre, &e["litre"], "per litre");
            }
            "blend" => {
                let parts: Vec<b::BlendPart> = c
                    .v("parts")
                    .as_array()
                    .expect("parts")
                    .iter()
                    .map(part)
                    .collect();
                let r = b::blend(&parts);
                c.check(&json!({
                    "V": r.v, "ABV": r.abv, "IBU": r.ibu, "SRM": r.srm, "CO2": r.co2, "P": r.p
                }));
            }
            "blend_volume_for_target" => {
                // A non-finite result must fail, not serialise to `null` (T1.12 C16).
                match b::blend_volume_for_target(c.f("v1"), c.f("x1"), c.f("x2"), c.f("xt")) {
                    Some(x) => c.check_f(x),
                    None => c.check(&Value::Null),
                }
            }
            "cells_needed_billion" => c.check_f(b::cells_needed_billion(
                c.f("pitch_m_ml_p"),
                c.f("v_l"),
                c.f("p"),
            )),
            "hemocytometer" => c.check_f(b::hemocytometer(
                c.f("total_counted"),
                c.f("squares_counted"),
                c.f("dilution"),
            )),
            "viability_stain" => c.check_f(b::viability_stain(c.f("unstained"), c.f("stained"))),
            "slurry_volume_ml" => c.check_f(b::slurry_volume_ml(
                c.f("cells_billion"),
                c.v("density_cells_ml").as_f64(),
                c.f("viability"),
                c.f("default_density"),
            )),
            "extract_kg" => c.check_f(b::extract_kg(c.f("v_l"), c.f("p"), c.b("hot"))),
            "brewhouse_yield" => c.check_f(b::brewhouse_yield(
                c.f("v_l"),
                c.f("p"),
                c.f("malt_kg"),
                c.b("hot"),
            )),
            "lab_extract_as_is" => c.check_f(b::lab_extract_as_is(
                c.f("fgdb"),
                c.f("moisture"),
                c.f("fc_diff"),
            )),
            "first_wort_max_p" => c.check_f(b::first_wort_max_p(
                c.f("grain_kg"),
                c.f("extract_as_is_pct"),
                c.f("moisture_pct"),
                c.f("mash_water_l"),
            )),
            "conversion_eff" => {
                c.check_f(b::conversion_eff(c.f("fw_measured_p"), c.f("fw_max_p")));
            }
            "malt_from_analysis" => {
                let r = b::malt_from_analysis(c.f("fgdb"), c.f("moisture"), c.f("fc_diff"));
                c.check(&json!({
                    "FGAI": r.fgai, "CGDB": r.cgdb, "CGAI_yield": r.cgai_yield,
                    "potansiyel": r.potential, "potansiyel_BS": r.potential_bs, "PPG": r.ppg
                }));
            }
            "ebc_from_lab" => {
                let r = b::ebc_from_lab(c.f("ebc"));
                c.check(&json!({"EBC": r.ebc, "SRM": r.srm, "°L": r.lovibond}));
            }
            "yeast_reuse_warnings" => {
                let w = b::yeast_reuse_warnings(
                    c.f("generation"),
                    c.f("days_since_harvest"),
                    c.v("viability").as_f64(),
                    c.f("max_generation"),
                    c.b("history_differs"),
                );
                let codes: Vec<&str> = w.iter().map(|x| x.as_str()).collect();
                c.check(&json!(codes));
            }
            "diastatic_check" => {
                let grains: Vec<(f64, f64)> = c
                    .v("grains")
                    .as_array()
                    .expect("grains")
                    .iter()
                    .map(|g| (num(&g[0]), num(&g[1])))
                    .collect();
                let r = b::diastatic_check(&grains, c.f("threshold"));
                let uyari: Vec<&str> = if r.low_diastatic_power {
                    vec!["dusuk_diastatik_guc"]
                } else {
                    vec![]
                };
                c.check(&json!({"ortalama": r.average, "uyari": uyari}));
            }
            _ => return false,
        }
        true
    });
}
