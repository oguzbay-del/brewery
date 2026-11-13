//! `TaxReport`: maps a country rate row and period lines to `engine::brewery_tax`.

use engine::brewery_tax::{
    self as bt, Band, InputRounding, MAX_DP, Method, RoundMode, RowStatus, TaxLine, TaxTrigger,
    TaxableVolume, TaxableVolumeInput, VolumeEvent, VolumeEventKind,
};
use rust_decimal::Decimal;
use tonic::Status;

use super::dec;
use crate::proto::{
    TaxBand, TaxLineResult, TaxProductLine, TaxRateRow, TaxReportRequest, TaxReportResponse,
    TaxRoundingScope, TaxRowStatus, tax_input_rounding::Kind, tax_product_line::Volume,
};
use crate::recipe::engine_code;
use crate::status::{bad_request, from_engine};

fn unknown() -> engine::Error {
    engine::Error::invalid("enum", "unknown enum value")
}

fn method(raw: i32) -> Result<Method, Status> {
    engine_code("row.method", raw, |c| {
        Ok(match c {
            0 => Method::Volume,
            1 => Method::PureAlcohol,
            2 => Method::HlPlato,
            3 => Method::PlatoBand,
            4 => Method::TieredVolume,
            _ => return Err(unknown()),
        })
    })
}

fn round_mode(field: &str, raw: i32) -> Result<RoundMode, Status> {
    engine_code(field, raw, |c| {
        Ok(match c {
            0 => RoundMode::HalfUp,
            1 => RoundMode::HalfEven,
            2 => RoundMode::HalfDown,
            3 => RoundMode::Up,
            4 => RoundMode::Down,
            5 => RoundMode::Ceiling,
            6 => RoundMode::Floor,
            _ => return Err(unknown()),
        })
    })
}

fn trigger(raw: i32) -> Result<TaxTrigger, Status> {
    engine_code("row.trigger", raw, |c| {
        Ok(match c {
            0 => TaxTrigger::Packaging,
            1 => TaxTrigger::Release,
            2 => TaxTrigger::Sale,
            _ => return Err(unknown()),
        })
    })
}

fn event_kind(field: &str, raw: i32) -> Result<VolumeEventKind, Status> {
    engine_code(field, raw, |c| {
        Ok(match c {
            0 => VolumeEventKind::Packaging,
            1 => VolumeEventKind::Release,
            2 => VolumeEventKind::Sale,
            3 => VolumeEventKind::Destroyed,
            4 => VolumeEventKind::Returned,
            5 => VolumeEventKind::Lab,
            _ => return Err(unknown()),
        })
    })
}

fn max_dp(field: &str, v: u32) -> Result<u32, Status> {
    if v > MAX_DP {
        return Err(bad_request(
            &format!("{field} must be at most {MAX_DP}"),
            field,
            "too many decimal places",
        ));
    }
    Ok(v)
}

fn bands(field: &str, list: &[TaxBand]) -> Result<Vec<Band>, Status> {
    list.iter()
        .enumerate()
        .map(|(i, b)| {
            Ok(Band {
                upper: b
                    .upper
                    .as_deref()
                    .map(|u| dec::tax_signed(&format!("{field}[{i}].upper"), u))
                    .transpose()?,
                rate: dec::tax_signed(&format!("{field}[{i}].rate"), &b.rate)?,
            })
        })
        .collect()
}

/// The parsed country row.
struct Row {
    method: Method,
    rate: Option<Decimal>,
    bands: Vec<Band>,
    tiers: Vec<Band>,
    inclusive: Option<bool>,
    plato_trunc: bool,
    relief: Decimal,
    input: InputRounding,
    trigger: TaxTrigger,
    deduct: Vec<VolumeEventKind>,
    money_dp: u32,
    money_mode: RoundMode,
    line_scope: bool,
}

fn input_rounding(row: &TaxRateRow) -> Result<InputRounding, Status> {
    let r = row.input_rounding.unwrap_or_default();
    match Kind::try_from(r.kind).unwrap_or(Kind::Unspecified) {
        Kind::Unspecified => Err(bad_request(
            "input rounding kind must be specified",
            "row.input_rounding.kind",
            "must not be UNSPECIFIED",
        )),
        Kind::Undefined => Ok(InputRounding::Undefined),
        Kind::NoRounding => Ok(InputRounding::NoRounding),
        Kind::Places => Ok(InputRounding::Places {
            dp: max_dp("row.input_rounding.decimal_places", r.decimal_places)?,
            mode: round_mode("row.input_rounding.mode", r.mode)?,
        }),
    }
}

fn parse_row(row: &TaxRateRow) -> Result<Row, Status> {
    let line_scope = match TaxRoundingScope::try_from(row.rounding_scope) {
        Ok(TaxRoundingScope::Line) => true,
        Ok(TaxRoundingScope::Total) => false,
        _ => {
            return Err(bad_request(
                "rounding scope must be specified",
                "row.rounding_scope",
                "must be LINE or TOTAL",
            ));
        }
    };
    Ok(Row {
        method: method(row.method)?,
        rate: row
            .rate
            .as_deref()
            .map(|s| dec::tax_signed("row.rate", s))
            .transpose()?,
        bands: bands("row.bands", &row.bands)?,
        tiers: bands("row.tiers", &row.tiers)?,
        inclusive: row.bands_inclusive,
        plato_trunc: row.plato_truncate.unwrap_or(true),
        relief: dec::tax_unsigned_or_zero("row.relief_per_lpa", &row.relief_per_lpa)?,
        input: input_rounding(row)?,
        trigger: trigger(row.trigger)?,
        deduct: row
            .deduct
            .iter()
            .enumerate()
            .map(|(i, k)| event_kind(&format!("row.deduct[{i}]"), *k))
            .collect::<Result<_, _>>()?,
        money_dp: max_dp("row.money_decimal_places", row.money_decimal_places)?,
        money_mode: round_mode("row.money_mode", row.money_mode)?,
        line_scope,
    })
}

