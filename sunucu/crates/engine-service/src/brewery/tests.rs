//! Service tests; expected numbers come from `formuller/testler/vektorler/08.json` and
//! `09.json` (case indices in the comments are positions in `vakalar`).
#![allow(
    clippy::unwrap_used,
    clippy::too_many_lines,
    clippy::clone_on_copy,
    clippy::inconsistent_digit_grouping
)]

use rust_decimal::Decimal;
use tonic::Code;
use tonic_types::StatusExt;

use super::*;
use crate::proto::{
    BlendComponent, CostElectric, CostEnergy, CostGas, CostIngredient, CostWater, DiastaticGrain,
    HemocytometerCount, StainCount, StockLot, TaxBand, TaxInputRounding, TaxMethod, TaxProductLine,
    TaxRateRow, TaxRoundMode, TaxRoundingScope, TaxRowStatus, TaxTrigger as ProtoTrigger,
    TaxVolumeEvents, VolumeEvent as ProtoEvent, VolumeEventKind as ProtoKind,
    tax_input_rounding::Kind, tax_product_line::Volume,
};

const H: BreweryHandler = BreweryHandler;

fn field_of(s: &Status) -> String {
    s.get_error_details()
        .bad_request()
        .unwrap()
        .field_violations[0]
        .field
        .clone()
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

/// Decimals compare by value; the scale is checked separately where the vector fixes it.
fn same(got: &str, expected: &str) {
    assert_eq!(
        got.parse::<Decimal>().unwrap(),
        expected.parse::<Decimal>().unwrap(),
        "got {got}, expected {expected}"
    );
}

fn lot(name: &str, remaining: &str, price: &str) -> StockLot {
    StockLot {
        lot: name.into(),
        remaining: remaining.into(),
        unit_price: price.into(),
    }
}

fn cost_req() -> BatchCostRequest {
    BatchCostRequest {
        ingredients: vec![CostIngredient {
            name: "pale malt".into(),
            lots: vec![lot("L-A", "3.0", "1.10"), lot("L-B", "25.0", "1.25")],
            quantity: "4.5".into(),
        }],
        pack_units: "60".into(),
        pack_unit_cost: "0.35".into(),
        packaged_l: "20.0".into(),
        ..Default::default()
    }
}

// ------------------------------------------------------------------------------- BatchCost

#[tokio::test]
async fn batch_cost_fifo_matches_vector_5() {
    // 08.json #5: two lots, rows 3.30 and 1.88 (1.875 half up), cost 5.18, missing 0.0.
    let r = H
        .batch_cost(Request::new(cost_req()))
        .await
        .unwrap()
        .into_inner();
    let ing = &r.ingredients[0];
    assert_eq!(ing.rows.len(), 2);
    assert_eq!(ing.rows[0].lot, "L-A");
    assert_eq!(ing.rows[0].taken, "3.0");
    assert_eq!(ing.rows[0].cost, "3.30");
    assert_eq!(ing.rows[1].taken, "1.5");
    assert_eq!(ing.rows[1].cost, "1.88");
    assert_eq!(ing.cost, "5.18");
    assert_eq!(ing.missing, "0.0");
    assert_eq!(r.ingredients_cost, "5.18");
    assert_eq!(r.energy_kwh, None);
    assert_eq!(r.gas_m3, None);
}

#[tokio::test]
async fn batch_cost_reports_missing_stock() {
    let mut req = cost_req();
    req.ingredients[0].quantity = "30.0".into();
    let r = H.batch_cost(Request::new(req)).await.unwrap().into_inner();
    same(&r.ingredients[0].missing, "2.0");
}

#[tokio::test]
async fn batch_cost_electric_matches_vector_38() {
    // 08.json #38: 18.40 + 1.05 + 12.97 + 21.00 = 53.42, 2.671 per litre. The ingredient cost
    // is a single 10.0 x 1.84 draw, energy is 4.99 kWh x 2.6 = 12.974 (#27, #32).
    let req = BatchCostRequest {
        ingredients: vec![CostIngredient {
            name: "malt".into(),
            lots: vec![lot("A", "10.0", "1.84")],
            quantity: "10.0".into(),
        }],
        water: Some(CostWater {
            water_l: "35.0".into(),
            price_per_m3: "30.0".into(),
        }),
        energy: Some(CostEnergy {
            heat_kj: 0.0,
            efficiency: 0.9,
            source: Some(Source::Electric(CostElectric {
                price_per_kwh: "2.6".into(),
                additional_kwh: "4.99".into(),
            })),
        }),
        ..cost_req()
    };
    let r = H.batch_cost(Request::new(req)).await.unwrap().into_inner();
    assert_eq!(r.ingredients_cost, "18.40");
    assert_eq!(r.water_cost, "1.05");
    assert_eq!(r.energy_cost, "12.97");
    assert_eq!(r.packaging_cost, "21.00");
    assert_eq!(r.total_cost, "53.42");
    assert_eq!(r.cost_per_l, "2.671");
    close(r.energy_kwh.unwrap(), 4.99);
}

#[tokio::test]
async fn batch_cost_electric_heat_matches_vector_31() {
    // 08.json #31: 6716.022 kJ at 0.9 is 2.072846296296296 kWh.
    let mut req = cost_req();
    req.energy = Some(CostEnergy {
        heat_kj: 6716.022,
        efficiency: 0.9,
        source: Some(Source::Electric(CostElectric {
            price_per_kwh: "1.0".into(),
            additional_kwh: String::new(),
        })),
    });
    let r = H.batch_cost(Request::new(req)).await.unwrap().into_inner();
    close(r.energy_kwh.unwrap(), 2.072_846_296_296_296);
    assert_eq!(r.energy_cost, "2.07");
}

#[tokio::test]
async fn batch_cost_gas_matches_vector_35() {
    // 08.json #35: 6716.022 kJ, eff 0.4, 34500 kJ/m3 -> 0.4866682608695652 m3.
    let mut req = cost_req();
    req.energy = Some(CostEnergy {
        heat_kj: 6716.022,
        efficiency: 0.4,
        source: Some(Source::Gas(CostGas {
            price_per_m3: "2".into(),
            heating_value_kj_m3: Some(34_500.0),
        })),
    });
    let r = H
        .batch_cost(Request::new(req.clone()))
        .await
        .unwrap()
        .into_inner();
    close(r.gas_m3.unwrap(), 0.486_668_260_869_565_2);
    assert_eq!(r.energy_kwh, None);
    assert_eq!(r.energy_cost, "0.97");
    // Default heating value is 34 500.
    if let Some(CostEnergy {
        source: Some(Source::Gas(g)),
        ..
    }) = req.energy.as_mut()
    {
        g.heating_value_kj_m3 = None;
    }
    let d = H.batch_cost(Request::new(req)).await.unwrap().into_inner();
    close(d.gas_m3.unwrap(), 0.486_668_260_869_565_2);
}

#[tokio::test]
async fn batch_cost_rejects_bad_input() {
    let mut cases: Vec<(&str, BatchCostRequest)> = Vec::new();
    let mut r = cost_req();
    r.ingredients[0].lots[0].remaining = "1e3".into();
    cases.push(("ingredients[0].lots[0].remaining", r));
    let mut r = cost_req();
    r.ingredients[0].lots[1].unit_price = "-1.25".into();
    cases.push(("ingredients[0].lots[1].unit_price", r));
    let mut r = cost_req();
    r.ingredients[0].quantity = "abc".into();
    cases.push(("ingredients[0].quantity", r));
    let mut r = cost_req();
    r.pack_units = String::new();
    cases.push(("pack_units", r));
    let mut r = cost_req();
    r.packaged_l = "0".into();
    cases.push(("packaged_l", r)); // engine error
    let mut r = cost_req();
    r.energy = Some(CostEnergy {
        heat_kj: f64::NAN,
        efficiency: 0.9,
        source: Some(Source::Gas(CostGas::default())),
    });
    cases.push(("energy.heat_kj", r));
    let mut r = cost_req();
    r.energy = Some(CostEnergy {
        heat_kj: 100.0,
        efficiency: 0.0,
        source: Some(Source::Gas(CostGas::default())),
    });
    cases.push(("energy.efficiency", r));
    let mut r = cost_req();
    r.energy = Some(CostEnergy {
        heat_kj: 100.0,
        efficiency: 0.9,
        source: None,
    });
    cases.push(("energy.source", r));
    let mut r = cost_req();
    r.water = Some(CostWater {
        water_l: "10".into(),
        price_per_m3: "9999999999999999".into(),
    });
    cases.push(("water.price_per_m3", r));
    for (field, req) in cases {
        let e = H.batch_cost(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{field}");
        assert_eq!(field_of(&e), field);
    }
}

// ----------------------------------------------------------------------------- LandedPrice

#[tokio::test]
async fn landed_price_matches_vectors() {
    // 08.json #21 (1.36), #23 (54.6).
    for (price, qty, ship, want) in [
        ("1.20", "25", "4.0", "1.36"),
        ("52.0", "0.5", "1.3", "54.6"),
    ] {
        let r = H
            .landed_price(Request::new(LandedPriceRequest {
                invoice_unit_price: price.into(),
                quantity: qty.into(),
                shipping_share: ship.into(),
            }))
            .await
            .unwrap()
            .into_inner();
        same(&r.unit_price, want);
    }
}

#[tokio::test]
async fn landed_price_rejects() {
    let base = LandedPriceRequest {
        invoice_unit_price: "1.20".into(),
        quantity: "25".into(),
        shipping_share: "4.0".into(),
    };
    for (field, req) in [
        (
            "qty",
            LandedPriceRequest {
                quantity: "0".into(),
                ..base.clone()
            },
        ),
        (
            "invoice_unit_price",
            LandedPriceRequest {
                invoice_unit_price: "-1".into(),
                ..base.clone()
            },
        ),
        (
            "shipping_share",
            LandedPriceRequest {
                shipping_share: "1.2.3".into(),
                ..base.clone()
            },
        ),
        (
            "invoice_unit_price",
            LandedPriceRequest {
                invoice_unit_price: " 1".into(),
                ..base
            },
        ),
    ] {
        let e = H.landed_price(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{field}");
        assert_eq!(field_of(&e), field);
    }
}

// ------------------------------------------------------------------------------- Blend

fn comp(v: f64, abv: f64, ibu: f64, srm: f64, co2: f64, p: f64) -> BlendComponent {
    BlendComponent {
        source: "tank".into(),
        volume_l: v,
        abv_pct: abv,
        ibu,
        color_srm: srm,
        co2_vol: co2,
        extract_p: p,
    }
}

#[tokio::test]
async fn blend_matches_vectors_51_and_52() {
    let r = H
        .blend(Request::new(BlendRequest {
            components: vec![
                comp(960.0, 5.4, 38.0, 7.0, 1.0, 2.8),
                comp(400.0, 4.6, 22.0, 4.0, 1.0, 2.4),
            ],
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.volume_l, 1360.0);
    close(r.abv_pct, 5.164_705_882_352_941_5);
    close(r.ibu, 33.294_117_647_058_826);
    close(r.color_srm, 6.117_647_058_823_529);
    close(r.co2_vol, 1.0);
    close(r.extract_p, 2.682_482_338_578_13);

    let r = H
        .blend(Request::new(BlendRequest {
            components: vec![
                comp(960.0, 5.4, 38.0, 7.0, 1.0, 2.8),
                comp(100.5, 0.0, 0.0, 0.0, 0.0, 0.0),
            ],
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.volume_l, 1060.5);
    close(r.abv_pct, 4.888_260_254_596_888);
    close(r.co2_vol, 0.905_233_380_480_905_2);
    close(r.extract_p, 2.537_253_483_257_601_7);
}

#[tokio::test]
async fn blend_rejects() {
    let e = H
        .blend(Request::new(BlendRequest { components: vec![] }))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument); // engine error: no volume
    let e = H
        .blend(Request::new(BlendRequest {
            components: vec![comp(f64::NAN, 5.0, 1.0, 1.0, 1.0, 1.0)],
        }))
        .await
        .unwrap_err();
    assert_eq!(field_of(&e), "components[0].volume_l");
    let e = H
        .blend(Request::new(BlendRequest {
            components: vec![comp(10.0, 5.0, -1.0, 1.0, 1.0, 1.0)],
        }))
        .await
        .unwrap_err();
    assert_eq!(field_of(&e), "components[0].ibu");
}

// ------------------------------------------------------------------------- BlendToTarget

#[tokio::test]
async fn blend_to_target_matches_vectors() {
    // 08.json #56 (320) and #57 (76.80000000000007).
    for (v1, x1, x2, xt, want) in [
        (960.0, 5.4, 4.6, 5.2, 320.0),
        (960.0, 5.4, 0.0, 5.0, 76.800_000_000_000_07),
    ] {
        let r = H
            .blend_to_target(Request::new(BlendToTargetRequest {
                first_volume_l: v1,
                first_value: x1,
                second_value: x2,
                target_value: xt,
            }))
            .await
            .unwrap()
            .into_inner();
        close(r.second_volume_l, want);
    }
}

#[tokio::test]
async fn blend_to_target_rejects() {
    let req = |v1, x1, x2, xt| {
        Request::new(BlendToTargetRequest {
            first_volume_l: v1,
            first_value: x1,
            second_value: x2,
            target_value: xt,
        })
    };
    // Target outside the two values: Unreachable -> FAILED_PRECONDITION (ADR-0005).
    let e = H
        .blend_to_target(req(960.0, 5.4, 4.6, 6.0))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition);
    let e = H
        .blend_to_target(req(960.0, 5.4, 4.6, 5.4))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition);
    let e = H
        .blend_to_target(req(960.0, 5.4, f64::NAN, 5.0))
        .await
        .unwrap_err();
    assert_eq!(field_of(&e), "second_value");
    let e = H
        .blend_to_target(req(-1.0, 5.4, 4.6, 5.0))
        .await
        .unwrap_err();
    assert_eq!(field_of(&e), "first_volume_l");
}

// ------------------------------------------------------------------------------ YeastReuse

fn yeast_req() -> YeastReuseRequest {
    YeastReuseRequest {
        pitch_rate_m_ml_p: 0.75,
        wort_volume_l: 1000.0,
        wort_extract_p: 12.0,
        generation: 1.0,
        harvest_age_days: 3.0,
        ..Default::default()
    }
}

#[tokio::test]
async fn yeast_reuse_count_and_stain_match_vectors() {
    // 08.json #70 (9000 billion), #73 (1.15e9 cells/mL), #77 (0.9217391304347826), #81 (slurry).
    let r = H
        .yeast_reuse(Request::new(YeastReuseRequest {
            count: Some(HemocytometerCount {
                total_counted: 230.0,
                squares_counted: 5.0,
                dilution: 100.0,
            }),
            stain: Some(StainCount {
                unstained: 212.0,
                stained: 18.0,
            }),
            ..yeast_req()
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.cells_needed_billion, 9000.0);
    close(r.density_cells_ml, 1.15e9);
    close(r.counted_density_cells_ml.unwrap(), 1.15e9);
    close(r.viability_pct.unwrap(), 92.173_913_043_478_26);
    close(r.slurry_volume_ml, 8490.566_037_735_849);
    assert!(r.warnings.is_empty());
}

#[tokio::test]
async fn yeast_reuse_defaults_match_vector_82() {
    // 08.json #82: no density, viability 0.9, default density 1e9 -> 10000 mL.
    let r = H
        .yeast_reuse(Request::new(YeastReuseRequest {
            viability_pct: Some(90.0),
            ..yeast_req()
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.slurry_volume_ml, 10_000.0);
    close(r.density_cells_ml, 1.0e9);
    assert_eq!(r.counted_density_cells_ml, None);
    close(r.viability_pct.unwrap(), 90.0);

    // Not measured: 100 % assumed and no viability warning (#82 with viability 1.0 is 9000 mL
    // at 1e9: 9000 billion / 1e9).
    let r = H
        .yeast_reuse(Request::new(yeast_req()))
        .await
        .unwrap()
        .into_inner();
    close(r.slurry_volume_ml, 9000.0 * 1e9 / 1e9);
    assert_eq!(r.viability_pct, None);

    // Entered density 0 falls back to the default (#83).
    let r = H
        .yeast_reuse(Request::new(YeastReuseRequest {
            measured_density_cells_ml: Some(0.0),
            viability_pct: Some(90.0),
            ..yeast_req()
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.slurry_volume_ml, 10_000.0);
}

#[tokio::test]
async fn yeast_reuse_warnings_match_vector_115() {
    // 08.json #115: generation 9, 15 days, viability 0.79, limit 8 -> three warnings.
    let r = H
        .yeast_reuse(Request::new(YeastReuseRequest {
            generation: 9.0,
            harvest_age_days: 15.0,
            viability_pct: Some(79.0),
            max_generation: Some(8.0),
            history_differs: true,
            ..yeast_req()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        r.warnings,
        vec![
            YeastReuseWarning::GenerationLimit as i32,
            YeastReuseWarning::HarvestOver14Days as i32,
            YeastReuseWarning::LowViability as i32,
            YeastReuseWarning::HistoryInfo as i32,
        ]
    );
    // #114: at the limits there is no warning (default limit 8 applies when unset).
    let r = H
        .yeast_reuse(Request::new(YeastReuseRequest {
            generation: 8.0,
            harvest_age_days: 14.0,
            viability_pct: Some(80.0),
            ..yeast_req()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(r.warnings.is_empty());
}

#[tokio::test]
async fn yeast_reuse_rejects() {
    let nan = f64::NAN;
    let cases = [
        (
            "pitch_rate_m_ml_p",
            YeastReuseRequest {
                pitch_rate_m_ml_p: nan,
                ..yeast_req()
            },
        ),
        (
            "wort_volume_l",
            YeastReuseRequest {
                wort_volume_l: -1.0,
                ..yeast_req()
            },
        ),
        (
            "viability_pct",
            YeastReuseRequest {
                viability_pct: Some(101.0),
                ..yeast_req()
            },
        ),
        (
            "generation",
            YeastReuseRequest {
                generation: nan,
                ..yeast_req()
            },
        ),
        (
            // Engine error: no squares counted.
            "squares_counted",
            YeastReuseRequest {
                count: Some(HemocytometerCount {
                    total_counted: 10.0,
                    squares_counted: 0.0,
                    dilution: 1.0,
                }),
                ..yeast_req()
            },
        ),
        (
            // Engine error: nothing counted at all.
            "unstained + stained",
            YeastReuseRequest {
                stain: Some(StainCount::default()),
                ..yeast_req()
            },
        ),
        (
            // Engine error: zero viability makes the slurry volume infinite.
            "density * viability",
            YeastReuseRequest {
                viability_pct: Some(0.0),
                ..yeast_req()
            },
        ),
    ];
    for (field, req) in cases {
        let e = H.yeast_reuse(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{field}");
        assert_eq!(field_of(&e), field);
    }
}

// ----------------------------------------------------------------------- BrewhouseEfficiency

fn lab() -> MaltLabValues {
    MaltLabValues {
        fine_grind_dry_pct: 81.5,
        moisture_pct: 4.3,
        fine_coarse_diff_pct: 0.0,
    }
}

#[tokio::test]
async fn brewhouse_efficiency_matches_vectors() {
    // 08.json #87 (extract 125.60371897570585), #91 (yield 73.88454057394462), #94 (77.9955),
    // #98 (first wort max 20.09272280967025); efficiency is the #101 ratio.
    let r = H
        .brewhouse_efficiency(Request::new(BrewhouseEfficiencyRequest {
            wort_volume_l: 1040.0,
            wort_extract_p: 12.0,
            measured_hot: true,
            malt_kg: 170.0,
            lab: Some(lab()),
            mash_water_l: Some(520.0),
            first_wort_p: Some(19.6),
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.extract_kg, 125.603_718_975_705_85);
    close(r.brewhouse_yield_pct, 73.884_540_573_944_62);
    close(r.lab_extract_as_is_pct.unwrap(), 77.995_499_999_999_99);
    close(r.first_wort_max_p.unwrap(), 20.092_722_809_670_25);
    close(
        r.conversion_efficiency.unwrap(),
        19.6 / 20.092_722_809_670_25,
    );

    // #88, #92: measured cold, no lab values.
    let r = H
        .brewhouse_efficiency(Request::new(BrewhouseEfficiencyRequest {
            wort_volume_l: 1040.0,
            wort_extract_p: 12.0,
            measured_hot: false,
            malt_kg: 170.0,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.extract_kg, 130.837_207_266_360_25);
    close(r.brewhouse_yield_pct, 76.963_063_097_858_85);
    assert_eq!(r.lab_extract_as_is_pct, None);
    assert_eq!(r.first_wort_max_p, None);
    assert_eq!(r.conversion_efficiency, None);
}

#[tokio::test]
async fn brewhouse_efficiency_rejects() {
    let base = BrewhouseEfficiencyRequest {
        wort_volume_l: 20.0,
        wort_extract_p: 14.0,
        measured_hot: true,
        malt_kg: 4.0,
        ..Default::default()
    };
    let cases = [
        (
            "malt_kg", // engine error: yield divides by the malt weight
            BrewhouseEfficiencyRequest {
                malt_kg: 0.0,
                ..base.clone()
            },
        ),
        (
            "wort_volume_l",
            BrewhouseEfficiencyRequest {
                wort_volume_l: f64::NAN,
                ..base.clone()
            },
        ),
        (
            "wort_extract_p",
            BrewhouseEfficiencyRequest {
                wort_extract_p: -1.0,
                ..base.clone()
            },
        ),
        (
            "lab.moisture_pct",
            BrewhouseEfficiencyRequest {
                lab: Some(MaltLabValues {
                    moisture_pct: f64::INFINITY,
                    ..lab()
                }),
                ..base.clone()
            },
        ),
        (
            "lab",
            BrewhouseEfficiencyRequest {
                first_wort_p: Some(18.0),
                ..base.clone()
            },
        ),
        (
            "mash_water_l",
            BrewhouseEfficiencyRequest {
                lab: Some(lab()),
                first_wort_p: Some(18.0),
                ..base.clone()
            },
        ),
        (
            "extract + water", // engine error: nothing in the mash
            BrewhouseEfficiencyRequest {
                lab: Some(MaltLabValues {
                    fine_grind_dry_pct: 0.0,
                    moisture_pct: 0.0,
                    fine_coarse_diff_pct: 0.0,
                }),
                mash_water_l: Some(0.0),
                malt_kg: 0.0,
                ..base.clone()
            },
        ),
    ];
    for (field, req) in cases {
        let e = H.brewhouse_efficiency(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{field}");
    }
    let e = H
        .brewhouse_efficiency(Request::new(BrewhouseEfficiencyRequest {
            malt_kg: 0.0,
            ..base
        }))
        .await
        .unwrap_err();
    assert_eq!(field_of(&e), "malt_kg");
}

// ------------------------------------------------------------------------- MaltAnalysis

#[tokio::test]
async fn malt_analysis_matches_vectors_104_and_109() {
    let r = H
        .malt_analysis(Request::new(MaltAnalysisRequest {
            lab: Some(MaltLabValues {
                fine_grind_dry_pct: 81.5,
                moisture_pct: 4.3,
                fine_coarse_diff_pct: 1.5,
            }),
            color_ebc: Some(3.5),
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.fine_grind_as_is_pct, 77.995_499_999_999_99);
    close(r.coarse_grind_dry_pct, 80.0);
    close(r.yield_pct, 76.56);
    close(r.potential_sg, 1.035_378_376);
    close(r.potential_sg_beersmith, 1.035_217_6);
    close(r.ppg, 35.378_376);
    close(r.color_srm.unwrap(), 1.776_649_746_192_893_4);
    close(r.color_lovibond.unwrap(), 1.872_619_036_020_148_7);

    let r = H
        .malt_analysis(Request::new(MaltAnalysisRequest {
            lab: Some(lab()),
            color_ebc: None,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.color_srm, None);
    assert_eq!(r.color_lovibond, None);
}

#[tokio::test]
async fn malt_analysis_rejects() {
    let e = H
        .malt_analysis(Request::new(MaltAnalysisRequest::default()))
        .await
        .unwrap_err();
    assert_eq!(field_of(&e), "lab");
    for (field, req) in [
        (
            "lab.fine_grind_dry_pct",
            MaltAnalysisRequest {
                lab: Some(MaltLabValues {
                    fine_grind_dry_pct: f64::NAN,
                    ..lab()
                }),
                color_ebc: None,
            },
        ),
        (
            "lab.fine_coarse_diff_pct",
            MaltAnalysisRequest {
                lab: Some(MaltLabValues {
                    fine_coarse_diff_pct: -0.5,
                    ..lab()
                }),
                color_ebc: None,
            },
        ),
        (
            "color_ebc",
            MaltAnalysisRequest {
                lab: Some(lab()),
                color_ebc: Some(f64::NAN),
            },
        ),
    ] {
        let e = H.malt_analysis(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument);
        assert_eq!(field_of(&e), field);
    }
}

// ----------------------------------------------------------------------- DiastaticCheck

fn grain(kg: f64, l: f64) -> DiastaticGrain {
    DiastaticGrain {
        amount_kg: kg,
        diastatic_power_lintner: l,
    }
}

#[tokio::test]
async fn diastatic_check_matches_vectors() {
    // 08.json #132 (31.5, warning) and #133 (40.0, none); default threshold 35.
    let r = H
        .diastatic_check(Request::new(DiastaticCheckRequest {
            grains: vec![grain(4.5, 35.0), grain(0.5, 0.0)],
            threshold_lintner: Some(35.0),
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.average_lintner, 31.5);
    assert!(r.low_diastatic_power);
    let r = H
        .diastatic_check(Request::new(DiastaticCheckRequest {
            grains: vec![grain(5.0, 40.0)],
            threshold_lintner: None,
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.average_lintner, 40.0);
    assert!(!r.low_diastatic_power);
    // The default threshold (35) flags #132 as well.
    let r = H
        .diastatic_check(Request::new(DiastaticCheckRequest {
            grains: vec![grain(4.5, 35.0), grain(0.5, 0.0)],
            threshold_lintner: None,
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(r.low_diastatic_power);
}

#[tokio::test]
async fn diastatic_check_rejects() {
    for (field, req) in [
        (
            "grains[1].amount_kg",
            DiastaticCheckRequest {
                grains: vec![grain(1.0, 10.0), grain(f64::NAN, 10.0)],
                threshold_lintner: None,
            },
        ),
        (
            "grains[0].diastatic_power_lintner",
            DiastaticCheckRequest {
                grains: vec![grain(1.0, -10.0)],
                threshold_lintner: None,
            },
        ),
        (
            "threshold_lintner",
            DiastaticCheckRequest {
                grains: vec![],
                threshold_lintner: Some(f64::INFINITY),
            },
        ),
    ] {
        let e = H.diastatic_check(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument);
        assert_eq!(field_of(&e), field);
    }
}

// ------------------------------------------------------------------------- LabelEnergy

#[tokio::test]
async fn label_energy_matches_vectors() {
    // 09.json #21: estimated ABV, default protein 0.40.
    let r = H
        .label_energy(Request::new(LabelEnergyRequest {
            og_sg: 1.048,
            fg_sg: 1.009,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.abv_pct, 5.14);
    assert!(!r.abv_measured);
    close(r.alcohol_g, 4.09);
    close(r.real_extract_g, 4.08);
    close(r.carbohydrate_g, 3.7);
    close(r.protein_g, 0.4);
    assert_eq!((r.energy_kj, r.energy_kcal), (188, 45));
    assert_eq!(r.declaration, LabelDeclaration::EnergyOptional as i32);

    // #26: measured ABV 4.7.
    let r = H
        .label_energy(Request::new(LabelEnergyRequest {
            og_sg: 1.048,
            fg_sg: 1.009,
            protein_g_100ml: Some(0.4),
            measured_abv_pct: Some(4.7),
        }))
        .await
        .unwrap()
        .into_inner();
    close(r.abv_pct, 4.7);
    assert!(r.abv_measured);
    close(r.alcohol_g, 3.71);
    assert_eq!((r.energy_kj, r.energy_kcal), (177, 42));

    // #29: 0.5 % ABV -> the country template decides.
    let r = H
        .label_energy(Request::new(LabelEnergyRequest {
            og_sg: 1.01,
            fg_sg: 1.005,
            protein_g_100ml: Some(0.4),
            measured_abv_pct: Some(0.5),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!((r.energy_kj, r.energy_kcal), (37, 9));
    assert_eq!(r.declaration, LabelDeclaration::CountryTemplate as i32);
}

#[tokio::test]
async fn label_declaration_boundary_is_not_above_1_2() {
    // 09.json #226: 1.2 itself is not above the limit.
    let r = H
        .label_energy(Request::new(LabelEnergyRequest {
            og_sg: 1.01,
            fg_sg: 1.005,
            measured_abv_pct: Some(1.2),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.declaration, LabelDeclaration::CountryTemplate as i32);
    let r = H
        .label_energy(Request::new(LabelEnergyRequest {
            og_sg: 1.01,
            fg_sg: 1.005,
            measured_abv_pct: Some(1.21),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.declaration, LabelDeclaration::EnergyOptional as i32);
}

#[tokio::test]
async fn label_energy_rejects() {
    for (field, req) in [
        (
            "og_sg",
            LabelEnergyRequest {
                og_sg: f64::NAN,
                fg_sg: 1.0,
                ..Default::default()
            },
        ),
        (
            "protein_g_100ml",
            LabelEnergyRequest {
                og_sg: 1.05,
                fg_sg: 1.01,
                protein_g_100ml: Some(-0.1),
                ..Default::default()
            },
        ),
        (
            "measured_abv_pct",
            LabelEnergyRequest {
                og_sg: 1.05,
                fg_sg: 1.01,
                measured_abv_pct: Some(f64::INFINITY),
                ..Default::default()
            },
        ),
    ] {
        let e = H.label_energy(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument);
        assert_eq!(field_of(&e), field);
    }
    // Engine error: values beyond the engine's accepted range.
    let e = H
        .label_energy(Request::new(LabelEnergyRequest {
            og_sg: 1.0e9,
            fg_sg: 1.0,
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
}

// --------------------------------------------------------------------------- Spunding

fn spund_req() -> SpundingRequest {
    SpundingRequest {
        target_co2_vol: 2.5,
        temp_c: 12.0,
        fg_expected_sg: 1.01,
        ..Default::default()
    }
}

#[tokio::test]
async fn spunding_matches_vector_216() {
    // 09.json #216.
    let r = H
        .spunding(Request::new(spund_req()))
        .await
        .unwrap()
        .into_inner();
    close(r.residual_co2_vol, 1.12);
    close(r.needed_co2_g_l, 2.71);
    close(r.closing_sg, 1.0128);
    close(r.points_before_fg, 2.8);
    close(r.setting_psi, 19.5);
    close(r.setting_bar, 1.35);
    assert!(r.warnings.is_empty());
    assert!(!r.blocked);
}

#[tokio::test]
async fn spunding_warnings_match_vectors() {
    // #248: tank limit 15 psi blocks the 2.5 vol plan. The limit travels in bar.
    let r = H
        .spunding(Request::new(SpundingRequest {
            tank_max_bar: Some(convert::psi_to_bar(15.0)),
            fg_low_sg: Some(1.01),
            fg_high_sg: Some(1.011),
            wide_limit_units: Some(200),
            ..spund_req()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        r.warnings,
        vec![SpundingWarning::TankPressureExceeded as i32]
    );
    assert!(r.blocked);

    // #247: 25 psi is enough; #249: a 4-point... 1.010-1.014 interval is wide (> 200 units
    // is not met, so 1.01-1.014 raises it only with the default of 200 exceeded at 1.01201).
    let r = H
        .spunding(Request::new(SpundingRequest {
            tank_max_bar: Some(convert::psi_to_bar(25.0)),
            fg_low_sg: Some(1.01),
            fg_high_sg: Some(1.011),
            ..spund_req()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(r.warnings.is_empty());
    assert!(!r.blocked);

    // #252: interval 1.01 - 1.01201 is wide at the default limit.
    let r = H
        .spunding(Request::new(SpundingRequest {
            tank_max_bar: Some(convert::psi_to_bar(25.0)),
            fg_low_sg: Some(1.01),
            fg_high_sg: Some(1.012_01),
            ..spund_req()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.warnings, vec![SpundingWarning::EarlyClosingRisk as i32]);
    assert!(!r.blocked);

    // #253: both warnings (also #220 for the plan itself).
    let r = H
        .spunding(Request::new(SpundingRequest {
            target_co2_vol: 3.0,
            temp_c: 4.0,
            fg_expected_sg: 1.016,
            tank_max_bar: Some(convert::psi_to_bar(10.0)),
            fg_low_sg: Some(1.008),
            fg_high_sg: Some(1.016),
            wide_limit_units: Some(200),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        r.warnings,
        vec![
            SpundingWarning::TankPressureExceeded as i32,
            SpundingWarning::EarlyClosingRisk as i32
        ]
    );
    assert!(r.blocked);
    close(r.residual_co2_vol, 1.48);
    close(r.needed_co2_g_l, 2.98);
    close(r.closing_sg, 1.0191);
    close(r.points_before_fg, 3.1);
    close(r.setting_psi, 17.2);
    close(r.setting_bar, 1.19);
}

#[tokio::test]
async fn spunding_rejects() {
    for (field, req) in [
        (
            "target_co2_vol",
            SpundingRequest {
                target_co2_vol: f64::NAN,
                ..spund_req()
            },
        ),
        (
            "temp_c",
            SpundingRequest {
                temp_c: f64::INFINITY,
                ..spund_req()
            },
        ),
        (
            "tank_max_bar",
            SpundingRequest {
                tank_max_bar: Some(-1.0),
                ..spund_req()
            },
        ),
        (
            "fg_low_sg",
            SpundingRequest {
                fg_low_sg: Some(1.01),
                ..spund_req()
            },
        ),
        (
            "wide_limit_units",
            SpundingRequest {
                wide_limit_units: Some(-1),
                ..spund_req()
            },
        ),
    ] {
        let e = H.spunding(Request::new(req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{field}");
        assert_eq!(field_of(&e), field);
    }
    // 09.json #214: expected final gravity 0 is an engine error.
    let e = H
        .spunding(Request::new(SpundingRequest {
            fg_expected_sg: 0.0,
            ..spund_req()
        }))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
}

// -------------------------------------------------------------------------------- TaxReport

fn row(method: TaxMethod) -> TaxRateRow {
    TaxRateRow {
        row_id: "row".into(),
        currency: "GBP".into(),
        method: method as i32,
        input_rounding: Some(TaxInputRounding {
            kind: Kind::NoRounding as i32,
            decimal_places: 0,
            mode: 0,
        }),
        trigger: ProtoTrigger::Packaging as i32,
        money_decimal_places: 2,
        money_mode: TaxRoundMode::HalfUp as i32,
        rounding_scope: TaxRoundingScope::Line as i32,
        ..Default::default()
    }
}

fn direct(product: &str, taxable_l: &str) -> TaxProductLine {
    TaxProductLine {
        product: product.into(),
        package: "keg".into(),
        volume: Some(Volume::TaxableL(taxable_l.into())),
        ..Default::default()
    }
}

fn tax_req(row: TaxRateRow, lines: Vec<TaxProductLine>) -> TaxReportRequest {
    TaxReportRequest {
        row: Some(row),
        period_start: "2026-04-01".into(),
        period_end: "2026-04-30".into(),
        lines,
    }
}

async fn tax(req: TaxReportRequest) -> Result<TaxReportResponse, Status> {
    H.tax_report(Request::new(req))
        .await
        .map(tonic::Response::into_inner)
}

fn band(upper: Option<&str>, rate: &str) -> TaxBand {
    TaxBand {
        upper: upper.map(Into::into),
        rate: rate.into(),
    }
}

#[tokio::test]
async fn tax_pure_alcohol_matches_vectors_31_and_32() {
    // 09.json #31: 2000 L at 5.0 % x 21.78 = "2178.0000".
    let mut r = row(TaxMethod::PureAlcohol);
    r.rate = Some("21.78".into());
    let line = TaxProductLine {
        abv_pct: Some("5.0".into()),
        abv_estimated: true,
        ..direct("pale", "2000.0")
    };
    let res = tax(tax_req(r, vec![line.clone()])).await.unwrap();
    let l = &res.lines[0];
    assert_eq!(l.tax_unrounded, "2178.0000");
    assert_eq!(l.tax, "2178.00");
    assert_eq!(l.taxable_l, "2000.0");
    assert_eq!(l.trigger_l, "0");
    assert!(l.abv_estimated);
    assert_eq!(res.total_tax, "2178.00");
    assert_eq!(res.currency, "GBP");
    assert_eq!(res.row_status, TaxRowStatus::Unverified as i32);

    // #32: ABV bands with 5.0 per-LPA relief -> "1678.0000".
    let mut r = row(TaxMethod::PureAlcohol);
    r.bands = vec![
        band(Some("3.5"), "9.27"),
        band(Some("8.5"), "21.78"),
        band(None, "29.54"),
    ];
    r.relief_per_lpa = "5.0".into();
    let res = tax(tax_req(r, vec![line])).await.unwrap();
    assert_eq!(res.lines[0].tax_unrounded, "1678.0000");
}

#[tokio::test]
async fn tax_other_methods_match_vectors() {
    // #33 hl_plato.
    let mut r = row(TaxMethod::HlPlato);
    r.rate = Some("0.787".into());
    let line = TaxProductLine {
        original_extract_p: Some("12.6".into()),
        ..direct("p", "2000.0")
    };
    let res = tax(tax_req(r, vec![line])).await.unwrap();
    assert_eq!(res.lines[0].tax_unrounded, "188.8800");

    // #40 plato band, input rounded to 1 place half up.
    let mut r = row(TaxMethod::PlatoBand);
    r.bands = vec![
        band(Some("10.0"), "16.88"),
        band(Some("14.0"), "25.32"),
        band(None, "33.76"),
    ];
    r.input_rounding = Some(TaxInputRounding {
        kind: Kind::Places as i32,
        decimal_places: 1,
        mode: TaxRoundMode::HalfUp as i32,
    });
    let line = TaxProductLine {
        original_extract_p: Some("10.05".into()),
        ..direct("p", "2000.0")
    };
    let res = tax(tax_req(r, vec![line])).await.unwrap();
    assert_eq!(res.lines[0].tax_unrounded, "506.400");

    // #36 tiered volume with hL already taxed this year.
    let mut r = row(TaxMethod::TieredVolume);
    r.tiers = vec![
        band(Some("70408.65900000"), "2.9826"),
        band(None, "13.6347"),
    ];
    let line = TaxProductLine {
        ytd_hl: "70398.659000000".into(),
        ..direct("p", "2000.0")
    };
    let res = tax(tax_req(r, vec![line])).await.unwrap();
    same(&res.lines[0].tax_unrounded, "166.173000000000");

    // #72 volume.
    let mut r = row(TaxMethod::Volume);
    r.rate = Some("12.5".into());
    r.input_rounding = Some(TaxInputRounding {
        kind: Kind::Undefined as i32,
        ..Default::default()
    });
    let res = tax(tax_req(r, vec![direct("p", "2000.0")])).await.unwrap();
    assert_eq!(res.lines[0].tax_unrounded, "250.00");
}

#[tokio::test]
async fn tax_events_match_vector_236() {
    // 09.json #236: packaging trigger, destroyed volume deducted: 2950.5 taxable of 3000.5.
    let e = |d: &str, k: ProtoKind, v: &str| ProtoEvent {
        date: d.into(),
        kind: k as i32,
        volume_l: v.into(),
    };
    let events = vec![
        e("2026-03-31", ProtoKind::Packaging, "1500"),
        e("2026-04-01", ProtoKind::Packaging, "2000.5"),
        e("2026-04-10", ProtoKind::Release, "800"),
        e("2026-04-15", ProtoKind::Sale, "700.25"),
        e("2026-04-20", ProtoKind::Destroyed, "50"),
        e("2026-04-22", ProtoKind::Returned, "30.5"),
        e("2026-04-25", ProtoKind::Lab, "5"),
        e("2026-04-30", ProtoKind::Packaging, "1000"),
        e("2026-05-01", ProtoKind::Packaging, "999"),
        e("2026-04-28", ProtoKind::Release, "100"),
    ];
    let mut r = row(TaxMethod::Volume);
    r.rate = Some("100".into());
    r.deduct = vec![ProtoKind::Destroyed as i32];
    let line = TaxProductLine {
        volume: Some(Volume::Events(TaxVolumeEvents { events })),
        ..direct("p", "")
    };
    let res = tax(tax_req(r, vec![line])).await.unwrap();
    let l = &res.lines[0];
    assert_eq!(l.taxable_l, "2950.5");
    assert_eq!(l.trigger_l, "3000.5");
    assert_eq!(l.destroyed_l, "50");
    assert_eq!(l.returned_l, "30.5");
    assert_eq!(l.lab_l, "5");
    // 29.505 hL x 100 per hL.
    same(&l.tax_unrounded, "2950.5");
}

#[tokio::test]
async fn tax_rounding_scope_and_row_status() {
    let mut r = row(TaxMethod::Volume);
    r.rate = Some("12.5".into());
    r.input_rounding = Some(TaxInputRounding {
        kind: Kind::Undefined as i32,
        ..Default::default()
    });
    r.money_decimal_places = 0;
    // Scope LINE: each line rounded half up to whole units (250.00 -> 250), 2 x 250.
    let lines = vec![direct("a", "2000.0"), direct("b", "2001.0")];
    let res = tax(tax_req(r.clone(), lines.clone())).await.unwrap();
    assert_eq!(res.lines[0].tax, "250");
    assert_eq!(res.lines[1].tax_unrounded, "250.125");
    assert_eq!(res.lines[1].tax, "250");
    assert_eq!(res.total_tax, "500");

    // Scope TOTAL: lines stay unrounded, the sum 500.125 is rounded once.
    r.rounding_scope = TaxRoundingScope::Total as i32;
    let res = tax(tax_req(r.clone(), lines)).await.unwrap();
    assert_eq!(res.lines[1].tax, "250.125");
    assert_eq!(res.total_tax, "500");

    // 09.json #192 (source only: unverified) and a row with source and date: verified.
    r.official_source = Some("gov.uk SPR".into());
    let res = tax(tax_req(r.clone(), vec![])).await.unwrap();
    assert_eq!(res.row_status, TaxRowStatus::Unverified as i32);
    r.source_date = Some("2026-10-31".into());
    let res = tax(tax_req(r, vec![])).await.unwrap();
    assert_eq!(res.row_status, TaxRowStatus::Verified as i32);
    assert_eq!(res.total_tax, "0");
}

#[tokio::test]
async fn tax_rejects_enums_and_decimals() {
    let base = || {
        let mut r = row(TaxMethod::Volume);
        r.rate = Some("12.5".into());
        r
    };
    let ok_line = || vec![direct("a", "10")];
    let mut cases: Vec<(&str, TaxReportRequest)> = Vec::new();
    cases.push((
        "row.method",
        tax_req(
            TaxRateRow {
                method: 0,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.method",
        tax_req(
            TaxRateRow {
                method: 99,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.trigger",
        tax_req(
            TaxRateRow {
                trigger: 0,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.rounding_scope",
        tax_req(
            TaxRateRow {
                rounding_scope: 0,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.money_mode",
        tax_req(
            TaxRateRow {
                money_mode: 0,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.money_decimal_places",
        tax_req(
            TaxRateRow {
                money_decimal_places: 13,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.input_rounding.kind",
        tax_req(
            TaxRateRow {
                input_rounding: None,
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.input_rounding.mode",
        tax_req(
            TaxRateRow {
                input_rounding: Some(TaxInputRounding {
                    kind: Kind::Places as i32,
                    decimal_places: 2,
                    mode: 0,
                }),
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.deduct[0]",
        tax_req(
            TaxRateRow {
                deduct: vec![0],
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.rate",
        tax_req(
            TaxRateRow {
                rate: Some("1,5".into()),
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "row.bands[1].upper",
        tax_req(
            TaxRateRow {
                bands: vec![band(Some("1"), "2"), band(Some("x"), "3")],
                ..base()
            },
            ok_line(),
        ),
    ));
    cases.push((
        "lines[0].taxable_l",
        tax_req(base(), vec![direct("a", "12e3")]),
    ));
    cases.push((
        "lines[0].taxable_l",
        tax_req(base(), vec![direct("a", "-5")]),
    ));
    cases.push((
        "lines[0].volume",
        tax_req(base(), vec![TaxProductLine::default()]),
    ));
    cases.push((
        "lines[0].ytd_hl",
        tax_req(
            base(),
            vec![TaxProductLine {
                ytd_hl: "nope".into(),
                ..direct("a", "10")
            }],
        ),
    ));
    cases.push((
        "lines[0].abv_pct",
        tax_req(
            base(),
            vec![TaxProductLine {
                abv_pct: Some(String::new()),
                ..direct("a", "10")
            }],
        ),
    ));
    cases.push((
        "lines[0].events[0].kind",
        tax_req(
            base(),
            vec![TaxProductLine {
                volume: Some(Volume::Events(TaxVolumeEvents {
                    events: vec![ProtoEvent {
                        date: "2026-04-02".into(),
                        kind: 0,
                        volume_l: "10".into(),
                    }],
                })),
                ..direct("a", "")
            }],
        ),
    ));
    cases.push((
        "lines[0].events[0].date",
        tax_req(
            base(),
            vec![TaxProductLine {
                volume: Some(Volume::Events(TaxVolumeEvents {
                    events: vec![ProtoEvent {
                        date: "02.04.2026".into(),
                        kind: ProtoKind::Packaging as i32,
                        volume_l: "10".into(),
                    }],
                })),
                ..direct("a", "")
            }],
        ),
    ));
    cases.push((
        "period_start",
        TaxReportRequest {
            period_start: "2026-05-01".into(),
            ..tax_req(base(), ok_line())
        },
    ));
    cases.push((
        "row",
        TaxReportRequest {
            row: None,
            ..tax_req(base(), ok_line())
        },
    ));
    for (field, req) in cases {
        let e = tax(req).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{field}");
        assert_eq!(field_of(&e), field);
    }
}

#[tokio::test]
async fn tax_engine_errors_map_to_status() {
    // 09.json #44: ABV method with an undefined input rounding rule.
    let mut r = row(TaxMethod::PureAlcohol);
    r.rate = Some("21.78".into());
    r.input_rounding = Some(TaxInputRounding {
        kind: Kind::Undefined as i32,
        ..Default::default()
    });
    let line = TaxProductLine {
        abv_pct: Some("5.0".into()),
        ..direct("a", "2000.0")
    };
    let e = tax(tax_req(r, vec![line])).await.unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
    assert_eq!(field_of(&e), "input_rounding");

    // #43: the last tier is not open-ended.
    let mut r = row(TaxMethod::TieredVolume);
    r.tiers = vec![band(Some("100.0"), "3.0")];
    let line = TaxProductLine {
        ytd_hl: "95.0".into(),
        ..direct("a", "2000.0")
    };
    let e = tax(tax_req(r, vec![line])).await.unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);

    // A pure-alcohol line without ABV.
    let mut r = row(TaxMethod::PureAlcohol);
    r.rate = Some("21.78".into());
    let e = tax(tax_req(r, vec![direct("a", "10")])).await.unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument);
}
