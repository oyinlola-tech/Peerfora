# Student desktop application

The student desktop shell uses Tauri 2 with a Leptos CSR frontend in `apps/desktop`. Its application identifier and permissions are separate from the admin client. The shell exposes only Tauri core defaults; filesystem, shell, and remote-page privileges are not enabled.

Install Rust, the `wasm32-unknown-unknown` target, Trunk, Tauri CLI, and the platform WebView/build prerequisites. Run `just setup-desktop`, then `just desktop`. Packaging is disabled until approved platform icons and signing configuration exist. Linux, macOS, and Windows builds are unverified.
