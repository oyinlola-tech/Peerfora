# Implementation status

Updated: 2026-11-13

Peerfora is pre-launch. This page records source and test evidence in the current checkout; it does not certify production readiness or legal compliance. The PRD PDF is not present in the repository, so unresolved product and legal policy details are not inferred.

## Implemented slices

- Rust/Axum modular-monolith API using the Ansofra startup and layered request flows.
- Student email/password registration, email confirmation, password reset, opaque hashed sessions, Google/Apple OIDC integration points, and pending social signup requiring birthday and eligibility completion.
- Global self-declared minimum age of 18 with stricter-only country overrides; date of birth is not persisted.
- Country/territory search, starter academic-level and subject search, community create/discovery/membership, profiles and preferences.
- Dedicated administrator identities and credentials separate from student accounts; one migration-seeded, credential-free super-administrator root activated by a one-time local prompt command.
- Permission-gated administrator invitations with 48-hour expiry, single-use hashed tokens, email delivery, atomic activation/role assignment/audit, dynamic role selection, and effective-permission delegation. Super Admins may invite additional Super Admins.
- Administrator invitation, email verification, and password reset copy is built by dedicated plain-text email templates. They explain the purpose, link lifetime, next steps, and what to do with an unexpected email; reset copy also explains session revocation. Templates have no product color styling pending approved brand tokens.
- Runtime bind, port, logging, SMTP-port and public-origin defaults use named constants, while deployment-specific public and OAuth callback URLs continue to come from environment-backed settings. Email links use named route and one-time-token query constants; provider issuer URLs are named protocol constants. Test URLs are named fixtures rather than embedded into assertions.
- Administrator password login with five-failure/15-minute per-account lockout; five-minute one-use MFA challenges; mandatory TOTP enrollment and verification using `totp-rs`. Five MFA failures across separate challenges also trigger a 15-minute administrator-level lockout.
- Shared PostgreSQL fixed-window rate limits for registration, student/admin login, and student/admin password-reset requests. HMACs of normalized email identifiers are stored using the required `PEERFORA_AUTH_RATE_LIMIT_HASH_KEY`; no raw email is written to the rate-limit table. These limits work across API instances and return `429` after their documented thresholds. The hash key must stay stable across API instances; rotating it clears the effectiveness of active identifier buckets.
- Credentialed CORS is restricted to configured HTTP(S) origins. State-changing requests that carry a Peerfora session cookie require a matching `Origin`; disallowed or duplicate origins return `403`. Bearer-only clients may omit the origin header.
- Whole-request processing is bounded by `PEERFORA_REQUEST_TIMEOUT_SECONDS`, default 30 seconds. Timeout responses use the stable `504` application error.
- Unknown-email and social-only password-login attempts run Argon2id verification against a process-cached throwaway hash before returning the generic credentials error. Password-reset request timing can still differ because known-address delivery remains synchronous.
- TOTP secrets encrypted using AES-256-GCM and `PEERFORA_ADMIN_MFA_ENCRYPTION_KEY`; enrollment returns ten one-time recovery codes once, stores only their SHA-256 hashes, and consumes a code atomically with login session creation. Administrators can replace unused recovery codes after recent-session and current-TOTP verification; the old codes are explicitly revoked and the new plaintext is returned once with `Cache-Control: no-store`. Password change verifies the current credential, stores a new Argon2id hash, invalidates sessions, pending MFA challenges, and outstanding password-reset links, and clears the session cookie. Administrator password recovery uses expiring hashed one-use links, revokes sessions and pending MFA challenges transactionally, and retains mandatory MFA. MFA reset requires a different administrator with `admin.accounts.manage` permission and recent reauthentication; it clears the target factor, revokes sessions, pending challenges, and unused recovery codes, audits the reason, and requires new TOTP enrollment before issuing another session. Session/challenge/reset-token hashes are persisted; TOTP steps cannot be reused.
- Administrator sessions use a distinct HttpOnly, SameSite=Strict cookie, eight-hour absolute expiry, 20-minute idle timeout, and database revocation. Student cookies and bearer tokens do not authorize admin routes.
- Administrators can list and revoke their own sessions. Authorized administrators can also revoke every session and consume pending MFA challenges for another administrator after recent authentication; this operation locks target identity state, rechecks permission, and audits the reason atomically.
- Permission-gated administrator suspension, reinstatement, and logical deletion require MFA-authenticated reauthentication within 15 minutes. These changes lock current authorization state, revoke affected sessions/challenges and open invitations, redact deleted identity credentials, preserve append-only audit attribution, and protect the migration-seeded root. Suspension/deletion refuses to remove the last active MFA-enabled Super Admin. Role assignments now retain revocation actor, time, and reason; deleted identities lose all effective role assignments.
- Seeded role/permission catalogue plus custom-role create/update/deactivate/logical-delete, assignable permission selection, role assignment replacement, effective-permission inspection, and delegated-permission checks. System roles are immutable through the API; the seeded root identity/role assignment are additionally protected by database triggers. Role changes are transactional and audited. Audit-log reads require `audit_logs.read`; denial and read events are audited. An update/delete trigger blocks ordinary audit-row mutation.
- Permission-gated `GET /api/v1/admin/overview` returns actual stored account, institution, and community aggregates only. It does not label account totals as active-user analytics or fabricate metrics for missing content/verification tables.
- Permission-gated `GET /api/v1/admin/accounts` and `GET /api/v1/admin/accounts/{id}` provide bounded email/username prefix search and lifecycle, country, and academic-level filters. Responses omit credentials, date of birth, tokens, and eligibility evidence.
- `POST /api/v1/admin/accounts/{id}/suspend` and `/reinstate` enforce permanent account suspension and reinstatement. Suspension transactionally changes lifecycle state, revokes active sessions and reset/verification tokens, and writes an audit record; reinstatement requires a fresh student login. A separate permission-gated operation revokes all current student sessions and audits the reason atomically.
- Permission-gated community search/detail return metadata only. Suspend/restore transitions are transactional, append audit events, and cause suspended communities to leave public discovery and reject joins.
- `GET /health` liveness, `GET /health/ready` PostgreSQL readiness with a two-second query timeout, 1 MiB request-body cap, baseline security response headers, and server-generated UUID request correlation IDs returned in `X-Request-Id` and attached to administrative audit records.

