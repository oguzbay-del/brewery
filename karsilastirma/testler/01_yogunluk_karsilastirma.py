#!/usr/bin/env python3
"""
01 - Yogunluk / verim / alkol / renk: BeerSmith 4 spesifikasyonu vs Brewtarget cross-check.

BeerSmith tarafi: /home/user/formuller/testler/01_yogunluk_tv.py ile ayni formuller (spesifikasyon 01-yogunluk-alkol-renk.md).
Brewtarget tarafi: src/Algorithms.cpp, src/model/Recipe.cpp, src/model/RecipeAdditionFermentable.cpp,
src/measurement/{ColorMethods,Unit,SucroseConversion}.cpp, src/tools/{RefractoDialog,OgCorrectionTool}.cpp,
src/model/BrewLog.cpp formullerinin Python'a tasinmis halleri.

Calistirma: python3 -I 01_yogunluk_karsilastirma.py  [brewtarget_kaynak_dizini]
"""
import math, re, sys, os

BT_SRC = sys.argv[1] if len(sys.argv) > 1 else "/home/user/src/brewtarget"

LB = 0.45359237          # kg
GAL = 3.785411784        # L
FLOZ = GAL / 128

# ======================================================================================
# BeerSmith (spesifikasyondan)
# ======================================================================================
def bs_ToPlato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868
def bs_ToSG(p): return p/(258.6 - (p/258.6)*227.1) + 1.0
def bs_BrixToSG(b): return b/(258.6 - (b/258.2)*227.1) + 1.0
def bs_SGToBrix(sg): return ((182.4601*sg - 775.6821)*sg + 1262.7794)*sg - 669.5622
def bs_ABW(og, fg):
    o = bs_ToPlato(og); f = bs_ToPlato(fg); o = max(o, f); re_ = 0.1808*o + 0.8192*f
    return (o - re_)/(2.0665 - 0.010665*o)
def bs_ABV(og, fg): return fg/0.79661*bs_ABW(og, fg)
def bs_AA(og, fg):
    o = bs_ToPlato(og); f = bs_ToPlato(fg); o = max(o, f); return (o - f)/o*100
def bs_RA(og, fg):
    o = bs_ToPlato(og); f = bs_ToPlato(fg); o = max(o, f); return (1 - (0.1808*o + 0.8192*f)/o)*100
def bs_cal12(og, fg):
    og = max(og, fg)
    return 1881.22*fg*(og - fg)/(1.775 - og) + 3550.0*fg*((0.1808*og + 0.8192*fg) - 1.0004)
def bs_uk(og, fg):
    d = (og - fg)*1000; f = 0.125
    for lim, ff in [(6.95,.126),(10.45,.127),(17.25,.128),(26.15,.129),(36.05,.13),(46.55,.131),
                    (57.15,.132),(67.95,.133),(78.85,.134),(89.75,.135)]:
        if d > lim: f = ff
    return f*d
def bs_morey(lb_col_sum, gal): return 1.4922*(lb_col_sum/gal)**0.6859
def bs_hydro(sg, T_F, Tc_F):
    t = (T_F - 32)*5/9; tc = (Tc_F - 32)*5/9
    return sg + sg*((tc**3 - t**3)*1.53243635e-08 + (tc - t)*1.62860587e-05 - (tc**2 - t**2)*5.84994855e-06)
def bs_refr0(b, zero=0.0, wcf=1.02): return bs_ToSG((b - zero)/wcf)
def bs_refr1(og, b, zero=0.0):
    ob = bs_ToPlato(og); fb = b - zero
    return 1.001843 - 0.002318474*ob - 7.775e-6*ob*ob - 3.4e-8*ob**3 + 0.00574*fb + 3.344e-5*fb*fb + 8.6e-8*fb**3
def bs_RI(b): return 1.33302 + 0.001427193*b + 5.791157e-6*b*b
def bs_refr2(fg, b):
    ri = bs_RI(b)
    abw = 1017.5596 - 277.4*fg + ri*(937.8135*ri - 1805.1288)
    re_ = 194.5935 + 129.8*fg + ri*(410.8815*ri - 790.8732)
    oe = (re_ + 2.0665*abw)/(1 + 0.010665*abw)
    return bs_ToSG(oe), abw, re_
def bs_fg_tol(og, fg, tol):
    hi = 1.18; lo = fg
    for _ in range(50):
        mid = (hi + lo)/2; a = bs_ABV(og, mid)
        if abs(a - tol) < 0.04: return mid
        if a > tol: lo = mid
        elif a < tol: hi = mid
    return mid
