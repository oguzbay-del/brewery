//! Field tables of the settings resources: what a field is, how it is checked and how the
//! `OpenAPI` document describes it. One table row drives validation, the schema and the example.

use serde_json::{Map, Value, json};

use crate::{authz::Permission, error::FieldIssue};

/// Type of a field.
#[derive(Debug, Clone, Copy)]
pub enum Ty {
    Text,
    Num,
    Bool,
    Uuid,
    Enum(&'static [&'static str]),
}

/// Example value for the `OpenAPI` document.
#[derive(Debug, Clone, Copy)]
pub enum Ex {
    S(&'static str),
    N(f64),
    B(bool),
}

impl Ex {
    pub fn value(self) -> Value {
        match self {
            Self::S(s) => json!(s),
            Self::N(n) => json!(n),
            Self::B(b) => json!(b),
        }
    }
}

/// One field of a resource.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub name: &'static str,
    pub ty: Ty,
    /// Needed when the record is created.
    pub required: bool,
    /// `null` is accepted (the column can be empty).
    pub nullable: bool,
    pub min: Option<f64>,
    pub min_exclusive: bool,
    pub max: Option<f64>,
    pub min_len: usize,
    pub max_len: usize,
    pub ex: Ex,
}

pub const fn text(name: &'static str, ex: &'static str) -> Field {
    field(name, Ty::Text, Ex::S(ex))
}
pub const fn num(name: &'static str, ex: f64) -> Field {
    field(name, Ty::Num, Ex::N(ex))
}
pub const fn flag(name: &'static str, ex: bool) -> Field {
    field(name, Ty::Bool, Ex::B(ex))
}
pub const fn id(name: &'static str, ex: &'static str) -> Field {
    field(name, Ty::Uuid, Ex::S(ex))
}
pub const fn choice(name: &'static str, values: &'static [&'static str]) -> Field {
    field(name, Ty::Enum(values), Ex::S(values[0]))
}

const fn field(name: &'static str, ty: Ty, ex: Ex) -> Field {
    Field {
        name,
        ty,
        required: false,
        nullable: false,
        min: None,
        min_exclusive: false,
        max: None,
        min_len: 0,
        max_len: 0,
        ex,
    }
}

impl Field {
    pub const fn req(mut self) -> Self {
        self.required = true;
        self
    }
    pub const fn null(mut self) -> Self {
        self.nullable = true;
        self
    }
    /// Greater than `v`.
    pub const fn gt(mut self, v: f64) -> Self {
        self.min = Some(v);
        self.min_exclusive = true;
        self
    }
    /// Greater than or equal to `v`.
    pub const fn ge(mut self, v: f64) -> Self {
        self.min = Some(v);
        self
    }
    pub const fn le(mut self, v: f64) -> Self {
        self.max = Some(v);
        self
    }
    /// Text length in characters.
    pub const fn len(mut self, min: usize, max: usize) -> Self {
        self.min_len = min;
        self.max_len = max;
        self
    }

    /// Checks one value; `path` is the JSON Pointer of the field.
    pub fn check(&self, path: &str, v: &Value) -> Result<(), FieldIssue> {
        let issue = |code: &str| FieldIssue::new(path, code);
        if v.is_null() {
            return if self.nullable {
                Ok(())
            } else {
                Err(issue("null_not_allowed"))
            };
        }
        match self.ty {
            Ty::Bool if !v.is_boolean() => Err(issue("invalid_type")),
            Ty::Num => {
                let n = v.as_f64().ok_or_else(|| issue("invalid_type"))?;
                let low = self
                    .min
                    .is_some_and(|m| if self.min_exclusive { n <= m } else { n < m });
                if low || self.max.is_some_and(|m| n > m) {
                    return Err(issue("out_of_range")
                        .with_params(json!({"min": self.min, "max": self.max})));
                }
                Ok(())
            }
            Ty::Text => {
                let s = v.as_str().ok_or_else(|| issue("invalid_type"))?;
                let n = s.chars().count();
                if n < self.min_len {
                    return Err(issue("too_short").with_params(json!({"min": self.min_len})));
                }
                if self.max_len > 0 && n > self.max_len {
                    return Err(issue("too_long").with_params(json!({"max": self.max_len})));
                }
                Ok(())
            }
            Ty::Uuid => match v.as_str().map(uuid::Uuid::parse_str) {
                Some(Ok(_)) => Ok(()),
                _ => Err(issue("invalid_uuid")),
            },
            Ty::Enum(allowed) => match v.as_str() {
                Some(s) if allowed.contains(&s) => Ok(()),
                Some(_) => Err(issue("invalid_value").with_params(json!({"allowed": allowed}))),
                None => Err(issue("invalid_type")),
            },
            Ty::Bool => Ok(()),
        }
    }

    /// `OpenAPI` schema of the field.
    pub fn schema(&self) -> Value {
        let mut s = match self.ty {
            Ty::Text => json!({"type": "string"}),
            Ty::Num => json!({"type": "number", "format": "double"}),
            Ty::Bool => json!({"type": "boolean"}),
            Ty::Uuid => json!({"type": "string", "format": "uuid"}),
            Ty::Enum(values) => json!({"type": "string", "enum": values}),
        };
        let Some(o) = s.as_object_mut() else {
            return s;
        };
        if self.nullable {
            // OpenAPI 3.1: a type list that includes `null`.
            let t = o["type"].clone();
            o.insert("type".to_owned(), json!([t, "null"]));
            if let Some(Value::Array(values)) = o.get_mut("enum") {
                values.push(Value::Null);
            }
        }
        if let Some(m) = self.min {
            o.insert(
                if self.min_exclusive {
                    "exclusiveMinimum"
                } else {
                    "minimum"
                }
                .to_owned(),
                json!(m),
            );
        }
        if let Some(m) = self.max {
            o.insert("maximum".to_owned(), json!(m));
        }
        if self.max_len > 0 {
            o.insert("maxLength".to_owned(), json!(self.max_len));
        }
        if self.min_len > 0 {
            o.insert("minLength".to_owned(), json!(self.min_len));
        }
        s
    }
}

/// Checks the keys and values of `body` against `fields`. `creating` demands the required ones.
/// `prefix` is the JSON Pointer of the object (`""` or `/steps/0`).
pub fn check_object(
    fields: &[Field],
    body: &Map<String, Value>,
    prefix: &str,
    creating: bool,
    skip: &[&str],
    issues: &mut Vec<FieldIssue>,
) {
    for (key, value) in body {
        if skip.contains(&key.as_str()) {
            continue;
        }
        match fields.iter().find(|f| f.name == key) {
            None => issues.push(FieldIssue::new(format!("{prefix}/{key}"), "unknown_field")),
            Some(f) => {
                if let Err(e) = f.check(&format!("{prefix}/{key}"), value) {
                    issues.push(e);
                }
            }
        }
    }
    if creating {
        for f in fields.iter().filter(|f| f.required) {
            if !body.contains_key(f.name) {
                issues.push(FieldIssue::new(format!("{prefix}/{}", f.name), "required"));
            }
        }
    }
}

/// Equality of JSON values where `1` and `1.0` are the same number.
pub fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| json_eq(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| json_eq(v, w)))
        }
        _ => a == b,
    }
}

