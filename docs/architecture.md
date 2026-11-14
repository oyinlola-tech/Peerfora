# Peerfora API architecture

## Startup

`main.rs → index.rs → server.rs → app.rs → loaders/`

`main.rs` calls the index entry point. The index starts the server. The server reads typed settings, connects PostgreSQL, applies migrations, installs the versioned country snapshot and validates dependent country foreign keys, wires configured adapters, and starts the listener. `/health/ready` runs a database query; `/health` remains process liveness. The app builds the router. Loaders register routes, middleware, application state, and background jobs.

## Request flow

`client → route → middleware → validation → controller → service → repository → model → database`

The route chooses an HTTP handler. Middleware handles cross-cutting request behavior. Validators check incoming data shape and syntax. Controllers handle HTTP and call a named account use case. Services enforce product rules. Repositories own SQL and transactions. Models describe persisted records. DTOs define the public request and response shape.

## Folder map

| Folder | Responsibility | Current Peerfora use |
| --- | --- | --- |
| `configs/` | Parse environment variables into typed settings | PostgreSQL, cookie behavior, country policy, SMTP, Google, Apple, and authentication rate-limit key, allowed origins, and request timeout settings |
| `constants/` | Fixed application values | Health, account-auth, administrator-auth, and rate-limit settings; token lifetimes, cookie names, input limits |
| `controllers/` | HTTP request and response handling | Health, student auth, administrator auth/audit/account operations, directory, communities, profiles, and preferences |
| `database/` | PostgreSQL pool, migration startup, and pinned reference-data installation | Connects to PostgreSQL, applies SQL migrations, imports the versioned country snapshot, and validates country foreign keys before serving |
| `middlewares/` | Cross-cutting request behavior | Server-generated request correlation IDs, request logging, security response headers/body limits, configured CORS, cookie-session origin checks, request timeout, admin session and permission guards |
| `dtos/` | API wire shapes | Student/admin auth, audit, directory, community, profile, and preference requests |
| `enums/` | Finite domain values | Account/admin auth, authentication rate-limit operations, directory, community, profile failures, and provider identity |
| `errors/` | Error types and HTTP mapping | Stable domain errors, including admin auth/permission failures, rate limits, and safe database/setup responses |
| `interfaces/` | Contracts at real external boundaries | `EmailSender` separates account use cases from SMTP transport |
| `integrations/` | External service adapters and integration-specific message templates | SMTP implementation of `EmailSender` and detailed neutral email verification, password reset, and administrator invitation copy, kept separate from use-case logic until brand tokens are approved |
| `jobs/` | Independent background work | Remove expired auth, reset, session, and social-flow records |
| `loaders/` | Startup wiring | Routes, request logging, and the expired-token job |
| `middlewares/` | Cross-cutting request behavior | Request method/path/status and correlation ID logging without query strings |
| `models/` | Persisted data and shared app state | Student/admin identities, audit entries, directory, community, profile, pool and adapters |
| `repositories/` | SQL reads, writes, and transactions | Account/admin auth, account administration, admin RBAC/audit, subject search, community, profile, and preference storage |
| `routes/` | HTTP method/path mapping | Health, student/admin auth and account search, directory, community, and profile routes |
| `services/<domain>/commands/` | State-changing domain use cases | Student/admin authentication, bootstrap, invitations, MFA, account suspension/reinstatement, session revocation, community membership, profile and preference updates |
| `services/<domain>/queries/` | Read-only domain use cases | Student/admin session lookup, audit/account administration search, academic subject search, community discovery, and profile retrieval |
| `utils/` | Stateless reusable helpers | Argon2, one-time token hashing, TOTP construction, authenticated secret encryption, request context, and language/email normalization |
| `validations/` | Incoming data syntax checks | Email, date format, and two-letter country-code validation |

`communities`, `academic_directory`, and `profiles` contain initial Phase Two use cases. Profiles and preferences are separate from authentication records; public and own-profile query shapes differ to protect private account data. `discussions`, `discovery`, `moderation`, `notifications`, `institutions`, `memberships`, `voting`, `reputation`, `feeds`, and `bookmarks` remain planned domains with no implemented use cases. The community repository writes audit and outbox records transactionally, but there is no outbox worker yet. The background job currently handles auth cleanup; email remains request-driven until a durable delivery queue is implemented. API JSON bodies are capped at 1 MiB. Responses receive no-store, content-type, referrer, framing, and content-security headers; HSTS is emitted only when `PEERFORA_ENV=production`.

## Account-auth flow

