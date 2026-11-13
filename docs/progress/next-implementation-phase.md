# Next implementation phase

This checklist continues from the repository audit. It does not claim the listed work is implemented.

## 1. Restore executable Rust validation

- Provide Cargo with the missing crates.io index/package data, then run `cargo check --locked`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`.
- Resolve failures from compiler output and retain the repository's checked-in lockfile.
- Add a clean PostgreSQL-backed migration check when a local database is available.

## 2. Resolve repository layout decision

- Confirm whether the root API crate/Ansofra startup layout remains the architecture or whether it should migrate to a Cargo workspace with `services/api`.
- If migration is approved, plan it as a dedicated path/CI/Docker/documentation change and preserve the current account flows.

## 3. Complete API operational foundation

- Add real liveness and PostgreSQL readiness behavior and versioned status behavior.
- Document every added endpoint in `docs/endpoints.md` in the same change.
- Add request IDs, body/time limits, controlled CORS/security headers and sanitized error logging where absent.
- Add Utoipa/OpenAPI generation after deciding the contract ownership and compatibility policy.

## 4. Complete persistence and worker boundaries

- Configure explicit SQLx pool limits/timeouts and a fresh-database migration workflow.
- Select and document a durable Apalis queue backend and its retry, concurrency, delivery and idempotency semantics before registering production jobs.
- Keep AI worker dependencies optional and isolated; do not download model weights.

## 5. Add shared SDK and contracts

- Define versioned public request/response contracts separate from database models.
- Implement a small Rust client SDK and add UniFFI only when its exported API and target build matrix are ready.
- Generate and validate OpenAPI/client bindings from the chosen contract source.

## 6. Verify platform scaffolds

- Run Leptos SSR/hydration and browser startup checks on a Rust/WASM-capable runner.
- Run Android Gradle checks with the pinned wrapper, JDK, SDK and NDK versions.
- Run XcodeGen, XCTest and iOS/iPadOS/macOS widget builds on macOS.
- Build both Tauri applications on Linux, Windows and macOS and verify capabilities/package identities independently.
- Implement the Windows widget provider only against a supported Windows App SDK provider API and validate on Windows.
- Record GNOME/KDE/Apple widget verification separately; keep preview content static and privacy-safe.

## 7. Improve automation and security checks

- Pin third-party GitHub Actions to reviewed commit SHAs.
- Add Rust dependency/license, secret scanning, migration and API contract workflows without exposing privileged secrets to untrusted events.
- Add supported-runner workflows for every application; treat workflow definition as unverified until Actions actually runs.
- Validate Terraform locally when real module definitions exist; never apply or provision without authorization.
