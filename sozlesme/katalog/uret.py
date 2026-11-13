#!/usr/bin/env python3
"""Alan, birim ve metin kataloğundan istemci kodunu üretir (T2.17, ADR-0008 madde 7).

Kullanım: python3 -I sozlesme/katalog/uret.py

Girdi:  sozlesme/katalog/katalog.json (şema: katalog.schema.json)
Çıktı:  sozlesme/istemci/typescript/src/katalog.ts
        sozlesme/istemci/swift/Katalog.swift
        sunucu/crates/common/src/catalog/katalog.json (Rust'ın gömdüğü kopya; Docker
        derleme bağlamı yalnız sunucu/ olduğu için dosya orada da durur)

Yalnız standart kütüphane. Çıktı deterministiktir. Şema doğrulaması küçük bir
yerleşik doğrulayıcıyla yapılır (jsonschema bağımlılığı yok); şemanın kullandığı
anahtar kelimeler dışındakiler desteklenmez ve hata verir.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
KATALOG = ROOT / "sozlesme/katalog/katalog.json"
SEMA = ROOT / "sozlesme/katalog/katalog.schema.json"
TS_CIKTI = ROOT / "sozlesme/istemci/typescript/src/katalog.ts"
SWIFT_CIKTI = ROOT / "sozlesme/istemci/swift/Katalog.swift"
RUST_CIKTI = ROOT / "sunucu/crates/common/src/catalog/katalog.json"

DESTEKLENEN = {
    "$schema", "$defs", "$ref", "title", "type", "required", "properties",
    "patternProperties", "additionalProperties", "enum", "minimum", "maximum", "items",
}


def hata(mesaj):
    sys.exit(f"hata: {mesaj}")


# --- küçük JSON Schema doğrulayıcı -----------------------------------------------------

def tip_uyar(deger, tip):
    if tip == "object":
        return isinstance(deger, dict)
    if tip == "array":
        return isinstance(deger, list)
    if tip == "string":
        return isinstance(deger, str)
    if tip == "boolean":
        return isinstance(deger, bool)
    if tip == "integer":
        return isinstance(deger, int) and not isinstance(deger, bool)
    if tip == "number":
        return isinstance(deger, (int, float)) and not isinstance(deger, bool)
    hata(f"şema tipi desteklenmiyor: {tip}")


def dogrula(deger, sema, kok, yol, hatalar):
    for anahtar in sema:
        if anahtar not in DESTEKLENEN:
            hata(f"şema anahtarı desteklenmiyor: {anahtar}")
    if "$ref" in sema:
        ad = sema["$ref"].removeprefix("#/$defs/")
        dogrula(deger, kok["$defs"][ad], kok, yol, hatalar)
        return
    if "type" in sema and not tip_uyar(deger, sema["type"]):
        hatalar.append(f"{yol}: tip {sema['type']} olmalı")
        return
    if "enum" in sema and deger not in sema["enum"]:
        hatalar.append(f"{yol}: {sema['enum']} değerlerinden biri olmalı")
    if "minimum" in sema and isinstance(deger, (int, float)) and deger < sema["minimum"]:
        hatalar.append(f"{yol}: en az {sema['minimum']}")
    if "maximum" in sema and isinstance(deger, (int, float)) and deger > sema["maximum"]:
        hatalar.append(f"{yol}: en çok {sema['maximum']}")
    if isinstance(deger, dict):
        for ad in sema.get("required", []):
            if ad not in deger:
                hatalar.append(f"{yol}: '{ad}' eksik")
        ozellikler = sema.get("properties", {})
        desenler = sema.get("patternProperties", {})
        ek = sema.get("additionalProperties", True)
        for ad, alt in deger.items():
            alt_yol = f"{yol}/{ad}"
            if ad in ozellikler:
                dogrula(alt, ozellikler[ad], kok, alt_yol, hatalar)
                continue
            eslesen = [s for d, s in desenler.items() if re.search(d, ad)]
            if eslesen:
                for s in eslesen:
                    dogrula(alt, s, kok, alt_yol, hatalar)
            elif ek is False:
                hatalar.append(f"{alt_yol}: tanımsız anahtar")
            elif isinstance(ek, dict):
                dogrula(alt, ek, kok, alt_yol, hatalar)
    if isinstance(deger, list) and "items" in sema:
        for i, oge in enumerate(deger):
            dogrula(oge, sema["items"], kok, f"{yol}/{i}", hatalar)


# --- anlamsal denetim ------------------------------------------------------------------

def anlamsal(k):
    h = []
    for q, d in k["quantities"].items():
        if d["si"] not in k["units"]:
            h.append(f"quantities/{q}: SI birimi '{d['si']}' units içinde yok")
        elif k["units"][d["si"]]["quantity"] != q:
            h.append(f"quantities/{q}: SI birimi başka büyüklükte")
        else:
            si = k["units"][d["si"]]
            if (si["mul"], si["div"], si["pre"], si["post"]) != (1, 1, 0, 0) or "nonlinear" in si:
                h.append(f"quantities/{q}: SI birimi özdeşlik olmalı")
    for ad, d in k["units"].items():
        if d["quantity"] not in k["quantities"]:
            h.append(f"units/{ad}: büyüklük '{d['quantity']}' yok")
        if d["mul"] == 0 or d["div"] == 0:
            h.append(f"units/{ad}: mul ve div sıfır olamaz")
    for ad, d in k["fields"].items():
        if d["quantity"] not in k["quantities"]:
            h.append(f"fields/{ad}: büyüklük yok")
        elif d["unit"] != k["quantities"][d["quantity"]]["si"]:
            h.append(f"fields/{ad}: birim büyüklüğün SI birimi olmalı")
        if not d["min"] < d["max"]:
            h.append(f"fields/{ad}: min < max olmalı")
        if d["step"] <= 0:
            h.append(f"fields/{ad}: step > 0 olmalı")
        for mod, s in d.get("modes", {}).items():
            lo, hi = s.get("min", d["min"]), s.get("max", d["max"])
            if lo < d["min"] or hi > d["max"] or lo >= hi:
                h.append(f"fields/{ad}/modes/{mod}: temel aralığın içinde ve min < max olmalı")
    for yerel, t in k["texts"].items():
        if set(t["quantities"]) != set(k["quantities"]):
            h.append(f"texts/{yerel}/quantities: büyüklük kümesi uyuşmuyor")
        if set(t["units"]) != set(k["units"]):
            h.append(f"texts/{yerel}/units: birim kümesi uyuşmuyor")
        if set(t["fields"]) != set(k["fields"]):
            h.append(f"texts/{yerel}/fields: alan kümesi uyuşmuyor")
        if set(t["errors"]) != set(k["texts"]["tr"]["errors"]):
            h.append(f"texts/{yerel}/errors: kod kümesi uyuşmuyor")
    return h


def yukle():
    k = json.loads(KATALOG.read_text(encoding="utf-8"))
    sema = json.loads(SEMA.read_text(encoding="utf-8"))
    hatalar = []
    dogrula(k, sema, sema, "", hatalar)
    if not hatalar:
        hatalar = anlamsal(k)
    if hatalar:
        hata("katalog geçersiz:\n  " + "\n  ".join(hatalar))
    return k


# --- ortak yardımcılar -----------------------------------------------------------------

def sayi(x):
    """Sayıyı TypeScript ve Swift'in ikisinde de geçerli, deterministik yazar."""
    if isinstance(x, float) and x.is_integer() and abs(x) < 1e15:
        return str(int(x))
    return repr(x)


