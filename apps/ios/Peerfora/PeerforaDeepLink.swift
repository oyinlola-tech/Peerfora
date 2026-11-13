import Foundation

enum PeerforaDeepLink: Equatable {
    case home
    case completeSocialSignup
    case community(String)
    case discussion(String)

    init?(url: URL) {
        guard url.scheme == "peerfora" else {
            return nil
        }

        let pathComponents = url.pathComponents.filter { $0 != "/" }
        let route = url.host ?? pathComponents.first
        let routeArguments = url.host == nil
            ? Array(pathComponents.dropFirst())
            : pathComponents
        switch route {
        case "home", nil:
            self = .home
        case "complete-social-signup":
            self = .completeSocialSignup
        case "communities":
            guard let communitySlug = routeArguments.first else {
                return nil
            }
            self = .community(communitySlug)
        case "posts":
            guard let postIdentifier = routeArguments.first else {
                return nil
            }
            self = .discussion(postIdentifier)
        default:
            return nil
        }
    }
}
