//! `BreweryService` implementation: batch cost, blending, yeast reuse, professional efficiency,
//! malt analysis, label energy, tax and spunding over `engine::brewery` and
//! `engine::brewery_tax` (specs 08 and 09). Money, stock and tax values are decimal text on the
//! wire and `rust_decimal::Decimal` inside; physical quantities are `f64`.

use engine::brewery::{
    self as eb, BatchCostInput, BlendPart, DIASTATIC_MIN_L, GAS_HU_KJ_M3, Lot,
    SLURRY_DEFAULT_DENSITY, YEAST_MAX_GENERATION, YeastReuseInput, YeastWarning,
};
use engine::brewery_tax::{self as bt, LabelDeclaration as EngineDeclaration, SpundWarning};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use tonic::{Request, Response, Status};

use crate::check::{finite, non_negative, out};
use crate::convert;
use crate::proto::{
    BatchCostRequest, BatchCostResponse, BlendRequest, BlendResponse, BlendToTargetRequest,
    BlendToTargetResponse, BrewhouseEfficiencyRequest, BrewhouseEfficiencyResponse, CostEnergy,
    DiastaticCheckRequest, DiastaticCheckResponse, FifoRow, IngredientCost, LabelDeclaration,
    LabelEnergyRequest, LabelEnergyResponse, LandedPriceRequest, LandedPriceResponse,
    MaltAnalysisRequest, MaltAnalysisResponse, MaltLabValues, SpundingRequest, SpundingResponse,
    SpundingWarning, TaxReportRequest, TaxReportResponse, YeastReuseRequest, YeastReuseResponse,
    YeastReuseWarning,
    brewery_service_server::{BreweryService, BreweryServiceServer},
    cost_energy::Source,
};
use crate::status::{bad_request, from_engine};

mod dec;
mod tax;

/// Handler implementing the brewery gRPC service.
#[derive(Debug, Default, Clone, Copy)]
pub struct BreweryHandler;

/// Builds the tonic server wrapper for the brewery handler.
#[must_use]
pub fn server() -> BreweryServiceServer<BreweryHandler> {
    BreweryServiceServer::new(BreweryHandler)
}

/// `INVALID_ARGUMENT` for a value outside its allowed range.
fn range_error(field: &str, why: &str) -> Status {
    bad_request(&format!("{field} {why}"), field, why)
}

/// Converts an engine `f64` quantity to a decimal; out of range is `INVALID_ARGUMENT`.
fn to_decimal(field: &str, v: f64) -> Result<Decimal, Status> {
    Decimal::from_f64(v).ok_or_else(|| range_error(field, "result is out of range"))
}

// ------------------------------------------------------------------------------- 1. Cost

/// Energy line of the batch cost: cost plus the electric kWh or gas m3 behind it.
struct EnergyLine {
    cost: Decimal,
    kwh: Option<f64>,
    gas_m3: Option<f64>,
}

fn energy_line(energy: Option<&CostEnergy>) -> Result<EnergyLine, Status> {
    let mut line = EnergyLine {
        cost: Decimal::ZERO,
        kwh: None,
        gas_m3: None,
    };
    let Some(e) = energy else {
        return Ok(line);
    };
    let heat = non_negative("energy.heat_kj", e.heat_kj)?;
    let eff = finite("energy.efficiency", e.efficiency)?;
    if eff <= 0.0 || eff > 1.0 {
        return Err(range_error("energy.efficiency", "must be in (0, 1]"));
    }
    match &e.source {
        Some(Source::Electric(el)) => {
            let price = dec::stock("energy.electric.price_per_kwh", &el.price_per_kwh)?;
            let extra = dec::stock_or_zero("energy.electric.additional_kwh", &el.additional_kwh)?;
            let kwh = out(
                "energy_kwh",
                eb::kwh_from_heat(heat, eff).map_err(|e| from_engine(&e))?,
            )?;
            let total = dec::add("energy_kwh", to_decimal("energy_kwh", kwh)?, extra)?;
            line.cost = eb::energy_cost_electric(total, price);
            line.kwh = Some(kwh + extra.to_f64().unwrap_or(0.0));
        }
        Some(Source::Gas(g)) => {
            let price = dec::stock("energy.gas.price_per_m3", &g.price_per_m3)?;
            let hu = match g.heating_value_kj_m3 {
                Some(v) => finite("energy.gas.heating_value_kj_m3", v)?,
                None => GAS_HU_KJ_M3,
            };
            let m3 = out(
                "gas_m3",
                eb::gas_m3_from_heat(heat, eff, hu).map_err(|e| from_engine(&e))?,
            )?;
            line.cost = eb::energy_cost_electric(to_decimal("gas_m3", m3)?, price);
            line.gas_m3 = Some(m3);
        }
        None => {
            return Err(bad_request(
                "energy source must be set",
                "energy.source",
                "must be electric or gas",
            ));
        }
    }
    Ok(line)
}

