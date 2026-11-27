import Foundation

/// Loads the module state of the current workspace.
protocol ModulesService: Sendable {
    /// Returns `nil` when the user has no workspace yet.
    func loadModules() async throws -> [ModuleSnapshot]?
}

/// Reads `GET /v1/me/session` for the workspace, then `GET /v1/meta?workspace_id=` (same as the web).
struct LiveModulesService: ModulesService {
    let api: APIClient

    func loadModules() async throws -> [ModuleSnapshot]? {
        let session = try await api.client.me_session_get(.init())
        guard case let .ok(sessionOK) = session else { throw APIError.unexpected(status: 0) }
        let info = try sessionOK.body.json
        guard let workspace = info.workspaces.first else { return nil }

        let meta = try await api.client.meta_modules_get(.init(query: .init(workspace_id: workspace.id)))
        guard case let .ok(metaOK) = meta else { throw APIError.unexpected(status: 0) }
        return try metaOK.body.json.modules.map {
            ModuleSnapshot(id: $0.id, enabled: $0.enabled, readOnly: $0.read_only)
        }
    }
}

/// Used when the app has no valid API configuration.
struct UnavailableModulesService: ModulesService {
    let error: any Error

    func loadModules() async throws -> [ModuleSnapshot]? {
        throw error
    }
}

#if DEBUG
/// Fixed module state for UI tests: `GRISTORA_UITEST_MODULES="core:on,production:off,quality:ro"`.
struct FixtureModulesService: ModulesService {
    let spec: String

    func loadModules() async throws -> [ModuleSnapshot]? {
        spec.split(separator: ",").compactMap { entry in
            let parts = entry.split(separator: ":").map(String.init)
            guard parts.count == 2 else { return nil }
            return ModuleSnapshot(id: parts[0], enabled: parts[1] != "off", readOnly: parts[1] == "ro")
        }
    }
}
#endif
