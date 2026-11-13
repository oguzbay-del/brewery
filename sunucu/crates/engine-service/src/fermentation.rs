//! `FermentationService` implementation: fermentation tracking, gravity trigger, device
//! calibration, keg and cask conditioning and no-chill IBU over `engine::home` (spec 07),
//! `engine::trigger` (spec 11) and `engine::manual` (spec 10).
//!
//! These engine modules are metric, so no unit conversion is needed except percent to
//! fraction. Gravity threshold comparisons stay in the engine (integer units).

use engine::home::{self, Forecast, ForecastInput, ForecastResult};
use engine::manual::{
    self, FewForecast, FewForecastInput, ForecastMethod, Insufficient, NochillMode, ReadingRole,
    ReadingSource,
};
use engine::trigger::{
    self, CaskPrimingInput, FermentationType, FgSource, KegMode, KegParams, PolyPoint,
    PredictedTriggerInput, PrimingWarning, Stability, Sugar, TriggerForecast,
};
use tonic::{Request, Response, Status};

use crate::check::{finite, non_negative, out};
use crate::proto::{
    self, CalibrateIspindelRequest, CalibrateIspindelResponse, CalibrateTiltRequest,
    CalibrateTiltResponse, CarbonationPoint, CaskConditioningRequest, CaskConditioningResponse,
    ConditioningWarning, ConditioningWarningCode, ConvertIspindelAnglesRequest,
    ConvertIspindelAnglesResponse, DeviceForecast, FermentationForecastMethod,
    FermentationScheduleRequest, FermentationScheduleResponse, FgReferenceSource, FgStability,
    FgStabilityState, GravityPoint, GravityTrigger, IspindelSg, KegPrimingRequest,
    KegPrimingResponse, ManualForecast, ManualForecastStatus, NoChillIbuRequest,
    NoChillIbuResponse, PredictedTriggerState, TrackFermentationRequest, TrackFermentationResponse,
    fermentation_service_server::{FermentationService, FermentationServiceServer},
};
use crate::recipe::engine_code;
use crate::status::{bad_request, from_engine};

/// Handler implementing the fermentation gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct FermentationHandler;

/// Consecutive hourly medians that must meet the trigger on device data (spec 11 §1.2).
const TRIGGER_CONSECUTIVE: usize = 3;
/// Device forecast: finished when this close to FG, SG (spec 07 §1.5).
const FINISH_EPS_SG: f64 = 0.0005;
/// Device forecast: minimum hourly points (spec 07 §1.4).
const MIN_POINTS: usize = 12;
/// Device forecast: minimum share of the expected drop, percent (spec 07 §1.4).
const MIN_DROP_PCT: f64 = 30.0;
/// "FG stable" window, hours (spec 11 §1.4).
const STABLE_WINDOW_H: f64 = 48.0;
/// "FG stable" minimum hourly points in the window (spec 11 §1.4).
const STABLE_MIN_POINTS: usize = 24;
/// Lagering rule: OG points per week (spec 11 §1.5).
const LAGER_POINTS_PER_WEEK: i64 = 8;
/// Lagering rule: shortest suggestion, weeks (spec 11 §1.5).
const LAGER_MIN_WEEKS: i64 = 2;
/// Lagering rule: longest suggestion, weeks (spec 11 §1.5).
const LAGER_MAX_WEEKS: i64 = 12;
/// Cask conditioning: days at the base temperature (spec 11 §3.5).
const CASK_BASE_DAYS: f64 = 3.0;
/// Cask conditioning: base temperature, C (spec 11 §3.5).
const CASK_BASE_TEMP_C: f64 = 12.0;
/// No-chill model: wort temperature when the cube is filled, C (spec 10 §3.2).
const CUBE_FILL_C: f64 = 95.0;
/// No-chill model: ambient temperature, C (spec 10 §3.2).
const AMBIENT_C: f64 = 20.0;

/// Maps a wire count to `usize` (saturating; the engine rejects absurd values).
fn to_usize(v: u32) -> usize {
    usize::try_from(v).unwrap_or(usize::MAX)
}

/// Optional finite, non-negative value.
fn opt_non_negative(field: &str, v: Option<f64>) -> Result<Option<f64>, Status> {
    v.map(|x| non_negative(field, x)).transpose()
}

/// Error for a proto enum number the engine does not know.
fn unknown_code() -> engine::Error {
    engine::Error::invalid("enum", "unknown enum value")
}

fn reading_source(field: &str, raw: i32) -> Result<ReadingSource, Status> {
    engine_code(field, raw, |c| match c {
        0 => Ok(ReadingSource::Device),
        1 => Ok(ReadingSource::Manual),
        2 => Ok(ReadingSource::Paper),
        _ => Err(unknown_code()),
    })
}

fn fermentation_type(field: &str, raw: i32) -> Result<FermentationType, Status> {
    engine_code(field, raw, |c| match c {
        0 => Ok(FermentationType::Lager),
        1 => Ok(FermentationType::Ale),
        _ => Err(unknown_code()),
    })
}

fn sugar(field: &str, raw: i32) -> Result<Sugar, Status> {
    engine_code(field, raw, |c| match c {
        0 => Ok(Sugar::Dextrose),
        1 => Ok(Sugar::Sucrose),
        2 => Ok(Sugar::Invert),
        _ => Err(unknown_code()),
    })
}

fn keg_mode(field: &str, raw: i32) -> Result<KegMode, Status> {
    engine_code(field, raw, |c| match c {
        0 => Ok(KegMode::Full),
        1 => Ok(KegMode::Partial),
        _ => Err(unknown_code()),
    })
}

/// No-chill mode without its parameters: `true` is the cooling model.
fn nochill_is_model(field: &str, raw: i32) -> Result<bool, Status> {
    engine_code(field, raw, |c| match c {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(unknown_code()),
    })
}

fn point_code(p: PolyPoint) -> CarbonationPoint {
    match p {
        PolyPoint::Conditioning => CarbonationPoint::Conditioning,
        PolyPoint::Serving => CarbonationPoint::Serving,
        PolyPoint::Target => CarbonationPoint::Target,
        PolyPoint::Peak => CarbonationPoint::Peak,
    }
}

/// Maps one engine priming warning to the wire message (code plus numeric fields).
fn map_priming_warning(w: &PrimingWarning) -> Result<ConditioningWarning, Status> {
    let mut m = ConditioningWarning::default();
    match *w {
        PrimingWarning::DryMatterUnknown => {
            m.code = ConditioningWarningCode::DryMatterUnknown as i32;
        }
        PrimingWarning::PolynomialBelowTemperature { point, t_c } => {
            m.code = ConditioningWarningCode::PolynomialBelowTemperature as i32;
            m.point = point_code(point) as i32;
            m.temp_c = Some(out("temp_c", t_c)?);
        }
        PrimingWarning::PolynomialAboveVolume { point, co2_vol } => {
            m.code = ConditioningWarningCode::PolynomialAboveVolume as i32;
            m.point = point_code(point) as i32;
            m.co2_vol = Some(out("co2_vol", co2_vol)?);
        }
        PrimingWarning::PeakVolumeSaturated { limit_co2_vol } => {
            m.code = ConditioningWarningCode::PeakVolumeSaturated as i32;
            m.co2_vol = Some(out("co2_vol", limit_co2_vol)?);
        }
        PrimingWarning::PeakAboveLimit {
            peak_bar,
            limit_bar,
            t_cond_c,
        } => {
            m.code = ConditioningWarningCode::PeakAboveLimit as i32;
            m.peak_bar = Some(out("peak_bar", peak_bar)?);
            m.limit_bar = Some(out("limit_bar", limit_bar)?);
            m.temp_c = Some(out("temp_c", t_cond_c)?);
        }
    }
    Ok(m)
}

fn map_priming_warnings(ws: &[PrimingWarning]) -> Result<Vec<ConditioningWarning>, Status> {
    ws.iter().map(map_priming_warning).collect()
}

fn map_calibration_warnings(ws: &[home::CalibrationWarning]) -> Vec<i32> {
    ws.iter()
        .map(|w| match w {
            home::CalibrationWarning::NarrowAngleRange => {
                proto::CalibrationWarning::NarrowAngleRange
            }
            home::CalibrationWarning::AngleOutsideRange => {
                proto::CalibrationWarning::AngleOutsideRange
            }
        } as i32)
        .collect()
}

// ------------------------------------------------------------------ TrackFermentation

/// Validated optional parameters of `TrackFermentation`, defaults applied.
struct TrackParams {
    delta_sg: f64,
    consecutive: usize,
    eps_sg: f64,
    min_points: usize,
    min_drop: f64,
    window_h: f64,
    stable_min: usize,
    lag_h: f64,
    sigma_sg: f64,
}

