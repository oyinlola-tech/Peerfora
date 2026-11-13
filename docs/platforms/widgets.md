# Widget foundations

Widget placeholders live under `apps/ios/PeerforaWidgets`, `apps/android/widgets`, `widgets/macos`, `widgets/windows`, and `widgets/linux`. They display static development text only; no account or private community content is requested.

The iOS/macOS targets use WidgetKit, Android uses Glance, GNOME uses Shell extension APIs, and KDE uses a Plasma applet. The Windows folder includes an Adaptive Card preview and project shell, not a registered Windows Widgets provider. No widget API endpoint is implemented. Live data requires server authorization, privacy review, safe deep links, offline handling, and OS-scheduled refresh behavior.
