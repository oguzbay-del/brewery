//! `WaterService` implementation: water profiles, mash pH, acid amounts and salt matching over
//! `engine::water` (metric; only the salt matching volumes go to US gallons, via `convert`).
//!
//! Unset messages are read as their defaults (all-zero ion profile, zero volumes), as in the
//! spec's field defaults (§8).

use engine::recipe::{GrainType, MiscUse, RecipeType};
use engine::water::{
    self, Acid, AcidMaltInput, AcidMaltRaInput, AgentVolumeInput, BoilScaleInput, ION_COUNT,
    MashAcidAmountInput, MashModel, MiscAcidRaInput, Salt, SpargeAcidInput, TotalWaterIonInput,
};
use tonic::{Request, Response, Status};

use crate::check::{finite, non_negative, out};
use crate::convert;
use crate::proto::{
    AdjustAcidRequest, AdjustAcidResponse, CalculateWaterRequest, CalculateWaterResponse,
    DescribeProfileRequest, DescribeProfileResponse, IonProfile, MashGrain, MashPhResult,
    MatchProfileRequest, MatchProfileResponse, ProfileStats, ProfileToolRequest,
    ProfileToolResponse, SaltAmount, SourceWater, SulfateChlorideBalance as WireBalance,
    WaterVolumes, WaterWarning, adjust_acid_request,
    water_service_server::{WaterService, WaterServiceServer},
};
use crate::recipe::engine_code;
use crate::status::{bad_request, from_engine};

/// Handler implementing the water gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct WaterHandler;

/// Salts below this mass are left out of the result (spec §7.2 `SetWaterAddition`), g.
const MIN_SALT_G: f64 = 0.01;

type Ions = [f64; ION_COUNT];

