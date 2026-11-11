from math import log, exp, sqrt, floor
FLOZ_ML=29.57352956; OZ_G=0.4535924/16*1000
def L_to_floz(v): return v*1000/FLOZ_ML
def floz_to_L(v): return v*FLOZ_ML/1000
def gal_from_L(v): return v*1000/FLOZ_ML/128
def ToPlato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868
def ToSG(p): return p/(258.6-(p/258.6)*227.1)+1
def BrixToSG(b): return b/(258.6-(b/258.2)*227.1)+1
def F(c): return c*9/5+32
STIR=45.0; GROWTH_LIMIT=4.0
def viability(v0, rate, days, floor_=5.0):
    if days<=0: return v0
    v=v0*((100-rate)/100)**(days/30.0); return max(v,floor_)
def InnocRate(g): return exp(log(g)*-2.1791038+5.512221)
def GrowthRate(I,stir):
    g=exp((log(I)-5.512221)/-2.1781038)
    if stir: g*=1+STIR/100
    return min(max(g,1.0),6.0) if True else g
def cells_needed(rate, V_L, sg): return rate*V_L*ToPlato(sg)
def new_starter_L(need, avail, stir):
    if not need>avail: return 0.0
    r=need/avail
    if stir: r/=1+STIR/100
    if r<=1: return 0.0
    return avail/InnocRate(r)
def two_stage_tool(need, avail, stir):
    avail=max(avail,0.1)
    if not avail<=need: return 0,0
    r=need/avail; s=1.0
    if stir: s=1+STIR/100; r/=s*s
    g=sqrt(r)
    v1=avail/InnocRate(g); v2=avail*g*s/InnocRate(g)
    return v1,v2
def dme_oz(vol_floz, og): return (og-1)*1000*vol_floz/128/45*16
def sugar_g(vols,T_F,gal,dme=False):
    T=max(T_F,32.0); g=((T*0.050062+(vols-3.0378))-T*0.00026555*T)*gal*15.195
    return g*1.538 if dme else g
def residual(T_F): T=max(T_F,32.0); return 3.0378-0.050062*T+0.00026555*T*T
def co2_psi(vols,T_F):
    T=max(T_F,32.0); p=vols*4.24267+(-16.6999-T*0.0101059)+T*0.00116512*T+T*0.173354*vols-vols*0.0684226*vols
    return max(p,0.0)
def so2_rec(pH): return exp(pH*2.21828-4.27809)
def kmeta_g(V_gal,tgt,free): return V_gal*3.785*(tgt-free)/576.0
def sorbate_g(V_gal): return V_gal*1.25
def mead_pitch(sg):
    p=ToPlato(sg)
    if p<=24: return 0.25
    if p<28: return (p-24)*0.0625+0.25
    return 0.5
def tosna(sg, V_gal, pct, fac, method):
    k={0:0.75,1:0.9,2:1.25}[fac]
    base=ToPlato(sg)*10*(pct/100)*k*V_gal
    return base/50 if method==0 else base/100
def yan(method, grams, V_L):
    r={0:40.0,1:100.0,2:212.3}.get(method,100.0)
    return r*grams/V_L if V_L!=0 else r
P=print
P("== Canlilik"); 
for nm,v0,r in [("Sivi (genel, 10%/ay)",97,10),("Sivi White Labs (5%/ay)",97,5),("Kuru (1.66%/ay)",98,1.66),("Slurry (25%/ay)",96,25)]:
    v=viability(v0,r,90); P(f"{nm}: 90 gun -> {v:.2f} %  ; 100 milyar hucreli pakette canli = {100*v/100:.2f} milyar")
P("== 20 L 1.060 ale")
p=ToPlato(1.060); need=cells_needed(0.75,20,1.060)
P(f"Plato={p:.4f}  gerekli hucre={need:.2f} milyar  (min {0.45*20*p:.1f}, max {1.05*20*p:.1f})  EstCellsML={need*1000/20000:.3f} M/ml")
avail=100*viability(97,10,90)/100
for stir in (False,True):
    V=new_starter_L(need,avail,stir); P(f"stir={stir}: avail={avail:.2f} -> starter {V:.4f} L = {L_to_floz(V):.2f} fl oz; DME(1.036)={dme_oz(L_to_floz(V),1.036):.3f} oz = {dme_oz(L_to_floz(V),1.036)*OZ_G:.1f} g")
    I=avail/V; g=GrowthRate(I,stir); P(f"   kontrol: I={I:.2f} M/ml -> buyume {g:.4f} -> {avail*g:.2f} milyar")
    v1,v2=two_stage_tool(need,avail,stir); P(f"   iki kademe: V1={v1:.4f} L, V2={v2:.4f} L")