// --- resources -------------------------------------------------------------------------------

/// Steps of a profile (mash and fermentation): replaced as a whole.
#[derive(Debug)]
pub struct Steps {
    pub table: &'static str,
    pub schema: &'static str,
    pub fields: &'static [Field],
}

/// A settings resource backed by one table.
#[derive(Debug)]
pub struct Resource {
    /// Operation id prefix (`<name>_list`).
    pub name: &'static str,
    /// Path segment (`mash-profiles`).
    pub path: &'static str,
    pub table: &'static str,
    /// Schema name in the `OpenAPI` document.
    pub schema: &'static str,
    pub fields: &'static [Field],
    pub steps: Option<Steps>,
    /// Fields the list can sort by (the first is the default, ascending).
    pub sort: &'static [&'static str],
    /// Only one record can be the default (`is_default`).
    pub has_default: bool,
    /// Field that has a unique index (conflict answers name it).
    pub unique: Option<&'static str>,
    /// What the resource is, for the document.
    pub about: &'static str,
}

pub const READ_PERMISSIONS: [Permission; 2] =
    [Permission::RecipeView, Permission::SettingsEquipment];
pub const WRITE_PERMISSION: Permission = Permission::SettingsEquipment;

const EQUIPMENT: &[Field] = &[
    text("name", "Brew house").req().len(1, 200),
    flag("is_default", true),
    num("batch_volume_l", 20.0).req().gt(0.0),
    num("boil_volume_l", 26.0).null().gt(0.0),
    num("boil_time_min", 60.0).ge(0.0),
    num("boil_off_l_per_h", 3.0).ge(0.0),
    num("trub_loss_l", 1.0).ge(0.0),
    num("pre_trub_loss_l", 0.0).ge(0.0),
    num("fermenter_loss_l", 0.5).ge(0.0),
    num("top_up_l", 0.0).ge(0.0),
    num("cooling_shrinkage_pct", 4.0).ge(0.0).le(100.0),
    num("mash_tun_volume_l", 30.0).null().gt(0.0),
    num("mash_tun_mass_kg", 0.0).ge(0.0),
    num("mash_tun_specific_heat", 0.12).ge(0.0),
    num("mash_dead_space_l", 0.0).ge(0.0),
    num("grain_absorption_l_per_kg", 1.04).ge(0.0),
    num("efficiency_pct", 72.0).req().ge(0.0).le(100.0),
    num("conversion_efficiency_pct", 90.0).null().gt(0.0).le(100.0),
    num("lauter_efficiency_pct", 80.0).null().gt(0.0).le(100.0),
    num("altitude_m", 0.0),
    num("hop_utilization_pct", 100.0).ge(0.0),
    flag("whirlpool_carry", false),
    num("whirlpool_time_min", 0.0).ge(0.0),
    text("notes", "Electric kettle, 30 L").null(),
];

