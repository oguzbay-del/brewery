import Foundation
import Observation

/// Loading state of the module list.
enum ModulesPhase: Sendable, Equatable {
    case idle
    case loading
    case ready([ModuleSnapshot])
    case noWorkspace
    case signInRequired
    case notMember
    case failed(requestID: String?)
}

/// Problem the API returned for a write (403 `module_disabled` or `module_read_only`).
enum ModuleNotice: Sendable, Equatable {
    case disabled
    case readOnly
}

/// Holds the module state for the screens. It holds no business rule: the server decides.
@MainActor
@Observable
final class ModulesStore {
    private(set) var phase: ModulesPhase = .idle
    private(set) var notice: ModuleNotice?

    @ObservationIgnored private let service: any ModulesService

    init(service: any ModulesService) {
        self.service = service
    }

    func load() async {
        phase = .loading
        do {
            if let modules = try await service.loadModules() {
                phase = .ready(modules)
            } else {
                phase = .noWorkspace
            }
        } catch let error as APIError {
            switch error.status {
            case 401: phase = .signInRequired
            case 404: phase = .notMember
            default: phase = .failed(requestID: error.requestId)
            }
        } catch {
            phase = .failed(requestID: nil)
        }
    }

    /// Access state of a module id. `nil` means the screen needs no module.
    func access(for moduleID: String?) -> ModuleAccess {
        guard let moduleID else { return .on }
        switch phase {
        case .idle, .loading: return .loading
        case let .ready(modules): return ModuleAccess.resolve(moduleID, in: modules)
        case .noWorkspace, .signInRequired, .notMember, .failed: return .unknown
        }
    }

    /// Call this with the error of a failed write; a module problem becomes a notice.
    func noteWriteError(_ error: any Error) {
        switch (error as? APIError)?.code {
        case "module_disabled": notice = .disabled
        case "module_read_only": notice = .readOnly
        default: break
        }
    }

    func dismissNotice() {
        notice = nil
    }
}
