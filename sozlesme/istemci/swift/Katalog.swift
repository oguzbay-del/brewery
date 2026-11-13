// Bu dosya sozlesme/katalog/uret.py ile katalog.json'dan üretilir; elle düzenlenmez.
// Alan sınırları yalnız kullanıcıya erken uyarı içindir; asıl doğrulama sunucudadır (ADR-0008 madde 7).

import Foundation

public enum KatalogLocale: String, CaseIterable, Sendable {
    case tr
    case en
}

public enum KatalogMode: String, Sendable {
    case home
    case brewery
}

public enum KatalogQuantity: String, CaseIterable, Sendable {
    case volume = "volume"
    case mass = "mass"
    case temperature = "temperature"
    case gravity = "gravity"
    case color = "color"
    case pressure = "pressure"
    case time = "time"
    case length = "length"
    case percent = "percent"
    case concentration = "concentration"
    case hardness = "hardness"
    case co2 = "co2"
    case dose = "dose"
    case ratio = "ratio"
    case bitterness = "bitterness"
    case ph = "ph"
}

public enum KatalogUnit: String, CaseIterable, Sendable {
    case l = "l"
    case ml = "ml"
    case hl = "hl"
    case galUs = "gal_us"
    case qtUs = "qt_us"
    case flOzUs = "fl_oz_us"
    case galUk = "gal_uk"
    case qtUk = "qt_uk"
    case flOzUk = "fl_oz_uk"
    case bblUs = "bbl_us"
    case bblUk = "bbl_uk"
    case kg = "kg"
    case g = "g"
    case lb = "lb"
    case oz = "oz"
    case c = "c"
    case f = "f"
    case sg = "sg"
    case sgX1000 = "sg_x1000"
    case plato = "plato"
    case brix = "brix"
    case srm = "srm"
    case ebc = "ebc"
    case lovibond = "lovibond"
    case bar = "bar"
    case kpa = "kpa"
    case psi = "psi"
    case min = "min"
    case h = "h"
    case d = "d"
    case m = "m"
    case ft = "ft"
    case pct = "pct"
    case ppm = "ppm"
    case mgL = "mg_l"
    case caco3MgL = "caco3_mg_l"
    case dh = "dh"
    case fh = "fh"
    case meqL = "meq_l"
    case vol = "vol"
    case gLCo2 = "g_l_co2"
    case gL = "g_l"
    case ozGalUs = "oz_gal_us"
    case ozGalUk = "oz_gal_uk"
    case lbBblUs = "lb_bbl_us"
    case lKg = "l_kg"
    case qtLb = "qt_lb"
    case ibu = "ibu"
    case ph = "ph"
}

public enum KatalogField: String, CaseIterable, Sendable {
    case recipeBatchVolumeL = "recipe.batch_volume_l"
    case recipeBoilVolumeL = "recipe.boil_volume_l"
    case recipeBoilTimeMin = "recipe.boil_time_min"
    case recipeEfficiencyPct = "recipe.efficiency_pct"
    case equipmentTrubLossL = "equipment.trub_loss_l"
    case equipmentPreTrubLossL = "equipment.pre_trub_loss_l"
    case equipmentAltitudeM = "equipment.altitude_m"
    case equipmentHopUtilPct = "equipment.hop_util_pct"
    case equipmentWhirlpoolTimeMin = "equipment.whirlpool_time_min"
    case fermentableAmountKg = "fermentable.amount_kg"
    case fermentableYieldPct = "fermentable.yield_pct"
    case fermentableColorSrm = "fermentable.color_srm"
    case fermentableLateExtractMin = "fermentable.late_extract_min"
    case hopAmountKg = "hop.amount_kg"
    case hopAlphaPct = "hop.alpha_pct"
    case hopTimeMin = "hop.time_min"
    case hopWhirlpoolTempC = "hop.whirlpool_temp_c"
    case yeastMinAttenuationPct = "yeast.min_attenuation_pct"
    case yeastMaxAttenuationPct = "yeast.max_attenuation_pct"
    case yeastToleranceAbvPct = "yeast.tolerance_abv_pct"
    case yeastViabilityPct = "yeast.viability_pct"
    case mashGrainWeightKg = "mash.grain_weight_kg"
    case mashStepTempC = "mash.step_temp_c"
    case mashStepTimeMin = "mash.step_time_min"
    case mashThicknessLPerKg = "mash.thickness_l_per_kg"
    case measurementSg = "measurement.sg"
    case measurementOgSg = "measurement.og_sg"
    case measurementFgSg = "measurement.fg_sg"
    case measurementTempC = "measurement.temp_c"
    case measurementPh = "measurement.ph"
    case targetIbu = "target.ibu"
    case targetColorSrm = "target.color_srm"
    case targetMashPh = "target.mash_ph"
    case targetCo2Vol = "target.co2_vol"
    case processKegPressureBar = "process.keg_pressure_bar"
    case processPrimingGPerL = "process.priming_g_per_l"
    case waterCalciumPpm = "water.calcium_ppm"
    case waterMagnesiumPpm = "water.magnesium_ppm"
    case waterSodiumPpm = "water.sodium_ppm"
    case waterChloridePpm = "water.chloride_ppm"
    case waterSulfatePpm = "water.sulfate_ppm"
    case waterBicarbonatePpm = "water.bicarbonate_ppm"
    case waterPh = "water.ph"
    case toolVolumeL = "tool.volume_l"
    case toolMassKg = "tool.mass_kg"
}

