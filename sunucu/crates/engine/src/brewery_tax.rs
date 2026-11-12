//! Section 09: brewery release 2, label energy, tax, spunding (`formuller/09-isletme-2-hesaplar.md`).
//!
//! Units follow the specification of each function; see the section file.
//!
//! Money, rates, volumes, ABV and °P in the tax engine use [`Decimal`], never binary floats.
//! Results reproduce Python's `decimal` module including the number of decimal places
//! (`Decimal("20.0") * Decimal("12")` has scale 1, `max()` keeps the first of two equal values, ...),
//! because the parity vectors compare the printed string (ADR-0003).

use rust_decimal::{Decimal, RoundingStrategy};

use crate::error::Error;
use crate::units::{BALLING_CO2, CO2_G_PER_VOL, c_to_f, gravity_units, round_py, to_plato, to_sg};
use crate::yeast;

// ---------------------------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------------------------

/// Energy conversion in kJ per gram (EU 1169/2011 Annex XIV): alcohol.
pub const KJ_ALCOHOL: f64 = 29.0;
/// kJ per gram: carbohydrate.
pub const KJ_CARBOHYDRATE: f64 = 17.0;
/// kJ per gram: protein.
pub const KJ_PROTEIN: f64 = 17.0;
/// kcal per gram: alcohol.
pub const KCAL_ALCOHOL: f64 = 7.0;
/// kcal per gram: carbohydrate.
pub const KCAL_CARBOHYDRATE: f64 = 4.0;
/// kcal per gram: protein.
pub const KCAL_PROTEIN: f64 = 4.0;
/// Ethanol density, g/mL at 20 °C.
pub const ETOH_DENS: f64 = 0.78924;
/// Water density, g/mL at 20 °C.
pub const WATER_DENS: f64 = 0.9982;

/// Hectolitres per US beer barrel (31 US gal), exact finite decimal (T1.12 D1: factors are finite
/// decimals in the rate table and are applied by multiplication, never derived by division).
#[must_use]
pub fn hl_per_bbl() -> Decimal {
    Decimal::new(117_347_765, 8)
}

/// Absolute tolerance of the pressure limit comparison, psi (T1.12 D13).
pub const PSI_TOL: f64 = 1e-9;

// ---------------------------------------------------------------------------------------------
// Density helpers in °P as the 09 reference writes them (01 `density` works in SG and
// clamps OE ≥ AE; these do not)
// ---------------------------------------------------------------------------------------------

/// Real extract from original and apparent extract, °P (`0.1808·OE + 0.8192·AE`).
#[must_use]
pub fn real_extract(original_extract_p: f64, apparent_extract_p: f64) -> f64 {
    0.1808 * original_extract_p + 0.8192 * apparent_extract_p
}

/// Alcohol by weight (g/100 g) from original and real extract, °P.
#[must_use]
pub fn abw(original_extract_p: f64, real_extract_p: f64) -> f64 {
    (original_extract_p - real_extract_p) / (2.0665 - 0.010_665 * original_extract_p)
}

/// Alcohol values from OG and FG, see [`abv_from`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlcoholFromGravity {
    /// ABV, % vol.
    pub abv_pct: f64,
    /// ABW, g/100 g.
    pub abw_pct: f64,
    /// Real extract, °P.
    pub real_extract_p: f64,
}

/// ABV (% vol), ABW (g/100 g) and real extract (°P) from OG and FG (SG).
#[must_use]
pub fn abv_from(og: f64, fg: f64) -> AlcoholFromGravity {
    let oe = to_plato(og);
    let ae = to_plato(fg);
    let re = real_extract(oe, ae);
    let w = abw(oe, re);
    AlcoholFromGravity {
        abv_pct: w * fg / 0.79661,
        abw_pct: w,
        real_extract_p: re,
    }
}

// ---------------------------------------------------------------------------------------------
// 1. Label energy
// ---------------------------------------------------------------------------------------------

