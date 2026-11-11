//! Section 03: water chemistry and mash pH (`formuller/03-su-ph.md`).
//!
//! Units follow the specification (BeerSmith internal units): ion concentrations in ppm
//! (mg/L), alkalinity and RA in ppm as `CaCO₃` (50 ppm = 1 mEq/L), mash thickness in L/kg,
//! grain weights in kg, grain colour in SRM, acid strength in percent by weight, volumes in
//! litres. Conversion from other units is the API layer's job (ADR-0003).
//!
//! Acid codes (`F_R_ACID`, spec §0.2 and §6.1): 0 none, 1 lactic, 2 phosphoric, 3 acid malt,
//! 4 citric, 5 acetic, 6 `HCl`, 7 sulphuric, 8 tartaric. Model codes: 0 MPH 3 (default),
//! 1 BW. The `ours` flag selects the project's own model (spec §6.8) instead of the
//! BeerSmith-compatible one.
//!
//! The reference is `formuller/testler/03_su_ph_tv.py`; where the spec text and the reference
//! differ, the reference is followed (e.g. phosphoric density coefficient 0.375 in
//! [`acid_per_liter`], as in the reference).

#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::needless_range_loop
)]

use crate::units::GAL_L;

/// Number of tracked ions: Ca, Mg, Na, SO₄, Cl, HCO₃ (in this order).
pub const ION_COUNT: usize = 6;

/// Water-treatment salts in the column order of the contribution matrix (spec §1.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Salt {
    /// Gypsum, `CaSO₄·2H₂O`.
    Gypsum,
    /// Table salt, `NaCl`.
    Nacl,
    /// Epsom salt, `MgSO₄·7H₂O`.
    Epsom,
    /// Calcium chloride, anhydrous `CaCl₂`.
    Cacl2,
    /// Baking soda, `NaHCO₃`.
    Soda,
    /// Chalk, `CaCO₃`.
    Chalk,
    /// Magnesium chloride, `MgCl₂·6H₂O`.
    Mgcl2,
    /// Pickling lime, `Ca(OH)₂`.
    Lime,
}

impl Salt {
    /// All salts in matrix column order.
    pub const ALL: [Salt; 8] = [
        Salt::Gypsum,
        Salt::Nacl,
        Salt::Epsom,
        Salt::Cacl2,
        Salt::Soda,
        Salt::Chalk,
        Salt::Mgcl2,
        Salt::Lime,
    ];

    /// Ion contribution in ppm per 1 g of salt in 1 US gallon: Ca, Mg, Na, SO₄, Cl, HCO₃ (spec §1.1).
    #[must_use]
    pub fn contribution(self) -> [f64; ION_COUNT] {
        match self {
            Salt::Gypsum => [61.478, 0.0, 0.0, 147.359, 0.0, 0.0],
            Salt::Nacl => [0.0, 0.0, 103.891, 0.0, 160.212, 0.0],
            Salt::Epsom => [0.0, 26.043, 0.0, 102.933, 0.0, 0.0],
            Salt::Cacl2 => [95.371, 0.0, 0.0, 0.0, 168.731, 0.0],
            Salt::Soda => [0.0, 0.0, 72.275, 0.0, 0.0, 191.827],
            Salt::Chalk => [105.754, 0.0, 0.0, 0.0, 0.0, 322.016],
            Salt::Mgcl2 => [0.0, 31.573, 0.0, 0.0, 92.111, 0.0],
            Salt::Lime => [142.856, 0.0, 0.0, 0.0, 0.0, 434.991],
        }
    }

    /// Lower-case key used in the parity vectors.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Salt::Gypsum => "gypsum",
            Salt::Nacl => "nacl",
            Salt::Epsom => "epsom",
            Salt::Cacl2 => "cacl2",
            Salt::Soda => "soda",
            Salt::Chalk => "chalk",
            Salt::Mgcl2 => "mgcl2",
            Salt::Lime => "lime",
        }
    }

    /// Parses [`Salt::key`].
    #[must_use]
    pub fn from_key(key: &str) -> Option<Salt> {
        Salt::ALL.into_iter().find(|s| s.key() == key)
    }
}

