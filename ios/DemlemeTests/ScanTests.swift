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
    "region":"us","status":"active","permissions":["container.status","field.view"]}]}
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
        do {
            _ = try await makeService(transport).resolve(code: "X")
            Issue.record("expected an error")
        } catch {
            #expect(APIError.unwrap(error)?.status == 404)
            #expect(APIError.unwrap(error)?.code == "not_found")
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
    var statusError: APIError?
    var granted: Set<String> = ["container.status", "container.manage"]

    func resolve(code _: String) async throws -> ScanCard {
        try result.get()
    }

    func setStatus(code _: String, status _: String, note _: String?) async throws {
        if let statusError { throw statusError }
    }

    func setStatusBatch(codes: [String], status _: String, note _: String?) async throws -> BatchOutcome {
        if let statusError { throw statusError }
        let items = codes.enumerated().map { BatchItem(index: $0, code: $1, ok: true, unchanged: false, error: nil) }
        return BatchOutcome(changed: items.count, failed: 0, items: items)
    }

    func permissions() async throws -> Set<String> {
        granted
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

private let batchJSON = """
{"changed":1,"failed":2,"results":[{"index":0,"ok":true,"unchanged":false,"container":null,"error":null},\
{"index":1,"ok":false,"unchanged":false,"container":null,"error":"not_found"},\
{"index":2,"ok":false,"unchanged":false,"container":null,"error":"container_retired"}]}
"""

struct ScanActionTests {
    @Test func batchResultHasOneRowForEachCode() async throws {
        let transport = ScanTransport(resolution: batchJSON)
        let service = makeService(transport)
        let outcome = try await service.setStatusBatch(codes: ["A", "B", "C"], status: "clean", note: nil)
        #expect(outcome.changed == 1)
        #expect(outcome.failed == 2)
        #expect(outcome.items.map(\.code) == ["A", "B", "C"])
        #expect(outcome.items.map(BatchScanStore.resultKey) == [
            "scan.batch_result_ok", "scan.batch_error_not_found", "scan.batch_error_container_retired",
        ])
        let paths = await transport.paths
        #expect(paths.last == "/v1/workspaces/0192f1c0-7a3b-7000-8000-00000000000c/scan/status-batch")
        let bodies = await transport.bodies
        #expect(bodies.last?.contains("\"clean\"") == true)
    }

    @Test func unknownBatchErrorHasAGenericText() {
        let item = BatchItem(index: 0, code: "X", ok: false, unchanged: false, error: "something_new")
        #expect(BatchScanStore.resultKey(item) == "scan.batch_error_other")
        let same = BatchItem(index: 0, code: "X", ok: true, unchanged: true, error: nil)
        #expect(BatchScanStore.resultKey(same) == "scan.batch_result_unchanged")
    }

    @Test func permissionsComeFromTheSession() async throws {
        let transport = ScanTransport(resolution: "{}")
        let permissions = try await makeService(transport).permissions()
        #expect(permissions == ["container.status", "field.view"])
    }

    @Test func catalogHasEveryBatchAndStatusText() {
        let keys = ["scan.batch_result_ok", "scan.batch_result_unchanged", "scan.batch_error_not_found",
                    "scan.batch_error_not_a_container", "scan.batch_error_container_retired",
                    "scan.batch_error_other", "scan.action_conflict"]
            + ContainerStatusChoice.allCases.map(\.textKey)
        for key in keys {
            #expect(UiText.text(key, locale: .en) != key)
            #expect(UiText.text(key, locale: .tr) != key)
        }
    }
}

@MainActor
struct ScanActionStoreTests {
    private let card = ScanCard(kind: .container, id: UUID(), label: "K", details: [], actions: [.changeStatus, .fill])

    @Test func writeActionsNeedContainerStatusPermission() async {
        let store = ScanStore(service: StubScan(result: .success(card), granted: ["field.view"]))
        await store.loadPermissions()
        #expect(!store.isOffered(.changeStatus))
        #expect(!store.canBatch)
        let statusOnly = ScanStore(service: StubScan(result: .success(card), granted: ["container.status"]))
        await statusOnly.loadPermissions()
        #expect(statusOnly.isOffered(.changeStatus))
        #expect(!statusOnly.isOffered(.retire))
        let allowed = ScanStore(service: StubScan(result: .success(card)))
        await allowed.loadPermissions()
        #expect(allowed.isOffered(.changeStatus))
        #expect(allowed.isOffered(.retire))
        #expect(!allowed.isOffered(.fill))
        #expect(allowed.canBatch)
    }

    @Test func statusChangeReadsTheCardAgain() async {
        let store = ScanStore(service: StubScan(result: .success(card)))
        await store.resolve("K")
        await store.setStatus("clean", note: "  ")
        #expect(store.actionState == .done)
        #expect(store.phase == .card(card))
    }

    @Test func statusErrorsMapToFailures() async {
        var service = StubScan(result: .success(card))
        service.statusError = .unexpected(status: 409)
        let store = ScanStore(service: service)
        await store.resolve("K")
        var noticed = false
        await store.setStatus("out", note: nil) { _ in noticed = true }
        #expect(store.actionState == .failed(.conflict))
        #expect(noticed)
        #expect(ScanStore.actionFailure(APIError.unexpected(status: 403)) == .forbidden)
        #expect(ScanStore.actionFailure(APIError.unexpected(status: 500)) == .generic)
    }

    @Test func batchAddsEachCodeOnceAndSubmits() async {
        let batch = BatchScanStore(service: StubScan(result: .success(card)))
        batch.add("KEG-1")
        batch.add(" KEG-1 ")
        batch.add("")
        batch.manualCode = "KEG-2"
        batch.addManualCode()
        #expect(batch.codes == ["KEG-1", "KEG-2"])
        await batch.submit()
        #expect(batch.outcome?.changed == 2)
        batch.remove("KEG-1")
        #expect(batch.codes == ["KEG-2"])
        #expect(batch.outcome == nil)
    }

    @Test func emptyNoteIsNotSent() {
        #expect(ScanStore.clean("  \n") == nil)
        #expect(ScanStore.clean(" late ") == "late")
    }
}
