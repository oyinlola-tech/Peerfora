# Local development

The current API development path is `docker compose up -d postgres`, `cp .env.example .env`, then `cargo run`. Optional service profiles are `cache`, `search`, `object-storage`, and `email`; start only the ones a feature needs. Use `just --list` to see repository commands.

Platform setup is separate: web requires the WASM target, cargo-leptos, Node, and pnpm; Android requires JDK and Android SDK; iOS requires macOS, Xcode, and XcodeGen; Tauri desktop requires native WebView build packages for the target OS. The repository does not install global tools automatically.