## Administrative API implemented

- `POST /api/v1/admin/auth/login`
- `POST /api/v1/admin/auth/password/change`
- `POST /api/v1/admin/auth/password-reset/request`
- `POST /api/v1/admin/auth/password-reset/confirm`
- `POST /api/v1/admin/invitations`
- `POST /api/v1/admin/invitations/accept`
- `GET /api/v1/admin/roles`
- `POST /api/v1/admin/roles`
- `GET /api/v1/admin/roles/{id}`
- `PATCH /api/v1/admin/roles/{id}`
- `DELETE /api/v1/admin/roles/{id}`
- `GET /api/v1/admin/permissions`
- `GET /api/v1/admin/roles/{id}/permissions`
- `PUT /api/v1/admin/roles/{id}/permissions`
- `GET /api/v1/admin/administrators/{id}/roles`
- `PUT /api/v1/admin/administrators/{id}/roles`
- `GET /api/v1/admin/administrators/{id}/permissions`
- `POST /api/v1/admin/administrators/{id}/suspend`
- `POST /api/v1/admin/administrators/{id}/reinstate`
- `DELETE /api/v1/admin/administrators/{id}`
- `DELETE /api/v1/admin/administrators/{id}/sessions`
- `POST /api/v1/admin/administrators/{id}/mfa/reset`
- `POST /api/v1/admin/auth/mfa/enroll`
- `POST /api/v1/admin/auth/mfa/confirm`
- `POST /api/v1/admin/auth/mfa/verify`
- `POST /api/v1/admin/auth/mfa/recovery-codes/regenerate`
- `GET /api/v1/admin/auth/me`
- `POST /api/v1/admin/auth/logout`
- `GET /api/v1/admin/sessions`
- `DELETE /api/v1/admin/sessions/{id}`
- `GET /api/v1/admin/overview`
- `GET /api/v1/admin/audit-logs`
- `GET /api/v1/admin/accounts`
- `GET /api/v1/admin/accounts/{id}`
- `POST /api/v1/admin/accounts/{id}/suspend`
- `POST /api/v1/admin/accounts/{id}/reinstate`
- `POST /api/v1/admin/accounts/{id}/sessions/revoke`
- `GET /api/v1/admin/communities`
- `GET /api/v1/admin/communities/{id}`
- `POST /api/v1/admin/communities/{id}/suspend`
- `POST /api/v1/admin/communities/{id}/restore`

