import math
def ToPlato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868
def ToSG(p): return p/(258.6 - (p/258.6)*227.1) + 1.0
def BrixToSG(b): return b/(258.6 - (b/258.2)*227.1) + 1.0
def SGToBrix(sg): return ((182.4601*sg - 775.6821)*sg + 1262.7794)*sg - 669.5622
def BrixToRI(b): return 1.33302 + 0.001427193*b + 5.791157e-6*b*b
def RIToBrix(ri): return (math.sqrt(2.036879859249e-06 - (1.33302-ri)*2.3164628e-05) - 0.001427193)/1.1582314e-05
def RE(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); return 0.1808*o+0.8192*f
def ABW(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); re=0.1808*o+0.8192*f
    return (o-re)/(2.0665-0.010665*o)
def ABV(og,fg): return fg/0.79661*ABW(og,fg)
def AA(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); return (o-f)/o*100
def RA(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); return (1-(0.1808*o+0.8192*f)/o)*100
def alc_cal(og,fg): return 1881.22*fg*(og-fg)/(1.775-og)
def ext_cal(og,fg): return 3550.0*fg*((0.1808*og+0.8192*fg)-1.0004)
def uk(og,fg):
    # §6.7 (T1.12 C1): eşikler tam sayı gravite biriminde (1 birim = 0.00001 SG), kesin ">"
    du=round(og*100000)-round(fg*100000); f=0.125
    for lim,ff in [(695,.126),(1045,.127),(1725,.128),(2615,.129),(3605,.13),(4655,.131),(5715,.132),(6795,.133),(7885,.134),(8975,.135)]:
        if du>lim: f=ff
    return f*(du/100)
def morey(lb_srm_sum, gal):
    # Karar (T1.16): negatif toplam ya da hacim <= 0 karmaşık sayı/sıfıra bölme verirdi; ValueError
    if lb_srm_sum < 0 or gal <= 0: raise ValueError("morey: toplam >= 0, hacim > 0 olmalı")
    return 1.4922*(lb_srm_sum/gal)**0.6859
def hydro(sg,T_F,Tc_F):
    t=(T_F-32)*5/9; tc=(Tc_F-32)*5/9
    return sg + sg*((tc**3-t**3)*1.53243635e-08 + (tc-t)*1.62860587e-05 - (tc**2-t**2)*5.84994855e-06)
def refr0(b,zero=0.0,wcf=1.02): return ToSG((b-zero)/wcf)
def refr1(og,b,zero=0.0):
    ob=ToPlato(og); fb=b-zero
    return 1.001843-0.002318474*ob-7.775e-6*ob*ob-3.4e-8*ob**3+0.00574*fb+3.344e-5*fb*fb+8.6e-8*fb**3
def refr2(fg,b,zero=0.0):
    ri=BrixToRI(b-zero)
    abw=(1017.5596-277.4*fg+ri*(937.8135*ri-1805.1288))/100.0
    re=194.5935+129.8*fg+ri*(410.8815*ri-790.8732)
    oe=(re+2.0665*abw*100)/(1+0.010665*abw*100)
    return ToSG(oe), abw*100, re, oe
def fg_tol(og,fg,tol):
    hi=1.18; lo=fg
    for i in range(50):
        mid=(hi+lo)/2; a=ABV(og,mid)
        if abs(a-tol)<0.04: return mid
        if a>tol: lo=mid
        elif a<tol: hi=mid
    return mid
def gadj_oz(g,t,vol_floz,typ):
    if typ not in (0, 1): raise ValueError("özüt türü 0 (DME) ya da 1 (LME) olmalı")  # Karar (T1.16)
    p=(t-g)*1000
    if p<=0: return 0
    p*=max(vol_floz,0.01)/128
    return p*16/(44 if typ==0 else 36)
def gadj_water(g,t,v): return max(0, v*(g-1)*1000/((t-1)*1000)-v)
print("ToPlato(1.050)=%.4f ToSG(12)=%.5f BrixToSG(12)=%.5f SGToBrix(1.050)=%.4f"%(ToPlato(1.050),ToSG(12),BrixToSG(12),SGToBrix(1.050)))
print("RI(10Bx)=%.6f Brix(RI)=%.4f"%(BrixToRI(10),RIToBrix(BrixToRI(10))))
for og,fg in [(1.050,1.010),(1.065,1.012),(1.080,1.016)]:
    print(og,fg,"ABW=%.3f ABV=%.3f AA=%.2f RA=%.2f RE=%.3f alc=%.1f ext=%.1f tot=%.1f UK=%.3f"%(ABW(og,fg),ABV(og,fg),AA(og,fg),RA(og,fg),RE(og,fg),alc_cal(og,fg),ext_cal(og,fg),alc_cal(og,fg)+ext_cal(og,fg),uk(og,fg)))
