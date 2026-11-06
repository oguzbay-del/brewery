#!/usr/bin/env python3
"""Maya / karbonasyon: Brewtarget (BT) formullerinin BeerSmith 4 (BS) test vektorleriyle karsilastirmasi.

BT kaynaklari:
  src/tools/PitchDialog.cpp   (calculate 94-140, updateViabilityFromDate 178-183)
  src/tools/PrimingDialog.cpp (calculate 57-101)
  src/Algorithms.cpp          (platoFromSG_20C20C 52-54, BrixToSgAt20C 331-372, interpolatedConversion 118-175)
  src/measurement/SucroseConversion.cpp (USDA 1970 Brix tablosu, 0-80 Brix)
  src/measurement/Unit.cpp:766 (vol <-> g/L, 1.96)
BS kaynagi: /home/user/formuller/04-maya-mead-karbonasyon.md (+ testler/04_maya_karb_tv.py)
Calistir: python3 -I 04_maya_karb_bt_vs_bs.py [brewtarget_kok_dizini]
"""
import math, re, sys, bisect
BT_ROOT = sys.argv[1] if len(sys.argv) > 1 else "/home/user/src/brewtarget"
FLOZ_ML = 29.57352956
def gal(L): return L*1000/FLOZ_ML/128
def C2F(c): return c*9/5+32

# ---------------- ortak ----------------
def plato(sg):  # BT Algorithms.cpp:52 ve BS §0.2 ayni polinom
    return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868

# ---------------- BeerSmith (spesifikasyondan) ----------------
def bs_viab(v0, r, days): return v0 if days <= 0 else max(v0*((100-r)/100)**(days/30), 5.0)
def bs_innoc(g): return math.exp(math.log(g)*-2.1791038+5.512221)
def bs_growth(I, stir): 
    g = math.exp((math.log(I)-5.512221)/-2.1781038)
    if stir: g *= 1.45
    return min(max(g, 1.0), 6.0)
def bs_starter(need, avail, stir):
    if need <= avail: return 0.0
    r = need/avail/(1.45 if stir else 1)
    return 0.0 if r <= 1 else avail/bs_innoc(r)
def bs_residual(TF): T = max(TF, 32); return 3.0378-0.050062*T+0.00026555*T*T
def bs_corn_g(vols, TF, V_gal): return 15.195*V_gal*(vols-bs_residual(TF))
def bs_psi(vols, TF):
    T = max(TF, 32); V = vols
    return max(-16.6999-0.0101059*T+0.00116512*T*T+0.173354*T*V+4.24267*V-0.0684226*V*V, 0)
def bs_toSG(p): return p/(258.6-(p/258.6)*227.1)+1
def bs_brixToSG(b): return b/(258.6-(b/258.2)*227.1)+1

# ---------------- Brewtarget (C++'tan bire bir) ----------------
def bt_viab(days):  # PitchDialog.cpp:182, QSpinBox int'e kirpar (min 1, max 100: ui/pitchDialog.ui:209-214)
    return min(max(int(97 - 0.7*days), 1), 100)
def bt_pitch(vol_l, og, rate=0.75, viab=97, vials=1, aeration=0):
    p = plato(og)
    cells = rate*1e6 * vol_l*1e3 * p                     # :107
    vials_needed = cells/(viab*1e9)                      # :108  (paket = 100 milyar varsayimi)
    dry_g = cells/20e9                                   # :109  (20 milyar/g)
    vials = min(vials, max(1, math.floor(vials_needed))) # :112 spinbox max
    af = {1: 1.33, 2: 2.66}.get(aeration, 1.0)          # :115-125
    tot = vials*viab                                     # :128 (milyar)
    growth = (cells/1e9)/tot                             # :131
    inoc = (12.522/growth)**2.18                         # :132
    starter_l = tot/(inoc*af)                            # :133
    return dict(cells_bn=cells/1e9, vials_needed=vials_needed, dry_g=dry_g, growth=growth, inoc=inoc, starter_l=starter_l)
def bt_white_starter(need, avail, af=1.0):  # ayni modelin genel hali (avail BS'den alinarak)
    g = need/avail; return avail/(((12.522/g)**2.18)*af)
def bt_residual(Tc): return 1.57*0.97**Tc                # PrimingDialog.cpp:66
def bt_priming_g(vol_l, Tc, vols, sugar):
    co2_mol = (vols-bt_residual(Tc))*vol_l/22.4          # :67-69
    return {"glukoz_monohidrat": co2_mol/2*198.0, "glukoz": co2_mol/2*180.0,
            "sakaroz": co2_mol/4*342.0, "DME": co2_mol/2*180.0/0.60}[sugar]  # :77-90
def load_usda():
    txt = open(BT_ROOT+"/src/measurement/SucroseConversion.cpp", encoding="utf-8").read()
    rows = re.findall(r"\{\s*([0-9.]+),\s*([0-9.]+),\s*([0-9.]+)\s*\}", txt)
    return [(float(b), float(sg)) for _, b, sg in rows]
USDA = load_usda()
def bt_brixToSG(b):  # Algorithms.cpp:331 + interpolatedConversion: lineer interpolasyon, araligin disinda kirp
    xs = [r[0] for r in USDA]
    if b >= xs[-1]: return USDA[-1][1]
    if b <= xs[0]: return USDA[0][1]
    i = bisect.bisect_left(xs, b)
    if xs[i] == b: return USDA[i][1]
    (x0, y0), (x1, y1) = USDA[i-1], USDA[i]
    return y0+(y1-y0)*(b-x0)/(x1-x0)

