import Foundation

/// State of one product module for a workspace, as `GET /v1/meta` returns it.
struct ModuleSnapshot: Sendable, Equatable {
    let id: String
    let enabled: Bool
    let readOnly: Bool
}

/// What the interface does with a module. Same meaning as `ModuleAccess` in `web/src/modules`.
/// `unknown` means the state could not be loaded; the server still enforces the module state,
/// so the interface hides nothing for `unknown`.
enum ModuleAccess: Sendable, Equatable {
    case on
    case readOnly
    case off
    case loading
    case unknown

    /// A write action is blocked when the module is read only or off.
    var blocksWrites: Bool { self == .readOnly || self == .off }

    /// Access state of `moduleID` in a loaded list. A module the API does not list is off.
    static func resolve(_ moduleID: String, in modules: [ModuleSnapshot]) -> ModuleAccess {
        guard let module = modules.first(where: { $0.id == moduleID }), module.enabled else { return .off }
        return module.readOnly ? .readOnly : .on
    }
}
