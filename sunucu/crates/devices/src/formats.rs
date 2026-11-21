//! Body formats of the device senders, and the range check.
//!
//! The field lists below come from the public documentation or firmware source of each sender.
//! Nobody has verified them against a real device yet (to be verified, T2.25).
//!
//! - iSpindel "Generic HTTP" (POST, JSON). Source: iSpindel firmware `Ubidots`/`genericHTTP`
//!   code, github.com/universam1/iSpindel (`name`, `ID`, `token`, `angle`, `temperature`,
//!   `temp_units`, `battery`, `gravity`, `interval`, `RSSI`). `gravity` is degrees Plato in the
//!   default firmware; a custom formula may give SG.
//! - RAPT Pill webhook (POST, JSON). Source: `KegLand` RAPT portal documentation for
//!   "Webhooks" (`device_id`, `device_name`, `temperature` in Celsius, `gravity` in points such as
//!   `1050.5`, `battery` in percent, `rssi`, `created_date`).
//! - Brewfather "Custom Stream" (POST, JSON). Source: docs.brewfather.app, page "Custom Stream":
//!   `name`, `temp`, `aux_temp`, `ext_temp`, `temp_unit` (`C`, `F`, `K`), `gravity`,
//!   `gravity_unit` (`G` or `P`), `pressure`, `pressure_unit` (`PSI`, `BAR`, `KPA`), `ph`,
//!   `bpm`, `comment`, `beer`, `battery` (volt). `TiltBridge` and Pitch can send this format.
//!
//! Devices send no usable clock, so the server takes the receive time. Values outside the ranges
//! of [`check_ranges`] reject the whole body: a sensor that reports a nonsense value is not
//! trusted for the other values either.

use serde_json::{Map, Value};

use crate::Reading;

/// A body format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// iSpindel Generic HTTP.
    Ispindel,
    /// RAPT Pill webhook.
    Rapt,
    /// Brewfather Custom Stream.
    Brewfather,
}

impl Format {
    /// The format named in the address (`/v1/ingest/{key}/{format}`).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ispindel" => Some(Self::Ispindel),
            "rapt" => Some(Self::Rapt),
            "brewfather" => Some(Self::Brewfather),
            _ => None,
        }
    }

    /// The format a device kind sends by default. Tilt bridges and other senders use the
    /// Brewfather Custom Stream format.
    #[must_use]
    pub fn for_kind(kind: &str) -> Self {
        match kind {
            "ispindel" => Self::Ispindel,
            "rapt" => Self::Rapt,
            _ => Self::Brewfather,
        }
    }
}

/// How a gravity number is read when the format does not say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GravityUnit {
    /// Specific gravity (`1.050`).
    Sg,
    /// Degrees Plato (`12.4`).
    Plato,
}

impl GravityUnit {
    /// The unit in `devices.calibration` (`gravity_unit`: `sg` or `plato`); `None` means auto.
    #[must_use]
    pub fn from_calibration(calibration: Option<&Value>) -> Option<Self> {
        match calibration?.get("gravity_unit")?.as_str()? {
            "sg" => Some(Self::Sg),
            "plato" => Some(Self::Plato),
            _ => None,
        }
    }
}

/// One out-of-range or invalid value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Field name in the body.
    pub field: &'static str,
    /// `out_of_range` or `invalid_value`.
    pub code: &'static str,
}

/// Why a body is not accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Not JSON, not an object, no values, or a field of the wrong type.
    Malformed(String),
    /// A value is outside the allowed range.
    Rejected(Vec<Issue>),
}

fn malformed(text: impl Into<String>) -> ParseError {
    ParseError::Malformed(text.into())
}

/// Allowed range of a measurement kind (inclusive), in SI units.
#[must_use]
pub fn range(kind: &str) -> Option<(f64, f64)> {
    match kind {
        "sg" => Some((0.980, 1.200)),
        "temp_c" => Some((-10.0, 100.0)),
        "ph" => Some((0.0, 14.0)),
        "pressure_bar" => Some((0.0, 10.0)),
        "battery_v" => Some((0.0, 6.0)),
        "signal_dbm" => Some((-130.0, 0.0)),
        _ => None,
    }
}

/// Plato to specific gravity (standard polynomial-free formula, ASBC).
#[must_use]
pub fn plato_to_sg(plato: f64) -> f64 {
    1.0 + plato / (258.6 - (plato / 258.2) * 227.1)
}

fn field(
    obj: &Map<String, Value>,
    names: &[&'static str],
) -> Result<Option<(&'static str, f64)>, ParseError> {
    for name in names {
        match obj.get(*name) {
            None | Some(Value::Null) => {}
            Some(Value::Number(n)) => {
                let v = n
                    .as_f64()
                    .ok_or_else(|| malformed(format!("{name} is not a number")))?;
                return Ok(Some((name, v)));
            }
            Some(_) => return Err(malformed(format!("{name} is not a number"))),
        }
    }
    Ok(None)
}

fn text<'a>(obj: &'a Map<String, Value>, name: &str) -> Result<Option<&'a str>, ParseError> {
    match obj.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.as_str())),
        Some(_) => Err(malformed(format!("{name} is not text"))),
    }
}

