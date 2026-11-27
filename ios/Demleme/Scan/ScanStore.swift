import Foundation
import Observation

/// Why a scan failed, as the user sees it.
enum ScanFailure: Sendable, Equatable {
    case notFound
    case forbidden
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
            phase = .card(card)
        } catch let error as APIError {
            switch error.status {
            case 404: phase = .failed(.notFound)
            case 401, 403: phase = .failed(.forbidden)
            default: phase = .failed(.generic)
            }
        } catch {
            phase = .failed(.generic)
        }
    }

    /// Back to the entry state, for the next scan.
    func reset() {
        phase = .idle
        manualCode = ""
    }
}