fn volume(
    i: usize,
    row: &Row,
    line: &TaxProductLine,
    period: (&str, &str),
) -> Result<TaxableVolume, Status> {
    match &line.volume {
        Some(Volume::TaxableL(s)) => Ok(TaxableVolume {
            taxable_l: dec::tax_unsigned(&format!("lines[{i}].taxable_l"), s)?,
            taxed_l: Decimal::ZERO,
            destroyed_l: Decimal::ZERO,
            returned_l: Decimal::ZERO,
            lab_l: Decimal::ZERO,
        }),
        Some(Volume::Events(list)) => {
            let mut events = Vec::with_capacity(list.events.len());
            for (j, e) in list.events.iter().enumerate() {
                let f = format!("lines[{i}].events[{j}]");
                dec::date(&format!("{f}.date"), &e.date)?;
                events.push(VolumeEvent {
                    date: e.date.clone(),
                    kind: event_kind(&format!("{f}.kind"), e.kind)?,
                    volume_l: dec::tax_unsigned(&format!("{f}.volume_l"), &e.volume_l)?,
                });
            }
            bt::taxable_volume(&TaxableVolumeInput {
                events: &events,
                trigger: row.trigger,
                start: period.0,
                end: period.1,
                deduct: &row.deduct,
            })
            .map_err(|e| from_engine(&e))
        }
        None => Err(bad_request(
            "line volume must be set",
            &format!("lines[{i}].volume"),
            "must be events or taxable_l",
        )),
    }
}

fn line_tax(
    i: usize,
    row: &Row,
    line: &TaxProductLine,
    taxable_l: Decimal,
) -> Result<bt::TaxLineDetail, Status> {
    let opt = |name: &str, v: &Option<String>| {
        v.as_deref()
            .map(|s| dec::tax_signed(&format!("lines[{i}].{name}"), s))
            .transpose()
    };
    let mut p = TaxLine::new(row.method, taxable_l);
    p.abv_pct = opt("abv_pct", &line.abv_pct)?;
    p.plato_p = opt("original_extract_p", &line.original_extract_p)?;
    p.rate = row.rate;
    p.bands = &row.bands;
    p.tiers = &row.tiers;
    p.ytd_hl = dec::tax_unsigned_or_zero(&format!("lines[{i}].ytd_hl"), &line.ytd_hl)?;
    p.plato_trunc = row.plato_trunc;
    p.discount_per_lpa = row.relief;
    p.input = row.input;
    p.inclusive = row.inclusive;
    bt::tax_line_detail(&p).map_err(|e| from_engine(&e))
}

pub(super) fn report(r: &TaxReportRequest) -> Result<TaxReportResponse, Status> {
    let proto_row = r
        .row
        .as_ref()
        .ok_or_else(|| bad_request("row is required", "row", "must be set"))?;
    let row = parse_row(proto_row)?;
    dec::date("period_start", &r.period_start)?;
    dec::date("period_end", &r.period_end)?;
    if r.period_start > r.period_end {
        return Err(bad_request(
            "period_start must not be after period_end",
            "period_start",
            "after period_end",
        ));
    }
    let status = match bt::row_status(
        proto_row.official_source.as_deref(),
        proto_row.source_date.as_deref(),
    ) {
        RowStatus::Verified => TaxRowStatus::Verified,
        RowStatus::Unverified => TaxRowStatus::Unverified,
    };

    let mut lines = Vec::with_capacity(r.lines.len());
    let mut total = Decimal::ZERO;
    for (i, line) in r.lines.iter().enumerate() {
        let vol = volume(i, &row, line, (&r.period_start, &r.period_end))?;
        let detail = line_tax(i, &row, line, vol.taxable_l)?;
        let unrounded = detail.tax;
        let tax = if row.line_scope {
            bt::money(unrounded, row.money_dp, row.money_mode).map_err(|e| from_engine(&e))?
        } else {
            unrounded
        };
        total = dec::add("total_tax", total, tax)?;
        lines.push(TaxLineResult {
            product: line.product.clone(),
            package: line.package.clone(),
            abv_estimated: line.abv_estimated,
            taxable_l: dec::fmt(vol.taxable_l),
            trigger_l: dec::fmt(vol.taxed_l),
            destroyed_l: dec::fmt(vol.destroyed_l),
            returned_l: dec::fmt(vol.returned_l),
            lab_l: dec::fmt(vol.lab_l),
            tax_unrounded: dec::fmt(unrounded),
            tax: dec::fmt(tax),
            lpa: detail.lpa.map(dec::fmt),
            applied_rate: detail.rate.map(dec::fmt),
        });
    }
    if !row.line_scope {
        total = bt::money(total, row.money_dp, row.money_mode).map_err(|e| from_engine(&e))?;
    }
    Ok(TaxReportResponse {
        lines,
        total_tax: dec::fmt(total),
        currency: proto_row.currency.clone(),
        row_status: status as i32,
    })
}
