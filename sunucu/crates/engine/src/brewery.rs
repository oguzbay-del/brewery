//! Section 08: brewery core calculations (`formuller/08-isletme-cekirdek-hesaplar.md`).
//!
//! Units follow the specification of each function; see the section file.
//!
//! Stock quantities and money are [`Decimal`] (T1.12 C2, C6): quantities are never rounded;
//! every money line is rounded to 2 decimals half up and totals are sums of rounded lines.
//! Physical quantities (L, kg, kWh, °P) stay `f64`. Results are unrounded (T1.12 C9) except
//! the money rules above.

use rust_decimal::{Decimal, RoundingStrategy};

use crate::error::{Error, Result, ensure_finite, finite_result};
use crate::units::py_sum;
use crate::units::to_sg;

/// Money line: 2 decimals, half up (spec 08 §1, T1.12 C6).
fn money(x: Decimal) -> Decimal {
    x.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Volume contraction factor, hot (about 100 °C) to 20 °C volume (spec 08 §4.1).
pub const CONTRACTION: f64 = 0.96;

// ------------------------------------------------------------------ 1. Cost

/// One stock lot in acceptance order: name, remaining quantity, unit price.
#[derive(Debug, Clone, PartialEq)]
pub struct Lot {
    pub lot: String,
    /// Remaining quantity (kg or other stock unit).
    pub left: Decimal,
    /// Unit price (currency per stock unit).
    pub price: Decimal,
}

/// One consumption row of a FIFO draw.
#[derive(Debug, Clone, PartialEq)]
pub struct FifoRow {
    pub lot: String,
    /// Quantity taken from the lot.
    pub take: Decimal,
    pub price: Decimal,
    /// `take · price` rounded to 2 decimals half up.
    pub cost: Decimal,
}

/// Result of [`fifo_consume`].
#[derive(Debug, Clone, PartialEq)]
pub struct Fifo {
    pub rows: Vec<FifoRow>,
    /// Total cost: sum of the rounded row costs.
    pub cost: Decimal,
    /// Quantity stock could not cover (never negative).
    pub missing: Decimal,
}

/// FIFO consumption (spec 08 §1.2): draws `qty` from `lots` in order, skipping empty lots.
/// Quantities are exact; each row cost is rounded half up to 2 decimals.
#[must_use]
pub fn fifo_consume(lots: &[Lot], qty: Decimal) -> Fifo {
    let mut rows = Vec::new();
    let mut cost = Decimal::ZERO;
    let mut need = qty;
    for l in lots {
        if need <= Decimal::ZERO {
            break;
        }
        let take = l.left.min(need);
        if take > Decimal::ZERO {
            let line = money(take * l.price);
            rows.push(FifoRow {
                lot: l.lot.clone(),
                take,
                price: l.price,
                cost: line,
            });
            cost += line;
            need -= take;
        }
    }
    Fifo {
        rows,
        cost,
        missing: need.max(Decimal::ZERO),
    }
}

/// Lot unit price with shipping share: `(price · qty + share) / qty` (spec 08 §1.1), not
/// not rounded. Errors when `qty ≤ 0` (the reference raises) or the amount overflows.
pub fn landed_price(
    invoice_price: Decimal,
    qty: Decimal,
    shipping_share: Decimal,
) -> Result<Decimal> {
    if qty <= Decimal::ZERO {
        return Err(Error::invalid("qty", "must be > 0"));
    }
    (invoice_price * qty + shipping_share)
        .checked_div(qty)
        .ok_or(Error::invalid("invoice_price", "amount out of range"))
}

/// Electricity cost: `kWh · price per kWh` (spec 08 §1.3), not rounded.
#[must_use]
pub fn energy_cost_electric(kwh: Decimal, price_kwh: Decimal) -> Decimal {
    kwh * price_kwh
}

/// Electric energy for a heat demand: `kJ / η / 3600` in kWh (spec 08 §1.3).
/// Errors on non-finite input and `eff = 0`.
pub fn kwh_from_heat(heat_kj: f64, eff: f64) -> Result<f64> {
    ensure_finite("heat_kj", heat_kj)?;
    ensure_nonzero("eff", eff)?;
    Ok(heat_kj / eff / 3600.0)
}

/// Rejects non-finite values and zero (a divisor).
fn ensure_nonzero(field: &'static str, x: f64) -> Result<()> {
    ensure_finite(field, x)?;
    if x == 0.0 {
        Err(Error::invalid(field, "must not be 0"))
    } else {
        Ok(())
    }
}

/// Default lower heating value of natural gas, kJ/m³ (spec 08 §1.3, to be verified).
pub const GAS_HU_KJ_M3: f64 = 34_500.0;

/// Gas volume in m³ for a heat demand: `kJ / (η · Hu)` (spec 08 §1.3).
/// Errors on non-finite input and `eff · hu = 0`.
pub fn gas_m3_from_heat(heat_kj: f64, eff: f64, hu_kj_m3: f64) -> Result<f64> {
    ensure_finite("heat_kj", heat_kj)?;
    ensure_finite("eff", eff)?;
    ensure_finite("hu_kj_m3", hu_kj_m3)?;
    ensure_nonzero("eff * hu_kj_m3", eff * hu_kj_m3)?;
    Ok(heat_kj / (eff * hu_kj_m3))
}

/// Batch cost breakdown (spec 08 §1.4, T1.12 C6): lines rounded to 2 decimals half up,
/// `total` is the sum of the rounded lines, `per_litre` is `total / packaged litres` rounded
/// to 3 decimals half up.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchCost {
    /// Ingredients.
    pub ingredients: Decimal,
    /// Water.
    pub water: Decimal,
    /// Energy.
    pub energy: Decimal,
    /// Packaging.
    pub packaging: Decimal,
    /// Sum of the four rounded lines.
    pub total: Decimal,
    /// Cost per packaged litre, 3 decimals.
    pub per_litre: Decimal,
}

