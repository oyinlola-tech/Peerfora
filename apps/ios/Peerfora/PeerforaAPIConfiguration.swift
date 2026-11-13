import Foundation

struct PeerforaAPIConfiguration {
    let baseURL: URL

    static func readFromApplicationBundle() throws -> PeerforaAPIConfiguration {
        guard
            let configuredURL = Bundle.main.object(forInfoDictionaryKey: "PEERFORA_API_BASE_URL") as? String,
            let baseURL = URL(string: configuredURL),
            ["http", "https"].contains(baseURL.scheme?.lowercased() ?? "")
        else {
            throw PeerforaAPIConfigurationError.invalidBaseURL
        }

        return PeerforaAPIConfiguration(baseURL: baseURL)
    }
}

enum PeerforaAPIConfigurationError: Error {
    case invalidBaseURL
}
