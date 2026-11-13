import SwiftUI

struct PeerforaRootView: View {
    var body: some View {
        NavigationStack {
            ContentUnavailableView(
                "Peerfora is ready",
                systemImage: "person.3",
                description: Text("Development foundation. Product screens will follow approved designs.")
            )
            .navigationTitle("Peerfora")
        }
    }
}
