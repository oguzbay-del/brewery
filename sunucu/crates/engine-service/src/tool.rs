//! `ToolService` implementation: small calculators over `engine::density` and `engine::mash`.

use engine::density::{self, HydrometerUnit};
use engine::mash::{self, DilutionInput};
use tonic::{Request, Response, Status};

use crate::convert::c_to_f;
use crate::proto::{
    CalculateAlcoholRequest, CalculateAlcoholResponse, ConvertGravityRequest,
    ConvertGravityResponse, CorrectHydrometerRequest, CorrectHydrometerResponse, DiluteRequest,
    DiluteResponse, GravityUnit, RefractometerFermentingRequest, RefractometerFermentingResponse,
    RefractometerUnfermentedRequest, RefractometerUnfermentedResponse,
    tool_service_server::{ToolService, ToolServiceServer},
};
use crate::status::{bad_request, from_engine};

/// Default zero offset of the refractometer, degrees Brix.
const DEFAULT_ZERO_BRIX: f64 = 0.0;
/// Default wort correction factor of the refractometer.
const DEFAULT_WCF: f64 = 1.02;

/// Handler implementing the tool gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolHandler;

fn gravity_unit(field: &str, raw: i32) -> Result<HydrometerUnit, Status> {
    match GravityUnit::try_from(raw) {
        Ok(GravityUnit::Sg) => Ok(HydrometerUnit::Sg),
        Ok(GravityUnit::Plato) => Ok(HydrometerUnit::Plato),
        Ok(GravityUnit::Brix) => Ok(HydrometerUnit::Brix),
        Ok(GravityUnit::Unspecified) | Err(_) => Err(bad_request(
            "gravity unit must be specified",
            field,
            "must be SG, PLATO or BRIX",
        )),
    }
}

#[tonic::async_trait]
impl ToolService for ToolHandler {
    async fn convert_gravity(
        &self,
        request: Request<ConvertGravityRequest>,
    ) -> Result<Response<ConvertGravityResponse>, Status> {
        let r = request.into_inner();
        let from = gravity_unit("from", r.from)?;
        let to = gravity_unit("to", r.to)?;
        let value = density::convert_hydrometer(r.value, from, to).map_err(|e| from_engine(&e))?;
        Ok(Response::new(ConvertGravityResponse { value }))
    }

    async fn correct_hydrometer(
        &self,
        request: Request<CorrectHydrometerRequest>,
    ) -> Result<Response<CorrectHydrometerResponse>, Status> {
        let r = request.into_inner();
        let corrected_sg = density::hydrometer_correction(
            r.sg,
            c_to_f(r.sample_temp_c),
            c_to_f(r.calibration_temp_c),
        )
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(CorrectHydrometerResponse { corrected_sg }))
    }

    async fn refractometer_unfermented(
        &self,
        request: Request<RefractometerUnfermentedRequest>,
    ) -> Result<Response<RefractometerUnfermentedResponse>, Status> {
        let r = request.into_inner();
        let og_sg = density::refractometer_unfermented(
            r.brix,
            r.zero_brix.unwrap_or(DEFAULT_ZERO_BRIX),
            r.wort_correction_factor.unwrap_or(DEFAULT_WCF),
        )
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(RefractometerUnfermentedResponse { og_sg }))
    }

    async fn refractometer_fermenting(
        &self,
        request: Request<RefractometerFermentingRequest>,
    ) -> Result<Response<RefractometerFermentingResponse>, Status> {
        let r = request.into_inner();
        let fg_sg = density::refractometer_fermenting(
            r.og_sg,
            r.brix,
            r.zero_brix.unwrap_or(DEFAULT_ZERO_BRIX),
        )
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(RefractometerFermentingResponse { fg_sg }))
    }

    async fn calculate_alcohol(
        &self,
        request: Request<CalculateAlcoholRequest>,
    ) -> Result<Response<CalculateAlcoholResponse>, Status> {
        let r = request.into_inner();
        let by_volume = density::abv(r.og_sg, r.fg_sg).map_err(|e| from_engine(&e))?;
        let by_weight = density::abw(r.og_sg, r.fg_sg).map_err(|e| from_engine(&e))?;
        Ok(Response::new(CalculateAlcoholResponse {
            abv_pct: by_volume,
            abw_pct: by_weight,
        }))
    }

    async fn dilute(
        &self,
        request: Request<DiluteRequest>,
    ) -> Result<Response<DiluteResponse>, Status> {
        let r = request.into_inner();
        let d = mash::dilution(&DilutionInput {
            sg0: r.sg,
            v0_l: r.volume_l,
            v_add_l: r.added_volume_l,
            wort: r.added_is_wort,
            sg_add: r.added_sg.unwrap_or(1.0),
        })
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(DiluteResponse {
            total_volume_l: d.v_tot_l,
            sg: d.sg,
        }))
    }
}