/// Inputs of [`batch_cost`].
#[derive(Debug, Clone, Copy)]
pub struct BatchCostInput {
    /// Ingredient cost, currency.
    pub ingredient_cost: Decimal,
    /// Water volume, L.
    pub water_l: Decimal,
    /// Water price per m³, currency.
    pub water_price_m3: Decimal,
    /// Energy cost, currency.
    pub energy_cost: Decimal,
    /// Packaging units used.
    pub pack_units: Decimal,
    /// Cost per packaging unit, currency.
    pub pack_unit_cost: Decimal,
    /// Packaged volume, L.
    pub packaged_l: Decimal,
}

/// Batch cost (spec 08 §1.3–1.4): water in L at a per-m³ price, packaging units, packaged
/// volume in L. Errors when `packaged_l ≤ 0` (the reference raises).
pub fn batch_cost(p: &BatchCostInput) -> Result<BatchCost> {
    let BatchCostInput {
        ingredient_cost,
        water_l,
        water_price_m3,
        energy_cost,
        pack_units,
        pack_unit_cost,
        packaged_l,
    } = *p;
    if packaged_l <= Decimal::ZERO {
        return Err(Error::invalid("packaged_l", "must be > 0"));
    }
    let water = water_l / Decimal::from(1000) * water_price_m3;
    let pack = pack_units * pack_unit_cost;
    let (ingredients, water, energy, packaging) = (
        money(ingredient_cost),
        money(water),
        money(energy_cost),
        money(pack),
    );
    let total = ingredients + water + energy + packaging;
    Ok(BatchCost {
        ingredients,
        water,
        energy,
        packaging,
        total,
        per_litre: total
            .checked_div(packaged_l)
            .ok_or(Error::invalid("packaged_l", "amount out of range"))?
            .round_dp_with_strategy(3, RoundingStrategy::MidpointAwayFromZero),
    })
}

// --------------------------------------------------------------- 2. Blending

/// One blend component: volume (L), ABV (%), IBU, SRM, CO₂ (volumes), extract (°P).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendPart {
    pub volume_l: f64,
    pub abv_pct: f64,
    pub ibu: f64,
    pub srm: f64,
    pub co2_vol: f64,
    pub wort_p: f64,
}

/// Blend result, unrounded (T1.12 C9). An empty or zero-volume blend yields NaN (the
/// reference raises; error type: T1.16).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blend {
    pub volume_l: f64,
    pub abv_pct: f64,
    pub ibu: f64,
    pub srm: f64,
    pub co2_vol: f64,
    pub wort_p: f64,
}

