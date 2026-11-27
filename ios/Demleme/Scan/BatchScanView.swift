import SwiftUI

/// Batch scan: collect many container codes, choose one status, send them in one request,
/// then show one result for each code (a partial failure is shown row by row).
struct BatchScanView: View {
    let store: BatchScanStore
    let modules: ModulesStore
    @State private var cameraOn = false

    var body: some View {
        @Bindable var store = store
        VStack(alignment: .leading, spacing: 12) {
            Text(UiText.text("scan.batch_title")).font(.title2.bold())
            if DataScannerView.isAvailable {
                if cameraOn {
                    DataScannerView { code in store.add(code) }
                        .frame(height: 240)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
                Button(UiText.text(cameraOn ? "scan.camera_close" : "scan.camera")) { cameraOn.toggle() }
            }
            HStack {
                TextField(UiText.text("scan.code_placeholder"), text: $store.manualCode)
                    .textFieldStyle(.roundedBorder)
                    .textInputAutocapitalization(.characters)
                    .autocorrectionDisabled()
                    .onSubmit { store.addManualCode() }
                    .accessibilityLabel(UiText.text("scan.code_label"))
                    .accessibilityIdentifier("batch.code_field")
                Button(UiText.text("scan.batch_add")) { store.addManualCode() }
                    .accessibilityIdentifier("batch.add")
            }
            codeList
            Picker(UiText.text("scan.new_status"), selection: $store.status) {
                ForEach(ContainerStatusChoice.allCases) { choice in
                    Text(UiText.text(choice.textKey)).tag(choice)
                }
            }
            .accessibilityIdentifier("batch.status")
            TextField(UiText.text("scan.note"), text: $store.note)
                .textFieldStyle(.roundedBorder)
            submit
            result
        }
    }

    @ViewBuilder
    private var codeList: some View {
        Text(UiText.text("scan.batch_codes", ["count": String(store.codes.count)]))
            .font(.headline)
            .accessibilityIdentifier("batch.count")
        if store.codes.isEmpty {
            Text(UiText.text("scan.batch_empty"))
                .font(.footnote)
                .foregroundStyle(DemlemeColor.label2)
        }
        ForEach(store.codes, id: \.self) { code in
            HStack {
                Text(code)
                Spacer()
                Button(UiText.text("scan.batch_remove"), role: .destructive) { store.remove(code) }
                    .font(.footnote)
            }
        }
    }

    @ViewBuilder
    private var submit: some View {
        let blocked = modules.access(for: "production").blocksWrites
        HStack {
            Button(UiText.text("scan.batch_submit")) {
                Task { await store.submit(onModuleError: modules.noteWriteError) }
            }
            .buttonStyle(.borderedProminent)
            .disabled(store.codes.isEmpty || store.running || blocked)
            .accessibilityIdentifier("batch.submit")
            Button(UiText.text("scan.batch_clear")) { store.clear() }
                .disabled(store.codes.isEmpty)
        }
        if blocked {
            Text(UiText.text("modules.read_only_reason", ["module": UiText.moduleName("production")]))
                .font(.caption)
                .foregroundStyle(DemlemeColor.label2)
        }
        if store.running { ProgressView() }
        if let failure = store.failure {
            Text(UiText.text(ScanCardView.failureKey(failure)))
                .foregroundStyle(DemlemeColor.red)
                .accessibilityIdentifier("batch.error")
        }
    }

    @ViewBuilder
    private var result: some View {
        if let outcome = store.outcome {
            Text(UiText.text(
                "scan.batch_summary",
                ["changed": String(outcome.changed), "failed": String(outcome.failed)]
            ))
            .font(.headline)
            .accessibilityIdentifier("batch.summary")
            ForEach(outcome.items) { item in
                HStack {
                    Image(systemName: item.ok ? "checkmark.circle" : "exclamationmark.triangle")
                        .foregroundStyle(item.ok ? DemlemeColor.green : DemlemeColor.red)
                        .accessibilityHidden(true)
                    Text(item.code)
                    Spacer()
                    Text(UiText.text(BatchScanStore.resultKey(item)))
                }
                .accessibilityElement(children: .combine)
                .accessibilityIdentifier("batch.result.\(item.index)")
            }
        }
    }
}