def camel(s):
    parcalar = re.split(r"[._]", s)
    return parcalar[0] + "".join(p.capitalize() for p in parcalar[1:])


def js(x):
    return json.dumps(x, ensure_ascii=False)


BASLIK = "Bu dosya sozlesme/katalog/uret.py ile katalog.json'dan üretilir; elle düzenlenmez."


# --- TypeScript ------------------------------------------------------------------------

def typescript(k):
    birimler, alanlar, buy = k["units"], k["fields"], k["quantities"]
    o = []
    w = o.append
    w(f"// {BASLIK}")
    w("// Alan sınırları yalnız kullanıcıya erken uyarı içindir; asıl doğrulama sunucudadır (ADR-0008 madde 7).")
    w("")
    w("export type Locale = " + " | ".join(js(x) for x in k["texts"]) + ";")
    w('export type Mode = "home" | "brewery";')
    w("export type QuantityId = " + " | ".join(js(x) for x in buy) + ";")
    w("export type UnitId = " + " | ".join(js(x) for x in birimler) + ";")
    w("export type FieldKey = " + " | ".join(js(x) for x in alanlar) + ";")
    w('export type FieldErrorCode = "invalid_number" | "out_of_range";')
    w("")
    w("export interface UnitDef {")
    w("  quantity: QuantityId;")
    w("  /** SI = (değer + pre) × mul / div + post */")
    w("  mul: number;")
    w("  div: number;")
    w("  pre: number;")
    w("  post: number;")
    w('  nonlinear: "plato" | "brix" | null;')
    w("}")
    w("export interface FieldLimits {")
    w("  min: number;")
    w("  max: number;")
    w("}")
    w("export interface FieldDef extends FieldLimits {")
    w("  quantity: QuantityId;")
    w("  /** Alanın SI birimi; API ve veritabanı bu birimle çalışır. */")
    w("  unit: UnitId;")
    w("  step: number;")
    w("  decimals: number;")
    w("  required: boolean;")
    w("  /** true: sınır ürün kararıdır, kaynakla doğrulanmadı. */")
    w("  verify: boolean;")
    w("  modes: Partial<Record<Mode, Partial<FieldLimits>>>;")
    w("}")
    w("export interface LocaleTexts {")
    w("  quantities: Record<QuantityId, string>;")
    w("  units: Record<UnitId, { name: string; symbol: string }>;")
    w("  fields: Record<FieldKey, { label: string; hint?: string }>;")
    w("  errors: Record<FieldErrorCode | \"required\" | \"unknown_field\", string>;")
    w("}")
    w("")
    w("export const SI_UNIT: Record<QuantityId, UnitId> = {")
    for q, d in buy.items():
        w(f"  {q}: {js(d['si'])},")
    w("};")
    w("")
    w("export const UNITS: Record<UnitId, UnitDef> = {")
    for ad, d in birimler.items():
        nl = js(d["nonlinear"]) if "nonlinear" in d else "null"
        w(f"  {ad}: {{ quantity: {js(d['quantity'])}, mul: {sayi(d['mul'])}, div: {sayi(d['div'])}, "
          f"pre: {sayi(d['pre'])}, post: {sayi(d['post'])}, nonlinear: {nl} }},")
    w("};")
    w("")
    w("export const FIELDS: Record<FieldKey, FieldDef> = {")
    for ad, d in alanlar.items():
        modlar = ", ".join(
            f"{m}: {{ " + ", ".join(f"{a}: {sayi(v)}" for a, v in s.items()) + " }"
            for m, s in d.get("modes", {}).items())
        w(f"  {js(ad)}: {{ quantity: {js(d['quantity'])}, unit: {js(d['unit'])}, min: {sayi(d['min'])}, "
          f"max: {sayi(d['max'])}, step: {sayi(d['step'])}, decimals: {d['decimals']}, "
          f"required: {js(d['required'])}, verify: {js(d['verify'])}, modes: {{ {modlar} }} }},".replace("{  }", "{}"))
    w("};")
    w("")
    w("export const TEXTS: Record<Locale, LocaleTexts> = " + json.dumps(k["texts"], ensure_ascii=False, indent=2) + ";")
    w("")
    w(TS_GOVDE)
    return "\n".join(o) + "\n"


