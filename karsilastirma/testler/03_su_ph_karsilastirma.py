#!/usr/bin/env python3
"""
03 - Su kimyasi / mash pH cross-check: BeerSmith 4 spesifikasyonu (formuller/03-su-ph.md)
ile Brewtarget (src/widgets/WaterAdjuster.cpp, src/model/Misc.cpp,
src/measurement/PhysicalConstants.h, RecipeAdditionMiscTableModel.cpp,
RecipeAdditionFermentableTableModel.cpp) formullerinin birebir Python portu.

Brewtarget portu HATALARIYLA BIRLIKTE ("as-is") yazildi; ayrica hatalari duzeltilmis
"niyet edilen" varyantlar da hesaplanir. Calistir:  python3 03_su_ph_karsilastirma.py
"""
import math

GAL_L = 128 * 29.57352956 / 1000.0          # 3.785411784
IONS = ['Ca', 'Mg', 'Na', 'SO4', 'Cl', 'HCO3']

# ------------------------------------------------------------------ BeerSmith (spec)
BS_A = {  # ppm for 1 g in 1 US gal  (03-su-ph.md §1.1)
    'gypsum': [61.478, 0, 0, 147.359, 0, 0],
    'nacl':   [0, 0, 103.891, 0, 160.212, 0],
    'epsom':  [0, 26.043, 0, 102.933, 0, 0],
    'cacl2':  [95.371, 0, 0, 0, 168.731, 0],
    'soda':   [0, 0, 72.275, 0, 0, 191.827],
    'chalk':  [105.754, 0, 0, 0, 0, 322.016],
    'mgcl2':  [0, 31.573, 0, 0, 92.111, 0],
    'lime':   [142.856, 0, 0, 0, 0, 434.991],
}

def bs_salt_ppm(grams, vol_L):
    gal = vol_L / GAL_L
    out = [0.0] * 6
    for s, g in grams.items():
        for i in range(6):
            out[i] += g / gal * BS_A[s][i]
    return out

def bs_RA(ca, mg, hco3):                       # ppm CaCO3  (§4.1-4.3)
    return hco3 * 50 / 61 - (ca / 1.4 + mg / 1.7)

def bs_grist_ph(bill):                         # bill: [(kg, SRM)]  (§5.1)
    W = sum(w for w, _ in bill)
    g = lambda c: c - 1.9 if c < 14 else (0.453 * c + 5.0 if c < 200 else 42.0)
    return 5.72 - 0.0337 * sum(w / W * g(c) for w, c in bill)

def bs_slope(model, t):                        # §5.2
    return t * 0.47930613 * 0.17 / 1.5 if model == 1 else 0.037 + 0.014 * t

def acidfrac(pks, ph):
    r = [10 ** (ph - k) for k in pks] + [0, 0]
    D = 1 + r[0] + r[0] * r[1] + r[0] * r[1] * r[2]
    return (r[0] + 2 * r[0] * r[1] + 3 * r[0] * r[1] * r[2]) / D
LACTIC = (3.86,); PHOS = (2.12, 7.20, 12.44)

def bs_lactic_ra(mL, pct, W, t, model):        # §5.4a  mEq/L (negatif)
    p = pct / 100; rho = 1 + 0.237 * p
    f = acidfrac(LACTIC, 5.4) if model == 1 else 1
    return -p * 1000 * rho * (mL / 1000) * f / (0.09009 * W * t)

def bs_acidmalt_ra(g, pct, W, t, model):       # §5.4b
    f = acidfrac(LACTIC, 5.4) if model == 1 else 1
    return -(pct / 100) * 1000 * (g / 1000) * f / (W * t * 0.09009)

def bs_mash_ph(bill, prof, t, model, lactic_mL=0, lactic_pct=88, acidmalt_g=0, am_pct=3):
    b = bill + ([(acidmalt_g / 1000, 3.0)] if acidmalt_g else [])
    W = sum(w for w, _ in b)
    ra = bs_RA(prof[0], prof[1], prof[5])
    m = bs_slope(model, t)
    acid = bs_lactic_ra(lactic_mL, lactic_pct, W, t, model) + bs_acidmalt_ra(acidmalt_g, am_pct, W, t, model)
    return bs_grist_ph(b) + m * (ra / 50 + acid)

