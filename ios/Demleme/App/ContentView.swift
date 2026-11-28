import SwiftUI

struct ContentView: View {
    let store: ModulesStore
    let scanStore: ScanStore
    let batchStore: BatchScanStore
    // Kept in scene storage, not in a layout-dependent view: a size or orientation change keeps them.
    @SceneStorage("app.selectedTab") private var tabRaw = AppTab.home.rawValue
    @SceneStorage("app.selectedModule") private var selectedModule: String?

    private var selection: Binding<AppTab> {
        Binding(
            get: { AppTab(rawValue: tabRaw) ?? .home },
            set: { tabRaw = $0.rawValue }
        )
    }

    /// Tabs whose module is off are not shown. While the state loads or when it cannot be
    /// loaded, nothing is hidden: the server still enforces the module state.
    private var visibleTabs: [AppTab] {
        AppTab.allCases.filter { store.access(for: $0.moduleID) != .off }
    }

    var body: some View {
        TabView(selection: selection) {
            ForEach(visibleTabs) { tab in
                Tab(UiText.text(tab.titleKey), systemImage: tab.systemImage, value: tab) {
                    tabContent(for: tab)
                }
            }
        }
        .tabViewStyle(.sidebarAdaptable)
        .tint(DemlemeColor.tint)
        .task { await store.load() }
        .onChange(of: visibleTabs) { _, tabs in
            if !tabs.contains(selection.wrappedValue) { selection.wrappedValue = .home }
        }
    }

    /// Settings owns a `NavigationSplitView`, so it is not wrapped in a `NavigationStack`.
    @ViewBuilder
    private func tabContent(for tab: AppTab) -> some View {
        if tab == .settings {
            screen(for: tab)
        } else {
            NavigationStack {
                screen(for: tab)
                    .navigationTitle(UiText.text(tab.titleKey))
            }
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
                ScanView(store: scanStore, batch: batchStore, modules: store)
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
                    ModulesSplitView(store: store, selection: $selectedModule)
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
        scanStore: ScanStore(service: UnavailableScanService(error: error)),
        batchStore: BatchScanStore(service: UnavailableScanService(error: error))
    )
}
