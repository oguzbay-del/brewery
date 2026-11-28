import SwiftUI

@main
struct DemlemeApp: App {
    @State private var store: ModulesStore
    @State private var scanStore: ScanStore
    @State private var batchStore: BatchScanStore

    init() {
        let (modules, scan) = DemlemeApp.makeServices()
        _store = State(initialValue: ModulesStore(service: modules))
        _scanStore = State(initialValue: ScanStore(service: scan))
        _batchStore = State(initialValue: BatchScanStore(service: scan))
    }

    var body: some Scene {
        WindowGroup {
            #if DEBUG
            if ProcessInfo.processInfo.arguments.contains("-KitGallery") {
                KitGallery()
            } else {
                ContentView(store: store, scanStore: scanStore, batchStore: batchStore)
            }
            #else
            ContentView(store: store, scanStore: scanStore, batchStore: batchStore)
            #endif
        }
    }

    private static func makeServices() -> (any ModulesService, any ScanService) {
        #if DEBUG
        let environment = ProcessInfo.processInfo.environment
        if let spec = environment["GRISTORA_UITEST_MODULES"] {
            let granted = environment["GRISTORA_UITEST_PERMISSIONS"] ?? "container.status,container.manage,field.view"
            let permissions = Set(granted.split(separator: ",").map(String.init))
            return (FixtureModulesService(spec: spec), FixtureScanService(granted: permissions))
        }
        #endif
        do {
            let api = try APIClient()
            let workspace = WorkspaceResolver(api: api)
            return (
                LiveModulesService(api: api, workspace: workspace),
                LiveScanService(api: api, workspace: workspace)
            )
        } catch {
            return (UnavailableModulesService(error: error), UnavailableScanService(error: error))
        }
    }
}
