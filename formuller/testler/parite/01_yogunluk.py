"""Bölüm 01 parite vektörleri: yoğunluk, alkol, renk, ölçüm araçları (formuller/01-yogunluk-alkol-renk.md)."""
from ortak import Vektor, yukle

# (og, fg) çiftleri: tipik, spesifikasyon örnekleri, og == fg, og < fg (OE kırpması)
CIFTLER = [
    (1.050, 1.010), (1.065, 1.012), (1.080, 1.016), (1.0609, 1.0102), (1.100, 1.010),
    (1.030, 1.006), (1.120, 1.025), (1.040, 1.040), (1.010, 1.020), (1.0001, 1.0),
]


def uret():
    ref = yukle("01_yogunluk_tv.py")
    v = Vektor("01", "formuller/testler/01_yogunluk_tv.py")

    # §1 dönüşümler
    for sg in (1.000, 1.010, 1.040, 1.050, 1.100, 1.120):
        v.ekle("to_plato", {"sg": sg}, ref.ToPlato(sg))
        v.ekle("sg_to_brix", {"sg": sg}, ref.SGToBrix(sg))
    for p in (0.0, 5.0, 10.0, 12.0, 20.0, 30.0):
        v.ekle("to_sg", {"plato": p}, ref.ToSG(p))
        v.ekle("brix_to_sg", {"brix": p}, ref.BrixToSG(p))
        v.ekle("brix_to_ri", {"brix": p}, ref.BrixToRI(p))
    for b in (0.0, 5.0, 10.0, 20.0, 30.0):
        v.ekle("ri_to_brix", {"ri": ref.BrixToRI(b)}, ref.RIToBrix(ref.BrixToRI(b)), "gidiş-dönüş")
    v.ekle("ri_to_brix", {"ri": 1.3450}, ref.RIToBrix(1.3450))

    # §6 alkol, ekstrakt, attenuasyon, kalori, UK
    for og, fg in CIFTLER:
        g = {"og": og, "fg": fg}
        v.ekle("real_extract", g, ref.RE(og, fg))
        v.ekle("abw", g, ref.ABW(og, fg))
        v.ekle("abv", g, ref.ABV(og, fg))
        v.ekle("apparent_attenuation", g, ref.AA(og, fg))
        v.ekle("real_attenuation", g, ref.RA(og, fg))
        v.ekle("alcohol_calories", g, ref.alc_cal(og, fg))
        v.ekle("extract_calories", g, ref.ext_cal(og, fg))
        v.ekle("uk_tax_abv", g, ref.uk(og, fg))
    # UK eşikleri: d = (og-fg)*1000 her eşiğin hemen altı, tam eşik ve hemen üstü
    for lim in (6.95, 10.45, 17.25, 26.15, 36.05, 46.55, 57.15, 67.95, 78.85, 89.75):
        for d in (lim - 0.01, lim, lim + 0.01):
            og, fg = 1.0 + d / 1000.0, 1.0
            v.ekle("uk_tax_abv", {"og": og, "fg": fg}, ref.uk(og, fg), "eşik komşuluğu d=%.2f" % d)
    v.ekle("uk_tax_abv", {"og": 1.0, "fg": 1.0}, ref.uk(1.0, 1.0), "d = 0")
    v.ekle("uk_tax_abv", {"og": 1.100, "fg": 1.005}, ref.uk(1.100, 1.005), "d > 89.75")

    # §8 renk (Morey)
    for toplam, gal in ((80.0, 6.0), (20.0, 5.0), (150.0, 5.5), (0.0, 5.0), (1.0, 20.0)):
        v.ekle("morey_srm", {"mcu_sum": toplam, "volume_gal": gal}, ref.morey(toplam, gal))

    # §9.1 hidrometre sıcaklık düzeltmesi (°F)
    for sg, t, tc in ((1.050, 77, 60), (1.050, 60, 60), (1.040, 68, 68), (1.060, 100, 60), (1.020, 40, 60),
                      (1.050, 59.5, 68)):
        v.ekle("hydrometer_correction", {"sg": sg, "temp_f": t, "calib_f": tc}, ref.hydro(sg, t, tc))

    # §9.2 refraktometre
    for b, z, w in ((12.0, 0.0, 1.02), (10.0, 0.0, 1.0), (15.0, 0.5, 1.04), (0.0, 0.0, 1.02), (20.0, -0.3, 1.02)):
        v.ekle("refractometer_unfermented", {"brix": b, "zero": z, "wcf": w}, ref.refr0(b, z, w))
    for og, b, z in ((1.050, 6.5, 0.0), (1.060, 8.0, 0.0), (1.040, 5.0, 0.3), (1.080, 12.0, 0.0), (1.100, 0.0, 0.0)):
        v.ekle("refractometer_fermenting", {"og": og, "brix": b, "zero": z}, ref.refr1(og, b, z))
    for fg, b, z in ((1.010, 6.5, 0.0), (1.012, 10.0, 0.0), (1.008, 8.0, 0.4), (1.020, 14.0, 0.0), (1.005, 4.0, 0.0)):
        v.ekle("refractometer_finished", {"fg": fg, "brix": b, "zero": z}, list(ref.refr2(fg, b, z)),
               "[og, abw_yuzde, re, oe]")

    # §5.2 toleranstan FG (ikiye bölme)
    for og, fg, tol in ((1.100, 1.010, 10.0), (1.090, 1.005, 8.0), (1.120, 1.010, 12.0), (1.080, 1.016, 7.0),
                        (1.060, 1.012, 5.0), (1.130, 1.020, 9.5), (1.100, 1.050, 10.0)):
        v.ekle("fg_from_tolerance", {"og": og, "fg": fg, "tol": tol}, ref.fg_tol(og, fg, tol))

    # §9.3 gravite yükseltme (özüt); kind 0 = DME, 1 = LME
    for g, t, vol, typ in ((1.040, 1.050, 640.0, 0), (1.040, 1.050, 640.0, 1), (1.030, 1.060, 704.0, 0),
                           (1.050, 1.050, 640.0, 0), (1.060, 1.050, 640.0, 1), (1.040, 1.050, 0.0, 0),
                           (1.040, 1.050, 0.005, 1), (1.020, 1.070, 1280.0, 1)):
        v.ekle("gravity_adjust_extract_oz", {"current": g, "target": t, "volume_floz": vol, "kind": typ},
               ref.gadj_oz(g, t, vol, typ))
    # §9.4 seyreltme
    for g, t, vol in ((1.060, 1.050, 640.0), (1.080, 1.040, 704.0), (1.040, 1.050, 640.0), (1.050, 1.050, 640.0),
                      (1.100, 1.030, 1280.0), (1.050, 1.020, 0.0)):
        v.ekle("gravity_adjust_water_floz", {"current": g, "target": t, "volume_floz": vol},
               ref.gadj_water(g, t, vol))

    _uret_tam(v, ref)
    return v.yaz()


