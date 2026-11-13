#!/usr/bin/env python3
"""Katalog denetimi (T2.17): şema, dönüşümler, üretilen TypeScript kodunun davranışı.

Kullanım: python3 -I sozlesme/katalog/test.py
TypeScript denetimi için sozlesme/istemci/typescript/node_modules (npm ci) ve node gerekir;
yoksa o adım atlanır ve söylenir.
"""
import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

KLASOR = Path(__file__).resolve().parent
ROOT = KLASOR.parents[1]
TS_DIR = ROOT / "sozlesme/istemci/typescript"

spec = importlib.util.spec_from_file_location("uret", KLASOR / "uret.py")
uret = importlib.util.module_from_spec(spec)
spec.loader.exec_module(uret)


def to_si(k, unit, v):
    u = k["units"][unit]
    if u.get("nonlinear") == "plato":
        return 1 + v / (258.6 - (v / 258.6) * 227.1)
    if u.get("nonlinear") == "brix":
        return 1 + v / (258.6 - (v / 258.2) * 227.1)
    return (v + u["pre"]) * u["mul"] / u["div"] + u["post"]


def near(a, b, tol=1e-9):
    return abs(a - b) <= tol * max(1.0, abs(a), abs(b))


# (birim, değer, SI birim, beklenen SI) — tanımlardan bağımsız bilinen değerler
BILINEN = [
    ("gal_us", 1, "l", 3.785411784),
    ("gal_uk", 1, "l", 4.54609),
    ("qt_us", 4, "l", 3.785411784),
    ("bbl_us", 1, "l", 31 * 3.785411784),
    ("bbl_uk", 1, "l", 36 * 4.54609),
    ("fl_oz_us", 128, "l", 3.785411784),
    ("lb", 1, "kg", 0.45359237),
    ("oz", 16, "kg", 0.45359237),
    ("g", 1000, "kg", 1.0),
    ("f", 212, "c", 100.0),
    ("f", 32, "c", 0.0),
    ("f", -40, "c", -40.0),
    ("sg_x1000", 1050, "sg", 1.05),
    ("ebc", 19.7, "srm", 10.0),
    ("lovibond", 10, "srm", 12.804),
    ("psi", 14.5037738, "bar", 1.0),
    ("kpa", 100, "bar", 1.0),
    ("h", 1, "min", 60.0),
    ("ft", 1000, "m", 304.8),
    ("g_l_co2", 1.9635, "vol", 1.0),
    ("dh", 1, "caco3_mg_l", 17.848),
    ("qt_lb", 1, "l_kg", 0.946352946 / 0.45359237),
    ("oz_gal_us", 1, "g_l", 28.349523125 / 3.785411784),
]