/// Ion increase from salt additions (spec §2), ppm: Ca, Mg, Na, SO₄, Cl, HCO₃.
///
/// `grams` is a list of (salt, grams); `vol_l` the water volume in litres. Returns zeros when
/// the volume is not positive.
#[must_use]
pub fn salt_ppm(grams: &[(Salt, f64)], vol_l: f64) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    if vol_l <= 0.0 {
        return out;
    }
    let gal = vol_l / GAL_L;
    for &(salt, g) in grams {
        let a = salt.contribution();
        for (o, k) in out.iter_mut().zip(a) {
            *o += g / gal * k;
        }
    }
    out
}

/// Alkalinity in ppm `CaCO₃` from bicarbonate in ppm: `HCO₃ · 50 / 61` (spec §4.1).
#[must_use]
pub fn alkalinity(hco3: f64) -> f64 {
    hco3 * 50.0 / 61.0
}

/// Effective (Kolbach) hardness in ppm `CaCO₃`: `Ca/1.4 + Mg/1.7` (spec §4.2).
#[must_use]
pub fn hardness(ca: f64, mg: f64) -> f64 {
    ca / 1.4 + mg / 1.7
}

/// Residual alkalinity in ppm `CaCO₃`: alkalinity minus effective hardness (spec §4.3).
#[must_use]
pub fn residual_alkalinity(ca: f64, mg: f64, hco3: f64) -> f64 {
    alkalinity(hco3) - hardness(ca, mg)
}

/// Sulphate to chloride ratio (spec §4.4); returns 0 when chloride is 0.
#[must_use]
pub fn sulfate_chloride_ratio(so4: f64, cl: f64) -> f64 {
    if cl == 0.0 { 0.0 } else { so4 / cl }
}

/// Suggested beer colour range `(min, max)` in SRM from RA (spec §4.5), floored at 0.
#[must_use]
pub fn color_range(ra: f64) -> (f64, f64) {
    (
        f64::max(0.0, 5.2 + 0.082 * ra),
        f64::max(0.0, (ra + 122.4) / 12.2),
    )
}

/// Boil concentration factor `S_boil` (spec §3.4). All volumes in litres: boil volume `vb`,
/// post-boil volume `vp`, cooling loss, trub loss and fermenter top-up water.
/// Returns 1 unless `vb > 0` and `vp > 0`.
#[must_use]
pub fn boil_scale(vb: f64, vp: f64, cool_loss: f64, trub_loss: f64, top_up: f64) -> f64 {
    if vb > 0.0 && vp > 0.0 {
        let a = vp - cool_loss;
        let b = a - trub_loss;
        ((vb / a) * b + top_up) / (b + top_up)
    } else {
        1.0
    }
}

/// Grist acidity function `g(c)` for malt colour `c` in SRM (spec §5.1).
///
/// BeerSmith form is discontinuous at 14 SRM; with `ours` the smaller of the two lines is used
/// below 200 SRM (continuous, spec §5.1 "Bizim model"). 42 from 200 SRM on in both.
#[must_use]
pub fn grist_g(c: f64, ours: bool) -> f64 {
    if c >= 200.0 {
        42.0
    } else if ours {
        f64::min(c - 1.9, 0.453 * c + 5.0)
    } else if c < 14.0 {
        c - 1.9
    } else {
        0.453 * c + 5.0
    }
}

/// Grist pH (spec §5.1). `bill` is a list of (weight kg, colour SRM). Returns
/// `(pH_grist, S)`; an empty or zero-weight bill gives `(7.0, 0.0)`.
#[must_use]
pub fn grist_ph(bill: &[(f64, f64)], ours: bool) -> (f64, f64) {
    let total: f64 = bill.iter().map(|&(w, _)| w).sum();
    if total <= 0.0 {
        return (7.0, 0.0);
    }
    let s: f64 = bill
        .iter()
        .map(|&(w, c)| w / total * grist_g(c, ours))
        .sum();
    (5.72 - 0.0337 * s, s)
}

/// RA to pH slope in pH per mEq/L (spec §5.2). `model` 0 = MPH 3, 1 = BW; `t` is the mash
/// thickness in L/kg.
#[must_use]
pub fn mash_ra_slope(model: u8, t: f64) -> f64 {
    if model == 1 {
        t * 0.479_306_13 * 0.17 / 1.5
    } else {
        0.037 + 0.014 * t
    }
}

/// Estimated mash pH without acid: `pH_grist + RA/50 · slope` (spec §5.3). `ra` in ppm `CaCO₃`.
#[must_use]
pub fn est_mash_ph(grist_ph: f64, ra: f64, slope: f64) -> f64 {
    grist_ph + ra / 50.0 * slope
}

