"""Bölüm 03 (su kimyası, mash pH, NNLS) parite vektörleri (ADR-0003).

Referans: formuller/testler/03_su_ph_tv.py. Beklenen değerin hepsi referans dosyasındaki fonksiyonlardan
gelir (ADR-0003 madde 1; T1.15); bu betikte hesap gövdesi yoktur, yalnız girdi seçimi ve çağrı vardır.

NNLS: scipy yok (CI'da da yok), bu yüzden referansta saf Python Lawson–Hanson bulunur (ADR-0003 madde 8).
scipy varsa sonuçlar onunla da karşılaştırılır; KKT koşulları her çözümde denetlenir.
"""
import math

from ortak import Vektor, dene, yukle

# NNLS (saf Python Lawson–Hanson) ve KKT denetimi referansta: 03_su_ph_tv.py (nnls_lh, nnls_kkt_ok).


def _scipy_kontrol(cols, b, x, rnorm):
    """scipy varsa aynı çözümü beklenen değerle karşılaştırır (profil ve rnorm; x tek olmayabilir)."""
    try:
        import numpy as np
        from scipy.optimize import nnls
    except ImportError:
        return False
    M = np.array(cols).T
    xs, rs = nnls(M, np.array(b))
    assert abs(rs - rnorm) <= 1e-7 * max(1.0, rs), (rs, rnorm)
    assert np.allclose(M @ xs, M @ np.array(x), atol=1e-6), "NNLS profili scipy'dan farklı"
    return True


