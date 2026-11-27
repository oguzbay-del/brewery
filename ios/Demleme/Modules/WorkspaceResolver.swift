import Foundation

/// Finds the current workspace: the first workspace of `GET /v1/me/session` (same as the web).
/// The id is kept after the first success.
actor WorkspaceResolver {
    private let api: APIClient
    private var cached: String?

    init(api: APIClient) {
        self.api = api
    }

    /// Returns `nil` when the user has no workspace yet.
    func workspaceID() async throws -> String? {
        if let cached { return cached }
        let session = try await api.client.me_session_get(.init())
        guard case let .ok(ok) = session else { throw APIError.unexpected(status: 0) }
        let id = try ok.body.json.workspaces.first?.id
        cached = id
        return id
    }
}
