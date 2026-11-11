//! Section 08: brewery core calculations (`formuller/08-isletme-cekirdek-hesaplar.md`).
//!
//! Units follow the specification of each function; see the section file.
//!
//! Money is `f64` in abstract currency units, as in the Python reference; the report
//! cost (`batch_cost`) is the candidate for a `rust_decimal` port. Intermediate roundings
//! of the reference use [`crate::units::round_py`].

use crate::units::round_py;

/// Specific gravity from °Plato (spec 08 §2.1, same polynomial as 01 §1.2).
#[must_use]
pub fn plato_to_sg(p: f64) -> f64 {
    1.0 + p / (258.6 - (p / 258.6) * 227.1)
}

/// Volume contraction factor, hot (about 100 °C) to 20 °C volume (spec 08 §4.1).
pub const CONTRACTION: f64 = 0.96;

// ------------------------------------------------------------------ 1. Cost

/// One stock lot in acceptance order: name, remaining quantity, unit price.
#[derive(Debug, Clone, PartialEq)]
pub struct Lot {
    pub lot: String,
    /// Remaining quantity (kg or other stock unit).
    pub left: f64,
    /// Unit price (currency per stock unit).
    pub price: f64,
}

/// One consumption row of a FIFO draw.
#[derive(Debug, Clone, PartialEq)]
pub struct FifoRow {
    pub lot: String,
    /// Quantity taken from the lot.
    pub take: f64,
    pub price: f64,
    /// `take · price`.
    pub cost: f64,
}

/// Result of [`fifo_consume`].
#[derive(Debug, Clone, PartialEq)]
pub struct Fifo {
    pub rows: Vec<FifoRow>,
    /// Total cost of the quantity taken from stock.
    pub cost: f64,
    /// Quantity stock could not cover (never negative).
    pub missing: f64,
}

/// FIFO consumption (spec 08 §1.2): draws `qty` from `lots` in order, skipping empty lots.
#[must_use]
pub fn fifo_consume(lots: &[Lot], qty: f64) -> Fifo {
    let mut rows = Vec::new();
    let mut cost = 0.0;
    let mut need = qty;
    for l in lots {
        if need <= 0.0 {
            break;
        }
        let take = l.left.min(need);
        if take > 0.0 {
            rows.push(FifoRow {
                lot: l.lot.clone(),
                take,
                price: l.price,
                cost: take * l.price,
            });
            cost += take * l.price;
            need -= take;
        }
    }
    Fifo {
        rows,
        cost,
        missing: need.max(0.0),
    }
}

/// Lot unit price with shipping share: `(price · qty + share) / qty` (spec 08 §1.1).
#[must_use]
pub fn landed_price(invoice_price: f64, qty: f64, shipping_share: f64) -> f64 {
    (invoice_price * qty + shipping_share) / qty
}

/// Electricity cost: `kWh · price per kWh` (spec 08 §1.3).
#[must_use]
pub fn energy_cost_electric(kwh: f64, price_kwh: f64) -> f64 {
    kwh * price_kwh
}

/// Electric energy for a heat demand: `kJ / η / 3600` in kWh (spec 08 §1.3).
#[must_use]
pub fn kwh_from_heat(kj: f64, eff: f64) -> f64 {
    kj / eff / 3600.0
}

/// Default lower heating value of natural gas, kJ/m³ (spec 08 §1.3, to be verified).
pub const GAS_HU_KJ_M3: f64 = 34_500.0;

/// Gas volume in m³ for a heat demand: `kJ / (η · Hu)` (spec 08 §1.3).
#[must_use]
pub fn gas_m3_from_heat(kj: f64, eff: f64, hu_kj_m3: f64) -> f64 {
    kj / (eff * hu_kj_m3)
}

