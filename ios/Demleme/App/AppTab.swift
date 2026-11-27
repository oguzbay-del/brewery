import Foundation

/// Main navigation. The module ids match `ROUTE_MODULES` in `web/src/shell/navigation.ts`.
enum AppTab: String, CaseIterable, Identifiable, Sendable {
    case home
    case recipes
    case batches
    case containers
    case settings

    var id: String { rawValue }

    /// Module the screen needs. `nil` means no module (Home, and Settings where modules are shown).
    var moduleID: String? {
        switch self {
        case .home, .settings: nil
        case .recipes, .batches: "core"
        case .containers: "production"
        }
    }

    var titleKey: String { "nav.\(rawValue)" }

    var systemImage: String {
        switch self {
        case .home: "house"
        case .recipes: "book"
        case .batches: "drop"
        case .containers: "shippingbox"
        case .settings: "gearshape"
        }
    }
}
