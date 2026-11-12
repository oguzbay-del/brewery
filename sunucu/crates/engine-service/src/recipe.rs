//! `RecipeService` implementation: maps the metric wire request to `engine::recipe::RecipeInput`
//! (BeerSmith units) and computes the recipe summary (ADR-0005 items 2-5).

use engine::density::{self, ColorUnit};
use engine::hops::{self, HopEquation};
use engine::recipe::{
    Equipment, Fermentable, GrainType, GrainUse, Hop, HopForm, HopUse, Mash, MashStep, Misc,
    MiscUse, Options, RecipeInput, RecipeType, StepKind, Yeast, YeastForm,
};
use tonic::{Request, Response, Status};

use crate::convert;
use crate::proto::{
    self, CalculateRecipeRequest, RecipeSummary,
    recipe_service_server::{RecipeService, RecipeServiceServer},
};
use crate::status::{bad_request, from_engine};

/// Handler implementing the recipe gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct RecipeHandler;

/// Maps a proto enum value to the engine code (`N - 1`); UNSPECIFIED (0), negative or unknown
/// values are `INVALID_ARGUMENT` naming `field`.
fn engine_code<T>(
    field: &str,
    raw: i32,
    from_code: impl FnOnce(i64) -> engine::Result<T>,
) -> Result<T, Status> {
    if raw <= 0 {
        return Err(bad_request(
            "enum value must be specified",
            field,
            "must not be UNSPECIFIED",
        ));
    }
    from_code(i64::from(raw) - 1)
        .map_err(|_| bad_request("unknown enum value", field, "unknown enum value"))
}

fn require<'a, T>(field: &str, value: Option<&'a T>) -> Result<&'a T, Status> {
    value.ok_or_else(|| bad_request("message is required", field, "must be set"))
}

fn map_fermentable(i: usize, f: &proto::Fermentable) -> Result<Fermentable, Status> {
    Ok(Fermentable {
        name: f.name.clone(),
        amount_oz: convert::kg_to_oz(f.amount_kg),
        yield_pct: f.yield_pct,
        color_srm: f.color_srm,
        kind: engine_code(
            &format!("fermentables[{i}].kind"),
            f.kind,
            GrainType::from_code,
        )?,
        usage: engine_code(
            &format!("fermentables[{i}].usage"),
            f.usage,
            GrainUse::from_code,
        )?,
        not_fermentable: f.not_fermentable,
        late_extract_min: f.late_extract_min,
        max_in_batch_pct: f.max_in_batch_pct,
        ibu_gal_per_lb: convert::ibu_l_per_kg_to_gal_per_lb(f.ibu_l_per_kg),
    })
}

fn map_hop(i: usize, h: &proto::Hop) -> Result<Hop, Status> {
    let form = engine_code(&format!("hops[{i}].form"), h.form, HopForm::from_code)?;
    // Amounts of the extract forms are mL, all others g.
    let amount_oz = if matches!(form, HopForm::Co2Extract | HopForm::IsomerizedExtract) {
        convert::ml_to_floz(h.amount)
    } else {
        convert::g_to_oz(h.amount)
    };
    Ok(Hop {
        amount_oz,
        alpha_pct: h.alpha_pct,
        form,
        usage: engine_code(&format!("hops[{i}].usage"), h.usage, HopUse::from_code)?,
        time_min: h.time_min,
        whirlpool_temp_f: convert::c_to_f(h.whirlpool_temp_c),
    })
}

fn map_yeast(y: &proto::RecipeYeast) -> Yeast {
    Yeast {
        min_attenuation_pct: y.min_attenuation_pct,
        max_attenuation_pct: y.max_attenuation_pct,
        tolerance_abv_pct: y.tolerance_abv_pct,
        add_to_secondary: y.add_to_secondary,
        form: YeastForm::Liquid,
        total_cells_billion: 0.0,
        viable_cells_billion: 0.0,
        packs: 0.0,
        viability_pct: 0.0,
        use_packages: true,
        amount_oz: 0.0,
        cells_per_ml_billion: 0.0,
        dry_pitch_rate_g_per_hl: 0.0,
    }
}

