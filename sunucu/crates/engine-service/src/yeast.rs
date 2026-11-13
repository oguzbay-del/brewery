//! `YeastService` implementation: pitching and starters, mead nutrients, wine stabilizers,
//! backsweetening and carbonation over `engine::yeast`. The wire is metric; the engine works in
//! BeerSmith units (US gal, fl oz, oz, F, psi), so every conversion goes through
//! [`crate::convert`] (ADR-0005 item 3).

use engine::density::brix_to_sg;
use engine::recipe::{MAX_LIST_LEN, RecipeType, Yeast as RecipeYeast, YeastForm};
use engine::yeast::{
    self, AleType, BackAgent, CarbAmount, CarbAmountInput, CarbType, CellsNeededInput,
    MeadFermInput, MeadMethod, PrimingAgent, StarterType, ToolViabilityInput, YanMethod,
    YeastNitrogen, YeastTotalCellsInput,
};
use tonic::{Request, Response, Status};

use crate::check::{finite, non_negative, out};
use crate::convert;
use crate::proto::{
    BacksweetenRequest, BacksweetenResponse, CarbonationRequest, CarbonationResponse,
    MeadNutrientsRequest, MeadNutrientsResponse, PitchOptions, PitchPlanRequest, PitchPlanResponse,
    PitchYeast, PitchYeastResult, StarterOptions, StarterPlan, WineStabilizersRequest,
    WineStabilizersResponse,
    pitch_plan_request::Target,
    yeast_service_server::{YeastService, YeastServiceServer},
};
use crate::recipe::engine_code;
use crate::status::{bad_request, from_engine};

/// Handler implementing the yeast gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct YeastHandler;

/// Largest accepted epoch day magnitude (about 273 000 years), so day differences cannot overflow.
const MAX_EPOCH_DAY: i64 = 100_000_000;

/// Requires a value in 0..=100.
fn percent(field: &str, v: f64) -> Result<f64, Status> {
    non_negative(field, v)?;
    if v > 100.0 {
        return Err(bad_request(
            &format!("{field} must not exceed 100"),
            field,
            "must not exceed 100",
        ));
    }
    Ok(v)
}

/// Optional non-negative value with a default.
fn opt_non_negative(field: &str, v: Option<f64>, default: f64) -> Result<f64, Status> {
    v.map_or(Ok(default), |x| non_negative(field, x))
}

/// Optional percentage with a default.
fn opt_percent(field: &str, v: Option<f64>, default: f64) -> Result<f64, Status> {
    v.map_or(Ok(default), |x| percent(field, x))
}

/// Requires a day number within [`MAX_EPOCH_DAY`].
fn epoch_day(field: &str, v: i64) -> Result<i64, Status> {
    if v.unsigned_abs() > MAX_EPOCH_DAY.unsigned_abs() {
        return Err(bad_request(
            &format!("{field} is out of range"),
            field,
            "is out of range",
        ));
    }
    Ok(v)
}

/// Converts litres to the engine's US gallons after checking the volume.
fn gallons(field: &str, litres: f64) -> Result<f64, Status> {
    Ok(convert::l_to_gal(non_negative(field, litres)?))
}

// ---------------------------------------------------------------------------
// PitchPlan
// ---------------------------------------------------------------------------

/// Options of the pitch screen with the engine defaults applied.
struct Settings {
    targets: yeast::PitchTargets,
    use_viable: bool,
    dry_cells_per_gram: f64,
    hydrate_ml_per_g: f64,
    goferm_g_per_ml: f64,
}

fn settings(o: Option<&PitchOptions>) -> Result<Settings, Status> {
    let d = yeast::PitchTargets::default();
    let t = o.and_then(|o| o.targets.as_ref());
    let f = |name: &str, v: Option<f64>, default: f64| {
        opt_non_negative(&format!("options.targets.{name}"), v, default)
    };
    let targets = yeast::PitchTargets {
        ale_m_ml_p: f("ale_m_ml_p", t.and_then(|t| t.ale_m_ml_p), d.ale_m_ml_p)?,
        lager_m_ml_p: f(
            "lager_m_ml_p",
            t.and_then(|t| t.lager_m_ml_p),
            d.lager_m_ml_p,
        )?,
        mix_m_ml_p: f("mix_m_ml_p", t.and_then(|t| t.mix_m_ml_p), d.mix_m_ml_p)?,
        wine_m_ml_p: f("wine_m_ml_p", t.and_then(|t| t.wine_m_ml_p), d.wine_m_ml_p)?,
    };
    Ok(Settings {
        targets,
        use_viable: o.and_then(|o| o.use_viable_cells).unwrap_or(true),
        dry_cells_per_gram: opt_non_negative(
            "options.dry_cells_per_gram_billion",
            o.and_then(|o| o.dry_cells_per_gram_billion),
            yeast::O_Y_DRY_CELLS_PER_GRAM,
        )?,
        hydrate_ml_per_g: opt_non_negative(
            "options.hydrate_ml_per_g",
            o.and_then(|o| o.hydrate_ml_per_g),
            yeast::O_Y_DRY_HYDRATE,
        )?,
        goferm_g_per_ml: opt_non_negative(
            "options.goferm_g_per_ml",
            o.and_then(|o| o.goferm_g_per_ml),
            yeast::O_Y_GOFERM,
        )?,
    })
}

/// Viability of one yeast: the typed value as is, otherwise the aged `V0`.
#[allow(clippy::cast_precision_loss)]
fn yeast_viability(
    f: &dyn Fn(&str) -> String,
    y: &PitchYeast,
    form: YeastForm,
    days: i64,
) -> Result<f64, Status> {
    if let Some(v) = y.viability_pct {
        return Ok(yeast::tool_viability(&ToolViabilityInput {
            calc: false,
            user_viability_pct: percent(&f("viability_pct"), v)?,
            v0_pct: 0.0,
            rate_pct_per_day: 0.0,
            days: 0.0,
        }));
    }
    let v0 = opt_percent(
        &f("initial_viability_pct"),
        y.initial_viability_pct,
        yeast::viability_v0_ingredient(form),
    )?;
    let rate = opt_percent(
        &f("age_rate_pct_per_month"),
        y.age_rate_pct_per_month,
        yeast::import_age_rate(form, y.white_labs),
    )?;
    Ok(yeast::viability(v0, rate, days as f64, 5.0))
}