fresh=100*0.97
for stir in (False,True):
    V=new_starter_L(need,fresh,stir); P(f"taze paket (97 milyar) stir={stir}: {V:.4f} L")
P("== Kuru maya 20 L, 75 g/hL ->", 20*75/100, "g ; hucre(arac)=", 20*7.5, "milyar ; 11.5g paket=", (20*75/100)/11.5, "; hidrasyon ml=",20*75/100*25, "; GoFerm g=",20*75/100*25*0.05)
P("== Karbonasyon 2.4 vol, 20C, 19 L")
gal=gal_from_L(19); T=F(20)
g=sugar_g(2.4,T,gal); P(f"gal={gal:.4f}, T={T}F, residual={residual(T):.4f} vol; misir sekeri={g:.2f} g ({g/OZ_G:.3f} oz); sofra={g/1.099:.2f} g; DME={g*1.538:.2f} g")
P("== Basinc 4C 2.5 vol:", f"{co2_psi(2.5,F(4)):.3f} psi = {co2_psi(2.5,F(4))*6.894757:.2f} kPa (T={F(4)}F)")
P("   kontrol 38F 2.5vol:", f"{co2_psi(2.5,38):.3f}", " 68F 2.5:",f"{co2_psi(2.5,68):.3f}")
P("== SO2 pH 3.5:", f"{so2_rec(3.5):.3f} ppm; 19 L icin K-meta (0 serbestten) = {kmeta_g(gal,so2_rec(3.5),0):.4f} g ; sorbat = {sorbate_g(gal):.3f} g")
for ph in (3.0,3.2,3.4,3.6,3.8,4.0): P(f"   pH {ph}: {so2_rec(ph):.1f} ppm")
P("== Mead 1.100, 19 L, %100 bal, orta N")
gal=gal_from_L(19)
fo=tosna(1.100,gal,100,1,0); fk=tosna(1.100,gal,100,1,1)
P(f"Plato={ToPlato(1.100):.3f}; Fermaid-O toplam={fo:.3f} g (4x {fo/4:.3f}); Fermaid-K toplam={fk:.3f} g (4x {fk/4:.3f})")
P(f"YAN(O)={yan(0,fo,19):.2f} ppm, esdeger={4*yan(0,fo,19):.2f}; YAN(K)={yan(1,fk,19):.2f} ppm")
P(f"Mead maya: {mead_pitch(1.100)} g/L -> {19*mead_pitch(1.100):.2f} g; 1.120 icin {mead_pitch(1.120):.4f} g/L (P={ToPlato(1.120):.3f})")
P(f"GoFerm (5 g maya) = {5*25*0.05} g, su = {5*25} ml")
P("== Backsweeten: 19 L 0.998 -> 1.010, bal 82.5 Bx")
sgA=ToSG(82.5); Va=19*(0.998-1.010)/(1.010-sgA)
P(f"Plato->SG(82.5)={sgA:.5f}; bal hacmi={Va:.4f} L; agirlik(Brix->SG)={BrixToSG(82.5)*Va:.4f} kg  (BrixToSG={BrixToSG(82.5):.5f})")
P("== Birim: 2.4 vol -> g/L", 2.4*1.9642, " ; 4.71408 g/L -> vol", 4.71408/1.96042)

# ---------------------------------------------------------------------------
# T1.14f: spesifikasyonda olup yukarida referansi olmayan hesaplar (eklenti).
# Ustteki fonksiyonlar ve cikti degismez. Burasi hicbir sey yazdirmaz.
# Tarihler gun numarasi olarak alinir (motorun saati yok).
# ---------------------------------------------------------------------------

# --- Enum'lar (§1.1, §2.1, §3.1, §2.8): kod -> ad ---
ENUMS = {
    "y_form": ["Liquid", "Dry", "Slant", "Culture"],
    "yc_yeast_form": ["Liquid", "Dry", "Slurry"],
    "yc_type": ["Ale or Cider", "Lager", "Hybrid/Wheat", "Mead or Wine"],
    "y_type": ["Ale", "Lager", "Wine", "Champagne", "Wheat"],
    "r_type": ["Extract", "Partial Mash", "All Grain", "Cider", "Mead", "Wine", "Seltzer"],
    "r_ale_type": ["Ale", "Lager", "Hybrid"],
    "mead_method": ["TOSNA 2/3 (Fermaid-O)", "TiOSNA (Fermaid-K)"],
    "yeast_factor": ["Low N", "Medium N", "High N"],
    "carb_type": ["Bottle", "Keg", "Keg with priming agent"],
    "back_agent": ["Honey", "Apple Juice Concentrate", "Juice", "Dry Malt Extract",
                   "Table Sugar", "Other"],
}