/// Batch cost breakdown, rounded as the reference (spec 08 §1.4).
#[derive(Debug, Clone, PartialEq)]
pub struct BatchCost {
    /// Ingredients, 2 decimals.
    pub ingredients: f64,
    /// Water, 2 decimals.
    pub water: f64,
    /// Energy, 2 decimals.
    pub energy: f64,
    /// Packaging, 2 decimals.
    pub packaging: f64,
    /// Total, 2 decimals.
    pub total: f64,
    /// Cost per packaged litre, 3 decimals.
    pub per_litre: f64,
}

/// Batch cost (spec 08 §1.3–1.4): water in L at a per-m³ price, packaging units, packaged volume in L.
#[must_use]
pub fn batch_cost(
    ingredient_cost: f64,
    water_l: f64,
    water_price_m3: f64,
    energy_cost: f64,
    pack_units: f64,
    pack_unit_cost: f64,
    packaged_l: f64,
) -> BatchCost {
    let water = water_l / 1000.0 * water_price_m3;
    let pack = pack_units * pack_unit_cost;
    let total = ingredient_cost + water + energy_cost + pack;
    BatchCost {
        ingredients: round_py(ingredient_cost, 2),
        water: round_py(water, 2),
        energy: round_py(energy_cost, 2),
        packaging: round_py(pack, 2),
        total: round_py(total, 2),
        per_litre: round_py(total / packaged_l, 3),
    }
}

// --------------------------------------------------------------- 2. Blending

/// One blend component: volume (L), ABV (%), IBU, SRM, CO₂ (volumes), extract (°P).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendPart {
    pub v: f64,
    pub abv: f64,
    pub ibu: f64,
    pub srm: f64,
    pub co2: f64,
    pub p: f64,
}

/// Blend result; every field rounded to 3 decimals as the reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blend {
    pub v: f64,
    pub abv: f64,
    pub ibu: f64,
    pub srm: f64,
    pub co2: f64,
    pub p: f64,
}

/// Blend of components (spec 08 §2.1): volume-weighted ABV, IBU, SRM, CO₂; mass-weighted °P.
#[must_use]
pub fn blend(parts: &[BlendPart]) -> Blend {
    let v: f64 = parts.iter().map(|p| p.v).sum();
    let wavg = |f: fn(&BlendPart) -> f64| parts.iter().map(|p| p.v * f(p)).sum::<f64>() / v;
    let m: Vec<f64> = parts.iter().map(|p| p.v * plato_to_sg(p.p)).collect();
    let p = m.iter().zip(parts).map(|(mi, q)| mi * q.p).sum::<f64>() / m.iter().sum::<f64>();
    Blend {
        v: round_py(v, 3),
        abv: round_py(wavg(|q| q.abv), 3),
        ibu: round_py(wavg(|q| q.ibu), 3),
        srm: round_py(wavg(|q| q.srm), 3),
        co2: round_py(wavg(|q| q.co2), 3),
        p: round_py(p, 3),
    }
}

/// Volume of the second component for a target value (spec 08 §2.2); `None` unless the
/// target lies strictly between `x1` and `x2`.
#[must_use]
pub fn blend_volume_for_target(v1: f64, x1: f64, x2: f64, xt: f64) -> Option<f64> {
    if !(x1.min(x2) < xt && xt < x1.max(x2)) {
        return None;
    }
    Some(v1 * (x1 - xt) / (xt - x2))
}

// -------------------------------------------------------------- 3. Yeast reuse

/// Cells needed in billions: `pitch rate (M cells/mL/°P) · V (L) · °P` (spec 08 §3.1).
#[must_use]
pub fn cells_needed_billion(pitch_m_ml_p: f64, v_l: f64, p: f64) -> f64 {
    pitch_m_ml_p * v_l * p
}

/// Neubauer count in cells/mL: `counted / squares · 25 · 10⁴ · dilution` (spec 08 §3.2).
#[must_use]
pub fn hemocytometer(total_counted: f64, squares_counted: f64, dilution: f64) -> f64 {
    total_counted / squares_counted * 25.0 * 1e4 * dilution
}

