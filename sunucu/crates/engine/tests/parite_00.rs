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

/// Every vector file must have a `tests/parite_NN.rs` (T1.12 P5).
#[test]
fn every_vector_file_has_a_test() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("../../../formuller/testler/vektorler");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if let Some(id) = name.strip_suffix(".json") {
            let test = root.join(format!("tests/parite_{id}.rs"));
            assert!(test.exists(), "{name} has no {}", test.display());
        }
    }
}
