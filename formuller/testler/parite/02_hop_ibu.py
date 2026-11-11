"""Bölüm 02 (hop ve IBU) parite vektörleri: formuller/02-hop-ibu.md, referans formuller/testler/02_hop_ibu.py."""
from ortak import Vektor, yukle

SGLER = (1.000, 1.030, 1.040, 1.050, 1.051, 1.070, 1.100)


def uret():
    ref = yukle("02_hop_ibu.py")
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
               ref.garetz_ibu(1.0, 5.0, t, 1.050, 5.0, 5.0))
    for sg in (1.030, 1.040, 1.050, 1.070):
        v.ekle("garetz_ibu", {"w": 2.0, "a": 12.0, "t": 60, "sg": sg, "v": 5.0, "bv": 6.5},
               ref.garetz_ibu(2.0, 12.0, 60, sg, 5.0, 6.5), "v != bv: konsantrasyon faktörü")
    v.ekle("garetz_ibu", {"w": 4.0, "a": 8.0, "t": 30, "sg": 1.060, "v": 10.0, "bv": 12.0},
           ref.garetz_ibu(4.0, 8.0, 30, 1.060, 10.0, 12.0))
    # ön koşullar: t<=0, w<=0, a<0 -> 0; a=0
    for (w, a, t) in ((1.0, 5.0, 0), (1.0, 5.0, -5), (0.0, 5.0, 60), (-1.0, 5.0, 60),
                      (1.0, -1.0, 60), (1.0, 0.0, 60)):
        v.ekle("garetz_ibu", {"w": w, "a": a, "t": t, "sg": 1.050, "v": 5.0, "bv": 5.0},
               ref.garetz_ibu(w, a, t, 1.050, 5.0, 5.0), "ön koşul kenarı")
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
    return v.yaz()