const MASH: &[Field] = &[
    text("name", "Single infusion").req().len(1, 200),
    flag("biab", false),
    num("grain_temp_c", 20.0).null(),
    num("sparge_temp_c", 76.0).null(),
    num("target_ph", 5.4).null().ge(0.0).le(14.0),
    text("notes", "Mash for pale ales").null(),
];
const MASH_STEPS: &[Field] = &[
    text("name", "Saccharification").len(0, 200),
    choice("kind", &["infusion", "decoction", "temperature"]).req(),
    num("temp_c", 66.0).req(),
    num("time_min", 60.0).req().ge(0.0),
    num("ramp_min", 0.0).ge(0.0),
    num("infusion_l", 15.0).null().ge(0.0),
    num("infusion_temp_c", 72.0).null(),
];

const FERMENTATION: &[Field] = &[
    text("name", "Ale, 18 C").req().len(1, 200),
    text("notes", "Primary then diacetyl rest").null(),
];
const FERMENTATION_STEPS: &[Field] = &[
    text("name", "Primary").len(0, 200),
    num("temp_c", 18.0).req(),
    num("duration_min", 10080.0).req().ge(0.0),
    num("ramp_min", 0.0).ge(0.0),
];

const CARBONATION: &[Field] = &[
    text("name", "Bottle conditioned ale").req().len(1, 200),
    choice("method", &["forced", "priming", "natural", "spunding"]).req(),
    num("target_co2_vols", 2.4).req().ge(0.0),
    num("temp_c", 20.0).null(),
    choice(
        "priming_agent",
        &["corn_sugar", "table_sugar", "dme", "honey"],
    )
    .null(),
    text("notes", "Priming with corn sugar").null(),
];

const WATER: &[Field] = &[
    text("name", "Tap water").req().len(1, 200),
    choice("kind", &["source", "target"]).req(),
    num("calcium_mg_l", 50.0).ge(0.0),
    num("magnesium_mg_l", 10.0).ge(0.0),
    num("sodium_mg_l", 15.0).ge(0.0),
    num("chloride_mg_l", 40.0).ge(0.0),
    num("sulfate_mg_l", 60.0).ge(0.0),
    num("bicarbonate_mg_l", 120.0).ge(0.0),
    num("alkalinity_mg_l", 100.0).null().ge(0.0),
    num("ph", 7.4).null().ge(0.0).le(14.0),
    id("source_document_id", "0192f1c07a3b7000800000000000000f").null(),
    text("notes", "Lab report of March").null(),
];

