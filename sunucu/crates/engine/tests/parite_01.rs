//! Section 01 parity: `engine::density` against `vektorler/01.json`.
mod ortak;

use engine::density as d;
use engine::units;
use serde_json::json;

#[test]
fn parite_01() {
    ortak::run("01", |c| {
        match c.fonk.as_str() {
            "to_plato" => c.check_f(units::to_plato(c.f("sg"))),
            "to_sg" => c.check_f(units::to_sg(c.f("plato"))),
            "sg_to_brix" => c.check_f(d::sg_to_brix(c.f("sg"))),
            "brix_to_sg" => c.check_f(d::brix_to_sg(c.f("brix"))),
            "brix_to_ri" => c.check_f(d::brix_to_ri(c.f("brix"))),
            "ri_to_brix" => c.check_f(d::ri_to_brix(c.f("ri"))),
            "real_extract" => c.check_f(d::real_extract(c.f("og"), c.f("fg"))),
            "abw" => c.check_f(d::abw(c.f("og"), c.f("fg"))),
            "abv" => c.check_f(d::abv(c.f("og"), c.f("fg"))),
            "apparent_attenuation" => c.check_f(d::apparent_attenuation(c.f("og"), c.f("fg"))),
            "real_attenuation" => c.check_f(d::real_attenuation(c.f("og"), c.f("fg"))),
            "alcohol_calories" => c.check_f(d::alcohol_calories(c.f("og"), c.f("fg"))),
            "extract_calories" => c.check_f(d::extract_calories(c.f("og"), c.f("fg"))),
            "uk_tax_abv" => c.check_f(d::uk_tax_abv(c.f("og"), c.f("fg"))),
            "morey_srm" => c.check_f(d::morey_srm(c.f("mcu_sum"), c.f("volume_gal"))),
            "hydrometer_correction" => c.check_f(d::hydrometer_correction(
                c.f("sg"),
                c.f("temp_f"),
                c.f("calib_f"),
            )),
            "refractometer_unfermented" => c.check_f(d::refractometer_unfermented(
                c.f("brix"),
                c.f("zero"),
                c.f("wcf"),
            )),
            "refractometer_fermenting" => c.check_f(d::refractometer_fermenting(
                c.f("og"),
                c.f("brix"),
                c.f("zero"),
            )),
            "refractometer_finished" => {
                let r = d::refractometer_finished(c.f("fg"), c.f("brix"), c.f("zero"));
                c.check(&json!([r.og, r.abw_percent, r.re, r.oe]));
            }
            "fg_from_tolerance" => {
                c.check_f(d::fg_from_tolerance(c.f("og"), c.f("fg"), c.f("tol")));
            }
            "gravity_adjust_extract_oz" => c.check_f(d::gravity_adjust_extract_oz(
                c.f("current"),
                c.f("target"),
                c.f("volume_floz"),
                c.i("kind"),
            )),
            "gravity_adjust_water_floz" => c.check_f(d::gravity_adjust_water_floz(
                c.f("current"),
                c.f("target"),
                c.f("volume_floz"),
            )),
            _ => return false,
        }
        true
    });
}
