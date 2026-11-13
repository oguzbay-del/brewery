// Bu dosya sozlesme/katalog/uret.py ile katalog.json'dan üretilir; elle düzenlenmez.
// Alan sınırları yalnız kullanıcıya erken uyarı içindir; asıl doğrulama sunucudadır (ADR-0008 madde 7).

export type Locale = "tr" | "en";
export type Mode = "home" | "brewery";
export type QuantityId = "volume" | "mass" | "temperature" | "gravity" | "color" | "pressure" | "time" | "length" | "percent" | "concentration" | "hardness" | "co2" | "dose" | "ratio" | "bitterness" | "ph";
export type UnitId = "l" | "ml" | "hl" | "gal_us" | "qt_us" | "fl_oz_us" | "gal_uk" | "qt_uk" | "fl_oz_uk" | "bbl_us" | "bbl_uk" | "kg" | "g" | "lb" | "oz" | "c" | "f" | "sg" | "sg_x1000" | "plato" | "brix" | "srm" | "ebc" | "lovibond" | "bar" | "kpa" | "psi" | "min" | "h" | "d" | "m" | "ft" | "pct" | "ppm" | "mg_l" | "caco3_mg_l" | "dh" | "fh" | "meq_l" | "vol" | "g_l_co2" | "g_l" | "oz_gal_us" | "oz_gal_uk" | "lb_bbl_us" | "l_kg" | "qt_lb" | "ibu" | "ph";
export type FieldKey = "recipe.batch_volume_l" | "recipe.boil_volume_l" | "recipe.boil_time_min" | "recipe.efficiency_pct" | "equipment.trub_loss_l" | "equipment.pre_trub_loss_l" | "equipment.altitude_m" | "equipment.hop_util_pct" | "equipment.whirlpool_time_min" | "fermentable.amount_kg" | "fermentable.yield_pct" | "fermentable.color_srm" | "fermentable.late_extract_min" | "hop.amount_kg" | "hop.alpha_pct" | "hop.time_min" | "hop.whirlpool_temp_c" | "yeast.min_attenuation_pct" | "yeast.max_attenuation_pct" | "yeast.tolerance_abv_pct" | "yeast.viability_pct" | "mash.grain_weight_kg" | "mash.step_temp_c" | "mash.step_time_min" | "mash.thickness_l_per_kg" | "measurement.sg" | "measurement.og_sg" | "measurement.fg_sg" | "measurement.temp_c" | "measurement.ph" | "target.ibu" | "target.color_srm" | "target.mash_ph" | "target.co2_vol" | "process.keg_pressure_bar" | "process.priming_g_per_l" | "water.calcium_ppm" | "water.magnesium_ppm" | "water.sodium_ppm" | "water.chloride_ppm" | "water.sulfate_ppm" | "water.bicarbonate_ppm" | "water.ph" | "tool.volume_l" | "tool.mass_kg";
export type FieldErrorCode = "invalid_number" | "out_of_range";

export interface UnitDef {
  quantity: QuantityId;
  /** SI = (değer + pre) × mul / div + post */
  mul: number;
  div: number;
  pre: number;
  post: number;
  nonlinear: "plato" | "brix" | null;
}
export interface FieldLimits {
  min: number;
  max: number;
}
export interface FieldDef extends FieldLimits {
  quantity: QuantityId;
  /** Alanın SI birimi; API ve veritabanı bu birimle çalışır. */
  unit: UnitId;
  step: number;
  decimals: number;
  required: boolean;
  /** true: sınır ürün kararıdır, kaynakla doğrulanmadı. */
  verify: boolean;
  modes: Partial<Record<Mode, Partial<FieldLimits>>>;
}
export interface LocaleTexts {
  quantities: Record<QuantityId, string>;
  units: Record<UnitId, { name: string; symbol: string }>;
  fields: Record<FieldKey, { label: string; hint?: string }>;
  errors: Record<FieldErrorCode | "required" | "unknown_field", string>;
}

export const SI_UNIT: Record<QuantityId, UnitId> = {
  volume: "l",
  mass: "kg",
  temperature: "c",
  gravity: "sg",
  color: "srm",
  pressure: "bar",
  time: "min",
  length: "m",
  percent: "pct",
  concentration: "ppm",
  hardness: "caco3_mg_l",
  co2: "vol",
  dose: "g_l",
  ratio: "l_kg",
  bitterness: "ibu",
  ph: "ph",
};