def bs_lactic_needed(bill, prof, t, model, target=5.4, pct=88):   # §6.2-6.3
    W = sum(w for w, _ in bill)
    est = bs_mash_ph(bill, prof, t, model)
    d = est - target
    if d < 0:
        return 0.0
    meq = d / bs_slope(model, t)
    f = acidfrac(LACTIC, 5.4) if model == 1 else 1
    rho = 1 + 0.237 * pct / 100
    return meq * 100 * 90.09 / (pct * f * rho * 1000) * W * t

# ------------------------------------------------------------------ Brewtarget (port)
# PhysicalConstants.h:66-102
Ca, C, Cl, H, Mg, O, P, Na, S = 40.0784, 12.0112, 35.45, 1.00802, 24.3052, 15.9991, 30.9737619985, 22.989769282, 32.062
HCO3 = H + C + 3 * O; CO3 = C + 3 * O; SO4 = S + 4 * O; PO4 = P + 4 * O
OH_BT = H + 2 * O            # PhysicalConstants.h:84 -> HATA: OH = H + O olmali
H2O = 2 * H + O
CaCl2 = Ca + 2 * Cl; CaCl2_2H2O = CaCl2 + 2 * H2O; CaCO3 = Ca + CO3; CaSO4 = Ca + SO4
MgSO4 = Mg + SO4; MgCl2 = Mg + 2 * Cl; NaCl = Na + Cl; NaHCO3 = Na + HCO3
CaOH2_BT = Ca + OH_BT        # PhysicalConstants.h:99 -> HATA: Ca + 2*(H+O) olmali
CaSO4_2H2O = CaSO4 + 2 * H2O; MgSO4_7H2O = MgSO4 + 7 * H2O; MgCl2_6H2O = MgCl2 + 6 * H2O

# Misc.cpp:409-563  ppm (mg/L) per 1 g in 1 L ; iyon sirasi Ca,Mg,Na,SO4,Cl,HCO3,CO3
def bt_factor(salt):
    z = dict(Ca=0, Mg=0, Na=0, SO4=0, Cl=0, HCO3=0, CO3=0)
    k = 1000.0
    if salt == 'cacl2':      z.update(Ca=Ca / CaCl2 * k, Cl=2 * Cl / CaCl2 * k)
    if salt == 'cacl2_2h2o': z.update(Ca=Ca / CaCl2_2H2O * k, Cl=2 * Cl / CaCl2_2H2O * k)
    if salt == 'chalk':      z.update(Ca=Ca / CaCO3 * k, CO3=CO3 / CaCO3 * k)
    if salt == 'gypsum':     z.update(Ca=Ca / CaSO4 * k,            # Misc.cpp:414 HATA: susuz CaSO4
                                      SO4=SO4 / CaSO4_2H2O * k)    # Misc.cpp:546 dihidrat
    if salt == 'lime':       z.update(Ca=Ca / CaOH2_BT * k)        # alkalinite katkisi YOK
    if salt == 'epsom':      z.update(Mg=Mg / MgSO4_7H2O * k, SO4=SO4 / MgSO4_7H2O * k)
    if salt == 'mgcl2':      z.update(Mg=Mg / MgCl2_6H2O * k)      # Cl=0 (Misc.cpp:444); Ca fonksiyonunda
                                                                   # 'break' -> Q_UNREACHABLE (Misc.cpp:427-431)
    if salt == 'nacl':       z.update(Na=Na / NaCl * k, Cl=Cl / NaCl * k)
    if salt == 'soda':       z.update(Na=Na / NaHCO3 * k, HCO3=HCO3 / NaHCO3 * k)
    return z

