"""Bölüm 07 parite vektörleri: Ev MVP ek hesapları (ADR-0003)."""
import math

from ortak import Vektor, yukle


def _sentetik(og, fg_true, k_true, lag, saat, adim=4):
    pts = []
    for i in range(0, saat * adim):
        t = i / adim
        sg = og if t < lag else fg_true + (og - fg_true) * math.exp(-k_true * (t - lag))
        noise = 0.0004 * math.sin(i * 1.7) + 0.0002 * math.sin(i * 0.31)
        pts.append((t, sg + noise))
    return pts


def uret():
    ref = yukle("07_ev_mvp_ek_tv.py")
    v = Vektor("07", "formuller/testler/07_ev_mvp_ek_tv.py")

    # 1. Fermantasyon tahmini
    og, fg_true, k_true = 1.052, 1.0110, 0.045
    pts = _sentetik(og, fg_true, k_true, 12, 96)
    act = [p for p in pts if p[0] >= 24]
    ham = [[0.2, 1.050], [0.7, 1.052], [0.9, 1.049], [1.1, 1.040], [1.5, 1.041], [2.4, 1.030],
           [2.5, 1.032], [2.6, 1.031], [2.9, 1.0295]]
    v.ekle("hourly_median", {"points": ham}, ref.hourly_median([tuple(p) for p in ham]), "tek ve çift sayılı kovalar")
    v.ekle("hourly_median", {"points": [[5.0, 1.01]]}, ref.hourly_median([(5.0, 1.01)]))
    v.ekle("hourly_median", {"points": [[3.9, 1.02], [3.1, 1.01]]}, ref.hourly_median([(3.9, 1.02), (3.1, 1.01)]),
           "sırasız")

    def fit(p, lo, hi, steps, not_=None):
        best, table = ref.fit_decay(p, lo, hi, steps)
        v.ekle("fit_decay", {"pts": p, "fg_lo": lo, "fg_hi": hi, "steps": steps},
               {"f": best[0], "a": best[1], "k": best[2], "rss": best[3], "n_table": len(table),
                "table_ilk": list(table[0]), "table_son": list(table[-1])}, not_)

    fit_pts = ref.hourly_median([p for p in act if p[0] < 72])
    fit(fit_pts, 1.012 - 0.015, fit_pts[-1][1] - 0.0002, 400)
    sade = [(h + 0.5, 1.010 + 0.040 * math.exp(-0.05 * (h + 0.5))) for h in range(30)]
    fit(sade, 1.000, 1.0099, 100, "gürültüsüz")

    def fc(points, og_, fgr, **kw):
        girdi = {"points": points, "og": og_, "fg_recipe": fgr}
        girdi.update(kw)
        v.ekle("forecast", girdi, ref.forecast([tuple(p) for p in points], og_, fgr, **kw))

    fc([p for p in act if p[0] < 72], og, 1.012)
    fc([p for p in act if p[0] < 30], og, 1.012)
    fc([p for p in act if p[0] < 48], og, 1.012)
    fc([p for p in act if p[0] < 96], og, 1.012)
    fc([p for p in act if p[0] < 72], og, 1.030)
    fc([p for p in act if p[0] < 72], og, 1.012, eps=0.001)
    fc([p for p in act if p[0] < 72], og, 1.012, min_n=40)
    p2 = _sentetik(1.060, 1.014, 0.03, 8, 120)
    fc([p for p in p2 if 20 <= p[0] < 90], 1.060, 1.014)
    fc([p for p in p2 if 20 <= p[0] < 60], 1.060, 1.014, min_drop=0.5)

    # T1.12 C3, C5, C7: eşikler tam sınırda (tam sayı gravite birimi), min_n, geçersiz seri
    def seri(son, n=12):
        return [[i + 0.2, round(son + (1.09 - son) * (1 - i / (n - 1)) ** 2, 6)] for i in range(n)]

    fc(seri(1.0681), 1.090, 1.017)       # tam %30 düşüş: yeterli (f64 "yetersiz" derdi)
    fc(seri(1.06811), 1.090, 1.017)      # bir birim eksik: yetersiz
    fc(seri(1.06809), 1.090, 1.017)      # bir birim fazla: yeterli
    for og_, sg_, fgr, md in ((1.090, 1.0681, 1.017, 0.30), (1.090, 1.06811, 1.017, 0.30), (1.090, 1.06809, 1.017, 0.30),
                              (1.048, 1.0452, 1.012, 0.30), (1.048, 1.04521, 1.012, 0.30), (1.060, 1.045, 1.010, 0.50),
                              (1.060, 1.035, 1.010, 0.50), (1.060, 1.0349, 1.010, 0.50)):
        v.ekle("drop_enough", {"og": og_, "sg_now": sg_, "fg_recipe": fgr, "min_drop": md},
               ref.drop_enough(og_, sg_, fgr, md), "tam sınır, bir birim altı/üstü")
    for f_, fr in ((1.022, 1.012), (1.02201, 1.012), (1.02199, 1.012), (1.002, 1.012), (1.00199, 1.012),
                   (1.00201, 1.012), (1.0110, 1.012), (1.0140, 1.012), (1.021, 1.0110), (1.0210, 1.0110000)):
        v.ekle("fg_warning", {"fg_inf": f_, "fg_recipe": fr}, ref.fg_warning(f_, fr), "|Δu| > 1000: tam 0.010 uyarı vermez")
    fc([p for p in act if p[0] < 72], og, 1.012, min_n=48)     # tam 48 saatlik nokta: yeterli
    fc([p for p in act if p[0] < 72], og, 1.012, min_n=49)     # bir eksik: yetersiz
    for mn in (0, 1, 2):
        fc([p for p in act if p[0] < 72], og, 1.012, min_n=mn)
    fc([[i + 0.2, 1.040] for i in range(14)], 1.050, 1.010)    # düz SG
    fc([[i + 0.2, 1.040 + 0.001 * i] for i in range(14)], 1.050, 1.010)   # yükselen SG
    fc([[i + 0.2, g] for i, g in enumerate([1.050, 1.044, 1.039, 1.035, 1.032, 1.030, 1.029, 1.0285, 1.0284, 1.0286,
                                              1.0290, 1.0296, 1.0304, 1.0314])], 1.050, 1.010)   # k ≤ 0 ızgara noktaları atlanır
    fc([p for p in act if p[0] < 72], og, 1.0220)               # FG∞ tariften ~0.010 uzak: uyarı sınırı çevresi
    fc([p for p in act if p[0] < 72], og, 1.0235)
    fc([p for p in act if p[0] < 72], og, 1.0005)
    fc([p for p in act if p[0] < 72], og, 1.012, eps=0.0001)
    hm_neg = [[-0.5, 1.05], [-1.5, 1.06], [0.5, 1.04], [-0.2, 1.051]]
    v.ekle("hourly_median", {"points": hm_neg}, ref.hourly_median([tuple(p) for p in hm_neg]), "negatif zaman: trunc, floor değil")
    kenar = [(i + 0.5, 1.0625 + 0.04 * math.exp(-0.2 * i)) for i in range(12)]
    best, table = ref.fit_decay(kenar, 1.0, 1.0625, 400)
    v.ekle("fit_decay", {"pts": kenar, "fg_lo": 1.0, "fg_hi": 1.0625, "steps": 400},
           {"f": best[0], "a": best[1], "k": best[2], "rss": best[3], "n_table": len(table),
            "table_ilk": list(table[0]), "table_son": list(table[-1])}, "ızgara üst ucu, SG − FG∞ > 0 sınırında")

    # 2. Cihaz kalibrasyonu
    cal = [(25.0, 1.000), (38.2, 1.020), (49.0, 1.040), (58.7, 1.060), (66.5, 1.080)]
    cx, cy = [a for a, _ in cal], [g for _, g in cal]
    for xs, ys, deg in [(cx, cy, 2), (cx, cy, 1), (cx, cy, 3),
                        ([1.0, 2.0, 3.0, 4.0], [3.0, 5.0, 7.0, 9.0], 1)]:
        v.ekle("polyfit", {"xs": xs, "ys": ys, "deg": deg}, ref.polyfit(xs, ys, deg))
    co2 = ref.polyfit(cx, cy, 2)
    for x in (0.0, 25.0, 45.0, 66.5):
        v.ekle("peval", {"co": co2, "x": x}, ref.peval(co2, x))
    v.ekle("peval", {"co": [2.5], "x": 7.0}, ref.peval([2.5], 7.0), "sabit polinom")

    def isp(points, deg):
        girdi = {"points": [list(p) for p in points], "deg": deg}
        try:
            co, r2, mx, uy = ref.ispindel_cal(points, deg)
            v.ekle("ispindel_cal", girdi, {"co": co, "r2": r2, "max_res": mx, "uyari": uy})
        except ValueError:
            v.ekle("ispindel_cal", girdi, {"hata": "ValueError"}, "yetersiz nokta / farklı açı / sabit SG")

    isp(cal, 2)
    isp(cal, 3)
    isp(cal, 1)
    isp(cal[:3], 2)
    isp(cal[:4], 2)
    isp(cal[:3], 1)
    isp([(25.0, 1.000), (38.2, 1.021), (49.0, 1.038), (58.7, 1.063), (66.5, 1.079), (70.0, 1.090)], 2)

    # T1.12 C4: tekil sistemler, dar aralık, aralık dışı değerleme
    isp([(30.0, 1.000), (30.0, 1.000), (50.0, 1.040), (50.0, 1.040)], 2)       # yalnız iki farklı açı
    isp([(30.0, 1.000), (30.0, 1.000), (30.0, 1.000), (30.0, 1.000)], 2)       # hep aynı açı
    isp([(30.0, 1.000), (40.0, 1.000), (50.0, 1.000), (60.0, 1.000)], 2)       # hep aynı SG
    isp([(30.0, 1.000), (30.0, 1.001), (50.0, 1.040)], 1)                       # 3 nokta, 2 farklı açı
    isp([(30.0, 1.000), (40.0, 1.020), (50.0, 1.040)], 1)                       # 3 farklı açı, derece 1: yeterli
    isp([(60.0, 1.0300), (62.0, 1.0340), (64.5, 1.0390), (66.0, 1.0421)], 2)  # açı aralığı 6° (dar)
    isp([(30.0, 1.0000), (33.0, 1.0050), (36.0, 1.0100), (40.0, 1.0160)], 1)  # aralık 10° tam sınır: uyarı yok
    isp([(30.0, 1.0000), (33.0, 1.0050), (36.0, 1.0100), (39.9999999, 1.0160)], 1)   # 10° - 1e-7: uyarı
    isp([(30.0, 1.0000), (33.0, 1.0050), (36.0, 1.0100), (39.9999999995, 1.0160)], 1)  # 10° - 5e-10: uyarı yok
    for ang in (25.0, 45.0, 66.5, 24.9999999, 24.9999999995, 66.5000000005, 66.51, 10.0, 90.0):
        v.ekle("ispindel_eval", {"co": co2, "a_min": 25.0, "a_max": 66.5, "angle": ang},
               ref.ispindel_eval(co2, 25.0, 66.5, ang), "aralık dışı uyarı, 1e-9 tolerans")

    def tilt(points):
        e, s = ref.tilt_cal(points)
        v.ekle("tilt_cal", {"points": [list(p) for p in points]}, [e, s])

    tilt([(1.0545, 1.0520)])
    tilt([(1.0020, 1.0000), (1.0545, 1.0520)])
    tilt([(1.0020, 1.0000), (1.0300, 1.0285), (1.0545, 1.0520)])
    tilt([(1.0000, 1.0000)])

    # 3. Isınma ve soğuma
    adlar = ("water_l", "grain_kg", "kettle_kg", "t0", "t1", "power_kw", "eff")
    for a in [(15, 0, 8, 15, 72, 2.5, 0.90), (15, 5, 8, 64, 72, 2.5, 0.90), (28, 0, 8, 76, 100, 2.5, 0.90),
              (30, 6.5, 12, 10, 78, 3.5, 0.40), (20, 0, 0, 20, 20, 2.0, 0.9), (10, 0, 5, 90, 60, 2.0, 0.9)]:
        v.ekle("heat_time_min", dict(zip(adlar, a)), ref.heat_time_min(*a))
    for p, e in [(2.5, 0.85), (3.5, 0.9), (1.0, 0.5), (0.0, 0.8)]:
        v.ekle("boiloff_l_per_h", {"power_kw": p, "eff_boil": e}, ref.boiloff_L_per_h(p, e))
    for a in [(100, 15, 20, 25), (95, 10, 30, 18), (100, 25, 40, 12.5)]:
        v.ekle("tau_from_batch", dict(zip(("t0", "tc", "t", "minutes"), a)), ref.tau_from_batch(*a))
    tau = ref.tau_from_batch(100, 15, 20, 25)
    for a in [(100, 18, 20, tau, 40, 24), (100, 15, 20, tau, 24, 24), (95, 12, 25, 9.0, 24, 24),
              (90, 20, 30, 7.5, 12, 24)]:
        v.ekle("cool_time_immersion", dict(zip(("t0", "tc", "t", "tau", "vol_l", "vol_ref_l"), a)),
               ref.cool_time_immersion(*a))
    for a in [(23, 4), (20, 2.5), (0, 4)]:
        v.ekle("cool_time_flow", {"vol_l": a[0], "flow_l_min": a[1]}, ref.cool_time_flow(*a))

    # 4. Ölçekleme
    ing = {"Pale Ale malt": (4.50, "mash"), "Munich": (0.45, "mash"), "Crystal 40": (0.30, "mash"),
           "Magnum": (19, "hop_ibu"), "Cascade": (30, "hop_ibu"), "Citra wp": (50, "hop_aroma"),
           "Gypsum": (5.0, "tuz"), "Dekstroz": (0.25, "seker"), "Whirlfloc": (1.0, "misc"),
           "US-05": (1, "maya_paket")}
    for v0, v1, e0, e1, mode in [(23, 40, 0.72, 0.78, "hedef"), (23, 40, 0.72, 0.78, "oran"),
                                 (23, 23, 0.72, 0.72, "hedef"), (40, 19, 0.80, 0.68, "hedef"),
                                 (20, 5, 0.75, 0.75, "oran")]:
        out = ref.scale_recipe(ing, v0, v1, e0, e1, mode)
        v.ekle("scale_recipe", {"ing": {k: list(x) for k, x in ing.items()}, "v0": v0, "v1": v1,
                                "eff0": e0, "eff1": e1, "mode": mode}, out)
    out = ref.scale_recipe(ing, 23, 40, 0.72, 0.78)
    for q, step in [(out["Pale Ale malt"], 0.01), (out["Magnum"], 1), (out["Gypsum"], 0.1), (0.125, 0.25),
                    (0.375, 0.25), (2.5, 1), (3.5, 1), (0.0, 0.1), (8.6957, 0.1), (0.35, 0.1), (0.05, 0.1),
                    (0.0499, 0.1), (1.5, 1), (0.5, 1), (0.49, 1), (7.825, 0.01), (-0.3, 0.1)]:
        v.ekle("round_to", {"q": q, "step": step}, ref.round_to(q, step))

    # 4b. T1.14g: çarpan kuralları (§4.1–4.2) ve ölçekleme kaydı (§4.4)
    for tip in ("mash", "seker", "hop_ibu", "hop_aroma", "tuz", "misc", "maya_paket"):
        for mode in ("hedef", "oran"):
            for pb, wr in ((False, None), (True, 1.4), (False, 0.8)):
                girdi = {"tip": tip, "v0": 23, "v1": 40, "eff0": 0.72, "eff1": 0.78, "mode": mode,
                         "per_batch": pb, "water_ratio": wr}
                v.ekle("scale_factor", girdi, ref.scale_factor(tip, 23, 40, 0.72, 0.78, mode, pb, wr))
    v.ekle("scale_factor", {"tip": "mash", "v0": 40, "v1": 19, "eff0": 0.80, "eff1": 0.68, "mode": "hedef",
                            "per_batch": False, "water_ratio": None},
           ref.scale_factor("mash", 40, 19, 0.80, 0.68, "hedef"))
    for need, per in ((200.0, 100.0), (200.0000001, 100.0), (200.0000000001, 100.0), (50.0, 100.0), (0.0, 100.0),
                      (305.0, 100.0), (99.9999999999, 100.0), (450.2, 200.0),
                      (200.0000005, 100.0), (200.0000002, 100.0), (200.0000010, 100.0), (100.0, 100.0), (100.0000001, 100.0)):
        v.ekle("yeast_packs", {"cells_needed_billion": need, "cells_per_pack_billion": per},
               ref.yeast_packs(need, per), "yukarı yuvarlama, 1e-9 tolerans, en az 1")
    for scope, mode, v0, v1, e0, e1, ver in (("tarif", "hedef", 23, 40, 0.72, 0.78, 3), ("parti", "hedef", 23, 40, 0.72, 0.78, 3),
                                             ("tarif", "oran", 20, 5, 0.75, 0.75, 1), ("parti", "oran", 20, 20, 0.7, 0.7, 0)):
        v.ekle("scaling_record", {"scope": scope, "mode": mode, "v0": v0, "v1": v1, "eff0": e0, "eff1": e1,
                                  "recipe_version": ver},
               ref.scaling_record(scope, mode, v0, v1, e0, e1, ver))
    for scope, mode in (("seri", "hedef"), ("tarif", "karisik")):
        v.ekle("scaling_record", {"scope": scope, "mode": mode, "v0": 20, "v1": 10, "eff0": 0.7, "eff1": 0.7,
                                  "recipe_version": 1}, {"hata": "ValueError"}, "geçersiz kapsam/mod")

    # 5. İyon dengesi ve sertlik
    sular = [{"Ca": 38, "Mg": 8, "Na": 18, "SO4": 32, "Cl": 40, "HCO3": 110},
             {"Ca": 100, "Mg": 10, "Na": 10, "SO4": 250, "Cl": 50, "HCO3": 30},
             {"Ca": 50, "Mg": 5, "Na": 5, "K": 2, "SO4": 20, "Cl": 10, "HCO3": 200, "CO3": 3, "NO3": 5},
             {"Ca": 10, "Mg": 1, "Na": 2, "SO4": 5, "Cl": 5, "HCO3": 10},
             {"Ca": 150, "Mg": 30, "Na": 80, "SO4": 20, "Cl": 20, "HCO3": 20},
             {"Ca": 20.039, "Cl": 35.453}, {"Ca": 20.039 * 1.05, "Cl": 35.453},
             # T1.12 C7: sınıf sınırları (denge %5, %10), K, CO3, NO3
             {"Ca": 20.039 * 2.1, "Cl": 35.453 * 1.9},                    # tam %5: iyi
             {"Ca": 20.039 * 2.1002, "Cl": 35.453 * 1.9},                 # %5'in hemen üstü: kabul
             {"Ca": 20.039 * 2.0998, "Cl": 35.453 * 1.9},                 # %5'in hemen altı: iyi
             {"Ca": 20.039 * 1.1, "Cl": 35.453 * 0.9},                    # tam %10: kabul
             {"Ca": 20.039 * 1.1003, "Cl": 35.453 * 0.9},                 # %10'un üstü: kontrol et
             {"Ca": 20.039 * 1.0997, "Cl": 35.453 * 0.9},                 # %10'un altı: kabul
             {"Ca": 20.039 * 1.115, "Cl": 35.453 * 0.885},                # ~%11.5: kontrol et (sınır 12 olsaydı kabul)
             {"Ca": 20.039 * 1.056, "Cl": 35.453 * 0.944},                # ~%5.6: kabul (sınır 6 olsaydı iyi)
             {"Ca": 40, "Mg": 10, "Na": 15, "K": 12, "SO4": 30, "Cl": 25, "HCO3": 90, "CO3": 4, "NO3": 12},
             {"Ca": 40, "Mg": 10, "Na": 15, "K": 30, "SO4": 30, "Cl": 25, "HCO3": 90, "CO3": 12, "NO3": 40},
             {"K": 39.098, "NO3": 62.004}, {"K": 39.098 * 1.05, "NO3": 62.004},
             {"Na": 22.990, "CO3": 30.004}]
    for w in sular:
        c, a, b, s = ref.ion_balance(w)
        v.ekle("ion_balance", {"p": w}, {"cation": c, "anion": a, "balance_pct": b, "class": s})
    for ca, mg in [(38, 8), (0, 0), (100, 20), (50, 0)]:
        v.ekle("hardness_caco3", {"ca": ca, "mg": mg}, ref.hardness_caco3(ca, mg))
    for h in (0, 90.2, 110, 250):
        v.ekle("alk_caco3_from_hco3", {"hco3": h}, ref.alk_caco3_from_hco3(h))
    for x in (0, 127.8, 90.2, 300):
        v.ekle("caco3_to", {"x": x}, ref.caco3_to(x))

    # 6. Kayıplar
    for g in [{"pellet": 49}, {"pellet": 60, "cryo": 20}, {"yaprak": 100},
              {"pellet": 30, "yaprak": 20, "cryo": 10}, {}]:
        v.ekle("kettle_hop_loss", {"g_by_form": g}, ref.kettle_hop_loss(g))
    for g in [{"pellet": 60, "cryo": 20}, {"yaprak": 45}, {"pellet": 100}, {}]:
        v.ekle("dryhop_loss", {"g_by_form": g}, ref.dryhop_loss(g))
    for kg, d, s in [(2.0, 1.05, 0.20), (0.0, 1.05, 0.20), (5.0, 1.10, 0.15), (1.5, 1.0, 0.30)]:
        a, b = ref.fruit(kg, d, s)
        v.ekle("fruit", {"kg": kg, "density": d, "sediment": s}, [a, b])
    a, b = ref.fruit(3.0)
    v.ekle("fruit", {"kg": 3.0, "density": 1.05, "sediment": 0.20}, [a, b], "varsayılanlar")
    return v.yaz()
