import Foundation
import HTTPTypes
import OpenAPIRuntime
import OpenAPIURLSession

/// Per-request headers a caller can set. They travel in a task-local value, so the generated
/// operations stay unchanged and the values never leak to other requests.
public struct APIRequestContext: Sendable, Equatable {
    public var idempotencyKey: String?
    public var ifMatch: String?

    @TaskLocal static var current = APIRequestContext()
}

/// Adds the common headers (same set as `web/src/api/client.ts`) and maps error responses to `APIError`.
public struct APIHeadersMiddleware: ClientMiddleware {
    static let requestIdName = HTTPField.Name("X-Request-Id")!
    static let idempotencyKeyName = HTTPField.Name("Idempotency-Key")!
    static let ifMatchName = HTTPField.Name("If-Match")!
    static let maxErrorBodyBytes = 1 << 20

    private let languageProvider: @Sendable () -> String
    private let makeRequestId: @Sendable () -> String

    public init(
        languageProvider: @escaping @Sendable () -> String = { APIClient.currentLanguage() },
        makeRequestId: @escaping @Sendable () -> String = { UUID().uuidString.lowercased() }
    ) {
        self.languageProvider = languageProvider
        self.makeRequestId = makeRequestId
    }

    public func intercept(
        _ request: HTTPRequest,
        body: HTTPBody?,
        baseURL: URL,
        operationID _: String,
        next: @Sendable (HTTPRequest, HTTPBody?, URL) async throws -> (HTTPResponse, HTTPBody?)
    ) async throws -> (HTTPResponse, HTTPBody?) {
        var request = request
        let context = APIRequestContext.current
        request.headerFields[Self.requestIdName] = makeRequestId()
        request.headerFields[.acceptLanguage] = languageProvider()
        if let key = context.idempotencyKey { request.headerFields[Self.idempotencyKeyName] = key }
        if let etag = context.ifMatch { request.headerFields[Self.ifMatchName] = etag }

        let (response, responseBody) = try await next(request, body, baseURL)
        guard response.status.code >= 400 else { return (response, responseBody) }

        var data = Data()
        if let responseBody {
            data = (try? await Data(collecting: responseBody, upTo: Self.maxErrorBodyBytes)) ?? Data()
        }
        throw APIError.make(
            status: response.status.code,
            body: data,
            headerRequestId: response.headerFields[Self.requestIdName]
        )
    }
}

/// Thin wrapper around the generated `Client`. It holds no business rule and does no calculation.
public struct APIClient: Sendable {
    /// The generated client. Call the contract operations on it.
    public let client: Client

    public init(
        serverURL: URL,
        transport: any ClientTransport,
        middleware: APIHeadersMiddleware = APIHeadersMiddleware()
    ) {
        client = Client(serverURL: serverURL, transport: transport, middlewares: [middleware])
    }

    /// Production client: base URL from `Info.plist`, `URLSession` transport.
    public init(infoDictionary: [String: Any]? = Bundle.main.infoDictionary) throws {
        let serverURL = try APIConfiguration.baseURL(infoDictionary: infoDictionary)
        self.init(serverURL: serverURL, transport: URLSessionTransport())
    }

    /// Runs `operation` with `Idempotency-Key` and/or `If-Match` set on the requests it sends.
    public static func withHeaders<T: Sendable>(
        idempotencyKey: String? = nil,
        ifMatch: String? = nil,
        operation: @Sendable () async throws -> T
    ) async rethrows -> T {
        try await APIRequestContext.$current.withValue(
            APIRequestContext(idempotencyKey: idempotencyKey, ifMatch: ifMatch),
            operation: operation
        )
    }

    /// Language tag for `Accept-Language`, from the app locale.
    public static func currentLanguage() -> String {
        Locale.preferredLanguages.first ?? Locale.current.identifier(.bcp47)
    }
}