/// Blend of components (spec 08 §2.1): volume-weighted ABV, IBU, SRM, CO₂; mass-weighted °P.
/// Errors (T1.16, was NaN) on an empty blend, a total volume or mass of 0, and non-finite input.
pub fn blend(parts: &[BlendPart]) -> Result<Blend> {
    for q in parts {
        for (field, x) in [
            ("v", q.volume_l),
            ("abv", q.abv_pct),
            ("ibu", q.ibu),
            ("srm", q.srm),
            ("co2", q.co2_vol),
            ("p", q.wort_p),
        ] {
            ensure_finite(field, x)?;
        }
    }
    // Compensated sums like CPython's `sum` (T1.12 C1).
    let v = py_sum(parts.iter().map(|p| p.volume_l));
    ensure_nonzero("total volume", v)?;
    let wavg = |f: fn(&BlendPart) -> f64| py_sum(parts.iter().map(|p| p.volume_l * f(p))) / v;
    let m: Vec<f64> = parts.iter().map(|p| p.volume_l * to_sg(p.wort_p)).collect();
    let mass = py_sum(m.iter().copied());
    ensure_nonzero("total mass", mass)?;
    let p = py_sum(m.iter().zip(parts).map(|(mi, q)| mi * q.wort_p)) / mass;
    Ok(Blend {
        volume_l: v,
        abv_pct: wavg(|q| q.abv_pct),
        ibu: wavg(|q| q.ibu),
        srm: wavg(|q| q.srm),
        co2_vol: wavg(|q| q.co2_vol),
        wort_p: p,
    })
}

/// Volume of the second component for a target value (spec 08 §2.2); an error (T1.16, was
/// `None`) unless the target lies strictly between `x1` and `x2`, with a 1e-9 margin at both
/// ends (T1.12 C7), or an input is non-finite.
pub fn blend_volume_for_target(v1_l: f64, x1: f64, x2: f64, xt: f64) -> Result<f64> {
    for (field, x) in [("v1_l", v1_l), ("x1", x1), ("x2", x2), ("xt", xt)] {
        ensure_finite(field, x)?;
    }
    if !(x1.min(x2) + 1e-9 < xt && xt < x1.max(x2) - 1e-9) {
        return Err(Error::Unreachable {
            what: "blend target",
        });
    }
    Ok(v1_l * (x1 - xt) / (xt - x2))
}

// -------------------------------------------------------------- 3. Yeast reuse

/// Cells needed in billions: `pitch rate (M cells/mL/°P) · V (L) · °P` (spec 08 §3.1).
/// Errors on non-finite input.
pub fn cells_needed_billion(pitch_m_ml_p: f64, v_l: f64, wort_p: f64) -> Result<f64> {
    ensure_finite("pitch_m_ml_p", pitch_m_ml_p)?;
    ensure_finite("v_l", v_l)?;
    ensure_finite("wort_p", wort_p)?;
    Ok(pitch_m_ml_p * v_l * wort_p)
}

/// Neubauer count in cells/mL: `counted / squares · 25 · 10⁴ · dilution` (spec 08 §3.2).
/// Errors on non-finite input and `squares_counted = 0`.
pub fn hemocytometer(total_counted: f64, squares_counted: f64, dilution: f64) -> Result<f64> {
    ensure_finite("total_counted", total_counted)?;
    ensure_nonzero("squares_counted", squares_counted)?;
    ensure_finite("dilution", dilution)?;
    Ok(total_counted / squares_counted * 25.0 * 1e4 * dilution)
}

/// Viability from a stain count: `unstained / (unstained + stained)` (spec 08 §3.2).
/// Errors on non-finite input and `unstained + stained = 0`.
pub fn viability_stain(unstained: f64, stained: f64) -> Result<f64> {
    ensure_finite("unstained", unstained)?;
    ensure_finite("stained", stained)?;
    ensure_nonzero("unstained + stained", unstained + stained)?;
    Ok(unstained / (unstained + stained))
}

/// Default effective slurry density, cells/mL (spec 08 §3.3, to be verified).
pub const SLURRY_DEFAULT_DENSITY: f64 = 1.0e9;

