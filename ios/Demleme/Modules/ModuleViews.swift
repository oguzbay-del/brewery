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
        Callout(
            tone: .info,
            title: UiText.text("modules.read_only_banner", ["module": UiText.moduleName(moduleID)])
        )
        .padding(DemlemeMetric.space3)
        .accessibilityElement(children: .contain)
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
        let reason = UiText.text("modules.read_only_reason", ["module": UiText.moduleName(moduleID)])
        VStack(spacing: DemlemeMetric.space1) {
            Button(title, action: action)
                .buttonStyle(.gristoraPrimary)
                .disabled(blocked)
                .accessibilityHint(blocked ? reason : "")
                .accessibilityIdentifier("module.write_button")
            if blocked {
                Text(reason)
                    .font(.caption)
                    .foregroundStyle(DemlemeColor.label2)
            }
        }
    }
}

/// Callout for a 403 `module_disabled` or `module_read_only` answer to a write.
struct ModuleNoticeView: View {
    let store: ModulesStore

    var body: some View {
        if let notice = store.notice {
            Callout(
                tone: notice == .disabled ? .error : .warning,
                title: UiText.text(notice == .disabled ? "modules.api_disabled" : "modules.api_read_only"),
                action: CalloutAction(title: UiText.text("modules.dismiss")) { store.dismissNotice() }
            )
            .padding(DemlemeMetric.space3)
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("module.notice")
        }
    }
}

/// Settings screen: the modules of the workspace and their state.
struct ModulesListView: View {
    let store: ModulesStore
    @Binding var selection: String?

    var body: some View {
        List(selection: $selection) {
            switch store.phase {
            case .idle, .loading:
                ProgressView(UiText.text("modules.loading"))
            case let .ready(modules):
                ForEach(modules, id: \.id) { module in
                    HStack {
                        Text(UiText.moduleName(module.id))
                        Spacer()
                        Text(Self.stateText(module)).foregroundStyle(DemlemeColor.label2)
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

    static func stateText(_ module: ModuleSnapshot) -> String {
        UiText.text(module.readOnly ? "modules.read_only" : (module.enabled ? "modules.on" : "modules.off"))
    }
}