Requests, responses, permissions, cookie behavior and errors are documented in `docs/endpoints.md`.

## Requirement coverage snapshot

| Product area | Current state |
| --- | --- |
| Student authentication and age eligibility | Source flows compile; login, registration, and password-reset limits are shared through PostgreSQL; provider and SMTP operation depends on external configuration. |
| Social signup birthday gate | Backend completion requires birthday/eligibility before account/session creation; client page is not connected here. |
| Profiles/preferences | Self-service and public privacy projections exist; export/deletion and explicit-null PATCH clearing are missing. |
| Global directory | Country and starter academic lookup exist; institution/course management and suggestion review are absent. |
| Communities | Create/discover/join/leave plus permission-gated admin metadata search, suspension, and restoration exist; private invite/review, ownership transfer, and moderator workflows are absent. Admin community metadata excludes private content. |
| Administrator identity/MFA | Bootstrap, permission-gated invitations and activation, password challenge, TOTP enrollment/verification/reset, recovery-code replacement, session cookie, logout, current identity, own-session listing/revocation, cross-administrator session revocation, password change and recovery, suspension/reinstatement, and logical deletion exist. |
| RBAC/audit | Dynamic custom role/permission APIs, active-role permission resolution, current-permission delegation limits, protected system roles/root, permission-gated administration routes, transactional assignment, lifecycle management, and audit events are implemented. Resource-scoped delegation remains absent. |
| Accounts/content/trust and safety | Account search/details use `accounts.read`; permanent suspension/reinstatement use `accounts.suspend`; all-current-session revocation uses `accounts.sessions.revoke`. Individual session listing, temporary restrictions/expiry, security-history/report views, posts/comments, reports, cases, other enforcement, and appeals are absent. |
| Verification/privacy | Student/institution verification, evidence storage, export/deletion requests and retention processing are absent. |
| Global administration | Country-policy and academic-directory administrative workflows are absent. |
| Overview/operations | Limited stored account/institution/community overview counts exist; no date/country/academic/community filters, content/verification metrics, durable outbox worker, or production telemetry. |
| Platform shells | Web/mobile/desktop/widgets remain scaffolds; admin Tauri is not connected and frontend designs remain frozen. |

## Database changes

The core migration is split into `202611130001_create_peerfora_core.sql` and `202611130002_create_community_and_account_support.sql`; the user confirmed the original had never been applied, so later migration versions were shifted one place to preserve SQLx order. The forward migrations `202611130003_create_administrator_identity.sql` through `202611130014_create_administrator_password_reset_tokens.sql` add separate administrator identities/RBAC/audit, administrator-wide MFA lockout, account administration indexes, expiring invitations, suspended community status, hashed one-use admin MFA recovery codes, the credential-free pending root identity, dynamic role metadata and assignment safeguards, sensitive account-session revocation permission, administrator lifecycle protections, role-assignment history, recovery-code replacement revocation, and one-use administrator password reset links. `202611130015_create_authentication_rate_limits.sql` adds shared fixed-window buckets for high-risk email-address authentication operations. The bootstrap command activates that fixed root under a PostgreSQL advisory transaction lock and cannot be repeated. It prompts for email and a hidden password rather than reading credentials from environment variables. Custom role/permission changes and administrator role assignment are transactional and audited; a shared lock and effective-permission subset checks prevent delegation beyond the actor's current permissions. Account suspension uses the existing lifecycle field and appends to administrative audit logs; it deletes existing student sessions and one-time account recovery tokens in the same transaction. The separate session-revocation command locks the account before deleting sessions, serializing against session creation, and writes its audit event in the same transaction. Community status changes use audit transactions and preserve the existing owner relationship.

The existing account/community/profile schema remains as previously implemented. The country reference snapshot contains 249 entries. No production database was accessed.

## Foundational defects corrected during this continuation

- Restored Cargo dependency resolution over the reachable crates.io registry. The old lockfile did not match the current manifest; Cargo resolved and fetched the graph, including `der` from the `openidconnect` cryptography chain, and updated `Cargo.lock` without removing security dependencies.
- Fixed OIDC 4.x endpoint state typing, authorization flow import, optional token endpoint handling, and verified-email comparison.
- Fixed a cookie format string that could not compile, corrected the Axum `DefaultBodyLimit` import, and fixed country-policy environment parsing.
- Fixed Clippy failures for duplicate-key handling and a positional profile update function by passing a cohesive update record.
- Added bounded PostgreSQL readiness checks and tested liveness/readiness separately.

