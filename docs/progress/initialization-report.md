# Initialization report

Date: 2026-11-12

## Summary

The Peerfora checkout already contains the Rust/Axum authentication API and initial web, iOS, Android, student desktop, administrator desktop, and widget shells. This pass audited those foundations against the supplied PRD, the repository's master-specification alignment rules, and the pasted initialization specification. It corrected the global minimum-age enforcement and local PostgreSQL port exposure, and created a repository audit plus a concrete continuation list.

The repository is **not yet ready to claim full initialization or production readiness**. Most platform applications are scaffolds whose native builds have not been verified. Worker processing, shared SDK/UniFFI, generated OpenAPI, full API operational middleware, and several security/CI controls remain incomplete.

## Component status

| Component | Status | Evidence / limitation |
| --- | --- | --- |
| Repository and Ansofra layout | Implemented and inspected | Root `peerfora-api` crate; startup and layer flow documented. Workspace/`services/api` relocation is unresolved by current repository rules. |
| Authentication and social-signup API | Implemented, integration unverified | Routes, services, repositories, migration and endpoint reference exist; SMTP/OIDC/database runtime were not verified. |
| 18+ eligibility policy | Implemented, tests added but unverified | Global floor is enforced even when a country config is lower; Cargo could not resolve a missing offline package. |
| PostgreSQL | Partially implemented | SQLx pool and startup migration exist; no fresh/live DB migration check. Compose host port now binds loopback. |
| Redis/search/object storage/email | Scaffolded | Optional Compose profiles only; API adapters/readiness are not connected. |
| Worker / Apalis | Scaffolded or deferred | One cleanup-job module exists; no durable queue startup or worker binary. |
| AI worker | Deferred | No model dependencies or weights; separation is documented only. |
| API contracts/OpenAPI | Scaffolded | Contract folders and endpoint document exist; Utoipa generation absent. |
| Rust client SDK/UniFFI | Deferred | Architecture notes only; no SDK crate or bindings. |
| Web | Scaffolded; CSS verified | Tailwind asset build passed; Leptos compilation/startup not verified. |
| iOS/iPadOS | Scaffolded; native build blocked by host | Linux host lacks Xcode/Apple SDK. |
| Android | Scaffolded; native build unverified | Gradle wrapper exists; Android SDK/NDK build was not run. |
| Student desktop | Scaffolded; native build unverified | Tauri and Leptos CSR projects exist; no target build was run. |
| Admin desktop | Scaffolded; native build unverified | Separate project identity/config exists; administrative API/MFA implementation absent. |
| Apple widgets | Scaffolded; native build blocked by host | WidgetKit shells exist; no Xcode build. |
| Android widget | Scaffolded; build unverified | Glance module exists; no Android build. |
| Windows widget | Partial scaffold; not verified | Static Adaptive Card preview does not establish provider registration/runtime support. |
| GNOME/KDE widgets | Scaffolded; runtime unverified | Extension/applet files exist; no target desktop runtime validation. |
| Docker Compose | Implemented and verified for syntax | `docker compose config --quiet` passed. Services are development-only and API integrations are not wired. |
| Terraform | Scaffolded | Formatting passed; no provider/module definition or validation/apply. |
| CI | Partially implemented | Per-platform workflow YAML exists; no workflow execution, immutable action pinning, or security/migration suites. |
| Tests | Partially implemented | Two age-policy tests added; execution blocked before compilation by Cargo registry availability. |
| Documentation | Partially implemented | Audit, initialization report, next phase, endpoint/architecture/platform docs exist; some roadmap areas remain status notes. |

## Files created or updated in this pass

- Added `docs/progress/repository-audit.md`.
- Added `docs/progress/next-implementation-phase.md`.
- Added this initialization report.
- Updated `src/constants/account_auth.rs` and `src/services/accounts/commands/account_eligibility.rs` to enforce the global 18+ rule and add focused tests.
- Updated `docs/endpoints.md` to describe that rule.
- Updated `compose.yaml` so the PostgreSQL port binds to loopback by default.

## Commands and actual results

| Command | Result |
| --- | --- |
| `cargo metadata --no-deps --format-version 1` | Passed |
| `cargo fmt --all` and `cargo fmt --all -- --check` | Passed after the final Rust edit |
| `docker compose config --quiet` | Passed after the final Compose edit |
| `pnpm install --frozen-lockfile` | Passed |
| `pnpm --dir apps/web run build:css` | Passed with Tailwind CSS 4.3.5 |
| `terraform fmt -check -recursive infrastructure/terraform` | Passed |
| `git diff --check` | Passed |
| `cargo check --locked` | Interrupted while updating crates.io index; no compile result |
| `cargo clippy --all-targets --locked -- -D warnings` | Interrupted waiting on Cargo registry/cache lock; no lint result |
| `cargo test --locked` | Interrupted waiting on Cargo registry/cache lock; no test result |
| `cargo test --locked --offline` | Failed before compilation: local index/cache lacks `curve25519-dalek` required by `openidconnect` |

## Readiness

The current checkout is ready for continued engineering after Rust dependencies are available and the outstanding decisions/checklist in `docs/progress/next-implementation-phase.md` are handled. It is not yet a verified multi-platform release foundation or a production-ready service.
