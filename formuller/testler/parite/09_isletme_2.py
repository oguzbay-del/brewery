"""Bölüm 09 parite vektörleri: etiket enerjisi, vergi motoru (Decimal), spunding (ADR-0003).

Ondalık girdiler vektörde metin olarak ({"ondalik": "..."}) yazılır; referansa aynı Decimal değerleri verilir.
Hata vakalarının beklenen değeri {"hata": "<istisna sınıfı>"} olur."""
from decimal import Decimal as D, ROUND_HALF_UP, ROUND_HALF_EVEN, ROUND_HALF_DOWN, ROUND_UP, ROUND_DOWN, \
    ROUND_CEILING, ROUND_FLOOR

from ortak import Vektor, yukle

MODLAR = {"ROUND_HALF_UP": ROUND_HALF_UP, "ROUND_HALF_EVEN": ROUND_HALF_EVEN, "ROUND_HALF_DOWN": ROUND_HALF_DOWN,
          "ROUND_UP": ROUND_UP, "ROUND_DOWN": ROUND_DOWN, "ROUND_CEILING": ROUND_CEILING, "ROUND_FLOOR": ROUND_FLOOR}


def _hata_ile(fn):
    try:
        return fn()
    except (ValueError, ArithmeticError):  # ArithmeticError: Decimal InvalidOperation (28 basamak taşması)
        return {"hata": "ValueError"}


def _dec(x):
    return None if x is None else D(x)


