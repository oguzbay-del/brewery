import SwiftUI

@main
struct DemlemeApp: App {
    @State private var store: ModulesStore
    @State private var scanStore: ScanStore

    init() {
        let (modules, scan) = DemlemeApp.makeStores()
        _store = State(initialValue: modules)
        _scanStore = State(initialValue: scan)
    }

    var body: some Scene {
        WindowGroup {
            ContentView(store: store, scanStore: scanStore)
        }
    }

    @MainActor
    private static func makeStores() -> (ModulesStore, ScanStore) {
        #if DEBUG
        if let spec = ProcessInfo.processInfo.environment["GRISTORA_UITEST_MODULES"] {
            return (
                ModulesStore(service: FixtureModulesService(spec: spec)),
                ScanStore(service: FixtureScanService())
            )
        }
        #endif
        do {
            let api = try APIClient()
            let workspace = WorkspaceResolver(api: api)
            return (
                ModulesStore(service: LiveModulesService(api: api, workspace: workspace)),
                ScanStore(service: LiveScanService(api: api, workspace: workspace))
            )
        } catch {
            return (
                ModulesStore(service: UnavailableModulesService(error: error)),
                ScanStore(service: UnavailableScanService(error: error))
            )
        }
    }
}