def enum_name(kind, code):
    t = ENUMS[kind]
    return t[code] if 0 <= code < len(t) else None


# --- Varsayilanlar (§1.2, §1.7, §1.13, §2.1, §3.1) ---
DEFAULTS = {
    "o_y_dry_viability": 98.0, "o_y_liquid_viability": 97.0, "o_y_slurry_viability": 96.0,
    "o_y_dry_aging": 1.66, "o_y_liquid_aging": 10.0, "o_y_slurry_aging": 25.0,
    "o_y_ale_pitch": 0.75, "o_y_lager_pitch": 1.5, "o_y_mix_pitch": 1.0,
    "o_y_wine_pitch": 0.5, "o_y_growth_starter": 4.0, "o_y_stir_pct": 45.0,
    "o_y_dry_cells_per_gram": 18.0, "o_y_dry_hydrate": 25.0, "o_y_goferm": 0.05,
    "yc_pitch_rate": 0.75, "yc_starter_og": 1.036, "rec_dry_rate": 75.0,
    "dry_pack_g": 11.5, "dry_pack5_g": 5.0, "dry_pkg_g": 11.0, "dry_cells_per_g_eff": 10.0,
    "mead_og": 1.090, "mead_volume_floz": 640.0, "mead_yeast_g": 5.0, "mead_honey_pct": 100.0,
    "wine_ph": 3.4, "wine_volume_floz": 640.0, "wine_free_so2": 0.0, "wine_target_so2": 30.0,
    "carb_vols": 2.3, "carb_temp_f": 68.0, "carb_volume_floz": 640.0,
    "back_volume_floz": 640.0, "back_sg": 1.015, "back_target_sg": 1.025,
}


def Default(name):
    return DEFAULTS.get(name)


# --- §1.3 yas ---
def AgeDays(pkg_day, in_recipe, brew_day, today):
    ref = brew_day if in_recipe else today
    return ref - pkg_day


def YeastAgeTool(today, date_day):
    d = today - date_day
    if d < 1:
        return 0.0
    return d / 30.0


def YeastAgeRecipe(age_days_list):
    return sum(age_days_list) / 30.0


# --- §1.4 canlilik yardimcilari ---
def ViabilityV0Ingredient(form):
    return 98.0 if form == 1 else 96.0 if form == 3 else 97.0


def ViabilityV0Tool(form):
    return 98.0 if form == 1 else 96.0 if form == 2 else 97.0


def ImportAgeRate(form, white_labs):
    if form == 1:
        return 1.66
    if form == 3:
        return 25.0
    if form == 0 and white_labs:
        return 5.0
    return 10.0


def ToolViability(calc, user_v, v0, rate, days):
    return viability(v0, rate, days) if calc else user_v


def ViableCells(total, v):
    return total * v / 100


# --- §1.5 toplam hucre ---
def YeastTotalCells(use_pkgs, cells_pkg, new_amount, form, amount, cells_ml):
    if use_pkgs:
        return cells_pkg * new_amount
    if form == 1:
        return amount * 10
    if form in (0, 2, 3):
        return amount * cells_ml
    return 0.0


def YeastCellsAvail(dry, dry_g, wet_ml, cells_ml):
    return dry_g * 10 if dry else wet_ml * cells_ml


# --- §1.6 recete toplamlari (yeasts: sozluk listesi; ikincile eklenenler atlanir) ---
def _primary(yeasts):
    return [y for y in yeasts if not y["secondary"]]


def YeastCellsNoStarter(calc_viability, yeasts):
    k = "viable_cells" if calc_viability else "tot_cells"
    return sum(y[k] for y in _primary(yeasts))


def YeastPackCount(yeasts):
    return sum(y["new_amount"] for y in _primary(yeasts))


def YeastAvgPackSize(cells, amount_sum, valid=True):
    if not valid:
        return 0.0
    if cells <= 0 or amount_sum == 0:
        return 100.0
    return cells / amount_sum


def YeastPackViability(yeasts):
    p = _primary(yeasts)
    if not p:
        return 98.0
    return sum(y["viability_pct"] for y in p) / len(p)


def YeastTotGrams(yeasts):
    t = 0.0
    for y in _primary(yeasts):
        if y["form"] == 1:
            t += y["new_amount"] * 11.0 if y["use_pkgs"] == 1 else y["amount_oz"] * OZ_G
    return t


