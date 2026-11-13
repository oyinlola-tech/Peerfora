# Known limitations

- The PRD was supplied as an external attachment and is not checked into the repository.
- The full prompt's Cargo workspace and `services/api` layout conflicts with the current codebase-alignment instruction to keep the API crate at the root; resolve this as a dedicated architecture decision before moving files.
- The API exposes `GET /health`, not the proposed live/readiness/status routes. Readiness, OpenAPI generation, request timeouts/body limits, and full middleware/security headers are not implemented.
- SQLx connects and applies migrations at startup, but pool limits/timeouts and a live fresh-database migration validation are outstanding.
- Redis, Meilisearch, object storage, SMTP, OpenTelemetry, and background queue services have local/configuration foundations only; the API does not connect to optional Compose services.
- A cleanup job module exists, but no standalone Apalis worker or selected durable queue backend is configured.
- Shared Rust client SDK, UniFFI, generated OpenAPI/client bindings, and contract validation are not implemented.
- Web, Android, SwiftUI, and Tauri projects remain initial shells. Tailwind CSS generation passed; app builds and UI/runtime behavior are unverified.
- Linux does not provide Xcode or Apple SDKs, so iOS/iPadOS/macOS builds are unavailable on this host.
- Android builds were not run; the checked-in Gradle wrapper requires its configured Gradle and Android SDK/NDK toolchains.
- Windows widget provider registration/build is not established; its checked-in Adaptive Card is only a static preview. GNOME/KDE/Apple widget runtime behavior is unverified.
- Authentication provider calls and database behavior were not tested against configured services. The global minimum-age rule now has tests, but Cargo could not compile or run them because the offline cache lacks a locked dependency and online index update stalled.
- CI workflow files exist but were not executed. They use mutable action version tags and lack dedicated security, migration, and API contract validation workflows.
- Terraform remains documentation/scaffolding; no provider modules or cloud resources were initialized.