# OG example: 10 lb (160 oz) 2-row yield 80%, 1 lb crystal 72%, 1 lb corn sugar (type 2, yield 100%), 5.5 gal (704 floz), eff 72%
pts = 160*(80*46/100)*72/1600 + 16*(72*46/100)*72/1600 + 16*(100*46/100)/16
og = pts/(704/128)/1000+1
print("pts=%.3f OG=%.4f"%(pts,og))
# FG: atten avg (73+77)/2=75, mash step 152F -> adj = -1.25*(152-153.5)=+1.875 -> 76.875
att=75+(-1.25)*(152-153.5)
fgp = 160*(80*46/100)*72/1600*(1-att/100) + 16*(72*46/100)*72/1600*(1-att/100) + 16*(100*46/100)/16*(-0.231)
fg = fgp/(704/128)/1000+1
print("att=%.3f fgpts=%.3f FG=%.4f ABV=%.3f"%(att,fgp,fg,ABV(og,fg)))
print("tol: fg_tol(1.100,1.010,10)=%.4f ABV=%.3f"%(fg_tol(1.100,1.010,10),ABV(1.100,fg_tol(1.100,1.010,10))))
# color: 10lb 2 SRM + 1 lb 60 SRM, vol 5.5 + trub 0.5 = 6 gal
mcu=(10*2+1*60)/6
print("MCU=%.3f SRM=%.3f EBC=%.3f"%(mcu,morey(80,6),morey(80,6)*1.97))
print("Hydro 1.050 @ 77F cal 60F -> %.5f"%hydro(1.050,77,60))
print("Refr0 12Bx -> %.5f"%refr0(12))
print("Refr1 OG1.050, 6.5Bx -> FG %.5f, ABV %.3f"%(refr1(1.050,6.5),ABV(1.050,refr1(1.050,6.5))))
r=refr2(1.010,6.5); print("Refr2 FG1.010, 6.5Bx -> OG %.5f ABW%% %.3f RE %.3f OE %.3f"%r)
print("GravAdj 1.040->1.050 5gal DME oz=%.3f LME oz=%.3f"%(gadj_oz(1.040,1.050,640,0),gadj_oz(1.040,1.050,640,1)))
print("Water 1.060->1.050 5gal add floz=%.2f (gal %.3f)"%(gadj_water(1.060,1.050,640),gadj_water(1.060,1.050,640)/128))
print("Dilution water: 5gal 1.060 + 1 gal -> %.5f; wort 1gal@1.030 -> %.5f"%(1+(5/6)*0.060, 1+0.060*5/6+0.030/6))
# efficiency: 10 lb grain 80% yield, measured OG 1.050 in 5.5 gal, plus 1 lb sugar
pot=(160*(80*46/100)/16)/(704/128); meas=50 - (16*46/16)/(704/128)
print("Eff=%.2f%%"%(meas/pot*100))
print("UK 1.050/1.010 =%.3f"%uk(1.050,1.010))

# ======================================================================================
# T1.14e: spesifikasyonda olup yukarıda satır içi hesaplanan ya da hiç olmayan fonksiyonlar.
# Yukarıdaki fonksiyonlar ve basılan çıktı değişmedi; burası yalnız eklemedir.
# Gösterim: tip/kullanım kodları §0.2, tüm birimler §0.1 (oz, fl oz, °F, SRM, %).
# ======================================================================================
import copy

FIXED_TYPES = (1, 2, 4, 5, 6, 7)      # sabit verimli tipler: Extract, Sugar, Dry Extract, Fruit, Juice, Honey


def ferm(name="", amount_oz=0.0, yield_pct=0.0, color_srm=0.0, typ=0, use=0, not_fermentable=False,
         late_extract_min=0.0, max_in_batch_pct=100.0, ibu_gal_per_lb=0.0):
    """Fermentable girdisi (§0.2, §2): amount oz, yield %, renk SRM, tip F_G_TYPE, kullanım F_G_USE.
    Alan adları Rust `recipe::Fermentable` ile aynıdır (ADR-0004 madde 1 ve 4); 02 üreticisi de bunu kullanır."""
    return {"name": name, "amount_oz": float(amount_oz), "yield_pct": float(yield_pct),
            "color_srm": float(color_srm), "kind": int(typ), "usage": int(use),
            "not_fermentable": bool(not_fermentable), "late_extract_min": float(late_extract_min),
            "max_in_batch_pct": float(max_in_batch_pct), "ibu_gal_per_lb": float(ibu_gal_per_lb)}


def yeast(min_att=73.0, max_att=77.0, tol=10.0, add_to_secondary=False, form=0):
    """Maya girdisi; Rust `recipe::Yeast` ile aynı alan adları (04'ün toplam alanları sıfır varsayılır)."""
    return {"min_attenuation_pct": float(min_att), "max_attenuation_pct": float(max_att),
            "tolerance_abv_pct": float(tol), "add_to_secondary": bool(add_to_secondary), "form": int(form),
            "total_cells_billion": 0.0, "viable_cells_billion": 0.0, "packs": 0.0, "viability_pct": 0.0,
            "use_packages": True, "amount_oz": 0.0, "cells_per_ml_billion": 1.0,
            "dry_pitch_rate_g_per_hl": 75.0}