/// Acid pK values `[pK1, pK2, pK3]` (spec §6.1); 20 means "no such proton". Unknown codes
/// (0, 1, 3, ...) are lactic.
#[must_use]
#[allow(clippy::approx_constant)] // 3.14 is citric acid pK1, not pi
pub fn acid_pks(acid: u8) -> [f64; 3] {
    match acid {
        2 => [2.12, 7.2, 12.44],
        4 => [3.14, 4.77, 6.39],
        5 => [4.75, 20.0, 20.0],
        6 => [-7.0, 20.0, 20.0],
        7 => [-1.0, 1.92, 20.0],
        8 => [2.98, 4.34, 20.0],
        _ => [3.86, 20.0, 20.0],
    }
}

/// Mean protons released per acid molecule at `ph` (mEq/mmol, spec §6.1).
#[must_use]
pub fn acid_fraction(acid: u8, ph: f64) -> f64 {
    let [k1, k2, k3] = acid_pks(acid);
    let r1 = 10f64.powf(ph - k1);
    let r2 = 10f64.powf(ph - k2);
    let r3 = 10f64.powf(ph - k3);
    let f0 = 1.0 / (1.0 + r1 + r1 * r2 + r1 * r2 * r3);
    r1 * f0 + 2.0 * r1 * r2 * f0 + 3.0 * r1 * r2 * r3 * f0
}

/// Dissociation factor `f` used for mash acids (spec §5.4, §6.2, §6.8).
///
/// BeerSmith: only model 1, at fixed pH 5.4, else 1. Own model: at the target pH `ph_t` for
/// both models.
#[must_use]
pub fn dissociation_fraction(acid: u8, model: u8, ph_t: f64, ours: bool) -> f64 {
    if ours {
        acid_fraction(acid, ph_t)
    } else if model == 1 {
        acid_fraction(acid, 5.4)
    } else {
        1.0
    }
}

/// Acid needed per litre of mash water for `meq` mEq/L (spec §6.2): mL of solution for liquid
/// acids, grams of malt for acid malt (code 3). `conc` is percent strength. Code 0 gives 0.
/// As in the reference, every code other than 1 and 3 uses the phosphoric constants.
#[must_use]
pub fn acid_per_liter(meq: f64, acid: u8, conc: f64, model: u8, ph_t: f64, ours: bool) -> f64 {
    if acid == 0 {
        return 0.0;
    }
    let (mw, rho) = if acid == 1 || acid == 3 {
        (90.09, 1.0 + conc * 0.237 / 100.0)
    } else {
        let p = conc / 100.0;
        (98.0, 1.0 + 0.49 * conc / 100.0 + 0.375 * p.powi(2))
    };
    let f = dissociation_fraction(acid, model, ph_t, ours);
    if acid == 3 {
        meq * 100.0 * mw / (conc * f * 1000.0)
    } else {
        meq * 100.0 * mw / (conc * f * rho * 1000.0)
    }
}

/// Total mash acid amount (spec §6.3): mL for liquid acids, grams for acid malt, for the
/// whole mash of `w_kg` kg grain at `t` L/kg. Zero when no acid is chosen or the estimate is
/// already at or below target.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn mash_acid_amount(
    est_ph: f64,
    target_ph: f64,
    model: u8,
    t: f64,
    w_kg: f64,
    acid: u8,
    conc: f64,
    ours: bool,
) -> f64 {
    let dph = est_ph - target_ph;
    if acid == 0 || dph < 0.0 {
        return 0.0;
    }
    let slope = mash_ra_slope(model, t);
    acid_per_liter(dph / slope, acid, conc, model, target_ph, ours) * w_kg * t
}

/// RA equivalent in mEq/L (negative) of liquid lactic acid in the mash (spec §5.4a).
/// `conc` percent, `vol_l` acid volume in litres, `f` dissociation factor, `w_kg` grain, `t` L/kg.
#[must_use]
pub fn lactic_acid_ra(conc: f64, vol_l: f64, f: f64, w_kg: f64, t: f64) -> f64 {
    let p = conc / 100.0;
    let rho = 1.0 + 0.237 * p;
    -(p * 1000.0 * rho * vol_l * f) / (0.09009 * w_kg * t)
}

