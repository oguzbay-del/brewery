import Foundation

/// Error thrown by the API client for every non-success response.
///
/// A response with an RFC 9457 problem body (`ProblemDetails` in the contract) becomes `.problem`.
/// Any other error body, for example an HTML page from a proxy, becomes `.unexpected`.
public enum APIError: Error, Sendable, Equatable {
    case problem(code: String, status: Int, requestId: String?, title: String?)
    case unexpected(status: Int)

    /// Stable problem code the app branches on. `nil` for `.unexpected`.
    public var code: String? {
        if case let .problem(code, _, _, _) = self { return code }
        return nil
    }

    public var status: Int {
        switch self {
        case let .problem(_, status, _, _), let .unexpected(status): status
        }
    }

    public var requestId: String? {
        if case let .problem(_, _, requestId, _) = self { return requestId }
        return nil
    }
}

/// Fields of `ProblemDetails` the client needs. The body is decoded here so the mapping
/// does not depend on generated type names.
private struct ProblemBody: Decodable {
    let code: String
    let status: Int?
    let title: String?
    let requestId: String?

    enum CodingKeys: String, CodingKey {
        case code, status, title
        case requestId = "request_id"
    }
}

extension APIError {
    /// Maps an error response to an `APIError`.
    /// - Parameters:
    ///   - status: HTTP status code of the response.
    ///   - body: Response body bytes (may be empty).
    ///   - headerRequestId: Value of the `X-Request-Id` response header, used when the body has none.
    static func make(status: Int, body: Data, headerRequestId: String?) -> APIError {
        guard let problem = try? JSONDecoder().decode(ProblemBody.self, from: body) else {
            return .unexpected(status: status)
        }
        return .problem(
            code: problem.code,
            status: problem.status ?? status,
            requestId: problem.requestId ?? headerRequestId,
            title: problem.title
        )
    }
}