/// One yeast of the request: the engine record for the totals and the per-yeast result.
fn map_yeast(
    i: usize,
    y: &PitchYeast,
    in_recipe: bool,
) -> Result<(RecipeYeast, PitchYeastResult), Status> {
    let f = |n: &str| format!("yeasts[{i}].{n}");
    let form = engine_code(&f("form"), y.form, YeastForm::from_code)?;
    let packs = non_negative(&f("packs"), y.packs)?;
    let cells_pack = non_negative(&f("cells_per_pack_billion"), y.cells_per_pack_billion)?;
    let amount = non_negative(&f("amount"), y.amount)?;
    let cells_ml = opt_non_negative(&f("cells_per_ml_billion"), y.cells_per_ml_billion, 1.0)?;
    let dry_rate = opt_non_negative(
        &f("dry_pitch_rate_g_per_hl"),
        y.dry_pitch_rate_g_per_hl,
        75.0,
    )?;
    let reference = epoch_day(&f("reference_epoch_day"), y.reference_epoch_day)?;
    let package = y
        .package_epoch_day
        .map(|d| epoch_day(&f("package_epoch_day"), d))
        .transpose()?
        .unwrap_or(reference);
    let days = yeast::age_days(package, in_recipe, reference, reference);
    let viability = yeast_viability(&f, y, form, days)?;
    let total = yeast::yeast_total_cells(&YeastTotalCellsInput {
        use_pkgs: y.use_packages,
        cells_pkg_billion: cells_pack,
        new_amount: packs,
        form,
        // The engine reads this field as g for dry yeast and mL otherwise (proto comment).
        amount_oz: amount,
        cells_ml_billion: cells_ml,
    });
    let viable = yeast::viable_cells(total, viability);
    let record = RecipeYeast {
        min_attenuation_pct: 0.0,
        max_attenuation_pct: 0.0,
        tolerance_abv_pct: 0.0,
        add_to_secondary: y.add_to_secondary,
        form,
        total_cells_billion: total,
        viable_cells_billion: viable,
        packs,
        viability_pct: viability,
        use_packages: y.use_packages,
        // `yeast_tot_grams` multiplies by the ounce weight: dry in oz, wet forms in fl oz.
        amount_oz: if form == YeastForm::Dry {
            convert::g_to_oz(amount)
        } else {
            convert::ml_to_floz(amount)
        },
        cells_per_ml_billion: cells_ml,
        dry_pitch_rate_g_per_hl: dry_rate,
    };
    let result = PitchYeastResult {
        age_days: days,
        viability_pct: out(&f("viability_pct"), viability)?,
        total_cells_billion: out(&f("total_cells_billion"), total)?,
        viable_cells_billion: out(&f("viable_cells_billion"), viable)?,
    };
    Ok((record, result))
}

/// Numbers every part of the starter plan starts from.
struct Basis {
    /// Cells needed, billions.
    need: f64,
    /// Cells available, billions.
    avail: f64,
    /// Wort volume, mL.
    wort_ml: f64,
    /// Average pack size, billions.
    pack_size: f64,
}

/// Cells after a starter stage of `litres`, or 0 when the stage is not planned.
fn stage_cells(planned: bool, from: f64, litres: f64, stir: bool) -> Result<f64, Status> {
    if !planned {
        return Ok(0.0);
    }
    yeast::cells_starter(from, litres, stir).map_err(|e| from_engine(&e))
}

/// Starter plan: recommended volumes, DME, cells after each stage and packs needed.
fn starter_plan(s: &StarterOptions, in_recipe: bool, b: &Basis) -> Result<StarterPlan, Status> {
    let stir = s.stir_plate;
    let sg = opt_non_negative("starter.starter_sg", s.starter_sg, yeast::YC_STARTER_OG)?;
    let cap = opt_non_negative(
        "starter.growth_cap",
        s.growth_cap,
        yeast::O_Y_GROWTH_STARTER,
    )?;
    let rec_l = yeast::new_starter_l(b.need, b.avail, stir).map_err(|e| from_engine(&e))?;
    let rec2 = if in_recipe {
        yeast::two_stage_recipe(b.need, b.avail, stir, cap)
    } else {
        yeast::two_stage_tool(b.need, b.avail, stir)
    }
    .map_err(|e| from_engine(&e))?;
    let (default1, default2) = if s.two_stage {
        (rec2.stage1_l, rec2.stage2_l)
    } else {
        (rec_l, 0.0)
    };
    let stage1 = opt_non_negative("starter.stage1_l", s.stage1_l, default1)?;
    let stage2 = if s.two_stage {
        opt_non_negative("starter.stage2_l", s.stage2_l, default2)?
    } else {
        0.0
    };
    let dme_g = |litres: f64| convert::oz_to_g(yeast::dme_oz(convert::l_to_floz(litres), sg));
    let cells1 = stage_cells(true, b.avail, stage1, stir)?;
    let cells2 = if s.two_stage {
        yeast::cells_sec_starter(cells1, stage2, stir).map_err(|e| from_engine(&e))?
    } else {
        0.0
    };
    let after = if s.two_stage { cells2 } else { cells1 };
    let packs =
        yeast::packs_starter(b.need, b.pack_size, cap, stir).map_err(|e| from_engine(&e))?;
    Ok(StarterPlan {
        recommended_l: out("starter.recommended_l", rec_l)?,
        recommended_stage1_l: out("starter.recommended_stage1_l", rec2.stage1_l)?,
        recommended_stage2_l: out("starter.recommended_stage2_l", rec2.stage2_l)?,
        stage1_l: out("starter.stage1_l", stage1)?,
        stage2_l: out("starter.stage2_l", stage2)?,
        stage1_dme_g: out("starter.stage1_dme_g", dme_g(stage1))?,
        stage2_dme_g: out(
            "starter.stage2_dme_g",
            if s.two_stage { dme_g(stage2) } else { 0.0 },
        )?,
        stage1_cells_billion: out("starter.stage1_cells_billion", cells1)?,
        stage2_cells_billion: out(
            "starter.stage2_cells_billion",
            stage_cells(false, 0.0, 0.0, stir)? + cells2,
        )?,
        pitched_m_per_ml: out(
            "starter.pitched_m_per_ml",
            yeast::est_cells_ml(after, b.wort_ml),
        )?,
        packs_needed: out("starter.packs_needed", packs)?,
    })
}

/// The pitch and dry rates of the request context with their ranges.
struct Rates {
    pitch: f64,
    pitch_min: f64,
    pitch_max: f64,
    dry: f64,
    rec_dry: f64,
}

fn rates(
    r: &PitchPlanRequest,
    target: &Target,
    first_dry_rate: f64,
    targets: &yeast::PitchTargets,
) -> Result<Rates, Status> {
    let (default_pitch, range) = match target {
        Target::Recipe(t) => {
            let kind = engine_code("target.recipe.kind", t.kind, RecipeType::from_code)?.code();
            // The fermentation type only matters outside the wine-like kinds.
            let ale = if yeast::is_wine_like_recipe(kind) && t.ale_type == 0 {
                0
            } else {
                engine_code("target.recipe.ale_type", t.ale_type, AleType::from_code)?.code()
            };
            let rec = yeast::recommended_liquid_pitch(kind, ale, targets);
            (rec, yeast::pitch_min_max_recipe(rec))
        }
        Target::StarterType(raw) => {
            let kind = engine_code("starter_type", *raw, StarterType::from_code)?;
            (
                yeast::YC_PITCH_RATE,
                yeast::pitch_min_max_tool(kind, targets),
            )
        }
    };
    let default_dry = match target {
        Target::Recipe(_) => first_dry_rate,
        Target::StarterType(_) => yeast::REC_DRY_RATE,
    };
    let rec_dry = opt_non_negative(
        "rec_dry_rate_g_per_hl",
        r.rec_dry_rate_g_per_hl,
        default_dry,
    )?;
    Ok(Rates {
        pitch: opt_non_negative("pitch_rate_m_ml_p", r.pitch_rate_m_ml_p, default_pitch)?,
        pitch_min: range.min_m_ml_p,
        pitch_max: range.max_m_ml_p,
        dry: opt_non_negative("dry_rate_g_per_hl", r.dry_rate_g_per_hl, rec_dry)?,
        rec_dry,
    })
}