## Verification actually run

- `cargo fetch --locked` passed after Cargo resolved the manifest/lockfile mismatch; crates.io was reachable. `der` and the remaining graph were downloaded from the configured crates.io registry.
- `cargo check --locked --all-targets` passed after adding the authentication rate-limit dependency and configuration.
- `cargo clippy --locked --all-targets -- -D warnings` passed after adding configured CORS, session-cookie origin enforcement, and request timeouts.
- `cargo clippy --locked --all-targets -- -D warnings` passed.
- `cargo test --locked` passed all 25 unit tests. Without `PEERFORA_TEST_DATABASE_URL`, the PostgreSQL integration harness returns early; its pass alone does not mean database checks ran.
- `PEERFORA_TEST_DATABASE_URL` set to an isolated disposable PostgreSQL 17 database with `cargo test --locked` passed: 25 unit tests and the PostgreSQL integration suite. The integration suite exercised fresh migrations, seeded-root verification and one-time activation, bootstrap repeat rejection, TOTP enrollment, recovery-code use/replay rejection and replacement/revocation, invitation issue/acceptance/replay rejection, mandatory MFA after invitation activation, password and MFA lockout, authentication rate-limit enforcement, CORS allowlist behavior, disallowed-origin rejection, cookie-session requests without Origin rejection, configured request-timeout behavior, dynamic role creation and assignment, delegation limits, seeded-root and reserved-role protections, administrator suspension/reinstatement/deletion, own and cross-administrator session revocation, pending MFA challenge invalidation, independent MFA reset and self-reset rejection, reset state revocation and audit, permission denial, administrator password change and recovery, reset-token replacement/replay rejection, session/challenge/token revocation, mandatory MFA after password recovery, credential redaction, protected-root status enforcement, permission-gated account/community operations, transactional account suspension/reinstatement, community suspension/restoration, audit persistence and append-only rejection, expired/revoked sessions, malformed/oversized JSON, security headers, server-generated request correlation IDs, and readiness/liveness behavior.
- `cargo fmt --all -- --check`, `git diff --check`, and the repository-wide 400-line file check passed after the password recovery, MFA reset, recovery-controller split, and request-correlation changes.
- `bash scripts/check_source_file_line_limits.sh` checks source and configuration formats across the whole repository, including infrastructure and root build files, while excluding generated build trees, data files, and dependency lockfiles. All checked files are at or below 400 lines. The API CI workflow runs the line-limit check.
- `cargo-audit` and `cargo-deny` are not installed. No external security assessment or production-provider verification was performed.

The PostgreSQL instance was disposable and local. It did not contain production data; its container was stopped after validation. The reset email integration test used the recording SMTP adapter, so live email delivery was not verified.

## Remaining Phase Three work

1. Harden the initial administrator boundary further with password-reset request timing equalization. Reset email delivery is currently synchronous, and failed-delivery handling was not exercised against a live SMTP provider. MFA reset currently uses a single separately authorized administrator plus recent reauthentication, without an independent second approver.
2. Extend the overview with approved date, country, academic-level, and community-category filters after defining their metric scope.
3. Extend account search/details with security history/reports, granular restrictions mapped only to existing student operations, temporary suspension expiry, and account session listing.
4. Add community ownership transfer, ownerless/abandoned handling, community reports, and any private-content review under a separate explicit permission.
5. Implement posts/comments before report, moderation-case, enforcement, and appeal workflows; each action needs transactional audit and regression coverage.
6. Implement student/institution verification only with controlled evidence storage and access/retention safeguards.
7. Add academic-directory suggestions/administration and approved country policy workflows while preserving the 18+ floor and international coverage.
8. Add privacy export/deletion lifecycle and the durable jobs required by actual workflows.
9. Extend rate limiting to social sign-in, MFA challenges, and network-level abuse signals; add outbox processing, and operational health/telemetry.
10. Add broader real-PostgreSQL concurrency and authorization tests, dependency security tooling, OpenAPI generation, and update docs as each API is added.

Phase Three remains incomplete. Phase Four readiness is not established. No frontend design was changed, and no commit, push, deployment, or production database operation was performed.
