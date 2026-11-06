"""İşletme 2. sürüm hesapları: etiket enerjisi, vergi/beyan, spunding.
Çalıştır: python3 -I testler/09_isletme_2_tv.py   (yalnız standart kütüphane)
Vergi oranları ÖRNEKTİR; kaynak ve tarih spesifikasyonda (09 §2.4). Gerçek oranlar veri tablosundan gelir."""
import math

def r(x, n=3): return round(x, n)
def to_plato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868      # 01 §1.1
def to_sg(P): return 1 + P/(258.6 - (P/258.6)*227.1)                              # 01 §1.2
def real_extract(OE, AE): return 0.1808*OE + 0.8192*AE                            # 01 §6.1
def abw(OE, RE): return (OE - RE)/(2.0665 - 0.010665*OE)                          # 01 §6.2
def abv_from(og, fg):
    OE, AE = to_plato(og), to_plato(fg); RE = real_extract(OE, AE)
    w = abw(OE, RE); return w*fg/0.79661, w, RE                                    # 01 §6.3

# ---------------------------------------------------------------- 1. Etiket enerjisi (AB 1169/2011 Ek XIV)
KJ = {"alkol": 29, "karbonhidrat": 17, "protein": 17, "yag": 37, "organik_asit": 13, "poliol": 10, "lif": 8}
KCAL = {"alkol": 7, "karbonhidrat": 4, "protein": 4, "yag": 9, "organik_asit": 3, "poliol": 2.4, "lif": 2}
ETOH_DENS = 0.78924          # g/mL, 20 °C [kesin]

def label_energy(og, fg, protein_g100mL=0.40, abv_measured=None):
    abv, _, RE = abv_from(og, fg)
    if abv_measured is not None: abv = abv_measured
    alc_g = abv*ETOH_DENS                                  # g/100 mL
    re_g = RE*fg                                           # g/100 g × g/mL -> g/100 mL
    carb = max(re_g - protein_g100mL, 0.0)
    kj = KJ["alkol"]*alc_g + KJ["karbonhidrat"]*carb + KJ["protein"]*protein_g100mL
    kcal = KCAL["alkol"]*alc_g + KCAL["karbonhidrat"]*carb + KCAL["protein"]*protein_g100mL
    return {"ABV": r(abv, 2), "alkol_g": r(alc_g, 2), "gercek_ekstrakt_g": r(re_g, 2), "karbonhidrat_g": r(carb, 1),
            "protein_g": r(protein_g100mL, 1), "kJ": round(kj), "kcal": round(kcal)}

# ---------------------------------------------------------------- 2. Vergi motoru (yöntemler veri, oranlar veri)
HL_PER_BBL = 1.17348         # 31 US gal [kesin]

def tax_line(method, V_L, abv=None, plato=None, rate=None, bands=None, tiers=None, ytd=0.0, plato_trunc=True,
             discount_per_lpa=0.0):
    """method:
      'hacim'        : oran × hacim birimi (rate = PB/hL)
      'saf_alkol'    : oran × litre saf alkol (LPA = V × ABV/100); bands=[(abv_üst, oran)] varsa ABV bandından oran
      'hl_plato'     : oran × hL × °P (°P tam sayıya kesilebilir)
      'plato_bandi'  : bands=[(plato_üst, PB/hL)]
      'kademeli_hacim': tiers=[(yıllık_üst_hL, PB/hL)], ytd = bu yıl daha önce vergilenen hL"""
    hl = V_L/100.0
    if method == "hacim":
        return rate*hl
    if method == "saf_alkol":
        lpa = V_L*abv/100.0
        rt = rate
        if bands:
            rt = next(b for u, b in bands if abv < u)
        return lpa*(rt - discount_per_lpa)
    if method == "hl_plato":
        p = math.floor(plato) if plato_trunc else plato
        return rate*hl*p
    if method == "plato_bandi":
        rt = next(b for u, b in bands if plato <= u)
        return rt*hl
    if method == "kademeli_hacim":
        tax, start, left = 0.0, ytd, hl
        for upper, rt in tiers:
            if left <= 0: break
            room = max(upper - start, 0.0)
            take = min(room, left)
            tax += take*rt; start += take; left -= take
        return tax
    raise ValueError(method)

