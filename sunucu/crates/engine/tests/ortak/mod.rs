//! Parity vector loader shared by `tests/parite_NN.rs` (ADR-0003).
//!
//! Each vector file is produced by `formuller/testler/parite/NN_*.py` from the Python
//! reference. A test maps every case's `fonk` to an engine call and compares results.
#![allow(
    dead_code,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used
)]

use std::path::PathBuf;

use serde_json::Value;

/// Relative tolerance for floating-point results: `|a − b| ≤ 1e-9 · max(1, |b|)`.
pub const REL_TOL: f64 = 1e-9;

/// One case of a vector file.
#[derive(Debug)]
pub struct Case {
    pub index: usize,
    pub fonk: String,
    pub girdi: Value,
    pub beklenen: Value,
}

impl Case {
    /// Numeric input by name.
    pub fn f(&self, name: &str) -> f64 {
        self.girdi
            .get(name)
            .and_then(Value::as_f64)
            .unwrap_or_else(|| {
                panic!(
                    "case {} ({}): missing number input `{name}`",
                    self.index, self.fonk
                )
            })
    }

    /// Integer input by name.
    pub fn i(&self, name: &str) -> i64 {
        self.girdi
            .get(name)
            .and_then(Value::as_i64)
            .unwrap_or_else(|| {
                panic!(
                    "case {} ({}): missing integer input `{name}`",
                    self.index, self.fonk
                )
            })
    }

    /// Boolean input by name.
    pub fn b(&self, name: &str) -> bool {
        self.girdi
            .get(name)
            .and_then(Value::as_bool)
            .unwrap_or_else(|| {
                panic!(
                    "case {} ({}): missing bool input `{name}`",
                    self.index, self.fonk
                )
            })
    }

    /// Raw input value by name.
    pub fn v(&self, name: &str) -> &Value {
        self.girdi.get(name).unwrap_or_else(|| {
            panic!(
                "case {} ({}): missing input `{name}`",
                self.index, self.fonk
            )
        })
    }

    fn label(&self) -> String {
        format!("case {} `{}` girdi={}", self.index, self.fonk, self.girdi)
    }

    /// Compares a single number with `beklenen`.
    pub fn check_f(&self, actual: f64) {
        check_value(&self.label(), &serde_json::json!(actual), &self.beklenen);
    }

    /// Compares any JSON-shaped result (numbers within tolerance, everything else exact).
    pub fn check(&self, actual: &Value) {
        check_value(&self.label(), actual, &self.beklenen);
    }
}

/// Loads `formuller/testler/vektorler/<bolum>.json`.
pub fn load(bolum: &str) -> Vec<Case> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../formuller/testler/vektorler")
        .join(format!("{bolum}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let root: Value = serde_json::from_str(&text).expect("vector file is valid JSON");
    assert_eq!(
        root["bolum"],
        bolum,
        "section id mismatch in {}",
        path.display()
    );
    let cases: Vec<Case> = root["vakalar"]
        .as_array()
        .expect("`vakalar` is an array")
        .iter()
        .enumerate()
        .map(|(index, c)| Case {
            index,
            fonk: c["fonk"].as_str().expect("`fonk` is a string").to_owned(),
            girdi: c["girdi"].clone(),
            beklenen: c["beklenen"].clone(),
        })
        .collect();
    assert!(!cases.is_empty(), "no cases in {}", path.display());
    cases
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= REL_TOL * b.abs().max(1.0)
}

fn check_value(label: &str, actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(e)) => {
            let (a, e) = (a.as_f64().unwrap(), e.as_f64().unwrap());
            assert!(close(a, e), "{label}: got {a}, expected {e}");
        }
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(
                a.len(),
                e.len(),
                "{label}: length {} vs {}",
                a.len(),
                e.len()
            );
            for (x, y) in a.iter().zip(e) {
                check_value(label, x, y);
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            let mut ak: Vec<_> = a.keys().collect();
            let mut ek: Vec<_> = e.keys().collect();
            ak.sort();
            ek.sort();
            assert_eq!(ak, ek, "{label}: keys differ");
            for (k, y) in e {
                check_value(label, &a[k], y);
            }
        }
        _ => assert_eq!(actual, expected, "{label}"),
    }
}

/// Runs `f` on every case and fails if any `fonk` is unhandled (`f` returns `false`).
pub fn run(bolum: &str, mut f: impl FnMut(&Case) -> bool) {
    let cases = load(bolum);
    for c in &cases {
        assert!(
            f(c),
            "{}: unhandled fonk `{}` in section {bolum}",
            c.label(),
            c.fonk
        );
    }
}