public enum KatalogFieldError: String, Sendable {
    case invalidNumber = "invalid_number"
    case outOfRange = "out_of_range"
}

public struct KatalogUnitDef: Sendable {
    public let quantity: KatalogQuantity
    /// SI = (değer + pre) × mul / div + post
    public let mul: Double
    public let div: Double
    public let pre: Double
    public let post: Double
    public let nonlinear: String?
}

public struct KatalogLimits: Sendable {
    public let min: Double
    public let max: Double
}

public struct KatalogFieldDef: Sendable {
    public let quantity: KatalogQuantity
    /// Alanın SI birimi; API ve veritabanı bu birimle çalışır.
    public let unit: KatalogUnit
    public let min: Double
    public let max: Double
    public let step: Double
    public let decimals: Int
    public let required: Bool
    /// true: sınır ürün kararıdır, kaynakla doğrulanmadı.
    public let verify: Bool
    public let homeLimits: (min: Double?, max: Double?)?
    public let breweryLimits: (min: Double?, max: Double?)?
}

public enum Katalog {
    public static let siUnit: [KatalogQuantity: KatalogUnit] = [
        .volume: .l,
        .mass: .kg,
        .temperature: .c,
        .gravity: .sg,
        .color: .srm,
        .pressure: .bar,
        .time: .min,
        .length: .m,
        .percent: .pct,
        .concentration: .ppm,
        .hardness: .caco3MgL,
        .co2: .vol,
        .dose: .gL,
        .ratio: .lKg,
        .bitterness: .ibu,
        .ph: .ph,
    ]