- Email/password registration checks terms, username, date of birth, configured country eligibility, password length, and email syntax. The date of birth is not persisted. Account creation records the country and policy version. Email confirmation is sent through the configured `EmailSender` adapter.
- Email/password login requires a confirmed email. Session tokens are opaque, generated from secure randomness, and only their SHA-256 hashes are persisted. Browser sessions use HttpOnly cookies; bearer tokens are accepted for API clients.
- Registration, student/admin password login, and student/admin password-reset requests use PostgreSQL-backed fixed-window limits keyed by an HMAC of normalized email. The HMAC key is required in typed configuration so database rows do not retain raw email addresses; failed PostgreSQL checks reject the request. Student and administrator unknown-email login paths verify against a process-cached Argon2id dummy hash to reduce account-enumeration timing differences.
- Credentialed CORS uses an exact HTTP(S) allowlist from `PEERFORA_ALLOWED_ORIGINS`, defaulting to the configured public origin. Middleware rejects supplied disallowed origins and requires an allowed origin on state-changing requests that carry a Peerfora session cookie. Bearer-only API clients may omit `Origin`.
- A configurable request timeout defaults to 30 seconds and covers route processing, including request body extraction, authorization, and database work. Readiness retains its shorter database-query timeout.
- Password-reset links are single-use, expire after one hour, and revoke all account sessions when used.
- Google and Apple use OpenID Connect authorization code flow with state, nonce, and PKCE. Provider issuer, signature, audience, expiry, and nonce validation are performed by the OIDC library. Provider email must be verified. Email matches do not automatically link identities.
- A first-time social identity is held as a short-lived pending signup. The browser is redirected to `PEERFORA_PUBLIC_URL/complete-social-signup`. It must POST its birthday, country, username, and terms acceptance to `/api/v1/auth/social/complete`. The API creates no account or session until those checks pass. Provider birthday claims do not bypass this page.
- Country policies are supplied as JSON in `PEERFORA_COUNTRY_POLICIES`; no country is enabled by default. A configured policy cannot lower the global 18+ floor. A policy requiring age assurance cannot pass until its provider is implemented and configured.

## Administration and access-control boundary

Administrator identities and password hashes live in a separate `administrators` table. A forward migration seeds one pending root identity without credentials and assigns it the super-administrator role. The one-shot `provision_first_administrator` binary activates that fixed identity after taking a database advisory lock; it prompts locally for credentials, with password entry hidden. Student account records and student session cookies/tokens are never consulted by admin authentication.

Password authentication yields only a five-minute, one-use MFA challenge. A new admin can use that challenge only for TOTP enrollment; an enrolled admin must verify TOTP. TOTP secrets are encrypted with AES-256-GCM using a deployment-managed, base64-encoded 32-byte key. Successful MFA consumes the challenge and TOTP step, records an audit entry, then creates an opaque admin session whose SHA-256 hash is stored in PostgreSQL. The separate HttpOnly, SameSite=Strict admin cookie has an eight-hour absolute lifetime and a 20-minute idle timeout.

Roles and role-permission links are seeded in the identity migration; a forward migration adds stable role IDs, dynamic custom-role metadata, and an explicit assignable permission catalog. Permission checks query active database assignments on each protected operation. Current guards include `/api/v1/admin/overview` (`admin.overview.read`), `/api/v1/admin/audit-logs` (`audit_logs.read`), account reads (`accounts.read`), account enforcement (`accounts.suspend`), account session revocation (`accounts.sessions.revoke`), community metadata (`communities.read`), community enforcement (`communities.manage`), role operations (`admin.roles.*`), and permission catalog/assignment (`admin.permissions.*`). Role and permission mutations are transactional and audited. An actor may delegate only permissions already in their own current effective set; inactive roles stop authorizing immediately. The original seeded identity, its Super Admin assignment, and the reserved Super Admin role are protected through app checks and database triggers. Audit updates/deletes are rejected by a trigger under ordinary database access, but privileged database owners can bypass these protections; logs are not claimed to be tamper-proof.

Administrator auth, MFA enrollment/verification and controlled reset, recovery-code regeneration, self-service password change and recovery, invitation issue/accept, custom role and permission management, scoped role delegation, root protection, current-session lookup/logout, own-session listing/revocation, recent-authenticated cross-administrator session revocation, permission-gated overview, audit-log reads, account search/details and enforcement, and community metadata/suspension/restoration are implemented. Super Admins can invite additional Super Admins; ordinary invitation and role delegation stays within the inviter's effective permissions. Invitation acceptance creates an identity and role in one transaction but does not create a session; MFA enrollment remains mandatory at first sign-in. Administrator suspension, reinstatement, and logical deletion require recent MFA authentication; lifecycle changes revoke sessions/challenges and pending invitations, while deletion redacts credentials, revokes role assignments with history, and preserves audit references. A separate authorized administrator can reset another active administrator's MFA after recent reauthentication; this clears the TOTP secret, revokes sessions/challenges and unused recovery codes, and requires password login plus fresh TOTP enrollment before an administrative session is issued. The original seeded root status is blocked by both application checks and a database trigger. Account suspension and session/token revocation are atomic; reinstatement does not restore old sessions. Account owners with communities cannot be suspended until ownership is resolved, preserving the owner integrity constraint. Private community API reads return metadata only, never descriptions, rules, posts, or member identities. Recovery-code regeneration requires a recent session and a current TOTP step; previous unused codes are revoked and replacements are returned once. The overview currently returns only account, institution, and community row counts without date/country/academic/category filters. Granular account restrictions, temporary suspension expiry, community ownership transfer, and other Phase Three workflows remain unimplemented. The admin desktop shell is not an authorization boundary and remains disconnected.

