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
        veri = {"bolum": self.bolum, "kaynak": self.kaynak, "vakalar": self.vakalar}
        yol.write_text(json.dumps(veri, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
        return yol, len(self.vakalar)