    public static let units: [KatalogUnit: KatalogUnitDef] = [
        .l: KatalogUnitDef(quantity: .volume, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .ml: KatalogUnitDef(quantity: .volume, mul: 0.001, div: 1, pre: 0, post: 0, nonlinear: nil),
        .hl: KatalogUnitDef(quantity: .volume, mul: 100, div: 1, pre: 0, post: 0, nonlinear: nil),
        .galUs: KatalogUnitDef(quantity: .volume, mul: 3.785411784, div: 1, pre: 0, post: 0, nonlinear: nil),
        .qtUs: KatalogUnitDef(quantity: .volume, mul: 0.946352946, div: 1, pre: 0, post: 0, nonlinear: nil),
        .flOzUs: KatalogUnitDef(quantity: .volume, mul: 0.0295735295625, div: 1, pre: 0, post: 0, nonlinear: nil),
        .galUk: KatalogUnitDef(quantity: .volume, mul: 4.54609, div: 1, pre: 0, post: 0, nonlinear: nil),
        .qtUk: KatalogUnitDef(quantity: .volume, mul: 1.1365225, div: 1, pre: 0, post: 0, nonlinear: nil),
        .flOzUk: KatalogUnitDef(quantity: .volume, mul: 0.0284130625, div: 1, pre: 0, post: 0, nonlinear: nil),
        .bblUs: KatalogUnitDef(quantity: .volume, mul: 117.347765304, div: 1, pre: 0, post: 0, nonlinear: nil),
        .bblUk: KatalogUnitDef(quantity: .volume, mul: 163.65924, div: 1, pre: 0, post: 0, nonlinear: nil),
        .kg: KatalogUnitDef(quantity: .mass, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .g: KatalogUnitDef(quantity: .mass, mul: 0.001, div: 1, pre: 0, post: 0, nonlinear: nil),
        .lb: KatalogUnitDef(quantity: .mass, mul: 0.45359237, div: 1, pre: 0, post: 0, nonlinear: nil),
        .oz: KatalogUnitDef(quantity: .mass, mul: 0.028349523125, div: 1, pre: 0, post: 0, nonlinear: nil),
        .c: KatalogUnitDef(quantity: .temperature, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .f: KatalogUnitDef(quantity: .temperature, mul: 5, div: 9, pre: -32, post: 0, nonlinear: nil),
        .sg: KatalogUnitDef(quantity: .gravity, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .sgX1000: KatalogUnitDef(quantity: .gravity, mul: 1, div: 1000, pre: 0, post: 0, nonlinear: nil),
        .plato: KatalogUnitDef(quantity: .gravity, mul: 1, div: 1, pre: 0, post: 0, nonlinear: "plato"),
        .brix: KatalogUnitDef(quantity: .gravity, mul: 1, div: 1, pre: 0, post: 0, nonlinear: "brix"),
        .srm: KatalogUnitDef(quantity: .color, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .ebc: KatalogUnitDef(quantity: .color, mul: 1, div: 1.97, pre: 0, post: 0, nonlinear: nil),
        .lovibond: KatalogUnitDef(quantity: .color, mul: 1.3564, div: 1, pre: 0, post: -0.76, nonlinear: nil),
        .bar: KatalogUnitDef(quantity: .pressure, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .kpa: KatalogUnitDef(quantity: .pressure, mul: 0.01, div: 1, pre: 0, post: 0, nonlinear: nil),
        .psi: KatalogUnitDef(quantity: .pressure, mul: 0.0689475729316836, div: 1, pre: 0, post: 0, nonlinear: nil),
        .min: KatalogUnitDef(quantity: .time, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .h: KatalogUnitDef(quantity: .time, mul: 60, div: 1, pre: 0, post: 0, nonlinear: nil),
        .d: KatalogUnitDef(quantity: .time, mul: 1440, div: 1, pre: 0, post: 0, nonlinear: nil),
        .m: KatalogUnitDef(quantity: .length, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .ft: KatalogUnitDef(quantity: .length, mul: 0.3048, div: 1, pre: 0, post: 0, nonlinear: nil),
        .pct: KatalogUnitDef(quantity: .percent, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .ppm: KatalogUnitDef(quantity: .concentration, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .mgL: KatalogUnitDef(quantity: .concentration, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .caco3MgL: KatalogUnitDef(quantity: .hardness, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .dh: KatalogUnitDef(quantity: .hardness, mul: 17.848, div: 1, pre: 0, post: 0, nonlinear: nil),
        .fh: KatalogUnitDef(quantity: .hardness, mul: 10.009, div: 1, pre: 0, post: 0, nonlinear: nil),
        .meqL: KatalogUnitDef(quantity: .hardness, mul: 50.04, div: 1, pre: 0, post: 0, nonlinear: nil),
        .vol: KatalogUnitDef(quantity: .co2, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .gLCo2: KatalogUnitDef(quantity: .co2, mul: 1, div: 1.9635, pre: 0, post: 0, nonlinear: nil),
        .gL: KatalogUnitDef(quantity: .dose, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .ozGalUs: KatalogUnitDef(quantity: .dose, mul: 28.349523125, div: 3.785411784, pre: 0, post: 0, nonlinear: nil),
        .ozGalUk: KatalogUnitDef(quantity: .dose, mul: 28.349523125, div: 4.54609, pre: 0, post: 0, nonlinear: nil),
        .lbBblUs: KatalogUnitDef(quantity: .dose, mul: 453.59237, div: 117.347765304, pre: 0, post: 0, nonlinear: nil),
        .lKg: KatalogUnitDef(quantity: .ratio, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .qtLb: KatalogUnitDef(quantity: .ratio, mul: 0.946352946, div: 0.45359237, pre: 0, post: 0, nonlinear: nil),
        .ibu: KatalogUnitDef(quantity: .bitterness, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
        .ph: KatalogUnitDef(quantity: .ph, mul: 1, div: 1, pre: 0, post: 0, nonlinear: nil),
    ]

    public static let fields: [KatalogField: KatalogFieldDef] = [
        .recipeBatchVolumeL: KatalogFieldDef(quantity: .volume, unit: .l, min: 0.5, max: 100000, step: 0.5, decimals: 1, required: true, verify: true, homeLimits: (min: nil, max: 200), breweryLimits: (min: 5, max: nil)),
        .recipeBoilVolumeL: KatalogFieldDef(quantity: .volume, unit: .l, min: 0.5, max: 120000, step: 0.5, decimals: 1, required: true, verify: true, homeLimits: (min: nil, max: 250), breweryLimits: (min: 5, max: nil)),
        .recipeBoilTimeMin: KatalogFieldDef(quantity: .time, unit: .min, min: 0, max: 600, step: 5, decimals: 0, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .recipeEfficiencyPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 1, max: 100, step: 1, decimals: 0, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .equipmentTrubLossL: KatalogFieldDef(quantity: .volume, unit: .l, min: 0, max: 5000, step: 0.1, decimals: 1, required: false, verify: true, homeLimits: (min: nil, max: 50), breweryLimits: nil),
        .equipmentPreTrubLossL: KatalogFieldDef(quantity: .volume, unit: .l, min: 0, max: 5000, step: 0.1, decimals: 1, required: false, verify: true, homeLimits: (min: nil, max: 50), breweryLimits: nil),
        .equipmentAltitudeM: KatalogFieldDef(quantity: .length, unit: .m, min: -500, max: 5000, step: 10, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .equipmentHopUtilPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 150, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .equipmentWhirlpoolTimeMin: KatalogFieldDef(quantity: .time, unit: .min, min: 0, max: 240, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .fermentableAmountKg: KatalogFieldDef(quantity: .mass, unit: .kg, min: 0, max: 20000, step: 0.01, decimals: 2, required: true, verify: true, homeLimits: (min: nil, max: 100), breweryLimits: nil),
        .fermentableYieldPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 100, step: 0.1, decimals: 1, required: true, verify: false, homeLimits: nil, breweryLimits: nil),
        .fermentableColorSrm: KatalogFieldDef(quantity: .color, unit: .srm, min: 0, max: 1000, step: 0.5, decimals: 1, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .fermentableLateExtractMin: KatalogFieldDef(quantity: .time, unit: .min, min: 0, max: 600, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .hopAmountKg: KatalogFieldDef(quantity: .mass, unit: .kg, min: 0, max: 100, step: 0.001, decimals: 3, required: true, verify: true, homeLimits: (min: nil, max: 5), breweryLimits: nil),
        .hopAlphaPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 40, step: 0.1, decimals: 1, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .hopTimeMin: KatalogFieldDef(quantity: .time, unit: .min, min: 0, max: 10080, step: 1, decimals: 0, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .hopWhirlpoolTempC: KatalogFieldDef(quantity: .temperature, unit: .c, min: 0, max: 100, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .yeastMinAttenuationPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 100, step: 1, decimals: 0, required: false, verify: false, homeLimits: nil, breweryLimits: nil),
        .yeastMaxAttenuationPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 100, step: 1, decimals: 0, required: false, verify: false, homeLimits: nil, breweryLimits: nil),
        .yeastToleranceAbvPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 25, step: 0.5, decimals: 1, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .yeastViabilityPct: KatalogFieldDef(quantity: .percent, unit: .pct, min: 0, max: 100, step: 1, decimals: 0, required: false, verify: false, homeLimits: nil, breweryLimits: nil),
        .mashGrainWeightKg: KatalogFieldDef(quantity: .mass, unit: .kg, min: 0, max: 20000, step: 0.01, decimals: 2, required: false, verify: true, homeLimits: (min: nil, max: 100), breweryLimits: nil),
        .mashStepTempC: KatalogFieldDef(quantity: .temperature, unit: .c, min: 0, max: 100, step: 0.5, decimals: 1, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .mashStepTimeMin: KatalogFieldDef(quantity: .time, unit: .min, min: 0, max: 600, step: 1, decimals: 0, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .mashThicknessLPerKg: KatalogFieldDef(quantity: .ratio, unit: .lKg, min: 0.5, max: 10, step: 0.1, decimals: 1, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .measurementSg: KatalogFieldDef(quantity: .gravity, unit: .sg, min: 0.98, max: 1.25, step: 0.001, decimals: 3, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .measurementOgSg: KatalogFieldDef(quantity: .gravity, unit: .sg, min: 1, max: 1.25, step: 0.001, decimals: 3, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .measurementFgSg: KatalogFieldDef(quantity: .gravity, unit: .sg, min: 0.98, max: 1.25, step: 0.001, decimals: 3, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .measurementTempC: KatalogFieldDef(quantity: .temperature, unit: .c, min: -20, max: 120, step: 0.5, decimals: 1, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .measurementPh: KatalogFieldDef(quantity: .ph, unit: .ph, min: 0, max: 14, step: 0.01, decimals: 2, required: true, verify: true, homeLimits: nil, breweryLimits: nil),
        .targetIbu: KatalogFieldDef(quantity: .bitterness, unit: .ibu, min: 0, max: 300, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .targetColorSrm: KatalogFieldDef(quantity: .color, unit: .srm, min: 0, max: 200, step: 0.5, decimals: 1, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .targetMashPh: KatalogFieldDef(quantity: .ph, unit: .ph, min: 4.5, max: 6.5, step: 0.01, decimals: 2, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .targetCo2Vol: KatalogFieldDef(quantity: .co2, unit: .vol, min: 0, max: 5, step: 0.1, decimals: 2, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .processKegPressureBar: KatalogFieldDef(quantity: .pressure, unit: .bar, min: 0, max: 6, step: 0.05, decimals: 2, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .processPrimingGPerL: KatalogFieldDef(quantity: .dose, unit: .gL, min: 0, max: 20, step: 0.1, decimals: 1, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterCalciumPpm: KatalogFieldDef(quantity: .concentration, unit: .ppm, min: 0, max: 1000, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterMagnesiumPpm: KatalogFieldDef(quantity: .concentration, unit: .ppm, min: 0, max: 500, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterSodiumPpm: KatalogFieldDef(quantity: .concentration, unit: .ppm, min: 0, max: 1000, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterChloridePpm: KatalogFieldDef(quantity: .concentration, unit: .ppm, min: 0, max: 2000, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterSulfatePpm: KatalogFieldDef(quantity: .concentration, unit: .ppm, min: 0, max: 3000, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterBicarbonatePpm: KatalogFieldDef(quantity: .concentration, unit: .ppm, min: 0, max: 1500, step: 1, decimals: 0, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .waterPh: KatalogFieldDef(quantity: .ph, unit: .ph, min: 0, max: 14, step: 0.01, decimals: 2, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .toolVolumeL: KatalogFieldDef(quantity: .volume, unit: .l, min: 0, max: 100000, step: 0.1, decimals: 1, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
        .toolMassKg: KatalogFieldDef(quantity: .mass, unit: .kg, min: 0, max: 20000, step: 0.01, decimals: 2, required: false, verify: true, homeLimits: nil, breweryLimits: nil),
    ]

    public static let trQuantities: [String: String] = [
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
        "ph": "pH",
    ]
    public static let trUnitNames: [String: String] = [
        "l": "litre",
        "ml": "mililitre",
        "hl": "hektolitre",
        "gal_us": "galon (ABD)",
        "qt_us": "quart (ABD)",
        "fl_oz_us": "sıvı ons (ABD)",
        "gal_uk": "galon (İngiltere)",
        "qt_uk": "quart (İngiltere)",
        "fl_oz_uk": "sıvı ons (İngiltere)",
        "bbl_us": "varil (ABD bira)",
        "bbl_uk": "varil (İngiltere bira)",
        "kg": "kilogram",
        "g": "gram",
        "lb": "libre",
        "oz": "ons",
        "c": "santigrat derece",
        "f": "fahrenhayt derece",
        "sg": "özgül ağırlık",
        "sg_x1000": "özgül ağırlık (1050 biçimi)",
        "plato": "Plato derecesi",
        "brix": "Brix derecesi",
        "srm": "SRM",
        "ebc": "EBC",
        "lovibond": "Lovibond derecesi",
        "bar": "bar",
        "kpa": "kilopaskal",
        "psi": "psi",
        "min": "dakika",
        "h": "saat",
        "d": "gün",
        "m": "metre",
        "ft": "fit",
        "pct": "yüzde",
        "ppm": "mg/L (ppm)",
        "mg_l": "miligram/litre",
        "caco3_mg_l": "mg/L (CaCO₃ olarak)",
        "dh": "Alman sertliği",
        "fh": "Fransız sertliği",
        "meq_l": "mEq/L",
        "vol": "hacim CO₂",
        "g_l_co2": "g/L CO₂",
        "g_l": "gram/litre",
        "oz_gal_us": "ons/galon (ABD)",
        "oz_gal_uk": "ons/galon (İngiltere)",
        "lb_bbl_us": "libre/varil (ABD)",
        "l_kg": "litre/kg",
        "qt_lb": "quart/libre",
        "ibu": "IBU",
        "ph": "pH",
    ]
    public static let trUnitSymbols: [String: String] = [
        "l": "L",
        "ml": "mL",
        "hl": "hL",
        "gal_us": "gal (ABD)",
        "qt_us": "qt (ABD)",
        "fl_oz_us": "fl oz (ABD)",
        "gal_uk": "gal (İng.)",
        "qt_uk": "qt (İng.)",
        "fl_oz_uk": "fl oz (İng.)",
        "bbl_us": "bbl (ABD)",
        "bbl_uk": "bbl (İng.)",
        "kg": "kg",
        "g": "g",
        "lb": "lb",
        "oz": "oz",
        "c": "°C",
        "f": "°F",
        "sg": "SG",
        "sg_x1000": "SG×1000",
        "plato": "°P",
        "brix": "°Bx",
        "srm": "SRM",
        "ebc": "EBC",
        "lovibond": "°L",
        "bar": "bar",
        "kpa": "kPa",
        "psi": "psi",
        "min": "dk",
        "h": "sa",
        "d": "gün",
        "m": "m",
        "ft": "ft",
        "pct": "%",
        "ppm": "ppm",
        "mg_l": "mg/L",
        "caco3_mg_l": "mg/L CaCO₃",
        "dh": "°dH",
        "fh": "°fH",
        "meq_l": "mEq/L",
        "vol": "vol",
        "g_l_co2": "g/L",
        "g_l": "g/L",
        "oz_gal_us": "oz/gal",
        "oz_gal_uk": "oz/gal (İng.)",
        "lb_bbl_us": "lb/bbl",
        "l_kg": "L/kg",
        "qt_lb": "qt/lb",
        "ibu": "IBU",
        "ph": "pH",
    ]
    public static let trFieldLabels: [String: String] = [
        "recipe.batch_volume_l": "Parti hacmi (fermentöre)",
        "recipe.boil_volume_l": "Kaynatma hacmi",
        "recipe.boil_time_min": "Kaynatma süresi",
        "recipe.efficiency_pct": "Verim",
        "equipment.trub_loss_l": "Tortu kaybı",
        "equipment.pre_trub_loss_l": "Tortu öncesi kayıp",
        "equipment.altitude_m": "Rakım",
        "equipment.hop_util_pct": "Hop kullanım oranı",
        "equipment.whirlpool_time_min": "Whirlpool süresi",
        "fermentable.amount_kg": "Malt miktarı",
        "fermentable.yield_pct": "Ekstrakt verimi",
        "fermentable.color_srm": "Malt rengi",
        "fermentable.late_extract_min": "Geç ekstrakt süresi",
        "hop.amount_kg": "Hop miktarı",
        "hop.alpha_pct": "Alfa asit",
        "hop.time_min": "Hop süresi",
        "hop.whirlpool_temp_c": "Whirlpool sıcaklığı",
        "yeast.min_attenuation_pct": "En düşük atenüasyon",
        "yeast.max_attenuation_pct": "En yüksek atenüasyon",
        "yeast.tolerance_abv_pct": "Alkol toleransı",
        "yeast.viability_pct": "Canlılık",
        "mash.grain_weight_kg": "Tahıl ağırlığı",
        "mash.step_temp_c": "Mash adım sıcaklığı",
        "mash.step_time_min": "Mash adım süresi",
        "mash.thickness_l_per_kg": "Su/tahıl oranı",
        "measurement.sg": "Gravite",
        "measurement.og_sg": "Başlangıç gravitesi (OG)",
        "measurement.fg_sg": "Bitiş gravitesi (FG)",
        "measurement.temp_c": "Sıcaklık",
        "measurement.ph": "pH",
        "target.ibu": "Hedef IBU",
        "target.color_srm": "Hedef renk",
        "target.mash_ph": "Hedef mash pH",
        "target.co2_vol": "Hedef karbonasyon",
        "process.keg_pressure_bar": "Fıçı basıncı",
        "process.priming_g_per_l": "Priming şekeri oranı",
        "water.calcium_ppm": "Kalsiyum",
        "water.magnesium_ppm": "Magnezyum",
        "water.sodium_ppm": "Sodyum",
        "water.chloride_ppm": "Klorür",
        "water.sulfate_ppm": "Sülfat",
        "water.bicarbonate_ppm": "Bikarbonat",
        "water.ph": "Su pH'ı",
        "tool.volume_l": "Hacim (araç)",
        "tool.mass_kg": "Kütle (araç)",
    ]
    public static let trErrors: [String: String] = [
        "required": "{label} gerekli.",
        "invalid_number": "{label} geçerli bir sayı olmalı.",
        "out_of_range": "{label} {min} ile {max} {unit} arasında olmalı.",
        "unknown_field": "Bilinmeyen alan: {field}.",
    ]

    public static let enQuantities: [String: String] = [
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
        "ph": "pH",
    ]
    public static let enUnitNames: [String: String] = [
        "l": "litre",
        "ml": "millilitre",
        "hl": "hectolitre",
        "gal_us": "gallon (US)",
        "qt_us": "quart (US)",
        "fl_oz_us": "fluid ounce (US)",
        "gal_uk": "gallon (UK)",
        "qt_uk": "quart (UK)",
        "fl_oz_uk": "fluid ounce (UK)",
        "bbl_us": "barrel (US beer)",
        "bbl_uk": "barrel (UK brewer's)",
        "kg": "kilogram",
        "g": "gram",
        "lb": "pound",
        "oz": "ounce",
        "c": "degree Celsius",
        "f": "degree Fahrenheit",
        "sg": "specific gravity",
        "sg_x1000": "specific gravity (1050 form)",
        "plato": "degrees Plato",
        "brix": "degrees Brix",
        "srm": "SRM",
        "ebc": "EBC",
        "lovibond": "degrees Lovibond",
        "bar": "bar",
        "kpa": "kilopascal",
        "psi": "psi",
        "min": "minute",
        "h": "hour",
        "d": "day",
        "m": "metre",
        "ft": "foot",
        "pct": "percent",
        "ppm": "mg/L (ppm)",
        "mg_l": "milligram per litre",
        "caco3_mg_l": "mg/L (as CaCO₃)",
        "dh": "German degree",
        "fh": "French degree",
        "meq_l": "mEq/L",
        "vol": "volumes CO₂",
        "g_l_co2": "g/L CO₂",
        "g_l": "gram per litre",
        "oz_gal_us": "ounce per US gallon",
        "oz_gal_uk": "ounce per UK gallon",
        "lb_bbl_us": "pound per US barrel",
        "l_kg": "litre per kilogram",
        "qt_lb": "quart per pound",
        "ibu": "IBU",
        "ph": "pH",
    ]
    public static let enUnitSymbols: [String: String] = [
        "l": "L",
        "ml": "mL",
        "hl": "hL",
        "gal_us": "gal (US)",
        "qt_us": "qt (US)",
        "fl_oz_us": "fl oz (US)",
        "gal_uk": "gal (UK)",
        "qt_uk": "qt (UK)",
        "fl_oz_uk": "fl oz (UK)",
        "bbl_us": "bbl (US)",
        "bbl_uk": "bbl (UK)",
        "kg": "kg",
        "g": "g",
        "lb": "lb",
        "oz": "oz",
        "c": "°C",
        "f": "°F",
        "sg": "SG",
        "sg_x1000": "SG×1000",
        "plato": "°P",
        "brix": "°Bx",
        "srm": "SRM",
        "ebc": "EBC",
        "lovibond": "°L",
        "bar": "bar",
        "kpa": "kPa",
        "psi": "psi",
        "min": "min",
        "h": "h",
        "d": "d",
        "m": "m",
        "ft": "ft",
        "pct": "%",
        "ppm": "ppm",
        "mg_l": "mg/L",
        "caco3_mg_l": "mg/L CaCO₃",
        "dh": "°dH",
        "fh": "°fH",
        "meq_l": "mEq/L",
        "vol": "vol",
        "g_l_co2": "g/L",
        "g_l": "g/L",
        "oz_gal_us": "oz/gal",
        "oz_gal_uk": "oz/gal (UK)",
        "lb_bbl_us": "lb/bbl",
        "l_kg": "L/kg",
        "qt_lb": "qt/lb",
        "ibu": "IBU",
        "ph": "pH",
    ]
    public static let enFieldLabels: [String: String] = [
        "recipe.batch_volume_l": "Batch volume (into fermenter)",
        "recipe.boil_volume_l": "Boil volume",
        "recipe.boil_time_min": "Boil time",
        "recipe.efficiency_pct": "Efficiency",
        "equipment.trub_loss_l": "Trub loss",
        "equipment.pre_trub_loss_l": "Pre-trub loss",
        "equipment.altitude_m": "Altitude",
        "equipment.hop_util_pct": "Hop utilization",
        "equipment.whirlpool_time_min": "Whirlpool time",
        "fermentable.amount_kg": "Fermentable amount",
        "fermentable.yield_pct": "Extract yield",
        "fermentable.color_srm": "Fermentable colour",
        "fermentable.late_extract_min": "Late extract time",
        "hop.amount_kg": "Hop amount",
        "hop.alpha_pct": "Alpha acid",
        "hop.time_min": "Hop time",
        "hop.whirlpool_temp_c": "Whirlpool temperature",
        "yeast.min_attenuation_pct": "Minimum attenuation",
        "yeast.max_attenuation_pct": "Maximum attenuation",
        "yeast.tolerance_abv_pct": "Alcohol tolerance",
        "yeast.viability_pct": "Viability",
        "mash.grain_weight_kg": "Grain weight",
        "mash.step_temp_c": "Mash step temperature",
        "mash.step_time_min": "Mash step time",
        "mash.thickness_l_per_kg": "Water-to-grain ratio",
        "measurement.sg": "Gravity",
        "measurement.og_sg": "Original gravity (OG)",
        "measurement.fg_sg": "Final gravity (FG)",
        "measurement.temp_c": "Temperature",
        "measurement.ph": "pH",
        "target.ibu": "Target IBU",
        "target.color_srm": "Target colour",
        "target.mash_ph": "Target mash pH",
        "target.co2_vol": "Target carbonation",
        "process.keg_pressure_bar": "Keg pressure",
        "process.priming_g_per_l": "Priming sugar rate",
        "water.calcium_ppm": "Calcium",
        "water.magnesium_ppm": "Magnesium",
        "water.sodium_ppm": "Sodium",
        "water.chloride_ppm": "Chloride",
        "water.sulfate_ppm": "Sulfate",
        "water.bicarbonate_ppm": "Bicarbonate",
        "water.ph": "Water pH",
        "tool.volume_l": "Volume (tool)",
        "tool.mass_kg": "Mass (tool)",
    ]
    public static let enErrors: [String: String] = [
        "required": "{label} is required.",
        "invalid_number": "{label} must be a valid number.",
        "out_of_range": "{label} must be between {min} and {max} {unit}.",
        "unknown_field": "Unknown field: {field}.",
    ]

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
    }
}