def bs_gadj_oz(g, t, vol_floz, typ):
    p = (t - g)*1000
    if p <= 0: return 0
    return p*max(vol_floz, 0.01)/128*16/(44 if typ == 0 else 36)
def bs_water(g, t, v): return max(0, v*(g - 1)/(t - 1) - v)

# BeerSmith recete OG/FG. ferm: dict(oz, y, type(0 grain,2 sugar...), use(0 mash,1 steep,2 boil...))
BS_FIXED = {1,2,4,5,6,7}
def bs_og_pts(f, eff, steep_eff=15.0):
    ppg = f['y']*46/100; lb = f['oz']/16
    if f['type'] in BS_FIXED: return lb*ppg
    if f['use'] in (0, 8): return lb*ppg*eff/100
    return lb*ppg*steep_eff/100
def bs_fg_pts(f, eff, att):
    p = bs_og_pts(f, eff); fac = 1 - att/100
    if f['type'] == 2: fac = -0.231
    elif f['type'] in (5,6,7): fac = -0.05
    return p*fac
def bs_recipe(ferms, vol_floz, eff, att_min, att_max, mash_temp_F=None, tol=10.0, all_grain=True):
    gal = vol_floz/128
    og = 1 + sum(bs_og_pts(f, eff) for f in ferms if f['use'] <= 4 or f['use'] == 8)/gal/1000
    att = (att_min + att_max)/2
    if all_grain and mash_temp_F is not None:
        best = min(max(mash_temp_F, 144), 160)
        att += -1.25*(best - 153.5)
    fg = 1 + sum(bs_fg_pts(f, eff, att) for f in ferms)/gal/1000
    if tol > 0 and bs_ABV(og, fg) > tol: fg = bs_fg_tol(og, fg, tol)
    return og, fg, att

# ======================================================================================
# Brewtarget (C++ kaynaktan tasindi)
# ======================================================================================
# Algorithms.cpp:52-54 platoFromSG_20C20C (BeerSmith ToPlato ile ayni kubik)
def bt_SGtoPlato(sg): return -616.868 + 1111.14*sg - 630.272*sg**2 + 135.997*sg**3
# Algorithms.cpp:231-248 Polynomial::rootFind (secant), 266-274 PlatoToSG_20C20C, aralik 0.900..1.150
def bt_PlatoToSG(p):
    f = lambda x: bt_SGtoPlato(x) - p
    g0, g1 = 0.900, 1.150; new = g0; maxsep = abs(g0 - g1)*1e3
    while abs(g0 - g1) > 1e-7:
        new = g1 - (g1 - g0)*f(g1)/(f(g1) - f(g0))
        g0, g1 = g1, new
        if abs(g0 - g1) > maxsep: return float('inf')
    return new

# SucroseConversion.cpp tablosu (USDA), Algorithms.cpp:276-375 lineer interpolasyon
def _load_sucrose():
    path = os.path.join(BT_SRC, "src/measurement/SucroseConversion.cpp")
    rows = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            m = re.match(r"\s*\{\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*\}", line)
            if m: rows.append(tuple(float(x) for x in m.groups()))
    return rows
SUC = _load_sucrose()
def _interp(x, ki, vi):
    pts = sorted((r[ki], r[vi]) for r in SUC)
    if x <= pts[0][0]: return pts[0][1]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        if x0 <= x <= x1:
            return y0 if x1 == x0 else y0 + (x - x0)/(x1 - x0)*(y1 - y0)
    return pts[-1][1]
def bt_BrixToSG(b): return _interp(b, 1, 2)
def bt_SGToBrix(sg): return 0.0 if sg <= 1.0 else _interp(sg, 2, 1)

# Algorithms.cpp:377-388
def bt_platoFromSugar(sugar_kg, wort_l):
    water = wort_l - sugar_kg/1.587
    tot = sugar_kg + water
    return 0.0 if tot == 0 else sugar_kg/tot*100

# Algorithms.cpp:92-104, 435-531 HMRC tablosu (fark 0.1 puana yuvarlanir) + fallback
HMRC = [(0,69,.125),(70,104,.126),(105,172,.127),(173,261,.128),(262,360,.129),(361,465,.130),
        (466,571,.131),(572,679,.132),(680,788,.133),(789,897,.134),(898,1007,.135)]
def bt_ABV(og, fg):
    dx10 = int(round(10*((og - 1)*1000 - (fg - 1)*1000)))
    for lo, hi, k in HMRC:
        if lo <= dx10 <= hi: return dx10/10*k
    return (76.08*(og - fg)/(1.775 - og))*(fg/0.794)
