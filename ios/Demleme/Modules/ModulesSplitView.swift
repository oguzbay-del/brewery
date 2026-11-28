import SwiftUI

/// Settings on the iPad: module list, module detail and a helper panel (inspector).
/// On the iPhone (compact width) the split view collapses to a stack.
struct ModulesSplitView: View {
    let store: ModulesStore
    @Binding var selection: String?
    @SceneStorage("modules.inspector") private var showInspector = false

    var body: some View {
        NavigationSplitView {
            ModulesListView(store: store, selection: $selection)
        } detail: {
            detail
                .navigationTitle(UiText.text("modules.title"))
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .primaryAction) {
                        Button {
                            showInspector.toggle()
                        } label: {
                            Label(UiText.text("modules.title"), systemImage: "sidebar.trailing")
                        }
                        .disabled(selectedModule == nil)
                        .accessibilityIdentifier("module.inspector_toggle")
                    }
                }
                .inspector(isPresented: $showInspector) {
                    inspector
                        .inspectorColumnWidth(
                            min: DemlemeMetric.inspectorMin,
                            ideal: DemlemeMetric.inspectorIdeal,
                            max: DemlemeMetric.inspectorMax
                        )
                }
        }
    }

    private var selectedModule: ModuleSnapshot? {
        guard case let .ready(modules) = store.phase else { return nil }
        return modules.first { $0.id == selection }
    }

    @ViewBuilder
    private var detail: some View {
        if let module = selectedModule {
            VStack(spacing: DemlemeMetric.space2) {
                Text(UiText.moduleName(module.id)).font(.title2.bold())
                Text(ModulesListView.stateText(module)).foregroundStyle(DemlemeColor.label2)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .accessibilityElement(children: .combine)
            .accessibilityIdentifier("module.detail")
        } else {
            ContentUnavailableView(UiText.text("modules.title"), systemImage: "square.grid.2x2")
                .accessibilityIdentifier("module.detail.empty")
        }
    }

    @ViewBuilder
    private var inspector: some View {
        if let module = selectedModule {
            List {
                LabeledContent(UiText.moduleName(module.id), value: ModulesListView.stateText(module))
            }
            .accessibilityIdentifier("module.inspector")
        } else {
            ContentUnavailableView(UiText.text("modules.title"), systemImage: "square.grid.2x2")
        }
    }
}