# ---------------------------------------------------------------- 3. Spunding
CO2_G_PER_VOL = 1.977         # g/L, 0 °C 1 atm [kesin]
BALLING_CO2 = 0.9565/2.0665   # g CO2 / g fermente ekstrakt [Balling]

def residual_co2(T_C):                                     # 04 §3.2
    T = max(T_C*9/5 + 32, 32); return 3.0378 - 0.050062*T + 0.00026555*T*T

def co2_pressure_psi(T_C, vol):                            # 04 §3.5
    T = max(T_C*9/5 + 32, 32)
    return max(-16.6999 - 0.0101059*T + 0.00116512*T*T + 0.173354*T*vol + 4.24267*vol - 0.0684226*vol*vol, 0.0)

def spund(target_vol, T_C, fg_expected):
    need_vol = target_vol - residual_co2(T_C)
    need_g = max(need_vol, 0)*CO2_G_PER_VOL
    sg = fg_expected
    dRE_gL = need_g/BALLING_CO2                           # fermente olması gereken gerçek ekstrakt (g/L)
    dRE_P = dRE_gL/(10*sg)                                 # °P (g/100 g)
    dAE_P = dRE_P/0.8192                                   # görünür ekstrakt düşüşü
    close_P = to_plato(fg_expected) + dAE_P
    psi = co2_pressure_psi(T_C, target_vol)
    return {"kalinti_vol": r(residual_co2(T_C), 2), "gereken_g_L": r(need_g, 2), "kapama_SG": r(to_sg(close_P), 4),
            "puan_once": r((to_sg(close_P) - fg_expected)*1000, 1), "ayar_psi": r(psi, 1), "ayar_bar": r(psi*0.0689476, 2)}

# ================================================================ TEST VEKTÖRLERİ
if __name__ == "__main__":
    print("== 1 Etiket enerjisi")
    print("TV1.1 Pilsner OG 1.048 FG 1.009:", label_energy(1.048, 1.009))
    print("TV1.2 Citra PA OG 1.052 FG 1.011:", label_energy(1.052, 1.011))
    print("TV1.3 ölçülen ABV %5.0 ile Pilsner:", label_energy(1.048, 1.009, abv_measured=5.0))

    print("== 2 Vergi (örnek oranlar, 20 hL, %5.0, 12 °P)")
    V, A, P = 2000.0, 5.0, 12.0
    print("TV2.1 saf alkol, 21.78 PB/LPA (örnek):", r(tax_line("saf_alkol", V, abv=A, rate=21.78), 2))
    print("TV2.2 saf alkol, ABV bandı [(3.5, 9.27), (8.5, 21.78), (99, 29.54)] + indirim 5.0/LPA:",
          r(tax_line("saf_alkol", V, abv=A, rate=None, bands=[(3.5, 9.27), (8.5, 21.78), (99, 29.54)], discount_per_lpa=5.0), 2))
    print("TV2.3 hL·°P, 0.787 PB (12.6 °P kesilir -> 12):", r(tax_line("hl_plato", V, plato=12.6, rate=0.787), 2))
    print("TV2.4 Plato bandı [(10.0,16.88),(14.0,25.32),(99,33.76)] PB/hL:", r(tax_line("plato_bandi", V, plato=P, bands=[(10.0, 16.88), (14.0, 25.32), (99, 33.76)]), 2))
    tiers = [(60000*HL_PER_BBL, 3.50/HL_PER_BBL), (2000000*HL_PER_BBL, 16.0/HL_PER_BBL)]
    print("TV2.5 kademeli (60 000 bbl @3.50, sonra @16 /bbl), 20 hL, ytd 0:", r(tax_line("kademeli_hacim", V, tiers=tiers), 2))
    print("TV2.6 aynı, eşiği aşan: ytd = 60000 bbl − 10 hL:", r(tax_line("kademeli_hacim", V, tiers=tiers, ytd=60000*HL_PER_BBL - 10), 2))
    print("TV2.7 hacim başına, 12.49 PB/L (örnek):", r(tax_line("hacim", V, rate=1249.0), 2))

    print("== 3 Spunding")
    print("TV3.1 hedef 2.5 vol, 12 °C, FG 1.010:", spund(2.5, 12.0, 1.010))
    print("TV3.2 hedef 2.6 vol, 18 °C, FG 1.012:", spund(2.6, 18.0, 1.012))
