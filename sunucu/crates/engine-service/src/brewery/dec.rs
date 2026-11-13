//! Decimal text on the wire: parsing, bounds and exact formatting (ADR-0005 item 6).
//!
//! Plain notation only (`"12.34"`, optional leading `-` where a signed value is meaningful);
//! no exponent, no whitespace, no `+`. No floating point is involved in either direction.

use std::str::FromStr;

use rust_decimal::Decimal;
use tonic::Status;

use crate::status::bad_request;

/// Largest magnitude of stock and money values (the cost engine multiplies without overflow
/// checks, so products of two accepted values must fit a `Decimal`).
const STOCK_CAP: i64 = 1_000_000_000_000;
/// Largest magnitude of tax-table values (the tax engine checks its own arithmetic).
const TAX_CAP: i64 = 1_000_000_000_000_000;

fn invalid(field: &str, why: &str) -> Status {
    bad_request(&format!("{field}: {why}"), field, why)
}

/// Parses plain decimal text; `signed` allows one leading `-`; `cap` bounds the magnitude.
fn parse(field: &str, s: &str, signed: bool, cap: i64) -> Result<Decimal, Status> {
    let digits = match s.strip_prefix('-') {
        Some(rest) if signed => rest,
        _ => s,
    };
    let (int, frac) = match digits.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (digits, None),
    };
    let plain = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    if !plain(int) || frac.is_some_and(|f| !plain(f)) {
        return Err(invalid(
            field,
            if signed {
                "must be a decimal number in plain notation"
            } else {
                "must be a non-negative decimal number in plain notation"
            },
        ));
    }
    let d = Decimal::from_str(s).map_err(|_| invalid(field, "decimal out of range"))?;
    if d.abs() > Decimal::from(cap) {
        return Err(invalid(field, "decimal out of range"));
    }
    Ok(d)
}

/// Non-negative stock or money value.
pub(super) fn stock(field: &str, s: &str) -> Result<Decimal, Status> {
    parse(field, s, false, STOCK_CAP)
}

/// Non-negative stock or money value; empty text means zero.
pub(super) fn stock_or_zero(field: &str, s: &str) -> Result<Decimal, Status> {
    if s.is_empty() {
        Ok(Decimal::ZERO)
    } else {
        stock(field, s)
    }
}

/// Non-negative tax-table value (volumes, hL already taxed, relief).
pub(super) fn tax_unsigned(field: &str, s: &str) -> Result<Decimal, Status> {
    parse(field, s, false, TAX_CAP)
}

/// Non-negative tax-table value; empty text means zero.
pub(super) fn tax_unsigned_or_zero(field: &str, s: &str) -> Result<Decimal, Status> {
    if s.is_empty() {
        Ok(Decimal::ZERO)
    } else {
        tax_unsigned(field, s)
    }
}

/// Signed tax-table value (rates, bands, ABV, Plato); the engine decides what is valid.
pub(super) fn tax_signed(field: &str, s: &str) -> Result<Decimal, Status> {
    parse(field, s, true, TAX_CAP)
}

/// Exact text of a decimal: scale kept, no exponent, no negative zero.
pub(super) fn fmt(mut d: Decimal) -> String {
    if d.is_zero() {
        d.set_sign_positive(true);
    }
    d.to_string()
}

/// Checked sum; overflow is an `INVALID_ARGUMENT` naming `field`.
pub(super) fn add(field: &str, a: Decimal, b: Decimal) -> Result<Decimal, Status> {
    a.checked_add(b)
        .ok_or_else(|| invalid(field, "amount out of range"))
}

/// Validates an ISO `YYYY-MM-DD` date shape (the engine compares dates as text).
pub(super) fn date(field: &str, s: &str) -> Result<(), Status> {
    let b = s.as_bytes();
    let shape = b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    let num = |r: std::ops::Range<usize>| s[r].parse::<u32>().unwrap_or(0);
    if shape && (1..=12).contains(&num(5..7)) && (1..=31).contains(&num(8..10)) {
        Ok(())
    } else {
        Err(invalid(field, "must be an ISO date YYYY-MM-DD"))
    }
}