def YeastFirstFields(yeasts):
    """(YeastForm, YeastUsePkgs, YeastCellsMl, YeastDryPitchRate) ilk birincil mayadan."""
    p = _primary(yeasts)
    if not p:
        return [0, 1, 1.0, 75.0]
    y = p[0]
    return [y["form"], y["use_pkgs"], y["cells_ml"], y["dry_pitch_rate"]]


# --- §1.7 hedef pitch ---
def RecommendedLiquidPitch(r_type, ale_type, ale=0.75, lager=1.5, mix=1.0, wine=0.5):
    if r_type in (4, 5, 6, 7):
        return wine
    if ale_type == 0:
        return ale
    if ale_type == 1:
        return lager
    if ale_type == 2:
        return mix
    return 0.75


def PitchMinMaxRecipe(rec):
    return [rec - 0.4, rec + 0.4]


def PitchMinMaxTool(tool_type, ale=0.75, lager=1.5, mix=1.0, wine=0.5):
    t = {0: (ale, 0.30), 1: (lager, 0.25), 2: (mix, 0.40), 3: (wine, 0.25)}.get(tool_type)
    if t is None:
        return None
    return [t[0] - t[1], t[0] + t[1]]


def PitchRange(lo, hi):
    return "%.2f-%.2f" % (lo, hi)


def DryMinPitchRate(rec_dry):
    return max(5.0, float(floor(rec_dry / 4)))


def DryMaxPitchRate(rec_dry):
    return 2 * rec_dry


def DryPitchRange(rec_dry):
    return "%.1f-%.1f" % (DryMinPitchRate(rec_dry), DryMaxPitchRate(rec_dry))


def IsDryByWeight(form, use_pkgs, tool):
    return form == 1 if tool else (form == 1 and use_pkgs != 1)


# --- §1.8 gerekli hucre ---
def TotYeastCellsNeeded(dry_by_weight, pitch, V_L, sg, dry_rate, rec_dry_rate):
    if dry_by_weight:
        return V_L * 7.5 * (max(dry_rate, 0.01) / max(rec_dry_rate, 0.01))
    return cells_needed(pitch, V_L, sg)


def EstCellsML(n, V_ml):
    return n * 1000 / max(V_ml, 0.1)


# --- §1.9 starter sonrasi hucre (starter_L <= 0: buyume yok, hucre ayni kalir) ---
def CellsStarter(viable, starter_L, stir):
    I = viable / starter_L if starter_L > 0 else 0.0
    if I <= 0:
        return viable
    return viable * GrowthRate(I, stir)


def CellsSecStarter(cells1, sec_L, stir):
    return CellsStarter(cells1, sec_L, stir)


# --- §1.11 recete iki kademe (buyume sinirli, A < N) ---
def two_stage_recipe(need, avail, stir, growth_cap=4.0):
    avail = max(avail, 0.1)
    if not avail < need:
        return 0, 0
    r = need / avail; s = 1.0
    if stir: s = 1 + STIR / 100; r /= s * s
    g = min(sqrt(r), growth_cap)
    v1 = avail / InnocRate(g); v2 = avail * g * s / InnocRate(g)
    return v1, v2


# --- §1.12 paket sayilari ---
def YeastPacksNoStarter(n, pkt):
    if pkt <= 0: pkt = 1.0
    return 1 if n < pkt else int(n / pkt + 0.99)


def YeastPacksStarter(n, pkt, growth, stir):
    """CalcYeastPacksNeeded (starter varsa): int(paket) + 0.1."""
    if pkt <= 0: pkt = 1.0
    G = growth if growth > 0 else 1.2
    if stir: G /= 1 + STIR / 100
    p = 1 if n < pkt else int(n / (G * pkt) + 0.99)
    return int(p) + 0.1


def CalcYeastStarterSize(r, pkt_size, pkt):
    return pkt * pkt_size / InnocRate(r) if r > 1 else 0.0


# --- §1.13 kuru maya ---
def DryGramsVol(dry_rate, V_L):
    return dry_rate * V_L / 100


def DryGramsPkg(n, cells_per_g=18.0):
    return n / (cells_per_g if cells_per_g > 0 else 0.1)


def YeastPacksDry(g): return g / 11.5
def YeastDry5(g): return g / 5.0
def YeastDryHydrate(tot_g, per_g=25.0): return tot_g * per_g
def YeastGoferm(tot_g, per_g=25.0, goferm=0.05): return tot_g * per_g * goferm