def bt_ABV_fallback(og, fg): return (76.08*(og - fg)/(1.775 - og))*(fg/0.794)

# Recipe.cpp:1297-1350 kalori (kcal/L), Recipe.cpp:2674-2677 12 US oz
def bt_cal_L(og, fg):
    sp = bt_SGtoPlato(og); fp = bt_SGtoPlato(fg)
    re_ = 0.1808*sp + 0.8192*fp
    abw = (sp - re_)/(2.0665 - 0.010665*sp)
    return max(0.0, ((6.9*abw) + 4.0*(re_ - 0.1))*fg*10.0)
def bt_cal12(og, fg): return bt_cal_L(og, fg)*12*FLOZ

# BrewLog.cpp:586-598 gorunur attenuasyon (SG puanlari)
def bt_AA(og, fg): return (og - fg)/(og - 1)*100

# Recipe recalcOgFg (Recipe.cpp:1040-1186), sugarTotals (2855-2895), equivSucrose (RecipeAdditionFermentable.cpp:137-153)
BT_EXISTING = {'Sugar','Extract','Dry_Extract','Fruit','Juice','Honey'}
def bt_equiv_sucrose(f):
    r = f['kg']*f['y']*(1 - f.get('moist', 0)/100)/100
    if f['type'] == 'Grain' and f['stage'] != 'Mash': r *= 0.60
    return r
def bt_recipe(ferms, batch_l, eff, atten_list, trub_l=0.0, postboil_l=None):
    starch = sum(bt_equiv_sucrose(f) for f in ferms if f['type'] not in BT_EXISTING)
    exist = sum(bt_equiv_sucrose(f) for f in ferms if f['type'] in BT_EXISTING)
    nonferm = sum(bt_equiv_sucrose(f)*(1 - f.get('fermfrac', 100)/100) for f in ferms if f['type'] in BT_EXISTING)
    nonferm_starch = sum(bt_equiv_sucrose(f)*(1 - f.get('fermfrac', 100)/100) for f in ferms if f['type'] not in BT_EXISTING)
    if postboil_l is None: postboil_l = batch_l + trub_l
    ratio = min(1.0, max(0.0, (postboil_l - trub_l)/postboil_l))
    exist *= ratio; nonferm *= ratio
    sugar = starch*eff/100 + exist
    nonferm += nonferm_starch*eff/100
    vol = batch_l + trub_l                      # m_finalVolumeNoLosses_l = batchSize + kettleTrubChillerLoss
    og = bt_PlatoToSG(bt_platoFromSugar(sugar, vol))
    pts = (og - 1)*1000
    if nonferm:
        og_f = bt_PlatoToSG(bt_platoFromSugar(sugar - nonferm, vol))
        nf_pts = (bt_PlatoToSG(bt_platoFromSugar(nonferm, vol)) - 1)*1000
    else:
        og_f = og; nf_pts = 0.0
    att = max(atten_list) if atten_list else 0.0
    if atten_list and att <= 0: att = 75.0
    ferm_pts = (pts - nf_pts)*(1 - att/100)
    fg = 1 + (ferm_pts + nf_pts)/1000
    fg_f = 1 + ferm_pts/1000 if nonferm else fg
    return og, fg, att, og_f, fg_f

# ColorMethods.cpp:40-52; Recipe.cpp:989-1030 (renk °L ile, hacim batch+trub)
def bt_mcu(lb_lovibond_sum, gal): return lb_lovibond_sum/gal
def bt_morey(m): return 1.4922*m**0.6859
def bt_daniel(m): return 0.2*m + 8.4
def bt_mosher(m): return 0.3*m + 4.7
def bt_srm_to_L(srm): return (srm + 0.76)/1.3546   # Unit.cpp:705-707
def bt_srm_to_ebc(srm): return srm*25/12.7         # Unit.cpp:704

# Algorithms.cpp:533-570 (°F oran formulu)
def bt_hydro(sg, T_F, Tc_F):
    p = lambda t: 1.00130346 - 0.000134722124*t + 0.00000204052596*t**2 - 0.00000000232820948*t**3
    return sg*p(T_F)/p(Tc_F)

# Algorithms.cpp:394-433 + RefractoDialog.cpp:57-121
def bt_RI(p): return 1.33302 + 0.001427193*p + 0.000005791157*p*p
def bt_terrill(op, cp):
    return (1.001843 - 0.002318474*op - 0.000007775*op**2 - 0.000000034*op**3
            + 0.00574*cp + 0.00003344*cp**2 + 0.000000086*cp**3)
