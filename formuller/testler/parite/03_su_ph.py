"""Bölüm 03 (su kimyası, mash pH, NNLS) parite vektörleri (ADR-0003).

Referans: formuller/testler/03_su_ph_tv.py. Referansta fonksiyon olarak bulunmayan hesaplar (S_boil,
renk aralığı, SO4/Cl, mash asidi miktarı, asit malt yinelemesi, NNLS) referanstaki ifadelerin ve
fonksiyonların birebir bileşimidir; ifade kaynağı her yerde yazılıdır.

NNLS: scipy yok (CI'da da yok), bu yüzden saf Python Lawson–Hanson burada yazılır (ADR-0003 madde 8).
scipy varsa sonuçlar onunla da karşılaştırılır.
"""
import math

from ortak import Vektor, yukle

EPS = 2.220446049250313e-16


# ---------------------------------------------------------------- saf Python Lawson–Hanson NNLS
def _lstsq(cols, b):
    """En küçük kareler: Householder QR (sütun öncelikli). Rust `water::lstsq` ile aynı işlem sırası."""
    m = len(b)
    k = len(cols)
    a = [list(c) for c in cols]
    qtb = list(b)
    for j in range(k):
        norm = math.sqrt(sum(a[j][i] * a[j][i] for i in range(j, m)))
        if norm == 0.0:
            continue
        alpha = -norm if a[j][j] > 0.0 else norm
        v = a[j][j:]
        v = list(v)
        v[0] -= alpha
        vn2 = sum(x * x for x in v)
        if vn2 == 0.0:
            continue
        for c in range(j, k):
            dot = sum(v[i] * a[c][j + i] for i in range(len(v)))
            f = 2.0 * dot / vn2
            for i in range(len(v)):
                a[c][j + i] -= f * v[i]
        dot = sum(v[i] * qtb[j + i] for i in range(len(v)))
        f = 2.0 * dot / vn2
        for i in range(len(v)):
            qtb[j + i] -= f * v[i]
    x = [0.0] * k
    for i in range(k - 1, -1, -1):
        s = qtb[i]
        for c in range(i + 1, k):
            s -= a[c][i] * x[c]
        d = a[i][i]
        x[i] = s / d if abs(d) > 1e-300 else 0.0
    return x


