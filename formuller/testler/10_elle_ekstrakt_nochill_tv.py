"""10: az okumayla fermantasyon tahmini, ekstrakt araçları, soğutmasız (no-chill) IBU.
Referans uygulama ve test vektörleri.
Çalıştır: python3 -I testler/10_elle_ekstrakt_nochill_tv.py   (yalnız standart kütüphane)
Birimler: °C, L, g, kg, dk, saat; SG yoğunluk."""
import math

def r(x, n=3): return round(x, n)

# ================================================================ 1. Az okumayla fermantasyon tahmini
LAG_H = 12.0          # gecikme fazı varsayılanı, saat            [doğrulanacak]
SIGMA_FG = 0.003      # FG ön tahmininin belirsizliği (±)          [doğrulanacak]
EPS = 0.0005          # 07 §1.5 ile aynı

def k_fit(readings, og, fg, lag=LAG_H):
    """FG sabit (ön tahmin), OG t=0'da bilinir. y = ln((SG-FG)/(OG-FG)) = -k (t-lag).
    Orijinden geçen en küçük kareler: k = -Σxy/Σx²."""
    xs, ys = [], []
    for t, sg in readings:
        x = t - lag
        if x <= 0 or sg - fg <= EPS: continue
        xs.append(x); ys.append(math.log((sg - fg) / (og - fg)))
    if not xs: return None, []
    sxx = sum(x * x for x in xs)
    k = -sum(x * y for x, y in zip(xs, ys)) / sxx
    k_each = [-y / x for x, y in zip(xs, ys)]
    return k, k_each

def t_end(og, fg, k, lag=LAG_H):
    return lag + math.log((og - fg) / EPS) / k

def few_forecast(readings, og, fg_prior, lag=LAG_H, sigma=SIGMA_FG):
    """readings: [(t_saat, SG)] elle/kağıt okumalar (t parti başlangıcından). Kurallar §1.3–§1.5."""
    readings = sorted(readings)
    post = [(t, g) for t, g in readings if t > lag]
    sg_now = readings[-1][1]; t_now = readings[-1][0]
    # durgunluk kontrolü (§1.5): son iki okuma ≥48 saat arayla ve ≤0.001 fark
    if len(readings) >= 2:
        (t1, g1), (t2, g2) = readings[-2], readings[-1]
        if t2 - t1 >= 48 and abs(g1 - g2) <= 0.001:
            if g2 <= fg_prior + 0.004: return {"durum": "bitmiş görünüyor", "FG_olculen": g2}
            return {"durum": "takılmış olabilir", "SG": g2, "FG_beklenen": fg_prior}
    if len(post) < 2:
        return {"durum": "veri yetersiz", "neden": "gecikme sonrası en az 2 okuma gerekir"}
    if (og - sg_now) < 0.30 * (og - fg_prior):
        return {"durum": "veri yetersiz", "neden": "beklenen düşüşün %30'u tamamlanmadı"}
    if sg_now <= fg_prior + EPS:
        return {"durum": "hedefe ulaştı; durgunluk için 48 saat sonra tekrar ölç"}
    k, k_each = k_fit(post, og, fg_prior, lag)
    if k is None or k <= 0:
        return {"durum": "veri yetersiz", "neden": "k ≤ 0"}
    ends = []
    for fg in (fg_prior - sigma, fg_prior, fg_prior + sigma):
        if sg_now - fg <= EPS:            # bu FG'ye zaten ulaşılmış
            ends.append(t_now); continue
        kk, _ = k_fit(post, og, fg, lag)
        ends.append(t_end(og, fg, kk, lag) if kk and kk > 0 else float("inf"))
    tutarsiz = len(k_each) >= 2 and max(k_each) / min(k_each) > 2.0 if min(k_each) > 0 else True
    lo, mid, hi = min(ends), ends[1], max(ends)
    return {"k_saat": r(k, 4), "bitis_saat": r(mid, 1),
            "aralik_gun": (math.floor(lo / 24), math.ceil(hi / 24)),
            "kalan_gun": (max(0, math.floor((lo - t_now) / 24)), math.ceil((hi - t_now) / 24)),
            "tutarsiz": tutarsiz, "etiket": "yaklaşık"}