/// Label values per 100 mL of beer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelEnergy {
    /// ABV, % vol, rounded to 2 places.
    pub abv_pct: f64,
    /// Alcohol, g/100 mL, 2 places.
    pub alcohol_g: f64,
    /// Real extract, g/100 mL, 2 places.
    pub real_extract_g: f64,
    /// Carbohydrate, g/100 mL, 1 place.
    pub carbohydrate_g: f64,
    /// Protein, g/100 mL, 1 place.
    pub protein_g: f64,
    /// Energy, kJ/100 mL (rounded half to even).
    pub kj: i64,
    /// Energy, kcal/100 mL (rounded half to even).
    pub kcal: i64,
}

/// EU 1169/2011 label energy per 100 mL from OG/FG (SG).
///
/// `protein_g100ml` is the protein content in g/100 mL (reference default 0.40). A measured ABV
/// (% vol) replaces the estimate and converts to grams with the ethanol density; otherwise
/// ABW × SG × 0.9982 is used.
///
/// # Errors
/// [`Error::InvalidInput`] for non-finite input or a non-finite result (T1.16).
#[allow(clippy::cast_possible_truncation)]
pub fn label_energy(
    og: f64,
    fg: f64,
    protein_g100ml: f64,
    abv_measured_pct: Option<f64>,
) -> Result<LabelEnergy, Error> {
    let inputs = [og, fg, protein_g100ml, abv_measured_pct.unwrap_or(0.0)];
    if !inputs.iter().all(|x| x.is_finite() && x.abs() <= 1.0e6) {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let AlcoholFromGravity {
        abv_pct: mut abv,
        abw_pct: w,
        real_extract_p: re,
    } = abv_from(og, fg);
    let alc_g = if let Some(measured) = abv_measured_pct {
        abv = measured;
        abv * ETOH_DENS
    } else {
        w * fg * WATER_DENS
    };
    let re_g = re * fg;
    let carb = (re_g - protein_g100ml).max(0.0);
    let kj = KJ_ALCOHOL * alc_g + KJ_CARBOHYDRATE * carb + KJ_PROTEIN * protein_g100ml;
    let kcal = KCAL_ALCOHOL * alc_g + KCAL_CARBOHYDRATE * carb + KCAL_PROTEIN * protein_g100ml;
    if ![abv, alc_g, re_g, carb, kj, kcal]
        .iter()
        .all(|x| x.is_finite())
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    Ok(LabelEnergy {
        abv_pct: round_py(abv, 2),
        alcohol_g: round_py(alc_g, 2),
        real_extract_g: round_py(re_g, 2),
        carbohydrate_g: round_py(carb, 1),
        protein_g: round_py(protein_g100ml, 1),
        kj: kj.round_ties_even() as i64,
        kcal: kcal.round_ties_even() as i64,
    })
}

// ---------------------------------------------------------------------------------------------
// 2. Tax engine
// ---------------------------------------------------------------------------------------------

/// Decimal rounding direction (Python `decimal` rounding modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundMode {
    /// `ROUND_HALF_UP`: nearest, ties away from zero.
    HalfUp,
    /// `ROUND_HALF_EVEN`: nearest, ties to even.
    HalfEven,
    /// `ROUND_HALF_DOWN`: nearest, ties toward zero.
    HalfDown,
    /// `ROUND_UP`: away from zero.
    Up,
    /// `ROUND_DOWN`: toward zero (truncate).
    Down,
    /// `ROUND_CEILING`: toward +infinity.
    Ceiling,
    /// `ROUND_FLOOR`: toward -infinity.
    Floor,
}

impl RoundMode {
    fn strategy(self) -> RoundingStrategy {
        match self {
            Self::HalfUp => RoundingStrategy::MidpointAwayFromZero,
            Self::HalfEven => RoundingStrategy::MidpointNearestEven,
            Self::HalfDown => RoundingStrategy::MidpointTowardZero,
            Self::Up => RoundingStrategy::AwayFromZero,
            Self::Down => RoundingStrategy::ToZero,
            Self::Ceiling => RoundingStrategy::ToPositiveInfinity,
            Self::Floor => RoundingStrategy::ToNegativeInfinity,
        }
    }

    /// Parses a Python `decimal` constant name such as `ROUND_HALF_UP`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "ROUND_HALF_UP" => Self::HalfUp,
            "ROUND_HALF_EVEN" => Self::HalfEven,
            "ROUND_HALF_DOWN" => Self::HalfDown,
            "ROUND_UP" => Self::Up,
            "ROUND_DOWN" => Self::Down,
            "ROUND_CEILING" => Self::Ceiling,
            "ROUND_FLOOR" => Self::Floor,
            _ => return None,
        })
    }
}