def bt_factor_fixed(salt):
    z = bt_factor(salt)
    if salt == 'gypsum': z['Ca'] = Ca / CaSO4_2H2O * 1000
    if salt == 'lime':   z['Ca'] = Ca / (Ca + 2 * (H + O)) * 1000
    if salt == 'mgcl2':  z['Cl'] = 2 * Cl / MgCl2_6H2O * 1000
    return z

NOCOLOR_PH, SLOPE_LIGHT, SLOPE_DARK, MEQ = 5.6, 0.21, 0.06, 50.0   # WaterAdjuster.cpp:152-158

def sg_to_plato(sg):                            # Algorithms.cpp:51-54
    return -616.868 + 1111.14 * sg - 630.272 * sg ** 2 + 135.997 * sg ** 3

def bt_grist_ph(bill, og, beer_srm, intended=False):
    """WaterAdjuster.cpp:290-333 + setRecipe 600-642. bill: [(kg, SRM, DP_lintner)]"""
    Wt = sum(w for w, _, _ in bill)
    wcol = sum(w / Wt * (c + 0.6) / 1.35 for w, c, _ in bill)
    cfg = 0.0
    for w, c, dp in bill:
        if dp < 1:
            lovi = 19.0 if c > 120 else (c + 0.6) / 1.35
            if intended: cfg += w / Wt * lovi
            else:        cfg = w / Wt * lovi       # HATA: '=' (satir 314) -> sadece son malt
    cr = cfg / wcol
    adj = (1 / sg_to_plato(og)) * (SLOPE_LIGHT * (1 - cr) + SLOPE_DARK * cr) * beer_srm
    return NOCOLOR_PH - adj, cr

def bt_mash_ph(bill, og, beer_srm, base, alk_ppm_hco3, infusion_L, sparge_L, salts_g,
               lactic_mL=0.0, lactic_pct=88.0, phos_mL=0.0, phos_pct=10.0,
               acidmalt=None, ro_mash_pct=0.0, ro_sparge_pct=0.0, fix=False, intended_grist=False):
    """WaterAdjuster.cpp:272-420 + newTotals 688-734. base=dict(Ca,Mg,...).
    alk_ppm_hco3 = Water::alkalinity_ppm (as HCO3) -- bicarbonate_ppm alani pH'ta KULLANILMAZ.
    acidmalt = (kg, lacticAcidByWeight_pct)  (fatura ici sayilir)."""
    b = list(bill)
    if acidmalt:
        b = b + [(acidmalt[0], 3.0, 50)]          # asit malt: DP>1 kabul (baz malt gibi)
    Wt = sum(w for w, _, _ in b)
    total_L = infusion_L + sparge_L                # Mash.cpp:170 totalMashWater_l (sparge dahil)
    t = infusion_L / Wt                            # :644
    ro_tot = ro_mash_pct / 100 * infusion_L + ro_sparge_pct / 100 * sparge_L
    base_frac = (total_L - ro_tot) / total_L       # :84
    grist, _ = bt_grist_ph(b, og, beer_srm, intended_grist)
    # calculateRA :272-284  (HATA: (1 - pct) pct 0..100 iken)
    ro_f = (1 - ro_mash_pct / 100) if fix else (1 - ro_mash_pct)
    ra = ro_f * alk_ppm_hco3 / 61
    ca = base_frac * base['Ca'] / Ca * 2
    mg = base_frac * base['Mg'] / Mg * 2
    base_ph = (ra - ca / 3.5 - mg / 7) * infusion_L / t / MEQ                 # :354-358
    if fix:   # Troester: dpH = RA[mEq/L] * t[L/kg] / BC ; BC = 50 mEq/(kg.pH) varsayimi
        base_ph = (ra - ca / 3.5 - mg / 7) * t / MEQ
    # calculateAddedWaterAdjustmentpH :364-378  (mg toplam / 1 L "ppm")
    fz = bt_factor_fixed if fix else bt_factor
    tot = {k: sum(g * fz(s)[k] for s, g in salts_g.items()) for k in ['Ca', 'Mg', 'HCO3', 'CO3']}
    sca = tot['Ca'] / Ca * 2; smg = tot['Mg'] / Mg * 2
    shco3 = tot['HCO3'] / HCO3; sco3 = tot['CO3'] / CO3
    if fix:   # mEq toplam -> mEq/L (infuzyon), CO3 2 esdeger, cift /61 yok
        dl = (-sca / 3.5 - smg / 7 + shco3 + 2 * sco3) / infusion_L
        salt_ph = dl * t / MEQ
    else:
        salt_ph = (0.0 - sca / 3.5 - smg / 7 + (shco3 + sco3) / 61.0) / t / MEQ
    # calculateAcidpH :383-401 + totalAcid_kg (RecipeAdditionMiscTableModel.cpp:225-255)
    q = lactic_mL / 1000.0                          # kanonik L
    dens = lactic_pct / 88.0 * (1.2 - 1.0) + 1.0
    lactic_kg = (lactic_pct / 100) * 1000.0 * q * (1 if fix else 1000.0) * dens   # HATA: ekstra *1000
    qp = phos_mL / 1000.0
    pd = phos_pct / 85.0 * (1.685 - 1.0) + 1.0
    phos_kg = (phos_pct / 100) * 1000.0 * qp * pd
    am_kg = 0.0
    if acidmalt:   # RecipeAdditionFermentableTableModel.cpp:149  HATA: pct * kg (/100 yok)
        am_kg = acidmalt[1] * acidmalt[0] / (100 if fix else 1)
    if fix:  # gram/MW = mmol = mEq ; mEq/L ; * t / BC
        lac_g = lactic_kg; ph_g = phos_kg; am_g = am_kg * 1000
        acid_meq_L = ((lac_g + am_g) / 90.08 + ph_g / 98.0) * 1000 / infusion_L
        acids = acid_meq_L * t / MEQ
    else:
        d = 0.0
        if lactic_kg + am_kg > 0: d += 1000.0 * (lactic_kg + am_kg) / 90.0
        if phos_kg > 0:           d += 1000.0 * phos_kg / 98.0
        acids = d / MEQ / t
    return base_ph + grist + salt_ph - acids, dict(grist=grist, base=base_ph, salt=salt_ph, acid=acids, t=t)

