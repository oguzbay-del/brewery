//! `MashService` implementation: volume chain, strike temperature and sparge plan over
//! `engine::mash` (metric, so no unit conversion).

use engine::mash::{
    self, ABOVE_BOILING_C, DecoctVolInput, InfusionAmountInput, InfusionTempInput, RecalcBiabInput,
    RecalcMashInput, SpargeAction as EngineAction, SpargeParams, StepInput, StrikeTempInput,
    TotalWaterInput, Tun, VolumeChainInput,
};
use engine::recipe::{RecipeType, StepKind};
use tonic::{Request, Response, Status};

use crate::proto::{
    DecoctionVolumeRequest, DecoctionVolumeResponse, InfusionAmountRequest, InfusionAmountResponse,
    InfusionTempRequest, InfusionTempResponse, MashScheduleRequest, MashScheduleResponse,
    MashStepResult, MashTun, SpargeAction, SpargePlanRequest, SpargePlanResponse, SpargeStep,
    StrikeTempRequest, StrikeTempResponse, TotalWaterRequest, TotalWaterResponse,
    VolumeChainRequest, VolumeChainResponse,
    mash_service_server::{MashService, MashServiceServer},
};
use crate::recipe::engine_code;
use crate::status::bad_request;

/// Handler implementing the mash gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct MashHandler;

/// Requires a finite value.
fn finite(field: &str, v: f64) -> Result<f64, Status> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(bad_request(
            &format!("{field} must be finite"),
            field,
            "must be finite",
        ))
    }
}

/// Requires a finite, non-negative value.
fn non_negative(field: &str, v: f64) -> Result<f64, Status> {
    finite(field, v)?;
    if v < 0.0 {
        return Err(bad_request(
            &format!("{field} must not be negative"),
            field,
            "must not be negative",
        ));
    }
    Ok(v)
}

/// Fails when a computed output is not finite.
fn out(field: &str, v: f64) -> Result<f64, Status> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(bad_request(
            &format!("{field} result is not finite"),
            field,
            "result is not finite",
        ))
    }
}

/// Maps the optional wire tun to the engine tun; unset means the tun is ignored.
fn map_tun(tun: Option<MashTun>) -> Result<Tun, Status> {
    match tun {
        Some(t) => Ok(Tun {
            mass_kg: non_negative("tun.mass_kg", t.mass_kg)?,
            specific_heat_cal: non_negative("tun.specific_heat_cal", t.specific_heat_cal)?,
            volume_l: non_negative("tun.volume_l", t.volume_l)?,
        }),
        None => Ok(Tun::NONE),
    }
}

/// Maps one result of the engine's mash recalculation to the wire message.
fn map_step_result(r: &mash::StepResult) -> Result<MashStepResult, Status> {
    Ok(MashStepResult {
        infusion_l: out("infusion_l", r.infusion_l)?,
        tun_addition_l: out("tun_addition_l", r.tun_addition_l)?,
        total_infusion_l: out("total_infusion_l", r.tot_infusion_l)?,
        start_volume_l: out("start_volume_l", r.start_vol_l)?,
        start_c: out("start_c", r.start_c)?,
        infusion_c: r.infusion_c.map(|v| out("infusion_c", v)).transpose()?,
        decoction_l: out("decoction_l", r.decoction_l)?,
        above_boiling: r.above_boiling,
        no_water: r.no_water,
    })
}

#[tonic::async_trait]
impl MashService for MashHandler {
    async fn volume_chain(
        &self,
        request: Request<VolumeChainRequest>,
    ) -> Result<Response<VolumeChainResponse>, Status> {
        let r = request.into_inner();
        let v = mash::volume_chain(&VolumeChainInput {
            batch_l: non_negative("batch_l", r.batch_l)?,
            topup_ferm_l: non_negative("topup_ferm_l", r.topup_ferm_l)?,
            trub_l: non_negative("trub_l", r.trub_l)?,
            cool_pct: non_negative("cool_pct", r.cool_pct)?,
            boil_off_l: non_negative("boil_off_l", r.boil_off_l)?,
            boil_time_min: non_negative("boil_time_min", r.boil_time_min)?,
            rate_flag: r.boil_off_is_rate,
            fermenter_loss_l: non_negative("fermenter_loss_l", r.fermenter_loss_l)?,
        });
        Ok(Response::new(VolumeChainResponse {
            boil_off_l: out("boil_off_l", v.boil_off_l)?,
            preboil_l: out("preboil_l", v.preboil_l)?,
            postboil_l: out("postboil_l", v.postboil_l)?,
            cool_loss_l: out("cool_loss_l", v.cool_loss_l)?,
            cold_wort_l: out("cold_wort_l", v.cold_wort_l)?,
            pretrub_l: out("pretrub_l", v.pretrub_l)?,
            evap_pct_per_h: out("evap_pct_per_h", v.evap_pct)?,
            bottle_l: out("bottle_l", v.bottle_l)?,
        }))
    }