/// Input rounding rule for ABV and °P (country row data). There is no silent default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRounding {
    /// The row does not say: methods that use ABV or °P fail (F1 decision 4).
    Undefined,
    /// Explicitly no rounding.
    NoRounding,
    /// Round to `dp` decimal places in the given direction.
    Places {
        /// Decimal places.
        dp: u32,
        /// Rounding direction.
        mode: RoundMode,
    },
}

/// Tax method (`formuller/09-isletme-2-hesaplar.md` §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// `hacim`: rate × hL.
    Volume,
    /// `saf_alkol`: rate × litres of pure alcohol, optional ABV band and per-LPA relief.
    PureAlcohol,
    /// `hl_plato`: rate × hL × °P.
    HlPlato,
    /// `plato_bandi`: per-hL rate from the °P band.
    PlatoBand,
    /// `kademeli_hacim`: tiered by cumulative yearly volume.
    TieredVolume,
}

impl Method {
    /// Parses the data-table method name (`hacim`, `saf_alkol`, ...).
    pub fn parse(name: &str) -> Result<Self, Error> {
        match name {
            "hacim" => Ok(Self::Volume),
            "saf_alkol" => Ok(Self::PureAlcohol),
            "hl_plato" => Ok(Self::HlPlato),
            "plato_bandi" => Ok(Self::PlatoBand),
            "kademeli_hacim" => Ok(Self::TieredVolume),
            _ => Err(Error::invalid("method", "unknown method")),
        }
    }
}

/// One band or tier: `upper` is the upper bound (`None` = open-ended), `rate` the rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    /// Upper bound; the last band must be open-ended.
    pub upper: Option<Decimal>,
    /// Rate for this band.
    pub rate: Decimal,
}

/// Largest number of decimals a rounding rule may ask for (T1.16, was unbounded).
pub const MAX_DP: u32 = 12;

/// Input of [`tax_line`]. Build with [`TaxLine::new`] and set the fields the method needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxLine<'a> {
    /// Method.
    pub method: Method,
    /// Volume, litres.
    pub volume_l: Decimal,
    /// ABV, % vol (`PureAlcohol`).
    pub abv_pct: Option<Decimal>,
    /// Original extract, °P (`HlPlato`, `PlatoBand`).
    pub plato_p: Option<Decimal>,
    /// Flat rate (`Volume`: per hL; `PureAlcohol` without bands: per LPA; `HlPlato`: per hL and °P).
    pub rate: Option<Decimal>,
    /// Bands (`PureAlcohol` ABV bands, `PlatoBand` °P bands); empty = none.
    pub bands: &'a [Band],
    /// Tiers (`TieredVolume`): upper bound in hL and rate per hL.
    pub tiers: &'a [Band],
    /// hL already taxed this year (`TieredVolume`).
    pub ytd_hl: Decimal,
    /// Truncate °P to an integer (`HlPlato`).
    pub plato_trunc: bool,
    /// Relief per LPA (`PureAlcohol`).
    pub discount_per_lpa: Decimal,
    /// Input rounding rule for ABV and °P.
    pub input: InputRounding,
    /// Band boundary type override: `None` = method default.
    pub inclusive: Option<bool>,
}

impl TaxLine<'_> {
    /// Defaults as in the reference: no ytd, no relief, `°P` truncation on, rounding rule undefined.
    #[must_use]
    pub fn new(method: Method, volume_l: Decimal) -> Self {
        Self {
            method,
            volume_l,
            abv_pct: None,
            plato_p: None,
            rate: None,
            bands: &[],
            tiers: &[],
            ytd_hl: Decimal::ZERO,
            plato_trunc: true,
            discount_per_lpa: Decimal::ZERO,
            input: InputRounding::Undefined,
            inclusive: None,
        }
    }
}

/// Python `max(a, b)`: the first argument wins ties (keeps its scale).
fn max_py(a: Decimal, b: Decimal) -> Decimal {
    if b > a { b } else { a }
}