## Scaling and ownership

The API remains one deployable Rust modular monolith. A domain may call another domain's public service or interface, never another domain's repository or tables. Keep writes and transactions inside their owning repository/domain. PostgreSQL is the current source of truth. Add a queue, cache, replica, or separate service only when an actual workflow or operational evidence requires it.

## Endpoint reference

Every registered endpoint, with its request and response contracts, authentication, cookies, and errors, is documented in [endpoints.md](endpoints.md). Update that reference alongside route, DTO, or error changes.

The product and platform roadmap is in [master-engineering-specification.md](master-engineering-specification.md). Its opening codebase-alignment section defines how roadmap items fit the current API structure and implementation phases.

## Not implemented yet

Administrator invitation issue/accept, administrator lifecycle management and controlled MFA reset, recovery-code regeneration, self-service password change and recovery, cross-administrator session revocation, account search/detail/suspend/reinstate, community metadata/suspend/restore, role assignment/delegation management, cookie-session origin checks, configured CORS, initial authentication rate limits, and configurable request timeouts are implemented. Overview filters, account restrictions/temporary suspensions, community ownership transfer, reporting/moderation, appeals, student/institution verification, directory administration, country policy management, privacy requests, social/MFA/network-level rate limits, outbox processing, generated OpenAPI, and SDK integrations remain unimplemented. OAuth providers require real client credentials and registered callback URLs. Email confirmation and recovery require SMTP configuration. The social completion API exists, but the frontend birthday page is not in this backend repository. Country policy values are unresolved product-owner decisions. Detailed age-assurance integrations, eligibility-change audit workflow, user linking UI, account export/deletion, and full email-delivery retry infrastructure remain future work.


## Cross-platform and future boundaries

The student web, iOS, Android, student desktop, admin desktop, and widget projects are separate scaffolds under `apps/` and `widgets/`. They do not own authorization or persistence. The admin Tauri project has separate identity/configuration from the student app but is not connected to the implemented backend admin APIs. The design freeze remains in effect.

There is no shared Rust SDK/UniFFI, OpenAPI generator, realtime transport, AI/recommendation service, or outbox worker. Do not create service boundaries or connect optional Redis/search/object-storage containers until a concrete implemented workflow requires them. Current auth, directory, profile and community capabilities and known omissions are summarized in [status.md](status.md).

## Current route authorization map

| Operation | Current authorization |
| --- | --- |
| Country, academic-level, and subject directory reads | Public |
| Create community | Authenticated account |
| Read active public/restricted community | Public |
| Read private community | Active member only; otherwise not found |
| Join open community / request restricted membership | Authenticated account; private join requires existing active/invited state |
| Read/update own profile and preferences | Authenticated owner |
| Read public profile | Public projection; private profiles return not found and academic affiliation is opt-in |
| Admin login/MFA enrollment | Separate admin password and limited five-minute MFA challenge |
| Admin current identity/logout | Active admin session after successful TOTP |
| Admin password change | Active admin session plus current password; changes credential and revokes all sessions and pending MFA challenges |
| Admin own sessions | Active admin session; current-session revocation is limited to the same administrator |
| Admin session revocation for another identity | Active admin session, `admin.accounts.manage`, and authentication within 15 minutes; target state is locked and audited transactionally |
| Admin recovery-code replacement | Active admin session created within 15 minutes and a current unused TOTP step; returns replacement codes once |
| Admin overview | Active admin session and current `admin.overview.read` permission |
| Admin audit-log read | Active admin session and current `audit_logs.read` permission |
| Account search/detail read | Active admin session and current `accounts.read` permission |
| Account suspension/reinstatement | Active admin session and current `accounts.suspend` permission |
| Community metadata read | Active admin session and current `communities.read` permission; private content excluded |
| Community suspension/restoration | Active admin session and current `communities.manage` permission |
| Granular account restrictions, community ownership transfer, content moderation, verification evidence | No registered endpoints |

Community owners/moderators do not have platform-wide privileges; those role operations are not implemented. Student verification status is not an administrative role. Private verification evidence does not exist in the current API.
