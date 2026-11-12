"""Parite vektörü üretimi için ortak yardımcılar (ADR-0003).

Her bölüm betiği (`NN_*.py`) referans modülü `yukle()` ile yükler, `Vektor` ile vakaları toplar ve
`formuller/testler/vektorler/NN.json` dosyasını yazar. Rust motoru (`sunucu/crates/engine/tests/parite_NN.rs`)
aynı dosyayı okur ve her vakayı karşılaştırır.

Çalıştır (depo kökünden): python3 -I formuller/testler/parite/uret.py
"""
import contextlib
import importlib.util
import io
import json
import math
import os
import pathlib

KOK = pathlib.Path(__file__).resolve().parents[1]          # formuller/testler
VEKTOR_DIZINI = KOK / "vektorler"


def yukle(dosya_adi):
    """Referans betiğini modül olarak yükler; üst düzeydeki print çıktısı bastırılır."""
    yol = KOK / dosya_adi
    spec = importlib.util.spec_from_file_location("ref_" + yol.stem, yol)
    mod = importlib.util.module_from_spec(spec)
    with contextlib.redirect_stdout(io.StringIO()):
        spec.loader.exec_module(mod)
    return mod


def _temizle(x):
    """JSON'a yazılabilir hale getirir: tuple → liste, Decimal → metin, NaN/sonsuz reddedilir."""
    if isinstance(x, bool) or x is None or isinstance(x, (int, str)):
        return x
    if isinstance(x, float):
        if not math.isfinite(x):
            raise ValueError("sonlu olmayan sayı vektöre yazılamaz: %r" % x)
        return x
    if isinstance(x, (list, tuple)):
        return [_temizle(v) for v in x]
    if isinstance(x, dict):
        return {str(k): _temizle(v) for k, v in x.items()}
    if type(x).__name__ == "Decimal":
        return {"ondalik": str(x)}
    raise TypeError("desteklenmeyen tip: %s" % type(x).__name__)


def dene(f, *args, **kw):
    """Referans çağrısı; istisna atarsa vektöre `{"hata": "ValueError"}` yazılır (ADR-0004 madde 3).

    Motor tek hata tipi (`engine::Error`) döndürür; Python istisna sınıfı (ZeroDivisionError, KeyError, ...)
    vektörde `ValueError` olarak normalleştirilir, Rust testi yalnız `hata` anahtarının varlığını denetler."""
    try:
        return f(*args, **kw)
    except (ValueError, ZeroDivisionError, OverflowError, KeyError, ArithmeticError):
        return {"hata": "ValueError"}


class Vektor:
    def __init__(self, bolum, kaynak):
        self.bolum = bolum
        self.kaynak = kaynak
        self.vakalar = []

    def ekle(self, fonk, girdi, beklenen, not_=None):
        """fonk: Rust tarafındaki eşleme anahtarı (snake_case). girdi: ad → değer sözlüğü."""
        vaka = {"fonk": fonk, "girdi": _temizle(girdi), "beklenen": _temizle(beklenen)}
        if not_:
            vaka["not"] = not_
        self.vakalar.append(vaka)

    def yaz(self):
        VEKTOR_DIZINI.mkdir(exist_ok=True)
        yol = VEKTOR_DIZINI / ("%s.json" % self.bolum)
        # Satır başına bir vaka: dosya küçük kalır, fark okunur.
        satirlar = [json.dumps(v, ensure_ascii=False, separators=(",", ":")) for v in self.vakalar]
        bas = '{"bolum":%s,"kaynak":%s,"vakalar":[\n' % (json.dumps(self.bolum), json.dumps(self.kaynak, ensure_ascii=False))
        # Atomik yazım: eşzamanlı okuyan test yarım dosya görmesin.
        gecici = yol.with_suffix(".json.tmp")
        gecici.write_text(bas + ",\n".join(satirlar) + "\n]}\n", encoding="utf-8")
        os.replace(gecici, yol)
        return yol, len(self.vakalar)


def fikstur(ref01):
    """01 ve 02 üreticilerinin paylaştığı reçete fikstürleri (ADR-0004 madde 1 ve 4, T1.16 B).

    Alan adları Rust `recipe::RecipeInput` ve alt yapılarıyla aynıdır; kurucular 01 referansındadır
    (`ferm`, `hop`, `make_recipe`). Aynı fermentable listeleri hem 01'in geç ekstrakt profilini hem
    02'nin tarif düzeyindeki IBU hesabını sınar; böylece iki bölümün aynı reçetede tutarlılığı sınanır."""
    f = ref01.ferm
    pale = f("pale", 160, 80, 2, 0, 0)
    ferms_a = [pale, f("munich", 32, 75, 9, 0, 0)]
    ferm_late = [pale, f("dme", 16, 95, 4, 5, 2, late_extract_min=15), f("lme", 24, 80, 8, 1, 2, late_extract_min=5),
                 f("honey", 8, 75, 2, 2, 2, late_extract_min=90),
                 f("dme2", 8, 95, 4, 4, 2, late_extract_min=15.05), f("amber", 8, 95, 4, 6, 2, late_extract_min=15)]
    return type("Fikstur", (), {
        "ferms_a": ferms_a,
        "ferm_late": ferm_late,
        # use = 3 (Whirlpool) fermentable
        "ferm_wp": ferms_a + [f("whirl sugar", 8, 100, 0, 2, 3)],
        # tür 3 ve süresi 0 olan geç ekstrakt sayılmaz
        "ferm_misc": ferms_a + [f("grain-type", 8, 80, 3, 3, 2, late_extract_min=20),
                                f("late-zero", 8, 95, 4, 5, 2, late_extract_min=0)],
        # T1.16 B: kullanım yeri Whirlpool olan geç ekstrakt zaman 0'da sayılır (önceden 02'de t = geç süre)
        "ferm_late3": ferms_a + [f("late whirl dme", 16, 95, 4, 5, 3, late_extract_min=20)],
        # T1.16 B: kazan dışı (Primary/Bottling) geç ekstrakt profile girmez
        "ferm_late_dis": ferms_a + [f("primary dme", 16, 95, 4, 4, 4, late_extract_min=10),
                                    f("bottling sugar", 8, 100, 0, 2, 7, late_extract_min=30)],
        # son girdiye 0.1 dk'dan yakın geç ekstrakt: son girdi t = 0 kalır
        "ferm_late_son": ferms_a + [f("dme son", 16, 95, 4, 4, 2, late_extract_min=0.05)],
        "ferm_hopped": ferm_late + [f("hopped LME", 48, 78, 8, 1, 2, ibu_gal_per_lb=30.0),
                                    f("plain LME", 48, 78, 8, 1, 2, ibu_gal_per_lb=0.0)],
    })()