    async fn strike_temp(
        &self,
        request: Request<StrikeTempRequest>,
    ) -> Result<Response<StrikeTempResponse>, Status> {
        let r = request.into_inner();
        let tun = map_tun(r.tun)?;
        let tun_c = r.tun_c.map(|v| finite("tun_c", v)).transpose()?;
        let strike = mash::strike_temp(&StrikeTempInput {
            water_l: non_negative("water_l", r.water_l)?,
            step_c: finite("step_c", r.step_c)?,
            grain_kg: non_negative("grain_kg", r.grain_kg)?,
            grain_c: finite("grain_c", r.grain_c)?,
            tun,
            tun_c,
        })
        .ok_or_else(|| {
            bad_request(
                "water_l must be greater than zero",
                "water_l",
                "must be greater than zero",
            )
        })?;
        Ok(Response::new(StrikeTempResponse {
            strike_c: out("strike_c", strike)?,
        }))
    }

    async fn sparge_plan(
        &self,
        request: Request<SpargePlanRequest>,
    ) -> Result<Response<SpargePlanResponse>, Status> {
        let r = request.into_inner();
        // `sparge_l` may be negative: the engine clips it to zero (spec 05 §4.2).
        let steps = mash::sparge_plan(&SpargeParams {
            sparge_l: finite("sparge_l", r.sparge_l)?,
            absorb_l: non_negative("absorb_l", r.absorb_l)?,
            grain_kg: non_negative("grain_kg", r.grain_kg)?,
            tun_l: non_negative("tun_l", r.tun_l)?,
            fill_pct: non_negative("fill_pct", r.fill_pct)?,
            mash_water_l: non_negative("mash_water_l", r.mash_water_l)?,
            even: r.even,
            drain: r.drain_first,
        });
        let steps = steps
            .into_iter()
            .map(|s| {
                Ok(SpargeStep {
                    index: s.index,
                    volume_l: out("volume_l", s.volume_l)?,
                    action: match s.action {
                        EngineAction::Drain => SpargeAction::Drain,
                        EngineAction::Add => SpargeAction::Add,
                    } as i32,
                })
            })
            .collect::<Result<Vec<_>, Status>>()?;
        Ok(Response::new(SpargePlanResponse { steps }))
    }

