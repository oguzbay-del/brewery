import Foundation

/// Resolves a scanned code to a card and changes the status of containers.
protocol ScanService: Sendable {
    func resolve(code: String) async throws -> ScanCard
    /// `POST .../scan/status`: one container gets a new status (`clean`, `out`, `dirty`,
    /// `maintenance` or `retired`).
    func setStatus(code: String, status: String, note: String?) async throws
    /// `POST .../scan/status-batch`: one result for each code; some codes can fail.
    func setStatusBatch(codes: [String], status: String, note: String?) async throws -> BatchOutcome
    /// Workspace-wide permission codes of the caller (`GET /v1/me/session`).
    func permissions() async throws -> Set<String>
}

/// Result of one code in a batch scan.
struct BatchItem: Sendable, Equatable, Identifiable {
    let index: Int
    let code: String
    let ok: Bool
    let unchanged: Bool
    /// `not_found`, `not_a_container`, `container_retired`, or another code from the server.
    let error: String?

    var id: Int { index }
}

/// Result of a batch scan, as the server counts it.
struct BatchOutcome: Sendable, Equatable {
    let changed: Int
    let failed: Int
    let items: [BatchItem]
}

/// Problems the scan service can have that are not an `APIError`.
enum ScanServiceError: Error, Sendable, Equatable {
    case noWorkspace
    case unreadableResponse
}

/// `POST /v1/workspaces/{workspace_id}/scan/resolve`.
struct LiveScanService: ScanService {
    let api: APIClient
    let workspace: WorkspaceResolver

    func resolve(code: String) async throws -> ScanCard {
        guard let workspaceID = try await workspace.workspaceID() else { throw ScanServiceError.noWorkspace }
        let capture = APIResponseCapture()
        let output = try await APIClient.capturing(capture) {
            try await api.client.scan_resolve(
                .init(path: .init(workspace_id: workspaceID), body: .json(.init(code: code)))
            )
        }
        guard case .ok = output else { throw APIError.unexpected(status: 0) }

        // The card is read from the raw JSON: the generated type drops the nullable objects
        // `container`, `vessel` and `lot`.
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        guard let data = capture.data,
              let dto = try? decoder.decode(ScanResolutionDTO.self, from: data),
              let card = ScanCard(dto)
        else {
            throw ScanServiceError.unreadableResponse
        }
        return card
    }

    func setStatus(code: String, status: String, note: String?) async throws {
        guard let workspaceID = try await workspace.workspaceID() else { throw ScanServiceError.noWorkspace }
        let output = try await api.client.scan_status(
            .init(path: .init(workspace_id: workspaceID), body: .json(.init(code: code, note: note, status: status)))
        )
        guard case .ok = output else { throw APIError.unexpected(status: 0) }
    }

    func setStatusBatch(codes: [String], status: String, note: String?) async throws -> BatchOutcome {
        guard let workspaceID = try await workspace.workspaceID() else { throw ScanServiceError.noWorkspace }
        let output = try await api.client.scan_status_batch(
            .init(path: .init(workspace_id: workspaceID), body: .json(.init(codes: codes, note: note, status: status)))
        )
        guard case let .ok(ok) = output else { throw APIError.unexpected(status: 0) }
        let result = try ok.body.json
        let items = result.results.map { item in
            BatchItem(
                index: item.index,
                code: codes.indices.contains(item.index) ? codes[item.index] : "",
                ok: item.ok,
                unchanged: item.unchanged,
                error: item.error
            )
        }
        return BatchOutcome(changed: result.changed, failed: result.failed, items: items)
    }

    func permissions() async throws -> Set<String> {
        try await workspace.context()?.permissions ?? []
    }
}

/// Used when the app has no valid API configuration.
struct UnavailableScanService: ScanService {
    let error: any Error

    func resolve(code _: String) async throws -> ScanCard {
        throw error
    }

    func setStatus(code _: String, status _: String, note _: String?) async throws {
        throw error
    }

    func setStatusBatch(codes _: [String], status _: String, note _: String?) async throws -> BatchOutcome {
        throw error
    }

    func permissions() async throws -> Set<String> {
        throw error
    }
}

#if DEBUG
/// Fixed cards for UI tests: `KEG-1` (view, change status, retire), `TANK-1` (view), `LOT-1` (no action).
/// In a batch, codes that start with `KEG` change; other codes fail with `not_found`.
struct FixtureScanService: ScanService {
    /// Permission codes; UI tests set them with `GRISTORA_UITEST_PERMISSIONS` (comma separated).
    let granted: Set<String>

    func resolve(code: String) async throws -> ScanCard {
        switch code {
        case "KEG-1":
            ScanCard(
                kind: .container,
                id: UUID(),
                label: "KEG-1",
                details: [ScanDetail(labelKey: "scan.detail_serial", value: "KEG-1")],
                actions: [.view, .changeStatus, .retire]
            )
        case "TANK-1":
            ScanCard(kind: .vessel, id: UUID(), label: "TANK-1", details: [], actions: [.view])
        case "LOT-1":
            ScanCard(kind: .lot, id: UUID(), label: "LOT-1", details: [], actions: [])
        default:
            throw APIError.problem(code: "not_found", status: 404, requestId: nil, title: nil)
        }
    }

    func setStatus(code _: String, status _: String, note _: String?) async throws {}

    func setStatusBatch(codes: [String], status _: String, note _: String?) async throws -> BatchOutcome {
        let items = codes.enumerated().map { index, code in
            let known = code.hasPrefix("KEG")
            return BatchItem(index: index, code: code, ok: known, unchanged: false, error: known ? nil : "not_found")
        }
        return BatchOutcome(
            changed: items.filter(\.ok).count,
            failed: items.filter { !$0.ok }.count,
            items: items
        )
    }

    func permissions() async throws -> Set<String> {
        granted
    }
}
#endif
