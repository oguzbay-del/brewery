import Foundation

/// Reads the server base URL from `Info.plist` (key `GRISTORA_API_BASE_URL`).
public enum APIConfiguration {
    public static let baseURLKey = "GRISTORA_API_BASE_URL"

    public enum ConfigurationError: Error, Sendable, Equatable {
        case missingBaseURL
        case invalidBaseURL(String)
    }

    public static func baseURL(infoDictionary: [String: Any]? = Bundle.main.infoDictionary) throws -> URL {
        guard let raw = infoDictionary?[baseURLKey] as? String,
              !raw.trimmingCharacters(in: .whitespaces).isEmpty
        else {
            throw ConfigurationError.missingBaseURL
        }
        guard let url = URL(string: raw), let scheme = url.scheme, ["http", "https"].contains(scheme), url.host != nil
        else {
            throw ConfigurationError.invalidBaseURL(raw)
        }
        return url
    }
}
