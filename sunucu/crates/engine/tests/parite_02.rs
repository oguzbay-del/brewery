//! Section 02 parity: hop utilisation and IBU against `vektorler/02.json`.
mod ortak;

use engine::hops;

#[test]
fn parite_02() {
    ortak::run("02", |c| {
        match c.fonk.as_str() {
            "tinseth_utilization" => c.check_f(hops::tinseth_utilization(c.f("sg"), c.f("t"))),
            "temperature_factor" => c.check_f(hops::temperature_factor(c.f("temp_c"))),
            "rager_table" => c.check_f(hops::rager_table(c.i("n"))),
            "rager_gravity_adjustment" => c.check_f(hops::rager_gravity_adjustment(c.f("sg"))),
            "rager_utilization" => c.check_f(hops::rager_utilization(c.f("sg"), c.f("t"))),
            "garetz_ibu" => c.check_f(hops::garetz_ibu(
                c.f("w"),
                c.f("a"),
                c.f("t"),
                c.f("sg"),
                c.f("v"),
                c.f("bv"),
            )),
            "garetz_utilization" => c.check_f(hops::garetz_utilization(c.f("sg"), c.f("t"))),
            "post_boil_utilization" => c.check_f(hops::post_boil_utilization(
                c.f("sg"),
                c.f("t0"),
                c.f("dur"),
            )),
            "boil_temp_c" => c.check_f(hops::boil_temp_c(c.f("alt_ft"))),
            "combine_utilizations" => {
                let segs: Vec<f64> = c
                    .v("segs")
                    .as_array()
                    .expect("segs is an array")
                    .iter()
                    .map(|x| x.as_f64().expect("seg is a number"))
                    .collect();
                c.check_f(hops::combine_utilizations(&segs));
            }
            "aged_alpha" => c.check_f(hops::aged_alpha(
                c.f("alpha"),
                c.f("hsi"),
                c.f("temp_f"),
                c.f("days"),
                c.f("storage"),
            )),
            _ => return false,
        }
        true
    });
}
