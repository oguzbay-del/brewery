//! `TaxReport`: maps a country rate row and period lines to `engine::brewery_tax`.

use engine::brewery_tax::{
    self as bt, Band, InputRounding, MAX_DP, Method, RoundMode, RowStatus, TaxLine, TaxTrigger,
    TaxableVolume, TaxableVolumeInput, VolumeEvent, VolumeEventKind,
};
use rust_decimal::Decimal;
use tonic::Status;

use super::dec;
use crate::proto::{
    TaxBand, TaxLineResult, TaxMethod, TaxProductLine, TaxRateRow, TaxReportRequest,
    TaxReportResponse, TaxRoundingScope, TaxRowStatus, TaxStepResult, tax_input_rounding::Kind,
    tax_product_line::Volume,
};
use crate::recipe::engine_code;
use crate::status::{bad_request, from_engine};

fn unknown() -> engine::Error {
    engine::Error::invalid("enum", "unknown enum value")
}

fn method(field: &str, raw: i32) -> Result<Method, Status> {
    engine_code(field, raw, |c| {
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

fn trigger(field: &str, raw: i32) -> Result<TaxTrigger, Status> {
    engine_code(field, raw, |c| {
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
#[derive(Clone)]
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

fn input_rounding(p: &str, row: &TaxRateRow) -> Result<InputRounding, Status> {
    let r = row.input_rounding.unwrap_or_default();
    match Kind::try_from(r.kind).unwrap_or(Kind::Unspecified) {
        Kind::Unspecified => Err(bad_request(
            "input rounding kind must be specified",
            &format!("{p}.input_rounding.kind"),
            "must not be UNSPECIFIED",
        )),
        Kind::Undefined => Ok(InputRounding::Undefined),
        Kind::NoRounding => Ok(InputRounding::NoRounding),
        Kind::Places => Ok(InputRounding::Places {
            dp: max_dp(
                &format!("{p}.input_rounding.decimal_places"),
                r.decimal_places,
            )?,
            mode: round_mode(&format!("{p}.input_rounding.mode"), r.mode)?,
        }),
    }
}

fn parse_row(p: &str, row: &TaxRateRow) -> Result<Row, Status> {
    let line_scope = match TaxRoundingScope::try_from(row.rounding_scope) {
        Ok(TaxRoundingScope::Line) => true,
        Ok(TaxRoundingScope::Total) => false,
        _ => {
            return Err(bad_request(
                "rounding scope must be specified",
                &format!("{p}.rounding_scope"),
                "must be LINE or TOTAL",
            ));
        }
    };
    Ok(Row {
        method: method(&format!("{p}.method"), row.method)?,
        rate: row
            .rate
            .as_deref()
            .map(|s| dec::tax_signed(&format!("{p}.rate"), s))
            .transpose()?,
        bands: bands(&format!("{p}.bands"), &row.bands)?,
        tiers: bands(&format!("{p}.tiers"), &row.tiers)?,
        inclusive: row.bands_inclusive,
        plato_trunc: row.plato_truncate.unwrap_or(true),
        relief: dec::tax_unsigned_or_zero(&format!("{p}.relief_per_lpa"), &row.relief_per_lpa)?,
        input: input_rounding(p, row)?,
        trigger: trigger(&format!("{p}.trigger"), row.trigger)?,
        deduct: row
            .deduct
            .iter()
            .enumerate()
            .map(|(i, k)| event_kind(&format!("{p}.deduct[{i}]"), *k))
            .collect::<Result<_, _>>()?,
        money_dp: max_dp(
            &format!("{p}.money_decimal_places"),
            row.money_decimal_places,
        )?,
        money_mode: round_mode(&format!("{p}.money_mode"), row.money_mode)?,
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

fn proto_method(m: Method) -> TaxMethod {
    match m {
        Method::Volume => TaxMethod::Volume,
        Method::PureAlcohol => TaxMethod::PureAlcohol,
        Method::HlPlato => TaxMethod::HlPlato,
        Method::PlatoBand => TaxMethod::PlatoBand,
        Method::TieredVolume => TaxMethod::TieredVolume,
    }
}

fn period(r: &TaxReportRequest) -> Result<(), Status> {
    dec::date("period_start", &r.period_start)?;
    dec::date("period_end", &r.period_end)?;
    if r.period_start > r.period_end {
        return Err(bad_request(
            "period_start must not be after period_end",
            "period_start",
            "after period_end",
        ));
    }
    Ok(())
}

fn status_of(row: &TaxRateRow) -> RowStatus {
    bt::row_status(row.official_source.as_deref(), row.source_date.as_deref())
}

/// Rounds a line tax with the row's money rule (scope LINE) and adds it to the total.
fn finish_line(row: &Row, unrounded: Decimal, total: &mut Decimal) -> Result<Decimal, Status> {
    let tax = if row.line_scope {
        bt::money(unrounded, row.money_dp, row.money_mode).map_err(|e| from_engine(&e))?
    } else {
        unrounded
    };
    *total = dec::add("total_tax", *total, tax)?;
    Ok(tax)
}

fn line_result(
    line: &TaxProductLine,
    vol: &TaxableVolume,
    unrounded: Decimal,
    tax: Decimal,
) -> TaxLineResult {
    TaxLineResult {
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
        lpa: None,
        applied_rate: None,
        ytd_start_hl: None,
        steps: Vec::new(),
    }
}

fn single_row(r: &TaxReportRequest, proto_row: &TaxRateRow) -> Result<TaxReportResponse, Status> {
    if !r.ytd_hl.is_empty() {
        return Err(bad_request(
            "ytd_hl needs a chain",
            "ytd_hl",
            "only with chain; use lines[].ytd_hl with a single row",
        ));
    }
    let row = parse_row("row", proto_row)?;
    period(r)?;
    let status = match status_of(proto_row) {
        RowStatus::Verified => TaxRowStatus::Verified,
        RowStatus::Unverified => TaxRowStatus::Unverified,
    };

    let mut lines = Vec::with_capacity(r.lines.len());
    let mut total = Decimal::ZERO;
    for (i, line) in r.lines.iter().enumerate() {
        let vol = volume(i, &row, line, (&r.period_start, &r.period_end))?;
        let detail = line_tax(i, &row, line, vol.taxable_l)?;
        let unrounded = detail.tax;
        let tax = finish_line(&row, unrounded, &mut total)?;
        let mut res = line_result(line, &vol, unrounded, tax);
        res.lpa = detail.lpa.map(dec::fmt);
        res.applied_rate = detail.rate.map(dec::fmt);
        lines.push(res);
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

/// The row data of a chain step plus its chain fields.
fn chain_step(i: usize, p: &TaxRateRow, row: &Row) -> Result<bt::ChainStep, Status> {
    let f = format!("chain[{i}]");
    dec::date(&format!("{f}.valid_from"), &p.valid_from)?;
    if let Some(to) = &p.valid_to {
        dec::date(&format!("{f}.valid_to"), to)?;
    }
    let conditions = p.discount_conditions.clone().unwrap_or_default();
    let limit = |name: &str, v: &Option<String>| {
        v.as_deref()
            .map(|s| dec::tax_signed(&format!("{f}.discount_conditions.{name}"), s))
            .transpose()
    };
    Ok(bt::ChainStep {
        position: p.chain_position,
        rate_class: p.rate_class.clone(),
        valid_from: p.valid_from.clone(),
        valid_to: p.valid_to.clone(),
        method: row.method,
        rate: row.rate,
        bands: row.bands.clone(),
        tiers: row.tiers.clone(),
        inclusive: row.inclusive,
        plato_trunc: row.plato_trunc,
        discount_per_lpa: row.relief,
        discount_conditions: bt::DiscountConditions {
            abv_below: limit("abv_below", &conditions.abv_below)?,
            ytd_hl_below: limit("ytd_hl_below", &conditions.ytd_hl_below)?,
        },
        input: row.input,
    })
}

fn disagree(what: &str) -> Status {
    bad_request(
        &format!("chain rows disagree on {what}"),
        "chain",
        "rows in force must share currency, trigger, deductions and money rounding",
    )
}

fn check_agree(head: (&TaxRateRow, &Row), other: (&TaxRateRow, &Row)) -> Result<(), Status> {
    let ((hp, h), (op, o)) = (head, other);
    if hp.currency != op.currency {
        return Err(disagree("currency"));
    }
    if h.trigger != o.trigger {
        return Err(disagree("trigger"));
    }
    if h.deduct != o.deduct {
        return Err(disagree("deduct"));
    }
    if (h.money_dp, h.money_mode, h.line_scope) != (o.money_dp, o.money_mode, o.line_scope) {
        return Err(disagree("money rounding"));
    }
    Ok(())
}

fn chain_report(r: &TaxReportRequest) -> Result<TaxReportResponse, Status> {
    period(r)?;
    let parsed = r
        .chain
        .iter()
        .enumerate()
        .map(|(i, p)| parse_row(&format!("chain[{i}]"), p))
        .collect::<Result<Vec<_>, _>>()?;
    let steps = r
        .chain
        .iter()
        .zip(&parsed)
        .enumerate()
        .map(|(i, (p, row))| chain_step(i, p, row))
        .collect::<Result<Vec<_>, _>>()?;
    let class = if r.rate_class.is_empty() {
        "beer"
    } else {
        &r.rate_class
    };
    let selected = bt::select_chain(&steps, class, &r.period_start, &r.period_end)
        .map_err(|e| from_engine(&e))?;
    let chosen: Vec<usize> = selected
        .iter()
        .filter_map(|s| steps.iter().position(|x| std::ptr::eq(x, *s)))
        .collect();
    let head = chosen[0];
    for &k in &chosen[1..] {
        check_agree((&r.chain[head], &parsed[head]), (&r.chain[k], &parsed[k]))?;
    }
    let row = &parsed[head];
    let status = if chosen
        .iter()
        .all(|&k| status_of(&r.chain[k]) == RowStatus::Verified)
    {
        TaxRowStatus::Verified
    } else {
        TaxRowStatus::Unverified
    };

    let mut volumes = Vec::with_capacity(r.lines.len());
    let mut inputs = Vec::with_capacity(r.lines.len());
    for (i, line) in r.lines.iter().enumerate() {
        if !line.ytd_hl.is_empty() {
            return Err(bad_request(
                "lines[].ytd_hl is not used with a chain",
                &format!("lines[{i}].ytd_hl"),
                "use the request ytd_hl",
            ));
        }
        let vol = volume(i, row, line, (&r.period_start, &r.period_end))?;
        let opt = |name: &str, v: &Option<String>| {
            v.as_deref()
                .map(|s| dec::tax_signed(&format!("lines[{i}].{name}"), s))
                .transpose()
        };
        inputs.push(bt::ChainLine {
            volume_l: vol.taxable_l,
            abv_pct: opt("abv_pct", &line.abv_pct)?,
            plato_p: opt("original_extract_p", &line.original_extract_p)?,
        });
        volumes.push(vol);
    }
    let ytd = dec::tax_unsigned_or_zero("ytd_hl", &r.ytd_hl)?;
    let report = bt::tax_chain(&steps, class, &r.period_start, &r.period_end, &inputs, ytd)
        .map_err(|e| from_engine(&e))?;

    let mut lines = Vec::with_capacity(r.lines.len());
    let mut total = Decimal::ZERO;
    for ((line, vol), done) in r.lines.iter().zip(&volumes).zip(&report.lines) {
        let tax = finish_line(row, done.tax, &mut total)?;
        let mut res = line_result(line, vol, done.tax, tax);
        res.ytd_start_hl = Some(dec::fmt(done.ytd_start_hl));
        res.steps = done
            .steps
            .iter()
            .map(|s| TaxStepResult {
                position: s.position,
                method: proto_method(s.method) as i32,
                tax: dec::fmt(s.detail.tax),
                lpa: s.detail.lpa.map(dec::fmt),
                applied_rate: s.detail.rate.map(dec::fmt),
                discount_applied: s.discount_applied,
            })
            .collect();
        lines.push(res);
    }
    if !row.line_scope {
        total = bt::money(total, row.money_dp, row.money_mode).map_err(|e| from_engine(&e))?;
    }
    Ok(TaxReportResponse {
        lines,
        total_tax: dec::fmt(total),
        currency: r.chain[head].currency.clone(),
        row_status: status as i32,
    })
}

pub(super) fn report(r: &TaxReportRequest) -> Result<TaxReportResponse, Status> {
    match (&r.row, r.chain.is_empty()) {
        (Some(row), true) => single_row(r, row),
        (None, false) => chain_report(r),
        (Some(_), false) => Err(bad_request(
            "set either row or chain, not both",
            "chain",
            "row and chain are exclusive",
        )),
        (None, true) => Err(bad_request("row is required", "row", "must be set")),
    }
}