fn to_celsius(value: f64, unit: Option<&str>) -> Result<f64, ParseError> {
    match unit.map(str::to_ascii_uppercase).as_deref() {
        None | Some("C") => Ok(value),
        Some("F") => Ok((value - 32.0) * 5.0 / 9.0),
        Some("K") => Ok(value - 273.15),
        Some(other) => Err(malformed(format!("unknown temperature unit {other}"))),
    }
}

/// Gravity from a bare number. A known unit wins. Without one, a value of 1.2 or more is Plato
/// (SG never gets there) and a smaller value is SG.
fn gravity_sg(value: f64, unit: Option<GravityUnit>) -> f64 {
    match unit {
        Some(GravityUnit::Plato) => plato_to_sg(value),
        None if value >= 1.2 => plato_to_sg(value),
        Some(GravityUnit::Sg) | None => value,
    }
}

type Collected = Vec<(&'static str, &'static str, f64)>; // (field, kind, value)

fn ispindel(obj: &Map<String, Value>, unit: Option<GravityUnit>) -> Result<Collected, ParseError> {
    let mut out = Collected::new();
    if let Some((_, t)) = field(obj, &["temperature"])? {
        out.push((
            "temperature",
            "temp_c",
            to_celsius(t, text(obj, "temp_units")?)?,
        ));
    }
    if let Some((_, g)) = field(obj, &["gravity"])? {
        out.push(("gravity", "sg", gravity_sg(g, unit)));
    }
    if let Some((_, b)) = field(obj, &["battery"])? {
        out.push(("battery", "battery_v", b));
    }
    if let Some((name, r)) = field(obj, &["RSSI", "rssi"])? {
        out.push((name, "signal_dbm", r));
    }
    Ok(out)
}

fn rapt(obj: &Map<String, Value>, unit: Option<GravityUnit>) -> Result<Collected, ParseError> {
    let mut out = Collected::new();
    if let Some((_, t)) = field(obj, &["temperature"])? {
        out.push(("temperature", "temp_c", t));
    }
    if let Some((_, g)) = field(obj, &["gravity"])? {
        // Points (`1050.5`) when 500 or more; otherwise like any bare gravity number.
        let sg = if g >= 500.0 {
            g / 1000.0
        } else {
            gravity_sg(g, unit)
        };
        out.push(("gravity", "sg", sg));
    }
    // `battery` is a percentage in RAPT; `measurements` has no kind for it yet (TODO).
    if let Some((_, r)) = field(obj, &["rssi", "RSSI"])? {
        out.push(("rssi", "signal_dbm", r));
    }
    Ok(out)
}

fn brewfather(obj: &Map<String, Value>) -> Result<Collected, ParseError> {
    let mut out = Collected::new();
    if let Some((_, t)) = field(obj, &["temp"])? {
        out.push(("temp", "temp_c", to_celsius(t, text(obj, "temp_unit")?)?));
    }
    if let Some((_, g)) = field(obj, &["gravity"])? {
        let sg = match text(obj, "gravity_unit")?
            .map(str::to_ascii_uppercase)
            .as_deref()
        {
            None | Some("G") => g,
            Some("P") => plato_to_sg(g),
            Some(other) => return Err(malformed(format!("unknown gravity unit {other}"))),
        };
        out.push(("gravity", "sg", sg));
    }
    if let Some((_, p)) = field(obj, &["pressure"])? {
        let bar = match text(obj, "pressure_unit")?
            .map(str::to_ascii_uppercase)
            .as_deref()
        {
            None | Some("PSI") => p * 0.068_947_6,
            Some("BAR") => p,
            Some("KPA") => p / 100.0,
            Some(other) => return Err(malformed(format!("unknown pressure unit {other}"))),
        };
        out.push(("pressure", "pressure_bar", bar));
    }
    if let Some((_, v)) = field(obj, &["ph"])? {
        out.push(("ph", "ph", v));
    }
    if let Some((_, b)) = field(obj, &["battery"])? {
        out.push(("battery", "battery_v", b));
    }
    // `aux_temp`, `ext_temp` and `bpm` have no measurement kind yet (TODO).
    Ok(out)
}

/// Checks every value against [`range`].
///
/// # Errors
/// [`ParseError::Rejected`] with one issue for each value out of range.
pub fn check_ranges(values: &Collected) -> Result<Vec<Reading>, ParseError> {
    let mut issues = Vec::new();
    let mut readings = Vec::with_capacity(values.len());
    for (field, kind, value) in values {
        let ok = range(kind).is_some_and(|(lo, hi)| value.is_finite() && (lo..=hi).contains(value));
        if ok {
            readings.push(Reading {
                kind: (*kind).to_owned(),
                value: *value,
            });
        } else {
            issues.push(Issue {
                field,
                code: "out_of_range",
            });
        }
    }
    if issues.is_empty() {
        Ok(readings)
    } else {
        Err(ParseError::Rejected(issues))
    }
}