#[allow(clippy::too_many_lines)]
fn pitch_plan(r: &PitchPlanRequest) -> Result<PitchPlanResponse, Status> {
    let volume_l = non_negative("volume_l", r.volume_l)?;
    let og = non_negative("og_sg", r.og_sg)?;
    let target = r
        .target
        .as_ref()
        .ok_or_else(|| bad_request("target must be set", "target", "must be set"))?;
    let in_recipe = matches!(target, Target::Recipe(_));
    if r.yeasts.len() > MAX_LIST_LEN || (!in_recipe && r.yeasts.len() != 1) {
        return Err(bad_request(
            "yeasts has the wrong number of items",
            "yeasts",
            "at most 128 for a recipe, exactly one for the starter tool",
        ));
    }
    let set = settings(r.options.as_ref())?;
    let mapped = r
        .yeasts
        .iter()
        .enumerate()
        .map(|(i, y)| map_yeast(i, y, in_recipe))
        .collect::<Result<Vec<_>, _>>()?;
    let list: Vec<RecipeYeast> = mapped.iter().map(|(y, _)| *y).collect();
    let ages: Vec<i64> = mapped
        .iter()
        .zip(&list)
        .filter(|(_, y)| !y.add_to_secondary)
        .map(|((_, res), _)| res.age_days)
        .collect();

    let first = yeast::yeast_first_fields(&list);
    let rate = rates(r, target, first.dry_pitch_rate_g_per_hl, &set.targets)?;
    let avail = yeast::yeast_cells_no_starter(set.use_viable, &list);
    let pack_count = yeast::yeast_pack_count(&list);
    let pack_size = yeast::yeast_avg_pack_size(avail, pack_count, true);
    let by_weight = yeast::is_dry_by_weight(first.form, first.use_packages, !in_recipe);
    let need = yeast::tot_yeast_cells_needed(&CellsNeededInput {
        dry_by_weight: by_weight,
        pitch_m_ml_p: rate.pitch,
        v_l: volume_l,
        sg: og,
        dry_rate_g_per_hl: rate.dry,
        rec_dry_rate_g_per_hl: rate.rec_dry,
    });
    let wort_ml = volume_l * 1000.0;
    let dry_needed = if first.form != YeastForm::Dry {
        0.0
    } else if by_weight {
        yeast::dry_grams_vol(rate.dry, volume_l)
    } else {
        yeast::dry_grams_pkg(need, set.dry_cells_per_gram)
    };
    let dry_in_recipe = yeast::yeast_tot_grams(&list);
    let starter = r
        .starter
        .as_ref()
        .map(|s| {
            starter_plan(
                s,
                in_recipe,
                &Basis {
                    need,
                    avail,
                    wort_ml,
                    pack_size,
                },
            )
        })
        .transpose()?;
    let packs_needed = yeast::packs_no_starter(need, pack_size).map_err(|e| from_engine(&e))?;
    Ok(PitchPlanResponse {
        yeasts: mapped.into_iter().map(|(_, res)| res).collect(),
        avg_age_months: out("avg_age_months", yeast::yeast_age_recipe(&ages))?,
        cells_available_billion: out("cells_available_billion", avail)?,
        pack_count: out("pack_count", pack_count)?,
        avg_pack_size_billion: out("avg_pack_size_billion", pack_size)?,
        pack_viability_pct: out("pack_viability_pct", yeast::yeast_pack_viability(&list))?,
        pitch_rate_m_ml_p: out("pitch_rate_m_ml_p", rate.pitch)?,
        pitch_min_m_ml_p: out("pitch_min_m_ml_p", rate.pitch_min)?,
        pitch_max_m_ml_p: out("pitch_max_m_ml_p", rate.pitch_max)?,
        dry_rate_g_per_hl: out("dry_rate_g_per_hl", rate.dry)?,
        dry_min_g_per_hl: out("dry_min_g_per_hl", yeast::dry_min_pitch_rate(rate.rec_dry))?,
        dry_max_g_per_hl: out("dry_max_g_per_hl", yeast::dry_max_pitch_rate(rate.rec_dry))?,
        dry_by_weight: by_weight,
        cells_needed_billion: out("cells_needed_billion", need)?,
        pitched_m_per_ml: out("pitched_m_per_ml", yeast::est_cells_ml(avail, wort_ml))?,
        packs_needed,
        dry_needed_g: out("dry_needed_g", dry_needed)?,
        dry_packs_11_5g: out("dry_packs_11_5g", yeast::yeast_packs_dry(dry_needed))?,
        dry_packs_5g: out("dry_packs_5g", yeast::yeast_dry5(dry_needed))?,
        dry_in_recipe_g: out("dry_in_recipe_g", dry_in_recipe)?,
        rehydrate_water_ml: out(
            "rehydrate_water_ml",
            yeast::yeast_dry_hydrate(dry_in_recipe, set.hydrate_ml_per_g),
        )?,
        goferm_g: out(
            "goferm_g",
            yeast::yeast_goferm(dry_in_recipe, set.hydrate_ml_per_g, set.goferm_g_per_ml),
        )?,
        starter,
    })
}

// ---------------------------------------------------------------------------
// MeadNutrients
// ---------------------------------------------------------------------------

fn mead_nutrients(r: &MeadNutrientsRequest) -> Result<MeadNutrientsResponse, Status> {
    let sg = non_negative("og_sg", r.og_sg)?;
    let volume_l = non_negative("volume_l", r.volume_l)?;
    let honey_pct = opt_non_negative("honey_pct", r.honey_pct, yeast::MEAD_HONEY_PCT)?;
    let fac = engine_code("nitrogen", r.nitrogen, YeastNitrogen::from_code)?;
    let method = engine_code("method", r.method, MeadMethod::from_code)?;
    let recommended = yeast::mead_yeast_grams(sg, volume_l);
    let yeast_g = opt_non_negative("yeast_g", r.yeast_g, recommended)?;
    let target = yeast::mead_yan_target(sg, honey_pct, fac).map_err(|e| from_engine(&e))?;
    let input = MeadFermInput {
        sg,
        v_gal: convert::l_to_gal(volume_l),
        yan_pct: honey_pct,
        fac,
        method,
    };
    let (total, per_addition, yan_method) = match method {
        MeadMethod::Tosna => (
            yeast::mead_tot_ferm_o(&input),
            yeast::mead_ferm_o(&input),
            YanMethod::FermaidO,
        ),
        MeadMethod::Tiosna => (
            yeast::mead_tot_ferm_k(&input),
            yeast::mead_ferm_k(&input),
            YanMethod::FermaidK,
        ),
    };
    let total = total.map_err(|e| from_engine(&e))?;
    let per_addition = per_addition.map_err(|e| from_engine(&e))?;
    Ok(MeadNutrientsResponse {
        pitch_rate_g_per_l: out("pitch_rate_g_per_l", yeast::mead_pitch(sg))?,
        recommended_yeast_g: out("recommended_yeast_g", recommended)?,
        goferm_g: out("goferm_g", yeast::mead_goferm(yeast_g))?,
        hydrate_water_ml: out("hydrate_water_ml", yeast::mead_hydrate(yeast_g))?,
        yan_target_ppm: out("yan_target_ppm", target)?,
        nutrient_total_g: out("nutrient_total_g", total)?,
        nutrient_per_addition_g: out("nutrient_per_addition_g", per_addition)?,
        yan_added_ppm: out(
            "yan_added_ppm",
            yeast::mead_yan_added(yan_method, total, volume_l, false),
        )?,
        yan_equivalent_ppm: out(
            "yan_equivalent_ppm",
            yeast::mead_yan_equivalent(yan_method, total, volume_l, false),
        )?,
    })
}

