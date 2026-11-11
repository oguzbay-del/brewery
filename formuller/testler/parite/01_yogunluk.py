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
    return v.yaz()