def bisect(f, lo, hi, target, n=200):
    flo = f(lo) - target
    if flo <= 0: return 0.0
    for _ in range(n):
        mid = (lo + hi) / 2
        if f(mid) - target > 0: lo = mid
        else: hi = mid
    return (lo + hi) / 2

def pct(a, b):
    return float('nan') if b == 0 else (a - b) / b * 100

# ------------------------------------------------------------------ Senaryo
# T3 tahil faturasi (spec §10 T3) + Brewtarget icin DP (crystal/roast DP=0)
BILL_BS = [(4.5, 2.0), (0.3, 60.0), (0.1, 350.0)]
BILL_BT = [(4.5, 2.0, 140), (0.3, 60.0, 0), (0.1, 350.0, 0)]
W = 4.9; T = 3.0; MASH_L = W * T          # 14.7 L
BATCH_L = 20.0
# Brewtarget grist modeli recetenin OG ve bira rengine bakar -> varsayimlar:
lb = lambda kg: kg / 0.45359237
gal = BATCH_L / GAL_L
MCU = sum(lb(w) * c for w, c in BILL_BS) / gal              # renk SRM~Lovibond kabul
SRM = 1.4922 * MCU ** 0.6859                                 # Morey (BT varsayilan)
PPG = [37, 34, 28]; EFF = 0.75
OG = 1 + sum(lb(w) * p for (w, _), p in zip(BILL_BS, PPG)) * EFF / gal / 1000
BASE = dict(Ca=50, Mg=10, Na=15, SO4=40, Cl=30, HCO3=150)
SALTS = {'gypsum': 5.0, 'cacl2': 3.0}

def line(*a): print(' | '.join(str(x) for x in a))

