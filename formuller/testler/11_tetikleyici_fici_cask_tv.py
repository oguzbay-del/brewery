"""11: fermantasyon adımına gravite tetikleyicisi, fıçıda priming, cask (İngiltere).
Referans uygulama ve test vektörleri.
Çalıştır: python3 -I testler/11_tetikleyici_fici_cask_tv.py   (yalnız standart kütüphane)
Birimler: °C, L, g, saat; SG yoğunluk; basınç bar (gösterge) ve Pa (mutlak)."""
import math

def r(x, n=3): return round(x, n)

# Karşılaştırma ilkesi (00-OKU-BENI.md › Önemli genel notlar): gravite eşikleri tam sayı birimde, 1 birim = 0.00001 SG
# (0.01 puan): u(x) = round(x*100000). Sürekli büyüklüklerde (eğim, oran) açık tolerans TOL.
def u(sg): return round(sg * 100000)
TOL = 1e-9

# ---- 04'ten ortak fonksiyonlar (BeerSmith kalıntı CO2, fıçı basıncı; °F girdili)
def F(c): return c * 9 / 5 + 32
def residual_vol(T_C):
    T = max(F(T_C), 32.0); return 3.0378 - 0.050062 * T + 0.00026555 * T * T
def co2_psi(vols, T_C):
    T = max(F(T_C), 32.0)
    p = -16.6999 - 0.0101059 * T + 0.00116512 * T * T + 0.173354 * T * vols + 4.24267 * vols - 0.0684226 * vols * vols
    return max(p, 0.0)
def ToPlato(sg): return 135.997 * sg**3 - 630.272 * sg**2 + 1111.14 * sg - 616.868

# ================================================================ 1. Gravite tetikleyicisi
STABLE_PER_DAY = 0.0005   # [tasarım]

def trigger_index(points, fg_ref, delta, consecutive=3):
    """points: saatlik medyan [(t_saat, SG)]. SG ≤ FG_ref + Δ art arda 'consecutive' noktada sağlanınca
    tetiklenir; dönen değer, şartın ilk sağlandığı noktanın zamanı."""
    run = 0
    for i, (t, g) in enumerate(points):
        run = run + 1 if u(g) <= u(fg_ref) + u(delta) else 0
        if run >= consecutive: return points[i - consecutive + 1][0]
    return None

def predicted_trigger(fg_inf, A, k, fg_ref, delta):
    """07 §1 modeliyle tetik zamanı: SG(t) = FGinf + A e^(-kt) = FG_ref + Δ.
    Dönüş (durum, saat): ("tetik", t) | ("gecmis", 0.0) eşik zaten t=0'da sağlanıyor | ("erisilemez", None)
    eğri asimptotu (FG∞) eşiğin üstünde veya eşitinde, eşiğe hiç inilmez."""
    target_u = u(fg_ref) + u(delta) - u(fg_inf)
    if target_u <= 0: return ("erisilemez", None)
    if u(A) <= target_u: return ("gecmis", 0.0)
    return ("tetik", math.log(A / (target_u / 100000)) / k)

def fg_reference(fg_recipe, fg_inf=None, shown=False, delta=0.004):
    """Tahmin gösteriliyorsa ve FG∞ eşiğin (FG_tarif + Δ) 1 puan altından yukarıdaysa FG_ref = FG∞ (ölü bölge
    yok: eşik asimptotun altında kalmaz). Aşağı yönde eski 0.004 farkı korunur."""
    if shown and fg_inf is not None:
        if u(fg_inf) > u(fg_recipe) + u(delta) - 100: return fg_inf, "tahmin"
        if u(fg_inf) < u(fg_recipe) - 400: return fg_inf, "tahmin"
    return fg_recipe, "tarif"

def is_stable(points, hours=48, min_n=24):
    """Son 'hours' saatin eğimi; |eğim|·24 ≤ 0.0005 ise FG sabit."""
    t_end = points[-1][0]
    p = [(t, g) for t, g in points if t >= t_end - hours]
    if len(p) < min_n: return None
    n = len(p); mx = sum(t for t, _ in p) / n; my = sum(g for _, g in p) / n
    sxx = sum((t - mx) ** 2 for t, _ in p)
    if sxx <= 0: return None                       # bütün noktalar aynı zamanda: eğim tanımsız
    b = sum((t - mx) * (g - my) for t, g in p) / sxx
    return abs(b * 24) <= STABLE_PER_DAY + TOL, b * 24

def ramp_hours(T_from, T_to, rate_C_per_day):
    return abs(T_to - T_from) / rate_C_per_day * 24