/// Python `min(a, b)`: the first argument wins ties (keeps its scale).
fn min_py(a: Decimal, b: Decimal) -> Decimal {
    if b < a { b } else { a }
}

/// Python's default context keeps 28 significant digits (half even); `rust_decimal` keeps 96 bits.
fn prec28(x: Decimal) -> Decimal {
    let digits = x.mantissa().unsigned_abs().to_string().len();
    if digits <= 28 {
        return x;
    }
    let drop = u32::try_from(digits - 28).unwrap_or(0);
    let scale = x.scale().saturating_sub(drop);
    x.round_dp_with_strategy(scale, RoundingStrategy::MidpointNearestEven)
}

fn mul(a: Decimal, b: Decimal) -> Result<Decimal, Error> {
    let mut r = a
        .checked_mul(b)
        .map(prec28)
        .ok_or(Error::Overflow { what: "amount" })?;
    if r.is_zero() {
        // rust_decimal collapses zero products to scale 0; Python keeps the summed exponent.
        r.rescale((a.scale() + b.scale()).min(28));
    }
    Ok(r)
}

/// Python's sum and difference keep the larger scale; `rust_decimal` drops it for zero operands
/// (T1.12 D4).
fn with_max_scale(mut r: Decimal, a: Decimal, b: Decimal) -> Decimal {
    let want = a.scale().max(b.scale());
    if r.scale() < want {
        r.rescale(want);
    }
    r
}

fn add(a: Decimal, b: Decimal) -> Result<Decimal, Error> {
    let r = a
        .checked_add(b)
        .map(prec28)
        .ok_or(Error::Overflow { what: "amount" })?;
    Ok(with_max_scale(r, a, b))
}

fn sub(a: Decimal, b: Decimal) -> Result<Decimal, Error> {
    let r = a
        .checked_sub(b)
        .map(prec28)
        .ok_or(Error::Overflow { what: "amount" })?;
    Ok(with_max_scale(r, a, b))
}

/// Validates a band or tier table (T1.12 D8): strictly ascending upper bounds, open end only
/// last, no negative rate. An empty table is valid (no band); lookup fails later.
fn check_bands(bands: &[Band]) -> Result<(), Error> {
    let mut prev: Option<Decimal> = None;
    for (i, b) in bands.iter().enumerate() {
        if b.rate.is_sign_negative() && !b.rate.is_zero() {
            return Err(Error::InvalidBands);
        }
        match b.upper {
            None if i + 1 != bands.len() => return Err(Error::InvalidBands),
            None => {}
            Some(u) => {
                if prev.is_some_and(|p| u <= p) {
                    return Err(Error::InvalidBands);
                }
                prev = Some(u);
            }
        }
    }
    Ok(())
}

/// Exact `x / 100` with Python's result scale: the ideal scale of the dividend, or more digits when
/// the quotient needs them (`2000.0 / 100 = 20.0`, `1234.5 / 100 = 12.345`).
fn div100(x: Decimal) -> Result<Decimal, Error> {
    let scale = x.scale();
    let mut m = x.mantissa();
    let mut s = scale + 2;
    if s > 28 {
        return Err(Error::Overflow { what: "amount" });
    }
    while s > scale && m % 10 == 0 {
        m /= 10;
        s -= 1;
    }
    Ok(Decimal::from_i128_with_scale(m, s))
}