fn batch_cost(r: &BatchCostRequest) -> Result<BatchCostResponse, Status> {
    let mut ingredients = Vec::with_capacity(r.ingredients.len());
    let mut ingredient_total = Decimal::ZERO;
    for (i, ing) in r.ingredients.iter().enumerate() {
        let mut lots = Vec::with_capacity(ing.lots.len());
        for (j, l) in ing.lots.iter().enumerate() {
            lots.push(Lot {
                lot: l.lot.clone(),
                left: dec::stock(
                    &format!("ingredients[{i}].lots[{j}].remaining"),
                    &l.remaining,
                )?,
                price: dec::stock(
                    &format!("ingredients[{i}].lots[{j}].unit_price"),
                    &l.unit_price,
                )?,
            });
        }
        let qty = dec::stock(&format!("ingredients[{i}].quantity"), &ing.quantity)?;
        let fifo = eb::fifo_consume(&lots, qty);
        ingredient_total = dec::add("ingredients", ingredient_total, fifo.cost)?;
        ingredients.push(IngredientCost {
            name: ing.name.clone(),
            rows: fifo
                .rows
                .iter()
                .map(|row| FifoRow {
                    lot: row.lot.clone(),
                    taken: dec::fmt(row.take),
                    unit_price: dec::fmt(row.price),
                    cost: dec::fmt(row.cost),
                })
                .collect(),
            cost: dec::fmt(fifo.cost),
            missing: dec::fmt(fifo.missing),
        });
    }

    let (water_l, water_price_m3) = match &r.water {
        Some(w) => (
            dec::stock("water.water_l", &w.water_l)?,
            dec::stock("water.price_per_m3", &w.price_per_m3)?,
        ),
        None => (Decimal::ZERO, Decimal::ZERO),
    };

    let energy = energy_line(r.energy.as_ref())?;
    let (energy_cost, energy_kwh, gas_m3) = (energy.cost, energy.kwh, energy.gas_m3);

    let c = eb::batch_cost(&BatchCostInput {
        ingredient_cost: ingredient_total,
        water_l,
        water_price_m3,
        energy_cost,
        pack_units: dec::stock("pack_units", &r.pack_units)?,
        pack_unit_cost: dec::stock("pack_unit_cost", &r.pack_unit_cost)?,
        packaged_l: dec::stock("packaged_l", &r.packaged_l)?,
    })
    .map_err(|e| from_engine(&e))?;

    Ok(BatchCostResponse {
        ingredients,
        ingredients_cost: dec::fmt(c.ingredients),
        water_cost: dec::fmt(c.water),
        energy_cost: dec::fmt(c.energy),
        packaging_cost: dec::fmt(c.packaging),
        total_cost: dec::fmt(c.total),
        cost_per_l: dec::fmt(c.per_litre),
        energy_kwh,
        gas_m3,
    })
}

// ----------------------------------------------------------------- 3. Yeast and 4. efficiency

