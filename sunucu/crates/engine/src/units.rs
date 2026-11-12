//! Shared conversions and comparison helpers used by every section.
//!
//! Formulas are the BeerSmith forms in `formuller/01-yogunluk-alkol-renk.md` §Birim
//! dönüşümleri; the comparison rules are in `formuller/00-OKU-BENI.md`.

/// Continuous-quantity tolerance (slopes, ratios): `formuller/00-OKU-BENI.md`.
pub const TOL: f64 = 1e-9;

/// Millilitres per US fluid ounce.
pub const FLOZ_ML: f64 = 29.573_529_56;

/// Litres per US fluid ounce.
pub const FLOZ_L: f64 = FLOZ_ML / 1000.0;

/// One US gallon in litres (`128 fl oz × 29.57352956 mL`).
pub const GAL_L: f64 = 128.0 * FLOZ_ML / 1000.0;

/// Kilograms per pound (BeerSmith: `0.4535924`).
pub const LB_KG: f64 = 0.453_592_4;

/// Grams per ounce (`LB_KG / 16 × 1000` = 28.349525, BeerSmith's constant; spec 04 §0 and, since
/// T1.16 B, spec 02 §8 as well).
pub const OZ_G: f64 = LB_KG / 16.0 * 1000.0;

/// CO₂ in g/L per volume of gas (`44.01 / 22.414`, ideal gas, 0 °C, 1 atm); one constant for
/// sections 09, 10 and 11.
pub const CO2_G_PER_VOL: f64 = 44.01 / 22.414;

/// g CO₂ per g fermented extract (Balling, spec 11 §3.2).
pub const BALLING_CO2: f64 = 0.9565 / 2.0665;

/// Litres to US fluid ounces.
#[must_use]
pub fn l_to_floz(litres: f64) -> f64 {
    litres * 1000.0 / FLOZ_ML
}

/// US fluid ounces to litres.
#[must_use]
pub fn floz_to_l(floz: f64) -> f64 {
    floz * FLOZ_ML / 1000.0
}

/// Litres to US gallons.
#[must_use]
pub fn gal_from_l(litres: f64) -> f64 {
    litres * 1000.0 / FLOZ_ML / 128.0
}

/// Boiling point in °C at an altitude: `100 − alt · 0.3048 · 0.00324` (spec 02 §3.2, 05 §6.1).
/// `alt_ft` in feet.
#[must_use]
pub fn boil_temp_c(alt_ft: f64) -> f64 {
    100.0 - alt_ft * 0.3048 * 0.00324
}

/// Boiling point in °F at an altitude (spec 02 §3.2 `BoilTemp`); `alt_ft` in feet.
#[must_use]
pub fn boil_temp_f(alt_ft: f64) -> f64 {
    c_to_f(boil_temp_c(alt_ft))
}

/// Sum with the same compensated algorithm as `CPython`'s `sum()` over floats (3.12+),
/// so ill-conditioned fits match the reference to the last digits.
#[must_use]
#[allow(clippy::float_cmp)]
pub fn py_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut comp = 0.0_f64;
    for x in values {
        let t = sum + x;
        if sum.abs() >= x.abs() {
            comp += (sum - t) + x;
        } else {
            comp += (x - t) + sum;
        }
        sum = t;
    }
    if comp != 0.0 && comp.is_finite() {
        sum += comp;
    }
    sum
}

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
