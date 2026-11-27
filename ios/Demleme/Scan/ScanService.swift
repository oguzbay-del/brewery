import Foundation

/// Resolves a scanned code to a card.
protocol ScanService: Sendable {
    func resolve(code: String) async throws -> ScanCard
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
}

/// Used when the app has no valid API configuration.
struct UnavailableScanService: ScanService {
    let error: any Error

    func resolve(code _: String) async throws -> ScanCard {
        throw error
    }
}

#if DEBUG
/// Fixed cards for UI tests: `KEG-1` (view, change status), `TANK-1` (view), `LOT-1` (no action).
struct FixtureScanService: ScanService {
    func resolve(code: String) async throws -> ScanCard {
        switch code {
        case "KEG-1":
            ScanCard(
                kind: .container,
                id: UUID(),
                label: "KEG-1",
                details: [ScanDetail(labelKey: "scan.detail_serial", value: "KEG-1")],
                actions: [.view, .changeStatus]
            )
        case "TANK-1":
            ScanCard(kind: .vessel, id: UUID(), label: "TANK-1", details: [], actions: [.view])
        case "LOT-1":
            ScanCard(kind: .lot, id: UUID(), label: "LOT-1", details: [], actions: [])
        default:
            throw APIError.problem(code: "not_found", status: 404, requestId: nil, title: nil)
        }
    }
}
#endif
