import Foundation

extension KatalogLocale {
    /// Catalog language for the device: Turkish for a Turkish device, English otherwise.
    static var device: KatalogLocale {
        Locale.preferredLanguages.first?.lowercased().hasPrefix("tr") == true ? .tr : .en
    }
}

/// User-visible text from the text catalog (`sozlesme/katalog/katalog.json`, group `ui`).
enum UiText {
    /// Text for `key` (for example `nav.home`). `{name}` placeholders are replaced from `values`.
    /// An unknown key returns the key, so a missing text is easy to see.
    static func text(
        _ key: String,
        _ values: [String: String] = [:],
        locale: KatalogLocale = .device,
        fallback: String? = nil
    ) -> String {
        let table = locale == .tr ? Katalog.trUi : Katalog.enUi
        var result = table[key] ?? fallback ?? key
        for (name, value) in values {
            result = result.replacingOccurrences(of: "{\(name)}", with: value)
        }
        return result
    }

    /// Display name of a module id (`module.<id>`); falls back to the id.
    static func moduleName(_ moduleID: String, locale: KatalogLocale = .device) -> String {
        let key = "module.\(moduleID)"
        let name = text(key, locale: locale)
        return name == key ? moduleID : name
    }

    /// Symbol of a catalog unit code (for example `l` gives `L`). Unknown codes return the code.
    static func unitSymbol(_ unitCode: String, locale: KatalogLocale = .device) -> String {
        let table = locale == .tr ? Katalog.trUnitSymbols : Katalog.enUnitSymbols
        return table[unitCode] ?? unitCode
    }

    /// A number as text in the device format. This is display only; no calculation.
    static func number(_ value: Double, locale: KatalogLocale = .device) -> String {
        value.formatted(.number.locale(Locale(identifier: locale.rawValue)))
    }

    /// A value with the symbol of its catalog unit, as the server sent it (no conversion).
    static func quantity(_ value: Double, unitCode: String, locale: KatalogLocale = .device) -> String {
        "\(number(value, locale: locale)) \(unitSymbol(unitCode, locale: locale))"
    }
}