/// Python `Decimal.quantize(Decimal(1).scaleb(-dp), rounding=mode)`: exactly `dp` decimal places.
/// Errors when `dp` exceeds [`MAX_DP`] or the result needs more than 28 digits (the reference
/// raises `InvalidOperation`).
fn quantize(x: Decimal, dp: u32, mode: RoundMode) -> Result<Decimal, Error> {
    if dp > MAX_DP {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let mut r = x.round_dp_with_strategy(dp, mode.strategy());
    r.rescale(dp);
    if r.scale() != dp || r.mantissa().unsigned_abs() >= 10_u128.pow(28) {
        return Err(Error::Overflow { what: "amount" });
    }
    Ok(r)
}

/// Input rounding of ABV or °P (data table): `None` rounding leaves the value unchanged.
pub fn round_in(x: Decimal, rounding: Option<(u32, RoundMode)>) -> Result<Decimal, Error> {
    match rounding {
        None => Ok(x),
        Some((dp, mode)) => quantize(x, dp, mode),
    }
}

/// Picks the rate of the first band whose upper bound is not below `value`.
///
/// `inclusive`: a value equal to the upper bound belongs to that band. An open-ended band
/// (`upper = None`) matches everything. No match is an error, never a silent zero.
pub fn band(value: Decimal, bands: &[Band], inclusive: bool) -> Result<Decimal, Error> {
    for b in bands {
        match b.upper {
            None => return Ok(b.rate),
            Some(upper) if (inclusive && value <= upper) || (!inclusive && value < upper) => {
                return Ok(b.rate);
            }
            Some(_) => {}
        }
    }
    Err(Error::invalid("bands", "no band matched"))
}

/// Verification status of a country row (the display text is the client's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowStatus {
    /// No official source and date.
    Unverified,
    /// An official source and its date are both given.
    Verified,
}

/// Verification status of a country row: [`RowStatus::Verified`] only when an official source
/// and its date are both given (non-empty).
#[must_use]
pub fn row_status(official_source: Option<&str>, date: Option<&str>) -> RowStatus {
    let given = |s: Option<&str>| s.is_some_and(|v| !v.is_empty());
    if given(official_source) && given(date) {
        RowStatus::Verified
    } else {
        RowStatus::Unverified
    }
}

/// Money rounding to `dp` places (data table rule; reference default is half up).
pub fn money(x: Decimal, dp: u32, mode: RoundMode) -> Result<Decimal, Error> {
    quantize(x, dp, mode)
}