TS_GOVDE = """\
/** SG → °Plato (formuller/01 §1.1). */
export function sgToPlato(sg: number): number {
  return 135.997 * sg ** 3 - 630.272 * sg ** 2 + 1111.14 * sg - 616.868;
}
/** °Plato → SG (formuller/01 §1.2). */
export function platoToSg(p: number): number {
  return 1 + p / (258.6 - (p / 258.6) * 227.1);
}
/** SG → °Brix (formuller/01 §1.3). */
export function sgToBrix(sg: number): number {
  return ((182.4601 * sg - 775.6821) * sg + 1262.7794) * sg - 669.5622;
}
/** °Brix → SG (formuller/01 §1.3; paydadaki 258.2 BeerSmith'te böyledir). */
export function brixToSg(bx: number): number {
  return 1 + bx / (258.6 - (bx / 258.2) * 227.1);
}

/** Birimdeki değeri büyüklüğün SI birimine çevirir. */
export function toSi(unit: UnitId, value: number): number {
  const u = UNITS[unit];
  if (u.nonlinear === "plato") return platoToSg(value);
  if (u.nonlinear === "brix") return brixToSg(value);
  return ((value + u.pre) * u.mul) / u.div + u.post;
}

/** SI değerini birime çevirir. */
export function fromSi(unit: UnitId, si: number): number {
  const u = UNITS[unit];
  if (u.nonlinear === "plato") return sgToPlato(si);
  if (u.nonlinear === "brix") return sgToBrix(si);
  return ((si - u.post) * u.div) / u.mul - u.pre;
}

/** Aynı büyüklükteki iki birim arasında çevirir; farklı büyüklükte hata fırlatır. */
export function convert(from: UnitId, to: UnitId, value: number): number {
  if (UNITS[from].quantity !== UNITS[to].quantity) {
    throw new Error(`incompatible units: ${from} -> ${to}`);
  }
  return fromSi(to, toSi(from, value));
}

/** Alanın sınırları; mode verilirse ev veya işletme sınırı uygulanır. */
export function limitsFor(key: FieldKey, mode?: Mode): FieldLimits {
  const f = FIELDS[key];
  const m = mode ? f.modes[mode] : undefined;
  return { min: m?.min ?? f.min, max: m?.max ?? f.max };
}

/** SI değerini alanın sınırlarına göre denetler; geçerliyse null. */
export function validateField(key: FieldKey, value: number, mode?: Mode): FieldErrorCode | null {
  if (typeof value !== "number" || !Number.isFinite(value)) return "invalid_number";
  const { min, max } = limitsFor(key, mode);
  return value < min || value > max ? "out_of_range" : null;
}

function plain(x: number): string {
  return String(Number(x.toPrecision(6)));
}

/** Hata kodunun kullanıcı metni. displayUnit verilirse sınırlar o birimde yazılır. */
export function errorMessage(
  locale: Locale,
  key: FieldKey,
  code: FieldErrorCode,
  options: { mode?: Mode; displayUnit?: UnitId } = {},
): string {
  const t = TEXTS[locale];
  const { min, max } = limitsFor(key, options.mode);
  const unit = options.displayUnit ?? FIELDS[key].unit;
  const lo = fromSi(unit, min);
  const hi = fromSi(unit, max);
  return t.errors[code]
    .replace("{label}", t.fields[key].label)
    .replace("{min}", plain(Math.min(lo, hi)))
    .replace("{max}", plain(Math.max(lo, hi)))
    .replace("{unit}", t.units[unit].symbol);
}
"""


