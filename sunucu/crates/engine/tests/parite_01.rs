//! Section 01 parity: `engine::density` against `vektorler/01.json`.
#![allow(clippy::expect_used, clippy::too_many_lines)] // one long fonk-to-call table
mod ortak;

use engine::density as d;
use engine::units;
use serde_json::{Value, json};

fn num(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing number `{key}` in {v}"))
}

fn flag(v: &Value, key: &str) -> bool {
    v[key]
        .as_bool()
        .unwrap_or_else(|| panic!("missing bool `{key}` in {v}"))
}

fn code(v: &Value, key: &str) -> i64 {
    v[key]
        .as_i64()
        .unwrap_or_else(|| panic!("missing integer `{key}` in {v}"))
}

fn grain(v: &Value) -> d::Fermentable {
    d::Fermentable {
        name: v["name"].as_str().expect("name").to_owned(),
        amount_oz: num(v, "amount_oz"),
        yield_pct: num(v, "yield_pct"),
        color_srm: num(v, "color_srm"),
        kind: d::GrainType::from_code(code(v, "type")).expect("grain type"),
        usage: d::GrainUse::from_code(code(v, "use")).expect("grain use"),
        not_fermentable: flag(v, "not_fermentable"),
        late_extract_min: num(v, "late_extract_min"),
        max_in_batch_pct: num(v, "max_in_batch_pct"),
    }
}

fn mash(v: &Value) -> d::Mash {
    d::Mash {
        grain_weight: num(v, "grain_weight"),
        biab: flag(v, "biab"),
        steps: v["steps"]
            .as_array()
            .expect("steps")
            .iter()
            .map(|s| d::MashStep {
                temp_f: num(s, "temp_f"),
                infusion: num(s, "infusion"),
            })
            .collect(),
    }
}

fn mash_json(m: &d::Mash) -> Value {
    json!({
        "grain_weight": m.grain_weight,
        "biab": m.biab,
        "steps": m.steps.iter()
            .map(|s| json!({"temp_f": s.temp_f, "infusion": s.infusion}))
            .collect::<Vec<_>>(),
    })
}

fn recipe(v: &Value) -> d::Recipe {
    d::Recipe {
        kind: d::RecipeType::from_code(code(v, "type")).expect("recipe type"),
        volume_floz: num(v, "volume_floz"),
        trub_loss_floz: num(v, "trub_loss_floz"),
        pre_trub_loss_floz: num(v, "pre_trub_loss_floz"),
        boil_vol_floz: num(v, "boil_vol_floz"),
        boil_time_min: num(v, "boil_time_min"),
        efficiency: num(v, "efficiency"),
        steep_efficiency: num(v, "steep_efficiency"),
        mash_fg_adjust: flag(v, "mash_fg_adjust"),
        mash_fg_adj_temp: num(v, "mash_fg_adj_temp"),
        mash_fg_adj_slope: num(v, "mash_fg_adj_slope"),
        fermentables: v["fermentables"]
            .as_array()
            .expect("fermentables")
            .iter()
            .map(grain)
            .collect(),
        yeasts: v["yeasts"]
            .as_array()
            .expect("yeasts")
            .iter()
            .map(|y| d::Yeast {
                min_attenuation: num(y, "min_attenuation"),
                max_attenuation: num(y, "max_attenuation"),
                tolerance_abv: num(y, "tolerance"),
                add_to_secondary: flag(y, "add_to_secondary"),
            })
            .collect(),
        mash: mash(&v["mash"]),
    }
}

fn amounts(r: &d::Recipe) -> Vec<f64> {
    r.fermentables.iter().map(|g| g.amount_oz).collect()
}

fn hydro_unit(c: &ortak::Case, key: &str) -> engine::Result<d::HydrometerUnit> {
    d::HydrometerUnit::from_code(c.i(key))
}

fn refr_unit(c: &ortak::Case, key: &str) -> engine::Result<d::RefractometerUnit> {
    d::RefractometerUnit::from_code(c.i(key))
}

fn color_unit(c: &ortak::Case, key: &str) -> engine::Result<d::ColorUnit> {
    d::ColorUnit::from_code(c.i(key))
}

fn temp_unit(c: &ortak::Case, key: &str) -> engine::Result<d::TemperatureUnit> {
    d::TemperatureUnit::from_code(c.i(key))
}

fn refr_kind(c: &ortak::Case) -> engine::Result<d::RefractometerType> {
    d::RefractometerType::from_code(c.i("kind"))
}

fn field(name: &str) -> d::GrainField {
    match name {
        "amount_oz" => d::GrainField::AmountOz,
        "yield_pct" => d::GrainField::YieldPct,
        "color_srm" => d::GrainField::ColorSrm,
        other => panic!("unknown field `{other}`"),
    }
}