/// Tax of one line, unrounded, never negative. Units: volume in litres, ABV in % vol, °P, rates
/// in the data table's currency (see [`Method`]).
///
/// Methods that use ABV or °P (`PureAlcohol`, `HlPlato`, `PlatoBand`) fail with
/// [`Error::InvalidInput`] when the rounding rule is [`InputRounding::Undefined`].
pub fn tax_line(p: &TaxLine<'_>) -> Result<Decimal, Error> {
    let uses_input = matches!(
        p.method,
        Method::PureAlcohol | Method::HlPlato | Method::PlatoBand
    );
    let rounding = match p.input {
        InputRounding::Undefined if uses_input => {
            return Err(Error::invalid("input_rounding", "rule is undefined"));
        }
        InputRounding::Places { dp, mode } => Some((dp, mode)),
        _ => None,
    };
    match p.method {
        Method::PureAlcohol if !p.bands.is_empty() => check_bands(p.bands)?,
        Method::PlatoBand => check_bands(p.bands)?,
        Method::TieredVolume => check_bands(p.tiers)?,
        _ => {}
    }
    let hl = div100(p.volume_l)?;
    match p.method {
        Method::Volume => {
            let rate = p
                .rate
                .ok_or(Error::invalid("rate", "required input is missing"))?;
            Ok(max_py(mul(rate, hl)?, Decimal::ZERO))
        }
        Method::PureAlcohol => {
            let abv = p
                .abv_pct
                .ok_or(Error::invalid("abv", "required input is missing"))?;
            let a = round_in(abv, rounding)?;
            let lpa = div100(mul(p.volume_l, a)?)?;
            let rate = if p.bands.is_empty() {
                p.rate
                    .ok_or(Error::invalid("rate", "required input is missing"))?
            } else {
                band(a, p.bands, p.inclusive.unwrap_or(false))?
            };
            let net = max_py(sub(rate, p.discount_per_lpa)?, Decimal::ZERO);
            Ok(max_py(mul(lpa, net)?, Decimal::ZERO))
        }
        Method::HlPlato => {
            let plato = p
                .plato_p
                .ok_or(Error::invalid("plato", "required input is missing"))?;
            let rate = p
                .rate
                .ok_or(Error::invalid("rate", "required input is missing"))?;
            let mut pl = round_in(plato, rounding)?;
            if p.plato_trunc {
                pl = quantize(pl, 0, RoundMode::Floor)?;
            }
            Ok(max_py(mul(mul(rate, hl)?, pl)?, Decimal::ZERO))
        }
        Method::PlatoBand => {
            let plato = p
                .plato_p
                .ok_or(Error::invalid("plato", "required input is missing"))?;
            let pl = round_in(plato, rounding)?;
            let rate = band(pl, p.bands, p.inclusive.unwrap_or(true))?;
            mul(rate, hl)
        }
        Method::TieredVolume => {
            let mut tax = Decimal::ZERO;
            let mut start = p.ytd_hl;
            let mut left = hl;
            for t in p.tiers {
                if left <= Decimal::ZERO {
                    break;
                }
                let room = match t.upper {
                    None => left,
                    Some(upper) => max_py(sub(upper, start)?, Decimal::ZERO),
                };
                let take = min_py(room, left);
                tax = add(tax, mul(take, t.rate)?)?;
                start = add(start, take)?;
                left = sub(left, take)?;
            }
            if left > Decimal::ZERO {
                return Err(Error::invalid("volume", "above the last tier"));
            }
            Ok(tax)
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 3. Spunding
// ---------------------------------------------------------------------------------------------

/// Spunding plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spund {
    /// Residual CO₂, volumes, 2 places.
    pub residual_co2_vol: f64,
    /// CO₂ to be produced, g/L, 2 places.
    pub needed_g_per_l: f64,
    /// Gravity at which to close the tank (SG), 4 places.
    pub closing_sg: f64,
    /// Points before final gravity, 1 place.
    pub points_before: f64,
    /// Pressure setting, psi, 1 place.
    pub setting_psi: f64,
    /// Pressure setting, bar, 2 places.
    pub setting_bar: f64,
}

/// Spunding: closing gravity and valve pressure for `target_co2_vol` volumes CO₂ at `t_c` (°C) with
/// expected final gravity `fg_expected` (SG).
///
/// # Errors
/// [`Error::InvalidInput`] when an input is not finite or `fg_expected <= 0` (T1.12 D2).
pub fn spund(target_co2_vol: f64, t_c: f64, fg_expected: f64) -> Result<Spund, Error> {
    if !(target_co2_vol.is_finite() && t_c.is_finite() && fg_expected.is_finite())
        || fg_expected <= 0.0
    {
        return Err(Error::invalid("input", "not finite or out of range"));
    }
    let need_vol = target_co2_vol - yeast::residual_co2(c_to_f(t_c));
    let need_g = need_vol.max(0.0) * CO2_G_PER_VOL;
    let dre_g_per_l = need_g / BALLING_CO2;
    let fermented_re_p = dre_g_per_l / (10.0 * fg_expected);
    let apparent_drop_p = fermented_re_p / 0.8192;
    let close_p = to_plato(fg_expected) + apparent_drop_p;
    let psi = yeast::co2_psi(target_co2_vol, c_to_f(t_c));
    Ok(Spund {
        residual_co2_vol: round_py(yeast::residual_co2(c_to_f(t_c)), 2),
        needed_g_per_l: round_py(need_g, 2),
        closing_sg: round_py(to_sg(close_p), 4),
        points_before: round_py((to_sg(close_p) - fg_expected) * 1000.0, 1),
        setting_psi: round_py(psi, 1),
        setting_bar: round_py(psi * 0.068_947_6, 2),
    })
}

// ---------------------------------------------------------------------------------------------
// 4. Declaration, taxable volume, spunding warnings
// ---------------------------------------------------------------------------------------------

/// Which nutrition declaration applies (spec §1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDeclaration {
    /// ABV above 1.2 %: nutrition declaration not mandatory, energy may be declared.
    EnergyOptional,
    /// ABV at most 1.2 %: declaration follows the country label template.
    CountryTemplate,
}

impl LabelDeclaration {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnergyOptional => "enerji_istege_bagli",
            Self::CountryTemplate => "ulke_sablonu",
        }
    }
}

/// Declaration rule by ABV (spec §1.3) [to be verified]: above 1.2 % vol energy is optional,
/// otherwise the country template decides. Compared as `Decimal`; 1.2 itself is not above.
#[must_use]
pub fn label_declaration(abv_pct: Decimal) -> LabelDeclaration {
    if abv_pct > Decimal::new(12, 1) {
        LabelDeclaration::EnergyOptional
    } else {
        LabelDeclaration::CountryTemplate
    }
}

