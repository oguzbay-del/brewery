//! `MashService` implementation: volume chain, strike temperature and sparge plan over
//! `engine::mash` (metric, so no unit conversion).

use engine::mash::{
    self, SpargeAction as EngineAction, SpargeParams, StrikeTempInput, Tun, VolumeChainInput,
};
use tonic::{Request, Response, Status};

use crate::proto::{
    SpargeAction, SpargePlanRequest, SpargePlanResponse, SpargeStep, StrikeTempRequest,
    StrikeTempResponse, VolumeChainRequest, VolumeChainResponse,
    mash_service_server::{MashService, MashServiceServer},
};
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
        let tun = match r.tun {
            Some(t) => Tun {
                mass_kg: non_negative("tun.mass_kg", t.mass_kg)?,
                specific_heat_cal: non_negative("tun.specific_heat_cal", t.specific_heat_cal)?,
                volume_l: non_negative("tun.volume_l", t.volume_l)?,
            },
            None => Tun::NONE,
        };
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
}

/// Builds the tonic server wrapper for the mash handler.
#[must_use]
pub fn server() -> MashServiceServer<MashHandler> {
    MashServiceServer::new(MashHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::MashTun;
    use tonic::Code;

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
}