/// Builds the tonic server wrapper for the tool handler.
#[must_use]
pub fn server() -> ToolServiceServer<ToolHandler> {
    ToolServiceServer::new(ToolHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    async fn convert(value: f64, from: GravityUnit, to: GravityUnit) -> f64 {
        ToolHandler
            .convert_gravity(Request::new(ConvertGravityRequest {
                value,
                from: from as i32,
                to: to as i32,
            }))
            .await
            .unwrap()
            .into_inner()
            .value
    }

    #[tokio::test]
    async fn convert_gravity_matches_vectors() {
        close(
            convert(1.05, GravityUnit::Sg, GravityUnit::Plato).await,
            12.387_647_125_000_058,
        );
        close(
            convert(1.05, GravityUnit::Sg, GravityUnit::Brix).await,
            12.387_028_012_500_082,
        );
        close(
            convert(12.0, GravityUnit::Plato, GravityUnit::Sg).await,
            1.048_375_058_224_040_5,
        );
        close(
            convert(12.0, GravityUnit::Brix, GravityUnit::Sg).await,
            1.048_378_242_157_272_5,
        );
    }

    #[tokio::test]
    async fn convert_gravity_rejects_unspecified_and_nan() {
        let unspecified = ToolHandler
            .convert_gravity(Request::new(ConvertGravityRequest {
                value: 1.05,
                from: GravityUnit::Unspecified as i32,
                to: GravityUnit::Plato as i32,
            }))
            .await
            .unwrap_err();
        assert_eq!(unspecified.code(), Code::InvalidArgument);
        let unknown = ToolHandler
            .convert_gravity(Request::new(ConvertGravityRequest {
                value: 1.05,
                from: 99,
                to: GravityUnit::Plato as i32,
            }))
            .await
            .unwrap_err();
        assert_eq!(unknown.code(), Code::InvalidArgument);
        let nan = ToolHandler
            .convert_gravity(Request::new(ConvertGravityRequest {
                value: f64::NAN,
                from: GravityUnit::Sg as i32,
                to: GravityUnit::Plato as i32,
            }))
            .await
            .unwrap_err();
        assert_eq!(nan.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn correct_hydrometer_converts_celsius() {
        // 77 F = 25 C, 60 F = 15.5556 C (01.json: sg 1.05, 77/60 F -> 1.0520003556412).
        let r = ToolHandler
            .correct_hydrometer(Request::new(CorrectHydrometerRequest {
                sg: 1.05,
                sample_temp_c: 25.0,
                calibration_temp_c: (60.0 - 32.0) * 5.0 / 9.0,
            }))
            .await
            .unwrap()
            .into_inner();
        close(r.corrected_sg, 1.052_000_355_641_2);
        let bad = ToolHandler
            .correct_hydrometer(Request::new(CorrectHydrometerRequest {
                sg: f64::NAN,
                sample_temp_c: 20.0,
                calibration_temp_c: 20.0,
            }))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn refractometer_unfermented_uses_defaults() {
        let r = ToolHandler
            .refractometer_unfermented(Request::new(RefractometerUnfermentedRequest {
                brix: 12.0,
                zero_brix: None,
                wort_correction_factor: None,
            }))
            .await
            .unwrap()
            .into_inner();
        close(r.og_sg, 1.047_387_054_714_236_8);
        let bad = ToolHandler
            .refractometer_unfermented(Request::new(RefractometerUnfermentedRequest {
                brix: 12.0,
                zero_brix: None,
                wort_correction_factor: Some(0.0),
            }))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn refractometer_fermenting_matches_vector() {
        let r = ToolHandler
            .refractometer_fermenting(Request::new(RefractometerFermentingRequest {
                og_sg: 1.05,
                brix: 6.5,
                zero_brix: Some(0.0),
            }))
            .await
            .unwrap()
            .into_inner();
        close(r.fg_sg, 1.010_611_284_992_081_4);
        let bad = ToolHandler
            .refractometer_fermenting(Request::new(RefractometerFermentingRequest {
                og_sg: f64::NAN,
                brix: 6.5,
                zero_brix: None,
            }))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn calculate_alcohol_matches_vector() {
        let r = ToolHandler
            .calculate_alcohol(Request::new(CalculateAlcoholRequest {
                og_sg: 1.065,
                fg_sg: 1.012,
            }))
            .await
            .unwrap()
            .into_inner();
        close(r.abv_pct, 7.041_764_857_774_678_5);
        close(r.abw_pct, 5.543_024_015_169_848);
        let bad = ToolHandler
            .calculate_alcohol(Request::new(CalculateAlcoholRequest {
                og_sg: f64::NAN,
                fg_sg: 1.01,
            }))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn dilute_matches_vectors() {
        let water = ToolHandler
            .dilute(Request::new(DiluteRequest {
                sg: 1.06,
                volume_l: 20.0,
                added_volume_l: 5.0,
                added_is_wort: false,
                added_sg: None,
            }))
            .await
            .unwrap()
            .into_inner();
        close(water.total_volume_l, 25.0);
        close(water.sg, 1.048);
        let wort = ToolHandler
            .dilute(Request::new(DiluteRequest {
                sg: 1.06,
                volume_l: 20.0,
                added_volume_l: 5.0,
                added_is_wort: true,
                added_sg: Some(1.04),
            }))
            .await
            .unwrap()
            .into_inner();
        close(wort.sg, 1.056);
        let bad = ToolHandler
            .dilute(Request::new(DiluteRequest {
                sg: f64::NAN,
                volume_l: 20.0,
                added_volume_l: 5.0,
                added_is_wort: false,
                added_sg: None,
            }))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), Code::InvalidArgument);
    }
}