// ---------------------------------------------------------------------------
// WineStabilizers
// ---------------------------------------------------------------------------

fn wine_stabilizers(r: &WineStabilizersRequest) -> Result<WineStabilizersResponse, Status> {
    let ph = non_negative("ph", r.ph)?;
    let v_gal = gallons("volume_l", r.volume_l)?;
    let free = opt_non_negative("free_so2_ppm", r.free_so2_ppm, yeast::WINE_FREE_SO2)?;
    let target = opt_non_negative("target_so2_ppm", r.target_so2_ppm, yeast::WINE_TARGET_SO2)?;
    Ok(WineStabilizersResponse {
        recommended_so2_ppm: out("recommended_so2_ppm", yeast::so2_rec(ph))?,
        kmeta_g: out("kmeta_g", yeast::kmeta_g(v_gal, target, free))?,
        sorbate_g: out("sorbate_g", yeast::sorbate_g(v_gal))?,
    })
}

// ---------------------------------------------------------------------------
// Backsweeten
// ---------------------------------------------------------------------------

fn backsweeten(r: &BacksweetenRequest) -> Result<BacksweetenResponse, Status> {
    let volume_l = non_negative("volume_l", r.volume_l)?;
    let bev = non_negative("beverage_sg", r.beverage_sg)?;
    let target = non_negative("target_sg", r.target_sg)?;
    let agent = engine_code("agent", r.agent, BackAgent::from_code)?;
    let (brix, agent_l, agent_kg) = match r.agent_brix {
        None => (
            yeast::back_agent_og(agent),
            yeast::back_agent_vol(volume_l, bev, target, agent),
            yeast::back_agent_weight(volume_l, bev, target, agent),
        ),
        Some(b) => {
            let brix = non_negative("agent_brix", b)?;
            let l = yeast::backsweeten_volume_l(volume_l, bev, target, yeast::honey_sg(brix));
            (brix, l, brix_to_sg(brix) * l)
        }
    };
    Ok(BacksweetenResponse {
        agent_brix: out("agent_brix", brix)?,
        agent_l: out("agent_l", agent_l)?,
        agent_kg: out("agent_kg", agent_kg)?,
    })
}

// ---------------------------------------------------------------------------
// Carbonation
// ---------------------------------------------------------------------------

