"""Bölüm 11 parite vektörleri: gravite tetikleyicisi, fıçıda priming, cask (ADR-0003)."""
import math

from ortak import Vektor, yukle


def _hata(f):
    """Python istisnasını {"hata": ad} olarak yazar (ValueError, ZeroDivisionError, IndexError)."""
    try:
        return f()
    except (ValueError, ZeroDivisionError, IndexError) as e:
        return {"hata": type(e).__name__}


def uret():
    ref = yukle("11_tetikleyici_fici_cask_tv.py")
    v = Vektor("11", "formuller/testler/11_tetikleyici_fici_cask_tv.py")

    # ---- ortak yardımcılar (04 ve 01 karşılıkları)
    for sg in [1.0, 1.012, 1.04, 1.0485, 1.05, 1.000005, 1.016, 1.01601, 1.01599]:
        v.ekle("u", {"sg": sg}, ref.u(sg))
    for c in [-5.0, 0.0, 4.0, 20.0, 25.0, 100.0]:
        v.ekle("f_from_c", {"c": c}, ref.F(c))
    for sg in [1.0, 1.010, 1.012, 1.048, 1.100]:
        v.ekle("to_plato", {"sg": sg}, ref.ToPlato(sg))
    for t in [-10.0, -0.5, 0.0, 4.0, 12.0, 19.0, 20.0, 25.0, 30.0]:
        v.ekle("residual_vol", {"t_c": t}, ref.residual_vol(t))
    for vol, t in [(2.4, 4.0), (2.4, 20.0), (1.1, 12.0), (0.0, 4.0), (3.5, 25.0), (2.4, -5.0), (0.8, 30.0), (2.4, 0.0)]:
        v.ekle("co2_psi", {"vols": vol, "t_c": t}, ref.co2_psi(vol, t))

    # ---- 1. Gravite tetikleyicisi
    def ti(points, fg_ref, delta, consecutive=3, not_=None):
        girdi = {"points": points, "fg_ref": fg_ref, "delta": delta, "consecutive": consecutive}
        v.ekle("trigger_index", girdi,
               _hata(lambda: ref.trigger_index([tuple(p) for p in points], fg_ref, delta, consecutive)), not_)

    curve = [[h, 1.012 + 0.036 * math.exp(-0.03 * h)] for h in range(12, 200)]
    ti(curve, 1.012, 0.004, 3, "T1a")
    ti(curve, 1.012, 0.004, 1)
    ti(curve, 1.012, 0.002, 3)
    ti([[12, 1.0170], [13, 1.0158], [14, 1.0162], [15, 1.0159], [16, 1.0158], [17, 1.0157]], 1.012, 0.004, 3, "T1c")
    # eşik kenarları: eşik 1.016 = 101600 birim
    ti([[0, 1.01600], [1, 1.01600], [2, 1.01600]], 1.012, 0.004, 3, "tam eşikte")
    ti([[0, 1.01601], [1, 1.01601], [2, 1.01601]], 1.012, 0.004, 3, "+1 birim: tetiklemez")
    ti([[0, 1.01599], [1, 1.01599], [2, 1.01599]], 1.012, 0.004, 3, "-1 birim")
    ti([[0, 1.01600], [1, 1.01601], [2, 1.01599], [3, 1.01600], [4, 1.01600]], 1.012, 0.004, 3,
       "dizi koparsa sayaç sıfırlanır")
    # 1.014 - 1.013 > 0.001 ikili kayan nokta durumu
    ti([[0, 1.014], [1, 1.014], [2, 1.014]], 1.013, 0.001, 3, "1.014 = 1.013 + 0.001")
    ti([[0, 1.014], [1, 1.014]], 1.013, 0.001, 2)
    ti([], 1.012, 0.004, 3, "boş")
    ti(curve, 1.012, 0.004, 0, "T1.12 D9: consecutive = 0 hata")
    ti([[0, 1.0], [1, 1e9]], 1.012, 0.004, 1, "T1.12 D2: makul olmayan SG hata")
    ti([[0, 1.0]], 1.012, 1e9, 1, "T1.12 D2: makul olmayan Δ hata")
    ti([[0, 1.010]], 1.012, 0.004, 3, "tek nokta, 3 gerekir")
    ti([[0, 1.010]], 1.012, 0.004, 1, "elle okuma")
    ti([[3, 1.0], [1, 1.0], [2, 1.0]], 1.0, 0.0, 3, "sırasız zamanlar, ilk sağlanan nokta")

    def pt(fg_inf, A, k, fg_ref, delta, not_=None):
        girdi = {"fg_inf": fg_inf, "a": A, "k": k, "fg_ref": fg_ref, "delta": delta}
        beklenen = _hata(lambda: list(ref.predicted_trigger(fg_inf, A, k, fg_ref, delta)))
        v.ekle("predicted_trigger", girdi, beklenen, not_)

    pt(1.012, 0.036, 0.03, 1.012, 0.004, "T1b")
    pt(1.013, 0.037, 0.03, 1.010, 0.002, "T1j erişilemez")
    pt(1.012, 0.003, 0.03, 1.012, 0.004, "T1k geçmiş")
    pt(1.012, 0.00001, 0.03, 1.012, 0.00001, "A = hedef: geçmiş")
    pt(1.012, 0.036, 0.0, 1.012, 0.004, "k = 0: ZeroDivisionError")
    pt(1.012, 0.036, -0.03, 1.012, 0.004, "k < 0: ValueError")
    pt(1.012, 0.036, 1e9, 1.012, 0.004, "k çok büyük: sonlu, ~0 saat")
    pt(1.012, 1e9, 0.03, 1.012, 0.004, "T1.12 D2: makul olmayan A")
    pt(1e9, 0.036, 0.03, 1.012, 0.004, "T1.12 D2: makul olmayan FG∞")
    pt(1.012, 0.00002, 0.03, 1.012, 0.00001, "A = hedef + 1 birim")
    pt(1.012, 0.00001, 0.03, 1.012, 0.0, "hedef 0: erişilemez")
    pt(1.01201, 0.03, 0.03, 1.012, 0.0, "FG∞ eşiğin 1 birim üstünde")
    pt(1.01199, 0.03, 0.03, 1.012, 0.0, "FG∞ eşiğin 1 birim altında")
    pt(1.012, 0.030, 0.05, 1.012, 0.004)
    pt(1.010, 0.040, 0.01, 1.012, 0.002)
    pt(1.010, 0.040, 0.0, 1.012, 0.002, "k = 0")

    def fr(fg_recipe, fg_inf=None, shown=False, delta=0.004, not_=None):
        fg, kaynak = ref.fg_reference(fg_recipe, fg_inf, shown, delta)
        v.ekle("fg_reference", {"fg_recipe": fg_recipe, "fg_inf": fg_inf, "shown": shown, "delta": delta},
               [fg, kaynak], not_)

    fr(1.012, 1.018, True, 0.004, "T1d")
    fr(1.010, 1.0085, True, 0.002, "T1m")
    fr(1.012, 1.0085, True, 0.004)
    fr(1.012, 1.0095, True, 0.004)
    fr(1.010, 1.013, True, 0.002, "T1i")
    fr(1.012, 1.018, False, 0.004, "tahmin gösterilmiyor")
    fr(1.012, None, True, 0.004, "FG∞ yok")
    # kenarlar: Δ 0.004 → marj 300 birim; tarif 1.012 = 101200
    fr(1.012, 1.01500, True, 0.004, "+marj: tarif")
    fr(1.012, 1.01501, True, 0.004, "+marj+1: tahmin")
    fr(1.012, 1.00900, True, 0.004, "-marj: tarif")
    fr(1.012, 1.00899, True, 0.004, "-marj-1: tahmin")
    fr(1.010, 1.01200, True, 0.002, "Δ 0.002, marj 100: +200 tahmin")
    fr(1.010, 1.01100, True, 0.002, "marj kenarı: tarif")
    fr(1.010, 1.01101, True, 0.002, "marj+1: tahmin")
    fr(1.010, 1.00900, True, 0.002, "-marj kenarı: tarif")
    fr(1.010, 1.00899, True, 0.002, "-marj-1: tahmin")
    fr(1.012, 1.012, True, 0.0, "Δ 0: marj -100")
    fr(1.012, 1.01199, True, 0.0)
    fr(1.012, 1.01201, True, 0.0)

    def st(points, hours=48, min_n=24, not_=None):
        girdi = {"points": points, "hours": hours, "min_n": min_n}
        try:
            r = ref.is_stable([tuple(p) for p in points], hours, min_n)
            beklenen = None if r is None else {"sabit": r[0], "egim_gun": r[1]}
        except IndexError:
            beklenen = {"hata": "IndexError"}
        except ValueError:
            beklenen = {"hata": "ValueError"}
        v.ekle("is_stable", girdi, beklenen, not_)

    flat = [[h, 1.0122 - 0.00001 * (h - 150)] for h in range(150, 200)]
    drop = [[h, 1.0200 - 0.00005 * (h - 150)] for h in range(150, 200)]
    st(flat, 48, 24, "T1e")
    st(drop, 48, 24, "T1f")
    st([[h, 1.0100 + 0.00001 * h] for h in range(150, 200)], 48, 24, "yükselen")
    st(flat[:20], 48, 24, "yetersiz nokta")
    st(flat, 24, 24, "24 saat penceresi")
    st(flat, 24, 25, "24 saatte 25 nokta yok")
    st([[100.0, 1.01]] * 30, 48, 24, "aynı zaman: eğim tanımsız")
    st([], 48, 24, "boş")
    st([[0.0, 1.0], [1e9, 1.0]], 48, 24, "T1.16: makul olmayan zaman: hata")
    st([[0.0, 1.0], [1.0, 1.0]], 1e12, 1, "T1.16: makul olmayan pencere: hata")
    # eşik kenarları: günde 0.0005 SG
    st([[h, 1.0120 - (0.0005 / 24) * h] for h in range(0, 60)], 48, 24, "eşikte")
    st([[h, 1.0120 - (0.00051 / 24) * h] for h in range(0, 60)], 48, 24, "eşik üstü")
    st([[h, 1.0120 - (0.00049 / 24) * h] for h in range(0, 60)], 48, 24, "eşik altı")
    st([[h, 1.0120 + (0.0005 / 24) * h] for h in range(0, 60)], 48, 24, "eşikte artan")

    def rh(a, b, rate, not_=None):
        try:
            beklenen = ref.ramp_hours(a, b, rate)
        except ZeroDivisionError:
            beklenen = {"hata": "ZeroDivisionError"}
        except ValueError:
            beklenen = {"hata": "ValueError"}
        v.ekle("ramp_hours", {"t_from": a, "t_to": b, "rate": rate}, beklenen, not_)

    rh(20.0, 5.0, 1e9, "T1.16: makul olmayan hız: hata")
    rh(15, 2, 2, "T1g")
    rh(2, 15, 2)
    rh(10, 10, 2)
    rh(20, 4, 0.5)
    rh(12.5, 3.25, 1.5)
    rh(15, 2, 0.0, "hız 0")

    def lw(og, ppw=8, lo=2, hi=12, not_=None):
        beklenen = _hata(lambda: ref.lager_weeks(og, ppw, lo, hi))
        v.ekle("lager_weeks", {"og": og, "pts_per_week": ppw, "lo": lo, "hi": hi}, beklenen, not_)

    for og in [1.048, 1.072, 1.050, 1.056, 1.080, 1.0485, 1.040, 1.032, 1.033, 1.030, 1.016, 1.017,
               1.096, 1.097, 1.120, 1.200, 1.00001, 1.0, 1.04801, 1.04799, 1.04800]:
        lw(og)
    lw(1.048, 4, 1, 20)
    lw(1.048, 10, 2, 12)
    lw(1.048, 8, 3, 4)
    lw(1.048, 0, 2, 12, "bölen 0")
    lw(1.048, -8, 2, 12, "T1.12 D9: negatif pts_per_week hata")
    lw(0.9969, -8, 2, 12, "T1.12 D9: negatif, Python // ile Rust div_euclid ayrışırdı")
    lw(1.048, 10 ** 9, 2, 12, "T1.12 D2: taşma sınırı")
    lw(1e9, 8, 2, 12, "T1.12 D2: makul olmayan OG")

    # ---- 2. Fıçıda priming
    for kind in ["dekstroz", "sakaroz", "invert"]:
        for mol in [0.0, 0.5, 1.0, 2.37]:
            for dry in [None, 1.0, 0.8]:
                v.ekle("sugar_from_mol", {"mol": mol, "kind": kind, "dry": dry},
                       ref.sugar_from_mol(mol, kind, dry))
    v.ekle("sugar_from_mol", {"mol": 1.0, "kind": "invert", "dry": 0.0}, {"hata": "ZeroDivisionError"},
           "kuru madde 0")

    for vol, t, vb in [(2.4, 20, 19), (2.4, 4, 19), (1.1, 19, 40.9), (0.8, 20, 19), (3.0, 12, 20.5), (2.4, 20, 0)]:
        v.ekle("bottle_mol", {"vol_t": vol, "t_res": t, "v_beer": vb}, ref.bottle_mol(vol, t, vb))
    for vol, t in [(2.4, 4), (2.4, 20), (1.1, 12), (3.0, 25), (0.0, 4)]:
        v.ekle("p_abs_pa", {"vol": vol, "t_c": t}, ref.p_abs_Pa(vol, t))
    for vol, t, vh in [(2.4, 4, 2), (2.4, 20, 2), (2.4, 20, 0), (1.1, 12, 5), (3.0, 25, 1.5)]:
        v.ekle("head_gas_mol", {"vol": vol, "t_c": t, "v_head": vh}, ref.head_gas_mol(vol, t, vh))
    for vol, tc, vh, pur in [(2.4, 20, 2, True), (2.4, 20, 2, False), (2.4, 4, 2, True), (2.4, 20, 0, True),
                             (1.1, 12, 5, True), (0.5, 25, 3, True), (3.0, 25, 1.5, False)]:
        n, p = ref.head_mol(vol, tc, vh, pur)
        v.ekle("head_mol", {"vol_t": vol, "t_cond": tc, "v_head": vh, "purged": pur}, [n, p])
    for nt, tc, vb, vh in [(1.0, 20, 19, 2), (2.0, 20, 19, 2), (3.0, 25, 19, 2), (1.5, 4, 19, 0), (2.5, 12, 40.9, 5),
                           (0.0, 20, 19, 2), (50.0, 20, 19, 2)]:
        v.ekle("equilibrium_vol", {"n_total": nt, "t_c": tc, "v_beer": vb, "v_head": vh},
               ref.equilibrium_vol(nt, tc, vb, vh))

    D = {"vol_t": 2.4, "T_res": 20, "T_cond": 20, "V_beer": 19, "V_head": 2, "kind": "dekstroz", "purged": True,
         "mode": "tam", "part": 1 / 3, "T_serv": 4.0, "dry": None, "limit_bar": None}

    def kp(not_=None, **kw):
        a = dict(D)
        a.update(kw)
        v.ekle("keg_priming", a, _hata(lambda: ref.keg_priming(**a)), not_)

    kp("T2a", V_head=0, dry=1.0)
    kp("T2b", dry=1.0)
    kp("T2c", purged=False, dry=1.0)
    kp("T2d", mode="kismi", dry=1.0)
    kp("T2e", kind="sakaroz", dry=1.0)
    kp("T2g", dry=None)
    kp("T2h", T_serv=20.0, dry=1.0)
    kp("T2i", T_cond=25, purged=False, dry=1.0, limit_bar=2.5)
    kp("limit aşılmıyor", T_cond=25, purged=False, dry=1.0, limit_bar=5.0)
    kp("invert", kind="invert", dry=0.8)
    kp("kısmi yarım", mode="kismi", part=0.5, dry=1.0)
    kp("soğuk kondisyon", T_cond=4, T_serv=10.0, dry=1.0)
    kp("hedef < kalıntı", vol_t=0.5, dry=1.0)
    kp("boşluk 0, havayla", V_head=0, purged=False, dry=1.0)
    kp("büyük boşluk", V_head=8, dry=1.0, limit_bar=1.0)
    kp("sıfır hacim", V_beer=0, dry=1.0)
    kp("kuru madde 0", dry=0.0)
    kp("20 L Corny", V_beer=20, V_head=3, T_res=18, T_cond=15, T_serv=3.0, kind="sakaroz", dry=1.0, mode="kismi")
    kp("sıcaklık sınırı altı", T_serv=-5.0, dry=1.0)
    kp("yüksek vol", vol_t=3.5, T_cond=22, dry=1.0, purged=False, limit_bar=3.0)
    # T1.12 D6: aralık dışı uyarı kodları, doyma; D13: sınır toleransı; D2: geçersiz girdi
    kp("hedef 0 °C altı kondisyon", T_cond=-2.0, T_serv=-1.0, dry=1.0)
    kp("0 °C sınırı: uyarı yok", T_cond=0.0, T_serv=0.0, dry=1.0)
    kp("hedef 3.6 vol", vol_t=3.6, T_cond=10, T_serv=4.0, dry=1.0)
    kp("hedef tam 3.5 vol: hedef uyarısı yok", vol_t=3.5, T_cond=4, T_serv=4.0, dry=1.0)
    kp("hedef 12 vol: tepe doyar", vol_t=12.0, T_cond=20, T_serv=4.0, dry=1.0, limit_bar=2.0)
    kp("büyük boşluk, hedef 9 vol", vol_t=9.0, V_head=8, T_cond=25, dry=1.0)
    kp("tepe basıncı sınıra tam eşit", T_cond=25, purged=False, dry=1.0,
       limit_bar=ref.keg_priming(**dict(D, T_cond=25, purged=False, dry=1.0))["tepe_toplam_gosterge_bar"])
    kp("T1.12 D2: negatif boşluk", V_head=-1.0, dry=1.0)
    kp("T1.12 D2: negatif kuru madde", dry=-0.5)
    kp("T1.12 D2: kuru madde > 1", dry=1.5)
    kp("T1.12 D2: part 0", mode="kismi", part=0.0, dry=1.0)
    kp("T1.12 D2: makul olmayan hacim", V_beer=1e9, dry=1.0)

    # ---- 3. Cask
    for sg, fg in [(1.012, 1.010), (1.011, 1.010), (1.010, 1.010), (1.00999, 1.010), (1.01001, 1.010), (1.020, 1.010),
                   (1.015, 1.008), (1.030, 1.012), (1.0, 1.0)]:
        v.ekle("co2_from_remaining", {"sg_rack": sg, "fg_ref": fg}, ref.co2_from_remaining(sg, fg))

    def cp(V, vol_t, T_res, sg_rack, fg_ref, kind="invert", dry=None, not_=None):
        beklenen = _hata(lambda: ref.cask_priming(V, vol_t, T_res, sg_rack, fg_ref, kind, dry))
        v.ekle("cask_priming", {"v": V, "vol_t": vol_t, "t_res": T_res, "sg_rack": sg_rack, "fg_ref": fg_ref,
                                "kind": kind, "dry": dry}, beklenen, not_)

    cp(40.9, 1.1, 19, 1.010, 1.010, dry=1.0, not_="T3a")
    cp(40.9, 1.1, 19, 1.011, 1.010, dry=1.0, not_="T3b")
    cp(40.9, 1.1, 19, 1.012, 1.010, dry=1.0, not_="T3c")
    cp(40.9, 1.1, 19, 1.010, 1.010, dry=0.8, not_="T3g")
    cp(40.9, 1.1, 19, 1.010, 1.010, not_="T3h")
    cp(40.9, 1.1, 19, 1.010, 1.010, kind="sakaroz", dry=1.0)
    cp(40.9, 1.1, 19, 1.010, 1.010, kind="dekstroz", dry=1.0)
    cp(20.0, 1.4, 12, 1.0105, 1.010, dry=0.8)
    cp(20.0, 0.9, 12, 1.0101, 1.010, dry=0.8)
    cp(20.0, 1.1, 40, 1.010, 1.010, dry=1.0, not_="kalıntı polinomu yüksek T")
    cp(40.9, 1.1, 19, 1.010, 1.010, dry=0.0, not_="kuru madde 0")
    cp(0.0, 1.1, 19, 1.010, 1.010, dry=1.0, not_="hacim 0")
    cp(-5.0, 1.1, 19, 1.010, 1.010, dry=1.0, not_="T1.12 D2: negatif hacim")
    cp(40.9, 1.1, 19, 1.010, 1.010, dry=-0.1, not_="T1.12 D2: negatif kuru madde")
    cp(40.9, 1.1, 19, 1e9, 1.010, dry=1.0, not_="T1.12 D2: makul olmayan SG")
    cp(40.9, 1.1, 19, 1.0105, 1.010, dry=1.0, not_="eksik_vol = 0'a yakın: yarım birim")
    cp(40.9, 0.89, 19, 1.010, 1.010, dry=1.0, not_="eksik_vol ~ 0")

    for V, rate in [(40.9, 2.0), (20.0, 2.0), (40.9, 1.5), (0.0, 2.0), (22.7, 0.0)]:
        v.ekle("finings_ml", {"v": V, "rate": rate}, ref.finings_mL(V, rate))
    for t in [12, 8, 20, 0, 11, 13, -5, 4.5]:
        v.ekle("cask_days", {"t_cellar": t, "base": 3.0, "t0": 12.0}, ref.cask_days(t))
    v.ekle("cask_days", {"t_cellar": 8, "base": 5.0, "t0": 10.0}, ref.cask_days(8, 5.0, 10.0))

    # ---- 4. Varsayılanlar ve ilerleme (T1.14d)
    for kind in ["lager", "ale"]:
        v.ekle("default_delta", {"kind": kind}, ref.default_delta(kind))
    v.ekle("defaults", {}, {"delta_lager": ref.DELTA_LAGER, "delta_ale": ref.DELTA_ALE,
                            "lager_cool_rate": ref.LAGER_COOL_RATE, "partial_fraction": ref.PARTIAL_FRACTION,
                            "cask_target_vol": ref.CASK_TARGET_VOL, "finings_ml_per_l": ref.FININGS_ML_PER_L},
           "varsayılan sabitler")
    for og, sg, fg, n in [(1.048, 1.048, 1.012, "başlangıç: 0"), (1.048, 1.012, 1.012, "bitiş: 1"),
                          (1.048, 1.030, 1.012, "yarı"), (1.048, 1.020, 1.012, None), (1.048, 1.010, 1.012, "FG_ref altı: >1"),
                          (1.048, 1.050, 1.012, "SG > OG: negatif"), (1.012, 1.010, 1.012, "OG = FG_ref: None"),
                          (1.010, 1.008, 1.012, "OG < FG_ref: None"), (1.0485, 1.0165, 1.0105, None),
                          (1.050, 1.016, 1.010, None)]:
        v.ekle("progress", {"og": og, "sg": sg, "fg_ref": fg}, ref.progress(og, sg, fg), n)

    return v.yaz()
