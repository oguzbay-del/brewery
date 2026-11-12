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
}