/// Adjusted mash pH with `acid_malt_g` grams of acid malt (3 SRM, `pct` percent acid) added to
/// the bill (spec §5.5). Returns `(pH_adj, total grain kg)`. `ra` in ppm `CaCO₃`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn adjusted_ph(
    bill: &[(f64, f64)],
    acid_malt_g: f64,
    pct: f64,
    model: u8,
    ra: f64,
    t: f64,
    target_ph: f64,
    ours: bool,
) -> (f64, f64) {
    let mut b = bill.to_vec();
    if acid_malt_g > 0.0 {
        b.push((acid_malt_g / 1000.0, 3.0));
    }
    let w: f64 = b.iter().map(|&(wj, _)| wj).sum();
    let (gp, _) = grist_ph(&b, ours);
    let slope = mash_ra_slope(model, t);
    let f = dissociation_fraction(3, model, target_ph, ours);
    let acid_ra = -((pct / 100.0) * 1000.0 * (acid_malt_g / 1000.0) * f) / (w * t * 0.09009);
    (gp + slope * (ra / 50.0 + acid_ra), w)
}

/// Iterative acid malt amount (spec §6.4), at most 8 rounds. Returns `(total grams, final
/// adjusted pH)`. `pct` is the malt's acid percent; the added malt is 3 SRM.
#[must_use]
pub fn acid_malt_amount(
    bill: &[(f64, f64)],
    pct: f64,
    model: u8,
    ra: f64,
    t: f64,
    target_ph: f64,
    ours: bool,
) -> (f64, f64) {
    let mut total = 0.0;
    let mut ph2 = 0.0;
    for _ in 0..8 {
        let (ph, w) = adjusted_ph(bill, total, pct, model, ra, t, target_ph, ours);
        let slope = mash_ra_slope(model, t);
        let amt = if ph - target_ph >= 0.0 {
            acid_per_liter((ph - target_ph) / slope, 3, pct, model, target_ph, ours) * w * t
        } else {
            0.0
        };
        total += amt;
        ph2 = adjusted_ph(bill, total, pct, model, ra, t, target_ph, ours).0;
        if ph2 - target_ph < 0.005 {
            break;
        }
    }
    (total, ph2)
}

/// Sparge water volume in litres: total water minus the main mash water `w_kg · t` (spec §6.5).
#[must_use]
pub fn sparge_volume_l(total_l: f64, w_kg: f64, t: f64) -> f64 {
    total_l - w_kg * t
}

/// Sparge acid, model 0 (closed carbonate system charge balance, spec §6.5), mol/L.
/// `hco3` in ppm, water pH `ph_w`, target pH `ph_t`.
#[must_use]
pub fn sparge_acid_mph(hco3: f64, ph_w: f64, ph_t: f64) -> f64 {
    let c = hco3 / 61_016.0;
    let k1 = 4.4435e-07;
    let k1k2 = 2.073_781_45e-17;
    let k2 = 4.667e-11;
    let xw = 10f64.powf(ph_w);
    let xt = 10f64.powf(ph_t);
    let co3 = xw * c * k2;
    let ct = (k1k2 * xw * xw + k1 * xw + 1.0) * c / (k1 * xw);
    (10f64.powf(-ph_t) + (10f64.powf(ph_w - 14.0) + 2.0 * co3 + c) - 10f64.powf(-ph_w))
        - 10f64.powf(ph_t - 14.0)
        - (2.0 * k1k2 * xt * xt + k1 * xt) * ct / (k1k2 * xt * xt + k1 * xt + 1.0)
}

/// Sparge acid, own model 0: [`sparge_acid_mph`] divided by the acid's dissociation fraction at
/// the sparge target pH (spec §6.8), mol/L.
#[must_use]
pub fn sparge_acid_mph_ours(hco3: f64, ph_w: f64, ph_t: f64, acid: u8) -> f64 {
    sparge_acid_mph(hco3, ph_w, ph_t) / acid_fraction(acid, ph_t)
}