def bt_refr_abv(sg, cp): return (277.8851 - 277.4*sg + 0.9956*cp + 0.00523*cp**2 + 0.000013*cp**3)*(sg/0.79)
def bt_refr_abw(sg, cp):
    ri = bt_RI(cp); return 1017.5596 - 277.4*sg + ri*(937.8135*ri - 1805.1228)
def bt_refr_re(sg, cp):
    ri = bt_RI(cp); return 194.5935 + 129.8*sg + ri*(410.8815*ri - 790.8732)
def bt_refracto(op_plato, cp):
    og = bt_PlatoToSG(op_plato)
    sg = og if op_plato == cp else bt_terrill(op_plato, cp)
    return og, sg, bt_refr_abv(sg, cp), bt_refr_abw(sg, cp), bt_refr_re(sg, cp)

# OgCorrectionTool.cpp:190-302 (kaynatma suresi 0 kabul: buharlasma yok)
def bt_water_density(c):
    co = [0.9999776532, 6.557692037e-5, -1.007534371e-5, 1.372076106e-7, -1.414581892e-9, 5.6890971e-12]
    return sum(k*c**i for i, k in enumerate(co))
def bt_ogcorr(sg_now, wort_l, target):
    rho = bt_water_density(20.0); p = bt_SGtoPlato(sg_now)
    sugar = sg_now*rho*wort_l*p/100; water = sg_now*rho*wort_l*(1 - p/100)
    tp = bt_SGtoPlato(target)
    water_add_l = (100*sugar/tp - sugar - water)/rho
    sugar_add_kg = (target - sg_now)*1000*wort_l*0.1/46.0
    return water_add_l, sugar_add_kg

# BrewLog.cpp:500-534 brewhouse verimi (seker paydaya dahil)
def bt_brewhouse_eff(ferms, og_meas, vol_l):
    allsug = sum(bt_equiv_sucrose(f) for f in ferms)
    act = bt_SGtoPlato(og_meas)/100*vol_l*og_meas
    return 100*act/allsug

# ======================================================================================
def pct(a, b):
    return float('nan') if b == 0 else (a - b)/b*100
rows = []
def row(name, bs, bt, fmt="%.4f"):
    rows.append((name, fmt % bs, fmt % bt, "%+.2f%%" % pct(bt, bs)))

# --- 1. donusumler
row("SG->Plato 1.050", bs_ToPlato(1.050), bt_SGtoPlato(1.050))
for p in (5, 12, 20, 30):
    row("Plato->SG %g" % p, bs_ToSG(p), bt_PlatoToSG(p), "%.5f")
for b in (5, 12, 20):
    row("Brix->SG %g" % b, bs_BrixToSG(b), bt_BrixToSG(b), "%.5f")
row("SG->Brix 1.050", bs_SGToBrix(1.050), bt_SGToBrix(1.050))
row("SG->Brix 1.080", bs_SGToBrix(1.080), bt_SGToBrix(1.080))
row("Brix->RI 10", bs_RI(10), bt_RI(10), "%.6f")
row("SRM 8.819 -> EBC", 8.819*1.97, bt_srm_to_ebc(8.819), "%.3f")
row("30 °L -> SRM", 1.3564*30 - 0.76, 1.3546*30 - 0.76, "%.3f")

# --- 2. alkol / attenuasyon / kalori
for og, fg in [(1.050, 1.010), (1.065, 1.012), (1.080, 1.016), (1.100, 1.020), (1.120, 1.010)]:
    t = "%.3f/%.3f" % (og, fg)
    row("ABV %s (BS Balling vs BT HMRC)" % t, bs_ABV(og, fg), bt_ABV(og, fg), "%.3f")
    row("ABV %s (BS Balling vs BT fallback)" % t, bs_ABV(og, fg), bt_ABV_fallback(og, fg), "%.3f")
    row("UK-vergi ABV %s (BS vs BT)" % t, bs_uk(og, fg), bt_ABV(og, fg), "%.3f")
    row("Gorunur atten. %s (BS Plato vs BT puan)" % t, bs_AA(og, fg), bt_AA(og, fg), "%.2f")
    row("kcal/12oz %s" % t, bs_cal12(og, fg), bt_cal12(og, fg), "%.1f")

