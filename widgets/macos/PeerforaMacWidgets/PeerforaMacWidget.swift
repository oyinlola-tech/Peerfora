import SwiftUI
import WidgetKit

struct PeerforaMacWidgetEntry: TimelineEntry {
    let date: Date
}

struct PeerforaMacWidgetProvider: TimelineProvider {
    func placeholder(in context: Context) -> PeerforaMacWidgetEntry {
        PeerforaMacWidgetEntry(date: .now)
    }

    func getSnapshot(in context: Context, completion: @escaping (PeerforaMacWidgetEntry) -> Void) {
        completion(PeerforaMacWidgetEntry(date: .now))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<PeerforaMacWidgetEntry>) -> Void) {
        completion(Timeline(entries: [PeerforaMacWidgetEntry(date: .now)], policy: .never))
    }
}

struct PeerforaMacWidgetView: View {
    var entry: PeerforaMacWidgetEntry

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

struct PeerforaMacWidget: Widget {
    let kind = "PeerforaMacWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: PeerforaMacWidgetProvider()) { entry in
            PeerforaMacWidgetView(entry: entry)
        }
        .configurationDisplayName("Peerfora")
        .description("A development preview for Peerfora widgets.")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}

@main
struct PeerforaMacWidgetBundle: WidgetBundle {
    var body: some Widget {
        PeerforaMacWidget()
    }
}
