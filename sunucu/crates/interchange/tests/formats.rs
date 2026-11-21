//! Round-trip and safety tests with the sample files in `tests/fixtures`.
#![allow(clippy::float_cmp)]

use interchange::{Error, beerjson, beerxml, brewfather, csv, xml};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

#[test]
fn beerxml_sample_is_read() {
    let doc = beerxml::parse(&fixture("sample.beerxml")).unwrap();
    let r = &doc.recipes[0];
    assert_eq!(r.name, "Test Pale & Hoppy");
    assert_eq!(r.fermentables.len(), 2);
    assert_eq!(r.fermentables[1].usage.as_deref(), Some("boil"));
    assert_eq!(r.hops[1].usage.as_deref(), Some("dry_hop"));
    assert_eq!(r.source.og_sg, Some(1.052));
    assert_eq!(r.equipment.as_ref().and_then(|e| e.trub_loss_l), Some(1.5));
    // 10 percent of 25 L per hour.
    assert_eq!(
        r.equipment.as_ref().and_then(|e| e.boil_off_l_per_h),
        Some(2.5)
    );
}

#[test]
fn beerxml_round_trip() {
    let a = beerxml::parse(&fixture("sample.beerxml")).unwrap();
    let text = beerxml::write(&a.recipes);
    let b = beerxml::parse(text.as_bytes()).unwrap();
    assert_eq!(a.recipes, b.recipes);
}

#[test]
fn beerjson_round_trip_from_beerxml() {
    let a = beerxml::parse(&fixture("sample.beerxml")).unwrap();
    let text = beerjson::write(&a.recipes);
    let b = beerjson::parse(text.as_bytes()).unwrap();
    let (x, y) = (&a.recipes[0], &b.recipes[0]);
    assert_eq!(x.name, y.name);
    assert_eq!(x.source, y.source);
    assert_eq!(x.fermentables, y.fermentables);
    assert_eq!(x.hops, y.hops);
    assert_eq!(x.miscs, y.miscs);
    assert_eq!(x.mash_steps, y.mash_steps);
    assert_eq!(x.yeasts[0].attenuation_pct, y.yeasts[0].attenuation_pct);
    assert_eq!(x.batch_volume_l, y.batch_volume_l);
    assert_eq!(x.efficiency_pct, y.efficiency_pct);
    // Writing again gives the same text.
    assert_eq!(text, beerjson::write(&b.recipes));
}

#[test]
fn beerjson_converts_units() {
    let json = br#"{"beerjson":{"version":1.0,"recipes":[{"name":"U","type":"all grain",
      "batch_size":{"value":5,"unit":"gal"},
      "ingredients":{"fermentable_additions":[{"name":"M","type":"grain","amount":{"value":10,"unit":"lb"},"timing":{"use":"add_to_mash"}}],
      "hop_additions":[{"name":"H","amount":{"value":1,"unit":"oz"},"timing":{"use":"add_to_boil","duration":{"value":1,"unit":"hr"}}}]}}]}}"#;
    let r = &beerjson::parse(json).unwrap().recipes[0];
    assert!((r.batch_volume_l.unwrap() - 18.927).abs() < 0.01);
    assert!((r.fermentables[0].amount_kg - 4.5359).abs() < 0.001);
    assert!((r.hops[0].amount_kg - 0.02835).abs() < 0.0001);
    assert_eq!(r.hops[0].time_min, Some(60.0));
}

#[test]
fn brewfather_sample_is_read() {
    let doc = brewfather::parse(&fixture("brewfather.json")).unwrap();
    let r = &doc.recipes[0];
    assert_eq!(r.name, "Test Brewfather IPA");
    // Hops are grams in the file.
    assert!((r.hops[0].amount_kg - 0.030).abs() < 1e-9);
    // Potential 1.037 is 80 percent yield.
    assert!((r.fermentables[0].yield_pct.unwrap() - 80.07).abs() < 0.1);
    assert_eq!(r.fermentables[1].usage.as_deref(), Some("boil"));
    assert_eq!(r.miscs[0].amount, 0.004);
    assert_eq!(r.source.ibu, Some(55.0));
    assert_eq!(r.equipment.as_ref().unwrap().name, "Test Rig");
    assert_eq!(r.mash_steps[0].temp_c, 65.0);
}

#[test]
fn xml_bomb_is_rejected() {
    assert_eq!(
        beerxml::parse(&fixture("xml_bomb.xml")).unwrap_err(),
        Error::DoctypeNotAllowed
    );
}

#[test]
fn external_entity_is_rejected() {
    assert_eq!(
        beerxml::parse(&fixture("xml_external_entity.xml")).unwrap_err(),
        Error::DoctypeNotAllowed
    );
    assert_eq!(
        beerxml::parse(&fixture("xml_undeclared_entity.xml")).unwrap_err(),
        Error::EntityNotAllowed
    );
}

#[test]
fn size_depth_and_count_limits() {
    let big = vec![b' '; 9 * 1024 * 1024];
    assert_eq!(beerxml::parse(&big).unwrap_err(), Error::TooLarge);
    let deep = format!("{}{}", "<a>".repeat(100), "</a>".repeat(100));
    assert_eq!(
        beerxml::parse(deep.as_bytes()).unwrap_err(),
        Error::TooComplex
    );
    let limits = xml::Limits {
        max_elements: 10,
        ..xml::Limits::default()
    };
    let many = format!("<r>{}</r>", "<e/>".repeat(50));
    assert_eq!(
        xml::parse(many.as_bytes(), limits).unwrap_err(),
        Error::TooComplex
    );
}

#[test]
fn numeric_character_references_work() {
    let r = xml::parse(b"<r><n>a&#65;&#x42;&lt;</n></r>", xml::Limits::default()).unwrap();
    assert_eq!(r.text_of("n"), Some("aAB<"));
}

#[test]
fn csv_cells_are_escaped() {
    assert_eq!(csv::cell("=1+1"), "'=1+1");
    assert_eq!(csv::cell("@SUM(A1)"), "'@SUM(A1)");
    assert_eq!(csv::cell("-2"), "'-2");
    assert_eq!(csv::cell("a,b"), "\"a,b\"");
    assert_eq!(csv::cell("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(csv::cell("=\"x\",y"), "\"'=\"\"x\"\",y\"");
    assert_eq!(csv::cell("plain"), "plain");
    assert_eq!(csv::text_line(&["a", "=b"]), "a,'=b\r\n");
}

#[test]
fn version_input_matches_the_api_shape() {
    let doc = beerxml::parse(&fixture("sample.beerxml")).unwrap();
    let v = doc.recipes[0].version_input(None);
    assert_eq!(v["kind"], "all_grain");
    assert_eq!(v["target_og_sg"], 1.052);
    assert_eq!(v["fermentables"][0]["grain_type"], "grain");
    assert!(v["fermentables"][0].get("yield_pct").is_some());
    assert_eq!(v["mash_snapshot"]["steps"][0]["kind"], "infusion");
}
