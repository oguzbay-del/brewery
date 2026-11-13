//! Conversion layer between the metric wire units and the engine's BeerSmith units
//! (ADR-0004 item 4, ADR-0005 item 3). Services convert only through this module.

/// Degrees Celsius to degrees Fahrenheit.
#[must_use]
pub fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

/// Degrees Fahrenheit to degrees Celsius.
#[must_use]
pub fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

/// Litres to US fluid ounces.
#[must_use]
pub fn l_to_floz(l: f64) -> f64 {
    engine::units::l_to_floz(l)
}

/// US fluid ounces to litres.
#[must_use]
pub fn floz_to_l(floz: f64) -> f64 {
    engine::units::floz_to_l(floz)
}

/// Metres per foot (exact).
const FT_M: f64 = 0.3048;

/// US fluid ounces per millilitre.
fn floz_per_ml() -> f64 {
    1.0 / engine::units::FLOZ_ML
}

/// Kilograms to ounces (avoirdupois).
#[must_use]
pub fn kg_to_oz(kg: f64) -> f64 {
    kg * 1000.0 / engine::units::OZ_G
}

/// Ounces (avoirdupois) to kilograms.
#[must_use]
pub fn oz_to_kg(oz: f64) -> f64 {
    oz * engine::units::OZ_G / 1000.0
}

/// Grams to ounces (avoirdupois).
#[must_use]
pub fn g_to_oz(g: f64) -> f64 {
    g / engine::units::OZ_G
}

/// Ounces (avoirdupois) to grams.
#[must_use]
pub fn oz_to_g(oz: f64) -> f64 {
    oz * engine::units::OZ_G
}

/// Millilitres to US fluid ounces.
#[must_use]
pub fn ml_to_floz(ml: f64) -> f64 {
    ml * floz_per_ml()
}

/// US fluid ounces to millilitres.
#[must_use]
pub fn floz_to_ml(floz: f64) -> f64 {
    floz * engine::units::FLOZ_ML
}

/// Metres to feet.
#[must_use]
pub fn m_to_ft(m: f64) -> f64 {
    m / FT_M
}

/// Feet to metres.
#[must_use]
pub fn ft_to_m(ft: f64) -> f64 {
    ft * FT_M
}

/// A rate per degree Celsius (e.g. attenuation change in %/°C) to the rate per degree
/// Fahrenheit: a Fahrenheit degree is 5/9 of a Celsius degree, so the rate scales by 5/9.
#[must_use]
pub fn per_c_to_per_f(rate_per_c: f64) -> f64 {
    rate_per_c * 5.0 / 9.0
}

/// A rate per degree Fahrenheit to the rate per degree Celsius (inverse of [`per_c_to_per_f`]).
#[must_use]
pub fn per_f_to_per_c(rate_per_f: f64) -> f64 {
    rate_per_f * 9.0 / 5.0
}

/// Hopped-extract rating, IBU·L/kg to IBU·gal/lb (1 IBU·gal/lb = 8.3454 IBU·L/kg).
#[must_use]
pub fn ibu_l_per_kg_to_gal_per_lb(v: f64) -> f64 {
    v * engine::units::LB_KG / engine::units::GAL_L
}

/// Hopped-extract rating, IBU·gal/lb to IBU·L/kg.
#[must_use]
pub fn ibu_gal_per_lb_to_l_per_kg(v: f64) -> f64 {
    v * engine::units::GAL_L / engine::units::LB_KG
}

/// Energy or mass per 12 US fl oz (the engine's calorie basis) to the value per 100 mL.
#[must_use]
pub fn per_12floz_to_per_100ml(v: f64) -> f64 {
    v / (12.0 * engine::units::FLOZ_ML / 100.0)
}

/// Value per 100 mL to the value per 12 US fl oz (inverse of [`per_12floz_to_per_100ml`]).
#[must_use]
pub fn per_100ml_to_per_12floz(v: f64) -> f64 {
    v * (12.0 * engine::units::FLOZ_ML / 100.0)
}

/// Bar per pound-force per square inch (1 psi = 6894.757293168 Pa, exact by definition).
const PSI_BAR: f64 = 0.068_947_572_931_683_6;

/// Pressure, bar to psi.
#[must_use]
pub fn bar_to_psi(bar: f64) -> f64 {
    bar / PSI_BAR
}

/// Pressure, psi to bar.
#[must_use]
pub fn psi_to_bar(psi: f64) -> f64 {
    psi * PSI_BAR
}

/// Litres to US gallons.
#[must_use]
pub fn l_to_gal(l: f64) -> f64 {
    l / engine::units::GAL_L
}

/// US gallons to litres.
#[must_use]
pub fn gal_to_l(gal: f64) -> f64 {
    gal * engine::units::GAL_L
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_round_trip() {
        assert!((c_to_f(25.0) - 77.0).abs() < 1e-12);
        assert!((f_to_c(77.0) - 25.0).abs() < 1e-12);
        assert!((f_to_c(c_to_f(18.3)) - 18.3).abs() < 1e-12);
    }

    #[test]
    fn volume_round_trip() {
        assert!((floz_to_l(l_to_floz(5.0)) - 5.0).abs() < 1e-12);
        assert!((l_to_floz(0.029_573_529_56) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn mass_length_round_trip() {
        assert!((oz_to_kg(kg_to_oz(0.75)) - 0.75).abs() < 1e-12);
        assert!((oz_to_g(g_to_oz(28.0)) - 28.0).abs() < 1e-12);
        assert!((oz_to_g(1.0) - 28.349_525).abs() < 1e-6);
        assert!((ft_to_m(m_to_ft(914.4)) - 914.4).abs() < 1e-9);
        assert!((m_to_ft(0.3048) - 1.0).abs() < 1e-12);
        assert!((floz_to_ml(ml_to_floz(100.0)) - 100.0).abs() < 1e-12);
    }

    #[test]
    fn rate_and_basis_round_trip() {
        assert!((per_c_to_per_f(-2.25) - (-1.25)).abs() < 1e-12);
        assert!((per_f_to_per_c(per_c_to_per_f(-3.0)) + 3.0).abs() < 1e-12);
        assert!((ibu_l_per_kg_to_gal_per_lb(8.3454) - 1.0).abs() < 1e-4);
        assert!(
            (ibu_gal_per_lb_to_l_per_kg(ibu_l_per_kg_to_gal_per_lb(250.0)) - 250.0).abs() < 1e-9
        );
        let per_100 = per_12floz_to_per_100ml(150.0);
        assert!((per_100 - 150.0 / 3.548_823_547_2).abs() < 1e-9);
        assert!((per_100ml_to_per_12floz(per_100) - 150.0).abs() < 1e-9);
    }

    #[test]
    fn pressure_and_gallon_round_trip() {
        assert!((psi_to_bar(bar_to_psi(1.7)) - 1.7).abs() < 1e-12);
        assert!((psi_to_bar(14.503_773_773_022) - 1.0).abs() < 1e-9);
        assert!((gal_to_l(l_to_gal(20.0)) - 20.0).abs() < 1e-12);
        assert!((gal_to_l(1.0) - 3.785_411_784).abs() < 1e-9);
    }
}
