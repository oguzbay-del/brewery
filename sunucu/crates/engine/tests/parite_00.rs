//! Harness self-test: shared unit conversions against `vektorler/00.json`.
mod ortak;

use engine::units;

#[test]
fn parite_00() {
    ortak::run("00", |c| {
        match c.fonk.as_str() {
            "to_plato" => c.check_f(units::to_plato(c.f("sg"))),
            "to_sg" => c.check_f(units::to_sg(c.f("plato"))),
            "gravity_units" => c.check(&serde_json::json!(units::gravity_units(c.f("sg")))),
            _ => return false,
        }
        true
    });
}
