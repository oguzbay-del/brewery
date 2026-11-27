@testable import Demleme
import Foundation
import HTTPTypes
import OpenAPIRuntime
import Testing

private let modules = [
    ModuleSnapshot(id: "core", enabled: true, readOnly: false),
    ModuleSnapshot(id: "stock", enabled: true, readOnly: true),
    ModuleSnapshot(id: "production", enabled: false, readOnly: false),
]

private struct StubService: ModulesService {
    let result: Result<[ModuleSnapshot]?, APIError>

    func loadModules() async throws -> [ModuleSnapshot]? {
        try result.get()
    }
}

struct ModuleAccessTests {
    @Test func resolveMapsEnabledAndReadOnly() {
        #expect(ModuleAccess.resolve("core", in: modules) == .on)
        #expect(ModuleAccess.resolve("stock", in: modules) == .readOnly)
        #expect(ModuleAccess.resolve("production", in: modules) == .off)
    }

    @Test func unlistedModuleIsOff() {
        #expect(ModuleAccess.resolve("quality", in: modules) == .off)
    }

    @Test func onlyReadOnlyAndOffBlockWrites() {
        #expect(ModuleAccess.readOnly.blocksWrites)
        #expect(ModuleAccess.off.blocksWrites)
        #expect(!ModuleAccess.on.blocksWrites)
        #expect(!ModuleAccess.unknown.blocksWrites)
        #expect(!ModuleAccess.loading.blocksWrites)
    }

    @Test func textsComeFromTheCatalog() {
        #expect(UiText.text("modules.upgrade_title", ["module": "Stok"], locale: .tr) == "Stok modülü kapalı")
        #expect(UiText.text("modules.upgrade_title", ["module": "Stock"], locale: .en) == "The Stock module is off")
        #expect(UiText.moduleName("production", locale: .en) == "Production")
        #expect(UiText.moduleName("no_such_module", locale: .en) == "no_such_module")
    }
}

@MainActor
struct ModulesStoreTests {
    @Test func accessIsLoadingBeforeTheFirstLoad() {
        let store = ModulesStore(service: StubService(result: .success(modules)))
        #expect(store.access(for: "core") == .loading)
        #expect(store.access(for: nil) == .on)
    }

    @Test func loadedStateDrivesAccess() async {
        let store = ModulesStore(service: StubService(result: .success(modules)))
        await store.load()
        #expect(store.access(for: "core") == .on)
        #expect(store.access(for: "stock") == .readOnly)
        #expect(store.access(for: "production") == .off)
    }

    @Test func failedLoadHidesNothing() async {
        let store = ModulesStore(service: StubService(result: .failure(.unexpected(status: 500))))
        await store.load()
        #expect(store.phase == .failed(requestID: nil))
        #expect(store.access(for: "production") == .unknown)
    }

    @Test func unauthorizedAndNotMemberHaveOwnPhases() async {
        let unauthorized = ModulesStore(service: StubService(result: .failure(.unexpected(status: 401))))
        await unauthorized.load()
        #expect(unauthorized.phase == .signInRequired)
        let notMember = ModulesStore(service: StubService(result: .failure(.unexpected(status: 404))))
        await notMember.load()
        #expect(notMember.phase == .notMember)
    }

    @Test func noWorkspaceHasItsOwnPhase() async {
        let store = ModulesStore(service: StubService(result: .success(nil)))
        await store.load()
        #expect(store.phase == .noWorkspace)
    }

    @Test func moduleProblemsBecomeNotices() {
        let store = ModulesStore(service: StubService(result: .success(modules)))
        store.noteWriteError(APIError.problem(code: "module_disabled", status: 403, requestId: nil, title: nil))
        #expect(store.notice == .disabled)
        store.noteWriteError(APIError.problem(code: "module_read_only", status: 403, requestId: nil, title: nil))
        #expect(store.notice == .readOnly)
        store.dismissNotice()
        #expect(store.notice == nil)
        store.noteWriteError(APIError.problem(code: "other", status: 403, requestId: nil, title: nil))
        #expect(store.notice == nil)
    }

    @Test func fixtureServiceParsesSpec() async throws {
        let result = try await FixtureModulesService(spec: "core:on,stock:ro,production:off").loadModules()
        #expect(result == modules)
    }
}

/// Answers by path: the session first, then the module list.
private actor RoutingTransport: ClientTransport {
    private(set) var paths: [String] = []
    private let workspaceID = "0192f1c0-7a3b-7000-8000-00000000000c"

    func send(
        _ request: HTTPRequest,
        body _: HTTPBody?,
        baseURL _: URL,
        operationID _: String
    ) async throws -> (HTTPResponse, HTTPBody?) {
        let path = request.path ?? ""
        paths.append(path)
        var response = HTTPResponse(status: .ok)
        response.headerFields[.contentType] = "application/json"
        if path.hasPrefix("/v1/me/session") {
            let json = """
            {"session":{"client":"ios","created_at":"2026-11-13T10:00:00Z","csrf_token":"x",\
            "expires_at":"2026-11-13T22:00:00Z","id":"0192f1c0-7a3b-7000-8000-00000000000b","step_up_at":null},\
            "user":{"contact_email":"a@example.com","display_name":"A","id":"0192f1c0-7a3b-7000-8000-00000000000a"},\
            "workspaces":[{"id":"\(workspaceID)","mode":"home","name":"W","region":"us","status":"active",\
            "permissions":["container.status","field.view"]}]}
            """
            return (response, HTTPBody(json))
        }
        let json = """
        {"modules":[{"enabled":true,"id":"core","read_only":false},\
        {"enabled":true,"id":"stock","read_only":true},{"enabled":false,"id":"production","read_only":false}]}
        """
        return (response, HTTPBody(json))
    }
}

struct LiveModulesServiceTests {
    @Test func loadsSessionThenModulesForTheWorkspace() async throws {
        let transport = RoutingTransport()
        let api = APIClient(serverURL: URL(string: "https://api.example.test")!, transport: transport)
        let result = try await LiveModulesService(api: api).loadModules()
        #expect(result == modules)
        let paths = await transport.paths
        #expect(paths.count == 2)
        #expect(paths.last == "/v1/meta?workspace_id=0192f1c0-7a3b-7000-8000-00000000000c")
    }
}