def uret():
    ref = yukle("09_isletme_2_tv.py")
    v = Vektor("09", "formuller/testler/09_isletme_2_tv.py")

    # ---- 01'den gelen yardımcılar (09 referansındaki kopyalar)
    for sg in (1.0, 1.009, 1.030, 1.048, 1.052, 1.090):
        v.ekle("to_plato", {"sg": sg}, ref.to_plato(sg))
    for p in (0.0, 2.3, 12.0, 20.0):
        v.ekle("to_sg", {"plato": p}, ref.to_sg(p))
    for oe, ae in ((12.0, 2.3), (11.9, 3.0), (20.0, 5.0)):
        v.ekle("real_extract", {"oe": oe, "ae": ae}, ref.real_extract(oe, ae))
    for oe, re in ((12.0, 4.0), (11.9, 4.1), (20.0, 7.0)):
        v.ekle("abw", {"oe": oe, "re": re}, ref.abw(oe, re))
    for og, fg in ((1.048, 1.009), (1.052, 1.011), (1.030, 1.006), (1.090, 1.020), (1.040, 1.040)):
        v.ekle("abv_from", {"og": og, "fg": fg}, ref.abv_from(og, fg))

    # ---- 1 Etiket enerjisi
    for og, fg, prot, ab in ((1.048, 1.009, 0.40, None), (1.052, 1.011, 0.40, None), (1.048, 1.009, 0.40, 5.0),
                             (1.030, 1.006, 0.30, None), (1.090, 1.020, 0.55, None), (1.048, 1.009, 0.40, 4.7),
                             (1.040, 1.040, 0.40, None), (1.048, 1.009, 5.0, None),   # protein > RE: karbonhidrat 0'a kenetlenir
                             (1.010, 1.005, 0.40, 0.5), (1.060, 1.012, 0.45, 6.2)):
        v.ekle("label_energy", {"og": og, "fg": fg, "protein_g100ml": prot, "abv_measured": ab},
               ref.label_energy(og, fg, prot, ab))

    # ---- 2 Vergi motoru
    def tax(method, volume, **o):
        """o: abv, plato, rate (metin), bands/tiers [[üst|None, oran]] (metin), ytd, plato_trunc, discount_per_lpa,
        input_dp (yoksa tanımsız), input_mode (ad), inclusive. Girdi sözlüğü ve referans çağrısı aynı veriden kurulur."""
        girdi = {"method": method, "volume_l": volume}
        kw = {}
        for k in ("abv", "plato", "rate", "ytd", "discount_per_lpa"):
            if k in o:
                girdi[k] = o[k]
                kw[k] = D(o[k])
        for k in ("bands", "tiers"):
            if k in o:
                girdi[k] = [[b[0], b[1]] for b in o[k]]
                kw[k] = [(_dec(b[0]), D(b[1])) for b in o[k]]
        for k in ("plato_trunc", "inclusive", "input_dp"):
            if k in o:
                girdi[k] = kw[k] = o[k]
        if "input_mode" in o:
            girdi["input_mode"] = o["input_mode"]
            kw["input_mode"] = MODLAR[o["input_mode"]]
        beklenen = _hata_ile(lambda: ref.tax_line(method, D(volume), **kw))
        if isinstance(beklenen, D):
            assert "E" not in str(beklenen), "bilimsel gösterim: %s" % beklenen
        v.ekle("tax_line", girdi, beklenen)
        # T1.17 (§2.3): aynı girdiyle vergi + LPA + uygulanan oran
        v.ekle("tax_line_detail", girdi, _hata_ile(lambda: ref.tax_line_detail(method, D(volume), **kw)))

    B22 = [["3.5", "9.27"], ["8.5", "21.78"], [None, "29.54"]]
    B24 = [["10.0", "16.88"], ["14.0", "25.32"], [None, "33.76"]]
    V = "2000.0"
    HL_BBL = ref.HL_PER_BBL
    # T1.12 D1: oranlar sonlu ondalık (hL başına), bölmeyle türetilmez; 3.50 $/bbl ≈ 2.9826, 16 $/bbl ≈ 13.6347 $/hL
    TIERS = [[str(60000 * HL_BBL), "2.9826"], [None, "13.6347"]]
    TIERS_SIMPLE = [["100", "3.0"], [None, "16.0"]]
    TIERS3 = [["100", "3.0"], ["1000", "5.0"], [None, "7.5"]]

    # spesifikasyon vektörleri 2.1-2.13b
    tax("saf_alkol", V, abv="5.0", rate="21.78", input_dp=None)
    tax("saf_alkol", V, abv="5.0", bands=B22, discount_per_lpa="5.0", input_dp=None)
    tax("hl_plato", V, plato="12.6", rate="0.787", input_dp=None)
    tax("plato_bandi", V, plato="12.0", bands=B24, input_dp=None)
    tax("kademeli_hacim", V, tiers=TIERS)
    tax("kademeli_hacim", V, tiers=TIERS, ytd=str(60000 * HL_BBL - 10))
    tax("hacim", V, rate="1249.0")
    for x, dp in (("10.0", None), ("10.05", None), ("10.05", 1)):
        kw = {"input_dp": dp}
        if dp is not None:
            kw["input_mode"] = "ROUND_HALF_UP"
        tax("plato_bandi", V, plato=x, bands=B24, **kw)
    tax("saf_alkol", V, abv="3.5", bands=B22, input_dp=None)
    tax("saf_alkol", V, abv="5.0", rate="4.0", discount_per_lpa="5.0", input_dp=None)
    tax("kademeli_hacim", V, tiers=[["100.0", "3.0"]], ytd="95.0")                # hata
    tax("saf_alkol", V, abv="5.0", rate="21.78")                                  # hata: kural tanımsız
    tax("saf_alkol", V, abv="5.0", rate="21.78", input_dp=None)

    # T1.12 D8: bant/kademe tablosu doğrulaması (hepsi hata)
    tax("plato_bandi", V, plato="9", bands=[["14.0", "25.32"], ["10.0", "16.88"], [None, "33.76"]], input_dp=None)
    tax("plato_bandi", V, plato="9", bands=[["10.0", "16.88"], ["10.0", "25.32"], [None, "33.76"]], input_dp=None)
    tax("plato_bandi", V, plato="9", bands=[[None, "16.88"], ["14.0", "25.32"]], input_dp=None)
    tax("plato_bandi", V, plato="9", bands=[["10.0", "-1"], [None, "33.76"]], input_dp=None)
    tax("saf_alkol", V, abv="5.0", bands=[["8.5", "21.78"], ["3.5", "9.27"], [None, "29.54"]], input_dp=None)
    tax("saf_alkol", V, abv="5.0", bands=[["3.5", "9.27"], [None, "-29.54"]], input_dp=None)
    tax("kademeli_hacim", "20000", tiers=[["1000", "5.0"], ["100", "3.0"], [None, "7.5"]])
    tax("kademeli_hacim", V, tiers=[["100", "3.0"], [None, "5.0"], ["1000", "7.5"]])
    tax("kademeli_hacim", V, tiers=[["100", "-3.0"], [None, "5.0"]])
    tax("kademeli_hacim", V, tiers=[["100", "3.0"], ["100", "5.0"], [None, "7.5"]])
    # T1.12 D4/D5: sıfır işlenenli toplama/çıkarma ölçeği, eksi sıfır
    tax("saf_alkol", V, abv="5.0", rate="4", discount_per_lpa="0.00", input_dp=None)
    tax("kademeli_hacim", V, tiers=[["100.0", "3.0"], [None, "5.0"]], ytd="100.00")
    tax("kademeli_hacim", "0.0", tiers=TIERS_SIMPLE, ytd="0.00")
    tax("hl_plato", "0", plato="-12.6", rate="2", input_dp=None)
    tax("hl_plato", V, plato="0", rate="-2", input_dp=None)
    tax("plato_bandi", "0", plato="9", bands=[["10.0", "-0.0"], [None, "33.76"]], input_dp=None)
    # T1.12 D1: 28 basamaklı oran yerine sonlu ondalık çarpanlar (≤ 15 anlamlı basamak)
    tax("plato_bandi", "10.3", plato="9", bands=[["10.0", "0.613559668677778"], [None, "1"]], input_dp=None)
    tax("kademeli_hacim", "8744557284130", tiers=[[None, "2.9825876956412"]])
    tax("hacim", "1234.5", rate=str(ref.HL_PER_BBL))
    # hacim
    tax("hacim", "2000", rate="1249")
    tax("hacim", "1234.5", rate="12.49")
    tax("hacim", "0", rate="1249")
    tax("hacim", "0.0", rate="0.00")
    tax("hacim", "5", rate="-3.0")                                                # negatif oran: 0
    tax("hacim", "50.00", rate="1249.50")
    tax("hacim", "100", rate="0.00")
    tax("hacim", V, rate="12.5", ytd="7")                                         # ytd yok sayılır
    # saf_alkol: bantlar, kenarlar, indirim, yuvarlama
    for a in ("0.5", "3.4", "3.5", "3.6", "8.4", "8.5", "8.6", "10.0"):
        tax("saf_alkol", V, abv=a, bands=B22, input_dp=None)
    for a in ("3.5", "8.5"):
        tax("saf_alkol", V, abv=a, bands=B22, inclusive=True, input_dp=None)
        tax("saf_alkol", V, abv=a, bands=B22, inclusive=False, input_dp=None)
    tax("saf_alkol", V, abv="3.46", bands=B22, input_dp=1, input_mode="ROUND_HALF_UP")
    tax("saf_alkol", V, abv="3.46", bands=B22, input_dp=1, input_mode="ROUND_DOWN")
    tax("saf_alkol", V, abv="3.44", bands=B22, input_dp=1, input_mode="ROUND_UP")
    tax("saf_alkol", V, abv="3.45", bands=B22, input_dp=1, input_mode="ROUND_HALF_EVEN")
    tax("saf_alkol", V, abv="3.45", bands=B22, input_dp=1, input_mode="ROUND_HALF_DOWN")
    tax("saf_alkol", V, abv="3.41", bands=B22, input_dp=1, input_mode="ROUND_CEILING")
    tax("saf_alkol", V, abv="3.49", bands=B22, input_dp=1, input_mode="ROUND_FLOOR")
    tax("saf_alkol", V, abv="5.0", rate="21.78", input_dp=2, input_mode="ROUND_HALF_UP")
    tax("saf_alkol", V, abv="4.375", rate="8.58", discount_per_lpa="1.99", input_dp=2, input_mode="ROUND_HALF_UP")
    tax("saf_alkol", V, abv="4.5", rate="8.58", discount_per_lpa="8.58", input_dp=None)
    tax("saf_alkol", "1234.5", abv="4.2", rate="8.58", input_dp=None)
    tax("saf_alkol", "0", abv="4.2", rate="8.58", input_dp=None)
    tax("saf_alkol", V, abv="0.0", rate="8.58", input_dp=None)
    tax("saf_alkol", V, abv="5.0", bands=[], rate="8.58", input_dp=None)          # boş bant: oran kullanılır
    tax("saf_alkol", V, abv="9.0", bands=[["8.5", "21.78"]], input_dp=None)       # hata: bant yok
    # hl_plato
    for p in ("12.6", "12.0", "11.99", "0.4", "12.5"):
        tax("hl_plato", V, plato=p, rate="0.787", input_dp=None)
    tax("hl_plato", V, plato="12.6", rate="0.787", plato_trunc=False, input_dp=None)
    tax("hl_plato", V, plato="12.5", rate="0.787", input_dp=0, input_mode="ROUND_HALF_UP")
    tax("hl_plato", V, plato="12.5", rate="0.787", input_dp=0, input_mode="ROUND_HALF_EVEN")
    tax("hl_plato", V, plato="12.45", rate="2", input_dp=1, input_mode="ROUND_HALF_UP", plato_trunc=False)
    tax("hl_plato", V, plato="11.96", rate="2", input_dp=1, input_mode="ROUND_HALF_UP")
    tax("hl_plato", "1234.5", plato="12.6", rate="2", input_dp=None)
    tax("hl_plato", "0", plato="12.6", rate="2", input_dp=None)
    tax("hl_plato", V, plato="-1.5", rate="2", input_dp=None)                     # negatif °P: 0
    tax("hl_plato", V, plato="12.6", rate="0.787")                                # hata: kural tanımsız
    # plato_bandi
    for p in ("0", "9.9", "10.0", "10.01", "10.05", "10.1", "13.99", "14.0", "14.01", "20.0"):
        tax("plato_bandi", V, plato=p, bands=B24, input_dp=None)
    tax("plato_bandi", V, plato="10.0", bands=B24, inclusive=False, input_dp=None)
    tax("plato_bandi", V, plato="14.0", bands=B24, inclusive=False, input_dp=None)
    tax("plato_bandi", V, plato="9.99", bands=B24, inclusive=False, input_dp=None)
    tax("plato_bandi", V, plato="10.04", bands=B24, input_dp=1, input_mode="ROUND_DOWN")
    tax("plato_bandi", V, plato="10.04", bands=B24, input_dp=1, input_mode="ROUND_UP")
    tax("plato_bandi", V, plato="10.05", bands=B24, input_dp=1, input_mode="ROUND_HALF_EVEN")
    tax("plato_bandi", "1234.5", plato="12.0", bands=B24, input_dp=None)
    tax("plato_bandi", "0", plato="12.0", bands=B24, input_dp=None)
    tax("plato_bandi", V, plato="15.0", bands=[["14.0", "25.32"]], input_dp=None)  # hata: bant yok
    tax("plato_bandi", V, plato="12.0", bands=B24)                                # hata: kural tanımsız
    # kademeli_hacim
    tax("kademeli_hacim", V, tiers=TIERS_SIMPLE)
    tax("kademeli_hacim", V, tiers=TIERS_SIMPLE, ytd="90")
    tax("kademeli_hacim", V, tiers=TIERS_SIMPLE, ytd="100")
    tax("kademeli_hacim", V, tiers=TIERS_SIMPLE, ytd="150.5")
    tax("kademeli_hacim", V, tiers=TIERS_SIMPLE, ytd="99.9")
    tax("kademeli_hacim", V, tiers=TIERS_SIMPLE, ytd="-5")
    tax("kademeli_hacim", "0", tiers=TIERS_SIMPLE)
    tax("kademeli_hacim", "50000.0", tiers=TIERS3)
    tax("kademeli_hacim", "9500", tiers=TIERS3, ytd="50")
    tax("kademeli_hacim", V, tiers=[[None, "3.25"]])
    tax("kademeli_hacim", V, tiers=[[None, "3.25"]], ytd="12345")
    tax("kademeli_hacim", V, tiers=[["100", "3.0"]])                              # 20 hL sığar
    tax("kademeli_hacim", "20000", tiers=[["100", "3.0"]])                        # hata: aşım
    tax("kademeli_hacim", V, tiers=[])                                            # hata
    tax("bilinmeyen", V, rate="1.0")                                              # hata: bilinmeyen yöntem

    # ---- T2.8: yöntem zinciri (09 §2.2a). Bütün oranlar, eşikler ve tarihler SENTETİKTİR (gerçek oran değil).
    def adim(konum, yontem, bas="2026-01-01", bit=None, sinif="beer", **o):
        a = {"konum": konum, "sinif": sinif, "yontem": yontem, "baslangic": bas, "bitis": bit}
        a.update(o)
        return a

    def _ref_adimlar(adimlar):
        out = []
        for a in adimlar:
            b = dict(a)
            if "input_mode" in b:
                b["input_mode"] = MODLAR[b["input_mode"]]
            out.append(b)
        return out

    def zincir(adimlar, bas, bit, satirlar=(), ytd="0", sinif="beer", not_=None, sec_only=False):
        girdi = {"steps": adimlar, "sinif": sinif, "baslangic": bas, "bitis": bit}
        if sec_only:
            v.ekle("select_chain", girdi, _hata_ile(
                lambda: [a["konum"] for a in ref.select_chain(_ref_adimlar(adimlar), sinif, bas, bit)]), not_)
            return
        girdi["satirlar"] = [dict(x) for x in satirlar]
        girdi["ytd"] = ytd
        beklenen = _hata_ile(lambda: ref.tax_chain(_ref_adimlar(adimlar), sinif, bas, bit, satirlar, ytd))
        v.ekle("tax_chain", girdi, beklenen, not_)

    # Yürürlük: iki sürüm; [başlangıç, bitiş) ve dönem seçimi
    S1 = adim(1, "hacim", bas="2026-01-01", bit="2026-07-01", rate="1000.0")
    S2 = adim(1, "hacim", bas="2026-07-01", bit=None, rate="1200.0")
    for bas, bit, n in (("2026-03-01", "2026-03-31", "eski sürüm"), ("2026-06-01", "2026-06-30", "bitişten bir gün önce biter"),
                        ("2026-06-30", "2026-06-30", "bitiş günü hariç: son gün eski"),
                        ("2026-07-01", "2026-07-31", "yürürlük günü yeni sürüm"),
                        ("2026-07-01", "2026-07-01", "yürürlük günü"), ("2026-01-01", "2026-01-31", "ilk yürürlük günü"),
                        ("2026-06-15", "2026-07-15", "dönem içinde tablo değişiyor: hata"),
                        ("2026-06-30", "2026-07-01", "dönem tam sınırda bölünüyor: hata"),
                        ("2025-12-31", "2025-12-31", "yürürlükte tablo yok: hata"),
                        ("2025-12-15", "2026-01-15", "başlangıçta tablo yok: hata"),
                        ("2026-07-31", "2026-07-01", "başlangıç bitişten sonra: hata"),
                        ("2026-7-01", "2026-07-31", "tarih biçimi: hata")):
        zincir([S1, S2], bas, bit, sec_only=True, not_=n)
        zincir([S1, S2], bas, bit, [{"volume_l": "2000.0"}], not_=n)
    zincir([S1, S2], "2026-03-01", "2026-03-31", sinif="draught", sec_only=True, not_="başka sınıf: hata")
    zincir([S1, adim(2, "hacim", sinif="draught", rate="1.0")], "2026-03-01", "2026-03-31", [{"volume_l": "2000.0"}],
           not_="başka sınıfın adımı sayılmaz")
    zincir([S1, adim(1, "hacim", bas="2026-03-01", bit=None, rate="5.0")], "2026-03-10", "2026-03-20",
           [{"volume_l": "100"}], not_="aynı konumda çakışan yürürlük: hata")
    zincir([S1, adim(0, "hacim", rate="5.0")], "2026-03-10", "2026-03-20", [{"volume_l": "100"}], not_="konum 0: hata")
    zincir([adim(1, "hacim", bas="2026-05-01", bit="2026-05-01", rate="5.0")], "2026-03-10", "2026-03-20",
           [{"volume_l": "100"}], not_="boş yürürlük aralığı: hata")
    zincir([S1, adim(2, "hacim", bas="2026-01-01", bit="2025-12-01", rate="5.0")], "2026-03-10", "2026-03-20",
           [{"volume_l": "100"}], not_="ters yürürlük aralığı: hata")
    zincir([S2, S1], "2026-03-01", "2026-03-31", sec_only=True, not_="liste sırası önemsiz")
    zincir([adim(3, "hacim", rate="1.0"), adim(1, "hacim", rate="2.0"), adim(2, "hacim", rate="3.0")],
           "2026-03-01", "2026-03-31", sec_only=True, not_="konuma göre sıralı")

    # Bant + kademe + indirim zinciri (sentetik): 1 saf_alkol bant + indirim (koşullu), 2 kademeli hacim, 3 hacim
    ZB = adim(1, "saf_alkol", bands=B22, discount_per_lpa="5.0", input_dp=None,
              indirim_kosul={"abv_alti": "8.5", "ytd_hl_alti": "100"})
    ZT = adim(2, "kademeli_hacim", tiers=TIERS3)
    ZH = adim(3, "hacim", rate="1.5")
    ZINCIR = [ZH, ZB, ZT]                              # liste sırası karışık; sıra `konum`dur
    D1, D2 = "2026-04-01", "2026-04-30"
    L = lambda vl="2000.0", abv="5.0": {"volume_l": vl, "abv": abv}
    zincir(ZINCIR, D1, D2, [L()], ytd="0", not_="yıl başı: indirim var, ilk kademe")
    zincir(ZINCIR, D1, D2, [L()], ytd="90", not_="kademe geçişi bir partinin içinde (90 → 110 hL), indirim var")
    zincir(ZINCIR, D1, D2, [L()], ytd="99.9", not_="eşiğin hemen altı: indirim var")
    zincir(ZINCIR, D1, D2, [L()], ytd="100", not_="ytd = eşik: indirim yok (< sınır), ikinci kademe")
    zincir(ZINCIR, D1, D2, [L()], ytd="990", not_="ikinci → açık uçlu üçüncü kademe geçişi")
    zincir(ZINCIR, D1, D2, [L()], ytd="5000", not_="tamamen açık uçlu kademe")
    zincir(ZINCIR, D1, D2, [L(abv="8.5")], ytd="0", not_="ABV = indirim sınırı: indirim yok; dışlayıcı bant: üçüncü bant")
    zincir(ZINCIR, D1, D2, [L(abv="8.4")], ytd="0", not_="ABV sınırın altı: indirim var, ikinci bant")
    zincir(ZINCIR, D1, D2, [L(abv="3.5")], ytd="0", not_="ABV bant sınırı (dışlayıcı): ikinci bant")
    zincir(ZINCIR, D1, D2, [L(abv="3.4999")], ytd="0", not_="bant sınırının hemen altı: ilk bant")
    zincir(ZINCIR, D1, D2, [L(abv="3.5001")], ytd="0", not_="bant sınırının hemen üstü: ikinci bant")
    zincir(ZINCIR, D1, D2, [L("1000.0"), L("1000.0"), L("3000.0", "4.0")], ytd="85",
           not_="ytd satırlar arasında taşınır: 85 → 95 → 105 (ikinci satırda kademe geçişi), üçüncüde indirim biter")
    zincir(ZINCIR, D1, D2, [L("0.0"), L()], ytd="99.5", not_="sıfır hacimli satır ytd'yi değiştirmez")
    zincir(ZINCIR, D1, D2, [], ytd="10", not_="satır yok")
    zincir(ZINCIR, D1, D2, [L()], ytd="-1", not_="ytd negatif: hata")
    zincir(ZINCIR, D1, D2, [{"volume_l": "2000.0"}], not_="saf_alkol adımı ABV ister: hata")
    zincir([adim(1, "saf_alkol", bands=B22, discount_per_lpa="5.0")], D1, D2, [L()],
           not_="girdi yuvarlama kuralı tanımsız: hata")
    zincir([adim(1, "saf_alkol", bands=B22, discount_per_lpa="5.0", input_dp=1, input_mode="ROUND_DOWN",
                 indirim_kosul={"abv_alti": "8.5"})], D1, D2, [L(abv="8.59")],
           not_="koşul girdi yuvarlamasından sonra: 8.59 → 8.5 (aşağı), 8.5 < 8.5 değil: indirim yok")
    zincir([adim(1, "saf_alkol", bands=B22, discount_per_lpa="5.0", input_dp=1, input_mode="ROUND_DOWN",
                 indirim_kosul={"abv_alti": "8.6"})], D1, D2, [L(abv="8.59")],
           not_="8.59 → 8.5 < 8.6: indirim var")
    zincir([adim(1, "saf_alkol", rate="4.0", discount_per_lpa="5.0", input_dp=None)], D1, D2, [L()],
           not_="indirim oranı aşar: vergi 0, indirim uygulandı")
    zincir([adim(1, "saf_alkol", rate="21.78", discount_per_lpa="5.0", input_dp=None)], D1, D2, [L()],
           not_="koşulsuz indirim her zaman uygulanır")
    zincir([adim(1, "saf_alkol", rate="21.78", discount_per_lpa="0", input_dp=None,
                 indirim_kosul={"ytd_hl_alti": "100"})], D1, D2, [L()], ytd="500", not_="indirim sıfır: koşul önemsiz")
    zincir([adim(1, "hacim", rate="10.0", discount_per_lpa="1.0")], D1, D2, [L()],
           not_="indirim yalnız saf_alkol: hata")
    zincir([adim(1, "saf_alkol", rate="21.78", discount_per_lpa="-1", input_dp=None)], D1, D2, [L()],
           not_="negatif indirim: hata")
    zincir([adim(1, "saf_alkol", rate="21.78", discount_per_lpa="5.0", input_dp=None,
                 indirim_kosul={"abv_alti": "8.5"})], D1, D2, [{"volume_l": "2000.0"}],
           not_="ABV koşulu ABV girdisi ister: hata")
    zincir([adim(1, "kademeli_hacim", tiers=[["100", "3.0"]])], D1, D2, [L()], ytd="95",
           not_="kapalı son kademe aşıldı: hata")
    zincir([adim(1, "kademeli_hacim", tiers=[["100", "3.0"]])], D1, D2, [L("1000.0")], ytd="95",
           not_="kademe tam sığar (95 + 10 hL ≤ 100 değil: 5 hL kademesiz)")
    zincir([adim(1, "kademeli_hacim", tiers=[["100", "3.0"]])], D1, D2, [L("500.0")], ytd="95",
           not_="kademe tam dolar (95 + 5 hL = 100)")
    zincir([adim(1, "kademeli_hacim", tiers=[["100", "3.0"]])], D1, D2, [L("500.0"), L("100.0")], ytd="95",
           not_="ikinci satır dolu kademeyi aşar: hata")
    zincir([adim(1, "kademeli_hacim", tiers=[[None, "3.25"]])], D1, D2, [L()], ytd="12345", not_="tek açık uçlu kademe")
    zincir([adim(1, "kademeli_hacim", tiers=[["100", "3.0"], ["50", "5.0"], [None, "7.5"]])], D1, D2, [L()],
           not_="kademe tablosu artan değil: hata")
    # Tek adımlı zincirler: yöntem başına bant sınırı tam üstü/altı
    PB = adim(1, "plato_bandi", bands=B24, input_dp=None)
    for pl in ("9.99", "10.0", "10.01", "14.0", "14.01"):
        zincir([PB], D1, D2, [{"volume_l": "2000.0", "plato": pl}], not_="plato bandı, kapsayıcı sınır")
    zincir([adim(1, "plato_bandi", bands=B24, inclusive=False, input_dp=None)], D1, D2,
           [{"volume_l": "2000.0", "plato": "10.0"}], not_="plato bandı, dışlayıcı sınır")
    zincir([adim(1, "hl_plato", rate="0.787", input_dp=None),
            adim(2, "kademeli_hacim", tiers=TIERS_SIMPLE)], D1, D2,
           [{"volume_l": "2000.0", "plato": "12.6"}], ytd="90", not_="hl_plato + kademe, plato kesilir")
    zincir([adim(1, "hl_plato", rate="0.787", input_dp=None, plato_trunc=False)], D1, D2,
           [{"volume_l": "2000.0", "plato": "12.6"}], not_="plato kesilmez")
    zincir([adim(1, "hl_plato", rate="0.787", input_dp=None)], D1, D2, [{"volume_l": "2000.0"}],
           not_="plato yok: hata")
    zincir([adim(1, "hacim", rate="0.1"), adim(2, "hacim", rate="0.2")], D1, D2,
           [{"volume_l": "3.0"}], not_="ölçek: bileşen toplamı")
    zincir([adim(1, "bilinmeyen", rate="1.0")], D1, D2, [L()], not_="bilinmeyen yöntem: hata")

    # ---- _round_in / _band / money / row_status
    for x, dp, mode in (("2.675", 2, "ROUND_HALF_UP"), ("2.675", 2, "ROUND_HALF_EVEN"), ("2.665", 2, "ROUND_HALF_EVEN"),
                        ("2.675", 2, "ROUND_DOWN"), ("2.671", 2, "ROUND_UP"), ("2.675", 2, "ROUND_HALF_DOWN"),
                        ("-2.675", 2, "ROUND_CEILING"), ("-2.675", 2, "ROUND_FLOOR"), ("2.5", 0, "ROUND_HALF_UP"),
                        ("2", 3, "ROUND_HALF_UP"), ("12.6", 1, "ROUND_HALF_UP"), ("10.049", 1, "ROUND_HALF_UP"),
                        ("0.004", 2, "ROUND_HALF_UP")):
        v.ekle("round_in", {"x": x, "dp": dp, "mode": mode}, ref._round_in(D(x), dp, MODLAR[mode]))
    for x in ("2.675", "12", "0.10"):
        v.ekle("round_in", {"x": x, "dp": None, "mode": "ROUND_HALF_UP"}, ref._round_in(D(x), None, ROUND_HALF_UP))

    bands_d = [(_dec(a), D(b)) for a, b in B22]
    for x in ("0", "3.4", "3.5", "3.6", "8.5", "9.0"):
        for inc in (False, True):
            v.ekle("band", {"value": x, "bands": B22, "inclusive": inc},
                   _hata_ile(lambda: ref._band(D(x), bands_d, inc)))
    v.ekle("band", {"value": "5", "bands": [["3", "1.0"]], "inclusive": True},
           _hata_ile(lambda: ref._band(D("5"), [(D("3"), D("1.0"))], True)))
    v.ekle("band", {"value": "5", "bands": [], "inclusive": False}, _hata_ile(lambda: ref._band(D("5"), [], False)))

    for x, dp, mode in (("188.875", 2, "ROUND_HALF_UP"), ("188.875", 2, "ROUND_HALF_EVEN"), ("188.875", 2, "ROUND_DOWN"),
                        ("0.005", 2, "ROUND_HALF_UP"), ("1678", 2, "ROUND_HALF_UP"), ("1678.456", 0, "ROUND_HALF_UP"),
                        ("59.649999", 2, "ROUND_UP"), ("-0.005", 2, "ROUND_HALF_UP"), ("166.1666667", 4, "ROUND_FLOOR"),
                        ("24980.00", 2, "ROUND_HALF_UP"), ("0", 2, "ROUND_HALF_UP"), ("2.5", 0, "ROUND_HALF_EVEN")):
        v.ekle("money", {"x": x, "dp": dp, "mode": mode}, ref.money(D(x), dp, MODLAR[mode]))

    for a, b in ((None, None), ("gov.uk SPR kılavuzu", None), (None, "2026-10-31"), ("gov.uk SPR kılavuzu", "2026-10-31"),
                 ("", "2026-10-31"), ("HMRC", ""), ("", ""), ("TTB", "2026-01-15")):
        v.ekle("row_status", {"resmi_kaynak": a, "tarih": b}, ref.row_status(a, b))

    # ---- 3 Spunding
    for t in (-3.0, 0.0, 4.0, 12.0, 18.0, 20.0, 25.0):
        v.ekle("residual_co2", {"t_c": t}, ref.residual_co2(t))
    for t, vol in ((12.0, 2.5), (18.0, 2.6), (0.0, 2.4), (-3.0, 2.0), (20.0, 3.0), (4.0, 0.0), (0.0, 0.5), (25.0, 1.0)):
        v.ekle("co2_pressure_psi", {"t_c": t, "vol": vol}, ref.co2_pressure_psi(t, vol))
    # T1.12 D2: FG <= 0 hata (NaN/sonsuz JSON'a yazılamaz; Rust birim testinde)
    for tv, t, fg in ((2.5, 12.0, 0.0), (2.5, 12.0, -1.0)):
        v.ekle("spund", {"target_vol": tv, "t_c": t, "fg_expected": fg}, _hata_ile(lambda: ref.spund(tv, t, fg)))
    for tv, t, fg in ((2.5, 12.0, 1.010), (2.6, 18.0, 1.012), (2.2, 20.0, 1.008), (2.8, 10.0, 1.014),
                      (3.0, 4.0, 1.016), (1.0, 25.0, 1.010), (0.8, 25.0, 1.010),   # son ikisi: hedef kalıntının altında
                      (2.5, 15.0, 1.004), (2.4, 0.0, 1.012)):
        v.ekle("spund", {"target_vol": tv, "t_c": t, "fg_expected": fg}, ref.spund(tv, t, fg))

    # ---- T1.14g: beyan, vergilenecek hacim (§2.1), spunding uyarıları (§3.4)
    for abv in ("0.5", "1.2", "1.20", "1.21", "1.3", "5.0", "5.14", "0", "1.1999", "12"):
        v.ekle("label_declaration", {"abv": abv}, ref.label_declaration(abv), "%1.2 sınırı Decimal; 1.2 dahil değil")

    ev = [["2026-03-31", "ambalaj", "1500"], ["2026-04-01", "ambalaj", "2000.5"], ["2026-04-10", "cikis", "800"],
          ["2026-04-15", "satis", "700.25"], ["2026-04-20", "imha", "50"], ["2026-04-22", "iade", "30.5"],
          ["2026-04-25", "lab", "5"], ["2026-04-30", "ambalaj", "1000"], ["2026-05-01", "ambalaj", "999"],
          ["2026-04-28", "cikis", "100"]]
    for trig, dd in (("ambalaj", ()), ("ambalaj", ("imha",)), ("ambalaj", ("imha", "iade", "lab")),
                     ("cikis", ()), ("cikis", ("iade",)), ("satis", ("imha", "iade", "lab")),
                     ("satis", ("lab",))):
        girdi = {"events": ev, "trigger": trig, "start": "2026-04-01", "end": "2026-04-30", "deduct": list(dd)}
        r = ref.taxable_volume([tuple(e) for e in ev], trig, "2026-04-01", "2026-04-30", dd)
        v.ekle("taxable_volume", girdi, r, "dönem sınırları dahil; diğer tetikleyiciler sayılmaz")
    küçük = [["2026-04-02", "ambalaj", "10"], ["2026-04-03", "imha", "25"]]
    r = ref.taxable_volume([tuple(e) for e in küçük], "ambalaj", "2026-04-01", "2026-04-30", ("imha",))
    v.ekle("taxable_volume", {"events": küçük, "trigger": "ambalaj", "start": "2026-04-01", "end": "2026-04-30",
                              "deduct": ["imha"]}, r, "düşüş vergilenenden büyük: sıfıra kenetlenir")
    v.ekle("taxable_volume", {"events": [], "trigger": "ambalaj", "start": "2026-04-01", "end": "2026-04-30",
                              "deduct": []},
           ref.taxable_volume([], "ambalaj", "2026-04-01", "2026-04-30"), "boş")
    for evs, trig, dd, not_ in (([["2026-04-02", "ambalaj", "10"]], "uretim", [], "bilinmeyen tetikleyici"),
                                ([["2026-04-02", "ambalaj", "10"]], "ambalaj", ["satis"], "düşülemeyen tür"),
                                ([["2026-04-02", "fire", "10"]], "ambalaj", [], "bilinmeyen olay türü")):
        v.ekle("taxable_volume", {"events": evs, "trigger": trig, "start": "2026-04-01", "end": "2026-04-30",
                                  "deduct": dd},
               _hata_ile(lambda: ref.taxable_volume([tuple(e) for e in evs], trig, "2026-04-01", "2026-04-30", dd)), not_)

    for tv, t, mx, rng, wu in ((2.5, 12.0, 25.0, (1.0100, 1.0110), 200), (2.5, 12.0, 15.0, (1.0100, 1.0110), 200),
                               (2.5, 12.0, 25.0, (1.0100, 1.0140), 200), (2.5, 12.0, None, None, 200),
                               (2.5, 12.0, 25.0, (1.010, 1.012), 200), (2.5, 12.0, 25.0, (1.010, 1.01201), 200),
                               (3.0, 4.0, 10.0, (1.008, 1.016), 200), (2.6, 18.0, 27.1, None, 200),
                               (2.6, 18.0, 27.0, (1.012, 1.013), 50), (0.0, 12.0, 0.0, None, 200),
                               (2.5, 12.0, None, (1.014, 1.010), 200),
                               # T1.12 D13: sınır eşitliği ve ~1e-12 üstü (tolerans içinde: engellenmez)
                               (2.6, 18.0, 27.076251107200008, None, 200), (2.6, 18.0, 27.076251107200008 - 1e-12, None, 200),
                               (2.6, 18.0, 27.076251107200008 - 1e-8, None, 200)):
        v.ekle("spund_warnings", {"target_vol": tv, "t_c": t, "tank_max_psi": mx,
                                  "fg_range": None if rng is None else list(rng), "wide_u": wu},
               ref.spund_warnings(tv, t, mx, rng, wu), "kod listesi + engellendi")
    n = "T1.16: hata"
    for x, dp_ in (("1.5", 13), ("1.5", 30), ("95533426188478241544266.310", 7)):
        v.ekle("round_in", {"x": x, "dp": dp_, "mode": "ROUND_HALF_UP"},
               _hata_ile(lambda x=x, dp_=dp_: ref._round_in(D(x), dp_, ROUND_HALF_UP)), "hane sınırı ya da 28 basamak: " + n)
        v.ekle("money", {"x": x, "dp": dp_, "mode": "ROUND_HALF_UP"},
               _hata_ile(lambda x=x, dp_=dp_: ref.money(D(x), dp_, ROUND_HALF_UP)), "hane sınırı ya da 28 basamak: " + n)
    return v.yaz()
