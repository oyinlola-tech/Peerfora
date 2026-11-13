# Implementation status

Updated: 2026-11-13

Peerfora is pre-launch. This document records source and test evidence in the current checkout; it does not certify production readiness or legal compliance. The Peerfora PRD PDF is not present in the repository, so unresolved product and legal policy details are not inferred.

## Implemented slices

- Root Rust/Axum modular-monolith API following the Ansofra startup and layered request flows.
- Email/password registration and login, email confirmation, password reset, opaque hashed sessions, Google/Apple OIDC integration points, and pending social signup that requires the Peerfora birthday/eligibility step.
- Global self-declared 18+ eligibility with stricter-only country policy overrides; DOB is not stored.
- Country/territory reference search, starter academic-level and subject search, community create/discovery/membership, profile self-service and preferences.
- Account lifecycle status and session/social/account lookups blocked for non-active accounts; password-or-social-identity DB invariant; eligibility evidence without DOB.
- Two migrations, versioned 249-country JSON importer, community ownership consistency, profile privacy defaults, audit/outbox writes for community changes.
- `GET /health` liveness and `GET /health/ready` PostgreSQL query readiness. JSON bodies are capped at 1 MiB and standard security response headers are applied.

## Requirement coverage snapshot

| Product area | Current state |
| --- | --- |
| Authentication and 18+ eligibility | Source flows exist; provider/runtime and Rust compilation remain unverified. |
| Social signup birthday gate | Backend completion route requires birthday/eligibility before account/session creation; frontend page is not connected. |
| Profiles/preferences | Self-service routes and privacy projection exist; export/deletion and clear-null PATCH semantics are missing. |
| Global directory | Country and starter academic queries exist; institutions/courses are schema-only, suggestions have no workflow. |
| Communities | Create/discover/join/leave exist; private invite/review, roles, ownership transfer and moderator operations are absent. |
| Discussions and moderation | Posts, comments, votes, reputation, bookmarks, feeds, reports, enforcement and appeals are absent. |
| Verification and administration | Student/institution verification, admin identities/MFA/RBAC, audit APIs and overview APIs are absent. |
| Platform shells | Web, mobile, desktop and widgets are scaffolds; native and browser runtime builds are not established. |
| Infrastructure/contracts | PostgreSQL is used; Redis, durable queue, generated OpenAPI, shared SDK and production observability are not connected. |

## Not implemented

- Dedicated administrator identities, provisioning/bootstrap, password recovery, mandatory MFA, admin sessions, RBAC, scopes, delegation controls, administrative APIs, dashboard metrics, or admin audit log.
- Reports, moderation cases/actions, appeals, posts/comments, student/institution verification, country-policy administration, feature-flag administration, data export/deletion workflows, or durable job processing.
- Directory suggestion submission/review endpoints, institution management APIs, complete institution/course datasets, community approval/moderator/ownership-transfer APIs.
- Rate limiting, CSRF/origin enforcement, request timeouts, correlation IDs, OpenAPI generation, outbox worker, SDK, Redis integration, or production telemetry. Security headers, request-size limits and PostgreSQL readiness were added but not compiled or exercised via HTTP tests.

The administration Tauri app remains a separate development placeholder and is not connected to student credentials. No privileged endpoint has been exposed. Admin login/MFA should not be opened until an established TOTP/WebAuthn implementation, secret-encryption key management, recovery policy, and compile/test path are available.

## Verification actually run

- Both SQL migrations applied to a disposable PostgreSQL 17 database.
- Three SQL smoke scenarios passed after loading the country snapshot. Checks cover starter directory and 249 countries, profile/preference privacy defaults, account eligibility evidence, case-insensitive email uniqueness, password-or-social identity enforcement, and community ownership.
- `cargo fmt --all -- --check`, `cargo metadata --no-deps --format-version 1`, `git diff --check`, `bash -n scripts/refresh_country_directory.sh`, `docker compose config --quiet`, and country snapshot checks passed.
- `cargo check --locked --offline`, `cargo clippy --locked --all-targets --offline -- -D warnings`, and `cargo test --locked --offline` could not resolve the existing OIDC dependency tree because `der` is missing from the local Cargo cache. A 25-second online `cargo check --locked` retry timed out while updating the crates.io index. Rust unit/API tests, Clippy, OpenAPI validation, and platform builds were not verified. `cargo-audit` and `cargo-deny` are not installed.

The PostgreSQL container was disposable and removed after the SQL checks. No production database or external system was accessed.

## Prior audit evidence retained

Earlier repository audits identified: Phase Two remains incomplete; community membership review and moderation do not exist; posts, comments, votes, feeds, bookmarks, and content reporting are absent; academic entries are starter data; Redis, Apalis, OpenTelemetry, generated OpenAPI, and SDKs are not connected; frontend projects are shells; and iOS builds require macOS/Xcode. Cargo dependency resolution has blocked compilation repeatedly. These earlier findings have been consolidated here rather than kept as multiple phase reports.

## Next priorities

1. Restore locked Cargo dependency access; run compile, Clippy, Rust tests, and PostgreSQL-backed HTTP tests, fixing failures first.
2. Complete core discussion content and authorization foundations needed for report/moderation workflows.
3. Select and implement dedicated admin authentication with mandatory MFA, encrypted TOTP secret handling, controlled bootstrap/invitation, and recovery safeguards before exposing privileged routes.
4. Add explicit RBAC, report/case/action and appeal workflows with transactional audit records and regression coverage.
5. Implement verification and privacy lifecycle only after evidence storage, access, retention, and policy decisions are approved.
6. Add operational controls (shared rate limiting, CSRF/origin policy, request limits/timeouts, readiness, observability) and update this document with measured verification.

Phase Three is not complete, and Phase Four readiness is not established.

## Historical verification notes

The earlier repository initialization pass recorded successful `cargo metadata --no-deps`, `cargo fmt`, Compose configuration, frozen web package installation, Tailwind CSS generation, Terraform formatting, and `git diff --check`. These checks covered configuration/syntax only; they did not establish client runtime behavior, production infrastructure, or full Rust compilation. The repository has consistently failed to reach Rust compilation when the local Cargo cache lacks the locked `der` dependency, and online index resolution stalled.