fn map_options(o: Option<&proto::RecipeOptions>) -> Options {
    let mut out = Options::default();
    let Some(o) = o else { return out };
    if let Some(v) = o.steep_efficiency_pct {
        out.steep_efficiency_pct = v;
    }
    if let Some(v) = o.mash_fg_adjust {
        out.mash_fg_adjust = v;
    }
    if let Some(v) = o.mash_fg_adj_temp_c {
        out.mash_fg_adj_temp_f = convert::c_to_f(v);
    }
    if let Some(v) = o.mash_fg_adj_slope_pct_per_c {
        out.mash_fg_adj_slope_pct_per_f = convert::per_c_to_per_f(v);
    }
    if let Some(v) = o.hop_first_wort_adj_pct {
        out.hop.first_wort_adj_pct = v;
    }
    if let Some(v) = o.hop_type_adj {
        out.hop.type_adj = v;
    }
    if let Some(v) = o.hop_leaf_adj_pct {
        out.hop.leaf_adj_pct = v;
    }
    if let Some(v) = o.hop_plug_adj_pct {
        out.hop.plug_adj_pct = v;
    }
    if let Some(v) = o.hop_pellet_adj_pct {
        out.hop.pellet_adj_pct = v;
    }
    if let Some(v) = o.hop_mash_adj_pct {
        out.hop.mash_adj_pct = v;
    }
    out
}

