import Foundation
import Observation

/// Container statuses a scan can set. `filled` comes only from a fill; `retired` is the
/// retire action on the card.
enum ContainerStatusChoice: String, CaseIterable, Sendable, Identifiable {
    case clean
    case out
    case dirty
    case maintenance

    var id: String { rawValue }
    var textKey: String { "containers.status_" + rawValue }
}

/// State of the batch scan: a list of codes, one status, then one result for each code.
@MainActor
@Observable
final class BatchScanStore {
    private(set) var codes: [String] = []
    var status: ContainerStatusChoice = .clean
    var note = ""
    var manualCode = ""
    private(set) var running = false
    private(set) var outcome: BatchOutcome?
    private(set) var failure: ScanActionFailure?

    @ObservationIgnored private let service: any ScanService

    init(service: any ScanService) {
        self.service = service
    }

    /// Adds a code once. The camera reports the same code many times.
    func add(_ rawCode: String) {
        let code = rawCode.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !code.isEmpty, !codes.contains(code) else { return }
        codes.append(code)
        outcome = nil
    }

    func addManualCode() {
        add(manualCode)
        manualCode = ""
    }

    func remove(_ code: String) {
        codes.removeAll { $0 == code }
        outcome = nil
    }

    func clear() {
        codes = []
        outcome = nil
        failure = nil
        note = ""
    }

    /// Sends all codes with one status. A partial failure is a normal result: each code has its row.
    func submit(onModuleError: @MainActor (any Error) -> Void = { _ in }) async {
        guard !codes.isEmpty, !running else { return }
        running = true
        failure = nil
        defer { running = false }
        do {
            outcome = try await service.setStatusBatch(
                codes: codes,
                status: status.rawValue,
                note: ScanStore.clean(note)
            )
        } catch {
            onModuleError(error)
            failure = ScanStore.actionFailure(error)
        }
    }

    /// Catalog key for the result of one code.
    static func resultKey(_ item: BatchItem) -> String {
        if item.ok { return item.unchanged ? "scan.batch_result_unchanged" : "scan.batch_result_ok" }
        switch item.error {
        case "not_found", "not_a_container", "container_retired": return "scan.batch_error_" + (item.error ?? "")
        default: return "scan.batch_error_other"
        }
    }
}
