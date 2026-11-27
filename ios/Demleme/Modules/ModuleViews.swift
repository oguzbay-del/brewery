import SwiftUI

/// Shows the upgrade message for an off module and a banner for a read-only module.
struct ModuleGate<Content: View>: View {
    let moduleID: String?
    let store: ModulesStore
    @ViewBuilder let content: () -> Content

    var body: some View {
        switch store.access(for: moduleID) {
        case .loading:
            ProgressView(UiText.text("modules.loading"))
                .accessibilityIdentifier("module.loading")
        case .off:
            ModuleUpgradeMessage(moduleID: moduleID ?? "")
        case .readOnly:
            VStack(spacing: 0) {
                ModuleReadOnlyBanner(moduleID: moduleID ?? "")
                content()
            }
        case .on, .unknown:
            content()
        }
    }
}

struct ModuleUpgradeMessage: View {
    let moduleID: String

    var body: some View {
        let name = UiText.moduleName(moduleID)
        VStack(spacing: 8) {
            Text(UiText.text("modules.upgrade_title", ["module": name]))
                .font(.headline)
            Text(UiText.text("modules.upgrade_body", ["module": name]))
                .font(.subheadline)
                .foregroundStyle(DemlemeColor.label2)
                .multilineTextAlignment(.center)
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier("module.upgrade")
    }
}

struct ModuleReadOnlyBanner: View {
    let moduleID: String

    var body: some View {
        Text(UiText.text("modules.read_only_banner", ["module": UiText.moduleName(moduleID)]))
            .font(.footnote)
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(DemlemeColor.tintSoft)
            .foregroundStyle(DemlemeColor.tintInk)
            .accessibilityIdentifier("module.read_only_banner")
    }
}

/// A button for a write action. It is disabled when the module is read only or off,
/// and a visible text gives the reason.
struct ModuleWriteButton: View {
    let moduleID: String
    let store: ModulesStore
    let title: String
    let action: () -> Void

    var body: some View {
        let blocked = store.access(for: moduleID).blocksWrites
        VStack(spacing: 4) {
            Button(title, action: action)
                .buttonStyle(.borderedProminent)
                .disabled(blocked)
                .accessibilityIdentifier("module.write_button")
            if blocked {
                Text(UiText.text("modules.read_only_reason", ["module": UiText.moduleName(moduleID)]))
                    .font(.caption)
                    .foregroundStyle(DemlemeColor.label2)
            }
        }
    }
}

/// Alert for a 403 `module_disabled` or `module_read_only` answer to a write.
struct ModuleNoticeView: View {
    let store: ModulesStore

    var body: some View {
        if let notice = store.notice {
            HStack {
                Text(UiText.text(notice == .disabled ? "modules.api_disabled" : "modules.api_read_only"))
                    .font(.footnote)
                Spacer()
                Button(UiText.text("modules.dismiss")) { store.dismissNotice() }
            }
            .padding(12)
            .background(DemlemeColor.fill)
            .accessibilityIdentifier("module.notice")
        }
    }
}

/// Settings screen: the modules of the workspace and their state.
struct ModulesListView: View {
    let store: ModulesStore

    var body: some View {
        List {
            switch store.phase {
            case .idle, .loading:
                ProgressView(UiText.text("modules.loading"))
            case let .ready(modules):
                ForEach(modules, id: \.id) { module in
                    HStack {
                        Text(UiText.moduleName(module.id))
                        Spacer()
                        Text(stateText(module)).foregroundStyle(DemlemeColor.label2)
                    }
                    .accessibilityElement(children: .combine)
                    .accessibilityIdentifier("module.row.\(module.id)")
                }
            case .noWorkspace:
                Text(UiText.text("modules.empty"))
            case .signInRequired:
                Text(UiText.text("modules.sign_in_required"))
            case .notMember:
                Text(UiText.text("modules.not_member"))
            case let .failed(requestID):
                VStack(alignment: .leading) {
                    Text(UiText.text("modules.error"))
                    if let requestID {
                        Text("\(UiText.text("modules.error_request_id")): \(requestID)")
                            .font(.caption)
                    }
                }
            }
        }
        .navigationTitle(UiText.text("modules.title"))
    }

    private func stateText(_ module: ModuleSnapshot) -> String {
        UiText.text(module.readOnly ? "modules.read_only" : (module.enabled ? "modules.on" : "modules.off"))
    }
}
