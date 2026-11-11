"""Bütün parite vektörlerini yeniden üretir (ADR-0003). CI, üretimden sonra `git diff --exit-code` ile
vektörlerin güncel olduğunu denetler. Çalıştır: python3 -I formuller/testler/parite/uret.py"""
import importlib.util
import pathlib
import sys

BURA = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(BURA))

toplam = 0
for betik in sorted(BURA.glob("[0-9][0-9]_*.py")):
    spec = importlib.util.spec_from_file_location(betik.stem, betik)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    yol, n = mod.uret()
    toplam += n
    print("%-28s %4d vaka -> %s" % (betik.name, n, yol.relative_to(BURA.parents[2])))
print("toplam", toplam, "vaka")