fn track_params(r: &TrackFermentationRequest) -> Result<TrackParams, Status> {
    let kind = fermentation_type("kind", r.kind)?;
    let delta_sg = match opt_non_negative("trigger_delta_sg", r.trigger_delta_sg)? {
        Some(d) => d,
        None => trigger::default_delta(kind),
    };
    Ok(TrackParams {
        delta_sg,
        consecutive: r.trigger_consecutive.map_or(TRIGGER_CONSECUTIVE, to_usize),
        eps_sg: opt_non_negative("finish_eps_sg", r.finish_eps_sg)?.unwrap_or(FINISH_EPS_SG),
        min_points: r.min_points.map_or(MIN_POINTS, to_usize),
        // The engine takes a fraction.
        min_drop: opt_non_negative("min_drop_pct", r.min_drop_pct)?.unwrap_or(MIN_DROP_PCT) / 100.0,
        window_h: opt_non_negative("stable_window_h", r.stable_window_h)?
            .unwrap_or(STABLE_WINDOW_H),
        stable_min: r.stable_min_points.map_or(STABLE_MIN_POINTS, to_usize),
        lag_h: opt_non_negative("lag_h", r.lag_h)?.unwrap_or(manual::LAG_H),
        sigma_sg: opt_non_negative("fg_sigma_sg", r.fg_sigma_sg)?.unwrap_or(manual::SIGMA_FG),
    })
}

/// Model readings of a batch, split by feed: `(t_hours, SG)`.
struct Readings {
    /// Device readings (spec 07 model).
    device: Vec<(f64, f64)>,
    /// Manual and paper readings of a batch without device (spec 10 model), by time.
    manual: Vec<(f64, f64)>,
}

fn split_readings(r: &TrackFermentationRequest) -> Result<Readings, Status> {
    if r.readings.is_empty() {
        return Err(bad_request(
            "readings must not be empty",
            "readings",
            "must not be empty",
        ));
    }
    let mut device = Vec::new();
    let mut manual_pts = Vec::new();
    for (i, g) in r.readings.iter().enumerate() {
        let t = non_negative(&format!("readings[{i}].time_h"), g.time_h)?;
        let sg = non_negative(&format!("readings[{i}].sg"), g.sg)?;
        let source = reading_source(&format!("readings[{i}].source"), g.source)?;
        // Bias-correction points are not model points (spec 10 §1.1).
        if manual::reading_role(source, r.has_device) == ReadingRole::BiasCorrection {
            continue;
        }
        if source == ReadingSource::Device {
            device.push((t, sg));
        } else {
            manual_pts.push((t, sg));
        }
    }
    manual_pts.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    Ok(Readings {
        device,
        manual: manual_pts,
    })
}

fn map_device_forecast(f: Option<&ForecastResult>) -> Result<DeviceForecast, Status> {
    let Some(f) = f else {
        return Ok(DeviceForecast {
            ready: false,
            ..DeviceForecast::default()
        });
    };
    Ok(DeviceForecast {
        ready: true,
        fg_sg: Some(out("fg_sg", f.fg)?),
        fg_low_sg: Some(out("fg_low_sg", f.range_sg.0)?),
        fg_high_sg: Some(out("fg_high_sg", f.range_sg.1)?),
        a_sg: Some(out("a_sg", f.a_sg)?),
        k_per_h: Some(out("k_per_h", f.k_per_h)?),
        finish_h: Some(out("finish_h", f.finish_h)?),
        remaining_h: Some(out("remaining_h", f.remaining_h)?),
        fg_warning: Some(f.warning),
    })
}

fn map_manual_forecast(f: &FewForecast) -> Result<ManualForecast, Status> {
    let mut m = ManualForecast::default();
    match *f {
        FewForecast::LooksFinished { fg_measured } => {
            m.status = ManualForecastStatus::LooksFinished as i32;
            m.fg_measured_sg = Some(out("fg_measured_sg", fg_measured)?);
        }
        FewForecast::PossiblyStuck { sg, .. } => {
            m.status = ManualForecastStatus::PossiblyStuck as i32;
            m.fg_measured_sg = Some(out("fg_measured_sg", sg)?);
        }
        FewForecast::Insufficient(why) => {
            m.status = match why {
                Insufficient::NeedTwoReadings => ManualForecastStatus::NeedTwoReadings,
                Insufficient::DropBelow30Percent => ManualForecastStatus::DropBelow30Percent,
                Insufficient::NonPositiveK => ManualForecastStatus::NonPositiveK,
            } as i32;
        }
        FewForecast::TargetReached => m.status = ManualForecastStatus::TargetReached as i32,
        FewForecast::Estimate {
            k_per_h,
            finish_h,
            range_days,
            remaining_days,
            inconsistent,
        } => {
            m.status = ManualForecastStatus::Estimate as i32;
            m.k_per_h = Some(out("k_per_h", k_per_h)?);
            m.finish_h = Some(out("finish_h", finish_h)?);
            m.finish_min_days = Some(range_days.0);
            m.finish_max_days = Some(range_days.1);
            m.remaining_min_days = Some(remaining_days.0);
            m.remaining_max_days = Some(remaining_days.1);
            m.inconsistent = Some(inconsistent);
        }
    }
    Ok(m)
}

/// Trigger state, predicted time included when the device forecast is ready.
fn track_trigger(
    p: &TrackParams,
    readings: &Readings,
    hourly: &[(f64, f64)],
    fg_ref: f64,
    forecast: Option<&ForecastResult>,
) -> Result<GravityTrigger, Status> {
    // Device data: hourly medians, several in a row; manual data: single readings.
    let on_device = !readings.device.is_empty();
    let (points, consecutive) = if on_device {
        (hourly, p.consecutive)
    } else {
        (readings.manual.as_slice(), 1)
    };
    let at = trigger::trigger_index(points, fg_ref, p.delta_sg, consecutive)
        .map_err(|e| from_engine(&e))?;
    let mut t = GravityTrigger {
        delta_sg: p.delta_sg,
        threshold_sg: out("threshold_sg", fg_ref + p.delta_sg)?,
        triggered_at_h: at.map(|h| out("triggered_at_h", h)).transpose()?,
        needs_confirmation: !on_device && at.is_some(),
        ..GravityTrigger::default()
    };
    if let Some(f) = forecast {
        let predicted = trigger::predicted_trigger(&PredictedTriggerInput {
            fg_inf: f.fg,
            a_sg: f.a_sg,
            k_per_h: f.k_per_h,
            fg_ref,
            delta: p.delta_sg,
        })
        .map_err(|e| from_engine(&e))?;
        match predicted {
            TriggerForecast::At(h) => {
                t.predicted = PredictedTriggerState::At as i32;
                t.predicted_at_h = Some(out("predicted_at_h", h)?);
            }
            TriggerForecast::AlreadyPast => {
                t.predicted = PredictedTriggerState::AlreadyPast as i32;
            }
            TriggerForecast::Unreachable => {
                t.predicted = PredictedTriggerState::Unreachable as i32;
            }
        }
    }
    Ok(t)
}

/// "FG stable" state: device slope over hourly medians, else the manual stall rule.
fn track_stability(
    p: &TrackParams,
    readings: &Readings,
    hourly: &[(f64, f64)],
    fg_ref: f64,
) -> Result<FgStability, Status> {
    let mut s = FgStability {
        state: FgStabilityState::Insufficient as i32,
        slope_sg_per_day: None,
    };
    if !hourly.is_empty() {
        match trigger::is_stable(hourly, p.window_h, p.stable_min).map_err(|e| from_engine(&e))? {
            Stability::Insufficient => {}
            Stability::Measured {
                stable,
                slope_per_day,
            } => {
                s.state = if stable {
                    FgStabilityState::Stable
                } else {
                    FgStabilityState::NotStable
                } as i32;
                s.slope_sg_per_day = Some(out("slope_sg_per_day", slope_per_day)?);
            }
        }
    } else if readings.manual.len() >= 2 {
        s.state = if manual::stall_check(&readings.manual, fg_ref).is_some() {
            FgStabilityState::Stable
        } else {
            FgStabilityState::NotStable
        } as i32;
    }
    Ok(s)
}