/// Sparge acid, model 1 (Bru'n Water style alkalinity titration, spec §6.5), mol/L.
/// Not defined for `ph_w <= 4.3` (water already acidic): returns 0.
#[must_use]
pub fn sparge_acid_bw(hco3: f64, ph_w: f64, ph_t: f64, acid: u8) -> f64 {
    if ph_w <= 4.3 {
        return 0.0;
    }
    let alpha = |ph: f64| {
        let r1 = 10f64.powf(ph - 6.38);
        let r2 = 10f64.powf(ph - 10.373);
        let d = 1.0 + r1 + r1 * r2;
        (1.0 / d, r1 * r2 / d)
    };
    let (a0w, a2w) = alpha(ph_w);
    let (a0e, a2e) = alpha(4.3);
    let (a0t, a2t) = alpha(ph_t);
    let ct = (hco3 * 50.0 / 61.0 / 50.0) / ((a0e - a0w) + (a2w - a2e));
    ct * ((a2w - a2t) + (a0t - a0w)) / acid_fraction(acid, ph_t) / 1000.0
}

/// Volume in mL of lactic acid solution for `mol_l` mol/L over `v_l` litres (spec §6.5).
/// `conc` is percent strength, floored at 1.
#[must_use]
pub fn lactic_ml(mol_l: f64, v_l: f64, conc: f64) -> f64 {
    let c = f64::max(conc, 1.0) / 100.0;
    (1.0 / c) * 0.09009 / (1.0 + 0.237 * c) * mol_l * v_l * 1000.0
}

/// Calculation warnings (spec §6.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warning {
    /// BW model is not yet validated against measured mash pH.
    BwModelUnverified,
    /// Chalk dissolves poorly; effect is overstated without acidic mash.
    ChalkSolubility,
}

impl Warning {
    /// User-facing text (Turkish, as in the reference).
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Warning::BwModelUnverified => {
                "BW modeli doğrulanacak: MPH'ye göre yaklaşık yarı asit önerir (ölçülmüş mash pH ile kalibre edilene kadar)"
            }
            Warning::ChalkSolubility => {
                "çözünürlük düşük, asitli mash olmadan etkisi abartılı olabilir (tebeşir)"
            }
        }
    }
}

/// Warnings for the chosen model and salt additions in grams (spec §6.8).
#[must_use]
pub fn warnings(model: u8, salt_grams: &[(Salt, f64)]) -> Vec<Warning> {
    let mut out = Vec::new();
    if model == 1 {
        out.push(Warning::BwModelUnverified);
    }
    let chalk: f64 = salt_grams
        .iter()
        .filter(|(s, _)| *s == Salt::Chalk)
        .map(|&(_, g)| g)
        .sum();
    if chalk > 0.0 {
        out.push(Warning::ChalkSolubility);
    }
    out
}

/// Least squares for the given columns via Householder QR; `cols` are columns of equal length
/// `b.len()` and at most that many.
fn lstsq(cols: &[&[f64]], b: &[f64]) -> Vec<f64> {
    let m = b.len();
    let k = cols.len();
    let mut a: Vec<Vec<f64>> = cols.iter().map(|c| c.to_vec()).collect();
    let mut qtb = b.to_vec();
    for j in 0..k {
        let norm = (j..m).map(|i| a[j][i] * a[j][i]).sum::<f64>().sqrt();
        if norm == 0.0 {
            continue;
        }
        let alpha = if a[j][j] > 0.0 { -norm } else { norm };
        let mut v: Vec<f64> = a[j][j..].to_vec();
        v[0] -= alpha;
        let vn2: f64 = v.iter().map(|x| x * x).sum();
        if vn2 == 0.0 {
            continue;
        }
        for c in j..k {
            let dot: f64 = v.iter().enumerate().map(|(i, vi)| vi * a[c][j + i]).sum();
            let f = 2.0 * dot / vn2;
            for (i, vi) in v.iter().enumerate() {
                a[c][j + i] -= f * vi;
            }
        }
        let dot: f64 = v.iter().enumerate().map(|(i, vi)| vi * qtb[j + i]).sum();
        let f = 2.0 * dot / vn2;
        for (i, vi) in v.iter().enumerate() {
            qtb[j + i] -= f * vi;
        }
    }
    let mut x = vec![0.0; k];
    for i in (0..k).rev() {
        let mut s = qtb[i];
        for c in (i + 1)..k {
            s -= a[c][i] * x[c];
        }
        let d = a[i][i];
        x[i] = if d.abs() > 1e-300 { s / d } else { 0.0 };
    }
    x
}