#[test]
fn parite_01() {
    ortak::run("01", |c| {
        match c.fonk.as_str() {
            "to_plato" => c.check_f(units::to_plato(c.f("sg"))),
            "to_sg" => c.check_f(units::to_sg(c.f("plato"))),
            "sg_to_brix" => c.check_f(d::sg_to_brix(c.f("sg"))),
            "brix_to_sg" => c.check_f(d::brix_to_sg(c.f("brix"))),
            "brix_to_ri" => c.check_f(d::brix_to_ri(c.f("brix"))),
            "ri_to_brix" => c.check_r(d::ri_to_brix(c.f("ri"))),
            "real_extract" => c.check_r(d::real_extract(c.f("og"), c.f("fg"))),
            "abw" => c.check_r(d::abw(c.f("og"), c.f("fg"))),
            "abv" => c.check_r(d::abv(c.f("og"), c.f("fg"))),
            "apparent_attenuation" => c.check_r(d::apparent_attenuation(c.f("og"), c.f("fg"))),
            "real_attenuation" => c.check_r(d::real_attenuation(c.f("og"), c.f("fg"))),
            "alcohol_calories" => c.check_r(d::alcohol_calories(c.f("og"), c.f("fg"))),
            "extract_calories" => c.check_r(d::extract_calories(c.f("og"), c.f("fg"))),
            "uk_tax_abv" => c.check_r(d::uk_tax_abv(c.f("og"), c.f("fg"))),
            "morey_srm" => c.check_r(d::morey_srm(c.f("mcu_sum"), c.f("volume_gal"))),
            "hydrometer_correction" => c.check_r(d::hydrometer_correction(
                c.f("sg"),
                c.f("temp_f"),
                c.f("calib_f"),
            )),
            "refractometer_unfermented" => c.check_r(d::refractometer_unfermented(
                c.f("brix"),
                c.f("zero"),
                c.f("wcf"),
            )),
            "refractometer_fermenting" => c.check_r(d::refractometer_fermenting(
                c.f("og"),
                c.f("brix"),
                c.f("zero"),
            )),
            "abv_abw_unclamped" => {
                c.check_res(
                    d::abv_abw_unclamped(c.f("og"), c.f("fg")).map(|(abv, abw)| json!([abv, abw])),
                );
            }
            "refractometer_abv" => c.check_r((|| {
                d::refractometer_abv(refr_kind(c)?, c.f("gravity"), c.f("brix"), c.f("zero"))
            })()),
            "refractometer_abw" => c.check_r((|| {
                d::refractometer_abw(refr_kind(c)?, c.f("gravity"), c.f("brix"), c.f("zero"))
            })()),
            "refractometer_finished_abv" => c.check_r(d::refractometer_finished_abv(
                c.f("fg"),
                c.f("brix"),
                c.f("zero"),
            )),
            "recipe_est_mash_efficiency" => {
                c.check_f(d::recipe_est_mash_efficiency(&recipe(c.v("recipe"))));
            }
            "recipe_act_mash_efficiency" => c.check_f(d::recipe_act_mash_efficiency(
                &recipe(c.v("recipe")),
                c.f("og_mash_measured"),
                c.f("boil_vol_measured"),
            )),
            "recipe_actual_efficiency" => c.check_f(d::recipe_actual_efficiency(
                &recipe(c.v("recipe")),
                c.f("og_measured"),
                c.f("volume_measured"),
            )),
            "do_adjust_color" => {
                let (status, r, ok) = d::do_adjust_color(&recipe(c.v("recipe")), c.f("target"));
                let name = match status {
                    d::ColorAdjustStatus::NotApplicable => "not_applicable",
                    d::ColorAdjustStatus::OutOfRange => "out_of_range",
                    d::ColorAdjustStatus::Adjusted => "adjusted",
                };
                c.check(&json!([name, d::recipe_color(&r), ok, amounts(&r)]));
            }
            "refractometer_finished" => {
                c.check_res(
                    d::refractometer_finished(c.f("fg"), c.f("brix"), c.f("zero"))
                        .map(|r| json!([r.og, r.abw_percent, r.re, r.oe])),
                );
            }
            "fg_from_tolerance" => {
                c.check_r(d::fg_from_tolerance(c.f("og"), c.f("fg"), c.f("tol")));
            }
            "gravity_adjust_extract_oz" => c.check_r((|| {
                d::gravity_adjust_extract_oz(
                    c.f("current"),
                    c.f("target"),
                    c.f("volume_floz"),
                    d::AdjustExtract::from_code(c.i("kind"))?,
                )
            })()),
            "gravity_adjust_water_floz" => c.check_r(d::gravity_adjust_water_floz(
                c.f("current"),
                c.f("target"),
                c.f("volume_floz"),
            )),
            "convert_hydrometer" => c.check_r((|| {
                d::convert_hydrometer(c.f("x"), hydro_unit(c, "from")?, hydro_unit(c, "to")?)
            })()),
            "convert_refractometer" => c.check_r((|| {
                d::convert_refractometer(c.f("x"), refr_unit(c, "from")?, refr_unit(c, "to")?)
            })()),
            "convert_color" => c.check_r((|| {
                d::convert_color(c.f("x"), color_unit(c, "from")?, color_unit(c, "to")?)
            })()),
            "convert_temperature" => c.check_r((|| {
                d::convert_temperature(c.f("x"), temp_unit(c, "from")?, temp_unit(c, "to")?)
            })()),
            "potential_from_yield" => c.check_f(d::potential_from_yield(c.f("yield_pct"))),
            "yield_from_potential" => c.check_f(d::yield_from_potential(c.f("sg"))),
            "grain_yield_convert" => c.check_r((|| {
                d::grain_yield_convert(c.f("x"), d::HydrometerUnit::from_code(c.i("from"))?)
            })()),
            "grain_og_points" => c.check_f(d::grain_og_points(
                &grain(c.v("grain")),
                c.f("eff"),
                c.f("steep_eff"),
            )),
            "grain_fg_points" => c.check_f(d::grain_fg_points(
                &grain(c.v("grain")),
                c.f("eff"),
                c.f("atten"),
                c.f("steep_eff"),
            )),
            "og_points_phase" => {
                c.check_r((|| {
                    Ok(d::og_points_phase(
                        &recipe(c.v("recipe")),
                        d::GrainUse::from_code(c.i("phase"))?,
                    ))
                })());
            }
            "estimate_og" => c.check_r((|| {
                Ok(d::estimate_og(
                    &recipe(c.v("recipe")),
                    c.f("volume_floz"),
                    d::GrainUse::from_code(c.i("phase"))?,
                ))
            })()),
            "recipe_og" => c.check_f(d::recipe_og(&recipe(c.v("recipe")))),
            "recipe_virtual_og" => c.check_f(d::recipe_virtual_og(&recipe(c.v("recipe")))),
            "est_post_mash_og" => c.check_f(d::est_post_mash_og(&recipe(c.v("recipe")))),
            "late_extract_profile" => {
                let profile = d::late_extract_profile(&recipe(c.v("recipe")), c.f("boil_vol_floz"));
                let rows: Vec<Value> = profile.iter().map(|p| json!([p.sg, p.time_min])).collect();
                c.check(&Value::Array(rows));
            }
            "recipe_og_boil" => c.check_f(d::recipe_og_boil(
                &recipe(c.v("recipe")),
                c.f("boil_vol_floz"),
            )),
            "scale_gravity" => c.check_res(
                d::scale_gravity(&recipe(c.v("recipe")), c.f("og_target"))
                    .map(|r| json!(amounts(&r))),
            ),
            "scale_mash_from_grain_weight" => {
                let m =
                    d::scale_mash_from_grain_weight(&mash(c.v("mash")), c.f("grain_weight_new"));
                c.check(&mash_json(&m));
            }
            "total_grain_use_field" => c.check_f(d::total_grain_use_field(
                &recipe(c.v("recipe")),
                d::GrainUse::from_code(c.i("use")).expect("grain use"),
                field(c.v("field").as_str().expect("field name")),
            )),
            "efficiency" => c.check_f(d::efficiency(
                &recipe(c.v("recipe")),
                c.f("og"),
                c.f("volume_floz"),
                c.b("mash_only"),
            )),
            "yeast_attenuation" => {
                let (atten, tol) = d::yeast_attenuation(&recipe(c.v("recipe")));
                c.check(&json!([atten, tol]));
            }
            "mash_adjusted_attenuation" => c.check_f(d::mash_adjusted_attenuation(
                &recipe(c.v("recipe")),
                c.f("atten"),
            )),
            "recipe_fg" => c.check_f(d::recipe_fg(&recipe(c.v("recipe")))),
            "avg_attenuation" => c.check_f(d::avg_attenuation(c.f("min"), c.f("max"))),
            "original_extract" => c.check_f(d::original_extract(c.f("start_sg"))),
            "apparent_extract" => c.check_f(d::apparent_extract(c.f("final_sg"))),
            "calories_alcohol" => c.check_r(d::calories_alcohol(c.f("og"), c.f("fg"))),
            "calories_carbs" => c.check_r(d::calories_carbs(c.f("og"), c.f("fg"))),
            "calories_total" => c.check_r(d::calories_total(c.f("og"), c.f("fg"))),
            "carbohydrates_grams" => c.check_f(d::carbohydrates_grams(c.f("carb_kcal"))),
            "carb_calories" => c.check_f(d::carb_calories(c.f("total_kcal"), c.f("alc_kcal"))),
            "recipe_color" => c.check_f(d::recipe_color(&recipe(c.v("recipe")))),
            "mcu_from_srm" => c.check_f(d::mcu_from_srm(c.f("srm"))),
            "color_flexibility" => {
                let f = d::color_flexibility(&recipe(c.v("recipe")), c.f("target"));
                c.check(&json!([f.lo, f.hi, f.ok]));
            }
            "iterate_color" => {
                let (r, colour) = d::iterate_color(&recipe(c.v("recipe")), c.f("target"));
                c.check(&json!([colour, amounts(&r)]));
            }
            "adjust_color" => {
                let (r, ok) = d::adjust_color(&recipe(c.v("recipe")), c.f("target"));
                c.check(&json!([d::recipe_color(&r), ok, amounts(&r)]));
            }
            "color_adj_range" => {
                let (lo, hi) = d::color_adj_range(&recipe(c.v("recipe")));
                c.check(&json!([lo, hi]));
            }
            _ => return false,
        }
        true
    });
}