/// Slurry volume in mL: `N · 10⁹ / (density · viability)` (spec 08 §3.3). A missing or zero
/// measured density falls back to `default_density_cells_ml`. Errors on non-finite input and a zero
/// density or viability.
pub fn slurry_volume_ml(
    cells_billion: f64,
    density_cells_ml: Option<f64>,
    viability: f64,
    default_density_cells_ml: f64,
) -> Result<f64> {
    ensure_finite("cells_billion", cells_billion)?;
    ensure_finite("viability", viability)?;
    ensure_finite("default_density_cells_ml", default_density_cells_ml)?;
    let d = match density_cells_ml {
        Some(x) if x != 0.0 => x,
        _ => default_density_cells_ml,
    };
    ensure_finite("density", d)?;
    ensure_nonzero("density * viability", d * viability)?;
    Ok(cells_billion * 1e9 / (d * viability))
}

// ------------------------------------------------- 4. Professional efficiency

/// Extract in the kettle, kg: `V · (0.96 if hot) · SG(°P) · °P / 100` (spec 08 §4.1).
/// Errors on non-finite input.
pub fn extract_kg(v_l: f64, wort_p: f64, hot: bool) -> Result<f64> {
    ensure_finite("v_l", v_l)?;
    ensure_finite("wort_p", wort_p)?;
    let v = v_l * if hot { CONTRACTION } else { 1.0 };
    finite_result("extract_kg", v * to_sg(wort_p) * wort_p / 100.0)
}

/// Brewhouse yield (Sudhausausbeute) in %: `100 · extract / malt` (spec 08 §4.2).
/// Errors on non-finite input and `malt_kg = 0`.
pub fn brewhouse_yield(v_l: f64, wort_p: f64, malt_kg: f64, hot: bool) -> Result<f64> {
    ensure_nonzero("malt_kg", malt_kg)?;
    Ok(100.0 * extract_kg(v_l, wort_p, hot)? / malt_kg)
}

/// Laboratory extract as-is, %: `(FGDB − coarse/fine diff) · (1 − moisture/100)` (spec 08 §4.2, §5.1).
#[must_use]
pub fn lab_extract_as_is(fgdb_pct: f64, moisture_pct: f64, fc_diff_pct: f64) -> f64 {
    (fgdb_pct - fc_diff_pct) * (1.0 - moisture_pct / 100.0)
}

/// Maximum first-wort °P at full conversion (spec 08 §4.3, Braukaiser). Errors on non-finite
/// input and when extract plus water is 0.
pub fn first_wort_max_p(
    grain_kg: f64,
    extract_as_is_pct: f64,
    moisture_pct: f64,
    mash_water_l: f64,
) -> Result<f64> {
    for (field, x) in [
        ("grain_kg", grain_kg),
        ("extract_as_is_pct", extract_as_is_pct),
        ("moisture_pct", moisture_pct),
        ("mash_water_l", mash_water_l),
    ] {
        ensure_finite(field, x)?;
    }
    let e = grain_kg * extract_as_is_pct / 100.0;
    let w = mash_water_l + grain_kg * moisture_pct / 100.0;
    ensure_nonzero("extract + water", e + w)?;
    Ok(100.0 * e / (e + w))
}

/// Conversion efficiency (ratio): measured / maximum first-wort °P (spec 08 §4.3).
/// Errors on non-finite input and `fw_max_p = 0`.
pub fn conversion_eff(fw_measured_p: f64, fw_max_p: f64) -> Result<f64> {
    ensure_finite("fw_measured_p", fw_measured_p)?;
    ensure_nonzero("fw_max_p", fw_max_p)?;
    Ok(fw_measured_p / fw_max_p)
}

/// Brewhouse yield to lab extract ratio (spec 08 §4.2, as a fraction; ×100 for the percentage).
/// Errors on non-finite input and `lab_extract_pct = 0` (T1.17).
pub fn lab_ratio(brewhouse_pct: f64, lab_extract_pct: f64) -> Result<f64> {
    ensure_finite("brewhouse_pct", brewhouse_pct)?;
    ensure_nonzero("lab_extract_pct", lab_extract_pct)?;
    Ok(brewhouse_pct / lab_extract_pct)
}

/// Lauter efficiency (ratio): `(brewhouse yield / lab extract) / conversion efficiency` (spec 08
/// §4.3). Errors on non-finite input and when the lab extract or the conversion efficiency is 0.
/// Meaningless for BIAB and no-sparge mashes; hiding it is the caller's job (T1.17).
pub fn lauter_eff(brewhouse_pct: f64, lab_extract_pct: f64, conversion: f64) -> Result<f64> {
    ensure_finite("conversion", conversion)?;
    ensure_nonzero("conversion", conversion)?;
    Ok(lab_ratio(brewhouse_pct, lab_extract_pct)? / conversion)
}

