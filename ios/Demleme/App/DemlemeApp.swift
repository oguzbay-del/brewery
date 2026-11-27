import SwiftUI

@main
struct DemlemeApp: App {
    @State private var store = DemlemeApp.makeStore()

    var body: some Scene {
        WindowGroup {
            ContentView(store: store)
        }
    }

    @MainActor
    private static func makeStore() -> ModulesStore {
        #if DEBUG
        if let spec = ProcessInfo.processInfo.environment["GRISTORA_UITEST_MODULES"] {
            return ModulesStore(service: FixtureModulesService(spec: spec))
        }
        #endif
        do {
            return ModulesStore(service: LiveModulesService(api: try APIClient()))
        } catch {
            return ModulesStore(service: UnavailableModulesService(error: error))
        }
    }
}