fn yeast_reuse(r: &YeastReuseRequest) -> Result<YeastReuseResponse, Status> {
    let cells = out(
        "cells_needed_billion",
        eb::cells_needed_billion(
            non_negative("pitch_rate_m_ml_p", r.pitch_rate_m_ml_p)?,
            non_negative("wort_volume_l", r.wort_volume_l)?,
            non_negative("wort_extract_p", r.wort_extract_p)?,
        )
        .map_err(|e| from_engine(&e))?,
    )?;

    let counted = match &r.count {
        Some(c) => Some(out(
            "counted_density_cells_ml",
            eb::hemocytometer(
                non_negative("count.total_counted", c.total_counted)?,
                non_negative("count.squares_counted", c.squares_counted)?,
                non_negative("count.dilution", c.dilution)?,
            )
            .map_err(|e| from_engine(&e))?,
        )?),
        None => None,
    };
    let measured = match (counted, r.measured_density_cells_ml) {
        (Some(c), _) => Some(c),
        (None, Some(m)) => Some(non_negative("measured_density_cells_ml", m)?),
        (None, None) => None,
    };
    let default_density = match r.default_density_cells_ml {
        Some(d) => non_negative("default_density_cells_ml", d)?,
        None => SLURRY_DEFAULT_DENSITY,
    };

    let viability = if let Some(s) = &r.stain {
        Some(
            eb::viability_stain(
                non_negative("stain.unstained", s.unstained)?,
                non_negative("stain.stained", s.stained)?,
            )
            .map_err(|e| from_engine(&e))?,
        )
    } else if let Some(p) = r.viability_pct {
        let p = finite("viability_pct", p)?;
        if !(0.0..=100.0).contains(&p) {
            return Err(range_error("viability_pct", "must be between 0 and 100"));
        }
        Some(p / 100.0)
    } else {
        None
    };

    let slurry = out(
        "slurry_volume_ml",
        eb::slurry_volume_ml(cells, measured, viability.unwrap_or(1.0), default_density)
            .map_err(|e| from_engine(&e))?,
    )?;
    let density = match measured {
        Some(m) if m != 0.0 => m,
        _ => default_density,
    };

    let max_generation = match r.max_generation {
        Some(g) => non_negative("max_generation", g)?,
        None => f64::from(YEAST_MAX_GENERATION),
    };
    let warnings = eb::yeast_reuse_warnings(&YeastReuseInput {
        generation: non_negative("generation", r.generation)?,
        harvest_age_days: non_negative("harvest_age_days", r.harvest_age_days)?,
        viability,
        max_generation,
        history_differs: r.history_differs,
    })
    .into_iter()
    .map(|w| {
        (match w {
            YeastWarning::GenerationLimit => YeastReuseWarning::GenerationLimit,
            YeastWarning::HarvestOver14Days => YeastReuseWarning::HarvestOver14Days,
            YeastWarning::LowViability => YeastReuseWarning::LowViability,
            YeastWarning::HistoryInfo => YeastReuseWarning::HistoryInfo,
        }) as i32
    })
    .collect();

    Ok(YeastReuseResponse {
        cells_needed_billion: cells,
        density_cells_ml: density,
        counted_density_cells_ml: counted,
        viability_pct: viability.map(|v| v * 100.0),
        slurry_volume_ml: slurry,
        warnings,
    })
}

fn lab_values(lab: &MaltLabValues) -> Result<(f64, f64, f64), Status> {
    Ok((
        non_negative("lab.fine_grind_dry_pct", lab.fine_grind_dry_pct)?,
        non_negative("lab.moisture_pct", lab.moisture_pct)?,
        non_negative("lab.fine_coarse_diff_pct", lab.fine_coarse_diff_pct)?,
    ))
}