fn carbonation(r: &CarbonationRequest) -> Result<CarbonationResponse, Status> {
    let c_type = engine_code("type", r.r#type, CarbType::from_code)?;
    let vols = non_negative("co2_volumes", r.co2_volumes)?;
    let temp_f = convert::c_to_f(finite("temp_c", r.temp_c)?);
    let gal = gallons("volume_l", r.volume_l)?;
    let rate = opt_non_negative("carb_rate_pct", r.carb_rate_pct, 0.0)?;
    // A keg without priming needs no agent; every other type does.
    let agent = if c_type == CarbType::Keg && r.agent == 0 {
        PrimingAgent::CornSugar
    } else {
        engine_code("agent", r.agent, PrimingAgent::from_code)?
    };
    let priming = match yeast::carb_amount(&CarbAmountInput {
        c_type,
        co2_vol: vols,
        temp_f,
        volume_gal: gal,
        carb_rate_pct: rate,
        agent,
    }) {
        CarbAmount::SugarG(g) => g,
        CarbAmount::PressurePsi(_) => 0.0,
    };
    let by = |a| yeast::carb_sugar_agent(a, vols, temp_f, gal);
    Ok(CarbonationResponse {
        residual_co2_volumes: out("residual_co2_volumes", yeast::residual_co2(temp_f))?,
        priming_g: out("priming_g", priming)?,
        corn_sugar_g: out("corn_sugar_g", by(PrimingAgent::CornSugar))?,
        dme_g: out("dme_g", by(PrimingAgent::Dme))?,
        table_sugar_g: out("table_sugar_g", by(PrimingAgent::TableSugar))?,
        keg_pressure_bar: out(
            "keg_pressure_bar",
            convert::psi_to_bar(yeast::co2_psi(vols, temp_f)),
        )?,
        sugar_equiv: out("sugar_equiv", yeast::carb_equiv(rate))?,
    })
}

#[tonic::async_trait]
impl YeastService for YeastHandler {
    async fn pitch_plan(
        &self,
        request: Request<PitchPlanRequest>,
    ) -> Result<Response<PitchPlanResponse>, Status> {
        pitch_plan(request.get_ref()).map(Response::new)
    }

    async fn mead_nutrients(
        &self,
        request: Request<MeadNutrientsRequest>,
    ) -> Result<Response<MeadNutrientsResponse>, Status> {
        mead_nutrients(request.get_ref()).map(Response::new)
    }

    async fn wine_stabilizers(
        &self,
        request: Request<WineStabilizersRequest>,
    ) -> Result<Response<WineStabilizersResponse>, Status> {
        wine_stabilizers(request.get_ref()).map(Response::new)
    }

    async fn backsweeten(
        &self,
        request: Request<BacksweetenRequest>,
    ) -> Result<Response<BacksweetenResponse>, Status> {
        backsweeten(request.get_ref()).map(Response::new)
    }

    async fn carbonation(
        &self,
        request: Request<CarbonationRequest>,
    ) -> Result<Response<CarbonationResponse>, Status> {
        carbonation(request.get_ref()).map(Response::new)
    }
}

/// Builds the tonic server wrapper for the yeast handler.
#[must_use]
pub fn server() -> YeastServiceServer<YeastHandler> {
    YeastServiceServer::new(YeastHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{
        self, BacksweetenAgent, CarbonationType, PitchTargets as ProtoTargets, PrimingAgent as PA,
        RecipeKind, RecipePitchTarget,
    };
    use tonic::Code;

    fn field_of(s: &Status) -> String {
        use tonic_types::StatusExt;
        s.get_error_details()
            .bad_request()
            .unwrap()
            .field_violations[0]
            .field
            .clone()
    }

    /// Relative tolerance of 1e-9 against a parity vector value.
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-9 * b.abs() + 1e-12, "{a} != {b}");
    }

    const REF_DAY: i64 = 100;

    fn form(f: proto::YeastForm) -> i32 {
        f as i32
    }

    /// Liquid yeast in package mode with a typed viability.
    fn liquid_pack(packs: f64, cells_per_pack: f64, viability: f64) -> PitchYeast {
        PitchYeast {
            form: form(proto::YeastForm::Liquid),
            use_packages: true,
            packs,
            cells_per_pack_billion: cells_per_pack,
            viability_pct: Some(viability),
            reference_epoch_day: REF_DAY,
            ..Default::default()
        }
    }

    fn dry_by_weight(grams: f64) -> PitchYeast {
        PitchYeast {
            form: form(proto::YeastForm::Dry),
            amount: grams,
            reference_epoch_day: REF_DAY,
            ..Default::default()
        }
    }

    fn recipe_target() -> Target {
        Target::Recipe(RecipePitchTarget {
            kind: RecipeKind::AllGrain as i32,
            ale_type: proto::AleType::Ale as i32,
        })
    }

    fn plan_req(yeasts: Vec<PitchYeast>) -> PitchPlanRequest {
        PitchPlanRequest {
            volume_l: 20.0,
            og_sg: 1.06,
            target: Some(recipe_target()),
            yeasts,
            ..Default::default()
        }
    }

    async fn plan(r: PitchPlanRequest) -> PitchPlanResponse {
        YeastHandler
            .pitch_plan(Request::new(r))
            .await
            .unwrap()
            .into_inner()
    }

    async fn plan_err(r: PitchPlanRequest) -> Status {
        YeastHandler.pitch_plan(Request::new(r)).await.unwrap_err()
    }

    /// Pitch rate that makes the cells needed `cells` for 20 L of 1.06 SG wort.
    fn rate_for(cells: f64) -> f64 {
        cells / (20.0 * engine::units::to_plato(1.06))
    }

    #[tokio::test]
    async fn pitch_recipe_totals_match_vectors() {
        // Vector tot_grams (4 yeasts) 36.1747625, pack_count 3, pack_viability 82.667,
        // yeast_age_recipe [30, 60, 90] 6.0, recommended pitch 0.75 and its range, dry range 75.
        let mut y1 = liquid_pack(1.0, 100.0, 70.0);
        y1.package_epoch_day = Some(REF_DAY - 30);
        let mut y2 = PitchYeast {
            form: form(proto::YeastForm::Dry),
            ..liquid_pack(2.0, 75.0, 93.0)
        };
        y2.package_epoch_day = Some(REF_DAY - 60);
        let y3 = PitchYeast {
            add_to_secondary: true,
            use_packages: false,
            packs: 1.0,
            amount: convert::oz_to_g(1.0),
            viability_pct: Some(80.0),
            package_epoch_day: Some(0),
            ..y2
        };
        let y4 = PitchYeast {
            use_packages: false,
            packs: 0.0,
            amount: convert::oz_to_g(0.5),
            viability_pct: Some(85.0),
            package_epoch_day: Some(REF_DAY - 90),
            ..y2
        };
        let r = plan(plan_req(vec![y1, y2, y3, y4])).await;
        close(r.dry_in_recipe_g, 36.174_762_5);
        close(r.pack_count, 3.0);
        close(r.pack_viability_pct, 82.666_666_666_666_67);
        close(r.avg_age_months, 6.0);
        assert_eq!(r.yeasts.len(), 4);
        assert_eq!(r.yeasts[0].age_days, 30);
        close(r.yeasts[0].viable_cells_billion, 70.0);
        close(r.yeasts[1].total_cells_billion, 150.0);
        close(r.cells_needed_billion, 221.120_756_279_999);
        close(r.pitch_rate_m_ml_p, 0.75);
        close(r.pitch_min_m_ml_p, 0.35);
        close(r.pitch_max_m_ml_p, 1.15);
        close(r.dry_min_g_per_hl, 18.0);
        close(r.dry_max_g_per_hl, 150.0);
        assert!(!r.dry_by_weight);
        close(r.rehydrate_water_ml, 36.174_762_5 * 25.0);
        close(r.goferm_g, 36.174_762_5 * 25.0 * 0.05);
        assert!(r.starter.is_none());
    }

    #[tokio::test]
    async fn pitch_aged_liquid_matches_vectors() {
        // Vectors: age_days 90, viability(97, 10, 90) = 70.713, yeast_total_cells 200,
        // packs_no_starter(221.12, 100) = 3, age 90 days = 3 months.
        let y = PitchYeast {
            viability_pct: None,
            package_epoch_day: Some(REF_DAY - 90),
            ..liquid_pack(2.0, 100.0, 0.0)
        };
        let mut req = plan_req(vec![y]);
        req.options = Some(PitchOptions {
            use_viable_cells: Some(false),
            ..Default::default()
        });
        let r = plan(req).await;
        assert_eq!(r.yeasts[0].age_days, 90);
        close(r.yeasts[0].viability_pct, 70.713_000_000_000_01);
        close(r.yeasts[0].total_cells_billion, 200.0);
        close(r.yeasts[0].viable_cells_billion, 200.0 * 70.713 / 100.0);
        close(r.cells_available_billion, 200.0);
        close(r.avg_pack_size_billion, 100.0);
        close(r.avg_age_months, 3.0);
        assert_eq!(r.packs_needed, 3);
        close(r.pitched_m_per_ml, 10.0);
    }

    #[tokio::test]
    async fn pitch_viability_defaults_follow_form_and_white_labs() {
        // Vectors: v0_ingredient liquid 97, import_age_rate liquid 10 and White Labs 5,
        // viability(97, 5, 90) = 83.165375.
        let base = PitchYeast {
            viability_pct: None,
            package_epoch_day: Some(REF_DAY - 90),
            ..liquid_pack(1.0, 100.0, 0.0)
        };
        let wl = PitchYeast {
            white_labs: true,
            ..base
        };
        let r = plan(plan_req(vec![base, wl])).await;
        close(r.yeasts[0].viability_pct, 70.713_000_000_000_01);
        close(r.yeasts[1].viability_pct, 83.165_374_999_999_98);
    }

    #[tokio::test]
    async fn pitch_starter_recommendation_matches_vectors() {
        // 100 viable cells, 225 needed: new_starter_l 2.3633, two_stage_recipe, packs_starter 1.1.
        let y = liquid_pack(1.0, 200.0, 50.0);
        for (stir, one, s1, s2) in [
            (
                false,
                2.363_279_567_648_351,
                0.976_773_724_452_597_1,
                1.465_160_586_678_895_7,
            ),
            (
                true,
                1.051_665_127_649_653_6,
                0.434_666_671_549_761_13,
                0.652_000_007_352_411_6,
            ),
        ] {
            let mut req = plan_req(vec![y]);
            req.pitch_rate_m_ml_p = Some(rate_for(225.0));
            req.starter = Some(StarterOptions {
                stir_plate: stir,
                two_stage: true,
                ..Default::default()
            });
            let r = plan(req).await;
            close(r.cells_needed_billion, 225.0);
            assert_eq!(r.packs_needed, 3);
            let s = r.starter.unwrap();
            close(s.recommended_l, one);
            close(s.recommended_stage1_l, s1);
            close(s.recommended_stage2_l, s2);
            close(s.stage1_l, s1);
            close(s.stage2_l, s2);
            close(s.packs_needed, 1.1);
            // Default stage volumes are the recommended ones, DME from dme_oz.
            close(
                s.stage1_dme_g,
                convert::oz_to_g(yeast::dme_oz(convert::l_to_floz(s1), 1.036)),
            );
        }
    }

    #[tokio::test]
    async fn pitch_starter_cells_match_vectors() {
        // cells_starter(70.71, 1.524) 152.565 (no stir) and 221.2195 (stir).
        for (stir, expected) in [
            (false, 152.565_175_603_792_68),
            (true, 221.219_504_625_492_49),
        ] {
            let mut req = plan_req(vec![liquid_pack(1.0, 100.0, 70.71)]);
            req.starter = Some(StarterOptions {
                stir_plate: stir,
                stage1_l: Some(1.524),
                ..Default::default()
            });
            let s = plan(req).await.starter.unwrap();
            close(s.stage1_cells_billion, expected);
            close(s.stage2_cells_billion, 0.0);
            close(s.stage2_l, 0.0);
            close(s.pitched_m_per_ml, expected * 1000.0 / 20_000.0);
        }
        // cells_sec_starter(221, 1.75) 301.104 (no stir) and 436.601 (stir); a zero first
        // stage leaves the 221 cells unchanged (cells_starter with no starter).
        for (stir, expected) in [
            (false, 301.104_247_106_370_57),
            (true, 436.601_158_304_237_23),
        ] {
            let mut req = plan_req(vec![liquid_pack(1.0, 221.0, 100.0)]);
            req.starter = Some(StarterOptions {
                stir_plate: stir,
                two_stage: true,
                stage1_l: Some(0.0),
                stage2_l: Some(1.75),
                ..Default::default()
            });
            let s = plan(req).await.starter.unwrap();
            close(s.stage1_cells_billion, 221.0);
            close(s.stage2_cells_billion, expected);
            close(s.pitched_m_per_ml, expected * 1000.0 / 20_000.0);
        }
    }

    #[tokio::test]
    async fn pitch_starter_dme_matches_vector() {
        // dme_oz(33.8 fl oz, 1.036) = 3.38 oz.
        let mut req = plan_req(vec![liquid_pack(1.0, 100.0, 70.71)]);
        req.starter = Some(StarterOptions {
            stage1_l: Some(convert::floz_to_l(33.8)),
            ..Default::default()
        });
        let s = plan(req).await.starter.unwrap();
        close(s.stage1_dme_g, convert::oz_to_g(3.380_000_000_000_002_6));
        close(s.stage2_dme_g, 0.0);
    }

    #[tokio::test]
    async fn pitch_no_starter_est_cells_matches_vector() {
        // est_cells_ml(221.12, 20 000 mL) = 11.056.
        let r = plan(plan_req(vec![liquid_pack(1.0, 221.12, 100.0)])).await;
        close(r.pitched_m_per_ml, 11.056);
    }

    #[tokio::test]
    async fn pitch_dry_by_weight_matches_vectors() {
        // Vectors: tot_yeast_cells_needed 150 (rate 75), dry_grams_vol 15, packs_dry(5),
        // dry5(5), dry_hydrate 125 and goferm 6.25 for 5 g, dry range 18-150.
        let r = plan(plan_req(vec![dry_by_weight(5.0)])).await;
        assert!(r.dry_by_weight);
        close(r.cells_needed_billion, 150.0);
        close(r.dry_needed_g, 15.0);
        close(r.dry_rate_g_per_hl, 75.0);
        close(r.dry_min_g_per_hl, 18.0);
        close(r.dry_max_g_per_hl, 150.0);
        close(r.rehydrate_water_ml, 125.0);
        close(r.goferm_g, 6.25);
        let mut req = plan_req(vec![dry_by_weight(5.0)]);
        req.dry_rate_g_per_hl = Some(25.0);
        req.rec_dry_rate_g_per_hl = Some(75.0);
        let r = plan(req).await;
        close(r.dry_needed_g, 5.0);
        close(r.dry_packs_11_5g, 0.434_782_608_695_652_16);
        close(r.dry_packs_5g, 1.0);
    }

    #[tokio::test]
    async fn pitch_dry_package_mode_matches_vector() {
        // dry_grams_pkg(150 billion, 18 per g) = 8.3333; a package counts 11 g.
        let y = PitchYeast {
            form: form(proto::YeastForm::Dry),
            ..liquid_pack(1.0, 100.0, 90.0)
        };
        let mut req = plan_req(vec![y]);
        req.pitch_rate_m_ml_p = Some(rate_for(150.0));
        let r = plan(req).await;
        assert!(!r.dry_by_weight);
        close(r.cells_needed_billion, 150.0);
        close(r.dry_needed_g, 8.333_333_333_333_334);
        close(r.dry_in_recipe_g, 11.0);
    }

    #[tokio::test]
    async fn pitch_starter_tool_ranges_match_vectors() {
        // pitch_min_max_tool: ale/cider [0.45, 1.05], lager [1.25, 1.75].
        for (kind, lo, hi) in [
            (proto::StarterType::AleOrCider, 0.45, 1.05),
            (proto::StarterType::Lager, 1.25, 1.75),
        ] {
            let mut req = plan_req(vec![liquid_pack(1.0, 100.0, 80.0)]);
            req.target = Some(Target::StarterType(kind as i32));
            let r = plan(req).await;
            close(r.pitch_min_m_ml_p, lo);
            close(r.pitch_max_m_ml_p, hi);
            close(r.pitch_rate_m_ml_p, 0.75);
            close(r.dry_rate_g_per_hl, 75.0);
        }
    }

    #[tokio::test]
    async fn pitch_option_targets_override_defaults() {
        let mut req = plan_req(vec![liquid_pack(1.0, 100.0, 80.0)]);
        req.options = Some(PitchOptions {
            targets: Some(ProtoTargets {
                ale_m_ml_p: Some(1.0),
                ..Default::default()
            }),
            ..Default::default()
        });
        close(plan(req).await.pitch_rate_m_ml_p, 1.0);
    }

    #[tokio::test]
    async fn pitch_rejects_bad_input() {
        let ok = || liquid_pack(1.0, 100.0, 80.0);
        let mut cases: Vec<(&str, PitchPlanRequest)> = Vec::new();
        let mut r = plan_req(vec![ok()]);
        r.volume_l = f64::NAN;
        cases.push(("volume_l", r));
        let mut r = plan_req(vec![ok()]);
        r.og_sg = -1.0;
        cases.push(("og_sg", r));
        let mut r = plan_req(vec![ok()]);
        r.target = None;
        cases.push(("target", r));
        cases.push((
            "yeasts[0].form",
            plan_req(vec![PitchYeast { form: 0, ..ok() }]),
        ));
        cases.push((
            "yeasts[0].amount",
            plan_req(vec![PitchYeast {
                amount: -1.0,
                ..ok()
            }]),
        ));
        cases.push((
            "yeasts[0].viability_pct",
            plan_req(vec![PitchYeast {
                viability_pct: Some(120.0),
                ..ok()
            }]),
        ));
        cases.push((
            "yeasts[0].package_epoch_day",
            plan_req(vec![PitchYeast {
                package_epoch_day: Some(i64::MIN),
                ..ok()
            }]),
        ));
        let mut r = plan_req(vec![ok()]);
        r.target = Some(Target::StarterType(0));
        cases.push(("starter_type", r));
        let mut r = plan_req(vec![ok(), ok()]);
        r.target = Some(Target::StarterType(1));
        cases.push(("yeasts", r));
        let mut r = plan_req(vec![ok()]);
        r.target = Some(Target::Recipe(RecipePitchTarget {
            kind: 0,
            ale_type: 1,
        }));
        cases.push(("target.recipe.kind", r));
        let mut r = plan_req(vec![ok()]);
        r.target = Some(Target::Recipe(RecipePitchTarget {
            kind: RecipeKind::AllGrain as i32,
            ale_type: 0,
        }));
        cases.push(("target.recipe.ale_type", r));
        let mut r = plan_req(vec![ok()]);
        r.starter = Some(StarterOptions {
            stage1_l: Some(-1.0),
            ..Default::default()
        });
        cases.push(("starter.stage1_l", r));
        for (field, req) in cases {
            let e = plan_err(req).await;
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field, "{}", e.message());
        }
    }

    #[tokio::test]
    async fn pitch_wine_like_kind_ignores_unset_ale_type() {
        let mut req = plan_req(vec![liquid_pack(1.0, 100.0, 80.0)]);
        req.target = Some(Target::Recipe(RecipePitchTarget {
            kind: RecipeKind::Mead as i32,
            ale_type: 0,
        }));
        close(plan(req).await.pitch_rate_m_ml_p, 0.5);
    }

    #[tokio::test]
    async fn pitch_engine_error_maps_to_status() {
        // No cells and a starter requested: new_starter_l rejects a zero supply.
        let mut req = plan_req(vec![]);
        req.starter = Some(StarterOptions::default());
        let e = plan_err(req).await;
        assert_eq!(e.code(), Code::InvalidArgument);
        assert_eq!(field_of(&e), "avail_billion");
    }

    // --- MeadNutrients ---

    fn mead_req() -> MeadNutrientsRequest {
        MeadNutrientsRequest {
            og_sg: 1.1,
            volume_l: 19.0,
            honey_pct: None,
            nitrogen: proto::YeastNitrogen::Low as i32,
            method: proto::MeadMethod::Tosna as i32,
            yeast_g: None,
        }
    }

    async fn mead(r: MeadNutrientsRequest) -> Result<MeadNutrientsResponse, Status> {
        YeastHandler
            .mead_nutrients(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn mead_yeast_and_yan_match_vectors() {
        // mead_yeast_grams(1.1, 19) 4.75, goferm 5.9375, hydrate 118.75, yan_target 178.2667.
        let r = mead(mead_req()).await.unwrap();
        close(r.pitch_rate_g_per_l, 0.25);
        close(r.recommended_yeast_g, 4.75);
        close(r.goferm_g, 5.9375);
        close(r.hydrate_water_ml, 118.75);
        close(r.yan_target_ppm, 178.266_652_499_999_62);
        let r = mead(MeadNutrientsRequest {
            yeast_g: Some(5.0),
            ..mead_req()
        })
        .await
        .unwrap();
        close(r.goferm_g, 6.25);
        close(r.hydrate_water_ml, 125.0);
    }

    #[tokio::test]
    async fn mead_nutrient_programme_matches_vectors() {
        let volume_l = convert::gal_to_l(5.0197);
        let fermaid_o = mead(MeadNutrientsRequest {
            volume_l,
            ..mead_req()
        })
        .await
        .unwrap();
        close(fermaid_o.nutrient_total_g, 17.896_902_311_084_965);
        close(fermaid_o.nutrient_per_addition_g, 4.474_225_577_771_241);
        // Fermaid-O: 40 ppm per g/L, equivalent x4 (vectors mead_yan_added 45.2 / 180.8).
        close(
            fermaid_o.yan_added_ppm,
            40.0 * fermaid_o.nutrient_total_g / volume_l,
        );
        close(fermaid_o.yan_equivalent_ppm, 4.0 * fermaid_o.yan_added_ppm);
        let fermaid_k = mead(MeadNutrientsRequest {
            volume_l,
            method: proto::MeadMethod::Tiosna as i32,
            ..mead_req()
        })
        .await
        .unwrap();
        close(fermaid_k.nutrient_total_g, 8.948_451_155_542_482);
        close(fermaid_k.nutrient_per_addition_g, 2.237_112_788_885_620_6);
        close(
            fermaid_k.yan_added_ppm,
            100.0 * fermaid_k.nutrient_total_g / volume_l,
        );
        close(fermaid_k.yan_equivalent_ppm, fermaid_k.yan_added_ppm);
        let medium = mead(MeadNutrientsRequest {
            volume_l,
            nitrogen: proto::YeastNitrogen::Medium as i32,
            ..mead_req()
        })
        .await
        .unwrap();
        close(medium.yan_target_ppm, 213.919_982_999_999_55);
        close(medium.nutrient_per_addition_g, 5.369_070_693_325_488);
    }

    #[tokio::test]
    async fn mead_rejects_bad_input() {
        for (field, req) in [
            (
                "og_sg",
                MeadNutrientsRequest {
                    og_sg: f64::NAN,
                    ..mead_req()
                },
            ),
            (
                "volume_l",
                MeadNutrientsRequest {
                    volume_l: -1.0,
                    ..mead_req()
                },
            ),
            (
                "honey_pct",
                MeadNutrientsRequest {
                    honey_pct: Some(-5.0),
                    ..mead_req()
                },
            ),
            (
                "yeast_g",
                MeadNutrientsRequest {
                    yeast_g: Some(f64::INFINITY),
                    ..mead_req()
                },
            ),
            (
                "nitrogen",
                MeadNutrientsRequest {
                    nitrogen: 0,
                    ..mead_req()
                },
            ),
            (
                "method",
                MeadNutrientsRequest {
                    method: 3,
                    ..mead_req()
                },
            ),
        ] {
            let e = mead(req).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    // --- WineStabilizers ---

    async fn wine(r: WineStabilizersRequest) -> Result<WineStabilizersResponse, Status> {
        YeastHandler
            .wine_stabilizers(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn wine_matches_vectors() {
        // so2_rec(3.0), kmeta_g(5.0197, 35, 0), sorbate_g(5.0197).
        let r = wine(WineStabilizersRequest {
            ph: 3.0,
            volume_l: convert::gal_to_l(5.0197),
            free_so2_ppm: None,
            target_so2_ppm: Some(35.0),
        })
        .await
        .unwrap();
        close(r.recommended_so2_ppm, 10.769_843_931_248_255);
        close(r.kmeta_g, 1.154_487_426_215_278);
        close(r.sorbate_g, 6.274_625);
        // kmeta_g(5, 50, 10); the default target 30 against free 30 gives 0; below target is
        // negative and not clamped (kmeta_g(10, 20, 25)).
        let r = wine(WineStabilizersRequest {
            ph: 3.4,
            volume_l: convert::gal_to_l(5.0),
            free_so2_ppm: Some(10.0),
            target_so2_ppm: Some(50.0),
        })
        .await
        .unwrap();
        close(r.kmeta_g, 1.314_236_111_111_111_2);
        let r = wine(WineStabilizersRequest {
            ph: 3.4,
            volume_l: convert::gal_to_l(1.0),
            free_so2_ppm: Some(30.0),
            target_so2_ppm: None,
        })
        .await
        .unwrap();
        close(r.kmeta_g, 0.0);
        let r = wine(WineStabilizersRequest {
            ph: 3.4,
            volume_l: convert::gal_to_l(10.0),
            free_so2_ppm: Some(25.0),
            target_so2_ppm: Some(20.0),
        })
        .await
        .unwrap();
        close(r.kmeta_g, -0.328_559_027_777_777_8);
    }

    #[tokio::test]
    async fn wine_rejects_bad_input() {
        let base = || WineStabilizersRequest {
            ph: 3.4,
            volume_l: 19.0,
            ..Default::default()
        };
        for (field, req) in [
            (
                "ph",
                WineStabilizersRequest {
                    ph: f64::NAN,
                    ..base()
                },
            ),
            (
                "volume_l",
                WineStabilizersRequest {
                    volume_l: -1.0,
                    ..base()
                },
            ),
            (
                "target_so2_ppm",
                WineStabilizersRequest {
                    target_so2_ppm: Some(-1.0),
                    ..base()
                },
            ),
            (
                "free_so2_ppm",
                WineStabilizersRequest {
                    free_so2_ppm: Some(f64::NAN),
                    ..base()
                },
            ),
        ] {
            let e = wine(req).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    // --- Backsweeten ---

    fn back_req(agent: BacksweetenAgent) -> BacksweetenRequest {
        BacksweetenRequest {
            volume_l: 19.0,
            beverage_sg: 0.998,
            target_sg: 1.01,
            agent: agent as i32,
            agent_brix: None,
        }
    }

    async fn back(r: BacksweetenRequest) -> Result<BacksweetenResponse, Status> {
        YeastHandler
            .backsweeten(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn backsweeten_matches_vectors() {
        let r = back(back_req(BacksweetenAgent::Honey)).await.unwrap();
        close(r.agent_brix, 82.5);
        close(r.agent_l, 0.526_324_741_107_420_3);
        close(r.agent_kg, 0.759_728_720_614_342_3);
        let r = back(BacksweetenRequest {
            volume_l: 18.93,
            beverage_sg: 1.015,
            target_sg: 1.025,
            ..back_req(BacksweetenAgent::Honey)
        })
        .await
        .unwrap();
        close(r.agent_l, 0.452_662_207_360_102_76);
        close(r.agent_kg, 0.653_399_798_277_707_3);
        let r = back(back_req(BacksweetenAgent::AppleJuiceConcentrate))
            .await
            .unwrap();
        close(r.agent_brix, 70.0);
        close(r.agent_l, 0.660_675_003_437_882_1);
        close(r.agent_kg, 0.895_395_148_852_162_2);
        let r = back(BacksweetenRequest {
            volume_l: 0.0,
            beverage_sg: 1.0,
            target_sg: 1.02,
            ..back_req(BacksweetenAgent::Honey)
        })
        .await
        .unwrap();
        close(r.agent_l, 0.0);
    }

    #[tokio::test]
    async fn backsweeten_custom_potential() {
        // The agent's own potential gives the same result as the default.
        let same = back(BacksweetenRequest {
            agent_brix: Some(82.5),
            ..back_req(BacksweetenAgent::Honey)
        })
        .await
        .unwrap();
        let default = back(back_req(BacksweetenAgent::Honey)).await.unwrap();
        close(same.agent_l, default.agent_l);
        close(same.agent_kg, default.agent_kg);
        // honey_sg(15) = 1.0611179335048229 (vector), brix_to_sg(15) from the Brix formula.
        let r = back(BacksweetenRequest {
            agent_brix: Some(15.0),
            ..back_req(BacksweetenAgent::Other)
        })
        .await
        .unwrap();
        close(r.agent_brix, 15.0);
        close(
            r.agent_l,
            19.0 * (0.998 - 1.01) / (1.01 - 1.061_117_933_504_822_9),
        );
        close(r.agent_kg, brix_to_sg(15.0) * r.agent_l);
    }

    #[tokio::test]
    async fn backsweeten_rejects_bad_input() {
        for (field, req) in [
            (
                "agent",
                BacksweetenRequest {
                    agent: 0,
                    ..back_req(BacksweetenAgent::Honey)
                },
            ),
            (
                "beverage_sg",
                BacksweetenRequest {
                    beverage_sg: f64::NAN,
                    ..back_req(BacksweetenAgent::Honey)
                },
            ),
            (
                "volume_l",
                BacksweetenRequest {
                    volume_l: -1.0,
                    ..back_req(BacksweetenAgent::Honey)
                },
            ),
            (
                "agent_brix",
                BacksweetenRequest {
                    agent_brix: Some(-1.0),
                    ..back_req(BacksweetenAgent::Honey)
                },
            ),
        ] {
            let e = back(req).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    // --- Carbonation ---

    fn carb_req(kind: CarbonationType, agent: PA) -> CarbonationRequest {
        CarbonationRequest {
            r#type: kind as i32,
            co2_volumes: 2.4,
            temp_c: convert::f_to_c(68.0),
            volume_l: convert::gal_to_l(5.0197),
            agent: agent as i32,
            carb_rate_pct: None,
        }
    }

    async fn carb(r: CarbonationRequest) -> Result<CarbonationResponse, Status> {
        YeastHandler
            .carbonation(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn carbonation_bottle_matches_vectors() {
        // carb_sugar_agent 117.349 / 180.483 / 106.778, residual_co2(68) 0.8615, carb_equiv(0) 100.
        let r = carb(carb_req(CarbonationType::Bottle, PA::CornSugar))
            .await
            .unwrap();
        close(r.priming_g, 117.349_050_709_321_26);
        close(r.corn_sugar_g, 117.349_050_709_321_26);
        close(r.dme_g, 180.482_839_990_936_1);
        close(r.table_sugar_g, 106.778_026_123_131_27);
        close(r.residual_co2_volumes, 0.861_487_199_999_999_3);
        close(r.sugar_equiv, 100.0);
        // Priming agent and efficiency: carb_amount(rate 80) DME 225.6035, equiv 1.25.
        let r = carb(CarbonationRequest {
            carb_rate_pct: Some(80.0),
            ..carb_req(CarbonationType::Bottle, PA::Dme)
        })
        .await
        .unwrap();
        close(r.priming_g, 225.603_549_988_670_12);
        close(r.sugar_equiv, 1.25);
        close(r.corn_sugar_g, 117.349_050_709_321_26);
        let r = carb(CarbonationRequest {
            carb_rate_pct: Some(60.0),
            ..carb_req(CarbonationType::Bottle, PA::TableSugar)
        })
        .await
        .unwrap();
        close(r.priming_g, 177.963_376_871_885_46);
    }

    #[tokio::test]
    async fn carbonation_keg_matches_vectors() {
        // co2_psi(2.5, 39.2 F) = 11.8620 psi, co2_psi(2.5, 68 F) = 27.6496 psi.
        let keg = CarbonationRequest {
            co2_volumes: 2.5,
            temp_c: 4.0,
            agent: 0,
            ..carb_req(CarbonationType::Keg, PA::CornSugar)
        };
        let r = carb(keg).await.unwrap();
        close(
            r.keg_pressure_bar,
            convert::psi_to_bar(11.862_044_466_800_004),
        );
        close(r.priming_g, 0.0);
        let r = carb(CarbonationRequest {
            temp_c: 20.0,
            ..keg
        })
        .await
        .unwrap();
        close(r.keg_pressure_bar, convert::psi_to_bar(27.649_627_43));
        // Keg with priming sugar: same sugar as bottling.
        let r = carb(carb_req(CarbonationType::KegPriming, PA::CornSugar))
            .await
            .unwrap();
        close(r.priming_g, 117.349_050_709_321_26);
    }

    #[tokio::test]
    async fn carbonation_rejects_bad_input() {
        let ok = || carb_req(CarbonationType::Bottle, PA::CornSugar);
        for (field, req) in [
            ("type", CarbonationRequest { r#type: 0, ..ok() }),
            ("type", CarbonationRequest { r#type: 9, ..ok() }),
            ("agent", CarbonationRequest { agent: 0, ..ok() }),
            (
                "temp_c",
                CarbonationRequest {
                    temp_c: f64::NAN,
                    ..ok()
                },
            ),
            (
                "volume_l",
                CarbonationRequest {
                    volume_l: -1.0,
                    ..ok()
                },
            ),
            (
                "co2_volumes",
                CarbonationRequest {
                    co2_volumes: -0.1,
                    ..ok()
                },
            ),
            (
                "carb_rate_pct",
                CarbonationRequest {
                    carb_rate_pct: Some(-1.0),
                    ..ok()
                },
            ),
        ] {
            let e = carb(req).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }
}