# --- §1.15 diger ---
def AvgAttenuation(lo, hi): return (lo + hi) / 2


def YeastUnitsCode(use_pkgs, form, dry_unit=10, wet_unit=15):
    if use_pkgs: return "pkgs"
    return dry_unit if form == 1 else wet_unit


def ImportHookTolerance(tol, y_type):
    if tol == 10:
        if y_type == 2: return 15
        if y_type == 3: return 18
    return tol


# --- §2.2, §2.3 mead maya ---
def MeadYeastGrams(sg, V_L): return V_L * mead_pitch(sg)
def MeadGoferm(g): return g * 25.0 * 0.05
def MeadHydrate(g): return g * 25.0
def MeadHydrate1(g): return g * 25.0


# --- §2.4 besin programi ---
YEAST_N = {0: 0.75, 1: 0.9, 2: 1.25}


def MeadYanTarget(sg, pct, fac):
    return ToPlato(sg) * 10 * YEAST_N[fac] * (pct / 100)


def MeadTotFermO(sg, V_gal, pct, fac, method):
    return MeadYanTarget(sg, pct, fac) * V_gal / 50 if method == 0 else 0.0


def MeadTotFermK(sg, V_gal, pct, fac, method):
    return MeadYanTarget(sg, pct, fac) * V_gal / 100 if method == 1 else 0.0


def MeadFermO(sg, V_gal, pct, fac, method): return MeadTotFermO(sg, V_gal, pct, fac, method) / 4
def MeadFermK(sg, V_gal, pct, fac, method): return MeadTotFermK(sg, V_gal, pct, fac, method) / 4


def MeadPctHoney(ferms, primary_pts):
    """ferms: {type, subtype, pts}; bal = type 7 ve subtype < 7. primary_pts 0 ise 0."""
    if primary_pts == 0:
        return 0.0
    h = sum(f["pts"] for f in ferms if f["type"] == 7 and f["subtype"] < 7)
    return h / primary_pts * 100


# --- §2.5 YAN ---
def _yan_c(method): return {0: 40.0, 1: 100.0, 2: 212.3}.get(method, 100.0)


def YANEquivalent(method, y): return y * 4 if method == 0 else y


def MeadYANAdded(method, total_g, V_L, recipe):
    return yan(method, total_g / 4 if recipe else total_g, V_L)


def MeadYANEquivalent(method, total_g, V_L, recipe):
    return YANEquivalent(method, MeadYANAdded(method, total_g, V_L, recipe))


def NutrientNeeded(ppm, method, V_L): return ppm * V_L / _yan_c(method)


# --- §2.8 tatlandirma ---
BACK_OG = {0: 82.5, 1: 70.0, 2: 15.0, 3: 75.0, 4: 98.0}


def BackAgentOG(agent): return BACK_OG.get(agent, 15.0)


def BackAgentVolSG(V_L, sg_bev, sg_target, sg_a):
    if sg_target == sg_a: return 0.0
    return V_L * (sg_bev - sg_target) / (sg_target - sg_a)


def BackAgentVol(V_L, sg_bev, sg_target, agent):
    return BackAgentVolSG(V_L, sg_bev, sg_target, ToSG(BackAgentOG(agent)))


def BackAgentWeight(V_L, sg_bev, sg_target, agent):
    return BrixToSG(BackAgentOG(agent)) * BackAgentVol(V_L, sg_bev, sg_target, agent)


# --- §3.3/§3.4 karbonasyon miktari ---
def CarbSugarAgent(agent, vols, T_F, gal):
    """agent 0 misir, 1 DME, 2 sofra sekeri (misir / 1.099); gram."""
    g = sugar_g(vols, T_F, gal, False)
    return g * 1.538 if agent == 1 else g / 1.099 if agent == 2 else g


def CarbAmount(c_type, vols, T_F, gal, carb_rate, agent=0):
    """Keg (1): ["psi", basinc]; diger: ["g", seker], carb_rate != 0 ise / (rate/100)."""
    if c_type == 1:
        return ["psi", co2_psi(vols, T_F)]
    g = CarbSugarAgent(agent, vols, T_F, gal)
    if carb_rate != 0: g /= carb_rate / 100
    return ["g", g]


def CarbEquivFunc(rate): return 100 / max(rate, 1)


# --- §3.6, §3.7 ---
def CarbCalories(total, alc): return total - alc
def CarbGrams(carb_cal): return carb_cal / 4


def BConvertCarbonation(v, to_gl): return v * 1.9642 if to_gl else v / 1.96042
