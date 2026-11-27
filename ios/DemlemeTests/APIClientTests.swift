@testable import Demleme
import Foundation
import HTTPTypes
import OpenAPIRuntime
import Testing

/// Records every request and answers with a fixed response.
private actor MockTransport: ClientTransport {
    private(set) var requests: [HTTPRequest] = []
    private let status: HTTPResponse.Status
    private let contentType: String
    private let body: String
    private let responseHeaders: [HTTPField.Name: String]

    init(
        status: HTTPResponse.Status = .ok,
        contentType: String = "application/json",
        body: String = "{}",
        responseHeaders: [HTTPField.Name: String] = [:]
    ) {
        self.status = status
        self.contentType = contentType
        self.body = body
        self.responseHeaders = responseHeaders
    }

    func send(
        _ request: HTTPRequest,
        body _: HTTPBody?,
        baseURL _: URL,
        operationID _: String
    ) async throws -> (HTTPResponse, HTTPBody?) {
        requests.append(request)
        var response = HTTPResponse(status: status)
        response.headerFields[.contentType] = contentType
        for (name, value) in responseHeaders {
            response.headerFields[name] = value
        }
        return (response, HTTPBody(body))
    }
}

private let baseURL = URL(string: "https://api.example.test")!
private let requestIdName = HTTPField.Name("X-Request-Id")!
private let idempotencyKeyName = HTTPField.Name("Idempotency-Key")!
private let ifMatchName = HTTPField.Name("If-Match")!

/// Sends one request through the middleware into the mock transport.
private func send(through transport: MockTransport) async throws -> HTTPResponse {
    let middleware = APIHeadersMiddleware(languageProvider: { "tr-TR" })
    let request = HTTPRequest(method: .get, scheme: nil, authority: nil, path: "/healthz")
    let (response, _) = try await middleware.intercept(
        request, body: nil, baseURL: baseURL, operationID: "health_check"
    ) { request, body, baseURL in
        try await transport.send(request, body: body, baseURL: baseURL, operationID: "health_check")
    }
    return response
}

struct APIClientTests {
    @Test func everyRequestHasItsOwnRequestId() async throws {
        let transport = MockTransport()
        _ = try await send(through: transport)
        _ = try await send(through: transport)
        let ids = await transport.requests.compactMap { $0.headerFields[requestIdName] }
        #expect(ids.count == 2)
        #expect(Set(ids).count == 2)
        for id in ids {
            #expect(UUID(uuidString: id) != nil)
            #expect(id.dropFirst(14).first == "4", "UUID version 4")
        }
    }

    @Test func acceptLanguageComesFromTheLocale() async throws {
        let transport = MockTransport()
        _ = try await send(through: transport)
        let request = try #require(await transport.requests.first)
        #expect(request.headerFields[.acceptLanguage] == "tr-TR")
    }

    @Test func optionalHeadersAreAbsentByDefault() async throws {
        let transport = MockTransport()
        _ = try await send(through: transport)
        let request = try #require(await transport.requests.first)
        #expect(request.headerFields[idempotencyKeyName] == nil)
        #expect(request.headerFields[ifMatchName] == nil)
    }

    @Test func optionalHeadersAreSentWhenGiven() async throws {
        let transport = MockTransport()
        _ = try await APIClient.withHeaders(idempotencyKey: "key-1", ifMatch: "\"v3\"") {
            try await send(through: transport)
        }
        let request = try #require(await transport.requests.first)
        #expect(request.headerFields[idempotencyKeyName] == "key-1")
        #expect(request.headerFields[ifMatchName] == "\"v3\"")
        // The values do not leak to the next request.
        _ = try await send(through: transport)
        let second = try #require(await transport.requests.last)
        #expect(second.headerFields[idempotencyKeyName] == nil)
    }

    @Test func problemBodyBecomesApiError() async throws {
        let body = """
        {"type":"/errors/module_disabled","title":"Module disabled","status":403,\
        "code":"module_disabled","request_id":"req-1"}
        """
        let transport = MockTransport(status: .forbidden, contentType: "application/problem+json", body: body)
        do {
            _ = try await send(through: transport)
            Issue.record("expected APIError")
        } catch let error as APIError {
            #expect(error.code == "module_disabled")
            #expect(error.status == 403)
            #expect(error.requestId == "req-1")
        }
    }

    @Test func requestIdFallsBackToResponseHeader() async throws {
        let body = #"{"type":"/errors/internal","title":"x","status":500,"code":"internal"}"#
        let transport = MockTransport(
            status: .internalServerError,
            body: body,
            responseHeaders: [requestIdName: "hdr-9"]
        )
        do {
            _ = try await send(through: transport)
            Issue.record("expected APIError")
        } catch let error as APIError {
            #expect(error.requestId == "hdr-9")
        }
    }

    @Test func htmlErrorBodyBecomesUnexpected() async throws {
        let transport = MockTransport(
            status: .internalServerError,
            contentType: "text/html",
            body: "<html><body>Bad gateway</body></html>"
        )
        do {
            _ = try await send(through: transport)
            Issue.record("expected APIError")
        } catch let error as APIError {
            #expect(error == .unexpected(status: 500))
        }
    }

    @Test func successResponseIsPassedThrough() async throws {
        let transport = MockTransport()
        let response = try await send(through: transport)
        #expect(response.status == .ok)
    }

    @Test func baseURLComesFromInfoDictionary() throws {
        let url = try APIConfiguration.baseURL(infoDictionary: ["GRISTORA_API_BASE_URL": "https://api.example.test"])
        #expect(url.host == "api.example.test")
        #expect(throws: APIConfiguration.ConfigurationError.missingBaseURL) {
            try APIConfiguration.baseURL(infoDictionary: [:])
        }
        #expect(throws: APIConfiguration.ConfigurationError.invalidBaseURL("not a url")) {
            try APIConfiguration.baseURL(infoDictionary: ["GRISTORA_API_BASE_URL": "not a url"])
        }
    }
}
