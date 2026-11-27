import Foundation

/// The current workspace and the caller's workspace-wide permission codes.
struct WorkspaceContext: Sendable, Equatable {
    let id: String
    let permissions: Set<String>
}

/// Finds the current workspace: the first workspace of `GET /v1/me/session` (same as the web).
/// The result is kept after the first success.
actor WorkspaceResolver {
    private let api: APIClient
    private var cached: WorkspaceContext?

    init(api: APIClient) {
        self.api = api
    }

    /// Returns `nil` when the user has no workspace yet.
    func context() async throws -> WorkspaceContext? {
        if let cached { return cached }
        let session = try await api.client.me_session_get(.init())
        guard case let .ok(ok) = session else { throw APIError.unexpected(status: 0) }
        guard let workspace = try ok.body.json.workspaces.first else { return nil }
        let context = WorkspaceContext(id: workspace.id, permissions: Set(workspace.permissions))
        cached = context
        return context
    }

    /// Returns `nil` when the user has no workspace yet.
    func workspaceID() async throws -> String? {
        try await context()?.id
    }
}