/// Moment the tax arises (spec §2.1); chosen by the country data table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaxTrigger {
    /// Packaging (`ambalaj`).
    Packaging,
    /// Release from bond (`cikis`).
    Release,
    /// Sale (`satis`).
    Sale,
}

/// Kind of a volume event (spec §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeEventKind {
    /// Packaging event.
    Packaging,
    /// Release from bond.
    Release,
    /// Sale.
    Sale,
    /// Destroyed volume (`imha`).
    Destroyed,
    /// Returned volume (`iade`).
    Returned,
    /// Laboratory sample (`lab`).
    Lab,
}

impl VolumeEventKind {
    /// Parses the vector name; `None` for an unknown kind.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "ambalaj" => Self::Packaging,
            "cikis" => Self::Release,
            "satis" => Self::Sale,
            "imha" => Self::Destroyed,
            "iade" => Self::Returned,
            "lab" => Self::Lab,
            _ => return None,
        })
    }

    fn is_trigger(self, t: TaxTrigger) -> bool {
        matches!(
            (self, t),
            (Self::Packaging, TaxTrigger::Packaging)
                | (Self::Release, TaxTrigger::Release)
                | (Self::Sale, TaxTrigger::Sale)
        )
    }
}

/// One volume event: ISO date (`YYYY-MM-DD`), kind, litres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeEvent {
    /// ISO date, compared as text.
    pub date: String,
    /// Event kind.
    pub kind: VolumeEventKind,
    /// Litres.
    pub volume_l: Decimal,
}

/// Result of [`taxable_volume`], litres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxableVolume {
    /// Taxable volume after the deductions, never negative.
    pub taxable_l: Decimal,
    /// Sum of events of the trigger kind.
    pub taxed_l: Decimal,
    /// Destroyed volume (own line).
    pub destroyed_l: Decimal,
    /// Returned volume (own line).
    pub returned_l: Decimal,
    /// Laboratory volume (own line).
    pub lab_l: Decimal,
}

/// Inputs of [`taxable_volume`].
#[derive(Debug, Clone, Copy)]
pub struct TaxableVolumeInput<'a> {
    /// Volume events.
    pub events: &'a [VolumeEvent],
    /// Which event kind triggers the tax.
    pub trigger: TaxTrigger,
    /// First day of the period, ISO date.
    pub start: &'a str,
    /// Last day of the period, ISO date.
    pub end: &'a str,
    /// Event kinds subtracted from the taxable volume.
    pub deduct: &'a [VolumeEventKind],
}

/// Taxable volume of a period (spec §2.1): events of the trigger kind within
/// `start ≤ date ≤ end` (ISO text comparison); destroyed, returned and lab volumes are reported
/// on own lines and subtracted only for the kinds in `deduct` (data table rule); other trigger
/// kinds are ignored; the result is clamped at zero.
///
/// # Errors
/// [`Error::InvalidInput`] when `deduct` holds a kind other than destroyed, returned or lab;
/// [`Error::Overflow`] on decimal overflow.
pub fn taxable_volume(p: &TaxableVolumeInput<'_>) -> Result<TaxableVolume, Error> {
    use VolumeEventKind as K;
    let TaxableVolumeInput {
        events,
        trigger,
        start,
        end,
        deduct,
    } = *p;
    if deduct
        .iter()
        .any(|k| !matches!(k, K::Destroyed | K::Returned | K::Lab))
    {
        return Err(Error::invalid("deduct", "unknown deduction kind"));
    }
    let (mut taxed, mut destroyed, mut returned, mut lab) =
        (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
    for e in events {
        if e.date.as_str() < start || e.date.as_str() > end {
            continue;
        }
        if e.kind.is_trigger(trigger) {
            taxed = add(taxed, e.volume_l)?;
        } else {
            match e.kind {
                K::Destroyed => destroyed = add(destroyed, e.volume_l)?,
                K::Returned => returned = add(returned, e.volume_l)?,
                K::Lab => lab = add(lab, e.volume_l)?,
                _ => {}
            }
        }
    }
    let mut net = taxed;
    for (kind, amount) in [
        (K::Destroyed, destroyed),
        (K::Returned, returned),
        (K::Lab, lab),
    ] {
        if deduct.contains(&kind) {
            net = sub(net, amount)?;
        }
    }
    Ok(TaxableVolume {
        taxable_l: max_py(net, Decimal::ZERO),
        taxed_l: taxed,
        destroyed_l: destroyed,
        returned_l: returned,
        lab_l: lab,
    })
}

/// Default FG confidence-interval width above which early closing is flagged: 0.002 SG
/// (200 gravity units) [design, to be verified; the spec gives no number].
pub const SPUND_WIDE_U: i64 = 200;

/// Spunding warning (spec §3.4); codes, not text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpundWarning {
    /// Set pressure exceeds the tank working pressure: blocked.
    TankPressureExceeded,
    /// Wide FG confidence interval: risk of closing 1–2 points early.
    EarlyClosingRisk,
}