def hop(amount_oz=1.0, alpha_pct=5.0, form=0, usage=0, time_min=60.0, whirlpool_temp_f=176.0):
    """Hop girdisi; Rust `recipe::Hop` ile aynı alan adları."""
    return {"amount_oz": float(amount_oz), "alpha_pct": float(alpha_pct), "form": int(form), "usage": int(usage),
            "time_min": float(time_min), "whirlpool_temp_f": float(whirlpool_temp_f)}


def misc(usage=0, time_min=0.0):
    """Misc girdisi; Rust `recipe::Misc` ile aynı alan adları."""
    return {"usage": int(usage), "time_min": float(time_min)}


DEFAULT_HOP_OPTIONS = {"first_wort_adj_pct": 10.0, "type_adj": True, "leaf_adj_pct": 0.0, "plug_adj_pct": 2.0,
                       "pellet_adj_pct": 10.0, "mash_adj_pct": -80.0}


def make_recipe(ferms, typ=2, volume_floz=704.0, trub_loss_floz=0.0, pre_trub_loss_floz=0.0, boil_vol_floz=0.0,
                boil_time_min=60.0, efficiency=72.0, yeasts=None, mash_temps=(), grain_weight_oz=0.0, biab=False,
                steep_efficiency=15.0, mash_fg_adjust=True, mash_fg_adj_temp=153.5, mash_fg_adj_slope=-1.25,
                hops=None, miscs=None, altitude_ft=0.0, hop_util_pct=100.0, whirlpool_carry=False,
                whirlpool_time_min=0.0, hop_options=None):
    """Ortak reçete girdisi (Rust `recipe::RecipeInput`, ADR-0004 madde 1): alan adları yapıyla aynıdır.
    Hacimler fl oz, sıcaklıklar °F, verim %, kind = typ (0 Extract .. 6 Seltzer). Seçenekler reçeteyle taşınır.
    Mash adımı infüzyonu = 10 fl oz × adım sırası (örnek veri), adım türü 0 (infüzyon)."""
    return {"kind": int(typ), "volume_floz": float(volume_floz), "boil_vol_floz": float(boil_vol_floz),
            "boil_time_min": float(boil_time_min), "efficiency_pct": float(efficiency),
            "equipment": {"trub_loss_floz": float(trub_loss_floz), "pre_trub_loss_floz": float(pre_trub_loss_floz),
                          "altitude_ft": float(altitude_ft), "hop_util_pct": float(hop_util_pct),
                          "whirlpool_carry": bool(whirlpool_carry), "whirlpool_time_min": float(whirlpool_time_min)},
            "fermentables": list(ferms), "hops": list(hops or []), "yeasts": list(yeasts or []),
            "miscs": list(miscs or []),
            "mash": {"grain_weight_oz": float(grain_weight_oz), "biab": bool(biab),
                     "steps": [{"kind": 0, "temp_f": float(t), "infusion_floz": 10.0 * (i + 1)}
                               for i, t in enumerate(mash_temps)]},
            "options": {"steep_efficiency_pct": float(steep_efficiency), "mash_fg_adjust": bool(mash_fg_adjust),
                        "mash_fg_adj_temp_f": float(mash_fg_adj_temp), "mash_fg_adj_slope": float(mash_fg_adj_slope),
                        "hop": dict(hop_options or DEFAULT_HOP_OPTIONS)}}


# §12.3 örneği (All Grain, 5.5 gal, %72)
ORNEK_AG = make_recipe(
    [ferm("2-row", 160, 80, 2, 0, 0), ferm("Crystal", 16, 72, 60, 0, 0), ferm("Corn sugar", 16, 100, 0, 2, 2)],
    yeasts=[yeast(73, 77, 10)], mash_temps=[152], volume_floz=704)


# --- §1.3 yoğunluk birimi çevirici (3 SG, 4 Plato, 31 Brix); bilinmeyen kod: ValueError (T1.16) ---
def bconv_hydrometer(x, frm, to):
    if frm not in (3, 4, 31) or to not in (3, 4, 31): raise ValueError("bilinmeyen yoğunluk birimi kodu")  # Karar (T1.16)
    if frm == to:
        return x
    if frm == 3:
        if x > 880: x = x / 1000
        elif x > 88: x = x / 100
        elif x > 8.8: x = x / 10
    if frm == 3: sg = x
    elif frm == 4: sg = ToSG(x)
    elif frm == 31: sg = BrixToSG(x)
    else: return x
    if to == 3: return sg
    if to == 4: return ToPlato(sg)
    if to == 31: return SGToBrix(sg)
    return x


