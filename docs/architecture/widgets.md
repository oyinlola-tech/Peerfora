# Widget architecture

Widget previews are platform-owned and contain no live account or community data. iOS and macOS use WidgetKit, Android uses Glance, KDE uses a Plasma applet, and GNOME uses a Shell extension. The Windows folder contains an Adaptive Card preview and project shell; its packaged provider is not implemented.

No widget API routes are registered. Do not add private content or background polling until server authorization, compact response contracts, safe deep links, and refresh limits are implemented and documented.
