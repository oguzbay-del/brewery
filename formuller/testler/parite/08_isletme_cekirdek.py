"""Bölüm 08 (işletme çekirdek hesapları) parite vektörleri (ADR-0003)."""
from decimal import Decimal

from ortak import Vektor, dene, yukle


def dd(x):
    """Girdi: Decimal -> düz metin (ölçek korunur, bilimsel gösterim yok)."""
    return format(x, "f")


def dj(x):
    """Beklenen: Decimal -> {"ondalik": metin}; Rust sayısal eşitle karşılaştırır (ölçek önemsiz)."""
    return {"ondalik": format(x, "f")}


def uret():
    ref = yukle("08_isletme_cekirdek_tv.py")
    v = Vektor("08", "formuller/testler/08_isletme_cekirdek_tv.py")

    for p in (0.0, 2.4, 2.8, 12.0, 20.0):
        v.ekle("plato_to_sg", {"p": p}, ref.plato_to_sg(p))

    # 1. Maliyet (T1.12 C2, C6: Decimal; satır başına 2 ondalık yarım yukarı; toplam = satırların toplamı)
    D = Decimal
    lots = [["L-A", "3.0", "1.10"], ["L-B", "25.0", "1.25"]]
    tozlu = [["L-1", "0.1", "1.00"], ["L-2", "0.2", "1.00"], ["L-3", "5.0", "1.00"]]
    for lotlar, qty, not_ in (
        (lots, "4.5", "iki lota yayılır; 1.5 x 1.25 = 1.875 -> 1.88"),
        (lots, "2.0", "ilk lot yeter"),
        (lots, "3.0", "ilk lot tam biter"),
        (lots, "28.0", "tüm stok, eksik yok"),
        (lots, "30.0", "eksik 2.0"),
        ([["L-2411", "0.20", "52.0"]], "0.30", "stok yetmez (TV1.2)"),
        (lots, "0.0", "sıfır ihtiyaç"),
        (lots, "-1.0", "negatif ihtiyaç"),
        ([], "5.0", "boş stok"),
        ([["L-0", "0.0", "9.0"], ["L-1", "2.0", "1.5"]], "1.0", "boş lot atlanır"),
        ([["L-1", "0.1", "1.00"], ["L-2", "0.2", "1.00"]], "0.3", "0.1 + 0.2 tam biter: toz satır ve eksik yok (C2)"),
        (tozlu, "0.3", "0.1 + 0.2 ihtiyaç: üçüncü lottan satır açılmaz (C2)"),
        ([["L-1", "0.1", "1.00"], ["L-2", "0.2", "1.00"]], "0.4", "eksik tam 0.1"),
        ([["L-1", "0.1", "0.05"], ["L-2", "0.1", "0.05"], ["L-3", "0.1", "0.05"]], "0.3", "0.005 satırlar: 0.01 x 3, toplam 0.03"),
        ([["L-1", "1.0", "1.005"], ["L-2", "1.0", "2.675"], ["L-3", "1.0", "0.125"]], "3", "yarım yukarı: 1.01, 2.68, 0.13"),
        ([["L-1", "1.0", "0.004"], ["L-2", "1.0", "0.004"], ["L-3", "1.0", "0.004"]], "3", "satırlar 0.00, toplam 0.00"),
    ):
        rows, cost, miss = ref.fifo_consume([(a, D(b), D(c_)) for a, b, c_ in lotlar], D(qty))
        v.ekle("fifo_consume", {"lots": lotlar, "qty": qty},
               {"rows": [[r_[0], dj(r_[1]), dj(r_[2]), dj(r_[3])] for r_ in rows], "cost": dj(cost), "missing": dj(miss)}, not_)

    for inv, qty, ship in (("1.20", "25", "4.0"), ("1.20", "25", "0.0"), ("52.0", "0.5", "1.3"), ("3.75", "100", "12.5")):
        v.ekle("landed_price", {"invoice_price": inv, "qty": qty, "shipping_share": ship},
               dj(ref.landed_price(D(inv), D(qty), D(ship))))
    for qty in ("0", "-5"):
        v.ekle("landed_price", {"invoice_price": "1.0", "qty": qty, "shipping_share": "0"}, {"hata": "ValueError"}, "miktar <= 0")
    for kwh, pr in (("4.99", "2.6"), ("0.0", "2.6"), ("120.0", "0.18"), ("3.333", "0.1234")):
        v.ekle("energy_cost_electric", {"kwh": kwh, "price_kwh": pr}, dj(ref.energy_cost_electric(D(kwh), D(pr))))
    kj_strike = (15 * 4.186 + 8 * 0.5) * (72 - 15)
    kj_boil = (28 * 4.186 + 8 * 0.5) * (100 - 76)
    for kj, eff in ((kj_strike + kj_boil, 0.90), (0.0, 0.9), (1000.0, 1.0), (50000.0, 0.45)):
        v.ekle("kwh_from_heat", {"kj": kj, "eff": eff}, dene(ref.kwh_from_heat, kj, eff))
    for kj, eff, hu in ((kj_strike + kj_boil, 0.40, 34500.0), (1000.0, 1.0, 34500.0), (50000.0, 0.8, 36000.0)):
        v.ekle("gas_m3_from_heat", {"kj": kj, "eff": eff, "hu_kj_m3": hu}, dene(ref.gas_m3_from_heat, kj, eff, hu))
    adlar = ("ingredient_cost", "water_l", "water_price_m3", "energy_cost", "pack_units", "pack_unit_cost", "packaged_l")
    for a in (("18.40", "35.0", "30.0", "12.9740", "60", "0.35", "20.0"), ("0.0", "0.0", "0.0", "0.0", "0", "0.0", "10.0"),
              ("100.0", "500.0", "1.2", "40.0", "1200", "0.4", "950.0"), ("7.005", "12.5", "2.333", "1.005", "3", "0.125", "3.0"),
              ("0.004", "1.0", "4.0", "0.004", "1", "0.004", "1.0"),             # kalemler 0.00, toplam 0.00 (C6)
              ("1.005", "0", "0", "2.675", "0", "0", "7"),                      # yarım yukarı: 1.01 + 2.68
              ("0.125", "0", "0", "0.375", "0", "0", "3"),                      # 0.13 + 0.38 = 0.51
              ("1000.00", "35000", "30.0", "0", "0", "0", "100"),               # su 1050.00 (sabit %0.01 hatası görünür)
              ("0", "123456", "2.3571", "0", "0", "0", "1000"),                 # su 290.99 civarı
              ("10.00", "0", "0", "0", "1", "0.005", "1.5"),                    # ambalaj 0.005 -> 0.01; litre 6.673
              ("10.00", "0", "0", "0", "0", "0", "3")):                         # litre 3.333
        arg = dict(zip(adlar, a))
        r_ = ref.batch_cost(*[D(x) for x in a])
        v.ekle("batch_cost", arg, {"malzeme": dj(r_["malzeme"]), "su": dj(r_["su"]), "enerji": dj(r_["enerji"]),
                                    "ambalaj": dj(r_["ambalaj"]), "toplam": dj(r_["toplam"]), "litre": dj(r_["litre"])})
    for pl in ("0", "-2"):
        v.ekle("batch_cost", dict(zip(adlar, ("10", "0", "0", "0", "0", "0", pl))), {"hata": "ValueError"}, "ambalajlanan hacim <= 0")

    # 2. Harmanlama
    a = dict(V=960, ABV=5.4, IBU=38, SRM=7.0, CO2=1.0, P=2.8)
    b = dict(V=400, ABV=4.6, IBU=22, SRM=4.0, CO2=1.0, P=2.4)
    w = dict(V=100.5, ABV=0.0, IBU=0.0, SRM=0.0, CO2=0.0, P=0.0)
    c = dict(V=250, ABV=9.5, IBU=60, SRM=25.0, CO2=2.4, P=6.1)
    for parts in ([a, b], [a, w], [a], [a, b, c], [c, w]):
        v.ekle("blend", {"parts": parts}, dene(ref.blend, parts))
    for v1, x1, x2, xt in ((960, 5.4, 4.6, 5.2), (960, 5.4, 0.0, 5.0), (500, 20.0, 40.0, 30.0),
                            (960, 5.4, 4.6, 5.4), (960, 5.4, 4.6, 4.6), (960, 5.4, 4.6, 6.0), (960, 5.4, 4.6, 4.0),
                            (960, 5.4, 4.6, 4.6 + 5e-10), (960, 5.4, 4.6, 5.4 - 5e-10), (960, 5.4, 4.6, 4.6 + 2e-9),
                            (960, 5.4, 4.6, 5.4 - 2e-9), (960, 4.6, 5.4, 5.0), (500, 20.0, 40.0, 20.0), (500, 20.0, 40.0, 40.0)):
        v.ekle("blend_volume_for_target", {"v1": v1, "x1": x1, "x2": x2, "xt": xt}, dene(ref.blend_volume_for_target, v1, x1, x2, xt),
               "hedef aralık dışı/uçta ise null")

    # 3. Maya
    for pitch, vl, p in ((0.75, 1000, 12.0), (1.5, 20, 15.0), (0.5, 0, 10.0)):
        v.ekle("cells_needed_billion", {"pitch_m_ml_p": pitch, "v_l": vl, "p": p}, dene(ref.cells_needed_billion, pitch, vl, p))
    for tot, sq, dil in ((230, 5, 100), (80, 5, 1), (0, 5, 10), (410, 10, 50)):
        v.ekle("hemocytometer", {"total_counted": tot, "squares_counted": sq, "dilution": dil}, dene(ref.hemocytometer, tot, sq, dil))
    for un, st in ((212, 18), (100, 0), (0, 40), (95, 5)):
        v.ekle("viability_stain", {"unstained": un, "stained": st}, dene(ref.viability_stain, un, st))
    n = dene(ref.cells_needed_billion, 0.75, 1000, 12.0)
    d = dene(ref.hemocytometer, 230, 5, 100)
    via = dene(ref.viability_stain, 212, 18)
    for cells, dens, vi, ddef in ((n, d, via, 1.0e9), (n, None, 0.90, 1.0e9), (n, 0.0, 0.90, 1.0e9),
                                   (n, None, 1.0, 1.0e9), (100.0, 2.0e9, 0.8, 1.0e9), (100.0, None, 0.5, 5.0e8)):
        v.ekle("slurry_volume_ml", {"cells_billion": cells, "density_cells_ml": dens, "viability": vi, "default_density": ddef},
               dene(ref.slurry_volume_mL, cells, dens, vi, ddef), "yoğunluk null/0 ise varsayılan")

    # 4. Profesyonel verim
    for vl, p, hot in ((1040, 12.0, True), (1040, 12.0, False), (20.0, 0.0, True), (500.0, 16.5, True)):
        v.ekle("extract_kg", {"v_l": vl, "p": p, "hot": hot}, dene(ref.extract_kg, vl, p, hot))
    for vl, p, mk, hot in ((1040, 12.0, 170, True), (1040, 12.0, 170, False), (20.0, 14.0, 4.0, True)):
        v.ekle("brewhouse_yield", {"v_l": vl, "p": p, "malt_kg": mk, "hot": hot}, dene(ref.brewhouse_yield, vl, p, mk, hot))
    for fg, mo, fc in ((81.5, 4.3, 0.0), (81.5, 4.3, 1.5), (80.0, 0.0, 2.0), (82.0, 6.0, 0.0)):
        v.ekle("lab_extract_as_is", {"fgdb": fg, "moisture": mo, "fc_diff": fc}, ref.lab_extract_as_is(fg, mo, fc))
    lab = ref.lab_extract_as_is(81.5, 4.3)
    for gk, ex, mo, mw in ((170, lab, 4.3, 520), (5.0, 78.0, 4.0, 15.0), (5.0, 0.0, 4.0, 15.0)):
        v.ekle("first_wort_max_p", {"grain_kg": gk, "extract_as_is_pct": ex, "moisture_pct": mo, "mash_water_l": mw},
               dene(ref.first_wort_max_P, gk, ex, mo, mw))
    for m, mx in ((19.6, 20.09), (20.0, 20.0), (0.0, 18.0)):
        v.ekle("conversion_eff", {"fw_measured_p": m, "fw_max_p": mx}, dene(ref.conversion_eff, m, mx))
    # T1.17: laboratuvara oran (§4.2) ve süzme verimi (§4.3)
    for b, lx, ce in ((73.88, 78.0, 0.975), (73.88, 78.0, 1.0), (60.0, 78.0, 0.8), (80.0, 80.0, 1.0), (0.0, 78.0, 0.9),
                      (73.88, 78.0, 1.2)):
        v.ekle("lab_ratio", {"brewhouse_pct": b, "lab_extract_pct": lx}, dene(ref.lab_ratio, b, lx))
        v.ekle("lauter_eff", {"brewhouse_pct": b, "lab_extract_pct": lx, "conversion": ce},
               dene(ref.lauter_eff, b, lx, ce))

    # 5. Malt analizi
    for fg, mo, fc in ((81.5, 4.3, 1.5), (81.0, 4.0, 2.0), (80.0, 0.0, 0.0), (85.2, 5.5, 1.0), (70.0, 6.0, 3.0)):
        v.ekle("malt_from_analysis", {"fgdb": fg, "moisture": mo, "fc_diff": fc}, ref.malt_from_analysis(fg, mo, fc))
    for ebc in (3.5, 0.0, 25.0, 150.0):
        v.ekle("ebc_from_lab", {"ebc": ebc}, ref.ebc_from_lab(ebc))

    # T1.14g: uyarı kuralları (§3.4, §5.4)
    for gen, gun, via, mx, hist in (
            (1, 3, 0.95, 8, False), (8, 14, 0.80, 8, False), (9, 15, 0.79, 8, False), (3, 20, None, 8, False),
            (5, 2, 0.5, 8, True), (6, 14, 0.9, 5, False), (12, 30, 0.1, 8, True), (0, 0, 1.0, 8, False),
            (2, 14.5, None, 8, False), (4, 7, 0.7999999, 8, False), (4, 7, 230 / 288, 8, False),
            (4, 7, 0.80, 8, False), (4, 7, 0.8 - 5e-10, 8, False), (4, 7, 0.8 - 2e-9, 8, False),
            (4, 14 + 5e-10, None, 8, False), (4, 14 + 2e-9, None, 8, False), (8, 7, None, 8, False), (9, 7, None, 8, False),
            (9, 7, 0.5, 8, True)):
        v.ekle("yeast_reuse_warnings", {"generation": gen, "days_since_harvest": gun, "viability": via,
                                        "max_generation": mx, "history_differs": hist},
               ref.yeast_reuse_warnings(gen, gun, via, mx, hist), "kod listesi, tablo sırası")
    for gr, thr in (([(4.5, 35.0), (0.5, 0.0)], 35.0), ([(5.0, 40.0)], 35.0), ([(4.0, 30.0), (1.0, 60.0)], 35.0),
                    ([(4.5, 50.0), (0.45, 0.0), (0.3, 0.0)], 35.0), ([(2.0, 100.0), (3.0, 0.0)], 35.0),
                    ([(1.0, 35.0)], 35.0), ([(3.0, 20.0), (3.0, 50.0)], 35.0), ([], 35.0), ([(0.0, 10.0)], 35.0),
                    ([(5.0, 30.0)], 25.0),
                    # T1.12 C1: telafili toplam ve 1e-9 toleransı
                    ([(1.1, 35.0), (3.79, 35.0), (3.87, 35.0)], 35.0),
                    ([(1.0, 35.0 - 5e-10)], 35.0), ([(1.0, 35.0 - 2e-9)], 35.0), ([(1.0, 35.0 + 2e-9)], 35.0),
                    ([(2.0, 34.0), (2.0, 36.0)], 35.0), ([(0.1, 35.0), (0.2, 35.0), (0.3, 35.0), (0.4, 35.0)], 35.0),
                    ([(0.7, 35.0), (0.1, 35.0), (2.2, 35.0)], 35.0), ([(1.0, 34.99999)], 35.0)):
        res = ref.diastatic_check(gr, thr)
        v.ekle("diastatic_check", {"grains": [list(g) for g in gr], "threshold": thr}, res,
               "ağırlıklı ortalama < eşik; diastatik gücü olmayan tahıl 0")
    n = "T1.16: hata"
    v.ekle("kwh_from_heat", {"kj": 1000.0, "eff": 0.0}, dene(ref.kwh_from_heat, 1000.0, 0.0), "verim 0: " + n)
    v.ekle("gas_m3_from_heat", {"kj": 1000.0, "eff": 0.0, "hu_kj_m3": 34500.0}, dene(ref.gas_m3_from_heat, 1000.0, 0.0, 34500.0), n)
    v.ekle("blend", {"parts": []}, dene(ref.blend, []), "boş harman (eskiden NaN): " + n)
    v.ekle("hemocytometer", {"total_counted": 100.0, "squares_counted": 0.0, "dilution": 1.0}, dene(ref.hemocytometer, 100.0, 0.0, 1.0), n)
    v.ekle("viability_stain", {"unstained": 0.0, "stained": 0.0}, dene(ref.viability_stain, 0.0, 0.0), n)
    v.ekle("slurry_volume_ml", {"cells_billion": 100.0, "density_cells_ml": None, "viability": 0.0, "default_density": 1.0e9},
           dene(ref.slurry_volume_mL, 100.0, None, 0.0, 1.0e9), "canlılık 0: " + n)
    v.ekle("brewhouse_yield", {"v_l": 20.0, "p": 12.0, "malt_kg": 0.0, "hot": True}, dene(ref.brewhouse_yield, 20.0, 12.0, 0.0, True), n)
    v.ekle("first_wort_max_p", {"grain_kg": 0.0, "extract_as_is_pct": 80.0, "moisture_pct": 4.0, "mash_water_l": 0.0},
           dene(ref.first_wort_max_P, 0.0, 80.0, 4.0, 0.0), n)
    v.ekle("conversion_eff", {"fw_measured_p": 10.0, "fw_max_p": 0.0}, dene(ref.conversion_eff, 10.0, 0.0), n)
    v.ekle("lab_ratio", {"brewhouse_pct": 70.0, "lab_extract_pct": 0.0}, dene(ref.lab_ratio, 70.0, 0.0), "T1.17 bölen 0: " + n)
    v.ekle("lauter_eff", {"brewhouse_pct": 70.0, "lab_extract_pct": 0.0, "conversion": 0.9},
           dene(ref.lauter_eff, 70.0, 0.0, 0.9), "T1.17 laboratuvar ekstraktı 0: " + n)
    v.ekle("lauter_eff", {"brewhouse_pct": 70.0, "lab_extract_pct": 78.0, "conversion": 0.0},
           dene(ref.lauter_eff, 70.0, 78.0, 0.0), "T1.17 dönüşüm verimi 0: " + n)
    return v.yaz()