# --- §1.4 refraktometre birimi çevirici (3 SG, 31 Brix, 32 RI) ---
def bconv_refractometer(x, frm, to):
    if frm not in (3, 31, 32) or to not in (3, 31, 32): raise ValueError("bilinmeyen refraktometre birimi kodu")  # Karar (T1.16)
    if frm == to:
        return x
    if frm == 3: bx = bconv_hydrometer(x, 3, 31)
    elif frm == 31: bx = x
    elif frm == 32: bx = RIToBrix(x)
    else: return x
    if to == 3: return bconv_hydrometer(bx, 31, 3)
    if to == 31: return bx
    if to == 32: return BrixToRI(bx)
    return x


# --- §1.5 renk birimi çevirici (29 SRM, 30 EBC, 47 °L) ---
def bconv_color(x, frm, to):
    if frm not in (29, 30, 47) or to not in (29, 30, 47): raise ValueError("bilinmeyen renk birimi kodu")  # Karar (T1.16)
    if frm == to:
        return x
    if frm == 29: srm = x
    elif frm == 30: srm = x / 1.97
    elif frm == 47: srm = 1.3564 * x - 0.76
    else: return x
    if to == 29: return srm
    if to == 30: return srm * 1.97
    if to == 47: return (srm + 0.76) / 1.3564
    return x


# --- §1.6 yield <-> potansiyel ---
def potential_from_yield(y): return 1 + y * 0.046 / 100
def yield_from_potential(sg): return min(100.0, max(0.0, ((sg - 1) * 1000 / 46) * 100))
def bconv_grain_yield(x, frm):
    if frm not in (3, 4, 31): raise ValueError("bilinmeyen yoğunluk birimi kodu")  # Karar (T1.16)
    """from == 3 (SG): YieldFromPotential; aksi halde PotentialFromYield (`to` spesifikasyonda kullanılmaz)."""
    return yield_from_potential(x) if frm == 3 else potential_from_yield(x)


# --- §1.7 sıcaklık (1 °C, 2 °F, 40 K) ---
def bconv_temperature(x, frm, to):
    if frm not in (1, 2, 40) or to not in (1, 2, 40): raise ValueError("bilinmeyen sıcaklık birimi kodu")  # Karar (T1.16)
    if frm == to:
        return x
    if frm == 1: c = x
    elif frm == 2: c = (x - 32) * 5 / 9
    elif frm == 40: c = x - 273.15
    else: return x
    if to == 1: return c
    if to == 2: return c * 9 / 5 + 32
    if to == 40: return c + 273.15
    return x


# --- §2 fermentable puanları ---
def og_points(g, eff, steep_eff=15.0):
    """§2.1: puan·galon."""
    lb_ppg = (g["amount_oz"] / 16) * (g["yield_pct"] * 46 / 100)
    if g["kind"] in FIXED_TYPES:
        return lb_ppg
    if g["usage"] in (0, 8):
        return lb_ppg * eff / 100
    return lb_ppg * steep_eff / 100


def fg_points(g, eff, atten, steep_eff=15.0):
    """§2.2: fermantasyon sonrası kalan puan·galon."""
    p = og_points(g, eff, steep_eff)
    if g["not_fermentable"]:
        return p
    if g["kind"] == 2: f = -0.231
    elif g["kind"] in (5, 6, 7): f = -0.05
    else: f = 1 - atten / 100
    return p * f


# --- §3 OG ve türevleri ---
def og_points_phase(r, phase):
    if phase not in range(9): raise ValueError("aşama kodu 0-8 olmalı")  # Karar (T1.16)
    s = 0.0
    for g in r["fermentables"]:
        if g["usage"] <= phase or g["usage"] == 8:
            s += og_points(g, r["efficiency_pct"], r["options"]["steep_efficiency_pct"])
    return s


def estimate_og(r, v_floz, phase):
    if v_floz <= 0:
        return 1.0
    return 1 + og_points_phase(r, phase) / (v_floz / 128) / 1000


def og_volume(r):
    """Extract reçetede (F_R_TYPE 0) Ekipman.F_E_PRE_TRUB_LOSS, aksi halde F_R_VOLUME (fl oz)."""
    return r["equipment"]["pre_trub_loss_floz"] if r["kind"] == 0 else r["volume_floz"]


def recipe_og(r): return estimate_og(r, og_volume(r), 4)               # §3.3
def recipe_virtual_og(r): return estimate_og(r, og_volume(r), 6)       # §3.4


def est_post_mash_og(r):
    """§3.5."""
    og = estimate_og(r, r["volume_floz"], 0)
    if r["boil_vol_floz"] > 0:
        # Tek formül 05 §5.10 (`preboil_sg`): 1 + (OG - 1) (parti + trub) / kaynatma hacmi (T1.16 B)
        return 1 + (og - 1) * (r["volume_floz"] + r["equipment"]["trub_loss_floz"]) / r["boil_vol_floz"]
    return og


