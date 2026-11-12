"""Bölüm 01 parite vektörleri: yoğunluk, alkol, renk, ölçüm araçları (formuller/01-yogunluk-alkol-renk.md)."""
from ortak import Vektor, dene, fikstur, yukle

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
    v.ekle("ri_to_brix", {"ri": 1.2}, dene(ref.RIToBrix, 1.2), "kırılma indisi aralık dışı: karekök negatif (T1.16)")

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
    v.ekle("alcohol_calories", {"og": 1.775, "fg": 1.0}, dene(ref.alc_cal, 1.775, 1.0), "OG = 1.775: sıfıra bölme (T1.16)")
    # UK eşikleri (§6.7, T1.12 C1): du = u(og) - u(fg) tam sayı birimde; her eşiğin -1, tam eşik ve +1 birimi,
    # üç farklı FG tabanında
    for lim in (695, 1045, 1725, 2615, 3605, 4655, 5715, 6795, 7885, 8975):
        for fg0 in (1.0, 1.0105, 1.040):
            for du in (lim - 1, lim, lim + 1):
                fg = fg0
                og = fg0 + du / 100000.0
                v.ekle("uk_tax_abv", {"og": og, "fg": fg}, ref.uk(og, fg), "eşik komşuluğu du=%d fg=%s" % (du, fg0))
    for og, fg in ((1.00695, 1.0), (1.01045, 1.0), (1.01725, 1.0), (1.02615, 1.0), (1.03605, 1.0), (1.04655, 1.0),
                   (1.05715, 1.0), (1.06795, 1.0), (1.07885, 1.0), (1.08975, 1.0)):
        v.ekle("uk_tax_abv", {"og": og, "fg": fg}, ref.uk(og, fg),
               "ondalıkta tam eşik (eski f64 karşılaştırmasında üst katsayıya düşen çiftler)")
    v.ekle("uk_tax_abv", {"og": 1.0, "fg": 1.0}, ref.uk(1.0, 1.0), "d = 0")
    v.ekle("uk_tax_abv", {"og": 1.100, "fg": 1.005}, ref.uk(1.100, 1.005), "d > 89.75")

    # §8 renk (Morey)
    for toplam, gal in ((80.0, 6.0), (20.0, 5.0), (150.0, 5.5), (0.0, 5.0), (1.0, 20.0)):
        v.ekle("morey_srm", {"mcu_sum": toplam, "volume_gal": gal}, ref.morey(toplam, gal))
    for toplam, gal in ((80.0, 0.0), (-1.0, 5.0), (80.0, -5.0)):
        v.ekle("morey_srm", {"mcu_sum": toplam, "volume_gal": gal}, dene(ref.morey, toplam, gal),
               "hacim <= 0 ya da negatif toplam: hata (T1.16)")

    # §9.1 hidrometre sıcaklık düzeltmesi (°F)
    for sg, t, tc in ((1.050, 77, 60), (1.050, 60, 60), (1.040, 68, 68), (1.060, 100, 60), (1.020, 40, 60),
                      (1.050, 59.5, 68)):
        v.ekle("hydrometer_correction", {"sg": sg, "temp_f": t, "calib_f": tc}, ref.hydro(sg, t, tc))

    # §9.2 refraktometre
    for b, z, w in ((12.0, 0.0, 1.02), (10.0, 0.0, 1.0), (15.0, 0.5, 1.04), (0.0, 0.0, 1.02), (20.0, -0.3, 1.02)):
        v.ekle("refractometer_unfermented", {"brix": b, "zero": z, "wcf": w}, ref.refr0(b, z, w))
    v.ekle("refractometer_unfermented", {"brix": 12.0, "zero": 0.0, "wcf": 0.0}, dene(ref.refr0, 12.0, 0.0, 0.0),
           "wcf = 0: sıfıra bölme (T1.16)")
    for og, b, z in ((1.050, 6.5, 0.0), (1.060, 8.0, 0.0), (1.040, 5.0, 0.3), (1.080, 12.0, 0.0), (1.100, 0.0, 0.0)):
        v.ekle("refractometer_fermenting", {"og": og, "brix": b, "zero": z}, ref.refr1(og, b, z))
    for fg, b, z in ((1.010, 6.5, 0.0), (1.012, 10.0, 0.0), (1.008, 8.0, 0.4), (1.020, 14.0, 0.0), (1.005, 4.0, 0.0)):
        v.ekle("refractometer_finished", {"fg": fg, "brix": b, "zero": z}, list(ref.refr2(fg, b, z)),
               "[og, abw_yuzde, re, oe]")

    # §5.2 toleranstan FG (ikiye bölme)
    for og, fg, tol in ((1.100, 1.010, 10.0), (1.090, 1.005, 8.0), (1.120, 1.010, 12.0), (1.080, 1.016, 7.0),
                        (1.060, 1.012, 5.0), (1.130, 1.020, 9.5), (1.100, 1.050, 10.0)):
        v.ekle("fg_from_tolerance", {"og": og, "fg": fg, "tol": tol}, ref.fg_tol(og, fg, tol))

    # §6.3 üç parametreli CalcABV (kırpmasız) ve §9.2 refraktometre ABV/ABW
    for og, fg in CIFTLER + [(1.020, 1.060), (1.0, 1.0)]:
        v.ekle("abv_abw_unclamped", {"og": og, "fg": fg}, list(ref.calc_abv3(og, fg)), "[abv, abw]")
    for kind, grav, b, z in ((0, 1.050, 12.0, 0.0), (1, 1.050, 6.5, 0.0), (1, 1.080, 9.0, 0.3), (1, 1.040, 12.0, 0.0),
                             (2, 1.010, 6.5, 0.0), (2, 1.012, 10.0, 0.0), (2, 1.008, 8.0, 0.4), (2, 1.030, 3.0, 0.0)):
        g = {"kind": kind, "gravity": grav, "brix": b, "zero": z}
        v.ekle("refractometer_abv", g, ref.refractometer_abv(kind, grav, b, z))
        v.ekle("refractometer_abw", g, ref.refractometer_abw(kind, grav, b, z))
    for fg, b, z in ((1.010, 6.5, 0.0), (1.012, 10.0, 0.0), (1.008, 8.0, 0.4), (1.020, 14.0, 0.0)):
        v.ekle("refractometer_finished_abv", {"fg": fg, "brix": b, "zero": z}, ref.refractometer_finished_abv(fg, b, z),
               "ABW*FG/0.794")

    # §9.3 gravite yükseltme (özüt); kind 0 = DME, 1 = LME
    for g, t, vol, typ in ((1.040, 1.050, 640.0, 0), (1.040, 1.050, 640.0, 1), (1.030, 1.060, 704.0, 0),
                           (1.050, 1.050, 640.0, 0), (1.060, 1.050, 640.0, 1), (1.040, 1.050, 0.0, 0),
                           (1.040, 1.050, 0.005, 1), (1.020, 1.070, 1280.0, 1)):
        v.ekle("gravity_adjust_extract_oz", {"current": g, "target": t, "volume_floz": vol, "kind": typ},
               ref.gadj_oz(g, t, vol, typ))
    for kind in (2, -1):
        v.ekle("gravity_adjust_extract_oz", {"current": 1.040, "target": 1.050, "volume_floz": 640.0, "kind": kind},
               dene(ref.gadj_oz, 1.040, 1.050, 640.0, kind), "bilinmeyen özüt türü kodu: hata (T1.16)")
    # §9.4 seyreltme
    for g, t, vol in ((1.060, 1.050, 640.0), (1.080, 1.040, 704.0), (1.040, 1.050, 640.0), (1.050, 1.050, 640.0),
                      (1.100, 1.030, 1280.0), (1.050, 1.020, 0.0)):
        v.ekle("gravity_adjust_water", {"current": g, "target": t, "volume": vol},
               ref.gadj_water(g, t, vol))
    v.ekle("gravity_adjust_water", {"current": 1.050, "target": 1.0, "volume": 640.0},
           dene(ref.gadj_water, 1.050, 1.0, 640.0), "hedef 1.000: sıfıra bölme (T1.16)")

    _uret_tam(v, ref)
    _gec_ekstrakt(v, ref)
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
    # --- T1.15 (B1-B5): seçenekleri değiştiren ve dal sınırlarını ayıran reçeteler ---
    def ag(temps, **kw):
        return mk([f("2-row", 160, 80, 2, 0, 0), f("Crystal", 16, 72, 60, 0, 0)], typ=2, volume_floz=640,
                  trub_loss_floz=64, boil_vol_floz=800, efficiency=70, yeasts=[y(68, 76, 12)],
                  mash_temps=temps, grain_weight_oz=176, **kw)

    extra = {
        # B1: steep verimi 25, Steep aşamasında tahıl; Tref 150 ve eğim -2 ile adım 148
        "steep25": mk([f("LME", 96, 78, 8, 1, 2), f("Crystal steep", 24, 72, 40, 0, 1), f("Roast steep", 8, 70, 400, 3, 1)],
                      typ=0, volume_floz=640, trub_loss_floz=64, pre_trub_loss_floz=704, efficiency=72,
                      steep_efficiency=25.0, yeasts=[y(70, 78, 11)]),
        "ag_tref150_egim2": ag([148], mash_fg_adj_temp=150.0, mash_fg_adj_slope=-2.0),
        "ag_egim_sifir_adim": ag([], mash_fg_adj_temp=150.0, mash_fg_adj_slope=-0.5, steep_efficiency=30.0),
        # B2: kırpma ve prev/pencere sınırları (düzeltme açık)
        "ag_alt_kirpma": ag([135]), "ag_ust_kirpma": ag([165]),
        "ag_prev_139": ag([139, 156]), "ag_prev_1389": ag([138.9, 156]), "ag_prev_122": ag([122, 156]),
        "ag_pencere_alt_40": ag([113.5]), "ag_pencere_alt_399": ag([113.6]),
        "ag_pencere_ust_40": ag([193.5]), "ag_pencere_ust_399": ag([193.4]),
        "ag_tref140_adimsiz": ag([], mash_fg_adj_temp=140.0),
        # B3: tolerans; Secondary'de şeker olduğundan sanal OG != F_R_OG, sınır %4
        "ag_tol_sanal": mk([f("2-row", 160, 80, 2, 0, 0), f("Sugar sec", 48, 100, 0, 2, 5), f("Honey tert", 16, 75, 2, 7, 6)],
                           typ=2, volume_floz=704, trub_loss_floz=64, efficiency=75, yeasts=[y(75, 85, 4)],
                           mash_temps=[152], grain_weight_oz=160),
        "ag_tol_sifir": mk([f("2-row", 200, 80, 2, 0, 0), f("Sugar", 16, 100, 0, 2, 2)], typ=2, volume_floz=640,
                           efficiency=78, yeasts=[y(80, 90, 0)], mash_temps=[152], grain_weight_oz=200),
        "ag_iki_birincil": mk([f("2-row", 200, 80, 2, 0, 0), f("Sugar", 16, 100, 0, 2, 2)], typ=2, volume_floz=640,
                              efficiency=78, yeasts=[y(60, 70, 5), y(85, 95, 14)], mash_temps=[152], grain_weight_oz=216),
        "ag_ikinci_birincil_tol": mk([f("2-row", 200, 80, 2, 0, 0)], typ=2, volume_floz=640, efficiency=78,
                                     yeasts=[y(70, 74, 14, add_to_secondary=True), y(80, 90, 6), y(60, 62, 3)],
                                     mash_temps=[152], grain_weight_oz=200),
        # B4: geç özüt: kaynatma süresinden uzun, 0.1 dk eşiği, Whirlpool ve Primary geç özüt
        "gec_uzun": mk([f("Pale", 160, 80, 3, 0, 0), f("DME late75", 32, 95, 4, 4, 2, late_extract_min=75),
                        f("DME late15", 24, 95, 4, 4, 2, late_extract_min=15)],
                       typ=2, volume_floz=704, trub_loss_floz=64, boil_vol_floz=800, efficiency=72,
                       mash_temps=[152], grain_weight_oz=160, yeasts=[y()]),
        "gec_eslesen_005": mk([f("Pale", 160, 80, 3, 0, 0), f("DME a", 24, 95, 4, 4, 2, late_extract_min=15.0),
                               f("DME b", 16, 95, 4, 4, 2, late_extract_min=15.05)],
                              typ=2, volume_floz=704, trub_loss_floz=64, boil_vol_floz=800, efficiency=72, yeasts=[y()]),
        "gec_ayri_02": mk([f("Pale", 160, 80, 3, 0, 0), f("DME a", 24, 95, 4, 4, 2, late_extract_min=15.0),
                           f("DME b", 16, 95, 4, 4, 2, late_extract_min=15.2)],
                          typ=2, volume_floz=704, trub_loss_floz=64, boil_vol_floz=800, efficiency=72, yeasts=[y()]),
        "gec_whirlpool_primary": mk([f("Pale", 160, 80, 3, 0, 0), f("Honey wp", 16, 75, 2, 7, 3, late_extract_min=10),
                                     f("DME prim", 16, 95, 4, 4, 4, late_extract_min=10),
                                     f("DME boil", 16, 95, 4, 4, 2, late_extract_min=20)],
                                    typ=2, volume_floz=704, trub_loss_floz=64, boil_vol_floz=800, efficiency=72,
                                    yeasts=[y()]),
        "gec_ad_sirasi": mk([f("Pale", 160, 80, 3, 0, 0), f("Zeta", 24, 95, 4, 4, 2, late_extract_min=15),
                             f("Alpha", 16, 75, 2, 7, 2, late_extract_min=15)],
                            typ=2, volume_floz=704, trub_loss_floz=64, boil_vol_floz=800, efficiency=72, yeasts=[y()]),
        # B5: BIAB
        "ag_biab": mk([f("2-row", 160, 80, 2, 0, 0)], typ=2, volume_floz=704, efficiency=72, yeasts=[y()],
                      mash_temps=[152, 168], grain_weight_oz=160, biab=True),
    }
    return {"ornek_ag": ref.ORNEK_AG, **extra, "ekstrakt": ekstrakt, "kismi": partial, "ag_cok_adim": ag_cok_adim,
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
    # T1.15 (A3, B5): MaxInBatch + 5/15 sınırları, Partial Mash %25 tabanı, verimi 0 ya da hacimsiz reçete,
    # miktarı 0 olan koyu malzeme
    dusuk_max = mk([f("2-row", 160, 80, 2, 0, 0, max_in_batch_pct=100), f("Crystal", 16, 72, 60, 0, 0, max_in_batch_pct=5),
                    f("Chocolate", 4, 70, 350, 0, 0, max_in_batch_pct=10), f("Roast", 4, 70, 500, 0, 0, max_in_batch_pct=5)],
                   typ=2, volume_floz=640, trub_loss_floz=64, efficiency=75)
    kismi_taban = mk([f("Pale", 64, 80, 3, 0, 0, max_in_batch_pct=5), f("Pilsner", 40, 80, 2, 0, 0, max_in_batch_pct=5),
                      f("DME", 24, 95, 4, 4, 2), f("Black", 2, 70, 500, 0, 0, max_in_batch_pct=100)],
                     typ=1, volume_floz=640, trub_loss_floz=64, efficiency=65)
    verim0 = mk([f("2-row", 160, 80, 2, 0, 0), f("Crystal", 16, 72, 60, 0, 0), f("Black", 4, 70, 500, 0, 0)],
                typ=2, volume_floz=640, trub_loss_floz=64, efficiency=0)
    sifir_miktar = mk([f("2-row", 160, 80, 2, 0, 0), f("Crystal", 16, 72, 60, 0, 0), f("Black none", 0, 70, 500, 0, 0)],
                      typ=2, volume_floz=640, trub_loss_floz=64, efficiency=72)
    verim0_ozut = mk([f("2-row", 160, 80, 2, 0, 0), f("Pale", 40, 80, 3, 0, 0), f("Dark DME", 48, 95, 40, 4, 2),
                      f("Crystal 12", 16, 72, 12, 0, 0)], typ=2, volume_floz=640, trub_loss_floz=64, efficiency=0)
    sifir_koyu = mk([f("2-row", 160, 80, 2, 0, 0), f("Munich 12", 16, 78, 12, 0, 0), f("Crystal 20", 8, 72, 20, 0, 0),
                     f("Black none", 0, 70, 500, 0, 0)], typ=2, volume_floz=640, trub_loss_floz=64, efficiency=72)
    sifir_koyu2 = mk([f("2-row", 30, 80, 2, 0, 0), f("Munich 12", 10, 78, 12, 0, 0), f("Black none", 0, 70, 500, 0, 0)],
                     typ=2, volume_floz=640, trub_loss_floz=64, efficiency=72)
    hacimsiz = mk([f("2-row", 160, 80, 2, 0, 0), f("Crystal", 16, 72, 60, 0, 0)], typ=2, volume_floz=0.0, efficiency=72)
    return {"renk_ornek": ref.ORNEK_RENK, "pale": pale, "ekstrakt": ext, "kismi": pm, "dusuk_max": dusuk_max,
            "kismi_taban": kismi_taban, "verim0": verim0, "sifir_miktar": sifir_miktar, "hacimsiz": hacimsiz,
            "verim0_ozut": verim0_ozut, "sifir_koyu": sifir_koyu, "sifir_koyu2": sifir_koyu2}


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
    for x, frm in ((80.0, 31), (1.0368, 3), (1.0368, 4), (1.08, 3)):
        v.ekle("grain_yield_convert", {"x": x, "from": frm}, ref.bconv_grain_yield(x, frm))
    v.ekle("grain_yield_convert", {"x": 100.0, "from": 2}, dene(ref.bconv_grain_yield, 100.0, 2),
           "bilinmeyen birim kodu: hata (T1.16)")
    # T1.16: bilinmeyen birim kodları artık hata (eskiden x aynen dönerdi)
    for fonk, fn, x, frm, to in (("convert_hydrometer", ref.bconv_hydrometer, 1.050, 7, 3),
                                 ("convert_hydrometer", ref.bconv_hydrometer, 1.050, 3, 7),
                                 ("convert_refractometer", ref.bconv_refractometer, 12.0, 5, 3),
                                 ("convert_color", ref.bconv_color, 10.0, 29, 99),
                                 ("convert_temperature", ref.bconv_temperature, 20.0, 5, 1)):
        v.ekle(fonk, {"x": x, "from": frm, "to": to}, dene(fn, x, frm, to), "bilinmeyen birim kodu: hata (T1.16)")

    # --- §2 fermentable puanları: tip × kullanım × not_fermentable ---
    for typ in range(9):
        for use in (0, 1, 2, 3, 4, 8):
            for nf in (False, True):
                g = ref.ferm("x", 32, 75, 5, typ, use, not_fermentable=nf)
                v.ekle("grain_og_points", {"grain": g, "eff": 72.0, "steep_eff": 15.0}, ref.og_points(g, 72.0, 15.0))
                v.ekle("grain_fg_points", {"grain": g, "eff": 72.0, "atten": 76.875, "steep_eff": 15.0},
                       ref.fg_points(g, 72.0, 76.875, 15.0))
    for typ in (0, 1, 3, 8):   # steep verimi 25 ve 40
        for use in (0, 1, 2):
            g = ref.ferm("x", 24, 72, 5, typ, use)
            v.ekle("grain_og_points", {"grain": g, "eff": 65.0, "steep_eff": 25.0}, ref.og_points(g, 65.0, 25.0))
            v.ekle("grain_fg_points", {"grain": g, "eff": 65.0, "atten": 70.0, "steep_eff": 40.0},
                   ref.fg_points(g, 65.0, 70.0, 40.0))
    g = ref.ferm("x", 16, 100, 0, 2, 7)
    v.ekle("grain_fg_points", {"grain": g, "eff": 70.0, "atten": 100.0, "steep_eff": 20.0}, ref.fg_points(g, 70.0, 100.0, 20.0))

    # --- §3-§5 reçete düzeyi ---
    tarifler = _tarifler(ref)
    _r0 = next(iter(tarifler.values()))
    v.ekle("og_points_phase", {"recipe": _r0, "phase": 9}, dene(ref.og_points_phase, _r0, 9), "bilinmeyen aşama kodu: hata (T1.16)")
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
                   {"hata": "ValueError"} if s is None else [g["amount_oz"] for g in s["fermentables"]],
                   n + " [miktarlar oz] ya da hata")
        for w in (100.0, 232.0, 0.0, -1.0, r["mash"]["grain_weight_oz"]):
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
        v.ekle("recipe_est_mash_efficiency", {"recipe": r}, ref.recipe_est_mash_efficiency(r), n)
        v.ekle("recipe_act_mash_efficiency", {"recipe": r, "og_mash_measured": 1.045, "boil_vol_measured": 800.0},
               ref.recipe_act_mash_efficiency(r, 1.045, 800.0), n)
        v.ekle("recipe_actual_efficiency", {"recipe": r, "og_measured": 1.050, "volume_measured": 704.0},
               ref.recipe_actual_efficiency(r, 1.050, 704.0), n)
        v.ekle("yeast_attenuation", {"recipe": r}, list(ref.yeast_attenuation(r)), n + " [atten, tol]")
        for att_ in (73.0, 75.0, 60.0):
            v.ekle("mash_adjusted_attenuation", {"recipe": r, "atten": att_}, ref.mash_adjusted_attenuation(r, att_), n)
        v.ekle("recipe_fg", {"recipe": r}, ref.recipe_fg(r), n)
        v.ekle("recipe_color", {"recipe": r}, ref.recipe_color(r), n)

    # §5.1 tolerans dalı: ag_tol_sanal, ag_tol_sifir, ag_iki_birincil, ag_ikinci_birincil_tol (sanal OG != F_R_OG,
    # tol = 0, birden çok birincil maya) yukarıdaki reçete döngüsüyle sınanır.
    # §3.8: infüzyonu <= 0 olan adımlar (make_recipe hep pozitif üretir)
    for infs in ([0.0, 20.0, 30.0], [-5.0, 15.0], [40.0]):
        m = {"grain_weight_oz": 100.0, "biab": False,
             "steps": [{"kind": 0, "temp_f": 150.0 + i, "infusion_floz": x} for i, x in enumerate(infs)]}
        for w in (150.0, 60.0):
            v.ekle("scale_mash_from_grain_weight", {"mash": m, "grain_weight_new": w},
                   ref.scale_mash_from_grain_weight(m, w), "infüzyon <= 0 adımları")

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
        for hedef in (mn - 0.5, mn + 0.01, (mn + mx) / 2, mx - 0.01, mx + 0.5, 150.0):
            st, ar, basari = ref.do_adjust_color(r, hedef)
            v.ekle("do_adjust_color", {"recipe": r, "target": hedef},
                   [st, ref.recipe_color(ar), basari, [g["amount_oz"] for g in ar["fermentables"]]],
                   n + " [durum, renk, başarı, miktarlar oz]")
    # §8.3 DoAdjustColor: bira tipi olmayan reçeteler (F_R_TYPE >= 3)
    for typ in (2, 3, 4, 5, 6):
        r = ref.make_recipe([ref.ferm("2-row", 160, 80, 2, 0, 0), ref.ferm("Crystal", 16, 72, 60, 0, 0)],
                            typ=typ, volume_floz=640, trub_loss_floz=64, efficiency=72)
        st, ar, basari = ref.do_adjust_color(r, 15.0)
        v.ekle("do_adjust_color", {"recipe": r, "target": 15.0},
               [st, ref.recipe_color(ar), basari, [g["amount_oz"] for g in ar["fermentables"]]],
               "tip=%d [durum, renk, başarı, miktarlar oz]" % typ)
    # kapsam dışı: boş reçete ve hacimsiz reçete
    for r in (tarifler["bos"], ref.make_recipe([], volume_floz=640)):
        v.ekle("color_flexibility", {"recipe": r, "target": 10.0}, list(ref.color_flexibility(r, 10.0)), "ok = false")
        v.ekle("adjust_color", {"recipe": r, "target": 10.0},
               [ref.recipe_color(ref.adjust_color(r, 10.0)[0]), ref.adjust_color(r, 10.0)[1],
                [g["amount_oz"] for g in ref.adjust_color(r, 10.0)[0]["fermentables"]]], "ok = false")