    async fn mash_schedule(
        &self,
        request: Request<MashScheduleRequest>,
    ) -> Result<Response<MashScheduleResponse>, Status> {
        let r = request.into_inner();
        let steps = r
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Ok(StepInput {
                    kind: engine_code(&format!("steps[{i}].kind"), s.kind, StepKind::from_code)?,
                    infusion_l: non_negative(&format!("steps[{i}].infusion_l"), s.infusion_l)?,
                    step_c: finite(&format!("steps[{i}].temp_c"), s.temp_c)?,
                })
            })
            .collect::<Result<Vec<_>, Status>>()?;
        let grain_kg = non_negative("grain_kg", r.grain_kg)?;
        let grain_c = finite("grain_c", r.grain_c)?;
        let tun = map_tun(r.tun)?;
        let tun_c = r.tun_c.map(|v| finite("tun_c", v)).transpose()?;
        let boil_c = finite("boil_c", r.boil_c.unwrap_or(ABOVE_BOILING_C))?;
        let results = if r.biab {
            mash::recalc_biab(&RecalcBiabInput {
                steps: &steps,
                grain_kg,
                grain_c,
                biab_vol_l: non_negative("biab_volume_l", r.biab_volume_l)?,
                tun,
                tun_c,
                boil_c,
            })
        } else {
            mash::recalc_mash(&RecalcMashInput {
                steps: &steps,
                grain_kg,
                grain_c,
                tun,
                tun_c,
                deadspace_l: non_negative("deadspace_l", r.deadspace_l)?,
                boil_c,
            })
        };
        let water: Vec<(StepKind, f64, f64)> = steps
            .iter()
            .zip(&results)
            .map(|(s, x)| (s.kind, x.infusion_l, x.tun_addition_l))
            .collect();
        let mash_water_l = out("mash_water_l", mash::mash_tot_water(&water))?;
        let steps = results
            .iter()
            .map(map_step_result)
            .collect::<Result<Vec<_>, Status>>()?;
        Ok(Response::new(MashScheduleResponse {
            steps,
            mash_water_l,
        }))
    }

    async fn infusion_temp(
        &self,
        request: Request<InfusionTempRequest>,
    ) -> Result<Response<InfusionTempResponse>, Status> {
        let r = request.into_inner();
        let check = mash::infusion_temp_checked(&InfusionTempInput {
            added_l: non_negative("added_l", r.added_l)?,
            target_c: finite("target_c", r.target_c)?,
            mash_water_l: non_negative("mash_water_l", r.mash_water_l)?,
            mash_c: finite("mash_c", r.mash_c)?,
            grain_kg: non_negative("grain_kg", r.grain_kg)?,
            tun: map_tun(r.tun)?,
        })
        .ok_or_else(|| {
            bad_request(
                "added_l must be greater than zero",
                "added_l",
                "must be greater than zero",
            )
        })?;
        Ok(Response::new(InfusionTempResponse {
            infusion_c: out("infusion_c", check.temp_c)?,
            above_boiling: check.above_boiling,
        }))
    }

    async fn infusion_amount(
        &self,
        request: Request<InfusionAmountRequest>,
    ) -> Result<Response<InfusionAmountResponse>, Status> {
        let r = request.into_inner();
        let added = mash::infusion_amount(&InfusionAmountInput {
            inf_c: finite("infusion_c", r.infusion_c)?,
            target_c: finite("target_c", r.target_c)?,
            mash_water_l: non_negative("mash_water_l", r.mash_water_l)?,
            mash_c: finite("mash_c", r.mash_c)?,
            grain_kg: non_negative("grain_kg", r.grain_kg)?,
            tun: map_tun(r.tun)?,
        });
        Ok(Response::new(InfusionAmountResponse {
            added_l: out("added_l", added)?,
        }))
    }

    async fn decoction_volume(
        &self,
        request: Request<DecoctionVolumeRequest>,
    ) -> Result<Response<DecoctionVolumeResponse>, Status> {
        let r = request.into_inner();
        let v = mash::decoct_vol(&DecoctVolInput {
            target_c: finite("target_c", r.target_c)?,
            mash_water_l: non_negative("mash_water_l", r.mash_water_l)?,
            mash_c: finite("mash_c", r.mash_c)?,
            grain_kg: non_negative("grain_kg", r.grain_kg)?,
            tun: map_tun(r.tun)?,
            boil_c: finite("boil_c", r.boil_c.unwrap_or(ABOVE_BOILING_C))?,
        });
        Ok(Response::new(DecoctionVolumeResponse {
            decoction_l: out("decoction_l", v)?,
        }))
    }

    async fn total_water(
        &self,
        request: Request<TotalWaterRequest>,
    ) -> Result<Response<TotalWaterResponse>, Status> {
        let r = request.into_inner();
        let v = mash::total_water(&TotalWaterInput {
            recipe_type: engine_code("kind", r.kind, RecipeType::from_code)?,
            has_boil_ing: r.has_boil_ingredients,
            mash_tot_water_l: non_negative("mash_water_l", r.mash_water_l)?,
            sparge_l: non_negative("sparge_l", r.sparge_l)?,
            topup_kettle_l: non_negative("topup_kettle_l", r.topup_kettle_l)?,
            topup_ferm_l: non_negative("topup_fermenter_l", r.topup_fermenter_l)?,
            batch_l: non_negative("batch_l", r.batch_l)?,
            v_preboil_l: non_negative("preboil_l", r.preboil_l)?,
        });
        Ok(Response::new(TotalWaterResponse {
            total_water_l: out("total_water_l", v)?,
        }))
    }
}