# ================================================================ 2. Ekstrakt biracısı araçları
LB_PER_KG = 2.20462262
GAL_PER_L = 0.264172052
PPG_TO_PKL = LB_PER_KG / GAL_PER_L     # PPG -> puan/(kg/L) = 8.3454

# tipik değerler; tarif veritabanındaki malzeme değeri önceliklidir          [doğrulanacak]
EXTRACT = {
    "LME": {"yield": 78.0, "yogunluk": 1.42},   # g/mL (kütle yoğunluğu)
    "DME": {"yield": 95.0, "yigin": 0.60},      # g/mL, yığın (gevşek doldurma) yoğunluğu
    "bal": {"yield": 75.0, "yogunluk": 1.42},
}

def ppg(y): return y * 46 / 100
def points_kg_L(y): return ppg(y) * PPG_TO_PKL                  # 1 kg / 1 L'nin verdiği puan

def g_to_ml(kind, g):
    e = EXTRACT[kind]; d = e.get("yogunluk") or e["yigin"]
    return g / d
def ml_to_g(kind, ml):
    e = EXTRACT[kind]; d = e.get("yogunluk") or e["yigin"]
    return ml * d

def swap(kind_from, kind_to, g):
    """Aynı puanı veren miktar: m2 = m1 · yield1 / yield2."""
    return g * EXTRACT[kind_from]["yield"] / EXTRACT[kind_to]["yield"]

def extract_for_og(kind, og, v_final_L):
    """Hedef OG için gereken ekstrakt (g). Ekstraktta verim (efficiency) %100 kabul edilir."""
    pts = (og - 1) * 1000
    return pts * v_final_L / points_kg_L(EXTRACT[kind]["yield"]) * 1000

def partial_boil(og_final, v_final, v_kettle_post):
    """Kısmi kaynatma: kazandaki (seyreltme öncesi) gravite ve eklenecek su."""
    pts_k = (og_final - 1) * 1000 * v_final / v_kettle_post
    return {"SG_kazan": r(1 + pts_k / 1000, 4), "ust_su_L": r(v_final - v_kettle_post, 2)}

def tinseth_util(sg, t): return 1.65 * 0.000125 ** (sg - 1) * (1 - math.exp(-0.04 * t)) / 4.15

def partial_boil_ibu(hops, sg_kettle, v_final):
    """hops: [(g, AA_kesir, dk)]. Kullanım kazandaki gravitede; mg/L son hacimde."""
    return sum(tinseth_util(sg_kettle, t) * aa * g * 1000 / v_final for g, aa, t in hops)

def topoff_temp(v_wort, T_wort, v_water, T_target, c_ratio=1.0):
    """Hedef karışım sıcaklığı için eklenecek suyun sıcaklığı. c_ratio: şıranın ısı kapasitesi/su
    (yaklaşık 1; yoğun şırada biraz düşük)."""
    return (T_target * (v_wort * c_ratio + v_water) - v_wort * c_ratio * T_wort) / v_water

# ================================================================ 3. Soğutmasız (no-chill) IBU
def ftemp(Tc): return 1.0 if Tc >= 100 else 2.39e11 * math.exp(-9773 / (Tc + 273.15))

TAU_20L_H = 4.0        # 20 L HDPE küp, durgun hava 20 °C, saat       [doğrulanacak]
def tau_h(v_L, tau20=TAU_20L_H): return tau20 * (v_L / 20.0) ** (1 / 3)

def t_eq_min(T0, T_amb, tau_hours, dt=0.5, Tstop=60.0):
    """Küpte soğuma T(t) = Tamb + (T0-Tamb)e^(-t/τ). Eşdeğer kaynatma dakikası = ∫ ftemp(T) dt.
    60 °C altı katkısı ihmal edilir (ftemp < 0.01)."""
    tau = tau_hours * 60; t = 0.0; s = 0.0
    while True:
        T = T_amb + (T0 - T_amb) * math.exp(-t / tau)
        if T < Tstop: break
        s += ftemp(T) * dt; t += dt
    return s