export const UNITS: Record<UnitId, UnitDef> = {
  l: { quantity: "volume", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  ml: { quantity: "volume", mul: 0.001, div: 1, pre: 0, post: 0, nonlinear: null },
  hl: { quantity: "volume", mul: 100, div: 1, pre: 0, post: 0, nonlinear: null },
  gal_us: { quantity: "volume", mul: 3.785411784, div: 1, pre: 0, post: 0, nonlinear: null },
  qt_us: { quantity: "volume", mul: 0.946352946, div: 1, pre: 0, post: 0, nonlinear: null },
  fl_oz_us: { quantity: "volume", mul: 0.0295735295625, div: 1, pre: 0, post: 0, nonlinear: null },
  gal_uk: { quantity: "volume", mul: 4.54609, div: 1, pre: 0, post: 0, nonlinear: null },
  qt_uk: { quantity: "volume", mul: 1.1365225, div: 1, pre: 0, post: 0, nonlinear: null },
  fl_oz_uk: { quantity: "volume", mul: 0.0284130625, div: 1, pre: 0, post: 0, nonlinear: null },
  bbl_us: { quantity: "volume", mul: 117.347765304, div: 1, pre: 0, post: 0, nonlinear: null },
  bbl_uk: { quantity: "volume", mul: 163.65924, div: 1, pre: 0, post: 0, nonlinear: null },
  kg: { quantity: "mass", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  g: { quantity: "mass", mul: 0.001, div: 1, pre: 0, post: 0, nonlinear: null },
  lb: { quantity: "mass", mul: 0.45359237, div: 1, pre: 0, post: 0, nonlinear: null },
  oz: { quantity: "mass", mul: 0.028349523125, div: 1, pre: 0, post: 0, nonlinear: null },
  c: { quantity: "temperature", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  f: { quantity: "temperature", mul: 5, div: 9, pre: -32, post: 0, nonlinear: null },
  sg: { quantity: "gravity", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  sg_x1000: { quantity: "gravity", mul: 1, div: 1000, pre: 0, post: 0, nonlinear: null },
  plato: { quantity: "gravity", mul: 1, div: 1, pre: 0, post: 0, nonlinear: "plato" },
  brix: { quantity: "gravity", mul: 1, div: 1, pre: 0, post: 0, nonlinear: "brix" },
  srm: { quantity: "color", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  ebc: { quantity: "color", mul: 1, div: 1.97, pre: 0, post: 0, nonlinear: null },
  lovibond: { quantity: "color", mul: 1.3564, div: 1, pre: 0, post: -0.76, nonlinear: null },
  bar: { quantity: "pressure", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  kpa: { quantity: "pressure", mul: 0.01, div: 1, pre: 0, post: 0, nonlinear: null },
  psi: { quantity: "pressure", mul: 0.0689475729316836, div: 1, pre: 0, post: 0, nonlinear: null },
  min: { quantity: "time", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  h: { quantity: "time", mul: 60, div: 1, pre: 0, post: 0, nonlinear: null },
  d: { quantity: "time", mul: 1440, div: 1, pre: 0, post: 0, nonlinear: null },
  m: { quantity: "length", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  ft: { quantity: "length", mul: 0.3048, div: 1, pre: 0, post: 0, nonlinear: null },
  pct: { quantity: "percent", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  ppm: { quantity: "concentration", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  mg_l: { quantity: "concentration", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  caco3_mg_l: { quantity: "hardness", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  dh: { quantity: "hardness", mul: 17.848, div: 1, pre: 0, post: 0, nonlinear: null },
  fh: { quantity: "hardness", mul: 10.009, div: 1, pre: 0, post: 0, nonlinear: null },
  meq_l: { quantity: "hardness", mul: 50.04, div: 1, pre: 0, post: 0, nonlinear: null },
  vol: { quantity: "co2", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  g_l_co2: { quantity: "co2", mul: 1, div: 1.9635, pre: 0, post: 0, nonlinear: null },
  g_l: { quantity: "dose", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  oz_gal_us: { quantity: "dose", mul: 28.349523125, div: 3.785411784, pre: 0, post: 0, nonlinear: null },
  oz_gal_uk: { quantity: "dose", mul: 28.349523125, div: 4.54609, pre: 0, post: 0, nonlinear: null },
  lb_bbl_us: { quantity: "dose", mul: 453.59237, div: 117.347765304, pre: 0, post: 0, nonlinear: null },
  l_kg: { quantity: "ratio", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  qt_lb: { quantity: "ratio", mul: 0.946352946, div: 0.45359237, pre: 0, post: 0, nonlinear: null },
  ibu: { quantity: "bitterness", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
  ph: { quantity: "ph", mul: 1, div: 1, pre: 0, post: 0, nonlinear: null },
};

export const FIELDS: Record<FieldKey, FieldDef> = {
  "recipe.batch_volume_l": { quantity: "volume", unit: "l", min: 0.5, max: 100000, step: 0.5, decimals: 1, required: true, verify: true, modes: { home: { max: 200 }, brewery: { min: 5 } } },
  "recipe.boil_volume_l": { quantity: "volume", unit: "l", min: 0.5, max: 120000, step: 0.5, decimals: 1, required: true, verify: true, modes: { home: { max: 250 }, brewery: { min: 5 } } },
  "recipe.boil_time_min": { quantity: "time", unit: "min", min: 0, max: 600, step: 5, decimals: 0, required: true, verify: true, modes: {} },
  "recipe.efficiency_pct": { quantity: "percent", unit: "pct", min: 1, max: 100, step: 1, decimals: 0, required: true, verify: true, modes: {} },
  "equipment.trub_loss_l": { quantity: "volume", unit: "l", min: 0, max: 5000, step: 0.1, decimals: 1, required: false, verify: true, modes: { home: { max: 50 } } },
  "equipment.pre_trub_loss_l": { quantity: "volume", unit: "l", min: 0, max: 5000, step: 0.1, decimals: 1, required: false, verify: true, modes: { home: { max: 50 } } },
  "equipment.altitude_m": { quantity: "length", unit: "m", min: -500, max: 5000, step: 10, decimals: 0, required: false, verify: true, modes: {} },
  "equipment.hop_util_pct": { quantity: "percent", unit: "pct", min: 0, max: 150, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "equipment.whirlpool_time_min": { quantity: "time", unit: "min", min: 0, max: 240, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "fermentable.amount_kg": { quantity: "mass", unit: "kg", min: 0, max: 20000, step: 0.01, decimals: 2, required: true, verify: true, modes: { home: { max: 100 } } },
  "fermentable.yield_pct": { quantity: "percent", unit: "pct", min: 0, max: 100, step: 0.1, decimals: 1, required: true, verify: false, modes: {} },
  "fermentable.color_srm": { quantity: "color", unit: "srm", min: 0, max: 1000, step: 0.5, decimals: 1, required: true, verify: true, modes: {} },
  "fermentable.late_extract_min": { quantity: "time", unit: "min", min: 0, max: 600, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "hop.amount_kg": { quantity: "mass", unit: "kg", min: 0, max: 100, step: 0.001, decimals: 3, required: true, verify: true, modes: { home: { max: 5 } } },
  "hop.alpha_pct": { quantity: "percent", unit: "pct", min: 0, max: 40, step: 0.1, decimals: 1, required: true, verify: true, modes: {} },
  "hop.time_min": { quantity: "time", unit: "min", min: 0, max: 10080, step: 1, decimals: 0, required: true, verify: true, modes: {} },
  "hop.whirlpool_temp_c": { quantity: "temperature", unit: "c", min: 0, max: 100, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "yeast.min_attenuation_pct": { quantity: "percent", unit: "pct", min: 0, max: 100, step: 1, decimals: 0, required: false, verify: false, modes: {} },
  "yeast.max_attenuation_pct": { quantity: "percent", unit: "pct", min: 0, max: 100, step: 1, decimals: 0, required: false, verify: false, modes: {} },
  "yeast.tolerance_abv_pct": { quantity: "percent", unit: "pct", min: 0, max: 25, step: 0.5, decimals: 1, required: false, verify: true, modes: {} },
  "yeast.viability_pct": { quantity: "percent", unit: "pct", min: 0, max: 100, step: 1, decimals: 0, required: false, verify: false, modes: {} },
  "mash.grain_weight_kg": { quantity: "mass", unit: "kg", min: 0, max: 20000, step: 0.01, decimals: 2, required: false, verify: true, modes: { home: { max: 100 } } },
  "mash.step_temp_c": { quantity: "temperature", unit: "c", min: 0, max: 100, step: 0.5, decimals: 1, required: true, verify: true, modes: {} },
  "mash.step_time_min": { quantity: "time", unit: "min", min: 0, max: 600, step: 1, decimals: 0, required: true, verify: true, modes: {} },
  "mash.thickness_l_per_kg": { quantity: "ratio", unit: "l_kg", min: 0.5, max: 10, step: 0.1, decimals: 1, required: false, verify: true, modes: {} },
  "measurement.sg": { quantity: "gravity", unit: "sg", min: 0.98, max: 1.25, step: 0.001, decimals: 3, required: true, verify: true, modes: {} },
  "measurement.og_sg": { quantity: "gravity", unit: "sg", min: 1, max: 1.25, step: 0.001, decimals: 3, required: true, verify: true, modes: {} },
  "measurement.fg_sg": { quantity: "gravity", unit: "sg", min: 0.98, max: 1.25, step: 0.001, decimals: 3, required: true, verify: true, modes: {} },
  "measurement.temp_c": { quantity: "temperature", unit: "c", min: -20, max: 120, step: 0.5, decimals: 1, required: true, verify: true, modes: {} },
  "measurement.ph": { quantity: "ph", unit: "ph", min: 0, max: 14, step: 0.01, decimals: 2, required: true, verify: true, modes: {} },
  "target.ibu": { quantity: "bitterness", unit: "ibu", min: 0, max: 300, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "target.color_srm": { quantity: "color", unit: "srm", min: 0, max: 200, step: 0.5, decimals: 1, required: false, verify: true, modes: {} },
  "target.mash_ph": { quantity: "ph", unit: "ph", min: 4.5, max: 6.5, step: 0.01, decimals: 2, required: false, verify: true, modes: {} },
  "target.co2_vol": { quantity: "co2", unit: "vol", min: 0, max: 5, step: 0.1, decimals: 2, required: false, verify: true, modes: {} },
  "process.keg_pressure_bar": { quantity: "pressure", unit: "bar", min: 0, max: 6, step: 0.05, decimals: 2, required: false, verify: true, modes: {} },
  "process.priming_g_per_l": { quantity: "dose", unit: "g_l", min: 0, max: 20, step: 0.1, decimals: 1, required: false, verify: true, modes: {} },
  "water.calcium_ppm": { quantity: "concentration", unit: "ppm", min: 0, max: 1000, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "water.magnesium_ppm": { quantity: "concentration", unit: "ppm", min: 0, max: 500, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "water.sodium_ppm": { quantity: "concentration", unit: "ppm", min: 0, max: 1000, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "water.chloride_ppm": { quantity: "concentration", unit: "ppm", min: 0, max: 2000, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "water.sulfate_ppm": { quantity: "concentration", unit: "ppm", min: 0, max: 3000, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "water.bicarbonate_ppm": { quantity: "concentration", unit: "ppm", min: 0, max: 1500, step: 1, decimals: 0, required: false, verify: true, modes: {} },
  "water.ph": { quantity: "ph", unit: "ph", min: 0, max: 14, step: 0.01, decimals: 2, required: false, verify: true, modes: {} },
  "tool.volume_l": { quantity: "volume", unit: "l", min: 0, max: 100000, step: 0.1, decimals: 1, required: false, verify: true, modes: {} },
  "tool.mass_kg": { quantity: "mass", unit: "kg", min: 0, max: 20000, step: 0.01, decimals: 2, required: false, verify: true, modes: {} },
};

export const TEXTS: Record<Locale, LocaleTexts> = {
  "tr": {
    "quantities": {
      "volume": "Hacim",
      "mass": "Kütle",
      "temperature": "Sıcaklık",
      "gravity": "Yoğunluk",
      "color": "Renk",
      "pressure": "Basınç",
      "time": "Süre",
      "length": "Uzunluk",
      "percent": "Yüzde",
      "concentration": "Derişim",
      "hardness": "Sertlik",
      "co2": "CO₂ miktarı",
      "dose": "Doz oranı",
      "ratio": "Su/tahıl oranı",
      "bitterness": "Acılık",
      "ph": "pH"
    },
    "units": {
      "l": {
        "name": "litre",
        "symbol": "L"
      },
      "ml": {
        "name": "mililitre",
        "symbol": "mL"
      },
      "hl": {
        "name": "hektolitre",
        "symbol": "hL"
      },
      "gal_us": {
        "name": "galon (ABD)",
        "symbol": "gal (ABD)"
      },
      "qt_us": {
        "name": "quart (ABD)",
        "symbol": "qt (ABD)"
      },
      "fl_oz_us": {
        "name": "sıvı ons (ABD)",
        "symbol": "fl oz (ABD)"
      },
      "gal_uk": {
        "name": "galon (İngiltere)",
        "symbol": "gal (İng.)"
      },
      "qt_uk": {
        "name": "quart (İngiltere)",
        "symbol": "qt (İng.)"
      },
      "fl_oz_uk": {
        "name": "sıvı ons (İngiltere)",
        "symbol": "fl oz (İng.)"
      },
      "bbl_us": {
        "name": "varil (ABD bira)",
        "symbol": "bbl (ABD)"
      },
      "bbl_uk": {
        "name": "varil (İngiltere bira)",
        "symbol": "bbl (İng.)"
      },
      "kg": {
        "name": "kilogram",
        "symbol": "kg"
      },
      "g": {
        "name": "gram",
        "symbol": "g"
      },
      "lb": {
        "name": "libre",
        "symbol": "lb"
      },
      "oz": {
        "name": "ons",
        "symbol": "oz"
      },
      "c": {
        "name": "santigrat derece",
        "symbol": "°C"
      },
      "f": {
        "name": "fahrenhayt derece",
        "symbol": "°F"
      },
      "sg": {
        "name": "özgül ağırlık",
        "symbol": "SG"
      },
      "sg_x1000": {
        "name": "özgül ağırlık (1050 biçimi)",
        "symbol": "SG×1000"
      },
      "plato": {
        "name": "Plato derecesi",
        "symbol": "°P"
      },
      "brix": {
        "name": "Brix derecesi",
        "symbol": "°Bx"
      },
      "srm": {
        "name": "SRM",
        "symbol": "SRM"
      },
      "ebc": {
        "name": "EBC",
        "symbol": "EBC"
      },
      "lovibond": {
        "name": "Lovibond derecesi",
        "symbol": "°L"
      },
      "bar": {
        "name": "bar",
        "symbol": "bar"
      },
      "kpa": {
        "name": "kilopaskal",
        "symbol": "kPa"
      },
      "psi": {
        "name": "psi",
        "symbol": "psi"
      },
      "min": {
        "name": "dakika",
        "symbol": "dk"
      },
      "h": {
        "name": "saat",
        "symbol": "sa"
      },
      "d": {
        "name": "gün",
        "symbol": "gün"
      },
      "m": {
        "name": "metre",
        "symbol": "m"
      },
      "ft": {
        "name": "fit",
        "symbol": "ft"
      },
      "pct": {
        "name": "yüzde",
        "symbol": "%"
      },
      "ppm": {
        "name": "mg/L (ppm)",
        "symbol": "ppm"
      },
      "mg_l": {
        "name": "miligram/litre",
        "symbol": "mg/L"
      },
      "caco3_mg_l": {
        "name": "mg/L (CaCO₃ olarak)",
        "symbol": "mg/L CaCO₃"
      },
      "dh": {
        "name": "Alman sertliği",
        "symbol": "°dH"
      },
      "fh": {
        "name": "Fransız sertliği",
        "symbol": "°fH"
      },
      "meq_l": {
        "name": "mEq/L",
        "symbol": "mEq/L"
      },
      "vol": {
        "name": "hacim CO₂",
        "symbol": "vol"
      },
      "g_l_co2": {
        "name": "g/L CO₂",
        "symbol": "g/L"
      },
      "g_l": {
        "name": "gram/litre",
        "symbol": "g/L"
      },
      "oz_gal_us": {
        "name": "ons/galon (ABD)",
        "symbol": "oz/gal"
      },
      "oz_gal_uk": {
        "name": "ons/galon (İngiltere)",
        "symbol": "oz/gal (İng.)"
      },
      "lb_bbl_us": {
        "name": "libre/varil (ABD)",
        "symbol": "lb/bbl"
      },
      "l_kg": {
        "name": "litre/kg",
        "symbol": "L/kg"
      },
      "qt_lb": {
        "name": "quart/libre",
        "symbol": "qt/lb"
      },
      "ibu": {
        "name": "IBU",
        "symbol": "IBU"
      },
      "ph": {
        "name": "pH",
        "symbol": "pH"
      }
    },
    "fields": {
      "recipe.batch_volume_l": {
        "label": "Parti hacmi (fermentöre)"
      },
      "recipe.boil_volume_l": {
        "label": "Kaynatma hacmi"
      },
      "recipe.boil_time_min": {
        "label": "Kaynatma süresi"
      },
      "recipe.efficiency_pct": {
        "label": "Verim"
      },
      "equipment.trub_loss_l": {
        "label": "Tortu kaybı"
      },
      "equipment.pre_trub_loss_l": {
        "label": "Tortu öncesi kayıp"
      },
      "equipment.altitude_m": {
        "label": "Rakım"
      },
      "equipment.hop_util_pct": {
        "label": "Hop kullanım oranı"
      },
      "equipment.whirlpool_time_min": {
        "label": "Whirlpool süresi"
      },
      "fermentable.amount_kg": {
        "label": "Malt miktarı"
      },
      "fermentable.yield_pct": {
        "label": "Ekstrakt verimi"
      },
      "fermentable.color_srm": {
        "label": "Malt rengi"
      },
      "fermentable.late_extract_min": {
        "label": "Geç ekstrakt süresi"
      },
      "hop.amount_kg": {
        "label": "Hop miktarı"
      },
      "hop.alpha_pct": {
        "label": "Alfa asit"
      },
      "hop.time_min": {
        "label": "Hop süresi"
      },
      "hop.whirlpool_temp_c": {
        "label": "Whirlpool sıcaklığı"
      },
      "yeast.min_attenuation_pct": {
        "label": "En düşük atenüasyon"
      },
      "yeast.max_attenuation_pct": {
        "label": "En yüksek atenüasyon"
      },
      "yeast.tolerance_abv_pct": {
        "label": "Alkol toleransı"
      },
      "yeast.viability_pct": {
        "label": "Canlılık"
      },
      "mash.grain_weight_kg": {
        "label": "Tahıl ağırlığı"
      },
      "mash.step_temp_c": {
        "label": "Mash adım sıcaklığı"
      },
      "mash.step_time_min": {
        "label": "Mash adım süresi"
      },
      "mash.thickness_l_per_kg": {
        "label": "Su/tahıl oranı"
      },
      "measurement.sg": {
        "label": "Gravite"
      },
      "measurement.og_sg": {
        "label": "Başlangıç gravitesi (OG)"
      },
      "measurement.fg_sg": {
        "label": "Bitiş gravitesi (FG)"
      },
      "measurement.temp_c": {
        "label": "Sıcaklık"
      },
      "measurement.ph": {
        "label": "pH"
      },
      "target.ibu": {
        "label": "Hedef IBU"
      },
      "target.color_srm": {
        "label": "Hedef renk"
      },
      "target.mash_ph": {
        "label": "Hedef mash pH"
      },
      "target.co2_vol": {
        "label": "Hedef karbonasyon"
      },
      "process.keg_pressure_bar": {
        "label": "Fıçı basıncı"
      },
      "process.priming_g_per_l": {
        "label": "Priming şekeri oranı"
      },
      "water.calcium_ppm": {
        "label": "Kalsiyum"
      },
      "water.magnesium_ppm": {
        "label": "Magnezyum"
      },
      "water.sodium_ppm": {
        "label": "Sodyum"
      },
      "water.chloride_ppm": {
        "label": "Klorür"
      },
      "water.sulfate_ppm": {
        "label": "Sülfat"
      },
      "water.bicarbonate_ppm": {
        "label": "Bikarbonat"
      },
      "water.ph": {
        "label": "Su pH'ı"
      },
      "tool.volume_l": {
        "label": "Hacim (araç)"
      },
      "tool.mass_kg": {
        "label": "Kütle (araç)"
      }
    },
    "errors": {
      "required": "{label} gerekli.",
      "invalid_number": "{label} geçerli bir sayı olmalı.",
      "out_of_range": "{label} {min} ile {max} {unit} arasında olmalı.",
      "unknown_field": "Bilinmeyen alan: {field}."
    }
  },
  "en": {
    "quantities": {
      "volume": "Volume",
      "mass": "Mass",
      "temperature": "Temperature",
      "gravity": "Gravity",
      "color": "Colour",
      "pressure": "Pressure",
      "time": "Duration",
      "length": "Length",
      "percent": "Percent",
      "concentration": "Concentration",
      "hardness": "Hardness",
      "co2": "CO₂ amount",
      "dose": "Dose rate",
      "ratio": "Water-to-grain ratio",
      "bitterness": "Bitterness",
      "ph": "pH"
    },
    "units": {
      "l": {
        "name": "litre",
        "symbol": "L"
      },
      "ml": {
        "name": "millilitre",
        "symbol": "mL"
      },
      "hl": {
        "name": "hectolitre",
        "symbol": "hL"
      },
      "gal_us": {
        "name": "gallon (US)",
        "symbol": "gal (US)"
      },
      "qt_us": {
        "name": "quart (US)",
        "symbol": "qt (US)"
      },
      "fl_oz_us": {
        "name": "fluid ounce (US)",
        "symbol": "fl oz (US)"
      },
      "gal_uk": {
        "name": "gallon (UK)",
        "symbol": "gal (UK)"
      },
      "qt_uk": {
        "name": "quart (UK)",
        "symbol": "qt (UK)"
      },
      "fl_oz_uk": {
        "name": "fluid ounce (UK)",
        "symbol": "fl oz (UK)"
      },
      "bbl_us": {
        "name": "barrel (US beer)",
        "symbol": "bbl (US)"
      },
      "bbl_uk": {
        "name": "barrel (UK brewer's)",
        "symbol": "bbl (UK)"
      },
      "kg": {
        "name": "kilogram",
        "symbol": "kg"
      },
      "g": {
        "name": "gram",
        "symbol": "g"
      },
      "lb": {
        "name": "pound",
        "symbol": "lb"
      },
      "oz": {
        "name": "ounce",
        "symbol": "oz"
      },
      "c": {
        "name": "degree Celsius",
        "symbol": "°C"
      },
      "f": {
        "name": "degree Fahrenheit",
        "symbol": "°F"
      },
      "sg": {
        "name": "specific gravity",
        "symbol": "SG"
      },
      "sg_x1000": {
        "name": "specific gravity (1050 form)",
        "symbol": "SG×1000"
      },
      "plato": {
        "name": "degrees Plato",
        "symbol": "°P"
      },
      "brix": {
        "name": "degrees Brix",
        "symbol": "°Bx"
      },
      "srm": {
        "name": "SRM",
        "symbol": "SRM"
      },
      "ebc": {
        "name": "EBC",
        "symbol": "EBC"
      },
      "lovibond": {
        "name": "degrees Lovibond",
        "symbol": "°L"
      },
      "bar": {
        "name": "bar",
        "symbol": "bar"
      },
      "kpa": {
        "name": "kilopascal",
        "symbol": "kPa"
      },
      "psi": {
        "name": "psi",
        "symbol": "psi"
      },
      "min": {
        "name": "minute",
        "symbol": "min"
      },
      "h": {
        "name": "hour",
        "symbol": "h"
      },
      "d": {
        "name": "day",
        "symbol": "d"
      },
      "m": {
        "name": "metre",
        "symbol": "m"
      },
      "ft": {
        "name": "foot",
        "symbol": "ft"
      },
      "pct": {
        "name": "percent",
        "symbol": "%"
      },
      "ppm": {
        "name": "mg/L (ppm)",
        "symbol": "ppm"
      },
      "mg_l": {
        "name": "milligram per litre",
        "symbol": "mg/L"
      },
      "caco3_mg_l": {
        "name": "mg/L (as CaCO₃)",
        "symbol": "mg/L CaCO₃"
      },
      "dh": {
        "name": "German degree",
        "symbol": "°dH"
      },
      "fh": {
        "name": "French degree",
        "symbol": "°fH"
      },
      "meq_l": {
        "name": "mEq/L",
        "symbol": "mEq/L"
      },
      "vol": {
        "name": "volumes CO₂",
        "symbol": "vol"
      },
      "g_l_co2": {
        "name": "g/L CO₂",
        "symbol": "g/L"
      },
      "g_l": {
        "name": "gram per litre",
        "symbol": "g/L"
      },
      "oz_gal_us": {
        "name": "ounce per US gallon",
        "symbol": "oz/gal"
      },
      "oz_gal_uk": {
        "name": "ounce per UK gallon",
        "symbol": "oz/gal (UK)"
      },
      "lb_bbl_us": {
        "name": "pound per US barrel",
        "symbol": "lb/bbl"
      },
      "l_kg": {
        "name": "litre per kilogram",
        "symbol": "L/kg"
      },
      "qt_lb": {
        "name": "quart per pound",
        "symbol": "qt/lb"
      },
      "ibu": {
        "name": "IBU",
        "symbol": "IBU"
      },
      "ph": {
        "name": "pH",
        "symbol": "pH"
      }
    },
    "fields": {
      "recipe.batch_volume_l": {
        "label": "Batch volume (into fermenter)"
      },
      "recipe.boil_volume_l": {
        "label": "Boil volume"
      },
      "recipe.boil_time_min": {
        "label": "Boil time"
      },
      "recipe.efficiency_pct": {
        "label": "Efficiency"
      },
      "equipment.trub_loss_l": {
        "label": "Trub loss"
      },
      "equipment.pre_trub_loss_l": {
        "label": "Pre-trub loss"
      },
      "equipment.altitude_m": {
        "label": "Altitude"
      },
      "equipment.hop_util_pct": {
        "label": "Hop utilization"
      },
      "equipment.whirlpool_time_min": {
        "label": "Whirlpool time"
      },
      "fermentable.amount_kg": {
        "label": "Fermentable amount"
      },
      "fermentable.yield_pct": {
        "label": "Extract yield"
      },
      "fermentable.color_srm": {
        "label": "Fermentable colour"
      },
      "fermentable.late_extract_min": {
        "label": "Late extract time"
      },
      "hop.amount_kg": {
        "label": "Hop amount"
      },
      "hop.alpha_pct": {
        "label": "Alpha acid"
      },
      "hop.time_min": {
        "label": "Hop time"
      },
      "hop.whirlpool_temp_c": {
        "label": "Whirlpool temperature"
      },
      "yeast.min_attenuation_pct": {
        "label": "Minimum attenuation"
      },
      "yeast.max_attenuation_pct": {
        "label": "Maximum attenuation"
      },
      "yeast.tolerance_abv_pct": {
        "label": "Alcohol tolerance"
      },
      "yeast.viability_pct": {
        "label": "Viability"
      },
      "mash.grain_weight_kg": {
        "label": "Grain weight"
      },
      "mash.step_temp_c": {
        "label": "Mash step temperature"
      },
      "mash.step_time_min": {
        "label": "Mash step time"
      },
      "mash.thickness_l_per_kg": {
        "label": "Water-to-grain ratio"
      },
      "measurement.sg": {
        "label": "Gravity"
      },
      "measurement.og_sg": {
        "label": "Original gravity (OG)"
      },
      "measurement.fg_sg": {
        "label": "Final gravity (FG)"
      },
      "measurement.temp_c": {
        "label": "Temperature"
      },
      "measurement.ph": {
        "label": "pH"
      },
      "target.ibu": {
        "label": "Target IBU"
      },
      "target.color_srm": {
        "label": "Target colour"
      },
      "target.mash_ph": {
        "label": "Target mash pH"
      },
      "target.co2_vol": {
        "label": "Target carbonation"
      },
      "process.keg_pressure_bar": {
        "label": "Keg pressure"
      },
      "process.priming_g_per_l": {
        "label": "Priming sugar rate"
      },
      "water.calcium_ppm": {
        "label": "Calcium"
      },
      "water.magnesium_ppm": {
        "label": "Magnesium"
      },
      "water.sodium_ppm": {
        "label": "Sodium"
      },
      "water.chloride_ppm": {
        "label": "Chloride"
      },
      "water.sulfate_ppm": {
        "label": "Sulfate"
      },
      "water.bicarbonate_ppm": {
        "label": "Bicarbonate"
      },
      "water.ph": {
        "label": "Water pH"
      },
      "tool.volume_l": {
        "label": "Volume (tool)"
      },
      "tool.mass_kg": {
        "label": "Mass (tool)"
      }
    },
    "errors": {
      "required": "{label} is required.",
      "invalid_number": "{label} must be a valid number.",
      "out_of_range": "{label} must be between {min} and {max} {unit}.",
      "unknown_field": "Unknown field: {field}."
    }
  }
};

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