def _gec_ekstrakt(v, ref):
    """§3.6 geç ekstrakt profili: 02 §5.1 ile tek uygulama; aynı fikstürler 02'nin tarif IBU vakalarında da
    kullanılır (T1.16 B)."""
    fx = fikstur(ref)
    mk = ref.make_recipe
    for (nm, rc, vb) in (
        ("geç ekstrakt yok", mk(fx.ferms_a, boil_vol_floz=768), 0.0),
        ("çoklu geç ekstrakt, eşit süre ve <0.1 fark birleşmesi", mk(fx.ferm_late, boil_vol_floz=768), 0.0),
        ("whirlpool fermentable", mk(fx.ferm_wp, boil_vol_floz=768), 0.0),
        ("tür 3 ve süresi 0 olan geç ekstrakt sayılmaz", mk(fx.ferm_misc, boil_vol_floz=768), 0.0),
        ("boil_vol parametresi", mk(fx.ferms_a, boil_vol_floz=768), 900.0),
        ("boil_vol <= 0 -> alan, alt sınır 0.1", mk(fx.ferms_a, boil_vol_floz=0.0), -1.0),
        ("boil_time < geç ekleme süresi (min)", mk(fx.ferm_late, boil_vol_floz=768, boil_time_min=30), 0.0),
        ("V=0 -> SG 1", mk(fx.ferms_a, boil_vol_floz=768, volume_floz=0.0), 0.0),
        ("puan <= 0", mk([], boil_vol_floz=768), 0.0),
        ("trub", mk(fx.ferm_late, boil_vol_floz=768, trub_loss_floz=32.0), 0.0),
        ("Whirlpool kullanımlı geç ekstrakt t=0'da sayılır (T1.16 B)", mk(fx.ferm_late3, boil_vol_floz=768), 0.0),
        ("kazan dışı geç ekstrakt profile girmez (T1.16 B)", mk(fx.ferm_late_dis, boil_vol_floz=768), 0.0),
        ("0.1 dk'dan yakın geç ekstrakt: son girdi t=0 (T1.16 B)", mk(fx.ferm_late_son, boil_vol_floz=768), 0.0),
    ):
        v.ekle("late_extract_profile", {"recipe": rc, "boil_vol_floz": vb}, ref.late_extract_sgs(rc, vb),
               nm + " [[sg, dk]]")
        v.ekle("recipe_og_boil", {"recipe": rc, "boil_vol_floz": vb}, ref.recipe_og_boil(rc, vb), nm)