# --- Swift -----------------------------------------------------------------------------

def swift_sozluk(d, girinti):
    """{str: str} sözlüğünü Swift sözlük sabiti olarak yazar."""
    ic = " " * (girinti + 4)
    satirlar = [f'{ic}{js(a)}: {js(b)},' for a, b in d.items()]
    return "[\n" + "\n".join(satirlar) + "\n" + " " * girinti + "]"


def swift(k):
    birimler, alanlar, buy = k["units"], k["fields"], k["quantities"]
    o = []
    w = o.append
    w(f"// {BASLIK}")
    w("// Alan sınırları yalnız kullanıcıya erken uyarı içindir; asıl doğrulama sunucudadır (ADR-0008 madde 7).")
    w("")
    w("import Foundation")
    w("")
    w("public enum KatalogLocale: String, CaseIterable, Sendable {")
    for x in k["texts"]:
        w(f"    case {x}")
    w("}")
    w("")
    w("public enum KatalogMode: String, Sendable {")
    w("    case home")
    w("    case brewery")
    w("}")
    w("")
    w("public enum KatalogQuantity: String, CaseIterable, Sendable {")
    for q in buy:
        w(f'    case {camel(q)} = {js(q)}')
    w("}")
    w("")
    w("public enum KatalogUnit: String, CaseIterable, Sendable {")
    for a in birimler:
        w(f'    case {camel(a)} = {js(a)}')
    w("}")
    w("")
    w("public enum KatalogField: String, CaseIterable, Sendable {")
    for a in alanlar:
        w(f'    case {camel(a)} = {js(a)}')
    w("}")
    w("")
    w("public enum KatalogFieldError: String, Sendable {")
    w('    case invalidNumber = "invalid_number"')
    w('    case outOfRange = "out_of_range"')
    w("}")
    w("")
    w("public struct KatalogUnitDef: Sendable {")
    w("    public let quantity: KatalogQuantity")
    w("    /// SI = (değer + pre) × mul / div + post")
    w("    public let mul: Double")
    w("    public let div: Double")
    w("    public let pre: Double")
    w("    public let post: Double")
    w('    public let nonlinear: String?')
    w("}")
    w("")
    w("public struct KatalogLimits: Sendable {")
    w("    public let min: Double")
    w("    public let max: Double")
    w("}")
    w("")
    w("public struct KatalogFieldDef: Sendable {")
    w("    public let quantity: KatalogQuantity")
    w("    /// Alanın SI birimi; API ve veritabanı bu birimle çalışır.")
    w("    public let unit: KatalogUnit")
    w("    public let min: Double")
    w("    public let max: Double")
    w("    public let step: Double")
    w("    public let decimals: Int")
    w("    public let required: Bool")
    w("    /// true: sınır ürün kararıdır, kaynakla doğrulanmadı.")
    w("    public let verify: Bool")
    w("    public let homeLimits: (min: Double?, max: Double?)?")
    w("    public let breweryLimits: (min: Double?, max: Double?)?")
    w("}")
    w("")
    w("public enum Katalog {")
    w("    public static let siUnit: [KatalogQuantity: KatalogUnit] = [")
    for q, d in buy.items():
        w(f"        .{camel(q)}: .{camel(d['si'])},")
    w("    ]")
    w("")
    w("    public static let units: [KatalogUnit: KatalogUnitDef] = [")
    for a, d in birimler.items():
        nl = js(d["nonlinear"]) if "nonlinear" in d else "nil"
        w(f"        .{camel(a)}: KatalogUnitDef(quantity: .{camel(d['quantity'])}, mul: {sayi(float(d['mul']))}, "
          f"div: {sayi(float(d['div']))}, pre: {sayi(float(d['pre']))}, post: {sayi(float(d['post']))}, nonlinear: {nl}),")
    w("    ]")
    w("")
    w("    public static let fields: [KatalogField: KatalogFieldDef] = [")
    for a, d in alanlar.items():
        def mod(m):
            s = d.get("modes", {}).get(m)
            if not s:
                return "nil"
            return f"(min: {sayi(float(s['min'])) if 'min' in s else 'nil'}, max: {sayi(float(s['max'])) if 'max' in s else 'nil'})"
        w(f"        .{camel(a)}: KatalogFieldDef(quantity: .{camel(d['quantity'])}, unit: .{camel(d['unit'])}, "
          f"min: {sayi(float(d['min']))}, max: {sayi(float(d['max']))}, step: {sayi(float(d['step']))}, "
          f"decimals: {d['decimals']}, required: {js(d['required'])}, verify: {js(d['verify'])}, "
          f"homeLimits: {mod('home')}, breweryLimits: {mod('brewery')}),")
    w("    ]")
    w("")
    for yerel, t in k["texts"].items():
        w(f"    public static let {yerel}Quantities: [String: String] = " + swift_sozluk(t["quantities"], 4))
        w(f"    public static let {yerel}UnitNames: [String: String] = " + swift_sozluk({a: v["name"] for a, v in t["units"].items()}, 4))
        w(f"    public static let {yerel}UnitSymbols: [String: String] = " + swift_sozluk({a: v["symbol"] for a, v in t["units"].items()}, 4))
        w(f"    public static let {yerel}FieldLabels: [String: String] = " + swift_sozluk({a: v["label"] for a, v in t["fields"].items()}, 4))
        w(f"    public static let {yerel}Errors: [String: String] = " + swift_sozluk(t["errors"], 4))
        w("")
    w(SWIFT_GOVDE)
    w("}")
    return "\n".join(o) + "\n"


