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