/// Wire name and value of each ion of an [`IonProfile`] in engine order.
fn ion_values(p: &IonProfile) -> [(&'static str, f64); ION_COUNT] {
    [
        ("calcium_ppm", p.calcium_ppm),
        ("magnesium_ppm", p.magnesium_ppm),
        ("sodium_ppm", p.sodium_ppm),
        ("sulfate_ppm", p.sulfate_ppm),
        ("chloride_ppm", p.chloride_ppm),
        ("bicarbonate_ppm", p.bicarbonate_ppm),
    ]
}

/// Reads an optional wire profile (unset = all zero); every ion must be finite and not negative.
fn ions_in(field: &str, p: Option<&IonProfile>) -> Result<Ions, Status> {
    let mut a = [0.0; ION_COUNT];
    if let Some(p) = p {
        for (slot, (name, v)) in a.iter_mut().zip(ion_values(p)) {
            *slot = non_negative(&format!("{field}.{name}"), v)?;
        }
    }
    Ok(a)
}

/// Writes an engine profile to the wire; every ion must be finite.
fn ions_out(field: &str, a: &Ions) -> Result<IonProfile, Status> {
    let f = |name: &str, i: usize| out(&format!("{field}.{name}"), a[i]);
    Ok(IonProfile {
        calcium_ppm: f("calcium_ppm", 0)?,
        magnesium_ppm: f("magnesium_ppm", 1)?,
        sodium_ppm: f("sodium_ppm", 2)?,
        sulfate_ppm: f("sulfate_ppm", 3)?,
        chloride_ppm: f("chloride_ppm", 4)?,
        bicarbonate_ppm: f("bicarbonate_ppm", 5)?,
    })
}

fn add_ions(a: &Ions, b: &Ions) -> Ions {
    let mut o = *a;
    for (x, y) in o.iter_mut().zip(b) {
        *x += y;
    }
    o
}

/// Proto value to engine salt (proto value = position in `Salt::ALL` + 1).
fn salt_of(field: &str, raw: i32) -> Result<Salt, Status> {
    engine_code(field, raw, |c| {
        usize::try_from(c)
            .ok()
            .and_then(|i| Salt::ALL.get(i).copied())
            .ok_or_else(|| engine::Error::invalid("salt", "unknown code"))
    })
}

/// Proto value of a salt.
fn salt_wire(salt: Salt) -> i32 {
    Salt::ALL
        .iter()
        .position(|s| *s == salt)
        .and_then(|i| i32::try_from(i + 1).ok())
        .unwrap_or(0)
}

fn excluded_in(field: &str, raw: &[i32]) -> Result<Vec<Salt>, Status> {
    raw.iter()
        .enumerate()
        .map(|(i, s)| salt_of(&format!("{field}[{i}]"), *s))
        .collect()
}

/// Salt amounts above the leaving-out threshold, in `Salt::ALL` order.
fn salt_amounts(field: &str, grams: &[f64; 8]) -> Result<Vec<SaltAmount>, Status> {
    let mut v = Vec::new();
    for (salt, g) in Salt::ALL.into_iter().zip(grams) {
        let g = out(field, *g)?;
        if g >= MIN_SALT_G {
            v.push(SaltAmount {
                salt: salt_wire(salt),
                amount_g: g,
            });
        }
    }
    Ok(v)
}

fn is_mash_recipe(kind: RecipeType) -> bool {
    matches!(kind, RecipeType::PartialMash | RecipeType::AllGrain)
}

/// Validated recipe volumes, L.
#[derive(Debug, Clone, Copy)]
struct Volumes {
    mash_water: f64,
    sparge: f64,
    mash_available: f64,
    kettle_top_up: f64,
    boil: f64,
    post_boil: f64,
    cool_loss: f64,
    trub_loss: f64,
    fermenter_top_up: f64,
    batch: f64,
    has_boil_ing: bool,
}

fn volumes_in(v: Option<&WaterVolumes>) -> Result<Volumes, Status> {
    let d = WaterVolumes::default();
    let v = v.unwrap_or(&d);
    let n = |name: &str, x: f64| non_negative(&format!("volumes.{name}"), x);
    Ok(Volumes {
        mash_water: n("mash_water_l", v.mash_water_l)?,
        sparge: n("sparge_l", v.sparge_l)?,
        mash_available: n("mash_available_l", v.mash_available_l)?,
        kettle_top_up: n("kettle_top_up_l", v.kettle_top_up_l)?,
        boil: n("boil_l", v.boil_l)?,
        post_boil: n("post_boil_l", v.post_boil_l)?,
        cool_loss: n("cool_loss_l", v.cool_loss_l)?,
        trub_loss: n("trub_loss_l", v.trub_loss_l)?,
        fermenter_top_up: n("fermenter_top_up_l", v.fermenter_top_up_l)?,
        batch: n("batch_l", v.batch_l)?,
        has_boil_ing: v.has_boil_ingredients,
    })
}

/// Boil concentration factor `S_boil` of the recipe volumes (spec §3.4).
fn boil_scale_of(v: &Volumes) -> Result<f64, Status> {
    water::boil_scale(&BoilScaleInput {
        vb_l: v.boil,
        vp_l: v.post_boil,
        cool_loss_l: v.cool_loss,
        trub_loss_l: v.trub_loss,
        top_up_l: v.fermenter_top_up,
        has_boil_ing: v.has_boil_ing,
    })
    .map_err(|e| from_engine(&e))
}

/// Source waters: amount-weighted pairs for the base profile and the water pH.
type Waters = (Vec<(f64, Ions)>, Vec<(f64, f64)>);

fn waters_in(waters: &[SourceWater]) -> Result<Waters, Status> {
    let mut ions = Vec::with_capacity(waters.len());
    let mut ph = Vec::with_capacity(waters.len());
    for (i, w) in waters.iter().enumerate() {
        let amount = non_negative(&format!("waters[{i}].amount_l"), w.amount_l)?;
        ions.push((
            amount,
            ions_in(&format!("waters[{i}].ions"), w.ions.as_ref())?,
        ));
        ph.push((amount, finite(&format!("waters[{i}].ph"), w.ph)?));
    }
    Ok((ions, ph))
}

/// Derived figures of an ion profile (spec §4).
fn stats_of(p: &Ions) -> Result<ProfileStats, Status> {
    let ra = water::residual_alkalinity(p[0], p[1], p[5]);
    let ratio = water::sulfate_chloride_ratio(p[3], p[4]);
    let range = water::color_range(ra);
    Ok(ProfileStats {
        alkalinity_ppm: out("alkalinity_ppm", water::alkalinity(p[5]))?,
        hardness_ppm: out("hardness_ppm", water::hardness(p[0], p[1]))?,
        residual_alkalinity_ppm: out("residual_alkalinity_ppm", ra)?,
        sulfate_chloride_ratio: out("sulfate_chloride_ratio", ratio)?,
        balance: balance_wire(water::sulfate_chloride_balance(ratio)) as i32,
        color_min_srm: out("color_min_srm", range.min_srm)?,
        color_max_srm: out("color_max_srm", range.max_srm)?,
    })
}

fn balance_wire(b: water::SulfateChlorideBalance) -> WireBalance {
    use water::SulfateChlorideBalance as B;
    match b {
        B::OutOfRange => WireBalance::OutOfRange,
        B::ExtremelyMalty => WireBalance::ExtremelyMalty,
        B::VeryMalty => WireBalance::VeryMalty,
        B::Malty => WireBalance::Malty,
        B::Balanced => WireBalance::Balanced,
        B::SlightlyBitter => WireBalance::SlightlyBitter,
        B::Bitter => WireBalance::Bitter,
        B::VeryBitter => WireBalance::VeryBitter,
        B::ExtremelyBitter => WireBalance::ExtremelyBitter,
    }
}

/// A salt addition with its engine misc-use code.
#[derive(Debug, Clone, Copy)]
struct Agent {
    salt: Salt,
    grams: f64,
    use_code: u8,
}

fn agents_in(r: &CalculateWaterRequest) -> Result<Vec<Agent>, Status> {
    let mut v = Vec::with_capacity(r.salts.len());
    for (i, s) in r.salts.iter().enumerate() {
        let salt = salt_of(&format!("salts[{i}].salt"), s.salt)?;
        let grams = non_negative(&format!("salts[{i}].amount_g"), s.amount_g)?;
        let usage = engine_code(&format!("salts[{i}].usage"), s.usage, MiscUse::from_code)?;
        let use_code = u8::try_from(usage.code()).map_err(|_| {
            bad_request(
                "unknown enum value",
                &format!("salts[{i}].usage"),
                "unknown",
            )
        })?;
        v.push(Agent {
            salt,
            grams,
            use_code,
        });
    }
    Ok(v)
}

/// Ion increase of the salts of one use, in their reference volume (spec §2, §3.2).
fn adjustment(
    kind: RecipeType,
    vol: &Volumes,
    agents: &[Agent],
    ignore: bool,
    use_code: u8,
) -> Ions {
    let grams: Vec<(Salt, f64)> = agents
        .iter()
        .filter(|a| a.use_code == use_code)
        .map(|a| (a.salt, a.grams))
        .collect();
    let v = water::agent_volume(&AgentVolumeInput {
        rtype: kind,
        use_: use_code,
        mash_tot_water_l: vol.mash_water,
        sparge_vol_l: vol.sparge,
        boil_vol_l: vol.boil,
        volume_l: vol.batch,
        final_vol_l: vol.batch,
    });
    water::agents_ppm(&grams, v, ignore)
}

/// Engine misc-use codes of the profiles (spec §3.2).
const USE_BOIL: u8 = 0;
const USE_MASH: u8 = 1;
const USE_PRIMARY: u8 = 2;
const USE_SPARGE: u8 = 5;

/// A mash grain with validated figures.
#[derive(Debug, Clone, Copy)]
struct Grain {
    kg: f64,
    srm: f64,
    kind: GrainType,
    acid_malt: bool,
    acid_pct: f64,
}

fn grains_in(field: &str, grains: &[MashGrain]) -> Result<Vec<Grain>, Status> {
    let mut v = Vec::with_capacity(grains.len());
    for (i, g) in grains.iter().enumerate() {
        v.push(Grain {
            kg: non_negative(&format!("{field}[{i}].amount_kg"), g.amount_kg)?,
            srm: non_negative(&format!("{field}[{i}].color_srm"), g.color_srm)?,
            kind: engine_code(&format!("{field}[{i}].kind"), g.kind, GrainType::from_code)?,
            acid_malt: g.acid_malt,
            acid_pct: non_negative(&format!("{field}[{i}].acid_pct"), g.acid_pct)?,
        });
    }
    Ok(v)
}

/// Mash pH figures of `CalculateWater` (spec §5).
fn mash_ph_of(
    p: &crate::proto::MashPhInput,
    model: MashModel,
    ours: bool,
    ra_ppm: f64,
) -> Result<MashPhResult, Status> {
    let grains = grains_in("mash_ph.grains", &p.grains)?;
    let t = non_negative("mash_ph.mash_thickness_l_per_kg", p.mash_thickness_l_per_kg)?;
    let target = finite("mash_ph.target_ph", p.target_ph)?;
    let w_kg: f64 = grains.iter().map(|g| g.kg).sum();
    let slope = water::mash_ra_slope_checked(model, t, w_kg).map_err(|e| from_engine(&e))?;
    let bill: Vec<(f64, f64, GrainType)> = grains.iter().map(|g| (g.kg, g.srm, g.kind)).collect();
    let grist = water::grist_ph_typed(&bill, ours).ph;
    let estimated = water::est_mash_ph(grist, ra_ppm, slope);

    let mut acid_ra = 0.0;
    for (i, a) in p.acids.iter().enumerate() {
        let acid = engine_code(&format!("mash_ph.acids[{i}].acid"), a.acid, Acid::from_code)?;
        let vol_ml = non_negative(&format!("mash_ph.acids[{i}].volume_ml"), a.volume_ml)?;
        let conc = non_negative(
            &format!("mash_ph.acids[{i}].concentration_pct"),
            a.concentration_pct,
        )?;
        acid_ra += water::misc_acid_ra(&MiscAcidRaInput {
            acid,
            conc_pct: conc,
            // mL to L is a metric prefix, no unit system change.
            vol_l: vol_ml / 1000.0,
            f: water::dissociation_fraction(acid, model, target, ours),
            w_kg,
            t_l_per_kg: t,
        })
        .map_err(|e| from_engine(&e))?;
    }
    let malt_f = water::dissociation_fraction(Acid::AcidMalt, model, target, ours);
    for g in grains.iter().filter(|g| g.acid_malt) {
        acid_ra += water::acid_malt_ra(&AcidMaltRaInput {
            acid_pct: g.acid_pct,
            malt_kg: g.kg,
            f: malt_f,
            w_kg,
            t_l_per_kg: t,
        })
        .map_err(|e| from_engine(&e))?;
    }
    // Spec §5.5: pH_grist + m (RA/50 + MiscAcidRA), composed from the engine terms.
    let adjusted = water::est_mash_ph(grist, 50.0f64.mul_add(acid_ra, ra_ppm), slope);
    Ok(MashPhResult {
        grist_ph: out("grist_ph", grist)?,
        ra_slope: out("ra_slope", slope)?,
        estimated_ph: out("estimated_ph", estimated)?,
        adjusted_ph: out("adjusted_ph", adjusted)?,
        grain_kg: out("grain_kg", w_kg)?,
    })
}

#[tonic::async_trait]
impl WaterService for WaterHandler {
    async fn calculate_water(
        &self,
        request: Request<CalculateWaterRequest>,
    ) -> Result<Response<CalculateWaterResponse>, Status> {
        let r = request.into_inner();
        let kind = engine_code("kind", r.kind, RecipeType::from_code)?;
        let model = engine_code("model", r.model, MashModel::from_code)?;
        let ours = !r.beersmith_compatible;
        let (waters, ph_waters) = waters_in(&r.waters)?;
        let agents = agents_in(&r)?;
        let vol = volumes_in(r.volumes.as_ref())?;

        let base = water::base_profile(&waters);
        let adj = |use_code| adjustment(kind, &vol, &agents, r.ignore_salts, use_code);
        let mash = add_ions(&base, &adj(USE_MASH));
        let sparge = add_ions(&base, &adj(USE_SPARGE));
        let total = water::total_water_ion(&TotalWaterIonInput {
            mash: &mash,
            sparge: &sparge,
            vm_l: vol.mash_water,
            vs_l: vol.sparge,
            has_mash: is_mash_recipe(kind),
        });
        let k_top = water::top_up_factor(vol.mash_available, vol.sparge, vol.kettle_top_up);
        let boil = water::boil_ion(&total, k_top, &adj(USE_BOIL));
        let s_boil = boil_scale_of(&vol)?;
        let beer = water::beer_ion(&boil, s_boil, &adj(USE_PRIMARY));

        let mash_stats = stats_of(&mash)?;
        let mash_ph = match r.mash_ph.as_ref() {
            Some(p) => Some(mash_ph_of(
                p,
                model,
                ours,
                mash_stats.residual_alkalinity_ppm,
            )?),
            None => None,
        };
        let all_salts: Vec<(Salt, f64)> = agents.iter().map(|a| (a.salt, a.grams)).collect();
        let warnings = water::warnings(model, &all_salts)
            .into_iter()
            .map(|w| match w {
                water::Warning::BwModelUnverified => WaterWarning::BwModelUnverified as i32,
                water::Warning::ChalkSolubility => WaterWarning::ChalkSolubility as i32,
            })
            .collect();

        Ok(Response::new(CalculateWaterResponse {
            base: Some(ions_out("base", &base)?),
            water_ph: out("water_ph", water::water_ph(&ph_waters))?,
            mash: Some(ions_out("mash", &mash)?),
            sparge: Some(ions_out("sparge", &sparge)?),
            mash_and_sparge: Some(ions_out("mash_and_sparge", &total)?),
            boil: Some(ions_out("boil", &boil)?),
            beer: Some(ions_out("beer", &beer)?),
            boil_scale: out("boil_scale", s_boil)?,
            mash_stats: Some(mash_stats),
            mash_ph,
            warnings,
        }))
    }

    async fn adjust_acid(
        &self,
        request: Request<AdjustAcidRequest>,
    ) -> Result<Response<AdjustAcidResponse>, Status> {
        let r = request.into_inner();
        let model = engine_code("model", r.model, MashModel::from_code)?;
        let acid = engine_code("acid", r.acid, Acid::from_code)?;
        let ours = !r.beersmith_compatible;
        let conc = non_negative("concentration_pct", r.concentration_pct)?;
        let mash_ph = finite("mash_ph", r.mash_ph)?;
        let target = finite("target_ph", r.target_ph)?;
        let sparge_target = finite("sparge_target_ph", r.sparge_target_ph)?;
        let grain_kg = non_negative("grain_kg", r.grain_kg)?;
        let t = non_negative("mash_thickness_l_per_kg", r.mash_thickness_l_per_kg)?;
        let ra = finite("residual_alkalinity_ppm", r.residual_alkalinity_ppm)?;
        let hco3 = non_negative("sparge_bicarbonate_ppm", r.sparge_bicarbonate_ppm)?;
        let ph_w = finite("sparge_water_ph", r.sparge_water_ph)?;
        let grains = grains_in("grains", &r.grains)?;
        let sparge_l = match r.sparge_volume {
            Some(adjust_acid_request::SpargeVolume::SpargeL(v)) => non_negative("sparge_l", v)?,
            Some(adjust_acid_request::SpargeVolume::TotalWaterL(v)) => {
                water::sparge_volume_l(non_negative("total_water_l", v)?, grain_kg, t)
            }
            None => 0.0,
        };

        let (mash_amount, resulting_ph, converged) = if acid == Acid::AcidMalt {
            let bill: Vec<(f64, f64)> = grains.iter().map(|g| (g.kg, g.srm)).collect();
            let m = water::acid_malt_amount(&AcidMaltInput {
                bill: &bill,
                acid_pct: conc,
                model,
                ra_ppm: ra,
                t_l_per_kg: t,
                target_ph: target,
                ours,
            })
            .map_err(|e| from_engine(&e))?;
            (m.total_g, Some(m.ph), Some(m.converged))
        } else {
            let amount = water::mash_acid_amount(&MashAcidAmountInput {
                est_ph: mash_ph,
                target_ph: target,
                model,
                t_l_per_kg: t,
                w_kg: grain_kg,
                acid,
                conc_pct: conc,
                ours,
            })
            .map_err(|e| from_engine(&e))?;
            (amount, None, None)
        };
        let sparge_acid = water::sparge_acid_amount(&SpargeAcidInput {
            model,
            ours,
            hco3_ppm: hco3,
            ph_w,
            ph_t: if ours { sparge_target } else { target },
            acid,
            conc_pct: conc,
            mash_dph: mash_ph - target,
            v_sp_l: sparge_l,
        })
        .map_err(|e| from_engine(&e))?;

        Ok(Response::new(AdjustAcidResponse {
            mash_amount: out("mash_amount", mash_amount)?,
            resulting_ph: resulting_ph.map(|v| out("resulting_ph", v)).transpose()?,
            converged,
            sparge_acid_ml: out("sparge_acid_ml", sparge_acid)?,
            sparge_l: out("sparge_l", sparge_l)?,
        }))
    }

    async fn match_profile(
        &self,
        request: Request<MatchProfileRequest>,
    ) -> Result<Response<MatchProfileResponse>, Status> {
        let r = request.into_inner();
        let kind = engine_code("kind", r.kind, RecipeType::from_code)?;
        let (waters, _) = waters_in(&r.waters)?;
        let target = ions_in("target", r.target.as_ref())?;
        let vol = volumes_in(r.volumes.as_ref())?;
        let excluded = excluded_in("excluded", &r.excluded)?;

        let mash_recipe = is_mash_recipe(kind);
        let base = water::base_profile(&waters);
        // S_boil matters only for extract-type recipes with boil ingredients (spec §7.1).
        let s_boil = if !mash_recipe && vol.has_boil_ing {
            boil_scale_of(&vol)?
        } else {
            1.0
        };
        let s = water::match_scale(mash_recipe, vol.has_boil_ing, s_boil);
        let m = water::water_match(&base, &target, s, &excluded);

        // Mash-less recipes dissolve everything in the batch volume (spec §7.2).
        let (mash_l, sparge_l) = if mash_recipe {
            (vol.mash_water, vol.sparge)
        } else {
            (vol.batch, 0.0)
        };
        let g = water::salt_grams(
            &m.grams_per_gal,
            convert::l_to_gal(mash_l),
            convert::l_to_gal(sparge_l),
            r.combine_salts,
        );
        let best = water::best_fit_profile(&base, s, &m.grams_per_gal);
        Ok(Response::new(MatchProfileResponse {
            mash: salt_amounts("mash", &g.mash_g)?,
            sparge: salt_amounts("sparge", &g.sparge_g)?,
            best_fit: Some(ions_out("best_fit", &best)?),
            residual_ppm: out("residual_ppm", m.residual_norm)?,
        }))
    }

    async fn profile_tool(
        &self,
        request: Request<ProfileToolRequest>,
    ) -> Result<Response<ProfileToolResponse>, Status> {
        let r = request.into_inner();
        let base = ions_in("base", r.base.as_ref())?;
        let base_l = non_negative("base_l", r.base_l)?;
        let dilution = ions_in("dilution", r.dilution.as_ref())?;
        let dilution_l = non_negative("dilution_l", r.dilution_l)?;
        let target = ions_in("target", r.target.as_ref())?;
        let excluded = excluded_in("excluded", &r.excluded)?;

        let mix = water::profile_mix(base_l, &base, dilution_l, &dilution);
        let m = water::water_match(&mix, &target, 1.0, &excluded);
        let grams = water::wp_grams(&m.grams_per_gal, convert::l_to_gal(base_l));
        let best = water::best_fit_profile(&mix, 1.0, &m.grams_per_gal);
        Ok(Response::new(ProfileToolResponse {
            mix: Some(ions_out("mix", &mix)?),
            additions: salt_amounts("additions", &grams)?,
            best_fit: Some(ions_out("best_fit", &best)?),
            residual_ppm: out("residual_ppm", m.residual_norm)?,
        }))
    }

    async fn describe_profile(
        &self,
        request: Request<DescribeProfileRequest>,
    ) -> Result<Response<DescribeProfileResponse>, Status> {
        let r = request.into_inner();
        let ions = ions_in("ions", r.ions.as_ref())?;
        Ok(Response::new(DescribeProfileResponse {
            stats: Some(stats_of(&ions)?),
        }))
    }
}

/// Builds the tonic server wrapper for the water handler.
#[must_use]
pub fn server() -> WaterServiceServer<WaterHandler> {
    WaterServiceServer::new(WaterHandler)
}

#[cfg(test)]
#[allow(
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::many_single_char_names,
    clippy::type_complexity,
    clippy::similar_names,
    clippy::unnecessary_wraps,
    clippy::ref_option,
    clippy::doc_markdown
)]
mod tests {
    use super::*;
    use crate::proto::{
        AcidKind, FermentableKind, LiquidAcid, MashPhInput, MiscUse as WireUse, PhModel,
        RecipeKind, SaltAddition, WaterSalt,
    };
    use tonic::Code;
    use tonic_types::StatusExt;