fn brewhouse_efficiency(
    r: &BrewhouseEfficiencyRequest,
) -> Result<BrewhouseEfficiencyResponse, Status> {
    let (v, p, hot) = (
        non_negative("wort_volume_l", r.wort_volume_l)?,
        non_negative("wort_extract_p", r.wort_extract_p)?,
        r.measured_hot,
    );
    let malt = non_negative("malt_kg", r.malt_kg)?;
    let extract = out(
        "extract_kg",
        eb::extract_kg(v, p, hot).map_err(|e| from_engine(&e))?,
    )?;
    let yield_pct = out(
        "brewhouse_yield_pct",
        eb::brewhouse_yield(v, p, malt, hot).map_err(|e| from_engine(&e))?,
    )?;
    let mut resp = BrewhouseEfficiencyResponse {
        extract_kg: extract,
        brewhouse_yield_pct: yield_pct,
        lab_extract_as_is_pct: None,
        first_wort_max_p: None,
        conversion_efficiency: None,
        lab_ratio: None,
        lauter_efficiency: None,
    };
    if r.lab.is_none() && (r.mash_water_l.is_some() || r.first_wort_p.is_some()) {
        return Err(bad_request(
            "lab values are required for the first-wort checks",
            "lab",
            "required with mash_water_l or first_wort_p",
        ));
    }
    if let Some(lab) = &r.lab {
        let (fgdb, moisture, diff) = lab_values(lab)?;
        let as_is = out(
            "lab_extract_as_is_pct",
            eb::lab_extract_as_is(fgdb, moisture, diff),
        )?;
        resp.lab_extract_as_is_pct = Some(as_is);
        resp.lab_ratio = Some(out(
            "lab_ratio",
            eb::lab_ratio(yield_pct, as_is).map_err(|e| from_engine(&e))?,
        )?);
        match r.mash_water_l {
            Some(w) => {
                let max = out(
                    "first_wort_max_p",
                    eb::first_wort_max_p(malt, as_is, moisture, non_negative("mash_water_l", w)?)
                        .map_err(|e| from_engine(&e))?,
                )?;
                resp.first_wort_max_p = Some(max);
                if let Some(fw) = r.first_wort_p {
                    let conversion = out(
                        "conversion_efficiency",
                        eb::conversion_eff(non_negative("first_wort_p", fw)?, max)
                            .map_err(|e| from_engine(&e))?,
                    )?;
                    resp.conversion_efficiency = Some(conversion);
                    if !r.no_sparge {
                        resp.lauter_efficiency = Some(out(
                            "lauter_efficiency",
                            eb::lauter_eff(yield_pct, as_is, conversion)
                                .map_err(|e| from_engine(&e))?,
                        )?);
                    }
                }
            }
            None if r.first_wort_p.is_some() => {
                return Err(bad_request(
                    "mash water is required for the conversion efficiency",
                    "mash_water_l",
                    "required with first_wort_p",
                ));
            }
            None => {}
        }
    }
    Ok(resp)
}

// ------------------------------------------------------------- 6. Label energy and 8. spunding

fn label_energy(r: &LabelEnergyRequest) -> Result<LabelEnergyResponse, Status> {
    let protein = match r.protein_g_100ml {
        Some(p) => non_negative("protein_g_100ml", p)?,
        None => 0.40,
    };
    let measured = r
        .measured_abv_pct
        .map(|v| non_negative("measured_abv_pct", v))
        .transpose()?;
    let e = bt::label_energy(
        finite("og_sg", r.og_sg)?,
        finite("fg_sg", r.fg_sg)?,
        protein,
        measured,
    )
    .map_err(|e| from_engine(&e))?;
    // The declaration reads the rounded ABV as an exact two-decimal number.
    let abv: Decimal = format!("{:.2}", e.abv_pct)
        .parse()
        .map_err(|_| range_error("abv_pct", "result is out of range"))?;
    let declaration = match bt::label_declaration(abv) {
        EngineDeclaration::EnergyOptional => LabelDeclaration::EnergyOptional,
        EngineDeclaration::CountryTemplate => LabelDeclaration::CountryTemplate,
    };
    Ok(LabelEnergyResponse {
        abv_pct: e.abv_pct,
        abv_measured: measured.is_some(),
        alcohol_g: e.alcohol_g,
        real_extract_g: e.real_extract_g,
        carbohydrate_g: e.carbohydrate_g,
        protein_g: e.protein_g,
        energy_kj: e.kj,
        energy_kcal: e.kcal,
        declaration: declaration as i32,
    })
}