fn track(r: &TrackFermentationRequest) -> Result<TrackFermentationResponse, Status> {
    let p = track_params(r)?;
    let og = non_negative("og_sg", r.og_sg)?;
    let fg_recipe = non_negative("fg_recipe_sg", r.fg_recipe_sg)?;
    let readings = split_readings(r)?;

    let hourly = home::hourly_median(&readings.device);
    let forecast = if readings.device.is_empty() {
        None
    } else {
        match home::forecast(&ForecastInput {
            points: &readings.device,
            og,
            fg_recipe,
            eps_sg: p.eps_sg,
            min_points: p.min_points,
            min_drop: p.min_drop,
        })
        .map_err(|e| from_engine(&e))?
        {
            Forecast::Ready(f) => Some(f),
            Forecast::InsufficientData => None,
        }
    };
    let method = match manual::forecast_method(forecast.is_some(), readings.manual.len()) {
        ForecastMethod::Device => FermentationForecastMethod::Device,
        ForecastMethod::Manual => FermentationForecastMethod::Manual,
        ForecastMethod::None => FermentationForecastMethod::None,
    };
    let device = (!readings.device.is_empty())
        .then(|| map_device_forecast(forecast.as_ref()))
        .transpose()?;
    let manual_forecast = if readings.manual.is_empty() {
        None
    } else {
        let f = manual::few_forecast(&FewForecastInput {
            readings: &readings.manual,
            og,
            fg_prior: fg_recipe,
            lag_h: p.lag_h,
            sigma_sg: p.sigma_sg,
        })
        .map_err(|e| from_engine(&e))?;
        Some(map_manual_forecast(&f)?)
    };

    let fg_ref = trigger::fg_reference(
        fg_recipe,
        forecast.as_ref().map(|f| f.fg),
        forecast.is_some(),
        p.delta_sg,
    );
    let sg_last = hourly.last().or(readings.manual.last()).map(|&(_, sg)| sg);
    let progress_pct = sg_last
        .and_then(|sg| trigger::progress(og, sg, fg_ref.fg))
        .map(|ratio| out("progress_pct", ratio * 100.0))
        .transpose()?;
    let trig = track_trigger(&p, &readings, &hourly, fg_ref.fg, forecast.as_ref())?;
    let stability = track_stability(&p, &readings, &hourly, fg_ref.fg)?;

    Ok(TrackFermentationResponse {
        method: method as i32,
        hourly_median: hourly
            .iter()
            .map(|&(t, sg)| {
                Ok(GravityPoint {
                    time_h: out("time_h", t)?,
                    sg: out("sg", sg)?,
                })
            })
            .collect::<Result<_, Status>>()?,
        device,
        manual: manual_forecast,
        fg_reference_sg: out("fg_reference_sg", fg_ref.fg)?,
        fg_reference_source: match fg_ref.source {
            FgSource::Recipe => FgReferenceSource::Recipe,
            FgSource::Forecast => FgReferenceSource::Forecast,
        } as i32,
        progress_pct,
        trigger: Some(trig),
        stability: Some(stability),
    })
}

// ------------------------------------------------------------------ the service

#[tonic::async_trait]
impl FermentationService for FermentationHandler {
    async fn track_fermentation(
        &self,
        request: Request<TrackFermentationRequest>,
    ) -> Result<Response<TrackFermentationResponse>, Status> {
        track(&request.into_inner()).map(Response::new)
    }

    async fn fermentation_schedule(
        &self,
        request: Request<FermentationScheduleRequest>,
    ) -> Result<Response<FermentationScheduleResponse>, Status> {
        let r = request.into_inner();
        let og = non_negative("og_sg", r.og_sg)?;
        let kind = fermentation_type("kind", r.kind)?;
        let per_week = r
            .lager_points_per_week
            .map_or(LAGER_POINTS_PER_WEEK, i64::from);
        let lo = r.lager_min_weeks.map_or(LAGER_MIN_WEEKS, i64::from);
        let hi = r.lager_max_weeks.map_or(LAGER_MAX_WEEKS, i64::from);
        if lo > hi {
            return Err(bad_request(
                "lager_min_weeks must not exceed lager_max_weeks",
                "lager_min_weeks",
                "must not exceed lager_max_weeks",
            ));
        }
        let mut ramp_h = Vec::with_capacity(r.ramps.len());
        for (i, ramp) in r.ramps.iter().enumerate() {
            let from = finite(&format!("ramps[{i}].from_c"), ramp.from_c)?;
            let to = finite(&format!("ramps[{i}].to_c"), ramp.to_c)?;
            let rate =
                opt_non_negative(&format!("ramps[{i}].rate_c_per_day"), ramp.rate_c_per_day)?
                    .unwrap_or(trigger::LAGER_COOL_RATE);
            let h = trigger::ramp_hours(from, to, rate).map_err(|e| from_engine(&e))?;
            ramp_h.push(out("ramp_h", h)?);
        }
        let weeks = trigger::lager_weeks(og, per_week, lo, hi).map_err(|e| from_engine(&e))?;
        Ok(Response::new(FermentationScheduleResponse {
            ramp_h,
            lager_weeks: u32::try_from(weeks).map_err(|_| {
                bad_request("lager weeks out of range", "lager_weeks", "out of range")
            })?,
            default_delta_sg: trigger::default_delta(kind),
        }))
    }

    async fn calibrate_ispindel(
        &self,
        request: Request<CalibrateIspindelRequest>,
    ) -> Result<Response<CalibrateIspindelResponse>, Status> {
        let r = request.into_inner();
        let mut points = Vec::with_capacity(r.points.len());
        for (i, p) in r.points.iter().enumerate() {
            points.push((
                finite(&format!("points[{i}].angle_deg"), p.angle_deg)?,
                non_negative(&format!("points[{i}].sg"), p.sg)?,
            ));
        }
        let degree = r.degree.map_or(2, to_usize);
        let cal = home::ispindel_cal(&points, degree).map_err(|e| from_engine(&e))?;
        Ok(Response::new(CalibrateIspindelResponse {
            coefficients: cal
                .coefficients
                .iter()
                .map(|&c| out("coefficients", c))
                .collect::<Result<_, _>>()?,
            r2: out("r2", cal.r2)?,
            max_residual_sg: out("max_residual_sg", cal.max_residual_sg)?,
            angle_min_deg: out("angle_min_deg", cal.angle_min_deg)?,
            angle_max_deg: out("angle_max_deg", cal.angle_max_deg)?,
            warnings: map_calibration_warnings(&cal.warnings),
        }))
    }

    async fn convert_ispindel_angles(
        &self,
        request: Request<ConvertIspindelAnglesRequest>,
    ) -> Result<Response<ConvertIspindelAnglesResponse>, Status> {
        let r = request.into_inner();
        if r.coefficients.is_empty() {
            return Err(bad_request(
                "coefficients must not be empty",
                "coefficients",
                "must not be empty",
            ));
        }
        if r.angles_deg.is_empty() {
            return Err(bad_request(
                "angles_deg must not be empty",
                "angles_deg",
                "must not be empty",
            ));
        }
        let co = r
            .coefficients
            .iter()
            .enumerate()
            .map(|(i, &c)| finite(&format!("coefficients[{i}]"), c))
            .collect::<Result<Vec<_>, _>>()?;
        let a_min = finite("angle_min_deg", r.angle_min_deg)?;
        let a_max = finite("angle_max_deg", r.angle_max_deg)?;
        let readings = r
            .angles_deg
            .iter()
            .enumerate()
            .map(|(i, &a)| {
                let angle = finite(&format!("angles_deg[{i}]"), a)?;
                let v = home::ispindel_eval(&co, a_min, a_max, angle);
                Ok(IspindelSg {
                    sg: out("sg", v.sg)?,
                    warnings: map_calibration_warnings(&v.warnings),
                })
            })
            .collect::<Result<_, Status>>()?;
        Ok(Response::new(ConvertIspindelAnglesResponse { readings }))
    }

    async fn calibrate_tilt(
        &self,
        request: Request<CalibrateTiltRequest>,
    ) -> Result<Response<CalibrateTiltResponse>, Status> {
        let r = request.into_inner();
        let mut points = Vec::with_capacity(r.points.len());
        for (i, p) in r.points.iter().enumerate() {
            points.push((
                non_negative(&format!("points[{i}].tilt_sg"), p.tilt_sg)?,
                non_negative(&format!("points[{i}].reference_sg"), p.reference_sg)?,
            ));
        }
        let t = home::tilt_cal(&points).map_err(|e| from_engine(&e))?;
        Ok(Response::new(CalibrateTiltResponse {
            slope: out("slope", t.slope)?,
            offset_sg: out("offset_sg", t.offset_sg)?,
        }))
    }