/// Maps the request to the engine input; the boil volume for IBU is read from the result.
fn to_engine(r: &CalculateRecipeRequest) -> Result<RecipeInput, Status> {
    let eq = require("equipment", r.equipment.as_ref())?;
    let mash = require("mash", r.mash.as_ref())?;
    let kind = engine_code("kind", r.kind, RecipeType::from_code)?;
    // Bound the lists before iterating so the per-item error path stays cheap; the engine
    // `validate()` repeats the check.
    for (name, len) in [
        ("fermentables", r.fermentables.len()),
        ("hops", r.hops.len()),
        ("yeasts", r.yeasts.len()),
        ("miscs", r.miscs.len()),
        ("mash.steps", mash.steps.len()),
    ] {
        if len > engine::recipe::MAX_LIST_LEN {
            return Err(bad_request(
                "too many items",
                name,
                "too many items (max 128)",
            ));
        }
    }
    let fermentables = r
        .fermentables
        .iter()
        .enumerate()
        .map(|(i, f)| map_fermentable(i, f))
        .collect::<Result<Vec<_>, _>>()?;
    let hops = r
        .hops
        .iter()
        .enumerate()
        .map(|(i, h)| map_hop(i, h))
        .collect::<Result<Vec<_>, _>>()?;
    let miscs = r
        .miscs
        .iter()
        .enumerate()
        .map(|(i, m)| {
            Ok(Misc {
                usage: engine_code(&format!("miscs[{i}].usage"), m.usage, MiscUse::from_code)?,
                time_min: m.time_min,
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;
    let steps = mash
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Ok(MashStep {
                kind: engine_code(
                    &format!("mash.steps[{i}].kind"),
                    s.kind,
                    StepKind::from_code,
                )?,
                temp_f: convert::c_to_f(s.temp_c),
                infusion_floz: convert::l_to_floz(s.infusion_l),
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;
    Ok(RecipeInput {
        kind,
        volume_floz: convert::l_to_floz(r.batch_volume_l),
        boil_vol_floz: convert::l_to_floz(r.boil_volume_l),
        boil_time_min: r.boil_time_min,
        efficiency_pct: r.efficiency_pct,
        equipment: Equipment {
            trub_loss_floz: convert::l_to_floz(eq.trub_loss_l),
            pre_trub_loss_floz: convert::l_to_floz(eq.pre_trub_loss_l),
            altitude_ft: convert::m_to_ft(eq.altitude_m),
            hop_util_pct: eq.hop_util_pct,
            whirlpool_carry: eq.whirlpool_carry,
            whirlpool_time_min: eq.whirlpool_time_min,
        },
        fermentables,
        hops,
        yeasts: r.yeasts.iter().map(map_yeast).collect(),
        miscs,
        mash: Mash {
            grain_weight_oz: convert::kg_to_oz(mash.grain_weight_kg),
            biab: mash.biab,
            steps,
        },
        options: map_options(r.options.as_ref()),
    })
}

fn ibu_equation(raw: i32) -> Result<HopEquation, Status> {
    match proto::IbuEquation::try_from(raw) {
        Ok(proto::IbuEquation::Tinseth) => Ok(HopEquation::Tinseth),
        Ok(proto::IbuEquation::Rager) => Ok(HopEquation::Rager),
        Ok(proto::IbuEquation::Garetz) => Ok(HopEquation::Garetz),
        Ok(proto::IbuEquation::Unspecified) | Err(_) => Err(bad_request(
            "IBU equation must be specified",
            "ibu_equation",
            "must be TINSETH, RAGER or GARETZ",
        )),
    }
}

/// A non-finite result is an error, never a NaN response.
fn finite(field: &str, v: f64) -> Result<f64, Status> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(Status::invalid_argument(format!(
            "{field} is not finite for this recipe"
        )))
    }
}

fn summarize(r: &CalculateRecipeRequest) -> Result<RecipeSummary, Status> {
    let equation = ibu_equation(r.ibu_equation)?;
    let input = to_engine(r)?;
    input.validate().map_err(|e| from_engine(&e))?;
    let e = |err: engine::Error| from_engine(&err);

    let og = finite("og_sg", density::recipe_og(&input))?;
    let fg = finite("fg_sg", density::recipe_fg(&input))?;
    let virtual_og = finite("virtual_og_sg", density::recipe_virtual_og(&input))?;
    let pre_boil = finite("pre_boil_sg", density::est_post_mash_og(&input))?;
    let abv = finite("abv_pct", density::abv(og, fg).map_err(e)?)?;
    let abw = finite("abw_pct", density::abw(og, fg).map_err(e)?)?;
    let ibu = finite(
        "ibu",
        hops::recipe_ibu(&input, equation, input.boil_vol_floz).map_err(e)?,
    )?;
    let srm = finite(
        "color_srm",
        density::recipe_color_checked(&input).map_err(e)?,
    )?;
    let ebc = finite(
        "color_ebc",
        density::convert_color(srm, ColorUnit::Srm, ColorUnit::Ebc).map_err(e)?,
    )?;
    let mash_eff = finite(
        "mash_efficiency_pct",
        density::recipe_est_mash_efficiency(&input),
    )?;
    let kcal = finite(
        "energy_kcal_per_100ml",
        convert::per_12floz_to_per_100ml(density::calories_total(og, fg).map_err(e)?),
    )?;
    let carbs_g = finite(
        "carbohydrates_g_per_100ml",
        convert::per_12floz_to_per_100ml(density::carbohydrates_grams(
            density::calories_carbs(og, fg).map_err(e)?,
        )),
    )?;
    Ok(RecipeSummary {
        og_sg: og,
        fg_sg: fg,
        virtual_og_sg: virtual_og,
        pre_boil_sg: pre_boil,
        abv_pct: abv,
        abw_pct: abw,
        ibu,
        color_srm: srm,
        color_ebc: ebc,
        mash_efficiency_pct: mash_eff,
        energy_kcal_per_100ml: kcal,
        carbohydrates_g_per_100ml: carbs_g,
    })
}

#[tonic::async_trait]
impl RecipeService for RecipeHandler {
    async fn calculate(
        &self,
        request: Request<CalculateRecipeRequest>,
    ) -> Result<Response<RecipeSummary>, Status> {
        summarize(&request.into_inner()).map(Response::new)
    }
}

/// Builds the tonic server wrapper for the recipe handler.
#[must_use]
pub fn server() -> RecipeServiceServer<RecipeHandler> {
    RecipeServiceServer::new(RecipeHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{
        CalculateRecipeRequest, Equipment as PEquipment, Fermentable as PFermentable, Hop as PHop,
        IbuEquation, Mash as PMash, MashStep as PMashStep, RecipeYeast as PYeast,
    };
    use tonic::Code;

    const TOL: f64 = 1e-9;

    /// BeerSmith code (engine) to proto enum value.
    fn p(code: i32) -> i32 {
        code + 1
    }

    #[allow(clippy::too_many_arguments)]
    fn ferm(name: &str, oz: f64, yield_pct: f64, srm: f64, kind: i32, usage: i32) -> PFermentable {
        PFermentable {
            name: name.into(),
            amount_kg: convert::oz_to_kg(oz),
            yield_pct,
            color_srm: srm,
            kind: p(kind),
            usage: p(usage),
            not_fermentable: false,
            late_extract_min: 0.0,
            max_in_batch_pct: 100.0,
            ibu_l_per_kg: 0.0,
        }
    }

    fn hop(oz: f64, alpha: f64, form: i32, usage: i32, time: f64) -> PHop {
        let amount = if form >= 3 {
            convert::floz_to_ml(oz)
        } else {
            convert::oz_to_g(oz)
        };
        PHop {
            amount,
            alpha_pct: alpha,
            form: p(form),
            usage: p(usage),
            time_min: time,
            whirlpool_temp_c: convert::f_to_c(176.0),
        }
    }

    fn yeast(min: f64, max: f64, tol: f64) -> PYeast {
        PYeast {
            min_attenuation_pct: min,
            max_attenuation_pct: max,
            tolerance_abv_pct: tol,
            add_to_secondary: false,
        }
    }

    fn equipment(trub_floz: f64, pre_trub_floz: f64) -> PEquipment {
        PEquipment {
            trub_loss_l: convert::floz_to_l(trub_floz),
            pre_trub_loss_l: convert::floz_to_l(pre_trub_floz),
            altitude_m: 0.0,
            hop_util_pct: 100.0,
            whirlpool_carry: false,
            whirlpool_time_min: 0.0,
        }
    }

    /// 01.json `recipe_og`/`recipe_fg`, note `tarif=ornek_ag`.
    fn ornek_ag() -> CalculateRecipeRequest {
        CalculateRecipeRequest {
            kind: p(2),
            batch_volume_l: convert::floz_to_l(704.0),
            boil_volume_l: 0.0,
            boil_time_min: 60.0,
            efficiency_pct: 72.0,
            equipment: Some(equipment(0.0, 0.0)),
            fermentables: vec![
                ferm("2-row", 160.0, 80.0, 2.0, 0, 0),
                ferm("Crystal", 16.0, 72.0, 60.0, 0, 0),
                ferm("Corn sugar", 16.0, 100.0, 0.0, 2, 2),
            ],
            hops: vec![],
            yeasts: vec![yeast(73.0, 77.0, 10.0)],
            miscs: vec![],
            mash: Some(PMash {
                grain_weight_kg: 0.0,
                biab: false,
                steps: vec![PMashStep {
                    kind: p(0),
                    temp_c: convert::f_to_c(152.0),
                    infusion_l: convert::floz_to_l(10.0),
                }],
            }),
            options: None,
            ibu_equation: IbuEquation::Tinseth as i32,
        }
    }

    /// 01.json, note `tarif=steep25` (extract recipe, steep efficiency 25).
    fn steep25() -> CalculateRecipeRequest {
        CalculateRecipeRequest {
            kind: p(0),
            batch_volume_l: convert::floz_to_l(640.0),
            boil_volume_l: 0.0,
            boil_time_min: 60.0,
            efficiency_pct: 72.0,
            equipment: Some(equipment(64.0, 704.0)),
            fermentables: vec![
                ferm("LME", 96.0, 78.0, 8.0, 1, 2),
                ferm("Crystal steep", 24.0, 72.0, 40.0, 0, 1),
                ferm("Roast steep", 8.0, 70.0, 400.0, 3, 1),
            ],
            hops: vec![],
            yeasts: vec![yeast(70.0, 78.0, 11.0)],
            miscs: vec![],
            mash: Some(PMash::default()),
            options: Some(proto::RecipeOptions {
                steep_efficiency_pct: Some(25.0),
                ..Default::default()
            }),
            ibu_equation: IbuEquation::Tinseth as i32,
        }
    }

    /// 02.json `recipe_ibu`, note `carry + rakım` (equation 0, 3000 ft, whirlpool carry).
    fn ibu_recipe() -> CalculateRecipeRequest {
        let mut eq = equipment(0.0, 0.0);
        eq.altitude_m = convert::ft_to_m(3000.0);
        eq.whirlpool_carry = true;
        eq.whirlpool_time_min = 20.0;
        CalculateRecipeRequest {
            kind: p(2),
            batch_volume_l: convert::floz_to_l(640.0),
            boil_volume_l: convert::floz_to_l(768.0),
            boil_time_min: 60.0,
            efficiency_pct: 72.0,
            equipment: Some(eq),
            fermentables: vec![
                ferm("pale", 160.0, 80.0, 2.0, 0, 0),
                ferm("munich", 32.0, 75.0, 9.0, 0, 0),
            ],
            hops: vec![
                hop(1.0, 12.0, 0, 0, 60.0),
                hop(1.5, 5.5, 1, 0, 15.0),
                hop(1.0, 6.0, 0, 3, 90.0),
                hop(2.0, 4.0, 2, 4, 15.0),
                hop(1.0, 9.0, 0, 1, 0.0),
                hop(0.5, 14.0, 0, 2, 60.0),
                hop(0.1, 50.0, 4, 0, 0.0),
            ],
            yeasts: vec![],
            miscs: vec![],
            mash: Some(PMash::default()),
            options: None,
            ibu_equation: IbuEquation::Tinseth as i32,
        }
    }

    async fn calc(r: CalculateRecipeRequest) -> Result<RecipeSummary, Status> {
        RecipeHandler
            .calculate(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    fn close(name: &str, a: f64, b: f64) {
        assert!((a - b).abs() < TOL, "{name}: {a} != {b}");
    }

    #[tokio::test]
    async fn all_grain_og_fg_match_vectors() {
        let s = calc(ornek_ag()).await.unwrap();
        close("og", s.og_sg, 1.060_873_890_909_091);
        close("fg", s.fg_sg, 1.010_210_996_363_636_3);
        assert!(s.abv_pct > 0.0 && s.abw_pct > 0.0);
        assert!(s.energy_kcal_per_100ml > 0.0 && s.carbohydrates_g_per_100ml > 0.0);
        close("ebc", s.color_ebc, s.color_srm * 1.97);
    }

    #[tokio::test]
    async fn extract_steep_og_fg_match_vectors() {
        let s = calc(steep25()).await.unwrap();
        close("og", s.og_sg, 1.042_131_818_181_818_2);
        close("fg", s.fg_sg, 1.010_954_272_727_272_7);
    }

    #[tokio::test]
    async fn ibu_matches_vector() {
        let s = calc(ibu_recipe()).await.unwrap();
        close("ibu", s.ibu, 157.980_844_325_717_33);
    }

    async fn invalid(r: CalculateRecipeRequest) -> Status {
        let e = calc(r).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument, "{e:?}");
        e
    }

    fn field_of(s: &Status) -> String {
        use tonic_types::StatusExt;
        s.get_error_details()
            .bad_request()
            .unwrap()
            .field_violations[0]
            .field
            .clone()
    }

    #[tokio::test]
    async fn unspecified_and_unknown_enums_are_rejected_with_field() {
        let mut r = ornek_ag();
        r.fermentables[2].kind = 0;
        assert_eq!(field_of(&invalid(r).await), "fermentables[2].kind");
        let mut r = ornek_ag();
        r.fermentables[1].usage = 99;
        assert_eq!(field_of(&invalid(r).await), "fermentables[1].usage");
        let mut r = ornek_ag();
        r.kind = 0;
        assert_eq!(field_of(&invalid(r).await), "kind");
        let mut r = ibu_recipe();
        r.ibu_equation = 0;
        assert_eq!(field_of(&invalid(r).await), "ibu_equation");
        let mut r = ibu_recipe();
        r.hops[3].form = 0;
        assert_eq!(field_of(&invalid(r).await), "hops[3].form");
    }

    #[tokio::test]
    async fn nan_volume_is_rejected() {
        let mut r = ornek_ag();
        r.batch_volume_l = f64::NAN;
        invalid(r).await;
    }

    #[tokio::test]
    async fn missing_messages_are_rejected() {
        let mut r = ornek_ag();
        r.equipment = None;
        assert_eq!(field_of(&invalid(r).await), "equipment");
        let mut r = ornek_ag();
        r.mash = None;
        assert_eq!(field_of(&invalid(r).await), "mash");
    }

    #[tokio::test]
    async fn too_many_hops_are_rejected() {
        let mut r = ibu_recipe();
        r.hops = vec![hop(1.0, 5.0, 0, 0, 60.0); 129];
        assert_eq!(field_of(&invalid(r).await), "hops");
    }
}