    // Expected numbers come from `formuller/testler/vektorler/03.json`; the comments give the
    // case index in `vakalar`.

    fn field_of(s: &Status) -> String {
        s.get_error_details()
            .bad_request()
            .unwrap()
            .field_violations[0]
            .field
            .clone()
    }

    /// Tolerance of the vectors: 1e-9 relative with a 1e-12 floor around zero.
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-12 + 1e-9 * b.abs(), "{a} != {b}");
    }

    fn close_all(a: &Ions, b: &Ions) {
        for (x, y) in a.iter().zip(b) {
            close(*x, *y);
        }
    }

    fn ip(a: Ions) -> Option<IonProfile> {
        Some(IonProfile {
            calcium_ppm: a[0],
            magnesium_ppm: a[1],
            sodium_ppm: a[2],
            sulfate_ppm: a[3],
            chloride_ppm: a[4],
            bicarbonate_ppm: a[5],
        })
    }

    fn arr(p: &Option<IonProfile>) -> Ions {
        let p = p.as_ref().unwrap();
        [
            p.calcium_ppm,
            p.magnesium_ppm,
            p.sodium_ppm,
            p.sulfate_ppm,
            p.chloride_ppm,
            p.bicarbonate_ppm,
        ]
    }

    fn water_of(amount_l: f64, ions: Ions) -> SourceWater {
        SourceWater {
            amount_l,
            ions: ip(ions),
            ph: 7.0,
        }
    }

    /// A water whose mash RA is `ra` (no other ions).
    fn ra_water(ra: f64) -> SourceWater {
        if ra >= 0.0 {
            water_of(1.0, [0.0, 0.0, 0.0, 0.0, 0.0, ra * 61.0 / 50.0])
        } else {
            water_of(1.0, [-ra * 1.4, 0.0, 0.0, 0.0, 0.0, 0.0])
        }
    }

    fn calc_req() -> CalculateWaterRequest {
        CalculateWaterRequest {
            kind: RecipeKind::AllGrain as i32,
            model: PhModel::Mph3 as i32,
            beersmith_compatible: true,
            volumes: Some(WaterVolumes::default()),
            ..Default::default()
        }
    }

    async fn calc(r: CalculateWaterRequest) -> Result<CalculateWaterResponse, Status> {
        WaterHandler
            .calculate_water(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    fn salt(salt: WaterSalt, amount_g: f64, usage: WireUse) -> SaltAddition {
        SaltAddition {
            salt: salt as i32,
            amount_g,
            usage: usage as i32,
        }
    }

    /// Case 733: 5 g gypsum + 3 g CaCl2 in 20 L.
    const AGENTS_733: Ions = [
        112.33266249247156,
        0.0,
        0.0,
        139.4536237578253,
        95.80744735081652,
        0.0,
    ];

    fn salts_733(usage: WireUse) -> Vec<SaltAddition> {
        vec![
            salt(WaterSalt::Gypsum, 5.0, usage),
            salt(WaterSalt::CalciumChloride, 3.0, usage),
        ]
    }

    fn vols(f: impl FnOnce(&mut WaterVolumes)) -> Option<WaterVolumes> {
        let mut v = WaterVolumes::default();
        f(&mut v);
        Some(v)
    }

    fn grain(kg: f64, srm: f64, engine_kind: i32) -> MashGrain {
        MashGrain {
            amount_kg: kg,
            color_srm: srm,
            kind: engine_kind + 1,
            acid_malt: false,
            acid_pct: 0.0,
        }
    }

    fn mash_req(grains: Vec<MashGrain>, t: f64, target: f64, ra: f64) -> CalculateWaterRequest {
        CalculateWaterRequest {
            waters: vec![ra_water(ra)],
            mash_ph: Some(MashPhInput {
                grains,
                mash_thickness_l_per_kg: t,
                acids: vec![],
                target_ph: target,
            }),
            ..calc_req()
        }
    }

    #[tokio::test]
    async fn base_profile_and_water_ph_match_vectors() {
        // 724 base_profile, 729 water_ph.
        let r = calc(CalculateWaterRequest {
            waters: vec![
                water_of(10.0, [50.0, 10.0, 15.0, 40.0, 30.0, 150.0]),
                water_of(5.0, [20.0, 2.0, 5.0, 10.0, 8.0, 40.0]),
            ],
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(
            &arr(&r.base),
            &[
                40.0,
                7.333333333333333,
                11.666666666666666,
                30.0,
                22.666666666666668,
                113.33333333333333,
            ],
        );
        let r = calc(CalculateWaterRequest {
            waters: vec![
                SourceWater {
                    amount_l: 10.0,
                    ions: None,
                    ph: 7.2,
                },
                SourceWater {
                    amount_l: 5.0,
                    ions: None,
                    ph: 6.4,
                },
            ],
            ..calc_req()
        })
        .await
        .unwrap();
        close(r.water_ph, 6.933333333333334);
        // 731 water_ph without water is 7.0, 726 base is zero.
        let r = calc(calc_req()).await.unwrap();
        close(r.water_ph, 7.0);
        close_all(&arr(&r.base), &[0.0; 6]);
    }

    #[tokio::test]
    async fn salts_follow_their_use_volume() {
        // 733 agents_ppm: mash salts dissolve in the mash water.
        let r = calc(CalculateWaterRequest {
            salts: salts_733(WireUse::Mash),
            volumes: vols(|v| v.mash_water_l = 20.0),
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.mash), &AGENTS_733);
        close_all(&arr(&r.sparge), &[0.0; 6]);

        // Sparge salts in the sparge volume; the blend weights by volume.
        let r = calc(CalculateWaterRequest {
            salts: salts_733(WireUse::Sparge),
            volumes: vols(|v| {
                v.sparge_l = 20.0;
                v.mash_water_l = 20.0;
            }),
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.sparge), &AGENTS_733);
        close_all(&arr(&r.mash), &[0.0; 6]);
        let half: Ions = AGENTS_733.map(|x| x / 2.0);
        close_all(&arr(&r.mash_and_sparge), &half);

        // Boil salts (extract recipe: boil volume) reach the boil and the beer; primary
        // salts (batch volume) only the beer.
        let r = calc(CalculateWaterRequest {
            kind: RecipeKind::Extract as i32,
            salts: salts_733(WireUse::Boil),
            volumes: vols(|v| v.boil_l = 20.0),
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.boil), &AGENTS_733);
        close_all(&arr(&r.beer), &AGENTS_733);
        close_all(&arr(&r.mash), &[0.0; 6]);
        let r = calc(CalculateWaterRequest {
            salts: salts_733(WireUse::Primary),
            volumes: vols(|v| v.batch_l = 20.0),
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.boil), &[0.0; 6]);
        close_all(&arr(&r.beer), &AGENTS_733);

        // Other uses add to no profile (spec §3.3).
        let r = calc(CalculateWaterRequest {
            salts: salts_733(WireUse::Secondary),
            volumes: vols(|v| v.batch_l = 20.0),
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.beer), &[0.0; 6]);
    }

    #[tokio::test]
    async fn ignore_and_zero_volume_salts_add_nothing() {
        // 734 ignore = true, 735 volume 0.
        let r = calc(CalculateWaterRequest {
            salts: salts_733(WireUse::Mash),
            ignore_salts: true,
            volumes: vols(|v| v.mash_water_l = 20.0),
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.mash), &[0.0; 6]);
        let r = calc(CalculateWaterRequest {
            salts: vec![salt(WaterSalt::Gypsum, 5.0, WireUse::Mash)],
            ..calc_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.mash), &[0.0; 6]);
    }

    #[tokio::test]
    async fn kettle_top_up_and_boil_scale_match_vectors() {
        // 725 base, 758 top_up_factor (20, 10, 5) = 0.8571428571428571, 70 boil_scale
        // (28, 24, 1, 1, 2) = 1.1992753623188406.
        let base = [50.0, 10.0, 15.0, 40.0, 30.0, 150.0];
        let r = calc(CalculateWaterRequest {
            waters: vec![water_of(10.0, base)],
            volumes: vols(|v| {
                v.mash_water_l = 15.0;
                v.sparge_l = 10.0;
                v.mash_available_l = 20.0;
                v.kettle_top_up_l = 5.0;
                v.boil_l = 28.0;
                v.post_boil_l = 24.0;
                v.cool_loss_l = 1.0;
                v.trub_loss_l = 1.0;
                v.fermenter_top_up_l = 2.0;
                v.has_boil_ingredients = true;
            }),
            ..calc_req()
        })
        .await
        .unwrap();
        close(r.boil_scale, 1.1992753623188406);
        close_all(&arr(&r.mash_and_sparge), &base);
        close_all(&arr(&r.boil), &base.map(|x| x * 0.8571428571428571));
        close_all(
            &arr(&r.beer),
            &base.map(|x| x * 0.8571428571428571 * 1.1992753623188406),
        );
        // 69 without top-up and 75 without boil ingredients.
        let mut v = WaterVolumes {
            boil_l: 28.0,
            post_boil_l: 24.0,
            cool_loss_l: 1.0,
            trub_loss_l: 1.0,
            has_boil_ingredients: true,
            ..Default::default()
        };
        let r = calc(CalculateWaterRequest {
            volumes: Some(v),
            ..calc_req()
        })
        .await
        .unwrap();
        close(r.boil_scale, 1.2173913043478262);
        v.has_boil_ingredients = false;
        let r = calc(CalculateWaterRequest {
            volumes: Some(v),
            ..calc_req()
        })
        .await
        .unwrap();
        close(r.boil_scale, 1.0);
    }

    #[tokio::test]
    async fn mash_stats_match_vectors() {
        // Ca 50, Mg 10, HCO3 150: 19 alkalinity, 22 hardness, 26 residual alkalinity.
        let r = calc(CalculateWaterRequest {
            waters: vec![water_of(10.0, [50.0, 10.0, 0.0, 40.0, 30.0, 150.0])],
            ..calc_req()
        })
        .await
        .unwrap();
        let s = r.mash_stats.unwrap();
        close(s.alkalinity_ppm, 122.95081967213115);
        close(s.hardness_ppm, 41.596638655462186);
        close(s.residual_alkalinity_ppm, 81.35418101666896);
        close(s.sulfate_chloride_ratio, 40.0 / 30.0);
        close(s.color_min_srm, 5.2 + 0.082 * 81.35418101666896);
        assert!(r.mash_ph.is_none());
    }

    #[tokio::test]
    async fn grist_and_slope_match_vectors() {
        // 124 / 125 grist_ph_typed (types 0, 3, 7, 3; own model differs), 372 / 373 slope.
        let bill = vec![
            grain(3.0, 3.0, 0),
            grain(1.0, 13.99, 3),
            grain(0.5, 60.0, 7),
            grain(0.25, 450.0, 3),
        ];
        for (compat, grist) in [(true, 5.536317263157894), (false, 5.541656265473684)] {
            let r = calc(CalculateWaterRequest {
                beersmith_compatible: compat,
                ..mash_req(bill.clone(), 3.0, 5.4, 0.0)
            })
            .await
            .unwrap();
            let p = r.mash_ph.unwrap();
            close(p.grist_ph, grist);
            close(p.grain_kg, 4.75);
        }
        for (model, slope) in [(PhModel::Mph3, 0.079), (PhModel::Bw, 0.16296408420000003)] {
            let r = calc(CalculateWaterRequest {
                model: model as i32,
                ..mash_req(vec![grain(4.9, 3.0, 0)], 3.0, 5.4, 0.0)
            })
            .await
            .unwrap();
            close(r.mash_ph.unwrap().ra_slope, slope);
        }
    }

    #[tokio::test]
    async fn adjusted_ph_with_acid_malt_matches_vectors() {
        // 378, 379 (BeerSmith) and 388 (own model): bill, acid malt g, pct 3, RA 1.1166.
        let bill = || {
            vec![
                grain(4.5, 2.0, 0),
                grain(0.3, 60.0, 0),
                grain(0.1, 350.0, 0),
            ]
        };
        let malt = |g: f64| MashGrain {
            acid_malt: true,
            acid_pct: 3.0,
            ..grain(g / 1000.0, 3.0, 0)
        };
        let mut with_malt = bill();
        with_malt.push(malt(62.78));
        for (grains, compat, ph, kg) in [
            (bill(), true, 5.623387738204081, 4.9),
            (with_malt.clone(), true, 5.513233842788814, 4.96278),
            (with_malt, false, 5.518082662335404, 4.96278),
        ] {
            let r = calc(CalculateWaterRequest {
                beersmith_compatible: compat,
                ..mash_req(grains, 3.0, 5.2, 1.1166)
            })
            .await
            .unwrap();
            let p = r.mash_ph.unwrap();
            close(p.adjusted_ph, ph);
            close(p.grain_kg, kg);
        }
    }

    #[tokio::test]
    async fn adjusted_ph_with_liquid_acid_matches_vectors() {
        // 417, 419, 426, 428, 429 adjusted_ph_liquid: grist pH 5.6 (one grain of the colour
        // that gives it), RA 189.87..., 4.9 kg at 3 L/kg.
        let ra = 189.87341772151944;
        let s = (5.72 - 5.6) / 0.0337;
        let colour = s + 1.9;
        for (acid, conc, vol_l, target, compat, expected) in [
            (AcidKind::Lactic, 88.0, 0.011033517344210462, 5.2, true, 5.2),
            (
                AcidKind::Phosphoric,
                85.0,
                0.008899543644331773,
                5.2,
                true,
                5.20299714804252,
            ),
            (
                AcidKind::Lactic,
                88.0,
                0.011537846391005394,
                5.2,
                false,
                5.199999999999999,
            ),
            (
                AcidKind::Phosphoric,
                85.0,
                0.008819550381032926,
                5.2,
                false,
                5.20299714804252,
            ),
            (
                AcidKind::Lactic,
                88.0,
                0.008108376757371711,
                5.4,
                false,
                5.4,
            ),
        ] {
            let mut req = mash_req(vec![grain(4.9, colour, 0)], 3.0, target, ra);
            req.beersmith_compatible = compat;
            req.mash_ph.as_mut().unwrap().acids = vec![LiquidAcid {
                acid: acid as i32,
                volume_ml: vol_l * 1000.0,
                concentration_pct: conc,
            }];
            let p = calc(req).await.unwrap().mash_ph.unwrap();
            close(p.grist_ph, 5.6);
            close(p.adjusted_ph, expected);
            assert!(p.estimated_ph > p.adjusted_ph);
        }
    }

    #[tokio::test]
    async fn warnings_match_vectors() {
        // 776, 777, 778, 779, 780, 781.
        let bw = WaterWarning::BwModelUnverified as i32;
        let chalk = WaterWarning::ChalkSolubility as i32;
        for (model, salts, expected) in [
            (PhModel::Mph3, vec![(WaterSalt::Gypsum, 5.0)], vec![]),
            (PhModel::Bw, vec![(WaterSalt::Chalk, 2.0)], vec![bw, chalk]),
            (PhModel::Bw, vec![], vec![bw]),
            (PhModel::Mph3, vec![(WaterSalt::Chalk, 1.0)], vec![chalk]),
            (PhModel::Mph3, vec![(WaterSalt::Chalk, 0.0)], vec![]),
            (
                PhModel::Bw,
                vec![(WaterSalt::Gypsum, 3.0), (WaterSalt::Chalk, 0.5)],
                vec![bw, chalk],
            ),
        ] {
            let r = calc(CalculateWaterRequest {
                model: model as i32,
                salts: salts
                    .into_iter()
                    .map(|(s, g)| salt(s, g, WireUse::Mash))
                    .collect(),
                volumes: vols(|v| v.mash_water_l = 15.0),
                ..calc_req()
            })
            .await
            .unwrap();
            assert_eq!(r.warnings, expected);
        }
        // Warnings see salts even when they are ignored for the profiles.
        let r = calc(CalculateWaterRequest {
            ignore_salts: true,
            salts: vec![salt(WaterSalt::Chalk, 1.0, WireUse::Mash)],
            ..calc_req()
        })
        .await
        .unwrap();
        assert_eq!(r.warnings, vec![WaterWarning::ChalkSolubility as i32]);
    }

    #[tokio::test]
    async fn calculate_water_rejects_bad_input() {
        type Edit = Box<dyn Fn(&mut CalculateWaterRequest)>;
        let cases: Vec<(&str, Edit)> = vec![
            ("kind", Box::new(|r| r.kind = 0)),
            ("kind", Box::new(|r| r.kind = 99)),
            ("model", Box::new(|r| r.model = 0)),
            (
                "waters[0].amount_l",
                Box::new(|r| r.waters = vec![water_of(f64::NAN, [0.0; 6])]),
            ),
            (
                "waters[0].amount_l",
                Box::new(|r| r.waters = vec![water_of(-1.0, [0.0; 6])]),
            ),
            (
                "waters[0].ions.sulfate_ppm",
                Box::new(|r| r.waters = vec![water_of(1.0, [0.0, 0.0, 0.0, -1.0, 0.0, 0.0])]),
            ),
            (
                "waters[0].ph",
                Box::new(|r| {
                    r.waters = vec![SourceWater {
                        ph: f64::INFINITY,
                        ..water_of(1.0, [0.0; 6])
                    }];
                }),
            ),
            (
                "salts[0].salt",
                Box::new(|r| r.salts = vec![SaltAddition::default()]),
            ),
            (
                "salts[0].amount_g",
                Box::new(|r| r.salts = vec![salt(WaterSalt::Gypsum, -1.0, WireUse::Mash)]),
            ),
            (
                "salts[0].usage",
                Box::new(|r| {
                    r.salts = vec![SaltAddition {
                        usage: 0,
                        ..salt(WaterSalt::Gypsum, 1.0, WireUse::Mash)
                    }];
                }),
            ),
            (
                "volumes.boil_l",
                Box::new(|r| r.volumes = vols(|v| v.boil_l = f64::NAN)),
            ),
            (
                "volumes.batch_l",
                Box::new(|r| r.volumes = vols(|v| v.batch_l = -2.0)),
            ),
            // 813: post-boil volume equal to the cooling loss divides by zero in S_boil.
            (
                "vp",
                Box::new(|r| {
                    r.volumes = vols(|v| {
                        v.boil_l = 28.0;
                        v.post_boil_l = 1.0;
                        v.cool_loss_l = 1.0;
                        v.has_boil_ingredients = true;
                    });
                }),
            ),
            // 376: no grain, 374: no thickness (spec §5.2).
            (
                "w_kg",
                Box::new(|r| *r = mash_req(vec![grain(0.0, 3.0, 0)], 3.0, 5.4, 0.0)),
            ),
            (
                "t_l_per_kg",
                Box::new(|r| *r = mash_req(vec![grain(4.9, 3.0, 0)], 0.0, 5.4, 0.0)),
            ),
            (
                "mash_ph.target_ph",
                Box::new(|r| *r = mash_req(vec![grain(4.9, 3.0, 0)], 3.0, f64::NAN, 0.0)),
            ),
            (
                "mash_ph.grains[0].kind",
                Box::new(|r| {
                    *r = mash_req(
                        vec![MashGrain {
                            kind: 0,
                            ..grain(4.9, 3.0, 0)
                        }],
                        3.0,
                        5.4,
                        0.0,
                    );
                }),
            ),
            (
                "mash_ph.grains[0].amount_kg",
                Box::new(|r| *r = mash_req(vec![grain(-1.0, 3.0, 0)], 3.0, 5.4, 0.0)),
            ),
            (
                "mash_ph.acids[0].acid",
                Box::new(|r| {
                    *r = mash_req(vec![grain(4.9, 3.0, 0)], 3.0, 5.4, 0.0);
                    r.mash_ph.as_mut().unwrap().acids = vec![LiquidAcid::default()];
                }),
            ),
        ];
        for (field, edit) in cases {
            let mut r = calc_req();
            edit(&mut r);
            let e = calc(r).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    fn acid_req() -> AdjustAcidRequest {
        AdjustAcidRequest {
            model: PhModel::Mph3 as i32,
            beersmith_compatible: true,
            acid: AcidKind::Lactic as i32,
            concentration_pct: 88.0,
            mash_ph: 5.623387738204081,
            target_ph: 5.4,
            sparge_target_ph: 6.0,
            grain_kg: 4.9,
            mash_thickness_l_per_kg: 3.0,
            sparge_bicarbonate_ppm: 150.0,
            sparge_water_ph: 7.5,
            ..Default::default()
        }
    }

    async fn adjust(r: AdjustAcidRequest) -> Result<AdjustAcidResponse, Status> {
        WaterHandler
            .adjust_acid(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn mash_acid_amount_matches_vectors() {
        // 341, 342, 343, 345, 353 mash_acid_amount.
        for (est, target, model, t, w, acid, conc, compat, expected) in [
            (
                5.623387738204081,
                5.4,
                PhModel::Mph3,
                3.0,
                4.9,
                AcidKind::Lactic,
                88.0,
                true,
                3.5210749770838143,
            ),
            (
                5.623387738204081,
                5.4,
                PhModel::Mph3,
                3.0,
                4.9,
                AcidKind::Lactic,
                88.0,
                false,
                3.6226238886716096,
            ),
            (
                5.623387738204081,
                5.4,
                PhModel::Bw,
                3.0,
                4.9,
                AcidKind::Lactic,
                88.0,
                true,
                1.7561371796121024,
            ),
            (
                5.623387738204081,
                5.4,
                PhModel::Mph3,
                3.0,
                4.9,
                AcidKind::Phosphoric,
                85.0,
                false,
                2.797885457878023,
            ),
            (
                5.9,
                5.2,
                PhModel::Bw,
                2.5,
                4.9,
                AcidKind::Phosphoric,
                75.0,
                true,
                5.149461308657343,
            ),
        ] {
            let r = adjust(AdjustAcidRequest {
                model: model as i32,
                beersmith_compatible: compat,
                acid: acid as i32,
                concentration_pct: conc,
                mash_ph: est,
                target_ph: target,
                grain_kg: w,
                mash_thickness_l_per_kg: t,
                ..acid_req()
            })
            .await
            .unwrap();
            close(r.mash_amount, expected);
            assert!(r.resulting_ph.is_none() && r.converged.is_none());
        }
        // At or below target the amount is 0; no acid is 0.
        let r = adjust(AdjustAcidRequest {
            mash_ph: 5.3,
            ..acid_req()
        })
        .await
        .unwrap();
        close(r.mash_amount, 0.0);
        let r = adjust(AdjustAcidRequest {
            acid: AcidKind::None as i32,
            ..acid_req()
        })
        .await
        .unwrap();
        close(r.mash_amount, 0.0);
    }

    #[tokio::test]
    async fn acid_malt_matches_vectors() {
        // 490, 491, 492 (not converged), 494 (already below target) acid_malt_amount_checked.
        let b1 = vec![
            grain(4.5, 2.0, 0),
            grain(0.3, 60.0, 0),
            grain(0.1, 350.0, 0),
        ];
        for (grains, pct, model, ra, t, target, compat, total, ph, converged) in [
            (
                b1.clone(),
                3.0,
                PhModel::Mph3,
                1.1166,
                3.0,
                5.4,
                true,
                128.87855134100954,
                5.400229424914829,
                true,
            ),
            (
                vec![grain(5.0, 4.0, 0), grain(0.4, 120.0, 0)],
                2.5,
                PhModel::Bw,
                40.0,
                2.5,
                5.2,
                false,
                161.81428152993,
                5.200678464020039,
                true,
            ),
            (
                b1,
                0.1,
                PhModel::Mph3,
                200.0,
                3.0,
                5.2,
                true,
                22214501.17569249,
                5.706680648299921,
                false,
            ),
            (
                vec![grain(1.0, 13.0, 0)],
                3.0,
                PhModel::Mph3,
                -20.0,
                3.0,
                5.4,
                true,
                0.0,
                5.31433,
                true,
            ),
        ] {
            let r = adjust(AdjustAcidRequest {
                model: model as i32,
                beersmith_compatible: compat,
                acid: AcidKind::AcidMalt as i32,
                concentration_pct: pct,
                residual_alkalinity_ppm: ra,
                mash_thickness_l_per_kg: t,
                target_ph: target,
                grains,
                ..acid_req()
            })
            .await
            .unwrap();
            close(r.mash_amount, total);
            close(r.resulting_ph.unwrap(), ph);
            assert_eq!(r.converged, Some(converged));
            close(r.sparge_acid_ml, 0.0);
        }
    }

    #[tokio::test]
    async fn sparge_acid_matches_vectors() {
        // sparge_acid_amount: model, own model, sparge target, acid, conc, mash dPH, sparge L.
        let cases = [
            (
                600,
                PhModel::Mph3,
                false,
                6.0,
                AcidKind::Lactic,
                88.0,
                0.2,
                15.3,
                2.145701440422508,
            ),
            (
                601,
                PhModel::Mph3,
                false,
                5.4,
                AcidKind::Lactic,
                88.0,
                0.2,
                15.3,
                2.857905752447038,
            ),
            (
                607,
                PhModel::Mph3,
                false,
                5.4,
                AcidKind::Phosphoric,
                88.0,
                0.2,
                15.3,
                2.192256055337766,
            ),
            (
                625,
                PhModel::Mph3,
                true,
                5.4,
                AcidKind::Lactic,
                88.0,
                0.2,
                15.3,
                2.9403286546772716,
            ),
            (
                630,
                PhModel::Mph3,
                true,
                6.0,
                AcidKind::Phosphoric,
                88.0,
                0.2,
                15.3,
                1.5539127985286034,
            ),
            (
                648,
                PhModel::Bw,
                false,
                6.0,
                AcidKind::Lactic,
                88.0,
                0.2,
                15.3,
                2.214966451793767,
            ),
            (
                655,
                PhModel::Bw,
                false,
                5.4,
                AcidKind::Phosphoric,
                88.0,
                0.2,
                15.3,
                2.1824554040873005,
            ),
            (
                673,
                PhModel::Bw,
                true,
                5.4,
                AcidKind::Lactic,
                88.0,
                0.2,
                15.3,
                2.971317585261659,
            ),
            (
                714,
                PhModel::Mph3,
                false,
                5.4,
                AcidKind::Lactic,
                0.5,
                0.2,
                15.3,
                303.2289979784018,
            ),
            (
                718,
                PhModel::Bw,
                true,
                5.4,
                AcidKind::Phosphoric,
                75.0,
                0.3,
                4.2,
                0.7659881016954401,
            ),
            (
                719,
                PhModel::Mph3,
                true,
                5.4,
                AcidKind::Lactic,
                50.0,
                0.05,
                30.0,
                10.964039608471259,
            ),
            (
                602,
                PhModel::Mph3,
                false,
                5.4,
                AcidKind::Lactic,
                88.0,
                -0.1,
                15.3,
                0.0,
            ),
            (
                594,
                PhModel::Mph3,
                false,
                6.0,
                AcidKind::None,
                88.0,
                0.2,
                15.3,
                0.0,
            ),
        ];
        for (case, model, ours, ph_t, acid, conc, dph, v_sp, expected) in cases {
            // BeerSmith mode has one target; the own model has a separate sparge target.
            let (target, sparge_target) = if ours { (5.4, ph_t) } else { (ph_t, 0.0) };
            let r = adjust(AdjustAcidRequest {
                model: model as i32,
                beersmith_compatible: !ours,
                acid: acid as i32,
                concentration_pct: conc,
                mash_ph: target + dph,
                target_ph: target,
                sparge_target_ph: sparge_target,
                sparge_volume: Some(adjust_acid_request::SpargeVolume::SpargeL(v_sp)),
                ..acid_req()
            })
            .await
            .unwrap();
            close(r.sparge_acid_ml, expected);
            close(r.sparge_l, v_sp);
            assert!(r.mash_amount.is_finite(), "case {case}");
        }
    }

    #[tokio::test]
    async fn sparge_volume_from_total_water_matches_vector() {
        // 722 sparge_volume_l (30 L total, 4.9 kg, 3 L/kg) = 15.3; with mash dPH 0.2 and
        // target 5.4 the sparge acid is that of case 601.
        let r = adjust(AdjustAcidRequest {
            mash_ph: 5.6,
            sparge_volume: Some(adjust_acid_request::SpargeVolume::TotalWaterL(30.0)),
            ..acid_req()
        })
        .await
        .unwrap();
        close(r.sparge_l, 15.299999999999999);
        close(r.sparge_acid_ml, 2.857905752447038);
    }

    #[tokio::test]
    async fn adjust_acid_rejects_bad_input() {
        type Edit = Box<dyn Fn(&mut AdjustAcidRequest)>;
        let cases: Vec<(&str, Edit)> = vec![
            ("model", Box::new(|r| r.model = 0)),
            ("acid", Box::new(|r| r.acid = 0)),
            ("acid", Box::new(|r| r.acid = 42)),
            ("mash_ph", Box::new(|r| r.mash_ph = f64::NAN)),
            ("target_ph", Box::new(|r| r.target_ph = f64::INFINITY)),
            (
                "concentration_pct",
                Box::new(|r| r.concentration_pct = -1.0),
            ),
            ("grain_kg", Box::new(|r| r.grain_kg = -4.9)),
            (
                "sparge_bicarbonate_ppm",
                Box::new(|r| r.sparge_bicarbonate_ppm = -1.0),
            ),
            (
                "sparge_water_ph",
                Box::new(|r| r.sparge_water_ph = f64::NAN),
            ),
            (
                "sparge_l",
                Box::new(|r| {
                    r.sparge_volume = Some(adjust_acid_request::SpargeVolume::SpargeL(-1.0));
                }),
            ),
            (
                "grains[0].kind",
                Box::new(|r| r.grains = vec![MashGrain::default()]),
            ),
            // Engine errors: acids without a mash amount, no strength, no thickness or grain.
            ("acid", Box::new(|r| r.acid = AcidKind::Citric as i32)),
            ("conc", Box::new(|r| r.concentration_pct = 0.0)),
            ("t_l_per_kg", Box::new(|r| r.mash_thickness_l_per_kg = 0.0)),
            ("w_kg", Box::new(|r| r.grain_kg = 0.0)),
            (
                "pct",
                Box::new(|r| {
                    r.acid = AcidKind::AcidMalt as i32;
                    r.concentration_pct = 0.0;
                    r.grains = vec![grain(5.0, 3.0, 0)];
                }),
            ),
            ("bill", Box::new(|r| r.acid = AcidKind::AcidMalt as i32)),
        ];
        for (field, edit) in cases {
            let mut r = acid_req();
            edit(&mut r);
            let e = adjust(r).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    fn dense(v: &[SaltAmount]) -> [f64; 8] {
        let mut a = [0.0; 8];
        for s in v {
            a[usize::try_from(s.salt).unwrap() - 1] = s.amount_g;
        }
        a
    }

    const BASE_789: Ions = [50.0, 10.0, 15.0, 40.0, 30.0, 150.0];
    const TARGET_789: Ions = [100.0, 15.0, 20.0, 250.0, 60.0, 150.0];

    fn match_req(base: Option<Ions>, target: Ions, excluded: &[WaterSalt]) -> MatchProfileRequest {
        MatchProfileRequest {
            kind: RecipeKind::AllGrain as i32,
            waters: base.map(|b| water_of(10.0, b)).into_iter().collect(),
            target: ip(target),
            volumes: vols(|v| {
                v.mash_water_l = 20.0;
                v.sparge_l = 10.0;
                v.batch_l = 23.0;
            }),
            excluded: excluded.iter().map(|s| *s as i32).collect(),
            combine_salts: false,
        }
    }

    async fn matched(r: MatchProfileRequest) -> Result<MatchProfileResponse, Status> {
        WaterHandler
            .match_profile(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    const NO_CHALK_LIME_MGCL: [WaterSalt; 3] = [
        WaterSalt::Chalk,
        WaterSalt::PicklingLime,
        WaterSalt::MagnesiumChloride,
    ];

    #[tokio::test]
    async fn match_profile_matches_vectors() {
        // 789, 790, 791, 794 water_match (s = 1) and 793 (no water).
        let t791 = [30.0, 5.0, 10.0, 60.0, 10.0, 40.0];
        let b794 = [10.0, 3.0, 8.0, 12.0, 9.0, 35.0];
        let t794 = [140.0, 10.0, 25.0, 300.0, 100.0, 60.0];
        let t793 = [50.0, 10.0, 5.0, 100.0, 70.0, 20.0];
        let cases: Vec<(MatchProfileRequest, Ions, f64)> = vec![
            (
                match_req(Some(BASE_789), TARGET_789, &[]),
                [
                    108.43985842405158,
                    28.916878832730422,
                    27.356365751502864,
                    246.4789010770035,
                    55.229694440557616,
                    150.0,
                ],
                18.819691721406933,
            ),
            (
                match_req(Some(BASE_789), TARGET_789, &NO_CHALK_LIME_MGCL),
                [
                    108.43983131572742,
                    28.91683413254193,
                    27.35651861013551,
                    246.4789123865642,
                    55.229595317918836,
                    150.0,
                ],
                18.819729269683997,
            ),
            (
                match_req(Some(BASE_789), t791, &NO_CHALK_LIME_MGCL),
                [
                    50.0,
                    14.454939364491274,
                    15.0,
                    57.60781298641402,
                    30.0,
                    150.0,
                ],
                114.10573358553975,
            ),
            (
                match_req(
                    Some(b794),
                    t794,
                    &[WaterSalt::Chalk, WaterSalt::PicklingLime],
                ),
                [
                    144.6135357820485,
                    17.6076582825333,
                    29.021355471396685,
                    298.07523834439183,
                    97.39231367638584,
                    58.48486674610355,
                ],
                10.398694194642813,
            ),
            (
                match_req(None, t793, &[]),
                [
                    53.34739104564555,
                    15.519802688508882,
                    7.917729466559823,
                    98.60347242649455,
                    68.10797048903724,
                    18.90068187900759,
                ],
                7.544863089389141,
            ),
        ];
        for (req, profile, rnorm) in cases {
            let excluded: Vec<i32> = req.excluded.clone();
            let r = matched(req).await.unwrap();
            // The fitted profile is unique; the gram split is not (spec §7.1), so the
            // grams are checked against the engine fit in the next test.
            close_all(&arr(&r.best_fit), &profile);
            close(r.residual_ppm, rnorm);
            let g = dense(&r.mash);
            for s in excluded {
                close(g[usize::try_from(s).unwrap() - 1], 0.0);
            }
        }
    }

    #[tokio::test]
    async fn match_profile_grams_follow_volumes() {
        let excluded = [WaterSalt::Chalk, WaterSalt::PicklingLime];
        let ex: Vec<Salt> = vec![Salt::Chalk, Salt::Lime];
        let fit = water::water_match(&BASE_789, &TARGET_789, 1.0, &ex);
        let expect = |litres: f64| -> [f64; 8] {
            fit.grams_per_gal
                .map(|x| x * convert::l_to_gal(litres))
                .map(|g| if g >= MIN_SALT_G { g } else { 0.0 })
        };
        let r = matched(match_req(Some(BASE_789), TARGET_789, &excluded))
            .await
            .unwrap();
        for (a, b) in dense(&r.mash).iter().zip(expect(20.0)) {
            close(*a, b);
        }
        for (a, b) in dense(&r.sparge).iter().zip(expect(10.0)) {
            close(*a, b);
        }
        // 767 / 768: combining puts all salt in the mash water (mash + sparge amounts).
        let mut req = match_req(Some(BASE_789), TARGET_789, &excluded);
        req.volumes = vols(|v| {
            v.mash_water_l = convert::gal_to_l(3.88);
            v.sparge_l = convert::gal_to_l(4.04);
        });
        let sep = matched(req.clone()).await.unwrap();
        req.combine_salts = true;
        let comb = matched(req).await.unwrap();
        assert!(comb.sparge.is_empty());
        let (m, s, c) = (dense(&sep.mash), dense(&sep.sparge), dense(&comb.mash));
        for k in 0..8 {
            close(c[k], m[k] + s[k]);
        }
        // Mash-less recipes use the batch volume and no sparge (spec §7.2).
        let mut req = match_req(Some(BASE_789), TARGET_789, &excluded);
        req.kind = RecipeKind::Extract as i32;
        let r = matched(req).await.unwrap();
        assert!(r.sparge.is_empty());
        for (a, b) in dense(&r.mash).iter().zip(expect(23.0)) {
            close(*a, b);
        }
    }

    #[tokio::test]
    async fn match_profile_scales_extract_base_by_boil_scale() {
        // 69 boil_scale = 1.2173913043478262 and 765 match_scale: extract with boil
        // ingredients fits against S_boil x base; without them (766) s = 1 as case 790.
        let mut req = match_req(Some(BASE_789), TARGET_789, &NO_CHALK_LIME_MGCL);
        req.kind = RecipeKind::Extract as i32;
        req.volumes = vols(|v| {
            v.boil_l = 28.0;
            v.post_boil_l = 24.0;
            v.cool_loss_l = 1.0;
            v.trub_loss_l = 1.0;
            v.batch_l = 23.0;
            v.has_boil_ingredients = true;
        });
        let r = matched(req.clone()).await.unwrap();
        let ex = [Salt::Chalk, Salt::Lime, Salt::Mgcl2];
        let m = water::water_match(&BASE_789, &TARGET_789, 1.2173913043478262, &ex);
        close(r.residual_ppm, m.residual_norm);
        close_all(
            &arr(&r.best_fit),
            &water::best_fit_profile(&BASE_789, 1.2173913043478262, &m.grams_per_gal),
        );
        req.volumes.as_mut().unwrap().has_boil_ingredients = false;
        let r = matched(req).await.unwrap();
        close(r.residual_ppm, 18.819729269683997);
        // A mash recipe ignores S_boil (763).
        let mut req = match_req(Some(BASE_789), TARGET_789, &NO_CHALK_LIME_MGCL);
        req.volumes.as_mut().unwrap().has_boil_ingredients = true;
        req.volumes.as_mut().unwrap().boil_l = 28.0;
        let r = matched(req).await.unwrap();
        close(r.residual_ppm, 18.819729269683997);
    }

    #[tokio::test]
    async fn match_profile_rejects_bad_input() {
        type Edit = Box<dyn Fn(&mut MatchProfileRequest)>;
        let cases: Vec<(&str, Edit)> = vec![
            ("kind", Box::new(|r| r.kind = 0)),
            ("excluded[0]", Box::new(|r| r.excluded = vec![0])),
            ("excluded[1]", Box::new(|r| r.excluded = vec![1, 9])),
            (
                "target.calcium_ppm",
                Box::new(|r| r.target = ip([f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0])),
            ),
            (
                "target.bicarbonate_ppm",
                Box::new(|r| r.target = ip([0.0, 0.0, 0.0, 0.0, 0.0, -1.0])),
            ),
            (
                "waters[0].ions.sodium_ppm",
                Box::new(|r| r.waters = vec![water_of(1.0, [0.0, 0.0, -3.0, 0.0, 0.0, 0.0])]),
            ),
            (
                "volumes.mash_water_l",
                Box::new(|r| r.volumes = vols(|v| v.mash_water_l = f64::INFINITY)),
            ),
            // 813: S_boil divides by zero for an extract recipe with boil ingredients.
            (
                "vp",
                Box::new(|r| {
                    r.kind = RecipeKind::Extract as i32;
                    r.volumes = vols(|v| {
                        v.boil_l = 28.0;
                        v.post_boil_l = 1.0;
                        v.cool_loss_l = 1.0;
                        v.has_boil_ingredients = true;
                    });
                }),
            ),
        ];
        for (field, edit) in cases {
            let mut r = match_req(Some(BASE_789), TARGET_789, &[]);
            edit(&mut r);
            let e = matched(r).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    fn tool_req() -> ProfileToolRequest {
        ProfileToolRequest {
            base: ip(BASE_789),
            base_l: 10.0,
            dilution: ip([5.0, 1.0, 2.0, 3.0, 4.0, 10.0]),
            dilution_l: 0.0,
            target: ip(TARGET_789),
            excluded: vec![],
        }
    }

    async fn tool(r: ProfileToolRequest) -> Result<ProfileToolResponse, Status> {
        WaterHandler
            .profile_tool(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn profile_tool_matches_vectors() {
        // 772 profile_mix without dilution, then 789 / 790 water_match on that mix.
        let r = tool(tool_req()).await.unwrap();
        close_all(&arr(&r.mix), &BASE_789);
        close_all(
            &arr(&r.best_fit),
            &[
                108.43985842405158,
                28.916878832730422,
                27.356365751502864,
                246.4789010770035,
                55.229694440557616,
                150.0,
            ],
        );
        close(r.residual_ppm, 18.819691721406933);
        let r = tool(ProfileToolRequest {
            excluded: NO_CHALK_LIME_MGCL.iter().map(|s| *s as i32).collect(),
            ..tool_req()
        })
        .await
        .unwrap();
        close(r.residual_ppm, 18.819729269683997);

        // 771 profile_mix (10 L base + 5 L dilution), 773 (no base).
        let r = tool(ProfileToolRequest {
            dilution_l: 5.0,
            ..tool_req()
        })
        .await
        .unwrap();
        let mix = [
            35.0,
            7.0,
            10.666666666666666,
            27.666666666666668,
            21.333333333333332,
            103.33333333333333,
        ];
        close_all(&arr(&r.mix), &mix);
        // Salt amounts scale with the base volume only (spec §7.4), not the 15 L mix.
        let fit = water::water_match(&mix, &TARGET_789, 1.0, &[]);
        let expect = fit.grams_per_gal.map(|x| x * convert::l_to_gal(10.0));
        for (a, b) in dense(&r.additions).iter().zip(expect) {
            close(*a, if b >= MIN_SALT_G { b } else { 0.0 });
        }
        let r = tool(ProfileToolRequest {
            base_l: 0.0,
            dilution_l: 5.0,
            ..tool_req()
        })
        .await
        .unwrap();
        close_all(&arr(&r.mix), &[5.0, 1.0, 2.0, 3.0, 4.0, 10.0]);
        assert!(r.additions.is_empty());
    }

    #[tokio::test]
    async fn profile_tool_rejects_bad_input() {
        type Edit = Box<dyn Fn(&mut ProfileToolRequest)>;
        let cases: Vec<(&str, Edit)> = vec![
            ("base_l", Box::new(|r| r.base_l = f64::NAN)),
            ("dilution_l", Box::new(|r| r.dilution_l = -1.0)),
            (
                "base.chloride_ppm",
                Box::new(|r| r.base = ip([0.0, 0.0, 0.0, 0.0, -2.0, 0.0])),
            ),
            (
                "dilution.calcium_ppm",
                Box::new(|r| r.dilution = ip([f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0])),
            ),
            (
                "target.sulfate_ppm",
                Box::new(|r| r.target = ip([0.0, 0.0, 0.0, f64::NAN, 0.0, 0.0])),
            ),
            ("excluded[0]", Box::new(|r| r.excluded = vec![0])),
        ];
        for (field, edit) in cases {
            let mut r = tool_req();
            edit(&mut r);
            let e = tool(r).await.unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert_eq!(field_of(&e), field);
        }
    }

    async fn describe(ions: Ions) -> ProfileStats {
        WaterHandler
            .describe_profile(Request::new(DescribeProfileRequest { ions: ip(ions) }))
            .await
            .unwrap()
            .into_inner()
            .stats
            .unwrap()
    }

    #[tokio::test]
    async fn describe_profile_matches_vectors() {
        // 16-20 alkalinity.
        for (hco3, e) in [
            (0.0, 0.0),
            (61.0, 50.0),
            (122.5, 100.40983606557377),
            (150.0, 122.95081967213115),
            (300.5, 246.31147540983608),
        ] {
            close(
                describe([0.0, 0.0, 0.0, 0.0, 0.0, hco3])
                    .await
                    .alkalinity_ppm,
                e,
            );
        }
        // 22-25 hardness.
        for (ca, mg, e) in [
            (50.0, 10.0, 41.596638655462186),
            (112.33, 0.0, 80.2357142857143),
            (162.33, 10.0, 121.83235294117648),
            (5.0, 80.0, 50.63025210084034),
        ] {
            close(describe([ca, mg, 0.0, 0.0, 0.0, 0.0]).await.hardness_ppm, e);
        }
        // 26, 27, 29, 30, 31 residual alkalinity.
        for (ca, mg, hco3, e) in [
            (50.0, 10.0, 150.0, 81.35418101666896),
            (162.33, 10.0, 150.0, 1.1184667309546654),
            (10.0, 2.0, 20.0, 8.07411489185838),
            (200.0, 30.0, 40.0, -127.71731643477065),
            (80.0, 5.0, 400.0, 267.784818845571),
        ] {
            close(
                describe([ca, mg, 0.0, 0.0, 0.0, hco3])
                    .await
                    .residual_alkalinity_ppm,
                e,
            );
        }
        // 32, 33, 34 SO4/Cl ratio (34: no chloride gives 0).
        for (so4, cl, e) in [
            (179.4536237578253, 125.80744735081652, 1.4264149502804504),
            (100.0, 25.0, 4.0),
            (40.0, 0.0, 0.0),
        ] {
            close(
                describe([0.0, 0.0, 0.0, so4, cl, 0.0])
                    .await
                    .sulfate_chloride_ratio,
                e,
            );
        }
        // 60-68 colour range from the RA of crafted waters.
        for (ra, lo, hi) in [
            (1.1166, 5.2915612, 10.124311475409838),
            (81.354, 11.871028, 16.70114754098361),
            (0.0, 5.2, 10.032786885245903),
            (-50.0, 1.0999999999999996, 5.9344262295081975),
            (-200.0, 0.0, 0.0),
            (150.0, 17.5, 22.327868852459016),
            (-63.4, 0.0011999999999998678, 4.836065573770493),
            (-122.4, 0.0, 0.0),
            (-130.0, 0.0, 0.0),
        ] {
            let w = ra_water(ra).ions;
            let s = describe(arr(&w)).await;
            close(s.color_min_srm, lo);
            close(s.color_max_srm, hi);
        }
    }

    #[tokio::test]
    async fn describe_profile_balance_matches_label_vectors() {
        // 36-59 sulfate_chloride_label (35 has a negative ratio, not a valid input).
        use WireBalance as B;
        for (ratio, balance) in [
            (0.0, B::ExtremelyMalty),
            (0.2, B::ExtremelyMalty),
            (0.39999, B::ExtremelyMalty),
            (0.4, B::VeryMalty),
            (0.5, B::VeryMalty),
            (0.6, B::Malty),
            (0.7, B::Malty),
            (0.8, B::Balanced),
            (1.2, B::Balanced),
            (1.426, B::Balanced),
            (1.5, B::SlightlyBitter),
            (1.9, B::SlightlyBitter),
            (2.0, B::SlightlyBitter),
            (2.01, B::Bitter),
            (4.0, B::Bitter),
            (6.0, B::Bitter),
            (6.01, B::VeryBitter),
            (7.0, B::VeryBitter),
            (8.0, B::VeryBitter),
            (8.01, B::ExtremelyBitter),
            (12.0, B::ExtremelyBitter),
            (18.0, B::ExtremelyBitter),
            (18.01, B::OutOfRange),
            (40.0, B::OutOfRange),
        ] {
            let s = describe([0.0, 0.0, 0.0, ratio, 1.0, 0.0]).await;
            assert_eq!(s.balance, balance as i32, "ratio {ratio}");
        }
    }

    #[tokio::test]
    async fn describe_profile_rejects_bad_input() {
        for (field, ions) in [
            ("ions.calcium_ppm", [f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("ions.magnesium_ppm", [0.0, -1.0, 0.0, 0.0, 0.0, 0.0]),
            (
                "ions.bicarbonate_ppm",
                [0.0, 0.0, 0.0, 0.0, 0.0, f64::INFINITY],
            ),
        ] {
            let e = WaterHandler
                .describe_profile(Request::new(DescribeProfileRequest { ions: ip(ions) }))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
            assert_eq!(field_of(&e), field);
        }
        // An unset profile reads as all zero.
        let s = WaterHandler
            .describe_profile(Request::new(DescribeProfileRequest { ions: None }))
            .await
            .unwrap()
            .into_inner()
            .stats
            .unwrap();
        close(s.alkalinity_ppm, 0.0);
    }

    #[test]
    fn salt_values_follow_the_engine_order() {
        for (i, s) in Salt::ALL.into_iter().enumerate() {
            let wire = i32::try_from(i + 1).unwrap();
            assert_eq!(salt_wire(s), wire);
            assert_eq!(salt_of("salt", wire).unwrap(), s);
        }
        assert_eq!(WaterSalt::Gypsum as i32, salt_wire(Salt::Gypsum));
        assert_eq!(WaterSalt::PicklingLime as i32, salt_wire(Salt::Lime));
        assert!(salt_of("salt", 0).is_err());
        assert!(salt_of("salt", 9).is_err());
        assert_eq!(FermentableKind::Grain as i32, 1);
    }
}
