# Peerfora repository audit

Audit date: 2026-11-12

## Scope and sources

This audit inspected the current Git tree and working changes, root `AGENTS.md`, the codebase-alignment layer in `docs/master-engineering-specification.md`, the architecture and endpoint references, the platform and implementation reports, root and app manifests, Compose, CI workflows, migrations, and the supplied `Global_Student_Community_PRD_v1.pdf` attachment. The PRD is available as an external attachment and is not currently stored in the repository. Its working product name is Student Commons; the user's explicit product identity decision is Peerfora. Product-facing repository references use Peerfora.

The working tree already contained user changes before this audit. They were preserved. No branch changes, commits, pushes, deployments, or infrastructure provisioning were performed.

The full master prompt asks for a root Cargo workspace and a `services/api` location. Current repository instructions explicitly keep the API crate at the root and prohibit that migration without a separate approved architecture change. The repository-alignment overlay also says to apply the master prompt as a phased roadmap. This audit therefore records that structural divergence as requiring an architectural decision rather than moving the existing API in this pass.

## Findings

| Area | Existing implementation and expected state | Status | Severity | Correction applied | Remaining limitation |
| --- | --- | --- | --- | --- | --- |
| Repository safety | Existing main branch and uncommitted user work; preserve it | Compliant | Informational | Kept all existing changes; made no Git history changes | Working tree remains uncommitted by design |
| Product identity and source | Repository uses Peerfora; supplied PRD calls it Student Commons as a working name | Partially compliant | Low | Audit records provenance and follows Peerfora naming decision | PRD PDF is not checked into `docs/product/` |
| Rust/API layout | Root `peerfora-api` Axum crate follows Ansofra startup and layered flow | Compliant with current repository rules | Architectural decision | Retained root crate and current `src/` boundaries | Full prompt's workspace/`services/api` target conflicts with current approved alignment |
| Authentication | Registration, login, email confirmation, password recovery, social OIDC, pending social signup, and session routes exist | Partially compliant | Medium | Enforced the PRD's global 18+ floor even if a country config sets a lower age | Provider, SMTP, and PostgreSQL behavior still depends on local configuration; no auth tests existed before this audit |
| Eligibility policy | Country policies are typed and registration rejects missing country rules | Partially compliant | High | Added `MINIMUM_ACCOUNT_AGE_YEARS`; country rules can now raise but not lower the 18+ threshold; added focused unit tests | Age-assurance provider is not implemented; policy review/version governance remains future work |
| Endpoint documentation | `docs/endpoints.md` catalogs the registered HTTP endpoints, request fields, cookies, success and error behavior | Partially compliant | Medium | Updated country-policy documentation for the global age floor | API uses legacy `GET /health`; `/health/live`, `/health/ready`, `/api/v1/status`, Utoipa, and generated OpenAPI are absent |
| Database | SQLx PostgreSQL pool and one authentication migration exist; API startup applies migrations | Partially compliant | Medium | No migration changed | Pool limits/timeouts are mostly defaults; no live database or fresh-migration check was run |
| Background jobs | One cleanup job module is wired by the loader | Scaffolded | Medium | None | No Apalis queue backend, standalone worker application, durable delivery, retry policy, or worker health endpoint |
| Shared contracts and SDK | Contract directories and architecture notes exist | Scaffolded | Medium | None | No shared Rust SDK, UniFFI crate/bindings, generated OpenAPI, event schemas, or contract checks |
| Web | Leptos SSR/hydration shell, route placeholders, Tailwind CLI and PWA files exist | Scaffolded | Medium | CSS build was run successfully | Leptos build was not verified; no Playwright setup or backend SDK integration |
| iOS/iPadOS | SwiftUI project specification, API/credential/deep-link boundaries and WidgetKit shell exist | Scaffolded | Medium | Swift syntax checks were run in the preceding initialization pass; no Xcode build in this audit | Xcode, Apple SDK builds, XCTest, App Intents, persistence and UniFFI are not verified/implemented |
| Android | Compose app/core/widget Gradle modules, version catalog and wrapper exist | Scaffolded | Medium | Android configuration is present | Gradle/SDK build and UI tests were not run; Room, secure storage, WorkManager, FCM and UniFFI remain absent |
| Student/admin desktop | Two independent Tauri 2 + Leptos CSR projects and app identities exist | Scaffolded | Medium | Kept identities/configurations separate | Native builds, secure credential plugins, notifications, update signing, and admin MFA/API foundations remain unverified or absent |
| Widgets | Static/native project shells exist for Apple, Android, Windows and Linux | Partially compliant | Medium | Recorded platform limits in existing docs | Windows provider registration/build support is not established; Linux/Apple builds and runtime behavior unverified; no live private data contract |
| Local infrastructure | Compose defines PostgreSQL plus optional Redis, Meilisearch, S3Mock and Mailpit profiles | Partially compliant | Medium | Bound PostgreSQL's host port to loopback by default | API does not connect to optional services; PostgreSQL is the only default service; no OTEL collector or worker queue |
| Terraform | Directory and status notes exist | Scaffolded | Low | Terraform formatting check passed | No provider/module/environment implementation or `terraform validate` target exists |
| Security | Safe `.env.example`, request logging and Tauri CSP/capability boundaries exist | Partially compliant | High | Reduced default PostgreSQL host exposure; enforced 18+ floor | No complete middleware suite, rate limiting, vulnerability/secret scan CI, OpenTelemetry redaction audit, or completed threat controls |
| Tests | Rust source had no test functions before this audit; test directory has status notes | Partially compliant | High | Added two eligibility behavior tests | Cargo test execution is blocked by unavailable registry/index package data in this environment |
| CI | Separate API, web, iOS, Android and desktop/widget workflows exist | Partially compliant | Medium | No workflow edits in this pass | Actions use mutable version tags; no CI run was triggered; security, migration, contract, and Linux/macOS/Windows coverage gaps remain |
| Developer commands | `justfile` exposes setup/dev/check/test and platform commands | Partially compliant | Low | No change | Worker, OpenAPI and binding commands currently report that those implementations are not configured |
| Dependency compatibility | Root Cargo lockfile and Rust 2024 edition exist; platform dependency files are present | Partially compliant | Medium | Retained locked dependency state | `cargo check`, Clippy, and tests could not reach compilation because Cargo attempted to update the index; offline resolution lacks `curve25519-dalek` |
| Documentation and governance | Root rules, architecture, endpoint docs, platform notes, threat model, ADRs and reports exist | Partially compliant | Medium | Added this audit, a continuation checklist, and an initialization report | Several roadmap documents are status notes, not implemented technical designs |