SWIFT_GOVDE = """\
    // MARK: Gravite formülleri (formuller/01 §1.1-§1.3)

    public static func sgToPlato(_ sg: Double) -> Double {
        135.997 * pow(sg, 3) - 630.272 * pow(sg, 2) + 1111.14 * sg - 616.868
    }

    public static func platoToSg(_ p: Double) -> Double {
        1 + p / (258.6 - (p / 258.6) * 227.1)
    }

    public static func sgToBrix(_ sg: Double) -> Double {
        ((182.4601 * sg - 775.6821) * sg + 1262.7794) * sg - 669.5622
    }

    /// Paydadaki 258.2 BeerSmith'te böyledir.
    public static func brixToSg(_ bx: Double) -> Double {
        1 + bx / (258.6 - (bx / 258.2) * 227.1)
    }

    // MARK: Dönüşüm

    /// Birimdeki değeri büyüklüğün SI birimine çevirir.
    public static func toSi(_ unit: KatalogUnit, _ value: Double) -> Double {
        let u = units[unit]!
        switch u.nonlinear {
        case "plato": return platoToSg(value)
        case "brix": return brixToSg(value)
        default: return (value + u.pre) * u.mul / u.div + u.post
        }
    }

    /// SI değerini birime çevirir.
    public static func fromSi(_ unit: KatalogUnit, _ si: Double) -> Double {
        let u = units[unit]!
        switch u.nonlinear {
        case "plato": return sgToPlato(si)
        case "brix": return sgToBrix(si)
        default: return (si - u.post) * u.div / u.mul - u.pre
        }
    }

    /// Aynı büyüklükteki iki birim arasında çevirir; farklı büyüklükte nil döner.
    public static func convert(from: KatalogUnit, to: KatalogUnit, _ value: Double) -> Double? {
        guard units[from]!.quantity == units[to]!.quantity else { return nil }
        return fromSi(to, toSi(from, value))
    }

    // MARK: Doğrulama

    public static func limits(_ key: KatalogField, mode: KatalogMode? = nil) -> KatalogLimits {
        let f = fields[key]!
        let m: (min: Double?, max: Double?)?
        switch mode {
        case .home: m = f.homeLimits
        case .brewery: m = f.breweryLimits
        case nil: m = nil
        }
        return KatalogLimits(min: m?.min ?? f.min, max: m?.max ?? f.max)
    }

    /// SI değerini alanın sınırlarına göre denetler; geçerliyse nil.
    public static func validate(_ key: KatalogField, _ value: Double, mode: KatalogMode? = nil) -> KatalogFieldError? {
        guard value.isFinite else { return .invalidNumber }
        let l = limits(key, mode: mode)
        return value < l.min || value > l.max ? .outOfRange : nil
    }

    private static func plain(_ x: Double) -> String {
        String(format: "%.6g", x)
    }

    /// Hata kodunun kullanıcı metni. displayUnit verilirse sınırlar o birimde yazılır.
    public static func errorMessage(
        _ locale: KatalogLocale, _ key: KatalogField, _ code: KatalogFieldError,
        mode: KatalogMode? = nil, displayUnit: KatalogUnit? = nil
    ) -> String {
        let (errors, labels, symbols): ([String: String], [String: String], [String: String])
        switch locale {
        case .tr: (errors, labels, symbols) = (trErrors, trFieldLabels, trUnitSymbols)
        case .en: (errors, labels, symbols) = (enErrors, enFieldLabels, enUnitSymbols)
        }
        let l = limits(key, mode: mode)
        let unit = displayUnit ?? fields[key]!.unit
        let lo = fromSi(unit, l.min)
        let hi = fromSi(unit, l.max)
        return (errors[code.rawValue] ?? code.rawValue)
            .replacingOccurrences(of: "{label}", with: labels[key.rawValue] ?? key.rawValue)
            .replacingOccurrences(of: "{min}", with: plain(Swift.min(lo, hi)))
            .replacingOccurrences(of: "{max}", with: plain(Swift.max(lo, hi)))
            .replacingOccurrences(of: "{unit}", with: symbols[unit.rawValue] ?? unit.rawValue)
    }"""


def main():
    k = yukle()
    TS_CIKTI.write_text(typescript(k), encoding="utf-8")
    SWIFT_CIKTI.parent.mkdir(parents=True, exist_ok=True)
    SWIFT_CIKTI.write_text(swift(k), encoding="utf-8")
    RUST_CIKTI.parent.mkdir(parents=True, exist_ok=True)
    RUST_CIKTI.write_text(KATALOG.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"ok: {len(k['fields'])} alan, {len(k['units'])} birim, {len(k['texts'])} dil")


if __name__ == "__main__":
    main()
