import SwiftUI

struct ContentView: View {
    let store: ModulesStore
    let scanStore: ScanStore
    @State private var selection: AppTab = .home

    /// Tabs whose module is off are not shown. While the state loads or when it cannot be
    /// loaded, nothing is hidden: the server still enforces the module state.
    private var visibleTabs: [AppTab] {
        AppTab.allCases.filter { store.access(for: $0.moduleID) != .off }
    }

    var body: some View {
        TabView(selection: $selection) {
            ForEach(visibleTabs) { tab in
                Tab(UiText.text(tab.titleKey), systemImage: tab.systemImage, value: tab) {
                    NavigationStack {
                        screen(for: tab)
                            .navigationTitle(UiText.text(tab.titleKey))
                    }
                }
            }
        }
        .tint(DemlemeColor.tint)
        .task { await store.load() }
        .onChange(of: visibleTabs) { _, tabs in
            if !tabs.contains(selection) { selection = .home }
        }
    }

    /// Home: the app name and the scan screen. Scanning reads data only, so a read-only
    /// module needs no banner here; the screen is hidden only when the module is off.
    private var home: some View {
        VStack(spacing: 0) {
            Text(UiText.text("shell.app_name"))
                .font(.largeTitle.bold())
                .foregroundStyle(DemlemeColor.label)
                .accessibilityIdentifier("app.title")
            if store.access(for: "production") != .off {
                ScanView(store: scanStore)
            }
        }
    }

    @ViewBuilder
    private func screen(for tab: AppTab) -> some View {
        VStack(spacing: 0) {
            ModuleNoticeView(store: store)
            ModuleGate(moduleID: tab.moduleID, store: store) {
                switch tab {
                case .home:
                    home
                case .settings:
                    ModulesListView(store: store)
                case .containers:
                    ModuleWriteButton(
                        moduleID: "production",
                        store: store,
                        title: UiText.text("containers.create_title"),
                        action: {}
                    )
                case .recipes, .batches:
                    Color.clear
                }
            }
            .frame(maxHeight: .infinity)
        }
        .background(DemlemeColor.bg.ignoresSafeArea())
    }
}

#Preview {
    let error = APIConfiguration.ConfigurationError.missingBaseURL
    return ContentView(
        store: ModulesStore(service: UnavailableModulesService(error: error)),
        scanStore: ScanStore(service: UnavailableScanService(error: error))
    )
}