# --- 3. recete OG/FG (spes. 12.3)
bs_f = [dict(oz=160, y=80, type=0, use=0), dict(oz=16, y=72, type=0, use=0), dict(oz=16, y=100, type=2, use=2)]
bt_f = [dict(kg=10*LB, y=80, type='Grain', stage='Mash'), dict(kg=1*LB, y=72, type='Grain', stage='Mash'),
        dict(kg=1*LB, y=100, type='Sugar', stage='Boil')]
og1, fg1, att1 = bs_recipe(bs_f, 704, 72, 73, 77, mash_temp_F=152)
og2, fg2, att2, ogf2, fgf2 = bt_recipe(bt_f, 5.5*GAL, 72, [75])
row("Recete OG (12.3)", og1, og2, "%.4f"); row("Recete FG (12.3)", fg1, fg2, "%.4f")
row("Recete ABV (12.3)", bs_ABV(og1, fg1), bt_ABV(ogf2, fgf2), "%.3f")
row("Recete atten. kullanilan", att1, att2, "%.3f")
# FG: BS mash-sicaklik duzeltmesi kapali + seker -0.231 yerine normal attenuasyon -> etkinin ayristirilmasi
og1b, fg1b, _ = bs_recipe(bs_f, 704, 72, 73, 77, mash_temp_F=None)
row("Recete FG (BS mash duz. KAPALI)", fg1b, fg2, "%.4f")

# 3b. Steep tahili: 1 lb crystal 72% Steep, diger her sey ayni
bs_fs = [dict(bs_f[0]), dict(oz=16, y=72, type=0, use=1), dict(bs_f[2])]
bt_fs = [dict(bt_f[0]), dict(kg=1*LB, y=72, type='Grain', stage='Boil'), dict(bt_f[2])]
og1s, fg1s, _ = bs_recipe(bs_fs, 704, 72, 73, 77, mash_temp_F=152)
og2s, fg2s, *_ = bt_recipe(bt_fs, 5.5*GAL, 72, [75])
row("Steep crystal 1 lb: OG", og1s, og2s, "%.4f")
row("Steep crystal 1 lb: puan katkisi", (16/16)*72*46/100*15/100/5.5, (16/16)*72*46/100*0.6*72/100/5.5, "%.3f")

# 3c. Trub kaybi 0.5 gal (2.0 gal kaynatma sonu 6.0)
og2t, *_ = bt_recipe(bt_f, 5.5*GAL, 72, [75], trub_l=0.5*GAL)
row("Trub 0.5 gal: OG (BS hacmi parti, BT parti+trub)", og1, og2t, "%.4f")

# 3d. Extract recetesi: 6 lb DME (y 95) + 1 lb steep crystal (72), 5 gal, eff 72
bs_fe = [dict(oz=96, y=95, type=4, use=2), dict(oz=16, y=72, type=0, use=1)]
bt_fe = [dict(kg=6*LB, y=95, type='Dry_Extract', stage='Boil'), dict(kg=LB, y=72, type='Grain', stage='Boil')]
og1e, fg1e, _ = bs_recipe(bs_fe, 640, 72, 73, 77, all_grain=False)
og2e, fg2e, *_ = bt_recipe(bt_fe, 5*GAL, 72, [75])
row("Extract+steep OG", og1e, og2e, "%.4f"); row("Extract+steep FG", fg1e, fg2e, "%.4f")

# 3e. Laktoz (fermente olmaz) 1 lb: BS F_G_NOT_FERMENTABLE, BT fermentableFraction 0
bs_fl = bs_f + [dict(oz=16, y=76, type=2, use=2)]
og1l, _, _ = bs_recipe(bs_fl, 704, 72, 73, 77, mash_temp_F=152)
# BS: laktoz -> tam puan FG'ye (not fermentable): elle
fg1l = fg1 + (16/16)*76*46/100/5.5/1000
bt_fl = bt_f + [dict(kg=LB, y=76, type='Sugar', stage='Boil', fermfrac=0)]
og2l, fg2l, _, ogf2l, fgf2l = bt_recipe(bt_fl, 5.5*GAL, 72, [75])
row("Laktoz eklenince OG", og1l, og2l, "%.4f"); row("Laktoz eklenince FG", fg1l, fg2l, "%.4f")
row("Laktoz eklenince ABV", bs_ABV(og1l, fg1l), bt_ABV(ogf2l, fgf2l), "%.3f")

# --- 4. tolerans
fgt = bs_fg_tol(1.100, 1.010, 10)
row("OG 1.100 att.%90: FG (BS tol=10, BT sinirsiz)", fgt, 1.010, "%.4f")
row("OG 1.100 att.%90: ABV", bs_ABV(1.100, fgt), bt_ABV(1.100, 1.010), "%.3f")

