import SwiftUI

/// The scan screen: camera scan with a manual entry as a fallback, then the card of the code.
struct ScanView: View {
    let store: ScanStore
    @State private var cameraOn = false

    var body: some View {
        @Bindable var store = store
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                switch store.phase {
                case let .card(card):
                    ScanCardView(card: card)
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
                }
            }
            .padding()
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
        .buttonStyle(.borderedProminent)
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
                .clipShape(RoundedRectangle(cornerRadius: 12))
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

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
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
            if card.actions.isEmpty {
                Text(UiText.text("scan.no_actions"))
                    .font(.footnote)
                    .foregroundStyle(DemlemeColor.label2)
                    .accessibilityIdentifier("scan.no_actions")
            }
            ForEach(card.actions, id: \.rawValue) { action in
                Label(UiText.text(action.textKey), systemImage: "chevron.right")
                    .accessibilityIdentifier("scan.action." + action.rawValue)
            }
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(DemlemeColor.card, in: RoundedRectangle(cornerRadius: 12))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("scan.card")
    }
}
