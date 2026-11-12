//! Shared conversions and comparison helpers used by every section.
//!
//! Formulas are the BeerSmith forms in `formuller/01-yogunluk-alkol-renk.md` §Birim
//! dönüşümleri; the comparison rules are in `formuller/00-OKU-BENI.md`.

/// Continuous-quantity tolerance (slopes, ratios): `formuller/00-OKU-BENI.md`.
pub const TOL: f64 = 1e-9;

/// One US gallon in litres (`128 fl oz × 29.57352956 mL`).
pub const GAL_L: f64 = 128.0 * 29.573_529_56 / 1000.0;

/// Specific gravity to degrees Plato (BeerSmith cubic).
#[must_use]
pub fn to_plato(sg: f64) -> f64 {
    135.997 * sg.powi(3) - 630.272 * sg.powi(2) + 1111.14 * sg - 616.868
}

/// Degrees Plato to specific gravity (BeerSmith inverse).
#[must_use]
pub fn to_sg(plato: f64) -> f64 {
    plato / (258.6 - (plato / 258.6) * 227.1) + 1.0
}

/// °C to °F.
#[must_use]
pub fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

/// °F to °C.
#[must_use]
pub fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

/// Gravity in integer units, 1 unit = 0.00001 SG: `u(SG) = round(SG × 100000)`.
///
/// Rounds half to even, exactly like Python's `round`, so threshold decisions match the
/// reference. Gravity thresholds are always compared in these units, never as `f64`.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn gravity_units(sg: f64) -> i64 {
    (sg * 100_000.0).round_ties_even() as i64
}

/// Checked [`gravity_units`]: rejects NaN, infinities and values that would saturate `i64`.
pub fn gravity_units_checked(sg: f64) -> crate::Result<i64> {
    crate::error::ensure_finite("sg", sg)?;
    if (sg * 100_000.0).abs() >= 9.0e18 {
        return Err(crate::Error::invalid("sg", "out of range"));
    }
    Ok(gravity_units(sg))
}

/// Python-compatible `round(x, n)` for `n ≥ 0`.
///
/// Rounds the exact binary value to `n` decimals, ties to even, as Python does
/// (`round(2.675, 2) = 2.67`, `round(0.125, 2) = 0.12`); a scaled `round_ties_even`
/// would differ on such values. Use only where the reference rounds an intermediate
/// value; outputs are never rounded.
#[must_use]
pub fn round_py(x: f64, n: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{x:.n$}").parse().unwrap_or(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_units_ties_to_even() {
        assert_eq!(gravity_units(1.000_005), 100_000); // 100000.5 → even
        assert_eq!(gravity_units(1.050), 105_000);
    }

    #[test]
    fn round_py_matches_python() {
        assert!((round_py(2.675, 2) - 2.67).abs() < 1e-12);
        assert!((round_py(0.125, 2) - 0.12).abs() < 1e-12);
        assert!((round_py(0.375, 2) - 0.38).abs() < 1e-12);
        assert!((round_py(2.5, 0) - 2.0).abs() < 1e-12);
    }
}