impl SpundWarning {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TankPressureExceeded => "tank_basinc_asildi",
            Self::EarlyClosingRisk => "erken_kapama_riski",
        }
    }
}

/// Warnings and blocking flag of [`spund_warnings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpundWarnings {
    /// Warning codes, in this order: tank pressure, early closing.
    pub warnings: Vec<SpundWarning>,
    /// True when the plan is blocked (set pressure above the tank limit).
    pub blocked: bool,
}

/// Inputs of [`spund_warnings`].
#[derive(Debug, Clone, Copy)]
pub struct SpundWarningsInput {
    /// Target carbonation, volumes of CO2.
    pub target_co2_vol: f64,
    /// Temperature, °C.
    pub t_c: f64,
    /// Tank working pressure, psi; `None` for no check.
    pub tank_max_psi: Option<f64>,
    /// FG confidence interval `(low, high)`, SG; `None` for no check.
    pub fg_range: Option<(f64, f64)>,
    /// Width limit in gravity units.
    pub wide_u: i64,
}

/// Spunding warnings (spec §3.4). `tank_max_psi`: tank working pressure (`None`: no check);
/// `fg_range`: FG confidence interval `(low, high)` from 07 §1 (`None`: no check), width
/// compared in gravity units against `wide_u`.
#[must_use]
pub fn spund_warnings(p: &SpundWarningsInput) -> SpundWarnings {
    let SpundWarningsInput {
        target_co2_vol: target_vol,
        t_c,
        tank_max_psi,
        fg_range,
        wide_u,
    } = *p;
    let mut warnings = Vec::new();
    let psi = yeast::co2_psi(target_vol, c_to_f(t_c));
    let blocked = tank_max_psi.is_some_and(|m| psi > m + PSI_TOL);
    if blocked {
        warnings.push(SpundWarning::TankPressureExceeded);
    }
    if fg_range.is_some_and(|(lo, hi)| gravity_units(hi) - gravity_units(lo) > wide_u) {
        warnings.push(SpundWarning::EarlyClosingRisk);
    }
    SpundWarnings { warnings, blocked }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    #[test]
    fn div100_keeps_python_scale() {
        assert_eq!(div100(d("2000.0")).unwrap().to_string(), "20.0");
        assert_eq!(div100(d("2000")).unwrap().to_string(), "20");
        assert_eq!(div100(d("1234.5")).unwrap().to_string(), "12.345");
    }

    #[test]
    fn spund_rejects_non_finite_and_zero_fg() {
        assert_eq!(
            spund(2.5, 12.0, f64::NAN),
            Err(Error::invalid("input", "not finite or out of range"))
        );
        assert_eq!(
            spund(f64::INFINITY, 12.0, 1.01),
            Err(Error::invalid("input", "not finite or out of range"))
        );
        assert_eq!(
            spund(2.5, f64::NAN, 1.01),
            Err(Error::invalid("input", "not finite or out of range"))
        );
        assert_eq!(
            spund(2.5, 12.0, 0.0),
            Err(Error::invalid("input", "not finite or out of range"))
        );
        assert!(spund(2.5, 12.0, 1.01).is_ok());
    }

    #[test]
    fn row_status_needs_source_and_date() {
        assert_eq!(row_status(None, None), RowStatus::Unverified);
        assert_eq!(
            row_status(Some("gov.uk"), Some("2026-10-31")),
            RowStatus::Verified
        );
    }
}