def late_extract_sgs(r, boil_vol_floz=0.0):
    """§3.6: [[sg, t_dk], ...]. Vb: parametre > 0 ise o, değilse reçetedeki alan; <= 0.1 ise 0.1."""
    vb = boil_vol_floz if boil_vol_floz > 0 else r["boil_vol_floz"]
    if vb <= 0.1:
        vb = 0.1
    vol = r["volume_floz"]
    eff, steep = r["efficiency_pct"], r["options"]["steep_efficiency_pct"]
    ratio = (vol + r["equipment"]["trub_loss_floz"]) / vb

    def is_late(g): return g["kind"] in FIXED_TYPES and g["late_extract_min"] > 0

    def sg_of(p): return 1.0 if vol <= 0 else 1 + p * ratio / (vol / 128) / 1000

    def add(prof, sg, t):
        if prof and abs(t - prof[-1][1]) < 0.1:
            prof[-1][0] = sg
            if t == 0.0:
                prof[-1][1] = 0.0  # son girdi her zaman t = 0 (02 §5.1; T1.16 B)
        else:
            prof.append([sg, t])

    p = 0.0
    for g in r["fermentables"]:
        if g["usage"] in (0, 1, 2, 8) and not is_late(g):
            p += og_points(g, eff, steep)
    prof = [[sg_of(p), r["boil_time_min"]]]
    lates = [g for g in r["fermentables"] if g["usage"] in (0, 1, 2, 8) and is_late(g)]
    lates.sort(key=lambda g: (-g["late_extract_min"], g["name"]))
    for g in lates:
        p += og_points(g, eff, steep)
        add(prof, sg_of(p), min(g["late_extract_min"], r["boil_time_min"]))
    for g in r["fermentables"]:
        if g["usage"] == 3:
            p += og_points(g, eff, steep)
    add(prof, sg_of(p), 0.0)
    return prof


def recipe_og_boil(r, boil_vol_floz=0.0): return late_extract_sgs(r, boil_vol_floz)[0][0]


def scale_gravity(r, og_target):
    """§3.7 (IBU yeniden ölçeklemesi 02'nin işi, burada yok). Hata (P_yeni <= 0): None."""
    p_old = (recipe_og(r) - 1) * 1000
    p_new = (og_target - 1) * 1000
    out = copy.deepcopy(r)
    if p_old > 0 and p_new > 0:
        for g in out["fermentables"]:
            g["amount_oz"] = g["amount_oz"] * p_new / p_old
        return out
    if p_new <= 0:
        return None
    return out


def scale_mash_from_grain_weight(mash, w_new):
    """§3.8."""
    m = copy.deepcopy(mash)
    w_old = m["grain_weight_oz"]
    m["grain_weight_oz"] = w_new
    if w_new <= 0 or w_old <= 0 or w_new == w_old:
        return m
    if not m["biab"]:
        for s in m["steps"]:
            if s["infusion_floz"] > 0:
                s["infusion_floz"] = w_new * (s["infusion_floz"] / w_old)
    return m


def total_grain_use_field(r, use, field):
    """§3.9: field = fermentable sözlüğündeki anahtar (ör. "amount_oz")."""
    s = 0.0
    for g in r["fermentables"]:
        if g["usage"] == use:
            s += g[field]
    return s


# --- §4 verim ---
def get_efficiency(r, og, v_floz, mash_only):
    v = 0.01 if v_floz <= 0 else v_floz
    if og <= 0:
        og = recipe_og(r)
    if r["kind"] not in (1, 2):
        return 0.0
    uses = (0, 1, 8) if mash_only else (0, 1, 2, 3, 4, 8)
    s = m = 0.0
    for g in r["fermentables"]:
        if g["usage"] in uses:
            x = (g["amount_oz"] / 16) * (g["yield_pct"] * 46 / 100)
            if g["kind"] in FIXED_TYPES: s += x
            else: m += x
    mg = m / (v / 128)
    pm = (og - 1) * 1000
    if not mash_only:
        pm = pm - s / (v / 128)
    if pm < 0 or mg <= 0:
        return 0.0
    return pm / mg * 100


# --- §5.1 reçete FG'si ve §6.8 ---
def avg_attenuation(min_att, max_att): return (min_att + max_att) / 2


def yeast_attenuation(r):
    """§5.1 adım 1: (atten %, tol %ABV); ilk F_Y_ADD_TO_SECONDARY == 0 maya, yoksa (73, 10)."""
    for y in r["yeasts"]:
        if not y["add_to_secondary"]:
            return avg_attenuation(y["min_attenuation_pct"], y["max_attenuation_pct"]), y["tolerance_abv_pct"]
    return 73.0, 10.0


def mash_adjusted_attenuation(r, atten):
    """§5.1 adım 2 (yalnız All Grain)."""
    if r["kind"] != 2:
        return atten
    tref = r["options"]["mash_fg_adj_temp_f"]
    best, best_diff, prev = tref, 40.0, 0.0
    for s in r["mash"]["steps"]:
        t = s["temp_f"]
        d = abs(t - tref)
        if d < best_diff:
            if prev < tref and prev >= 139 and t > tref:
                continue
            best, best_diff, prev = t, d, t
    if r["options"]["mash_fg_adjust"]:
        best = min(max(best, 144.0), 160.0)
        atten = atten + r["options"]["mash_fg_adj_slope"] * (best - tref)
    return atten


