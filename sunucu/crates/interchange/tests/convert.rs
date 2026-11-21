//! Tests of the conversions with the API version JSON.
#![allow(clippy::float_cmp)]

use interchange::{
    Recipe,
    model::{Fermentable, Hop},
};

#[test]
fn defaults_and_version_json() {
    let mut r = Recipe {
        name: "D".into(),
        kind: "all_grain".into(),
        fermentables: vec![Fermentable {
            name: "S".into(),
            grain_type: Some("sugar".into()),
            ..Default::default()
        }],
        hops: vec![Hop {
            name: "H".into(),
            form: Some("weird".into()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let notes = r.fill_defaults();
    assert!(notes.len() >= 4, "{notes:?}");
    assert_eq!(r.fermentables[0].yield_pct, Some(100.0));
    assert_eq!(r.hops[0].form.as_deref(), Some("pellet"));
    let v = serde_json::json!({"kind":"all_grain","batch_volume_l":20.0,"calc_summary":{"og_sg":1.05,"ibu":30.0},
        "target_og_sg":1.04,"fermentables":[{"name":"M","amount_kg":4.0,"grain_type":"grain","yield_pct":80.0}]});
    let back = Recipe::from_version_json("X", None, &v);
    assert_eq!(back.source.og_sg, Some(1.05));
    assert_eq!(back.fermentables[0].amount_kg, 4.0);
}