/// Viability from a stain count: `unstained / (unstained + stained)` (spec 08 §3.2).
#[must_use]
pub fn viability_stain(unstained: f64, stained: f64) -> f64 {
    unstained / (unstained + stained)
}

/// Default effective slurry density, cells/mL (spec 08 §3.3, to be verified).
pub const SLURRY_DEFAULT_DENSITY: f64 = 1.0e9;

/// Slurry volume in mL: `N · 10⁹ / (density · viability)` (spec 08 §3.3). A missing or zero
/// measured density falls back to `default_density`.
#[must_use]
pub fn slurry_volume_ml(
    cells_billion: f64,
    density_cells_ml: Option<f64>,
    viability: f64,
    default_density: f64,
) -> f64 {
    let d = match density_cells_ml {
        Some(x) if x != 0.0 => x,
        _ => default_density,
    };
    cells_billion * 1e9 / (d * viability)
}

// ------------------------------------------------- 4. Professional efficiency

/// Extract in the kettle, kg: `V · (0.96 if hot) · SG(°P) · °P / 100` (spec 08 §4.1).
#[must_use]
pub fn extract_kg(v_l: f64, p: f64, hot: bool) -> f64 {
    let v = v_l * if hot { CONTRACTION } else { 1.0 };
    v * plato_to_sg(p) * p / 100.0
}

/// Brewhouse yield (Sudhausausbeute) in %: `100 · extract / malt` (spec 08 §4.2).
#[must_use]
pub fn brewhouse_yield(v_l: f64, p: f64, malt_kg: f64, hot: bool) -> f64 {
    100.0 * extract_kg(v_l, p, hot) / malt_kg
}

/// Laboratory extract as-is, %: `(FGDB − coarse/fine diff) · (1 − moisture/100)` (spec 08 §4.2, §5.1).
#[must_use]
pub fn lab_extract_as_is(fgdb: f64, moisture: f64, fc_diff: f64) -> f64 {
    (fgdb - fc_diff) * (1.0 - moisture / 100.0)
}

/// Maximum first-wort °P at full conversion (spec 08 §4.3, Braukaiser).
#[must_use]
pub fn first_wort_max_p(
    grain_kg: f64,
    extract_as_is_pct: f64,
    moisture_pct: f64,
    mash_water_l: f64,
) -> f64 {
    let e = grain_kg * extract_as_is_pct / 100.0;
    let w = mash_water_l + grain_kg * moisture_pct / 100.0;
    100.0 * e / (e + w)
}

/// Conversion efficiency (ratio): measured / maximum first-wort °P (spec 08 §4.3).
#[must_use]
pub fn conversion_eff(fw_measured_p: f64, fw_max_p: f64) -> f64 {
    fw_measured_p / fw_max_p
}

// ------------------------------------------------------ 5. Malt analysis

/// Malt potential from lab analysis (spec 08 §5.1–5.2); fields rounded as the reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaltAnalysis {
    /// Fine grind, as-is extract %, 2 decimals.
    pub fgai: f64,
    /// Coarse grind, dry basis %, 2 decimals.
    pub cgdb: f64,
    /// Coarse grind, as-is % (the recipe yield), 2 decimals.
    pub cgai_yield: f64,
    /// Potential SG with sucrose 46.21 PPG, 4 decimals.
    pub potential: f64,
    /// Potential SG with the BeerSmith constant 0.046, 4 decimals.
    pub potential_bs: f64,
    /// Points per pound per gallon, 2 decimals.
    pub ppg: f64,
}

/// Malt analysis → extract potential (spec 08 §5.1–5.2).
#[must_use]
pub fn malt_from_analysis(fgdb: f64, moisture: f64, fc_diff: f64) -> MaltAnalysis {
    let fgai = fgdb * (1.0 - moisture / 100.0);
    let cgdb = fgdb - fc_diff;
    let cgai = cgdb * (1.0 - moisture / 100.0);
    let yield_pct = cgai;
    MaltAnalysis {
        fgai: round_py(fgai, 2),
        cgdb: round_py(cgdb, 2),
        cgai_yield: round_py(cgai, 2),
        potential: round_py(1.0 + yield_pct * 0.04621 / 100.0, 4),
        potential_bs: round_py(1.0 + yield_pct * 0.046 / 100.0, 4),
        ppg: round_py(yield_pct * 0.4621, 2),
    }
}