def _tarifler(ref):
    """Reçete fikstürleri: her biri farklı dalları açar (tip, aşama, geç özüt, mash adımı, maya)."""
    f, y, mk = ref.ferm, ref.yeast, ref.make_recipe
    ekstrakt = mk(
        [f("LME", 96, 78, 8, 1, 2), f("DME late", 48, 95, 4, 4, 2, late_extract_min=15),
         f("Honey late", 16, 75, 2, 7, 2, late_extract_min=15), f("DME late5", 16, 95, 4, 4, 2, late_extract_min=5),
         f("Crystal steep", 16, 72, 40, 0, 1), f("Lactose", 8, 100, 1, 2, 2, not_fermentable=True),
         f("Whirlpool honey", 8, 75, 2, 7, 3), f("Fruit", 32, 10, 3, 5, 4), f("Sugar sec", 12, 100, 0, 2, 5),
         f("Honey tert", 8, 75, 2, 7, 6), f("Priming", 5, 100, 0, 2, 7), f("Sparge grain", 8, 70, 3, 3, 8)],
        typ=0, volume_floz=640, trub_loss_floz=64, pre_trub_loss_floz=704, boil_vol_floz=768, efficiency=72,
        yeasts=[y(60, 70, 8, add_to_secondary=True), y(70, 80, 11)])
    partial = mk(
        [f("Pale", 64, 80, 3, 0, 0, max_in_batch_pct=100), f("Crystal", 16, 72, 60, 0, 1), f("DME", 48, 95, 4, 4, 2),
         f("Sparge adj", 16, 75, 2, 3, 8), f("Acid", 2, 0, 0, 8, 0)],
        typ=1, volume_floz=640, trub_loss_floz=64, boil_vol_floz=800, efficiency=65,
        yeasts=[y(70, 75, 12)], mash_temps=[156, 168], grain_weight_oz=80)
    ag_cok_adim = mk(
        [f("2-row", 200, 80, 2, 0, 0), f("Munich", 20, 78, 9, 0, 0), f("Honey", 16, 75, 2, 7, 4),
         f("Sugar", 12, 100, 0, 2, 2)],
        typ=2, volume_floz=704, trub_loss_floz=96, boil_vol_floz=900, efficiency=78, yeasts=[y(75, 85, 5)],
        mash_temps=[122, 148, 158, 170], grain_weight_oz=232)
    ag_atlama = mk([f("2-row", 160, 80, 2, 0, 0), f("Crystal", 16, 72, 60, 0, 0)], typ=2, volume_floz=640,
                   trub_loss_floz=64, boil_vol_floz=800, efficiency=70, yeasts=[y(68, 72, 9)],
                   mash_temps=[150, 156], grain_weight_oz=176)
    ag_kapali = mk([f("2-row", 160, 80, 2, 0, 0), f("Sugar", 16, 100, 0, 2, 2)], typ=2, volume_floz=704,
                   efficiency=72, yeasts=[y(73, 77, 10)], mash_temps=[135, 162], mash_fg_adjust=False,
                   mash_fg_adj_slope=-2.0)
    ag_yok = mk([f("2-row", 160, 80, 2, 0, 0), f("Sugar late", 8, 100, 0, 2, 2, late_extract_min=10)], typ=2,
                volume_floz=704, trub_loss_floz=64, efficiency=75, boil_time_min=90, yeasts=[], mash_temps=[])
    cider = mk([f("Juice", 640, 12, 1, 6, 4), f("Sugar", 16, 100, 0, 2, 2)], typ=3, volume_floz=640,
               efficiency=0, yeasts=[y(95, 100, 14)], mash_temps=[152])
    bos = mk([f("2-row", 160, 80, 2, 0, 0)], typ=2, volume_floz=0.0, efficiency=72, yeasts=[y()],
             mash_temps=[152])
    return {"ornek_ag": ref.ORNEK_AG, "ekstrakt": ekstrakt, "kismi": partial, "ag_cok_adim": ag_cok_adim,
            "ag_atlama": ag_atlama, "ag_kapali": ag_kapali, "ag_yok": ag_yok, "cider": cider, "bos": bos}


