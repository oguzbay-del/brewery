"""Örnek bölüm betiği: altyapının kendi testi. Gerçek bölümler bu kalıbı izler."""
from ortak import Vektor, yukle


def uret():
    ref = yukle("01_yogunluk_tv.py")
    v = Vektor("00", "formuller/testler/01_yogunluk_tv.py")
    for sg in (1.000, 1.040, 1.050, 1.100):
        v.ekle("to_plato", {"sg": sg}, ref.ToPlato(sg))
    for p in (0.0, 10.0, 12.0, 20.0):
        v.ekle("to_sg", {"plato": p}, ref.ToSG(p))
    for sg in (1.0, 1.0500049999, 1.012345, 0.999995):
        v.ekle("gravity_units", {"sg": sg}, round(sg * 100000), "Python round: yarımda çifte yuvarlar")
    return v.yaz()