/// Colour from a lab EBC value (spec 08 §5.3); SRM and °L rounded to 2 decimals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabColor {
    pub ebc: f64,
    pub srm: f64,
    pub lovibond: f64,
}

/// EBC → SRM (`EBC / 1.97`) and °L (`(SRM + 0.76) / 1.3546`) (spec 08 §5.3).
#[must_use]
pub fn ebc_from_lab(ebc: f64) -> LabColor {
    LabColor {
        ebc,
        srm: round_py(ebc / 1.97, 2),
        lovibond: round_py((ebc / 1.97 + 0.76) / 1.3546, 2),
    }
}

// ------------------------------------------------------------------ 6. Warning rules

/// Generation limit default for reused yeast (spec §3.4).
pub const YEAST_MAX_GENERATION: u32 = 8;

/// Warning on reusing harvested yeast (spec §3.4); codes, not text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YeastWarning {
    /// Generation above the limit: "new culture recommended".
    GenerationLimit,
    /// More than 14 days since harvest: "viability may be low, count recommended".
    HarvestOver14Days,
    /// Viability below 80 %: "starter or fresh yeast".
    LowViability,
    /// Information: different style or strain history; show the generation chain.
    HistoryInfo,
}

impl YeastWarning {
    /// Stable code used by the parity vectors.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GenerationLimit => "nesil_siniri",
            Self::HarvestOver14Days => "hasat_14_gun",
            Self::LowViability => "canlilik_dusuk",
            Self::HistoryInfo => "bilgi_gecmis",
        }
    }
}

/// Yeast reuse warnings in table order (spec §3.4). `viability` is a fraction (0–1) or
/// `None` when not measured (no viability warning then).
#[must_use]
pub fn yeast_reuse_warnings(
    generation: f64,
    days_since_harvest: f64,
    viability: Option<f64>,
    max_generation: f64,
    history_differs: bool,
) -> Vec<YeastWarning> {
    let mut out = Vec::new();
    if generation > max_generation {
        out.push(YeastWarning::GenerationLimit);
    }
    if days_since_harvest > 14.0 {
        out.push(YeastWarning::HarvestOver14Days);
    }
    if viability.is_some_and(|v| v < 0.80) {
        out.push(YeastWarning::LowViability);
    }
    if history_differs {
        out.push(YeastWarning::HistoryInfo);
    }
    out
}

/// Minimum weighted-average diastatic power of the grain bill, °L (spec §5.4)
/// [to be verified].
pub const DIASTATIC_MIN_L: f64 = 35.0;

/// Result of [`diastatic_check`].
#[derive(Debug, Clone, PartialEq)]
pub struct DiastaticCheck {
    /// Weight-averaged diastatic power, °L.
    pub average: f64,
    /// True when the grain bill may not convert itself (average below the threshold).
    pub low_diastatic_power: bool,
}

/// "Does the grain bill convert itself" check (spec §5.4). `grains` are
/// `(kg, diastatic power °L)` for every grain (0 when a grain has none). An empty or
/// zero-weight bill gives average 0 and no warning.
#[must_use]
pub fn diastatic_check(grains: &[(f64, f64)], threshold: f64) -> DiastaticCheck {
    let w: f64 = grains.iter().map(|g| g.0).sum();
    if w <= 0.0 {
        return DiastaticCheck {
            average: 0.0,
            low_diastatic_power: false,
        };
    }
    let average = grains.iter().map(|g| g.0 * g.1).sum::<f64>() / w;
    DiastaticCheck {
        average,
        low_diastatic_power: average < threshold,
    }
}