def _renk_tarifleri(ref):
    f, mk = ref.ferm, ref.make_recipe
    pale = mk([f("2-row", 144, 80, 2, 0, 0, max_in_batch_pct=100), f("Munich", 16, 78, 9, 0, 0, max_in_batch_pct=40),
               f("Crystal 60", 12, 72, 60, 0, 0, max_in_batch_pct=15),
               f("Black", 4, 70, 500, 0, 0, max_in_batch_pct=5)],
              typ=2, volume_floz=640, trub_loss_floz=64, efficiency=75)
    ext = mk([f("LME pale", 96, 78, 8, 1, 2), f("DME", 32, 95, 4, 4, 2), f("Crystal steep", 16, 72, 40, 0, 1),
              f("Roast steep", 4, 70, 400, 0, 1)],
             typ=0, volume_floz=640, trub_loss_floz=64, pre_trub_loss_floz=704, efficiency=72)
    pm = mk([f("Pale", 64, 80, 3, 0, 0), f("Crystal", 16, 72, 60, 0, 0), f("DME", 48, 95, 4, 4, 2),
             f("Chocolate", 6, 70, 350, 0, 0)],
            typ=1, volume_floz=640, trub_loss_floz=64, efficiency=65)
    return {"renk_ornek": ref.ORNEK_RENK, "pale": pale, "ekstrakt": ext, "kismi": pm}


