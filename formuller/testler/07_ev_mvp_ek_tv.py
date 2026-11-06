"""Ev MVP ek hesapları: referans uygulama ve test vektörleri.
Çalıştır: python3 -I testler/07_ev_mvp_ek_tv.py   (yalnız standart kütüphane)
Birimler: °C, L, kg, g, dk, saat; SG yoğunluk."""
import math

def r(x, n=3): return round(x, n)

# ---------------------------------------------------------------- 1. Fermantasyon tahmini
def hourly_median(points):
    """points: [(t_saat, SG)] -> saatlik medyan (gürültüyü bastırır)."""
    b = {}
    for t, g in points: b.setdefault(int(t), []).append(g)
    out = []
    for h in sorted(b):
        v = sorted(b[h]); n = len(v)
        out.append((h + 0.5, v[n//2] if n % 2 else (v[n//2-1] + v[n//2]) / 2))
    return out

def _linfit(xs, ys):
    n = len(xs); mx = sum(xs)/n; my = sum(ys)/n
    sxx = sum((x-mx)**2 for x in xs); sxy = sum((x-mx)*(y-my) for x, y in zip(xs, ys))
    b = sxy/sxx; return my - b*mx, b

def fit_decay(pts, fg_lo, fg_hi, steps=400):
    """SG(t) = FGinf + A*exp(-k t). FGinf ızgarada aranır; her FGinf için ln(SG-FGinf) doğrusal."""
    best = None; table = []
    for i in range(steps + 1):
        f = fg_lo + (fg_hi - fg_lo) * i / steps
        if any(g - f <= 0 for _, g in pts): continue
        a, b = _linfit([t for t, _ in pts], [math.log(g - f) for _, g in pts])
        k = -b
        if k <= 0: continue
        A = math.exp(a)
        rss = sum((g - (f + A*math.exp(-k*t)))**2 for t, g in pts)
        table.append((f, rss))
        if best is None or rss < best[3]: best = (f, A, k, rss)
    return best, table

def forecast(points, og, fg_recipe, eps=0.0005, min_n=12, min_drop=0.30):
    pts = hourly_median(points)
    sg_now = pts[-1][1]
    if len(pts) < min_n or (og - sg_now) < min_drop * (og - fg_recipe):
        return {"durum": "veri yetersiz"}
    fg_lo, fg_hi = fg_recipe - 0.015, sg_now - 0.0002
    (f, A, k, rss), table = fit_decay(pts, fg_lo, fg_hi)
    n = len(pts)
    lim = rss * (1 + 4.0/(n - 2))                       # ≈ %95 profil aralığı (F≈4)
    ok = [x for x, s in table if s <= lim]
    t_end = math.log(A/eps)/k                           # SG - FGinf ≤ eps olduğu an
    warn = abs(f - fg_recipe) > 0.010
    return {"FG": r(f, 4), "aralik": (r(min(ok), 4), r(max(ok), 4)), "k_saat": r(k, 4),
            "bitis_saat": r(t_end, 1), "kalan_saat": r(t_end - pts[-1][0], 1), "uyari": warn}

# ---------------------------------------------------------------- 2. Cihaz kalibrasyonu
def polyfit(xs, ys, deg):
    """En küçük kareler; normal denklemler + Gauss eleme."""
    m = deg + 1
    A = [[sum(x**(i+j) for x in xs) for j in range(m)] for i in range(m)]
    B = [sum(y * x**i for x, y in zip(xs, ys)) for i in range(m)]
    for c in range(m):
        p = max(range(c, m), key=lambda r_: abs(A[r_][c])); A[c], A[p] = A[p], A[c]; B[c], B[p] = B[p], B[c]
        for r_ in range(c+1, m):
            f = A[r_][c]/A[c][c]
            for j in range(c, m): A[r_][j] -= f*A[c][j]
            B[r_] -= f*B[c]
    co = [0.0]*m
    for i in range(m-1, -1, -1):
        co[i] = (B[i] - sum(A[i][j]*co[j] for j in range(i+1, m)))/A[i][i]
    return co                                           # c0 + c1 x + c2 x² ...

def peval(co, x): return sum(c * x**i for i, c in enumerate(co))

def ispindel_cal(points, deg=2):
    """points: [(açı°, SG_referans)]; en az deg+2 nokta."""
    if len(points) < deg + 2: raise ValueError("en az %d nokta" % (deg + 2))
    co = polyfit([a for a, _ in points], [g for _, g in points], deg)
    res = [g - peval(co, a) for a, g in points]
    my = sum(g for _, g in points)/len(points)
    r2 = 1 - sum(x*x for x in res)/sum((g-my)**2 for _, g in points)
    return co, r2, max(abs(x) for x in res)

def tilt_cal(points):
    """points: [(SG_tilt, SG_referans)]. 1 nokta: sabit sapma; 2+: doğrusal."""
    if len(points) == 1:
        return 1.0, points[0][1] - points[0][0]
    a, b = _linfit([x for x, _ in points], [y for _, y in points])
    return b, a                                         # SG_düz = egim*SG_tilt + sapma

# ---------------------------------------------------------------- 3. Isınma ve soğuma
C_WATER, C_GRAIN, C_STEEL = 4.186, 1.59, 0.50          # kJ/(kg·K); tahıl = 0.38 cal/g°C (05 ile aynı)
H_VAP = 2257.0                                          # kJ/kg

def heat_time_min(water_L, grain_kg, kettle_kg, T0, T1, power_kW, eff):
    C = water_L*C_WATER + grain_kg*C_GRAIN + kettle_kg*C_STEEL      # kJ/K (su yoğunluğu 1 kabul)
    return C*(T1 - T0)/(power_kW*eff)/60.0

def boiloff_L_per_h(power_kW, eff_boil):
    return power_kW*eff_boil*3600.0/H_VAP

def tau_from_batch(T0, Tc, T, minutes):
    """Daldırma soğutucu: T(t) = Tc + (T0-Tc)e^(-t/τ). Geçmiş partiden τ."""
    return minutes/math.log((T0 - Tc)/(T - Tc))

def cool_time_immersion(T0, Tc, T, tau, vol_L, vol_ref_L):
    tau_v = tau * vol_L/vol_ref_L                       # τ hacimle doğrusal ölçeklenir (yaklaşık)
    return tau_v*math.log((T0 - Tc)/(T - Tc))

def cool_time_flow(vol_L, flow_L_min): return vol_L/flow_L_min

# ---------------------------------------------------------------- 4. Ekipmana göre ölçekleme
def scale_recipe(ing, V0, V1, eff0, eff1, mode="hedef"):
    """ing: dict ad -> (miktar, tip). tip: 'mash','seker','hop_ibu','hop_aroma','tuz','misc','maya_paket'.
    mode 'oran': hepsi V1/V0. mode 'hedef': mash eff oranıyla; IBU hopları burada yalnız ilk tahmin (V oranı),
    ardından ScaleBitterness (02 §6.1) hedef ekipmanla IBU'yu tam tutturur."""
    rv = V1/V0; out = {}
    for k, (q, t) in ing.items():
        if mode == "oran": f = rv
        elif t == "mash": f = rv*eff0/eff1
        else: f = rv
        out[k] = q*f
    return out

def round_to(q, step): return round(q/step)*step

# ---------------------------------------------------------------- 5. İyon dengesi ve sertlik
EQW = {"Ca": 20.039, "Mg": 12.153, "Na": 22.990, "K": 39.098,
       "SO4": 48.031, "Cl": 35.453, "HCO3": 61.017, "CO3": 30.004, "NO3": 62.004}
CAT = ("Ca", "Mg", "Na", "K"); AN = ("SO4", "Cl", "HCO3", "CO3", "NO3")

def ion_balance(p):
    c = sum(p.get(i, 0)/EQW[i] for i in CAT); a = sum(p.get(i, 0)/EQW[i] for i in AN)
    b = 100*(c - a)/(c + a)
    s = "iyi" if abs(b) <= 5 else ("kabul" if abs(b) <= 10 else "kontrol et")
    return r(c, 3), r(a, 3), r(b, 2), s

def hardness_caco3(Ca, Mg): return Ca*50.04/EQW["Ca"] + Mg*50.04/EQW["Mg"]
def alk_caco3_from_hco3(hco3): return hco3*50.04/EQW["HCO3"]
def caco3_to(x): return {"dH": x/17.848, "fH": x/10.009, "mEq/L": x/50.04}

# ---------------------------------------------------------------- 6. Hop ve meyve kayıpları
HOP_ABS = {"pellet": 0.0070, "yaprak": 0.0130, "cryo": 0.0045}   # L/g  [varsayılan, doğrulanacak]
def kettle_hop_loss(g_by_form): return sum(HOP_ABS[f]*g for f, g in g_by_form.items())
def dryhop_loss(g_by_form): return sum(HOP_ABS[f]*g for f, g in g_by_form.items())
def fruit(kg, density=1.05, sediment=0.20):
    v = kg/density; return v, v*sediment                 # eklenen hacim, tortuda kalan

# ================================================================ TEST VEKTÖRLERİ
if __name__ == "__main__":
    print("== 1 Fermantasyon tahmini")
    og, fg_true, k_true = 1.052, 1.0110, 0.045
    lag = 12
    pts = []
    for i in range(0, 96*4):                            # 15 dk'da bir, 96 saat, deterministik gürültü
        t = i/4
        sg = og if t < lag else fg_true + (og - fg_true)*math.exp(-k_true*(t - lag))
        noise = 0.0004*math.sin(i*1.7) + 0.0002*math.sin(i*0.31)
        pts.append((t, sg + noise))
    act = [p for p in pts if p[0] >= 24]                # aktif fazdan sonrası
    print("TV1.1 72 saat veri:", forecast([p for p in act if p[0] < 72], og, 1.012))
    print("TV1.2 30 saat veri:", forecast([p for p in act if p[0] < 30], og, 1.012))
    print("TV1.3 gerçek: FG", fg_true, "bitiş saati", r(lag + math.log((og-fg_true)/0.0005)/k_true, 1))

    print("== 2 Kalibrasyon")
    cal = [(25.0, 1.000), (38.2, 1.020), (49.0, 1.040), (58.7, 1.060), (66.5, 1.080)]
    co, r2, mx = ispindel_cal(cal, 2)
    print("TV2.1 iSpindel katsayılar", [round(c, 8) for c in co], "R²", r(r2, 5), "max hata", r(mx, 4))
    print("TV2.2 açı 45° ->", r(peval(co, 45.0), 4))
    print("TV2.3 Tilt tek nokta (1.0545 vs 1.0520):", tuple(r(x, 4) for x in tilt_cal([(1.0545, 1.0520)])))
    print("TV2.4 Tilt iki nokta:", tuple(r(x, 4) for x in tilt_cal([(1.0020, 1.0000), (1.0545, 1.0520)])))

    print("== 3 Isınma ve soğuma")
    print("TV3.1 strike: 15 L 15→72 °C, 8 kg kazan, 2.5 kW, η .90 ->", r(heat_time_min(15, 0, 8, 15, 72, 2.5, 0.90), 1), "dk")
    print("TV3.2 mash adımı 64→72, 15 L + 5 kg tahıl ->", r(heat_time_min(15, 5, 8, 64, 72, 2.5, 0.90), 1), "dk")
    print("TV3.3 kaynamaya 28 L 76→100 ->", r(heat_time_min(28, 0, 8, 76, 100, 2.5, 0.90), 1), "dk")
    print("TV3.4 boil-off 2.5 kW, η .85 ->", r(boiloff_L_per_h(2.5, 0.85), 2), "L/saat")
    tau = tau_from_batch(100, 15, 20, 25)
    print("TV3.5 τ (24 L, 100→20 °C, şebeke 15 °C, 25 dk) ->", r(tau, 2), "dk")
    print("TV3.6 aynı soğutucu 40 L, şebeke 18 °C ->", r(cool_time_immersion(100, 18, 20, tau, 40, 24), 1), "dk")
    print("TV3.7 plakalı 23 L, 4 L/dk ->", r(cool_time_flow(23, 4), 2), "dk")

    print("== 4 Ölçekleme")
    ing = {"Pale Ale malt": (4.50, "mash"), "Munich": (0.45, "mash"), "Crystal 40": (0.30, "mash"),
           "Magnum": (19, "hop_ibu"), "Cascade": (30, "hop_ibu"), "Citra wp": (50, "hop_aroma"),
           "Gypsum": (5.0, "tuz"), "Dekstroz": (0.25, "seker")}
    out = scale_recipe(ing, 23, 40, 0.72, 0.78)
    print("TV4.1 hedef modu:", {k: r(v, 3) for k, v in out.items()})
    print("TV4.2 yuvarlanmış: malt", r(round_to(out["Pale Ale malt"], 0.01), 2), "kg, Magnum", round_to(out["Magnum"], 1), "g")
    print("TV4.3 oran modu Pale:", r(scale_recipe(ing, 23, 40, .72, .78, "oran")["Pale Ale malt"], 3))

    print("== 5 Su")
    w = {"Ca": 38, "Mg": 8, "Na": 18, "SO4": 32, "Cl": 40, "HCO3": 110}
    print("TV5.1 denge (katyon, anyon, %, sınıf):", ion_balance(w))
    h = hardness_caco3(38, 8); a = alk_caco3_from_hco3(110)
    print("TV5.2 sertlik CaCO3", r(h, 1), {k: r(v, 2) for k, v in caco3_to(h).items()})
    print("TV5.3 alkalinite CaCO3", r(a, 1), {k: r(v, 2) for k, v in caco3_to(a).items()})

    print("== 6 Kayıplar")
    print("TV6.1 kazan: Magnum 19 + Cascade 30 pellet ->", r(kettle_hop_loss({"pellet": 49}), 3), "L")
    print("TV6.2 dry hop 60 g pellet + 20 g cryo ->", r(dryhop_loss({"pellet": 60, "cryo": 20}), 3), "L")
    v, s = fruit(2.0)
    print("TV6.3 2 kg meyve püresi: eklenen", r(v, 3), "L, tortuda", r(s, 3), "L")