    async fn keg_priming(
        &self,
        request: Request<KegPrimingRequest>,
    ) -> Result<Response<KegPrimingResponse>, Status> {
        let r = request.into_inner();
        let k = trigger::keg_priming(&KegParams {
            co2_vol: non_negative("co2_vol", r.co2_vol)?,
            t_res_c: finite("fermentation_max_c", r.fermentation_max_c)?,
            t_cond_c: finite("conditioning_c", r.conditioning_c)?,
            beer_l: non_negative("beer_l", r.beer_l)?,
            head_l: non_negative("headspace_l", r.headspace_l)?,
            sugar: sugar("sugar", r.sugar)?,
            purged: r.purged,
            mode: keg_mode("mode", r.mode)?,
            part: opt_non_negative("partial_fraction", r.partial_fraction)?
                .unwrap_or(trigger::PARTIAL_FRACTION),
            t_serv_c: finite("serving_c", r.serving_c)?,
            dry: opt_non_negative("dry_matter_fraction", r.dry_matter_fraction)?,
            limit_bar: opt_non_negative("keg_limit_bar", r.keg_limit_bar)?,
        })
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(KegPrimingResponse {
            sugar_g: out("sugar_g", k.sugar_g)?,
            bottle_equivalent_g: out("bottle_equivalent_g", k.bottle_equivalent_g)?,
            headspace_share_pct: k
                .head_share_pct
                .map(|v| out("headspace_share_pct", v))
                .transpose()?,
            serving_bar: out("serving_bar", k.serving_bar)?,
            peak_co2_vol: out("peak_co2_vol", k.peak_co2_vol)?,
            peak_bar: out("peak_bar", k.peak_bar)?,
            peak_total_gauge_bar: out("peak_total_gauge_bar", k.peak_total_gauge_bar)?,
            co2_topup_bar: k
                .co2_topup_bar
                .map(|v| out("co2_topup_bar", v))
                .transpose()?,
            warnings: map_priming_warnings(&k.warnings)?,
        }))
    }

    async fn cask_conditioning(
        &self,
        request: Request<CaskConditioningRequest>,
    ) -> Result<Response<CaskConditioningResponse>, Status> {
        let r = request.into_inner();
        let volume_l = non_negative("volume_l", r.volume_l)?;
        let cellar_c = finite("cellar_c", r.cellar_c)?;
        let finings_rate = opt_non_negative("finings_ml_per_l", r.finings_ml_per_l)?
            .unwrap_or(trigger::FININGS_ML_PER_L);
        let base_days = opt_non_negative("base_days", r.base_days)?.unwrap_or(CASK_BASE_DAYS);
        let base_temp_c = r
            .base_temp_c
            .map(|v| finite("base_temp_c", v))
            .transpose()?
            .unwrap_or(CASK_BASE_TEMP_C);
        let c = trigger::cask_priming(&CaskPrimingInput {
            volume_l,
            co2_vol: opt_non_negative("co2_vol", r.co2_vol)?.unwrap_or(trigger::CASK_TARGET_VOL),
            t_res_c: finite("fermentation_max_c", r.fermentation_max_c)?,
            sg_rack: non_negative("rack_sg", r.rack_sg)?,
            fg_ref: non_negative("fg_reference_sg", r.fg_reference_sg)?,
            sugar: sugar("sugar", r.sugar)?,
            dry: opt_non_negative("dry_matter_fraction", r.dry_matter_fraction)?,
        })
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(CaskConditioningResponse {
            residual_co2_vol: out("residual_co2_vol", c.residual_co2_vol)?,
            from_extract_co2_vol: out("from_extract_co2_vol", c.from_extract_co2_vol)?,
            missing_co2_vol: out("missing_co2_vol", c.missing_co2_vol)?,
            sugar_g: out("sugar_g", c.sugar_g)?,
            sugar_g_per_l: out("sugar_g_per_l", c.g_per_l)?,
            excess: c.excess,
            warnings: map_priming_warnings(&c.warnings)?,
            finings_ml: out("finings_ml", trigger::finings_ml(volume_l, finings_rate))?,
            conditioning_days: out(
                "conditioning_days",
                trigger::cask_days(cellar_c, base_days, base_temp_c),
            )?,
        }))
    }

    async fn no_chill_ibu(
        &self,
        request: Request<NoChillIbuRequest>,
    ) -> Result<Response<NoChillIbuResponse>, Status> {
        let r = request.into_inner();
        let mut hops = Vec::with_capacity(r.hops.len());
        for (i, h) in r.hops.iter().enumerate() {
            hops.push((
                non_negative(&format!("hops[{i}].amount_g"), h.amount_g)?,
                // The engine takes the alpha acid as a fraction.
                non_negative(&format!("hops[{i}].alpha_pct"), h.alpha_pct)? / 100.0,
                non_negative(&format!("hops[{i}].time_min"), h.time_min)?,
            ));
        }
        let sg = non_negative("boil_sg", r.boil_sg)?;
        let volume_l = non_negative("volume_l", r.volume_l)?;
        let mode = if nochill_is_model("mode", r.mode)? {
            NochillMode::Model {
                t0_c: finite("cube_fill_c", r.cube_fill_c.unwrap_or(CUBE_FILL_C))?,
                t_amb_c: finite("ambient_c", r.ambient_c.unwrap_or(AMBIENT_C))?,
                v_cube_l: opt_non_negative("cube_l", r.cube_l)?,
            }
        } else {
            NochillMode::Rule
        };
        let n = manual::nochill_ibu(&hops, sg, volume_l, mode).map_err(|e| from_engine(&e))?;
        Ok(Response::new(NoChillIbuResponse {
            base_ibu: out("base_ibu", n.base_ibu)?,
            no_chill_ibu: out("no_chill_ibu", n.nochill_ibu)?,
            extra_min: out("extra_min", n.extra_min)?,
        }))
    }
}

