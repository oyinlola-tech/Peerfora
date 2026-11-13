import Foundation

struct PeerforaAPIClient {
    let configuration: PeerforaAPIConfiguration
    let session: URLSession

    init(
        configuration: PeerforaAPIConfiguration,
        session: URLSession = .shared
    ) {
        self.configuration = configuration
        self.session = session
    }

    func fetchHealthStatus() async throws -> PeerforaHealthStatus {
        let healthURL = configuration.baseURL.appending(path: "health")
        let (responseData, _) = try await session.data(from: healthURL)
        return try JSONDecoder().decode(PeerforaHealthStatus.self, from: responseData)
    }
}