/// Parses a body in `format` into checked readings.
///
/// # Errors
/// [`ParseError::Malformed`] or [`ParseError::Rejected`].
pub fn parse(
    format: Format,
    body: &[u8],
    unit: Option<GravityUnit>,
) -> Result<Vec<Reading>, ParseError> {
    let value: Value = serde_json::from_slice(body).map_err(|_| malformed("body is not JSON"))?;
    let Value::Object(obj) = value else {
        return Err(malformed("body is not a JSON object"));
    };
    let collected = match format {
        Format::Ispindel => ispindel(&obj, unit)?,
        Format::Rapt => rapt(&obj, unit)?,
        Format::Brewfather => brewfather(&obj)?,
    };
    if collected.is_empty() {
        return Err(malformed("body has no known values"));
    }
    check_ranges(&collected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(r: &[Reading]) -> Vec<(&str, f64)> {
        r.iter().map(|x| (x.kind.as_str(), x.value)).collect()
    }

    #[test]
    fn ispindel_plato_and_units() {
        let body = br#"{"name":"iSpindel000","ID":1,"token":"x","angle":40.1,"temperature":68.0,
            "temp_units":"F","battery":4.05,"gravity":12.5,"interval":900,"RSSI":-70}"#;
        let r = parse(Format::Ispindel, body, None).unwrap();
        let k = kinds(&r);
        assert_eq!(k[0].0, "temp_c");
        assert!((k[0].1 - 20.0).abs() < 1e-9);
        assert!((k[1].1 - 1.0509).abs() < 1e-3, "{}", k[1].1);
        assert_eq!(k[2], ("battery_v", 4.05));
        assert_eq!(k[3], ("signal_dbm", -70.0));
    }

    #[test]
    fn ispindel_sg_by_calibration_and_auto() {
        let body = br#"{"temperature":20.0,"gravity":1.012}"#;
        let r = parse(Format::Ispindel, body, None).unwrap();
        assert!((r[1].value - 1.012).abs() < 1e-12);
        let r = parse(Format::Ispindel, body, Some(GravityUnit::Sg)).unwrap();
        assert!((r[1].value - 1.012).abs() < 1e-12);
        let dry = br#"{"gravity":2.5}"#;
        let r = parse(Format::Ispindel, dry, None).unwrap();
        assert!((r[0].value - 1.0098).abs() < 1e-3);
    }

    #[test]
    fn rapt_points() {
        let body =
            br#"{"device_id":"a","temperature":19.5,"gravity":1048.5,"battery":88,"rssi":-60}"#;
        let r = parse(Format::Rapt, body, None).unwrap();
        assert!((r[1].value - 1.0485).abs() < 1e-9);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn brewfather_units() {
        let body = br#"{"name":"x","temp":68,"temp_unit":"F","gravity":1.05,"gravity_unit":"G",
            "pressure":14.5,"pressure_unit":"PSI","ph":4.4,"battery":4.1}"#;
        let r = parse(Format::Brewfather, body, None).unwrap();
        assert_eq!(r.len(), 5);
        assert!((r[2].value - 1.0).abs() < 0.01);
        let p = parse(
            Format::Brewfather,
            br#"{"gravity":12,"gravity_unit":"P"}"#,
            None,
        )
        .unwrap();
        assert!((p[0].value - 1.048).abs() < 1e-3);
    }

    #[test]
    fn out_of_range_is_rejected() {
        let r = parse(
            Format::Ispindel,
            br#"{"temperature":20.0,"gravity":0.5}"#,
            None,
        );
        assert_eq!(
            r,
            Err(ParseError::Rejected(vec![Issue {
                field: "gravity",
                code: "out_of_range"
            }]))
        );
        assert!(matches!(
            parse(Format::Brewfather, br#"{"ph":15}"#, None),
            Err(ParseError::Rejected(_))
        ));
        assert!(matches!(
            parse(Format::Brewfather, br#"{"temp":250}"#, None),
            Err(ParseError::Rejected(_))
        ));
        assert!(matches!(
            parse(Format::Rapt, br#"{"rssi":10}"#, None),
            Err(ParseError::Rejected(_))
        ));
    }

    #[test]
    fn malformed_bodies() {
        for body in [&b"nope"[..], b"[1]", b"{}", br#"{"temperature":"hot"}"#] {
            assert!(matches!(
                parse(Format::Ispindel, body, None),
                Err(ParseError::Malformed(_))
            ));
        }
        assert!(matches!(
            parse(Format::Brewfather, br#"{"temp":1,"temp_unit":"X"}"#, None),
            Err(ParseError::Malformed(_))
        ));
    }
}