def main():
    k = uret.yukle()
    hatalar = []
    try:
        import jsonschema  # isteğe bağlı çapraz denetim

        sema = json.loads(uret.SEMA.read_text(encoding="utf-8"))
        jsonschema.Draft202012Validator(sema).validate(k)
        print("ok: jsonschema ile de geçerli")
    except ImportError:
        print("not: jsonschema yok, yalnız yerleşik doğrulayıcı kullanıldı")

    for birim, deger, si_birim, beklenen in BILINEN:
        if k["units"][birim]["quantity"] != k["units"][si_birim]["quantity"]:
            hatalar.append(f"{birim}: büyüklük {si_birim} ile aynı değil")
        sonuc = to_si(k, birim, deger)
        if not near(sonuc, beklenen, 1e-7):
            hatalar.append(f"{birim} {deger} -> {sonuc}, beklenen {beklenen}")

    # gidiş dönüş: her doğrusal birim için ters formül
    for ad, u in k["units"].items():
        if "nonlinear" in u:
            continue
        for v in (0.0, 1.0, 37.5, -12.0):
            si = (v + u["pre"]) * u["mul"] / u["div"] + u["post"]
            geri = (si - u["post"]) * u["div"] / u["mul"] - u["pre"]
            if not near(v, geri):
                hatalar.append(f"{ad}: gidiş dönüş {v} -> {geri}")

    # alan sınırları ve mod sınırları
    for ad, d in k["fields"].items():
        if k["texts"]["tr"]["fields"][ad]["label"] == "" or k["texts"]["en"]["fields"][ad]["label"] == "":
            hatalar.append(f"{ad}: boş etiket")

    # TypeScript davranışı
    node = shutil.which("node")
    tsc = TS_DIR / "node_modules/.bin/tsc"
    if node and tsc.exists():
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            r = subprocess.run(
                [str(tsc), "--target", "ES2022", "--module", "ES2022", "--strict", "--outDir", str(tmp),
                 str(TS_DIR / "src/katalog.ts")], capture_output=True, text=True)
            if r.returncode != 0:
                hatalar.append("tsc: " + r.stdout + r.stderr)
            else:
                (tmp / "package.json").write_text('{"type":"module"}')
                vakalar = [[b, d, sb] for b, d, sb, _ in BILINEN]
                vakalar += [["plato", 12.0, "sg"], ["brix", 12.0, "sg"]]
                betik = (
                    "import * as K from './katalog.js';\n"
                    f"const vakalar = {json.dumps(vakalar)};\n"
                    "const out = vakalar.map(([b, d, s]) => [K.toSi(b, d), K.fromSi(b, K.toSi(b, d)), K.convert(b, s, d)]);\n"
                    "out.push([K.validateField('hop.alpha_pct', 12), 0, 0]);\n"
                    "out.push([K.validateField('hop.alpha_pct', 41), 0, 0]);\n"
                    "out.push([K.validateField('hop.amount_kg', Number.NaN), 0, 0]);\n"
                    "out.push([K.validateField('hop.amount_kg', 6, 'home'), 0, 0]);\n"
                    "out.push([K.validateField('hop.amount_kg', 6, 'brewery'), 0, 0]);\n"
                    "out.push([K.errorMessage('tr', 'hop.alpha_pct', 'out_of_range'), K.errorMessage('en', 'recipe.batch_volume_l', 'out_of_range', {mode:'home', displayUnit:'gal_us'}), 0]);\n"
                    "console.log(JSON.stringify(out));\n"
                )
                (tmp / "calis.mjs").write_text(betik, encoding="utf-8")
                r = subprocess.run([node, str(tmp / "calis.mjs")], capture_output=True, text=True)
                if r.returncode != 0:
                    hatalar.append("node: " + r.stderr)
                else:
                    cikti = json.loads(r.stdout)
                    for (b, d, s), (si, geri, donusum) in zip(vakalar, cikti):
                        if not near(si, to_si(k, b, d), 1e-12):
                            hatalar.append(f"TS toSi({b}, {d}) = {si}")
                        # Plato ve Brix çok terimlilerinin tersi tam değildir (yaklaşık 1e-4 göreli)
                        if not near(geri, d, 5e-4 if "nonlinear" in k["units"][b] else 1e-9):
                            hatalar.append(f"TS fromSi gidiş dönüş {b}: {geri} != {d}")
                        if not near(donusum, si, 1e-12):
                            hatalar.append(f"TS convert({b}, {s}) = {donusum}")
                    n = len(vakalar)
                    beklenen = [None, "out_of_range", "invalid_number", "out_of_range", None]
                    for i, b in enumerate(beklenen):
                        if cikti[n + i][0] != b:
                            hatalar.append(f"TS validateField #{i}: {cikti[n + i][0]} != {b}")
                    tr_msg, en_msg = cikti[n + 5][0], cikti[n + 5][1]
                    if tr_msg != "Alfa asit 0 ile 40 % arasında olmalı.":
                        hatalar.append(f"TS hata metni (tr): {tr_msg}")
                    if "gal (US)" not in en_msg or "0.132086" not in en_msg:
                        hatalar.append(f"TS hata metni (en): {en_msg}")
                    print("ok: TypeScript davranışı denetlendi")
    else:
        print("not: node veya tsc yok, TypeScript davranış denetimi atlandı")

    if hatalar:
        sys.exit("hata:\n  " + "\n  ".join(hatalar))
    print("ok: katalog denetimi geçti")


if __name__ == "__main__":
    main()
