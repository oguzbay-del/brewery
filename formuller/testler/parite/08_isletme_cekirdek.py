"""Bölüm 08 (işletme çekirdek hesapları) parite vektörleri (ADR-0003)."""
from ortak import Vektor, yukle


def uret():
    ref = yukle("08_isletme_cekirdek_tv.py")
    v = Vektor("08", "formuller/testler/08_isletme_cekirdek_tv.py")

    for p in (0.0, 2.4, 2.8, 12.0, 20.0):
        v.ekle("plato_to_sg", {"p": p}, ref.plato_to_sg(p))

    # 1. Maliyet
    lots = [["L-A", 3.0, 1.10], ["L-B", 25.0, 1.25]]
    for lotlar, qty, not_ in (
        (lots, 4.5, "iki lota yayılır"),
        (lots, 2.0, "ilk lot yeter"),
        (lots, 3.0, "ilk lot tam biter"),
        (lots, 28.0, "tüm stok, eksik yok"),
        (lots, 30.0, "eksik 2.0"),
        ([["L-2411", 0.20, 52.0]], 0.30, "stok yetmez (TV1.2)"),
        (lots, 0.0, "sıfır ihtiyaç"),
        (lots, -1.0, "negatif ihtiyaç"),
        ([], 5.0, "boş stok"),
        ([["L-0", 0.0, 9.0], ["L-1", 2.0, 1.5]], 1.0, "boş lot atlanır"),
    ):
        rows, cost, miss = ref.fifo_consume(lotlar, qty)
        v.ekle("fifo_consume", {"lots": lotlar, "qty": qty}, {"rows": rows, "cost": cost, "missing": miss}, not_)

    for inv, qty, ship in ((1.20, 25, 4.0), (1.20, 25, 0.0), (52.0, 0.5, 1.3), (3.75, 100, 12.5)):
        v.ekle("landed_price", {"invoice_price": inv, "qty": qty, "shipping_share": ship}, ref.landed_price(inv, qty, ship))
    for kwh, pr in ((4.99, 2.6), (0.0, 2.6), (120.0, 0.18)):
        v.ekle("energy_cost_electric", {"kwh": kwh, "price_kwh": pr}, ref.energy_cost_electric(kwh, pr))
    kj_strike = (15 * 4.186 + 8 * 0.5) * (72 - 15)
    kj_boil = (28 * 4.186 + 8 * 0.5) * (100 - 76)
    for kj, eff in ((kj_strike + kj_boil, 0.90), (0.0, 0.9), (1000.0, 1.0), (50000.0, 0.45)):
        v.ekle("kwh_from_heat", {"kj": kj, "eff": eff}, ref.kwh_from_heat(kj, eff))
    for kj, eff, hu in ((kj_strike + kj_boil, 0.40, 34500.0), (1000.0, 1.0, 34500.0), (50000.0, 0.8, 36000.0)):
        v.ekle("gas_m3_from_heat", {"kj": kj, "eff": eff, "hu_kj_m3": hu}, ref.gas_m3_from_heat(kj, eff, hu))
    for a in ((18.40, 35.0, 30.0, 12.9740, 60, 0.35, 20.0), (0.0, 0.0, 0.0, 0.0, 0, 0.0, 10.0),
              (100.0, 500.0, 1.2, 40.0, 1200, 0.4, 950.0), (7.005, 12.5, 2.333, 1.005, 3, 0.125, 3.0)):
        arg = dict(zip(("ingredient_cost", "water_l", "water_price_m3", "energy_cost", "pack_units", "pack_unit_cost", "packaged_l"), a))
        v.ekle("batch_cost", arg, ref.batch_cost(*a))

    # 2. Harmanlama
    a = dict(V=960, ABV=5.4, IBU=38, SRM=7.0, CO2=1.0, P=2.8)
    b = dict(V=400, ABV=4.6, IBU=22, SRM=4.0, CO2=1.0, P=2.4)
    w = dict(V=100.5, ABV=0.0, IBU=0.0, SRM=0.0, CO2=0.0, P=0.0)
    c = dict(V=250, ABV=9.5, IBU=60, SRM=25.0, CO2=2.4, P=6.1)
    for parts in ([a, b], [a, w], [a], [a, b, c], [c, w]):
        v.ekle("blend", {"parts": parts}, ref.blend(parts))
    for v1, x1, x2, xt in ((960, 5.4, 4.6, 5.2), (960, 5.4, 0.0, 5.0), (500, 20.0, 40.0, 30.0),
                            (960, 5.4, 4.6, 5.4), (960, 5.4, 4.6, 4.6), (960, 5.4, 4.6, 6.0), (960, 5.4, 4.6, 4.0)):
        v.ekle("blend_volume_for_target", {"v1": v1, "x1": x1, "x2": x2, "xt": xt}, ref.blend_volume_for_target(v1, x1, x2, xt),
               "hedef aralık dışı/uçta ise null")

    # 3. Maya
    for pitch, vl, p in ((0.75, 1000, 12.0), (1.5, 20, 15.0), (0.5, 0, 10.0)):
        v.ekle("cells_needed_billion", {"pitch_m_ml_p": pitch, "v_l": vl, "p": p}, ref.cells_needed_billion(pitch, vl, p))
    for tot, sq, dil in ((230, 5, 100), (80, 5, 1), (0, 5, 10), (410, 10, 50)):
        v.ekle("hemocytometer", {"total_counted": tot, "squares_counted": sq, "dilution": dil}, ref.hemocytometer(tot, sq, dil))
    for un, st in ((212, 18), (100, 0), (0, 40), (95, 5)):
        v.ekle("viability_stain", {"unstained": un, "stained": st}, ref.viability_stain(un, st))
    n = ref.cells_needed_billion(0.75, 1000, 12.0)
    d = ref.hemocytometer(230, 5, 100)
    via = ref.viability_stain(212, 18)
    for cells, dens, vi, ddef in ((n, d, via, 1.0e9), (n, None, 0.90, 1.0e9), (n, 0.0, 0.90, 1.0e9),
                                   (n, None, 1.0, 1.0e9), (100.0, 2.0e9, 0.8, 1.0e9), (100.0, None, 0.5, 5.0e8)):
        v.ekle("slurry_volume_ml", {"cells_billion": cells, "density_cells_ml": dens, "viability": vi, "default_density": ddef},
               ref.slurry_volume_mL(cells, dens, vi, ddef), "yoğunluk null/0 ise varsayılan")

    # 4. Profesyonel verim
    for vl, p, hot in ((1040, 12.0, True), (1040, 12.0, False), (20.0, 0.0, True), (500.0, 16.5, True)):
        v.ekle("extract_kg", {"v_l": vl, "p": p, "hot": hot}, ref.extract_kg(vl, p, hot))
    for vl, p, mk, hot in ((1040, 12.0, 170, True), (1040, 12.0, 170, False), (20.0, 14.0, 4.0, True)):
        v.ekle("brewhouse_yield", {"v_l": vl, "p": p, "malt_kg": mk, "hot": hot}, ref.brewhouse_yield(vl, p, mk, hot))
    for fg, mo, fc in ((81.5, 4.3, 0.0), (81.5, 4.3, 1.5), (80.0, 0.0, 2.0), (82.0, 6.0, 0.0)):
        v.ekle("lab_extract_as_is", {"fgdb": fg, "moisture": mo, "fc_diff": fc}, ref.lab_extract_as_is(fg, mo, fc))
    lab = ref.lab_extract_as_is(81.5, 4.3)
    for gk, ex, mo, mw in ((170, lab, 4.3, 520), (5.0, 78.0, 4.0, 15.0), (5.0, 0.0, 4.0, 15.0)):
        v.ekle("first_wort_max_p", {"grain_kg": gk, "extract_as_is_pct": ex, "moisture_pct": mo, "mash_water_l": mw},
               ref.first_wort_max_P(gk, ex, mo, mw))
    for m, mx in ((19.6, 20.09), (20.0, 20.0), (0.0, 18.0)):
        v.ekle("conversion_eff", {"fw_measured_p": m, "fw_max_p": mx}, ref.conversion_eff(m, mx))

    # 5. Malt analizi
    for fg, mo, fc in ((81.5, 4.3, 1.5), (81.0, 4.0, 2.0), (80.0, 0.0, 0.0), (85.2, 5.5, 1.0), (70.0, 6.0, 3.0)):
        v.ekle("malt_from_analysis", {"fgdb": fg, "moisture": mo, "fc_diff": fc}, ref.malt_from_analysis(fg, mo, fc))
    for ebc in (3.5, 0.0, 25.0, 150.0):
        v.ekle("ebc_from_lab", {"ebc": ebc}, ref.ebc_from_lab(ebc))
    return v.yaz()