def recipe_fg(r):
    atten, tol = yeast_attenuation(r)
    atten = mash_adjusted_attenuation(r, atten)
    fgp = 0.0
    for g in r["fermentables"]:
        fgp += fg_points(g, r["efficiency_pct"], atten, r["options"]["steep_efficiency_pct"])
    v = og_volume(r)
    fgv = 1.0 if v <= 0 else 1 + fgp / (v / 128) / 1000
    og_ = recipe_og(r)
    if ABV(og_, fgv) > tol and tol > 0:
        fgv = fg_tol(og_, fgv, tol)
    return fgv


# --- §6.6 araç ekstraktları ---
def original_extract(start_sg): return ToPlato(start_sg)
def apparent_extract(final_sg): return ToPlato(final_sg)


# --- §7.3 kalori kullanımları (hepsinde önce OG = max(OG, FG)) ---
def cal_alcohol(og_, fg_): return alc_cal(max(og_, fg_), fg_)
def cal_carbs(og_, fg_): return ext_cal(max(og_, fg_), fg_)
def cal_total(og_, fg_): return cal_alcohol(og_, fg_) + cal_carbs(og_, fg_)
def carb_grams(carb_kcal): return carb_kcal / 4
def carb_calories(total_kcal, alc_kcal): return total_kcal - alc_kcal


# --- §8 renk ---
def recipe_color(r):
    """§8.1: Morey; tüm fermentable'lar, hacim = (F_R_VOLUME + F_R_TRUB_LOSS)/128 gal."""
    mcu_total = 0.0
    for g in r["fermentables"]:
        mcu_total += (g["amount_oz"] / 16) * g["color_srm"]
    v_gal = (r["volume_floz"] + r["equipment"]["trub_loss_floz"]) / 128
    if v_gal <= 0:
        return 0.0
    return morey(mcu_total, v_gal)


def mcu_from_srm(srm): return (0.67015 * srm) ** 1.45794                 # §8.2


def total_grains_oz(r):
    """F_R_TOT_GRAINS: fermentable miktarları toplamı (oz) (§11.5 belirsiz; varsayım)."""
    s = 0.0
    for g in r["fermentables"]:
        s += g["amount_oz"]
    return s


def color_flexibility(r, target):
    """§8.3 GetColorFlexibility -> (lo, hi, ok). Açarken 'koyu' = rengi hedefin üstünde olan."""
    cur = recipe_color(r)
    tot = total_grains_oz(r)
    vol = r["volume_floz"]
    if tot <= 0 or vol <= 0:
        return cur, cur, False
    lo, hi = 1000.0, cur
    for g in r["fermentables"]:
        c, a = g["color_srm"], g["amount_oz"]
        share = a / tot
        if target > cur:
            if c >= target and share * 100 >= g["max_in_batch_pct"] + 5:
                continue
        elif a == 0 and c > target:
            continue
        s = min(2 * share, 1.0)
        mcu = ((1 - s) * cur * tot + c * s * tot) / 16 / (vol / 128)
        srm = 1.4922 * mcu ** 0.6859
        lo, hi = min(lo, srm), max(hi, srm)
    if lo > hi:
        return 0.0, 0.0, False
    return lo, hi, lo != hi


def _grav_eq(rtype, eff, g, amount):
    y = g["yield_pct"] / 100
    if g["kind"] in FIXED_TYPES: return y * amount / 16
    if rtype == 0: return y * amount * 15 / 1600
    return y * amount * eff / 1600


def iterate_color(r, tt):
    """§8.3 IterateColor: (yeni reçete, yeni renk). OG eski değere ScaleGravity ile döndürülür."""
    r = copy.deepcopy(r)
    og0 = recipe_og(r)
    cur = recipe_color(r)
    lo, hi, _ = color_flexibility(r, tt)
    vg = (r["volume_floz"] + r["equipment"]["trub_loss_floz"]) / 128
    mc = mcu_from_srm(cur) * vg
    delta = mcu_from_srm(tt) * vg - mc
    w = total_grains_oz(r)
    rtype, eff = r["kind"], r["efficiency_pct"]
    dp, wb, base = 0.0, 0.0, []
    for g in r["fermentables"]:
        c, a = g["color_srm"], g["amount_oz"]
        m = a * c / 16
        if (tt <= c and lo < c) or hi <= c:
            if c > 0 and mc > 0:
                amax = (g["max_in_batch_pct"] + 15) / 100 * w
                new = (m + (m / mc) * delta) * 16 / c
                new = min(max(new, 0.0), amax)
                dp += _grav_eq(rtype, eff, g, new - a)
                g["amount_oz"] = new
        else:
            wb += a
            base.append(g)
    if wb > 0 and dp != 0:
        for g in base:
            c, a = g["color_srm"], g["amount_oz"]
            if c < hi and (c <= tt or c <= lo) and g["yield_pct"] != 0:
                e = _grav_eq(rtype, eff, g, a)
                if e <= 0:
                    continue
                new = 0.0 if e <= dp else a * (1 - a * (dp / e) / wb)
                amax = (g["max_in_batch_pct"] + 15) / 100 * w
                if rtype == 1 and g["kind"] == 0:
                    amax = max(amax, 0.25 * w)
                g["amount_oz"] = min(max(new, 0.0), amax)
    scaled = scale_gravity(r, og0)
    if scaled is not None:
        r = scaled
    return r, recipe_color(r)