fn spunding(r: &SpundingRequest) -> Result<SpundingResponse, Status> {
    let target = non_negative("target_co2_vol", r.target_co2_vol)?;
    let temp = finite("temp_c", r.temp_c)?;
    let fg = finite("fg_expected_sg", r.fg_expected_sg)?;
    let plan = bt::spund(target, temp, fg).map_err(|e| from_engine(&e))?;
    let tank_max_psi = r
        .tank_max_bar
        .map(|b| non_negative("tank_max_bar", b).map(convert::bar_to_psi))
        .transpose()?;
    let fg_range = match (r.fg_low_sg, r.fg_high_sg) {
        (Some(lo), Some(hi)) => Some((finite("fg_low_sg", lo)?, finite("fg_high_sg", hi)?)),
        (None, None) => None,
        _ => {
            return Err(bad_request(
                "fg_low_sg and fg_high_sg must be set together",
                "fg_low_sg",
                "both or neither",
            ));
        }
    };
    let wide_u = match r.wide_limit_units {
        Some(u) if u < 0 => return Err(range_error("wide_limit_units", "must not be negative")),
        Some(u) => u,
        None => bt::SPUND_WIDE_U,
    };
    let w = bt::spund_warnings(&bt::SpundWarningsInput {
        target_co2_vol: target,
        t_c: temp,
        tank_max_psi,
        fg_range,
        wide_u,
    });
    Ok(SpundingResponse {
        residual_co2_vol: out("residual_co2_vol", plan.residual_co2_vol)?,
        needed_co2_g_l: out("needed_co2_g_l", plan.needed_g_per_l)?,
        closing_sg: out("closing_sg", plan.closing_sg)?,
        points_before_fg: out("points_before_fg", plan.points_before)?,
        setting_bar: out("setting_bar", plan.setting_bar)?,
        setting_psi: out("setting_psi", plan.setting_psi)?,
        warnings: w
            .warnings
            .iter()
            .map(|x| {
                (match x {
                    SpundWarning::TankPressureExceeded => SpundingWarning::TankPressureExceeded,
                    SpundWarning::EarlyClosingRisk => SpundingWarning::EarlyClosingRisk,
                }) as i32
            })
            .collect(),
        blocked: w.blocked,
    })
}

// ---------------------------------------------------------------------------------- service

#[tonic::async_trait]
impl BreweryService for BreweryHandler {
    async fn batch_cost(
        &self,
        request: Request<BatchCostRequest>,
    ) -> Result<Response<BatchCostResponse>, Status> {
        batch_cost(&request.into_inner()).map(Response::new)
    }

