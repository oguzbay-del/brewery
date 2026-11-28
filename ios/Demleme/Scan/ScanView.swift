import SwiftUI

/// The scan screen: camera scan with a manual entry as a fallback, then the card of the code.
struct ScanView: View {
    let store: ScanStore
    let batch: BatchScanStore
    let modules: ModulesStore
    @State private var cameraOn = false
    @State private var batchMode = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: DemlemeMetric.space4) {
                if batchMode {
                    BatchScanView(store: batch, modules: modules)
                    Button(UiText.text("scan.batch_close")) { batchMode = false }
                        .accessibilityIdentifier("scan.batch_close")
                } else {
                    single
                }
            }
            .padding(DemlemeMetric.space4)
        }
        .task { await store.loadPermissions() }
    }

    @ViewBuilder
    private var single: some View {
        @Bindable var store = store
        switch store.phase {
        case let .card(card):
            ScanCardView(
                card: card,
                store: store,
                writesBlocked: modules.access(for: "production").blocksWrites,
                onModuleError: modules.noteWriteError
            )
            Button(UiText.text("scan.title")) { reset() }
                .accessibilityIdentifier("scan.again")
        case .resolving:
            ProgressView(UiText.text("scan.resolving"))
                .accessibilityIdentifier("scan.resolving")
        case .idle, .failed:
            entry(code: $store.manualCode)
            if case let .failed(failure) = store.phase {
                Text(UiText.text(Self.failureKey(failure)))
                    .foregroundStyle(DemlemeColor.red)
                    .accessibilityIdentifier("scan.error")
            }
            if store.canBatch {
                Button(UiText.text("scan.batch_title")) { batchMode = true }
                    .accessibilityIdentifier("scan.batch_open")
            }
        }
    }

    @ViewBuilder
    private func entry(code: Binding<String>) -> some View {
        Text(UiText.text("scan.intro"))
            .font(.subheadline)
            .foregroundStyle(DemlemeColor.label2)
        camera
        TextField(UiText.text("scan.code_placeholder"), text: code)
            .textFieldStyle(.roundedBorder)
            .textInputAutocapitalization(.characters)
            .autocorrectionDisabled()
            .submitLabel(.go)
            .onSubmit { Task { await store.resolve(store.manualCode) } }
            .accessibilityLabel(UiText.text("scan.code_label"))
            .accessibilityIdentifier("scan.code_field")
        Button(UiText.text("scan.resolve")) {
            Task { await store.resolve(store.manualCode) }
        }
        .buttonStyle(.gristoraPrimary)
        .accessibilityIdentifier("scan.resolve")
    }

    @ViewBuilder
    private var camera: some View {
        if DataScannerView.isAvailable {
            if cameraOn {
                DataScannerView { code in
                    cameraOn = false
                    Task { await store.resolve(code) }
                }
                .frame(height: 280)
                .clipShape(RoundedRectangle(cornerRadius: DemlemeMetric.radiusMd))
                .accessibilityIdentifier("scan.camera_view")
            }
            Button(UiText.text(cameraOn ? "scan.camera_close" : "scan.camera")) { cameraOn.toggle() }
                .accessibilityIdentifier("scan.camera")
        } else {
            Text(UiText.text("scan.camera_unavailable"))
                .font(.footnote)
                .foregroundStyle(DemlemeColor.label2)
                .accessibilityIdentifier("scan.camera_unavailable")
        }
    }

    private func reset() {
        cameraOn = false
        store.reset()
    }

    private static func failureKey(_ failure: ScanFailure) -> String {
        switch failure {
        case .notFound: "scan.error_not_found"
        case .forbidden: "scan.error_forbidden"
        case .generic: "scan.error_generic"
        }
    }
}

/// The object and only the actions the server lists for the caller.
struct ScanCardView: View {
    let card: ScanCard
    let store: ScanStore
    let writesBlocked: Bool
    let onModuleError: @MainActor (any Error) -> Void
    @State private var askStatus = false
    @State private var askRetire = false

    private var offered: [ScanAction] { card.actions.filter(store.isOffered) }

    var body: some View {
        VStack(alignment: .leading, spacing: DemlemeMetric.space3) {
            Text(UiText.text("scan.kind_" + card.kind.rawValue))
                .font(.caption)
                .foregroundStyle(DemlemeColor.label2)
            Text(card.label)
                .font(.title2.bold())
                .accessibilityIdentifier("scan.card_label")
            ForEach(card.details, id: \.labelKey) { detail in
                HStack {
                    Text(UiText.text(detail.labelKey)).foregroundStyle(DemlemeColor.label2)
                    Spacer()
                    Text(detail.value)
                }
                .font(.subheadline)
            }
            Divider()
            Text(UiText.text("scan.actions_title")).font(.headline)
            if offered.isEmpty {
                Text(UiText.text("scan.no_actions"))
                    .font(.footnote)
                    .foregroundStyle(DemlemeColor.label2)
                    .accessibilityIdentifier("scan.no_actions")
            }
            ForEach(offered, id: \.rawValue) { action in
                Button(UiText.text(action.textKey)) { start(action) }
                    .buttonStyle(.gristoraSecondary)
                    .disabled(writesBlocked || store.actionState == .running)
                    .accessibilityIdentifier("scan.action." + action.rawValue)
            }
            if writesBlocked, !offered.isEmpty {
                Text(UiText.text("modules.read_only_reason", ["module": UiText.moduleName("production")]))
                    .font(.caption)
                    .foregroundStyle(DemlemeColor.label2)
            }
            actionMessage
        }
        .padding(DemlemeMetric.space4)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(DemlemeColor.card, in: RoundedRectangle(cornerRadius: DemlemeMetric.radiusMd))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("scan.card")
        .confirmationDialog(UiText.text("scan.new_status"), isPresented: $askStatus, titleVisibility: .visible) {
            ForEach(ContainerStatusChoice.allCases) { choice in
                Button(UiText.text(choice.textKey)) { run(choice.rawValue) }
                    .accessibilityIdentifier("scan.status." + choice.rawValue)
            }
            Button(UiText.text("scan.cancel"), role: .cancel) {}
        }
        .confirmationDialog(UiText.text("scan.retire_confirm"), isPresented: $askRetire, titleVisibility: .visible) {
            Button(UiText.text("scan.action_retire"), role: .destructive) { run("retired") }
                .accessibilityIdentifier("scan.retire_confirm")
            Button(UiText.text("scan.cancel"), role: .cancel) {}
        }
    }

    @ViewBuilder
    private var actionMessage: some View {
        switch store.actionState {
        case .idle:
            EmptyView()
        case .running:
            ProgressView()
        case .done:
            Text(UiText.text("scan.action_done"))
                .foregroundStyle(DemlemeColor.green)
                .accessibilityIdentifier("scan.action_done")
        case let .failed(failure):
            Text(UiText.text(Self.failureKey(failure)))
                .foregroundStyle(DemlemeColor.red)
                .accessibilityIdentifier("scan.action_error")
        }
    }

    private func start(_ action: ScanAction) {
        switch action {
        case .changeStatus: askStatus = true
        case .retire: askRetire = true
        case .view, .fill, .recall, .printLabel: break
        }
    }

    private func run(_ status: String) {
        Task { await store.setStatus(status, note: nil, onModuleError: onModuleError) }
    }

    static func failureKey(_ failure: ScanActionFailure) -> String {
        switch failure {
        case .forbidden: "scan.error_forbidden"
        case .conflict: "scan.action_conflict"
        case .generic: "scan.action_error"
        }
    }
}
