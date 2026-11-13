# Monorepo map

```text
src/                         Rust modular-monolith API
migrations/                  PostgreSQL migrations
apps/web/                    Leptos SSR and hydration foundation
apps/ios/                    SwiftUI app and WidgetKit extension
apps/android/                Compose app, shared core, and Glance widget
apps/desktop/                Student Tauri app with Leptos CSR
apps/admin-desktop/          Separate administration Tauri app
widgets/                     macOS, Windows, and Linux widget foundations
packages/design-tokens/      Neutral system-based CSS token interface
packages/localization/       Initial locale resource
packages/platform-assets/    Approved assets are not available yet
contracts/                   OpenAPI/events/schemas status notes
tests/                       Test-category guidance, no test cases yet
infrastructure/              Local containers and build files
docs/                        Product, platform, architecture, and API notes
```

The app directories are initial scaffolds. Their route placeholders are not approved product screens. The backend remains at the repository root; it has not been moved into a Cargo workspace or split into crates.