/// Builds the tonic server wrapper for the mash handler.
#[must_use]
pub fn server() -> MashServiceServer<MashHandler> {
    MashServiceServer::new(MashHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{MashStep, MashStepKind, MashTun, RecipeKind};
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

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    fn chain_req() -> VolumeChainRequest {
        VolumeChainRequest {
            batch_l: 23.0,
            topup_ferm_l: 3.0,
            trub_l: 1.0,
            cool_pct: 4.0,
            boil_off_l: 5.5,
            boil_off_is_rate: true,
            boil_time_min: 90.0,
            fermenter_loss_l: 1.0,
        }
    }

    fn strike_req() -> StrikeTempRequest {
        StrikeTempRequest {
            water_l: 15.0,
            step_c: 66.0,
            grain_kg: 5.0,
            grain_c: 20.0,
            tun: Some(MashTun {
                mass_kg: 4.0,
                specific_heat_cal: 0.12,
                volume_l: 40.0,
            }),
            tun_c: Some(20.0),
        }
    }

    fn sparge_req() -> SpargePlanRequest {
        SpargePlanRequest {
            sparge_l: 19.007_242_339_615_92,
            absorb_l: 5.007_242_339_615_919,
            grain_kg: 5.0,
            tun_l: 40.0,
            fill_pct: 95.0,
            mash_water_l: 16.0,
            even: true,
            drain_first: true,
        }
    }

    #[tokio::test]
    async fn volume_chain_matches_vector() {
        let r = MashHandler
            .volume_chain(Request::new(chain_req()))
            .await
            .unwrap()
            .into_inner();
        close(r.boil_off_l, 8.25);
        close(r.preboil_l, 30.125);
        close(r.postboil_l, 21.875);
        close(r.cool_loss_l, 0.875);
        close(r.cold_wort_l, 21.0);
        close(r.pretrub_l, 24.0);
        close(r.evap_pct_per_h, 18.257_261_410_788_38);
        close(r.bottle_l, 22.0);
    }

    #[tokio::test]
    async fn volume_chain_rejects_bad_input() {
        for (field, req) in [
            (
                "batch_l",
                VolumeChainRequest {
                    batch_l: f64::NAN,
                    ..chain_req()
                },
            ),
            (
                "trub_l",
                VolumeChainRequest {
                    trub_l: -1.0,
                    ..chain_req()
                },
            ),
            (
                "boil_time_min",
                VolumeChainRequest {
                    boil_time_min: f64::INFINITY,
                    ..chain_req()
                },
            ),
        ] {
            let e = MashHandler
                .volume_chain(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument, "{field}");
            assert!(e.message().contains(field), "{}", e.message());
        }
    }

    #[tokio::test]
    async fn strike_temp_matches_vectors() {
        let with_tun = MashHandler
            .strike_temp(Request::new(strike_req()))
            .await
            .unwrap()
            .into_inner();
        close(with_tun.strike_c, 72.666_626_666_666_67);
        // No tun, tun_c unset: grain temperature is used.
        let bare = MashHandler
            .strike_temp(Request::new(StrikeTempRequest {
                tun: None,
                tun_c: None,
                ..strike_req()
            }))
            .await
            .unwrap()
            .into_inner();
        close(bare.strike_c, 71.826_666_666_666_67);
        // Tun given, tun_c unset.
        let defaulted = MashHandler
            .strike_temp(Request::new(StrikeTempRequest {
                tun_c: None,
                ..strike_req()
            }))
            .await
            .unwrap()
            .into_inner();
        close(defaulted.strike_c, 72.666_626_666_666_67);
    }

    #[tokio::test]
    async fn strike_temp_rejects_bad_input() {
        let zero = MashHandler
            .strike_temp(Request::new(StrikeTempRequest {
                water_l: 0.0,
                ..strike_req()
            }))
            .await
            .unwrap_err();
        assert_eq!(zero.code(), Code::InvalidArgument);
        assert!(zero.message().contains("water_l"));
        for req in [
            StrikeTempRequest {
                step_c: f64::NAN,
                ..strike_req()
            },
            StrikeTempRequest {
                grain_kg: -1.0,
                ..strike_req()
            },
            StrikeTempRequest {
                tun_c: Some(f64::NAN),
                ..strike_req()
            },
            StrikeTempRequest {
                tun: Some(MashTun {
                    mass_kg: f64::NAN,
                    specific_heat_cal: 0.12,
                    volume_l: 40.0,
                }),
                ..strike_req()
            },
        ] {
            let e = MashHandler
                .strike_temp(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }

    #[tokio::test]
    async fn sparge_plan_matches_vectors() {
        let r = MashHandler
            .sparge_plan(Request::new(sparge_req()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.steps.len(), 2);
        assert_eq!(r.steps[0].index, 0);
        close(r.steps[0].volume_l, 10.992_757_660_384_08);
        assert_eq!(r.steps[0].action, SpargeAction::Drain as i32);
        assert_eq!(r.steps[1].index, 1);
        close(r.steps[1].volume_l, 19.007_242_339_615_92);
        assert_eq!(r.steps[1].action, SpargeAction::Add as i32);

        let none = MashHandler
            .sparge_plan(Request::new(SpargePlanRequest {
                sparge_l: 0.0,
                even: false,
                drain_first: false,
                ..sparge_req()
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(none.steps.is_empty());
    }

    #[tokio::test]
    async fn sparge_plan_rejects_bad_input() {
        for req in [
            SpargePlanRequest {
                sparge_l: f64::NAN,
                ..sparge_req()
            },
            SpargePlanRequest {
                tun_l: -1.0,
                ..sparge_req()
            },
            SpargePlanRequest {
                mash_water_l: f64::NEG_INFINITY,
                ..sparge_req()
            },
        ] {
            let e = MashHandler
                .sparge_plan(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }

    fn step(kind: MashStepKind, temp_c: f64, infusion_l: f64) -> MashStep {
        MashStep {
            kind: kind as i32,
            temp_c,
            infusion_l,
        }
    }

    fn tun40() -> MashTun {
        MashTun {
            mass_kg: 4.0,
            specific_heat_cal: 0.12,
            volume_l: 40.0,
        }
    }

    /// Expected step: infusion, tun addition, total, start volume, start temp, infusion temp,
    /// decoction, above boiling, no water.
    type Want = (f64, f64, f64, f64, f64, Option<f64>, f64, bool, bool);

    fn check_steps(got: &[MashStepResult], want: &[Want]) {
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(want) {
            close(g.infusion_l, w.0);
            close(g.tun_addition_l, w.1);
            close(g.total_infusion_l, w.2);
            close(g.start_volume_l, w.3);
            close(g.start_c, w.4);
            assert_eq!(g.infusion_c.is_some(), w.5.is_some());
            if let (Some(a), Some(b)) = (g.infusion_c, w.5) {
                close(a, b);
            }
            close(g.decoction_l, w.6);
            assert_eq!(g.above_boiling, w.7);
            assert_eq!(g.no_water, w.8);
        }
    }

    fn schedule_req() -> MashScheduleRequest {
        MashScheduleRequest {
            steps: vec![
                step(MashStepKind::Infusion, 66.0, 15.0),
                step(MashStepKind::Infusion, 72.0, 4.0),
                step(MashStepKind::Infusion, 76.0, 3.0),
            ],
            grain_kg: 5.0,
            grain_c: 20.0,
            ..Default::default()
        }
    }

    // Vector 05.json recalc_mash #1 (no tun, no deadspace).
    #[tokio::test]
    async fn mash_schedule_recalc_mash_matches_vector() {
        let r = MashHandler
            .mash_schedule(Request::new(schedule_req()))
            .await
            .unwrap()
            .into_inner();
        check_steps(
            &r.steps,
            &[
                (
                    15.0,
                    0.0,
                    15.0,
                    0.0,
                    0.0,
                    Some(71.826_666_666_666_67),
                    0.0,
                    false,
                    false,
                ),
                (4.0, 0.0, 4.0, 15.0, 66.0, Some(97.35), 0.0, false, false),
                (
                    3.0,
                    0.0,
                    3.0,
                    19.0,
                    72.0,
                    Some(103.866_666_666_666_66),
                    0.0,
                    true,
                    false,
                ),
            ],
        );
        close(r.mash_water_l, 22.0);
    }

    // Vector 05.json recalc_mash with tun, deadspace, tun_c (case "m_tun 4, T_tun 20, deadspace 1").
    #[tokio::test]
    async fn mash_schedule_with_tun_and_deadspace_matches_vector() {
        let r = MashHandler
            .mash_schedule(Request::new(MashScheduleRequest {
                tun: Some(tun40()),
                tun_c: Some(20.0),
                deadspace_l: 1.0,
                ..schedule_req()
            }))
            .await
            .unwrap()
            .into_inner();
        check_steps(
            &r.steps,
            &[
                (
                    15.0,
                    1.0,
                    16.0,
                    0.0,
                    0.0,
                    Some(72.293_087_5),
                    0.0,
                    false,
                    false,
                ),
                (
                    4.0,
                    0.0,
                    4.0,
                    16.0,
                    66.0,
                    Some(99.373_35),
                    0.0,
                    false,
                    false,
                ),
                (3.0, 0.0, 3.0, 20.0, 72.0, Some(105.725_2), 0.0, true, false),
            ],
        );
        close(r.mash_water_l, 23.0);
    }

    // Vector 05.json recalc_mash with decoction and temperature steps, Tb 99 (boil_c).
    #[tokio::test]
    async fn mash_schedule_decoction_matches_vector() {
        let r = MashHandler
            .mash_schedule(Request::new(MashScheduleRequest {
                steps: vec![
                    step(MashStepKind::Infusion, 50.0, 15.0),
                    step(MashStepKind::Decoction, 66.0, 0.0),
                    step(MashStepKind::Infusion, 72.0, 3.0),
                    step(MashStepKind::Temperature, 78.0, 2.0),
                ],
                tun: Some(tun40()),
                boil_c: Some(99.0),
                ..schedule_req()
            }))
            .await
            .unwrap()
            .into_inner();
        check_steps(
            &r.steps,
            &[
                (15.0, 0.0, 15.0, 0.0, 0.0, Some(54.3478), 0.0, false, false),
                (
                    0.0,
                    0.0,
                    0.0,
                    15.0,
                    50.0,
                    Some(0.0),
                    6.059_082_989_977_056,
                    false,
                    false,
                ),
                (3.0, 0.0, 3.0, 15.0, 66.0, Some(106.4378), 0.0, true, false),
                (2.0, 0.0, 2.0, 18.0, 72.0, None, 0.0, false, false),
            ],
        );
        // Decoction excluded, temperature step counted: 15 + 3 + 2.
        close(r.mash_water_l, 20.0);
    }

    // Vector 05.json recalc_biab with a decoction step (biab_vol 20).
    #[tokio::test]
    async fn mash_schedule_biab_matches_vector() {
        let r = MashHandler
            .mash_schedule(Request::new(MashScheduleRequest {
                steps: vec![
                    step(MashStepKind::Temperature, 55.0, 0.0),
                    step(MashStepKind::Decoction, 66.0, 0.0),
                    step(MashStepKind::Infusion, 66.0, 10.0),
                    step(MashStepKind::Decoction, 72.0, 0.0),
                ],
                biab: true,
                biab_volume_l: 20.0,
                ..schedule_req()
            }))
            .await
            .unwrap()
            .into_inner();
        check_steps(
            &r.steps,
            &[
                (10.0, 0.0, 10.0, 0.0, 0.0, None, 0.0, false, false),
                (
                    0.0,
                    0.0,
                    0.0,
                    10.0,
                    55.0,
                    Some(0.0),
                    3.241_333_333_333_333,
                    false,
                    false,
                ),
                (10.0, 0.0, 10.0, 10.0, 66.0, Some(66.0), 0.0, false, false),
                (
                    0.0,
                    0.0,
                    0.0,
                    20.0,
                    66.0,
                    Some(0.0),
                    4.104_705_882_352_942,
                    false,
                    false,
                ),
            ],
        );
        close(r.mash_water_l, 20.0);
    }

    #[tokio::test]
    async fn mash_schedule_rejects_bad_input() {
        let mut unspecified = schedule_req();
        unspecified.steps[1].kind = MashStepKind::Unspecified as i32;
        let e = MashHandler
            .mash_schedule(Request::new(unspecified))
            .await
            .unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument);
        assert_eq!(field_of(&e), "steps[1].kind");
        let mut unknown = schedule_req();
        unknown.steps[0].kind = 99;
        let e = MashHandler
            .mash_schedule(Request::new(unknown))
            .await
            .unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument);
        let mut nan_temp = schedule_req();
        nan_temp.steps[2].temp_c = f64::NAN;
        let mut neg_inf = schedule_req();
        neg_inf.steps[0].infusion_l = -1.0;
        for req in [
            nan_temp,
            neg_inf,
            MashScheduleRequest {
                grain_kg: -1.0,
                ..schedule_req()
            },
            MashScheduleRequest {
                grain_c: f64::NAN,
                ..schedule_req()
            },
            MashScheduleRequest {
                tun_c: Some(f64::INFINITY),
                ..schedule_req()
            },
            MashScheduleRequest {
                boil_c: Some(f64::NAN),
                ..schedule_req()
            },
            MashScheduleRequest {
                deadspace_l: -1.0,
                ..schedule_req()
            },
            MashScheduleRequest {
                biab: true,
                biab_volume_l: f64::NAN,
                ..schedule_req()
            },
        ] {
            let e = MashHandler
                .mash_schedule(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }

    fn infusion_temp_req() -> InfusionTempRequest {
        InfusionTempRequest {
            added_l: 2.5,
            target_c: 78.0,
            mash_water_l: 10.0,
            mash_c: 63.0,
            grain_kg: 4.2,
            tun: Some(MashTun {
                mass_kg: 4.0,
                specific_heat_cal: 0.12,
                volume_l: 12.0,
            }),
        }
    }

    // Vector 05.json infusion_temp_checked cases.
    #[tokio::test]
    async fn infusion_temp_matches_vectors() {
        let r = MashHandler
            .infusion_temp(Request::new(infusion_temp_req()))
            .await
            .unwrap()
            .into_inner();
        close(r.infusion_c, 150.456_000_000_000_02);
        assert!(r.above_boiling);
        let plain = MashHandler
            .infusion_temp(Request::new(InfusionTempRequest {
                added_l: 4.0,
                target_c: 72.0,
                mash_water_l: 15.0,
                mash_c: 66.0,
                grain_kg: 5.0,
                tun: None,
            }))
            .await
            .unwrap()
            .into_inner();
        close(plain.infusion_c, 97.35);
        assert!(!plain.above_boiling);
    }

    #[tokio::test]
    async fn infusion_temp_rejects_bad_input() {
        // Vector cases Va = 0 and Va = -1 are None in the engine.
        let zero = MashHandler
            .infusion_temp(Request::new(InfusionTempRequest {
                added_l: 0.0,
                ..infusion_temp_req()
            }))
            .await
            .unwrap_err();
        assert_eq!(zero.code(), Code::InvalidArgument);
        assert!(zero.message().contains("added_l"));
        for req in [
            InfusionTempRequest {
                added_l: -1.0,
                ..infusion_temp_req()
            },
            InfusionTempRequest {
                target_c: f64::NAN,
                ..infusion_temp_req()
            },
            InfusionTempRequest {
                mash_water_l: -1.0,
                ..infusion_temp_req()
            },
            InfusionTempRequest {
                mash_c: f64::INFINITY,
                ..infusion_temp_req()
            },
        ] {
            let e = MashHandler
                .infusion_temp(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }

    fn infusion_amount_req() -> InfusionAmountRequest {
        InfusionAmountRequest {
            infusion_c: 100.0,
            target_c: 72.0,
            mash_water_l: 15.0,
            mash_c: 66.0,
            grain_kg: 5.0,
            tun: None,
        }
    }

    // Vector 05.json infusion_amount cases.
    #[tokio::test]
    async fn infusion_amount_matches_vectors() {
        let plain = MashHandler
            .infusion_amount(Request::new(infusion_amount_req()))
            .await
            .unwrap()
            .into_inner();
        close(plain.added_l, 3.621_428_571_428_571);
        let with_tun = MashHandler
            .infusion_amount(Request::new(InfusionAmountRequest {
                tun: Some(tun40()),
                ..infusion_amount_req()
            }))
            .await
            .unwrap()
            .into_inner();
        close(with_tun.added_l, 3.680_121_428_571_428_3);
        // Target outside the mash/infusion range: 0.
        let outside = MashHandler
            .infusion_amount(Request::new(InfusionAmountRequest {
                target_c: 105.0,
                ..infusion_amount_req()
            }))
            .await
            .unwrap()
            .into_inner();
        close(outside.added_l, 0.0);
    }

    #[tokio::test]
    async fn infusion_amount_rejects_bad_input() {
        for req in [
            InfusionAmountRequest {
                infusion_c: f64::NAN,
                ..infusion_amount_req()
            },
            InfusionAmountRequest {
                mash_water_l: -1.0,
                ..infusion_amount_req()
            },
            InfusionAmountRequest {
                grain_kg: f64::INFINITY,
                ..infusion_amount_req()
            },
        ] {
            let e = MashHandler
                .infusion_amount(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }

    fn decoction_req() -> DecoctionVolumeRequest {
        DecoctionVolumeRequest {
            target_c: 72.0,
            mash_water_l: 15.0,
            mash_c: 66.0,
            grain_kg: 5.0,
            tun: None,
            boil_c: None,
        }
    }

    // Vector 05.json decoct_vol cases.
    #[tokio::test]
    async fn decoction_volume_matches_vectors() {
        let plain = MashHandler
            .decoction_volume(Request::new(decoction_req()))
            .await
            .unwrap()
            .into_inner();
        close(plain.decoction_l, 3.222_352_941_176_471);
        let with_tun = MashHandler
            .decoction_volume(Request::new(DecoctionVolumeRequest {
                tun: Some(tun40()),
                ..decoction_req()
            }))
            .await
            .unwrap()
            .into_inner();
        close(with_tun.decoction_l, 3.274_577_939_436_130_5);
        let boil = MashHandler
            .decoction_volume(Request::new(DecoctionVolumeRequest {
                target_c: 78.0,
                mash_water_l: 12.5,
                mash_c: 62.0,
                grain_kg: 4.2,
                tun: Some(MashTun {
                    mass_kg: 4.0,
                    specific_heat_cal: 0.12,
                    volume_l: 12.0,
                }),
                boil_c: Some(99.0),
            }))
            .await
            .unwrap()
            .into_inner();
        close(boil.decoction_l, 6.813_968_107_494_555_5);
        // Mash already at boiling: nothing to pull.
        let hot = MashHandler
            .decoction_volume(Request::new(DecoctionVolumeRequest {
                mash_c: 100.0,
                ..decoction_req()
            }))
            .await
            .unwrap()
            .into_inner();
        close(hot.decoction_l, 0.0);
    }

    #[tokio::test]
    async fn decoction_volume_rejects_bad_input() {
        for req in [
            DecoctionVolumeRequest {
                target_c: f64::NAN,
                ..decoction_req()
            },
            DecoctionVolumeRequest {
                mash_water_l: -1.0,
                ..decoction_req()
            },
            DecoctionVolumeRequest {
                boil_c: Some(f64::NAN),
                ..decoction_req()
            },
        ] {
            let e = MashHandler
                .decoction_volume(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }

    fn total_water_req(kind: RecipeKind, has_boil: bool) -> TotalWaterRequest {
        TotalWaterRequest {
            kind: kind as i32,
            has_boil_ingredients: has_boil,
            mash_water_l: 16.0,
            sparge_l: 19.007,
            topup_kettle_l: 1.0,
            topup_fermenter_l: 2.0,
            batch_l: 23.0,
            preboil_l: 29.0,
        }
    }

    // Vector 05.json total_water cases (proto kind = engine recipe_type + 1).
    #[tokio::test]
    async fn total_water_matches_vectors() {
        for (kind, boil, want) in [
            (RecipeKind::Extract, false, 23.0),
            (RecipeKind::Extract, true, 31.0),
            (RecipeKind::PartialMash, false, 38.007),
            (RecipeKind::AllGrain, true, 38.007),
            (RecipeKind::Cider, false, 23.0),
            (RecipeKind::Cider, true, 31.0),
        ] {
            let r = MashHandler
                .total_water(Request::new(total_water_req(kind, boil)))
                .await
                .unwrap()
                .into_inner();
            close(r.total_water_l, want);
        }
    }

    #[tokio::test]
    async fn total_water_rejects_bad_input() {
        // Unspecified kind, and recipe_type 7 (Seltzer, proto 8 is unknown) as in the vectors.
        let unspecified = MashHandler
            .total_water(Request::new(total_water_req(
                RecipeKind::Unspecified,
                false,
            )))
            .await
            .unwrap_err();
        assert_eq!(unspecified.code(), Code::InvalidArgument);
        assert_eq!(field_of(&unspecified), "kind");
        let unknown = MashHandler
            .total_water(Request::new(TotalWaterRequest {
                kind: 99,
                ..total_water_req(RecipeKind::Extract, false)
            }))
            .await
            .unwrap_err();
        assert_eq!(unknown.code(), Code::InvalidArgument);
        for req in [
            TotalWaterRequest {
                sparge_l: f64::NAN,
                ..total_water_req(RecipeKind::AllGrain, false)
            },
            TotalWaterRequest {
                batch_l: -1.0,
                ..total_water_req(RecipeKind::AllGrain, false)
            },
            TotalWaterRequest {
                mash_water_l: f64::INFINITY,
                ..total_water_req(RecipeKind::AllGrain, false)
            },
        ] {
            let e = MashHandler
                .total_water(Request::new(req))
                .await
                .unwrap_err();
            assert_eq!(e.code(), Code::InvalidArgument);
        }
    }
}