# ---------------------------------------------------------------- vektör üretimi
def uret():
    ref = yukle("03_su_ph_tv.py")
    v = Vektor("03", "formuller/testler/03_su_ph_tv.py")
    ions = ref.ions
    salts = list(ref.A.keys())

    # --- §1-2 tuz katkısı
    for gr, vol in [
        ({"gypsum": 5, "cacl2": 3}, 20),
        ({"gypsum": 5, "cacl2": 3}, 5.28344),
        ({}, 20),
        ({"gypsum": 0.0}, 20),
        ({s: 1.5 for s in salts}, 38.0),
        ({"soda": 2.25, "chalk": 1.0, "lime": 0.5}, 18.927),
        ({"gypsum": 5, "cacl2": 3}, 0.0),      # hacim <= 0 koruması (T1.14c: ref.salt_ppm_guarded)
        ({"gypsum": 5}, -3.0),
    ]:
        v.ekle("salt_ppm", {"grams": gr, "vol_l": vol}, ref.salt_ppm_guarded(gr, vol))
    for s in salts:
        v.ekle("salt_ppm", {"grams": {s: 10}, "vol_l": 25}, ref.salt_ppm_guarded({s: 10}, 25), "tek tuz: " + s)

    # --- §4 alkalinite, sertlik, RA
    for h in (0, 61, 122.5, 150, 300.5):
        v.ekle("alkalinity", {"hco3": h}, ref.alk(h))
    for ca, mg in ((0, 0), (50, 10), (112.33, 0), (162.33, 10), (5, 80)):
        v.ekle("hardness", {"ca": ca, "mg": mg}, ref.hard(ca, mg))
    for ca, mg, h in ((50, 10, 150), (162.33, 10, 150), (0, 0, 0), (10, 2, 20), (200, 30, 40), (80, 5, 400)):
        v.ekle("residual_alkalinity", {"ca": ca, "mg": mg, "hco3": h}, ref.RA(ca, mg, h))

    # §4.4 SO4/Cl (referans: print("T5 SO4/Cl", mash[3]/mash[4]); Cl=0 koruması spesifikasyondan: 0)
    mash = [ref.base[k] + ref.salt_ppm({"gypsum": 5, "cacl2": 3}, 20)[i] for i, k in enumerate(ions)]
    v.ekle("sulfate_chloride_ratio", {"so4": mash[3], "cl": mash[4]}, ref.so4cl_ratio(mash[3], mash[4]))
    v.ekle("sulfate_chloride_ratio", {"so4": 100.0, "cl": 25.0}, ref.so4cl_ratio(100.0, 25.0))
    v.ekle("sulfate_chloride_ratio", {"so4": 40.0, "cl": 0.0}, ref.so4cl_ratio(40.0, 0.0), "Cl = 0 koruması (spesifikasyon §4.4 önerisi)")
    # §4.4 sınıf etiketleri: her eşiğin iki yanı
    for r in (-0.01, 0.0, 0.2, 0.39999, 0.4, 0.5, 0.6, 0.7, 0.8, 1.2, 1.426, 1.5, 1.9, 2.0, 2.01, 4.0, 6.0, 6.01, 7.0, 8.0, 8.01, 12.0, 18.0, 18.01, 40.0):
        v.ekle("sulfate_chloride_label", {"ratio": r}, ref.so4cl_label(r))

    # §4.5 renk aralığı (referans: ref.color_range)
    for ra in (1.1166, 81.354, 0.0, -50.0, -200.0, 150.0, -63.4, -122.4, -130.0):
        v.ekle("color_range", {"ra": ra}, ref.color_range(ra))

    # §3.4 kaynatma derişim faktörü (referans: ref.boil_scale; Vb>0, Vp>0 ve kaynatma malzemesi koşulu)
    for vb, vp, cool, trub, top, hb in (
        (28.0, 24.0, 1.0, 1.0, 0.0, True), (28.0, 24.0, 1.0, 1.0, 2.0, True), (30.0, 25.5, 0.5, 1.5, 3.0, True),
        (0.0, 24.0, 1.0, 1.0, 0.0, True), (28.0, 0.0, 1.0, 1.0, 0.0, True), (-1.0, 24.0, 1.0, 1.0, 0.0, True),
        (28.0, 24.0, 1.0, 1.0, 0.0, False), (30.0, 25.5, 0.5, 1.5, 3.0, False),
    ):
        v.ekle("boil_scale", {"vb": vb, "vp": vp, "cool_loss": cool, "trub_loss": trub, "top_up": top, "has_boil_ing": hb},
               dene(ref.boil_scale, vb, vp, cool, trub, top, hb))

    # --- §5.1 grist pH
    for c in (0, 1.9, 2, 5, 12.614, 6.9 / 0.547, 13.99, 14, 15, 60, 199.99, 200, 350, 500):
        for bizim in (False, True):
            v.ekle("grist_g", {"c": c, "ours": bizim}, ref.gfun(c, bizim))
    bills = [
        [(4.5, 2.0), (0.3, 60.0), (0.1, 350.0)],
        [(1.0, 13.0)],
        [(3.0, 3.0), (1.0, 13.99), (0.5, 14.0), (0.2, 200.0)],
        [(5.0, 4.0), (0.4, 120.0)],
        [(2.0, 1.5), (1.0, 40.0), (0.25, 450.0), (0.5, 8.0)],
    ]
    for b in bills:
        for bizim in (False, True):
            gp, s = ref.gristph_guarded(b, bizim)
            v.ekle("grist_ph", {"bill": b, "ours": bizim}, [gp, s])
    for b in ([], [(0.0, 20.0)], [(0.0, 3.0), (0.0, 60.0)]):   # W <= 0 koruması (ref.gristph_guarded)
        v.ekle("grist_ph", {"bill": b, "ours": False}, list(ref.gristph_guarded(b, False)), "W <= 0: 7.0")

    # §5.1 tahıl tipi süzgeci: bill = [kg, SRM, tip]; payda bütün mash tahılları, toplam yalnız tip 0 ve 3
    tbills = [
        [(4.5, 2.0, 0), (0.3, 60.0, 0), (0.1, 350.0, 0)],                  # hepsi tip 0: filtresiz ile aynı
        [(4.5, 2.0, 0), (0.5, 2.0, 2)],                                    # şeker S'ye 0 katar
        [(4.0, 3.0, 0), (1.0, 40.0, 1), (0.5, 5.0, 4)],                    # ekstrakt tipleri sayılmaz
        [(3.0, 3.0, 0), (1.0, 13.99, 3), (0.5, 60.0, 7), (0.25, 450.0, 3)],  # adjunct sayılır, bal sayılmaz
        [(2.0, 120.0, 5), (1.0, 8.0, 6)],                                  # hiçbiri sayılmaz: S = 0, pH 5.72
        [(0.0, 20.0, 2)], [],
    ]
    for tb in tbills:
        for bizim in (False, True):
            gp, ss = ref.gristph_typed(tb, bizim)
            v.ekle("grist_ph_typed", {"bill": [list(x) for x in tb], "ours": bizim}, [gp, ss])
    for units in range(0, 9):
        for amount in (0.0, 2.5, 100.0):
            v.ekle("to_grams", {"amount": amount, "units": units}, dene(ref.to_grams, amount, units),
                   "tanımsız birim kodu: hata (T1.16)")

    # --- §5.2 eğim
    for model in (0, 1):
        for t in (1.5, 2.0, 2.6, 3.0, 4.0):
            v.ekle("mash_ra_slope", {"model": model, "t": t}, ref.slope(model, t))

    # --- §5.3 tahmini pH (referans: gp + ra/50*sl)
    gp0 = ref.gristph(bills[0])[0]
    for gp, ra, t, model in ((gp0, 1.1166, 3.0, 0), (gp0, 1.1166, 3.0, 1), (5.4, 81.354, 2.5, 0), (5.72, -30.0, 4.0, 1)):
        sl = ref.slope(model, t)
        v.ekle("est_mash_ph", {"grist_ph": gp, "ra": ra, "slope": sl}, ref.est_mash_ph(gp, ra, sl))

    # --- §6.1 pK ve ayrışma kesri
    for a in range(0, 10):
        v.ekle("acid_pks", {"acid": a}, dene(ref.pks, a))
    for a in range(0, 9):
        for ph in (2.0, 3.0, 4.5, 5.4, 6.0, 7.5):
            v.ekle("acid_fraction", {"acid": a, "ph": ph}, ref.acidfrac(a, ph))
    for a in (1, 2, 3):
        for model in (0, 1):
            for bizim in (False, True):
                for pht in (5.2, 5.4):
                    v.ekle("dissociation_fraction", {"acid": a, "model": model, "ph_t": pht, "ours": bizim},
                           ref.fdiss(a, model, pht, bizim))

    # --- §6.2 litre başına asit
    for a in (0, 1, 2, 3):
        for model in (0, 1):
            for bizim in (False, True):
                for meq, conc, pht in ((2.82772, 88.0, 5.4), (1.38226, 80.0, 5.2), (0.0, 88.0, 5.4), (5.0, 3.0, 5.4), (3.5, 75.0, 5.6)):
                    v.ekle("acid_per_liter", {"meq": meq, "acid": a, "conc": conc, "model": model, "ph_t": pht, "ours": bizim},
                           dene(ref.acid_per_L, meq, a, conc, model, pht, bizim))

    # --- §6.3 mash asidi miktarı (referans: ref.mash_acid_amount); hedef pH 5.2, 5.4, 5.6 ve 6.0 ile
    est0 = gp0 + 1.1166 / 50 * ref.slope(0, 3.0)
    for est, target, model, t, w, acid, conc, bizim in (
        (est0, 5.4, 0, 3.0, 4.9, 1, 88.0, False), (est0, 5.4, 0, 3.0, 4.9, 1, 88.0, True),
        (est0, 5.4, 1, 3.0, 4.9, 1, 88.0, False), (est0, 5.4, 1, 3.0, 4.9, 1, 88.0, True),
        (est0, 5.4, 0, 3.0, 4.9, 2, 85.0, True), (est0, 5.2, 0, 2.5, 5.5, 3, 3.0, False),
        (est0, 5.4, 0, 3.0, 4.9, 0, 88.0, False), (5.3, 5.4, 0, 3.0, 4.9, 1, 88.0, False),
        (5.9, 5.2, 0, 3.0, 4.9, 1, 88.0, True), (5.9, 5.6, 0, 3.0, 4.9, 1, 88.0, True),
        (5.9, 5.2, 1, 2.5, 4.9, 1, 88.0, True), (5.9, 5.6, 1, 2.5, 4.9, 1, 88.0, True),
        (5.9, 5.2, 1, 2.5, 4.9, 2, 75.0, False), (5.9, 5.6, 1, 2.5, 4.9, 2, 75.0, False),
        (5.9, 5.2, 0, 3.0, 4.9, 3, 3.0, True), (5.9, 5.6, 0, 3.0, 4.9, 3, 3.0, True),
        (5.9, 5.6, 1, 3.0, 4.9, 3, 3.0, False), (6.2, 6.0, 0, 3.0, 4.9, 1, 80.0, True),
        (5.4, 5.4, 0, 3.0, 4.9, 1, 88.0, False),      # dph = 0: sınır
    ):
        v.ekle("mash_acid_amount",
               {"est_ph": est, "target_ph": target, "model": model, "t": t, "w_kg": w, "acid": acid, "conc": conc, "ours": bizim},
               dene(ref.mash_acid_amount, est, target, model, t, w, acid, conc, bizim))

    # --- §5.4(a) laktik asidin RA eşdeğeri (referans T3 kontrolü: acidra = -(p·1000·ρ·VL·f)/(0.09009·W·t))
    for conc, vol_l, f, w, t in ((88.0, 0.0035, 1.0, 4.9, 3.0), (88.0, 0.0036, 0.971968, 4.9, 3.0), (80.0, 0.01, 0.9, 2.0, 2.5)):
        v.ekle("lactic_acid_ra", {"conc": conc, "vol_l": vol_l, "f": f, "w_kg": w, "t": t},
               dene(ref.lactic_acid_ra, conc, vol_l, f, w, t))

    # --- §5.4 asitlerin RA eşdeğeri: fosforik dalı (ρ'da 0.365) ve asit malt (T1.14c)
    for acid, conc, vol_l, f, w, t in (
        (1, 88.0, 0.0035, 0.971968, 4.9, 3.0), (2, 85.0, 0.0030, 1.01508, 4.9, 3.0), (2, 10.0, 0.01, 0.6, 2.0, 2.5),
        (2, 88.0, 0.0, 1.0, 4.9, 3.0), (3, 88.0, 0.0035, 1.0, 4.9, 3.0), (0, 88.0, 0.0035, 1.0, 4.9, 3.0),
    ):
        v.ekle("misc_acid_ra", {"acid": acid, "conc": conc, "vol_l": vol_l, "f": f, "w_kg": w, "t": t},
               dene(ref.misc_acid_ra, acid, conc, vol_l, f, w, t))
    for pct, mk, f, w, t in ((3.0, 0.12888, 1.0, 5.03, 3.0), (2.5, 0.08, 0.9, 5.4, 2.6), (3.0, 0.0, 1.0, 4.9, 3.0)):
        v.ekle("acid_malt_ra", {"pct": pct, "malt_kg": mk, "f": f, "w_kg": w, "t": t}, dene(ref.acid_malt_ra, pct, mk, f, w, t))
    for model, t, w in ((0, 3.0, 4.9), (1, 3.0, 4.9), (0, 0.0, 4.9), (1, -1.0, 4.9), (0, 3.0, 0.0), (1, 2.5, -2.0)):
        v.ekle("mash_ra_slope_checked", {"model": model, "t": t, "w_kg": w}, dene(ref.slope_checked, model, t, w),
               "t <= 0 ya da W <= 0: null (hata)")

    # --- §5.5 ayarlanmış pH (referans adj_ph: asit maltlı); hedef pH değişken (T1.12 B4)
    for model in (0, 1):
        for bizim in (False, True):
            for target in (5.2, 5.4, 5.6):
                for ag in (0.0, 62.78, 128.88):
                    ph, w = dene(ref.adj_ph, bills[0], ag, 3.0, model, 1.1166, 3.0, bizim, target)
                    v.ekle("adjusted_ph", {"bill": bills[0], "acid_malt_g": ag, "pct": 3.0, "model": model, "ra": 1.1166,
                                           "t": 3.0, "target_ph": target, "ours": bizim}, [ph, w])
    for target in (5.2, 5.4, 6.0):
        ph, w = dene(ref.adj_ph, bills[3], 80.0, 2.0, 0, 20.0, 2.5, True, target)
        v.ekle("adjusted_ph", {"bill": bills[3], "acid_malt_g": 80.0, "pct": 2.0, "model": 0, "ra": 20.0,
                               "t": 2.5, "target_ph": target, "ours": True}, [ph, w])

    # --- §5.5 sıvı asitli ayarlanmış pH ve T3 geri kontrolü: önerilen asit eklenince pH hedefe oturur
    # (laktik: iki yoğunluk formülü aynı; fosforik: 0.375 / 0.365 tutarsızlığı yüzünden binde birler düzeyinde sapar)
    for model, bizim in ((0, False), (0, True), (1, False), (1, True)):
        for target in (5.2, 5.4, 5.6):
            est = 5.9
            for acid, conc in ((1, 88.0), (1, 80.0), (2, 85.0)):
                t_, w_ = 3.0, 4.9
                ml = dene(ref.mash_acid_amount, est, target, model, t_, w_, acid, conc, bizim)
                sl = ref.slope(model, t_)
                # ra/50*sl = est - gp  => gp ve ra seçimi: gp = 5.6, ra = (est - gp)/sl*50
                gp = 5.6
                ra = (est - gp) / sl * 50
                v.ekle("adjusted_ph_liquid", {"grist_ph": gp, "ra": ra, "t": t_, "model": model, "w_kg": w_, "acid": acid,
                                              "conc": conc, "vol_l": ml / 1000, "target_ph": target, "ours": bizim},
                       dene(ref.adj_ph_liquid, gp, ra, t_, model, w_, acid, conc, ml / 1000, target, bizim),
                       "geri kontrol: laktikte sonuç hedef pH")
    v.ekle("adjusted_ph_liquid", {"grist_ph": 5.6, "ra": 30.0, "t": 3.0, "model": 0, "w_kg": 4.9, "acid": 0,
                                  "conc": 88.0, "vol_l": 0.002, "target_ph": 5.4, "ours": False},
           dene(ref.adj_ph_liquid, 5.6, 30.0, 3.0, 0, 4.9, 0, 88.0, 0.002, 5.4, False), "asit 0 / tanımsız: asit katkısı yok")

    # --- §6.4 asit malt yinelemesi (referans: ref.acid_malt_amount, [toplam, son pH]); hedef pH değişken
    for bill, pct, model, ra, t in (
        (bills[0], 3.0, 0, 1.1166, 3.0), (bills[0], 3.0, 1, 1.1166, 3.0), (bills[3], 2.5, 0, 40.0, 2.5),
        (bills[3], 2.5, 1, 40.0, 2.5), (bills[1], 3.0, 0, -20.0, 3.0), (bills[2], 4.0, 0, 60.0, 3.5),
    ):
        for bizim in (False, True):
            for target in (5.2, 5.4, 5.6):
                tot, ph2, _ok = dene(ref.acid_malt_amount, bill, pct, model, ra, t, target, bizim)
                v.ekle("acid_malt_amount", {"bill": bill, "pct": pct, "model": model, "ra": ra, "t": t, "target_ph": target, "ours": bizim},
                       [tot, ph2])
    # yakınsama bayrağı (T1.12 B19): [toplam, pH, yakınsadı]; 8 tur dolunca false
    for bill, pct, model, ra, t, target, bizim in (
        (bills[0], 3.0, 0, 1.1166, 3.0, 5.4, False), (bills[3], 2.5, 1, 40.0, 2.5, 5.2, True),
        (bills[0], 0.1, 0, 200.0, 3.0, 5.2, False),     # çok zayıf asit malt: 8 turda yetişmez
        (bills[0], 0.5, 1, 150.0, 3.0, 5.2, True), (bills[1], 3.0, 0, -20.0, 3.0, 5.4, False),
        (bills[0], 3.0, 0, 500.0, 3.0, 5.2, False),
    ):
        tot, ph2, ok = dene(ref.acid_malt_amount, bill, pct, model, ra, t, target, bizim)
        v.ekle("acid_malt_amount_checked",
               {"bill": bill, "pct": pct, "model": model, "ra": ra, "t": t, "target_ph": target, "ours": bizim},
               [tot, ph2, ok], "yakınsadı = false: 8 tur doldu, pH hedefe girmedi")

    # --- §6.5 sparge asidi
    for h in (0.0, 50.0, 150.0, 300.0):
        for phw, pht in ((7.5, 5.4), (7.5, 6.0), (8.2, 5.4), (6.5, 6.0), (7.0, 5.7)):
            v.ekle("sparge_acid_mph", {"hco3": h, "ph_w": phw, "ph_t": pht}, ref.sparge_mpH(h, phw, pht))
            v.ekle("sparge_acid_mph_ours", {"hco3": h, "ph_w": phw, "ph_t": pht, "acid": 1}, ref.sparge_mpH_bizim(h, phw, pht, 1))
            for acid in (1, 2):
                v.ekle("sparge_acid_bw", {"hco3": h, "ph_w": phw, "ph_t": pht, "acid": acid}, ref.sparge_bnw_guarded(h, phw, pht, acid))
    for phw in (4.3, 4.0, 4.31):       # pH_w <= 4.3 koruması (ref.sparge_bnw_guarded)
        v.ekle("sparge_acid_bw", {"hco3": 150.0, "ph_w": phw, "ph_t": 5.4, "acid": 1}, ref.sparge_bnw_guarded(150.0, phw, 5.4, 1))
    v.ekle("sparge_acid_mph_ours", {"hco3": 150.0, "ph_w": 7.5, "ph_t": 6.0, "acid": 2}, ref.sparge_mpH_bizim(150.0, 7.5, 6.0, 2))
    for mol, vol, conc in ((0.00220511, 15.3, 88.0), (0.00229262, 15.3, 88.0), (0.00166758, 15.3, 88.0), (0.002, 10.0, 0.5), (0.0, 15.3, 88.0), (0.003, 20.0, 80.0)):
        v.ekle("lactic_ml", {"mol_l": mol, "v_l": vol, "conc": conc}, ref.lactic_mL(mol, vol, conc))
    # sparge asidi hacmi, fosforik dahil ve sınırlar (ref.sparge_acid_ml)
    for mol, vol, conc, acid in ((0.00220511, 15.3, 88.0, 1), (0.00220511, 15.3, 85.0, 2), (0.002, 10.0, 0.5, 2), (-0.001, 15.3, 88.0, 1),
                                 (0.0, 15.3, 88.0, 2), (0.002, 15.3, 88.0, 3), (0.002, 15.3, 88.0, 0), (0.002, 15.3, 88.0, 4)):
        v.ekle("sparge_acid_ml", {"mol_l": mol, "v_l": vol, "conc": conc, "acid": acid}, ref.sparge_acid_ml(mol, vol, conc, acid))
    # §6.5 bütün (mash ΔpH koşulu, asit türü, model, korumalar)
    for model in (0, 1):
        for bizim in (False, True):
            for acid in (0, 1, 2, 3):
                for phw, pht, dph in ((7.5, 6.0, 0.2), (7.5, 5.4, 0.2), (7.5, 5.4, -0.1), (4.3, 5.4, 0.2), (5.0, 5.4, 0.2), (7.5, 8.0, 0.2)):
                    v.ekle("sparge_acid_amount",
                           {"model": model, "ours": bizim, "hco3": 150.0, "ph_w": phw, "ph_t": pht, "acid": acid, "conc": 88.0,
                            "mash_dph": dph, "v_sp_l": 15.3},
                           ref.sparge_acid_amount(model, bizim, 150.0, phw, pht, acid, 88.0, dph, 15.3))
    # T1.12 B8/B17: küçük hco3, pH_w hedefe yakın, yoğunluk tabanı (conc < 1), mash dpH = 0 sınırı, tanımsız asit kodları
    for h in (5.0, 10.0, 25.0):
        for phw, pht in ((7.5, 5.4), (6.4, 6.0)):
            v.ekle("sparge_acid_mph", {"hco3": h, "ph_w": phw, "ph_t": pht}, ref.sparge_mpH(h, phw, pht))
            v.ekle("sparge_acid_mph_ours", {"hco3": h, "ph_w": phw, "ph_t": pht, "acid": 1}, ref.sparge_mpH_bizim(h, phw, pht, 1))
            for acid in (1, 2):
                v.ekle("sparge_acid_bw", {"hco3": h, "ph_w": phw, "ph_t": pht, "acid": acid}, ref.sparge_bnw_guarded(h, phw, pht, acid))
    for model, bizim, acid, conc, dph, vsp in (
        (0, False, 1, 0.5, 0.2, 15.3), (0, True, 2, 0.5, 0.0, 15.3), (1, False, 1, 88.0, 0.0, 15.3),
        (0, False, 1, 88.0, 0.2, 0.0), (1, True, 2, 75.0, 0.3, 4.2), (0, True, 1, 50.0, 0.05, 30.0),
        (0, False, 4, 88.0, 0.2, 15.3), (1, False, 8, 88.0, 0.2, 15.3),
    ):
        v.ekle("sparge_acid_amount",
               {"model": model, "ours": bizim, "hco3": 150.0, "ph_w": 7.5, "ph_t": 5.4, "acid": acid, "conc": conc,
                "mash_dph": dph, "v_sp_l": vsp},
               ref.sparge_acid_amount(model, bizim, 150.0, 7.5, 5.4, acid, conc, dph, vsp))
    for vt, w, t in ((30.0, 4.9, 3.0), (25.0, 5.5, 2.6)):
        v.ekle("sparge_volume_l", {"total_l": vt, "w_kg": w, "t": t}, ref.sparge_volume_l(vt, w, t))

    # --- §3 su profilleri (T1.14c)
    wl = [(10.0, [50.0, 10.0, 15.0, 40.0, 30.0, 150.0]), (5.0, [20.0, 2.0, 5.0, 10.0, 8.0, 40.0])]
    for waters in (wl, wl[:1], [], [(0.0, [1.0] * 6)], [(3.0, [0.0] * 6), (1.0, [8.0] * 6)]):
        v.ekle("base_profile", {"waters": [[a, i] for a, i in waters]}, ref.base_profile(waters))
    for phs in ([(10.0, 7.2), (5.0, 6.4)], [(4.0, 8.0)], [], [(0.0, 5.0)]):
        v.ekle("water_ph", {"waters": [[a, p] for a, p in phs]}, ref.water_ph_mix(phs))
    for gr, vol, ign in (({"gypsum": 5, "cacl2": 3}, 20.0, False), ({"gypsum": 5, "cacl2": 3}, 20.0, True), ({"gypsum": 5}, 0.0, False)):
        v.ekle("agents_ppm", {"grams": gr, "vol_l": vol, "ignore": ign}, ref.agents_ppm(gr, vol, ign))
    for rtype in (0, 1, 2):
        for use in (0, 1, 2, 3, 4, 5):
            v.ekle("agent_volume", {"rtype": rtype, "use": use, "mash_tot": 14.7, "sparge": 15.3, "boil": 28.0, "volume": 20.0, "final": 19.0},
                   ref.agent_volume(rtype, use, 14.7, 15.3, 28.0, 20.0, 19.0))
    mi, si = [162.33, 10.0, 15.0, 179.45, 125.81, 150.0], [60.0, 8.0, 12.0, 50.0, 40.0, 100.0]
    for hm, vm, vs in ((True, 14.7, 15.3), (True, 14.7, 0.0), (False, 14.7, 15.3), (True, 0.0, 10.0)):
        v.ekle("total_water_ion", {"mash": mi, "sparge": si, "vm": vm, "vs": vs, "has_mash": hm}, ref.total_water_ion(mi, si, vm, vs, hm))
    for va, vs, vt in ((20.0, 10.0, 5.0), (20.0, 10.0, 0.0), (0.0, 0.0, 4.0)):
        v.ekle("top_up_factor", {"v_avail": va, "vs": vs, "v_topk": vt}, ref.top_up_factor(va, vs, vt))
    adj = [5.0, 0.0, 1.0, 12.0, 3.0, 0.0]
    v.ekle("boil_ion", {"total": mi, "k_top": 0.8, "adj0": adj}, ref.boil_ion(mi, 0.8, adj))
    v.ekle("beer_ion", {"boil": mi, "s_boil": 1.21739, "adj2": adj}, ref.beer_ion(mi, 1.21739, adj))

    # --- §7.1 ölçek, §7.2 gram, §7.4 su profili aracı
    for mr, hb in ((True, True), (True, False), (False, True), (False, False)):
        v.ekle("match_scale", {"mash_recipe": mr, "has_boil_ing": hb, "s_boil": 1.21739}, ref.match_scale(mr, hb, 1.21739))
    xs = [0.95, 0.12, 0.65, 0.0, 0.0, 0.0, 0.07, 0.0]
    for vm, vsp, comb in ((3.88, 4.04, False), (3.88, 4.04, True), (5.28, 0.0, False), (5.28, 0.0, True)):
        v.ekle("salt_grams", {"x": xs, "v_mash_gal": vm, "v_sp_gal": vsp, "combine": comb}, ref.salt_grams(xs, vm, vsp, comb))
    for vb, vd in ((10.0, 5.0), (10.0, 0.0), (0.0, 5.0), (0.0, 0.0)):
        b_, d_ = [50.0, 10.0, 15.0, 40.0, 30.0, 150.0], [5.0, 1.0, 2.0, 3.0, 4.0, 10.0]
        v.ekle("profile_mix", {"vb": vb, "b": b_, "vd": vd, "d": d_}, ref.profile_mix(vb, b_, vd, d_), "Vb+Vd = 0: sıfır profil (seçim)")
    v.ekle("wp_grams", {"x": xs, "vb_gal": 2.64}, ref.wp_grams(xs, 2.64))

    # --- §6.8 uyarılar
    for model, sg in ((0, {"gypsum": 5}), (1, {"chalk": 2.0}), (1, {}), (0, {"chalk": 1.0}), (0, {"chalk": 0.0}), (1, {"gypsum": 3, "chalk": 0.5})):
        v.ekle("warnings", {"model": model, "salt_grams": sg}, ref.uyarilar(model, sg))

    # --- §7 NNLS
    kullanildi_scipy = False
    # a_cols: sütun listesi (her sütun m uzunlukta)
    nn = [
        ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], [3.0, -2.0, 5.0]),
        ([[1.0, 0.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 1.0], [2.0, 1.0, 0.0]], [2.0, 1.0, 3.0]),
        ([[2.0, 0.0, 1.0], [1.0, 3.0, 1.0], [0.5, 1.0, 4.0], [0.0, 2.0, 1.0], [1.0, 0.5, 0.0]], [1.0, -4.0, 2.0]),
        ([[1.0, 6.0], [2.0, 1.0], [3.0, 0.0], [4.0, 2.0], [5.0, 1.0], [6.0, 1.0]], [10.0, 3.0]),
        ([[0.0, 1.0], [0.0, 1.0]], [1.0, 1.0]),
        ([[1.0, 0.5], [0.5, 1.0]], [-1.0, -1.0]),
        ([[1.0, 2.0, 0.0, 1.0], [0.0, 1.0, 3.0, 1.0], [2.0, 0.0, 1.0, 1.0], [1.0, 1.0, 1.0, 0.0]], [5.0, -1.0, 4.0, 2.0]),
    ]
    for cols, b in nn:
        x, rn = ref.nnls_lh(cols, b)
        assert ref.nnls_kkt_ok(cols, b, x), (cols, b, x)
        kullanildi_scipy |=_scipy_kontrol(cols, b, x, rn)
        v.ekle("nnls", {"a_cols": cols, "b": b}, {"x": x, "rnorm": rn})

    # tuz eşleştirme: hedef − s·base üzerinde NNLS; beklenen profil = s·base + A·x (gram dağılımı tek değil)
    def eslestir(base, tgt, s, haric):
        r = ref.water_match_ref(base, tgt, s, haric)
        assert ref.nnls_kkt_ok(r["cols"], r["b"], r["xa"]), "NNLS KKT koşulu sağlanmadı"
        _scipy_kontrol(r["cols"], r["b"], r["xa"], r["rnorm"])
        # gram dağılımı tek olmayabilir: profil ve ‖r‖ sabitlenir; hariç tuzların 0 olması ve KKT Rust testinde denetlenir
        return {"profile": r["profile"], "rnorm": r["rnorm"]}

    base = [ref.base[k] for k in ions]
    hedef = [100.0, 15.0, 20.0, 250.0, 60.0, 150.0]
    for base_, tgt, s, haric in (
        (base, hedef, 1.0, []),
        (base, hedef, 1.0, ["chalk", "lime", "mgcl2"]),
        (base, [30.0, 5.0, 10.0, 60.0, 10.0, 40.0], 1.0, ["chalk", "lime", "mgcl2"]),  # hedef base'in altında
        (base, hedef, 1.21739, ["chalk", "lime", "mgcl2"]),
        ([0.0] * 6, [50.0, 10.0, 5.0, 100.0, 70.0, 20.0], 1.0, []),
        ([10.0, 3.0, 8.0, 12.0, 9.0, 35.0], [140.0, 10.0, 25.0, 300.0, 100.0, 60.0], 1.0, ["chalk", "lime"]),
    ):
        v.ekle("water_match", {"base": base_, "target": tgt, "s": s, "excluded": haric}, eslestir(base_, tgt, s, haric))

    print("scipy karşılaştırması:", "yapıldı" if kullanildi_scipy else "scipy yok, atlandı")
    _uret_hatalar(v, ref)
    return v.yaz()