def _uret_tam(v, ref):
    """T1.14e: §0.2, §1.3-1.7, §2-§5.1, §6.6, §6.8, §7.3, §8 için parite vakaları."""
    # --- §1.3-§1.7 çeviriciler ---
    for x, frm in ((1.050, 3), (1050.0, 3), (105.0, 3), (10.5, 3), (1.0, 3), (881.0, 3), (880.0, 3), (89.0, 3),
                   (88.0, 3), (8.9, 3), (8.8, 3), (12.4, 4), (0.0, 4), (20.0, 4), (12.0, 31), (0.0, 31), (25.0, 31)):
        for to in (3, 4, 31):
            v.ekle("convert_hydrometer", {"x": x, "from": frm, "to": to}, ref.bconv_hydrometer(x, frm, to))
    for x, frm in ((1.050, 3), (1050.0, 3), (12.0, 31), (0.0, 31), (22.0, 31), (1.347871, 32), (1.33302, 32)):
        for to in (3, 31, 32):
            v.ekle("convert_refractometer", {"x": x, "from": frm, "to": to}, ref.bconv_refractometer(x, frm, to))
    for x, frm in ((8.819, 29), (0.0, 29), (150.0, 29), (17.374, 30), (39.0, 30), (30.0, 47), (2.0, 47)):
        for to in (29, 30, 47):
            v.ekle("convert_color", {"x": x, "from": frm, "to": to}, ref.bconv_color(x, frm, to))
    for x, frm in ((77.0, 2), (32.0, 2), (-40.0, 2), (212.0, 2), (25.0, 1), (0.0, 1), (100.0, 1), (298.15, 40)):
        for to in (1, 2, 40):
            v.ekle("convert_temperature", {"x": x, "from": frm, "to": to}, ref.bconv_temperature(x, frm, to))
    for y in (0.0, 36.0, 72.0, 80.0, 100.0):
        v.ekle("potential_from_yield", {"yield_pct": y}, ref.potential_from_yield(y))
    for sg in (0.99, 1.0, 1.0368, 1.046, 1.0461, 1.05, 1.08):
        v.ekle("yield_from_potential", {"sg": sg}, ref.yield_from_potential(sg), "kırpma [0, 100]")
    for x, frm in ((80.0, 31), (1.0368, 3), (1.0368, 4), (100.0, 2), (1.08, 3)):
        v.ekle("grain_yield_convert", {"x": x, "from": frm}, ref.bconv_grain_yield(x, frm))

    # --- §2 fermentable puanları: tip × kullanım × not_fermentable ---
    for typ in range(9):
        for use in (0, 1, 2, 3, 4, 8):
            for nf in (False, True):
                g = ref.ferm("x", 32, 75, 5, typ, use, not_fermentable=nf)
                v.ekle("grain_og_points", {"grain": g, "eff": 72.0, "steep_eff": 15.0}, ref.og_points(g, 72.0, 15.0))
                v.ekle("grain_fg_points", {"grain": g, "eff": 72.0, "atten": 76.875, "steep_eff": 15.0},
                       ref.fg_points(g, 72.0, 76.875, 15.0))
    g = ref.ferm("x", 16, 100, 0, 2, 7)
    v.ekle("grain_fg_points", {"grain": g, "eff": 70.0, "atten": 100.0, "steep_eff": 20.0}, ref.fg_points(g, 70.0, 100.0, 20.0))

    # --- §3-§5 reçete düzeyi ---
    tarifler = _tarifler(ref)
    for ad, r in tarifler.items():
        n = "tarif=" + ad
        for phase in range(9):
            v.ekle("og_points_phase", {"recipe": r, "phase": phase}, ref.og_points_phase(r, phase), n)
        for vol in (640.0, 0.0, -5.0, r["volume_floz"]):
            for phase in (0, 4, 6):
                v.ekle("estimate_og", {"recipe": r, "volume_floz": vol, "phase": phase},
                       ref.estimate_og(r, vol, phase), n)
        v.ekle("recipe_og", {"recipe": r}, ref.recipe_og(r), n)
        v.ekle("recipe_virtual_og", {"recipe": r}, ref.recipe_virtual_og(r), n)
        v.ekle("est_post_mash_og", {"recipe": r}, ref.est_post_mash_og(r), n)
        for vb in (0.0, 900.0, 0.05, -3.0):
            v.ekle("late_extract_profile", {"recipe": r, "boil_vol_floz": vb}, ref.late_extract_sgs(r, vb), n + " [[sg, dk]]")
            v.ekle("recipe_og_boil", {"recipe": r, "boil_vol_floz": vb}, ref.recipe_og_boil(r, vb), n)
        for hedef in (1.040, 1.080, 1.0, 0.99):
            s = ref.scale_gravity(r, hedef)
            v.ekle("scale_gravity", {"recipe": r, "og_target": hedef},
                   "target_not_positive" if s is None else [g["amount_oz"] for g in s["fermentables"]],
                   n + " [miktarlar oz] ya da hata")
        for w in (100.0, 232.0, 0.0, -1.0, r["mash"]["grain_weight"]):
            m = ref.scale_mash_from_grain_weight(r["mash"], w)
            v.ekle("scale_mash_from_grain_weight", {"mash": r["mash"], "grain_weight_new": w}, m, n)
        for use in range(9):
            for field in ("amount_oz", "yield_pct", "color_srm"):
                v.ekle("total_grain_use_field", {"recipe": r, "use": use, "field": field},
                       ref.total_grain_use_field(r, use, field), n)
        for og_ in (1.050, 0.0, -1.0, 1.020):
            for vol in (704.0, 0.0, 640.0):
                for mo in (False, True):
                    v.ekle("efficiency", {"recipe": r, "og": og_, "volume_floz": vol, "mash_only": mo},
                           ref.get_efficiency(r, og_, vol, mo), n)
        v.ekle("yeast_attenuation", {"recipe": r}, list(ref.yeast_attenuation(r)), n + " [atten, tol]")
        for att_ in (73.0, 75.0, 60.0):
            v.ekle("mash_adjusted_attenuation", {"recipe": r, "atten": att_}, ref.mash_adjusted_attenuation(r, att_), n)
        v.ekle("recipe_fg", {"recipe": r}, ref.recipe_fg(r), n)
        v.ekle("recipe_color", {"recipe": r}, ref.recipe_color(r), n)

    # §5.1 tolerans dalı: tol < ABV olan bir vaka açıkça bulunsun
    r = tarifler["ag_cok_adim"]
    fg_ = ref.recipe_fg(r)
    v.ekle("recipe_fg", {"recipe": r}, fg_, "tarif=ag_cok_adim; tolerans 5 % ABV sınırı devrede")

    # --- §6.6, §6.8 ---
    for mn, mx in ((73.0, 77.0), (60.0, 70.0), (0.0, 0.0), (80.0, 80.0)):
        v.ekle("avg_attenuation", {"min": mn, "max": mx}, ref.avg_attenuation(mn, mx))
    for sg in (1.0, 1.010, 1.050, 1.120):
        v.ekle("original_extract", {"start_sg": sg}, ref.original_extract(sg))
        v.ekle("apparent_extract", {"final_sg": sg}, ref.apparent_extract(sg))

    # --- §7.3 ---
    for og_, fg_ in [(1.050, 1.010), (1.065, 1.012), (1.080, 1.016), (1.040, 1.040), (1.010, 1.020), (1.120, 1.025)]:
        g = {"og": og_, "fg": fg_}
        v.ekle("calories_alcohol", g, ref.cal_alcohol(og_, fg_))
        v.ekle("calories_carbs", g, ref.cal_carbs(og_, fg_))
        v.ekle("calories_total", g, ref.cal_total(og_, fg_))
    for kcal in (0.0, 60.4, 165.2):
        v.ekle("carbohydrates_grams", {"carb_kcal": kcal}, ref.carb_grams(kcal))
    for tot, alc in ((165.2, 104.8), (0.0, 0.0), (100.0, 120.0)):
        v.ekle("carb_calories", {"total_kcal": tot, "alc_kcal": alc}, ref.carb_calories(tot, alc))

    # --- §8 ---
    for srm in (0.0, 1.0, 5.0, 8.819, 10.0, 40.0, 150.0):
        v.ekle("mcu_from_srm", {"srm": srm}, ref.mcu_from_srm(srm))
    for ad, r in _renk_tarifleri(ref).items():
        n = "renk tarifi=" + ad
        v.ekle("recipe_color", {"recipe": r}, ref.recipe_color(r), n)
        for hedef in (1.0, 3.0, 5.0, 8.0, 12.0, 20.0, 35.0, 80.0, 150.0):
            lo, hi, ok = ref.color_flexibility(r, hedef)
            v.ekle("color_flexibility", {"recipe": r, "target": hedef}, [lo, hi, ok], n + " [lo, hi, ok]")
            nr, renk = ref.iterate_color(r, hedef)
            v.ekle("iterate_color", {"recipe": r, "target": hedef}, [renk, [g["amount_oz"] for g in nr["fermentables"]]],
                   n + " [renk, miktarlar oz]")
            ar, basari = ref.adjust_color(r, hedef)
            v.ekle("adjust_color", {"recipe": r, "target": hedef},
                   [ref.recipe_color(ar), basari, [g["amount_oz"] for g in ar["fermentables"]]],
                   n + " [renk, başarı, miktarlar oz]")
        mn, mx = ref.color_adj_range(r)
        v.ekle("color_adj_range", {"recipe": r}, [mn, mx], n + " [min, max]")
    # kapsam dışı: boş reçete ve hacimsiz reçete
    for r in (tarifler["bos"], ref.make_recipe([], volume_floz=640)):
        v.ekle("color_flexibility", {"recipe": r, "target": 10.0}, list(ref.color_flexibility(r, 10.0)), "ok = false")
        v.ekle("adjust_color", {"recipe": r, "target": 10.0},
               [ref.recipe_color(ref.adjust_color(r, 10.0)[0]), ref.adjust_color(r, 10.0)[1],
                [g["amount_oz"] for g in ref.adjust_color(r, 10.0)[0]["fermentables"]]], "ok = false")