def nnls_lh(cols, b):
    """Lawson–Hanson NNLS: min ||A x - b||, x >= 0. cols: sütun listesi. Döner: (x, rnorm)."""
    n = len(cols)
    m = len(b)
    x = [0.0] * n
    passive = [False] * n
    anorm = max((sum(abs(v) for v in c) for c in cols), default=0.0)
    tol = 10.0 * max(m, n) * EPS * anorm
    max_iter = 3 * n
    it = 0
    while True:
        r = [b[i] - sum(cols[j][i] * x[j] for j in range(n)) for i in range(m)]
        w = [sum(cols[j][i] * r[i] for i in range(m)) for j in range(n)]
        best = -1
        for j in range(n):
            if not passive[j] and w[j] > tol and (best < 0 or w[j] > w[best]):
                best = j
        if best < 0:
            break
        passive[best] = True
        while True:
            idx = [j for j in range(n) if passive[j]]
            s_p = _lstsq([cols[j] for j in idx], b)
            if all(v > 0.0 for v in s_p):
                for j in range(n):
                    x[j] = 0.0
                for j, v in zip(idx, s_p):
                    x[j] = v
                break
            alpha = min(x[j] / (x[j] - s) for j, s in zip(idx, s_p) if s <= 0.0)
            for j, s in zip(idx, s_p):
                x[j] += alpha * (s - x[j])
            for j in idx:
                if x[j] <= tol:
                    x[j] = 0.0
                    passive[j] = False
            it += 1
            if it > max_iter:
                break
        it += 1
        if it > max_iter:
            break
    r = [b[i] - sum(cols[j][i] * x[j] for j in range(n)) for i in range(m)]
    return x, math.sqrt(sum(v * v for v in r))


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
    ]:
        v.ekle("salt_ppm", {"grams": gr, "vol_l": vol}, ref.salt_ppm(gr, vol))
    for s in salts:
        v.ekle("salt_ppm", {"grams": {s: 10}, "vol_l": 25}, ref.salt_ppm({s: 10}, 25), "tek tuz: " + s)

    # --- §4 alkalinite, sertlik, RA
    for h in (0, 61, 122.5, 150, 300.5):
        v.ekle("alkalinity", {"hco3": h}, ref.alk(h))
    for ca, mg in ((0, 0), (50, 10), (112.33, 0), (162.33, 10), (5, 80)):
        v.ekle("hardness", {"ca": ca, "mg": mg}, ref.hard(ca, mg))
    for ca, mg, h in ((50, 10, 150), (162.33, 10, 150), (0, 0, 0), (10, 2, 20), (200, 30, 40), (80, 5, 400)):
        v.ekle("residual_alkalinity", {"ca": ca, "mg": mg, "hco3": h}, ref.RA(ca, mg, h))

    # §4.4 SO4/Cl (referans: print("T5 SO4/Cl", mash[3]/mash[4]); Cl=0 koruması spesifikasyondan: 0)
    mash = [ref.base[k] + ref.salt_ppm({"gypsum": 5, "cacl2": 3}, 20)[i] for i, k in enumerate(ions)]
    v.ekle("sulfate_chloride_ratio", {"so4": mash[3], "cl": mash[4]}, mash[3] / mash[4])
    v.ekle("sulfate_chloride_ratio", {"so4": 100.0, "cl": 25.0}, 100.0 / 25.0)
    v.ekle("sulfate_chloride_ratio", {"so4": 40.0, "cl": 0.0}, 0.0, "Cl = 0 koruması (spesifikasyon §4.4 önerisi)")

    # §4.5 renk aralığı (referans: 5.2+0.082*ra, (ra+122.4)/12.2; max(0, ·) spesifikasyondan)
    for ra in (1.1166, 81.354, 0.0, -50.0, -200.0, 150.0):
        v.ekle("color_range", {"ra": ra}, [max(0.0, 5.2 + 0.082 * ra), max(0.0, (ra + 122.4) / 12.2)])

    # §3.4 kaynatma derişim faktörü (referans T7 ifadesi; spesifikasyon koruması Vb>0 ve Vp>0)
    def boil_scale(vb, vp, cool, trub, top):
        if not (vb > 0 and vp > 0):
            return 1.0
        a_ = vp - cool
        b_ = a_ - trub
        return ((vb / a_) * b_ + top) / (b_ + top)

    for vb, vp, cool, trub, top in (
        (28.0, 24.0, 1.0, 1.0, 0.0), (28.0, 24.0, 1.0, 1.0, 2.0), (30.0, 25.5, 0.5, 1.5, 3.0),
        (0.0, 24.0, 1.0, 1.0, 0.0), (28.0, 0.0, 1.0, 1.0, 0.0),
    ):
        v.ekle("boil_scale", {"vb": vb, "vp": vp, "cool_loss": cool, "trub_loss": trub, "top_up": top},
               boil_scale(vb, vp, cool, trub, top))

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
            gp, s = ref.gristph(b, bizim)
            v.ekle("grist_ph", {"bill": b, "ours": bizim}, [gp, s])

    # --- §5.2 eğim
    for model in (0, 1):
        for t in (1.5, 2.0, 2.6, 3.0, 4.0):
            v.ekle("mash_ra_slope", {"model": model, "t": t}, ref.slope(model, t))

    # --- §5.3 tahmini pH (referans: gp + ra/50*sl)
    gp0 = ref.gristph(bills[0])[0]
    for gp, ra, t, model in ((gp0, 1.1166, 3.0, 0), (gp0, 1.1166, 3.0, 1), (5.4, 81.354, 2.5, 0), (5.72, -30.0, 4.0, 1)):
        sl = ref.slope(model, t)
        v.ekle("est_mash_ph", {"grist_ph": gp, "ra": ra, "slope": sl}, gp + ra / 50 * sl)

    # --- §6.1 pK ve ayrışma kesri
    for a in range(0, 10):
        v.ekle("acid_pks", {"acid": a}, ref.pks(a))
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
                           ref.acid_per_L(meq, a, conc, model, pht, bizim))

    # --- §6.3 mash asidi miktarı (referans T3: dph/sl → acid_per_L · W · t; koruma kuralları spesifikasyondan)
    def mash_acid(est, target, model, t, w, acid, conc, bizim):
        dph = est - target
        if acid == 0 or dph < 0:
            return 0.0
        sl = ref.slope(model, t)
        return ref.acid_per_L(dph / sl, acid, conc, model, target, bizim) * w * t

    est0 = gp0 + 1.1166 / 50 * ref.slope(0, 3.0)
    for est, target, model, t, w, acid, conc, bizim in (
        (est0, 5.4, 0, 3.0, 4.9, 1, 88.0, False), (est0, 5.4, 0, 3.0, 4.9, 1, 88.0, True),
        (est0, 5.4, 1, 3.0, 4.9, 1, 88.0, False), (est0, 5.4, 1, 3.0, 4.9, 1, 88.0, True),
        (est0, 5.4, 0, 3.0, 4.9, 2, 85.0, True), (est0, 5.2, 0, 2.5, 5.5, 3, 3.0, False),
        (est0, 5.4, 0, 3.0, 4.9, 0, 88.0, False), (5.3, 5.4, 0, 3.0, 4.9, 1, 88.0, False),
    ):
        v.ekle("mash_acid_amount",
               {"est_ph": est, "target_ph": target, "model": model, "t": t, "w_kg": w, "acid": acid, "conc": conc, "ours": bizim},
               mash_acid(est, target, model, t, w, acid, conc, bizim))

    # --- §5.4(a) laktik asidin RA eşdeğeri (referans T3 kontrolü: acidra = -(p·1000·ρ·VL·f)/(0.09009·W·t))
    for conc, vol_l, f, w, t in ((88.0, 0.0035, 1.0, 4.9, 3.0), (88.0, 0.0036, 0.971968, 4.9, 3.0), (80.0, 0.01, 0.9, 2.0, 2.5)):
        p_ = conc / 100
        rho = 1 + 0.237 * p_
        v.ekle("lactic_acid_ra", {"conc": conc, "vol_l": vol_l, "f": f, "w_kg": w, "t": t},
               -(p_ * 1000 * rho * vol_l * f) / (0.09009 * w * t))

    # --- §5.5 ayarlanmış pH (referans adj_ph: asit maltlı)
    for model in (0, 1):
        for bizim in (False, True):
            for ag in (0.0, 62.78, 128.88):
                ph, w = ref.adj_ph(bills[0], ag, 3.0, model, 1.1166, 3.0, bizim)
                v.ekle("adjusted_ph", {"bill": bills[0], "acid_malt_g": ag, "pct": 3.0, "model": model, "ra": 1.1166,
                                       "t": 3.0, "target_ph": 5.4, "ours": bizim}, [ph, w])
    ph, w = ref.adj_ph(bills[3], 80.0, 2.0, 0, 20.0, 2.5, True)
    v.ekle("adjusted_ph", {"bill": bills[3], "acid_malt_g": 80.0, "pct": 2.0, "model": 0, "ra": 20.0,
                           "t": 2.5, "target_ph": 5.4, "ours": True}, [ph, w])

    # --- §6.4 asit malt yinelemesi (referansın üst düzey döngüsü, birebir)
    def acid_malt(bill, pct, model, ra, t, bizim):
        tot = 0.0
        ph2 = 0.0
        for _ in range(8):
            ph, w = ref.adj_ph(bill, tot, pct, model, ra, t, bizim)
            sl = ref.slope(model, t)
            amt = ref.acid_per_L((ph - 5.4) / sl, 3, pct, model, 5.4, bizim) * w * t if ph - 5.4 >= 0 else 0
            tot += amt
            ph2, _ = ref.adj_ph(bill, tot, pct, model, ra, t, bizim)
            if ph2 - 5.4 < 0.005:
                break
        return [tot, ph2]

    for bill, pct, model, ra, t in (
        (bills[0], 3.0, 0, 1.1166, 3.0), (bills[0], 3.0, 1, 1.1166, 3.0), (bills[3], 2.5, 0, 40.0, 2.5),
        (bills[3], 2.5, 1, 40.0, 2.5), (bills[1], 3.0, 0, -20.0, 3.0), (bills[2], 4.0, 0, 60.0, 3.5),
    ):
        for bizim in (False, True):
            v.ekle("acid_malt_amount", {"bill": bill, "pct": pct, "model": model, "ra": ra, "t": t, "target_ph": 5.4, "ours": bizim},
                   acid_malt(bill, pct, model, ra, t, bizim))

    # --- §6.5 sparge asidi
    for h in (0.0, 50.0, 150.0, 300.0):
        for phw, pht in ((7.5, 5.4), (7.5, 6.0), (8.2, 5.4), (6.5, 6.0), (7.0, 5.7)):
            v.ekle("sparge_acid_mph", {"hco3": h, "ph_w": phw, "ph_t": pht}, ref.sparge_mpH(h, phw, pht))
            v.ekle("sparge_acid_mph_ours", {"hco3": h, "ph_w": phw, "ph_t": pht, "acid": 1}, ref.sparge_mpH_bizim(h, phw, pht, 1))
            for acid in (1, 2):
                v.ekle("sparge_acid_bw", {"hco3": h, "ph_w": phw, "ph_t": pht, "acid": acid}, ref.sparge_bnw(h, phw, pht, acid))
    v.ekle("sparge_acid_mph_ours", {"hco3": 150.0, "ph_w": 7.5, "ph_t": 6.0, "acid": 2}, ref.sparge_mpH_bizim(150.0, 7.5, 6.0, 2))
    for mol, vol, conc in ((0.00220511, 15.3, 88.0), (0.00229262, 15.3, 88.0), (0.00166758, 15.3, 88.0), (0.002, 10.0, 0.5), (0.0, 15.3, 88.0), (0.003, 20.0, 80.0)):
        v.ekle("lactic_ml", {"mol_l": mol, "v_l": vol, "conc": conc}, ref.lactic_mL(mol, vol, conc))
    for vt, w, t in ((30.0, 4.9, 3.0), (25.0, 5.5, 2.6)):
        v.ekle("sparge_volume_l", {"total_l": vt, "w_kg": w, "t": t}, vt - w * t)

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
        x, rn = nnls_lh(cols, b)
        kullanildi_scipy |= _scipy_kontrol(cols, b, x, rn)
        v.ekle("nnls", {"a_cols": cols, "b": b}, {"x": x, "rnorm": rn})

    # tuz eşleştirme: hedef − s·base üzerinde NNLS; beklenen profil = s·base + A·x (gram dağılımı tek değil)
    def eslestir(base, tgt, s, haric):
        aktif = [k for k in salts if k not in haric]
        cols = [ref.A[k] for k in aktif]
        b = [tgt[i] - s * base[i] for i in range(6)]
        x, rn = nnls_lh(cols, b)
        _scipy_kontrol(cols, b, x, rn)
        prof = [s * base[i] + sum(x[j] * cols[j][i] for j in range(len(cols))) for i in range(6)]
        return {"profile": prof, "rnorm": rn}

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
    return v.yaz()