def _uret_hatalar(v, ref):
    """T1.16: referansın istisna attığı ya da motorun hata döndürdüğü girdiler ({"hata": ...})."""
    n = "T1.16: hata"
    for a in (4, 5, 6, 7, 8):
        v.ekle("acid_per_liter", {"meq": 2.0, "acid": a, "conc": 50.0, "model": 0, "ph_t": 5.4, "ours": False},
               dene(ref.acid_per_L, 2.0, a, 50.0, 0, 5.4, False), "tanımsız mash asidi: " + n)
        v.ekle("mash_acid_amount", {"est_ph": 5.8, "target_ph": 5.4, "model": 0, "t": 3.0, "w_kg": 5.0, "acid": a,
                                    "conc": 50.0, "ours": False},
               dene(ref.mash_acid_amount, 5.8, 5.4, 0, 3.0, 5.0, a, 50.0, False), "tanımsız mash asidi: " + n)
    v.ekle("acid_pks", {"acid": 9}, dene(ref.pks, 9), "bilinmeyen asit kodu: " + n)
    v.ekle("acid_per_liter", {"meq": 2.0, "acid": 1, "conc": 0.0, "model": 0, "ph_t": 5.4, "ours": False},
           dene(ref.acid_per_L, 2.0, 1, 0.0, 0, 5.4, False), "derişim 0: " + n)
    for model, t, w in ((1, 0.0, 5.0), (0, 3.0, 0.0), (0, -1.0, 5.0)):
        v.ekle("mash_acid_amount", {"est_ph": 5.8, "target_ph": 5.4, "model": model, "t": t, "w_kg": w, "acid": 1,
                                    "conc": 88.0, "ours": False},
               dene(ref.mash_acid_amount, 5.8, 5.4, model, t, w, 1, 88.0, False), "kalınlık/ağırlık <= 0: " + n)
    for bill, t in (([], 3.0), ([(4.0, 3.0)], 0.0)):
        v.ekle("adjusted_ph", {"bill": [list(x) for x in bill], "acid_malt_g": 0.0, "pct": 3.0, "model": 0, "ra": 10.0,
                               "t": t, "target_ph": 5.4, "ours": False},
               dene(ref.adj_ph, bill, 0.0, 3.0, 0, 10.0, t, False, 5.4), "boş tarif ya da kalınlık 0: " + n)
    v.ekle("acid_malt_amount", {"bill": [[4.0, 3.0]], "pct": 0.0, "model": 0, "ra": 50.0, "t": 3.0, "target_ph": 5.4,
                                "ours": False},
           dene(ref.acid_malt_amount, [(4.0, 3.0)], 0.0, 0, 50.0, 3.0, 5.4, False), "asit malt yüzdesi 0: " + n)
    v.ekle("boil_scale", {"vb": 28.0, "vp": 1.0, "cool_loss": 1.0, "trub_loss": 0.0, "top_up": 0.0, "has_boil_ing": True},
           dene(ref.boil_scale, 28.0, 1.0, 1.0, 0.0, 0.0, True), "Vp = soğuma kaybı: " + n)
    v.ekle("boil_scale", {"vb": 28.0, "vp": 24.0, "cool_loss": 1.0, "trub_loss": 23.0, "top_up": 0.0, "has_boil_ing": True},
           dene(ref.boil_scale, 28.0, 24.0, 1.0, 23.0, 0.0, True), "b + top_up = 0: " + n)
    v.ekle("misc_acid_ra", {"acid": 1, "conc": 88.0, "vol_l": 0.01, "f": 1.0, "w_kg": 0.0, "t": 3.0},
           dene(ref.misc_acid_ra, 1, 88.0, 0.01, 1.0, 0.0, 3.0), "tahıl ağırlığı 0: " + n)