def adjust_color(r, target):
    """§8.3 AdjustColor (IBU koruması yok): (reçete, başarı). En fazla 40 iterasyon."""
    cur = copy.deepcopy(r)
    for _ in range(40):
        if not color_flexibility(cur, target)[2]:
            break
        prev = recipe_color(cur)
        cur, new = iterate_color(cur, target)
        if abs(new - prev) < 0.005:
            break
        if abs(new - target) < 0.049:
            break
    return cur, abs(recipe_color(cur) - target) < 0.049


def color_adj_range(r):
    """§8.3 GetColorAdjRange: (min, max) renk; min != max ise ayar mümkün."""
    c1 = recipe_color(adjust_color(r, 1.0)[0])
    c2 = recipe_color(adjust_color(r, 150.0)[0])
    return min(c1, c2), max(c1, c2)


# --- Satır içi hesaplarla uyum denetimi (yukarıda basılan sayılar) ---
ORNEK_RENK = make_recipe([ferm("2-row", 160, 80, 2, 0, 0), ferm("Crystal 60", 16, 72, 60, 0, 0)],
                         volume_floz=640, trub_loss_floz=128)
ORNEK_VERIM = make_recipe([ferm("2-row", 160, 80, 2, 0, 0), ferm("Corn sugar", 16, 100, 0, 2, 2)])
assert abs(sum(og_points(g, 72) for g in ORNEK_AG["fermentables"]) - pts) < 1e-9
assert abs(recipe_og(ORNEK_AG) - og) < 1e-12
assert abs(mash_adjusted_attenuation(ORNEK_AG, 75.0) - att) < 1e-12
assert abs(recipe_fg(ORNEK_AG) - fg) < 1e-12
assert abs(recipe_color(ORNEK_RENK) - morey(80, 6)) < 1e-12
assert abs(get_efficiency(ORNEK_VERIM, 1.050, 704, False) - meas / pot * 100) < 1e-9
print("T1.14e: OG=%.4f FG=%.4f att=%.3f SRM=%.3f Eff=%.2f%% (satır içi sayılarla aynı)"
      % (recipe_og(ORNEK_AG), recipe_fg(ORNEK_AG), mash_adjusted_attenuation(ORNEK_AG, 75.0),
         recipe_color(ORNEK_RENK), get_efficiency(ORNEK_VERIM, 1.050, 704, False)))
print("T1.14e: SRM8.819->EBC %.3f, 30L->SRM %.3f, 1050->°P %.4f, 77F->C %.2f"
      % (bconv_color(8.819, 29, 30), bconv_color(30, 47, 29), bconv_hydrometer(1050, 3, 4), bconv_temperature(77, 2, 1)))


# ======================================================================================
# T1.15: eksik spesifikasyon kalemleri (§6.3 üç parametreli ABV, §9.2 refraktometre ABV/ABW,
# §4.2 verim sarmalayıcıları, §8.3 DoAdjustColor ön koşulları) ve §12 çapası. Yalnız ekleme.
# ======================================================================================
def calc_abv3(og, fg):
    """§6.3 üç parametreli CalcABV(OG, FG, &abw): OE = max(OE, AE) kırpması YOK. (abv, abw) döner."""
    oe = ToPlato(og); ae = ToPlato(fg)
    re = 0.1808 * oe + 0.8192 * ae
    abw = (oe - re) / (2.0665 - 0.010665 * oe)
    return fg / 0.79661 * abw, abw


def refr_corrected(kind, gravity, brix, zero=0.0, wcf=1.02):
    """§9.2: kind 0 -> (None); 1 -> (OG=gravity, FG=düzeltilmiş); 2 -> (OG=düzeltilmiş, FG=gravity)."""
    if kind == 1:
        return gravity, refr1(gravity, brix, zero)
    if kind == 2:
        return refr2(gravity, brix, zero)[0], gravity
    return None


def refractometer_abv(kind, gravity, brix, zero=0.0, wcf=1.02):
    """§9.2 RefractometerABV: tip 0 -> 0; tip 1 ve 2 -> CalcABV (kırpmalı, 0.79661)."""
    c = refr_corrected(kind, gravity, brix, zero, wcf)
    return 0.0 if c is None else ABV(c[0], c[1])


def refractometer_abw(kind, gravity, brix, zero=0.0, wcf=1.02):
    """§9.2 RefractometerABW: tip 0 -> 0; tip 1 ve 2 -> CalcABW (kırpmalı)."""
    c = refr_corrected(kind, gravity, brix, zero, wcf)
    return 0.0 if c is None else ABW(c[0], c[1])