# --- 5. renk
mcu = (10*2 + 1*60)/6
row("Morey SRM (renkler °L girildi)", bs_morey(80, 6), bt_morey(mcu), "%.3f")
mcu_srm = (10*bt_srm_to_L(2) + 1*bt_srm_to_L(60))/6
row("Morey SRM (renkler SRM girildi, BT °L'ye cevirir)", bs_morey(80, 6), bt_morey(mcu_srm), "%.3f")
row("Daniels (BT) vs Morey (BS)", bs_morey(80, 6), bt_daniel(mcu), "%.3f")
row("Mosher (BT) vs Morey (BS)", bs_morey(80, 6), bt_mosher(mcu), "%.3f")
for m in (2, 5, 30, 100):
    row("Morey MCU=%g (aynı formül)" % m, 1.4922*m**0.6859, bt_morey(m), "%.3f")

# --- 6. araclar
row("Hidrometre 1.050 @77F cal60F", bs_hydro(1.050, 77, 60), bt_hydro(1.050, 77, 60), "%.5f")
row("Hidrometre 1.050 @100F cal60F", bs_hydro(1.050, 100, 60), bt_hydro(1.050, 100, 60), "%.5f")
row("Hidrometre 1.050 @68F cal59F", bs_hydro(1.050, 68, 59), bt_hydro(1.050, 68, 59), "%.5f")
row("Hidrometre 1.050 @150F cal60F", bs_hydro(1.050, 150, 60), bt_hydro(1.050, 150, 60), "%.5f")
og_r, sg_r, abv_r, abw_r, re_r = bt_refracto(12.0, 12.0)
row("Refrakto tip0 12 Bx (BS WCF1.02, BT WCF yok)", bs_refr0(12), og_r, "%.5f")
og_r, sg_r, abv_r, abw_r, re_r = bt_refracto(bs_ToPlato(1.050), 6.5)
row("Refrakto tip1 OG1.050 6.5Bx: FG", bs_refr1(1.050, 6.5), sg_r, "%.5f")
row("Refrakto tip1: ABV (BS Balling vs BT BYO)", bs_ABV(1.050, bs_refr1(1.050, 6.5)), abv_r, "%.3f")
row("Refrakto tip1: ABV (BS Balling vs BT HMRC)", bs_ABV(1.050, bs_refr1(1.050, 6.5)), bt_ABV(1.050, sg_r), "%.3f")
_, abw2, re2 = bs_refr2(sg_r, 6.5)
row("Refrakto ABW (Bonham 1805.1288 vs BT 1805.1228)", abw2, abw_r, "%.4f")
row("Refrakto RE °P", re2, re_r, "%.4f")
# Terrill: BS/BT OB'yi WCF'siz kullaniyor; Terrill orijinali WCF ile bolunmus Brix ister
row("Terrill WCF'siz vs WCF=1.04 ile (bilgi)", bs_refr1(1.050, 6.5), bt_terrill(bs_ToPlato(1.050)/1.04, 6.5/1.04), "%.5f")
wadd, sadd = bt_ogcorr(1.040, 5*GAL, 1.050)
row("1.040->1.050, 5 gal: ekstrakt oz (BS DME 44ppg vs BT seker)", bs_gadj_oz(1.040, 1.050, 640, 0), sadd/LB*16, "%.3f")
row("Ayni: sukroz dogru deger (46ppg) vs BT", 10*5/46*16, sadd/LB*16, "%.3f")
wadd, _ = bt_ogcorr(1.060, 5*GAL, 1.050)
row("1.060->1.050 5 gal: eklenecek su (gal)", bs_water(1.060, 1.050, 640)/128, wadd/GAL, "%.4f")

# --- 7. verim
row("Brewhouse verimi (spes. 12.6)", 62.23, bt_brewhouse_eff(
    [dict(kg=10*LB, y=80, type='Grain', stage='Mash'), dict(kg=LB, y=100, type='Sugar', stage='Boil')],
    1.050, 5.5*GAL), "%.2f")

# ======================================================================================
w0 = max(len(r[0]) for r in rows)
print("%-*s | %12s | %12s | %10s" % (w0, "Hesap", "BeerSmith", "Brewtarget", "Fark BT-BS"))
print("-"*(w0 + 44))
for r in rows:
    print("%-*s | %12s | %12s | %10s" % (w0, *r))
