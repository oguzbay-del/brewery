"""Bölüm 02 (hop ve IBU) parite vektörleri: formuller/02-hop-ibu.md, referans formuller/testler/02_hop_ibu.py."""
from ortak import Vektor, dene, fikstur, yukle

SGLER = (1.000, 1.030, 1.040, 1.050, 1.051, 1.070, 1.100)


def uret():
    ref = yukle("02_hop_ibu.py")
    ref01 = yukle("01_yogunluk_tv.py")      # ortak reçete kurucuları ve geç ekstrakt profili (T1.16 B)
    fx = fikstur(ref01)
    v = Vektor("02", "formuller/testler/02_hop_ibu.py")

    # §2.1 Tinseth: t=0 kenarı (U=0), SG 1.000 ve yüksek SG
    for sg in SGLER:
        for t in (0, 5, 20, 60, 120):
            v.ekle("tinseth_utilization", {"sg": sg, "t": t}, ref.tin_util(sg, t))

    # §3.1 sıcaklık faktörü: 100 °C süreksizliği, whirlpool ve rakım sıcaklıkları
    for tc in (-10.0, 0.0, 60.0, 80.0, 95.0622, 99.0, 99.99, 100.0, 100.5, 120.0):
        v.ekle("temperature_factor", {"temp_c": tc}, ref.tempf(tc),
               "100 °C süreksizliği" if tc in (99.99, 100.0) else None)

    # §2.2 Rager tablosu: her basamak sınırı, negatif ve sıfır
    for n in (-3, 0, 1, 5, 6, 10, 11, 15, 16, 20, 21, 25, 26, 30, 31, 35, 36, 40, 41, 45, 46, 50,
              51, 55, 56, 60, 200):
        v.ekle("rager_table", {"n": n}, ref.rager_tab(n))
    for sg in (1.000, 1.049, 1.050, 1.051, 1.070, 1.100, 1.250):
        v.ekle("rager_gravity_adjustment", {"sg": sg}, ref.rager_ga(sg))
    # t yuvarlama: int(t+0.5) sıfıra doğru keser (0.49 -> 0, 0.5 -> 1, 5.49 -> 5, 5.5 -> 6)
    for t in (-2.0, 0.0, 0.49, 0.5, 5.49, 5.5, 10.4, 10.5, 30, 55.49, 55.5, 60, 90):
        for sg in (1.040, 1.050, 1.070):
            v.ekle("rager_utilization", {"sg": sg, "t": t}, ref.rager_util(sg, t))

    # §2.3 Garetz iteratif: araç yolu (v == bv) ve tarif yolu
    for t in (0.5, 1, 5, 6, 10, 11, 15, 20, 30, 45, 60, 70, 75, 90, 120):
        v.ekle("garetz_ibu", {"w": 1.0, "a": 5.0, "t": t, "sg": 1.050, "v": 5.0, "bv": 5.0},
               dene(ref.garetz_ibu, 1.0, 5.0, t, 1.050, 5.0, 5.0))
    # T1.15 B9 / B18: büyük miktar, yavaş yakınsama, 10 000 adım sınırı
    for w in (1e3, 1e5, 1e6, 1e9, 1e12):  # 1e9 ve üstü sınırda yakınsamaz: hata (T1.16)
        v.ekle("garetz_ibu", {"w": w, "a": 20.0, "t": 60, "sg": 1.050, "v": 5.0, "bv": 5.0},
               dene(ref.garetz_ibu, w, 20.0, 60, 1.050, 5.0, 5.0), "T1.15 B9: yavaş yakınsama, adım sınırı")
    for sg in (1.030, 1.040, 1.050, 1.070):
        v.ekle("garetz_ibu", {"w": 2.0, "a": 12.0, "t": 60, "sg": sg, "v": 5.0, "bv": 6.5},
               dene(ref.garetz_ibu, 2.0, 12.0, 60, sg, 5.0, 6.5), "v != bv: konsantrasyon faktörü")
    v.ekle("garetz_ibu", {"w": 4.0, "a": 8.0, "t": 30, "sg": 1.060, "v": 10.0, "bv": 12.0},
           dene(ref.garetz_ibu, 4.0, 8.0, 30, 1.060, 10.0, 12.0))
    # ön koşullar: t<=0, w<=0, a<0 -> 0; a=0
    for (w, a, t) in ((1.0, 5.0, 0), (1.0, 5.0, -5), (0.0, 5.0, 60), (-1.0, 5.0, 60),
                      (1.0, -1.0, 60), (1.0, 0.0, 60)):
        v.ekle("garetz_ibu", {"w": w, "a": a, "t": t, "sg": 1.050, "v": 5.0, "bv": 5.0},
               dene(ref.garetz_ibu, w, a, t, 1.050, 5.0, 5.0), "ön koşul kenarı")
    # T1.16: hacim <= 0 ve gravite çarpanı <= 0 hata (eskiden NaN / negatif IBU)
    for (sg, vv, bv) in ((1.050, 0.0, 5.0), (1.050, 5.0, 0.0), (1.050, -5.0, 5.0), (0.80, 5.0, 5.0), (0.0, 5.0, 5.0)):
        v.ekle("garetz_ibu", {"w": 1.0, "a": 5.0, "t": 60, "sg": sg, "v": vv, "bv": bv},
               dene(ref.garetz_ibu, 1.0, 5.0, 60, sg, vv, bv), "T1.16: geçersiz hacim ya da gravite: hata")
    for sg in (1.040, 1.050, 1.070):
        for t in (0.5, 5, 10, 15, 20, 30, 45, 60, 90, 130):
            v.ekle("garetz_utilization", {"sg": sg, "t": t}, ref.garetz_util(sg, t))
    v.ekle("garetz_utilization", {"sg": 1.050, "t": 0}, ref.garetz_util(1.050, 0), "t=0 -> 0")

    # §3.5 PostBoilUtil: carry, 0 dk hop (tau<5 istisnası), t0 sınırları, dur=0
    for sg in (1.040, 1.050, 1.070):
        for t0 in (0, 2, 4.9, 5, 15, 60):
            for dur in (0, 5, 20, 30):
                v.ekle("post_boil_utilization", {"sg": sg, "t0": t0, "dur": dur},
                       ref.postboil(sg, t0, dur))

    # §3.5 T1.15 B1: uzun whirlpool (0 K sınırı 498.74 dk, adım üst sınırı, 1e7 dk)
    for sg in (1.050,):
        for t0 in (0, 60):
            for dur in (60, 120, 157, 300, 498, 499, 500, 600, 2000, 10000, 1e7):
                v.ekle("post_boil_utilization", {"sg": sg, "t0": t0, "dur": dur},
                       ref.postboil(sg, t0, dur), "T1.15 B1: uzun süre, sonlu ve sınırlı")

    # §3.2 rakım -> kaynama noktası
    for alt in (-500, 0, 1000, 3000, 5000, 8000):
        v.ekle("boil_temp_c", {"alt_ft": alt}, ref.boiltempC(alt))

    # §5.2 olasılıksal birleştirme: boş, tek, çok segmentli, 1 içeren
    for segs in ([], [0.0], [0.230664], [0.230664, 0.008003], [0.1, 0.2, 0.3], [1.0, 0.5],
                 [0.5, 1.0]):
        v.ekle("combine_utilizations", {"segs": segs}, ref.combine(segs))
    segs = [ref.tin_util(1.040, 60) - ref.tin_util(1.040, 15),
            ref.tin_util(1.050, 15) - ref.tin_util(1.050, 0)]
    v.ekle("combine_utilizations", {"segs": segs}, ref.combine(segs), "§12 #16 geç ekstrakt")
    segs = [ref.tin_util(1.050, 60), ref.postboil(1.050, 60, 20)]
    v.ekle("combine_utilizations", {"segs": segs}, ref.combine(segs), "§12 #10 carry")

    # §10 hop yaşlanması: HSI üst sınırı (99), 0 gün, depolama faktörleri, ısı
    for (a, hsi, tf, days, sf) in (
        (5, 30, 68, 180, 1.0), (5, 30, 40, 180, 0.5), (5, 30, 68, 180, 0.75),
        (12.5, 50, 77, 360, 1.0), (7.0, 100, 68, 180, 1.0), (7.0, 99, 68, 180, 1.0),
        (6.0, 0.0, 68, 180, 1.0), (6.0, 25, 68, 0, 1.0), (9.0, 40, 32, 90, 0.5),
    ):
        v.ekle("aged_alpha", {"alpha": a, "hsi": hsi, "temp_f": tf, "days": days, "storage": sf},
               ref.hop_alpha(a, hsi, tf, days, sf))
    v.ekle("aged_alpha", {"alpha": 5.0, "hsi": -1.0, "temp_f": 68, "days": 180, "storage": 1.0},
           dene(ref.hop_alpha, 5.0, -1.0, 68, 180, 1.0), "T1.16: negatif HSI hata")

    # ---- T1.14a: referansa sonradan eklenen fonksiyonlar ----
    opts = dict(ref.DEFAULT_OPTS)
    opts_off = dict(opts, type_adj=False)
    opts_x = dict(opts, first_wort_adj_pct=25.0, leaf_adj_pct=-5.0, plug_adj_pct=3.0, pellet_adj_pct=12.0,
                  mash_adj_pct=-50.0)

    def hop(amount=1.0, alpha=5.0, form=0, use=0, time=60, wtemp=176.0):
        return ref01.hop(amount, alpha, form, use, time, wtemp)

    def rec(ferms=None, hops=None, opts=None, **kw):
        """Ortak tarif (RecipeInput); eski anahtar adları (volume, trub_loss, type ...) çağrılarda korunur."""
        ad = {"volume": "volume_floz", "trub_loss": "trub_loss_floz", "boil_time": "boil_time_min",
              "boil_vol": "boil_vol_floz", "altitude": "altitude_ft", "type": "typ", "hop_util": "hop_util_pct",
              "whirlpool_time": "whirlpool_time_min"}
        a = dict(typ=2, volume_floz=640.0, boil_vol_floz=768.0, boil_time_min=60.0)
        a.update({ad.get(k, k): x for k, x in kw.items()})
        return ref01.make_recipe(ferms or [], hops=hops, hop_options=opts, **a)

    ferms_a, ferm_late, ferm_hopped = fx.ferms_a, fx.ferm_late, fx.ferm_hopped

    # §2.1 / §2.2 tek hop IBU ve ters (ön koşul kenarları dahil)
    for (w, a, t, sg, vg) in ((1.0, 5.0, 60, 1.050, 5.0), (2.0, 12.0, 30, 1.070, 6.0), (1.0, 5.0, 0, 1.05, 5.0),
                              (0.0, 5.0, 60, 1.05, 5.0), (1.0, -1.0, 60, 1.05, 5.0), (1.0, 0.0, 60, 1.05, 5.0),
                              (1.0, 5.0, 60, 1.05, 0.0), (1.5, 8.0, 10.4, 1.04, 5.5)):
        g = {"w": w, "a": a, "t": t, "sg": sg, "v": vg}
        v.ekle("tinseth_ibu", g, ref.tin_ibu(w, a, t, sg, vg))
        v.ekle("rager_ibu", g, ref.rager_ibu(w, a, t, sg, vg))
    for (ibu, vg, sg, t, a) in ((17.2137, 5.0, 1.05, 60, 5.0), (40.0, 5.5, 1.07, 30, 12.0), (30.0, 5.0, 1.05, 0, 5.0),
                                (30.0, 5.0, 1.05, 60, 0.0), (0.0, 5.0, 1.05, 60, 5.0)):
        g = {"ibu": ibu, "v": vg, "sg": sg, "t": t, "a": a}
        v.ekle("tinseth_weight", g, dene(ref.tin_weight, ibu, vg, sg, t, a))
        v.ekle("rager_weight", g, dene(ref.rager_weight, ibu, vg, sg, t, a),
               "R(n)=0 veya A=0 -> 0 (seçim)" if t == 0 or a == 0 else None)
    # §2.3 Garetz ters (ikiye bölme)
    for (ibu, a, t, sg, vg) in ((30.0, 5.0, 60, 1.050, 5.0), (45.0, 12.0, 45, 1.060, 5.5), (20.0, 7.0, 30, 1.040, 6.0),
                                (60.0, 14.0, 90, 1.070, 5.0), (15.0, 4.5, 20, 1.050, 5.0)):
        v.ekle("garetz_weight", {"ibu": ibu, "a": a, "t": t, "sg": sg, "v": vg},
               dene(ref.garetz_weight, ibu, a, t, sg, vg))

    # T1.15 B10: ulaşılamaz hedef (tablo değeri 0, t <= 10 dk) -> 0
    for (ibu, a, t, sg, vg) in ((30.0, 5.0, 5, 1.050, 5.0), (30.0, 5.0, 10, 1.050, 5.0),
                                (30.0, 5.0, 0.5, 1.050, 5.0), (30.0, 0.0, 60, 1.050, 5.0),
                                (1e9, 5.0, 60, 1.050, 5.0)):
        v.ekle("garetz_weight", {"ibu": ibu, "a": a, "t": t, "sg": sg, "v": vg},
               dene(ref.garetz_weight, ibu, a, t, sg, vg), "T1.16: ulaşılamaz hedef -> hata")

    # §3.4 whirlpool
    for tc in (60.0, 80.0, 90.0, 100.0):
        for t in (0, 10, 20, 45):
            g = {"w": 1.5, "a": 9.0, "t": t, "sg": 1.050, "v": 5.0, "temp_c": tc}
            v.ekle("whirlpool_ibu", g, ref.whirlpool_ibu(1.5, 9.0, t, 1.050, 5.0, tc))
    v.ekle("whirlpool_ibu", {"w": 1.0, "a": 5.0, "t": 20, "sg": 1.05, "v": 0.0, "temp_c": 80.0},
           ref.whirlpool_ibu(1.0, 5.0, 20, 1.05, 0.0, 80.0), "V=0 -> 0")
    v.ekle("whirlpool_utilization", {"sg": 1.050, "t": 20, "temp_c": 80.0}, ref.whirlpool_util(1.050, 20, 80.0),
           "§12 #7")

    # §3.2 °F kaynama noktası, §1 denklem çözümü
    for alt in (0, 5000, 10000):
        v.ekle("boil_temp_f", {"alt_ft": alt}, ref.boiltempF(alt))
    for (eq, ge) in ((0, 1), (1, 0), (2, 0), (-1, 2), (3, 1), (5, 0)):
        v.ekle("resolve_equation", {"equation": eq, "global_equation": ge}, ref.resolve_equation(eq, ge))

    # §3.3 HopUtil
    for eq in (0, 1, 2):
        for wp in (False, True):
            for tf in (203.112, 212.0, 176.0):
                for t in (0, 15, 60):
                    v.ekle("hop_util", {"sg": 1.060, "t": t, "equation": eq, "whirlpool_mu": wp, "temp_f": tf},
                           ref.hop_util(1.060, t, eq, wp, tf))

    # §4 HopAdjustments: form x use x extract_tarif x seçenekler
    for form in range(5):
        for use in range(5):
            for ex in (False, True):
                for (on, o) in (("varsayılan", opts), ("type_adj=0", opts_off), ("özel", opts_x)):
                    v.ekle("hop_adjustments", {"form": form, "use": use, "util_pct": 90.0,
                                               "extract_recipe": ex, "options": o},
                           ref.hop_adjust(form, use, 90.0, ex, o), on)

    # §5.2 HopTotalUtil / §5.3 NewHopIBU: profiller, denklemler, carry, rakım, kenarlar
    prof1 = [[1.050, 60.0], [1.050, 0.0]]
    prof_le = ref.late_extract_sgs(rec(ferm_late), 0.0)
    prof_le2 = [[1.040, 60.0], [1.050, 15.0], [1.055, 0.0]]
    prof_whirl = [[1.040, 60.0], [1.052, 0.0]]
    hop_cases = []
    for h in (hop(), hop(use=3), hop(use=2), hop(time=0), hop(time=15), hop(time=30), hop(time=75),
              hop(use=4, time=20), hop(use=4, time=0, wtemp=150.0), hop(use=1), hop(amount=0.0),
              hop(form=4, amount=0.1, alpha=50.0), hop(form=4, use=1, amount=0.1, alpha=50.0),
              hop(form=1, time=45), hop(form=2, time=10), hop(form=3, time=60), hop(use=3, time=90),
              hop(use=2, time=60, form=2)):
        hop_cases.append(h)
    carry = {"whirlpool_carry": True, "whirlpool_time": 20}
    for (pn, pr) in (("tek SG", prof1), ("geç ekstrakt (çok segment)", prof_le), ("geç 1.040/1.050", prof_le2),
                     ("whirlpool SG farklı", prof_whirl)):
        for (rn, rc) in (("deniz seviyesi", rec()), ("carry 20", rec(**carry)),
                         ("rakım 5000 + carry", rec(altitude=5000, **carry)),
                         ("Extract tarifi", rec(type=0, hop_util=80.0))):
            for eq in (0, 1, 2):
                for h in hop_cases:
                    nm = "%s, %s" % (pn, rn)
                    v.ekle("hop_total_utilization",
                           {"hop": h, "recipe": rc, "profile": pr, "equation": eq},
                           ref.hop_total_util(h, rc, pr, eq), nm)
                    if eq == 0 or rn == "deniz seviyesi":
                        v.ekle("new_hop_ibu",
                               {"hop": h, "recipe": rc, "profile": pr, "equation": eq},
                               ref.new_hop_ibu(h, rc, pr, eq), nm)
    # trub ile hacim, V+T <= 0 (0.01 alt sınırı)
    for rc in (rec(trub_loss=64.0), rec(volume=0.0, trub_loss=0.0), rec(volume=-5.0, trub_loss=0.0)):
        v.ekle("new_hop_ibu", {"hop": hop(), "recipe": rc, "profile": prof1, "equation": 0},
               ref.new_hop_ibu(hop(), rc, prof1, 0), "V_ibu kenarı")
    # §12 #12–#15 ve #10, #11 tarif düzeyi doğrulama vakaları
    v.ekle("new_hop_ibu", {"hop": hop(use=3), "recipe": rec(), "profile": prof1, "equation": 0},
           ref.new_hop_ibu(hop(use=3), rec(), prof1, 0), "§12 #13")

    # §5.4 hopped extract, hop yüzdesi, tarif toplamı (üç denklem)
    for (a_, g_, vol, tr) in ((16.0, 20.0, 640.0, 0.0), (48.0, 12.5, 640.0, 32.0), (16.0, 20.0, 0.0, 0.0)):
        v.ekle("hopped_extract_ibu", {"amount_oz": a_, "ibu_gal_per_lb": g_, "volume": vol, "trub": tr},
               ref.hopped_extract_ibu(a_, g_, vol, tr))
    for (w, tot) in ((1.0, 4.0), (0.5, 0.5), (1.0, 0.0), (1.0, -2.0)):
        v.ekle("hop_percent", {"w": w, "tot_hops": tot}, ref.hop_percent(w, tot))
    hops_r = [hop(1.0, 12.0, 0, 0, 60), hop(1.5, 5.5, 1, 0, 15), hop(1.0, 6.0, 0, 3, 90), hop(2.0, 4.0, 2, 4, 15),
              hop(1.0, 9.0, 0, 1, 0), hop(0.5, 14.0, 0, 2, 60), hop(0.1, 50.0, 4, 0, 0)]
    for eq in (0, 1, 2):
        for (nm, rc) in (
            ("karma hoplar, geç ekstrakt, hopped extract", rec(ferm_hopped, hops_r, trub_loss=16.0)),
            ("carry + rakım", rec(ferms_a, hops_r, altitude=3000, **carry)),
            ("Extract tarifi", rec(ferm_late, hops_r, type=0, hop_util=85.0)),
            ("hop yok", rec(ferm_hopped, [])),
        ):
            v.ekle("recipe_ibu", {"recipe": rc, "equation": eq, "v_boil": 0.0},
                   ref.recipe_ibu(rc, eq, 0.0), nm)
    rc = rec(ferm_late, hops_r, opts=opts_x)
    v.ekle("recipe_ibu", {"recipe": rc, "equation": 0, "v_boil": 1000.0},
           ref.recipe_ibu(rc, 0, 1000.0), "özel seçenekler, v_boil")

    # §6.1 ScaleBitterness
    for eq in (0, 1, 2):
        for (nm, rc, tgt) in (
            ("hedef büyük", rec(ferm_hopped, hops_r, trub_loss=16.0), 60.0),
            ("hedef küçük", rec(ferms_a, hops_r), 15.0),
            ("hedef = mevcut", rec(ferms_a, hops_r), None),
            ("mevcut 0", rec(ferms_a, [hop(use=1)]), 30.0),
            ("hedef 0", rec(ferms_a, hops_r), 0.0),
            ("hopped extract hedefi aşıyor", rec(ferm_hopped, hops_r), 10.0),
            ("carry + Extract tarifi", rec(ferm_late, hops_r, type=0, hop_util=90.0, **carry), 45.0),
            ("V=0", rec(ferms_a, hops_r, volume=0.0), 40.0),
        ):
            if nm == "mevcut 0":
                cur = 0.0
            elif nm == "V=0":
                cur = 30.0
            else:
                cur = ref.recipe_ibu(rc, eq, 0.0)
            tg = cur if tgt is None else tgt
            v.ekle("scale_bitterness", {"current_ibu": cur, "target": tg, "recipe": rc, "equation": eq,
                                        "v_boil": 0.0},
                   ref.scale_bitterness(cur, tg, rc, eq, 0.0), nm)

    # T1.15 B2: form 4 dry hop dahil ölçekleme (§6.1 adım 3); hedef tutmalı
    hops_f4 = [hop(1.0, 10.0, 0, 0, 60), hop(0.05, 50.0, 4, 1, 0)]
    for eq in (0, 1, 2):
        for (nm, rc, tgt) in (
            ("form 4 dry hop: 60 hedefi", rec(ferms_a, hops_f4), 60.0),
            ("form 4 dry hop: hedef büyük", rec(ferm_hopped, hops_r + [hops_f4[1]], trub_loss=16.0), 90.0),
            ("form 4 dry hop + dry hop karışık", rec(ferms_a, hops_f4 + [hop(use=1)]), 40.0),
        ):
            cur = ref.recipe_ibu(rc, eq, 0.0)
            v.ekle("scale_bitterness", {"current_ibu": cur, "target": tgt, "recipe": rc, "equation": eq,
                                        "v_boil": 0.0},
                   ref.scale_bitterness(cur, tgt, rc, eq, 0.0), nm)

    # T1.15 B18: total_hops_phase doğrudan
    for phase in (0, 1, 2, 3, 4):
        v.ekle("total_hops_phase", {"hops": hops_r, "phase": phase}, ref.tot_hops_phase(hops_r, phase))

    # §6.2 HopIBUAdjust
    for (wo, io, inew) in ((1.0, 20.0, 30.0), (2.5, 12.5, 5.0), (1.0, 0.0, 10.0), (1.0, -3.0, 10.0), (1.0, 20.0, 0.0)):
        v.ekle("hop_ibu_adjust", {"w_old": wo, "ibu_old": io, "ibu_new": inew}, dene(ref.hop_ibu_adjust, wo, io, inew))

    # §7 araç yolu: CalcIBU, CalcAAU ve toplamlar
    for eq in (0, 1, 2):
        for h in hops_r + [hop(use=1, form=4, amount=0.1, alpha=50.0), hop(amount=0.0), hop(use=3, time=60),
                           hop(use=2), hop(use=4, time=30, wtemp=170.0)]:
            for (sg, vf, ut) in ((1.050, 640.0, 100.0), (1.075, 768.0, 90.0), (1.050, 0.0, 100.0)):
                v.ekle("tool_hop_ibu", {"hop": h, "sg": sg, "v_floz": vf, "equation": eq, "util_pct": ut,
                                        "options": opts},
                       ref.calc_ibu(h, sg, vf, eq, ut, opts))
        v.ekle("tool_bitterness", {"hops": hops_r, "sg": 1.055, "v_floz": 640.0, "equation": eq,
                                   "util_pct": 95.0, "options": opts},
               ref.tool_bitterness(hops_r, 1.055, 640.0, eq, 95.0, opts))
    v.ekle("tool_bitterness", {"hops": [], "sg": 1.05, "v_floz": 640.0, "equation": 0, "util_pct": 100.0,
                               "options": opts},
           ref.tool_bitterness([], 1.05, 640.0, 0, 100.0, opts), "boş liste")
    for use in range(5):
        for t in (0, 14.99, 15, 60):
            v.ekle("aau", {"w": 1.5, "a": 8.0, "use": use, "t": t}, ref.calc_aau(1.5, 8.0, use, t))

    # §8 hop oranları, §9 whirlpool süresi
    for phase in (0, 1, 2, 3, 4):
        for vol in (640.0, 0.0, 128.0):
            v.ekle("hop_rate", {"hops": hops_r, "volume_floz": vol, "phase": phase},
                   ref.hop_rate(hops_r, vol, phase))
    for rt in (0, 10, 25):
        for (hs, ms) in ((hops_r, []), ([hop(use=4, time=29.52)], [ref01.misc(6, 33.4)]),
                         ([hop(use=4, time=29.47)], []), ([], [ref01.misc(6, 40.0), ref01.misc(5, 99.0)]),
                         ([], [])):
            v.ekle("total_whirlpool_time", {"recipe_time": rt, "hops": hs, "misc": ms},
                   ref.tot_whirlpool_time(rt, hs, ms))

    # §10 ay ve depolama tipi ile
    for st in (0, 1, 2, 7):
        for (a, hsi, tf, mo) in ((5, 30, 68, 6), (7.5, 40, 40, 12), (9.0, 100, 77, 3), (5, 30, 68, 0)):
            v.ekle("aged_alpha_months", {"alpha": a, "hsi": hsi, "temp_f": tf, "months": mo, "storage_type": st},
                   dene(ref.aged_alpha_months, a, hsi, tf, mo, st), "depolama tipi 7: tanımsız -> hata (T1.16)" if st == 7 else None)
    return v.yaz()
