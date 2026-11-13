import SwiftUI
import WidgetKit

struct PeerforaWidgetEntry: TimelineEntry {
    let date: Date
}

struct PeerforaWidgetTimelineProvider: TimelineProvider {
    func placeholder(in context: Context) -> PeerforaWidgetEntry {
        PeerforaWidgetEntry(date: .now)
    }

    func getSnapshot(in context: Context, completion: @escaping (PeerforaWidgetEntry) -> Void) {
        completion(PeerforaWidgetEntry(date: .now))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<PeerforaWidgetEntry>) -> Void) {
        let entry = PeerforaWidgetEntry(date: .now)
        completion(Timeline(entries: [entry], policy: .never))
    }
}

struct PeerforaWidgetView: View {
    var entry: PeerforaWidgetEntry

    var body: some View {
        VStack(alignment: .leading) {
            Text("Peerfora")
                .font(.headline)
            Text("Open the app to explore your student community.")
                .font(.caption)
        }
        .containerBackground(.fill.tertiary, for: .widget)
    }
}

struct PeerforaHomeWidget: Widget {
    let kind = "PeerforaHomeWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: PeerforaWidgetTimelineProvider()) { entry in
            PeerforaWidgetView(entry: entry)
        }
        .configurationDisplayName("Peerfora")
        .description("A development preview for Peerfora widgets.")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}

@main
struct PeerforaWidgetsBundle: WidgetBundle {
    var body: some Widget {
        PeerforaHomeWidget()
    }
}
