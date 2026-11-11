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
    _ek(v, ref)
    return v.yaz()


def _ek(v, ref):
    """T1.14f: referansa eklenen hesaplar (enum, varsayilan, maya, mead, karbonasyon)."""
    for kind, tab in ref.ENUMS.items():
        for code in range(-1, len(tab) + 2):
            v.ekle("enum_name", {"kind": kind, "code": code}, ref.enum_name(kind, code))
    for name in ref.DEFAULTS:
        v.ekle("default_value", {"name": name}, ref.Default(name))
    v.ekle("default_value", {"name": "yok"}, ref.Default("yok"), "bilinmeyen ad -> null")

    for pkg, rec, brew, today in [(10, 0, 40, 100), (10, 1, 40, 100), (100, 0, 40, 100),
                                  (100, 1, 40, 100)]:
        v.ekle("age_days", {"pkg_day": pkg, "in_recipe": rec, "brew_day": brew, "today": today},
               ref.AgeDays(pkg, rec, brew, today))
    for today, d in [(100, 40), (100, 99), (100, 100), (100, 120), (100, 70), (100, 10)]:
        v.ekle("yeast_age_tool", {"today": today, "date_day": d}, ref.YeastAgeTool(today, d))
    for days in ([], [30], [30, 60, 90], [0, 15]):
        v.ekle("yeast_age_recipe", {"days": days}, ref.YeastAgeRecipe(days))

    for f in range(4):
        v.ekle("v0_ingredient", {"form": f}, ref.ViabilityV0Ingredient(f))
    for f in range(3):
        v.ekle("v0_tool", {"form": f}, ref.ViabilityV0Tool(f))
    for form in range(4):
        for wl in (False, True):
            v.ekle("import_age_rate", {"form": form, "white_labs": wl},
                   ref.ImportAgeRate(form, wl))
    for calc, uv in [(True, 80.0), (False, 80.0)]:
        v.ekle("tool_viability", {"calc": calc, "user_v": uv, "v0": 97.0, "rate": 10.0,
                                  "days": 90.0}, ref.ToolViability(calc, uv, 97.0, 10.0, 90.0))
    for tot, vv in [(100.0, 70.71), (0.0, 50.0), (200.0, 5.0)]:
        v.ekle("viable_cells", {"total": tot, "v": vv}, ref.ViableCells(tot, vv))

    for up, cp, na, form, amt, cml in [(True, 100.0, 2.0, 0, 0.0, 1.0), (False, 0, 0, 1, 15.0, 1.0),
                                       (False, 0, 0, 0, 500.0, 1.0), (False, 0, 0, 2, 100.0, 0.5),
                                       (False, 0, 0, 3, 200.0, 2.0), (False, 0, 0, 4, 100.0, 1.0),
                                       (False, 0, 0, 7, 100.0, 1.0)]:
        v.ekle("yeast_total_cells", {"use_pkgs": up, "cells_pkg": cp, "new_amount": na,
                                     "form": form, "amount": amt, "cells_ml": cml},
               ref.YeastTotalCells(up, cp, na, form, amt, cml))
    for dry in (True, False):
        v.ekle("yeast_cells_avail", {"dry": dry, "dry_g": 11.5, "wet_ml": 250.0, "cells_ml": 1.0},
               ref.YeastCellsAvail(dry, 11.5, 250.0, 1.0))

    ys = [
        {"secondary": False, "tot_cells": 100.0, "viable_cells": 70.0, "new_amount": 1.0,
         "viability_pct": 70.0, "form": 0, "use_pkgs": 1, "amount_oz": 0.0, "cells_ml": 1.0,
         "dry_pitch_rate": 75.0},
        {"secondary": False, "tot_cells": 150.0, "viable_cells": 140.0, "new_amount": 2.0,
         "viability_pct": 93.0, "form": 1, "use_pkgs": 1, "amount_oz": 0.0, "cells_ml": 1.0,
         "dry_pitch_rate": 80.0},
        {"secondary": True, "tot_cells": 50.0, "viable_cells": 40.0, "new_amount": 1.0,
         "viability_pct": 80.0, "form": 1, "use_pkgs": 0, "amount_oz": 1.0, "cells_ml": 1.0,
         "dry_pitch_rate": 60.0},
        {"secondary": False, "tot_cells": 90.0, "viable_cells": 80.0, "new_amount": 0.0,
         "viability_pct": 85.0, "form": 1, "use_pkgs": 0, "amount_oz": 0.5, "cells_ml": 2.0,
         "dry_pitch_rate": 70.0},
    ]
    sets = [[], ys[:1], ys[:2], ys, ys[2:3], ys[2:]]
    for s in sets:
        for calc in (False, True):
            v.ekle("cells_no_starter", {"calc_viability": calc, "yeasts": s},
                   ref.YeastCellsNoStarter(calc, s))
        v.ekle("pack_count", {"yeasts": s}, ref.YeastPackCount(s))
        v.ekle("pack_viability", {"yeasts": s}, ref.YeastPackViability(s))
        v.ekle("tot_grams", {"yeasts": s}, ref.YeastTotGrams(s))
        v.ekle("first_fields", {"yeasts": s}, ref.YeastFirstFields(s))
    for cells, amt, valid in [(200.0, 2.0, True), (0.0, 2.0, True), (-5.0, 2.0, True),
                              (200.0, 0.0, True), (200.0, 2.0, False)]:
        v.ekle("avg_pack_size", {"cells": cells, "amount_sum": amt, "valid": valid},
               ref.YeastAvgPackSize(cells, amt, valid))

    for rt in (0, 1, 2, 3, 4, 5, 6, 7, 8):
        for at in (0, 1, 2, 5):
            v.ekle("recommended_liquid_pitch", {"r_type": rt, "ale_type": at},
                   ref.RecommendedLiquidPitch(rt, at))
    for rec in (0.5, 0.75, 1.5):
        v.ekle("pitch_min_max_recipe", {"rec": rec}, ref.PitchMinMaxRecipe(rec))
    for tt in (0, 1, 2, 3, 4, -1):
        v.ekle("pitch_min_max_tool", {"tool_type": tt}, ref.PitchMinMaxTool(tt))
    for lo, hi in [(0.45, 1.05), (1.25, 1.75), (0.6, 1.4), (0.25, 0.75), (0.005, 10.0)]:
        v.ekle("pitch_range", {"lo": lo, "hi": hi}, ref.PitchRange(lo, hi))
    for rd in (75.0, 0.0, 19.0, 20.0, 23.9, 100.0):
        v.ekle("dry_min_pitch", {"rec_dry": rd}, ref.DryMinPitchRate(rd))
        v.ekle("dry_max_pitch", {"rec_dry": rd}, ref.DryMaxPitchRate(rd))
        v.ekle("dry_pitch_range", {"rec_dry": rd}, ref.DryPitchRange(rd))
    for form in (0, 1):
        for up in (0, 1):
            for tool in (False, True):
                v.ekle("is_dry_by_weight", {"form": form, "use_pkgs": up, "tool": tool},
                       ref.IsDryByWeight(form, up, tool))

    for dbw, pitch, vl, sg, dr, rr in [(False, 0.75, 20.0, 1.060, 75.0, 75.0),
                                       (True, 0.75, 20.0, 1.060, 75.0, 75.0),
                                       (True, 0.75, 20.0, 1.060, 150.0, 75.0),
                                       (True, 0.75, 20.0, 1.060, 0.0, 0.0),
                                       (True, 0.75, 0.0, 1.060, 50.0, 75.0)]:
        v.ekle("tot_yeast_cells_needed", {"dry_by_weight": dbw, "pitch": pitch, "v_l": vl,
                                          "sg": sg, "dry_rate": dr, "rec_dry_rate": rr},
               ref.TotYeastCellsNeeded(dbw, pitch, vl, sg, dr, rr))
    for n, vml in [(221.12, 20000.0), (100.0, 0.05), (100.0, 0.0), (0.0, 1000.0)]:
        v.ekle("est_cells_ml", {"n": n, "v_ml": vml}, ref.EstCellsML(n, vml))

    for viable, sl in [(70.71, 1.524), (70.71, 3.424), (97.0, 0.0), (97.0, -1.0), (100.0, 0.1),
                       (0.0, 1.0), (100.0, 1000.0)]:
        for stir in (False, True):
            v.ekle("cells_starter", {"viable": viable, "starter_l": sl, "stir": stir},
                   ref.CellsStarter(viable, sl, stir))
    for c1, sl in [(221.0, 1.75), (221.0, 0.0), (50.0, 0.5)]:
        for stir in (False, True):
            v.ekle("cells_sec_starter", {"cells1": c1, "sec_l": sl, "stir": stir},
                   ref.CellsSecStarter(c1, sl, stir))

    for need, avail in [(225.0, 100.0), (225.0, 70.0), (100.0, 100.0), (50.0, 100.0),
                        (500.0, 100.0), (300.0, 0.0), (300.0, 0.05), (4000.0, 50.0)]:
        for stir in (False, True):
            v.ekle("two_stage_recipe", {"need": need, "avail": avail, "stir": stir,
                                        "growth_cap": 4.0},
                   ref.two_stage_recipe(need, avail, stir, 4.0))
    v.ekle("two_stage_recipe", {"need": 4000.0, "avail": 50.0, "stir": False, "growth_cap": 100.0},
           ref.two_stage_recipe(4000.0, 50.0, False, 100.0), "buyume siniri gevsek")

    for n, pkt in [(221.12, 100.0), (50.0, 100.0), (100.0, 100.0), (200.0, 100.0),
                   (201.0, 100.0), (221.0, 0.0), (221.0, -3.0), (0.0, 100.0)]:
        v.ekle("packs_no_starter", {"n": n, "pkt": pkt}, ref.YeastPacksNoStarter(n, pkt))
        for growth in (4.0, 0.0, -1.0, 1.2):
            for stir in (False, True):
                v.ekle("packs_starter", {"n": n, "pkt": pkt, "growth": growth, "stir": stir},
                       ref.YeastPacksStarter(n, pkt, growth, stir))
    for r, ps, pk in [(2.0, 100.0, 1), (1.0, 100.0, 1), (0.5, 100.0, 2), (5.0, 75.0, 3)]:
        v.ekle("calc_starter_size", {"r": r, "pkt_size": ps, "pkt": pk},
               ref.CalcYeastStarterSize(r, ps, pk))

    for rate, vl in [(75.0, 20.0), (0.0, 20.0), (150.0, 5.0), (75.0, 0.0)]:
        v.ekle("dry_grams_vol", {"dry_rate": rate, "v_l": vl}, ref.DryGramsVol(rate, vl))
    for n, c in [(150.0, 18.0), (150.0, 0.0), (150.0, -2.0), (0.0, 18.0)]:
        v.ekle("dry_grams_pkg", {"n": n, "cells_per_g": c}, ref.DryGramsPkg(n, c))
    for g in (0.0, 5.0, 11.5, 15.0, 100.0):
        v.ekle("packs_dry", {"g": g}, ref.YeastPacksDry(g))
        v.ekle("dry5", {"g": g}, ref.YeastDry5(g))
        v.ekle("dry_hydrate", {"tot_g": g, "per_g": 25.0}, ref.YeastDryHydrate(g, 25.0))
        v.ekle("goferm", {"tot_g": g, "per_g": 25.0, "goferm": 0.05},
               ref.YeastGoferm(g, 25.0, 0.05))
    v.ekle("goferm", {"tot_g": 15.0, "per_g": 30.0, "goferm": 0.04}, ref.YeastGoferm(15.0, 30.0, 0.04))
    v.ekle("dry_hydrate", {"tot_g": 15.0, "per_g": 20.0}, ref.YeastDryHydrate(15.0, 20.0))

    for lo, hi in [(70.0, 75.0), (0.0, 0.0), (65.5, 80.25)]:
        v.ekle("avg_attenuation", {"lo": lo, "hi": hi}, ref.AvgAttenuation(lo, hi))
    for up in (False, True):
        for form in (0, 1, 2):
            v.ekle("units_code", {"use_pkgs": up, "form": form, "dry_unit": 10, "wet_unit": 15},
                   ref.YeastUnitsCode(up, form, 10, 15))
    v.ekle("units_code", {"use_pkgs": False, "form": 1, "dry_unit": 9, "wet_unit": 14},
           ref.YeastUnitsCode(False, 1, 9, 14))
    for tol in (10, 12, 15):
        for yt in range(5):
            v.ekle("import_tolerance", {"tol": tol, "y_type": yt}, ref.ImportHookTolerance(tol, yt))

    for sg, vl in [(1.100, 19.0), (1.120, 19.0), (1.050, 5.0), (1.110, 0.0), (1.101, 10.0)]:
        v.ekle("mead_yeast_grams", {"sg": sg, "v_l": vl}, ref.MeadYeastGrams(sg, vl))
    for g in (0.0, 4.75, 5.0):
        v.ekle("mead_goferm", {"g": g}, ref.MeadGoferm(g))
        v.ekle("mead_hydrate", {"g": g}, ref.MeadHydrate(g))
        v.ekle("mead_hydrate1", {"g": g}, ref.MeadHydrate1(g))

    for sg, gal, pct in [(1.100, 5.0197, 100.0), (1.080, 5.0, 50.0), (1.120, 3.0, 75.0),
                         (1.000, 5.0, 100.0)]:
        for fac in (0, 1, 2):
            v.ekle("yan_target", {"sg": sg, "pct": pct, "fac": fac}, ref.MeadYanTarget(sg, pct, fac))
            for method in (0, 1, 2):
                a = {"sg": sg, "v_gal": gal, "pct": pct, "fac": fac, "method": method}
                v.ekle("mead_tot_ferm_o", a, ref.MeadTotFermO(sg, gal, pct, fac, method))
                v.ekle("mead_tot_ferm_k", a, ref.MeadTotFermK(sg, gal, pct, fac, method))
                v.ekle("mead_ferm_o", a, ref.MeadFermO(sg, gal, pct, fac, method))
                v.ekle("mead_ferm_k", a, ref.MeadFermK(sg, gal, pct, fac, method))
    fs = [{"type": 7, "subtype": 2, "pts": 60.0}, {"type": 7, "subtype": 7, "pts": 10.0},
          {"type": 1, "subtype": 0, "pts": 30.0}]
    for ferms, prim in [(fs, 100.0), (fs[:1], 80.0), (fs[1:], 100.0), ([], 50.0), (fs, 0.0)]:
        v.ekle("mead_pct_honey", {"ferms": ferms, "primary_pts": prim},
               ref.MeadPctHoney(ferms, prim))

    for method in (0, 1, 2, 7):
        v.ekle("yan_equivalent", {"method": method, "yan": 45.21}, ref.YANEquivalent(method, 45.21))
        for tg, vl in [(21.47, 19.0), (10.74, 19.0), (0.0, 19.0), (4.0, 0.0)]:
            for rec in (False, True):
                a = {"method": method, "total_g": tg, "v_l": vl, "recipe": rec}
                v.ekle("mead_yan_added", a, ref.MeadYANAdded(method, tg, vl, rec))
                v.ekle("mead_yan_equivalent", a, ref.MeadYANEquivalent(method, tg, vl, rec))
        for ppm, vl in [(100.0, 19.0), (0.0, 19.0), (60.0, 5.0)]:
            v.ekle("nutrient_needed", {"ppm": ppm, "method": method, "v_l": vl},
                   ref.NutrientNeeded(ppm, method, vl))

    for agent in range(-1, 8):
        v.ekle("back_agent_og", {"agent": agent}, ref.BackAgentOG(agent))
    for agent in range(6):
        for vl, sgb, sgt in [(19.0, 0.998, 1.010), (18.93, 1.015, 1.025), (19.0, 1.010, 1.010),
                             (0.0, 1.000, 1.020)]:
            a = {"v_l": vl, "sg_bev": sgb, "sg_target": sgt, "agent": agent}
            v.ekle("back_agent_vol", a, ref.BackAgentVol(vl, sgb, sgt, agent))
            v.ekle("back_agent_weight", a, ref.BackAgentWeight(vl, sgb, sgt, agent))
    for vl, sgb, sgt, sga in [(19.0, 0.998, 1.010, 1.44319), (19.0, 1.0, 1.02, 1.02),
                              (5.0, 1.015, 1.005, 1.1)]:
        v.ekle("back_agent_vol_sg", {"v_l": vl, "sg_bev": sgb, "sg_target": sgt, "sg_a": sga},
               ref.BackAgentVolSG(vl, sgb, sgt, sga))
    for p in (0.0, 15.0, 70.0, 82.5, 98.0):
        v.ekle("honey_sg", {"plato": p}, ref.ToSG(p))

    for agent in (0, 1, 2):
        for vols, tf, gal in [(2.4, 68.0, 5.0197), (2.0, 32.0, 5.0), (1.0, 100.0, 5.0),
                              (2.4, 68.0, 0.0), (3.0, 20.0, 2.5)]:
            v.ekle("carb_sugar_agent", {"agent": agent, "vols": vols, "temp_f": tf, "gal": gal},
                   ref.CarbSugarAgent(agent, vols, tf, gal))
    for ct in (0, 1, 2):
        for agent in (0, 1, 2):
            for rate in (0.0, 100.0, 80.0, 60.0):
                a = {"c_type": ct, "vols": 2.4, "temp_f": 68.0, "gal": 5.0197,
                     "carb_rate": rate, "agent": agent}
                v.ekle("carb_amount", a, ref.CarbAmount(ct, 2.4, 68.0, 5.0197, rate, agent))
    v.ekle("carb_amount", {"c_type": 1, "vols": 2.5, "temp_f": 39.2, "gal": 5.0,
                           "carb_rate": 50.0, "agent": 0},
           ref.CarbAmount(1, 2.5, 39.2, 5.0, 50.0, 0), "keg: carb_rate yok sayilir")
    for r in (0.0, 0.5, 1.0, 80.0, 100.0, 150.0, -10.0):
        v.ekle("carb_equiv", {"rate": r}, ref.CarbEquivFunc(r))

    for tot, alc in [(200.0, 120.0), (100.0, 100.0), (50.0, 80.0)]:
        v.ekle("carb_calories", {"total": tot, "alc": alc}, ref.CarbCalories(tot, alc))
    for c in (0.0, 80.0, 123.4):
        v.ekle("carb_grams", {"carb_cal": c}, ref.CarbGrams(c))
    for val in (0.0, 2.4, 4.71408, 2.5):
        for gl in (False, True):
            v.ekle("convert_carbonation", {"v": val, "to_gl": gl}, ref.BConvertCarbonation(val, gl),
                   "BeerSmith asimetrik sabitler")
