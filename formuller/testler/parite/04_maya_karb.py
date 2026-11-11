"""Bölüm 04 parite vektörleri: maya, mead/şarap, karbonasyon (ADR-0003)."""
from ortak import Vektor, yukle


def uret():
    ref = yukle("04_maya_karb_tv.py")
    v = Vektor("04", "formuller/testler/04_maya_karb_tv.py")

    for lt in (0.0, 1.0, 19.0, 20.0, 0.473176):
        v.ekle("l_to_floz", {"litres": lt}, ref.L_to_floz(lt))
        v.ekle("gal_from_l", {"litres": lt}, ref.gal_from_L(lt))
    for fl in (0.0, 8.0, 128.0, 640.0):
        v.ekle("floz_to_l", {"floz": fl}, ref.floz_to_L(fl))
    for b in (0.0, 12.0, 50.0, 82.5):
        v.ekle("brix_to_sg", {"brix": b}, ref.BrixToSG(b), "258.2 BeerSmith tuhaflığı")

    for v0, rate, days in [(97, 10, 90), (97, 5, 90), (98, 1.66, 90), (96, 25, 90),
                           (100, 10, 0), (97, 10, -5), (97, 10, 30), (50, 25, 720),
                           (97, 0, 365), (97, 100, 30), (80.5, 7.5, 45.5)]:
        v.ekle("viability", {"v0": v0, "rate": rate, "days": days, "floor": 5.0},
               ref.viability(v0, rate, days))
    v.ekle("viability", {"v0": 97, "rate": 10, "days": 1000, "floor": 8.0},
           ref.viability(97, 10, 1000, 8.0), "taban 8")

    for g in (1.01, 1.5, 2.0, 5.0, 10.0, 25.0):
        v.ekle("innoc_rate", {"g": g}, ref.InnocRate(g))
    for i in (0.5, 5.0, 40.0, 100.0, 200.0, 1000.0):
        for stir in (False, True):
            v.ekle("growth_rate", {"i": i, "stir": stir}, ref.GrowthRate(i, stir))

    for rate, vl, sg in [(0.75, 20, 1.060), (0.35, 19, 1.040), (1.5, 20, 1.100),
                         (1.0, 5, 1.000), (0.75, 0, 1.050)]:
        v.ekle("cells_needed", {"rate": rate, "volume_l": vl, "sg": sg},
               ref.cells_needed(rate, vl, sg))

    for need, avail in [(225.0, 100.0), (225.0, 70.0), (100.0, 100.0), (50.0, 100.0),
                        (120.0, 100.0), (1000.0, 97.0), (150.0, 100.0), (101.0, 100.0)]:
        for stir in (False, True):
            v.ekle("new_starter_l", {"need": need, "avail": avail, "stir": stir},
                   ref.new_starter_L(need, avail, stir))
    for need, avail in [(225.0, 100.0), (225.0, 70.0), (100.0, 100.0), (50.0, 100.0),
                        (500.0, 100.0), (300.0, 0.0), (300.0, 0.05)]:
        for stir in (False, True):
            v.ekle("two_stage_tool", {"need": need, "avail": avail, "stir": stir},
                   ref.two_stage_tool(need, avail, stir))

    for vol, og in [(0.0, 1.036), (33.8, 1.036), (101.4, 1.040), (200.0, 1.000), (64.0, 1.050)]:
        v.ekle("dme_oz", {"vol_floz": vol, "og": og}, ref.dme_oz(vol, og))

    for vols, tf, gal, dme in [(2.4, 68.0, 5.0197, False), (2.4, 68.0, 5.0197, True),
                               (2.0, 32.0, 5.0, False), (3.0, 20.0, 5.0, False),
                               (3.5, 80.0, 2.5, True), (1.0, 100.0, 5.0, False),
                               (2.4, 68.0, 0.0, False)]:
        v.ekle("sugar_g", {"vols": vols, "temp_f": tf, "gal": gal, "dme": dme},
               ref.sugar_g(vols, tf, gal, dme))
    for tf in (0.0, 32.0, 38.0, 55.0, 68.0, 75.0, 100.0):
        v.ekle("residual_co2", {"temp_f": tf}, ref.residual(tf))
    for vols, tf in [(2.5, 39.2), (2.5, 38.0), (2.5, 68.0), (0.0, 38.0), (1.0, 32.0),
                     (2.4, 0.0), (4.5, 50.0), (0.5, 90.0), (0.0, 100.0)]:
        v.ekle("co2_psi", {"vols": vols, "temp_f": tf}, ref.co2_psi(vols, tf))

    for ph in (2.8, 3.0, 3.2, 3.5, 3.8, 4.0):
        v.ekle("so2_rec", {"ph": ph}, ref.so2_rec(ph))
    for vg, tgt, free in [(5.0197, 35.0, 0.0), (5.0, 50.0, 10.0), (1.0, 30.0, 30.0),
                          (10.0, 20.0, 25.0)]:
        v.ekle("kmeta_g", {"v_gal": vg, "target": tgt, "free": free},
               ref.kmeta_g(vg, tgt, free))
    for vg in (0.0, 1.0, 5.0197):
        v.ekle("sorbate_g", {"v_gal": vg}, ref.sorbate_g(vg))

    for sg in (1.000, 1.050, 1.100, 1.101, 1.105, 1.110, 1.120, 1.130, 1.200):
        v.ekle("mead_pitch", {"sg": sg}, ref.mead_pitch(sg))

    for sg, gal, pct, method in [(1.100, 5.0197, 100, 0), (1.100, 5.0197, 100, 1),
                                 (1.080, 5.0, 50, 0), (1.120, 3.0, 75, 1),
                                 (1.000, 5.0, 100, 0)]:
        for fac in (0, 1, 2):
            v.ekle("tosna", {"sg": sg, "v_gal": gal, "pct": pct, "fac": fac, "method": method},
                   ref.tosna(sg, gal, pct, fac, method))

    for method in (0, 1, 2, 7):
        for grams, vl in [(4.0, 19.0), (0.0, 19.0), (10.5, 5.0), (3.0, 0.0)]:
            v.ekle("yan", {"method": method, "grams": grams, "v_l": vl},
                   ref.yan(method, grams, vl))
    return v.yaz()
