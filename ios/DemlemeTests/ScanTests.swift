@testable import Demleme
import Foundation
import HTTPTypes
import OpenAPIRuntime
import Testing

private let containerJSON = """
{"id":"0192f1c0-7a3b-7000-8000-000000000101","serial_number":"KEG-0007",\
"package_type_id":"0192f1c0-7a3b-7000-8000-000000000102","package_type_name":"Keg 20 L",\
"package_kind":"keg","volume_l":20,"status":"filled",\
"created_at":"2026-11-13T10:00:00Z","updated_at":"2026-11-13T10:00:00Z","version":3}
"""

private let defaultActions = #"["view","change_status","teleport"]"#

private func resolutionJSON(kind: String = "container", actions: String = defaultActions) -> String {
    """
    {"kind":"\(kind)","id":"0192f1c0-7a3b-7000-8000-000000000101",\
    "scan_code_id":"0192f1c0-7a3b-7000-8000-000000000103",\
    "label":"KEG-0007","actions":\(actions),"container":\(containerJSON),"vessel":null,"lot":null}
    """
}

/// Answers every request with a fixed body and records path and body text.
private actor ScanTransport: ClientTransport {
    private(set) var paths: [String] = []
    private(set) var bodies: [String] = []
    private let sessionJSON = """
    {"session":{"client":"ios","created_at":"2026-11-13T10:00:00Z","csrf_token":"x",\
    "expires_at":"2026-11-13T22:00:00Z","id":"0192f1c0-7a3b-7000-8000-00000000000b","step_up_at":null},\
    "user":{"contact_email":"a@example.com","display_name":"A","id":"0192f1c0-7a3b-7000-8000-00000000000a"},\
    "workspaces":[{"id":"0192f1c0-7a3b-7000-8000-00000000000c","mode":"business","name":"W",\
    "region":"us","status":"active"}]}
    """
    private let resolution: String
    private let status: HTTPResponse.Status

    init(resolution: String, status: HTTPResponse.Status = .ok) {
        self.resolution = resolution
        self.status = status
    }

    func send(
        _ request: HTTPRequest,
        body: HTTPBody?,
        baseURL _: URL,
        operationID _: String
    ) async throws -> (HTTPResponse, HTTPBody?) {
        let path = request.path ?? ""
        paths.append(path)
        if let body {
            bodies.append(try await String(collecting: body, upTo: 4096))
        }
        var response = HTTPResponse(status: path.hasPrefix("/v1/me/session") ? .ok : status)
        response.headerFields[.contentType] = "application/json"
        return (response, HTTPBody(path.hasPrefix("/v1/me/session") ? sessionJSON : resolution))
    }
}

private func makeService(_ transport: ScanTransport) -> LiveScanService {
    let api = APIClient(serverURL: URL(string: "https://api.example.test")!, transport: transport)
    return LiveScanService(api: api, workspace: WorkspaceResolver(api: api))
}

struct ScanCardTests {
    @Test func containerCardShowsDetailsAndOnlyKnownActions() async throws {
        let transport = ScanTransport(resolution: resolutionJSON())
        let card = try await makeService(transport).resolve(code: "KEG-0007")
        #expect(card.kind == .container)
        #expect(card.label == "KEG-0007")
        #expect(card.actions == [.view, .changeStatus])
        #expect(card.details.first == ScanDetail(labelKey: "scan.detail_serial", value: "KEG-0007"))
        #expect(card.details.contains { $0.labelKey == "scan.detail_package" && $0.value == "Keg 20 L" })
    }

    @Test func requestGoesToTheWorkspaceWithTheCode() async throws {
        let transport = ScanTransport(resolution: resolutionJSON())
        _ = try await makeService(transport).resolve(code: "KEG-0007")
        let paths = await transport.paths
        #expect(paths.last == "/v1/workspaces/0192f1c0-7a3b-7000-8000-00000000000c/scan/resolve")
        let bodies = await transport.bodies
        #expect(bodies.last?.contains("KEG-0007") == true)
    }

    @Test func noActionsMeansNoActionRows() async throws {
        let transport = ScanTransport(resolution: resolutionJSON(actions: "[]"))
        let card = try await makeService(transport).resolve(code: "KEG-0007")
        #expect(card.actions.isEmpty)
    }

    @Test func unknownKindIsUnreadable() async {
        let transport = ScanTransport(resolution: resolutionJSON(kind: "spaceship"))
        await #expect(throws: ScanServiceError.unreadableResponse) {
            _ = try await makeService(transport).resolve(code: "X")
        }
    }

    @Test func writeActionsAreMarked() {
        #expect(!ScanAction.view.isWrite)
        for action in ScanAction.allCases where action != .view {
            #expect(action.isWrite)
        }
    }

    @Test func notFoundProblemBecomesApiError() async {
        let body = #"{"type":"/errors/not_found","title":"Not found","status":404,"code":"not_found"}"#
        let transport = ScanTransport(resolution: body, status: .notFound)
        await #expect(throws: APIError.self) {
            _ = try await makeService(transport).resolve(code: "X")
        }
    }

    @Test func catalogHasATextForEveryAction() {
        for action in ScanAction.allCases {
            #expect(UiText.text(action.textKey, locale: .en) != action.textKey)
            #expect(UiText.text(action.textKey, locale: .tr) != action.textKey)
        }
    }
}

private struct StubScan: ScanService {
    let result: Result<ScanCard, APIError>

    func resolve(code _: String) async throws -> ScanCard {
        try result.get()
    }
}

@MainActor
struct ScanStoreTests {
    private let card = ScanCard(kind: .lot, id: UUID(), label: "LOT-1", details: [], actions: [.view])

    @Test func emptyCodeIsIgnored() async {
        let store = ScanStore(service: StubScan(result: .success(card)))
        await store.resolve("   ")
        #expect(store.phase == .idle)
    }

    @Test func resolvedCodeShowsTheCard() async {
        let store = ScanStore(service: StubScan(result: .success(card)))
        await store.resolve(" LOT-1 \n")
        #expect(store.phase == .card(card))
        store.reset()
        #expect(store.phase == .idle)
    }

    @Test func errorsMapToFailures() async {
        let missing = ScanStore(service: StubScan(result: .failure(.unexpected(status: 404))))
        await missing.resolve("X")
        #expect(missing.phase == .failed(.notFound))
        let denied = ScanStore(service: StubScan(result: .failure(.unexpected(status: 403))))
        await denied.resolve("X")
        #expect(denied.phase == .failed(.forbidden))
        let broken = ScanStore(service: StubScan(result: .failure(.unexpected(status: 500))))
        await broken.resolve("X")
        #expect(broken.phase == .failed(.generic))
    }
}