if __name__ == '__main__':
    print('# Varsayimlar: batch %.0f L, OG %.4f (%.2f P), Morey renk %.2f SRM, mash %.1f L, t=%.1f L/kg'
          % (BATCH_L, OG, sg_to_plato(OG), SRM, MASH_L, T))

    print('\n## A. Tuz iyon katsayilari (ppm, 1 g / 1 US gal)')
    line('tuz', 'iyon', 'BeerSmith', 'Brewtarget(as-is)', 'fark %', 'dogru(stokiyometri)')
    trueM = {'gypsum': CaSO4_2H2O, 'cacl2': CaCl2, 'epsom': MgSO4_7H2O, 'nacl': NaCl, 'soda': NaHCO3,
             'chalk': CaCO3, 'mgcl2': MgCl2_6H2O, 'lime': Ca + 2 * (H + O)}
    for s in BS_A:
        bt = bt_factor(s)
        for i, ion in enumerate(IONS):
            bsv = BS_A[s][i]; btv = bt[ion] / GAL_L
            if ion == 'HCO3' and s == 'chalk': btv = bt['CO3'] / GAL_L; ion_l = 'HCO3 (BT: CO3)'
            else: ion_l = ion
            if bsv == 0 and btv == 0: continue
            line(s, ion_l, '%.3f' % bsv, '%.3f' % btv, '%+.1f' % pct(btv, bsv) if bsv else 'BS=0', '')
    print('CaCl2.2H2O (yalniz BT): Ca %.3f, Cl %.3f' % (bt_factor('cacl2_2h2o')['Ca'] / GAL_L, bt_factor('cacl2_2h2o')['Cl'] / GAL_L))

    print('\n## B. T1: 20 L suya 5 g gypsum + 3 g CaCl2 (ppm)')
    bs1 = bs_salt_ppm(SALTS, 20)
    bt1 = {k: sum(g * bt_factor(s)[k] for s, g in SALTS.items()) / 20 for k in IONS}
    for i, ion in enumerate(IONS):
        if bs1[i] or bt1[ion]:
            line(ion, '%.2f' % bs1[i], '%.2f' % bt1[ion], '%+.2f%%' % pct(bt1[ion], bs1[i]))

    print('\n## C. T2: RA (Ca50 Mg10 HCO3 150)')
    bsra = bs_RA(50, 10, 150)
    btra_meq = 150 / 61 - (50 / Ca * 2) / 3.5 - (10 / Mg * 2) / 7
    line('BS RA ppm CaCO3 %.3f' % bsra, 'BT RA mEq/L %.4f -> ppm CaCO3 %.3f' % (btra_meq, btra_meq * 50), 'fark %+.3f%%' % pct(btra_meq * 50, bsra))

    print('\n## D. Grist (saf su) pH')
    g_bs = bs_grist_ph(BILL_BS)
    g_bt, cr = bt_grist_ph(BILL_BT, OG, SRM)
    g_bti, cri = bt_grist_ph(BILL_BT, OG, SRM, intended=True)
    line('BS', '%.4f' % g_bs)
    line('BT as-is (colorRatio %.3f)' % cr, '%.4f' % g_bt, 'fark %.3f pH' % (g_bt - g_bs))
    line('BT += duzeltilmis (colorRatio %.3f)' % cri, '%.4f' % g_bti, 'fark %.3f pH' % (g_bti - g_bs))

    print('\n## E. Mash pH: base su + 5 g gypsum + 3 g CaCl2 mash suyunda (%.1f L), sparge yok' % MASH_L)
    prof = [BASE[k] + v for k, v in zip(IONS, bs_salt_ppm(SALTS, MASH_L))]
    print('   mash profili (BS):', [round(x, 1) for x in prof], 'RA=%.2f' % bs_RA(prof[0], prof[1], prof[5]))
    res = {}
    for m in (0, 1):
        res['BS m%d' % m] = bs_mash_ph(BILL_BS, prof, T, m)
    res['BT as-is (alk=150)'] = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS)[0]
    res['BT as-is (alk bos, varsayilan icerik)'] = bt_mash_ph(BILL_BT, OG, SRM, BASE, 0, MASH_L, 0, SALTS)[0]
    res['BT duzeltilmis'] = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, fix=True, intended_grist=True)[0]
    for k, v in res.items():
        line(k, '%.4f' % v, 'BS-BW farki %+.3f' % (v - res['BS m1']))
    _, comp = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS)
    print('   BT bilesenler as-is:', {k: round(v, 4) for k, v in comp.items()})
    _, compf = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, fix=True, intended_grist=True)
    print('   BT bilesenler duz.:', {k: round(v, 4) for k, v in compf.items()})

    print('\n## F. 5.40 hedefi icin gereken %88 laktik asit (mL)')
    for m in (0, 1):
        need = bs_lactic_needed(BILL_BS, prof, T, m)
        chk = bs_mash_ph(BILL_BS, prof, T, m, lactic_mL=need)
        line('BS model %d' % m, '%.3f mL' % need, 'kontrol pH %.4f' % chk)
    f_asis = lambda x: bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, lactic_mL=x)[0]
    f_fix = lambda x: bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, lactic_mL=x, fix=True, intended_grist=True)[0]
    line('BT as-is (ileri modeli ters cevirerek; BT\'de cozucu yok)', '%.5f mL' % bisect(f_asis, 0, 50, 5.4))
    line('BT duzeltilmis', '%.3f mL' % bisect(f_fix, 0, 50, 5.4))
    line('BT as-is: 1 mL %88 laktik -> dpH', '%.2f' % (f_asis(0) - f_asis(1)))
    line('BS BW: 1 mL %88 laktik -> dpH', '%.4f' % (bs_mash_ph(BILL_BS, prof, T, 1) - bs_mash_ph(BILL_BS, prof, T, 1, lactic_mL=1)))

    print('\n## G. 100 g asit malt (%3 laktik) etkisi (dpH)')
    for m in (0, 1):
        d = bs_mash_ph(BILL_BS, prof, T, m) - bs_mash_ph(BILL_BS, prof, T, m, acidmalt_g=100, am_pct=3)
        line('BS model %d' % m, '%.4f' % d)
    a0 = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS)[0]
    a1 = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, acidmalt=(0.1, 3.0))[0]
    line('BT as-is', '%.4f' % (a0 - a1))
    a0 = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, fix=True, intended_grist=True)[0]
    a1 = bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, acidmalt=(0.1, 3.0), fix=True, intended_grist=True)[0]
    line('BT duzeltilmis', '%.4f' % (a0 - a1))

    print('\n## H. 3 g kabartma tozu / 3 g tebesir mash suyuna (dpH, saf su bazli)')
    RO = dict(Ca=0, Mg=0, Na=0, SO4=0, Cl=0, HCO3=0)
    for s in ('soda', 'chalk'):
        p0 = [0] * 6; p1 = bs_salt_ppm({s: 3.0}, MASH_L)
        for m in (0, 1):
            line('BS m%d %s' % (m, s), '%+.4f' % (bs_mash_ph(BILL_BS, p1, T, m) - bs_mash_ph(BILL_BS, p0, T, m)))
        b0 = bt_mash_ph(BILL_BT, OG, SRM, RO, 0, MASH_L, 0, {})[0]
        b1 = bt_mash_ph(BILL_BT, OG, SRM, RO, 0, MASH_L, 0, {s: 3.0})[0]
        line('BT as-is %s' % s, '%+.4f' % (b1 - b0))

    print('\n## I. Olcek testi: her sey x2 (10 kg tahil, 29.4 L, 10 g gypsum + 6 g CaCl2)')
    bill2 = [(w * 2, c, d) for w, c, d in BILL_BT]
    s2 = {k: v * 2 for k, v in SALTS.items()}
    for m in (0, 1):
        line('BS m%d' % m, '%.4f -> %.4f' % (bs_mash_ph(BILL_BS, prof, T, m), bs_mash_ph([(w * 2, c) for w, c in BILL_BS], prof, T, m)))
    line('BT as-is', '%.4f -> %.4f' % (bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS)[0],
                                       bt_mash_ph(bill2, OG, SRM, BASE, 150, 2 * MASH_L, 0, s2)[0]))
    line('BT duzeltilmis', '%.4f -> %.4f' % (bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, fix=True, intended_grist=True)[0],
                                             bt_mash_ph(bill2, OG, SRM, BASE, 150, 2 * MASH_L, 0, s2, fix=True, intended_grist=True)[0]))

    print('\n## J. Sparge suyu ekleme (15.3 L sparge adimi, tuzlar ayni) -> BT mash pH degisir mi?')
    line('BT as-is sparge yok', '%.4f' % bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS)[0])
    line('BT as-is sparge 15.3 L', '%.4f' % bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 15.3, SALTS)[0])
    line('BT as-is mash RO %50', '%.4f' % bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, ro_mash_pct=50)[0])
    line('BT duz. mash RO %50', '%.4f' % bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, SALTS, ro_mash_pct=50, fix=True, intended_grist=True)[0])

    print('\n## K. Asit yogunluklari (g/mL)')
    for p in (10, 75, 85, 88):
        line('laktik %d%%' % p, 'BS %.4f' % (1 + 0.237 * p / 100), 'BT %.4f' % (p / 88 * 0.2 + 1))
        line('fosforik %d%%' % p, 'BS %.4f (0.365) / %.4f (0.375)' % (1 + 0.49 * p / 100 + 0.365 * (p / 100) ** 2, 1 + 0.49 * p / 100 + 0.375 * (p / 100) ** 2), 'BT %.4f' % (p / 85 * 0.685 + 1))

    print('\n## L. Dogrulama: BS portu spec T3 degerlerini uretiyor mu? (profil = base + T1 tuzlari @20 L)')
    pspec = [BASE[k] + v for k, v in zip(IONS, bs_salt_ppm(SALTS, 20))]
    for m in (0, 1):
        line('BS m%d' % m, 'pH_est %.4f' % bs_mash_ph(BILL_BS, pspec, T, m), 'laktik %.3f mL' % bs_lactic_needed(BILL_BS, pspec, T, m),
             '(spec: %s)' % ('5.6234 / 3.521' if m == 0 else '5.6253 / 1.771'))

    print('\n## M. Tuzsuz senaryo: yalniz base su (Ca50 Mg10 HCO3 150), 5.40 icin laktik')
    pb = [BASE[k] for k in IONS]
    for m in (0, 1):
        line('BS m%d' % m, 'pH %.4f' % bs_mash_ph(BILL_BS, pb, T, m), 'laktik %.3f mL' % bs_lactic_needed(BILL_BS, pb, T, m))
    fa = lambda x: bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, {}, lactic_mL=x)[0]
    ff = lambda x: bt_mash_ph(BILL_BT, OG, SRM, BASE, 150, MASH_L, 0, {}, lactic_mL=x, fix=True, intended_grist=True)[0]
    line('BT as-is', 'pH %.4f' % fa(0), 'laktik %.5f mL' % bisect(fa, 0, 50, 5.4))
    line('BT duzeltilmis', 'pH %.4f' % ff(0), 'laktik %.3f mL' % bisect(ff, 0, 50, 5.4))
    # RO/saf su + koyu fatura: BT grist modeli ile BS farki
    dark = [(4.0, 3.0, 120), (0.5, 120.0, 0), (0.4, 500.0, 0)]
    darkbs = [(w, c) for w, c, _ in dark]
    mcu = sum(lb(w) * c for w, c in darkbs) / gal; srm = 1.4922 * mcu ** 0.6859
    og = 1 + sum(lb(w) * p for (w, _), p in zip(darkbs, [37, 34, 25])) * EFF / gal / 1000
    print('\n## N. Koyu fatura (stout benzeri, %.1f SRM), saf su: grist pH' % srm)
    line('BS', '%.4f' % bs_grist_ph(darkbs))
    line('BT as-is', '%.4f' % bt_grist_ph(dark, og, srm)[0])
    line('BT += duz.', '%.4f' % bt_grist_ph(dark, og, srm, True)[0])