def nochill_ibu(hops, sg, v, mode="kural", T0=95.0, T_amb=20.0, v_cube=None):
    """mode 'kural': her katkıya +20 dk. mode 'model': + ∫ftemp(T)dt (eşdeğer dakika)."""
    if mode == "kural": extra = 20.0
    else: extra = t_eq_min(T0, T_amb, tau_h(v_cube or v))
    base = sum(tinseth_util(sg, t) * aa * g * 1000 / v for g, aa, t in hops)
    nc = sum(tinseth_util(sg, t + extra) * aa * g * 1000 / v for g, aa, t in hops)
    return r(base, 1), r(nc, 1), r(extra, 1)


if __name__ == "__main__":
    print("== 1. Az okumayla tahmin (OG 1.050, FG ön 1.012, gecikme 12 sa)")
    print("T1a iki okuma 48 sa 1.030, 96 sa 1.018:", few_forecast([(48, 1.030), (96, 1.018)], 1.050, 1.012))
    print("T1b tek okuma 48 sa 1.030:", few_forecast([(48, 1.030)], 1.050, 1.012))
    print("T1c erken okumalar 24 sa 1.046, 36 sa 1.042:", few_forecast([(24, 1.046), (36, 1.042)], 1.050, 1.012))
    print("T1d durgun, hedefte 168 sa 1.013, 216 sa 1.013:", few_forecast([(96, 1.018), (168, 1.013), (216, 1.013)], 1.050, 1.012))
    print("T1e durgun, yüksek 120 sa 1.024, 168 sa 1.024:", few_forecast([(48, 1.030), (120, 1.024), (168, 1.024)], 1.050, 1.012))
    print("T1f tutarsız 48 sa 1.020, 96 sa 1.019:", few_forecast([(48, 1.020), (96, 1.019)], 1.050, 1.012))

    print("\n== 2. Ekstrakt")
    print("T2a PPG→puan/(kg/L) katsayısı: %.4f" % PPG_TO_PKL)
    for k in ("LME", "DME", "bal"):
        print("T2b %s yield %.0f: PPG %.2f, puan/(kg/L) %.1f" % (k, EXTRACT[k]["yield"], ppg(EXTRACT[k]["yield"]), points_kg_L(EXTRACT[k]["yield"])))
    print("T2c 1500 g LME -> mL: %.1f ; 1 lb LME -> mL: %.1f" % (g_to_ml("LME", 1500), g_to_ml("LME", 453.592)))
    print("T2d 500 mL bal -> g: %.0f ; 1 US cup DME -> g: %.0f" % (ml_to_g("bal", 500), ml_to_g("DME", 236.588)))
    print("T2e 3000 g LME yerine DME: %.0f g" % swap("LME", "DME", 3000))
    print("T2f OG 1.050, 20 L, LME: %.0f g ; DME: %.0f g" % (extract_for_og("LME", 1.050, 20), extract_for_og("DME", 1.050, 20)))
    pb = partial_boil(1.050, 20, 10)
    print("T2g kısmi kaynatma OG 1.050, 20 L, kazanda 10 L:", pb)
    hops = [(28, 0.10, 60), (28, 0.10, 10)]
    print("T2h IBU kısmi (SG_kazan %.3f): %.1f ; tam hacim (SG 1.050): %.1f" %
          (pb["SG_kazan"], partial_boil_ibu(hops, pb["SG_kazan"], 20), partial_boil_ibu(hops, 1.050, 20)))
    print("T2i 10 L şıra 30 °C + 10 L su, hedef 20 °C -> su sıcaklığı %.1f °C" % topoff_temp(10, 30, 10, 20))

    print("\n== 3. No-chill (1.050, 20 L)")
    for tau in (3, 4, 6):
        print("T3a τ=%d sa, T0 95, ortam 20: eşdeğer %.1f dk" % (tau, t_eq_min(95, 20, tau)))
    print("T3b τ(40 L) = %.2f sa" % tau_h(40))
    hops = [(28, 0.10, 60), (28, 0.10, 0)]
    print("T3c kural  (60 dk + 0 dk, 28 g %10):", nochill_ibu(hops, 1.050, 20, "kural"))
    print("T3d model  (aynı, τ 4 sa, T0 95):", nochill_ibu(hops, 1.050, 20, "model"))
    print("T3e model, T0 80 (whirlpool sonrası):", nochill_ibu(hops, 1.050, 20, "model", T0=80))
