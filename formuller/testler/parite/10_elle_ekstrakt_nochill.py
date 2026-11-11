"""Bölüm 10 parite vektörleri: elle giriş tahmini, ekstrakt araçları, no-chill (ADR-0003)."""
from ortak import Vektor, yukle


def uret():
    ref = yukle("10_elle_ekstrakt_nochill_tv.py")
    v = Vektor("10", "formuller/testler/10_elle_ekstrakt_nochill_tv.py")

    def dene(fonk, girdi, f, not_=None):
        """Python hata verirse {"hata": ad} yazar."""
        try:
            v.ekle(fonk, girdi, f(), not_)
        except Exception as e:  # noqa: BLE001
            v.ekle(fonk, girdi, {"hata": type(e).__name__}, not_)

    # ---- yardımcılar: u, r
    for sg in (1.050, 1.000005, 1.000015, 1.01234, 0.99999):
        v.ekle("u", {"sg": sg}, ref.u(sg))
    for x, n in ((2.675, 2), (0.125, 2), (1.04567, 4), (12.25, 1), (3.14159, 3)):
        v.ekle("r", {"x": x, "n": n}, ref.r(x, n))

    # ---- weight (eşik: SG-FG = 300 birim)
    for sg, fg in ((1.030, 1.012), (1.015, 1.012), (1.01499, 1.012), (1.01501, 1.012), (1.0125, 1.012),
                   (1.012, 1.012), (1.010, 1.012), (1.014, 1.012)):
        v.ekle("weight", {"sg": sg, "fg": fg}, ref.weight(sg, fg))

    # ---- k_fit
    def kf(readings, og, fg, **kw):
        girdi = {"readings": [list(p) for p in readings], "og": og, "fg": fg}
        girdi.update(kw)
        k, each = ref.k_fit(readings, og, fg, **kw)
        v.ekle("k_fit", girdi, {"yok": True} if k is None else {"yok": False, "k": k, "k_each": each})

    kf([(48, 1.030), (96, 1.018)], 1.050, 1.012)
    kf([(48, 1.030), (84, 1.014)], 1.050, 1.012)
    kf([(48, 1.030), (84, 1.014)], 1.050, 1.012, weighted=False)
    kf([(48, 1.030), (84, 1.013)], 1.050, 1.012)
    kf([(48, 1.030), (84, 1.013)], 1.050, 1.012, weighted=False)
    kf([(6, 1.049), (12, 1.049), (24, 1.046)], 1.050, 1.012)
    kf([(6, 1.049), (12, 1.049)], 1.050, 1.012)
    kf([(48, 1.0125), (60, 1.0120)], 1.050, 1.012)
    kf([(48, 1.01250), (60, 1.01251)], 1.050, 1.012)      # eşik: tam 50 birim atlanır, 51 alınır
    kf([(20, 1.040), (30, 1.030)], 1.050, 1.012, lag=6.0)
    kf([(48, 1.020), (94, 1.019)], 1.050, 1.012)
    kf([(48, 1.030)], 1.050, 1.012)

    # ---- t_end
    for og, fg, k, lag in ((1.050, 1.012, 0.0218, 12.0), (1.060, 1.014, 0.03, 8.0), (1.045, 1.010, 0.05, 12.0)):
        v.ekle("t_end", {"og": og, "fg": fg, "k": k, "lag": lag}, ref.t_end(og, fg, k, lag))

    # ---- few_forecast
    def ff(readings, og, fg, not_=None, **kw):
        girdi = {"readings": [list(p) for p in readings], "og": og, "fg_prior": fg}
        girdi.update(kw)
        dene("few_forecast", girdi, lambda: ref.few_forecast(readings, og, fg, **kw), not_)

    ff([(48, 1.030), (96, 1.018)], 1.050, 1.012)                              # 1a
    ff([(48, 1.030)], 1.050, 1.012)                                           # 1b
    ff([(24, 1.046), (36, 1.042)], 1.050, 1.012)                              # 1c
    ff([(96, 1.018), (168, 1.013), (216, 1.013)], 1.050, 1.012)               # 1d
    ff([(48, 1.030), (120, 1.024), (168, 1.024)], 1.050, 1.012)               # 1e
    ff([(48, 1.020), (94, 1.019)], 1.050, 1.012)                              # 1f
    ff([(96, 1.018), (168, 1.014), (216, 1.013)], 1.050, 1.012)               # 1g
    ff([(48, 1.030), (84, 1.014)], 1.050, 1.012)                              # 1h
    ff([(84, 1.014), (48, 1.030)], 1.050, 1.012, not_="sırasız")
    ff([], 1.050, 1.012, not_="boş liste: IndexError")
    ff([(48, 1.030)], 1.050, 1.012)
    ff([(5, 1.050)], 1.050, 1.012, not_="gecikme içinde tek okuma")
    # durgunluk eşikleri: 48 sa arayla, fark 100 / 101 birim; 47.9 sa
    ff([(48, 1.020), (96, 1.019)], 1.050, 1.012)
    ff([(48, 1.020), (96, 1.01899)], 1.050, 1.012)
    ff([(48, 1.020), (95.9, 1.019)], 1.050, 1.012)
    # "bitmiş görünüyor" eşiği: FG + 400 birim (1.016) ve 401
    ff([(48, 1.020), (96, 1.016)], 1.050, 1.012)
    ff([(48, 1.020), (96, 1.01601)], 1.050, 1.012)
    ff([(48, 1.020), (96, 1.0159)], 1.050, 1.012)
    # %30 düşüş eşiği: 1.0386 tam 1140 birim (geçer), 1.03861 (düşer)
    ff([(30, 1.045), (60, 1.0386)], 1.050, 1.012)
    ff([(30, 1.045), (60, 1.03861)], 1.050, 1.012)
    # hedefe ulaştı: SG = FG + 50 birim ve +51
    ff([(30, 1.030), (60, 1.0125)], 1.050, 1.012)
    ff([(30, 1.030), (60, 1.01251)], 1.050, 1.012)
    ff([(30, 1.030), (60, 1.012)], 1.050, 1.012)
    # fg_prior ± sigma'nın zaten aşıldığı dal
    ff([(30, 1.040), (60, 1.0140)], 1.050, 1.012)
    ff([(30, 1.040), (60, 1.0165)], 1.050, 1.012)
    # lag, sigma değişkenleri
    ff([(30, 1.030), (60, 1.020)], 1.050, 1.012, lag=6.0)
    ff([(30, 1.030), (60, 1.020)], 1.050, 1.012, sigma=0.001)
    ff([(30, 1.030), (60, 1.020)], 1.050, 1.012, lag=24.0)
    ff([(30, 1.030), (40, 1.0295), (60, 1.026)], 1.050, 1.012)
    ff([(72, 1.040), (120, 1.030)], 1.060, 1.014)
    ff([(40, 1.020), (70, 1.025)], 1.050, 1.012, not_="SG artıyor: k ≤ 0")
    ff([(30, 1.040), (50, 1.020), (90, 1.019)], 1.050, 1.012)

    # ---- ekstrakt
    kinds = ("LME", "DME", "bal")
    for y in (78.0, 95.0, 75.0, 100.0, 0.0):
        v.ekle("ppg", {"yield": y}, ref.ppg(y))
        v.ekle("points_kg_l", {"yield": y}, ref.points_kg_L(y))
    for k in kinds:
        for g in (1500, 453.592, 0, 25.5):
            v.ekle("g_to_ml", {"kind": k, "g": g}, ref.g_to_ml(k, g))
        for ml in (500, 236.588, 0, 10.25):
            v.ekle("ml_to_g", {"kind": k, "ml": ml}, ref.ml_to_g(k, ml))
    for a in kinds:
        for b in kinds:
            v.ekle("swap", {"kind_from": a, "kind_to": b, "g": 3000}, ref.swap(a, b, 3000))
    for k in kinds:
        for og, vol in ((1.050, 20), (1.080, 23), (1.030, 19.5), (1.0, 20)):
            v.ekle("extract_for_og", {"kind": k, "og": og, "v_final_l": vol}, ref.extract_for_og(k, og, vol))

    # ---- kısmi kaynatma
    def pb(og, vf, vk, loss=0.0, not_=None):
        girdi = {"og_final": og, "v_final": vf, "v_kettle_post": vk, "loss": loss}
        dene("partial_boil", girdi, lambda: ref.partial_boil(og, vf, vk, loss), not_)

    pb(1.050, 20, 10)
    pb(1.050, 20, 10, 1.0)
    pb(1.060, 23, 15, 1.5)
    pb(1.048, 19, 19, 0.0, "kazan = son hacim: eşitlik sınırı")
    pb(1.048, 19, 20, 0.0, "v_final < v_out: ValueError")
    pb(1.048, 19, 10, 10.0, "v_out = 0: ValueError")
    pb(1.048, 19, 10, 12.0, "v_out < 0: ValueError")
    pb(1.048, 19, 20, 1.0, "v_final = v_out")

    hops = [(28, 0.10, 60), (28, 0.10, 10)]
    for h, sg, vf in ((hops, 1.100, 20), (hops, 1.050, 20), ([(15, 0.12, 30), (20, 0.05, 5), (10, 0.07, 0)], 1.085, 23),
                      ([], 1.050, 20)):
        v.ekle("partial_boil_ibu", {"hops": [list(x) for x in h], "sg_kettle": sg, "v_final": vf},
               ref.partial_boil_ibu(h, sg, vf))

    for a in ((10, 30, 10, 20, 1.0), (10, 30, 10, 20, 0.95), (19, 80, 4, 20, 1.0), (5, 20, 15, 20, 0.9)):
        v.ekle("topoff_temp", dict(zip(("v_wort", "t_wort", "v_water", "t_target", "c_ratio"), a)),
               ref.topoff_temp(*a))

    # ---- tinseth_util (02) ve ftemp (02 §3.1): referansta tanımlı, 02 modülünden çağrılır
    for sg, t in ((1.050, 60), (1.050, 0), (1.100, 10), (1.030, 90), (1.050, 20.5)):
        v.ekle("tinseth_util", {"sg": sg, "t": t}, ref.tinseth_util(sg, t))
    for tc in (40, 60, 80, 99.9, 100, 105, 20):
        v.ekle("ftemp", {"tc": tc}, ref.ftemp(tc))

    # ---- no-chill
    for vol in (20, 40, 10, 5, 100):
        v.ekle("tau_h", {"v_l": vol, "tau20": 5.0}, ref.tau_h(vol, 5.0))
    v.ekle("tau_h", {"v_l": 40, "tau20": 4.4}, ref.tau_h(40, 4.4))

    def te(t0, ta, tau, dt=0.5, ts=40.0, not_=None):
        v.ekle("t_eq_min", {"t0": t0, "t_amb": ta, "tau_hours": tau, "dt": dt, "t_stop": ts},
               ref.t_eq_min(t0, ta, tau, dt, ts), not_)

    for tau in (3, 4, 6, 5):
        te(95, 20, tau)
    te(80, 20, 5)
    te(95, 20, 5, dt=0.01)
    te(95, 20, 5, dt=2.0)
    te(95, 20, 5, ts=60.0)
    te(95, 10, 6.3)
    te(100, 25, 5)
    te(35, 20, 5, not_="T0 < 40: sıfır")
    te(40, 20, 5, not_="T0 = 40: kesme sınırı, ilk adım sayılır")
    te(40.0001, 20, 5)

    hp = [(28, 0.10, 60), (28, 0.10, 0)]
    hp2 = [(30, 0.12, 60), (20, 0.06, 15), (25, 0.09, 5), (40, 0.04, 0)]
    for h, sg, vol, mode, kw in (
            (hp, 1.050, 20, "kural", {}),
            (hp, 1.050, 20, "model", {}),
            (hp, 1.050, 20, "model", {"T0": 80}),
            (hp, 1.050, 20, "model", {"T0": 95, "T_amb": 10}),
            (hp, 1.050, 20, "model", {"v_cube": 40}),
            (hp, 1.050, 20, "model", {"v_cube": 0}),
            (hp2, 1.065, 23, "kural", {}),
            (hp2, 1.065, 23, "model", {"T0": 90, "T_amb": 15, "v_cube": 25}),
            ([], 1.050, 20, "model", {})):
        girdi = {"hops": [list(x) for x in h], "sg": sg, "v": vol, "mode": mode}
        girdi.update(kw)
        base, nc, extra = ref.nochill_ibu(h, sg, vol, mode, **kw)
        v.ekle("nochill_ibu", girdi, [base, nc, extra])

    # T1.14g: yöntem seçimi, gösterim, durgunluk, basit no-chill kuralı
    okumalar = [
        ([(48, 1.030), (96, 1.018)], "iki okuma"),
        ([(48, 1.030)], "tek okuma"),
        ([(24, 1.046), (36, 1.042)], "gecikme öncesi/%30 yok"),
        ([(96, 1.018), (168, 1.013), (216, 1.013)], "durgun, hedefte"),
        ([(48, 1.030), (120, 1.024), (168, 1.024)], "durgun, yüksek"),
        ([(96, 1.018), (168, 1.014), (216, 1.013)], "1 puanlık durgunluk"),
        ([(48, 1.020), (94, 1.019)], "46 sa arayla, durgunluk değil"),
        ([(48, 1.030), (84, 1.014)], "geç okuma"),
        ([(50, 1.0125), (80, 1.0120)], "hedefe ulaştı"),
        ([(50, 1.0125), (98, 1.0135)], "48 sa tam, fark 100 birim"),
        ([(50, 1.0125), (98, 1.0136)], "fark 101 birim: durgun değil"),
        ([(50, 1.0125), (97.9, 1.0125)], "47.9 sa: durgun değil"),
        ([(96, 1.016), (144, 1.016)], "durgun, FG+0.004 sınırı (400 birim)"),
        ([(96, 1.0161), (144, 1.0161)], "durgun, sınırın 1 birim üstü"),
        ([(30, 1.040), (60, 1.0400)], "k ≈ pozitif, düşüş yeterli değil"),
        ([(13, 1.049), (14, 1.0495)], "k ≤ 0"),
        ([(60, 1.030), (36, 1.040)], "sırasız"),
        ([(5.0, 1.049), (8.0, 1.048)], "hepsi gecikme öncesi"),
    ]
    for rd, not_ in okumalar:
        girdi = {"readings": [list(p) for p in rd], "og": 1.050, "fg_prior": 1.012}
        dene("few_status", girdi, lambda rd=rd: ref.few_status(rd, 1.050, 1.012), not_)
        v.ekle("stall_check", {"readings": [list(p) for p in rd], "fg_prior": 1.012},
               ref.stall_check(rd, 1.012), not_)
    v.ekle("stall_check", {"readings": [], "fg_prior": 1.012}, ref.stall_check([], 1.012), "boş")
    v.ekle("few_status", {"readings": [], "og": 1.050, "fg_prior": 1.012, "lag": 12.0},
           ref.few_status([], 1.050, 1.012), "boş")
    v.ekle("few_status", {"readings": [[20, 1.040], [30, 1.030]], "og": 1.050, "fg_prior": 1.012, "lag": 24.0},
           ref.few_status([(20, 1.040), (30, 1.030)], 1.050, 1.012, 24.0), "lag 24")
    for dev, n in ((True, 0), (True, 3), (False, 2), (False, 0)):
        v.ekle("forecast_method", {"device_sufficient": dev, "manual_count": n}, ref.forecast_method(dev, n))
    for src in ("cihaz", "elle", "kagit"):
        for dev in (True, False):
            v.ekle("reading_role", {"source": src, "batch_has_device": dev}, ref.reading_role(src, dev))
    for t in (0, 5, 10, 60, 80.5):
        v.ekle("nochill_rule_time", {"t_min": t}, ref.nochill_rule_time(t))

    return v.yaz()