/// Non-negative least squares, Lawson–Hanson active set method (spec §7.1): minimise
/// `‖A x − b‖₂` subject to `x ≥ 0`. `a` holds the columns of `A` (each of length `b.len()`).
/// At most `3n` iterations. Returns `(x, ‖A x − b‖₂)`.
///
/// The solution `x` is not unique when columns are linearly dependent; `A x` and the residual
/// norm always are.
#[must_use]
pub fn nnls(a: &[Vec<f64>], b: &[f64]) -> (Vec<f64>, f64) {
    let n = a.len();
    let m = b.len();
    let mut x = vec![0.0; n];
    let mut passive = vec![false; n];
    let anorm = a
        .iter()
        .map(|c| c.iter().map(|v| v.abs()).sum::<f64>())
        .fold(0.0, f64::max);
    let tol = 10.0 * (m.max(n) as f64) * f64::EPSILON * anorm;
    let max_iter = 3 * n;
    let residual = |x: &[f64]| -> Vec<f64> {
        (0..m)
            .map(|i| b[i] - (0..n).map(|j| a[j][i] * x[j]).sum::<f64>())
            .collect()
    };
    let mut it = 0;
    loop {
        let r = residual(&x);
        let w: Vec<f64> = (0..n)
            .map(|j| (0..m).map(|i| a[j][i] * r[i]).sum())
            .collect();
        let mut best: Option<usize> = None;
        for j in 0..n {
            if !passive[j] && w[j] > tol && best.is_none_or(|bj| w[j] > w[bj]) {
                best = Some(j);
            }
        }
        let Some(best) = best else { break };
        passive[best] = true;
        loop {
            let idx: Vec<usize> = (0..n).filter(|&j| passive[j]).collect();
            let cols: Vec<&[f64]> = idx.iter().map(|&j| a[j].as_slice()).collect();
            let s = lstsq(&cols, b);
            if s.iter().all(|&v| v > 0.0) {
                x.fill(0.0);
                for (&j, &v) in idx.iter().zip(&s) {
                    x[j] = v;
                }
                break;
            }
            let alpha = idx
                .iter()
                .zip(&s)
                .filter(|&(_, &sv)| sv <= 0.0)
                .map(|(&j, &sv)| x[j] / (x[j] - sv))
                .fold(f64::INFINITY, f64::min);
            for (&j, &sv) in idx.iter().zip(&s) {
                x[j] += alpha * (sv - x[j]);
            }
            for &j in &idx {
                if x[j] <= tol {
                    x[j] = 0.0;
                    passive[j] = false;
                }
            }
            it += 1;
            if it > max_iter {
                break;
            }
        }
        it += 1;
        if it > max_iter {
            break;
        }
    }
    let rn = residual(&x).iter().map(|v| v * v).sum::<f64>().sqrt();
    (x, rn)
}

/// Best salt additions to reach a target profile (spec §7.1). `base` and `target` are ion
/// profiles in ppm (Ca, Mg, Na, SO₄, Cl, HCO₃), `s` the base scaling (1 for mash recipes,
/// `S_boil` for extract recipes), `excluded` salts are fixed at 0.
///
/// Returns grams per US gallon for every salt in [`Salt::ALL`] order, and the residual norm.
#[must_use]
pub fn water_match(
    base: &[f64; ION_COUNT],
    target: &[f64; ION_COUNT],
    s: f64,
    excluded: &[Salt],
) -> ([f64; 8], f64) {
    let active: Vec<Salt> = Salt::ALL
        .into_iter()
        .filter(|salt| !excluded.contains(salt))
        .collect();
    let cols: Vec<Vec<f64>> = active
        .iter()
        .map(|salt| salt.contribution().to_vec())
        .collect();
    let b: Vec<f64> = (0..ION_COUNT).map(|i| target[i] - s * base[i]).collect();
    let (x, rn) = nnls(&cols, &b);
    let mut out = [0.0; 8];
    for (salt, v) in active.iter().zip(x) {
        let pos = Salt::ALL.iter().position(|q| q == salt).unwrap_or(0);
        out[pos] = v;
    }
    (out, rn)
}

/// Resulting profile of a [`water_match`] solution (spec §7.3): `s·base + Σ x_k · A_k` in ppm.
#[must_use]
pub fn best_fit_profile(base: &[f64; ION_COUNT], s: f64, x: &[f64; 8]) -> [f64; ION_COUNT] {
    let mut out = [0.0; ION_COUNT];
    for (i, o) in out.iter_mut().enumerate() {
        *o = s * base[i]
            + Salt::ALL
                .iter()
                .zip(x)
                .map(|(salt, xk)| xk * salt.contribution()[i])
                .sum::<f64>();
    }
    out
}