/// Builds the tonic server wrapper for the fermentation handler.
#[must_use]
pub fn server() -> FermentationServiceServer<FermentationHandler> {
    FermentationServiceServer::new(FermentationHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{
        ConditioningSugar, FermentationKind, GravityReading, GravityReadingSource, IspindelPoint,
        KegPrimingMode, NoChillHop, NoChillMode, TemperatureRamp, TiltPoint,
    };
    use tonic::Code;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
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

    fn reading(t: f64, sg: f64, source: GravityReadingSource) -> GravityReading {
        GravityReading {
            time_h: t,
            sg,
            source: source as i32,
        }
    }

    /// Vector 07 case 16: twelve hourly device points (`og` 1.09, `fg_recipe` 1.017).
    const V07_16: [(f64, f64); 12] = [
        (0.2, 1.09),
        (1.2, 1.086_197),
        (2.2, 1.082_757),
        (3.2, 1.079_679),
        (4.2, 1.076_963),
        (5.2, 1.074_609),
        (6.2, 1.072_617),
        (7.2, 1.070_987),
        (8.2, 1.069_72),
        (9.2, 1.068_814),
        (10.2, 1.068_271),
        (11.2, 1.068_09),
    ];

    fn device_req(points: &[(f64, f64)], og: f64, fg: f64) -> TrackFermentationRequest {
        TrackFermentationRequest {
            readings: points
                .iter()
                .map(|&(t, sg)| reading(t, sg, GravityReadingSource::Device))
                .collect(),
            has_device: true,
            og_sg: og,
            fg_recipe_sg: fg,
            kind: FermentationKind::Ale as i32,
            ..TrackFermentationRequest::default()
        }
    }

    fn manual_req(points: &[(f64, f64)], og: f64, fg: f64) -> TrackFermentationRequest {
        TrackFermentationRequest {
            readings: points
                .iter()
                .map(|&(t, sg)| reading(t, sg, GravityReadingSource::Manual))
                .collect(),
            has_device: false,
            og_sg: og,
            fg_recipe_sg: fg,
            kind: FermentationKind::Ale as i32,
            ..TrackFermentationRequest::default()
        }
    }

    async fn track_call(r: TrackFermentationRequest) -> Result<TrackFermentationResponse, Status> {
        FermentationHandler
            .track_fermentation(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    // -------------------------------------------------------------- TrackFermentation

    #[tokio::test]
    async fn track_device_forecast_matches_vector_07_case_16() {
        let r = track_call(device_req(&V07_16, 1.09, 1.017)).await.unwrap();
        assert_eq!(r.method, FermentationForecastMethod::Device as i32);
        let d = r.device.unwrap();
        assert!(d.ready);
        close(d.fg_sg.unwrap(), 1.064_101_325);
        close(d.fg_low_sg.unwrap(), 1.063_112_975);
        close(d.fg_high_sg.unwrap(), 1.064_760_225_000_000_1);
        close(d.k_per_h.unwrap(), 0.183_218_724_751_760_13);
        close(d.finish_h.unwrap(), 22.103_106_410_484_24);
        close(d.remaining_h.unwrap(), 10.603_106_410_484_24);
        assert_eq!(d.fg_warning, Some(true));
        // A comes from the same fit as the forecast (no separate fit in the handler).
        let hm = home::hourly_median(&V07_16);
        let fit = home::fit_decay(&hm, 1.017 - 0.015, hm[hm.len() - 1].1 - 0.0002, 400)
            .unwrap()
            .fit;
        close(d.a_sg.unwrap(), fit.a);
        assert_eq!(r.hourly_median.len(), 12);
        close(r.hourly_median[0].time_h, 0.5);
        // The forecast FG replaces the recipe FG as the reference (vector 11 case 70 shape).
        assert_eq!(r.fg_reference_source, FgReferenceSource::Forecast as i32);
        close(r.fg_reference_sg, 1.064_101_325);
        // Progress in integer gravity units: (109000 - 106809) / (109000 - 106410).
        close(r.progress_pct.unwrap(), 2191.0 / 2590.0 * 100.0);
        // Predicted trigger (ale delta 0.002) is available and positive.
        let t = r.trigger.unwrap();
        close(t.delta_sg, 0.002);
        close(t.threshold_sg, 1.066_101_325);
        assert_eq!(t.predicted, PredictedTriggerState::At as i32);
        assert!(t.predicted_at_h.unwrap() > 0.0);
        assert!(t.triggered_at_h.is_none());
        assert!(!t.needs_confirmation);
        assert_eq!(
            r.stability.unwrap().state,
            FgStabilityState::Insufficient as i32
        );
    }

    #[tokio::test]
    async fn track_insufficient_device_data_triggers_like_vector_11_case_40() {
        // Vector 11 case 40: fg_ref 1.012, delta 0.004, 3 in a row, first point of the run is
        // hour 15; the hourly median reports bucket start + 0.5.
        let pts = [
            (12.0, 1.017),
            (13.0, 1.0158),
            (14.0, 1.0162),
            (15.0, 1.0159),
            (16.0, 1.0158),
            (17.0, 1.0157),
        ];
        let mut req = device_req(&pts, 1.05, 1.012);
        req.kind = FermentationKind::Lager as i32;
        let r = track_call(req).await.unwrap();
        assert_eq!(r.method, FermentationForecastMethod::None as i32);
        assert!(!r.device.unwrap().ready);
        assert!(r.manual.is_none());
        assert_eq!(r.fg_reference_source, FgReferenceSource::Recipe as i32);
        close(r.fg_reference_sg, 1.012);
        let t = r.trigger.unwrap();
        close(t.delta_sg, 0.004);
        close(t.threshold_sg, 1.016);
        close(t.triggered_at_h.unwrap(), 15.5);
        assert!(!t.needs_confirmation);
        assert_eq!(t.predicted, PredictedTriggerState::Unspecified as i32);
        assert_eq!(
            r.stability.unwrap().state,
            FgStabilityState::Insufficient as i32
        );
    }

    #[tokio::test]
    async fn track_explicit_trigger_parameters_override_defaults() {
        let pts = [(0.0, 1.016), (1.0, 1.016), (2.0, 1.016)];
        // Vector 11 case 41 (exactly at the threshold, three in a row) and case 42 variant:
        // one consecutive point suffices when asked for.
        let mut req = device_req(&pts, 1.05, 1.012);
        req.trigger_delta_sg = Some(0.004);
        let r = track_call(req.clone()).await.unwrap();
        close(r.trigger.unwrap().triggered_at_h.unwrap(), 0.5);
        req.trigger_delta_sg = Some(0.003);
        req.trigger_consecutive = Some(1);
        let r = track_call(req).await.unwrap();
        assert!(r.trigger.unwrap().triggered_at_h.is_none());
    }

    #[tokio::test]
    async fn track_bias_correction_readings_are_not_model_points() {
        let pts = [
            (12.0, 1.017),
            (13.0, 1.0158),
            (14.0, 1.0162),
            (15.0, 1.0159),
            (16.0, 1.0158),
            (17.0, 1.0157),
        ];
        let mut req = device_req(&pts, 1.05, 1.012);
        req.kind = FermentationKind::Lager as i32;
        // Vector 10 reading_role: manual reading of a device batch is a bias point.
        req.readings
            .push(reading(18.0, 1.0, GravityReadingSource::Manual));
        let r = track_call(req).await.unwrap();
        assert_eq!(r.method, FermentationForecastMethod::None as i32);
        assert!(r.manual.is_none());
        assert_eq!(r.hourly_median.len(), 6);
        close(r.trigger.unwrap().triggered_at_h.unwrap(), 15.5);
    }

    #[tokio::test]
    async fn track_manual_estimate_matches_vector_10_case_33() {
        let r = track_call(manual_req(&[(48.0, 1.03), (96.0, 1.018)], 1.05, 1.012))
            .await
            .unwrap();
        assert_eq!(r.method, FermentationForecastMethod::Manual as i32);
        assert!(r.device.is_none());
        let m = r.manual.unwrap();
        assert_eq!(m.status, ManualForecastStatus::Estimate as i32);
        close(m.k_per_h.unwrap(), 0.0218);
        close(m.finish_h.unwrap(), 210.8);
        assert_eq!(m.finish_min_days, Some(6));
        assert_eq!(m.finish_max_days, Some(11));
        assert_eq!(m.remaining_min_days, Some(2));
        assert_eq!(m.remaining_max_days, Some(7));
        assert_eq!(m.inconsistent, Some(false));
        assert!(r.hourly_median.is_empty());
        // Progress from the last model reading: (105000 - 101800) / (105000 - 101200).
        close(r.progress_pct.unwrap(), 3200.0 / 3800.0 * 100.0);
        let t = r.trigger.unwrap();
        assert!(t.triggered_at_h.is_none());
        assert!(!t.needs_confirmation);
        assert_eq!(
            r.stability.unwrap().state,
            FgStabilityState::NotStable as i32
        );
    }

    #[tokio::test]
    async fn track_manual_unsorted_readings_match_vector_10_case_41() {
        let r = track_call(manual_req(&[(84.0, 1.014), (48.0, 1.03)], 1.05, 1.012))
            .await
            .unwrap();
        let m = r.manual.unwrap();
        close(m.k_per_h.unwrap(), 0.0354);
        close(m.finish_h.unwrap(), 134.3);
        assert_eq!((m.finish_min_days, m.finish_max_days), (Some(3), Some(8)));
        assert_eq!(
            (m.remaining_min_days, m.remaining_max_days),
            (Some(0), Some(4))
        );
    }

    #[tokio::test]
    async fn track_manual_stall_rule_matches_vector_10_cases_36_and_37() {
        // Case 36: looks finished; one manual reading under FG + delta triggers, to confirm.
        let r = track_call(manual_req(
            &[(96.0, 1.018), (168.0, 1.013), (216.0, 1.013)],
            1.05,
            1.012,
        ))
        .await
        .unwrap();
        let m = r.manual.unwrap();
        assert_eq!(m.status, ManualForecastStatus::LooksFinished as i32);
        close(m.fg_measured_sg.unwrap(), 1.013);
        assert_eq!(r.stability.unwrap().state, FgStabilityState::Stable as i32);
        let t = r.trigger.unwrap();
        close(t.triggered_at_h.unwrap(), 168.0);
        assert!(t.needs_confirmation);

        // Case 37: possibly stuck.
        let r = track_call(manual_req(
            &[(48.0, 1.03), (120.0, 1.024), (168.0, 1.024)],
            1.05,
            1.012,
        ))
        .await
        .unwrap();
        let m = r.manual.unwrap();
        assert_eq!(m.status, ManualForecastStatus::PossiblyStuck as i32);
        close(m.fg_measured_sg.unwrap(), 1.024);
    }

    #[tokio::test]
    async fn track_manual_insufficient_cases_match_vector_10() {
        // Case 34: one reading after the lag phase.
        let r = track_call(manual_req(&[(48.0, 1.03)], 1.05, 1.012))
            .await
            .unwrap();
        assert_eq!(
            r.manual.unwrap().status,
            ManualForecastStatus::NeedTwoReadings as i32
        );
        assert_eq!(
            r.stability.unwrap().state,
            FgStabilityState::Insufficient as i32
        );
        // Case 35: under 30 % of the expected drop.
        let r = track_call(manual_req(&[(24.0, 1.046), (36.0, 1.042)], 1.05, 1.012))
            .await
            .unwrap();
        assert_eq!(
            r.manual.unwrap().status,
            ManualForecastStatus::DropBelow30Percent as i32
        );
        // Case 63: target reached.
        let r = track_call(manual_req(&[(30.0, 1.03), (60.0, 1.012)], 1.05, 1.012))
            .await
            .unwrap();
        assert_eq!(
            r.manual.unwrap().status,
            ManualForecastStatus::TargetReached as i32
        );
    }

    #[tokio::test]
    async fn track_rejects_bad_input() {
        // Empty readings.
        let s = track_call(manual_req(&[], 1.05, 1.012)).await.unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        assert_eq!(field_of(&s), "readings");
        // NaN SG, NaN time, negative time, NaN OG.
        let mut req = manual_req(&[(48.0, f64::NAN)], 1.05, 1.012);
        let s = track_call(req.clone()).await.unwrap_err();
        assert_eq!(field_of(&s), "readings[0].sg");
        req = manual_req(&[(f64::NAN, 1.03)], 1.05, 1.012);
        assert_eq!(
            field_of(&track_call(req).await.unwrap_err()),
            "readings[0].time_h"
        );
        let req = manual_req(&[(-1.0, 1.03)], 1.05, 1.012);
        assert_eq!(
            field_of(&track_call(req).await.unwrap_err()),
            "readings[0].time_h"
        );
        let req = manual_req(&[(48.0, 1.03)], f64::NAN, 1.012);
        assert_eq!(field_of(&track_call(req).await.unwrap_err()), "og_sg");
        // UNSPECIFIED source and kind.
        let mut req = manual_req(&[(48.0, 1.03)], 1.05, 1.012);
        req.readings[0].source = GravityReadingSource::Unspecified as i32;
        assert_eq!(
            field_of(&track_call(req).await.unwrap_err()),
            "readings[0].source"
        );
        let mut req = manual_req(&[(48.0, 1.03)], 1.05, 1.012);
        req.kind = FermentationKind::Unspecified as i32;
        assert_eq!(field_of(&track_call(req).await.unwrap_err()), "kind");
        // Negative optional parameter.
        let mut req = manual_req(&[(48.0, 1.03)], 1.05, 1.012);
        req.trigger_delta_sg = Some(-0.001);
        assert_eq!(
            field_of(&track_call(req).await.unwrap_err()),
            "trigger_delta_sg"
        );
    }

    #[tokio::test]
    async fn track_engine_errors_map_to_status() {
        // Zero consecutive points: engine InvalidInput.
        let mut req = device_req(&V07_16, 1.09, 1.017);
        req.trigger_consecutive = Some(0);
        let s = track_call(req).await.unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        // Vector 10 case 48: sigma far out of range is rejected by the engine.
        let mut req = manual_req(&[(48.0, 1.03), (96.0, 1.018)], 1.05, 1.012);
        req.fg_sigma_sg = Some(5_000_000.0);
        let s = track_call(req).await.unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
    }

    // -------------------------------------------------------------- FermentationSchedule

    #[tokio::test]
    async fn schedule_matches_vector_11() {
        // Cases 104 (15 -> 2 C at 2 C/day = 156 h), 106, 110 (OG 1.048 -> 6 weeks), 278.
        let r = FermentationHandler
            .fermentation_schedule(Request::new(FermentationScheduleRequest {
                og_sg: 1.048,
                kind: FermentationKind::Lager as i32,
                ramps: vec![
                    TemperatureRamp {
                        from_c: 15.0,
                        to_c: 2.0,
                        rate_c_per_day: Some(2.0),
                    },
                    TemperatureRamp {
                        from_c: 10.0,
                        to_c: 10.0,
                        rate_c_per_day: None,
                    },
                ],
                ..FermentationScheduleRequest::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.ramp_h, vec![156.0, 0.0]);
        assert_eq!(r.lager_weeks, 6);
        close(r.default_delta_sg, 0.004);
        // Case 111 (OG 1.072 -> 9 weeks) and case 279 (ale delta).
        let r = FermentationHandler
            .fermentation_schedule(Request::new(FermentationScheduleRequest {
                og_sg: 1.072,
                kind: FermentationKind::Ale as i32,
                ..FermentationScheduleRequest::default()
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.lager_weeks, 9);
        close(r.default_delta_sg, 0.002);
    }

    #[tokio::test]
    async fn schedule_default_rate_is_two_degrees_per_day() {
        let r = FermentationHandler
            .fermentation_schedule(Request::new(FermentationScheduleRequest {
                og_sg: 1.05,
                kind: FermentationKind::Lager as i32,
                ramps: vec![TemperatureRamp {
                    from_c: 2.0,
                    to_c: 15.0,
                    rate_c_per_day: None,
                }],
                ..FermentationScheduleRequest::default()
            }))
            .await
            .unwrap()
            .into_inner();
        // Vector 11 case 105.
        assert_eq!(r.ramp_h, vec![156.0]);
        assert_eq!(r.lager_weeks, 7);
    }

    #[tokio::test]
    async fn schedule_rejects_bad_input() {
        let ok = FermentationScheduleRequest {
            og_sg: 1.05,
            kind: FermentationKind::Lager as i32,
            ..FermentationScheduleRequest::default()
        };
        let call = |r: FermentationScheduleRequest| async move {
            FermentationHandler
                .fermentation_schedule(Request::new(r))
                .await
                .unwrap_err()
        };
        let s = call(FermentationScheduleRequest {
            og_sg: f64::NAN,
            ..ok.clone()
        })
        .await;
        assert_eq!(field_of(&s), "og_sg");
        let s = call(FermentationScheduleRequest {
            kind: FermentationKind::Unspecified as i32,
            ..ok.clone()
        })
        .await;
        assert_eq!(field_of(&s), "kind");
        let ramp = |rate| TemperatureRamp {
            from_c: 20.0,
            to_c: 5.0,
            rate_c_per_day: rate,
        };
        let s = call(FermentationScheduleRequest {
            ramps: vec![ramp(Some(-1.0))],
            ..ok.clone()
        })
        .await;
        assert_eq!(field_of(&s), "ramps[0].rate_c_per_day");
        // Zero rate is an engine DivisionByZero.
        let s = call(FermentationScheduleRequest {
            ramps: vec![ramp(Some(0.0))],
            ..ok.clone()
        })
        .await;
        assert_eq!(s.code(), Code::InvalidArgument);
        // Zero points per week likewise; inverted clamp is rejected up front.
        let s = call(FermentationScheduleRequest {
            lager_points_per_week: Some(0),
            ..ok.clone()
        })
        .await;
        assert_eq!(s.code(), Code::InvalidArgument);
        let s = call(FermentationScheduleRequest {
            lager_min_weeks: Some(13),
            ..ok
        })
        .await;
        assert_eq!(field_of(&s), "lager_min_weeks");
    }

    // -------------------------------------------------------------- device calibration

    fn v07_58_points() -> Vec<IspindelPoint> {
        [
            (25.0, 1.0),
            (38.2, 1.02),
            (49.0, 1.04),
            (58.7, 1.06),
            (66.5, 1.08),
        ]
        .iter()
        .map(|&(a, g)| IspindelPoint {
            angle_deg: a,
            sg: g,
        })
        .collect()
    }

    #[tokio::test]
    async fn calibrate_ispindel_matches_vector_07_cases_58_and_59() {
        let r = FermentationHandler
            .calibrate_ispindel(Request::new(CalibrateIspindelRequest {
                points: v07_58_points(),
                degree: None,
            }))
            .await
            .unwrap()
            .into_inner();
        let co = [
            0.977_355_710_266_040_3,
            0.000_532_953_691_355_25,
            1.511_337_504_529_696_6e-5,
        ];
        assert_eq!(r.coefficients.len(), 3);
        for (a, b) in r.coefficients.iter().zip(co) {
            close(*a, b);
        }
        close(r.r2, 0.999_805_988_477_524_6);
        close(r.max_residual_sg, 0.000_716_097_208_408_428_4);
        close(r.angle_min_deg, 25.0);
        close(r.angle_max_deg, 66.5);
        assert!(r.warnings.is_empty());
        // Case 59: degree 3.
        let r = FermentationHandler
            .calibrate_ispindel(Request::new(CalibrateIspindelRequest {
                points: v07_58_points(),
                degree: Some(3),
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.coefficients.len(), 4);
        close(r.coefficients[0], 0.964_272_066_423_485_2);
        close(r.r2, 0.999_920_922_194_877_6);
    }

    #[tokio::test]
    async fn calibrate_ispindel_reports_narrow_angle_range() {
        let pts = [(30.0, 1.0), (32.0, 1.02), (34.0, 1.04), (36.0, 1.061)];
        let r = FermentationHandler
            .calibrate_ispindel(Request::new(CalibrateIspindelRequest {
                points: pts
                    .iter()
                    .map(|&(a, g)| IspindelPoint {
                        angle_deg: a,
                        sg: g,
                    })
                    .collect(),
                degree: Some(1),
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            r.warnings,
            vec![proto::CalibrationWarning::NarrowAngleRange as i32]
        );
    }

    #[tokio::test]
    async fn calibrate_ispindel_rejects_bad_input() {
        let call = |pts: Vec<IspindelPoint>| async move {
            FermentationHandler
                .calibrate_ispindel(Request::new(CalibrateIspindelRequest {
                    points: pts,
                    degree: None,
                }))
                .await
                .unwrap_err()
        };
        // Vector 07 case 61: too few points; empty list likewise.
        let three = v07_58_points()[..3].to_vec();
        assert_eq!(call(three).await.code(), Code::InvalidArgument);
        assert_eq!(call(vec![]).await.code(), Code::InvalidArgument);
        // Vector 07 case 65: repeated angles.
        let dup = [(30.0, 1.0), (30.0, 1.0), (50.0, 1.04), (50.0, 1.04)]
            .iter()
            .map(|&(a, g)| IspindelPoint {
                angle_deg: a,
                sg: g,
            })
            .collect();
        assert_eq!(call(dup).await.code(), Code::InvalidArgument);
        let mut pts = v07_58_points();
        pts[2].angle_deg = f64::NAN;
        assert_eq!(field_of(&call(pts).await), "points[2].angle_deg");
        let mut pts = v07_58_points();
        pts[1].sg = -1.0;
        assert_eq!(field_of(&call(pts).await), "points[1].sg");
    }

    fn convert_req(angles: Vec<f64>) -> ConvertIspindelAnglesRequest {
        ConvertIspindelAnglesRequest {
            coefficients: vec![
                0.977_355_710_266_040_3,
                0.000_532_953_691_355_25,
                1.511_337_504_529_696_6e-5,
            ],
            angle_min_deg: 25.0,
            angle_max_deg: 66.5,
            angles_deg: angles,
        }
    }

    #[tokio::test]
    async fn convert_ispindel_angles_matches_vector_07_cases_74_to_77() {
        let r = FermentationHandler
            .convert_ispindel_angles(Request::new(convert_req(vec![
                25.0,
                45.0,
                66.5,
                24.999_999_9,
            ])))
            .await
            .unwrap()
            .into_inner();
        let expected = [
            1.000_125_411_953_232_1,
            1.031_943_210_843_753,
            1.079_632_253_535_229,
            1.000_125_411_824_369_9,
        ];
        assert_eq!(r.readings.len(), 4);
        for (got, want) in r.readings.iter().zip(expected) {
            close(got.sg, want);
        }
        assert!(r.readings[..3].iter().all(|x| x.warnings.is_empty()));
        assert_eq!(
            r.readings[3].warnings,
            vec![proto::CalibrationWarning::AngleOutsideRange as i32]
        );
    }

    #[tokio::test]
    async fn convert_ispindel_angles_rejects_bad_input() {
        let call = |r: ConvertIspindelAnglesRequest| async move {
            FermentationHandler
                .convert_ispindel_angles(Request::new(r))
                .await
                .unwrap_err()
        };
        let s = call(convert_req(vec![])).await;
        assert_eq!(field_of(&s), "angles_deg");
        let s = call(convert_req(vec![f64::NAN])).await;
        assert_eq!(field_of(&s), "angles_deg[0]");
        let s = call(ConvertIspindelAnglesRequest {
            coefficients: vec![],
            ..convert_req(vec![30.0])
        })
        .await;
        assert_eq!(field_of(&s), "coefficients");
        let s = call(ConvertIspindelAnglesRequest {
            coefficients: vec![1.0, f64::INFINITY],
            ..convert_req(vec![30.0])
        })
        .await;
        assert_eq!(field_of(&s), "coefficients[1]");
        let s = call(ConvertIspindelAnglesRequest {
            angle_min_deg: f64::NAN,
            ..convert_req(vec![30.0])
        })
        .await;
        assert_eq!(field_of(&s), "angle_min_deg");
    }

    fn tilt_points(pts: &[(f64, f64)]) -> CalibrateTiltRequest {
        CalibrateTiltRequest {
            points: pts
                .iter()
                .map(|&(t, r)| TiltPoint {
                    tilt_sg: t,
                    reference_sg: r,
                })
                .collect(),
        }
    }

    #[tokio::test]
    async fn calibrate_tilt_matches_vector_07_cases_83_and_85() {
        let one = FermentationHandler
            .calibrate_tilt(Request::new(tilt_points(&[(1.0545, 1.052)])))
            .await
            .unwrap()
            .into_inner();
        close(one.slope, 1.0);
        close(one.offset_sg, -0.002_499_999_999_999_946_7);
        let three = FermentationHandler
            .calibrate_tilt(Request::new(tilt_points(&[
                (1.002, 1.0),
                (1.03, 1.0285),
                (1.0545, 1.052),
            ])))
            .await
            .unwrap()
            .into_inner();
        close(three.slope, 0.991_124_260_355_029_5);
        close(three.offset_sg, 0.007_131_656_804_733_6);
    }

    #[tokio::test]
    async fn calibrate_tilt_rejects_bad_input() {
        let call = |r: CalibrateTiltRequest| async move {
            FermentationHandler
                .calibrate_tilt(Request::new(r))
                .await
                .unwrap_err()
        };
        // Vector 07 case 248 (empty) and 249 (same tilt reading twice).
        assert_eq!(call(tilt_points(&[])).await.code(), Code::InvalidArgument);
        assert_eq!(
            call(tilt_points(&[(1.0, 1.0), (1.0, 1.1)])).await.code(),
            Code::InvalidArgument
        );
        let s = call(tilt_points(&[(f64::NAN, 1.0)])).await;
        assert_eq!(field_of(&s), "points[0].tilt_sg");
        let s = call(tilt_points(&[(1.0, -1.0)])).await;
        assert_eq!(field_of(&s), "points[0].reference_sg");
    }

    // -------------------------------------------------------------- keg and cask

    fn keg_req() -> KegPrimingRequest {
        // Vector 11 case 206.
        KegPrimingRequest {
            co2_vol: 2.4,
            fermentation_max_c: 20.0,
            conditioning_c: 20.0,
            serving_c: 4.0,
            beer_l: 19.0,
            headspace_l: 0.0,
            sugar: ConditioningSugar::Dextrose as i32,
            purged: true,
            mode: KegPrimingMode::Full as i32,
            partial_fraction: None,
            dry_matter_fraction: Some(1.0),
            keg_limit_bar: None,
        }
    }

    async fn keg(r: KegPrimingRequest) -> Result<KegPrimingResponse, Status> {
        FermentationHandler
            .keg_priming(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn keg_priming_full_matches_vector_11_case_206() {
        let r = keg(keg_req()).await.unwrap();
        close(r.sugar_g, 129.2);
        close(r.bottle_equivalent_g, 129.2);
        assert_eq!(r.headspace_share_pct, Some(0.0));
        close(r.serving_bar, 0.74);
        close(r.peak_co2_vol, 2.4);
        close(r.peak_bar, 1.8);
        close(r.peak_total_gauge_bar, 1.8);
        assert!(r.co2_topup_bar.is_none());
        assert!(r.warnings.is_empty());
    }

    #[tokio::test]
    async fn keg_priming_partial_matches_vector_11_case_209() {
        let r = keg(KegPrimingRequest {
            headspace_l: 2.0,
            mode: KegPrimingMode::Partial as i32,
            ..keg_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 45.4);
        close(r.bottle_equivalent_g, 129.2);
        close(r.headspace_share_pct.unwrap(), 5.3);
        close(r.peak_co2_vol, 1.34);
        close(r.peak_bar, 0.64);
        close(r.co2_topup_bar.unwrap(), 0.74);
        // Case 216: explicit fraction 0.5.
        let r = keg(KegPrimingRequest {
            headspace_l: 2.0,
            mode: KegPrimingMode::Partial as i32,
            partial_fraction: Some(0.5),
            ..keg_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 68.0);
    }

    #[tokio::test]
    async fn keg_priming_without_bottle_sugar_has_no_share_vector_11_case_218() {
        let r = keg(KegPrimingRequest {
            co2_vol: 0.5,
            headspace_l: 2.0,
            ..keg_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 0.5);
        close(r.bottle_equivalent_g, 0.0);
        assert!(r.headspace_share_pct.is_none());
        close(r.peak_co2_vol, 0.86);
        close(r.peak_bar, 0.12);
    }

    #[tokio::test]
    async fn keg_priming_warnings_match_vector_11() {
        // Case 211: dry matter unknown.
        let r = keg(KegPrimingRequest {
            headspace_l: 2.0,
            dry_matter_fraction: None,
            ..keg_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 136.1);
        assert_eq!(r.warnings.len(), 1);
        assert_eq!(
            r.warnings[0].code,
            ConditioningWarningCode::DryMatterUnknown as i32
        );
        // Case 213: peak pressure above the keg limit, air in the keg.
        let r = keg(KegPrimingRequest {
            headspace_l: 2.0,
            conditioning_c: 25.0,
            purged: false,
            keg_limit_bar: Some(2.5),
            ..keg_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 144.3);
        close(r.peak_total_gauge_bar, 3.04);
        let w = &r.warnings[0];
        assert_eq!(w.code, ConditioningWarningCode::PeakAboveLimit as i32);
        close(w.peak_bar.unwrap(), 3.038_084_216_765_877);
        close(w.limit_bar.unwrap(), 2.5);
        close(w.temp_c.unwrap(), 25.0);
        // Case 224: serving temperature below the polynomial range.
        let r = keg(KegPrimingRequest {
            headspace_l: 2.0,
            serving_c: -5.0,
            ..keg_req()
        })
        .await
        .unwrap();
        let w = &r.warnings[0];
        assert_eq!(
            w.code,
            ConditioningWarningCode::PolynomialBelowTemperature as i32
        );
        assert_eq!(w.point, CarbonationPoint::Serving as i32);
        close(w.temp_c.unwrap(), -5.0);
        // Case 228: target above the polynomial volume range.
        let r = keg(KegPrimingRequest {
            co2_vol: 3.6,
            headspace_l: 2.0,
            conditioning_c: 10.0,
            ..keg_req()
        })
        .await
        .unwrap();
        let w = &r.warnings[0];
        assert_eq!(
            w.code,
            ConditioningWarningCode::PolynomialAboveVolume as i32
        );
        assert_eq!(w.point, CarbonationPoint::Target as i32);
        close(w.co2_vol.unwrap(), 3.6);
    }

    #[tokio::test]
    async fn keg_priming_rejects_bad_input() {
        // Vector 11 case 222 (dry matter 0): engine DivisionByZero.
        let s = keg(KegPrimingRequest {
            dry_matter_fraction: Some(0.0),
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        // Case 235 (dry matter above 1) and case 236 (partial fraction 0): engine range.
        let s = keg(KegPrimingRequest {
            dry_matter_fraction: Some(1.5),
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        let s = keg(KegPrimingRequest {
            mode: KegPrimingMode::Partial as i32,
            partial_fraction: Some(0.0),
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        // Case 233 (negative headspace), NaN, UNSPECIFIED enums.
        let s = keg(KegPrimingRequest {
            headspace_l: -1.0,
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "headspace_l");
        let s = keg(KegPrimingRequest {
            co2_vol: f64::NAN,
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "co2_vol");
        let s = keg(KegPrimingRequest {
            sugar: ConditioningSugar::Unspecified as i32,
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "sugar");
        let s = keg(KegPrimingRequest {
            mode: KegPrimingMode::Unspecified as i32,
            ..keg_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "mode");
    }

    fn cask_req() -> CaskConditioningRequest {
        // Vector 11 case 247.
        CaskConditioningRequest {
            volume_l: 40.9,
            co2_vol: Some(1.1),
            fermentation_max_c: 19.0,
            rack_sg: 1.01,
            fg_reference_sg: 1.01,
            sugar: ConditioningSugar::Invert as i32,
            dry_matter_fraction: Some(1.0),
            finings_ml_per_l: None,
            cellar_c: 12.0,
            base_days: None,
            base_temp_c: None,
        }
    }

    async fn cask(r: CaskConditioningRequest) -> Result<CaskConditioningResponse, Status> {
        FermentationHandler
            .cask_conditioning(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn cask_matches_vector_11_cases_247_264_269() {
        let r = cask(cask_req()).await.unwrap();
        close(r.residual_co2_vol, 0.89);
        close(r.from_extract_co2_vol, 0.0);
        close(r.missing_co2_vol, 0.21);
        close(r.sugar_g, 34.9);
        close(r.sugar_g_per_l, 0.85);
        assert!(!r.excess);
        assert!(r.warnings.is_empty());
        // Case 264: finings 40.9 L at the default 2 mL/L.
        close(r.finings_ml, 81.8);
        // Case 269: 3 days at 12 C.
        close(r.conditioning_days, 3.0);
    }

    #[tokio::test]
    async fn cask_cellar_temperature_and_dose_follow_vector_11() {
        // Case 270: cellar 8 C, 3 days at 12 C; explicit finings dose (case 265 shape).
        let r = cask(CaskConditioningRequest {
            cellar_c: 8.0,
            volume_l: 20.0,
            finings_ml_per_l: Some(2.0),
            ..cask_req()
        })
        .await
        .unwrap();
        close(r.conditioning_days, 3.958_523_732_318_682_6);
        close(r.finings_ml, 40.0);
        // Case 271: cellar 20 C.
        let r = cask(CaskConditioningRequest {
            cellar_c: 20.0,
            base_days: Some(3.0),
            base_temp_c: Some(12.0),
            ..cask_req()
        })
        .await
        .unwrap();
        close(r.conditioning_days, 1.723_047_532_431_9);
    }

    #[tokio::test]
    async fn cask_excess_dry_matter_and_warning_match_vector_11() {
        // Case 248: rack above FG gives excess CO2, no sugar.
        let r = cask(CaskConditioningRequest {
            rack_sg: 1.011,
            ..cask_req()
        })
        .await
        .unwrap();
        close(r.from_extract_co2_vol, 0.5);
        close(r.missing_co2_vol, -0.28);
        close(r.sugar_g, 0.0);
        assert!(r.excess);
        // Case 250: 80 % dry matter.
        let r = cask(CaskConditioningRequest {
            dry_matter_fraction: Some(0.8),
            ..cask_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 43.7);
        close(r.sugar_g_per_l, 1.07);
        // Case 251: dry matter unknown.
        let r = cask(CaskConditioningRequest {
            dry_matter_fraction: None,
            ..cask_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 34.9);
        assert_eq!(
            r.warnings[0].code,
            ConditioningWarningCode::DryMatterUnknown as i32
        );
        // The target defaults to 1.1 volumes (spec 11 §3.1).
        let r = cask(CaskConditioningRequest {
            co2_vol: None,
            ..cask_req()
        })
        .await
        .unwrap();
        close(r.sugar_g, 34.9);
    }

    #[tokio::test]
    async fn cask_rejects_bad_input() {
        // Vector 11 cases 258 (volume 0), 257 (dry matter 0): engine DivisionByZero.
        let s = cask(CaskConditioningRequest {
            volume_l: 0.0,
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        let s = cask(CaskConditioningRequest {
            dry_matter_fraction: Some(0.0),
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(s.code(), Code::InvalidArgument);
        // Cases 259 and 260: negative volume and dry matter.
        let s = cask(CaskConditioningRequest {
            volume_l: -5.0,
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "volume_l");
        let s = cask(CaskConditioningRequest {
            dry_matter_fraction: Some(-0.1),
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "dry_matter_fraction");
        let s = cask(CaskConditioningRequest {
            rack_sg: f64::NAN,
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "rack_sg");
        let s = cask(CaskConditioningRequest {
            sugar: ConditioningSugar::Unspecified as i32,
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "sugar");
        let s = cask(CaskConditioningRequest {
            finings_ml_per_l: Some(-2.0),
            ..cask_req()
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "finings_ml_per_l");
    }

    // -------------------------------------------------------------- no-chill

    fn nochill_req(mode: NoChillMode) -> NoChillIbuRequest {
        // Vector 10 cases 184 and 185.
        NoChillIbuRequest {
            hops: vec![
                NoChillHop {
                    amount_g: 28.0,
                    alpha_pct: 10.0,
                    time_min: 60.0,
                },
                NoChillHop {
                    amount_g: 28.0,
                    alpha_pct: 10.0,
                    time_min: 0.0,
                },
            ],
            boil_sg: 1.05,
            volume_l: 20.0,
            mode: mode as i32,
            cube_fill_c: None,
            ambient_c: None,
            cube_l: None,
        }
    }

    async fn nochill(r: NoChillIbuRequest) -> Result<NoChillIbuResponse, Status> {
        FermentationHandler
            .no_chill_ibu(Request::new(r))
            .await
            .map(Response::into_inner)
    }

    #[tokio::test]
    async fn no_chill_rule_matches_vector_10_case_184() {
        let r = nochill(nochill_req(NoChillMode::Rule)).await.unwrap();
        close(r.base_ibu, 32.3);
        close(r.no_chill_ibu, 53.6);
        close(r.extra_min, 20.0);
    }

    #[tokio::test]
    async fn no_chill_model_matches_vector_10_cases_185_to_189() {
        let r = nochill(nochill_req(NoChillMode::Model)).await.unwrap();
        close(r.no_chill_ibu, 64.6);
        close(r.extra_min, 45.0);
        // Case 186: cube filled at 80 C.
        let r = nochill(NoChillIbuRequest {
            cube_fill_c: Some(80.0),
            ..nochill_req(NoChillMode::Model)
        })
        .await
        .unwrap();
        close(r.no_chill_ibu, 51.3);
        close(r.extra_min, 16.9);
        // Case 187: 10 C ambient.
        let r = nochill(NoChillIbuRequest {
            cube_fill_c: Some(95.0),
            ambient_c: Some(10.0),
            ..nochill_req(NoChillMode::Model)
        })
        .await
        .unwrap();
        close(r.no_chill_ibu, 62.7);
        close(r.extra_min, 38.4);
        // Case 188: 40 L cube; case 189: cube volume 0 means the batch volume.
        let r = nochill(NoChillIbuRequest {
            cube_l: Some(40.0),
            ..nochill_req(NoChillMode::Model)
        })
        .await
        .unwrap();
        close(r.no_chill_ibu, 67.0);
        close(r.extra_min, 56.7);
        let r = nochill(NoChillIbuRequest {
            cube_l: Some(0.0),
            ..nochill_req(NoChillMode::Model)
        })
        .await
        .unwrap();
        close(r.extra_min, 45.0);
    }

    #[tokio::test]
    async fn no_chill_without_hops_matches_vector_10_case_192() {
        let r = nochill(NoChillIbuRequest {
            hops: vec![],
            ..nochill_req(NoChillMode::Model)
        })
        .await
        .unwrap();
        close(r.base_ibu, 0.0);
        close(r.no_chill_ibu, 0.0);
        close(r.extra_min, 45.0);
    }

    #[tokio::test]
    async fn no_chill_rejects_bad_input() {
        // Vector 10 cases 193, 194: volume 0 and negative.
        for v in [0.0, -5.0] {
            let s = nochill(NoChillIbuRequest {
                volume_l: v,
                ..nochill_req(NoChillMode::Rule)
            })
            .await
            .unwrap_err();
            assert_eq!(s.code(), Code::InvalidArgument);
        }
        // Cases 196, 197: ambient at or above 40 C is an engine error.
        for amb in [45.0, 40.0] {
            let s = nochill(NoChillIbuRequest {
                ambient_c: Some(amb),
                ..nochill_req(NoChillMode::Model)
            })
            .await
            .unwrap_err();
            assert_eq!(s.code(), Code::InvalidArgument);
        }
        // Case 195: negative cube volume.
        let s = nochill(NoChillIbuRequest {
            cube_l: Some(-40.0),
            ..nochill_req(NoChillMode::Model)
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "cube_l");
        // NaN, negative amount, UNSPECIFIED mode.
        let s = nochill(NoChillIbuRequest {
            boil_sg: f64::NAN,
            ..nochill_req(NoChillMode::Rule)
        })
        .await
        .unwrap_err();
        assert_eq!(field_of(&s), "boil_sg");
        let mut req = nochill_req(NoChillMode::Rule);
        req.hops[1].amount_g = -1.0;
        assert_eq!(
            field_of(&nochill(req).await.unwrap_err()),
            "hops[1].amount_g"
        );
        let s = nochill(nochill_req(NoChillMode::Unspecified))
            .await
            .unwrap_err();
        assert_eq!(field_of(&s), "mode");
    }
}