## Prioritized corrections

1. **Done:** enforce the explicit 18+ launch rule independently of country-specific policy values.
2. **Done:** keep the local PostgreSQL development port on loopback by default.
3. **Done:** document these facts and the exact current-vs-target implementation state.
4. **Next:** run Cargo checks in an environment with the missing registry packages; address any actual compile/lint/test failures.
5. **Next:** decide whether the repository should retain its root API crate or migrate to the full prompt's workspace/`services/api` layout. The migration affects paths, lockfiles, CI, Docker, docs, commands, and the established Ansofra flow.
6. **Next:** add accurate liveness/readiness/status routes and complete endpoint documentation before adding OpenAPI generation.
7. **Next:** implement PostgreSQL health and migration verification, then a durable worker/queue using a selected backend.
8. **Next:** initialize real SDK/UniFFI and contract generation boundaries only after their ownership/API surface is decided.
9. **Next:** verify native platforms on their supported runners and add pinned CI actions plus security/migration checks.

## Validation evidence

Passed during this audit:

- `cargo metadata --no-deps --format-version 1`
- `cargo fmt --all` and `cargo fmt --all -- --check` after the age-policy change
- `docker compose config --quiet` after binding PostgreSQL to loopback
- `pnpm install --frozen-lockfile`
- `pnpm --dir apps/web run build:css`
- `terraform fmt -check -recursive infrastructure/terraform`
- `git diff --check`

Blocked or not run:

- `cargo check --locked`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked` were started but remained waiting while Cargo updated the crates.io index and shared package-cache lock. They were interrupted rather than reported as passing.
- `cargo test --locked --offline` failed before compilation because the local registry index/cache lacks `curve25519-dalek`, required by the locked `openidconnect` dependency graph.
- The newly added tests therefore remain unexecuted in this environment.
- No live PostgreSQL, SMTP, Google/Apple OIDC, Android, Xcode, Tauri, Windows widget, or Terraform cloud validation was run.
- No CI workflow was triggered.