    async fn landed_price(
        &self,
        request: Request<LandedPriceRequest>,
    ) -> Result<Response<LandedPriceResponse>, Status> {
        let r = request.into_inner();
        let price = eb::landed_price(
            dec::stock("invoice_unit_price", &r.invoice_unit_price)?,
            dec::stock("quantity", &r.quantity)?,
            dec::stock("shipping_share", &r.shipping_share)?,
        )
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(LandedPriceResponse {
            unit_price: dec::fmt(price),
        }))
    }

    async fn blend(
        &self,
        request: Request<BlendRequest>,
    ) -> Result<Response<BlendResponse>, Status> {
        let r = request.into_inner();
        let mut parts = Vec::with_capacity(r.components.len());
        for (i, c) in r.components.iter().enumerate() {
            let f = |name: &str, v: f64| non_negative(&format!("components[{i}].{name}"), v);
            parts.push(BlendPart {
                volume_l: f("volume_l", c.volume_l)?,
                abv_pct: f("abv_pct", c.abv_pct)?,
                ibu: f("ibu", c.ibu)?,
                srm: f("color_srm", c.color_srm)?,
                co2_vol: f("co2_vol", c.co2_vol)?,
                wort_p: f("extract_p", c.extract_p)?,
            });
        }
        let b = eb::blend(&parts).map_err(|e| from_engine(&e))?;
        Ok(Response::new(BlendResponse {
            volume_l: out("volume_l", b.volume_l)?,
            abv_pct: out("abv_pct", b.abv_pct)?,
            ibu: out("ibu", b.ibu)?,
            color_srm: out("color_srm", b.srm)?,
            co2_vol: out("co2_vol", b.co2_vol)?,
            extract_p: out("extract_p", b.wort_p)?,
        }))
    }

    async fn blend_to_target(
        &self,
        request: Request<BlendToTargetRequest>,
    ) -> Result<Response<BlendToTargetResponse>, Status> {
        let r = request.into_inner();
        let v = eb::blend_volume_for_target(
            non_negative("first_volume_l", r.first_volume_l)?,
            finite("first_value", r.first_value)?,
            finite("second_value", r.second_value)?,
            finite("target_value", r.target_value)?,
        )
        .map_err(|e| from_engine(&e))?;
        Ok(Response::new(BlendToTargetResponse {
            second_volume_l: out("second_volume_l", v)?,
        }))
    }

    async fn yeast_reuse(
        &self,
        request: Request<YeastReuseRequest>,
    ) -> Result<Response<YeastReuseResponse>, Status> {
        yeast_reuse(&request.into_inner()).map(Response::new)
    }

    async fn brewhouse_efficiency(
        &self,
        request: Request<BrewhouseEfficiencyRequest>,
    ) -> Result<Response<BrewhouseEfficiencyResponse>, Status> {
        brewhouse_efficiency(&request.into_inner()).map(Response::new)
    }

    async fn malt_analysis(
        &self,
        request: Request<MaltAnalysisRequest>,
    ) -> Result<Response<MaltAnalysisResponse>, Status> {
        let r = request.into_inner();
        let lab = r
            .lab
            .as_ref()
            .ok_or_else(|| bad_request("lab values are required", "lab", "must be set"))?;
        let (fgdb, moisture, diff) = lab_values(lab)?;
        let m = eb::malt_from_analysis(fgdb, moisture, diff);
        let color = r
            .color_ebc
            .map(|c| non_negative("color_ebc", c).map(eb::ebc_from_lab))
            .transpose()?;
        Ok(Response::new(MaltAnalysisResponse {
            fine_grind_as_is_pct: out("fine_grind_as_is_pct", m.fgai_pct)?,
            coarse_grind_dry_pct: out("coarse_grind_dry_pct", m.cgdb_pct)?,
            yield_pct: out("yield_pct", m.cgai_yield_pct)?,
            potential_sg: out("potential_sg", m.potential)?,
            potential_sg_beersmith: out("potential_sg_beersmith", m.potential_bs)?,
            ppg: out("ppg", m.ppg)?,
            color_srm: color.map(|c| out("color_srm", c.srm)).transpose()?,
            color_lovibond: color
                .map(|c| out("color_lovibond", c.lovibond))
                .transpose()?,
        }))
    }

    async fn diastatic_check(
        &self,
        request: Request<DiastaticCheckRequest>,
    ) -> Result<Response<DiastaticCheckResponse>, Status> {
        let r = request.into_inner();
        let mut grains = Vec::with_capacity(r.grains.len());
        for (i, g) in r.grains.iter().enumerate() {
            grains.push((
                non_negative(&format!("grains[{i}].amount_kg"), g.amount_kg)?,
                non_negative(
                    &format!("grains[{i}].diastatic_power_lintner"),
                    g.diastatic_power_lintner,
                )?,
            ));
        }
        let threshold = match r.threshold_lintner {
            Some(t) => non_negative("threshold_lintner", t)?,
            None => DIASTATIC_MIN_L,
        };
        let c = eb::diastatic_check(&grains, threshold);
        Ok(Response::new(DiastaticCheckResponse {
            average_lintner: out("average_lintner", c.average_lintner)?,
            low_diastatic_power: c.low_diastatic_power,
        }))
    }

    async fn label_energy(
        &self,
        request: Request<LabelEnergyRequest>,
    ) -> Result<Response<LabelEnergyResponse>, Status> {
        label_energy(&request.into_inner()).map(Response::new)
    }

    async fn tax_report(
        &self,
        request: Request<TaxReportRequest>,
    ) -> Result<Response<TaxReportResponse>, Status> {
        tax::report(&request.into_inner()).map(Response::new)
    }

    async fn spunding(
        &self,
        request: Request<SpundingRequest>,
    ) -> Result<Response<SpundingResponse>, Status> {
        spunding(&request.into_inner()).map(Response::new)
    }
}

#[cfg(test)]
mod tests;