def refractometer_finished_abv(fg, brix, zero=0.0):
    """§9.2 tip 2 ara değeri: ABV ~ ABW*FG/0.794 (0.79661 değil). ABW yüzde."""
    return refr2(fg, brix, zero)[1] * fg / 0.794


def recipe_est_mash_efficiency(r):
    """§4.2: tahmini mash verimi: OG = F_R_OG_MASH (§3.5), hacim = F_R_BOIL_VOL, mash_only."""
    return get_efficiency(r, est_post_mash_og(r), r["boil_vol_floz"], True)


def recipe_act_mash_efficiency(r, og_mash_measured, boil_vol_measured):
    """§4.2: ölçülen mash verimi."""
    return get_efficiency(r, og_mash_measured, boil_vol_measured, True)


def recipe_actual_efficiency(r, og_measured, volume_measured):
    """§4.2: ölçülen brewhouse verimi."""
    return get_efficiency(r, og_measured, volume_measured, False)


def do_adjust_color(r, target):
    """§8.3 DoAdjustColor ön koşulları: (durum, reçete, başarı). Tip >= 3 -> 'not_applicable';
    hedef [min, max] dışında -> 'out_of_range' (uçlar içeride); aksi 'adjusted'. [seçim: uçlar dahil]"""
    if r["kind"] >= 3:
        return "not_applicable", r, False
    lo, hi = color_adj_range(r)
    if target < lo or target > hi:
        return "out_of_range", r, False
    nr, ok = adjust_color(r, target)
    return "adjusted", nr, ok


# --- §12 çapası: spesifikasyondaki yuvarlanmış değerler referansla uyuşmalı (A7) ---
def _r(x, n): return round(x, n)
assert _r(ToPlato(1.050), 4) == 12.3876 and _r(ToSG(12.0), 5) == 1.04838 and _r(BrixToSG(12.0), 5) == 1.04838
assert _r(SGToBrix(1.050), 4) == 12.3870 and _r(BrixToRI(10.0), 6) == 1.347871 and _r(RIToBrix(BrixToRI(10.0)), 4) == 10.0
assert _r(bconv_color(morey(80, 6), 29, 30), 3) == 17.374 and _r(bconv_color(8.819, 29, 30), 3) == 17.373
for _og, _fg, _abw, _abv, _aa, _ra, _re, _ka, _ke, _kt, _uk in (
        (1.050, 1.010, 4.162, 5.276, 79.33, 64.99, 4.337, 104.8, 60.4, 165.2, 5.200),
        (1.065, 1.012, 5.543, 7.042, 80.71, 66.11, 5.389, 142.1, 76.1, 218.2, 6.943),
        (1.080, 1.016, 6.717, 8.566, 78.90, 64.64, 6.836, 176.0, 98.0, 274.0, 8.448)):
    assert (_r(ABW(_og, _fg), 3), _r(ABV(_og, _fg), 3), _r(AA(_og, _fg), 2), _r(RA(_og, _fg), 2),
            _r(RE(_og, _fg), 3)) == (_abw, _abv, _aa, _ra, _re), (_og, _fg)
    assert (_r(alc_cal(_og, _fg), 1), _r(ext_cal(_og, _fg), 1), _r(cal_total(_og, _fg), 1)) == (_ka, _ke, _kt)
    assert _r(uk(_og, _fg), 3) == _uk
assert _r(recipe_og(ORNEK_AG), 4) == 1.0609 and _r(recipe_fg(ORNEK_AG), 4) == 1.0102
assert _r(ABV(recipe_og(ORNEK_AG), recipe_fg(ORNEK_AG)), 3) == 6.717
assert _r(fg_tol(1.100, 1.010, 10), 4) == 1.0259 and _r(ABV(1.100, fg_tol(1.100, 1.010, 10)), 3) == 10.016
assert _r(recipe_color(ORNEK_RENK), 3) == 8.819 and _r(morey(80, 6), 3) == 8.819
assert _r(hydro(1.050, 77, 60), 5) == 1.05200 and _r(refr0(12.0), 5) == 1.04739
assert _r(refr1(1.050, 6.5), 5) == 1.01061 and _r(ABV(1.050, refr1(1.050, 6.5)), 3) == 5.196
_o, _w, _re, _oe = refr2(1.010, 6.5)
assert (_r(_o, 5), _r(_w, 3), _r(_re, 3), _r(_oe, 3)) == (1.05137, 4.257, 4.491, 12.711)
assert (_r(gadj_oz(1.040, 1.050, 640, 0), 3), _r(gadj_oz(1.040, 1.050, 640, 1), 3)) == (18.182, 22.222)
assert _r(gadj_water(1.060, 1.050, 640), 6) == 128.0
assert _r(get_efficiency(ORNEK_VERIM, 1.050, 704, False), 2) == 62.23
assert abs(calc_abv3(1.050, 1.010)[0] - ABV(1.050, 1.010)) < 1e-12 and calc_abv3(1.010, 1.020)[0] < 0