// ------------------------------------------------------ 5. Malt analysis

/// Malt potential from lab analysis (spec 08 §5.1–5.2), unrounded (T1.12 C9).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaltAnalysis {
    /// Fine grind, as-is extract %.
    pub fgai_pct: f64,
    /// Coarse grind, dry basis %.
    pub cgdb_pct: f64,
    /// Coarse grind, as-is % (the recipe yield).
    pub cgai_yield_pct: f64,
    /// Potential SG with sucrose 46.21 PPG.
    pub potential: f64,
    /// Potential SG with the BeerSmith constant 0.046.
    pub potential_bs: f64,
    /// Points per pound per gallon.
    pub ppg: f64,
}

/// Malt analysis → extract potential (spec 08 §5.1–5.2).
#[must_use]
pub fn malt_from_analysis(fgdb_pct: f64, moisture_pct: f64, fc_diff_pct: f64) -> MaltAnalysis {
    let fgai = fgdb_pct * (1.0 - moisture_pct / 100.0);
    let cgdb = fgdb_pct - fc_diff_pct;
    let cgai = cgdb * (1.0 - moisture_pct / 100.0);
    let yield_pct = cgai;
    MaltAnalysis {
        fgai_pct: fgai,
        cgdb_pct: cgdb,
        cgai_yield_pct: cgai,
        potential: 1.0 + yield_pct * 0.04621 / 100.0,
        potential_bs: 1.0 + yield_pct * 0.046 / 100.0,
        ppg: yield_pct * 0.4621,
    }
}

/// Colour from a lab EBC value (spec 08 §5.3), unrounded.
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
        srm: ebc / 1.97,
        lovibond: (ebc / 1.97 + 0.76) / 1.3546,
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

/// Inputs of [`yeast_reuse_warnings`].
#[derive(Debug, Clone, Copy)]
pub struct YeastReuseInput {
    /// Generation of the harvested yeast.
    pub generation: f64,
    /// Days since harvest.
    pub harvest_age_days: f64,
    /// Measured viability as a fraction (0-1), `None` when not measured.
    pub viability: Option<f64>,
    /// Maximum generation before a warning.
    pub max_generation: f64,
    /// True when the culture history differs from the recipe.
    pub history_differs: bool,
}

/// Yeast reuse warnings in table order (spec §3.4). `viability` is a fraction (0–1) or
/// `None` when not measured (no viability warning then).
#[must_use]
pub fn yeast_reuse_warnings(p: &YeastReuseInput) -> Vec<YeastWarning> {
    let YeastReuseInput {
        generation,
        harvest_age_days: days_since_harvest,
        viability,
        max_generation,
        history_differs,
    } = *p;
    let mut out = Vec::new();
    if generation > max_generation {
        out.push(YeastWarning::GenerationLimit);
    }
    if days_since_harvest > 14.0 + 1e-9 {
        out.push(YeastWarning::HarvestOver14Days);
    }
    if viability.is_some_and(|v| v < 0.80 - 1e-9) {
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
    pub average_lintner: f64,
    /// True when the grain bill may not convert itself (average below the threshold).
    pub low_diastatic_power: bool,
}

/// "Does the grain bill convert itself" check (spec §5.4); warns when
/// `average < threshold − 1e-9` (T1.12 C1), weighted with compensated sums. `grains` are
/// `(kg, diastatic power °L)` for every grain (0 when a grain has none). An empty or
/// zero-weight bill gives average 0 and no warning.
#[must_use]
pub fn diastatic_check(grains: &[(f64, f64)], threshold_lintner: f64) -> DiastaticCheck {
    let w = py_sum(grains.iter().map(|g| g.0));
    if w <= 0.0 {
        return DiastaticCheck {
            average_lintner: 0.0,
            low_diastatic_power: false,
        };
    }
    let average = py_sum(grains.iter().map(|g| g.0 * g.1)) / w;
    DiastaticCheck {
        average_lintner: average,
        low_diastatic_power: average < threshold_lintner - 1e-9,
    }
}
