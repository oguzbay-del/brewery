//! Section 08 parity against `vektorler/08.json` (ADR-0003).
#![allow(clippy::expect_used, clippy::too_many_lines)]
mod ortak;

use engine::brewery as b;
use serde_json::{Value, json};

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
                        left: num(&l[1]),
                        price: num(&l[2]),
                    })
                    .collect();
                let r = b::fifo_consume(&lots, c.f("qty"));
                let rows: Vec<Value> = r
                    .rows
                    .iter()
                    .map(|x| json!([x.lot, x.take, x.price, x.cost]))
                    .collect();
                c.check(&json!({"rows": rows, "cost": r.cost, "missing": r.missing}));
            }
            "landed_price" => c.check_f(b::landed_price(
                c.f("invoice_price"),
                c.f("qty"),
                c.f("shipping_share"),
            )),
            "energy_cost_electric" => {
                c.check_f(b::energy_cost_electric(c.f("kwh"), c.f("price_kwh")));
            }
            "kwh_from_heat" => c.check_f(b::kwh_from_heat(c.f("kj"), c.f("eff"))),
            "gas_m3_from_heat" => {
                c.check_f(b::gas_m3_from_heat(c.f("kj"), c.f("eff"), c.f("hu_kj_m3")));
            }
            "batch_cost" => {
                let r = b::batch_cost(
                    c.f("ingredient_cost"),
                    c.f("water_l"),
                    c.f("water_price_m3"),
                    c.f("energy_cost"),
                    c.f("pack_units"),
                    c.f("pack_unit_cost"),
                    c.f("packaged_l"),
                );
                c.check(&json!({
                    "malzeme": r.ingredients, "su": r.water, "enerji": r.energy,
                    "ambalaj": r.packaging, "toplam": r.total, "litre": r.per_litre
                }));
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
                let r = b::blend_volume_for_target(c.f("v1"), c.f("x1"), c.f("x2"), c.f("xt"));
                c.check(&r.map_or(Value::Null, |x| json!(x)));
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
            _ => return false,
        }
        true
    });
}