const VESSEL: &[Field] = &[
    text("code", "FV-1").req().len(1, 50),
    text("name", "Conical fermenter").len(0, 200),
    choice(
        "kind",
        &[
            "fermenter",
            "brite",
            "kettle",
            "mash_tun",
            "hlt",
            "yeast_brink",
            "keg_store",
            "other",
        ],
    )
    .req(),
    num("capacity_l", 30.0).req().gt(0.0),
    num("max_pressure_bar", 1.0).null().gt(0.0),
    choice(
        "cleaning_status",
        &["dirty", "cleaning", "clean", "sanitized"],
    ),
    text("notes", "Stainless steel").null(),
];

/// The resources, in the order of the tags and paths.
pub static RESOURCES: [Resource; 6] = [
    Resource {
        name: "equipment",
        path: "equipment",
        table: "equipment",
        schema: "Equipment",
        fields: EQUIPMENT,
        steps: None,
        sort: &["name", "updated_at"],
        has_default: true,
        unique: None,
        about: "Equipment profile: volumes, losses, efficiency, altitude and hop utilization",
    },
    Resource {
        name: "mash_profiles",
        path: "mash-profiles",
        table: "mash_profiles",
        schema: "MashProfile",
        fields: MASH,
        steps: Some(Steps {
            table: "mash_profile_steps",
            schema: "MashStep",
            fields: MASH_STEPS,
        }),
        sort: &["name", "updated_at"],
        has_default: false,
        unique: None,
        about: "Mash profile with its steps",
    },
    Resource {
        name: "fermentation_profiles",
        path: "fermentation-profiles",
        table: "fermentation_profiles",
        schema: "FermentationProfile",
        fields: FERMENTATION,
        steps: Some(Steps {
            table: "fermentation_profile_steps",
            schema: "FermentationStep",
            fields: FERMENTATION_STEPS,
        }),
        sort: &["name", "updated_at"],
        has_default: false,
        unique: None,
        about: "Fermentation profile with its temperature steps",
    },
    Resource {
        name: "carbonation_profiles",
        path: "carbonation-profiles",
        table: "carbonation_profiles",
        schema: "CarbonationProfile",
        fields: CARBONATION,
        steps: None,
        sort: &["name", "updated_at"],
        has_default: false,
        unique: None,
        about: "Carbonation profile: method and target CO2 volumes",
    },
    Resource {
        name: "water_profiles",
        path: "water-profiles",
        table: "water_profiles",
        schema: "WaterProfile",
        fields: WATER,
        steps: None,
        sort: &["name", "updated_at"],
        has_default: false,
        unique: None,
        about: "Source or target water profile; `source_document_id` links the lab water analysis report (a scanned document)",
    },
    Resource {
        name: "vessels",
        path: "vessels",
        table: "vessels",
        schema: "Vessel",
        fields: VESSEL,
        steps: None,
        sort: &["code", "updated_at"],
        has_default: false,
        unique: Some("code"),
        about: "Vessel (fermenter, kettle, keg store, ...) as Home mode uses it",
    },
];

// --- workspace and user settings ---------------------------------------------------------------

/// Keys of the workspace settings and of the user settings. The calculation option names and
/// types follow `RecipeOptions` of the engine contract, so T2.21 can pass them on unchanged.
pub const SETTING_KEYS: &[Field] = &[
    choice("unit_system", &["metric", "us", "imperial"]).null(),
    choice("temperature_unit", &["celsius", "fahrenheit"]).null(),
    choice("gravity_unit", &["sg", "plato", "brix"]).null(),
    choice("color_unit", &["srm", "ebc"]).null(),
    choice("pressure_unit", &["bar", "psi", "kpa"]).null(),
    choice("ibu_equation", &["tinseth", "rager", "garetz"]).null(),
    num("steep_efficiency_pct", 15.0).null().ge(0.0).le(100.0),
    flag("mash_fg_adjust", true).null(),
    num("mash_fg_adj_temp_c", 67.5).null(),
    num("mash_fg_adj_slope_pct_per_c", -2.25).null(),
    num("hop_first_wort_adj_pct", 10.0).null(),
    flag("hop_type_adj", true).null(),
    num("hop_leaf_adj_pct", 0.0).null(),
    num("hop_plug_adj_pct", 2.0).null(),
    num("hop_pellet_adj_pct", 10.0).null(),
    num("hop_mash_adj_pct", -80.0).null(),
];
