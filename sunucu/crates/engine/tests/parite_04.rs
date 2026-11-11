//! Section 04 parity: yeast, mead, carbonation against `vektorler/04.json`.
mod ortak;

use engine::yeast;
use serde_json::json;

#[test]
fn parite_04() {
    ortak::run("04", |c| {
        match c.fonk.as_str() {
            "l_to_floz" => c.check_f(yeast::l_to_floz(c.f("litres"))),
            "gal_from_l" => c.check_f(yeast::gal_from_l(c.f("litres"))),
            "floz_to_l" => c.check_f(yeast::floz_to_l(c.f("floz"))),
            "brix_to_sg" => c.check_f(yeast::brix_to_sg(c.f("brix"))),
            "viability" => c.check_f(yeast::viability(
                c.f("v0"),
                c.f("rate"),
                c.f("days"),
                c.f("floor"),
            )),
            "innoc_rate" => c.check_f(yeast::innoc_rate(c.f("g"))),
            "growth_rate" => c.check_f(yeast::growth_rate(c.f("i"), c.b("stir"))),
            "cells_needed" => {
                c.check_f(yeast::cells_needed(c.f("rate"), c.f("volume_l"), c.f("sg")));
            }
            "new_starter_l" => {
                c.check_f(yeast::new_starter_l(c.f("need"), c.f("avail"), c.b("stir")));
            }
            "two_stage_tool" => {
                let (a, b) = yeast::two_stage_tool(c.f("need"), c.f("avail"), c.b("stir"));
                c.check(&json!([a, b]));
            }
            "dme_oz" => c.check_f(yeast::dme_oz(c.f("vol_floz"), c.f("og"))),
            "sugar_g" => c.check_f(yeast::sugar_g(
                c.f("vols"),
                c.f("temp_f"),
                c.f("gal"),
                c.b("dme"),
            )),
            "residual_co2" => c.check_f(yeast::residual_co2(c.f("temp_f"))),
            "co2_psi" => c.check_f(yeast::co2_psi(c.f("vols"), c.f("temp_f"))),
            "so2_rec" => c.check_f(yeast::so2_rec(c.f("ph"))),
            "kmeta_g" => c.check_f(yeast::kmeta_g(c.f("v_gal"), c.f("target"), c.f("free"))),
            "sorbate_g" => c.check_f(yeast::sorbate_g(c.f("v_gal"))),
            "mead_pitch" => c.check_f(yeast::mead_pitch(c.f("sg"))),
            "tosna" => c.check_f(yeast::tosna(
                c.f("sg"),
                c.f("v_gal"),
                c.f("pct"),
                c.i("fac"),
                c.i("method"),
            )),
            "yan" => c.check_f(yeast::yan(c.i("method"), c.f("grams"), c.f("v_l"))),
            _ => return false,
        }
        true
    });
}
