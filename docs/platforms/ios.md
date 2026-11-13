# iOS and iPadOS application

The native SwiftUI app and WidgetKit extension are described by `apps/ios/project.yml`. Generate an Xcode project with `xcodegen generate --spec project.yml` from `apps/ios`, then build with Xcode on macOS.

The project includes a Keychain credential-store boundary, API base URL configuration, health client, and custom deep-link parser. The current machine is Linux and has no Xcode. Compilation, signing, simulator, device, notifications, and UI behavior are unverified. The native placeholder relies on system controls and appearance; no final product interface has been designed.