def pct(bt, bs): return float("nan") if bs == 0 else (bt-bs)/bs*100
rows = []
def row(name, bs, bt, unit=""):
    rows.append((name, bs, bt, unit)); 
    d = "" if bs is None or bt is None else f"{pct(bt,bs):+.1f} %"
    f = lambda x: "YOK" if x is None else f"{x:.3f}"
    print(f"| {name} | {f(bs)} {unit} | {f(bt)} {unit} | {d} |")

print("| Test | BeerSmith | Brewtarget | Fark (BT-BS)/BS |\n|---|---|---|---|")
# 1) canlilik 90 gun sivi
row("Canlilik, sivi paket 90 gun", bs_viab(97, 10, 90), bt_viab(90), "%")
row("Canlilik, sivi White Labs 90 gun", bs_viab(97, 5, 90), bt_viab(90), "%")
row("Canlilik, kuru 90 gun", bs_viab(98, 1.66, 90), None, "%")
row("Canlilik, sivi 30 gun", bs_viab(97, 10, 30), bt_viab(30), "%")
row("Canlilik, sivi 180 gun", bs_viab(97, 10, 180), bt_viab(180), "%")
# 2) 20 L 1.060 ale
need_bs = 0.75*20*plato(1.060)
bt = bt_pitch(20, 1.060, 0.75, viab=bt_viab(90), vials=1, aeration=0)
row("Gerekli hucre 20 L 1.060 @0.75", need_bs, bt["cells_bn"], "milyar")
avail_bs = 100*bs_viab(97, 10, 90)/100
row("Mevcut canli hucre (1 paket, 90 g)", avail_bs, float(bt_viab(90)), "milyar")
row("Paket sayisi (starter yok)", need_bs/avail_bs, bt["vials_needed"], "adet")
row("Starter, stir yok (kendi canliligi)", bs_starter(need_bs, avail_bs, False), bt["starter_l"], "L")
bt2 = bt_pitch(20, 1.060, 0.75, viab=bt_viab(90), vials=1, aeration=2)
row("Starter, stir plate (kendi canliligi)", bs_starter(need_bs, avail_bs, True), bt2["starter_l"], "L")
bt3 = bt_pitch(20, 1.060, 0.75, viab=bt_viab(90), vials=1, aeration=1)
row("Starter, O2 baslangic (BT 1.33)", None, bt3["starter_l"], "L")
# ayni mevcut hucre (70.71 milyar) ile sadece buyume modeli karsilastirmasi
row("Starter, stir yok, ayni A=70.71", bs_starter(need_bs, avail_bs, False), bt_white_starter(need_bs, avail_bs), "L")
row("Starter, stir, ayni A=70.71", bs_starter(need_bs, avail_bs, True), bt_white_starter(need_bs, avail_bs, 2.66), "L")
row("Starter, taze paket (97), stir yok", bs_starter(need_bs, 97, False), bt_pitch(20,1.060,0.75,97,1,0)["starter_l"], "L")
row("Starter, taze paket (97), stir", bs_starter(need_bs, 97, True), bt_pitch(20,1.060,0.75,97,1,2)["starter_l"], "L")
row("Kuru maya, g (BS 75 g/hL)", 20*75/100, bt["dry_g"], "g")
row("Kuru maya, g (BS paket modu 18 mlr/g)", need_bs/18, bt["dry_g"], "g")
# buyume egrisi: ayni inokulasyon orani icin buyume katsayisi
for I in (10, 25, 46.41, 100):
    g_bt = 12.522*I**(-1/2.18)
    row(f"Buyume katsayisi, I={I} M/ml (stir yok)", bs_growth(I, False), g_bt, "x")
# 3) priming 2.4 vol 20 C 19 L
V = 19; G = gal(V)
row("Kalinti CO2 20 C", bs_residual(68), bt_residual(20), "vol")
for Tc in (0, 4, 10, 15, 25):
    row(f"Kalinti CO2 {Tc} C", bs_residual(C2F(Tc)), bt_residual(Tc), "vol")
corn = bs_corn_g(2.4, 68, G)
row("Misir sekeri (dekstroz monohidrat)", corn, bt_priming_g(V, 20, 2.4, "glukoz_monohidrat"), "g")
row("Glukoz (susuz)", None, bt_priming_g(V, 20, 2.4, "glukoz"), "g")
row("Sofra sekeri (sakaroz)", corn/1.099, bt_priming_g(V, 20, 2.4, "sakaroz"), "g")
row("DME", corn*1.538, bt_priming_g(V, 20, 2.4, "DME"), "g")
# formul farkini izole et: ayni kalinti CO2 ile
res = bs_residual(68)
bt_same = (2.4-res)*V/22.4/2*198
row("Misir sekeri, ayni kalinti (0.8615)", corn, bt_same, "g")
# 4) basinc
row("Fici basinci 4 C 2.5 vol", bs_psi(2.5, C2F(4)), None, "psi")
# 5) birim
row("2.4 vol -> g/L", 2.4*1.9642, 2.4*1.96, "g/L")
row("4.714 g/L -> vol", 4.71408/1.96042, 4.71408/1.96, "vol")
# 6) Brix -> SG (tatlandirma araci girdisi; BT'de arac yok, yalniz donusum)
for b in (15, 70, 75, 80, 82.5, 98):
    row(f"Brix->SG {b}", bs_brixToSG(b), bt_brixToSG(b), "SG")
    row(f"Plato->SG {b} (BS hacim hesabi)", bs_toSG(b), bt_brixToSG(b), "SG")
# tatlandirma ornegi: 19 L 0.998 -> 1.010, bal 82.5
for nm, sgA in (("BS Plato->SG", bs_toSG(82.5)), ("BT USDA (80 Bx'te kirpilmis)", bt_brixToSG(82.5))):
    Va = V*(0.998-1.010)/(1.010-sgA)
    print(f"Tatlandirma bal hacmi ({nm}): SG_a={sgA:.5f} -> {Va:.4f} L")