def lager_weeks(og, pts_per_week=8, lo=2, hi=12):
    """Kaba kural: her 8 OG puanı için 1 hafta, YUKARI yuvarlanır [doğrulanacak]. Önce tam sayı puana çevrilir
    (round, ikili kayan nokta hatası: (1.048-1)*1000 = 48.00000000000004), sonra tam sayı bölmeyle tavan alınır."""
    pts = round((og - 1) * 100000)                         # 0.01 puan birimi
    w = -(-pts // (pts_per_week * 100))                    # ceil(pts / (8·100))
    return min(hi, max(lo, w))

# ================================================================ 2. Fıçıda priming
R = 8.314462618; P_ATM = 101325.0; VM = 22.414      # L/mol (0 °C, 1 atm)
M_DEX_MONO = 198.17; M_SUC = 342.30; M_INVERT = 360.31   # g/mol; CO2 verimi: 2, 4, 4 mol
SUGARS = {"dekstroz": (M_DEX_MONO, 2), "sakaroz": (M_SUC, 4), "invert": (M_INVERT, 4)}

def sugar_from_mol(mol, kind, dry=1.0):
    """dry = ürünün kuru madde oranı (0–1); şurup/blok ürünlerde < 1. Varsayılan 1.0 (saf kuru şeker)
    [doğrulanacak; malzeme veritabanından gelir]."""
    M, n = SUGARS[kind]; return mol / n * M / dry

def bottle_mol(vol_t, T_res, V_beer):
    return max(0.0, vol_t - residual_vol(T_res)) * V_beer / VM

def head_mol(vol_t, T_cond, V_head, purged=True):
    """Kafa boşluğunda dengedeki CO2. Kısmi basınç = gösterge basıncı + 1 atm (tablo saf CO2 varsayar).
    Fıçı CO2 ile süpürülmüşse başta 1 atm CO2 zaten var."""
    P_abs = co2_psi(vol_t, T_cond) * 6894.757 + P_ATM
    T_K = T_cond + 273.15
    n = P_abs * V_head / 1000 / (R * T_K)
    n0 = P_ATM * V_head / 1000 / (R * T_K) if purged else 0.0
    return max(0.0, n - n0), P_abs

def keg_priming(vol_t, T_res, T_cond, V_beer, V_head, kind="dekstroz", purged=True, mode="tam", part=1/3, T_serv=4.0, dry=1.0):
    nb = bottle_mol(vol_t, T_res, V_beer)
    nh, P_abs = head_mol(vol_t, T_cond, V_head, purged)
    full = sugar_from_mol(nb + nh, kind, dry)
    bottle_same = sugar_from_mol(nb, kind, dry)
    p_co2 = (P_abs - P_ATM) / 1e5
    # havalı fıçıda kafa boşluğundaki 1 atm hava ayrıca basınç yapar: gösterge basıncı = CO2 tablo basıncı + 1 atm
    p_total = p_co2 + (0.0 if purged or V_head <= 0 else P_ATM / 1e5)
    out = {"seker_g": r(full if mode == "tam" else full * part, 1), "sise_esdegeri_g": r(bottle_same, 1),
           "kafa_payi_%": r(100 * (full / bottle_same - 1), 1) if bottle_same > 0 else None,
           "denge_basinci_bar": r(p_co2, 2), "toplam_gosterge_bar": r(p_total, 2)}
    if mode == "kismi":
        out["co2_tamamlama_bar"] = r(co2_psi(vol_t, T_serv) * 0.0689476, 2)   # serviste hedefe çıkarır
    return out

# ================================================================ 3. Cask
CO2_PER_EXTRACT = 0.9565 / 2.0665     # Balling: 2.0665 g ekstrakt → 0.9565 g CO2
G_PER_VOL = 44.01 / VM                # 1 vol = 1.9635 g/L

def co2_from_remaining(sg_rack, fg_ref):
    """Cask'a kalan ekstraktla alınırsa oluşacak CO2 (vol). Gerçek ekstrakt = 0.8192 × görünür (Balling)."""
    if u(sg_rack) <= u(fg_ref): return 0.0
    dAE = ToPlato(sg_rack) - ToPlato(fg_ref)
    dRE_gL = 0.8192 * dAE * 10 * sg_rack
    return dRE_gL * CO2_PER_EXTRACT / G_PER_VOL

def cask_priming(V, vol_t, T_res, sg_rack, fg_ref, kind="invert", dry=1.0):
    res = residual_vol(T_res); fromx = co2_from_remaining(sg_rack, fg_ref)
    need = vol_t - res - fromx
    g = sugar_from_mol(max(0.0, need) * V / VM, kind, dry)
    return {"kalinti_vol": r(res, 2), "ekstrakttan_vol": r(fromx, 2), "eksik_vol": r(need, 2),
            "seker_g": r(g, 1), "g_L": r(g / V, 2), "fazla": need < 0}

def finings_mL(V, rate_mL_per_L=2.0): return V * rate_mL_per_L     # [doğrulanacak]

def cask_days(T_cellar, base=3.0, T0=12.0):
    """Mahzende kondisyon süresi: 12 °C'de 3 gün; her 10 °C düşüşte iki katı [tasarım, doğrulanacak]."""
    return base * 2 ** ((T0 - T_cellar) / 10)


if __name__ == "__main__":
    print("== 1. Gravite tetikleyicisi")
    pts = [(h, 1.012 + 0.036 * math.exp(-0.03 * h)) for h in range(12, 200)]
    print("T1a lager, FG 1.012, Δ 0.004 → tetik saati:", trigger_index(pts, 1.012, 0.004))
    _d, _t = predicted_trigger(1.012, 0.036, 0.03, 1.012, 0.004)
    print("T1b model ile tahmin (FGinf 1.012, A 0.036, k 0.03):", _d, r(_t, 1))
    noisy = [(12, 1.0170), (13, 1.0158), (14, 1.0162), (15, 1.0159), (16, 1.0158), (17, 1.0157)]
    print("T1c gürültü: tek nokta altına inip çıkıyor → tetik:", trigger_index(noisy, 1.012, 0.004))
    print("T1d FG referansı: tarif 1.012, tahmin 1.018 gösteriliyor (Δ 0.004):", fg_reference(1.012, 1.018, True, 0.004))
    flat = [(h, 1.0122 - 0.00001 * (h - 150)) for h in range(150, 200)]
    print("T1e son 48 sa eğim -0.00024/gün → sabit:", is_stable(flat))
    drop = [(h, 1.0200 - 0.00005 * (h - 150)) for h in range(150, 200)]
    print("T1f son 48 sa eğim -0.0012/gün → sabit değil:", is_stable(drop))
    print("T1i ölü bölge: tarif 1.010, FG∞ 1.013, Δ 0.002 (ale):", fg_reference(1.010, 1.013, True, 0.002))
    print("T1j tetik erişilemez (FG_ref 1.010, FG∞ 1.013, Δ 0.002):", predicted_trigger(1.013, 0.037, 0.03, 1.010, 0.002))
    print("T1k tetik zaten geçmiş (A 0.003 ≤ Δ 0.004):", predicted_trigger(1.012, 0.003, 0.03, 1.012, 0.004))
    print("T1g lager rampası 15→2 °C, 2 °C/gün: %.0f saat" % ramp_hours(15, 2, 2))
    print("T1h lager süresi OG 1.048: %d hafta; OG 1.072: %d hafta" % (lager_weeks(1.048), lager_weeks(1.072)))
    print("T1l lager süresi OG 1.050: %d; 1.056: %d; 1.080: %d; 1.0485: %d hafta" % (lager_weeks(1.050), lager_weeks(1.056), lager_weeks(1.080), lager_weeks(1.0485)))

    print("\n== 2. Fıçıda priming (2.4 vol, fermantasyon 20 °C, kondisyon 20 °C)")
    print("T2a 19 L bira, 0 L boşluk:", keg_priming(2.4, 20, 20, 19, 0))
    print("T2b 19 L bira, 2 L boşluk, CO2 ile süpürülmüş:", keg_priming(2.4, 20, 20, 19, 2))
    print("T2c aynı, hava ile:", keg_priming(2.4, 20, 20, 19, 2, purged=False))
    print("T2d kısmi (1/3) + CO2 ile tamamla, servis 4 °C:", keg_priming(2.4, 20, 20, 19, 2, mode="kismi"))
    print("T2e sakaroz, T2b koşulları:", keg_priming(2.4, 20, 20, 19, 2, kind="sakaroz"))

    print("\n== 3. Cask (firkin 40.9 L, hedef 1.1 vol)")
    print("T3a FG'de alındı, fermantasyon 19 °C:", cask_priming(40.9, 1.1, 19, 1.010, 1.010))
    print("T3b 1 puan kala alındı:", cask_priming(40.9, 1.1, 19, 1.011, 1.010))
    print("T3c 2 puan kala alındı (fazla):", cask_priming(40.9, 1.1, 19, 1.012, 1.010))
    print("T3d 2 puandan CO2: %.2f vol" % co2_from_remaining(1.012, 1.010))
    print("T3g invert şeker %80 kuru madde:", cask_priming(40.9, 1.1, 19, 1.010, 1.010, dry=0.8))
    print("T3e durultucu, firkin, 2 mL/L: %.0f mL" % finings_mL(40.9))
    print("T3f kondisyon 12 °C: %.1f gün; 8 °C: %.1f gün" % (cask_days(12), cask_days(8)))
