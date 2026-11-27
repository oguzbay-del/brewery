import Foundation
import Observation

/// Why a scan failed, as the user sees it.
enum ScanFailure: Sendable, Equatable {
    case notFound
    case forbidden
    case generic
}

/// State of an action on the card.
enum ScanActionState: Sendable, Equatable {
    case idle
    case running
    case done
    case failed(ScanActionFailure)
}

enum ScanActionFailure: Sendable, Equatable {
    /// 403: no permission, or the module is off or read only.
    case forbidden
    /// 409: the record changed; scan again.
    case conflict
    case generic
}

enum ScanPhase: Sendable, Equatable {
    case idle
    case resolving
    case card(ScanCard)
    case failed(ScanFailure)
}

/// State of the scan screen. It holds no business rule: the server resolves the code
/// and lists the permitted actions.
@MainActor
@Observable
final class ScanStore {
    var manualCode = ""
    private(set) var phase: ScanPhase = .idle
    private(set) var actionState: ScanActionState = .idle
    /// Workspace-wide permission codes of the caller. Empty until loaded.
    private(set) var permissions: Set<String> = []
    /// The last code that gave a card; actions use it.
    private(set) var lastCode: String?

    @ObservationIgnored private let service: any ScanService

    init(service: any ScanService) {
        self.service = service
    }

    /// Resolves a code from the camera or from the text field. An empty code is ignored.
    /// A second code that arrives while one is resolving is ignored (the camera repeats codes).
    func resolve(_ rawCode: String) async {
        let code = rawCode.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !code.isEmpty, phase != .resolving else { return }
        phase = .resolving
        do {
            let card = try await service.resolve(code: code)
            lastCode = code
            actionState = .idle
            phase = .card(card)
        } catch {
            switch APIError.unwrap(error)?.status {
            case 404: phase = .failed(.notFound)
            case 401, 403: phase = .failed(.forbidden)
            default: phase = .failed(.generic)
            }
        }
    }

    /// Back to the entry state, for the next scan.
    func reset() {
        phase = .idle
        actionState = .idle
        lastCode = nil
        manualCode = ""
    }

    /// Loads the caller's permissions. On failure the set stays empty, so write actions stay
    /// hidden; the server checks every action again.
    func loadPermissions() async {
        permissions = (try? await service.permissions()) ?? []
    }

    /// The server lists the permitted actions in the card. Write actions also need the
    /// workspace permission `container.status` of the session, so an action is not offered
    /// when the session says the caller cannot do it.
    func isOffered(_ action: ScanAction) -> Bool {
        switch action {
        case .changeStatus, .retire: permissions.contains("container.status")
        case .view, .fill, .recall, .printLabel: false
        }
    }

    /// The batch scan needs `container.status`.
    var canBatch: Bool { permissions.contains("container.status") }

    /// Sets a new status on the container of the card, then reads the card again.
    /// - Parameter onModuleError: receives a module problem (403), for the module notice.
    func setStatus(_ status: String, note: String?, onModuleError: @MainActor (any Error) -> Void = { _ in }) async {
        guard let code = lastCode, actionState != .running else { return }
        actionState = .running
        do {
            try await service.setStatus(code: code, status: status, note: Self.clean(note))
            let card = try await service.resolve(code: code)
            phase = .card(card)
            actionState = .done
        } catch {
            onModuleError(error)
            actionState = .failed(Self.actionFailure(error))
        }
    }

    static func actionFailure(_ error: any Error) -> ScanActionFailure {
        switch APIError.unwrap(error)?.status {
        case 401, 403: .forbidden
        case 409: .conflict
        default: .generic
        }
    }

    /// An empty note is sent as no note.
    static func clean(_ note: String?) -> String? {
        guard let note = note?.trimmingCharacters(in: .whitespacesAndNewlines), !note.isEmpty else { return nil }
        return note
    }
}
