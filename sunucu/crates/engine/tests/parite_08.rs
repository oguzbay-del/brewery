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
        volume_l: num(&v["V"]),
        abv_pct: num(&v["ABV"]),
        ibu: num(&v["IBU"]),
        srm: num(&v["SRM"]),
        co2_vol: num(&v["CO2"]),
        wort_p: num(&v["P"]),
    }
}

#[test]
fn parite_08() {
    ortak::run("08", |c| {
        match c.fonk.as_str() {
            "plato_to_sg" => c.check_f(engine::units::to_sg(c.f("p"))),
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
                    Ok(x) => same(c, x, &c.beklenen, "price"),
                    Err(_) => c.check_hata(),
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
            "kwh_from_heat" => c.check_r(b::kwh_from_heat(c.f("kj"), c.f("eff"))),
            "gas_m3_from_heat" => {
                c.check_r(b::gas_m3_from_heat(c.f("kj"), c.f("eff"), c.f("hu_kj_m3")));
            }
            "batch_cost" => {
                let r = b::batch_cost(&b::BatchCostInput {
                    ingredient_cost: dec(c, "ingredient_cost"),
                    water_l: dec(c, "water_l"),
                    water_price_m3: dec(c, "water_price_m3"),
                    energy_cost: dec(c, "energy_cost"),
                    pack_units: dec(c, "pack_units"),
                    pack_unit_cost: dec(c, "pack_unit_cost"),
                    packaged_l: dec(c, "packaged_l"),
                });
                let Ok(r) = r else {
                    c.check_hata();
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
                c.check_res(b::blend(&parts).map(|r| {
                    json!({
                        "V": r.volume_l, "ABV": r.abv_pct, "IBU": r.ibu, "SRM": r.srm, "CO2": r.co2_vol, "P": r.wort_p
                    })
                }));
            }
            "blend_volume_for_target" => {
                // A non-finite result must fail, not serialise to `null` (T1.12 C16).
                c.check_r(b::blend_volume_for_target(
                    c.f("v1"),
                    c.f("x1"),
                    c.f("x2"),
                    c.f("xt"),
                ));
            }
            "cells_needed_billion" => c.check_r(b::cells_needed_billion(
                c.f("pitch_m_ml_p"),
                c.f("v_l"),
                c.f("p"),
            )),
            "hemocytometer" => c.check_r(b::hemocytometer(
                c.f("total_counted"),
                c.f("squares_counted"),
                c.f("dilution"),
            )),
            "viability_stain" => c.check_r(b::viability_stain(c.f("unstained"), c.f("stained"))),
            "slurry_volume_ml" => c.check_r(b::slurry_volume_ml(
                c.f("cells_billion"),
                c.v("density_cells_ml").as_f64(),
                c.f("viability"),
                c.f("default_density"),
            )),
            "extract_kg" => c.check_r(b::extract_kg(c.f("v_l"), c.f("p"), c.b("hot"))),
            "brewhouse_yield" => c.check_r(b::brewhouse_yield(
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
            "first_wort_max_p" => c.check_r(b::first_wort_max_p(
                c.f("grain_kg"),
                c.f("extract_as_is_pct"),
                c.f("moisture_pct"),
                c.f("mash_water_l"),
            )),
            "conversion_eff" => {
                c.check_r(b::conversion_eff(c.f("fw_measured_p"), c.f("fw_max_p")));
            }
            "lab_ratio" => c.check_r(b::lab_ratio(c.f("brewhouse_pct"), c.f("lab_extract_pct"))),
            "lauter_eff" => c.check_r(b::lauter_eff(
                c.f("brewhouse_pct"),
                c.f("lab_extract_pct"),
                c.f("conversion"),
            )),
            "malt_from_analysis" => {
                let r = b::malt_from_analysis(c.f("fgdb"), c.f("moisture"), c.f("fc_diff"));
                c.check(&json!({
                    "FGAI": r.fgai_pct, "CGDB": r.cgdb_pct, "CGAI_yield": r.cgai_yield_pct,
                    "potansiyel": r.potential, "potansiyel_BS": r.potential_bs, "PPG": r.ppg
                }));
            }
            "ebc_from_lab" => {
                let r = b::ebc_from_lab(c.f("ebc"));
                c.check(&json!({"EBC": r.ebc, "SRM": r.srm, "°L": r.lovibond}));
            }
            "yeast_reuse_warnings" => {
                let w = b::yeast_reuse_warnings(&b::YeastReuseInput {
                    generation: c.f("generation"),
                    harvest_age_days: c.f("days_since_harvest"),
                    viability: c.v("viability").as_f64(),
                    max_generation: c.f("max_generation"),
                    history_differs: c.b("history_differs"),
                });
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
                c.check(&json!({"ortalama": r.average_lintner, "uyari": uyari}));
            }
            _ => return false,
        }
        true
    });
}

/// T1.16: degenerate input is an error, not NaN or infinity.
#[test]
fn invalid_inputs_are_errors() {
    assert!(b::blend(&[]).is_err());
    assert!(b::hemocytometer(100.0, 0.0, 1.0).is_err());
    assert!(b::viability_stain(0.0, 0.0).is_err());
    assert!(b::kwh_from_heat(1.0, 0.0).is_err());
    assert!(b::kwh_from_heat(f64::NAN, 0.9).is_err());
    assert!(b::conversion_eff(10.0, 0.0).is_err());
    assert!(b::lab_ratio(70.0, 0.0).is_err());
    assert!(b::lauter_eff(70.0, 0.0, 0.9).is_err());
    assert!(b::lauter_eff(70.0, 78.0, 0.0).is_err());
    assert!(b::lauter_eff(70.0, 78.0, f64::NAN).is_err());
    assert!(b::brewhouse_yield(20.0, 12.0, 0.0, true).is_err());
    assert!(b::slurry_volume_ml(100.0, Some(0.0), 0.0, 1.0e9).is_err());
    assert!(b::first_wort_max_p(0.0, 80.0, 4.0, 0.0).is_err());
}
